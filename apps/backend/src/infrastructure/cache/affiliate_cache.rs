use std::sync::Arc;
use super::{Cache, CacheExt};
use serde::{Serialize, Deserialize};

/// Affiliate-specific cache operations
pub struct AffiliateCache {
    cache: Arc<dyn Cache>,
}

impl AffiliateCache {
    pub fn new(cache: Arc<dyn Cache>) -> Self {
        Self { cache }
    }
    
    // Cache keys
    const AFFILIATE_PREFIX: &'static str = "affiliate:";
    const AFFILIATE_CODE_PREFIX: &'static str = "affiliate_code:";
    const AFFILIATE_STATS_PREFIX: &'static str = "affiliate_stats:";
    const REFERRAL_ATTRIBUTION_PREFIX: &'static str = "referral_attribution:";
    const COMMISSION_RATES_KEY: &'static str = "commission_rates:all";
    const AFFILIATE_MATERIALS_KEY: &'static str = "affiliate_materials:active";
    const REFERRAL_CLICK_PREFIX: &'static str = "referral_click:";
    
    // TTL constants (in seconds)
    const AFFILIATE_TTL: u64 = 3600; // 1 hour
    const STATS_TTL: u64 = 1800; // 30 minutes
    const ATTRIBUTION_TTL: u64 = 86400; // 24 hours
    const COMMISSION_RATES_TTL: u64 = 3600; // 1 hour
    const MATERIALS_TTL: u64 = 1800; // 30 minutes
    const CLICK_TRACKING_TTL: u64 = 300; // 5 minutes for rate limiting
    
    /// Cache affiliate by ID
    pub fn set_affiliate<T: Serialize>(&self, affiliate_id: i32, affiliate: &T) {
        let key = format!("{}{}", Self::AFFILIATE_PREFIX, affiliate_id);
        self.cache.set_typed(&key, affiliate, Some(Self::AFFILIATE_TTL));
    }
    
    /// Get affiliate by ID
    pub fn get_affiliate<T: for<'de> Deserialize<'de>>(&self, affiliate_id: i32) -> Option<T> {
        let key = format!("{}{}", Self::AFFILIATE_PREFIX, affiliate_id);
        self.cache.get_typed(&key)
    }
    
    /// Cache affiliate by code
    pub fn set_affiliate_by_code<T: Serialize>(&self, code: &str, affiliate: &T) {
        let key = format!("{}{}", Self::AFFILIATE_CODE_PREFIX, code);
        self.cache.set_typed(&key, affiliate, Some(Self::AFFILIATE_TTL));
    }
    
    /// Get affiliate by code (most common lookup)
    pub fn get_affiliate_by_code<T: for<'de> Deserialize<'de>>(&self, code: &str) -> Option<T> {
        let key = format!("{}{}", Self::AFFILIATE_CODE_PREFIX, code);
        self.cache.get_typed(&key)
    }
    
    /// Cache affiliate statistics
    pub fn set_affiliate_stats<T: Serialize>(&self, affiliate_id: i32, stats: &T) {
        let key = format!("{}{}", Self::AFFILIATE_STATS_PREFIX, affiliate_id);
        self.cache.set_typed(&key, stats, Some(Self::STATS_TTL));
    }
    
    /// Get affiliate statistics
    pub fn get_affiliate_stats<T: for<'de> Deserialize<'de>>(&self, affiliate_id: i32) -> Option<T> {
        let key = format!("{}{}", Self::AFFILIATE_STATS_PREFIX, affiliate_id);
        self.cache.get_typed(&key)
    }
    
    /// Track referral click for rate limiting and analytics
    pub fn track_referral_click(&self, affiliate_code: &str, ip_address: &str) -> u32 {
        let click_key = format!("{}{}:{}", Self::REFERRAL_CLICK_PREFIX, affiliate_code, ip_address);
        
        if let Some(current) = self.cache.get(&click_key) {
            if let Ok(count) = current.parse::<u32>() {
                let new_count = count + 1;
                self.cache.set(&click_key, new_count.to_string(), Some(Self::CLICK_TRACKING_TTL));
                return new_count;
            }
        }
        
        // First click
        self.cache.set(&click_key, "1".to_string(), Some(Self::CLICK_TRACKING_TTL));
        1
    }
    
    /// Cache referral attribution for a user
    pub fn set_referral_attribution<T: Serialize>(&self, user_id: i32, attribution: &T) {
        let key = format!("{}{}", Self::REFERRAL_ATTRIBUTION_PREFIX, user_id);
        self.cache.set_typed(&key, attribution, Some(Self::ATTRIBUTION_TTL));
    }
    
    /// Get referral attribution for a user
    pub fn get_referral_attribution<T: for<'de> Deserialize<'de>>(&self, user_id: i32) -> Option<T> {
        let key = format!("{}{}", Self::REFERRAL_ATTRIBUTION_PREFIX, user_id);
        self.cache.get_typed(&key)
    }
    
    /// Cache commission rates for all plans
    pub fn set_commission_rates<T: Serialize>(&self, rates: &T) {
        self.cache.set_typed(Self::COMMISSION_RATES_KEY, rates, Some(Self::COMMISSION_RATES_TTL));
    }
    
    /// Get commission rates for all plans
    pub fn get_commission_rates<T: for<'de> Deserialize<'de>>(&self) -> Option<T> {
        self.cache.get_typed(Self::COMMISSION_RATES_KEY)
    }
    
    /// Cache active marketing materials
    pub fn set_marketing_materials<T: Serialize>(&self, materials: &T) {
        self.cache.set_typed(Self::AFFILIATE_MATERIALS_KEY, materials, Some(Self::MATERIALS_TTL));
    }
    
