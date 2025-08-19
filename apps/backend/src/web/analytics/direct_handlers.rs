// Direct TradingView API handlers for comprehensive analytics
use axum::{
    extract::Query,
    response::Json,
};
use tracing::{info, error};

use crate::core::errors::AppError;
use super::tradingview_direct::*;

/// GET /api/analytics/rankings - Main unified rankings endpoint
/// Returns EPS rankings in card format with comprehensive TradingView API integration
pub async fn handle_unified_rankings(
    Query(params): Query<TradingViewQueryParams>
) -> Result<Json<CardDashboardResponse>, AppError> {
    let start_time = std::time::Instant::now();
    
    info!("Direct TradingView API request - Country: {:?}, Page: {:?}, Limit: {:?}", 
          params.country, params.page, params.limit);
    
    let request_body = build_tradingview_request(&params);
    let tv_response = match call_tradingview_api(&request_body).await {
        Ok(response) => response,
        Err(e) => {
            error!("TradingView API error: {}", e);
            return Err(AppError::new(
                crate::core::errors::ErrorKind::ExternalServiceError,
                format!("TradingView API error: {}", e)
            ));
        }
    };
    
    let card_data = convert_to_card_data(tv_response, &params);
    let total_count = card_data.len() as i64;
    let page = params.page.unwrap_or(1);
    let limit = params.limit.unwrap_or(20);
    let total_pages = ((total_count as f64) / (limit as f64)).ceil() as i32;
    
    let response = CardDashboardResponse {
        success: true,
        data: card_data,
        pagination: PaginationResponse {
            page,
            limit,
            total: total_count,
            total_pages,
            has_next: page < total_pages,
            has_prev: page > 1,
        },
        metadata: MetadataResponse {
            available_countries: get_all_markets(),
            available_sectors: get_all_sectors(),
            request_timestamp: chrono::Utc::now().to_rfc3339(),
            data_source: "live_tradingview_api".to_string(),
        },
        message: Some(format!("Fetched {} results from TradingView API", total_count)),
        processing_time_ms: start_time.elapsed().as_millis() as u64,
    };
    
    info!("TradingView API response completed - {} results in {}ms", 
          total_count, start_time.elapsed().as_millis());
    
    Ok(Json(response))
}

/// GET /api/analytics/eps-rankings - EPS rankings endpoint (alias for unified rankings)
pub async fn handle_eps_rankings(
    Query(params): Query<TradingViewQueryParams>
) -> Result<Json<CardDashboardResponse>, AppError> {
    handle_unified_rankings(Query(params)).await
}

/// GET /api/analytics/eps-rankings/countries - Available countries with labels
pub async fn handle_available_countries() -> Json<serde_json::Value> {
    let countries = get_countries_with_labels();
    
    Json(serde_json::json!({
        "countries": countries,
        "count": countries.len(),
        "message": "Available countries from TradingView API"
    }))
}

/// GET /api/analytics/eps-rankings/countries/all - All valid countries
pub async fn handle_all_valid_countries() -> Json<serde_json::Value> {
    let countries = get_all_markets();
    Json(serde_json::json!({
        "countries": countries,
        "count": countries.len(),
        "message": "All valid countries from TradingView API"
    }))
}

/// GET /api/analytics/eps-rankings/sectors - Available sectors
pub async fn handle_sectors_by_country() -> Json<serde_json::Value> {
    let sectors = get_all_sectors();
    Json(serde_json::json!({
        "sectors": sectors,
        "count": sectors.len(),
        "message": "Available sectors"
    }))
}

/// GET /api/analytics/eps-rankings/exchanges - Available exchanges
pub async fn handle_available_exchanges() -> Json<serde_json::Value> {
    let exchanges = get_all_exchanges();
    Json(serde_json::json!({
        "exchanges": exchanges,
        "count": exchanges.len(),
        "message": "Available exchanges"
    }))
}

/// GET /api/analytics/eps-rankings/stock-types - Available stock types
pub async fn handle_stock_types() -> Json<serde_json::Value> {
    let stock_types = get_stock_types();
    Json(serde_json::json!({
        "stock_types": stock_types,
        "count": stock_types.len(),
        "message": "Available stock types"
    }))
}

/// GET /api/analytics/eps-rankings/health - Health check endpoint
pub async fn handle_eps_health_check() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "healthy": true,
        "message": "TradingView analytics API is operational",
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "available_markets": get_all_markets().len(),
        "available_sectors": get_all_sectors().len(),
        "available_exchanges": get_all_exchanges().len(),
        "supported_filters": [
            "country", "sector", "exchange", "stock_type",
            "min_eps", "max_eps", "min_growth", "max_growth",
            "min_market_cap", "max_market_cap", "min_volume", "max_volume",
            "min_price", "max_price", "min_pe_ratio", "max_pe_ratio",
            "min_dividend_yield", "max_dividend_yield"
        ],
        "supported_sort_options": [
            "eps_growth", "current_eps", "market_cap", "volume", "price",
            "pe_ratio", "dividend_yield", "change", "relative_volume", "symbol"
        ]
    }))
}