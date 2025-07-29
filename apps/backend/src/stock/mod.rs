pub mod common;
pub mod financial_data;
pub mod price_data;
pub mod screener;

use axum::Router;
use std::sync::Arc;

pub use financial_data::FinancialDataService;
pub use price_data::PriceDataService;
pub use screener::ScreenerService;

use crate::config::Config;

pub fn stock_router_v1(
    config: &Config,
    _auth_service: Arc<crate::auth::AuthService>
) -> Router {
    let screener_service = Arc::new(ScreenerService::new(config));
    let financial_data_service = Arc::new(FinancialDataService::new(config));
    let price_data_service = Arc::new(PriceDataService::new(config));

    Router::new()
        .merge(screener::screener_router(screener_service))
        .merge(financial_data::financial_data_router(financial_data_service))
        .merge(price_data::price_data_router(price_data_service))
        // Add symbols endpoint
        .route("/market-data/symbols", axum::routing::get(get_symbols_handler))
        .layer(axum::middleware::from_fn(crate::auth::middleware::auth_middleware))
}

// Legacy stock routes removed - use v1 API structure only

/// Placeholder handler for symbols endpoint
async fn get_symbols_handler() -> axum::response::Json<Vec<String>> {
    // This would typically fetch from a database or configuration
    axum::response::Json(vec![
        "AAPL".to_string(),
        "GOOGL".to_string(),
        "MSFT".to_string(),
        "TSLA".to_string(),
    ])
}
