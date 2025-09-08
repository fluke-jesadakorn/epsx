use crate::domain::shared_kernel::value_objects::UserId;
use tracing::{info, error};
use std::sync::Arc;

// Payment Service Integration
// Orchestrates Payment bounded context with external payment systems
// Maintains API compatibility while using DDD internally

use crate::domain::payment::{
    PaymentId, PaymentAmount, PaymentMethod, PaymentStatus,
    repository_ports::{PaymentRepositoryPort, TransactionRepositoryPort, CryptoAddressRepositoryPort, PaymentMethodRepositoryPort},
    value_objects::{Currency, Network}
};
use crate::application::payment::commands::{CreatePaymentCommand, CreatePaymentCommandHandler};
use crate::application::shared::CommandHandler;

/// Integration service that orchestrates DDD payment components
pub struct PaymentServiceIntegration {
    // Payment bounded context repositories
    payment_repository: Arc<dyn PaymentRepositoryPort>,
    transaction_repository: Arc<dyn TransactionRepositoryPort>,
    crypto_address_repository: Arc<dyn CryptoAddressRepositoryPort>,
    payment_method_repository: Arc<dyn PaymentMethodRepositoryPort>,
    
    // Command handlers
    create_payment_handler: CreatePaymentCommandHandler,
}

impl PaymentServiceIntegration {
    pub fn new(
        payment_repository: Arc<dyn PaymentRepositoryPort>,
        transaction_repository: Arc<dyn TransactionRepositoryPort>,
        crypto_address_repository: Arc<dyn CryptoAddressRepositoryPort>,
        payment_method_repository: Arc<dyn PaymentMethodRepositoryPort>,
        event_bus: Arc<dyn crate::domain::shared_kernel::DomainEventBus>,
    ) -> Self {
        let create_payment_handler = CreatePaymentCommandHandler::new(
            payment_repository.clone(),
            event_bus,
        );
        
        Self {
            payment_repository,
            transaction_repository,
            crypto_address_repository,
            payment_method_repository,
            create_payment_handler,
        }
    }
}

/// Payment operations that replace legacy handler logic
impl PaymentServiceIntegration {
    /// Create new payment (replaces payment creation handler logic)
    pub async fn create_payment(
        &self,
        user_id: UserId,
        amount: PaymentAmount,
        method_type: String,
        reference: Option<String>,
        metadata: std::collections::HashMap<String, String>,
    ) -> Result<PaymentCreationResult, PaymentError> {
        info!(user_id = %user_id, amount = %amount.amount(), "Creating payment via DDD");
        
        // For now, use hardcoded currency and network since PaymentMethodConfig doesn't contain these fields
        // In production, this would be derived from the method_type or passed as parameters
        let currency = Currency::Usdt; // Default currency
        let network = Some(Network::Ethereum); // Default network for crypto
        
        // Create payment method
        let method_type_enum = method_type.parse().map_err(|_| PaymentError::UnsupportedMethod(method_type))?;
        let payment_method = PaymentMethod::new(
            method_type_enum,
            currency,
            network,
        ).map_err(|e| PaymentError::InvalidMethod(format!("{:?}", e)))?;
        
        // Create command
        let mut command = CreatePaymentCommand::new(user_id, amount, payment_method);
        
        if let Some(ref_str) = reference {
            command = command.with_reference(ref_str);
        }
        
        for (key, value) in metadata {
            command = command.with_metadata(key, value);
        }
        
        // Execute command
        let response = self.create_payment_handler
            .handle(command)
            .await
            .map_err(|e| PaymentError::CommandExecution(format!("{:?}", e)))?;
        
        Ok(PaymentCreationResult {
            payment_id: response.payment_id.to_string(),
            reference: response.reference,
            status: format!("{:?}", response.status),
            created_at: response.created_at,
            expires_at: response.expires_at,
            instructions: response.payment_instructions.map(|inst| PaymentInstructionsDto {
                method: inst.method,
                address: inst.address,
                amount: inst.amount.amount().to_string(),
                currency: inst.amount.currency().to_string(),
                memo: inst.memo,
                expires_at: inst.expires_at,
            }),
        })
    }
    
