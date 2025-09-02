// Clean Granular Permission System
// Modern permission validation with individual expiry times and metadata

use std::collections::HashMap;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Granular permission claim with metadata and expiry
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct GranularPermissionClaim {
    /// When this specific permission expires (None = permanent)
    pub expires_at: Option<i64>,
    
    /// Source of this permission grant
    pub source: PermissionSource,
    
    /// Who granted this permission (admin user ID)
    pub granted_by: Option<String>,
    
    /// When this permission was granted
    pub granted_at: i64,
    
    /// Additional metadata for this permission
    pub metadata: Option<HashMap<String, String>>,
}

/// Source of permission grant
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum PermissionSource {
    /// From user subscription
    Subscription,
    
    /// Manual grant by admin
    ManualGrant,
    
    /// Time-limited access grant
    TimeLimitedAccess,
    
    /// Admin-granted permission
    AdminGrant,
    
    /// System-generated permission
    SystemGrant,
    
    /// Temporary testing access
    TestAccess,
}

/// Complete granular permission set for a user
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GranularPermissionSet {
    /// Map of permission string to claim details
    pub permissions: HashMap<String, GranularPermissionClaim>,
    
    /// Version of this permission set (for cache invalidation)
    pub version: u32,
    
    /// Hash of all permissions for instant validation
    pub hash: String,
    
    /// When this permission set was last updated
    pub updated_at: i64,
    
    /// When to next validate permissions (for expiry checking)
    pub next_validation: i64,
}

/// Result of permission validation
#[derive(Debug, Clone)]
pub struct PermissionValidationResult {
    /// Whether the permission is granted
    pub granted: bool,
    
    /// Valid permissions after expiry cleanup
    pub valid_permissions: Vec<String>,
    
    /// Updated permission set (if cleanup occurred)
    pub updated_set: Option<GranularPermissionSet>,
    
    /// Reason for denial (if not granted)
    pub denial_reason: Option<String>,
}

/// Permission validation context
#[derive(Debug, Clone)]
pub struct ValidationContext {
    /// Current timestamp for expiry checking
    pub current_time: i64,
    
    /// Whether to perform cleanup of expired permissions
    pub cleanup_expired: bool,
    
    /// Required permission to validate
    pub required_permission: String,
    
    /// Additional context data
    pub context: HashMap<String, String>,
}

/// Granular permission errors
#[derive(Debug, Error)]
pub enum GranularPermissionError {
    #[error("Permission denied: {0}")]
    PermissionDenied(String),
    
    #[error("Permission expired at {0}")]
    PermissionExpired(DateTime<Utc>),
    
    #[error("Invalid permission format: {0}")]
    InvalidFormat(String),
    
    #[error("Permission set version mismatch: expected {expected}, got {actual}")]
    VersionMismatch { expected: u32, actual: u32 },
    
    #[error("Permission hash mismatch")]
    HashMismatch,
    
    #[error("Invalid permission source: {0}")]
    InvalidSource(String),
    
    #[error("Serialization error: {0}")]
    SerializationError(String),
}

impl GranularPermissionClaim {
    /// Create a new permanent permission claim
    pub fn permanent(source: PermissionSource, granted_by: Option<String>) -> Self {
        Self {
            expires_at: None,
            source,
            granted_by,
            granted_at: Utc::now().timestamp(),
            metadata: None,
        }
    }
    
    /// Create a new temporary permission claim
    pub fn temporary(
        expires_at: DateTime<Utc>, 
        source: PermissionSource, 
        granted_by: Option<String>
    ) -> Self {
        Self {
            expires_at: Some(expires_at.timestamp()),
            source,
            granted_by,
            granted_at: Utc::now().timestamp(),
            metadata: None,
        }
    }
    
    /// Create a permission claim with metadata
    pub fn with_metadata(
        expires_at: Option<DateTime<Utc>>,
        source: PermissionSource,
        granted_by: Option<String>,
        metadata: HashMap<String, String>,
    ) -> Self {
        Self {
            expires_at: expires_at.map(|dt| dt.timestamp()),
            source,
            granted_by,
            granted_at: Utc::now().timestamp(),
            metadata: Some(metadata),
        }
    }
    
    /// Check if this permission is currently valid
    pub fn is_valid(&self, current_time: i64) -> bool {
        match self.expires_at {
            Some(expiry) => expiry > current_time,
            None => true, // Permanent permission
        }
    }
    
    /// Check if this permission expires soon (within hours)
    pub fn expires_soon(&self, current_time: i64, hours: i64) -> bool {
        match self.expires_at {
            Some(expiry) => expiry <= current_time + (hours * 3600),
            None => false, // Permanent permissions don't expire
        }
    }
    
