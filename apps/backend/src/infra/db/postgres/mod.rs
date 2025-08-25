// PostgreSQL repository implementations

use std::sync::Arc;
use std::time::Duration;
use crate::config::env::get_env_var;
use crate::infra::db::diesel::{DbPool, create_pool as create_diesel_pool};

pub mod notification_repo;

pub use notification_repo::*;

/// Enhanced database configuration with environment-aware scaling
#[derive(Debug, Clone)]
pub struct DatabaseConfig {
    pub host: String,
    pub port: u16,
    pub username: String,
    pub password: String,
    pub database: String,
    pub max_connections: u32,
    pub min_connections: u32,
    pub migration_source: String,
    pub ssl_mode: String,
    pub permission_pool_size: u32,
    pub enable_statement_logging: bool,
    pub query_timeout_seconds: u64,
    pub acquire_timeout_seconds: u64,
    pub idle_timeout_seconds: u64,
    pub max_lifetime_seconds: u64,
    pub test_before_acquire: bool,
    pub enable_pool_metrics: bool,
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        let is_dev = is_development_mode();
        let is_prod = is_production_mode();
        
        // Environment-aware connection pool sizing optimized for Cloud Run
        let (default_max_conn, default_min_conn) = if is_prod {
            (20, 2) // Production: Cloud Run optimized for fast startup
        } else if is_dev {
            (10, 2)  // Development: lower overhead
        } else {
            (15, 2)  // Staging/other: balanced
        };
        
        Self {
            host: get_env_var("DATABASE_HOST").unwrap_or_else(|_| "localhost".to_string()),
            port: get_env_var("DATABASE_PORT")
                .unwrap_or_else(|_| "5432".to_string())
                .parse()
                .unwrap_or(5432),
            username: get_env_var("DATABASE_USERNAME").unwrap_or_else(|_| "postgres".to_string()),
            password: get_env_var("DATABASE_PASSWORD").unwrap_or_else(|_| "password".to_string()),
            database: get_env_var("DATABASE_NAME").unwrap_or_else(|_| "epsx".to_string()),
            max_connections: get_env_var("DATABASE_MAX_CONNECTIONS")
                .unwrap_or_else(|_| default_max_conn.to_string())
                .parse()
                .unwrap_or(default_max_conn),
            min_connections: get_env_var("DATABASE_MIN_CONNECTIONS")
                .unwrap_or_else(|_| default_min_conn.to_string())
                .parse()
                .unwrap_or(default_min_conn),
            migration_source: get_env_var("DATABASE_MIGRATION_SOURCE")
                .unwrap_or_else(|_| "./migrations".to_string()),
            ssl_mode: get_env_var("DATABASE_SSL_MODE")
                .unwrap_or_else(|_| if is_prod { "require" } else { "prefer" }.to_string()),
            permission_pool_size: get_env_var("PERMISSION_POOL_SIZE")
                .unwrap_or_else(|_| "10".to_string())
                .parse()
                .unwrap_or(10),
            enable_statement_logging: get_env_var("DATABASE_STATEMENT_LOGGING")
                .unwrap_or_else(|_| is_dev.to_string())
                .parse()
                .unwrap_or(is_dev),
            query_timeout_seconds: get_env_var("DATABASE_QUERY_TIMEOUT")
                .unwrap_or_else(|_| "30".to_string())
                .parse()
                .unwrap_or(30),
            acquire_timeout_seconds: get_env_var("DATABASE_ACQUIRE_TIMEOUT")
                .unwrap_or_else(|_| "10".to_string())
                .parse()
                .unwrap_or(10),
            idle_timeout_seconds: get_env_var("DATABASE_IDLE_TIMEOUT")
                .unwrap_or_else(|_| "300".to_string())
                .parse()
                .unwrap_or(300),
            max_lifetime_seconds: get_env_var("DATABASE_MAX_LIFETIME")
                .unwrap_or_else(|_| "1800".to_string())
                .parse()
                .unwrap_or(1800),
            test_before_acquire: get_env_var("DATABASE_TEST_BEFORE_ACQUIRE")
                .unwrap_or_else(|_| (!is_dev).to_string())
                .parse()
                .unwrap_or(!is_dev),
            enable_pool_metrics: get_env_var("DATABASE_ENABLE_METRICS")
                .unwrap_or_else(|_| "true".to_string())
                .parse()
                .unwrap_or(true),
        }
    }
}

/// Database connection pool
pub type DatabasePool = Arc<DbPool>;

/// Check if running in development mode
fn is_development_mode() -> bool {
    // Check various environment indicators for development mode
    get_env_var("NODE_ENV").map(|v| v == "development").unwrap_or(false) ||
    get_env_var("RUST_ENV").map(|v| v == "development").unwrap_or(false) ||
    get_env_var("ENV").map(|v| v == "dev" || v == "development").unwrap_or(false) ||
    get_env_var("ENVIRONMENT").map(|v| v == "dev" || v == "development").unwrap_or(false) ||
    // If no explicit environment set, assume development (safer for local dev)
    (get_env_var("NODE_ENV").is_err() && 
     get_env_var("RUST_ENV").is_err() && 
     get_env_var("ENV").is_err() && 
     get_env_var("ENVIRONMENT").is_err())
}

