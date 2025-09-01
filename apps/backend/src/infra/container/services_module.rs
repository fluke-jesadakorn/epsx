// ============================================================================
// SIMPLE SERVICES MODULE - REPLACING COMPLEX SERVICE MANAGEMENT
// ============================================================================
// This file replaces complex service modules with simple ones for the basic role system
// Works with the simple role system from auth/roles.rs

use std::sync::Arc;
use crate::app::ports::repositories::{UserRepository, UserPermissionRepository};
use crate::infra::{
    firebase_admin::FirebaseAdmin,
    services::notification_service::{NotificationService, InMemoryNotificationService},
    services::permission_infrastructure::{
        PermissionInfrastructureService, PermissionInfrastructureServiceFactory
    },
    db::diesel::DbPool,
    cache::Cache,
};
use crate::dom::services::{PermissionService, PermissionServiceFactory};
use crate::app::services::{
    PermissionApplicationService, PermissionApplicationServiceFactory
};
use crate::auth::{RefreshTokenService, RefreshTokenConfig};
use crate::infra::db::diesel::repos::{RefreshTokenRepository, RevokedTokenRepository};
// Removed legacy service imports

// ============================================================================
// PERMISSION SYSTEMS STUB (FOR COMPATIBILITY)
// ============================================================================

#[derive(Clone)]
pub struct PermissionSystems {
    pub simple_roles: bool,
}

impl PermissionSystems {
    pub fn simple() -> Self {
        Self {
            simple_roles: true,
        }
    }
}

// ============================================================================
// SIMPLE SERVICES MODULE
// ============================================================================

/// Simple services module with basic functionality
#[derive(Clone)]
pub struct ServicesModule {
    pub firebase_admin: Arc<FirebaseAdmin>,
    pub notification_service: Arc<dyn NotificationService>,
    pub permission_service: Arc<PermissionService>,
    pub permission_infrastructure_service: Arc<PermissionInfrastructureService>,
    pub permission_application_service: Arc<PermissionApplicationService>,
    pub refresh_token_service: Arc<RefreshTokenService>,
}

impl ServicesModule {
    /// Create a new simple services module with minimal dependencies
    pub async fn new(
        _database_pool: Arc<DbPool>,
        user_repo: Arc<dyn UserRepository>,
        user_permission_repo: Arc<dyn UserPermissionRepository>,
        _cache: Arc<dyn Cache>,
        refresh_token_repo: Arc<RefreshTokenRepository>,
        revoked_token_repo: Arc<RevokedTokenRepository>,
    ) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        
        // Create Firebase admin service - use test client for now to avoid async issues
        let firebase_admin = Arc::new(FirebaseAdmin::create_test_client());

        // Create simple notification service (in-memory for now)
        let notification_service: Arc<dyn NotificationService> = Arc::new(InMemoryNotificationService::new());

        // Create permission services with clean architecture
        tracing::info!("🔐 Creating permission services with environment configuration...");
        
        // 1. Create PermissionService (table-only mode)
        let permission_service = Arc::new(PermissionServiceFactory::create(
            user_permission_repo.clone()
        ));
        
        // Log completion of migration to table-only mode
        tracing::info!("🔄 Permission system: Table-only mode (migration completed)");

        // 2. Create PermissionInfrastructureService
        let permission_infrastructure_service = Arc::new(PermissionInfrastructureServiceFactory::create(
            user_repo.clone(),
            permission_service.clone(),
        ));

        // 3. Create PermissionApplicationService
        let permission_application_service = Arc::new(PermissionApplicationServiceFactory::create(
            permission_service.clone(),
            permission_infrastructure_service.clone(),
            user_repo.clone(),
        ));

        // 4. Create refresh token service
        let refresh_token_config = RefreshTokenConfig::default();
        let refresh_token_service = Arc::new(RefreshTokenService::new(
            refresh_token_config,
            refresh_token_repo,
            revoked_token_repo,
        ));

        tracing::info!("✅ Permission services created successfully");

        Ok(ServicesModule {
            firebase_admin,
            notification_service,
            permission_service,
            permission_infrastructure_service,
            permission_application_service,
            refresh_token_service,
        })
    }

    // ============================================================================
    // SIMPLE SERVICE ACCESSORS
    // ============================================================================

    pub fn get_firebase_admin(&self) -> Arc<FirebaseAdmin> {
        self.firebase_admin.clone()
    }

    pub fn get_notification_service(&self) -> Arc<dyn NotificationService> {
        self.notification_service.clone()
    }

    pub fn get_permission_service(&self) -> Arc<PermissionService> {
        self.permission_service.clone()
    }

    pub fn get_permission_infrastructure_service(&self) -> Arc<PermissionInfrastructureService> {
        self.permission_infrastructure_service.clone()
    }

    pub fn get_permission_application_service(&self) -> Arc<PermissionApplicationService> {
        self.permission_application_service.clone()
    }
}

// ============================================================================
// SIMPLE STUB SERVICES (FOR COMPATIBILITY)
// ============================================================================

/// Simple stub for feature expiration (always returns no expiration)
pub struct SimpleFeatureService;

impl SimpleFeatureService {
    pub fn new() -> Self {
        Self
    }

    pub fn is_feature_expired(&self, _user_id: &str, _feature: &str) -> bool {
        false // Simple system doesn't expire features
    }

    pub fn get_expiration_date(&self, _user_id: &str, _feature: &str) -> Option<chrono::DateTime<chrono::Utc>> {
        None // Simple system doesn't expire features
    }
}

/// Simple stub for admin module service (role-based access only)
pub struct SimpleAdminService;

impl SimpleAdminService {
    pub fn new() -> Self {
        Self
    }

    pub fn can_access_admin(&self, role: &str) -> bool {
        role == "admin"
    }

    pub fn can_manage_users(&self, role: &str) -> bool {
        role == "admin"
    }
}