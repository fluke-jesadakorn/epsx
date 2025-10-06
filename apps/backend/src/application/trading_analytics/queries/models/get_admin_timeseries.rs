// Get Admin Time Series Query
// Time-bucketed analytics data for admin dashboards

use serde::{Deserialize, Serialize};
use crate::application::shared::Query;

#[derive(Debug, Clone)]
pub struct GetAdminTimeSeriesQuery {
    pub start_date: chrono::DateTime<chrono::Utc>,
    pub end_date: chrono::DateTime<chrono::Utc>,
    pub granularity: TimeSeriesGranularity,
    pub metric_type: MetricType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TimeSeriesGranularity {
    Hourly,
    Daily,
    Weekly,
    Monthly,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MetricType {
    ApiRequests,
    CacheHits,
    DatabaseQueries,
    ActiveUsers,
    RankingUpdates,
}

impl Query for GetAdminTimeSeriesQuery {
    type Response = GetAdminTimeSeriesResponse;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetAdminTimeSeriesResponse {
    pub success: bool,
    pub data_points: Vec<TimeSeriesDataPoint>,
    pub summary: TimeSeriesSummary,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeSeriesDataPoint {
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub value: f64,
    pub count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeSeriesSummary {
    pub total: i64,
    pub average: f64,
    pub min: f64,
    pub max: f64,
    pub trend: String,
}
