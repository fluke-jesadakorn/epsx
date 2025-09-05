use serde::{Deserialize, Serialize};
use std::fmt::{self, Display};
use rust_decimal::Decimal;
use std::collections::HashMap;

/// Re-export Currency and Network from existing types
pub use crate::dom::values::payments::{Currency, Network};

/// Payment Amount Value Object
/// Represents an amount with currency and validation rules
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PaymentAmount {
    amount: Decimal,
    currency: Currency,
}

impl PaymentAmount {
    /// Create new payment amount with validation
    pub fn new(amount: Decimal, currency: Currency) -> Result<Self, PaymentAmountError> {
        // Validate amount is positive
        if amount <= Decimal::ZERO {
            return Err(PaymentAmountError::InvalidAmount(amount));
        }

        // Check minimum amount requirements
        let min_amount = Self::minimum_amount_for_currency(&currency);
        if amount < min_amount {
            return Err(PaymentAmountError::AmountTooSmall {
                amount,
                minimum: min_amount,
                currency: currency.clone(),
            });
        }

        // Check maximum amount requirements
        let max_amount = Self::maximum_amount_for_currency(&currency);
        if amount > max_amount {
            return Err(PaymentAmountError::AmountTooLarge {
                amount,
                maximum: max_amount,
                currency: currency.clone(),
            });
        }

        // Validate decimal places
        let max_decimals = currency.decimals();
        if Self::count_decimal_places(amount) > max_decimals as usize {
            return Err(PaymentAmountError::TooManyDecimals {
                amount,
                max_decimals,
                currency: currency.clone(),
            });
        }

        Ok(Self { amount, currency })
    }

    /// Create zero amount for a currency
    pub fn zero(currency: Currency) -> Self {
        Self {
            amount: Decimal::ZERO,
            currency,
        }
    }

    /// Get the amount
    pub fn amount(&self) -> Decimal {
        self.amount
    }

    /// Get the currency
    pub fn currency(&self) -> &Currency {
        &self.currency
    }

    /// Check if amount is zero
    pub fn is_zero(&self) -> bool {
        self.amount == Decimal::ZERO
    }

    /// Check if amount is positive
    pub fn is_positive(&self) -> bool {
        self.amount > Decimal::ZERO
    }

    /// Add two amounts (must be same currency)
    pub fn add(&self, other: &PaymentAmount) -> Result<PaymentAmount, PaymentAmountError> {
        if self.currency != other.currency {
            return Err(PaymentAmountError::CurrencyMismatch {
                left: self.currency.clone(),
                right: other.currency.clone(),
            });
        }

        PaymentAmount::new(self.amount + other.amount, self.currency.clone())
    }

    /// Subtract two amounts (must be same currency)
    pub fn subtract(&self, other: &PaymentAmount) -> Result<PaymentAmount, PaymentAmountError> {
        if self.currency != other.currency {
            return Err(PaymentAmountError::CurrencyMismatch {
                left: self.currency.clone(),
                right: other.currency.clone(),
            });
        }

        if other.amount > self.amount {
            return Err(PaymentAmountError::InsufficientAmount {
                available: self.amount,
                required: other.amount,
            });
        }

        PaymentAmount::new(self.amount - other.amount, self.currency.clone())
    }

    /// Multiply amount by factor
    pub fn multiply(&self, factor: Decimal) -> Result<PaymentAmount, PaymentAmountError> {
        if factor <= Decimal::ZERO {
            return Err(PaymentAmountError::InvalidMultiplier(factor));
        }

        PaymentAmount::new(self.amount * factor, self.currency.clone())
    }

    /// Get minimum amount for currency
    pub fn minimum_amount_for_currency(currency: &Currency) -> Decimal {
        match currency {
            Currency::USD => Decimal::from(10),        // $10 minimum
            Currency::USDT => Decimal::from(10),       // $10 minimum
            Currency::USDC => Decimal::from(10),       // $10 minimum
            Currency::ETH => "0.01".parse().unwrap(),  // 0.01 ETH minimum
            Currency::BTC => "0.001".parse().unwrap(), // 0.001 BTC minimum
            Currency::BNB => "0.1".parse().unwrap(),   // 0.1 BNB minimum
            Currency::TRX => Decimal::from(100),       // 100 TRX minimum
        }
    }

    /// Get maximum amount for currency
    pub fn maximum_amount_for_currency(currency: &Currency) -> Decimal {
        match currency {
            Currency::USD => Decimal::from(1_000_000),    // $1M maximum
            Currency::USDT => Decimal::from(1_000_000),   // $1M maximum
            Currency::USDC => Decimal::from(1_000_000),   // $1M maximum
            Currency::ETH => Decimal::from(1000),         // 1000 ETH maximum
            Currency::BTC => Decimal::from(100),          // 100 BTC maximum
            Currency::BNB => Decimal::from(10_000),       // 10k BNB maximum
            Currency::TRX => Decimal::from(10_000_000),   // 10M TRX maximum
        }
    }