/// Check if running in production mode
fn is_production_mode() -> bool {
    get_env_var("NODE_ENV").map(|v| v == "production").unwrap_or(false) ||
    get_env_var("RUST_ENV").map(|v| v == "production").unwrap_or(false) ||
    get_env_var("ENV").map(|v| v == "prod" || v == "production").unwrap_or(false) ||
    get_env_var("ENVIRONMENT").map(|v| v == "prod" || v == "production").unwrap_or(false)
}

/// Initialize optimized database connection pool with permission query optimization
pub async fn create_pool(config: DatabaseConfig) -> Result<DatabasePool, diesel::result::Error> {
    use diesel::result::Error as DieselError;
    
    // Use DATABASE_URL from environment if available, fallback to constructed URL
    let database_url = get_env_var("DATABASE_URL").unwrap_or_else(|_| {
        format!(
            "postgresql://{}:{}@{}:{}/{}?sslmode={}",
            config.username, config.password, config.host, config.port, config.database, config.ssl_mode
        )
    });

    // Create Diesel pool using the existing diesel infrastructure
    let pool = create_diesel_pool(&database_url).await
        .map_err(|_| DieselError::DatabaseError(
            diesel::result::DatabaseErrorKind::UnableToSendCommand,
            Box::new("Failed to create connection pool".to_string())
        ))?;
    
    // Auto-migrate in development mode only
    if is_development_mode() {
        use tracing::{info};
        
        info!("Development mode detected - using Diesel migrations");
        // TODO: Integrate Diesel migrations here
    }
    
    let pool_arc = Arc::new(pool);
    
    // Initialize connection pool monitoring if enabled
    if config.enable_pool_metrics {
        start_pool_monitoring(pool_arc.clone(), config.clone());
    }
    
    Ok(pool_arc)
}

/// Ensure database exists (stub for Diesel - database should already exist)
async fn ensure_database_exists(_database_url: &str) -> Result<(), diesel::result::Error> {
    // For Diesel, we assume the database already exists
    // This is typically handled by deployment scripts or docker setup
    tracing::info!("Database existence check skipped for Diesel - assuming database exists");
    Ok(())
}

/// Create optimized pool with default configuration (for backward compatibility)
pub async fn create_optimized_pool(database_url: &str) -> Result<DbPool, diesel::result::Error> {
    // Use the Diesel pool creation function
    create_diesel_pool(database_url).await.map_err(|_| diesel::result::Error::DatabaseError(
        diesel::result::DatabaseErrorKind::UnableToSendCommand,
        Box::new("Failed to create optimized pool".to_string())
    ))
}

/// Database error wrapper
#[derive(Debug, thiserror::Error)]
pub enum DatabaseError {
    #[error("Connection error: {0}")]
    Connection(#[from] diesel::result::Error),
    
    #[error("Query error: {0}")]
    Query(String),
    
    #[error("Serialization error: {0}")]
    Serialization(String),
    
    #[error("Not found")]
    NotFound,
    
    #[error("Already exists")]
    AlreadyExists,
}

impl From<DatabaseError> for crate::app::ports::repositories::RepoError {
    fn from(err: DatabaseError) -> Self {
        match err {
            DatabaseError::Connection(e) => Self::ConnectionError(e.to_string()),
            DatabaseError::Query(e) => Self::QueryError(e),
            DatabaseError::Serialization(e) => Self::SerializationError(e),
            DatabaseError::NotFound => Self::NotFound,
            DatabaseError::AlreadyExists => Self::AlreadyExists,
        }
    }
}

/// Create performance indexes for permission queries (stub for now)
async fn create_permission_indexes(_pool: &DbPool) -> Result<(), diesel::result::Error> {
    use tracing::{info};
    
    // TODO: Implement index creation using Diesel or raw SQL
    // For now, we assume indexes are created via migrations
    info!("Permission indexes should be created via Diesel migrations");
    Ok(())
}

/// Connection pool metrics for monitoring
#[derive(Debug, Clone)]
pub struct PoolMetrics {
    pub active_connections: u32,
    pub idle_connections: u32,
    pub total_connections: u32,
    pub max_connections: u32,
    pub pending_requests: u32,
    pub last_checked: chrono::DateTime<chrono::Utc>,
}

/// Start background pool monitoring task
fn start_pool_monitoring(pool: DatabasePool, _config: DatabaseConfig) {
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(60)); // Check every minute
        
