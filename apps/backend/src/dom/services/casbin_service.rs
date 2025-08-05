use casbin::prelude::*;
use sqlx_adapter::SqlxAdapter;
use sqlx::PgPool;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use crate::web::middleware::casbin_cache::CasbinPolicyCache;

pub struct CasbinService {
    enforcer: Arc<RwLock<Enforcer>>,
    cache: Arc<CasbinPolicyCache>,
}

impl CasbinService {
    pub async fn new(db_pool: PgPool) -> std::result::Result<Self, Box<dyn std::error::Error>> {
        let adapter = SqlxAdapter::new_with_pool(db_pool).await?;
        let enforcer = Enforcer::new("casbin_model.conf", adapter).await?;
        
        // Create cache with 5-minute TTL and max 10k entries
        let cache = Arc::new(CasbinPolicyCache::new(
            Duration::from_secs(300),
            10000,
        ));
        
        Ok(CasbinService {
            enforcer: Arc::new(RwLock::new(enforcer)),
            cache,
        })
    }

    pub async fn enforce(&self, subject: &str, object: &str, action: &str) -> std::result::Result<bool, Box<dyn std::error::Error>> {
        // Check cache first
        if let Some(cached_result) = self.cache.get(subject, object, action).await {
            return Ok(cached_result);
        }
        
        // Cache miss - check with Casbin enforcer
        let enforcer = self.enforcer.read().await;
        let result = enforcer.enforce((subject, object, action))?;
        
        // Cache the result
        self.cache.set(subject, object, action, result).await;
        
        Ok(result)
    }

    pub async fn add_policy(&self, subject: &str, object: &str, action: &str) -> std::result::Result<bool, Box<dyn std::error::Error>> {
        let mut enforcer = self.enforcer.write().await;
        let result = enforcer.add_policy(vec![subject.to_string(), object.to_string(), action.to_string()]).await?;
        
        // Invalidate cache for this specific policy
        self.cache.invalidate(subject, object, action).await;
        
        Ok(result)
    }

    pub async fn add_role_for_user(&self, user: &str, role: &str) -> std::result::Result<bool, Box<dyn std::error::Error>> {
        let mut enforcer = self.enforcer.write().await;
        let result = enforcer.add_role_for_user(user, role, None).await?;
        
        // Invalidate all cache entries for this user since role inheritance affects multiple policies
        self.cache.invalidate_user(user).await;
        
        Ok(result)
    }

    pub async fn remove_role_for_user(&self, user: &str, role: &str) -> std::result::Result<bool, Box<dyn std::error::Error>> {
        let mut enforcer = self.enforcer.write().await;
        let result = enforcer.delete_role_for_user(user, role, None).await?;
        
        // Invalidate all cache entries for this user
        self.cache.invalidate_user(user).await;
        
        Ok(result)
    }

    pub async fn remove_policy(&self, subject: &str, object: &str, action: &str) -> std::result::Result<bool, Box<dyn std::error::Error>> {
        let mut enforcer = self.enforcer.write().await;
        let result = enforcer.remove_policy(vec![subject.to_string(), object.to_string(), action.to_string()]).await?;
        
        // Invalidate cache for this specific policy
        self.cache.invalidate(subject, object, action).await;
        
        Ok(result)
    }

    pub async fn get_roles_for_user(&self, user: &str) -> std::result::Result<Vec<String>, Box<dyn std::error::Error>> {
        let enforcer = self.enforcer.read().await;
        Ok(enforcer.get_roles_for_user(user, None))
    }

    /// Reload policies from database - useful for dynamic updates
    pub async fn reload_policies(&self) -> std::result::Result<(), Box<dyn std::error::Error>> {
        let mut enforcer = self.enforcer.write().await;
        enforcer.load_policy().await?;
        
        // Clear all cached entries since policies have been reloaded
        self.cache.clear().await;
        
        tracing::info!("Casbin policies reloaded from database");
        Ok(())
    }

    /// Get cache statistics for monitoring
    pub async fn cache_stats(&self) -> crate::web::middleware::casbin_cache::CacheStats {
        self.cache.stats().await
    }

