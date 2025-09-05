use serde::{Deserialize, Serialize};
use std::fmt::{self, Display};
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use uuid::Uuid;

use crate::domain::shared_kernel::{AggregateRoot, DomainEvent, aggregate_root::AggregateBase};
use crate::domain::payment::value_objects::{
    PaymentId, PaymentAmount, PaymentMethod, CryptoAddress, TransactionHash,
    PaymentReference, Currency
};
use crate::dom::values::UserId; // Re-use existing user ID

/// Payment Aggregate Root
/// Manages the complete payment lifecycle from creation to completion
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Payment {
    id: PaymentId,
    user_id: UserId,
    reference: PaymentReference,
    amount: PaymentAmount,
    method: PaymentMethod,
    status: PaymentStatus,
    metadata: PaymentMetadata,
    crypto_details: Option<CryptoPaymentDetails>,
    fiat_details: Option<FiatPaymentDetails>,
    base: AggregateBase,
}

impl Payment {
    /// Create new payment
    pub fn create(
        user_id: UserId,
        amount: PaymentAmount,
        method: PaymentMethod,
    ) -> Result<Self, PaymentError> {
        // Validate amount is within method limits
        if !method.is_amount_valid(amount.amount()) {
            return Err(PaymentError::AmountOutOfRange {
                amount: amount.amount(),
                min: method.minimum_amount(),
                max: method.maximum_amount(),
            });
        }

        // Validate method is available
        if !method.is_available() {
            return Err(PaymentError::PaymentMethodUnavailable);
        }

        let payment_id = PaymentId::generate();
        let reference = PaymentReference::generate();
        let now = Utc::now();

        let mut payment = Self {
            id: payment_id.clone(),
            user_id: user_id.clone(),
            reference: reference.clone(),
            amount: amount.clone(),
            method: method.clone(),
            status: PaymentStatus::Created,
            metadata: PaymentMetadata::new(now),
            crypto_details: None,
            fiat_details: None,
            base: AggregateBase::new(),
        };

        // Initialize method-specific details
        payment.initialize_payment_details()?;

        // Record domain event
        payment.base.add_event(Box::new(PaymentCreated {
            payment_id: payment_id.clone(),
            user_id,
            amount: amount.clone(),
            method: method.clone(),
            reference,
            timestamp: now,
        }));

        Ok(payment)
    }

    /// Reconstruct payment from persistence
    pub fn reconstruct(
        id: PaymentId,
        user_id: UserId,
        reference: PaymentReference,
        amount: PaymentAmount,
        method: PaymentMethod,
        status: PaymentStatus,
        metadata: PaymentMetadata,
        crypto_details: Option<CryptoPaymentDetails>,
        fiat_details: Option<FiatPaymentDetails>,
        version: u64,
    ) -> Self {
        Self {
            id,
            user_id,
            reference,
            amount,
            method,
            status,
            metadata,
            crypto_details,
            fiat_details,
            base: {
                let mut base = AggregateBase::new();
                base.version = version;
                base
            },
        }
    }

    // Getters
    pub fn id(&self) -> &PaymentId { &self.id }
    pub fn user_id(&self) -> &UserId { &self.user_id }
    pub fn reference(&self) -> &PaymentReference { &self.reference }
    pub fn amount(&self) -> &PaymentAmount { &self.amount }
    pub fn method(&self) -> &PaymentMethod { &self.method }
    pub fn status(&self) -> &PaymentStatus { &self.status }
    pub fn metadata(&self) -> &PaymentMetadata { &self.metadata }
    pub fn crypto_details(&self) -> Option<&CryptoPaymentDetails> { self.crypto_details.as_ref() }
    pub fn fiat_details(&self) -> Option<&FiatPaymentDetails> { self.fiat_details.as_ref() }
    
    // Timestamp accessors
    pub fn created_at(&self) -> DateTime<Utc> { self.metadata.created_at }
    pub fn updated_at(&self) -> DateTime<Utc> { self.metadata.updated_at }
    pub fn expires_at(&self) -> Option<DateTime<Utc>> { self.metadata.expires_at }
    
