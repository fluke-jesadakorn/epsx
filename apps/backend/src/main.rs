mod auth;
mod config;

use crate::{auth::AuthService, config::Config};
use std::net::SocketAddr;
use axum::{routing::get, Router};
use tower_http::trace::TraceLayer;
use tracing::{info, error, Level};
use tracing_subscriber::{self, EnvFilter};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize the logger
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::from_default_env()
                .add_directive(Level::INFO.into())
        )
        .init();

    info!("Starting API server...");

    // Load configuration
    let config = Config::from_env()?;

    // Get address to bind to
    let addr = SocketAddr::from(([0, 0, 0, 0], config.port));

    // Bind to the address
    let listener = tokio::net::TcpListener::bind(&addr).await.map_err(|e| {
        error!("Failed to bind to address {}: {}", addr, e);
        e
    })?;

    info!("Server listening on {}", addr);

    // Initialize auth service
    let auth_service = AuthService::new(config.clone())
        .expect("Failed to initialize auth service");

    // Create router with API v1 routes
    let app = Router::new()
        .route("/", get(|| async { "API is running" }))
        .nest("/api/v1", Router::new()
            .nest("/auth", auth::auth_router(auth_service))
        )
        .layer(TraceLayer::new_for_http());

    // Start the server with graceful shutdown
    info!("Server initialized successfully, starting to serve requests");
    axum::serve(listener, app)
        .await
        .map_err(|e| {
            error!("Server error: {}", e);
            Box::new(e)
        })?;

    Ok(())
}
