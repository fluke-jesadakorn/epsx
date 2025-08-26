// Auth-related value objects

use serde::{Serialize, Deserialize};

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

    /// Create subscription from tier string (for compatibility with database)
    pub fn from_tier_string(tier_string: &str) -> Result<Self, String> {
        let tier = SubscriptionTier::from_string(tier_string)?;
        Ok(Self::new(tier))
    }
}

// ============================================================================
// PERMISSION GROUPS STUB (FOR COMPATIBILITY)
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PermissionGroups {
    pub groups: std::collections::HashMap<String, Vec<String>>,
}

impl PermissionGroups {
    pub fn new() -> Self {
        Self {
            groups: std::collections::HashMap::new(),
        }
    }

    pub fn simple_permissions() -> Self {
        let mut groups = std::collections::HashMap::new();
        groups.insert("admin".to_string(), vec![
            "view_eps".to_string(),
            "export_data".to_string(),
            "realtime".to_string(),
            "profile".to_string(),
            "notifications".to_string(),
            "billing".to_string(),
            "advanced_filters".to_string(),
        ]);
        groups.insert("user".to_string(), vec![
            "view_eps".to_string(),
            "export_data".to_string(),
            "realtime".to_string(),
            "profile".to_string(),
            "notifications".to_string(),
            "billing".to_string(),
            "advanced_filters".to_string(),
        ]);
        groups.insert("guest".to_string(), vec![
            "view_eps".to_string(),
        ]);
        
        Self { groups }
    }
}

impl Default for PermissionGroups {
    fn default() -> Self {
        Self::simple_permissions()
    }
}