use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};
use tracing::info;

/// Initialize the global logger instance for EPSX Backend
/// 
/// Provides unified configuration for all backend binaries (main, blockchain_monitor).
/// - **Production Mode**: Uses JSON structured logging over stdout. Strict filtering 
///   is applied to avoid verbose library spam (`tokio_postgres`, `rustls`, etc.).
/// - **Development Mode**: Uses standard console formatting with rich colors. Filtering
///   is mostly tied to the standard LOG_LEVEL.
pub fn init_logger(is_production: bool, default_log_level: &str) {
    // 1. Determine base filter from environment, or use default log level + strict overrides
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| {
        if is_production {
            // In production, we aggressively filter noisy libraries by default
            // unless the user specifically overrides them with RUST_LOG.
            let filter_str = format!(
                "{},tokio_postgres=warn,rustls=warn,hyper=error,h2=error,tower=warn,sqlx=warn,epsx=info",
                default_log_level
            );
            EnvFilter::new(&filter_str)
        } else {
            // In development, we can be slightly more permissive but still avoid 
            // the extreme noise of `tokio_postgres` DEBUG if global trace is set.
            let filter_str = format!(
                "{},tokio_postgres=warn,rustls=warn,hyper=warn,h2=warn,epsx=debug,blockchain_monitor=debug",
                default_log_level
            );
            EnvFilter::new(&filter_str)
        }
    });

    // 2. Build the subscriber based on the environment
    if is_production {
        // PRODUCTION: Use structured JSON logging for external ingestion (e.g. DataDog, ELK, CloudWatch).
        // It provides machine-readable parsed logs.
        let format_layer = tracing_subscriber::fmt::layer()
            .json()
            .with_current_span(true)
            .with_span_list(false);

        tracing_subscriber::registry()
            .with(filter)
            .with(format_layer)
            .init();
            
        info!("Logger initialized in PRODUCTION mode (JSON formatted)");
    } else {
        // DEVELOPMENT: Use standard, rich console output
        let format_layer = tracing_subscriber::fmt::layer()
            .compact()
            .with_ansi(true) // Enable colors
            .with_target(true); // Show the target module name

        tracing_subscriber::registry()
            .with(filter)
            .with(format_layer)
            .init();
            
        info!("Logger initialized in DEVELOPMENT mode (Compact human-readable)");
    }
}
