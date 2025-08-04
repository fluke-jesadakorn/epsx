// Market data HTTP handlers
use std::sync::Arc;
use axum::{
    extract::{Query, State, WebSocketUpgrade},
    routing::get,
    response::IntoResponse,
    Router,
    Json,
};
use utoipa::OpenApi;
use serde::Deserialize;
use tracing::error;

use crate::dom::entities::market_data::{StockScreeningResult, MarketDataError};
use crate::infra::services::tradingview::TradingViewService;

#[derive(OpenApi)]
#[openapi(
    paths(
        stock_screener,
        eps_growth_ranking,
        ws_handler
    ),
    components(
        schemas(EpsGrowthRankingParams, StockScreeningResult)
    ),
    tags(
        (name = "Market Data", description = "Stock screening and market data endpoints")
    )
)]
#[allow(dead_code)]
struct MarketDataApi;

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct EpsGrowthRankingParams {
    /// Maximum number of results to return
    #[schema(example = 10)]
    pub limit: Option<i32>,
    /// Number of results to skip (for pagination)
    #[schema(example = 0)]  
    pub skip: Option<i32>,
    /// Field to sort results by
    #[schema(example = "activityScore")]
    pub sort_by: Option<String>,
}

/// Create market data router with TradingView service
pub fn market_data_router(tradingview_service: Arc<dyn TradingViewService>) -> Router {
    Router::new()
        .route("/market-data/stocks/screener", get(stock_screener))
        .route("/market-data/stocks/eps-growth-ranking", get(eps_growth_ranking))
        .route("/market-data/stocks/screener/ws", get(ws_handler))
        .with_state(tradingview_service)
}

/// Create legacy screener routes (backward compatibility)
pub fn screener_router_legacy(tradingview_service: Arc<dyn TradingViewService>) -> Router {
    Router::new()
        .route("/screener", get(stock_screener))
        .route("/eps-growth-ranking", get(eps_growth_ranking))
        .route("/screener/ws", get(ws_handler))
        .with_state(tradingview_service)
}

/// Get stock screener data for all tracked symbols
#[utoipa::path(
    get,
    path = "/market-data/stocks/screener",
    responses(
        (status = 200, description = "Successfully retrieved screener data", body = Vec<StockScreeningResult>),
        (status = 500, description = "Internal server error", body = MarketDataError)
    ),
    tag = "Market Data"
)]
async fn stock_screener(
    State(tradingview_service): State<Arc<dyn TradingViewService>>,
) -> Result<Json<Vec<StockScreeningResult>>, MarketDataError> {
    let screening_results = tradingview_service.fetch_screener_data().await
        .map_err(|e| MarketDataError::ExternalApiError(e.to_string()))?;
    
    Ok(Json(screening_results))
}

/// WebSocket endpoint for real-time screener updates
#[utoipa::path(
    get,
    path = "/market-data/stocks/screener/ws",
    responses(
        (status = 101, description = "WebSocket connection upgraded successfully"),
        (status = 400, description = "Invalid WebSocket request"),
    ),
    tag = "Market Data"
)]
async fn ws_handler(
    ws: WebSocketUpgrade,
    State(tradingview_service): State<Arc<dyn TradingViewService>>,
) -> impl IntoResponse {
    ws.on_upgrade(move |_socket| async move {
        if let Err(e) = tradingview_service.connect_realtime_feed().await {
            error!("Failed to connect to TradingView screener: {:?}", e);
        }
    })
}

/// Get EPS growth ranking for stocks
#[utoipa::path(
    get,
    path = "/market-data/stocks/eps-growth-ranking",
    params(
        ("limit" = Option<i32>, Query, description = "Maximum number of results to return"),
        ("skip" = Option<i32>, Query, description = "Number of results to skip (for pagination)"),
        ("sort_by" = Option<String>, Query, description = "Field to sort results by")
    ),
    responses(
        (status = 200, description = "Successfully retrieved EPS growth ranking", body = Vec<StockScreeningResult>),
        (status = 500, description = "Internal server error", body = MarketDataError)
    ),
    tag = "Market Data"
)]
async fn eps_growth_ranking(
    State(tradingview_service): State<Arc<dyn TradingViewService>>,
    Query(params): Query<EpsGrowthRankingParams>,
) -> Result<Json<Vec<StockScreeningResult>>, MarketDataError> {
    let screening_results = tradingview_service
        .fetch_eps_growth_ranking(params.limit, params.skip, params.sort_by)
        .await
        .map_err(|e| MarketDataError::ExternalApiError(e.to_string()))?;
    
    Ok(Json(screening_results))
}

// HTTP response conversion for MarketDataError
impl IntoResponse for MarketDataError {
    fn into_response(self) -> axum::response::Response {
        use axum::http::StatusCode;
        
        let status = match self {
            MarketDataError::ExternalApiError(_) => StatusCode::BAD_GATEWAY,
            MarketDataError::NetworkError(_) => StatusCode::SERVICE_UNAVAILABLE,
            MarketDataError::ParsingError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            MarketDataError::ServiceUnavailable(_) => StatusCode::SERVICE_UNAVAILABLE,
            MarketDataError::InvalidInput(_) => StatusCode::BAD_REQUEST,
        };
        
        tracing::error!("Market data error: {}", self);
        (status, self.to_string()).into_response()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;
    use crate::infra::services::tradingview::TradingViewApiService;
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
        let service: Arc<dyn TradingViewService> = Arc::new(TradingViewApiService::new(Arc::new(config)));
        let app = market_data_router(service);

        let status = test_request(app, "/market-data/stocks/screener").await;
        assert_eq!(status, StatusCode::OK);
    }

    #[tokio::test]
    async fn test_eps_growth_ranking() {
        let config = setup_test().await;
        let service: Arc<dyn TradingViewService> = Arc::new(TradingViewApiService::new(Arc::new(config)));
        let app = market_data_router(service);

        let status = test_request(app, "/market-data/stocks/eps-growth-ranking?limit=10&skip=0&sort_by=activityScore").await;
        assert_eq!(status, StatusCode::OK);
    }
}