use std::sync::Arc;
use axum::{
    extract::{Query, State, WebSocketUpgrade},
    routing::get,
    response::IntoResponse,
    Router,
    Json,
};
use serde::Deserialize;
use tracing::error;

use crate::stock::error::StockServiceError;
use crate::stock::models::{TableDataMetrics, EpsGrowthRankingParams};
use crate::infra::services::tradingview::TradingViewService;

pub fn screener_router(tradingview_service: Arc<dyn TradingViewService>) -> Router {
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
async fn stock_screener(
    State(tradingview_service): State<Arc<dyn TradingViewService>>,
) -> Result<Json<Vec<TableDataMetrics>>, StockServiceError> {
    let data = tradingview_service.fetch_screener_data().await?;
    Ok(Json(data))
}

/// WebSocket endpoint for real-time screener updates
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
async fn eps_growth_ranking(
    State(tradingview_service): State<Arc<dyn TradingViewService>>,
    Query(params): Query<EpsGrowthRankingParams>,
) -> Result<Json<Vec<TableDataMetrics>>, StockServiceError> {
    let (data, _total_count) = tradingview_service
        .fetch_eps_growth_ranking(
            params.skip, 
            params.limit, 
            params.country, 
            params.sector,  
            params.sort_by
        )
        .await?;
    Ok(Json(data))
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
        let app = screener_router(service);

        let status = test_request(app, "/screener").await;
        assert_eq!(status, StatusCode::OK);
    }

    #[tokio::test]
    async fn test_eps_growth_ranking() {
        let config = setup_test().await;
        let service: Arc<dyn TradingViewService> = Arc::new(TradingViewApiService::new(Arc::new(config)));
        let app = screener_router(service);

        let status = test_request(app, "/eps-growth-ranking?limit=10&skip=0&sort_by=activityScore").await;
        assert_eq!(status, StatusCode::OK);
    }
}
