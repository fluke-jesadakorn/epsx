// Get Cached Rankings Query

use serde::{Deserialize, Serialize};
use crate::application::shared::Query;
use crate::application::trading_analytics::dtos::{
    UnifiedAnalyticsRankingsResponse,
    EPSRankingQueryParams,
};

#[derive(Debug, Clone)]
pub struct GetCachedRankingsQuery {
    pub params: EPSRankingQueryParams,
    pub user_permissions: Vec<String>,
}

impl Query for GetCachedRankingsQuery {
    type Response = GetCachedRankingsResponse;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetCachedRankingsResponse {
    pub rankings: UnifiedAnalyticsRankingsResponse,
}
