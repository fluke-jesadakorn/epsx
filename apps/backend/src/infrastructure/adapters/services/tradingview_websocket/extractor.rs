// EPS Data Extraction
// Extracts quarterly EPS data from TradingView ST4 format

use serde_json::Value;
use tracing::{info, warn};
use chrono::{Datelike, TimeZone};

use super::types::{QuarterlyEPSData, EPSWebSocketData, PriceData};

/// Extract EPS from ST4 array format
pub fn extract_eps_from_st4(
  st4_array: &[Value],
  symbol: &str,
  debug: bool,
  correlate_fn: impl Fn(Vec<QuarterlyEPSData>) -> Vec<QuarterlyEPSData>
) -> Vec<QuarterlyEPSData> {
  let mut quarterly_data: Vec<QuarterlyEPSData> = Vec::new();

  if debug && st4_array.len() >= 2 {
    let sample = &st4_array[..2.min(st4_array.len())];
    if let Ok(json_str) = serde_json::to_string_pretty(&sample) {
      info!("ST4 data structure: {}", json_str);
    }
  }

  for (index, entry) in st4_array.iter().enumerate() {
    if let Some(v) = entry["v"].as_array() {
      let i_value = entry["i"].as_i64().unwrap_or(0);

      if debug && index < 3 {
        info!("ST4 Entry {}: i={}, {} fields total", index, i_value, v.len());
        for (idx, val) in v.iter().enumerate() {
          let val_str = format_value_debug(val);
          info!("      v[{}] = {}", idx, val_str);
        }
      }

      let earnings_timestamp = v.first()
        .and_then(|v| v.as_f64())
        .map(|f| f as i64)
        .unwrap_or(0);
      let estimated_eps = v.get(1).and_then(|v| v.as_f64());
      let quarter_end_timestamp = v.get(3)
        .and_then(|v| v.as_f64())
        .map(|f| f as i64);
      let earnings_announcement_timestamp_ms = v.get(4)
        .and_then(|v| v.as_f64())
        .map(|f| f as i64);
      let actual_eps = v.get(5).and_then(|v| v.as_f64());

      let is_future_earnings = i_value > 100;
      if is_future_earnings {
        info!("FOUND FUTURE EARNINGS: i={}, announcement timestamp: {:?}",
              i_value, earnings_announcement_timestamp_ms);
      }

      let mut actual_eps_value = None;

      if is_future_earnings {
        if let Some(eps) = estimated_eps {
          if eps < 1e50 && is_valid_quarterly_eps(eps) {
            actual_eps_value = Some(eps);
            info!("Future earnings estimated EPS: {}", eps);
          }
        }
      } else {
        if let Some(eps) = actual_eps {
          if v.len() > 5 && is_valid_quarterly_eps(eps) {
            actual_eps_value = Some(eps);
          }
        }

        if actual_eps_value.is_none() {
          if let Some(eps) = estimated_eps {
            if v.len() > 1 && is_valid_quarterly_eps(eps) {
              actual_eps_value = Some(eps);
            }
          }
        }
      }

      if let Some(eps_value) = actual_eps_value {
        let fiscal_period = timestamp_to_fiscal_period(earnings_timestamp);
        let estimated_earnings_date = earnings_announcement_timestamp_ms
          .map(|ms| ms / 1000)
          .unwrap_or(earnings_timestamp);

        let quarter_end_date = quarter_end_timestamp
          .map(|ts| {
            chrono::Utc.timestamp_opt(ts, 0)
              .single()
              .unwrap_or_default()
              .format("%Y-%m-%d")
              .to_string()
          });

        let stored_estimated_eps = estimated_eps.filter(|&est| (est - eps_value).abs() > 0.01);
        let fiscal_quarter_number = extract_fiscal_quarter_number(earnings_timestamp);

        quarterly_data.push(QuarterlyEPSData {
          quarter_number: fiscal_quarter_number,
          period: fiscal_period.clone(),
          actual_eps: eps_value,
          timestamp: earnings_timestamp,
          estimated_eps: stored_estimated_eps,
          is_reported: !is_future_earnings,
          beat_estimate: None,
          eps_type: if is_future_earnings { "st4_future_estimate".to_string() } else { "st4_earnings_study".to_string() },
          source: if is_future_earnings { "st4_future_calendar".to_string() } else { "st4_earnings_data".to_string() },
          quarter_end_date: quarter_end_date.clone(),
          estimated_earnings_date: earnings_announcement_timestamp_ms.map(|ms| ms / 1000),
          price_data: None,
          eps: eps_value,
          quarter_name: fiscal_period.clone(),
        });

        if is_future_earnings {
          info!("Extracted FUTURE EPS: {} = {} (estimated) [announcement: {}, quarter_end: {:?}]",
                fiscal_period, eps_value, estimated_earnings_date, quarter_end_date);
        } else {
          info!("Extracted EPS: {} = {} (from st4 v[5]) [announcement: {}, quarter_end: {:?}]",
                fiscal_period, eps_value, estimated_earnings_date, quarter_end_date);
        }
      }
    }
  }

  if !quarterly_data.is_empty() {
    quarterly_data.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
    quarterly_data = correlate_fn(quarterly_data);
  } else {
    warn!("No EPS data extracted from st4 for {}", symbol);
  }

  quarterly_data
}

