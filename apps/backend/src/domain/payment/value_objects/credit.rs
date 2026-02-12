use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use rust_decimal::prelude::ToPrimitive;
use std::fmt::{self, Display};
use serde::{Serialize, Deserialize};

/// Credit Amount Value Object
/// Represents a credit amount with validation (always in USD equivalent)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct CreditAmount {
    amount: Decimal,
}

impl CreditAmount {
    /// Create new credit amount with validation
    pub fn new(amount: Decimal) -> Result<Self, CreditError> {
        // Validate amount is non-negative (credits can be zero)
        if amount < Decimal::ZERO {
            return Err(CreditError::NegativeAmount(amount));
        }

        // Validate max 2 decimal places (cents precision)
        if Self::count_decimal_places(amount) > 2 {
            return Err(CreditError::TooManyDecimals {
                amount,
                max_decimals: 2,
            });
        }

        // Validate reasonable maximum (prevent overflow)
        const MAX_CREDIT: Decimal = dec!(1_000_000); // $1M max
        if amount > MAX_CREDIT {
            return Err(CreditError::AmountTooLarge {
                amount,
                maximum: MAX_CREDIT,
            });
        }

        Ok(Self { amount })
    }

    /// Create zero credit amount
    pub fn zero() -> Self {
        Self {
            amount: Decimal::ZERO,
        }
    }

    /// Create from dollar amount (with validation)
    pub fn from_dollars(dollars: Decimal) -> Result<Self, CreditError> {
        Self::new(dollars)
    }

    /// Create from cents (avoids floating point issues)
    pub fn from_cents(cents: i64) -> Result<Self, CreditError> {
        let amount = Decimal::from(cents) / dec!(100);
        Self::new(amount)
    }

    /// Get the amount
    pub fn amount(&self) -> Decimal {
        self.amount
    }

    /// Get as cents (for precise integer arithmetic)
    pub fn as_cents(&self) -> i64 {
        (self.amount * dec!(100)).to_i64().unwrap_or(0)
    }

    /// Check if amount is zero
    pub fn is_zero(&self) -> bool {
        self.amount == Decimal::ZERO
    }

    /// Check if amount is positive
    pub fn is_positive(&self) -> bool {
        self.amount > Decimal::ZERO
    }

    /// Add two credit amounts
    pub fn add(&self, other: &CreditAmount) -> Result<Self, CreditError> {
        Self::new(self.amount + other.amount)
    }

    /// Subtract two credit amounts
    pub fn subtract(&self, other: &CreditAmount) -> Result<Self, CreditError> {
        let result = self.amount - other.amount;
        if result < Decimal::ZERO {
            return Err(CreditError::InsufficientCredits {
                available: self.amount,
                required: other.amount,
            });
        }
        Self::new(result)
    }

    /// Check if this amount can cover another amount
    pub fn can_cover(&self, other: &CreditAmount) -> bool {
        self.amount >= other.amount
    }

    /// Format as currency string
    pub fn as_currency_string(&self) -> String {
        format!("${:.2}", self.amount)
    }

    /// Count decimal places in a Decimal
    fn count_decimal_places(value: Decimal) -> usize {
        let scale = value.scale();
        scale as usize
    }
}

impl Display for CreditAmount {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "${:.2}", self.amount)
    }
}

impl Default for CreditAmount {
    fn default() -> Self {
        Self::zero()
    }
}

impl From<CreditAmount> for Decimal {
    fn from(credit: CreditAmount) -> Self {
        credit.amount
    }
}

impl TryFrom<Decimal> for CreditAmount {
    type Error = CreditError;

    fn try_from(value: Decimal) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

/// Credit Transaction Type
/// Represents different types of credit transactions
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CreditTransactionType {
    /// Admin grants credits to user
    Grant,
    /// Admin revokes credits from user
    Revoke,
    /// Credits deducted for payment
    PaymentDebit,
    /// Credits added from plan proration (upgrade/downgrade)
    ProrationCredit,
    /// Credits added from refund
    Refund,
    /// Credits expired (time-limited credits)
    Expiry,
    /// Manual adjustment by admin
    Adjustment,
}

impl CreditTransactionType {
    /// Check if transaction type adds credits
    pub fn is_credit(&self) -> bool {
        matches!(
            self,
            Self::Grant | Self::ProrationCredit | Self::Refund | Self::Adjustment
        )
    }

    /// Check if transaction type removes credits
    pub fn is_debit(&self) -> bool {
        matches!(
            self,
            Self::Revoke | Self::PaymentDebit | Self::Expiry
        )
    }

    /// Get human-readable label
    pub fn label(&self) -> &str {
        match self {
            Self::Grant => "Admin Grant",
            Self::Revoke => "Admin Revoke",
            Self::PaymentDebit => "Payment",
            Self::ProrationCredit => "Proration Credit",
            Self::Refund => "Refund",
            Self::Expiry => "Expired",
            Self::Adjustment => "Adjustment",
        }
    }

    /// Get all transaction types
    pub fn all() -> Vec<Self> {
        vec![
            Self::Grant,
            Self::Revoke,
            Self::PaymentDebit,
            Self::ProrationCredit,
            Self::Refund,
            Self::Expiry,
            Self::Adjustment,
        ]
    }

    /// Parse from string
    pub fn from_str(s: &str) -> Result<Self, CreditError> {
        match s.to_lowercase().as_str() {
            "grant" => Ok(Self::Grant),
            "revoke" => Ok(Self::Revoke),
            "payment_debit" => Ok(Self::PaymentDebit),
            "proration_credit" => Ok(Self::ProrationCredit),
            "refund" => Ok(Self::Refund),
            "expiry" => Ok(Self::Expiry),
            "adjustment" => Ok(Self::Adjustment),
            _ => Err(CreditError::InvalidTransactionType(s.to_string())),
        }
    }

