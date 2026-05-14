use std::net::{IpAddr, SocketAddr};

use epsx::bootstrap::{build_runtime, BackendBootstrapOptions};
use tracing::{error, info};

/// Main server entry point - Unified Router Architecture
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let runtime = build_runtime(BackendBootstrapOptions::stateful_server()).await?;

    info!("Starting EPSX Backend Server - Data Analytics Platform...");
    info!("Unified router created successfully");

    let host = std::env::var("HOST").unwrap_or_else(|_| "0.0.0.0".to_string());
    let port: u16 = std::env::var("PORT")
        .unwrap_or_else(|_| "8080".to_string())
        .parse()
        .unwrap_or(8080);
    let host_ip: IpAddr = host.parse().unwrap_or_else(|_| IpAddr::from([0, 0, 0, 0]));

    info!("Backend URL: {}", runtime.config.backend_url);
    info!("Frontend URL: {}", runtime.config.frontend_url);
    info!("Admin URL: {}", runtime.config.admin_frontend_url);

    let addr = SocketAddr::new(host_ip, port);

    info!("Server starting on {}:{}", host, port);
    info!("Health check: http://{}:{}/health", host, port);
    info!("");
    info!("UNIFIED API ENDPOINTS:");
    info!("   Auth:      http://{}:{}/api/auth/web3/*", host, port);
    info!("   Analytics: http://{}:{}/api/analytics/*", host, port);
    info!("   Public:    http://{}:{}/api/public/*", host, port);
    info!(
        "   Admin:     http://{}:{}/admin/* | http://{}:{}/api/admin/*",
        host, port, host, port
    );
    info!("   Docs:      http://{}:{}/docs", host, port);
    info!("");

    let listener = tokio::net::TcpListener::bind(addr).await?;

    info!("EPSX Backend Server is ready and listening!");

    match axum::serve(listener, runtime.router).await {
        Ok(_) => {
            info!("Server shutdown gracefully");
            Ok(())
        }
        Err(e) => {
            error!("Server error: {}", e);
            Err(e.into())
        }
    }
}
