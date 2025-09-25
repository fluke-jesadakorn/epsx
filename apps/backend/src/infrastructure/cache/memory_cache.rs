use std::collections::HashMap;use std::sync::{Arc, RwLock};
use super::{Cache, CacheConfig};

/// In-memory cache implementation
pub struct MemoryCache {
    data: Arc<RwLock<HashMap<String, String>>>,
    #[allow(dead_code)]
    config: CacheConfig,
}

impl MemoryCache {
    pub fn new() -> Self {
        Self {
            data: Arc::new(RwLock::new(HashMap::new())),
            config: CacheConfig::default(),
        }
    }

    pub fn with_config(config: CacheConfig) -> Self {
        Self {
            data: Arc::new(RwLock::new(HashMap::new())),
            config,
        }
    }
}

impl Cache for MemoryCache {
    fn get(&self, key: &str) -> Option<String> {
        self.data.read().unwrap().get(key).cloned()
    }

    fn set(&self, key: &str, value: String, _ttl: Option<u64>) {
        self.data.write().unwrap().insert(key.to_string(), value);
    }

    fn delete(&self, key: &str) {
        self.data.write().unwrap().remove(key);
    }

    fn clear(&self) {
        self.data.write().unwrap().clear();
    }
}