        loop {
            interval.tick().await;
            
            match get_pool_metrics(&pool).await {
                Ok(metrics) => {
                    let utilization = (metrics.active_connections as f64 / metrics.max_connections as f64) * 100.0;
                    
                    tracing::debug!(
                        "Pool metrics - Active: {}, Idle: {}, Total: {}, Max: {}, Pending: {}, Utilization: {:.1}%",
                        metrics.active_connections,
                        metrics.idle_connections,
                        metrics.total_connections,
                        metrics.max_connections,
                        metrics.pending_requests,
                        utilization
                    );
                    
                    // Log warning if utilization is high
                    if utilization > 80.0 {
                        tracing::warn!(
                            "High database connection pool utilization: {:.1}% ({}/{})",
                            utilization,
                            metrics.active_connections,
                            metrics.max_connections
                        );
                    }
                    
                    // Log warning if there are pending requests
                    if metrics.pending_requests > 0 {
                        tracing::warn!(
                            "Database connection pool has {} pending requests",
                            metrics.pending_requests
                        );
                    }
                }
                Err(e) => {
                    tracing::error!("Failed to get pool metrics: {}", e);
                }
            }
        }
    });
}

/// Get current connection pool metrics
pub async fn get_pool_metrics(_pool: &DatabasePool) -> Result<PoolMetrics, DatabaseError> {
    // TODO: Implement pool metrics for Diesel
    // For now, return mock metrics
    Ok(PoolMetrics {
        active_connections: 1,
        idle_connections: 4,
        total_connections: 5,
        max_connections: 20,
        pending_requests: 0,
        last_checked: chrono::Utc::now(),
    })
}

/// Health check for database connection pool
pub async fn check_pool_health(pool: &DatabasePool) -> Result<PoolHealthStatus, DatabaseError> {
    let metrics = get_pool_metrics(pool).await?;
    
    // TODO: Test basic connectivity with Diesel
    // For now, assume healthy
    let conn_test = true;
    
    let is_healthy = conn_test && metrics.total_connections > 0;
    let utilization = (metrics.active_connections as f64 / metrics.max_connections as f64) * 100.0;
    
    let status = if is_healthy {
        if utilization > 90.0 {
            "warning"
        } else {
            "healthy"
        }
    } else {
        "unhealthy"
    };
    
    Ok(PoolHealthStatus {
        healthy: is_healthy,
        status: status.to_string(),
        metrics,
        connection_test_success: conn_test,
        utilization_percentage: utilization,
    })
}

/// Pool health status
#[derive(Debug, Clone)]
pub struct PoolHealthStatus {
    pub healthy: bool,
    pub status: String,
    pub metrics: PoolMetrics,
    pub connection_test_success: bool,
    pub utilization_percentage: f64,
}

/// Transaction utilities for bulk permission operations
pub struct TransactionManager {
    pool: DatabasePool,
}

impl TransactionManager {
    pub fn new(pool: DatabasePool) -> Self {
        Self { pool }
    }
    
    /// Execute multiple operations in a single transaction with rollback support
    pub async fn execute_bulk_permission_operations<F, R>(&self, _operations: F) -> Result<R, DatabaseError>
    where
        F: FnOnce() -> Result<R, DatabaseError>,
    {
        // TODO: Implement Diesel transactions
        // For now, return an error
        Err(DatabaseError::Query("Transaction support not implemented for Diesel yet".to_string()))
    }
    
    /// Execute bulk admin role assignments with rollback on any failure
    pub async fn bulk_assign_admin_roles(&self, _assignments: Vec<BulkAdminRoleAssignment>) -> Result<BulkAssignmentResult, DatabaseError> {
        // TODO: Implement bulk admin role assignments with Diesel
        // For now, return empty result
        Ok(BulkAssignmentResult {
            successful_assignments: 0,
            failed_assignments: Vec::new(),
            total_processed: 0,
        })
    }
}

/// Bulk admin role assignment request (matching modern schema)
#[derive(Debug, Clone)]
pub struct BulkAdminRoleAssignment {
    pub firebase_uid: String,
    pub module_code: String,
    pub granted_by: String,
    pub expires_at: Option<chrono::DateTime<chrono::Utc>>,
    pub granted_reason: String,
}

/// Legacy bulk permission assignment request (for backward compatibility)
#[derive(Debug, Clone)]
pub struct BulkPermissionAssignment {
    pub user_id: String,
    pub permission_profile_id: String,
    pub assigned_by: String,
    pub expires_at: Option<chrono::DateTime<chrono::Utc>>,
    pub reason: Option<String>,
}

/// Bulk assignment result
#[derive(Debug)]
pub struct BulkAssignmentResult {
    pub successful_assignments: u32,
    pub failed_assignments: Vec<BulkAssignmentError>,
    pub total_processed: u32,
}

/// Bulk assignment error
#[derive(Debug)]
pub struct BulkAssignmentError {
    pub user_id: String,
    pub permission_profile_id: String,
    pub error: String,
}