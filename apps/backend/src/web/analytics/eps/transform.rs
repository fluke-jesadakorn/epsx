// Data Transformation and Formatting Utilities
// Main public transformation functions between formats

use tracing::warn;

use crate::domain::shared_kernel::entities::eps_growth::EPSRanking;
use crate::domain::shared_kernel::entities::market_data::StockScreeningResult;
use super::types::*;
use super::quarterly::*;
use super::date_metrics::*;
use super::estimate::*;
use super::system::*;

/// Transform EPS ranking to unified format with quarterly data
pub fn transform_ranking_to_unified_format(
  ranking: EPSRanking,
  position: usize
) -> UnifiedRankingItem {
  let current_date = chrono::Utc::now();
  let current_price = ranking.price_current.unwrap_or_else(|| {
    warn!(
      "No price data available for {}, calculation may be incomplete",
      ranking.symbol
    );
    0.0
  });
  let growth_factor_pct = ranking.growth_factor.unwrap_or(0.0);

  UnifiedRankingItem {
    symbol: ranking.symbol.clone(),
    company_name: ranking.name.clone(),
    ranking_position: position as i32,
    current_price,
    current_price_date: current_date,
    quarterly_data: generate_quarterly_data_from_websocket_or_fallback(
      &ranking,
      current_date
    ),
    market_data: MarketData {
      market_cap: ranking.market_cap,
      volume_24h: None,
      country: "US".to_string(),
      sector: ranking.sector.clone(),
      exchange: "NASDAQ".to_string(),
    },
    analytics: AnalyticsMetrics {
      growth_factor: growth_factor_pct,
      ranking_score: 0.0,
      trend: determine_trend(growth_factor_pct),
      volatility: calculate_simple_volatility(growth_factor_pct),
    },
    next_earnings_date: ranking.next_earnings_date
      .as_ref()
      .and_then(|date_str| {
        chrono::NaiveDate
          ::parse_from_str(date_str, "%Y-%m-%d")
          .ok()
          .and_then(|date| date.and_hms_opt(0, 0, 0)).map(|dt| dt.and_utc().timestamp())
      }),
    last_earnings_date: ranking.last_earnings_date
      .as_ref()
      .and_then(|date_str| {
        chrono::NaiveDate
          ::parse_from_str(date_str, "%Y-%m-%d")
          .ok()
          .and_then(|date| date.and_hms_opt(0, 0, 0)).map(|dt| dt.and_utc().timestamp())
      }),
  }
}

/// Transform StockScreeningResult to EPSQuarterlyData for 4-quarter display
pub fn transform_stock_screening_to_quarterly_data(
  screening_result: &StockScreeningResult
) -> Option<EPSQuarterlyData> {
  if
    screening_result.eps_q_current.is_none() &&
    screening_result.eps_q_minus_1.is_none() &&
    screening_result.eps_q_minus_2.is_none()
  {
    return None;
  }

  Some(EPSQuarterlyData {
    eps_q_minus_2: screening_result.eps_q_minus_2,
    eps_q_minus_1: screening_result.eps_q_minus_1,
    eps_q_current: screening_result.eps_q_current,
    eps_q_next_estimate: None,
    eps_q_minus_2_date: None,
    eps_q_minus_1_date: None,
    eps_q_current_date: None,
    eps_q_next_estimate_date: None,
    qoq_growth_current: None,
    yoy_growth_current: None,
    trend_direction: None,
    avg_growth_rate: None,
    consistency_score: None,
  })
}

/// Transform unified item to card format with quarterly performance
pub fn transform_unified_to_card_format(
  unified_item: &UnifiedRankingItem
) -> SymbolCardData {
  let current_date = chrono::Utc::now();

  let quarterly_performance: Vec<QuarterlyPerformanceData> = unified_item.quarterly_data
    .iter()
    .map(|q| {
      let (announcement_date, is_estimated) = format_announcement_date_from_quarter_data(q);
      QuarterlyPerformanceData {
        quarter: q.quarter.clone(),
        date: q.date.format("%b %d, %Y").to_string(),
        price: q.price,
        eps: q.eps,
        eps_growth: q.eps_growth,
        price_growth: q.price_growth,
        announcement_date,
        announcement_timestamp: Some(q.date.timestamp()),
        is_estimated,
      }
    })
    .collect();

  let next_quarter = generate_next_quarter_estimate(unified_item, &quarterly_performance);
  let active_status = calculate_system_mode(&quarterly_performance);

  let days_until_next_earnings = next_quarter.as_ref().map(|nq| nq.days_until_announcement);
  let progress_percentage = days_until_next_earnings.map(|days| {
    ((90 - days.min(90)) as f64 / 90.0 * 100.0).max(0.0)
  });

  // Extract top-level fields from quarterly_performance[0] for frontend
  let current_eps = quarterly_performance.first().map(|q| q.eps);
  let growth_factor = quarterly_performance.first().map(|q| q.eps_growth);
  let price_current = quarterly_performance.first().map(|q| q.price);

  SymbolCardData {
    rank: unified_item.ranking_position,
    symbol: unified_item.symbol.clone(),
    latest_date: current_date.format("%b %-d, %-I:%M %p").to_string(),
    value: unified_item.current_price,
    active_status,
    quarterly_performance,
    next_quarter_estimate: next_quarter,
    eps_quarterly: None,
    next_earnings_date: unified_item.next_earnings_date,
    last_earnings_date: unified_item.last_earnings_date,
    next_earnings_date_formatted: unified_item.next_earnings_date.and_then(|ts| {
      chrono::DateTime::from_timestamp(ts, 0).map(|dt| dt.format("%b %d, %Y").to_string())
    }),
    days_until_next_earnings,
    progress_percentage,
    current_eps,
    growth_factor,
    price_current,
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_determine_trend() {
    assert_eq!(determine_trend(25.0), "strong_bullish");
    assert_eq!(determine_trend(10.0), "bullish");
    assert_eq!(determine_trend(0.0), "neutral");
    assert_eq!(determine_trend(-10.0), "bearish");
    assert_eq!(determine_trend(-25.0), "strong_bearish");
  }

  #[test]
  fn test_calculate_simple_volatility() {
    assert_eq!(calculate_simple_volatility(10.0), 10.0);
    assert_eq!(calculate_simple_volatility(-10.0), 10.0);
    assert_eq!(calculate_simple_volatility(60.0), 50.0);
  }
}
