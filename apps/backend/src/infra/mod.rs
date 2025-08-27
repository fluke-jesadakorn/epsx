// Infrastructure layer implementations

pub mod cache;
pub mod db;
pub mod services;
pub mod events;
pub mod firebase_admin;
pub mod firebase;  // New focused modules architecture
// pub mod jobs; // Module not implemented yet
pub mod container;

// Re-export essential implementations only
pub use db::{DbPool, create_diesel_pool, DieselUserRepository, DieselAuditRepository, DieselSessionRepository};
pub use container::{AppContainer, AppContainerBuilder};
pub use services::{MockEmailService, notification_service::*};
pub use events::SimpleEventDispatcher;
pub use firebase_admin::FirebaseAdmin;
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

    /// Create EPS ranking service for analytics (stub implementation)
    pub fn create_eps_ranking_service(&self) -> std::sync::Arc<crate::dom::services::eps_ranking_service::EPSRankingService> {
        // TODO: Implement EPS ranking service with Diesel
        tracing::info!("Creating EPS ranking service - using stub implementation");
        let eps_repo = self.create_eps_repo();
        std::sync::Arc::new(crate::dom::services::eps_ranking_service::EPSRankingService::new(eps_repo))
    }

    /// Create EPS repository for analytics with cache
    pub fn create_eps_repo_with_cache(
        &self, 
        cache: std::sync::Arc<dyn crate::infra::cache::Cache>
    ) -> std::sync::Arc<dyn crate::dom::services::eps_ranking_service::EPSRepository> {
        tracing::info!("Creating EPS repository with cache - using PostgreSQL + Redis/InMemory implementation");
        std::sync::Arc::new(crate::infra::db::diesel::repos::DieselEPSRepository::new(
            self.diesel_pool.clone(), 
            cache
        ))
    }
    
    /// Create EPS repository for analytics (backwards compatibility)
    pub fn create_eps_repo(&self) -> std::sync::Arc<dyn crate::dom::services::eps_ranking_service::EPSRepository> {
        tracing::info!("Creating EPS repository - using PostgreSQL implementation with in-memory cache fallback");
        let cache = std::sync::Arc::new(crate::infra::cache::memory_cache::InMemoryCache::new(
            crate::infra::cache::CacheConfig::default()
        )) as std::sync::Arc<dyn crate::infra::cache::Cache>;
        
        std::sync::Arc::new(crate::infra::db::diesel::repos::DieselEPSRepository::new(
            self.diesel_pool.clone(), 
            cache
        ))
    }
}