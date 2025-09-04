// Container Builder - Main orchestration for dependency injection
// Replaces the God Object with focused composition of modules

use std::sync::Arc;
use super::{DatabaseModule, ServicesModule, CacheModule};
use crate::app::ports::repositories::*;
// Removed admin module service import - using simple roles
use crate::infra::{
    db::diesel::DbPool,
    firebase_admin::FirebaseAdmin,
};
use crate::app::use_cases::{AuthUC, UserMgmtUC};
use crate::dom::services::PermissionService;
use crate::infra::services::permission_infrastructure::PermissionInfrastructureService;
use crate::app::services::PermissionApplicationService;

/// Refined AppContainer using focused modules instead of God Object pattern
#[derive(Clone)]
pub struct AppContainer {
    // Focused modules (internal architecture)
    database: DatabaseModule,
    services: ServicesModule,
    cache: CacheModule,
    
    // Public service access fields
    pub database_pool: Arc<DbPool>,
    pub db_pool: Arc<DbPool>,
    pub infra: crate::infra::InfraFactory,
    pub user_repo: Arc<dyn UserRepository>,
    pub user_permission_repo: Arc<dyn UserPermissionRepository>,
    pub session_repo: Arc<dyn SessionRepository>,
    pub audit_repo: Arc<dyn AuditRepository>,
    pub stock_repo: Arc<dyn StockRepository>,
    pub firebase_admin: Arc<FirebaseAdmin>,
    // Permission services (clean architecture)
    pub permission_service: Arc<PermissionService>,
    pub permission_infrastructure_service: Arc<PermissionInfrastructureService>,
    pub permission_application_service: Arc<PermissionApplicationService>,
    // Removed admin_module_service - using simple roles
    
    // FCM services
    pub fcm_service: Arc<crate::infra::services::FcmService>,
    pub fcm_topic_service: Arc<crate::infra::services::FcmTopicService>,
    
    // Notification repository
}

impl AppContainer {
    /// Create AppState with all dependencies from focused modules
    pub async fn create_app_state(&self) -> Result<crate::web::auth::AppState, Box<dyn std::error::Error + Send + Sync>> {
        // Create stub repositories for components not yet migrated to Diesel
        
        // Create stub usage repo
        let usage_repo = Arc::new(
            crate::infra::db::diesel::repos::StubUsageRepository::new()
        ) as Arc<dyn crate::app::ports::repositories::UsageRepository>;
        
        // Create stub dependencies for use cases
        let event_dispatcher = Arc::new(
            crate::infra::events::SimpleEventDispatcher::new()
        ) as Arc<dyn crate::app::ports::events::EventDispatcher>;
        
        let level_history_repo = Arc::new(
            crate::infra::db::level_history_repo::InMemoryLevelHistoryRepo::new()
        ) as Arc<dyn crate::app::ports::repositories::LevelHistoryRepository>;
        
        // Create use cases with focused module dependencies
        let auth_uc = Arc::new(AuthUC::new(
            self.database.user_repo.clone(),
            self.database.session_repo.clone(),
            self.services.firebase_admin.clone(),
            self.services.permission_application_service.clone(),
            self.services.refresh_token_service.clone(),
        ));
        
        let user_mgmt_uc = Arc::new(UserMgmtUC::new(
            self.database.user_repo.clone(),
            event_dispatcher,
            level_history_repo,
            self.services.permission_application_service.clone(),
        ));

        // Create stub module repo
        let module_repo = Arc::new(
            crate::infra::db::diesel::repos::StubModuleRepository::new()
        ) as Arc<dyn crate::app::ports::repositories::ModuleRepository>;

        Ok(crate::web::auth::AppState::new(
            auth_uc,
            user_mgmt_uc,
            self.database.session_repo.clone(),
            self.database.user_repo.clone(),
            self.database.user_permission_repo.clone(),
            self.database.audit_repo.clone(),
            module_repo,
            usage_repo,
            self.services.firebase_admin.clone(),
            // Removed admin_module_service parameter - using simple roles
            self.database.database_pool.clone(),
            self.cache.cache.clone(),
            None, // Removed security_cache 
            None, // Removed brute_force_service
            None, // Removed notification_port - will be re-implemented
            // Clean architecture services
            self.permission_application_service.clone(),
        ))
    }

