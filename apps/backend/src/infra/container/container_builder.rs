// Container Builder - Main orchestration for dependency injection
// Replaces the God Object with focused composition of modules

use std::sync::Arc;
use super::{DatabaseModule, ServicesModule, CacheModule};
use crate::app::ports::repositories::*;
use crate::dom::services::feature_expiration::FeatureExpirationService;
use crate::dom::services::admin_module_service::AdminModuleService;
use crate::infra::{
    db::diesel::DbPool,
    firebase_admin::FirebaseAdmin,
};
use crate::app::use_cases::{AuthUC, UserMgmtUC};
use crate::security::brute_force_integration::BruteForceIntegrationFactory;

/// Refined AppContainer using focused modules instead of God Object pattern
#[derive(Clone)]
pub struct AppContainer {
    // Focused modules (internal architecture)
    database: DatabaseModule,
    services: ServicesModule,
    cache: CacheModule,
    
    // Legacy compatibility fields - exposed for backward compatibility
    pub database_pool: Arc<DbPool>,
    pub db_pool: Arc<DbPool>,
    pub infra: crate::infra::InfraFactory,
    pub user_repo: Arc<dyn UserRepo>,
    pub session_repo: Arc<dyn SessRepo>,
    pub audit_repo: Arc<dyn AuditRepo>,
    pub stock_repo: Arc<dyn StockRepo>,
    pub iam_repo: Arc<dyn IamRepo>,
    pub permission_profile_repo: Arc<dyn PermissionProfileRepo>,
    pub firebase_admin: Arc<FirebaseAdmin>,
    pub feature_expiration_service: Arc<dyn FeatureExpirationService>,
    pub admin_module_service: Arc<AdminModuleService>,
}

impl AppContainer {
    /// Create AppState with all dependencies from focused modules
    pub async fn create_app_state(&self) -> Result<crate::web::auth::AppState, Box<dyn std::error::Error + Send + Sync>> {
        // Create stub repositories for components not yet migrated to Diesel
        let (temporary_permission_repo, module_repo) = self.database.create_stub_repos();
        
        // Create stub usage repo
        let usage_repo = Arc::new(
            crate::infra::db::diesel::repos::StubUsageRepo::new()
        ) as Arc<dyn crate::app::ports::repositories::UsageRepo>;
        
        // Create stub dependencies for use cases
        let event_dispatcher = Arc::new(
            crate::infra::events::SimpleEventDispatcher::new()
        ) as Arc<dyn crate::app::ports::events::EventDispatcher>;
        
        let level_history_repo = Arc::new(
            crate::infra::db::level_history_repo::InMemoryLevelHistoryRepo::new()
        ) as Arc<dyn crate::app::ports::repositories::LevelHistoryRepo>;
        
        // Create use cases with focused module dependencies
        let auth_uc = Arc::new(AuthUC::new(
            self.database.user_repo.clone(),
            self.database.session_repo.clone(),
            self.services.firebase_admin.clone(),
        ));
        
        let user_mgmt_uc = Arc::new(UserMgmtUC::new(
            self.database.user_repo.clone(),
            event_dispatcher,
            level_history_repo,
        ));

        // Create initial AppState for brute force factory
        let temp_app_state = crate::web::auth::AppState::new(
            auth_uc.clone(),
            user_mgmt_uc.clone(),
            self.database.session_repo.clone(),
            self.database.user_repo.clone(),
            self.database.iam_repo.clone(),
            self.database.audit_repo.clone(),
            self.database.permission_profile_repo.clone(),
            temporary_permission_repo.clone(),
            module_repo.clone(),
            usage_repo.clone(),
            self.services.firebase_admin.clone(),
            self.services.admin_module_service.clone(),
            self.services.feature_expiration_service.clone(),
            self.database.database_pool.clone(),
            self.cache.cache.clone(),
            self.cache.security_cache.clone(),
            None,
            self.services.notification_service.clone(),
        );

        // Initialize brute force protection service
        tracing::info!("🔧 Creating brute force protection service...");
        let brute_force_service = match BruteForceIntegrationFactory::create(&temp_app_state).await {
            Ok(service) => {
                tracing::info!("✅ Brute force protection service initialized");
                Some(service)
            },
            Err(e) => {
                tracing::warn!("⚠️ Brute force protection service failed to initialize: {}", e);
                tracing::warn!("⚠️ Continuing without brute force protection");
                None
            }
        };

        Ok(crate::web::auth::AppState::new(
            auth_uc,
            user_mgmt_uc,
            self.database.session_repo.clone(),
            self.database.user_repo.clone(),
            self.database.iam_repo.clone(),
            self.database.audit_repo.clone(),
            self.database.permission_profile_repo.clone(),
            temporary_permission_repo,
            module_repo,
            usage_repo,
            self.services.firebase_admin.clone(),
            self.services.admin_module_service.clone(),
            self.services.feature_expiration_service.clone(),
            self.database.database_pool.clone(),
            self.cache.cache.clone(),
            self.cache.security_cache.clone(),
            brute_force_service,
            self.services.notification_service.clone(),
        ))
    }

    /// Simple permission system - replaced complex permissions with basic roles
    pub fn permission_systems(&self) -> Result<super::services_module::PermissionSystems, Box<dyn std::error::Error + Send + Sync>> {
        Ok(super::services_module::PermissionSystems::simple())
    }

    /// Simple role system - no complex permission systems needed
    pub fn get_role_checker(&self) -> Result<Arc<crate::auth::roles::Role>, Box<dyn std::error::Error + Send + Sync>> {
        // Simple stub - roles are checked directly via check_feature_access
        Ok(Arc::new(crate::auth::roles::Role::Guest))
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
            cache.cache.clone(),
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
            session_repo: database.session_repo.clone(),
            audit_repo: database.audit_repo.clone(),
            stock_repo: database.stock_repo.clone(),
            iam_repo: database.iam_repo.clone(),
            permission_profile_repo: database.permission_profile_repo.clone(),
            firebase_admin: services.firebase_admin.clone(),
            feature_expiration_service: services.feature_expiration_service.clone(),
            admin_module_service: services.admin_module_service.clone(),
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

/// Convenience methods for backward compatibility
impl AppContainer {
    /// Create AppContainer with default configuration using focused modules
    pub async fn new() -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        AppContainerBuilder::new().build().await
    }
}