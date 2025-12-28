//! Upgrade Credit Calculator
//!
//! Calculates pro-rata credit when a user upgrades from a lower-priced plan
//! to a higher-priced plan. The remaining value from the old plan is credited
//! as bonus days on the new plan.
//!
//! Anti-exploit measures:
//! - Downgrades are blocked (no credit for going to cheaper plan)
//! - Bonus days are floored (rounded down) to prevent gaming
//! - Minimum 1 day remaining required to get any credit

use rust_decimal::Decimal;
use rust_decimal::prelude::*;

/// Result of an upgrade calculation
#[derive(Debug, Clone)]
pub struct UpgradeCalculation {
    /// Price of the old/current plan
    pub old_plan_price: Decimal,
    /// Price of the new plan being purchased
    pub new_plan_price: Decimal,
    /// Days remaining on the old plan
    pub days_remaining: i64,
    /// Dollar value of remaining time on old plan
    pub remaining_credit: Decimal,
    /// Bonus days earned from credit (always floored)
    pub bonus_days: i64,
    /// Total days on new plan (standard + bonus)
    pub total_new_duration_days: i64,
    /// Whether this is a valid upgrade (new > old price)
    pub is_valid_upgrade: bool,
}

/// Upgrade credit calculator with anti-exploit protections
pub struct UpgradeCalculator;

impl UpgradeCalculator {
    /// Standard plan duration in days
    pub const STANDARD_DURATION_DAYS: i64 = 30;
    
    /// Calculate upgrade credit and new duration
    ///
    /// # Arguments
    /// * `old_plan_price` - Monthly price of current plan
    /// * `new_plan_price` - Monthly price of new plan
    /// * `days_remaining` - Days left on current plan
    ///
    /// # Returns
    /// `UpgradeCalculation` with credit details
    ///
    /// # Anti-exploit measures
    /// - Returns `is_valid_upgrade: false` if new price <= old price
    /// - Bonus days are FLOORED to prevent rounding exploits
    /// - No credit if less than 1 day remaining
    pub fn calculate(
        old_plan_price: Decimal,
        new_plan_price: Decimal,
        days_remaining: i64,
    ) -> UpgradeCalculation {
        let is_valid_upgrade = Self::is_valid_upgrade(old_plan_price, new_plan_price);
        
        // Anti-exploit: No credit for invalid upgrades or expired plans
        if !is_valid_upgrade || days_remaining < 1 {
            return UpgradeCalculation {
                old_plan_price,
                new_plan_price,
                days_remaining,
                remaining_credit: Decimal::ZERO,
                bonus_days: 0,
                total_new_duration_days: Self::STANDARD_DURATION_DAYS,
                is_valid_upgrade,
            };
        }
        
        // 1. Calculate remaining value from old plan
        // remaining_credit = old_price × (days_remaining / 30)
        let days_ratio = Decimal::from(days_remaining) / Decimal::from(Self::STANDARD_DURATION_DAYS);
        let remaining_credit = old_plan_price * days_ratio;
        
        // 2. Calculate daily rate on new plan
        let daily_rate = new_plan_price / Decimal::from(Self::STANDARD_DURATION_DAYS);
        
        // 3. Calculate bonus days from credit (FLOOR to prevent exploit)
        let bonus_days = if daily_rate > Decimal::ZERO {
            // Use floor() for anti-exploit - always round down
            (remaining_credit / daily_rate).floor().to_i64().unwrap_or(0)
        } else {
            0
        };
        
        // 4. Total = standard + bonus
        let total_days = Self::STANDARD_DURATION_DAYS + bonus_days;
        
        UpgradeCalculation {
            old_plan_price,
            new_plan_price,
            days_remaining,
            remaining_credit,
            bonus_days,
            total_new_duration_days: total_days,
            is_valid_upgrade,
        }
    }
    
    /// Check if this is a valid upgrade (new plan must cost MORE)
    ///
    /// Downgrades are NOT allowed - users must wait for current plan to expire
    pub fn is_valid_upgrade(old_price: Decimal, new_price: Decimal) -> bool {
        new_price > old_price
    }
    
    /// Check if this is a downgrade (blocked operation)
    pub fn is_downgrade(old_price: Decimal, new_price: Decimal) -> bool {
        new_price < old_price
    }
    
    /// Check if same plan (just extension, not upgrade)
    pub fn is_same_plan(old_price: Decimal, new_price: Decimal) -> bool {
        old_price == new_price
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_upgrade_starter_to_pro() {
        // Starter $15 with 15 days remaining → Pro $30
        let calc = UpgradeCalculator::calculate(
            Decimal::from(15),  // Starter
            Decimal::from(30),  // Pro
            15,                 // 15 days left
        );
        
        assert!(calc.is_valid_upgrade);
        // Credit = $15 × (15/30) = $7.50
        assert_eq!(calc.remaining_credit, Decimal::from_str("7.5").unwrap());
        // Bonus = $7.50 / ($30/30) = 7.5 → FLOOR = 7 days
        assert_eq!(calc.bonus_days, 7);
        // Total = 30 + 7 = 37 days
        assert_eq!(calc.total_new_duration_days, 37);
    }
    
    #[test]
    fn test_downgrade_blocked() {
        // Pro $30 → Starter $15 (downgrade)
        let calc = UpgradeCalculator::calculate(
            Decimal::from(30),
            Decimal::from(15),
            15,
        );
        
        assert!(!calc.is_valid_upgrade);
        assert_eq!(calc.bonus_days, 0);
        assert_eq!(calc.total_new_duration_days, 30); // Just standard
    }
    
    #[test]
    fn test_same_plan_no_bonus() {
        // Pro $30 → Pro $30 (same plan = extension)
        let calc = UpgradeCalculator::calculate(
            Decimal::from(30),
            Decimal::from(30),
            15,
        );
        
        assert!(!calc.is_valid_upgrade); // Same plan is not an "upgrade"
        assert_eq!(calc.bonus_days, 0);
    }
    
    #[test]
    fn test_expired_plan_no_credit() {
        // 0 days remaining
        let calc = UpgradeCalculator::calculate(
            Decimal::from(15),
            Decimal::from(30),
            0,
        );
        
        assert_eq!(calc.bonus_days, 0);
        assert_eq!(calc.remaining_credit, Decimal::ZERO);
    }
    
    #[test]
    fn test_floor_prevents_exploit() {
        // Edge case: ensure flooring works
        // $14.99 plan with 29 days → $30 plan
        // Credit = $14.99 × (29/30) = $14.49
        // Bonus = $14.49 / $1 = 14.49 → FLOOR = 14 days (not 15)
        let calc = UpgradeCalculator::calculate(
            Decimal::from_str("14.99").unwrap(),
            Decimal::from(30),
            29,
        );
        
        assert_eq!(calc.bonus_days, 14); // Floored, not 15
    }
}
