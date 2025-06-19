pub mod common;
pub mod financial_data;
pub mod price_data;
pub mod screener;

use axum::Router;
use std::sync::Arc;

use crate::db::DB;

pub use financial_data::FinancialDataService;
pub use price_data::PriceDataService;
pub use screener::ScreenerService;

use crate::config::Config;

pub fn stock_router(config: &Config, db: Arc<DB>) -> Router {
    let screener_service = Arc::new(ScreenerService::new(config, db.clone()));
    let financial_data_service = Arc::new(FinancialDataService::new(config));
    let price_data_service = Arc::new(PriceDataService::new(config));

    Router::new()
        .merge(screener::screener_router(screener_service))
        .merge(financial_data::financial_data_router(financial_data_service))
        .merge(price_data::price_data_router(price_data_service))
}
