// Next Quarter Estimation
// Handles estimation of next quarter EPS and announcement dates

use chrono::Datelike;
use tracing::{debug, info};

use super::types::{NextQuarterEstimate, QuarterlyPerformanceData, UnifiedRankingItem};

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
        info!(
            "[{}] next_earnings_date is zero or negative",
            unified_item.symbol
        );
        return None;
    }

    let announcement_datetime = chrono::DateTime::from_timestamp(next_timestamp, 0)?;
    if announcement_datetime.year() > 9999 {
        return None;
    }
    let days_until =
        (announcement_datetime.date_naive() - current_date.date_naive()).num_days() as i32;

    let latest_quarter = quarterly_performance.first()?;
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
        unified_item.symbol, next_timestamp, days_until
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

/// Estimate the next report exactly 90 calendar days after a supplied previous report.
fn calculate_fallback_estimate(
    unified_item: &UnifiedRankingItem,
    quarterly_performance: &[QuarterlyPerformanceData],
    current_date: chrono::DateTime<chrono::Utc>,
) -> Option<EstimateData> {
    debug!(
        "[DEBUG] Using fallback calculation for {}",
        unified_item.symbol
    );

    let previous_timestamp = unified_item.last_earnings_date?;
    if previous_timestamp <= 0 {
        return None;
    }
    let previous = chrono::DateTime::from_timestamp(previous_timestamp, 0)?.date_naive();
    if previous.year() > 9999 || previous > current_date.date_naive() {
        return None;
    }
    let estimated_announcement = previous.checked_add_days(chrono::Days::new(90))?;
    if estimated_announcement.year() > 9999 {
        return None;
    }
    let days = (estimated_announcement - current_date.date_naive()).num_days() as i32;
    let next_quarter_name = format!(
        "{}-Q{}",
        estimated_announcement.year(),
        (estimated_announcement.month() - 1) / 3 + 1
    );
    let latest_quarter = quarterly_performance.first()?;

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

    let estimated_price_target = if estimated_eps > 0.0 && latest_quarter.eps > 0.0 {
        let current_pe = unified_item.current_price / latest_quarter.eps;
        Some((estimated_eps * current_pe * 0.95).max(0.0))
    } else {
        None
    };

    Some(EstimateData {
        quarter: next_quarter_name,
        eps: estimated_eps,
        date_str: estimated_announcement.format("%b %-d, %Y").to_string(),
        timestamp: estimated_announcement
            .and_hms_opt(0, 0, 0)?
            .and_utc()
            .timestamp(),
        days_until: days,
        price_target: estimated_price_target,
        confidence: "Estimated 90 days after the previous company report.".to_string(),
    })
}

