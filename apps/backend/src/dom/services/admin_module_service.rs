// ============================================================================
// ADMIN MODULE SERVICE STUB - REPLACING COMPLEX MODULE SYSTEM
// ============================================================================
// Simple stub for removed admin module service

use crate::dom::values::UserId;
use serde::{Serialize, Deserialize};

// ============================================================================
// REQUEST TYPES (STUBS)
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdminModuleAssignRequest {
    pub user_id: UserId,
    pub module_name: String,
    pub access_level: String,
    pub expires_at: Option<chrono::DateTime<chrono::Utc>>,
}

// Simple stub service
pub struct AdminModuleService;

impl AdminModuleService {
    pub fn new() -> Self {
        Self
    }

    pub async fn assign_module_to_user(&self, _user_id: &UserId, _module_name: &str) -> Result<(), String> {
        Ok(()) // Stub - simple roles handle this now
    }

    pub async fn revoke_module_from_user(&self, _user_id: &UserId, _module_name: &str) -> Result<(), String> {
        Ok(()) // Stub
    }

    pub async fn has_module_access(&self, _user_id: &UserId, _module_name: &str) -> Result<bool, String> {
        Ok(true) // Stub - always allow for now
    }

    pub async fn get_user_admin_modules(&self, _firebase_uid: &str) -> Result<Vec<String>, String> {
        Ok(vec![]) // Stub - return empty modules for simple role system
    }

    pub async fn get_all_admin_modules(&self) -> Result<Vec<String>, String> {
        // Return basic admin modules for simple role system
        Ok(vec![
            "user_management".to_string(),
            "system_admin".to_string(),
            "security_monitoring".to_string(),
        ])
    }

    pub async fn assign_admin_modules(&self, _request: &AdminModuleAssignRequest) -> Result<Vec<String>, String> {
        // Stub - return the assigned module names
        Ok(vec![_request.module_name.clone()])
    }

    pub async fn revoke_admin_modules(&self, _user_id: &UserId, _module_names: Vec<String>) -> Result<Vec<String>, String> {
        // Stub - return the revoked module names
        Ok(_module_names)
    }

    pub async fn assign_all_admin_modules(&self, _user_id: &str, _admin_id: &str, _reason: &str) -> Result<Vec<String>, String> {
        // Stub - assign all modules to user
        Ok(vec![
            "user_management".to_string(),
            "system_admin".to_string(),
            "security_monitoring".to_string(),
        ])
    }

    pub async fn get_admin_role_audit(&self, _user_id: &str) -> Result<Vec<serde_json::Value>, String> {
        // Stub - return empty audit records
        Ok(vec![])
    }

    pub async fn user_has_admin_module(&self, _user_id: &str, _module_code: &str) -> Result<bool, String> {
        // Stub - return false for simple role system
        Ok(false)
    }
}

impl Default for AdminModuleService {
    fn default() -> Self {
        Self::new()
    }
}