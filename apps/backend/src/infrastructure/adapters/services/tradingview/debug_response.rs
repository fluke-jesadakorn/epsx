// DEBUG FILE: TradingView API Response Debug
// TEMPORARY - Will be deleted after finding estimate EPS data source

use serde_json;
use std::fs;
use chrono::Utc;

pub fn log_tradingview_response(symbol: &str, response: &crate::infrastructure::adapters::services::tradingview::types::TradingViewResponse) {
    let debug_data = serde_json::json!({
        "timestamp": Utc::now().to_rfc3339(),
        "symbol_focus": symbol,
        "total_stocks": response.data.len(),
        "sample_stock_data": response.data.get(0).map(|stock| {
            serde_json::json!({
                "symbol": stock.s,
                "raw_data_fields": stock.d.iter().enumerate().map(|(idx, field)| {
                    serde_json::json!({
                        "index": idx,
                        "value": field,
                        "field_name": get_field_name(idx)
                    })
                }).collect::<Vec<_>>()
            })
        }),
        "column_mapping": [
            "name", "description", "logoid", "update_mode", "type", "typespecs",
            "close", "pricescale", "minmov", "fractional", "minmove2", "currency",
            "change", "volume", "earnings_per_share_fq", "relative_volume_10d_calc", 
            "market_cap_basic", "fundamental_currency_code", "price_earnings_ttm", 
            "earnings_per_share_diluted_ttm", "earnings_per_share_diluted_yoy_growth_ttm", 
            "dividends_yield_current", "earnings_per_share_forecast_fq", 
            "earnings_per_share_forecast_next_fq", "sector.tr", "market", "sector", 
            "AnalystRating", "AnalystRating.tr", "exchange", "earnings_release_date", 
            "earnings_release_next_date"
        ]
    });

    let debug_file = format!(".devtools/tradingview_response_debug_{}.json", Utc::now().timestamp());
    
    if let Err(e) = fs::write(&debug_file, serde_json::to_string_pretty(&debug_data).unwrap()) {
        eprintln!("Failed to write debug file {}: {}", debug_file, e);
    } else {
        println!("DEBUG: TradingView response saved to {}", debug_file);
    }
}

fn get_field_name(idx: usize) -> &'static str {
    match idx {
        0 => "name",
        1 => "description", 
        2 => "logoid",
        3 => "update_mode",
        4 => "type",
        5 => "typespecs",
        6 => "close",
        7 => "pricescale",
        8 => "minmov",
        9 => "fractional",
        10 => "minmove2",
        11 => "currency",
        12 => "change",
        13 => "volume",
        14 => "earnings_per_share_fq",
        15 => "relative_volume_10d_calc",
        16 => "market_cap_basic",
        17 => "fundamental_currency_code",
        18 => "price_earnings_ttm",
        19 => "earnings_per_share_diluted_ttm",
        20 => "earnings_per_share_diluted_yoy_growth_ttm",
        21 => "dividends_yield_current",
        22 => "earnings_per_share_forecast_fq",
        23 => "earnings_per_share_forecast_next_fq",
        24 => "sector.tr",
        25 => "market",
        26 => "sector",
        27 => "AnalystRating",
        28 => "AnalystRating.tr",
        29 => "exchange",
        30 => "earnings_release_date",
        31 => "earnings_release_next_date",
        _ => "unknown_field"
    }
}