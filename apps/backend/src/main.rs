use std::{sync::Arc, net::SocketAddr};
use tracing::info;
use epsx::{infra::AppContainer, web::create_router};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::fmt::init();
    
    // Load environment variables
    dotenv::dotenv().ok();
    
    info!("Starting EPSX backend server with clean architecture...");
    
    // Initialize dependency container
    let container = Arc::new(AppContainer::new().await?);
    info!("Dependency container initialized");
    
    // Create application router
    let app = create_router(container);
    
    // Determine server address
    let host = std::env::var("HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
    let port = std::env::var("PORT")
        .unwrap_or_else(|_| "8080".to_string())
        .parse::<u16>()
        .unwrap_or(8080);
    
    let addr = SocketAddr::new(host.parse()?, port);
    info!("Server starting on {}", addr);
    
    // Start the server
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app.into_make_service()).await?;
    
    Ok(())
}