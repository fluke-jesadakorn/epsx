use std::sync::Arc;
use super::{Cache, CacheExt};
use serde::{Serialize, Deserialize};

/// Promotion-specific cache operations
pub struct PromotionCache {
    cache: Arc<dyn Cache>,
}

impl PromotionCache {
    pub fn new(cache: Arc<dyn Cache>) -> Self {
        Self { cache }
    }
    
    // Cache keys
    const ACTIVE_CAMPAIGNS_KEY: &'static str = "campaigns:active";
    const CAMPAIGN_PREFIX: &'static str = "campaign:";
    const DISCOUNT_CODE_PREFIX: &'static str = "discount_code:";
    const PLAN_PROMOTIONS_PREFIX: &'static str = "plan_promotions:";
    const EXPERIMENTS_KEY: &'static str = "experiments:active";
    
    // TTL constants (in seconds)
    const CAMPAIGNS_TTL: u64 = 600; // 10 minutes
    const DISCOUNT_CODE_TTL: u64 = 3600; // 1 hour
    const EXPERIMENTS_TTL: u64 = 1800; // 30 minutes
    
    /// Cache active promotional campaigns
    pub fn set_active_campaigns<T: Serialize>(&self, campaigns: &T) {
        self.cache.set_typed(Self::ACTIVE_CAMPAIGNS_KEY, campaigns, Some(Self::CAMPAIGNS_TTL));
    }
    
    /// Get active promotional campaigns
    pub fn get_active_campaigns<T: for<'de> Deserialize<'de>>(&self) -> Option<T> {
        self.cache.get_typed(Self::ACTIVE_CAMPAIGNS_KEY)
    }
    
    /// Cache individual campaign
    pub fn set_campaign<T: Serialize>(&self, campaign_id: i32, campaign: &T) {
        let key = format!("{}{}", Self::CAMPAIGN_PREFIX, campaign_id);
        self.cache.set_typed(&key, campaign, Some(Self::CAMPAIGNS_TTL));
    }
    
    /// Get individual campaign
    pub fn get_campaign<T: for<'de> Deserialize<'de>>(&self, campaign_id: i32) -> Option<T> {
        let key = format!("{}{}", Self::CAMPAIGN_PREFIX, campaign_id);
        self.cache.get_typed(&key)
    }
    
    /// Cache discount code validation result
    pub fn set_discount_code<T: Serialize>(&self, code: &str, discount_data: &T) {
        let key = format!("{}{}", Self::DISCOUNT_CODE_PREFIX, code);
        self.cache.set_typed(&key, discount_data, Some(Self::DISCOUNT_CODE_TTL));
    }
    
    /// Get discount code validation result
    pub fn get_discount_code<T: for<'de> Deserialize<'de>>(&self, code: &str) -> Option<T> {
        let key = format!("{}{}", Self::DISCOUNT_CODE_PREFIX, code);
        self.cache.get_typed(&key)
    }
    
    /// Increment discount code usage (for rate limiting)
    pub fn increment_code_usage(&self, code: &str) -> u32 {
        let usage_key = format!("{}{}:usage", Self::DISCOUNT_CODE_PREFIX, code);
        
        if let Some(current) = self.cache.get(&usage_key) {
            if let Ok(count) = current.parse::<u32>() {
                let new_count = count + 1;
                self.cache.set(&usage_key, new_count.to_string(), Some(Self::DISCOUNT_CODE_TTL));
                return new_count;
            }
        }
        
        // First use
        self.cache.set(&usage_key, "1".to_string(), Some(Self::DISCOUNT_CODE_TTL));
        1
    }
    
    /// Cache plan-specific promotions
    pub fn set_plan_promotions<T: Serialize>(&self, plan_id: i32, promotions: &T) {
        let key = format!("{}{}", Self::PLAN_PROMOTIONS_PREFIX, plan_id);
        self.cache.set_typed(&key, promotions, Some(Self::CAMPAIGNS_TTL));
    }
    
    /// Get plan-specific promotions
    pub fn get_plan_promotions<T: for<'de> Deserialize<'de>>(&self, plan_id: i32) -> Option<T> {
        let key = format!("{}{}", Self::PLAN_PROMOTIONS_PREFIX, plan_id);
        self.cache.get_typed(&key)
    }
    
    /// Cache active pricing experiments
    pub fn set_active_experiments<T: Serialize>(&self, experiments: &T) {
        self.cache.set_typed(Self::EXPERIMENTS_KEY, experiments, Some(Self::EXPERIMENTS_TTL));
    }
    
    /// Get active pricing experiments
    pub fn get_active_experiments<T: for<'de> Deserialize<'de>>(&self) -> Option<T> {
        self.cache.get_typed(Self::EXPERIMENTS_KEY)
    }
    
