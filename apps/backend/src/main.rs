mod app_state;
mod middleware;
mod models;
mod services;
mod config;
mod error;
mod routes;

use crate::{ app_state::AppState, config::Config, error::AppResult };
use std::net::SocketAddr;
use tracing_subscriber::{ layer::SubscriberExt, util::SubscriberInitExt };
use tokio::signal;

#[tokio::main]
async fn main() -> AppResult<()> {
    // Initialize logging
    tracing_subscriber
        ::registry()
        .with(
            tracing_subscriber::EnvFilter
                ::try_from_default_env()
                .unwrap_or_else(|_| "rust_backend=debug,tower_http=debug".into())
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Load configuration
    let config = Config::from_env()?;

    // Create application state
    let state = AppState::new(config.clone()).await?;

    // Create router with all routes
    let app = routes::create_router(state);

    // Get address to bind to
    let addr = SocketAddr::from(([0, 0, 0, 0], config.port));

    // Bind to the address
    let listener = match tokio::net::TcpListener::bind(&addr).await {
        Ok(listener) => {
            tracing::info!("Starting server on {}", addr);
            listener
        }
        Err(e) => {
            tracing::error!("Failed to bind to {}: {}", addr, e);
            return Err(error::AppError::Internal(format!("Failed to bind to {}: {}", addr, e)));
        }
    };

    // Start the server with graceful shutdown
    axum
        ::serve(listener, app.into_make_service())
        .with_graceful_shutdown(shutdown_signal()).await
        .map_err(|e| error::AppError::Internal(e.to_string()))?;

    tracing::info!("Server shutdown completed");
    Ok(())
}

async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c().await.expect("Failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix
            ::signal(signal::unix::SignalKind::terminate())
            .expect("Failed to install signal handler")
            .recv().await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }

    tracing::info!("Shutdown signal received, starting graceful shutdown");
}