    /// Get payment details (replaces payment lookup handler logic)
    pub async fn get_payment(
        &self,
        payment_id: &str,
        user_id: &UserId,
    ) -> Result<PaymentDetailsResult, PaymentError> {
        info!(payment_id = payment_id, user_id = %user_id, "Getting payment details via DDD");
        
        let payment_id = PaymentId::from_string(payment_id)
            .map_err(|e| PaymentError::InvalidId(e.to_string()))?;
        
        // Find payment
        let payment = self.payment_repository
            .find_by_id(&payment_id)
            .await
            .map_err(PaymentError::Repository)?
            .ok_or(PaymentError::NotFound)?;
        
        // Verify user owns payment
        if payment.user_id() != user_id {
            return Err(PaymentError::Unauthorized);
        }
        
        // Get transaction history if crypto payment
        let transactions = if matches!(payment.method().method_type(), crate::domain::payment::PaymentMethodType::Crypto) {
            self.transaction_repository
                .get_transaction_history(&payment_id)
                .await
                .map_err(PaymentError::Repository)?
        } else {
            vec![]
        };
        
        Ok(PaymentDetailsResult {
            payment_id: payment.id().to_string(),
            reference: payment.reference().to_string(),
            amount: payment.amount().amount().to_string(),
            currency: payment.amount().currency().to_string(),
            method: format!("{:?}", payment.method().method_type()),
            status: format!("{:?}", payment.status()),
            created_at: payment.created_at(),
            updated_at: payment.updated_at(),
            expires_at: payment.expires_at(),
            transactions: transactions.into_iter().map(|tx| TransactionDto {
                hash: tx.tx_hash.to_string(),
                network: tx.network,
                confirmations: tx.confirmations,
                required_confirmations: tx.required_confirmations,
                confirmed: tx.confirmed_at.is_some(),
                created_at: tx.created_at,
                confirmed_at: tx.confirmed_at,
            }).collect(),
        })
    }
    
    /// Get user payment history (replaces payment history handler logic)
    pub async fn get_user_payments(
        &self,
        user_id: &UserId,
        status_filter: Option<String>,
        limit: Option<u32>,
        offset: Option<u32>,
    ) -> Result<PaymentHistoryResult, PaymentError> {
        info!(user_id = %user_id, "Getting user payment history via DDD");
        
        // Get user payments
        let mut payments = self.payment_repository
            .find_by_user(user_id)
            .await
            .map_err(PaymentError::Repository)?;
        
        // Filter by status if specified
        if let Some(status_str) = status_filter {
            let filter_status: PaymentStatus = status_str.parse()
                .map_err(|_| PaymentError::InvalidStatus(status_str))?;
            payments.retain(|p| p.status() == filter_status);
        }
        
        // Sort by creation date (newest first)
        payments.sort_by(|a, b| b.created_at().cmp(&a.created_at()));
        
        // Apply pagination
        let total_count = payments.len();
        let offset = offset.unwrap_or(0) as usize;
        let limit = limit.unwrap_or(50) as usize;
        
        let paginated_payments = payments.into_iter()
            .skip(offset)
            .take(limit)
            .collect::<Vec<_>>();
        
        // Convert to DTOs
        let payment_dtos = paginated_payments.into_iter().map(|payment| PaymentSummaryDto {
            payment_id: payment.id().to_string(),
            reference: payment.reference().to_string(),
            amount: payment.amount().amount().to_string(),
            currency: payment.amount().currency().to_string(),
            method: format!("{:?}", payment.method().method_type()),
            status: format!("{:?}", payment.status()),
            created_at: payment.created_at(),
            expires_at: payment.expires_at(),
        }).collect();
        
        Ok(PaymentHistoryResult {
            payments: payment_dtos,
            total_count,
            offset,
            limit,
        })
    }
    
    /// Get payment statistics (replaces payment stats handler logic)
    pub async fn get_payment_stats(
        &self,
        user_id: &UserId,
    ) -> Result<PaymentStatsResult, PaymentError> {
        info!(user_id = %user_id, "Getting payment statistics via DDD");
        
        let stats = self.payment_repository
            .get_user_payment_stats(user_id)
            .await
            .map_err(PaymentError::Repository)?;
        
        Ok(PaymentStatsResult {
            total_payments: stats.total_payments,
            completed_payments: stats.completed_payments,
            failed_payments: stats.failed_payments,
            pending_payments: stats.total_payments - stats.completed_payments - stats.failed_payments,
            total_amount: stats.total_amount.amount().to_string(),
            total_currency: stats.total_amount.currency().to_string(),
            average_amount: stats.average_amount.amount().to_string(),
            last_payment_date: stats.last_payment_date,
        })
    }
    
