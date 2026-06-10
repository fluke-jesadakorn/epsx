// Admin analytics handlers (CQRS-based)
use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::Json,
};
use std::sync::Arc;

/// System metrics handler for admin dashboard (CQRS-based)
/// GET /api/admin/analytics/metrics
#[utoipa::path(
    get,
    path = "/api/admin/analytics/metrics",
    tag = "admin-analytics",
    responses(
        (status = 200, description = "Successfully retrieved system metrics"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 500, description = "Internal server error")
    ),
    security(("bearerAuth" = []))
)]
pub async fn system_metrics_handler(
    State(state): State<crate::web::auth::AppState>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    use crate::application::shared::QueryHandler;
    use crate::application::market_analytics::queries::{GetSystemMetricsQuery, GetSystemMetricsQueryHandler};

    tracing::info!("System metrics request (CQRS)");

    // Get DB pool from state
    let db_pool = state.db_pool;

    // Create query with optional includes (default to all enabled)
    let query = GetSystemMetricsQuery {
        include_cache: Some(true),
        include_database: Some(true),
        include_external_apis: Some(true),
    };

    // Create TradingView service and handler
    let tradingview_service = Arc::new(
        crate::infrastructure::adapters::services::tradingview::TradingViewApiService::new(
            Arc::new(crate::config::get_fallback_config())
        )
    );
    let handler = GetSystemMetricsQueryHandler::new(tradingview_service, db_pool);

    // Execute query via CQRS handler
    let response = handler.handle(query).await
        .map_err(|e| {
            tracing::error!("System metrics query failed: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    // Convert to JSON response (return whole response)
    let json_response = serde_json::to_value(&response)
        .map_err(|e| {
            tracing::error!("JSON serialization failed: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    tracing::info!("System metrics returned via CQRS handler");
    Ok(Json(json_response))
}

/// Admin time series data handler for dashboard (CQRS-based)
/// GET /api/admin/analytics/time-series
#[utoipa::path(
    get,
    path = "/api/admin/analytics/time-series",
    tag = "admin-analytics",
    responses(
        (status = 200, description = "Successfully retrieved time series data"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 500, description = "Internal server error")
    ),
    params(
        ("start_date" = Option<String>, Query, description = "Start date in RFC3339 format"),
        ("end_date" = Option<String>, Query, description = "End date in RFC3339 format"),
        ("granularity" = Option<String>, Query, description = "Time granularity: hourly, daily, weekly, monthly"),
        ("metric_type" = Option<String>, Query, description = "Metric type: api_requests, cache_hits, database_queries, active_users, ranking_updates")
    ),
    security(("bearerAuth" = []))
)]
pub async fn admin_time_series_handler(
    Query(params): Query<std::collections::HashMap<String, String>>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    use crate::application::shared::QueryHandler;
    use crate::application::market_analytics::queries::{GetAdminTimeSeriesQuery, GetAdminTimeSeriesQueryHandler};
    use crate::application::market_analytics::queries::{TimeSeriesGranularity, MetricType};

    // Parse query parameters with defaults
    let start_date = params.get("start_date")
        .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
        .map(|dt| dt.with_timezone(&chrono::Utc))
        .unwrap_or_else(|| chrono::Utc::now() - chrono::Duration::days(7));

    let end_date = params.get("end_date")
        .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
        .map(|dt| dt.with_timezone(&chrono::Utc))
        .unwrap_or_else(chrono::Utc::now);

    let granularity = params.get("granularity")
        .and_then(|s| match s.as_str() {
            "hourly" => Some(TimeSeriesGranularity::Hourly),
            "daily" => Some(TimeSeriesGranularity::Daily),
            "weekly" => Some(TimeSeriesGranularity::Weekly),
            "monthly" => Some(TimeSeriesGranularity::Monthly),
            _ => None,
        })
        .unwrap_or(TimeSeriesGranularity::Daily);

    let metric_type = params.get("metric_type")
        .and_then(|s| match s.as_str() {
            "api_requests" => Some(MetricType::ApiRequests),
            "cache_hits" => Some(MetricType::CacheHits),
            "database_queries" => Some(MetricType::DatabaseQueries),
            "active_users" => Some(MetricType::ActiveUsers),
            "ranking_updates" => Some(MetricType::RankingUpdates),
            _ => None,
        })
        .unwrap_or(MetricType::ApiRequests);

    tracing::info!("Admin timeseries request (CQRS) - Start: {}, End: {}, Granularity: {:?}",
                  start_date, end_date, granularity);

    // Create query
    let query = GetAdminTimeSeriesQuery {
        start_date,
        end_date,
        granularity,
        metric_type,
    };

    // Create handler (no dependencies)
    let handler = GetAdminTimeSeriesQueryHandler::new();

    // Execute query via CQRS handler
    let response = handler.handle(query).await
        .map_err(|e| {
            tracing::error!("Admin timeseries query failed: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    // Convert to JSON response (return whole response)
    let json_response = serde_json::to_value(&response)
        .map_err(|e| {
            tracing::error!("JSON serialization failed: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    tracing::info!("Admin timeseries returned via CQRS handler");
    Ok(Json(json_response))
}

/// Admin modules data handler for dashboard (CQRS-based)
/// GET /api/admin/analytics/modules
#[utoipa::path(
    get,
    path = "/api/admin/analytics/modules",
    tag = "admin-analytics",
    responses(
        (status = 200, description = "Successfully retrieved admin modules data"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 500, description = "Internal server error")
    ),
    params(
        ("include_inactive" = Option<bool>, Query, description = "Include inactive modules")
    ),
    security(("bearerAuth" = []))
)]
pub async fn admin_modules_handler(
    Query(params): Query<std::collections::HashMap<String, String>>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    use crate::application::shared::QueryHandler;
    use crate::application::market_analytics::queries::{GetAdminModulesQuery, GetAdminModulesQueryHandler};

    // Parse query parameters
    let include_inactive = params.get("include_inactive")
        .and_then(|s| s.parse::<bool>().ok());

    tracing::info!("Admin modules request (CQRS) - Include inactive: {:?}", include_inactive);

    // Create query
    let query = GetAdminModulesQuery {
        include_inactive,
    };

    // Create handler (no dependencies)
    let handler = GetAdminModulesQueryHandler::new();

    // Execute query via CQRS handler
    let response = handler.handle(query).await
        .map_err(|e| {
            tracing::error!("Admin modules query failed: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    // Convert to JSON response (return whole response)
    let json_response = serde_json::to_value(&response)
        .map_err(|e| {
            tracing::error!("JSON serialization failed: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    tracing::info!("Admin modules returned via CQRS handler");
    Ok(Json(json_response))
}
