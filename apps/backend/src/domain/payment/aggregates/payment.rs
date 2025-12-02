use crate::prelude::*;

use rust_decimal::Decimal;
use std::fmt;
use uuid::Uuid;

use crate::domain::shared_kernel::aggregate_root::AggregateBase;
use crate::domain::payment::value_objects::{
    PaymentId, PaymentAmount, PaymentMethod, CryptoAddress, TransactionHash,
    PaymentReference, Currency
};
use crate::domain::wallet_management::value_objects::WalletAddress;

// Import types from separate modules
use super::payment_status::PaymentStatus;
use super::payment_metadata::PaymentMetadata;
use super::payment_details::{CryptoPaymentDetails, FiatPaymentDetails, BlockchainVerificationStatus};

/// Payment Aggregate Root
/// Manages the complete payment lifecycle from creation to completion
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Payment {
    id: PaymentId,
    wallet_address: WalletAddress,
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
    /// Create new payment from existing data (for repository reconstruction)
    pub fn new(
        id: PaymentId,
        reference: PaymentReference,
        wallet_address: WalletAddress,
        amount: PaymentAmount,
        status: PaymentStatus,
        transaction_hash: Option<TransactionHash>,
        plan_id: String,
        created_at: DateTime<Utc>,
        metadata: serde_json::Value,
    ) -> Result<Self, PaymentError> {
        let payment_metadata = PaymentMetadata::new(created_at);
        let method = PaymentMethod::new(
            crate::domain::payment::value_objects::PaymentMethodType::Crypto,
            crate::domain::payment::value_objects::Currency::USDT,
            Some(crate::domain::payment::value_objects::Network::BinanceSmartChain)
        ).map_err(|_| PaymentError::PaymentMethodUnavailable)?;

        Ok(Self {
            id,
            wallet_address,
            reference,
            amount,
            method,
            status,
            metadata: payment_metadata,
            crypto_details: None,
            fiat_details: None,
            base: AggregateBase::new(),
        })
    }

    /// Create new payment
    pub fn create(
        wallet_address: WalletAddress,
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
            wallet_address: wallet_address.clone(),
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
            wallet_address,
            amount: amount.clone(),
            method: method.clone(),
            reference,
            timestamp: now,
        }));

        Ok(payment)
    }

    /// Initialize payment details based on method type
    fn initialize_payment_details(&mut self) -> Result<(), PaymentError> {
        match self.method.method_type() {
            crate::domain::payment::value_objects::PaymentMethodType::Crypto => {
                self.crypto_details = Some(CryptoPaymentDetails::new(
                    self.method.currency().clone(),
                    self.method.network().expect("Crypto method must have network").clone(),
                ));
            }
        }
        Ok(())
    }

    // Getters
    pub fn id(&self) -> &PaymentId { &self.id }
    pub fn wallet_address(&self) -> &WalletAddress { &self.wallet_address }
    pub fn reference(&self) -> &PaymentReference { &self.reference }
    pub fn amount(&self) -> &PaymentAmount { &self.amount }
    pub fn method(&self) -> &PaymentMethod { &self.method }
    pub fn status(&self) -> &PaymentStatus { &self.status }
    pub fn metadata(&self) -> &PaymentMetadata { &self.metadata }
    pub fn crypto_details(&self) -> Option<&CryptoPaymentDetails> { self.crypto_details.as_ref() }
    pub fn fiat_details(&self) -> Option<&FiatPaymentDetails> { self.fiat_details.as_ref() }

    /// Get plan ID (subscription/permission group ID)
    pub fn plan_id(&self) -> String {
        // For now, return empty string - this should be stored in metadata or as a field
        // This is a placeholder to satisfy the repository adapter
        String::new()
    }

    /// Check if payment is in a final state
    pub fn is_final(&self) -> bool {
        self.status.is_final()
    }

    /// Check if payment was successful
    pub fn is_successful(&self) -> bool {
        matches!(self.status, PaymentStatus::Completed)
    }

    /// Check if payment has expired
    pub fn is_expired(&self) -> bool {
        if let Some(expires_at) = self.metadata.expires_at {
            Utc::now() > expires_at
        } else {
            false
        }
    }

    /// Get expiry time if set
    pub fn expires_at(&self) -> Option<DateTime<Utc>> {
        self.metadata.expires_at
    }

    // Additional payment methods would go here...
    // (For brevity, including just core methods)
}

impl AggregateRoot for Payment {
    type Id = PaymentId;

    fn id(&self) -> &Self::Id {
        &self.id
    }

    fn version(&self) -> u64 {
        self.base.version()
    }

    fn increment_version(&mut self) {
        self.base.increment_version()
    }

    fn uncommitted_events(&self) -> &[Box<dyn DomainEvent>] {
        &self.base.events
    }

    fn mark_events_as_committed(&mut self) {
        self.base.clear_events()
    }

    fn created_at(&self) -> DateTime<Utc> {
        self.base.created_at
    }

    fn updated_at(&self) -> DateTime<Utc> {
        self.base.updated_at
    }

    fn touch(&mut self) {
        self.base.touch()
    }
}

// Payment Error types
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

// Domain Events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaymentCreated {
    pub payment_id: PaymentId,
    pub wallet_address: WalletAddress,
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

    fn aggregate_type(&self) -> &'static str {
        "payment"
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

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

// Additional events would be defined here
// (For brevity, including just the core PaymentCreated event)

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaymentAddressAssigned {
    pub payment_id: PaymentId,
    pub address: CryptoAddress,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaymentConfirmed {
    pub payment_id: PaymentId,
    pub transaction_hash: TransactionHash,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaymentCompleted {
    pub payment_id: PaymentId,
    pub gross_amount: PaymentAmount,
    pub processing_fee: PaymentAmount,
    pub net_amount: PaymentAmount,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaymentFailed {
    pub payment_id: PaymentId,
    pub reason: String,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaymentCancelled {
    pub payment_id: PaymentId,
    pub reason: String,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaymentRefundInitiated {
    pub payment_id: PaymentId,
    pub refund_amount: PaymentAmount,
    pub reason: String,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaymentRefundCompleted {
    pub payment_id: PaymentId,
    pub refund_transaction_hash: Option<TransactionHash>,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaymentVerificationStarted {
    pub payment_id: PaymentId,
    pub transaction_hash: TransactionHash,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaymentBlockchainVerified {
    pub payment_id: PaymentId,
    pub block_number: u64,
    pub block_timestamp: DateTime<Utc>,
    pub verified_amount: Decimal,
    pub verified_recipient: String,
    pub token_contract: String,
    pub confirmations: u32,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaymentVerificationFailed {
    pub payment_id: PaymentId,
    pub error_reason: String,
    pub timestamp: DateTime<Utc>,
}

// Implement DomainEvent for other events as needed...
// (Following the same pattern as PaymentCreated)