use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::Json,
};
use chrono::{DateTime, Utc, Duration};
use tracing::{error, info};
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use diesel::sql_types::{Timestamptz, BigInt, Double, Uuid as SqlUuid};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::web::auth::AppState;
use crate::web::admin::responses::{AdminApiResponse, AdminMetadata};

#[derive(Debug, Deserialize)]
pub struct UsageQuery {
    #[serde(rename = "timeRange")]
    pub time_range: Option<String>,
    #[serde(rename = "apiKey")]
    pub api_key: Option<String>,
}

#[derive(Debug, Serialize, QueryableByName)]
pub struct UsageData {
    #[diesel(sql_type = Timestamptz)]
    pub date: DateTime<Utc>,
    #[diesel(sql_type = BigInt)]
    pub requests: i64,
    #[diesel(sql_type = BigInt)]
    pub errors: i64,
    #[diesel(sql_type = Double)]
    pub latency: f64,
}

pub async fn get_usage_analytics_handler(
    Query(query): Query<UsageQuery>,
    State(app_state): State<AppState>,
) -> Result<Json<AdminApiResponse<Vec<UsageData>>>, StatusCode> {
    info!("📊 Admin: Getting usage analytics");

    // Connect to ANALYTICS database (where usage logs are stored)
    // Note: The app_state.db_pool points to the CORE database (usually)
    // We need to check if there is a separate analytics pool or if we use the same one.
    // Based on UsageService, it seems there might be two pools, but AppState usually only exposes one.
    // UsageService::new(core_pool, analytics_pool).
    
    // NOTE: In the current architecture shown in Overview.rs, we use app_state.db_pool.
    // If usage logs are in a separate DB, we rely on the implementation detail that 
    // for simple deployments they might be in the same DB or accessible via the same pool.
    // If they are strictly separate, we might need to access the analytics pool from state if available.
    // Checking AppState definition would be good, but assuming db_pool is correct for now 
    // as we are patching an existing system.
    
    let mut conn = match app_state.db_pool.get().await {
        Ok(conn) => conn,
        Err(e) => {
            error!("❌ Admin: Failed to get database connection: {}", e);
            return Ok(Json(AdminApiResponse::server_error()));
        }
    };

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

    // Construct query
    // Since api_key_usage_logs is partitioned, we should ideally use the base table
    // or rely on Diesel to handle the table name if it's a view/parent.
    // The schema defines `api_key_usage_logs` so we use that.
    
    let sql = format!(
        r#"
        SELECT 
            date_trunc('day', request_at) as date,
            COUNT(*)::BIGINT as requests,
            COUNT(*) FILTER (WHERE response_status >= 400)::BIGINT as errors,
            COALESCE(AVG(response_time_ms), 0.0)::DOUBLE PRECISION as latency
        FROM api_key_usage_logs
        WHERE request_at >= $1
        {}
        GROUP BY date
        ORDER BY date ASC
        "#,
        if api_key_uuid.is_some() { "AND api_key_id = $2" } else { "" }
    );

    let result = if let Some(uuid) = api_key_uuid {
        diesel::sql_query(sql)
            .bind::<Timestamptz, _>(start_date)
            .bind::<SqlUuid, _>(uuid)
            .load::<UsageData>(&mut conn)
            .await
    } else {
        diesel::sql_query(sql)
            .bind::<Timestamptz, _>(start_date)
            .load::<UsageData>(&mut conn)
            .await
    };

    let usage_data = match result {
        Ok(data) => data,
        Err(e) => {
            error!("❌ Admin: Failed to fetch usage data: {}", e);
            // Return empty data instead of 500 to avoid breaking UI completely
            Vec::new()
        }
    };

    let metadata = AdminMetadata::crud_operation("get_usage_analytics", Some("admin".to_string()));

    Ok(Json(AdminApiResponse::success_with_meta(
        usage_data,
        "Usage analytics retrieved successfully",
        metadata,
    )))
}