    /// Get time until expiry in seconds (None if permanent)
    pub fn time_until_expiry(&self, current_time: i64) -> Option<i64> {
        self.expires_at.map(|expiry| expiry - current_time)
    }
}

impl GranularPermissionSet {
    /// Create a new empty permission set
    pub fn new() -> Self {
        let now = Utc::now().timestamp();
        Self {
            permissions: HashMap::new(),
            version: 1,
            hash: Self::calculate_hash(&HashMap::new()),
            updated_at: now,
            next_validation: now + 300, // 5 minutes from now
        }
    }
    
    /// Create permission set from permissions map
    pub fn from_permissions(permissions: HashMap<String, GranularPermissionClaim>) -> Self {
        let now = Utc::now().timestamp();
        let hash = Self::calculate_hash(&permissions);
        Self {
            permissions,
            version: 1,
            hash,
            updated_at: now,
            next_validation: now + 300,
        }
    }
    
    /// Add or update a permission
    pub fn add_permission(&mut self, permission: String, claim: GranularPermissionClaim) {
        self.permissions.insert(permission, claim);
        self.version += 1;
        self.hash = Self::calculate_hash(&self.permissions);
        self.updated_at = Utc::now().timestamp();
    }
    
    /// Remove a permission
    pub fn remove_permission(&mut self, permission: &str) -> Option<GranularPermissionClaim> {
        let result = self.permissions.remove(permission);
        if result.is_some() {
            self.version += 1;
            self.hash = Self::calculate_hash(&self.permissions);
            self.updated_at = Utc::now().timestamp();
        }
        result
    }
    
    /// Clean up expired permissions
    pub fn cleanup_expired(&mut self, current_time: i64) -> Vec<String> {
        let mut removed = Vec::new();
        
        self.permissions.retain(|perm, claim| {
            if !claim.is_valid(current_time) {
                removed.push(perm.clone());
                false
            } else {
                true
            }
        });
        
        if !removed.is_empty() {
            self.version += 1;
            self.hash = Self::calculate_hash(&self.permissions);
            self.updated_at = current_time;
        }
        
        // Update next validation time
        self.next_validation = current_time + 300; // 5 minutes
        
        removed
    }
    
    /// Validate a specific permission
    pub fn validate_permission(&self, context: &ValidationContext) -> PermissionValidationResult {
        // Check if permission exists
        let claim = match self.permissions.get(&context.required_permission) {
            Some(claim) => claim,
            None => return PermissionValidationResult {
                granted: false,
                valid_permissions: self.get_valid_permissions(context.current_time),
                updated_set: None,
                denial_reason: Some("Permission not found".to_string()),
            }
        };
        
        // Check if permission is valid (not expired)
        if !claim.is_valid(context.current_time) {
            return PermissionValidationResult {
                granted: false,
                valid_permissions: self.get_valid_permissions(context.current_time),
                updated_set: None,
                denial_reason: Some(format!(
                    "Permission expired at {}", 
                    DateTime::from_timestamp(claim.expires_at.unwrap_or(0), 0)
                        .unwrap_or_default()
                        .format("%Y-%m-%d %H:%M:%S UTC")
                )),
            };
        }
        
        // Permission is valid
        PermissionValidationResult {
            granted: true,
            valid_permissions: self.get_valid_permissions(context.current_time),
            updated_set: None,
            denial_reason: None,
        }
    }
    
    /// Get all currently valid permissions
    pub fn get_valid_permissions(&self, current_time: i64) -> Vec<String> {
        self.permissions
            .iter()
            .filter_map(|(perm, claim)| {
                if claim.is_valid(current_time) {
                    Some(perm.clone())
                } else {
                    None
                }
            })
            .collect()
    }
    
    /// Get permissions expiring soon
    pub fn get_expiring_permissions(&self, current_time: i64, hours: i64) -> Vec<(String, i64)> {
        self.permissions
            .iter()
            .filter_map(|(perm, claim)| {
                if claim.expires_soon(current_time, hours) {
                    claim.expires_at.map(|expiry| (perm.clone(), expiry))
                } else {
                    None
                }
            })
            .collect()
    }
    
    /// Calculate hash of permissions for validation
    fn calculate_hash(permissions: &HashMap<String, GranularPermissionClaim>) -> String {
        use std::collections::BTreeMap;
        
        // Sort permissions for consistent hashing
        let sorted: BTreeMap<_, _> = permissions.iter().collect();
        let serialized = serde_json::to_string(&sorted).unwrap_or_default();
        
        // Simple hash - in production, use a proper cryptographic hash
        format!("{:x}", {
            use std::collections::hash_map::DefaultHasher;
            use std::hash::{Hash, Hasher};
            let mut hasher = DefaultHasher::new();
            serialized.hash(&mut hasher);
            hasher.finish()
        })
    }
    
