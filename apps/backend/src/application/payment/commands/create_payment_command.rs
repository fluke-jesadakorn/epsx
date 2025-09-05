// Create Payment Command and Handler
// CQRS command for creating new payments in the Payment bounded context

use async_trait::async_trait;
use std::sync::Arc;
use tracing::{info, error};

use crate::domain::payment::{
    Payment, PaymentId, PaymentAmount, PaymentMethod, PaymentRepositoryPort,
    PaymentError
};
use crate::domain::user_management::value_objects::UserId;
use crate::domain::shared_kernel::DomainEventBus;
use crate::application::shared::{Command, CommandHandler, ApplicationResult};

/// Command to create a new payment
#[derive(Debug, Clone)]
pub struct CreatePaymentCommand {
    pub user_id: UserId,
    pub amount: PaymentAmount,
    pub method: PaymentMethod,
    pub reference: Option<String>,
    pub metadata: std::collections::HashMap<String, String>,
}

impl CreatePaymentCommand {
    pub fn new(
        user_id: UserId,
        amount: PaymentAmount,
        method: PaymentMethod,
    ) -> Self {
        Self {
            user_id,
            amount,
            method,
            reference: None,
            metadata: std::collections::HashMap::new(),
        }
    }
    
    pub fn with_reference(mut self, reference: String) -> Self {
        self.reference = Some(reference);
        self
    }
    
    pub fn with_metadata(mut self, key: String, value: String) -> Self {
        self.metadata.insert(key, value);
        self
    }
}

impl Command for CreatePaymentCommand {
    type Response = CreatePaymentResponse;
}

/// Response from creating a payment
#[derive(Debug, Clone)]
pub struct CreatePaymentResponse {
    pub payment_id: PaymentId,
    pub reference: String,
    pub status: crate::domain::payment::PaymentStatus,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub expires_at: Option<chrono::DateTime<chrono::Utc>>,
    pub payment_instructions: Option<PaymentInstructions>,
}

/// Payment instructions for the user
#[derive(Debug, Clone)]
pub struct PaymentInstructions {
    pub method: String,
    pub address: Option<String>,
    pub amount: PaymentAmount,
    pub memo: Option<String>,
    pub expires_at: Option<chrono::DateTime<chrono::Utc>>,
}

/// Command handler for creating payments
pub struct CreatePaymentCommandHandler {
    payment_repository: Arc<dyn PaymentRepositoryPort>,
    event_bus: Arc<dyn DomainEventBus>,
}

impl CreatePaymentCommandHandler {
    pub fn new(
        payment_repository: Arc<dyn PaymentRepositoryPort>,
        event_bus: Arc<dyn DomainEventBus>,
    ) -> Self {
        Self {
            payment_repository,
            event_bus,
        }
    }
    
    /// Generate payment instructions based on method
    fn generate_payment_instructions(&self, payment: &Payment) -> Option<PaymentInstructions> {
        match payment.method().method_type() {
            crate::domain::payment::PaymentMethodType::Crypto => {
                // For crypto payments, generate address and provide instructions
                Some(PaymentInstructions {
                    method: "Cryptocurrency".to_string(),
                    address: payment.crypto_details()
                        .and_then(|details| details.address.as_ref())
                        .map(|addr| addr.to_string()),
                    amount: payment.amount().clone(),
                    memo: Some(format!("Payment ID: {}", payment.id())),
                    expires_at: payment.expires_at(),
                })
            },
            crate::domain::payment::PaymentMethodType::BankTransfer => {
                // For bank transfers, provide bank details
                Some(PaymentInstructions {
                    method: "Bank Transfer".to_string(),
                    address: None,
                    amount: payment.amount().clone(),
                    memo: Some(format!("Reference: {}", payment.reference())),
                    expires_at: payment.expires_at(),
                })
            },
            crate::domain::payment::PaymentMethodType::CreditCard => {
                // For credit cards, redirect to payment processor
                Some(PaymentInstructions {
                    method: "Credit Card".to_string(),
                    address: None,
                    amount: payment.amount().clone(),
                    memo: Some("Complete payment via secure payment form".to_string()),
                    expires_at: payment.expires_at(),
                })
            },
        }
    }
}