    /// Get active marketing materials
    pub fn get_marketing_materials<T: for<'de> Deserialize<'de>>(&self) -> Option<T> {
        self.cache.get_typed(Self::AFFILIATE_MATERIALS_KEY)
    }
    
    /// Invalidate affiliate cache
    pub fn invalidate_affiliate(&self, affiliate_id: i32) {
        let id_key = format!("{}{}", Self::AFFILIATE_PREFIX, affiliate_id);
        let stats_key = format!("{}{}", Self::AFFILIATE_STATS_PREFIX, affiliate_id);
        self.cache.delete(&id_key);
        self.cache.delete(&stats_key);
    }
    
    /// Invalidate affiliate by code
    pub fn invalidate_affiliate_by_code(&self, code: &str) {
        let key = format!("{}{}", Self::AFFILIATE_CODE_PREFIX, code);
        self.cache.delete(&key);
    }
    
    /// Invalidate all commission rates
    pub fn invalidate_commission_rates(&self) {
        self.cache.delete(Self::COMMISSION_RATES_KEY);
    }
    
    /// Invalidate marketing materials
    pub fn invalidate_marketing_materials(&self) {
        self.cache.delete(Self::AFFILIATE_MATERIALS_KEY);
    }
    
    /// Generate a tracking link with cache-friendly unique ID
    pub fn generate_tracking_id(&self, affiliate_code: &str, campaign: Option<&str>) -> String {
        use std::time::{SystemTime, UNIX_EPOCH};
        
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
            
        let base = format!("{}_{}", affiliate_code, timestamp);
        if let Some(campaign) = campaign {
            format!("{}_c{}", base, campaign)
        } else {
            base
        }
    }
    
    /// Check if affiliate exists in cache (quick existence check)
    pub fn has_affiliate(&self, affiliate_id: i32) -> bool {
        let key = format!("{}{}", Self::AFFILIATE_PREFIX, affiliate_id);
        self.cache.get(&key).is_some()
    }
    
    pub fn has_affiliate_by_code(&self, code: &str) -> bool {
        let key = format!("{}{}", Self::AFFILIATE_CODE_PREFIX, code);
        self.cache.get(&key).is_some()
    }
}

/// Referral attribution data
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ReferralAttribution {
    pub affiliate_id: i32,
    pub affiliate_code: String,
    pub referral_source: Option<String>,
    pub referral_medium: Option<String>,
    pub referral_campaign: Option<String>,
    pub attribution_timestamp: chrono::DateTime<chrono::Utc>,
    pub expires_at: chrono::DateTime<chrono::Utc>,
    pub tracking_id: String,
}

/// Affiliate statistics for dashboard
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AffiliateStats {
    pub total_clicks: u32,
    pub total_conversions: u32,
    pub total_commissions: f64,
    pub conversion_rate: f64,
    pub total_earnings: f64,
    pub pending_commissions: f64,
    pub last_30_days_clicks: u32,
    pub last_30_days_conversions: u32,
    pub last_30_days_earnings: f64,
}

impl AffiliateStats {
    pub fn new() -> Self {
        Self {
            total_clicks: 0,
            total_conversions: 0,
            total_commissions: 0.0,
            conversion_rate: 0.0,
            total_earnings: 0.0,
            pending_commissions: 0.0,
            last_30_days_clicks: 0,
            last_30_days_conversions: 0,
            last_30_days_earnings: 0.0,
        }
    }
    
    pub fn calculate_conversion_rate(&mut self) {
        self.conversion_rate = if self.total_clicks > 0 {
            (self.total_conversions as f64 / self.total_clicks as f64) * 100.0
        } else {
            0.0
        };
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::infrastructure::cache::MemoryCache;
    
    #[derive(Serialize, Deserialize, PartialEq, Debug)]
    struct TestAffiliate {
        id: i32,
        code: String,
        commission_rate: f64,
    }
    
    #[test]
    fn test_affiliate_cache_operations() {
        let memory_cache = Arc::new(MemoryCache::new()) as Arc<dyn Cache>;
        let affiliate_cache = AffiliateCache::new(memory_cache);
        
        let test_affiliate = TestAffiliate {
            id: 1,
            code: "PARTNER123".to_string(),
            commission_rate: 15.0,
        };
        
        // Test ID-based caching
        affiliate_cache.set_affiliate(1, &test_affiliate);
        let retrieved: Option<TestAffiliate> = affiliate_cache.get_affiliate(1);
        assert_eq!(retrieved, Some(test_affiliate.clone()));
        
        // Test code-based caching
        affiliate_cache.set_affiliate_by_code("PARTNER123", &test_affiliate);
        let retrieved_by_code: Option<TestAffiliate> = affiliate_cache.get_affiliate_by_code("PARTNER123");
        assert_eq!(retrieved_by_code, Some(test_affiliate));
        
        // Test click tracking
        let clicks1 = affiliate_cache.track_referral_click("PARTNER123", "192.168.1.1");
        let clicks2 = affiliate_cache.track_referral_click("PARTNER123", "192.168.1.1");
        assert_eq!(clicks1, 1);
        assert_eq!(clicks2, 2);
        
        // Test tracking ID generation
        let tracking_id = affiliate_cache.generate_tracking_id("PARTNER123", Some("summer_sale"));
        assert!(tracking_id.contains("PARTNER123"));
        assert!(tracking_id.contains("csummer_sale"));
    }
    
    #[test]
    fn test_affiliate_stats() {
        let mut stats = AffiliateStats::new();
        stats.total_clicks = 100;
        stats.total_conversions = 15;
        stats.calculate_conversion_rate();
        
        assert_eq!(stats.conversion_rate, 15.0);
    }
}