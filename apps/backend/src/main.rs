use std::{ sync::Arc, net::SocketAddr, time::Duration };
use tracing::{ info, warn };
use epsx::{
  infra::{ AppContainer },
  web::{create_router, create_demo_router},
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
  // Initialize tracing
  tracing_subscriber::fmt::init();

  // Load environment variables
  dotenv::dotenv().ok();

  info!("Starting EPSX backend server with clean architecture...");

  // Try to create full router with database, fallback to demo mode if needed
  let app = match AppContainer::new().await {
    Ok(container) => {
      info!("✅ Full application container created successfully");
      create_router(Arc::new(container)).await
    }
    Err(e) => {
      warn!("Failed to create container: {}, falling back to demo mode", e);
      create_demo_router().await
    }
  };
  
  // Determine server address - Cloud Run requires 0.0.0.0
  let host = std::env::var("HOST").unwrap_or_else(|_| "0.0.0.0".to_string());
  let port = std::env::var("PORT").unwrap_or_else(|_| "8080".to_string()).parse::<u16>().unwrap_or(8080);

  let addr = SocketAddr::new(host.parse()?, port);
  info!("Server starting on {}", addr);

  // Start the server
  let listener = tokio::net::TcpListener::bind(&addr).await?;

  // Setup graceful shutdown
  let (shutdown_tx, shutdown_rx) = tokio::sync::oneshot::channel();

  // Handle shutdown signals
  tokio::spawn(async move {
    let ctrl_c = tokio::signal::ctrl_c();

    #[cfg(unix)]
    let terminate = async {
      tokio::signal::unix
        ::signal(tokio::signal::unix::SignalKind::terminate())
        .expect("failed to install signal handler")
        .recv().await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
            _ = ctrl_c => {
                info!("Received SIGINT, shutting down gracefully...");
            }
            _ = terminate => {
                info!("Received SIGTERM, shutting down gracefully...");
            }
        }

    let _ = shutdown_tx.send(());
  });

  let graceful = axum
    ::serve(listener, app.into_make_service())
    .with_graceful_shutdown(async move {
      shutdown_rx.await.ok();
      info!("Starting graceful shutdown sequence...");

      // Set a timeout for the entire shutdown process
      let shutdown_timeout = Duration::from_secs(10);
      let shutdown_start = std::time::Instant::now();


      let elapsed = shutdown_start.elapsed();
      if elapsed > shutdown_timeout {
        warn!(
          "Shutdown process took {}ms (timeout: {}ms)",
          elapsed.as_millis(),
          shutdown_timeout.as_millis()
        );
      } else {
        info!("Graceful shutdown completed in {}ms", elapsed.as_millis());
      }
    });

  graceful.await?;

  Ok(())
}