    /// Validate hash matches current permissions
    pub fn validate_hash(&self) -> bool {
        let calculated = Self::calculate_hash(&self.permissions);
        calculated == self.hash
    }
    
    /// Check if validation is needed (based on next_validation time)
    pub fn needs_validation(&self, current_time: i64) -> bool {
        current_time >= self.next_validation
    }
}

impl Default for GranularPermissionSet {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_permission_claim_creation() {
        let claim = GranularPermissionClaim::permanent(
            PermissionSource::Subscription, 
            Some("admin_123".to_string())
        );
        
        assert!(claim.expires_at.is_none());
        assert_eq!(claim.source, PermissionSource::Subscription);
        assert_eq!(claim.granted_by, Some("admin_123".to_string()));
        
        let now = Utc::now().timestamp();
        assert!(claim.is_valid(now));
        assert!(!claim.expires_soon(now, 24));
    }
    
    #[test]
    fn test_temporary_permission() {
        let expires_at = Utc::now() + chrono::Duration::hours(2);
        let claim = GranularPermissionClaim::temporary(
            expires_at, 
            PermissionSource::ManualGrant, 
            Some("admin_456".to_string())
        );
        
        assert!(claim.expires_at.is_some());
        
        let now = Utc::now().timestamp();
        assert!(claim.is_valid(now));
        assert!(claim.expires_soon(now, 3)); // Expires within 3 hours
        assert!(!claim.expires_soon(now, 1)); // Doesn't expire within 1 hour
    }
    
    #[test]
    fn test_permission_set_operations() {
        let mut perm_set = GranularPermissionSet::new();
        
        let claim1 = GranularPermissionClaim::permanent(
            PermissionSource::Subscription, 
            None
        );
        let claim2 = GranularPermissionClaim::temporary(
            Utc::now() + chrono::Duration::hours(1),
            PermissionSource::AdminGrant,
            Some("admin_789".to_string())
        );
        
        perm_set.add_permission("epsx:rankings:view:5".to_string(), claim1);
        perm_set.add_permission("epsx:analytics:premium".to_string(), claim2);
        
        assert_eq!(perm_set.permissions.len(), 2);
        assert_eq!(perm_set.version, 3); // Started at 1, two additions
        
        let valid_perms = perm_set.get_valid_permissions(Utc::now().timestamp());
        assert_eq!(valid_perms.len(), 2);
        
        // Test hash validation
        assert!(perm_set.validate_hash());
    }
    
    #[test]
    fn test_permission_validation() {
        let mut perm_set = GranularPermissionSet::new();
        
        let claim = GranularPermissionClaim::permanent(
            PermissionSource::Subscription, 
            None
        );
        perm_set.add_permission("epsx:rankings:view:10".to_string(), claim);
        
        let context = ValidationContext {
            current_time: Utc::now().timestamp(),
            cleanup_expired: false,
            required_permission: "epsx:rankings:view:10".to_string(),
            context: HashMap::new(),
        };
        
        let result = perm_set.validate_permission(&context);
        assert!(result.granted);
        assert!(result.denial_reason.is_none());
        
        // Test non-existent permission
        let context_invalid = ValidationContext {
            current_time: Utc::now().timestamp(),
            cleanup_expired: false,
            required_permission: "epsx:rankings:view:50".to_string(),
            context: HashMap::new(),
        };
        
        let result_invalid = perm_set.validate_permission(&context_invalid);
        assert!(!result_invalid.granted);
        assert!(result_invalid.denial_reason.is_some());
    }
    
    #[test]
    fn test_expired_permission_cleanup() {
        let mut perm_set = GranularPermissionSet::new();
        
        // Add expired permission
        let expired_claim = GranularPermissionClaim::temporary(
            Utc::now() - chrono::Duration::hours(1), // 1 hour ago
            PermissionSource::TimeLimitedAccess,
            None
        );
        
        // Add valid permission
        let valid_claim = GranularPermissionClaim::permanent(
            PermissionSource::Subscription,
            None
        );
        
        perm_set.add_permission("expired_perm".to_string(), expired_claim);
        perm_set.add_permission("valid_perm".to_string(), valid_claim);
        
        assert_eq!(perm_set.permissions.len(), 2);
        
        // Clean up expired permissions
        let now = Utc::now().timestamp();
        let removed = perm_set.cleanup_expired(now);
        
        assert_eq!(removed.len(), 1);
        assert_eq!(removed[0], "expired_perm");
        assert_eq!(perm_set.permissions.len(), 1);
        assert!(perm_set.permissions.contains_key("valid_perm"));
    }
}