// ============================================================================
// SIMPLE IAM ENTITY STUBS - REPLACING COMPLEX IAM SYSTEM
// ============================================================================
// This file provides simple stubs for the deleted IAM entities
// Works with the simple role system from auth/roles.rs

use serde::{Serialize, Deserialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};

// ============================================================================
// TYPE ALIASES FOR COMPATIBILITY
// ============================================================================

pub type RoleId = String;
pub type PolicyId = String;
pub type GroupId = String;

// ============================================================================
// ERROR TYPES
// ============================================================================

#[derive(Debug, thiserror::Error)]
pub enum IamError {
    #[error("Not found")]
    NotFound,
    #[error("Invalid data: {0}")]
    InvalidData(String),
    #[error("Database error: {0}")]
    DatabaseError(String),
}

// ============================================================================
// IAM ENTITIES (SIMPLE STUBS)
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IamRole {
    pub id: RoleId,
    pub name: String,
    pub description: String,
    pub permissions: Vec<String>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IamPolicy {
    pub id: PolicyId,
    pub name: String,
    pub description: String,
    pub statements: Vec<String>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IamGroup {
    pub id: GroupId,
    pub name: String,
    pub description: String,
    pub members: Vec<crate::dom::values::UserId>,
    pub policies: Vec<PolicyId>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserPermissionOverride {
    pub user_id: crate::dom::values::UserId,
    pub granted_permissions: Vec<String>,
    pub denied_permissions: Vec<String>,
    pub expires_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub created_by: String,
}

// ============================================================================
// SIMPLE ROLE STUB (FOR COMPATIBILITY)
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum PackageTier {
    Free,
    Premium,
    Enterprise,
}

impl std::fmt::Display for PackageTier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PackageTier::Free => write!(f, "free"),
            PackageTier::Premium => write!(f, "premium"),
            PackageTier::Enterprise => write!(f, "enterprise"),
        }
    }
}

// Stub for backwards compatibility - maps to our simple roles
impl PackageTier {
    pub fn to_simple_role(&self) -> &str {
        match self {
            PackageTier::Free => "guest",
            PackageTier::Premium => "user",
            PackageTier::Enterprise => "admin",
        }
    }
}

// ============================================================================
// SIMPLE IAM STUBS
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimpleIamProfile {
    pub id: Uuid,
    pub user_id: Uuid,
    pub role: String, // Maps to our simple role system
}

impl SimpleIamProfile {
    pub fn new(user_id: Uuid, role: String) -> Self {
        Self {
            id: Uuid::new_v4(),
            user_id,
            role,
        }
    }

    pub fn is_admin(&self) -> bool {
        self.role == "admin"
    }

    pub fn is_user(&self) -> bool {
        self.role == "user"
    }

    pub fn is_guest(&self) -> bool {
        self.role == "guest"
    }
}