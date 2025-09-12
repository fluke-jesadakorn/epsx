// Access Context Value Objects
// Immutable objects representing different access contexts for resources

use serde::{Deserialize, Serialize};
use std::fmt;

/// Represents different access contexts for resource usage
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AccessContext {
    Internal,   // Web application access
    External,   // API access
    Both,       // Both web and API access
}

/// Configuration for access context
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AccessContextConfig {
    pub context: AccessContext,
    pub permissions: Vec<String>,
    pub rate_limits: AccessRateLimit,
    pub features_enabled: Vec<String>,
    pub api_key_required: bool,
}

/// Rate limiting configuration per access context
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AccessRateLimit {
    pub requests_per_minute: Option<u32>,
    pub requests_per_hour: Option<u32>,
    pub requests_per_day: Option<u32>,
    pub burst_limit: Option<u32>,
}

/// API access configuration
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ApiAccessConfig {
    pub api_key: String,
    pub allowed_endpoints: Vec<String>,
    pub rate_limit: AccessRateLimit,
    pub expires_at: Option<chrono::DateTime<chrono::Utc>>,
    pub is_active: bool,
}

impl AccessContext {
    /// Check if this context allows API access
    pub fn allows_api_access(&self) -> bool {
        matches!(self, AccessContext::External | AccessContext::Both)
    }

    /// Check if this context allows web access
    pub fn allows_web_access(&self) -> bool {
        matches!(self, AccessContext::Internal | AccessContext::Both)
    }

    /// Get the string representation for database storage
    pub fn as_str(&self) -> &'static str {
        match self {
            AccessContext::Internal => "internal",
            AccessContext::External => "external", 
            AccessContext::Both => "both",
        }
    }

    /// Parse from string representation
    pub fn from_str(s: &str) -> Result<Self, String> {
        match s.to_lowercase().as_str() {
            "internal" => Ok(AccessContext::Internal),
            "external" => Ok(AccessContext::External),
            "both" => Ok(AccessContext::Both),
            _ => Err(format!("Invalid access context: {}", s)),
        }
    }

    /// Get all available contexts
    pub fn all() -> Vec<AccessContext> {
        vec![
            AccessContext::Internal,
            AccessContext::External,
            AccessContext::Both,
        ]
    }
}

impl AccessContextConfig {
    pub fn new(context: AccessContext) -> Self {
        Self {
            context: context.clone(),
            permissions: Vec::new(),
            rate_limits: AccessRateLimit::default(),
            features_enabled: Vec::new(),
            api_key_required: context.allows_api_access(),
        }
    }

    pub fn with_permissions(mut self, permissions: Vec<String>) -> Self {
        self.permissions = permissions;
        self
    }

    pub fn with_rate_limits(mut self, rate_limits: AccessRateLimit) -> Self {
        self.rate_limits = rate_limits;
        self
    }

    pub fn with_features(mut self, features: Vec<String>) -> Self {
        self.features_enabled = features;
        self
    }

    pub fn has_permission(&self, permission: &str) -> bool {
        self.permissions.iter().any(|p| p == permission)
    }

    pub fn has_feature(&self, feature: &str) -> bool {
        self.features_enabled.iter().any(|f| f == feature)
    }
}

impl AccessRateLimit {
    pub fn new() -> Self {
        Self {
            requests_per_minute: None,
            requests_per_hour: None,
            requests_per_day: None,
            burst_limit: None,
        }
    }

    pub fn with_minute_limit(mut self, limit: u32) -> Self {
        self.requests_per_minute = Some(limit);
        self
    }

    pub fn with_hour_limit(mut self, limit: u32) -> Self {
        self.requests_per_hour = Some(limit);
        self
    }

    pub fn with_day_limit(mut self, limit: u32) -> Self {
        self.requests_per_day = Some(limit);
        self
    }

    pub fn with_burst_limit(mut self, limit: u32) -> Self {
        self.burst_limit = Some(limit);
        self
    }

    pub fn is_unlimited(&self) -> bool {
        self.requests_per_minute.is_none() 
            && self.requests_per_hour.is_none() 
            && self.requests_per_day.is_none()
    }
}

impl Default for AccessRateLimit {
    fn default() -> Self {
        Self::new()
    }
}

impl ApiAccessConfig {
    pub fn new(api_key: String) -> Self {
        Self {
            api_key,
            allowed_endpoints: Vec::new(),
            rate_limit: AccessRateLimit::default(),
            expires_at: None,
            is_active: true,
        }
    }

    pub fn with_endpoints(mut self, endpoints: Vec<String>) -> Self {
        self.allowed_endpoints = endpoints;
        self
    }

    pub fn with_rate_limit(mut self, rate_limit: AccessRateLimit) -> Self {
        self.rate_limit = rate_limit;
        self
    }

    pub fn with_expiry(mut self, expires_at: chrono::DateTime<chrono::Utc>) -> Self {
        self.expires_at = Some(expires_at);
        self
    }

    pub fn is_endpoint_allowed(&self, endpoint: &str) -> bool {
        if self.allowed_endpoints.is_empty() {
            return true; // Allow all if none specified
        }
        
        self.allowed_endpoints.iter().any(|allowed| {
            endpoint.starts_with(allowed) || allowed == "*"
        })
    }

    pub fn is_expired(&self) -> bool {
        if let Some(expires_at) = self.expires_at {
            chrono::Utc::now() > expires_at
        } else {
            false
        }
    }

    pub fn is_valid(&self) -> bool {
        self.is_active && !self.is_expired()
    }
}

impl fmt::Display for AccessContext {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}