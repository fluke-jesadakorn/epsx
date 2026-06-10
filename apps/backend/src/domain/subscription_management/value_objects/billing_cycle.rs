use crate::prelude::*;
use std::str::FromStr;

/// Billing cycle value object
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[derive(Default)]
pub enum BillingCycle {
    #[default]
    PayPerUse,
    Monthly,
    Quarterly,
    Yearly,
    Lifetime,
}

impl BillingCycle {
    #[allow(clippy::should_implement_trait)]
    pub fn from_str(s: &str) -> AppResult<Self> {
        s.parse()
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            BillingCycle::PayPerUse => "pay_per_use",
            BillingCycle::Monthly => "monthly",
            BillingCycle::Quarterly => "quarterly",
            BillingCycle::Yearly => "yearly",
            BillingCycle::Lifetime => "lifetime",
        }
    }

    pub fn duration_days(&self) -> Option<i64> {
        match self {
            BillingCycle::PayPerUse => None,
            BillingCycle::Monthly => Some(30),
            BillingCycle::Quarterly => Some(90),
            BillingCycle::Yearly => Some(365),
            BillingCycle::Lifetime => None,
        }
    }
}

impl FromStr for BillingCycle {
    type Err = AppError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "pay_per_use" => Ok(BillingCycle::PayPerUse),
            "monthly" => Ok(BillingCycle::Monthly),
            "quarterly" => Ok(BillingCycle::Quarterly),
            "yearly" | "annual" => Ok(BillingCycle::Yearly),
            "lifetime" => Ok(BillingCycle::Lifetime),
            _ => Err(AppError::validation_error(format!("Invalid billing cycle: {}", s))),
        }
    }
}

impl std::fmt::Display for BillingCycle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