/// Generate next quarter EPS estimate from enhanced TradingView data
pub(super) fn generate_next_quarter_estimate(
    unified_item: &UnifiedRankingItem,
    quarterly_performance: &[QuarterlyPerformanceData],
) -> Option<NextQuarterEstimate> {
    if quarterly_performance.is_empty() {
        return None;
    }

    let current_date = chrono::Utc::now();

    debug!(
        "[DEBUG] Generating next quarter estimate for {}",
        unified_item.symbol
    );

    // Both paths require a provider report date. Never synthesize dates from
    // request time, a fiscal period end, company symbol, or a disabled WebSocket.
    let estimate = extract_tradingview_estimate(unified_item, quarterly_performance, current_date)
        .or_else(|| {
            calculate_fallback_estimate(unified_item, quarterly_performance, current_date)
        })?;

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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::web::analytics::eps::types::{AnalyticsMetrics, MarketData};

    fn datetime(value: &str) -> chrono::DateTime<chrono::Utc> {
        chrono::DateTime::parse_from_rfc3339(value)
            .unwrap()
            .with_timezone(&chrono::Utc)
    }

    fn item(next: Option<&str>, previous: Option<&str>) -> UnifiedRankingItem {
        UnifiedRankingItem {
            symbol: "CPR".into(),
            company_name: "CPR Gomu".into(),
            ranking_position: 104,
            current_price: 1.0,
            current_price_date: datetime("2026-09-08T00:00:00Z"),
            quarterly_data: vec![],
            market_data: MarketData {
                market_cap: None,
                volume_24h: None,
                country: "thailand".into(),
                sector: "".into(),
                exchange: "SET".into(),
            },
            analytics: AnalyticsMetrics {
                growth_factor: 0.0,
                ranking_score: 0.0,
                trend: "".into(),
                volatility: 0.0,
            },
            next_earnings_date: next.map(|value| datetime(value).timestamp()),
            last_earnings_date: previous.map(|value| datetime(value).timestamp()),
        }
    }

    fn performance() -> Vec<QuarterlyPerformanceData> {
        vec![QuarterlyPerformanceData {
            quarter: "Q2".into(),
            date: "2026-06-30".into(),
            price: 1.0,
            eps: 1.0,
            eps_growth: 0.0,
            price_growth: 0.0,
            announcement_date: None,
            announcement_timestamp: None,
            is_estimated: false,
        }]
    }

    #[test]
    fn previous_report_fallback_crosses_leap_day_and_year_boundary() {
        for (previous, expected) in [
            ("2023-12-01T17:20:00Z", "2024-02-29T00:00:00Z"),
            ("2026-11-30T04:00:00Z", "2027-02-28T00:00:00Z"),
        ] {
            let row = item(None, Some(previous));
            let estimate =
                calculate_fallback_estimate(&row, &performance(), datetime(previous)).unwrap();
            assert_eq!(estimate.timestamp, datetime(expected).timestamp());
            assert_eq!(estimate.days_until, 90);
            assert_eq!(
                estimate.confidence,
                "Estimated 90 days after the previous company report."
            );
        }
    }

    #[test]
    fn fallback_date_does_not_move_with_request_time_or_symbol() {
        let mut row = item(None, Some("2026-07-29T23:59:00Z"));
        let first =
            calculate_fallback_estimate(&row, &performance(), datetime("2026-09-08T23:59:00Z"))
                .unwrap();
        row.symbol = "BRIGHT".into();
        let later =
            calculate_fallback_estimate(&row, &performance(), datetime("2026-09-09T00:01:00Z"))
                .unwrap();
        assert_eq!(first.timestamp, later.timestamp);
        assert_eq!(first.days_until, later.days_until + 1);
        assert_eq!(
            first.timestamp,
            datetime("2026-10-27T00:00:00Z").timestamp()
        );
    }

    #[test]
    fn actual_provider_date_wins_over_fallback() {
        let row = item(Some("2026-10-28T00:00:00Z"), Some("2026-07-29T00:00:00Z"));
        let estimate = generate_next_quarter_estimate(&row, &performance()).unwrap();
        assert_eq!(
            estimate.announcement_timestamp,
            datetime("2026-10-28T00:00:00Z").timestamp()
        );
        assert_eq!(estimate.confidence, "TradingView Real Data");
    }

    #[test]
    fn today_and_passed_dates_use_utc_calendar_days() {
        for (next, expected) in [
            ("2026-09-08T00:00:00Z", 0),
            ("2026-09-07T00:00:00Z", -1),
            ("2026-09-09T00:00:00Z", 1),
        ] {
            let estimate = extract_tradingview_estimate(
                &item(Some(next), None),
                &performance(),
                datetime("2026-09-08T23:59:00Z"),
            )
            .unwrap();
            assert_eq!(estimate.days_until, expected);
        }
        let row = item(None, Some("2026-01-01T00:00:00Z"));
        let estimate =
            calculate_fallback_estimate(&row, &performance(), datetime("2026-09-08T00:00:00Z"))
                .unwrap();
        assert_eq!(
            estimate.timestamp,
            datetime("2026-04-01T00:00:00Z").timestamp()
        );
        assert!(estimate.days_until < 0);
    }

    #[test]
    fn missing_and_malformed_dates_do_not_create_an_announcement() {
        let mut row = item(None, None);
        // This also runs outside a Tokio runtime; the disabled WebSocket must not be called.
        assert!(generate_next_quarter_estimate(&row, &performance()).is_none());
        for invalid in [0, -1, i64::MAX, 1785283200000] {
            row.next_earnings_date = Some(invalid);
            row.last_earnings_date = Some(invalid);
            assert!(generate_next_quarter_estimate(&row, &performance()).is_none());
        }
        assert!(calculate_fallback_estimate(
            &item(None, Some("2026-09-09T00:00:00Z")),
            &performance(),
            datetime("2026-09-08T00:00:00Z")
        )
        .is_none());
    }
}