    /// Cancel payment (alias for cancel_payment)
    pub fn cancel(&mut self, reason: String) -> Result<(), PaymentError> {
        self.cancel_payment(reason)
    }

    /// Assign crypto address for payment
    pub fn assign_crypto_address(&mut self, address: CryptoAddress) -> Result<(), PaymentError> {
        if !matches!(self.status, PaymentStatus::Created | PaymentStatus::AwaitingPayment) {
            return Err(PaymentError::InvalidStatusTransition {
                from: self.status.clone(),
                to: PaymentStatus::AwaitingPayment,
                reason: "Can only assign address to created payments".to_string(),
            });
        }

        if self.crypto_details.is_none() {
            return Err(PaymentError::NotCryptoPayment);
        }

        if let Some(details) = &mut self.crypto_details {
            details.payment_address = Some(address.clone());
            self.status = PaymentStatus::AwaitingPayment;
            self.metadata.mark_awaiting_payment();

            self.base.add_event(Box::new(PaymentAddressAssigned {
                payment_id: self.id.clone(),
                address: address,
                timestamp: Utc::now(),
            }));
        }

        Ok(())
    }

    /// Confirm payment with transaction hash
    pub fn confirm_payment(&mut self, tx_hash: TransactionHash) -> Result<(), PaymentError> {
        if !matches!(self.status, PaymentStatus::AwaitingPayment) {
            return Err(PaymentError::InvalidStatusTransition {
                from: self.status.clone(),
                to: PaymentStatus::Confirmed,
                reason: "Can only confirm payments that are awaiting payment".to_string(),
            });
        }

        if let Some(details) = &mut self.crypto_details {
            details.transaction_hash = Some(tx_hash.clone());
            self.status = PaymentStatus::Confirmed;
            self.metadata.mark_confirmed();

            self.base.add_event(Box::new(PaymentConfirmed {
                payment_id: self.id.clone(),
                transaction_hash: tx_hash,
                timestamp: Utc::now(),
            }));
        } else {
            return Err(PaymentError::NotCryptoPayment);
        }

        Ok(())
    }

    /// Complete the payment
    pub fn complete_payment(&mut self) -> Result<(), PaymentError> {
        if !matches!(self.status, PaymentStatus::Confirmed | PaymentStatus::Processing) {
            return Err(PaymentError::InvalidStatusTransition {
                from: self.status.clone(),
                to: PaymentStatus::Completed,
                reason: "Can only complete confirmed or processing payments".to_string(),
            });
        }

        let now = Utc::now();
        self.status = PaymentStatus::Completed;
        self.metadata.mark_completed(now);

        // Calculate actual fees
        let processing_fee = self.calculate_processing_fee();
        let net_amount = self.amount.amount_after_fees();

        self.base.add_event(Box::new(PaymentCompleted {
            payment_id: self.id.clone(),
            gross_amount: self.amount.clone(),
            processing_fee,
            net_amount,
            timestamp: now,
        }));

        Ok(())
    }

    /// Fail the payment
    pub fn fail_payment(&mut self, reason: String) -> Result<(), PaymentError> {
        if matches!(self.status, PaymentStatus::Completed | PaymentStatus::Refunded) {
            return Err(PaymentError::InvalidStatusTransition {
                from: self.status.clone(),
                to: PaymentStatus::Failed,
                reason: "Cannot fail completed or refunded payments".to_string(),
            });
        }

        let now = Utc::now();
        self.status = PaymentStatus::Failed;
        self.metadata.mark_failed(now, reason.clone());

        self.base.add_event(Box::new(PaymentFailed {
            payment_id: self.id.clone(),
            reason,
            timestamp: now,
        }));

        Ok(())
    }

