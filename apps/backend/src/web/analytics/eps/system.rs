// System Utilities
// System mode calculation and configuration helpers

use super::types::QuarterlyPerformanceData;

/// Calculate system mode (TRACK/STOP/WATCH) based on quarterly performance patterns
pub(super) fn calculate_system_mode(
  quarterly_performance: &[QuarterlyPerformanceData]
) -> String {
  if quarterly_performance.is_empty() {
    return "STOP".to_string();
  }

  let recent_quarters: Vec<&QuarterlyPerformanceData> = quarterly_performance
    .iter()
    .take(3)
    .collect();

  if recent_quarters.is_empty() {
    return "STOP".to_string();
  }

  let positive_count = recent_quarters
    .iter()
    .filter(|q| q.eps_growth > 0.0 && q.price_growth > 0.0)
    .count();

  let eps_positive_count = recent_quarters
    .iter()
    .filter(|q| q.eps_growth > 0.0)
    .count();

  match recent_quarters.len() {
    1 => {
      if recent_quarters[0].eps_growth > 0.0 {
        "TRACK".to_string()
      } else {
        "STOP".to_string()
      }
    }
    2 => {
      if positive_count >= 2 || eps_positive_count >= 2 || (eps_positive_count == 1 && recent_quarters[0].eps_growth > 0.0) {
        "TRACK".to_string()
      } else {
        "STOP".to_string()
      }
    }
    _ => {
      if positive_count >= 2 || eps_positive_count >= 2 || (eps_positive_count == 1 && recent_quarters[0].eps_growth > 0.0) {
        "TRACK".to_string()
      } else {
        "STOP".to_string()
      }
    }
  }
}

/// Simple fallback config for TradingView operations
pub(super) fn get_simple_fallback_config() -> crate::config::Config {
  crate::config::get_fallback_config()
}
