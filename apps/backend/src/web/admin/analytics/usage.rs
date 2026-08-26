use axum::{
    extract::{Query, State},
    response::IntoResponse,
};
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use tracing::{error, info};
use uuid::Uuid;

use crate::web::auth::AppState;
use crate::web::responses::wrappers::AdminResponse;

#[derive(Debug, Deserialize)]
pub struct UsageQuery {
    #[serde(rename = "timeRange")]
    pub time_range: Option<String>,
    #[serde(rename = "apiKey")]
    pub api_key: Option<String>,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct UsageData {
    pub date: DateTime<Utc>,
    pub requests: i64,
    pub errors: i64,
    pub latency: f64,
}

pub async fn get_usage_analytics_handler(
    Query(query): Query<UsageQuery>,
    State(app_state): State<AppState>,
) -> axum::response::Response {
    info!("Admin: Getting usage analytics");

    let period_days = match query.time_range.as_deref() {
        Some("24h") => 1,
        Some("7d") => 7,
        Some("30d") => 30,
        _ => 7,
    };

    let start_date = Utc::now() - Duration::days(period_days);

    // Parse API Key UUID if present and not "all"
    let api_key_uuid = match query.api_key.as_deref() {
        Some("all") => None,
        Some("") => None,
        Some(s) => Uuid::parse_str(s).ok(),
        None => None,
    };

    let pool = if let Some(analytics) = &app_state.analytics_db_pool {
        analytics
    } else {
        &app_state.db_pool
    };

    let usage_data: Vec<UsageData> = if let Some(uuid) = api_key_uuid {
        sqlx::query_as::<_, UsageData>(
            r#"
            SELECT 
                date_trunc('day', request_at) as date,
                COUNT(*)::BIGINT as requests,
                COUNT(*) FILTER (WHERE response_status >= 400)::BIGINT as errors,
                COALESCE(AVG(response_time_ms), 0.0)::DOUBLE PRECISION as latency
            FROM api_key_usage_logs
            WHERE request_at >= $1 AND api_key_id = $2
            GROUP BY date
            ORDER BY date ASC
            "#,
        )
        .bind(start_date)
        .bind(uuid)
        .fetch_all(pool.as_ref())
        .await
        .unwrap_or_default()
    } else {
        sqlx::query_as::<_, UsageData>(
            r#"
            SELECT 
                date_trunc('day', request_at) as date,
                COUNT(*)::BIGINT as requests,
                COUNT(*) FILTER (WHERE response_status >= 400)::BIGINT as errors,
                COALESCE(AVG(response_time_ms), 0.0)::DOUBLE PRECISION as latency
            FROM api_key_usage_logs
            WHERE request_at >= $1
            GROUP BY date
            ORDER BY date ASC
            "#,
        )
        .bind(start_date)
        .fetch_all(pool.as_ref())
        .await
        .unwrap_or_default()
    };

    AdminResponse::success_with_message(usage_data, "Usage analytics retrieved successfully")
        .into_response()
}