    /// Cancel the payment
    pub fn cancel_payment(&mut self, reason: String) -> Result<(), PaymentError> {
        if !matches!(self.status, PaymentStatus::Created | PaymentStatus::AwaitingPayment) {
            return Err(PaymentError::InvalidStatusTransition {
                from: self.status.clone(),
                to: PaymentStatus::Cancelled,
                reason: "Can only cancel created or awaiting payments".to_string(),
            });
        }

        let now = Utc::now();
        self.status = PaymentStatus::Cancelled;
        self.metadata.mark_cancelled(now, reason.clone());

        self.base.add_event(Box::new(PaymentCancelled {
            payment_id: self.id.clone(),
            reason,
            timestamp: now,
        }));

        Ok(())
    }

    /// Start refund process
    pub fn initiate_refund(&mut self, reason: String) -> Result<(), PaymentError> {
        if !matches!(self.status, PaymentStatus::Completed) {
            return Err(PaymentError::InvalidStatusTransition {
                from: self.status.clone(),
                to: PaymentStatus::Refunding,
                reason: "Can only refund completed payments".to_string(),
            });
        }

        let now = Utc::now();
        self.status = PaymentStatus::Refunding;

        self.base.add_event(Box::new(PaymentRefundInitiated {
            payment_id: self.id.clone(),
            refund_amount: self.amount.clone(),
            reason,
            timestamp: now,
        }));

        Ok(())
    }

    /// Complete refund
    pub fn complete_refund(&mut self, refund_tx_hash: Option<TransactionHash>) -> Result<(), PaymentError> {
        if !matches!(self.status, PaymentStatus::Refunding) {
            return Err(PaymentError::InvalidStatusTransition {
                from: self.status.clone(),
                to: PaymentStatus::Refunded,
                reason: "Can only complete refund for refunding payments".to_string(),
            });
        }

        let now = Utc::now();
        self.status = PaymentStatus::Refunded;
        self.metadata.mark_refunded(now);

        self.base.add_event(Box::new(PaymentRefundCompleted {
            payment_id: self.id.clone(),
            refund_transaction_hash: refund_tx_hash,
            timestamp: now,
        }));

        Ok(())
    }

    /// Check if payment is in final state
    pub fn is_final(&self) -> bool {
        matches!(
            self.status,
            PaymentStatus::Completed | PaymentStatus::Failed | 
            PaymentStatus::Cancelled | PaymentStatus::Refunded
        )
    }

    /// Check if payment is successful
    pub fn is_successful(&self) -> bool {
        matches!(self.status, PaymentStatus::Completed)
    }

    /// Get age of payment
    pub fn age(&self) -> chrono::Duration {
        Utc::now() - self.metadata.created_at
    }

    /// Check if payment has expired
    pub fn is_expired(&self) -> bool {
        if let Some(expires_at) = self.metadata.expires_at {
            Utc::now() > expires_at
        } else {
            false
        }
    }

    /// Calculate processing fee
    pub fn calculate_processing_fee(&self) -> PaymentAmount {
        self.amount.processing_fee()
    }

    /// Get net amount after fees
    pub fn net_amount(&self) -> PaymentAmount {
        self.amount.amount_after_fees()
    }

    /// Initialize payment-specific details based on method
    fn initialize_payment_details(&mut self) -> Result<(), PaymentError> {
        use crate::domain::payment::value_objects::PaymentMethodType;

        match self.method.method_type() {
            PaymentMethodType::Crypto => {
                self.crypto_details = Some(CryptoPaymentDetails::new(
                    self.method.currency().clone(),
                    self.method.network().unwrap().clone(),
                ));
            }
            PaymentMethodType::BankTransfer | PaymentMethodType::CreditCard => {
                self.fiat_details = Some(FiatPaymentDetails::new(
                    self.method.method_type().clone(),
                ));
            }
        }

        Ok(())
    }
}

impl AggregateRoot for Payment {
    type Id = PaymentId;

    fn id(&self) -> &Self::Id {
        &self.id
    }

    fn version(&self) -> u64 {
        self.base.version
    }