    /// Convert to database string
    pub fn as_db_string(&self) -> &str {
        match self {
            Self::Grant => "grant",
            Self::Revoke => "revoke",
            Self::PaymentDebit => "payment_debit",
            Self::ProrationCredit => "proration_credit",
            Self::Refund => "refund",
            Self::Expiry => "expiry",
            Self::Adjustment => "adjustment",
        }
    }
}

impl Display for CreditTransactionType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.label())
    }
}

/// Credit Error Types
#[derive(Debug, Clone, PartialEq)]
pub enum CreditError {
    NegativeAmount(Decimal),
    TooManyDecimals {
        amount: Decimal,
        max_decimals: usize,
    },
    AmountTooLarge {
        amount: Decimal,
        maximum: Decimal,
    },
    InsufficientCredits {
        available: Decimal,
        required: Decimal,
    },
    InvalidTransactionType(String),
}

impl Display for CreditError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NegativeAmount(amount) => {
                write!(f, "Credit amount cannot be negative: {}", amount)
            }
            Self::TooManyDecimals { amount, max_decimals } => {
                write!(
                    f,
                    "Credit amount {} has too many decimal places (max: {})",
                    amount, max_decimals
                )
            }
            Self::AmountTooLarge { amount, maximum } => {
                write!(
                    f,
                    "Credit amount {} exceeds maximum allowed: {}",
                    amount, maximum
                )
            }
            Self::InsufficientCredits { available, required } => {
                write!(
                    f,
                    "Insufficient credits: available ${:.2}, required ${:.2}",
                    available, required
                )
            }
            Self::InvalidTransactionType(type_str) => {
                write!(f, "Invalid credit transaction type: {}", type_str)
            }
        }
    }
}

impl std::error::Error for CreditError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_credit_amount_creation() {
        let amount = CreditAmount::new(dec!(100.50)).unwrap();
        assert_eq!(amount.amount(), dec!(100.50));
        assert!(!amount.is_zero());
        assert!(amount.is_positive());
    }

    #[test]
    fn test_credit_amount_zero() {
        let amount = CreditAmount::zero();
        assert!(amount.is_zero());
        assert!(!amount.is_positive());
    }

    #[test]
    fn test_credit_amount_from_cents() {
        let amount = CreditAmount::from_cents(12345).unwrap();
        assert_eq!(amount.amount(), dec!(123.45));
        assert_eq!(amount.as_cents(), 12345);
    }

    #[test]
    fn test_credit_amount_negative_fails() {
        let result = CreditAmount::new(dec!(-10.00));
        assert!(result.is_err());
    }

    #[test]
    fn test_credit_amount_too_many_decimals_fails() {
        let result = CreditAmount::new(dec!(10.123)); // 3 decimals
        assert!(result.is_err());
    }

    #[test]
    fn test_credit_amount_add() {
        let a = CreditAmount::new(dec!(50.00)).unwrap();
        let b = CreditAmount::new(dec!(25.50)).unwrap();
        let result = a.add(&b).unwrap();
        assert_eq!(result.amount(), dec!(75.50));
    }

    #[test]
    fn test_credit_amount_subtract() {
        let a = CreditAmount::new(dec!(100.00)).unwrap();
        let b = CreditAmount::new(dec!(25.50)).unwrap();
        let result = a.subtract(&b).unwrap();
        assert_eq!(result.amount(), dec!(74.50));
    }

    #[test]
    fn test_credit_amount_subtract_insufficient() {
        let a = CreditAmount::new(dec!(10.00)).unwrap();
        let b = CreditAmount::new(dec!(25.00)).unwrap();
        let result = a.subtract(&b);
        assert!(result.is_err());
    }

    #[test]
    fn test_credit_amount_can_cover() {
        let a = CreditAmount::new(dec!(100.00)).unwrap();
        let b = CreditAmount::new(dec!(50.00)).unwrap();
        let c = CreditAmount::new(dec!(150.00)).unwrap();

        assert!(a.can_cover(&b));
        assert!(!a.can_cover(&c));
    }

    #[test]
    fn test_transaction_type_is_credit() {
        assert!(CreditTransactionType::Grant.is_credit());
        assert!(CreditTransactionType::ProrationCredit.is_credit());
        assert!(CreditTransactionType::Refund.is_credit());
        assert!(!CreditTransactionType::PaymentDebit.is_credit());
    }

    #[test]
    fn test_transaction_type_is_debit() {
        assert!(CreditTransactionType::PaymentDebit.is_debit());
        assert!(CreditTransactionType::Revoke.is_debit());
        assert!(CreditTransactionType::Expiry.is_debit());
        assert!(!CreditTransactionType::Grant.is_debit());
    }

    #[test]
    fn test_transaction_type_from_str() {
        assert_eq!(
            CreditTransactionType::from_str("grant").unwrap(),
            CreditTransactionType::Grant
        );
        assert_eq!(
            CreditTransactionType::from_str("payment_debit").unwrap(),
            CreditTransactionType::PaymentDebit
        );
        assert!(CreditTransactionType::from_str("invalid").is_err());
    }

    #[test]
    fn test_credit_amount_display() {
        let amount = CreditAmount::new(dec!(123.45)).unwrap();
        assert_eq!(format!("{}", amount), "$123.45");
        assert_eq!(amount.as_currency_string(), "$123.45");
    }
}