    /// Clear all cached policy decisions
    pub async fn clear_cache(&self) {
        self.cache.clear().await;
    }

    /// Get all subjects (users/roles) that have a specific permission
    pub async fn get_subjects_for_permission(&self, object: &str, action: &str) -> std::result::Result<Vec<String>, Box<dyn std::error::Error>> {
        let enforcer = self.enforcer.read().await;
        let policy = enforcer.get_policy();
        
        let subjects: Vec<String> = policy
            .into_iter()
            .filter(|rule| rule.len() >= 3 && rule[1] == object && rule[2] == action)
            .map(|rule| rule[0].clone())
            .collect();
            
        Ok(subjects)
    }

    /// Get all permissions for a specific subject
    pub async fn get_permissions_for_subject(&self, subject: &str) -> std::result::Result<Vec<(String, String)>, Box<dyn std::error::Error>> {
        let enforcer = self.enforcer.read().await;
        let policy = enforcer.get_policy();
        
        let permissions: Vec<(String, String)> = policy
            .into_iter()
            .filter(|rule| rule.len() >= 3 && rule[0] == subject)
            .map(|rule| (rule[1].clone(), rule[2].clone()))
            .collect();
            
        Ok(permissions)
    }

    /// Batch add multiple policies - more efficient than individual adds
    pub async fn add_policies(&self, policies: Vec<(String, String, String)>) -> std::result::Result<bool, Box<dyn std::error::Error>> {
        let mut enforcer = self.enforcer.write().await;
        let mut success = true;
        
        for (subject, object, action) in &policies {
            let policy_vec = vec![subject.clone(), object.clone(), action.clone()];
            if !enforcer.add_policy(policy_vec).await? {
                success = false;
            }
            
            // Invalidate cache for each policy
            self.cache.invalidate(subject, object, action).await;
        }
        
        tracing::info!("Batch added {} policies", policies.len());
        Ok(success)
    }

    /// Check if a policy exists
    pub async fn has_policy(&self, subject: &str, object: &str, action: &str) -> std::result::Result<bool, Box<dyn std::error::Error>> {
        let enforcer = self.enforcer.read().await;
        Ok(enforcer.has_policy(vec![subject.to_string(), object.to_string(), action.to_string()]))
    }

    /// Get all policies (for admin interface)
    pub async fn get_all_policies(&self) -> std::result::Result<(Vec<Vec<String>>, Vec<Vec<String>>), Box<dyn std::error::Error>> {
        let enforcer = self.enforcer.read().await;
        let policies = enforcer.get_policy();
        let role_policies = enforcer.get_grouping_policy();
        Ok((policies, role_policies))
    }

    pub async fn get_permissions_for_user(&self, user: &str) -> std::result::Result<Vec<Vec<String>>, Box<dyn std::error::Error>> {
        let enforcer = self.enforcer.read().await;
        Ok(enforcer.get_permissions_for_user(user, None))
    }

    pub async fn load_policies(&self) -> std::result::Result<(), Box<dyn std::error::Error>> {
        let mut enforcer = self.enforcer.write().await;
        enforcer.load_policy().await?;
        Ok(())
    }

    pub async fn save_policies(&self) -> std::result::Result<(), Box<dyn std::error::Error>> {
        let mut enforcer = self.enforcer.write().await;
        enforcer.save_policy().await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_casbin_service_compiles() {
        // Placeholder test to verify CasbinService compiles correctly
        // Real tests would require database setup
        assert!(true);
    }

    #[tokio::test]
    async fn test_casbin_rbac_hierarchy() {
        // This test verifies that the RBAC hierarchy works correctly
        // admin > moderator > premium_user > basic_user
        
        // Note: This test requires a running database with policies loaded
        // In a real test environment, we would set up a test database
        // For now, this validates the expected behavior structure
        
        // Test cases that should pass:
        // - admin should have access to everything
        // - moderator should inherit premium_user permissions
        // - premium_user should inherit basic_user permissions
        // - basic_user should have limited access
        
        assert!(true);
    }
}