// PostgreSQL repository implementations

use sqlx::PgPool;
use std::sync::Arc;

pub mod user_repo;
pub mod session_repo;
pub mod payment_repo;
pub mod stock_repo;
pub mod iam_repo;
pub mod audit_repo;
pub mod template_repo;
// TODO: Implement level_history_repo
// pub mod level_history_repo;

pub use user_repo::*;
pub use session_repo::*;
pub use payment_repo::*;
pub use stock_repo::*;
pub use iam_repo::*;
pub use audit_repo::*;
pub use template_repo::*;
// pub use level_history_repo::*;

/// Database configuration
#[derive(Debug, Clone)]
pub struct DatabaseConfig {
    pub host: String,
    pub port: u16,
    pub username: String,
    pub password: String,
    pub database: String,
    pub max_connections: u32,
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
                .unwrap_or_else(|_| "10".to_string())
                .parse()
                .unwrap_or(10),
        }
    }
}

/// Database connection pool
pub type DatabasePool = Arc<PgPool>;

/// Initialize database connection pool
pub async fn create_pool(_config: DatabaseConfig) -> Result<DatabasePool, sqlx::Error> {
    // Use DATABASE_URL from environment if available, fallback to constructed URL
    let database_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| {
        format!(
            "postgresql://{}:{}@{}:{}/{}",
            _config.username, _config.password, _config.host, _config.port, _config.database
        )
    });

    let pool = PgPool::connect(&database_url).await?;
    
    // Run migrations (commented out since we've already run them manually)
    // sqlx::migrate!("./migrations").run(&pool).await?;
    
    Ok(Arc::new(pool))
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