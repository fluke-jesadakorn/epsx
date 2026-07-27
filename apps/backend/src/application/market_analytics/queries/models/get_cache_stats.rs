// Get Cache Statistics Query

use crate::application::market_analytics::dtos::CacheStatsResponse;
use crate::application::shared::Query;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone)]
pub struct GetCacheStatsQuery {}

impl Query for GetCacheStatsQuery {
    type Response = GetCacheStatsResponse;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetCacheStatsResponse {
    pub stats: CacheStatsResponse,
}
