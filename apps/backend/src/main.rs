use std::net::SocketAddr;
use tracing::{info, error, warn};

// Import from our library
use epsx::{
    config::env::init_config,
    infrastructure::container::{StatelessServiceFactory, StatelessConfig},
    create_stateless_router,
};

/// Main server entry point - Serverless Stateless Architecture
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // Initialize configuration (loads .env and validates)
    let config = init_config();
    
    // Initialize basic tracing
    tracing_subscriber::fmt::init();
    
    info!("🚀 Starting EPSX Backend Server with STATELESS SERVERLESS architecture...");
    
    // Create stateless configuration from environment
    let stateless_config = StatelessConfig::from_env()
        .map_err(|e| format!("Failed to create stateless config: {}", e))?;
    
    // Create stateless service factory (no shared state)
    let service_factory = StatelessServiceFactory::new(stateless_config);
    info!("✅ Stateless service factory initialized");
    
    // Test service creation (health check)
    match service_factory.create_health_services().await {
        Ok(health_services) => {
            if health_services.health_check().await {
                info!("✅ Database connectivity verified");
            } else {
                warn!("⚠️ Database health check failed");
            }
        }
        Err(e) => {
            error!("❌ Failed to create health services: {}", e);
            return Err(e.into());
        }
    }
    
    // Create router with stateless service factory
    let app = create_stateless_router(service_factory).await;
    info!("✅ Stateless router created successfully");
    
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
    info!("📊 Analytics endpoints available at: http://{}:{}/api/v1/analytics/*", host, port);
    info!("⚡ SERVERLESS MODE: Services created per request (no shared state)");
    
    // Start the server
    let listener = tokio::net::TcpListener::bind(addr).await?;
    
    info!("✨ EPSX Backend Server is ready and listening in STATELESS mode!");
    
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