    fn increment_version(&mut self) {
        self.base.increment_version();
    }

    fn uncommitted_events(&self) -> &[Box<dyn DomainEvent>] {
        &self.base.events
    }

    fn mark_events_as_committed(&mut self) {
        self.base.clear_events();
    }

    fn created_at(&self) -> DateTime<Utc> {
        self.base.created_at
    }

    fn updated_at(&self) -> DateTime<Utc> {
        self.base.updated_at
    }

    fn touch(&mut self) {
        self.base.touch();
    }
}

/// Payment status enum
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PaymentStatus {
    /// Payment has been created but not yet initiated
    Created,
    /// Awaiting payment from user (crypto address assigned)
    AwaitingPayment,
    /// Payment has been received and confirmed on blockchain
    Confirmed,
    /// Payment is being processed
    Processing,
    /// Payment completed successfully
    Completed,
    /// Payment failed
    Failed,
    /// Payment was cancelled before completion
    Cancelled,
    /// Refund is being processed
    Refunding,
    /// Payment has been refunded
    Refunded,
}

impl std::str::FromStr for PaymentStatus {
    type Err = PaymentError;
    
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "created" => Ok(PaymentStatus::Created),
            "awaiting_payment" | "awaiting" => Ok(PaymentStatus::AwaitingPayment),
            "confirmed" => Ok(PaymentStatus::Confirmed),
            "processing" => Ok(PaymentStatus::Processing),
            "completed" => Ok(PaymentStatus::Completed),
            "failed" => Ok(PaymentStatus::Failed),
            "cancelled" | "canceled" => Ok(PaymentStatus::Cancelled),
            "refunding" => Ok(PaymentStatus::Refunding),
            "refunded" => Ok(PaymentStatus::Refunded),
            _ => Err(PaymentError::InvalidStatusTransition {
                from: PaymentStatus::Created,
                to: PaymentStatus::Created,
                reason: format!("Unknown payment status: {}", s),
            }),
        }
    }
}

impl PaymentStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            PaymentStatus::Created => "created",
            PaymentStatus::AwaitingPayment => "awaiting_payment",
            PaymentStatus::Confirmed => "confirmed",
            PaymentStatus::Processing => "processing",
            PaymentStatus::Completed => "completed",
            PaymentStatus::Failed => "failed",
            PaymentStatus::Cancelled => "cancelled",
            PaymentStatus::Refunding => "refunding",
            PaymentStatus::Refunded => "refunded",
        }
    }

    pub fn is_final(&self) -> bool {
        matches!(
            self,
            PaymentStatus::Completed | PaymentStatus::Failed | 
            PaymentStatus::Cancelled | PaymentStatus::Refunded
        )
    }
}

impl PartialEq<PaymentStatus> for &PaymentStatus {
    fn eq(&self, other: &PaymentStatus) -> bool {
        **self == *other
    }
}

impl Display for PaymentStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Payment metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaymentMetadata {
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub confirmed_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub expires_at: Option<DateTime<Utc>>,
    pub failure_reason: Option<String>,
    pub cancellation_reason: Option<String>,
}

impl PaymentMetadata {
    pub fn new(created_at: DateTime<Utc>) -> Self {
        // Set expiry to 30 minutes from creation
        let expires_at = created_at + chrono::Duration::minutes(30);
        
        Self {
            created_at,
            updated_at: created_at,
            confirmed_at: None,
            completed_at: None,
            expires_at: Some(expires_at),
            failure_reason: None,
            cancellation_reason: None,
        }
    }

    pub fn mark_awaiting_payment(&mut self) {
        self.updated_at = Utc::now();
    }

    pub fn mark_confirmed(&mut self) {
        let now = Utc::now();
        self.confirmed_at = Some(now);
        self.updated_at = now;
    }

    pub fn mark_completed(&mut self, completed_at: DateTime<Utc>) {
        self.completed_at = Some(completed_at);
        self.updated_at = completed_at;
    }

