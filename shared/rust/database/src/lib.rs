use async_trait::async_trait;
use sqlx::{postgres::PgPoolOptions, PgPool};
use std::sync::Arc;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum DatabaseError {
    #[error("SQLx error: {0}")]
    Sqlx(#[from] sqlx::Error),
    #[error("Migration error: {0}")]
    Migration(String),
    #[error("Not found")]
    NotFound,
}

pub type Result<T> = std::result::Result<T, DatabaseError>;

pub struct Database {
    pool: Arc<PgPool>,
}

impl Database {
    pub async fn new(database_url: &str, max_connections: u32) -> Result<Self> {
        let pool = PgPoolOptions::new()
            .max_connections(max_connections)
            .connect(database_url)
            .await?;

        Ok(Self { pool: Arc::new(pool) })
    }

    pub fn pool(&self) -> &PgPool {
        &self.pool
    }

    pub async fn run_migrations(&self, migrations_dir: &str) -> Result<()> {
        // Use sqlx-cli for migrations in production, this is a placeholder
        // sqlx::migrate! requires a literal path at compile time
        tracing::info!("Migrations directory configured: {}", migrations_dir);
        Ok(())
    }
}

impl Clone for Database {
    fn clone(&self) -> Self {
        Self { pool: Arc::clone(&self.pool) }
    }
}

#[async_trait]
pub trait Repository<T, ID> {
    async fn find_by_id(&self, id: ID) -> Result<Option<T>>;
    async fn find_all(&self, limit: i64, offset: i64) -> Result<Vec<T>>;
    async fn count(&self) -> Result<i64>;
    async fn delete(&self, id: ID) -> Result<bool>;
}

#[async_trait]
pub trait Transactional {
    async fn begin(&self) -> Result<sqlx::Transaction<'static, sqlx::Postgres>>;
}
