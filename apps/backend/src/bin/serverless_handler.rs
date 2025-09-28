// Generic Serverless Handler
// Platform-agnostic entry point for any serverless deployment
// Uses stateless architecture with per-request service instantiation

use anyhow::Result;
use epsx::infrastructure::container::StatelessServiceFactory;
use epsx::web::stateless_router::create_stateless_router;
use std::env;
use tracing::{error, info};

/// Generic serverless entry point
/// Works with any containerized serverless platform with stateless architecture
#[tokio::main]
async fn main() -> Result<()> {
    // Initialize generic logging
    init_serverless_logging().await?;

    info!("🚀 Starting EPSX Serverless Handler...");

    // Validate serverless environment
    validate_serverless_env()?;

    // Pre-warm services for faster cold starts
    warm_up_services().await?;

    // Create stateless service factory from environment
    let stateless_config = epsx::infrastructure::container::StatelessConfig::from_env()
        .map_err(|e| anyhow::anyhow!("Failed to create stateless config: {}", e))?;

    let service_factory = StatelessServiceFactory::new(stateless_config);

    // Create stateless router
    let app = create_stateless_router(service_factory).await;

    // Use PORT environment variable (standard for most platforms)
    let port = env::var("PORT")
        .unwrap_or_else(|_| "8080".to_string())
        .parse::<u16>()
        .map_err(|e| anyhow::anyhow!("Invalid PORT environment variable: {}", e))?;

    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", port))
        .await
        .map_err(|e| anyhow::anyhow!("Failed to bind to port {}: {}", port, e))?;

    info!("🌐 Serverless handler listening on http://0.0.0.0:{}", port);
    info!("📊 Health check: http://0.0.0.0:{}/health", port);
    info!("🔐 Web3 auth: http://0.0.0.0:{}/api/auth/web3/*", port);
    info!("📈 Analytics: http://0.0.0.0:{}/api/v1/analytics/*", port);
    info!("⚡ SERVERLESS MODE: Per-request service instantiation");
    info!("🔄 STATELESS: No shared state between requests");

    // Start the server
    axum::serve(listener, app)
        .await
        .map_err(|e| anyhow::anyhow!("Serverless server error: {}", e))?;

    Ok(())
}

/// Initialize generic serverless logging
async fn init_serverless_logging() -> Result<()> {
    // Generic serverless logging configuration
    let log_level = env::var("RUST_LOG").unwrap_or_else(|_| {
        // Default to info level for production serverless
        "info".to_string()
    });

    env::set_var("RUST_LOG", log_level);

    // Initialize tracing subscriber optimized for serverless
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .with_target(false)    // Minimal info for serverless
        .with_thread_ids(false) // Not needed in serverless
        .with_file(false)      // Not needed in serverless
        .with_line_number(false) // Not needed in serverless
        .init();

    info!("✅ Serverless logging initialized");
    Ok(())
}

/// Generic serverless environment validation
fn validate_serverless_env() -> Result<()> {
    let required_vars = [
        "DATABASE_URL",
        "REDIS_URL", 
        "FRONTEND_URL",
        "BACKEND_URL",
    ];

    let mut missing_vars = Vec::new();

    for var in &required_vars {
        if env::var(var).is_err() {
            missing_vars.push(*var);
        }
    }

    if !missing_vars.is_empty() {
        let error_msg = format!(
            "Missing required environment variables for serverless deployment: {}",
            missing_vars.join(", ")
        );
        error!("{}", error_msg);
        return Err(anyhow::anyhow!(error_msg));
    }

    info!("✅ Serverless environment validation passed");
    Ok(())
}

/// Cold start optimization for serverless platforms
async fn warm_up_services() -> Result<()> {
    info!("🔥 Warming up serverless services...");

    // Pre-initialize database connection pool for faster requests
    match epsx::infrastructure::database::get_db_pool().await {
        Ok(_) => info!("✅ Database pool warmed up"),
        Err(e) => {
            error!("❌ Database warm-up failed: {}", e);
            return Err(anyhow::anyhow!("Database warm-up failed: {}", e));
        }
    }

    // Pre-initialize Redis cache if available
    if env::var("REDIS_URL").is_ok() {
        match epsx::infrastructure::cache::serverless_cache_health_check().await {
            true => info!("✅ Redis cache warmed up"),
            false => {
                error!("❌ Redis cache warm-up failed");
                return Err(anyhow::anyhow!("Redis cache warm-up failed"));
            }
        }
    }

    info!("🚀 Serverless services warm-up complete");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_serverless_env_validation() {
        // Test serverless environment validation logic
        std::env::set_var("DATABASE_URL", "postgresql://test");
        std::env::set_var("REDIS_URL", "redis://test");
        std::env::set_var("FRONTEND_URL", "http://test");
        std::env::set_var("BACKEND_URL", "http://test");

        let result = validate_serverless_env();
        assert!(result.is_ok());
    }

    #[test]
    fn test_port_parsing() {
        // Test port parsing for serverless platforms
        std::env::set_var("PORT", "8080");
        let port = std::env::var("PORT")
            .unwrap_or_else(|_| "8080".to_string())
            .parse::<u16>()
            .unwrap();
        assert_eq!(port, 8080);
    }

    #[test]
    fn test_rust_log_default() {
        // Test RUST_LOG environment variable handling
        std::env::remove_var("RUST_LOG");
        let log_level = std::env::var("RUST_LOG").unwrap_or_else(|_| "info".to_string());
        assert_eq!(log_level, "info");
    }
}