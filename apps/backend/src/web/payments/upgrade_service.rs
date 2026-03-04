//! Upgrade Service
//!
//! Handles plan upgrade by day conversion: remaining value on current plan
//! is converted into fewer days on the higher-priced plan. Upgrades are FREE.

use chrono::{DateTime, Utc};
use rust_decimal::Decimal;

/// Standard billing period days for a given cycle string
pub fn billing_period_days(cycle: Option<&str>) -> i64 {
    match cycle.map(|s| s.to_lowercase()).as_deref() {
        Some("monthly") => 30,
        Some("quarterly") => 90,
        Some("yearly") | Some("annual") => 365,
        _ => 30,
    }
}

/// Calculate how many days the user gets on the new plan (free upgrade).
///
/// Formula:
///   current_daily_rate = current_price / current_period
///   remaining_value    = current_daily_rate × days_remaining
///   new_daily_rate     = new_price / new_period
///   new_days           = floor(remaining_value / new_daily_rate)
pub fn calculate_upgrade_days(
    current_price: Decimal,
    current_period: i64,
    days_remaining: i64,
    new_price: Decimal,
    new_period: i64,
) -> i64 {
    if new_price <= Decimal::ZERO || days_remaining <= 0 || current_period <= 0 || new_period <= 0 {
        return 0;
    }
    let cur_daily = current_price / Decimal::from(current_period);
    let remaining_value = cur_daily * Decimal::from(days_remaining);
    let new_daily = new_price / Decimal::from(new_period);
    if new_daily <= Decimal::ZERO {
        return 0;
    }
    let new_days = remaining_value / new_daily;
    // Floor to i64
    new_days.to_string().split('.').next()
        .and_then(|s| s.parse::<i64>().ok())
        .unwrap_or(0)
        .max(0)
}

/// Calculate the dollar value of remaining plan time (for display purposes).
///
/// Uses daily rate based on billing period, not assigned_at→expires_at span.
pub fn calculate_upgrade_credit(
    original_price: Decimal,
    plan_expires_at: DateTime<Utc>,
    billing_period: i64,
) -> Decimal {
    let now = Utc::now();
    if now >= plan_expires_at || billing_period <= 0 {
        return Decimal::ZERO;
    }
    let days_remaining = (plan_expires_at - now).num_days();
    let daily_rate = original_price / Decimal::from(billing_period);
    (daily_rate * Decimal::from(days_remaining)).round_dp(2)
}

/// Check if the plan change is an upgrade (higher price)
pub fn is_upgrade_allowed(current_plan_price: Decimal, new_plan_price: Decimal) -> bool {
    new_plan_price > current_plan_price
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Duration;
    use rust_decimal_macros::dec;

    #[test]
    fn test_billing_period_days() {
        assert_eq!(billing_period_days(Some("monthly")), 30);
        assert_eq!(billing_period_days(Some("Monthly")), 30);
        assert_eq!(billing_period_days(Some("quarterly")), 90);
        assert_eq!(billing_period_days(Some("yearly")), 365);
        assert_eq!(billing_period_days(Some("annual")), 365);
        assert_eq!(billing_period_days(None), 30);
        assert_eq!(billing_period_days(Some("unknown")), 30);
    }

    #[test]
    fn test_upgrade_days_normal_half_remaining() {
        // $13.99/30d plan, 15 days left, upgrading to $29.99/30d
        let days = calculate_upgrade_days(dec!(13.99), 30, 15, dec!(29.99), 30);
        assert_eq!(days, 6); // floor(15 * 13.99 / 29.99) = floor(6.998) = 6
    }

    #[test]
    fn test_upgrade_days_extended_plan() {
        // $13.99/30d plan, 364 days left, upgrading to $14.99/30d
        let days = calculate_upgrade_days(dec!(13.99), 30, 364, dec!(14.99), 30);
        assert_eq!(days, 339); // floor(364 * 13.99 / 14.99) = floor(339.70)
    }

    #[test]
    fn test_upgrade_days_extended_to_2x_price() {
        // $13.99/30d plan, 364 days left, upgrading to $29.99/30d
        let days = calculate_upgrade_days(dec!(13.99), 30, 364, dec!(29.99), 30);
        assert_eq!(days, 169); // floor(364 * 13.99 / 29.99)
    }

    #[test]
    fn test_upgrade_days_extended_to_3_5x_price() {
        // $13.99/30d plan, 364 days left, upgrading to $49.99/30d
        let days = calculate_upgrade_days(dec!(13.99), 30, 364, dec!(49.99), 30);
        assert_eq!(days, 101); // floor(364 * 13.99 / 49.99)
    }

    #[test]
    fn test_upgrade_days_brand_new() {
        // $9.99/30d plan, 30 days left, upgrading to $29.99/30d
        let days = calculate_upgrade_days(dec!(9.99), 30, 30, dec!(29.99), 30);
        assert_eq!(days, 9); // floor(30 * 9.99 / 29.99)
    }

    #[test]
    fn test_upgrade_days_same_price() {
        // Same price: should get same days back
        let days = calculate_upgrade_days(dec!(13.99), 30, 15, dec!(13.99), 30);
        assert_eq!(days, 15);
    }

    #[test]
    fn test_upgrade_days_zero_remaining() {
        let days = calculate_upgrade_days(dec!(13.99), 30, 0, dec!(29.99), 30);
        assert_eq!(days, 0);
    }

    #[test]
    fn test_upgrade_days_zero_new_price() {
        let days = calculate_upgrade_days(dec!(13.99), 30, 15, dec!(0), 30);
        assert_eq!(days, 0);
    }

    #[test]
    fn test_credit_expired() {
        let now = Utc::now();
        let expires = now - Duration::days(5);
        let credit = calculate_upgrade_credit(dec!(30.00), expires, 30);
        assert_eq!(credit, Decimal::ZERO);
    }

    #[test]
    fn test_credit_half_remaining() {
        let now = Utc::now();
        // Add 1 hour buffer to avoid num_days() truncation from sub-ms timing
        let expires = now + Duration::days(15) + Duration::hours(1);
        let credit = calculate_upgrade_credit(dec!(30.00), expires, 30);
        assert_eq!(credit, dec!(15.00));
    }

    #[test]
    fn test_credit_extended_plan() {
        // Extended plan: 364 days left, $13.99/30d billing period
        let now = Utc::now();
        let expires = now + Duration::days(364);
        let credit = calculate_upgrade_credit(dec!(13.99), expires, 30);
        // daily_rate = 13.99/30 = 0.4663..., 364 * 0.4663 = ~169.74
        assert!(credit > dec!(169.00));
        assert!(credit < dec!(170.00));
    }

    #[test]
    fn test_upgrade_check() {
        assert!(is_upgrade_allowed(dec!(30.00), dec!(50.00)));
        assert!(!is_upgrade_allowed(dec!(50.00), dec!(30.00)));
        assert!(!is_upgrade_allowed(dec!(30.00), dec!(30.00)));
    }
}
