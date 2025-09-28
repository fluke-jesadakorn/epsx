// Serverless-Optimized Database Connection Manager
// Provides efficient connection pooling for serverless environments
// Reuses connections across invocations while maintaining stateless architecture

use sqlx::PgPool;
use std::sync::OnceLock;
use anyhow::Result;
use tracing::{info, warn, error};

/// Global connection pool that persists across serverless invocations
static GLOBAL_DB_POOL: OnceLock<PgPool> = OnceLock::new();

/// Serverless-optimized connection configuration
#[derive(Clone, Debug)]
pub struct ServerlessConnectionConfig {
    pub database_url: String,
    pub max_connections: u32,
    pub min_connections: u32,
    pub acquire_timeout: std::time::Duration,
    pub idle_timeout: std::time::Duration,
    pub max_lifetime: std::time::Duration,
}

impl ServerlessConnectionConfig {
    /// Create optimized config for serverless environments
    pub fn for_serverless(database_url: String) -> Self {
        Self {
            database_url,
            // Smaller pool for serverless - memory efficient
            max_connections: 10,  // Reduced from typical 20-50
            min_connections: 1,   // Always keep one connection warm
            // Faster timeouts for serverless
            acquire_timeout: std::time::Duration::from_secs(5),  // Reduced from 30s
            idle_timeout: std::time::Duration::from_secs(300),   // 5 minutes
            max_lifetime: std::time::Duration::from_secs(1800),  // 30 minutes
        }
    }

    /// Create config from environment variables
    pub fn from_env() -> Result<Self> {
        let database_url = std::env::var("DATABASE_URL")
            .map_err(|_| anyhow::anyhow!("DATABASE_URL is required"))?;

        let mut config = Self::for_serverless(database_url);

        // Allow environment overrides
        if let Ok(max_conn) = std::env::var("DB_MAX_CONNECTIONS") {
            config.max_connections = max_conn.parse().unwrap_or(config.max_connections);
        }

        if let Ok(min_conn) = std::env::var("DB_MIN_CONNECTIONS") {
            config.min_connections = min_conn.parse().unwrap_or(config.min_connections);
        }

        if let Ok(timeout) = std::env::var("DB_ACQUIRE_TIMEOUT_SECS") {
            config.acquire_timeout = std::time::Duration::from_secs(
                timeout.parse().unwrap_or(5)
            );
        }

        Ok(config)
    }
}

/// Serverless Connection Manager
pub struct ServerlessConnectionManager;

impl ServerlessConnectionManager {
    /// Get or create the global connection pool (optimized for serverless)
    pub async fn get_pool() -> Result<&'static PgPool> {
        // Try to get existing pool first (warm container scenario)
        if let Some(pool) = GLOBAL_DB_POOL.get() {
            return Ok(pool);
        }

        // Create new pool (cold start scenario)
        let config = ServerlessConnectionConfig::from_env()?;
        let pool = Self::create_optimized_pool(config).await?;

        // Store in global static (thread-safe)
        match GLOBAL_DB_POOL.set(pool) {
            Ok(()) => {
                info!("✅ Serverless database pool initialized and cached globally");
                Ok(GLOBAL_DB_POOL.get().unwrap())
            }
            Err(_) => {
                // Another thread already initialized it
                warn!("⚠️ Database pool was initialized by another thread");
                Ok(GLOBAL_DB_POOL.get().unwrap())
            }
        }
    }

    /// Create an optimized connection pool for serverless environments
    async fn create_optimized_pool(config: ServerlessConnectionConfig) -> Result<PgPool> {
        info!("🔗 Creating serverless-optimized database pool...");
        info!("   Max connections: {}", config.max_connections);
        info!("   Min connections: {}", config.min_connections);
        info!("   Acquire timeout: {:?}", config.acquire_timeout);

        let pool = sqlx::postgres::PgPoolOptions::new()
            .max_connections(config.max_connections)
            .min_connections(config.min_connections)
            .acquire_timeout(config.acquire_timeout)
            .idle_timeout(config.idle_timeout)
            .max_lifetime(config.max_lifetime)
            // Serverless optimizations
            .test_before_acquire(false) // Skip test queries for speed
            .after_connect(|_conn, _meta| Box::pin(async move {
                // Optimize connection settings for serverless
                Ok(())
            }))
            .connect(&config.database_url)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to create connection pool: {}", e))?;

        info!("✅ Serverless database pool created successfully");
        Ok(pool)
    }

    /// Get a connection from the pool (optimized for per-request usage)
    pub async fn get_connection() -> Result<&'static PgPool> {
        Self::get_pool().await
    }

    /// Health check for the connection pool
    pub async fn health_check() -> bool {
        match Self::get_pool().await {
            Ok(pool) => {
                match sqlx::query!("SELECT 1 as health_check")
                    .fetch_one(pool)
                    .await
                {
                    Ok(_) => true,
                    Err(e) => {
                        error!("❌ Database health check failed: {}", e);
                        false
                    }
                }
            }
            Err(e) => {
                error!("❌ Failed to get database pool: {}", e);
                false
            }
        }
    }

    /// Get pool statistics for monitoring
    pub async fn get_pool_stats() -> Option<PoolStats> {
        Self::get_pool().await.ok().map(|pool| PoolStats {
            size: pool.size(),
            idle: pool.num_idle(),
            is_closed: pool.is_closed(),
        })
    }

    /// Force close the pool (for testing or cleanup)
    pub async fn close_pool() {
        if let Some(pool) = GLOBAL_DB_POOL.get() {
            pool.close().await;
            info!("🛑 Database pool closed");
        }
    }
}

/// Pool statistics for monitoring
#[derive(Debug)]
pub struct PoolStats {
    pub size: u32,
    pub idle: usize,
    pub is_closed: bool,
}

/// Helper trait for database operations with automatic connection management
pub trait DatabaseExecutor {
    fn execute_with_pool<F, T>(&self, operation: F) -> impl std::future::Future<Output = Result<T>> + Send
    where
        F: FnOnce(&PgPool) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<T>> + Send + '_>> + Send,
        T: Send;
}

/// Implementation for any type that needs database access
impl<S> DatabaseExecutor for S {
    fn execute_with_pool<F, T>(&self, operation: F) -> impl std::future::Future<Output = Result<T>> + Send
    where
        F: FnOnce(&PgPool) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<T>> + Send + '_>> + Send,
        T: Send,
    {
        async move {
            let pool = ServerlessConnectionManager::get_connection().await?;
            operation(pool).await
        }
    }
}

/// Quick access function for getting database pool in handlers
pub async fn get_db_pool() -> Result<&'static PgPool> {
    ServerlessConnectionManager::get_connection().await
}

/// Health check function for health endpoints
pub async fn db_health_check() -> bool {
    ServerlessConnectionManager::health_check().await
}