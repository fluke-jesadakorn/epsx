use std::{net::SocketAddr, sync::Arc};
use tracing::{info, error};

// Import from our library
use epsx::{
    AppContainer,
    create_router,
    config::env::init_config,
};

/// Main server entry point
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // Initialize configuration (loads .env and validates)
    let config = init_config();
    
    // Initialize basic tracing
    tracing_subscriber::fmt::init();
    
    info!("🥞 Starting EPSX Backend Server with unified environment configuration...");
    
    // Create application container
    let container = match AppContainer::new().await {
        Ok(container) => {
            info!("✅ Application container initialized");
            Arc::new(container)
        }
        Err(e) => {
            error!("❌ Failed to initialize application container: {}", e);
            return Err(e.into());
        }
    };
    
    // Create router with all routes
    let app = match create_router(container.clone()).await {
        Ok(router) => {
            info!("✅ Router created successfully");
            router
        }
        Err(e) => {
            error!("❌ Failed to create router: {}", e);
            return Err(e);
        }
    };
    
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
    info!("🔐 OIDC endpoints available at: http://{}:{}/oauth/*", host, port);
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