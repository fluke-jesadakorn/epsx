// Payment DTOs

use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Serialize, Deserialize};

use crate::dom::entities::Payment;
use crate::dom::values::{PayId, UserId, Currency, PayStatus};

// Payment DTO for API responses
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaymentDto {
    pub id: String,
    pub usr_id: String,
    pub amt: Decimal,
    pub curr: String,
    pub stat: String,
    pub addr: Option<String>,
    pub tx_hash: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// Create payment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreatePayReq {
    pub usr_id: UserId,
    pub amt: Decimal,
    pub curr: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreatePayRes {
    pub pay: PaymentDto,
    pub addr: Option<String>,
    pub qr_code: Option<String>,
}

// Get payment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetPayReq {
    pub pay_id: PayId,
    pub usr_id: UserId, // For authorization
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetPayRes {
    pub pay: PaymentDto,
}

// Confirm payment (webhook)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfirmPayReq {
    pub pay_id: PayId,
    pub tx_hash: String,
    pub confirmations: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfirmPayRes {
    pub pay: PaymentDto,
    pub usr_notified: bool,
}

// List payments for user
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListPayReq {
    pub usr_id: UserId,
    pub offset: u32,
    pub limit: u32,
    pub status_filter: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListPayRes {
    pub payments: Vec<PaymentDto>,
    pub total: u64,
    pub offset: u32,
    pub limit: u32,
}

// Payment statistics (admin)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PayStatsReq {
    pub start_date: DateTime<Utc>,
    pub end_date: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PayStatsRes {
    pub total_payments: u64,
    pub total_revenue: Decimal,
    pub pending_payments: u64,
    pub completed_payments: u64,
    pub failed_payments: u64,
    pub avg_payment_amount: Decimal,
    pub revenue_by_currency: Vec<CurrencyRevenue>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CurrencyRevenue {
    pub curr: String,
    pub revenue: Decimal,
    pub count: u64,
}

// Utility implementations
impl PaymentDto {
    pub fn from_entity(payment: &Payment) -> Self {
        Self {
            id: payment.id().to_string(),
            usr_id: payment.uid().to_string(),
            amt: payment.amt(),
            curr: payment.curr().to_string(),
            stat: payment.stat().to_string(),
            addr: payment.addr().map(|s| s.to_string()),
            tx_hash: payment.tx_hash().map(|s| s.to_string()),
            created_at: payment.created_at(),
            updated_at: payment.updated_at(),
        }
    }
}

impl CreatePayReq {
    pub fn validate(&self) -> Result<(), ValidationError> {
        if self.amt <= Decimal::ZERO {
            return Err(ValidationError::InvalidAmount);
        }
        
        let currency = self.curr.parse::<Currency>()
            .map_err(|_| ValidationError::UnsupportedCurrency(self.curr.clone()))?;
        
        // Check minimum amount based on currency
        let min_amount = match currency {
            Currency::USDT | Currency::USDC => Decimal::from(10), // $10 minimum
            Currency::ETH => "0.01".parse::<Decimal>().unwrap(), // 0.01 ETH minimum
            Currency::BTC => "0.001".parse::<Decimal>().unwrap(), // 0.001 BTC minimum
            Currency::BNB => "0.1".parse::<Decimal>().unwrap(), // 0.1 BNB minimum
            Currency::TRX => Decimal::from(100), // 100 TRX minimum
            Currency::USD => Decimal::from(10), // $10 minimum
        };
        
        if self.amt < min_amount {
            return Err(ValidationError::AmountTooSmall(min_amount));
        }
        
        Ok(())
    }
}

impl ListPayReq {
    pub fn validate(&self) -> Result<(), ValidationError> {
        if self.limit > 1000 {
            return Err(ValidationError::LimitTooLarge(self.limit));
        }
        
        if let Some(status) = &self.status_filter {
            status.parse::<PayStatus>()
                .map_err(|_| ValidationError::InvalidStatus(status.clone()))?;
        }
        
        Ok(())
    }
}

impl PayStatsReq {
    pub fn validate(&self) -> Result<(), ValidationError> {
        if self.end_date <= self.start_date {
            return Err(ValidationError::InvalidDateRange);
        }
        
        let max_range = chrono::Duration::days(365);
        if self.end_date - self.start_date > max_range {
            return Err(ValidationError::DateRangeTooLarge);
        }
        
        Ok(())
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ValidationError {
    #[error("Invalid amount: must be positive")]
    InvalidAmount,
    
    #[error("Amount too small: minimum is {0}")]
    AmountTooSmall(Decimal),
    
    #[error("Unsupported currency: {0}")]
    UnsupportedCurrency(String),
    
    #[error("Invalid status: {0}")]
    InvalidStatus(String),
    
    #[error("Limit too large: {0} (max 1000)")]
    LimitTooLarge(u32),
    
    #[error("Invalid date range: end must be after start")]
    InvalidDateRange,
    
    #[error("Date range too large: maximum 1 year")]
    DateRangeTooLarge,
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;
    
    #[test]
    fn should_create_payment_dto_from_entity() {
        let payment = Payment::new(
            UserId::generate(),
            dec!(50.0),
            Currency::USDT
        );
        
        let dto = PaymentDto::from_entity(&payment);
        
        assert_eq!(dto.amt, dec!(50.0));
        assert_eq!(dto.curr, "USDT");
        assert_eq!(dto.stat, "pending");
    }
    
    #[test]
    fn should_validate_create_payment_request() {
        let valid_req = CreatePayReq {
            usr_id: UserId::generate(),
            amt: dec!(50.0),
            curr: "USDT".to_string(),
        };
        
        assert!(valid_req.validate().is_ok());
        
        let invalid_req = CreatePayReq {
            usr_id: UserId::generate(),
            amt: dec!(5.0), // Too small for USDT
            curr: "USDT".to_string(),
        };
        
        assert!(invalid_req.validate().is_err());
    }
    
    #[test]
    fn should_validate_payment_stats_request() {
        let now = Utc::now();
        let valid_req = PayStatsReq {
            start_date: now - chrono::Duration::days(30),
            end_date: now,
        };
        
        assert!(valid_req.validate().is_ok());
        
        let invalid_req = PayStatsReq {
            start_date: now,
            end_date: now - chrono::Duration::days(30), // End before start
        };
        
        assert!(invalid_req.validate().is_err());
    }
}