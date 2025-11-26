// Billing Calculation Service
// Domain service for calculating bills and overages

use crate::domain::resource_management::aggregates::UserResourceUsage;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BillingSummary {
    pub wallet_address: String,
    pub plan_id: Option<i32>,
    pub billing_period_start: DateTime<Utc>,
    pub billing_period_end: DateTime<Utc>,
    pub base_cost: f64,
    pub overage_costs: HashMap<String, f64>,
    pub total_cost: f64,
    pub currency: String,
}

pub struct BillingCalculationService;

impl BillingCalculationService {
    pub fn new() -> Self {
        Self
    }

    /// Calculate billing for a user's resource usage
    pub fn calculate_billing(
        &self,
        usage: &UserResourceUsage,
        base_plan_cost: f64,
        overage_pricing: &HashMap<String, f64>,
    ) -> BillingSummary {
        let mut overage_costs = HashMap::new();
        let mut total_overage = 0.0;

        // Calculate overage costs for each resource type
        for (resource_type, current_usage) in usage.current_usage() {
            if let Some(limit) = usage.quota_limits().get(resource_type) {
                if *limit > 0 && *current_usage > *limit {
                    let overage_amount = current_usage - limit;
                    if let Some(price_per_unit) = overage_pricing.get(resource_type) {
                        let overage_cost = (overage_amount as f64) * price_per_unit;
                        overage_costs.insert(resource_type.clone(), overage_cost);
                        total_overage += overage_cost;
                    }
                }
            }
        }

        BillingSummary {
            wallet_address: usage.wallet_address().to_string(),
            plan_id: usage.plan_id(),
            billing_period_start: usage.billing_period_start(),
            billing_period_end: usage.billing_period_end(),
            base_cost: base_plan_cost,
            overage_costs,
            total_cost: base_plan_cost + total_overage,
            currency: "USD".to_string(),
        }
    }

    /// Calculate prorated cost for plan changes
    pub fn calculate_prorated_cost(
        &self,
        old_plan_cost: f64,
        new_plan_cost: f64,
        days_remaining: u32,
        total_days_in_period: u32,
    ) -> f64 {
        if total_days_in_period == 0 {
            return 0.0;
        }

        let daily_old_rate = old_plan_cost / total_days_in_period as f64;
        let daily_new_rate = new_plan_cost / total_days_in_period as f64;
        
        let remaining_old_cost = daily_old_rate * days_remaining as f64;
        let new_cost_for_period = daily_new_rate * days_remaining as f64;
        
        new_cost_for_period - remaining_old_cost
    }

    /// Generate billing forecast based on current usage trends
    pub fn forecast_billing(
        &self,
        usage: &UserResourceUsage,
        base_plan_cost: f64,
        overage_pricing: &HashMap<String, f64>,
        days_in_period: u32,
        current_day: u32,
    ) -> BillingSummary {
        if current_day == 0 || days_in_period == 0 {
            return self.calculate_billing(usage, base_plan_cost, overage_pricing);
        }

        // Calculate projected usage based on current trends
        let projection_multiplier = days_in_period as f64 / current_day as f64;
        
        let mut projected_usage = usage.clone();
        for (resource_type, current) in &usage.current_usage {
            let projected = (*current as f64 * projection_multiplier) as i64;
            projected_usage.current_usage.insert(resource_type.clone(), projected);
        }

        self.calculate_billing(&projected_usage, base_plan_cost, overage_pricing)
    }
}

impl Default for BillingCalculationService {
    fn default() -> Self {
        Self::new()
    }
}