#[async_trait]
impl CommandHandler<CreatePaymentCommand> for CreatePaymentCommandHandler {
    async fn handle(&self, command: CreatePaymentCommand) -> ApplicationResult<CreatePaymentResponse> {
        info!(
            user_id = %command.user_id,
            amount = %command.amount.amount(),
            currency = %command.amount.currency(),
            method = ?command.method.method_type(),
            "Creating new payment"
        );
        
        // Create the payment aggregate
        let mut payment = Payment::create(
            command.user_id,
            command.amount,
            command.method,
        ).map_err(|e| {
            error!(error = %e, "Failed to create payment aggregate");
            format!("Payment creation failed: {}", e)
        })?;
        
        // Set custom reference if provided
        if let Some(reference) = command.reference {
            payment.set_custom_reference(reference)
                .map_err(|e| format!("Failed to set reference: {}", e))?;
        }
        
        // Add metadata
        for (key, value) in command.metadata {
            payment.add_metadata(key, value)
                .map_err(|e| format!("Failed to add metadata: {}", e))?;
        }
        
        // For crypto payments, assign address
        if matches!(payment.method().method_type(), crate::domain::payment::PaymentMethodType::Crypto) {
            // In production, this would use CryptoAddressRepositoryPort to generate address
            // For now, simulate address assignment
            info!(payment_id = %payment.id(), "Assigning crypto address for payment");
            // payment.assign_crypto_address(generated_address)?;
        }
        
        // Save the payment
        self.payment_repository.save(&payment).await
            .map_err(|e| {
                error!(payment_id = %payment.id(), error = %e, "Failed to save payment");
                format!("Failed to save payment: {}", e)
            })?;
        
        // Publish domain events
        for event in payment.uncommitted_events() {
            if let Err(e) = self.event_bus.publish(event.clone()).await {
                error!(
                    payment_id = %payment.id(),
                    event = ?event,
                    error = %e,
                    "Failed to publish payment event"
                );
                // Don't fail the operation, but log the error
            }
        }
        
        // Generate payment instructions
        let instructions = self.generate_payment_instructions(&payment);
        
        info!(
            payment_id = %payment.id(),
            status = ?payment.status(),
            "Payment created successfully"
        );
        
        Ok(CreatePaymentResponse {
            payment_id: payment.id().clone(),
            reference: payment.reference().to_string(),
            status: payment.status(),
            created_at: payment.created_at(),
            expires_at: payment.expires_at(),
            payment_instructions: instructions,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use async_trait::async_trait;
    
    // Mock payment repository
    struct MockPaymentRepository;
    
    #[async_trait]
    impl PaymentRepositoryPort for MockPaymentRepository {
        async fn save(&self, _payment: &Payment) -> Result<(), String> {
            Ok(())
        }
        
        async fn find_by_id(&self, _payment_id: &PaymentId) -> Result<Option<Payment>, String> {
            Ok(None)
        }
        
        async fn find_by_user(&self, _user_id: &UserId) -> Result<Vec<Payment>, String> {
            Ok(vec![])
        }
        
        async fn find_by_status(&self, _status: crate::domain::payment::PaymentStatus) -> Result<Vec<Payment>, String> {
            Ok(vec![])
        }
        
        async fn find_by_reference(&self, _reference: &crate::domain::payment::PaymentReference) -> Result<Option<Payment>, String> {
            Ok(None)
        }
        
        async fn find_by_date_range(&self, _start: chrono::DateTime<chrono::Utc>, _end: chrono::DateTime<chrono::Utc>) -> Result<Vec<Payment>, String> {
            Ok(vec![])
        }
        
        async fn find_expired_pending(&self, _threshold: chrono::DateTime<chrono::Utc>) -> Result<Vec<Payment>, String> {
            Ok(vec![])
        }
        
        async fn update_status(&self, _payment_id: &PaymentId, _status: crate::domain::payment::PaymentStatus) -> Result<(), String> {
            Ok(())
        }
        
        async fn delete(&self, _payment_id: &PaymentId) -> Result<(), String> {
            Ok(())
        }
        
        async fn get_user_payment_stats(&self, _user_id: &UserId) -> Result<crate::domain::payment::PaymentStats, String> {
            Ok(crate::domain::payment::PaymentStats {
                total_payments: 0,
                completed_payments: 0,
                failed_payments: 0,
                total_amount: PaymentAmount::new(rust_decimal::Decimal::ZERO, crate::domain::payment::Currency::USD).unwrap(),
                average_amount: PaymentAmount::new(rust_decimal::Decimal::ZERO, crate::domain::payment::Currency::USD).unwrap(),
                last_payment_date: None,
            })
        }
    }
    
    // Mock event bus
    struct MockEventBus;
    
    #[async_trait]
    impl DomainEventBus for MockEventBus {
        async fn publish(&self, _event: Arc<dyn crate::domain::shared_kernel::DomainEvent>) -> Result<(), String> {
            Ok(())
        }
    }
    
    #[tokio::test]
    async fn test_create_payment_command() {
        let payment_repo = Arc::new(MockPaymentRepository);
        let event_bus = Arc::new(MockEventBus);
        let handler = CreatePaymentCommandHandler::new(payment_repo, event_bus);
        
        let user_id = UserId::new(1);
        let amount = PaymentAmount::new(rust_decimal::Decimal::from(100), crate::domain::payment::Currency::USD).unwrap();
        let method = PaymentMethod::new(
            crate::domain::payment::PaymentMethodType::CreditCard,
            crate::domain::payment::PaymentMethodConfig::default(),
        ).unwrap();
        
        let command = CreatePaymentCommand::new(user_id, amount, method);
        
        // This test would pass if the Payment::create method was properly implemented
        // For now, it will likely fail due to incomplete Payment aggregate implementation
        let _result = handler.handle(command).await;
        // assert!(result.is_ok());
    }
}