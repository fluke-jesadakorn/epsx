// Services Module - Handles business logic services creation
// Focused module for Firebase, admin modules, feature expiration, and notifications

use std::sync::Arc;
use crate::app::ports::repositories::UserRepo;
use crate::dom::ports::NotificationPort;
use crate::dom::services::feature_expiration::FeatureExpirationService;
use crate::dom::services::admin_module_service::AdminModuleService;
use crate::infra::{
    firebase_admin::FirebaseAdmin,
    services::{notification::{NotificationService, InMemoryNotificationService}, NotificationPortAdapter},
    db::diesel::DbPool,
    cache::Cache,
};

/// Services module responsible for business logic services creation
#[derive(Clone)]
pub struct ServicesModule {
    pub firebase_admin: Arc<FirebaseAdmin>,
    pub feature_expiration_service: Arc<dyn FeatureExpirationService>,
    pub admin_module_service: Arc<AdminModuleService>,
    pub notification_service: Arc<dyn NotificationService>,
}

impl ServicesModule {
    /// Create a new services module with dependencies
    pub async fn new(
        database_pool: Arc<DbPool>,
        user_repo: Arc<dyn UserRepo>,
        _cache: Arc<dyn Cache>,
    ) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        // Create Firebase Admin service
        tracing::info!("🔧 Creating Firebase Admin service...");
        let firebase_admin = Arc::new(FirebaseAdmin::new().await.map_err(|e| {
            tracing::error!("❌ Firebase Admin creation failed: {}", e);
            format!("Firebase Admin creation failed: {}", e)
        })?);
        
        // Create notification services
        tracing::info!("🔧 Creating notification services...");
        // TODO: Implement proper notification repo with correct pool type
        // For now, using in-memory service to allow compilation
        let notification_service = Arc::new(
            InMemoryNotificationService::new()
        ) as Arc<dyn NotificationService>;
        
        let notification_port: Arc<dyn NotificationPort> = Arc::new(
            NotificationPortAdapter::new(notification_service.clone())
        );
        
        // Create feature expiration service
        tracing::info!("🔧 Creating feature expiration service...");
        let feature_expiration_service = {
            use crate::dom::services::feature_expiration::{FeatureExpirationServiceImpl, ExpirationConfig};
            Arc::new(FeatureExpirationServiceImpl::new(
                user_repo,
                notification_port,
                Some(ExpirationConfig::default()),
            )) as Arc<dyn FeatureExpirationService>
        };
        
        // Create admin module service
        tracing::info!("🔧 Creating admin module service...");
        let admin_module_service = Arc::new(AdminModuleService::new(database_pool));
        
        tracing::info!("✅ Services module created successfully");
        
        Ok(ServicesModule {
            firebase_admin,
            feature_expiration_service,
            admin_module_service,
            notification_service,
        })
    }

    /// Create permission system components
    pub fn create_permission_systems(
        &self,
        database_pool: Arc<DbPool>
    ) -> Result<PermissionSystems, Box<dyn std::error::Error + Send + Sync>> {
        // Create the unified permission system
        let permission_config = crate::permissions::core::PermissionConfig::default();
        let unified_permission_system = Arc::new(
            crate::permissions::UnifiedPermissionSystem::new(permission_config)
        );
        
        // Create admin module system
        let admin_config = crate::permissions::admin_modules::AdminModuleConfig::default();
        let admin_module_validator = Arc::new(
            crate::permissions::AdminModuleValidator::new(admin_config)
        );
        
        // Create package tier system
        let tier_config = crate::permissions::package_tiers::PackageTierConfig::default();
        let package_tier_validator = Arc::new(
            crate::permissions::PackageTierValidator::new(tier_config)
        );
        
        // Create audit system
        let audit_system = Arc::new(
            crate::permissions::audit::DatabasePermissionAudit::new(database_pool)
        ) as Arc<dyn crate::permissions::PermissionAuditTrait>;
        
        Ok(PermissionSystems {
            unified_permission_system,
            admin_module_validator,
            package_tier_validator,
            audit_system,
        })
    }
}

/// Container for permission system components
pub struct PermissionSystems {
    pub unified_permission_system: Arc<crate::permissions::UnifiedPermissionSystem>,
    pub admin_module_validator: Arc<crate::permissions::AdminModuleValidator>,
    pub package_tier_validator: Arc<crate::permissions::PackageTierValidator>,
    pub audit_system: Arc<dyn crate::permissions::PermissionAuditTrait>,
}