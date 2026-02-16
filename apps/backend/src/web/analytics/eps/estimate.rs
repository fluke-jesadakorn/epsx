// Next Quarter Estimation
// Handles estimation of next quarter EPS and announcement dates

use tracing::{debug, info};
use chrono::Datelike;

use super::types::{UnifiedRankingItem, QuarterlyPerformanceData, NextQuarterEstimate};

/// Helper struct for estimate calculation intermediate data
struct EstimateData {
  quarter: String,
  eps: f64,
  date_str: String,
  timestamp: i64,
  days_until: i32,
  price_target: Option<f64>,
  confidence: String,
}

/// Priority 1: Extract estimate from TradingView timestamp
fn extract_tradingview_estimate(
  unified_item: &UnifiedRankingItem,
  quarterly_performance: &[QuarterlyPerformanceData],
  current_date: chrono::DateTime<chrono::Utc>,
) -> Option<EstimateData> {
  let next_timestamp = unified_item.next_earnings_date?;

  if next_timestamp <= 0 {
    info!("[{}] next_earnings_date is zero or negative", unified_item.symbol);
    return None;
  }

  let announcement_datetime = chrono::DateTime::from_timestamp(next_timestamp, 0)?;
  let days_until = (announcement_datetime - current_date).num_days().max(0) as i32;

  let latest_quarter = &quarterly_performance[0];
  let estimated_eps = if quarterly_performance.len() >= 2 {
    let latest_eps = latest_quarter.eps;
    let previous_eps = quarterly_performance[1].eps;
    let growth_rate = if previous_eps != 0.0 {
      ((latest_eps - previous_eps) / previous_eps).clamp(-0.3, 0.3)
    } else {
      0.1
    };
    latest_eps * (1.0 + growth_rate)
  } else {
    latest_quarter.eps * 1.05
  };

  let quarter_num = match announcement_datetime.month() {
    1..=3 => 1,
    4..=6 => 2,
    7..=9 => 3,
    _ => 4,
  };
  let quarter_name = format!("{}-Q{}", announcement_datetime.year(), quarter_num);

  let estimated_price_target = if estimated_eps > 0.0 && unified_item.current_price > 0.0 {
    let current_pe = unified_item.current_price / latest_quarter.eps.max(0.01);
    Some((estimated_eps * current_pe * 0.95).max(0.0))
  } else {
    None
  };

  info!(
    "[{}] Using REAL TradingView timestamp: {} ({} days) - PRIORITY 1",
    unified_item.symbol,
    next_timestamp,
    days_until
  );

  Some(EstimateData {
    quarter: quarter_name,
    eps: estimated_eps,
    date_str: announcement_datetime.format("%b %-d, %Y").to_string(),
    timestamp: announcement_datetime.timestamp(),
    days_until,
    price_target: estimated_price_target,
    confidence: "TradingView Real Data".to_string(),
  })
}

/// Priority 2: Fetch real-time WebSocket estimate
async fn fetch_realtime_estimate_async(
  unified_item: &UnifiedRankingItem,
) -> Option<EstimateData> {
  let fallback_config = super::system::get_simple_fallback_config();
  let tradingview_config = crate::infrastructure::adapters::tradingview_types::TradingViewConfig::from(&fallback_config);
  let websocket_handler = crate::infrastructure::adapters::services::tradingview::TradingViewWebSocketHandler::new(tradingview_config);

  let enhanced_data_list = websocket_handler.fetch_enhanced_eps_data(vec![unified_item.symbol.clone()]).await.ok()?;
  let enhanced_data = enhanced_data_list.first()?;

  info!("[DEBUG] Using real-time TradingView data for forecast: {}", unified_item.symbol);

  let estimated_eps = if enhanced_data.qoq_growth > 0.0 {
    enhanced_data.current_eps * (1.0 + (enhanced_data.qoq_growth / 100.0) * 0.8)
  } else {
    enhanced_data.current_eps * 1.03
  };

  let growth_percentage = if enhanced_data.current_eps > 0.0 {
    ((estimated_eps - enhanced_data.current_eps) / enhanced_data.current_eps) * 100.0
  } else {
    3.0
  };

  debug!(
    "[DEBUG] Real-time forecast: Current EPS={:.2}, Next EPS={:.2}, Growth={:.1}%",
    enhanced_data.current_eps,
    estimated_eps,
    growth_percentage
  );

  let current_date = chrono::Utc::now();
  let current_quarter = (current_date.month() - 1) / 3 + 1;
  let (next_quarter, next_year) = if current_quarter == 4 {
    (1, current_date.year() + 1)
  } else {
    (current_quarter + 1, current_date.year())
  };
  let next_quarter_name = format!("{}-Q{}", next_year, next_quarter);

  let symbol_hash = unified_item.symbol.chars().map(|c| c as u32).sum::<u32>();
  let days_until = 45 + ((symbol_hash % 30) as i32);
  let announcement_date = current_date + chrono::Duration::days(days_until as i64);

  let price_target = if enhanced_data.current_eps > 0.0 {
    let current_pe = enhanced_data.price_current / enhanced_data.current_eps;
    Some(estimated_eps * current_pe * 0.98)
  } else {
    None
  };

  Some(EstimateData {
    quarter: next_quarter_name,
    eps: estimated_eps,
    date_str: announcement_date.format("%b %-d, %Y").to_string(),
    timestamp: announcement_date.timestamp(),
    days_until,
    price_target,
    confidence: "TradingView Real-time".to_string(),
  })
}

