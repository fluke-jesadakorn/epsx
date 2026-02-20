//! Upgrade Service
//!
//! Handles plan upgrade logic with credit calculation.
//! Users can only upgrade - downgrades are not allowed.

use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use rust_decimal::prelude::FromPrimitive;

/// Calculate pro-rata credit from remaining plan time
///
/// Formula: (days_remaining / total_days) × original_price
///
/// # Arguments
/// * `original_price` - The price the user originally paid
/// * `plan_started_at` - When the plan was activated
/// * `plan_expires_at` - When the plan expires
///
/// # Returns
/// The credit amount to apply towards the new plan
pub fn calculate_upgrade_credit(
    original_price: Decimal,
    plan_started_at: DateTime<Utc>,
    plan_expires_at: DateTime<Utc>,
) -> Decimal {
    let now = Utc::now();

    // If already expired, no credit
    if now >= plan_expires_at {
        return Decimal::ZERO;
    }

    let total_seconds = (plan_expires_at - plan_started_at).num_seconds() as f64;
    let seconds_remaining = (plan_expires_at - now).num_seconds() as f64;

    // Avoid division by zero
    if total_seconds <= 0.0 {
        return Decimal::ZERO;
    }

    // Pro-rata: (seconds_remaining / total_seconds) × original_price
    let ratio = Decimal::from_f64(seconds_remaining / total_seconds).unwrap_or(Decimal::ZERO);
    (original_price * ratio).round_dp(2)
}

/// Check if the plan change is an upgrade (higher price)
pub fn is_upgrade_allowed(current_plan_price: Decimal, new_plan_price: Decimal) -> bool {
    new_plan_price > current_plan_price
}

/// Calculate the final amount to pay after applying credit
pub fn calculate_amount_to_pay(
    new_plan_price: Decimal,
    credit: Decimal,
) -> Decimal {
    (new_plan_price - credit).max(Decimal::ZERO)
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Duration;
    use rust_decimal_macros::dec;

    #[test]
    fn test_full_term_no_credit() {
        let now = Utc::now();
        // Plan already expired
        let started = now - Duration::days(35);
        let expires = now - Duration::days(5);

        let credit = calculate_upgrade_credit(dec!(30.00), started, expires);
        assert_eq!(credit, Decimal::ZERO);
    }

    #[test]
    fn test_half_term_half_credit() {
        let now = Utc::now();
        // Plan is half-way through (15 of 30 days remaining)
        let started = now - Duration::days(15);
        let expires = now + Duration::days(15);

        let credit = calculate_upgrade_credit(dec!(30.00), started, expires);
        assert_eq!(credit, dec!(15.00));
    }

    #[test]
    fn test_full_term_remaining() {
        let now = Utc::now();
        // Brand new plan (30 days remaining of 30)
        let started = now;
        let expires = now + Duration::days(30);

        let credit = calculate_upgrade_credit(dec!(30.00), started, expires);
        // Should get almost all back (minus today)
        assert!(credit >= dec!(29.00));
    }

    #[test]
    fn test_upgrade_check() {
        assert!(is_upgrade_allowed(dec!(30.00), dec!(50.00)));
        assert!(!is_upgrade_allowed(dec!(50.00), dec!(30.00)));
        assert!(!is_upgrade_allowed(dec!(30.00), dec!(30.00)));
    }

    #[test]
    fn test_amount_to_pay() {
        // Upgrade from $30 to $50 with $15 credit
        let amount = calculate_amount_to_pay(dec!(50.00), dec!(15.00));
        assert_eq!(amount, dec!(35.00));

        // Credit exceeds new price (shouldn't happen but handle gracefully)
        let amount = calculate_amount_to_pay(dec!(20.00), dec!(25.00));
        assert_eq!(amount, Decimal::ZERO);
    }
}
