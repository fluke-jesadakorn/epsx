use std::{ sync::Arc, net::SocketAddr, time::Duration };
use tracing::{ info, warn, error };
use epsx::{
  infra::{ AppContainer, PostgresAuditRepo },
  web::create_router,
  dom::services::feature_expiration::{ ExpirationScheduler, ExpirationConfig },
  infra::jobs::{
    JobScheduler,
    ExpirationChecker,
    NotificationService as JobNotificationService,
    SimpleEmailProvider,
    NotificationConfig,
  },
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

  // Initialize feature expiration scheduler
  let mut expiration_scheduler = ExpirationScheduler::new(
    container.feature_expiration_service.clone(),
    Some(ExpirationConfig {
      warning_days_before: vec![30, 7, 3, 1],
      grace_period_days: 7,
      check_interval_hours: 1, // Check every hour
      batch_size: 100,
    })
  );

  expiration_scheduler.start().await?;
  info!("Feature expiration scheduler started");

  // Initialize job scheduler system
  let notification_config = NotificationConfig::default();
  let email_provider = Box::new(SimpleEmailProvider::new(true)); // Simulate emails in development
  let job_notification_service = Arc::new(
    JobNotificationService::new(
      email_provider,
      Box::new(
        PostgresAuditRepo::new((*container.infra.postgres_pool).clone())
      ),
      notification_config
    )
  );

  let expiration_checker = Arc::new(
    ExpirationChecker::new(
      container.permission_profile_repo.clone(),
      container.user_repo.clone(),
      container.audit_repo.clone(),
      job_notification_service.clone()
    )
  );

  let assignment_repo = Arc::new(
    epsx::infra::db::postgres::assign_repo::PostgresPermissionAssignmentRepo::new(
      (*container.infra.postgres_pool).clone()
    )
  );
  let auto_assignment_service = Arc::new(
    epsx::dom::services::auto_assignment::AutoAssignmentEngine::new(
      container.permission_profile_repo.clone(),
      assignment_repo,
      container.user_repo.clone()
    )
  );

  let mut job_scheduler = JobScheduler::new(
    container.permission_profile_repo.clone(),
    container.user_repo.clone(),
    container.audit_repo.clone(),
    auto_assignment_service,
    expiration_checker,
    job_notification_service
  ).await?;

  job_scheduler.start().await?;
  info!("Background job scheduler started");

  // Create application router
  let app = create_router(container).await;

  // Determine server address
  let host = std::env::var("HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
  let port = std::env
    ::var("PORT")
    .unwrap_or_else(|_| "8080".to_string())
    .parse::<u16>()
    .unwrap_or(8080);

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

      // Shutdown schedulers with timeout
      info!("Shutting down schedulers...");

      // Shutdown expiration scheduler
      expiration_scheduler.shutdown();
      info!("Expiration scheduler shutdown initiated");

      // Shutdown job scheduler with timeout
      let job_shutdown = tokio::time::timeout(
        Duration::from_secs(5),
        job_scheduler.stop()
      ).await;
      match job_shutdown {
        Ok(Ok(())) => info!("Job scheduler stopped successfully"),
        Ok(Err(e)) => error!("Error stopping job scheduler: {}", e),
        Err(_) => warn!("Job scheduler shutdown timed out after 5 seconds"),
      }

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
