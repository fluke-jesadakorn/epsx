use crate::prelude::*;
use rust_decimal::Decimal;

/// Price value object
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Price {
    amount: Decimal,
    currency: String,
}

impl Price {
    pub fn new(amount: Decimal, currency: impl Into<String>) -> AppResult<Self> {
        let currency = currency.into();

        if amount < Decimal::ZERO {
            return Err(AppError::validation_error("Price amount cannot be negative"));
        }

        if currency.is_empty() || currency.len() != 3 {
            return Err(AppError::validation_error("Currency must be a 3-letter code (e.g., USD)"));
        }

        Ok(Self { amount, currency: currency.to_uppercase() })
    }

    pub fn amount(&self) -> Decimal {
        self.amount
    }

    pub fn currency(&self) -> &str {
        &self.currency
    }

    pub fn to_f64(&self) -> f64 {
        self.amount.to_string().parse().unwrap_or(0.0)
    }
}

impl std::fmt::Display for Price {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {}", self.amount, self.currency)
    }
}
