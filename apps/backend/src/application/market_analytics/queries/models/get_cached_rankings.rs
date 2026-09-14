// Get Cached Rankings Query

use crate::application::market_analytics::dtos::{
    EPSRankingQueryParams, UnifiedAnalyticsRankingsResponse,
};
use crate::application::shared::Query;
use serde::{Deserialize, Serialize};

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
