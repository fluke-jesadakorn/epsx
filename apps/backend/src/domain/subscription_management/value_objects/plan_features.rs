use crate::prelude::*;

/// Plan features value object
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlanFeatures {
    api_calls_limit: Option<i32>,
    rankings_limit: Option<i32>,
    analytics_enabled: bool,
    premium_support: bool,
    custom_features: serde_json::Value,
    // Rate limiting fields
    rate_limit_per_minute: Option<i32>,
    rate_limit_per_hour: Option<i32>,
    rate_limit_per_day: Option<i32>,
    burst_capacity: Option<i32>,
}

impl PlanFeatures {
    pub fn new(
        api_calls_limit: Option<i32>,
        rankings_limit: Option<i32>,
        analytics_enabled: bool,
        premium_support: bool,
    ) -> Self {
        Self {
            api_calls_limit,
            rankings_limit,
            analytics_enabled,
            premium_support,
            custom_features: serde_json::json!({}),
            rate_limit_per_minute: Some(60),
            rate_limit_per_hour: Some(1000),
            rate_limit_per_day: Some(10000),
            burst_capacity: Some(10),
        }
    }

    pub fn with_rate_limits(
        mut self,
        per_minute: Option<i32>,
        per_hour: Option<i32>,
        per_day: Option<i32>,
        burst: Option<i32>,
    ) -> Self {
        self.rate_limit_per_minute = per_minute;
        self.rate_limit_per_hour = per_hour;
        self.rate_limit_per_day = per_day;
        self.burst_capacity = burst;
        self
    }

    pub fn with_custom_features(mut self, features: serde_json::Value) -> Self {
        self.custom_features = features;
        self
    }

    pub fn api_calls_limit(&self) -> Option<i32> {
        self.api_calls_limit
    }

    pub fn rankings_limit(&self) -> Option<i32> {
        self.rankings_limit
    }

    pub fn analytics_enabled(&self) -> bool {
        self.analytics_enabled
    }

    pub fn premium_support(&self) -> bool {
        self.premium_support
    }

    pub fn custom_features(&self) -> &serde_json::Value {
        &self.custom_features
    }

    pub fn rate_limit_per_minute(&self) -> Option<i32> {
        self.rate_limit_per_minute
    }

    pub fn rate_limit_per_hour(&self) -> Option<i32> {
        self.rate_limit_per_hour
    }

    pub fn rate_limit_per_day(&self) -> Option<i32> {
        self.rate_limit_per_day
    }

    pub fn burst_capacity(&self) -> Option<i32> {
        self.burst_capacity
    }
}

impl Default for PlanFeatures {
    fn default() -> Self {
        Self {
            api_calls_limit: Some(100),
            rankings_limit: Some(3),
            analytics_enabled: false,
            premium_support: false,
            custom_features: serde_json::json!({}),
            rate_limit_per_minute: Some(60),
            rate_limit_per_hour: Some(1000),
            rate_limit_per_day: Some(10000),
            burst_capacity: Some(10),
        }
    }
}
