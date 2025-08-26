// ============================================================================
// SIMPLE SERVICES MODULE - REPLACING COMPLEX SERVICE MANAGEMENT
// ============================================================================
// This file replaces complex service modules with simple ones for the basic role system
// Works with the simple role system from auth/roles.rs

use std::sync::Arc;
use crate::app::ports::repositories::UserRepository;
use crate::infra::{
    firebase_admin::FirebaseAdmin,
    services::notification_service::{NotificationService, InMemoryNotificationService},
    db::diesel::DbPool,
    cache::Cache,
};
use crate::dom::services::{
    admin_module_service::AdminModuleService,
};

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
    pub admin_module_service: Arc<AdminModuleService>,
}

impl ServicesModule {
    /// Create a new simple services module with minimal dependencies
    pub async fn new(
        _database_pool: Arc<DbPool>,
        _user_repo: Arc<dyn UserRepository>,
        _cache: Arc<dyn Cache>,
    ) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        
        // Create Firebase admin service - use test client for now to avoid async issues
        let firebase_admin = Arc::new(FirebaseAdmin::create_test_client());

        // Create simple notification service (in-memory for now)
        let notification_service: Arc<dyn NotificationService> = Arc::new(InMemoryNotificationService::new());
        
        // Create stub admin module service
        let admin_module_service = Arc::new(AdminModuleService::new());
        

        Ok(ServicesModule {
            firebase_admin,
            notification_service,
            admin_module_service,
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