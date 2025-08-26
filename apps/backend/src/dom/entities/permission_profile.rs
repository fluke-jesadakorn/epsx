// ============================================================================
// SIMPLE PERMISSION PROFILE STUB - REPLACING COMPLEX PERMISSION PROFILES
// ============================================================================
// This file provides simple stubs for deleted permission profile entities
// Works with the simple role system from auth/roles.rs

use serde::{Serialize, Deserialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};

// ============================================================================
// TYPE ALIASES FOR COMPATIBILITY
// ============================================================================

pub type PermissionProfileId = String;

// ============================================================================  
// ERROR TYPES
// ============================================================================

#[derive(Debug, thiserror::Error)]
pub enum PermissionProfileError {
    #[error("Permission profile not found")]
    NotFound,
    #[error("Invalid permission profile data")]
    InvalidData(String),
    #[error("Database error")]
    DatabaseError(String),
}

// ============================================================================
// QUERY TYPES
// ============================================================================

#[derive(Debug, Clone)]
pub struct PermissionProfileQuery {
    pub name: Option<String>,
    pub category: Option<PermissionProfileCategory>,
    pub is_active: Option<bool>,
    pub limit: Option<u32>,
    pub offset: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PermissionProfileCategory {
    System,
    User,
    Custom,
}

// ============================================================================
// REQUEST/RESPONSE TYPES
// ============================================================================

#[derive(Debug, Clone)]
pub struct ApplyPermissionProfileRequest {
    pub profile_id: PermissionProfileId,
    pub user_ids: Vec<crate::dom::values::UserId>,
    pub expires_at: Option<DateTime<Utc>>,
    pub reason: String,
}

#[derive(Debug, Clone)]
pub struct ApplyPermissionProfileResult {
    pub profile_id: String,
    pub applied_count: u32,
    pub failed_count: u32,
    pub failures: Vec<String>,
}

// ============================================================================
// PERMISSION PROFILE ENTITY
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PermissionProfile {
    pub id: PermissionProfileId,
    pub name: String,
    pub description: String,
    pub category: PermissionProfileCategory,
    pub permissions: Vec<String>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub created_by: Option<String>,
}

// ============================================================================
// SIMPLE PERMISSION PROFILE STUB
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimplePermissionProfile {
    pub id: Uuid,
    pub name: String,
    pub role: String, // Maps to our simple role system
    pub is_active: bool,
}

impl SimplePermissionProfile {
    pub fn new(name: String, role: String) -> Self {
        Self {
            id: Uuid::new_v4(),
            name,
            role,
            is_active: true,
        }
    }

    pub fn admin_profile() -> Self {
        Self::new("Admin Profile".to_string(), "admin".to_string())
    }

    pub fn user_profile() -> Self {
        Self::new("User Profile".to_string(), "user".to_string())
    }

    pub fn guest_profile() -> Self {
        Self::new("Guest Profile".to_string(), "guest".to_string())
    }

    pub fn can_access_feature(&self, feature: &str) -> bool {
        use crate::auth::roles::{Role, check_feature_access};
        use std::str::FromStr;
        
        match Role::from_str(&self.role) {
            Ok(role) => check_feature_access(&role, feature),
            Err(_) => false,
        }
    }
}

// ============================================================================
// STUB ENTITIES FOR COMPATIBILITY
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PermissionGroup {
    pub id: Uuid,
    pub name: String,
    pub permissions: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Permission {
    pub id: Uuid,
    pub name: String,
    pub resource: String,
    pub action: String,
}

impl Permission {
    pub fn new(name: String, resource: String, action: String) -> Self {
        Self {
            id: Uuid::new_v4(),
            name,
            resource,
            action,
        }
    }
}