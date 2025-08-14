// Simplified dependency container with builder pattern
// Only includes essential services actually used in the application

use std::sync::Arc;
use crate::app::ports::repositories::*;
use crate::dom::ports::NotificationPort;
use crate::dom::services::feature_expiration::FeatureExpirationService;
use crate::infra::{
    db::{PostgresUserRepo, PostgresAuditRepo, PostgresPermissionProfileRepo, DatabasePool},
    firebase_admin::FirebaseAdmin,
    services::{notification::InMemoryNotificationService, NotificationPortAdapter},
};

/// Simplified dependency container with only essential services
pub struct AppContainer {
    pub infra: InfraBuilder,
    
    // Essential repositories only
    pub user_repo: Arc<dyn UserRepo>,
    pub audit_repo: Arc<dyn AuditRepo>,
    pub permission_profile_repo: Arc<dyn PermissionProfileRepo>,
    
    // Essential services only  
    pub firebase_admin: Arc<FirebaseAdmin>,
    pub feature_expiration_service: Arc<dyn FeatureExpirationService>,
}

/// Builder for creating the dependency container
pub struct AppContainerBuilder {
    postgres_pool: Option<DatabasePool>,
}

impl AppContainerBuilder {
    pub fn new() -> Self {
        Self {
            postgres_pool: None,
        }
    }
    
    pub fn with_postgres_pool(mut self, pool: DatabasePool) -> Self {
        self.postgres_pool = Some(pool);
        self
    }
    
    pub async fn build(self) -> Result<AppContainer, Box<dyn std::error::Error>> {
        let postgres_pool = self.postgres_pool
            .ok_or("PostgreSQL pool is required")?;
        
        // Create infrastructure factory
        let infra = InfraBuilder::new(postgres_pool.clone());
        
        // Create repositories
        let user_repo = infra.create_user_repo();
        let audit_repo = infra.create_audit_repo();
        let permission_profile_repo = infra.create_permission_profile_repo();
        
        // Create services
        let firebase_admin = infra.create_firebase_admin().await?;
        let feature_expiration_service = infra.create_feature_expiration_service(user_repo.clone());
        
        Ok(AppContainer {
            infra,
            user_repo,
            audit_repo,
            permission_profile_repo,
            firebase_admin,
            feature_expiration_service,
        })
    }
}

impl AppContainer {
    pub async fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let postgres_pool = crate::infra::db::postgres::create_pool(
            crate::infra::db::postgres::DatabaseConfig::default()
        ).await?;
        
        AppContainerBuilder::new()
            .with_postgres_pool(postgres_pool)
            .build()
            .await
    }
}

/// Simplified infrastructure builder for essential services
pub struct InfraBuilder {
    pub postgres_pool: DatabasePool,
}

impl InfraBuilder {
    pub fn new(postgres_pool: DatabasePool) -> Self {
        Self { postgres_pool }
    }
    
    // Essential repository factories
    pub fn create_user_repo(&self) -> Arc<dyn UserRepo> {
        Arc::new(PostgresUserRepo::new(self.postgres_pool.clone()))
    }

    pub fn create_audit_repo(&self) -> Arc<dyn AuditRepo> {
        Arc::new(PostgresAuditRepo::new((*self.postgres_pool).clone()))
    }

    pub fn create_permission_profile_repo(&self) -> Arc<dyn PermissionProfileRepo> {
        Arc::new(PostgresPermissionProfileRepo::new((*self.postgres_pool).clone()))
    }

    // Essential service factories
    pub async fn create_firebase_admin(&self) -> Result<Arc<FirebaseAdmin>, Box<dyn std::error::Error>> {
        Ok(Arc::new(FirebaseAdmin::new().await?))
    }

    pub fn create_feature_expiration_service(&self, user_repo: Arc<dyn UserRepo>) -> Arc<dyn FeatureExpirationService> {
        use crate::dom::services::feature_expiration::{FeatureExpirationServiceImpl, ExpirationConfig};
        
        // Create a minimal notification service
        let notification_service = Arc::new(InMemoryNotificationService::new());
        let notification_port: Arc<dyn NotificationPort> = Arc::new(
            NotificationPortAdapter::new(notification_service)
        );
        
        Arc::new(FeatureExpirationServiceImpl::new(
            user_repo,
            notification_port,
            Some(ExpirationConfig::default()),
        ))
    }
}