    /// Simple permission system - replaced complex permissions with basic roles
    pub fn permission_systems(&self) -> Result<super::services_module::PermissionSystems, Box<dyn std::error::Error + Send + Sync>> {
        Ok(super::services_module::PermissionSystems::simple())
    }

    /// Simple permission system - returns default permissions
    pub fn get_default_permissions(&self) -> Result<Vec<String>, Box<dyn std::error::Error + Send + Sync>> {
        // Return basic user permissions as default
        use crate::auth::permissions::PermissionSets;
        Ok(PermissionSets::basic_user())
    }

    /// Simple audit stub - basic logging only
    pub fn get_simple_audit(&self) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
        // Simple system doesn't need complex audit
        Ok(true)
    }

    /// Get the cache system - delegates to cache module
    pub async fn cache(&self) -> Result<Arc<dyn crate::infra::cache::Cache>, Box<dyn std::error::Error + Send + Sync>> {
        Ok(self.cache.cache.clone())
    }

    /// Get the cache system - delegates to cache module
    pub async fn get_cache(&self) -> Result<Arc<dyn crate::infra::cache::Cache>, Box<dyn std::error::Error + Send + Sync>> {
        Ok(self.cache.cache.clone())
    }

    // Removed: FCM services - will be re-implemented
}

/// Builder for creating the refined AppContainer with focused modules
pub struct AppContainerBuilder {
    database_pool: Option<Arc<crate::infra::db::diesel::DbPool>>,
    database_url: Option<String>,
}

impl AppContainerBuilder {
    pub fn new() -> Self {
        Self {
            database_pool: None,
            database_url: None,
        }
    }
    
    pub fn with_database_pool(mut self, pool: Arc<crate::infra::db::diesel::DbPool>) -> Self {
        self.database_pool = Some(pool);
        self
    }
    
    pub fn with_database_url(mut self, database_url: String) -> Self {
        self.database_url = Some(database_url);
        self
    }
    
    pub async fn build(self) -> Result<AppContainer, Box<dyn std::error::Error + Send + Sync>> {
        // Create focused modules in sequence
        tracing::info!("🏗️ Building AppContainer with focused modules...");
        
        // 1. Database Module
        let database = match (self.database_pool, self.database_url) {
            (Some(pool), _) => DatabaseModule::new(pool).await?,
            (None, Some(url)) => DatabaseModule::from_url(&url).await?,
            (None, None) => DatabaseModule::from_env().await?,
        };
        
        // 2. Cache Module
        let cache = CacheModule::new().await?;
        
        // 3. Services Module
        let services = ServicesModule::new(
            database.database_pool.clone(),
            database.user_repo.clone(),
            database.user_permission_repo.clone(),
            cache.cache.clone(),
            database.refresh_token_repo.clone(),
            database.revoked_token_repo.clone(),
        ).await?;
        
        // 4. Legacy compatibility
        let infra = crate::infra::InfraFactory {
            database_backend: crate::infra::DatabaseBackend::PostgreSQL,
            diesel_pool: database.database_pool.clone(),
        };
        
        tracing::info!("✅ AppContainer built successfully with focused modules");
        
        Ok(AppContainer {
            // Legacy compatibility fields - expose module internals
            database_pool: database.database_pool.clone(),
            db_pool: database.database_pool.clone(),
            user_repo: database.user_repo.clone(),
            user_permission_repo: database.user_permission_repo.clone(),
            session_repo: database.session_repo.clone(),
            audit_repo: database.audit_repo.clone(),
            stock_repo: Arc::new(crate::infra::db::diesel::repos::StubStockRepository::new()) as Arc<dyn StockRepository>,
            firebase_admin: services.firebase_admin.clone(),
            // Permission services from services module
            permission_service: services.permission_service.clone(),
            permission_infrastructure_service: services.permission_infrastructure_service.clone(),
            permission_application_service: services.permission_application_service.clone(),
            // FCM services
            fcm_service: services.fcm_service.clone(),
            fcm_topic_service: services.fcm_topic_service.clone(),
            // Removed notification repository - using simple stubs
            // Removed admin_module_service field - using simple roles
            infra,
            
            // Store focused modules internally
            database,
            services,
            cache,
        })
    }
}

impl Default for AppContainerBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Convenience methods
impl AppContainer {
    /// Create AppContainer with default configuration using focused modules
    pub async fn new() -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        AppContainerBuilder::new().build().await
    }
}