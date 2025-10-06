// Date and Metrics Utilities
// Handles date calculations and analytics metrics

use super::types::QuarterlyData;

/// Determine trend based on growth factor
pub(super) fn determine_trend(growth_factor: f64) -> String {
  if growth_factor > 20.0 {
    "strong_bullish".to_string()
  } else if growth_factor > 5.0 {
    "bullish".to_string()
  } else if growth_factor > -5.0 {
    "neutral".to_string()
  } else if growth_factor > -20.0 {
    "bearish".to_string()
  } else {
    "strong_bearish".to_string()
  }
}

/// Calculate simple volatility from QoQ growth percentage
pub(super) fn calculate_simple_volatility(growth_factor: f64) -> f64 {
  growth_factor.abs().min(50.0)
}

/// Format announcement date from quarterly data
pub(super) fn format_announcement_date_from_quarter_data(
  quarter_data: &QuarterlyData
) -> (Option<String>, bool) {
  let is_estimated = quarter_data.quarter.starts_with("Est.");
  let announcement_date = if
    quarter_data.quarter.starts_with("Est.") ||
    quarter_data.quarter.starts_with("Announced")
  {
    Some(quarter_data.quarter.clone())
  } else {
    Some(quarter_data.date.format("%b %-d, %Y").to_string())
  };
  (announcement_date, is_estimated)
}
