// Infrastructure layer implementations

pub mod cache;
pub mod db;
pub mod services;
pub mod events;
pub mod firebase_admin;
pub mod jobs;
pub mod container;

// Re-export essential implementations only
pub use db::{PostgresUserRepo, PostgresAuditRepo, PostgresPermissionProfileRepo, DatabasePool, create_pool, DatabaseConfig};
pub use container::{AppContainer, AppContainerBuilder, InfraBuilder};
pub use services::{MockEmailService, notification::*};
pub use events::SimpleEventDispatcher;
pub use firebase_admin::FirebaseAdmin;
pub use jobs::{JobScheduler, ExpirationChecker, NotificationService as JobNotificationService};


/// Database backend type
#[derive(Debug, Clone)]
pub enum DatabaseBackend {
    PostgreSQL,
}

impl Default for DatabaseBackend {
    fn default() -> Self {
        Self::PostgreSQL
    }
}

/// Minimal infrastructure factory for backward compatibility
pub struct InfraFactory {
    pub database_backend: DatabaseBackend,
    pub postgres_pool: DatabasePool,
}

impl InfraFactory {
    pub async fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let database_backend = DatabaseBackend::PostgreSQL;
        let config = crate::infra::db::postgres::DatabaseConfig::default();
        let postgres_pool = crate::infra::db::postgres::create_pool(config).await
            .map_err(|e| Box::new(e) as Box<dyn std::error::Error>)?;
        
        Ok(Self {
            database_backend,
            postgres_pool,
        })
    }

    pub fn from_env() -> Result<Self, Box<dyn std::error::Error>> {
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(
                Self::new()
            )
        })
    }
}