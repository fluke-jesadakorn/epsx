use std::{net::SocketAddr, sync::Arc};
use tracing::{info, error};

// Import from our library
use epsx::{
    AppContainer,
    create_router,
};

/// Main server entry point
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize basic tracing
    tracing_subscriber::fmt::init();
    
    info!("🥞 Starting EPSX Backend Server...");
    
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
    let app = create_router(container.clone()).await;
    
    // Server configuration with environment variable support
    let host = std::env::var("HOST").unwrap_or_else(|_| "0.0.0.0".to_string());
    let port: u16 = std::env::var("PORT")
        .unwrap_or_else(|_| "8080".to_string())
        .parse()
        .unwrap_or(8080);
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