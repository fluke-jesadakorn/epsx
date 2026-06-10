use std::collections::HashMap;use std::sync::{Arc, RwLock};
use super::{Cache, CacheConfig};

/// In-memory cache implementation
pub struct MemoryCache {
    data: Arc<RwLock<HashMap<String, String>>>,
}

impl Default for MemoryCache {
    fn default() -> Self {
        Self::new()
    }
}

impl MemoryCache {
    pub fn new() -> Self {
        Self {
            data: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub fn with_config(_config: CacheConfig) -> Self {
        Self {
            data: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

impl Cache for MemoryCache {
    fn get(&self, key: &str) -> Option<String> {
        self.data.read().unwrap_or_else(|e| e.into_inner()).get(key).cloned()
    }

    fn set(&self, key: &str, value: String, _ttl: Option<u64>) {
        self.data.write().unwrap_or_else(|e| e.into_inner()).insert(key.to_string(), value);
    }

    fn delete(&self, key: &str) {
        self.data.write().unwrap_or_else(|e| e.into_inner()).remove(key);
    }

    fn clear(&self) {
        self.data.write().unwrap_or_else(|e| e.into_inner()).clear();
    }
}