    pub fn mark_failed(&mut self, failed_at: DateTime<Utc>, reason: String) {
        self.failure_reason = Some(reason);
        self.updated_at = failed_at;
    }

    pub fn mark_cancelled(&mut self, cancelled_at: DateTime<Utc>, reason: String) {
        self.cancellation_reason = Some(reason);
        self.updated_at = cancelled_at;
    }

    pub fn mark_refunded(&mut self, refunded_at: DateTime<Utc>) {
        self.updated_at = refunded_at;
    }
}

/// Crypto payment specific details
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CryptoPaymentDetails {
    pub currency: Currency,
    pub network: crate::dom::values::payments::Network,
    pub payment_address: Option<CryptoAddress>,
    pub transaction_hash: Option<TransactionHash>,
    pub confirmations: u32,
}

impl CryptoPaymentDetails {
    pub fn new(currency: Currency, network: crate::dom::values::payments::Network) -> Self {
        Self {
            currency,
            network,
            payment_address: None,
            transaction_hash: None,
            confirmations: 0,
        }
    }

    pub fn update_confirmations(&mut self, confirmations: u32) {
        self.confirmations = confirmations;
    }
}

/// Fiat payment specific details
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FiatPaymentDetails {
    pub method_type: crate::domain::payment::value_objects::PaymentMethodType,
    pub processor_reference: Option<String>,
    pub processor_fee: Option<Decimal>,
}

impl FiatPaymentDetails {
    pub fn new(method_type: crate::domain::payment::value_objects::PaymentMethodType) -> Self {
        Self {
            method_type,
            processor_reference: None,
            processor_fee: None,
        }
    }

    pub fn set_processor_details(&mut self, reference: String, fee: Decimal) {
        self.processor_reference = Some(reference);
        self.processor_fee = Some(fee);
    }
}

// Domain Events

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaymentCreated {
    pub payment_id: PaymentId,
    pub user_id: UserId,
    pub amount: PaymentAmount,
    pub method: PaymentMethod,
    pub reference: PaymentReference,
    pub timestamp: DateTime<Utc>,
}

impl DomainEvent for PaymentCreated {
    fn event_id(&self) -> Uuid {
        Uuid::new_v4()
    }

    fn occurred_at(&self) -> DateTime<Utc> {
        self.timestamp
    }

    fn aggregate_version(&self) -> u64 {
        1
    }

