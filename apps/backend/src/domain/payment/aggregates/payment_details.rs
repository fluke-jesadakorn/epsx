use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use crate::domain::payment::value_objects::{CryptoAddress, TransactionHash, Currency};

/// Crypto payment specific details
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CryptoPaymentDetails {
    pub currency: Currency,
    pub network: crate::domain::shared_kernel::value_objects::payments::Network,
    pub payment_address: Option<CryptoAddress>,
    pub transaction_hash: Option<TransactionHash>,
    pub confirmations: u32,
    /// Blockchain verification status
    pub verification_status: BlockchainVerificationStatus,
    /// Block number where transaction was mined
    pub block_number: Option<u64>,
    /// Block timestamp when transaction was mined
    pub block_timestamp: Option<DateTime<Utc>>,
    /// Amount verified on blockchain (in token units)
    pub verified_amount: Option<Decimal>,
    /// Recipient address verified on blockchain
    pub verified_recipient: Option<String>,
    /// Token contract address used for verification
    pub token_contract: Option<String>,
    /// Verification failure reason if any
    pub verification_error: Option<String>,
    /// Last verification attempt timestamp
    pub last_verification_attempt: Option<DateTime<Utc>>,
}

/// Blockchain verification status for crypto payments
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum BlockchainVerificationStatus {
    /// Not yet verified
    NotVerified,
    /// Verification in progress
    InProgress,
    /// Successfully verified on blockchain
    Verified,
    /// Verification failed
    Failed,
    /// Pending insufficient confirmations
    PendingConfirmations,
}

impl CryptoPaymentDetails {
    pub fn new(currency: Currency, network: crate::domain::shared_kernel::value_objects::payments::Network) -> Self {
        Self {
            currency,
            network,
            payment_address: None,
            transaction_hash: None,
            confirmations: 0,
            verification_status: BlockchainVerificationStatus::NotVerified,
            block_number: None,
            block_timestamp: None,
            verified_amount: None,
            verified_recipient: None,
            token_contract: None,
            verification_error: None,
            last_verification_attempt: None,
        }
    }
    
    /// Mark verification as in progress
    pub fn start_verification(&mut self) {
        self.verification_status = BlockchainVerificationStatus::InProgress;
        self.last_verification_attempt = Some(Utc::now());
        self.verification_error = None;
    }
    
    /// Mark verification as successful with blockchain data
    pub fn mark_verified(
        &mut self,
        block_number: u64,
        block_timestamp: DateTime<Utc>,
        verified_amount: Decimal,
        verified_recipient: String,
        token_contract: String,
    ) {
        self.verification_status = BlockchainVerificationStatus::Verified;
        self.block_number = Some(block_number);
        self.block_timestamp = Some(block_timestamp);
        self.verified_amount = Some(verified_amount);
        self.verified_recipient = Some(verified_recipient);
        self.token_contract = Some(token_contract);
        self.verification_error = None;
    }
    
    /// Mark verification as failed with error reason
    pub fn mark_verification_failed(&mut self, error: String) {
        self.verification_status = BlockchainVerificationStatus::Failed;
        self.verification_error = Some(error);
    }
    
    /// Mark as pending confirmations
    pub fn mark_pending_confirmations(&mut self) {
        self.verification_status = BlockchainVerificationStatus::PendingConfirmations;
    }
    
    /// Check if verification is complete (either verified or failed)
    pub fn is_verification_complete(&self) -> bool {
        matches!(
            self.verification_status,
            BlockchainVerificationStatus::Verified | BlockchainVerificationStatus::Failed
        )
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