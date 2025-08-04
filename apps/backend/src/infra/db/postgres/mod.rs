// PostgreSQL repository implementations

use sqlx::PgPool;
use std::sync::Arc;
use serde_json;

pub mod user_repo;
// pub mod user_repo_soft_delete;
pub mod session_repo;
pub mod payment_repo;
pub mod stock_repo;
pub mod iam_repo;
pub mod audit_repo;
pub mod permission_profile_repo;
pub mod permission_assignment_repo;
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
pub use permission_assignment_repo::*;
// pub use level_history_repo::*;

/// Enhanced database configuration
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
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        Self {
            host: std::env::var("DATABASE_HOST").unwrap_or_else(|_| "localhost".to_string()),
            port: std::env::var("DATABASE_PORT")
                .unwrap_or_else(|_| "5432".to_string())
                .parse()
                .unwrap_or(5432),
            username: std::env::var("DATABASE_USERNAME").unwrap_or_else(|_| "postgres".to_string()),
            password: std::env::var("DATABASE_PASSWORD").unwrap_or_else(|_| "password".to_string()),
            database: std::env::var("DATABASE_NAME").unwrap_or_else(|_| "epsx".to_string()),
            max_connections: std::env::var("DATABASE_MAX_CONNECTIONS")
                .unwrap_or_else(|_| "20".to_string())
                .parse()
                .unwrap_or(20),
            min_connections: std::env::var("DATABASE_MIN_CONNECTIONS")
                .unwrap_or_else(|_| "5".to_string())
                .parse()
                .unwrap_or(5),
            migration_source: std::env::var("DATABASE_MIGRATION_SOURCE")
                .unwrap_or_else(|_| "./migrations".to_string()),
            ssl_mode: std::env::var("DATABASE_SSL_MODE")
                .unwrap_or_else(|_| "prefer".to_string()),
            permission_pool_size: std::env::var("PERMISSION_POOL_SIZE")
                .unwrap_or_else(|_| "10".to_string())
                .parse()
                .unwrap_or(10),
            enable_statement_logging: std::env::var("DATABASE_STATEMENT_LOGGING")
                .unwrap_or_else(|_| "false".to_string())
                .parse()
                .unwrap_or(false),
            query_timeout_seconds: std::env::var("DATABASE_QUERY_TIMEOUT")
                .unwrap_or_else(|_| "30".to_string())
                .parse()
                .unwrap_or(30),
        }
    }
}

/// Database connection pool
pub type DatabasePool = Arc<PgPool>;

/// Check if running in development mode
fn is_development_mode() -> bool {
    // Check various environment indicators for development mode
    std::env::var("NODE_ENV").map(|v| v == "development").unwrap_or(false) ||
    std::env::var("RUST_ENV").map(|v| v == "development").unwrap_or(false) ||
    std::env::var("ENV").map(|v| v == "dev" || v == "development").unwrap_or(false) ||
    std::env::var("ENVIRONMENT").map(|v| v == "dev" || v == "development").unwrap_or(false) ||
    // If no explicit environment set, assume development (safer for local dev)
    (!std::env::var("NODE_ENV").is_ok() && 
     !std::env::var("RUST_ENV").is_ok() && 
     !std::env::var("ENV").is_ok() && 
     !std::env::var("ENVIRONMENT").is_ok())
}

