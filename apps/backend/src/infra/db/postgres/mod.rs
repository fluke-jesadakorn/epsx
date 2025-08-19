// PostgreSQL repository implementations

use sqlx::PgPool;
use std::sync::Arc;
use std::time::Duration;
use serde_json;
use crate::config::env::get_env_var;

pub mod user_repo;
// pub mod user_repo_soft_delete;
pub mod session_repo;
pub mod payment_repo;
pub mod stock_repo;
pub mod iam_repo;
pub mod audit_repo;
pub mod permission_profile_repo;
pub mod assign_repo;
pub mod eps_ranking_repo;
pub mod module_repo;
// TODO: Implement level_history_repo
// pub mod level_history_repo;

pub use user_repo::*;
// pub use user_repo_soft_delete::*;
pub use session_repo::*;
pub use payment_repo::*;
pub use stock_repo::*;
pub use iam_repo::*;
pub use audit_repo::*;
pub use permission_profile_repo::*;
pub use assign_repo::*;
pub use eps_ranking_repo::*;
pub use module_repo::*;
// pub use level_history_repo::*;

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
        
        // Environment-aware connection pool sizing
        let (default_max_conn, default_min_conn) = if is_prod {
            (50, 10) // Production: higher capacity
        } else if is_dev {
            (10, 2)  // Development: lower overhead
        } else {
            (25, 5)  // Staging/other: balanced
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
pub type DatabasePool = Arc<PgPool>;

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
pub async fn create_pool(config: DatabaseConfig) -> Result<DatabasePool, sqlx::Error> {
    use sqlx::postgres::PgPoolOptions;
    use std::time::Duration;
    
    // Use DATABASE_URL from environment if available, fallback to constructed URL
    let database_url = get_env_var("DATABASE_URL").unwrap_or_else(|_| {
        format!(
            "postgresql://{}:{}@{}:{}/{}?sslmode={}",
            config.username, config.password, config.host, config.port, config.database, config.ssl_mode
        )
    });

    // Ensure database exists before creating pool
    ensure_database_exists(&database_url).await?;

    let pool = PgPoolOptions::new()
        .max_connections(config.max_connections)
        .min_connections(config.min_connections)
        .acquire_timeout(Duration::from_secs(config.acquire_timeout_seconds))
        .idle_timeout(Duration::from_secs(config.idle_timeout_seconds))
        .max_lifetime(Duration::from_secs(config.max_lifetime_seconds))
        .test_before_acquire(config.test_before_acquire)
        .after_connect(move |conn, _meta| {
            let logging_enabled = config.enable_statement_logging;
            Box::pin(async move {
                // Optimize for permission queries
                sqlx::query("SET work_mem = '32MB'")
                    .execute(&mut *conn)
                    .await?;
                
                // Enable JIT for complex permission resolution queries
                sqlx::query("SET jit = on")
                    .execute(&mut *conn)
                    .await?;
                
                // Optimize random page cost for SSD storage
                sqlx::query("SET random_page_cost = 1.1")
                    .execute(&mut *conn)
                    .await?;
                
                if logging_enabled {
                    // Enable statement logging for debugging
                    sqlx::query("SET log_statement = 'all'")
                        .execute(&mut *conn)
                        .await?;
                }
                
                Ok(())
            })
        })
        .connect(&database_url)
        .await?;
    
    // Auto-migrate in development mode only
    if is_development_mode() {
        use tracing::{info};
        
        info!("Development mode detected - auto-migration temporarily disabled");
        
        // Create permission query performance indexes
        match create_permission_indexes(&pool).await {
            Ok(_) => info!("✅ Permission indexes created/verified"),
            Err(e) => info!("⚠️  Permission index creation failed: {}", e),
        }
    }
    
    let pool_arc = Arc::new(pool);
    
    // Initialize connection pool monitoring if enabled
    if config.enable_pool_metrics {
        start_pool_monitoring(pool_arc.clone(), config.clone());
    }
    
    Ok(pool_arc)
}

/// Ensure database exists by connecting to postgres and creating it if needed
async fn ensure_database_exists(database_url: &str) -> Result<(), sqlx::Error> {
    use sqlx::postgres::{PgPoolOptions, PgConnectOptions};
    use std::str::FromStr;
    
    tracing::info!("Checking if database exists...");
    
    // Parse the database URL to get connection details
    let opts = PgConnectOptions::from_str(database_url)?;
    let db_name = opts.get_database().unwrap_or("epsx_db");
    
    // Extract connection details to build master URL
    let master_url = database_url.replace(&format!("/{}", db_name), "/postgres");
    
    // Connect to master postgres database
    let master_pool = PgPoolOptions::new()
        .max_connections(1)
        .connect(&master_url)
        .await?;
    
    // Check if database exists
    let exists = sqlx::query_scalar::<_, bool>(
        "SELECT EXISTS(SELECT 1 FROM pg_database WHERE datname = $1)"
    )
    .bind(db_name)
    .fetch_one(&master_pool)
    .await?;
    
    if !exists {
        tracing::info!("Database '{}' does not exist, creating it...", db_name);
        
        // Create the database
        let create_query = format!("CREATE DATABASE \"{}\"", db_name);
        sqlx::query(&create_query)
            .execute(&master_pool)
            .await?;
        
        tracing::info!("✅ Database '{}' created successfully", db_name);
    } else {
        tracing::info!("✅ Database '{}' already exists", db_name);
    }
    
    master_pool.close().await;
    Ok(())
}

/// Create optimized pool with default configuration (for backward compatibility)
pub async fn create_optimized_pool(database_url: &str) -> Result<PgPool, sqlx::Error> {
    use sqlx::postgres::PgPoolOptions;
    use std::time::Duration;
    
    PgPoolOptions::new()
        .max_connections(20)
        .min_connections(5)
        .acquire_timeout(Duration::from_secs(10))
        .idle_timeout(Duration::from_secs(300))
        .max_lifetime(Duration::from_secs(1800))
        .connect(database_url)
        .await
}

/// Database error wrapper
#[derive(Debug, thiserror::Error)]
pub enum DatabaseError {
    #[error("Connection error: {0}")]
    Connection(#[from] sqlx::Error),
    
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

/// Create performance indexes for permission queries
async fn create_permission_indexes(pool: &PgPool) -> Result<(), sqlx::Error> {
    use tracing::{info, warn};
    
    let indexes = [
        // Admin role assignment indexes (matching modern schema)
        ("idx_user_admin_roles_firebase_uid_active", 
         "CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_user_admin_roles_firebase_uid_active 
          ON user_admin_roles(firebase_uid) WHERE is_active = true"),
        
        ("idx_user_admin_roles_module_active",
         "CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_user_admin_roles_module_active 
          ON user_admin_roles(module_code) WHERE is_active = true"),
        
        ("idx_user_admin_roles_expires_at_active",
         "CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_user_admin_roles_expires_at_active 
          ON user_admin_roles(expires_at) WHERE expires_at IS NOT NULL AND is_active = true"),
        
        // Admin module permission indexes
        ("idx_admin_module_permissions_module_code",
         "CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_admin_module_permissions_module_code 
          ON admin_module_permissions(module_code)"),
        
        ("idx_admin_module_permissions_access_level",
         "CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_admin_module_permissions_access_level 
          ON admin_module_permissions(access_level)"),
        
        // Admin role audit indexes (matching modern schema)
        ("idx_admin_role_audit_firebase_uid",
         "CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_admin_role_audit_firebase_uid 
          ON admin_role_audit(firebase_uid)"),
        
        ("idx_admin_role_audit_timestamp",
         "CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_admin_role_audit_timestamp 
          ON admin_role_audit(timestamp DESC)"),
        
        // Session indexes for JWT validation
        ("idx_sessions_user_id_active",
         "CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_sessions_user_id_active 
          ON sessions(user_id) WHERE is_active = true"),
        
        ("idx_sessions_expires_at_active",
         "CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_sessions_expires_at_active 
          ON sessions(expires_at) WHERE is_active = true"),
        
        // Audit logs performance indexes
        ("idx_audit_logs_user_timestamp",
         "CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_audit_logs_user_timestamp 
          ON audit_logs(user_id, timestamp DESC)"),
        
        ("idx_audit_logs_action_timestamp",
         "CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_audit_logs_action_timestamp 
          ON audit_logs(action, timestamp DESC)"),
        
        // Temporary permissions indexes (these exist in the schema)
        ("idx_temporary_permissions_user_active",
         "CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_temporary_permissions_user_active 
          ON temporary_permissions(user_id) WHERE status = 'active'"),
    ];
    
    for (name, sql) in indexes {
        match sqlx::query(sql).execute(pool).await {
            Ok(_) => info!("✅ Created/verified index: {}", name),
            Err(e) => {
                let error_msg = e.to_string();
                if error_msg.contains("already exists") {
                    info!("✅ Index already exists: {}", name);
                } else {
                    warn!("⚠️  Failed to create index {}: {}", name, e);
                }
            }
        }
    }
    
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
pub async fn get_pool_metrics(pool: &DatabasePool) -> Result<PoolMetrics, DatabaseError> {
    let pool_inner = pool.as_ref();
    
    let total_connections = pool_inner.size() as u32;
    let idle_connections = pool_inner.num_idle() as u32;
    let active_connections = total_connections.saturating_sub(idle_connections);
    
    Ok(PoolMetrics {
        active_connections,
        idle_connections,
        total_connections,
        max_connections: pool_inner.options().get_max_connections(),
        pending_requests: 0, // SQLx doesn't expose this directly
        last_checked: chrono::Utc::now(),
    })
}

/// Health check for database connection pool
pub async fn check_pool_health(pool: &DatabasePool) -> Result<PoolHealthStatus, DatabaseError> {
    let metrics = get_pool_metrics(pool).await?;
    
    // Test basic connectivity
    let conn_test = sqlx::query("SELECT 1").fetch_one(pool.as_ref()).await;
    
    let is_healthy = conn_test.is_ok() && metrics.total_connections > 0;
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
        connection_test_success: conn_test.is_ok(),
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
    pub async fn execute_bulk_permission_operations<F, Fut, R>(&self, operations: F) -> Result<R, DatabaseError>
    where
        F: FnOnce(&mut sqlx::Transaction<'_, sqlx::Postgres>) -> Fut,
        Fut: std::future::Future<Output = Result<R, DatabaseError>> + Send,
    {
        let mut tx = self.pool.begin().await.map_err(DatabaseError::Connection)?;
        
        match operations(&mut tx).await {
            Ok(result) => {
                tx.commit().await.map_err(DatabaseError::Connection)?;
                Ok(result)
            },
            Err(e) => {
                if let Err(rollback_err) = tx.rollback().await {
                    tracing::error!("Failed to rollback transaction: {}", rollback_err);
                }
                Err(e)
            }
        }
    }
    
    /// Execute bulk admin role assignments with rollback on any failure
    pub async fn bulk_assign_admin_roles(&self, assignments: Vec<BulkAdminRoleAssignment>) -> Result<BulkAssignmentResult, DatabaseError> {
        let mut tx = self.pool.begin().await.map_err(DatabaseError::Connection)?;
        let mut successful_assignments = 0;
        let mut failed_assignments = Vec::new();
        
        for assignment in assignments {
            let result = sqlx::query(
                r#"
                INSERT INTO user_admin_roles 
                (firebase_uid, module_code, granted_by, granted_reason, expires_at, is_active, created_at)
                VALUES ($1, $2, $3, $4, $5, true, NOW())
                ON CONFLICT (firebase_uid, module_code) 
                DO UPDATE SET 
                    expires_at = EXCLUDED.expires_at,
                    granted_reason = EXCLUDED.granted_reason,
                    is_active = true,
                    updated_at = NOW()
                "#,
            )
            .bind(&assignment.firebase_uid)
            .bind(&assignment.module_code)
            .bind(&assignment.granted_by)
            .bind(&assignment.granted_reason)
            .bind(assignment.expires_at.map(|dt| dt.naive_utc()))
            .execute(&mut *tx)
            .await;
            
            match result {
                Ok(_) => {
                    successful_assignments += 1;
                    
                    // Log the assignment in admin role audit
                    let _audit_result = sqlx::query(
                        r#"
                        INSERT INTO admin_role_audit 
                        (firebase_uid, module_code, action, new_status, performed_by, reason, timestamp)
                        VALUES ($1, $2, 'grant', $3, $4, $5, NOW())
                        "#,
                    )
                    .bind(&assignment.firebase_uid)
                    .bind(&assignment.module_code)
                    .bind(serde_json::json!({"is_active": true, "expires_at": assignment.expires_at}))
                    .bind(&assignment.granted_by)
                    .bind(&assignment.granted_reason)
                    .execute(&mut *tx)
                    .await;
                },
                Err(e) => {
                    if let Err(rollback_err) = tx.rollback().await {
                        tracing::error!("Failed to rollback transaction: {}", rollback_err);
                    }
                    failed_assignments.push(BulkAssignmentError {
                        user_id: assignment.firebase_uid,
                        permission_profile_id: assignment.module_code,
                        error: e.to_string(),
                    });
                    return Err(DatabaseError::Query(format!("Assignment failed: {:?}", failed_assignments)));
                }
            }
        }
        
        tx.commit().await.map_err(DatabaseError::Connection)?;
        
        Ok(BulkAssignmentResult {
            successful_assignments,
            failed_assignments,
            total_processed: successful_assignments,
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