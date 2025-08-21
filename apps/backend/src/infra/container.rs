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
        tracing::info!("🔧 Creating repository layer...");
        let user_repo = Arc::new(PostgresUserRepo::new(database_pool.clone())) as Arc<dyn UserRepo>;
        let audit_repo = Arc::new(PostgresAuditRepo::new((*database_pool).clone())) as Arc<dyn AuditRepo>;
        let permission_profile_repo = Arc::new(PostgresPermissionProfileRepo::new((*database_pool).clone())) as Arc<dyn PermissionProfileRepo>;
        
        // Create external services
        tracing::info!("🔧 Creating Firebase Admin service...");
        let firebase_admin = Arc::new(FirebaseAdmin::new().await.map_err(|e| {
            tracing::error!("❌ Firebase Admin creation failed: {}", e);
            e
        })?);
        
        // Create domain services with their dependencies
        tracing::info!("🔧 Creating notification services...");
        let notification_service = Arc::new(InMemoryNotificationService::new());
        let notification_port: Arc<dyn NotificationPort> = Arc::new(
            NotificationPortAdapter::new(notification_service)
        );
        
        tracing::info!("🔧 Creating feature expiration service...");
        let feature_expiration_service = {
            use crate::dom::services::feature_expiration::{FeatureExpirationServiceImpl, ExpirationConfig};
            Arc::new(FeatureExpirationServiceImpl::new(
                user_repo.clone(),
                notification_port,
                Some(ExpirationConfig::default()),
            )) as Arc<dyn FeatureExpirationService>
        };
        
        tracing::info!("🔧 Creating admin module service...");
        let admin_module_service = Arc::new(AdminModuleService::new((*database_pool).clone()));
        
        // Create legacy infrastructure factory for backward compatibility
        tracing::info!("🔧 Creating infrastructure factory...");
        let infra = crate::infra::InfraFactory {
            database_backend: crate::infra::DatabaseBackend::PostgreSQL,
            postgres_pool: database_pool.clone(),
        };
        
        tracing::info!("✅ AppContainer build completed successfully");
        
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
    /// Create AppState with all dependencies from this container
    pub fn create_app_state(&self) -> crate::web::auth::AppState {
        use crate::app::use_cases::{AuthUC, UserMgmtUC};
        use crate::infra::db::{PostgresSessRepo, PostgresIamRepo, PostgresTemporaryPermissionRepo, PostgresModuleRepository, StubUsageRepo};
        use crate::infra::events::SimpleEventDispatcher;
        use crate::infra::db::level_history_repo::InMemoryLevelHistoryRepo;
        
        // Create additional repositories first
        let session_repo = Arc::new(PostgresSessRepo::new(self.database_pool.clone())) as Arc<dyn crate::app::ports::repositories::SessRepo>;
        let iam_repo = Arc::new(PostgresIamRepo::new((*self.database_pool).clone())) as Arc<dyn crate::app::ports::repositories::IamRepo>;
        let temporary_permission_repo = Arc::new(PostgresTemporaryPermissionRepo::new((*self.database_pool).clone())) as Arc<dyn crate::app::ports::repositories::TemporaryPermissionRepo>;
        let module_repo = Arc::new(PostgresModuleRepository::new((*self.database_pool).clone())) as Arc<dyn crate::app::ports::repositories::ModuleRepo>;
        let usage_repo = Arc::new(StubUsageRepo::new()) as Arc<dyn crate::app::ports::repositories::UsageRepo>;
        
        // Create stub dependencies for use cases
        let event_dispatcher = Arc::new(SimpleEventDispatcher::new()) as Arc<dyn crate::app::ports::events::EventDispatcher>;
        let level_history_repo = Arc::new(InMemoryLevelHistoryRepo::new()) as Arc<dyn crate::app::ports::repositories::LevelHistoryRepo>;
        
        // Create use cases with dependencies
        let auth_uc = Arc::new(AuthUC::new(
            self.user_repo.clone(),
            session_repo.clone(),
            self.firebase_admin.clone(),
        ));
        let user_mgmt_uc = Arc::new(UserMgmtUC::new(
            self.user_repo.clone(),
            event_dispatcher,
            level_history_repo,
        ));
        
        crate::web::auth::AppState::new(
            auth_uc,
            user_mgmt_uc,
            session_repo,
            self.user_repo.clone(),
            iam_repo,
            self.audit_repo.clone(),
            self.permission_profile_repo.clone(),
            temporary_permission_repo,
            module_repo,
            usage_repo,
            self.firebase_admin.clone(),
            self.admin_module_service.clone(),
            self.feature_expiration_service.clone(),
        )
    }

    /// Convenience constructor that creates a container with default configuration
    pub async fn new() -> Result<Self, Box<dyn std::error::Error>> {
        tracing::info!("🔧 Creating database pool...");
        let database_pool = crate::infra::db::postgres::create_pool(
            crate::infra::db::postgres::DatabaseConfig::default()
        ).await.map_err(|e| {
            tracing::error!("❌ Database pool creation failed: {}", e);
            e
        })?;
        
        tracing::info!("✅ Database pool created successfully");
        tracing::info!("🔧 Building AppContainer...");
        
        AppContainerBuilder::new()
            .with_database_pool(database_pool)
            .build()
            .await
    }
}

