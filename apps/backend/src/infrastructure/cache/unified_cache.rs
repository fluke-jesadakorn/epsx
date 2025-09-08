use super::Cache;

/// Unified cache that can switch between different implementations
pub struct UnifiedCache {
    cache: Box<dyn Cache>,
}

impl UnifiedCache {
    pub fn new(cache: Box<dyn Cache>) -> Self {
        Self { cache }
    }
}

impl Cache for UnifiedCache {
    fn get(&self, key: &str) -> Option<String> {
        self.cache.get(key)
    }

    fn set(&self, key: &str, value: String, ttl: Option<u64>) {
        self.cache.set(key, value, ttl);
    }

    fn delete(&self, key: &str) {
        self.cache.delete(key);
    }

    fn clear(&self) {
        self.cache.clear();
    }
}