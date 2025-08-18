// Clean dependency injection container with builder pattern
// Consolidates all dependency creation in a single, clear pattern

use std::sync::Arc;
use crate::app::ports::repositories::*;
use crate::dom::ports::NotificationPort;
use crate::dom::services::feature_expiration::FeatureExpirationService;
use crate::dom::services::admin_module_service::AdminModuleService;
use crate::infra::{
    db::{PostgresUserRepo, PostgresAuditRepo, PostgresPermissionProfileRepo, DatabasePool},
    firebase_admin::FirebaseAdmin,
    services::{notification::InMemoryNotificationService, NotificationPortAdapter},
};

/// Application dependency container with essential services
pub struct AppContainer {
    // Database connection pool
    pub database_pool: DatabasePool,
    
    // Legacy infrastructure factory for backward compatibility
    pub infra: crate::infra::InfraFactory,
    
    // Repository layer
    pub user_repo: Arc<dyn UserRepo>,
    pub audit_repo: Arc<dyn AuditRepo>,
    pub permission_profile_repo: Arc<dyn PermissionProfileRepo>,
    
    // Service layer
    pub firebase_admin: Arc<FirebaseAdmin>,
    pub feature_expiration_service: Arc<dyn FeatureExpirationService>,
    pub admin_module_service: Arc<AdminModuleService>,
}

/// Builder for creating the dependency container with validation
pub struct AppContainerBuilder {
    database_pool: Option<DatabasePool>,
}

impl AppContainerBuilder {
    pub fn new() -> Self {
        Self {
            database_pool: None,
        }
    }
    
    pub fn with_database_pool(mut self, pool: DatabasePool) -> Self {
        self.database_pool = Some(pool);
        self
    }
    
    pub async fn build(self) -> Result<AppContainer, Box<dyn std::error::Error>> {
        let database_pool = self.database_pool
            .ok_or("Database pool is required")?;
        
        // Create repositories
        let user_repo = Arc::new(PostgresUserRepo::new(database_pool.clone())) as Arc<dyn UserRepo>;
        let audit_repo = Arc::new(PostgresAuditRepo::new((*database_pool).clone())) as Arc<dyn AuditRepo>;
        let permission_profile_repo = Arc::new(PostgresPermissionProfileRepo::new((*database_pool).clone())) as Arc<dyn PermissionProfileRepo>;
        
        // Create external services
        let firebase_admin = Arc::new(FirebaseAdmin::new().await?);
        
        // Create domain services with their dependencies
        let notification_service = Arc::new(InMemoryNotificationService::new());
        let notification_port: Arc<dyn NotificationPort> = Arc::new(
            NotificationPortAdapter::new(notification_service)
        );
        
        let feature_expiration_service = {
            use crate::dom::services::feature_expiration::{FeatureExpirationServiceImpl, ExpirationConfig};
            Arc::new(FeatureExpirationServiceImpl::new(
                user_repo.clone(),
                notification_port,
                Some(ExpirationConfig::default()),
            )) as Arc<dyn FeatureExpirationService>
        };
        
        let admin_module_service = Arc::new(AdminModuleService::new((*database_pool).clone()));
        
        // Create legacy infrastructure factory for backward compatibility
        let infra = crate::infra::InfraFactory {
            database_backend: crate::infra::DatabaseBackend::PostgreSQL,
            postgres_pool: database_pool.clone(),
        };
        
        Ok(AppContainer {
            database_pool,
            infra,
            user_repo,
            audit_repo,
            permission_profile_repo,
            firebase_admin,
            feature_expiration_service,
            admin_module_service,
        })
    }
}

impl AppContainer {
    /// Convenience constructor that creates a container with default configuration
    pub async fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let database_pool = crate::infra::db::postgres::create_pool(
            crate::infra::db::postgres::DatabaseConfig::default()
        ).await?;
        
        AppContainerBuilder::new()
            .with_database_pool(database_pool)
            .build()
            .await
    }
}