    /// Cancel payment (replaces payment cancellation handler logic)
    pub async fn cancel_payment(
        &self,
        payment_id: &str,
        user_id: &UserId,
        reason: Option<String>,
    ) -> Result<(), PaymentError> {
        info!(payment_id = payment_id, user_id = %user_id, "Cancelling payment via DDD");
        
        let payment_id = PaymentId::from_string(payment_id)
            .map_err(|e| PaymentError::InvalidId(e.to_string()))?;
        
        // Find payment
        let mut payment = self.payment_repository
            .find_by_id(&payment_id)
            .await
            .map_err(PaymentError::Repository)?
            .ok_or(PaymentError::NotFound)?;
        
        // Verify user owns payment
        if payment.user_id() != user_id {
            return Err(PaymentError::Unauthorized);
        }
        
        // Cancel payment
        payment.cancel(reason.unwrap_or_else(|| "User cancellation".to_string()))
            .map_err(|e| PaymentError::DomainError(format!("{:?}", e)))?;
        
        // Save updated payment
        self.payment_repository
            .save(&payment)
            .await
            .map_err(PaymentError::Repository)?;
        
        info!(payment_id = %payment_id, "Payment cancelled successfully");
        Ok(())
    }
}

/// Result types for payment operations
#[derive(Debug, Clone)]
pub struct PaymentCreationResult {
    pub payment_id: String,
    pub reference: String,
    pub status: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub expires_at: Option<chrono::DateTime<chrono::Utc>>,
    pub instructions: Option<PaymentInstructionsDto>,
}

#[derive(Debug, Clone)]
pub struct PaymentInstructionsDto {
    pub method: String,
    pub address: Option<String>,
    pub amount: String,
    pub currency: String,
    pub memo: Option<String>,
    pub expires_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Debug, Clone)]
pub struct PaymentDetailsResult {
    pub payment_id: String,
    pub reference: String,
    pub amount: String,
    pub currency: String,
    pub method: String,
    pub status: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
    pub expires_at: Option<chrono::DateTime<chrono::Utc>>,
    pub transactions: Vec<TransactionDto>,
}

#[derive(Debug, Clone)]
pub struct TransactionDto {
    pub hash: String,
    pub network: String,
    pub confirmations: u32,
    pub required_confirmations: u32,
    pub confirmed: bool,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub confirmed_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Debug, Clone)]
pub struct PaymentHistoryResult {
    pub payments: Vec<PaymentSummaryDto>,
    pub total_count: usize,
    pub offset: usize,
    pub limit: usize,
}

#[derive(Debug, Clone)]
pub struct PaymentSummaryDto {
    pub payment_id: String,
    pub reference: String,
    pub amount: String,
    pub currency: String,
    pub method: String,
    pub status: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub expires_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Debug, Clone)]
pub struct PaymentStatsResult {
    pub total_payments: u32,
    pub completed_payments: u32,
    pub failed_payments: u32,
    pub pending_payments: u32,
    pub total_amount: String,
    pub total_currency: String,
    pub average_amount: String,
    pub last_payment_date: Option<chrono::DateTime<chrono::Utc>>,
}

/// Error types for payment operations
#[derive(Debug, thiserror::Error)]
pub enum PaymentError {
    #[error("Payment not found")]
    NotFound,
    
    #[error("Unauthorized access to payment")]
    Unauthorized,
    
    #[error("Invalid payment ID: {0}")]
    InvalidId(String),
    
    #[error("Invalid payment status: {0}")]
    InvalidStatus(String),
    
    #[error("Unsupported payment method: {0}")]
    UnsupportedMethod(String),
    
    #[error("Invalid payment method: {0}")]
    InvalidMethod(String),
    
    #[error("Repository error: {0}")]
    Repository(String),
    
    #[error("Command execution failed: {0}")]
    CommandExecution(String),
    
    #[error("Domain error: {0}")]
    DomainError(String),
}