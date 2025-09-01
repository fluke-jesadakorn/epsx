// Payment domain entity with minimal naming
use chrono::{DateTime, Utc};

use rust_decimal::Decimal;
use serde::{Serialize, Deserialize};

use crate::dom::values::{PayId, UserId, Currency, PayStatus};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Payment {
    id: PayId,
    uid: UserId,
    amt: Decimal,
    curr: Currency,
    stat: PayStatus,
    addr: Option<String>, // Payment address for crypto
    tx_hash: Option<String>, // Transaction hash
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl Payment {
    pub fn new(uid: UserId, amt: Decimal, curr: Currency) -> Self {
        let now = Utc::now();
        
        Self {
            id: PayId::generate(),
            uid,
            amt,
            curr,
            stat: PayStatus::Pending,
            addr: None,
            tx_hash: None,
            created_at: now,
            updated_at: now,
        }
    }
    
    pub fn reconstruct(
        id: PayId,
        uid: UserId,
        amt: Decimal,
        curr: Currency,
        stat: PayStatus,
        addr: Option<String>,
        tx_hash: Option<String>,
        created_at: DateTime<Utc>,
        updated_at: DateTime<Utc>,
    ) -> Self {
        Self {
            id,
            uid,
            amt,
            curr,
            stat,
            addr,
            tx_hash,
            created_at,
            updated_at,
        }
    }
    
    // Getters
    pub fn id(&self) -> &PayId { &self.id }
    pub fn uid(&self) -> &UserId { &self.uid }
    pub fn amt(&self) -> Decimal { self.amt }
    pub fn curr(&self) -> &Currency { &self.curr }
    pub fn stat(&self) -> &PayStatus { &self.stat }
    pub fn addr(&self) -> Option<&str> { self.addr.as_deref() }
    pub fn tx_hash(&self) -> Option<&str> { self.tx_hash.as_deref() }
    pub fn created_at(&self) -> DateTime<Utc> { self.created_at }
    pub fn updated_at(&self) -> DateTime<Utc> { self.updated_at }

    // Compatibility aliases for mapper layer
    pub fn user_id(&self) -> &UserId { &self.uid }
    pub fn amount(&self) -> Decimal { self.amt }
    pub fn currency(&self) -> &Currency { &self.curr }
    pub fn status(&self) -> &PayStatus { &self.stat }
    pub fn payment_id(&self) -> &PayId { &self.id }
    pub fn payment_method(&self) -> Option<&str> { 
        // For now, return None - this field doesn't exist in current entity
        // TODO: Add payment_method field if needed
        None 
    }
    pub fn transaction_id(&self) -> Option<&str> { self.tx_hash.as_deref() }
    pub fn description(&self) -> Option<&str> { 
        // For now, return None - this field doesn't exist in current entity
        // TODO: Add description field if needed
        None 
    }
    
    // Business methods
    pub fn set_addr(&mut self, addr: String) {
        self.addr = Some(addr);
        self.updated_at = Utc::now();
    }
    
    pub fn confirm(&mut self, tx_hash: String) -> Result<(), PaymentError> {
        match self.stat {
            PayStatus::Pending => {
                self.stat = PayStatus::Confirmed;
                self.tx_hash = Some(tx_hash);
                self.updated_at = Utc::now();
                Ok(())
            },
            _ => Err(PaymentError::InvalidStatusTransition {
                from: self.stat.clone(),
                to: PayStatus::Confirmed,
            }),
        }
    }
    
    pub fn complete(&mut self) -> Result<(), PaymentError> {
        match self.stat {
            PayStatus::Confirmed => {
                self.stat = PayStatus::Completed;
                self.updated_at = Utc::now();
                Ok(())
            },
            _ => Err(PaymentError::InvalidStatusTransition {
                from: self.stat.clone(),
                to: PayStatus::Completed,
            }),
        }
    }
    
    pub fn fail(&mut self, _reason: String) {
        self.stat = PayStatus::Failed;
        self.updated_at = Utc::now();
        // Could store reason in additional field
    }
    
    pub fn is_completed(&self) -> bool {
        matches!(self.stat, PayStatus::Completed)
    }
    
    pub fn is_pending(&self) -> bool {
        matches!(self.stat, PayStatus::Pending)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum PaymentError {
    #[error("Invalid status transition from {from:?} to {to:?}")]
    InvalidStatusTransition { from: PayStatus, to: PayStatus },
    
    #[error("Payment amount must be positive")]
    InvalidAmount,
    
    #[error("Unsupported currency: {0}")]
    UnsupportedCurrency(String),
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dom::values::UserId;
    use rust_decimal_macros::dec;
    
    #[test]
    fn should_create_payment() {
        let uid = UserId::generate();
        let amt = dec!(50.0);
        let curr = Currency::USDT;
        
        let payment = Payment::new(uid.clone(), amt, curr.clone());
        
        assert_eq!(payment.uid(), &uid);
        assert_eq!(payment.amt(), amt);
        assert_eq!(payment.curr(), &curr);
        assert_eq!(payment.stat(), &PayStatus::Pending);
    }
    
    #[test]
    fn should_confirm_payment() {
        let uid = UserId::generate();
        let mut payment = Payment::new(uid, dec!(50.0), Currency::USDT);
        
        let result = payment.confirm("0x123...".to_string());
        
        assert!(result.is_ok());
        assert_eq!(payment.stat(), &PayStatus::Confirmed);
        assert_eq!(payment.tx_hash(), Some("0x123..."));
    }
    
    #[test]
    fn should_complete_payment() {
        let uid = UserId::generate();
        let mut payment = Payment::new(uid, dec!(50.0), Currency::USDT);
        
        payment.confirm("0x123...".to_string()).unwrap();
        let result = payment.complete();
        
        assert!(result.is_ok());
        assert_eq!(payment.stat(), &PayStatus::Completed);
        assert!(payment.is_completed());
    }
    
    #[test]
    fn should_reject_invalid_transition() {
        let uid = UserId::generate();
        let mut payment = Payment::new(uid, dec!(50.0), Currency::USDT);
        
        let result = payment.complete(); // Skip confirmation
        
        assert!(result.is_err());
    }
}