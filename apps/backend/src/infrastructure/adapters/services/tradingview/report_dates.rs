//! Report-date extraction shared by the scanner and legacy response mapper.
//! Keep provider dates separate from presentation estimates. Fiscal period-end
//! (`*_calendar_date`) fields are not publication dates and are never used here.

use chrono::{DateTime, Datelike, Utc};

use super::{types::StockDataField, utils::get_number_opt};

const LAST_RELEASE: usize = 32;
const NEXT_RELEASE: usize = 33;
const LAST_ANNUAL_RELEASE: usize = 34;
// Appended to both scanner column lists; existing column offsets stay stable.
const LAST_QUARTERLY_RELEASE: usize = 36;
const NEXT_QUARTERLY_RELEASE: usize = 37;
const NEXT_ANNUAL_RELEASE: usize = 38;

/// Apply report-date eligibility at the provider, before its range and count.
/// A previous release qualifies for the existing +90-day presentation estimate.
/// Never substitute a fiscal period end or require the supplied date to be future.
pub(super) fn ranking_date_filter() -> serde_json::Value {
    use serde_json::json;

    let operands: Vec<_> = [
        "earnings_release_date",
        "earnings_release_next_date",
        "earnings_release_trading_date_fy",
        "earnings_release_trading_date_fq",
        "earnings_release_next_trading_date_fq",
        "earnings_release_next_trading_date_fy",
    ]
    .into_iter()
    .map(|field| {
        json!({
            "operation": {
                "operator": "and",
                "operands": [
                    {"expression": {"left": field, "operation": "greater", "right": 0}},
                    // Exclusive upper bound: 10000-01-01 UTC. Reject milliseconds
                    // and out-of-range values before they consume a ranking slot.
                    {"expression": {"left": field, "operation": "less", "right": 253_402_300_800_i64}}
                ]
            }
        })
    })
    .collect();
    json!({"operator": "or", "operands": operands})
}

pub(super) fn valid_timestamp(value: f64) -> Option<i64> {
    if !value.is_finite() || value <= 0.0 || value.fract() != 0.0 {
        return None;
    }
    let timestamp = value as i64;
    let date = DateTime::from_timestamp(timestamp, 0)?;
    (date.year() <= 9999).then_some(timestamp)
}

pub(super) fn date_string(value: f64) -> Option<String> {
    DateTime::from_timestamp(valid_timestamp(value)?, 0)
        .map(|date| date.format("%Y-%m-%d").to_string())
}

#[derive(Debug, PartialEq, Eq)]
pub(super) struct ReportDates {
    pub last: Option<i64>,
    pub next: Option<i64>,
}

pub(super) fn extract_report_dates(data: &[StockDataField], now: DateTime<Utc>) -> ReportDates {
    let at = |index| get_number_opt(data, index).and_then(valid_timestamp);
    let reported = [
        at(LAST_RELEASE),
        at(LAST_QUARTERLY_RELEASE),
        at(LAST_ANNUAL_RELEASE),
    ];
    let last = reported
        .into_iter()
        .flatten()
        .filter(|&ts| ts <= now.timestamp())
        .max();
    // The provider's explicit next date wins, including Today and Date passed.
    // Do not silently discard a stale supplied date or roll it forward.
    let next = at(NEXT_RELEASE)
        .or_else(|| at(NEXT_QUARTERLY_RELEASE))
        .or_else(|| at(NEXT_ANNUAL_RELEASE))
        // Some feeds place an upcoming announcement in the generic release field.
        .or_else(|| {
            reported
                .into_iter()
                .flatten()
                .filter(|&ts| ts > now.timestamp())
                .min()
        });
    ReportDates { last, next }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn timestamp(date: &str) -> i64 {
        DateTime::parse_from_rfc3339(date).unwrap().timestamp()
    }

    fn fields(values: &[(usize, i64)]) -> Vec<StockDataField> {
        let mut fields = vec![StockDataField::Null; 39];
        for &(index, value) in values {
            fields[index] = StockDataField::Integer(value);
        }
        fields
    }

    #[test]
    fn preserves_explicit_dates_today_and_in_the_past() {
        let now = DateTime::parse_from_rfc3339("2026-09-08T12:00:00Z")
            .unwrap()
            .with_timezone(&Utc);
        for date in ["2026-09-08T00:00:00Z", "2026-09-07T00:00:00Z"] {
            let explicit = timestamp(date);
            let data = fields(&[
                (NEXT_RELEASE, explicit),
                (NEXT_QUARTERLY_RELEASE, timestamp("2026-11-01T00:00:00Z")),
            ]);
            assert_eq!(extract_report_dates(&data, now).next, Some(explicit));
        }
    }

    #[test]
    fn recovers_quarterly_dates_and_uses_the_latest_previous_report() {
        let now = DateTime::parse_from_rfc3339("2026-09-08T12:00:00Z")
            .unwrap()
            .with_timezone(&Utc);
        let previous = timestamp("2026-08-10T00:00:00Z");
        let next = timestamp("2026-11-10T00:00:00Z");
        let data = fields(&[
            (LAST_ANNUAL_RELEASE, timestamp("2026-02-10T00:00:00Z")),
            (LAST_QUARTERLY_RELEASE, previous),
            (NEXT_QUARTERLY_RELEASE, next),
        ]);
        assert_eq!(
            extract_report_dates(&data, now),
            ReportDates {
                last: Some(previous),
                next: Some(next)
            }
        );
    }

    #[test]
    fn upcoming_generic_release_is_not_a_previous_report() {
        let now = Utc::now();
        let upcoming = now.timestamp() + 86400;
        assert_eq!(
            extract_report_dates(&fields(&[(LAST_RELEASE, upcoming)]), now),
            ReportDates {
                last: None,
                next: Some(upcoming)
            }
        );
    }

    #[test]
    fn missing_and_invalid_provider_dates_are_not_invented() {
        assert_eq!(
            extract_report_dates(&[], Utc::now()),
            ReportDates {
                last: None,
                next: None
            }
        );
        for value in [
            0.0,
            -1.0,
            f64::NAN,
            f64::INFINITY,
            1785283200.5,
            1785283200000.0,
        ] {
            assert_eq!(valid_timestamp(value), None, "{value}");
        }
        assert_eq!(date_string(1785283200.0).as_deref(), Some("2026-07-29"));
    }
}