    /// Invalidate all promotion-related caches
    pub fn invalidate_all_promotions(&self) {
        self.cache.delete(Self::ACTIVE_CAMPAIGNS_KEY);
        self.cache.delete(Self::EXPERIMENTS_KEY);
    }
    
    /// Invalidate specific campaign cache
    pub fn invalidate_campaign(&self, campaign_id: i32) {
        let key = format!("{}{}", Self::CAMPAIGN_PREFIX, campaign_id);
        self.cache.delete(&key);
        self.cache.delete(Self::ACTIVE_CAMPAIGNS_KEY); // Refresh active campaigns list
    }
    
    /// Invalidate discount code cache
    pub fn invalidate_discount_code(&self, code: &str) {
        let key = format!("{}{}", Self::DISCOUNT_CODE_PREFIX, code);
        let usage_key = format!("{}{}:usage", Self::DISCOUNT_CODE_PREFIX, code);
        self.cache.delete(&key);
        self.cache.delete(&usage_key);
    }
    
    /// Invalidate plan promotions
    pub fn invalidate_plan_promotions(&self, plan_id: i32) {
        let key = format!("{}{}", Self::PLAN_PROMOTIONS_PREFIX, plan_id);
        self.cache.delete(&key);
    }
    
    /// Check if campaigns are cached
    pub fn has_active_campaigns(&self) -> bool {
        self.cache.get(Self::ACTIVE_CAMPAIGNS_KEY).is_some()
    }
}

/// Promotion calculation result
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PromotionResult {
    pub has_promotion: bool,
    pub original_price: f64,
    pub discounted_price: f64,
    pub discount_amount: f64,
    pub discount_percentage: f64,
    pub promotional_badge: Option<String>,
    pub promotional_message: Option<String>,
    pub expires_at: Option<chrono::DateTime<chrono::Utc>>,
}

impl PromotionResult {
    pub fn no_promotion(price: f64) -> Self {
        Self {
            has_promotion: false,
            original_price: price,
            discounted_price: price,
            discount_amount: 0.0,
            discount_percentage: 0.0,
            promotional_badge: None,
            promotional_message: None,
            expires_at: None,
        }
    }
    
    pub fn with_discount(
        original_price: f64,
        discounted_price: f64,
        badge: Option<String>,
        message: Option<String>,
        expires_at: Option<chrono::DateTime<chrono::Utc>>,
    ) -> Self {
        let discount_amount = original_price - discounted_price;
        let discount_percentage = if original_price > 0.0 {
            (discount_amount / original_price) * 100.0
        } else {
            0.0
        };
        
        Self {
            has_promotion: true,
            original_price,
            discounted_price,
            discount_amount,
            discount_percentage,
            promotional_badge: badge,
            promotional_message: message,
            expires_at,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::infrastructure::cache::MemoryCache;
    
    #[derive(Serialize, Deserialize, PartialEq, Debug)]
    struct TestCampaign {
        id: i32,
        name: String,
        active: bool,
    }
    
    #[test]
    fn test_promotion_cache_operations() {
        let memory_cache = Arc::new(MemoryCache::new()) as Arc<dyn Cache>;
        let promotion_cache = PromotionCache::new(memory_cache);
        
        let test_campaign = TestCampaign {
            id: 1,
            name: "Winter Sale".to_string(),
            active: true,
        };
        
        let campaigns = vec![test_campaign.clone()];
        
        // Test caching and retrieval
        promotion_cache.set_active_campaigns(&campaigns);
        let retrieved: Option<Vec<TestCampaign>> = promotion_cache.get_active_campaigns();
        assert_eq!(retrieved, Some(campaigns));
        
        // Test individual campaign caching
        promotion_cache.set_campaign(1, &test_campaign);
        let retrieved_campaign: Option<TestCampaign> = promotion_cache.get_campaign(1);
        assert_eq!(retrieved_campaign, Some(test_campaign));
        
        // Test discount code usage tracking
        let usage1 = promotion_cache.increment_code_usage("TESTCODE");
        let usage2 = promotion_cache.increment_code_usage("TESTCODE");
        assert_eq!(usage1, 1);
        assert_eq!(usage2, 2);
    }
    
    #[test]
    fn test_promotion_result() {
        let no_promo = PromotionResult::no_promotion(19.99);
        assert!(!no_promo.has_promotion);
        assert_eq!(no_promo.original_price, 19.99);
        assert_eq!(no_promo.discounted_price, 19.99);
        
        let with_discount = PromotionResult::with_discount(
            19.99,
            9.99,
            Some("50% OFF".to_string()),
            Some("Save big!".to_string()),
            None,
        );
        assert!(with_discount.has_promotion);
        assert_eq!(with_discount.discount_amount, 10.0);
        assert!((with_discount.discount_percentage - 50.025).abs() < 0.001); // ~50%
    }
}