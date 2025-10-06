use crate::prelude::*;

/// Plan features value object
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlanFeatures {
    api_calls_limit: Option<i32>,
    rankings_limit: Option<i32>,
    analytics_enabled: bool,
    premium_support: bool,
    custom_features: serde_json::Value,
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
        }
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
}

impl Default for PlanFeatures {
    fn default() -> Self {
        Self {
            api_calls_limit: Some(100),
            rankings_limit: Some(3),
            analytics_enabled: false,
            premium_support: false,
            custom_features: serde_json::json!({}),
        }
    }
}
