// Diesel Async Connection Manager for Serverless Environments
// Provides efficient connection pooling using diesel-async and deadpool
// Optimized for Cloud Run serverless deployment

use diesel_async::{AsyncPgConnection, pooled_connection::{AsyncDieselConnectionManager, deadpool::Pool}};
use std::sync::OnceLock;
use anyhow::Result;
use tracing::{info, warn, error};

/// Global Diesel async connection pool that persists across serverless invocations
static GLOBAL_DIESEL_POOL: OnceLock<Pool<AsyncPgConnection>> = OnceLock::new();

/// Serverless-optimized Diesel connection configuration
#[derive(Clone, Debug)]
pub struct DieselServerlessConfig {
    pub database_url: String,
    pub max_size: usize,
    pub acquire_timeout_secs: u64,
}

impl DieselServerlessConfig {
    /// Create optimized config for serverless environments (Cloud Run)
    pub fn for_serverless(database_url: String) -> Self {
        Self {
            database_url,
            // Smaller pool for serverless - memory efficient
            max_size: 10,               // Reduced from typical 20-50
            // Faster timeouts for serverless
            acquire_timeout_secs: 5,    // Reduced from 30s
        }
    }

    /// Create config from environment variables
    pub fn from_env() -> Result<Self> {
        let database_url = std::env::var("DATABASE_URL")
            .map_err(|_| anyhow::anyhow!("DATABASE_URL is required"))?;

        let mut config = Self::for_serverless(database_url);

        // Allow environment overrides
        if let Ok(max_conn) = std::env::var("DB_MAX_CONNECTIONS") {
            config.max_size = max_conn.parse().unwrap_or(config.max_size);
        }

        if let Ok(timeout) = std::env::var("DB_ACQUIRE_TIMEOUT_SECS") {
            config.acquire_timeout_secs = timeout.parse().unwrap_or(5);
        }

        Ok(config)
    }
}

/// Diesel Async Connection Manager for Serverless
pub struct DieselConnectionManager;

impl DieselConnectionManager {
    /// Get or create the global Diesel connection pool (optimized for serverless)
    pub async fn get_pool() -> Result<&'static Pool<AsyncPgConnection>> {
        // Try to get existing pool first (warm container scenario)
        if let Some(pool) = GLOBAL_DIESEL_POOL.get() {
            return Ok(pool);
        }

        // Create new pool (cold start scenario)
        let config = DieselServerlessConfig::from_env()?;
        let pool = Self::create_optimized_pool(config).await?;

        // Store in global static (thread-safe)
        match GLOBAL_DIESEL_POOL.set(pool) {
            Ok(()) => {
                info!("✅ Diesel async pool initialized and cached globally");
                Ok(GLOBAL_DIESEL_POOL.get().unwrap())
            }
            Err(_) => {
                // Another thread already initialized it
                warn!("⚠️ Diesel pool was initialized by another thread");
                Ok(GLOBAL_DIESEL_POOL.get().unwrap())
            }
        }
    }

    /// Create an optimized Diesel async connection pool for serverless environments
    async fn create_optimized_pool(config: DieselServerlessConfig) -> Result<Pool<AsyncPgConnection>> {
        info!("🔗 Creating Diesel async pool for serverless...");
        info!("   Max connections: {}", config.max_size);
        info!("   Acquire timeout: {}s", config.acquire_timeout_secs);

        // Create Diesel connection manager
        let manager = AsyncDieselConnectionManager::<AsyncPgConnection>::new(&config.database_url);

        // Create the pool with simplified configuration
        let pool = Pool::builder(manager)
            .max_size(config.max_size)
            .build()
            .map_err(|e| anyhow::anyhow!("Failed to create Diesel pool: {}", e))?;

        info!("✅ Diesel async pool created successfully");
        Ok(pool)
    }

    /// Get a connection from the pool (optimized for per-request usage)
    pub async fn get_connection() -> Result<&'static Pool<AsyncPgConnection>> {
        Self::get_pool().await
    }

    /// Health check for the Diesel connection pool
    pub async fn health_check() -> bool {
        match Self::get_pool().await {
            Ok(pool) => {
                use diesel::prelude::*;
                use diesel::sql_types::Integer;
                use diesel_async::RunQueryDsl;

                match pool.get().await {
                    Ok(mut conn) => {
                        #[derive(QueryableByName)]
                        struct HealthCheck {
                            #[allow(dead_code)]
                            #[diesel(sql_type = Integer)]
                            result: i32,
                        }

                        match diesel::sql_query("SELECT 1 as result")
                            .get_result::<HealthCheck>(&mut conn)
                            .await
                        {
                            Ok(_) => true,
                            Err(e) => {
                                error!("❌ Diesel health check query failed: {}", e);
                                false
                            }
                        }
                    }
                    Err(e) => {
                        error!("❌ Failed to get Diesel connection: {}", e);
                        false
                    }
                }
            }
            Err(e) => {
                error!("❌ Failed to get Diesel pool: {}", e);
                false
            }
        }
    }

    /// Get pool statistics for monitoring
    pub async fn get_pool_stats() -> Option<DieselPoolStats> {
        Self::get_pool().await.ok().map(|pool| {
            let status = pool.status();
            DieselPoolStats {
                size: status.size as usize,
                available: status.available as usize,
                max_size: status.max_size,
            }
        })
    }
}

/// Diesel pool statistics for monitoring
#[derive(Debug)]
pub struct DieselPoolStats {
    pub size: usize,
    pub available: usize,
    pub max_size: usize,
}

/// Quick access function for getting Diesel database pool in handlers
pub async fn get_diesel_pool() -> Result<&'static Pool<AsyncPgConnection>> {
    DieselConnectionManager::get_connection().await
}

/// Health check function for health endpoints
pub async fn diesel_health_check() -> bool {
    DieselConnectionManager::health_check().await
}
