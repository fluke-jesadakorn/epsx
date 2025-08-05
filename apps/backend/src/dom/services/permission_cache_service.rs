use redis::{Client, Commands};
use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Serialize, Deserialize)]
struct CachedPermission {
    allowed: bool,
    expires_at: u64,
}

pub struct PermissionCacheService {
    redis_client: Client,
}

impl PermissionCacheService {
    pub fn new(redis_url: &str) -> Result<Self, redis::RedisError> {
        let client = Client::open(redis_url)?;
        Ok(Self { redis_client: client })
    }

    pub async fn get_cached_permission(&self, user_id: &str, resource: &str, action: &str) -> Option<bool> {
        let mut conn = self.redis_client.get_connection().ok()?;
        let key = format!("perm:{}:{}:{}", user_id, resource, action);
        
        let cached: String = conn.get(&key).ok()?;
        let permission: CachedPermission = serde_json::from_str(&cached).ok()?;
        
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .ok()?
            .as_secs();
            
        if permission.expires_at > now {
            Some(permission.allowed)
        } else {
            None
        }
    }

    pub async fn cache_permission(&self, user_id: &str, resource: &str, action: &str, allowed: bool, ttl: Duration) {
        if let Ok(mut conn) = self.redis_client.get_connection() {
            let key = format!("perm:{}:{}:{}", user_id, resource, action);
            let expires_at = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs() + ttl.as_secs();
                
            let cached = CachedPermission { allowed, expires_at };
            if let Ok(json) = serde_json::to_string(&cached) {
                let _: Result<(), _> = conn.set_ex(&key, json, ttl.as_secs() as usize);
            }
        }
    }
}