    /// Format amount with currency symbol
    pub fn format(&self) -> String {
        match self.currency {
            Currency::USD => format!("${}", self.amount),
            _ => format!("{} {}", self.amount, self.currency.symbol()),
        }
    }

    /// Format amount with full precision
    pub fn format_precise(&self) -> String {
        format!("{} {}", self.amount, self.currency.symbol())
    }

    /// Get amount in smallest unit (e.g., cents for USD, wei for ETH)
    pub fn to_smallest_unit(&self) -> Decimal {
        let decimals = self.currency.decimals();
        let multiplier = Decimal::from(10_u64.pow(decimals as u32));
        self.amount * multiplier
    }

    /// Create from smallest unit
    pub fn from_smallest_unit(amount: Decimal, currency: Currency) -> Result<Self, PaymentAmountError> {
        let decimals = currency.decimals();
        let divisor = Decimal::from(10_u64.pow(decimals as u32));
        let actual_amount = amount / divisor;
        
        PaymentAmount::new(actual_amount, currency)
    }

    /// Convert to USD equivalent (placeholder - would integrate with price feeds)
    pub fn to_usd_equivalent(&self, exchange_rates: &ExchangeRates) -> Option<PaymentAmount> {
        if matches!(self.currency, Currency::USD) {
            return Some(self.clone());
        }

        exchange_rates.get_rate(&self.currency).map(|rate| {
            let usd_amount = self.amount * rate;
            PaymentAmount {
                amount: usd_amount,
                currency: Currency::USD,
            }
        })
    }

    /// Check if amount is within processing fee range
    pub fn processing_fee(&self) -> PaymentAmount {
        let fee_rate = match self.currency {
            Currency::USD => "0.03".parse::<Decimal>().unwrap(),  // 3% for USD
            Currency::USDT | Currency::USDC => "0.02".parse::<Decimal>().unwrap(), // 2% for stablecoins
            Currency::ETH => "0.025".parse::<Decimal>().unwrap(), // 2.5% for ETH
            Currency::BTC => "0.015".parse::<Decimal>().unwrap(), // 1.5% for BTC
            Currency::BNB => "0.02".parse::<Decimal>().unwrap(),  // 2% for BNB
            Currency::TRX => "0.03".parse::<Decimal>().unwrap(),  // 3% for TRX
        };

        let fee_amount = self.amount * fee_rate;
        PaymentAmount::new(fee_amount, self.currency.clone()).unwrap_or_else(|_| {
            PaymentAmount::zero(self.currency.clone())
        })
    }

    /// Get amount after deducting processing fee
    pub fn amount_after_fees(&self) -> PaymentAmount {
        let fee = self.processing_fee();
        self.subtract(&fee).unwrap_or_else(|_| PaymentAmount::zero(self.currency.clone()))
    }

    fn count_decimal_places(value: Decimal) -> usize {
        let str_repr = value.to_string();
        if let Some(dot_pos) = str_repr.find('.') {
            str_repr.len() - dot_pos - 1
        } else {
            0
        }
    }
}

/// Exchange rates for currency conversion
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExchangeRates {
    rates: HashMap<Currency, Decimal>, // Currency to USD rate
    last_updated: chrono::DateTime<chrono::Utc>,
}

impl ExchangeRates {
    /// Create new exchange rates
    pub fn new() -> Self {
        Self {
            rates: HashMap::new(),
            last_updated: chrono::Utc::now(),
        }
    }

    /// Add or update rate
    pub fn set_rate(&mut self, currency: Currency, usd_rate: Decimal) {
        self.rates.insert(currency, usd_rate);
        self.last_updated = chrono::Utc::now();
    }

    /// Get USD rate for currency
    pub fn get_rate(&self, currency: &Currency) -> Option<Decimal> {
        self.rates.get(currency).copied()
    }

    /// Check if rates are stale (older than 5 minutes)
    pub fn is_stale(&self) -> bool {
        let now = chrono::Utc::now();
        let age = now - self.last_updated;
        age > chrono::Duration::minutes(5)
    }

    /// Get last updated timestamp
    pub fn last_updated(&self) -> chrono::DateTime<chrono::Utc> {
        self.last_updated
    }
}

impl Default for ExchangeRates {
    fn default() -> Self {
        Self::new()
    }
}

impl Display for PaymentAmount {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.format())
    }
}

#[derive(Debug, thiserror::Error)]
pub enum PaymentAmountError {
    #[error("Invalid amount: {0} must be positive")]
    InvalidAmount(Decimal),

    #[error("Amount {amount} too small for {currency}, minimum is {minimum}")]
    AmountTooSmall {
        amount: Decimal,
        minimum: Decimal,
        currency: Currency,
    },

    #[error("Amount {amount} too large for {currency}, maximum is {maximum}")]
    AmountTooLarge {
        amount: Decimal,
        maximum: Decimal,
        currency: Currency,
    },

