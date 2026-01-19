// TradingView data extraction utilities
// Shared helper functions for extracting data from TradingView API responses

use super::types::StockDataField;

/// Extract a number from TradingView stock data array
/// Returns the numeric value as f64, or None if the field doesn't exist or isn't numeric
pub fn get_number_opt(data: &[StockDataField], idx: usize) -> Option<f64> {
    match data.get(idx) {
        Some(StockDataField::Number(n)) => Some(*n),
        Some(StockDataField::Integer(i)) => Some(*i as f64),
        _ => None,
    }
}

/// Extract a number from TradingView stock data array
/// Returns the numeric value as f64, or 0.0 if the field doesn't exist or isn't numeric
pub fn get_number(data: &[StockDataField], idx: usize) -> f64 {
    get_number_opt(data, idx).unwrap_or(0.0)
}

/// Extract a string from TradingView stock data array
/// Returns the string representation of the field, or the default value if not found
pub fn get_string(data: &[StockDataField], idx: usize, default: &str) -> String {
    data.get(idx)
        .map(|field| {
            match field {
                StockDataField::String(s) => s.clone(),
                StockDataField::Number(n) => n.to_string(),
                StockDataField::Integer(i) => i.to_string(),
                StockDataField::Boolean(b) => b.to_string(),
                StockDataField::Array(_) => "Array".to_string(),
                StockDataField::Object(_) => "Object".to_string(),
                StockDataField::Null => default.to_string(),
            }
        })
        .unwrap_or_else(|| default.to_string())
}

/// Extract symbol from full TradingView symbol format (e.g., "NASDAQ:AAPL" -> "AAPL")
/// Returns the symbol part after the colon, or the full string if no colon is found
pub fn extract_symbol(full_symbol: &str) -> String {
    full_symbol
        .split(':')
        .nth(1)
        .unwrap_or_else(|| full_symbol)
        .to_string()
}
