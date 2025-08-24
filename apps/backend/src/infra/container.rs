// Clean dependency injection container with builder pattern
// Consolidates all dependency creation in a single, clear pattern

use std::sync::Arc;
use crate::app::ports::repositories::*;
use crate::dom::ports::NotificationPort;
use crate::dom::services::feature_expiration::FeatureExpirationService;
use crate::dom::services::admin_module_service::AdminModuleService;
use crate::infra::{
    db::diesel::{
        DbPool, create_pool,
        repos::{
            DieselUserRepo, DieselSessionRepo, DieselAuditRepo, DieselPaymentRepo,
            DieselStockRepo, DieselIamRepo, DieselPermissionProfileRepo,
            StubTemporaryPermissionRepo, StubModuleRepo
        }
    },
    firebase_admin::FirebaseAdmin,
    services::{notification::InMemoryNotificationService, NotificationPortAdapter},
    cache::{CacheFactory, SecurityCacheFactory},
};
use crate::security::brute_force_integration::BruteForceIntegrationFactory;
// use crate::web::performance::PerformanceService;

/// Application dependency container with essential services
#[derive(Clone)]
pub struct AppContainer {
    // Database connection pool (Diesel)
    pub database_pool: Arc<DbPool>,
    
    // Alias for backward compatibility
    pub db_pool: Arc<DbPool>,
    
    // Legacy infrastructure factory for backward compatibility
    pub infra: crate::infra::InfraFactory,
    
    // Repository layer (All Diesel implementations)
    pub user_repo: Arc<dyn UserRepo>,
    pub session_repo: Arc<dyn SessRepo>,
    pub audit_repo: Arc<dyn AuditRepo>,
    pub payment_repo: Arc<dyn PayRepo>,
    pub stock_repo: Arc<dyn StockRepo>,
    pub iam_repo: Arc<dyn IamRepo>,
    pub permission_profile_repo: Arc<dyn PermissionProfileRepo>,
    
    // Service layer
    pub firebase_admin: Arc<FirebaseAdmin>,
    pub feature_expiration_service: Arc<dyn FeatureExpirationService>,
    pub admin_module_service: Arc<AdminModuleService>,
    // pub performance_service: Arc<PerformanceService>,
}

/// Builder for creating the dependency container with validation
pub struct AppContainerBuilder {
    database_pool: Option<Arc<DbPool>>,
    database_url: Option<String>,
}

impl AppContainerBuilder {
    pub fn new() -> Self {
        Self {
            database_pool: None,
            database_url: None,
        }
    }
    
    pub fn with_database_pool(mut self, pool: Arc<DbPool>) -> Self {
        self.database_pool = Some(pool);
        self
    }
    
    pub fn with_database_url(mut self, database_url: String) -> Self {
        self.database_url = Some(database_url);
        self
    }
    
