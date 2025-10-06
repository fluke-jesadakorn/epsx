// Get Cache Health Query

use serde::{Deserialize, Serialize};
use crate::application::shared::Query;
use crate::application::trading_analytics::dtos::CacheHealthResponse;

#[derive(Debug, Clone)]
pub struct GetCacheHealthQuery {}

impl Query for GetCacheHealthQuery {
    type Response = GetCacheHealthResponse;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetCacheHealthResponse {
    pub health: CacheHealthResponse,
}
