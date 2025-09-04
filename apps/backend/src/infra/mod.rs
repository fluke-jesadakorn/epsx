// Infrastructure layer implementations

pub mod cache;
pub mod db;
pub mod services;
pub mod events;
pub mod firebase_admin;
pub mod firebase;  // New focused modules architecture
pub mod oidc;      // OIDC Bearer authentication system
// pub mod jobs; // Module not implemented yet
pub mod container;

// Re-export essential implementations only
pub use db::{DbPool, create_diesel_pool, DieselUserRepository, DieselAuditRepository, DieselSessionRepository};
pub use container::{AppContainer, AppContainerBuilder};
pub use services::{MockEmailService};
pub use events::SimpleEventDispatcher;
pub use firebase_admin::FirebaseAdmin;
pub use oidc::{OIDCService, OIDCTokens, TokenStore};
pub use oidc::middleware::BearerAuthState;
// pub use jobs::{NotificationService as JobNotificationService}; // Module not implemented yet


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

/// Minimal infrastructure factory for Diesel migration (stub)
#[derive(Clone)]
pub struct InfraFactory {
    pub database_backend: DatabaseBackend,
    pub diesel_pool: std::sync::Arc<DbPool>,
}

impl InfraFactory {
    pub async fn new() -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let database_backend = DatabaseBackend::PostgreSQL;
        let database_url = std::env::var("DATABASE_URL")
            .unwrap_or_else(|_| "postgresql://localhost/epsx".to_string());
        let diesel_pool = create_diesel_pool(&database_url).await?;
        
        Ok(Self {
            database_backend,
            diesel_pool: std::sync::Arc::new(diesel_pool),
        })
    }

    pub fn from_env() -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(
                Self::new()
            )
        })
    }

}