    pub async fn build(self) -> Result<AppContainer, Box<dyn std::error::Error + Send + Sync>> {
        // Create or get database pool
        let database_pool = match (self.database_pool, self.database_url) {
            (Some(pool), _) => pool,
            (None, Some(url)) => {
                tracing::info!("🔧 Creating Diesel connection pool...");
                Arc::new(create_pool(&url).await?)
            }
            (None, None) => {
                let url = std::env::var("DATABASE_URL")
                    .map_err(|_| "DATABASE_URL environment variable is required")?;
                tracing::info!("🔧 Creating Diesel connection pool from env...");
                Arc::new(create_pool(&url).await?)
            }
        };
        
        // Create all Diesel repositories
        tracing::info!("🔧 Creating repository layer with Diesel...");
        let user_repo = Arc::new(DieselUserRepo::new(database_pool.clone())) as Arc<dyn UserRepo>;
        let session_repo = Arc::new(DieselSessionRepo::new(database_pool.clone())) as Arc<dyn SessRepo>;
        let audit_repo = Arc::new(DieselAuditRepo::new(database_pool.clone())) as Arc<dyn AuditRepo>;
        let payment_repo = Arc::new(DieselPaymentRepo::new(database_pool.clone())) as Arc<dyn PayRepo>;
        let stock_repo = Arc::new(DieselStockRepo::new(database_pool.clone())) as Arc<dyn StockRepo>;
        let iam_repo = Arc::new(DieselIamRepo::new(database_pool.clone())) as Arc<dyn IamRepo>;
        let permission_profile_repo = Arc::new(DieselPermissionProfileRepo::new(database_pool.clone())) as Arc<dyn PermissionProfileRepo>;
        
        // Create external services
        tracing::info!("🔧 Creating Firebase Admin service...");
        let firebase_admin = Arc::new(FirebaseAdmin::new().await.map_err(|e| {
            tracing::error!("❌ Firebase Admin creation failed: {}", e);
            format!("Firebase Admin creation failed: {}", e)
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
        // TODO: Update AdminModuleService to support Diesel
        let admin_module_service = Arc::new(AdminModuleService::new(database_pool.clone()));
        
        // Create performance service (disabled until migration is run)
        // tracing::info!("🔧 Creating performance service...");
        // let performance_service = Arc::new(PerformanceService::new((*database_pool).clone()));
        
        // Create legacy infrastructure factory for backward compatibility
        tracing::info!("🔧 Creating infrastructure factory...");
        let infra = crate::infra::InfraFactory {
            database_backend: crate::infra::DatabaseBackend::PostgreSQL,
            diesel_pool: database_pool.clone(),
        };
        
        tracing::info!("✅ AppContainer build completed successfully");
        
        Ok(AppContainer {
            database_pool: database_pool.clone(),
            db_pool: database_pool,
            infra,
            user_repo,
            session_repo,
            audit_repo,
            payment_repo,
            stock_repo,
            iam_repo,
            permission_profile_repo,
            firebase_admin,
            feature_expiration_service,
            admin_module_service,
            // performance_service,
        })
    }
}

impl AppContainer {
    /// Create AppState with all dependencies from this container
    pub async fn create_app_state(&self) -> Result<crate::web::auth::AppState, Box<dyn std::error::Error + Send + Sync>> {
        use crate::app::use_cases::{AuthUC, UserMgmtUC};
        use crate::infra::events::SimpleEventDispatcher;
        use crate::infra::db::level_history_repo::InMemoryLevelHistoryRepo;
        
        // Use the repositories from the container (all Diesel-based now)
        let session_repo = self.session_repo.clone();
        let iam_repo = self.iam_repo.clone();
        // TODO: Create Diesel implementations for these repositories
        let temporary_permission_repo = Arc::new(StubTemporaryPermissionRepo::new()) as Arc<dyn crate::app::ports::repositories::TemporaryPermissionRepo>;
        let module_repo = Arc::new(StubModuleRepo::new()) as Arc<dyn crate::app::ports::repositories::ModuleRepo>;
        let usage_repo = Arc::new(crate::infra::db::diesel::repos::DieselUsageRepo::new()) as Arc<dyn crate::app::ports::repositories::UsageRepo>;
        
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

        // Initialize cache
        tracing::info!("🔧 Creating cache service...");
        let cache = CacheFactory::from_env().await?;

        // Initialize security cache
        tracing::info!("🔧 Creating security cache...");
        let security_cache = Some(Arc::new(
            SecurityCacheFactory::create(cache.clone(), 60, 10, 100, 3600)
        ));

        // Create initial AppState for brute force factory
        let temp_app_state = crate::web::auth::AppState::new(
            auth_uc.clone(),
            user_mgmt_uc.clone(),
            session_repo.clone(),
            self.user_repo.clone(),
            iam_repo.clone(),
            self.audit_repo.clone(),
            self.permission_profile_repo.clone(),
            temporary_permission_repo.clone(),
            module_repo.clone(),
            usage_repo.clone(),
            self.firebase_admin.clone(),
            self.admin_module_service.clone(),
            self.feature_expiration_service.clone(),
            self.db_pool.clone(),
            cache.clone(),
            security_cache.clone(),
            None,
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
            self.db_pool.clone(),
            cache,
            security_cache,
            brute_force_service,
        ))
    }

    /// Convenience constructor that creates a container with default configuration
    pub async fn new() -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let database_url = std::env::var("DATABASE_URL")
            .map_err(|_| "DATABASE_URL environment variable is required")?;
        
        tracing::info!("🔧 Creating Diesel database pool...");
        let database_pool = Arc::new(create_pool(&database_url).await.map_err(|e| {
            tracing::error!("❌ Database pool creation failed: {}", e);
            format!("Database pool creation failed: {}", e)
        })?);
        
        tracing::info!("✅ Diesel database pool created successfully");
        tracing::info!("🔧 Building AppContainer...");
        
        AppContainerBuilder::new()
            .with_database_pool(database_pool)
            .build()
            .await
    }
    
    /// Get the unified permission system
    pub fn get_permission_system(&self) -> Result<Arc<crate::permissions::UnifiedPermissionSystem>, Box<dyn std::error::Error + Send + Sync>> {
        // Create the permission system with dependencies
        let config = crate::permissions::core::PermissionConfig::default();
        let system = crate::permissions::UnifiedPermissionSystem::new(config);
        
        Ok(Arc::new(system))
    }
    
    /// Get the admin module system
    pub fn get_admin_module_system(&self) -> Result<Arc<crate::permissions::AdminModuleValidator>, Box<dyn std::error::Error + Send + Sync>> {
        let config = crate::permissions::admin_modules::AdminModuleConfig::default();
        let validator = crate::permissions::AdminModuleValidator::new(config);
        
        Ok(Arc::new(validator))
    }
    
    /// Get the package tier system
    pub fn get_package_tier_system(&self) -> Result<Arc<crate::permissions::PackageTierValidator>, Box<dyn std::error::Error + Send + Sync>> {
        let config = crate::permissions::package_tiers::PackageTierConfig::default();
        let validator = crate::permissions::PackageTierValidator::new(config);
        
        Ok(Arc::new(validator))
    }
    
    /// Get the permission audit system
    pub fn get_audit_system(&self) -> Result<Arc<dyn crate::permissions::PermissionAuditTrait>, Box<dyn std::error::Error + Send + Sync>> {
        // Create database-backed audit system
        let audit_system = crate::permissions::audit::DatabasePermissionAudit::new(
            self.database_pool.clone(),
        );
        
        Ok(Arc::new(audit_system))
    }
    
    /// Get the cache system
    pub async fn cache(&self) -> Result<Arc<dyn crate::infra::cache::Cache>, Box<dyn std::error::Error + Send + Sync>> {
        self.get_cache().await
    }
    
    /// Get the cache system
    pub async fn get_cache(&self) -> Result<Arc<dyn crate::infra::cache::Cache>, Box<dyn std::error::Error + Send + Sync>> {
        // Try to create cache from environment, fallback to in-memory
        match std::env::var("REDIS_URL") {
            Ok(redis_url) if !redis_url.is_empty() => {
                // Try Redis cache
                match crate::infra::cache::redis_cache::RedisCache::new(
                    redis_url,
                    10, // pool size
                    crate::infra::cache::CacheConfig::default()
                ).await {
                    Ok(redis_cache) => Ok(Arc::new(redis_cache)),
                    Err(e) => {
                        tracing::warn!("Failed to create Redis cache: {}, falling back to in-memory", e);
                        Ok(Arc::new(crate::infra::cache::memory_cache::InMemoryCache::new(
                            crate::infra::cache::CacheConfig::default()
                        )))
                    }
                }
            }
            _ => {
                // Use in-memory cache
                Ok(Arc::new(crate::infra::cache::memory_cache::InMemoryCache::new(
                    crate::infra::cache::CacheConfig::default()
                )))
            }
        }
    }
}

