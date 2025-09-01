// ============================================================================
use uuid::Uuid;
// SIMPLE MODULE AUTH MIDDLEWARE STUB - REPLACING COMPLEX MODULE AUTH
// ============================================================================
// This file provides simple stubs for deleted module auth middleware
// Works with the simple role system from auth/roles.rs

use serde::{Serialize, Deserialize};


// ============================================================================
// SIMPLE AUTH CONTEXT STUBS
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModuleAuthCtx {
    pub user_id: String,
    pub role: String,
    pub is_authenticated: bool,
}

impl ModuleAuthCtx {
    pub fn new(user_id: String, role: String) -> Self {
        Self {
            user_id,
            role,
            is_authenticated: true,
        }
    }

    pub fn unauthenticated() -> Self {
        Self {
            user_id: "anonymous".to_string(),
            role: "user".to_string(),
            is_authenticated: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserModuleAccess {
    pub user_id: Uuid,
    pub module_id: String,
    pub access_level: AccessLevel,
    pub is_active: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AccessLevel {
    None,
    Read,
    Write,
    Admin,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ModuleAccess {
    Granted(AccessLevel),
    Denied,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiKeyAccess {
    pub api_key: String,
    pub permissions: Vec<String>,
    pub is_active: bool,
}

// ============================================================================
// SIMPLE MIDDLEWARE FUNCTIONS (STUBS)
// ============================================================================

impl UserModuleAccess {
    pub fn new(user_id: Uuid, module_id: String, access_level: AccessLevel) -> Self {
        Self {
            user_id,
            module_id,
            access_level,
            is_active: true,
        }
    }

    pub fn can_access(&self, _module: &str) -> bool {
        self.is_active && !matches!(self.access_level, AccessLevel::None)
    }

    pub fn can_write(&self, _module: &str) -> bool {
        self.is_active && matches!(self.access_level, AccessLevel::Write | AccessLevel::Admin)
    }

    pub fn is_admin(&self, _module: &str) -> bool {
        self.is_active && matches!(self.access_level, AccessLevel::Admin)
    }
}

impl ApiKeyAccess {
    pub fn new(api_key: String, permissions: Vec<String>) -> Self {
        Self {
            api_key,
            permissions,
            is_active: true,
        }
    }

    pub fn has_permission(&self, permission: &str) -> bool {
        self.is_active && self.permissions.contains(&permission.to_string())
    }
}

// ============================================================================
// SIMPLE ACCESS HELPERS
// ============================================================================

pub fn check_module_access(role: &str, module: &str) -> ModuleAccess {
    match role {
        "admin" => ModuleAccess::Granted(AccessLevel::Admin),
        "user" => ModuleAccess::Granted(AccessLevel::Write),
        // Guest role removed - unknown roles default to user with read access
        _ => ModuleAccess::Granted(AccessLevel::Read),
    }
}

// Stub middleware function for compatibility
pub async fn module_auth_casbin_middleware() -> Result<(), &'static str> {
    Ok(()) // Simple stub - always allows access
}