    fn event_type(&self) -> &'static str {
        "payment.created"
    }

    fn aggregate_id(&self) -> String {
        self.payment_id.to_string()
    }


    fn to_json(&self) -> Result<String, Box<dyn std::error::Error>> {
        Ok(serde_json::to_string(self)?)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaymentAddressAssigned {
    pub payment_id: PaymentId,
    pub address: CryptoAddress,
    pub timestamp: DateTime<Utc>,
}

impl DomainEvent for PaymentAddressAssigned {
    fn event_id(&self) -> Uuid {
        Uuid::new_v4()
    }

    fn occurred_at(&self) -> DateTime<Utc> {
        self.timestamp
    }

    fn aggregate_version(&self) -> u64 {
        1
    }


    fn event_type(&self) -> &'static str {
        "payment.address_assigned"
    }

    fn aggregate_id(&self) -> String {
        self.payment_id.to_string()
    }


    fn to_json(&self) -> Result<String, Box<dyn std::error::Error>> {
        Ok(serde_json::to_string(self)?)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaymentConfirmed {
    pub payment_id: PaymentId,
    pub transaction_hash: TransactionHash,
    pub timestamp: DateTime<Utc>,
}

impl DomainEvent for PaymentConfirmed {
    fn event_id(&self) -> Uuid {
        Uuid::new_v4()
    }

    fn occurred_at(&self) -> DateTime<Utc> {
        self.timestamp
    }

    fn aggregate_version(&self) -> u64 {
        1
    }


    fn event_type(&self) -> &'static str {
        "payment.confirmed"
    }

    fn aggregate_id(&self) -> String {
        self.payment_id.to_string()
    }


    fn to_json(&self) -> Result<String, Box<dyn std::error::Error>> {
        Ok(serde_json::to_string(self)?)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaymentCompleted {
    pub payment_id: PaymentId,
    pub gross_amount: PaymentAmount,
    pub processing_fee: PaymentAmount,
    pub net_amount: PaymentAmount,
    pub timestamp: DateTime<Utc>,
}

impl DomainEvent for PaymentCompleted {
    fn event_id(&self) -> Uuid {
        Uuid::new_v4()
    }

    fn occurred_at(&self) -> DateTime<Utc> {
        self.timestamp
    }

    fn aggregate_version(&self) -> u64 {
        1
    }


    fn event_type(&self) -> &'static str {
        "payment.completed"
    }

    fn aggregate_id(&self) -> String {
        self.payment_id.to_string()
    }


    fn to_json(&self) -> Result<String, Box<dyn std::error::Error>> {
        Ok(serde_json::to_string(self)?)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaymentFailed {
    pub payment_id: PaymentId,
    pub reason: String,
    pub timestamp: DateTime<Utc>,
}

impl DomainEvent for PaymentFailed {
    fn event_id(&self) -> Uuid {
        Uuid::new_v4()
    }

    fn occurred_at(&self) -> DateTime<Utc> {
        self.timestamp
    }

    fn aggregate_version(&self) -> u64 {
        1
    }


    fn event_type(&self) -> &'static str {
        "payment.failed"
    }

    fn aggregate_id(&self) -> String {
        self.payment_id.to_string()
    }


    fn to_json(&self) -> Result<String, Box<dyn std::error::Error>> {
        Ok(serde_json::to_string(self)?)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaymentCancelled {
    pub payment_id: PaymentId,
    pub reason: String,
    pub timestamp: DateTime<Utc>,
}

impl DomainEvent for PaymentCancelled {
    fn event_id(&self) -> Uuid {
        Uuid::new_v4()
    }

    fn occurred_at(&self) -> DateTime<Utc> {
        self.timestamp
    }

    fn aggregate_version(&self) -> u64 {
        1
    }


    fn event_type(&self) -> &'static str {
        "payment.cancelled"
    }

    fn aggregate_id(&self) -> String {
        self.payment_id.to_string()
    }


    fn to_json(&self) -> Result<String, Box<dyn std::error::Error>> {
        Ok(serde_json::to_string(self)?)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaymentRefundInitiated {
    pub payment_id: PaymentId,
    pub refund_amount: PaymentAmount,
    pub reason: String,
    pub timestamp: DateTime<Utc>,
}

impl DomainEvent for PaymentRefundInitiated {
    fn event_id(&self) -> Uuid {
        Uuid::new_v4()
    }

    fn occurred_at(&self) -> DateTime<Utc> {
        self.timestamp
    }

    fn aggregate_version(&self) -> u64 {
        1
    }


    fn event_type(&self) -> &'static str {
        "payment.refund_initiated"
    }

    fn aggregate_id(&self) -> String {
        self.payment_id.to_string()
    }


    fn to_json(&self) -> Result<String, Box<dyn std::error::Error>> {
        Ok(serde_json::to_string(self)?)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaymentRefundCompleted {
    pub payment_id: PaymentId,
    pub refund_transaction_hash: Option<TransactionHash>,
    pub timestamp: DateTime<Utc>,
}

impl DomainEvent for PaymentRefundCompleted {
    fn event_id(&self) -> Uuid {
        Uuid::new_v4()
    }

    fn occurred_at(&self) -> DateTime<Utc> {
        self.timestamp
    }

    fn aggregate_version(&self) -> u64 {
        1
    }


    fn event_type(&self) -> &'static str {
        "payment.refund_completed"
    }

    fn aggregate_id(&self) -> String {
        self.payment_id.to_string()
    }


    fn to_json(&self) -> Result<String, Box<dyn std::error::Error>> {
        Ok(serde_json::to_string(self)?)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum PaymentError {
    #[error("Amount {amount} is outside valid range [{min}, {max}]")]
    AmountOutOfRange {
        amount: Decimal,
        min: Decimal,
        max: Decimal,
    },

    #[error("Payment method is not available")]
    PaymentMethodUnavailable,

    #[error("Invalid status transition from {from} to {to}: {reason}")]
    InvalidStatusTransition {
        from: PaymentStatus,
        to: PaymentStatus,
        reason: String,
    },

    #[error("Operation not valid for crypto payment")]
    NotCryptoPayment,

    #[error("Operation not valid for fiat payment")]
    NotFiatPayment,

    #[error("Payment has expired")]
    PaymentExpired,

    #[error("Address already assigned")]
    AddressAlreadyAssigned,

    #[error("Transaction hash already set")]
    TransactionHashAlreadySet,

    #[error("Refund not allowed for this payment")]
    RefundNotAllowed,

    #[error("Insufficient confirmations: {current}, required: {required}")]
    InsufficientConfirmations { current: u32, required: u32 },
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::payment::value_objects::{PaymentMethodType, Network};
    use rust_decimal_macros::dec;

    #[test]
    fn test_payment_creation() {
        let user_id = UserId::generate();
        let amount = PaymentAmount::new(dec!(50.0), Currency::USDT).unwrap();
        let method = PaymentMethod::new(
            PaymentMethodType::Crypto,
            Currency::USDT,
            Some(Network::Ethereum),
        ).unwrap();

        let payment = Payment::create(user_id.clone(), amount, method).unwrap();

        assert_eq!(payment.user_id(), &user_id);
        assert_eq!(*payment.status(), PaymentStatus::Created);
        assert!(payment.crypto_details().is_some());
        assert!(payment.fiat_details().is_none());
        assert!(!payment.is_final());
        assert!(!payment.is_successful());
    }

    #[test]
    fn test_payment_lifecycle_crypto() {
        let user_id = UserId::generate();
        let amount = PaymentAmount::new(dec!(100.0), Currency::ETH).unwrap();
        let method = PaymentMethod::new(
            PaymentMethodType::Crypto,
            Currency::ETH,
            Some(Network::Ethereum),
        ).unwrap();

        let mut payment = Payment::create(user_id, amount, method).unwrap();
        assert_eq!(*payment.status(), PaymentStatus::Created);

        // Assign crypto address
        let address = CryptoAddress::new(
            "0x742d35Cc3681d452bC9a4D0c99D2DB8b4E8B5f43".to_string(),
            Network::Ethereum,
            Currency::ETH,
        ).unwrap();

        payment.assign_crypto_address(address).unwrap();
        assert_eq!(*payment.status(), PaymentStatus::AwaitingPayment);

        // Confirm payment
        let tx_hash = TransactionHash::new(
            "0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef12".to_string(),
            Network::Ethereum,
        ).unwrap();

        payment.confirm_payment(tx_hash).unwrap();
        assert_eq!(*payment.status(), PaymentStatus::Confirmed);

        // Complete payment
        payment.complete_payment().unwrap();
        assert_eq!(*payment.status(), PaymentStatus::Completed);
        assert!(payment.is_final());
        assert!(payment.is_successful());
    }

    #[test]
    fn test_payment_failure() {
        let user_id = UserId::generate();
        let amount = PaymentAmount::new(dec!(50.0), Currency::USDT).unwrap();
        let method = PaymentMethod::new(
            PaymentMethodType::Crypto,
            Currency::USDT,
            Some(Network::Ethereum),
        ).unwrap();

        let mut payment = Payment::create(user_id, amount, method).unwrap();
        
        payment.fail_payment("Transaction failed".to_string()).unwrap();
        assert_eq!(*payment.status(), PaymentStatus::Failed);
        assert!(payment.is_final());
        assert!(!payment.is_successful());
        assert_eq!(payment.metadata().failure_reason, Some("Transaction failed".to_string()));
    }

    #[test]
    fn test_payment_cancellation() {
        let user_id = UserId::generate();
        let amount = PaymentAmount::new(dec!(50.0), Currency::USDT).unwrap();
        let method = PaymentMethod::new(
            PaymentMethodType::Crypto,
            Currency::USDT,
            Some(Network::Ethereum),
        ).unwrap();

        let mut payment = Payment::create(user_id, amount, method).unwrap();
        
        payment.cancel_payment("User requested".to_string()).unwrap();
        assert_eq!(*payment.status(), PaymentStatus::Cancelled);
        assert!(payment.is_final());
        assert_eq!(payment.metadata().cancellation_reason, Some("User requested".to_string()));
    }

    #[test]
    fn test_payment_refund() {
        let user_id = UserId::generate();
        let amount = PaymentAmount::new(dec!(50.0), Currency::USDT).unwrap();
        let method = PaymentMethod::new(
            PaymentMethodType::Crypto,
            Currency::USDT,
            Some(Network::Ethereum),
        ).unwrap();

        let mut payment = Payment::create(user_id, amount, method).unwrap();
        
        // Complete payment first
        payment.status = PaymentStatus::Completed;
        
        // Initiate refund
        payment.initiate_refund("Duplicate payment".to_string()).unwrap();
        assert_eq!(*payment.status(), PaymentStatus::Refunding);

        // Complete refund
        payment.complete_refund(None).unwrap();
        assert_eq!(*payment.status(), PaymentStatus::Refunded);
        assert!(payment.is_final());
    }

    #[test]
    fn test_invalid_status_transitions() {
        let user_id = UserId::generate();
        let amount = PaymentAmount::new(dec!(50.0), Currency::USDT).unwrap();
        let method = PaymentMethod::new(
            PaymentMethodType::Crypto,
            Currency::USDT,
            Some(Network::Ethereum),
        ).unwrap();

        let mut payment = Payment::create(user_id, amount, method).unwrap();
        
        // Try to complete without confirming
        let result = payment.complete_payment();
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), PaymentError::InvalidStatusTransition { .. }));
    }

    #[test]
    fn test_amount_validation() {
        let user_id = UserId::generate();
        let amount = PaymentAmount::new(dec!(1.0), Currency::USDT).unwrap(); // Below minimum
        let method = PaymentMethod::new(
            PaymentMethodType::Crypto,
            Currency::USDT,
            Some(Network::Ethereum),
        ).unwrap();

        let result = Payment::create(user_id, amount, method);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), PaymentError::AmountOutOfRange { .. }));
    }

    #[test]
    fn test_payment_expiry() {
        let user_id = UserId::generate();
        let amount = PaymentAmount::new(dec!(50.0), Currency::USDT).unwrap();
        let method = PaymentMethod::new(
            PaymentMethodType::Crypto,
            Currency::USDT,
            Some(Network::Ethereum),
        ).unwrap();

        let payment = Payment::create(user_id, amount, method).unwrap();
        
        // Payment should expire in 30 minutes
        assert!(payment.metadata().expires_at.is_some());
        assert!(!payment.is_expired()); // Should not be expired immediately
    }

    #[test]
    fn test_processing_fee_calculation() {
        let user_id = UserId::generate();
        let amount = PaymentAmount::new(dec!(100.0), Currency::USDT).unwrap();
        let method = PaymentMethod::new(
            PaymentMethodType::Crypto,
            Currency::USDT,
            Some(Network::Ethereum),
        ).unwrap();

        let payment = Payment::create(user_id, amount, method).unwrap();
        
        let fee = payment.calculate_processing_fee();
        assert_eq!(fee.amount(), dec!(2.0)); // 2% of 100 = 2

        let net = payment.net_amount();
        assert_eq!(net.amount(), dec!(98.0)); // 100 - 2 = 98
    }
}