/// Initialize optimized database connection pool with permission query optimization
pub async fn create_pool(config: DatabaseConfig) -> Result<DatabasePool, sqlx::Error> {
    use sqlx::postgres::PgPoolOptions;
    use std::time::Duration;
    
    // Use DATABASE_URL from environment if available, fallback to constructed URL
    let database_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| {
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
        .acquire_timeout(Duration::from_secs(config.query_timeout_seconds))
        .idle_timeout(Duration::from_secs(300))
        .max_lifetime(Duration::from_secs(1800))
        .test_before_acquire(true)
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
        use crate::infra::db::MigrationRunner;
        use tracing::{info, warn, error};
        
        info!("Development mode detected - running auto-migration");
        let runner = MigrationRunner::new(pool.clone(), config.migration_source);
        
        match runner.migrate().await {
            Ok(count) => {
                if count > 0 {
                    info!("✅ Auto-applied {} migrations successfully", count);
                } else {
                    info!("✅ Database is up to date");
                }
                
                // Create permission query performance indexes
                match create_permission_indexes(&pool).await {
                    Ok(_) => info!("✅ Permission indexes created/verified"),
                    Err(e) => warn!("⚠️  Permission index creation failed: {}", e),
                }
            },
            Err(e) => {
                let error_msg = e.to_string();
                if error_msg.contains("already exists") {
                    warn!("⚠️  Some database objects already exist - this is normal after manual migrations");
                    info!("✅ Database schema appears to be current");
                    
                    // Still try to create indexes even if migration failed
                    match create_permission_indexes(&pool).await {
                        Ok(_) => info!("✅ Permission indexes created/verified"),
                        Err(e) => warn!("⚠️  Permission index creation failed: {}", e),
                    }
                } else {
                    error!("❌ Auto-migration failed: {}", e);
                    warn!("💡 Consider running migrations manually: cargo run --bin migrate up");
                }
                // Don't fail pool creation on migration error in development
            }
        }
    }
    
    Ok(Arc::new(pool))
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
        // User permission assignment indexes
        ("idx_user_permission_assignments_user_id", 
         "CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_user_permission_assignments_user_id 
          ON user_permission_assignments(user_id) WHERE is_active = true"),
        
        ("idx_user_permission_assignments_profile_id",
         "CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_user_permission_assignments_profile_id 
          ON user_permission_assignments(permission_profile_id) WHERE is_active = true"),
        
        ("idx_user_permission_assignments_expires_at",
         "CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_user_permission_assignments_expires_at 
          ON user_permission_assignments(expires_at) WHERE expires_at IS NOT NULL AND is_active = true"),
        
        // Permission profile indexes for auto-assignment
        ("idx_permission_profiles_auto_assignment",
         "CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_permission_profiles_auto_assignment 
          ON permission_profiles USING GIN (auto_assignment_rules) WHERE is_active = true"),
        
        ("idx_permission_profiles_name_active",
         "CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_permission_profiles_name_active 
          ON permission_profiles(name) WHERE is_active = true"),
        
        // Permission assignment audit indexes
        ("idx_permission_assignment_audit_user_id",
         "CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_permission_assignment_audit_user_id 
          ON permission_assignment_audit(user_id)"),
        
        ("idx_permission_assignment_audit_timestamp",
         "CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_permission_assignment_audit_timestamp 
          ON permission_assignment_audit(created_at DESC)"),
        
        // User lookup indexes for permission resolution
        ("idx_users_email_active",
         "CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_users_email_active 
          ON users(email) WHERE is_active = true"),
        
        ("idx_users_firebase_uid",
         "CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_users_firebase_uid 
          ON users(firebase_uid) WHERE firebase_uid IS NOT NULL"),
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
    
    /// Execute bulk user permission assignments with rollback on any failure
    pub async fn bulk_assign_permissions(&self, assignments: Vec<BulkPermissionAssignment>) -> Result<BulkAssignmentResult, DatabaseError> {
        let mut tx = self.pool.begin().await.map_err(DatabaseError::Connection)?;
        let mut successful_assignments = 0;
        let mut failed_assignments = Vec::new();
        
        for assignment in assignments {
            let user_uuid = uuid::Uuid::parse_str(&assignment.user_id)
                .map_err(|e| DatabaseError::Query(format!("Invalid user UUID: {}", e)))?;
            let profile_uuid = uuid::Uuid::parse_str(&assignment.permission_profile_id)
                .map_err(|e| DatabaseError::Query(format!("Invalid profile UUID: {}", e)))?;
            let assigned_by_uuid = uuid::Uuid::parse_str(&assignment.assigned_by)
                .map_err(|e| DatabaseError::Query(format!("Invalid assigned_by UUID: {}", e)))?;
            
            let result = sqlx::query(
                r#"
                INSERT INTO admin_permission_profile_assignments 
                (user_id, permission_profile_id, assigned_by, expires_at, assignment_reason, assignment_type, status, created_at)
                VALUES ($1, $2, $3, $4, $5, 'promotional', 'active', NOW())
                ON CONFLICT (user_id, permission_profile_id) 
                DO UPDATE SET 
                    expires_at = EXCLUDED.expires_at,
                    assignment_reason = EXCLUDED.assignment_reason
                "#,
            )
            .bind(user_uuid)
            .bind(profile_uuid)
            .bind(assigned_by_uuid)
            .bind(assignment.expires_at.map(|dt| dt.naive_utc()))
            .bind(assignment.reason.as_deref())
            .execute(&mut *tx)
            .await;
            
            match result {
                Ok(_) => {
                    successful_assignments += 1;
                    
                    // Log the assignment
                    let _audit_result = sqlx::query(
                        r#"
                        INSERT INTO assignment_audit_log 
                        (assignment_id, action, performed_by, details, timestamp)
                        VALUES ((SELECT id FROM admin_permission_profile_assignments WHERE user_id = $1 AND permission_profile_id = $2), 'assign', $3, $4, NOW())
                        "#,
                    )
                    .bind(user_uuid)
                    .bind(profile_uuid)
                    .bind(assigned_by_uuid)
                    .bind(serde_json::json!(assignment.reason.as_deref().unwrap_or("bulk_assignment")))
                    .execute(&mut *tx)
                    .await;
                },
                Err(e) => {
                    if let Err(rollback_err) = tx.rollback().await {
                        tracing::error!("Failed to rollback transaction: {}", rollback_err);
                    }
                    failed_assignments.push(BulkAssignmentError {
                        user_id: assignment.user_id,
                        permission_profile_id: assignment.permission_profile_id,
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

/// Bulk permission assignment request
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