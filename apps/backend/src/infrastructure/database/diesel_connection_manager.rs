// Diesel Async Connection Manager for Serverless Environments
// Provides efficient connection pooling using diesel-async and deadpool
// Optimized for Cloud Run serverless deployment
//
// kernel extraction wave10: the type definitions (`TlsConnectionManager`,
// `ManagerError`, `TlsPool`, `PoolExt`) now live in the shared
// `epsx-database-pools` crate so that `epsx-identity-shared` and
// `apps/backend` see the same `TlsPool` type. This file retains only
// the backend runtime wiring: the global `OnceLock` pools, the
// initializer struct, the serverless config, and the health-check /
// pool-statistics accessors.

use std::sync::OnceLock;
use anyhow::Result;
use tracing::{info, warn, error};

// Re-export the shared types so existing backend import paths
// (`crate::infrastructure::database::diesel_connection_manager::TlsPool`
// etc.) keep working without a 50-file import-site rewrite.
pub use epsx_database_pools::{ManagerError, PoolExt, TlsConnectionManager, TlsPool};

// Global Pool Type Definition - explicitly using our custom manager
// (re-exported from epsx-database-pools for backward compat)

// Global Diesel async connection pool that persists across serverless invocations
static GLOBAL_DIESEL_POOL: OnceLock<TlsPool> = OnceLock::new();

/// Global Analytics database pool (separate database for high-volume logs)
static GLOBAL_ANALYTICS_POOL: OnceLock<TlsPool> = OnceLock::new();

/// Global Notifications database pool (separate database for real-time notifications)
static GLOBAL_NOTIFICATIONS_POOL: OnceLock<TlsPool> = OnceLock::new();

/// Global Payments database pool (separate database for financial transactions)
static GLOBAL_PAYMENTS_POOL: OnceLock<TlsPool> = OnceLock::new();

