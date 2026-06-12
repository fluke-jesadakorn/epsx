// kernel extraction wave9 — moved verbatim from apps/backend/src/domain/shared_kernel/value_objects/user_limits.rs
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// User limits value object representing API and feature access limits
/// Moved from infrastructure layer to domain layer for clean architecture
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResolvedUserLimits {
    pub wallet_address: Option<Uuid>,
    pub ranking_limit: i32,
    pub api_minute_limit: i32,
    pub daily_limit: i32,
    pub weekly_limit: i32,
    pub monthly_limit: i32,
    pub total_limit: i32,
    pub has_premium_features: bool,
    pub is_admin: bool,
}

impl ResolvedUserLimits {
    pub fn new(ranking_limit: i32, api_minute_limit: i32, has_premium_features: bool, is_admin: bool) -> Self {
        Self {
            wallet_address: None,
            ranking_limit,
            api_minute_limit,
            daily_limit: api_minute_limit * 24,
            weekly_limit: api_minute_limit * 24 * 7,
            monthly_limit: api_minute_limit * 24 * 30,
            total_limit: api_minute_limit * 24 * 365,
            has_premium_features,
            is_admin,
        }
    }
    
    pub fn default_free() -> Self {
        Self::new(3, 10, false, false)
    }
    
    pub fn default_premium() -> Self {
        Self::new(100, 100, true, false)
    }
    
    pub fn default_admin() -> Self {
        Self::new(1000, 1000, true, true)
    }
}

/// User dynamic limit entity representing database-stored limit overrides
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserDynamicLimit {
    pub id: Uuid,
    pub wallet_address: Uuid,
    pub resource: String,
    pub limit_type: String,
    pub limit_value: i32,
    pub window_seconds: i32,
    pub created_at: DateTime<Utc>,
}
