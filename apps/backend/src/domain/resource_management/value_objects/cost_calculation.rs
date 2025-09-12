// Cost Calculation Value Objects
// Immutable objects for representing cost calculations and pricing

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Represents a cost calculation for resource usage
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CostCalculation {
    pub base_cost: Money,
    pub usage_costs: HashMap<String, Money>,
    pub overage_costs: HashMap<String, Money>,
    pub total_cost: Money,
    pub calculation_timestamp: chrono::DateTime<chrono::Utc>,
}

/// Represents a monetary amount with currency
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Money {
    pub amount: f64,
    pub currency: Currency,
}

/// Supported currencies
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Currency {
    USD,
    EUR,
    GBP,
    JPY,
}

/// Pricing model for resources
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PricingModel {
    pub resource_type: String,
    pub pricing_type: PricingType,
    pub base_price: Money,
    pub overage_price: Option<Money>,
    pub minimum_charge: Option<Money>,
}

/// Different pricing types
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum PricingType {
    PerUnit,
    Tiered(Vec<PricingTier>),
    Flat,
    PayPerUse,
}

/// Pricing tier for tiered pricing models
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PricingTier {
    pub from_quantity: i64,
    pub to_quantity: Option<i64>, // None for unlimited
    pub price_per_unit: Money,
}

impl CostCalculation {
    pub fn new(base_cost: Money) -> Self {
        Self {
            base_cost: base_cost.clone(),
            usage_costs: HashMap::new(),
            overage_costs: HashMap::new(),
            total_cost: base_cost,
            calculation_timestamp: chrono::Utc::now(),
        }
    }

    pub fn add_usage_cost(&mut self, resource_type: String, cost: Money) {
        self.usage_costs.insert(resource_type, cost.clone());
        self.recalculate_total();
    }

    pub fn add_overage_cost(&mut self, resource_type: String, cost: Money) {
        self.overage_costs.insert(resource_type, cost.clone());
        self.recalculate_total();
    }

    fn recalculate_total(&mut self) {
        let mut total = self.base_cost.amount;
        
        for cost in self.usage_costs.values() {
            total += cost.amount;
        }
        
        for cost in self.overage_costs.values() {
            total += cost.amount;
        }
        
        self.total_cost = Money {
            amount: total,
            currency: self.base_cost.currency.clone(),
        };
    }
}

impl Money {
    pub fn new(amount: f64, currency: Currency) -> Self {
        Self { amount, currency }
    }

    pub fn usd(amount: f64) -> Self {
        Self::new(amount, Currency::USD)
    }

    pub fn add(&self, other: &Money) -> Result<Money, String> {
        if self.currency != other.currency {
            return Err("Cannot add money with different currencies".to_string());
        }
        
        Ok(Money {
            amount: self.amount + other.amount,
            currency: self.currency.clone(),
        })
    }

    pub fn multiply(&self, factor: f64) -> Money {
        Money {
            amount: self.amount * factor,
            currency: self.currency.clone(),
        }
    }
}

impl PricingModel {
    pub fn calculate_cost(&self, quantity: i64) -> Money {
        match &self.pricing_type {
            PricingType::PerUnit => {
                Money {
                    amount: self.base_price.amount * quantity as f64,
                    currency: self.base_price.currency.clone(),
                }
            },
            PricingType::Flat => self.base_price.clone(),
            PricingType::PayPerUse => {
                Money {
                    amount: self.base_price.amount * quantity as f64,
                    currency: self.base_price.currency.clone(),
                }
            },
            PricingType::Tiered(tiers) => {
                let mut total_cost = 0.0;
                let mut remaining_quantity = quantity;

                for tier in tiers {
                    if remaining_quantity <= 0 {
                        break;
                    }

                    let tier_max = match tier.to_quantity {
                        Some(max) => (max - tier.from_quantity + 1).min(remaining_quantity),
                        None => remaining_quantity,
                    };

                    if remaining_quantity >= tier.from_quantity {
                        let quantity_in_tier = tier_max.min(remaining_quantity);
                        total_cost += tier.price_per_unit.amount * quantity_in_tier as f64;
                        remaining_quantity -= quantity_in_tier;
                    }
                }

                Money {
                    amount: total_cost,
                    currency: self.base_price.currency.clone(),
                }
            }
        }
    }
}

impl std::fmt::Display for Currency {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Currency::USD => write!(f, "USD"),
            Currency::EUR => write!(f, "EUR"),
            Currency::GBP => write!(f, "GBP"),
            Currency::JPY => write!(f, "JPY"),
        }
    }
}

impl std::fmt::Display for Money {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:.2} {}", self.amount, self.currency)
    }
}