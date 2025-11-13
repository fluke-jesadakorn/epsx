use crate::prelude::*;
use crate::domain::subscription_management::{Plan, Price, BillingCycle};
use rust_decimal::Decimal;

/// Domain service for pricing calculations
pub struct PricingService;

impl PricingService {
    /// Calculate prorated price for partial billing period
    pub fn calculate_prorated_price(
        plan: &Plan,
        days_used: i64,
    ) -> AppResult<Price> {
        let price = plan.price();

        let billing_days = match plan.billing_cycle().duration_days() {
            Some(days) => days as f64,
            None => return Ok(price.clone()), // Lifetime - no proration
        };

        let daily_rate = price.amount().to_string().parse::<f64>()
            .map_err(|e| AppError::internal_error(e.to_string()))?
            / billing_days;

        let prorated_amount = daily_rate * days_used as f64;
        let amount = Decimal::from_str_exact(&prorated_amount.to_string())
            .map_err(|e| AppError::internal_error(e.to_string()))?;

        Price::new(amount, price.currency())
    }

    /// Calculate discount based on billing cycle
    pub fn calculate_discount(_base_price: &Price, billing_cycle: &BillingCycle) -> f64 {
        match billing_cycle {
            BillingCycle::PayPerUse => 0.0,
            BillingCycle::Monthly => 0.0,
            BillingCycle::Quarterly => 0.05,  // 5% discount
            BillingCycle::Yearly => 0.15,     // 15% discount
            BillingCycle::Lifetime => 0.30,   // 30% discount
        }
    }

    /// Apply discount to price
    pub fn apply_discount(price: &Price, discount: f64) -> AppResult<Price> {
        if !(0.0..=1.0).contains(&discount) {
            return Err(AppError::validation_error("Discount must be between 0 and 1"));
        }

        let amount_f64 = price.amount().to_string().parse::<f64>()
            .map_err(|e| AppError::internal_error(e.to_string()))?;

        let discounted = amount_f64 * (1.0 - discount);
        let amount = Decimal::from_str_exact(&discounted.to_string())
            .map_err(|e| AppError::internal_error(e.to_string()))?;

        Price::new(amount, price.currency())
    }
}