    #[error("Amount {amount} has too many decimals for {currency}, maximum is {max_decimals}")]
    TooManyDecimals {
        amount: Decimal,
        max_decimals: u8,
        currency: Currency,
    },

    #[error("Currency mismatch: {left} != {right}")]
    CurrencyMismatch { left: Currency, right: Currency },

    #[error("Insufficient amount: available {available}, required {required}")]
    InsufficientAmount {
        available: Decimal,
        required: Decimal,
    },

    #[error("Invalid multiplier: {0}")]
    InvalidMultiplier(Decimal),
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn test_payment_amount_creation() {
        let amount = PaymentAmount::new(dec!(50.0), Currency::USDT).unwrap();
        assert_eq!(amount.amount(), dec!(50.0));
        assert_eq!(amount.currency(), &Currency::USDT);
        assert!(amount.is_positive());
        assert!(!amount.is_zero());
    }

    #[test]
    fn test_minimum_amount_validation() {
        let too_small = PaymentAmount::new(dec!(5.0), Currency::USDT);
        assert!(too_small.is_err());

        let valid = PaymentAmount::new(dec!(15.0), Currency::USDT);
        assert!(valid.is_ok());
    }

    #[test]
    fn test_decimal_places_validation() {
        // USDT has 6 decimal places
        let valid = PaymentAmount::new("10.123456".parse().unwrap(), Currency::USDT);
        assert!(valid.is_ok());

        let too_many_decimals = PaymentAmount::new("10.1234567".parse().unwrap(), Currency::USDT);
        assert!(too_many_decimals.is_err());
    }

    #[test]
    fn test_amount_addition() {
        let amount1 = PaymentAmount::new(dec!(50.0), Currency::USDT).unwrap();
        let amount2 = PaymentAmount::new(dec!(25.0), Currency::USDT).unwrap();
        
        let sum = amount1.add(&amount2).unwrap();
        assert_eq!(sum.amount(), dec!(75.0));
        assert_eq!(sum.currency(), &Currency::USDT);
    }

    #[test]
    fn test_currency_mismatch() {
        let usd_amount = PaymentAmount::new(dec!(50.0), Currency::USD).unwrap();
        let usdt_amount = PaymentAmount::new(dec!(25.0), Currency::USDT).unwrap();
        
        let result = usd_amount.add(&usdt_amount);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), PaymentAmountError::CurrencyMismatch { .. }));
    }

    #[test]
    fn test_amount_subtraction() {
        let amount1 = PaymentAmount::new(dec!(50.0), Currency::USDT).unwrap();
        let amount2 = PaymentAmount::new(dec!(25.0), Currency::USDT).unwrap();
        
        let diff = amount1.subtract(&amount2).unwrap();
        assert_eq!(diff.amount(), dec!(25.0));

        // Test insufficient amount
        let insufficient = amount2.subtract(&amount1);
        assert!(insufficient.is_err());
    }

    #[test]
    fn test_processing_fee() {
        let amount = PaymentAmount::new(dec!(100.0), Currency::USDT).unwrap();
        let fee = amount.processing_fee();
        
        assert_eq!(fee.amount(), dec!(2.0)); // 2% of 100
        assert_eq!(fee.currency(), &Currency::USDT);

        let after_fees = amount.amount_after_fees();
        assert_eq!(after_fees.amount(), dec!(98.0));
    }

    #[test]
    fn test_format() {
        let usd_amount = PaymentAmount::new(dec!(123.45), Currency::USD).unwrap();
        assert_eq!(usd_amount.format(), "$123.45");

        let eth_amount = PaymentAmount::new(dec!(1.5), Currency::ETH).unwrap();
        assert_eq!(eth_amount.format(), "1.5 ETH");
    }

    #[test]
    fn test_smallest_unit_conversion() {
        let eth_amount = PaymentAmount::new(dec!(1.0), Currency::ETH).unwrap();
        let wei = eth_amount.to_smallest_unit();
        
        // 1 ETH = 10^18 wei
        let expected_wei = "1000000000000000000".parse::<Decimal>().unwrap();
        assert_eq!(wei, expected_wei);

        let back_to_eth = PaymentAmount::from_smallest_unit(wei, Currency::ETH).unwrap();
        assert_eq!(back_to_eth.amount(), dec!(1.0));
    }

    #[test]
    fn test_exchange_rates() {
        let mut rates = ExchangeRates::new();
        rates.set_rate(Currency::ETH, dec!(2000.0)); // 1 ETH = $2000
        
        let eth_amount = PaymentAmount::new(dec!(0.5), Currency::ETH).unwrap();
        let usd_equivalent = eth_amount.to_usd_equivalent(&rates).unwrap();
        
        assert_eq!(usd_equivalent.amount(), dec!(1000.0));
        assert_eq!(usd_equivalent.currency(), &Currency::USD);
    }
}