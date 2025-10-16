use crate::prelude::*;

/// Billing cycle value object
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[derive(Default)]
pub enum BillingCycle {
    #[default]
    Monthly,
    Quarterly,
    Yearly,
    Lifetime,
}

impl BillingCycle {
    pub fn from_str(s: &str) -> AppResult<Self> {
        match s.to_lowercase().as_str() {
            "monthly" => Ok(BillingCycle::Monthly),
            "quarterly" => Ok(BillingCycle::Quarterly),
            "yearly" | "annual" => Ok(BillingCycle::Yearly),
            "lifetime" => Ok(BillingCycle::Lifetime),
            _ => Err(AppError::validation_error(format!("Invalid billing cycle: {}", s))),
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            BillingCycle::Monthly => "monthly",
            BillingCycle::Quarterly => "quarterly",
            BillingCycle::Yearly => "yearly",
            BillingCycle::Lifetime => "lifetime",
        }
    }

    pub fn duration_days(&self) -> Option<i64> {
        match self {
            BillingCycle::Monthly => Some(30),
            BillingCycle::Quarterly => Some(90),
            BillingCycle::Yearly => Some(365),
            BillingCycle::Lifetime => None,
        }
    }
}

impl std::fmt::Display for BillingCycle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