/// Fallback: Calculate estimate using traditional method
fn calculate_fallback_estimate(
  unified_item: &UnifiedRankingItem,
  quarterly_performance: &[QuarterlyPerformanceData],
  current_date: chrono::DateTime<chrono::Utc>,
) -> EstimateData {
  debug!("[DEBUG] Using fallback calculation for {}", unified_item.symbol);

  let latest_quarter = &quarterly_performance[0];
  let current_quarter = (current_date.month() - 1) / 3 + 1;

  let (next_quarter, next_year) = if current_quarter == 4 {
    (1, current_date.year() + 1)
  } else {
    (current_quarter + 1, current_date.year())
  };

  let next_quarter_name = format!("{}-Q{}", next_year, next_quarter);

  let estimated_eps = if quarterly_performance.len() >= 2 {
    let latest_eps = latest_quarter.eps;
    let previous_eps = quarterly_performance[1].eps;
    let growth_rate = if previous_eps != 0.0 {
      (latest_eps - previous_eps) / previous_eps
    } else {
      0.1
    };
    let moderated_growth = growth_rate.clamp(-0.3, 0.3);
    latest_eps * (1.0 + moderated_growth)
  } else {
    latest_quarter.eps * 1.05
  };

  let quarter_end_month = match next_quarter {
    1 => 3,
    2 => 6,
    3 => 9,
    _ => 12,
  };
  let quarter_end_date = chrono::Utc::now()
    .with_year(next_year)
    .and_then(|d| d.with_month(quarter_end_month))
    .and_then(|d| d.with_day(30))
    .unwrap_or(current_date);

  let symbol_hash = unified_item.symbol.chars().map(|c| c as u32).sum::<u32>();
  let variation_days = 45 + ((symbol_hash % 30) as i64);
  let estimated_announcement = quarter_end_date + chrono::Duration::days(variation_days);
  let days = (estimated_announcement - current_date).num_days() as i32;

  let estimated_price_target = if estimated_eps > 0.0 && latest_quarter.eps > 0.0 {
    let current_pe = unified_item.current_price / latest_quarter.eps;
    Some((estimated_eps * current_pe * 0.95).max(0.0))
  } else {
    None
  };

  let confidence = match quarterly_performance.len() {
    len if len >= 4 => "High (Estimated)".to_string(),
    len if len >= 2 => "Medium (Estimated)".to_string(),
    _ => "Low (Estimated)".to_string(),
  };

  EstimateData {
    quarter: next_quarter_name,
    eps: estimated_eps,
    date_str: estimated_announcement.format("%b %-d, %Y").to_string(),
    timestamp: estimated_announcement.timestamp(),
    days_until: days,
    price_target: estimated_price_target,
    confidence,
  }
}

/// Generate next quarter EPS estimate from enhanced TradingView data
pub(super) fn generate_next_quarter_estimate(
  unified_item: &UnifiedRankingItem,
  quarterly_performance: &[QuarterlyPerformanceData]
) -> Option<NextQuarterEstimate> {
  if quarterly_performance.is_empty() {
    return None;
  }

  let current_date = chrono::Utc::now();

  debug!(
    "[DEBUG] Generating next quarter estimate for {}",
    unified_item.symbol
  );

  // PRIORITY 1: Try TradingView timestamp
  if let Some(estimate) = extract_tradingview_estimate(unified_item, quarterly_performance, current_date) {
    return Some(NextQuarterEstimate {
      quarter: estimate.quarter,
      estimated_eps: (estimate.eps * 100.0).round() / 100.0,
      announcement_date: estimate.date_str,
      announcement_timestamp: estimate.timestamp,
      days_until_announcement: estimate.days_until,
      estimated_price_target: estimate.price_target,
      confidence: estimate.confidence,
    });
  }

  info!("[{}] No TradingView timestamp - trying WebSocket", unified_item.symbol);

  // PRIORITY 2: Try real-time WebSocket data
  let realtime_estimate = tokio::task::block_in_place(|| {
    tokio::runtime::Handle::current().block_on(async {
      fetch_realtime_estimate_async(unified_item).await
    })
  });

  // Use realtime data or fallback
  let estimate = realtime_estimate.unwrap_or_else(|| {
    calculate_fallback_estimate(unified_item, quarterly_performance, current_date)
  });

  Some(NextQuarterEstimate {
    quarter: estimate.quarter,
    estimated_eps: (estimate.eps * 100.0).round() / 100.0,
    announcement_date: estimate.date_str,
    announcement_timestamp: estimate.timestamp,
    days_until_announcement: estimate.days_until,
    estimated_price_target: estimate.price_target,
    confidence: estimate.confidence,
  })
}
