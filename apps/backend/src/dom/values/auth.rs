// Auth-related value objects

use serde::{Serialize, Deserialize};
use std::collections::HashSet;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Email(String);

impl Email {
    pub fn new(email: String) -> Result<Self, String> {
        if email.contains('@') && email.len() > 3 {
            Ok(Self(email))
        } else {
            Err("Invalid email format".to_string())
        }
    }

    pub fn value(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for Email {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Role {
    Free,
    User,
    Premium,
    Moderator,
    Admin,
    SuperAdmin,
    ApiClient,
}

impl Role {
    pub fn is_admin(&self) -> bool {
        matches!(self, Role::Admin | Role::SuperAdmin)
    }

    pub fn is_premium(&self) -> bool {
        matches!(self, Role::Premium | Role::Moderator | Role::Admin | Role::SuperAdmin)
    }

    pub fn hierarchy_level(&self) -> u8 {
        match self {
            Role::Free => 0,
            Role::User => 1,
            Role::Premium => 2,
            Role::Moderator => 3,
            Role::Admin => 4,
            Role::SuperAdmin => 5,
            Role::ApiClient => 1, // Similar to User but for API access
        }
    }

    pub fn can_upgrade_to(&self, target: &Role) -> bool {
        self.hierarchy_level() < target.hierarchy_level()
    }

    pub fn from_string(s: &str) -> Result<Self, String> {
        s.parse()
    }

    pub fn from_str(s: &str) -> Result<Self, String> {
        s.parse()
    }
}

impl Default for Role {
    fn default() -> Self {
        Role::Free
    }
}

impl std::fmt::Display for Role {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Role::Free => write!(f, "free"),
            Role::User => write!(f, "user"),
            Role::Premium => write!(f, "premium"),
            Role::Moderator => write!(f, "moderator"),
            Role::Admin => write!(f, "admin"),
            Role::SuperAdmin => write!(f, "super_admin"),
            Role::ApiClient => write!(f, "api_client"),
        }
    }
}

impl std::str::FromStr for Role {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "free" => Ok(Role::Free),
            "user" => Ok(Role::User),
            "premium" => Ok(Role::Premium),
            "moderator" => Ok(Role::Moderator),
            "admin" => Ok(Role::Admin),
            "super_admin" | "superadmin" => Ok(Role::SuperAdmin),
            "api_client" | "apiclient" => Ok(Role::ApiClient),
            _ => Err(format!("Invalid role: {}", s)),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PermissionSet {
    permissions: HashSet<String>,
}

impl PermissionSet {
    pub fn new() -> Self {
        Self {
            permissions: HashSet::new(),
        }
    }

    pub fn with_permissions(permissions: HashSet<String>) -> Self {
        Self { permissions }
    }

    pub fn add_permission(&mut self, permission: String) {
        self.permissions.insert(permission);
    }

    pub fn has_permission(&self, permission: &str) -> bool {
        self.permissions.contains(permission)
    }

    pub fn contains(&self, permission: &str) -> bool {
        self.permissions.contains(permission)
    }

    pub fn permissions(&self) -> &HashSet<String> {
        &self.permissions
    }

    pub fn for_role(role: &Role) -> Self {
        let mut permissions = HashSet::new();
        match role {
            Role::Free => {
                permissions.insert("read:basic".to_string());
            },
            Role::User => {
                permissions.insert("read:own".to_string());
                permissions.insert("write:own".to_string());
            },
            Role::Premium => {
                permissions.insert("read:own".to_string());
                permissions.insert("write:own".to_string());
                permissions.insert("read:premium".to_string());
            },
            Role::Moderator => {
                permissions.insert("read:own".to_string());
                permissions.insert("write:own".to_string());
                permissions.insert("read:premium".to_string());
                permissions.insert("moderate:content".to_string());
            },
            Role::Admin => {
                permissions.insert("read:all".to_string());
                permissions.insert("write:all".to_string());
                permissions.insert("manage:users".to_string());
                permissions.insert("moderate:content".to_string());
            },
            Role::SuperAdmin => {
                permissions.insert("read:all".to_string());
                permissions.insert("write:all".to_string());
                permissions.insert("manage:users".to_string());
                permissions.insert("manage:system".to_string());
                permissions.insert("moderate:content".to_string());
            },
            Role::ApiClient => {
                permissions.insert("read:api".to_string());
                permissions.insert("module:access".to_string());
            },
        }
        Self::with_permissions(permissions)
    }

    pub fn to_vec(&self) -> Vec<String> {
        self.permissions.iter().cloned().collect()
    }

    pub fn from_vec(permissions: Vec<String>) -> Self {
        Self::with_permissions(permissions.into_iter().collect())
    }
}

impl Default for PermissionSet {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SubscriptionTier {
    Free,
    Basic,
    Premium,
    Enterprise,
}

impl SubscriptionTier {
    pub fn is_paid(&self) -> bool {
        !matches!(self, SubscriptionTier::Free)
    }

    pub fn from_string(s: &str) -> Result<Self, String> {
        match s.to_lowercase().as_str() {
            "free" => Ok(SubscriptionTier::Free),
            "basic" => Ok(SubscriptionTier::Basic),
            "premium" => Ok(SubscriptionTier::Premium),
            "enterprise" => Ok(SubscriptionTier::Enterprise),
            _ => Err(format!("Invalid subscription tier: {}", s)),
        }
    }
}

impl std::fmt::Display for SubscriptionTier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SubscriptionTier::Free => write!(f, "free"),
            SubscriptionTier::Basic => write!(f, "basic"),
            SubscriptionTier::Premium => write!(f, "premium"),
            SubscriptionTier::Enterprise => write!(f, "enterprise"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Subscription {
    pub tier: SubscriptionTier,
    pub expires_at: Option<chrono::DateTime<chrono::Utc>>,
}

impl Subscription {
    pub fn new(tier: SubscriptionTier) -> Self {
        Self {
            tier,
            expires_at: None,
        }
    }

    pub fn free() -> Self {
        Self::new(SubscriptionTier::Free)
    }

    pub fn paid(tier: SubscriptionTier, expires_at: chrono::DateTime<chrono::Utc>) -> Self {
        Self::with_expiry(tier, expires_at)
    }

    pub fn with_expiry(tier: SubscriptionTier, expires_at: chrono::DateTime<chrono::Utc>) -> Self {
        Self {
            tier,
            expires_at: Some(expires_at),
        }
    }

    pub fn is_active(&self) -> bool {
        match self.expires_at {
            Some(expiry) => chrono::Utc::now() < expiry,
            None => true,
        }
    }

    pub fn tier(&self) -> &SubscriptionTier {
        &self.tier
    }
}

