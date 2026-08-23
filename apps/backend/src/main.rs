use epsx::prelude::{TlsConnectionManager, TlsPool};
use std::net::{IpAddr, SocketAddr};
use std::sync::Arc;
use tracing::{error, info};

// Import from our library
use epsx::{config::env::init_config, create_router, infrastructure::container::DomainContainer};

/// Main server entry point - Unified Router Architecture
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // Initialize configuration (loads .env and validates)
    let config = init_config();

    // Initialize tracing with level from configuration
    // Uses the unified logging infrastructure
    epsx::infrastructure::logger::init_logger(config.is_production(), &config.log_level);

    // Install default crypto provider for rustls
    let _ = rustls::crypto::ring::default_provider().install_default();

    info!("Starting EPSX Backend Server - Data Analytics Platform...");

    // Create database pool with Diesel (BIG-BANG: keep until last Diesel query gone, sqlx side-by-side)
    let database_url = std::env::var("DATABASE_URL").map_err(|_| "DATABASE_URL must be set")?;
    let database_url_for_sqlx = database_url.clone();

    info!("Connecting to database...");
    let db_config = TlsConnectionManager::new(database_url);
    let pool = TlsPool::builder(db_config)
        .max_size(10)
        .runtime(deadpool::Runtime::Tokio1)
        .build()
        .map_err(|e| format!("Failed to create database pool: {}", e))?;

    // Test database connection with timeout
    let connection_timeout = std::time::Duration::from_secs(10);
    match tokio::time::timeout(connection_timeout, pool.acquire().await).await {
        Ok(Ok(_)) => {
            info!("Database pool created and connection verified")
        }
        Ok(Err(e)) => {
            error!("Failed to connect to database: {}", e);
            return Err(format!("Database connection failed: {}", e).into());
        }
        Err(_) => {
            error!("Database connection check timed out after 10s");
            return Err("Database connection timed out".into());
        }
    }

    // BIG-BANG: Arc replaces Box::leak for graceful shutdown/rotation.
    // Keep 'static leak as fallback until all containers use Arc<PgPool>.
    let _db_pool: &'static TlsPool = Box::leak(Box::new(pool));
    // TODO(bigbang): replace with `let db_pool = Arc::new(pool)` and thread Arc through DomainContainer
    let _ = &_db_pool; // suppress unused until migrated

    // BIG-BANG Phase1: create canonical sqlx pool side-by-side (not yet wired to container)
    // This pool will replace TlsPool when the last Diesel query is removed.
    let _sqlx_pool = match sqlx::PgPool::connect(&database_url_for_sqlx).await {
        Ok(pool) => {
            info!("SQLx pool created (big-bang side-by-side)");
            Some(std::sync::Arc::new(pool))
        }
        Err(e) => {
            tracing::warn!("SQLx pool failed (will retry on next phase): {}", e);
            None
        }
    };
    let _ = &_sqlx_pool;

    // Seed system admin plans (idempotent)
    epsx::infrastructure::services::seed_system_admin_plans(_db_pool).await;

    // Seed production news (idempotent)
    epsx::infrastructure::services::seed_production_news(_db_pool).await;

    // Create cache (optional)
    let redis_timeout = std::time::Duration::from_secs(5);
    let cache = match std::env::var("REDIS_URL").ok() {
        Some(redis_url) => {
            match tokio::time::timeout(
                redis_timeout,
                epsx::infrastructure::cache::redis_cache::RedisCache::new(
                    redis_url,
                    10, // pool_size
                    epsx::infrastructure::cache::CacheConfig::default(),
                ),
            )
            .await
            {
                Ok(Ok(cache)) => {
                    info!("Redis cache initialized");
                    Some(Arc::new(cache) as Arc<dyn epsx::infrastructure::cache::Cache>)
                }
                Ok(Err(e)) => {
                    info!(
                        "Redis cache initialization failed, using memory cache: {}",
                        e
                    );
                    Some(
                        Arc::new(epsx::infrastructure::cache::memory_cache::MemoryCache::new())
                            as Arc<dyn epsx::infrastructure::cache::Cache>,
                    )
                }
                Err(_) => {
                    info!("Redis connection timed out after 5s, using memory cache");
                    Some(
                        Arc::new(epsx::infrastructure::cache::memory_cache::MemoryCache::new())
                            as Arc<dyn epsx::infrastructure::cache::Cache>,
                    )
                }
            }
        }
        None => {
            info!("No Redis URL configured, using memory cache");
            Some(
                Arc::new(epsx::infrastructure::cache::memory_cache::MemoryCache::new())
                    as Arc<dyn epsx::infrastructure::cache::Cache>,
            )
        }
    };

    // Create domain container with Web3 services
    let container = Arc::new(
        DomainContainer::new_with_web3_services(
            cache, None, // blockchain_config - will use defaults
        )
        .await,
    );
    info!("Domain container initialized with Web3 services and Redis notifications");

    // Start Transaction Monitor Service (Background task for verifying payments)
    epsx::infrastructure::blockchain::spawn_transaction_monitor();
    info!("Transaction Monitor background service started");

    // Start EventDispatcher (background worker for publishing events to Redis)
    if let Some(dispatcher) = &container.event_dispatcher {
        match dispatcher.clone().start().await {
            Ok(_) => info!("EventDispatcher started - events will be published to Redis Streams"),
            Err(e) => info!(
                "EventDispatcher failed to start: {} (continuing without event publishing)",
                e
            ),
        }
    } else {
        info!("EventDispatcher not configured (Redis URL not set)");
    }

    // Start ProjectionManager (background worker for updating read models)
    if let Some(projection_manager) = &container.projection_manager {
        match projection_manager.clone().start().await {
            Ok(_) => info!("ProjectionManager started - read models will be updated from events"),
            Err(e) => info!(
                "ProjectionManager failed to start: {} (continuing without projections)",
                e
            ),
        }
    } else {
        info!("ProjectionManager not configured");
    }

    // Wave 10 integration gate: build the in-process NotificationPort
    // (async constructor touches the notifications pool) and hand
    // it to the router so every AppState the router creates has
    // the port wired. Production startup is fail-closed if the adapter
    // cannot be built; otherwise publisher call sites could silently drop
    // notifications while the backend reports healthy.
    let notification_port: Option<Arc<dyn epsx_contracts::notification_port::NotificationPort>> =
        match epsx::infrastructure::adapters::notification::build_notification_port(
            container.pubsub.as_ref().map(Arc::clone),
        )
        .await
        {
            Ok(adapter) => {
                info!("NotificationPort wired (configured adapter)");
                Some(adapter)
            }
            Err(e) => {
                if epsx::infrastructure::adapters::notification::notification_adapter_required() {
                    return Err(format!("notification adapter is required at startup: {e}").into());
                }
                info!("NotificationPort not wired in non-production mode: {}", e);
                None
            }
        };

    // Start PlanExpirationService only after the shared notification port
    // has been constructed.  Starting it before adapter construction left
    // its optional port permanently unset, so expiry producers silently
    // skipped notification delivery even when the backend was healthy.
    {
        let svc = epsx::infrastructure::services::PlanExpirationService::new(
            Arc::clone(&container.db_pool),
            container.notifications_pool.as_ref().map(Arc::clone),
            container.pubsub.as_ref().map(Arc::clone),
        );
        let svc = if let Some(port) = notification_port.as_ref() {
            svc.with_notification_port(Arc::clone(port))
        } else {
            svc
        };
        svc.start();
        info!("PlanExpirationService background service started");
    }

    // Create unified router
    let app = create_router(container, notification_port).await;
    info!("Unified router created successfully");

    // Server configuration using unified config
    let host = std::env::var("HOST").unwrap_or_else(|_| "0.0.0.0".to_string());
    let port: u16 = std::env::var("PORT")
        .unwrap_or_else(|_| "8080".to_string())
        .parse()
        .unwrap_or(8080);
    let host_ip: IpAddr = host.parse().unwrap_or_else(|_| IpAddr::from([0, 0, 0, 0]));

    info!("Backend URL: {}", config.backend_url);
    info!("Frontend URL: {}", config.frontend_url);
    info!("Admin URL: {}", config.admin_frontend_url);
    let addr = SocketAddr::new(host_ip, port);

    info!("Server starting on {}:{}", host, port);
    info!("Health check: http://{}:{}/health", host, port);
    info!("");
    info!("UNIFIED API ENDPOINTS:");
    info!("   Auth:      http://{}:{}/api/auth/web3/*", host, port);
    info!("   Analytics: http://{}:{}/api/analytics/*", host, port);
    info!("   Public:    http://{}:{}/api/public/*", host, port);
    info!(
        "   Admin:     http://{}:{}/admin/* | http://{}:{}/api/admin/*",
        host, port, host, port
    );
    info!("   Docs:      http://{}:{}/docs", host, port);
    info!("");

    // Start the server
    let listener = tokio::net::TcpListener::bind(addr).await?;

    info!("EPSX Backend Server is ready and listening!");

    match axum::serve(listener, app).await {
        Ok(_) => {
            info!("Server shutdown gracefully");
            Ok(())
        }
        Err(e) => {
            error!("Server error: {}", e);
            Err(e.into())
        }
    }
}
