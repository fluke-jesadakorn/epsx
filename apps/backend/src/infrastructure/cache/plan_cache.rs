use std::sync::Arc;
use super::{Cache, CacheExt};
use serde::{Serialize, Deserialize};

/// Plan-specific cache operations with Redis + memory fallback
pub struct PlanCache {
    cache: Arc<dyn Cache>,
}

impl PlanCache {
    pub fn new(cache: Arc<dyn Cache>) -> Self {
        Self { cache }
    }
    
    // Cache keys
    const PERSONAL_PLANS_KEY: &'static str = "plans:personal:with_promos";
    const API_PLANS_KEY: &'static str = "plans:api:with_promos";
    const ALL_PLANS_KEY: &'static str = "plans:all:with_promos";
    const PLAN_DETAILS_PREFIX: &'static str = "plan:details:";
    
    // TTL constants (in seconds)
    const PLANS_TTL: u64 = 300; // 5 minutes
    const PLAN_DETAILS_TTL: u64 = 600; // 10 minutes
    
    /// Cache personal plans with promotions
    pub fn set_personal_plans<T: Serialize>(&self, plans: &T) {
        self.cache.set_typed(Self::PERSONAL_PLANS_KEY, plans, Some(Self::PLANS_TTL));
    }
    
    /// Get cached personal plans
    pub fn get_personal_plans<T: for<'de> Deserialize<'de>>(&self) -> Option<T> {
        self.cache.get_typed(Self::PERSONAL_PLANS_KEY)
    }
    
    /// Cache API plans with promotions
    pub fn set_api_plans<T: Serialize>(&self, plans: &T) {
        self.cache.set_typed(Self::API_PLANS_KEY, plans, Some(Self::PLANS_TTL));
    }
    
    /// Get cached API plans
    pub fn get_api_plans<T: for<'de> Deserialize<'de>>(&self) -> Option<T> {
        self.cache.get_typed(Self::API_PLANS_KEY)
    }
    
    /// Cache all plans (combined response)
    pub fn set_all_plans<T: Serialize>(&self, plans: &T) {
        self.cache.set_typed(Self::ALL_PLANS_KEY, plans, Some(Self::PLANS_TTL));
    }
    
    /// Get all cached plans
    pub fn get_all_plans<T: for<'de> Deserialize<'de>>(&self) -> Option<T> {
        self.cache.get_typed(Self::ALL_PLANS_KEY)
    }
    
    /// Cache individual plan details
    pub fn set_plan_details<T: Serialize>(&self, plan_id: i32, plan: &T) {
        let key = format!("{}{}", Self::PLAN_DETAILS_PREFIX, plan_id);
        self.cache.set_typed(&key, plan, Some(Self::PLAN_DETAILS_TTL));
    }
    
    /// Get individual plan details
    pub fn get_plan_details<T: for<'de> Deserialize<'de>>(&self, plan_id: i32) -> Option<T> {
        let key = format!("{}{}", Self::PLAN_DETAILS_PREFIX, plan_id);
        self.cache.get_typed(&key)
    }
    
    /// Invalidate all plan-related caches
    pub fn invalidate_all_plans(&self) {
        self.cache.delete(Self::PERSONAL_PLANS_KEY);
        self.cache.delete(Self::API_PLANS_KEY);
        self.cache.delete(Self::ALL_PLANS_KEY);
    }
    
    /// Invalidate specific plan cache
    pub fn invalidate_plan(&self, plan_id: i32) {
        let key = format!("{}{}", Self::PLAN_DETAILS_PREFIX, plan_id);
        self.cache.delete(&key);
        // Also invalidate the grouped caches
        self.invalidate_all_plans();
    }
    
    /// Check if plans are cached
    pub fn has_personal_plans(&self) -> bool {
        self.cache.get(Self::PERSONAL_PLANS_KEY).is_some()
    }
    
    pub fn has_api_plans(&self) -> bool {
        self.cache.get(Self::API_PLANS_KEY).is_some()
    }
    
    pub fn has_all_plans(&self) -> bool {
        self.cache.get(Self::ALL_PLANS_KEY).is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::infrastructure::cache::MemoryCache;
    
    #[derive(Serialize, Deserialize, PartialEq, Debug)]
    struct TestPlan {
        id: i32,
        name: String,
        price: f64,
    }
    
    #[test]
    fn test_plan_cache_operations() {
        let memory_cache = Arc::new(MemoryCache::new()) as Arc<dyn Cache>;
        let plan_cache = PlanCache::new(memory_cache);
        
        let test_plan = TestPlan {
            id: 1,
            name: "Gold Plan".to_string(),
            price: 19.99,
        };
        
        let plans = vec![test_plan.clone()];
        
        // Test caching and retrieval
        plan_cache.set_personal_plans(&plans);
        let retrieved: Option<Vec<TestPlan>> = plan_cache.get_personal_plans();
        assert_eq!(retrieved, Some(plans));
        
        // Test invalidation
        plan_cache.invalidate_all_plans();
        let after_invalidation: Option<Vec<TestPlan>> = plan_cache.get_personal_plans();
        assert_eq!(after_invalidation, None);
    }
}