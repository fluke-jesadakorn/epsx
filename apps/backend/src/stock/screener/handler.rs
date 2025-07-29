use std::sync::Arc;
use axum::{
    extract::{Query, State, WebSocketUpgrade},
    routing::get,
    response::IntoResponse,
    Router,
    Json,
};
use utoipa::OpenApi;

#[derive(OpenApi)]
#[openapi(
    paths(
        stock_screener,
        eps_growth_ranking,
        ws_handler
    ),
    components(
        schemas(EpsGrowthRankingParams, TableDataMetrics)
    ),
    tags(
        (name = "Stock Screener", description = "Stock screening and ranking endpoints")
    )
)]
#[allow(dead_code)]
struct ScreenerApi;
use serde::Deserialize;
use tracing::error;

use crate::stock::common::StockServiceError;
use super::{service::ScreenerService, models::TableDataMetrics};

pub fn screener_router(screener_service: Arc<ScreenerService>) -> Router {
    Router::new()
        .route("/market-data/stocks/screener", get(stock_screener))
        .route("/market-data/stocks/eps-growth-ranking", get(eps_growth_ranking))
        .route("/market-data/stocks/screener/ws", get(ws_handler))
        .with_state(screener_service)
}

/// Create legacy screener routes (backward compatibility)
pub fn screener_router_legacy(screener_service: Arc<ScreenerService>) -> Router {
    Router::new()
        .route("/screener", get(stock_screener))
        .route("/eps-growth-ranking", get(eps_growth_ranking))
        .route("/screener/ws", get(ws_handler))
        .with_state(screener_service)
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct EpsGrowthRankingParams {
    /// Maximum number of results to return
    #[schema(example = 10)]
    limit: Option<i32>,
    /// Number of results to skip (for pagination)
    #[schema(example = 0)]
    skip: Option<i32>,
    /// Field to sort results by
    #[schema(example = "activityScore")]
    sort_by: Option<String>,
}

/// Get stock screener data for all tracked symbols
#[utoipa::path(
    get,
    path = "/screener",
    responses(
        (status = 200, description = "Successfully retrieved screener data", body = Vec<TableDataMetrics>),
        (status = 500, description = "Internal server error", body = StockServiceError)
    ),
    tag = "Stock Screener"
)]
async fn stock_screener(
    State(screener_service): State<Arc<ScreenerService>>,
) -> Result<Json<Vec<TableDataMetrics>>, StockServiceError> {
    let data = screener_service.fetch_stock_screener_data().await?;
    Ok(Json(data))
}

/// WebSocket endpoint for real-time screener updates
#[utoipa::path(
    get,
    path = "/ws",
    responses(
        (status = 101, description = "WebSocket connection upgraded successfully"),
        (status = 400, description = "Invalid WebSocket request"),
    ),
    tag = "Stock Screener"
)]
async fn ws_handler(
    ws: WebSocketUpgrade,
    State(screener_service): State<Arc<ScreenerService>>,
) -> impl IntoResponse {
    ws.on_upgrade(move |_socket| async move {
        if let Err(e) = screener_service.connect_screener().await {
            error!("Failed to connect to TradingView screener: {:?}", e);
        }
    })
}

/// Get EPS growth ranking for stocks
#[utoipa::path(
    get,
    path = "/eps-growth-ranking",
    params(
        ("limit" = Option<i32>, Query, description = "Maximum number of results to return"),
        ("skip" = Option<i32>, Query, description = "Number of results to skip (for pagination)"),
        ("sort_by" = Option<String>, Query, description = "Field to sort results by")
    ),
    responses(
        (status = 200, description = "Successfully retrieved EPS growth ranking", body = Vec<TableDataMetrics>),
        (status = 500, description = "Internal server error", body = StockServiceError)
    ),
    tag = "Stock Screener"
)]
async fn eps_growth_ranking(
    State(screener_service): State<Arc<ScreenerService>>,
    Query(params): Query<EpsGrowthRankingParams>,
) -> Result<Json<Vec<TableDataMetrics>>, StockServiceError> {
    let data = screener_service
        .fetch_eps_growth_ranking(params.limit, params.skip, params.sort_by)
        .await?;
    Ok(Json(data))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;
    use axum::{
        body::Body,
        http::{Request, StatusCode},
    };
    use tower_service::Service;
    use std::{env, pin::Pin};

    async fn setup_test() -> Config {
        env::set_var("DATABASE_URL", "postgresql://localhost:5432/test_db");
        env::set_var("TRADINGVIEW_AUTH_TOKEN", "test_token");
        env::set_var("MUSEPAY_PARTNER_ID", "test_partner");
        env::set_var("MUSEPAY_PRIVATE_KEY", "test_key");
        env::set_var("FIREBASE_SERVICE_ACCOUNT_PATH", "./test-service-account.json");
        
        let config = Config::from_env();
        
        config
    }

    async fn test_request(mut app: Router, uri: &str) -> StatusCode {
        let response = Pin::new(&mut app)
            .call(
                Request::builder()
                    .uri(uri)
                    .body(Body::empty())
                    .unwrap()
            )
            .await
            .unwrap();
        
        response.status()
    }

    #[tokio::test]
    async fn test_stock_screener() {
        let config = setup_test().await;
        let service = Arc::new(ScreenerService::new(&config));
        let app = screener_router(service);

        let status = test_request(app, "/screener").await;
        assert_eq!(status, StatusCode::OK);
    }

    #[tokio::test]
    async fn test_eps_growth_ranking() {
        let config = setup_test().await;
        let service = Arc::new(ScreenerService::new(&config));
        let app = screener_router(service);

        let status = test_request(app, "/eps-growth-ranking?limit=10&skip=0&sort_by=activityScore").await;
        assert_eq!(status, StatusCode::OK);
    }
}
