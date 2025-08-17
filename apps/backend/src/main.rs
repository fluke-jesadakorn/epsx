use std::{ sync::Arc, net::SocketAddr, time::Duration };
use tracing::{ info, warn };
use epsx::{
  infra::{ AppContainer },
  web::create_router,
  config::{ Config },
};

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

  // Job scheduler system removed - using user-driven validation instead

  // Create application router
  let app = create_router(container).await;

  // Load configuration
  let config = Config::from_env().expect("Failed to load configuration");
  
  // Determine server address
  let host = config.server.host;
  let port = config.server.port;

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

      // Scheduler shutdown removed - using user-driven validation instead

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
