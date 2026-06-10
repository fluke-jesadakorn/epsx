// Get Cache Statistics Query

use serde::{Deserialize, Serialize};
use crate::application::shared::Query;
use crate::application::market_analytics::dtos::CacheStatsResponse;

#[derive(Debug, Clone)]
pub struct GetCacheStatsQuery {}

impl Query for GetCacheStatsQuery {
    type Response = GetCacheStatsResponse;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetCacheStatsResponse {
    pub stats: CacheStatsResponse,
}
