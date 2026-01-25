use std::net::SocketAddr;
use std::sync::Arc;
use epsx::prelude::{TlsPool, TlsConnectionManager};
use tracing::{info, error};

// Import from our library
use epsx::{
    config::env::init_config,
    infrastructure::container::DomainContainer,
    create_router,
};

/// Main server entry point - Unified Router Architecture
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // Initialize configuration (loads .env and validates)
    let config = init_config();

    // Initialize tracing with level from configuration
    // Filter out noisy tokio_postgres and rustls DEBUG logs by default
    // Users can still enable them via RUST_LOG=tokio_postgres=debug,rustls=debug if needed
    let filter = tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| {
            // Default: use LOG_LEVEL config + suppress noisy DEBUG logs
            // This prevents verbose query preparation/execution logs from tokio_postgres
            // and TLS handshake details from rustls
            let filter_str = format!("{},tokio_postgres=warn,rustls=warn", config.log_level);
            tracing_subscriber::EnvFilter::new(&filter_str)
        });
    
    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .init();

    // Install default crypto provider for rustls
    let _ = rustls::crypto::ring::default_provider().install_default();

    info!("🚀 Starting EPSX Backend Server - Data Analytics Platform...");

    // Create database pool with Diesel
    let database_url = std::env::var("DATABASE_URL")
        .map_err(|_| "DATABASE_URL must be set")?;

    info!("Connecting to database...");
    let db_config = TlsConnectionManager::new(database_url);
    let pool = TlsPool::builder(db_config)
        .max_size(10)
        .runtime(deadpool::Runtime::Tokio1)
        .build()
        .map_err(|e| format!("Failed to create database pool: {}", e))?;

    // Test database connection with timeout
    let connection_timeout = std::time::Duration::from_secs(10);
    match tokio::time::timeout(connection_timeout, pool.get()).await {
        Ok(Ok(_)) => {
            info!("✅ Database pool created and connection verified")
        },
        Ok(Err(e)) => {
            error!("❌ Failed to connect to database: {}", e);
            return Err(format!("Database connection failed: {}", e).into());
        },
        Err(_) => {
            error!("❌ Database connection check timed out after 10s");
            return Err("Database connection timed out".into());
        }
    }

    // Leak the pool to make it 'static (required for container)
    let _db_pool: &'static TlsPool = Box::leak(Box::new(pool));

    // Create cache (optional)
    let redis_timeout = std::time::Duration::from_secs(5);
    let cache = match std::env::var("REDIS_URL").ok() {
        Some(redis_url) => {
            match tokio::time::timeout(redis_timeout, epsx::infrastructure::cache::redis_cache::RedisCache::new(
                redis_url,
                10, // pool_size
                epsx::infrastructure::cache::CacheConfig::default()
            )).await {
                Ok(Ok(cache)) => {
                    info!("✅ Redis cache initialized");
                    Some(Arc::new(cache) as Arc<dyn epsx::infrastructure::cache::Cache>)
                }
                Ok(Err(e)) => {
                    info!("⚠️ Redis cache initialization failed, using memory cache: {}", e);
                    Some(Arc::new(epsx::infrastructure::cache::memory_cache::MemoryCache::new())
                        as Arc<dyn epsx::infrastructure::cache::Cache>)
                }
                Err(_) => {
                    info!("⚠️ Redis connection timed out after 5s, using memory cache");
                    Some(Arc::new(epsx::infrastructure::cache::memory_cache::MemoryCache::new())
                        as Arc<dyn epsx::infrastructure::cache::Cache>)
                }
            }
        }
        None => {
            info!("ℹ️ No Redis URL configured, using memory cache");
            Some(Arc::new(epsx::infrastructure::cache::memory_cache::MemoryCache::new())
                as Arc<dyn epsx::infrastructure::cache::Cache>)
        }
    };

    // Create domain container with Web3 services
    let container = Arc::new(DomainContainer::new_with_web3_services(
        cache,
        None, // blockchain_config - will use defaults
    ).await);
    info!("✅ Domain container initialized with Web3 services and Redis notifications");

    // Start Transaction Monitor Service (Background task for verifying payments)
    epsx::infrastructure::blockchain::spawn_transaction_monitor();
    info!("✅ Transaction Monitor background service started");

    // Start EventDispatcher (background worker for publishing events to Redis)
    if let Some(dispatcher) = &container.event_dispatcher {
        match dispatcher.clone().start().await {
            Ok(_) => info!("✅ EventDispatcher started - events will be published to Redis Streams"),
            Err(e) => info!("⚠️ EventDispatcher failed to start: {} (continuing without event publishing)", e),
        }
    } else {
        info!("ℹ️ EventDispatcher not configured (Redis URL not set)");
    }

    // Start ProjectionManager (background worker for updating read models)
    if let Some(projection_manager) = &container.projection_manager {
        match projection_manager.clone().start().await {
            Ok(_) => info!("✅ ProjectionManager started - read models will be updated from events"),
            Err(e) => info!("⚠️ ProjectionManager failed to start: {} (continuing without projections)", e),
        }
    } else {
        info!("ℹ️ ProjectionManager not configured");
    }

    // Create unified router
    let app = create_router(container);
    info!("✅ Unified router created successfully");

    // Server configuration using unified config
    let host = std::env::var("HOST").unwrap_or_else(|_| "0.0.0.0".to_string());
    let port: u16 = std::env::var("PORT")
        .unwrap_or_else(|_| "8080".to_string())
        .parse()
        .unwrap_or(8080);

    info!("🔗 Backend URL: {}", config.backend_url);
    info!("🌐 Frontend URL: {}", config.frontend_url);
    info!("⚙️  Admin URL: {}", config.admin_frontend_url);
    let addr = SocketAddr::from(([0, 0, 0, 0], port));

    info!("🚀 Server starting on {}:{}", host, port);
    info!("🌐 Health check: http://{}:{}/health", host, port);
    info!("");
    info!("📡 UNIFIED API ENDPOINTS:");
    info!("   🔐 Auth:      http://{}:{}/api/auth/web3/*", host, port);
    info!("   📊 Analytics: http://{}:{}/api/analytics/*", host, port);
    info!("   📊 Public:    http://{}:{}/api/public/*", host, port);
    info!("   👤 Admin:     http://{}:{}/admin/* | http://{}:{}/api/admin/*", host, port, host, port);
    info!("   📖 Docs:      http://{}:{}/docs", host, port);
    info!("");

    // Start the server
    let listener = tokio::net::TcpListener::bind(addr).await?;

    info!("✨ EPSX Backend Server is ready and listening!");
    
    match axum::serve(listener, app).await {
        Ok(_) => {
            info!("🛑 Server shutdown gracefully");
            Ok(())
        }
        Err(e) => {
            error!("❌ Server error: {}", e);
            Err(e.into())
        }
    }
}