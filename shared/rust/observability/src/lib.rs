use std::net::SocketAddr;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ObservabilityError {
    #[error("Prometheus error: {0}")]
    Prometheus(String),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

pub type Result<T> = std::result::Result<T, ObservabilityError>;

pub struct Observability;

impl Observability {
    pub fn init(service_name: &str) {
        tracing_subscriber::fmt()
            .with_env_filter(
                tracing_subscriber::EnvFilter::try_from_default_env()
                    .unwrap_or_else(|_| format!("{}=debug,tower_http=debug", service_name).into()),
            )
            .with_target(true)
            .with_thread_ids(true)
            .init();

        tracing::info!("Observability initialized for {}", service_name);
    }

    pub fn init_prometheus(port: u16) -> Result<()> {
        let addr: SocketAddr = ([0, 0, 0, 0], port).into();
        let builder = metrics_exporter_prometheus::PrometheusBuilder::new();
        builder
            .with_http_listener(addr)
            .install()
            .map_err(|e| ObservabilityError::Prometheus(e.to_string()))?;
        tracing::info!("Prometheus metrics on {}", addr);
        Ok(())
    }
}

pub fn increment_counter(name: &'static str) {
    metrics::counter!(name).increment(1);
}

pub fn record_gauge(name: &'static str, value: f64) {
    metrics::gauge!(name).set(value);
}

pub fn record_histogram(name: &'static str, value: f64) {
    metrics::histogram!(name).record(value);
}
