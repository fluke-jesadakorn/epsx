use std::net::SocketAddr;
use std::sync::Arc;
use diesel_async::AsyncPgConnection;
use diesel_async::pooled_connection::deadpool::Pool;
use diesel_async::pooled_connection::AsyncDieselConnectionManager;
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

    // Initialize basic tracing
    tracing_subscriber::fmt::init();

    info!("🚀 Starting EPSX Backend Server - Data Analytics Platform...");

    // Create database pool with Diesel
    let database_url = std::env::var("DATABASE_URL")
        .map_err(|_| "DATABASE_URL must be set")?;

    let db_config = AsyncDieselConnectionManager::<AsyncPgConnection>::new(&database_url);
    let pool = Pool::builder(db_config)
        .max_size(10)
        .build()
        .map_err(|e| format!("Failed to create database pool: {}", e))?;

    // Test database connection
    match pool.get().await {
        Ok(_) => info!("✅ Database pool created and connection verified"),
        Err(e) => {
            error!("❌ Failed to connect to database: {}", e);
            return Err(format!("Database connection failed: {}", e).into());
        }
    }

    // Leak the pool to make it 'static (required for container)
    let _db_pool: &'static Pool<AsyncPgConnection> = Box::leak(Box::new(pool));

    // Create cache (optional)
    let cache = match std::env::var("REDIS_URL").ok() {
        Some(redis_url) => {
            match epsx::infrastructure::cache::redis_cache::RedisCache::new(
                redis_url,
                10, // pool_size
                epsx::infrastructure::cache::CacheConfig::default()
            ).await {
                Ok(cache) => {
                    info!("✅ Redis cache initialized");
                    Some(Arc::new(cache) as Arc<dyn epsx::infrastructure::cache::Cache>)
                }
                Err(e) => {
                    info!("⚠️ Redis cache unavailable, using memory cache: {}", e);
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