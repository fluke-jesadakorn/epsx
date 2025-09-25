use std::{net::SocketAddr, sync::Arc};
use tracing::{info, error};

// Import from our library
use epsx::{
    DomainContainer,
    create_router,
    config::env::init_config,
    infrastructure::cache,
};

/// Main server entry point
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // Initialize configuration (loads .env and validates)
    let config = init_config();
    
    // Initialize basic tracing
    tracing_subscriber::fmt::init();
    
    info!("🥞 Starting EPSX Backend Server with unified environment configuration...");
    
    // Create database connection pool
    let database_url = std::env::var("DATABASE_URL")
        .map_err(|_| "DATABASE_URL environment variable is required")?;
    let db_pool = sqlx::PgPool::connect(&database_url).await
        .map_err(|e| format!("Failed to create database connection pool: {}", e))?;
    let db_pool = Arc::new(db_pool);
    
    // Create cache
    let cache_impl = cache::CacheFactory::with_fallback().await;
    let cache: Arc<dyn cache::Cache> = Arc::from(cache_impl);
    
    // Create unified domain container
    let container = Arc::new(DomainContainer::with_cache(db_pool, cache));
    info!("✅ Unified domain container initialized");
    
    // Create router with all routes
    let app = create_router(container.clone());
    info!("✅ Router created successfully");
    
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
    info!("🌐 Health check available at: http://{}:{}/health", host, port);
    info!("🔐 Web3 auth endpoints available at: http://{}:{}/api/auth/web3/*", host, port);
    info!("🏢 Enterprise API available at: http://{}:{}/api/v1/enterprise/*", host, port);
    info!("📊 Analytics endpoints available at: http://{}:{}/api/v1/analytics/*", host, port);
    
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