/// Build WebSocket EPS data structure
pub fn build_eps_websocket_data(
  symbol: &str,
  quarterly_data: Vec<QuarterlyEPSData>,
  price_data: Vec<PriceData>,
  sector: String,
  country: String,
  company_name: String
) -> EPSWebSocketData {
  let current_eps = quarterly_data.first().map(|q| q.actual_eps).unwrap_or(0.0);
  let quarterly_eps = quarterly_data.first().map(|q| q.actual_eps).unwrap_or(0.0);
  let historical_eps: Vec<f64> = quarterly_data.iter().map(|q| q.actual_eps).collect();

  EPSWebSocketData {
    symbol: symbol.to_string(),
    current_eps,
    quarterly_eps,
    historical_eps,
    quarterly_data,
    price_data,
    earnings_per_share_basic_ttm: current_eps * 4.0,
    market_cap_basic: 0.0,
    price_current: 0.0,
    volume: 0.0,
    sector,
    country,
    company_name,
  }
}

fn format_value_debug(val: &Value) -> String {
  match val {
    Value::Number(n) => {
      if let Some(ts) = n.as_i64() {
        if ts > 1000000000 {
          let dt = chrono::DateTime::from_timestamp(ts, 0)
            .or_else(|| chrono::DateTime::from_timestamp(ts / 1000, 0));
          if let Some(date) = dt {
            return format!("{} ({})", n, date.format("%Y-%m-%d"));
          }
        }
      }
      n.to_string()
    },
    Value::String(s) => format!("\"{}\"", s),
    Value::Null => "null".to_string(),
    Value::Bool(b) => b.to_string(),
    _ => "?".to_string()
  }
}

fn is_valid_quarterly_eps(eps: f64) -> bool {
  eps.is_finite() && eps != 0.0 && eps.abs() < 1000.0
}

fn timestamp_to_fiscal_period(timestamp: i64) -> String {
  if let Some(dt) = chrono::DateTime::from_timestamp(timestamp, 0) {
    let year = dt.year();
    let month = dt.month();
    let quarter = match month {
      1..=3 => "Q1",
      4..=6 => "Q2",
      7..=9 => "Q3",
      _ => "Q4",
    };
    format!("{}-{}", year, quarter)
  } else {
    "Unknown".to_string()
  }
}

fn extract_fiscal_quarter_number(timestamp: i64) -> usize {
  if let Some(dt) = chrono::DateTime::from_timestamp(timestamp, 0) {
    ((dt.month() - 1) / 3 + 1) as usize
  } else {
    0
  }
}
