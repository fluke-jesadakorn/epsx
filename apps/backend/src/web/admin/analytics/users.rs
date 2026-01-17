use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::Json,
};
use chrono::{Utc, Duration};
use tracing::{error, info};
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use crate::web::auth::AppState;
use crate::web::admin::responses::{AdminApiResponse, AdminMetadata};
use super::types::*;

/**
 * Get user analytics
 * GET /admin/analytics/users
 */
pub async fn get_user_analytics_handler(
    Query(query): Query<AnalyticsQuery>,
    State(app_state): State<AppState>,
) -> Result<Json<AdminApiResponse<UserAnalyticsResponse>>, StatusCode> {
    info!("👥 Admin: Getting user analytics");

    let mut conn = match app_state.db_pool.get().await {
        Ok(conn) => conn,
        Err(e) => {
            error!("❌ Admin: Failed to get database connection: {}", e);
            return Ok(Json(AdminApiResponse::server_error()));
        }
    };

    let period_days = match query.period.as_deref() {
        Some("7d") => 7,
        Some("30d") => 30,
        Some("90d") => 90,
        Some("1y") => 365,
        _ => 30,
    };

    #[derive(QueryableByName)]
    struct UserCounts {
        #[diesel(sql_type = diesel::sql_types::BigInt)]
        total_users: i64,
        #[diesel(sql_type = diesel::sql_types::BigInt)]
        active_users: i64,
    }

    // Get basic user counts
    let user_counts = match diesel::sql_query(
        "SELECT
            COUNT(*)::bigint as total_users,
            COUNT(*) FILTER (WHERE is_active = true)::bigint as active_users
         FROM wallet_users"
    )
    .get_result::<UserCounts>(&mut conn)
    .await
    {
        Ok(counts) => counts,
        Err(e) => {
            error!("❌ Admin: Failed to fetch user counts: {}", e);
            return Ok(Json(AdminApiResponse::server_error()));
        }
    };

    // Tier distribution removed - tier_level column deleted in migration #023
    // Returns empty vector for backwards compatibility
    let tier_distribution: Vec<TierStats> = Vec::new();

    // Generate time series data for registrations (temporary until activity tracking is implemented)
    let mut new_registrations = Vec::new();
    for i in 0..period_days {
        let date = Utc::now() - Duration::days(period_days - i - 1);
        new_registrations.push(TimeSeriesPoint {
            timestamp: date,
            value: (5.0 + (i as f64 % 10.0)), // Mock data - replace with real query
            label: date.format("%Y-%m-%d").to_string(),
        });
    }

    // Generate user activity data (temporary - replace with real activity tracking)
    let mut user_activity = Vec::new();
    for i in 0..period_days {
        let date = Utc::now() - Duration::days(period_days - i - 1);
        user_activity.push(TimeSeriesPoint {
            timestamp: date,
            value: user_counts.active_users as f64 * (0.8 + ((i as f64 % 5.0) / 10.0)),
            label: date.format("%Y-%m-%d").to_string(),
        });
    }

    #[derive(QueryableByName)]
    struct CohortRow {
        #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Text>)]
        cohort_period: Option<String>,
        #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Integer>)]
        cohort_size: Option<i32>,
        #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Double>)]
        retention_0m: Option<f64>,
        #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Double>)]
        retention_1m: Option<f64>,
        #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Double>)]
        retention_2m: Option<f64>,
    }

    // Implement cohort analysis (monthly cohorts with 3-month retention)
    let retention_cohorts = match diesel::sql_query(
        r#"
        WITH monthly_cohorts AS (
            SELECT
                DATE_TRUNC('month', created_at) as cohort_month,
                wallet_address
            FROM wallet_users
            WHERE created_at >= NOW() - INTERVAL '6 months'
        ),
        cohort_sizes AS (
            SELECT cohort_month, COUNT(*) as cohort_size
            FROM monthly_cohorts
            GROUP BY cohort_month
        ),
        cohort_retention AS (
            SELECT
                mc.cohort_month,
                cs.cohort_size,
                COUNT(DISTINCT CASE WHEN wu.last_auth_at >= mc.cohort_month THEN wu.wallet_address END) as retained_0m,
                COUNT(DISTINCT CASE WHEN wu.last_auth_at >= mc.cohort_month + INTERVAL '1 month' THEN wu.wallet_address END) as retained_1m,
                COUNT(DISTINCT CASE WHEN wu.last_auth_at >= mc.cohort_month + INTERVAL '2 months' THEN wu.wallet_address END) as retained_2m
            FROM monthly_cohorts mc
            JOIN cohort_sizes cs ON mc.cohort_month = cs.cohort_month
            LEFT JOIN wallet_users wu ON mc.wallet_address = wu.wallet_address
            GROUP BY mc.cohort_month, cs.cohort_size
        )
        SELECT
            TO_CHAR(cohort_month, 'YYYY-MM') as cohort_period,
            cohort_size::int as cohort_size,
            (retained_0m::float / cohort_size * 100) as retention_0m,
            (retained_1m::float / NULLIF(cohort_size, 0) * 100) as retention_1m,
            (retained_2m::float / NULLIF(cohort_size, 0) * 100) as retention_2m
        FROM cohort_retention
        ORDER BY cohort_month DESC
        "#
    )
    .load::<CohortRow>(&mut conn)
    .await
    {
        Ok(results) => results
            .into_iter()
            .map(|row| CohortData {
                cohort_period: row.cohort_period.unwrap_or_else(|| "Unknown".to_string()),
                cohort_size: row.cohort_size.unwrap_or(0),
                retention_periods: vec![
                    row.retention_0m.unwrap_or(100.0),
                    row.retention_1m.unwrap_or(0.0),
                    row.retention_2m.unwrap_or(0.0),
                ],
            })
            .collect(),
        Err(_) => Vec::new(),
    };

    let response = UserAnalyticsResponse {
        total_users: user_counts.total_users as i32,
        active_users: user_counts.active_users as i32,
        new_registrations,
        user_activity,
        tier_distribution,
        retention_cohorts,
        geographic_distribution: Vec::new(), // Geographic data not available (no IP/location tracking)
    };

    let metadata = AdminMetadata::crud_operation("get_user_analytics", Some("admin".to_string()));

    info!("✅ Admin: Successfully retrieved user analytics");
    Ok(Json(AdminApiResponse::success_with_meta(
        response,
        "User analytics retrieved successfully",
        metadata,
    )))
}
