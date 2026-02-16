// Diesel Async Connection Manager for Serverless Environments
// Provides efficient connection pooling using diesel-async and deadpool
// Optimized for Cloud Run serverless deployment

use diesel_async::{AsyncPgConnection, RunQueryDsl};
use deadpool::managed::{Manager, Pool, RecycleResult, RecycleError};
use std::sync::OnceLock;
use anyhow::Result;
use tracing::{debug, info, warn, error};
use tokio_postgres_rustls::MakeRustlsConnect;
use rustls::ClientConfig;
use std::str::FromStr;
use async_trait::async_trait;

/// Custom Error type for the Connection Manager
#[derive(Debug, thiserror::Error)]
pub enum ManagerError {
    #[error("Database connection error: {0}")]
    Connection(#[from] tokio_postgres::Error),
    #[error("Internal error: {0}")]
    Internal(String),
    #[error("Configuration error: {0}")]
    Config(String),
}

/// Custom Connection Manager that enforces TLS
#[derive(Clone)]
pub struct TlsConnectionManager {
    database_url: String,
}

impl TlsConnectionManager {
    pub fn new(database_url: String) -> Self {
        Self { database_url }
    }
}

#[async_trait]
impl Manager for TlsConnectionManager {
    type Type = AsyncPgConnection;
    type Error = ManagerError;

    async fn create(&self) -> Result<AsyncPgConnection, ManagerError> {
        let config = tokio_postgres::Config::from_str(&self.database_url)
            .map_err(|e| ManagerError::Config(e.to_string()))?;
        
        let connect_timeout = std::time::Duration::from_secs(5);
        
        debug!("Connecting to database (SSL Mode: {:?})...", config.get_ssl_mode());

        let client = match config.get_ssl_mode() {
            tokio_postgres::config::SslMode::Disable => {
                let (client, connection) = tokio::time::timeout(connect_timeout, config.connect(tokio_postgres::NoTls))
                    .await
                    .map_err(|_| ManagerError::Config("Database connection timed out".to_string()))?
                    .map_err(|e| {
                        error!("Connection error: {:?}", e);
                        ManagerError::Connection(e)
                    })?;

                tokio::spawn(async move {
                    if let Err(e) = connection.await {
                        error!("database connection error: {}", e);
                    }
                });
                client
            }
            _ => {
                let root_store = rustls::RootCertStore::from_iter(
                    webpki_roots::TLS_SERVER_ROOTS.iter().cloned()
                );
                let client_config = ClientConfig::builder()
                    .with_root_certificates(root_store)
                    .with_no_client_auth();
                let tls = MakeRustlsConnect::new(client_config);
                
                let (client, connection) = tokio::time::timeout(connect_timeout, config.connect(tls))
                    .await
                    .map_err(|_| ManagerError::Config("Database connection timed out during TLS handshake".to_string()))?
                    .map_err(|e| {
                        error!("TLS Connection error: {}", e);
                        ManagerError::Connection(e)
                    })?;

                tokio::spawn(async move {
                    if let Err(e) = connection.await {
                        error!("database connection error: {}", e);
                    }
                });
                client
            }
        };
        
        debug!("Wrapping in AsyncPgConnection...");
        tokio::time::timeout(connect_timeout, AsyncPgConnection::try_from(client))
            .await
            .map_err(|_| ManagerError::Config("AsyncPgConnection wrapper timed out".to_string()))?
            .map_err(|e| {
                error!("AsyncPgConnection conversion error: {}", e);
                ManagerError::Internal(e.to_string())
            })
    }

    async fn recycle(&self, conn: &mut AsyncPgConnection) -> RecycleResult<ManagerError> {
        // Simple health check query
        diesel::sql_query("SELECT 1")
            .execute(conn)
            .await
            .map(|_| ())
            .map_err(|e| RecycleError::Backend(ManagerError::Internal(e.to_string())))
    }
}

// Global Pool Type Definition - explicitly using our custom manager
pub type TlsPool = Pool<TlsConnectionManager>;

/// Extension trait for TlsPool to reduce DB connection boilerplate
#[async_trait]
pub trait PoolExt {
    /// Get a connection from the pool, mapping errors to AppError
    async fn conn(&self) -> crate::core::errors::AppResult<deadpool::managed::Object<TlsConnectionManager>>;
}

#[async_trait]
impl PoolExt for TlsPool {
    async fn conn(&self) -> crate::core::errors::AppResult<deadpool::managed::Object<TlsConnectionManager>> {
        self.get().await
            .map_err(|e| crate::core::errors::AppError::database_error(e.to_string()))
    }
}

/// Global Diesel async connection pool that persists across serverless invocations
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
        let manager = TlsConnectionManager {
            database_url: config.database_url,
        };

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