/// Health status for all database pools
#[derive(Debug, serde::Serialize, utoipa::ToSchema)]
pub struct AllPoolsHealth {
    pub primary: bool,
    pub analytics: bool,
    pub notifications: bool,
    pub payments: bool,
    pub healthy: bool,
}

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
            .unwrap_or_else(|_| crate::config::get_fallback_config().database_url);

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
    pub async fn get_pool() -> Result<&'static TlsPool> {
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
                info!("Diesel async pool initialized and cached globally");
                Ok(GLOBAL_DIESEL_POOL.get().expect("GLOBAL_DIESEL_POOL initialized above"))
            }
            Err(_) => {
                // Another thread already initialized it
                warn!("Diesel pool was initialized by another thread");
                Ok(GLOBAL_DIESEL_POOL.get().expect("GLOBAL_DIESEL_POOL initialized above"))
            }
        }
    }

    /// Create an optimized Diesel async connection pool for serverless environments
    async fn create_optimized_pool(config: DieselServerlessConfig) -> Result<TlsPool> {
        info!("Creating Diesel async pool for serverless...");
        info!("   Max connections: {}", config.max_size);
        info!("   Acquire timeout: {}s", config.acquire_timeout_secs);

        // Create TLS connection manager
        let manager = TlsConnectionManager::new(config.database_url);

        // Create the pool with simplified configuration
        use deadpool::managed::Timeouts;
        let timeout_dur = Some(std::time::Duration::from_secs(config.acquire_timeout_secs));
        let timeouts = Timeouts {
            wait: timeout_dur,
            create: timeout_dur,
            recycle: timeout_dur,
        };

        let pool = TlsPool::builder(manager)
            .max_size(config.max_size)
            .timeouts(timeouts)
            .runtime(deadpool::Runtime::Tokio1) // Crucial for timeouts
            .build()
            .map_err(|e| anyhow::anyhow!("Failed to create Diesel pool: {}", e))?;

        info!("Diesel async pool created successfully");
        Ok(pool)
    }

    /// Get a connection from the pool (optimized for per-request usage)
    pub async fn get_connection() -> Result<&'static TlsPool> {
        Self::get_pool().await
    }

    /// Get or create the analytics database pool (for high-volume logs)
    /// Falls back to main pool if ANALYTICS_DATABASE_URL is not configured
    pub async fn get_analytics_pool() -> Result<&'static TlsPool> {
        // Check if analytics DB is configured
        let analytics_url = match std::env::var("ANALYTICS_DATABASE_URL") {
            Ok(url) => url,
            Err(_) => {
                // Fall back to main pool if analytics DB not configured
                info!("ANALYTICS_DATABASE_URL not set, using main database pool");
                return Self::get_pool().await;
            }
        };

        // Try to get existing analytics pool first
        if let Some(pool) = GLOBAL_ANALYTICS_POOL.get() {
            return Ok(pool);
        }

        // Create new analytics pool
        let config = DieselServerlessConfig {
            database_url: analytics_url,
            max_size: 5,  // Smaller pool for analytics - write-heavy, less concurrent needs
            acquire_timeout_secs: 5,
        };
        let pool = Self::create_optimized_pool(config).await?;

        // Store in global static
        match GLOBAL_ANALYTICS_POOL.set(pool) {
            Ok(()) => {
                info!("Analytics database pool initialized");
                Ok(GLOBAL_ANALYTICS_POOL.get().expect("GLOBAL_ANALYTICS_POOL initialized above"))
            }
            Err(_) => {
                warn!("Analytics pool was initialized by another thread");
                Ok(GLOBAL_ANALYTICS_POOL.get().expect("GLOBAL_ANALYTICS_POOL initialized above"))
            }
        }
    }

    /// Get or create the notifications database pool (for real-time SSE notifications)
    /// Falls back to main pool if NOTIFICATIONS_DATABASE_URL is not configured
    pub async fn get_notifications_pool() -> Result<&'static TlsPool> {
        // Check if notifications DB is configured
        let notifications_url = match std::env::var("NOTIFICATIONS_DATABASE_URL") {
            Ok(url) => url,
            Err(_) => {
                // Fall back to main pool if notifications DB not configured
                info!("NOTIFICATIONS_DATABASE_URL not set, using main database pool");
                return Self::get_pool().await;
            }
        };

        // Try to get existing notifications pool first
        if let Some(pool) = GLOBAL_NOTIFICATIONS_POOL.get() {
            return Ok(pool);
        }

        // Create new notifications pool
        let config = DieselServerlessConfig {
            database_url: notifications_url,
            max_size: 8,  // Medium pool for notifications - read/write balanced
            acquire_timeout_secs: 3,  // Fast timeout for real-time SSE
        };
        let pool = Self::create_optimized_pool(config).await?;

        // Store in global static
        match GLOBAL_NOTIFICATIONS_POOL.set(pool) {
            Ok(()) => {
                info!("Notifications database pool initialized");
                Ok(GLOBAL_NOTIFICATIONS_POOL.get().expect("GLOBAL_NOTIFICATIONS_POOL initialized above"))
            }
            Err(_) => {
                warn!("Notifications pool was initialized by another thread");
                Ok(GLOBAL_NOTIFICATIONS_POOL.get().expect("GLOBAL_NOTIFICATIONS_POOL initialized above"))
            }
        }
    }

    /// Get or create the payments database pool (for financial transactions)
    /// Falls back to main pool if PAYMENTS_DATABASE_URL is not configured
    pub async fn get_payments_pool() -> Result<&'static TlsPool> {
        // Check if payments DB is configured
        let payments_url = match std::env::var("PAYMENTS_DATABASE_URL") {
            Ok(url) => url,
            Err(_) => {
                // Fall back to main pool if payments DB not configured
                info!("PAYMENTS_DATABASE_URL not set, using main database pool");
                return Self::get_pool().await;
            }
        };

        // Try to get existing payments pool first
        if let Some(pool) = GLOBAL_PAYMENTS_POOL.get() {
            return Ok(pool);
        }

        // Create new payments pool
        let config = DieselServerlessConfig {
            database_url: payments_url,
            max_size: 10,  // Higher pool for payments - critical path
            acquire_timeout_secs: 10,  // Longer timeout for financial transactions
        };
        let pool = Self::create_optimized_pool(config).await?;

        // Store in global static
        match GLOBAL_PAYMENTS_POOL.set(pool) {
            Ok(()) => {
                info!("Payments database pool initialized");
                Ok(GLOBAL_PAYMENTS_POOL.get().expect("GLOBAL_PAYMENTS_POOL initialized above"))
            }
            Err(_) => {
                warn!("Payments pool was initialized by another thread");
                Ok(GLOBAL_PAYMENTS_POOL.get().expect("GLOBAL_PAYMENTS_POOL initialized above"))
            }
        }
    }

    /// Health check for the Diesel connection pool (primary)
    pub async fn health_check() -> bool {
        Self::check_pool(Self::get_pool().await).await
    }

    /// Comprehensive health check for all pools
    pub async fn health_check_all() -> AllPoolsHealth {
        let primary = Self::check_pool(Self::get_pool().await).await;
        // Only check other pools if they are configured or initialized,
        // but for now we try to get them (which initializes/fallback) and check.
        // Falls back to primary pool if not configured, so it effectively checks primary again if not split.
        let analytics = Self::check_pool(Self::get_analytics_pool().await).await;
        let notifications = Self::check_pool(Self::get_notifications_pool().await).await;
        let payments = Self::check_pool(Self::get_payments_pool().await).await;

        AllPoolsHealth {
            primary,
            analytics,
            notifications,
            payments,
            healthy: primary && analytics && notifications && payments,
        }
    }

    /// Helper to check a specific pool health
    async fn check_pool(pool_result: Result<&'static TlsPool>) -> bool {
        match pool_result {
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
                                error!("Diesel health check query failed: {}", e);
                                false
                            }
                        }
                    }
                    Err(e) => {
                        error!("Failed to get Diesel connection: {}", e);
                        false
                    }
                }
            }
            Err(e) => {
                error!("Failed to get Diesel pool: {}", e);
                false
            }
        }
    }

    /// Get pool statistics for monitoring
    pub async fn get_pool_stats() -> Option<DieselPoolStats> {
        Self::get_pool().await.ok().map(|pool| {
            let status = pool.status();
            DieselPoolStats {
                size: status.size,
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
pub async fn get_diesel_pool() -> Result<&'static TlsPool> {
    DieselConnectionManager::get_connection().await
}

/// Quick access function for getting analytics database pool in handlers
pub async fn get_analytics_pool() -> Result<&'static TlsPool> {
    DieselConnectionManager::get_analytics_pool().await
}

/// Quick access function for getting notifications database pool in handlers
pub async fn get_notifications_pool() -> Result<&'static TlsPool> {
    DieselConnectionManager::get_notifications_pool().await
}

/// Quick access function for getting payments database pool in handlers
pub async fn get_payments_pool() -> Result<&'static TlsPool> {
    DieselConnectionManager::get_payments_pool().await
}

/// Health check function for health endpoints
pub async fn diesel_health_check() -> bool {
    DieselConnectionManager::health_check().await
}

/// Comprehensive health check for all databases
pub async fn diesel_health_check_all() -> AllPoolsHealth {
    DieselConnectionManager::health_check_all().await
}
