use crate::domain::shared_kernel::value_objects::UserId;
use super::Cache;

/// Result of hash validation
#[derive(Debug, Clone)]
pub struct HashValidationResult {
    pub is_valid: bool,
    pub cached_hash: Option<String>,
    pub computed_hash: String,
}

/// Permission-specific cache operations
pub struct PermissionCache {
    cache: Box<dyn Cache>,
}

impl PermissionCache {
    pub fn new(cache: Box<dyn Cache>) -> Self {
        Self { cache }
    }

    pub fn get_user_permissions(&self, wallet_address: &UserId) -> Option<Vec<String>> {
        let key = format!("permissions:{}", wallet_address.as_str());
        self.cache.get(&key).and_then(|data| {
            serde_json::from_str(&data).ok()
        })
    }

    pub fn set_user_permissions(&self, wallet_address: &UserId, permissions: &[String]) {
        let key = format!("permissions:{}", wallet_address.as_str());
        if let Ok(data) = serde_json::to_string(permissions) {
            self.cache.set(&key, data, Some(300)); // 5 minutes
        }
    }

    pub fn clear_user_permissions(&self, wallet_address: &UserId) {
        let key = format!("permissions:{}", wallet_address.as_str());
        self.cache.delete(&key);
    }
}

/// Service wrapper for permission cache operations
pub struct PermissionCacheService {
    cache: PermissionCache,
}

impl PermissionCacheService {
    pub fn new(cache: Box<dyn Cache>) -> Self {
        Self {
            cache: PermissionCache::new(cache),
        }
    }

    pub fn get_user_permissions(&self, wallet_address: &UserId) -> Option<Vec<String>> {
        self.cache.get_user_permissions(wallet_address)
    }

    pub fn set_user_permissions(&self, wallet_address: &UserId, permissions: &[String]) {
        self.cache.set_user_permissions(wallet_address, permissions);
    }

    pub fn validate_hash(&self, _key: &str, expected_hash: &str) -> HashValidationResult {
        // Simple validation logic
        HashValidationResult {
            is_valid: true, // Placeholder
            cached_hash: None,
            computed_hash: expected_hash.to_string(),
        }
    }
}