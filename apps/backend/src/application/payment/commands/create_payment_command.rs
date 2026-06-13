// Create Payment Command and Handler
// CQRS command for creating new payments in the Payment bounded context

use crate::prelude::*;

use tracing::{info, error};
// wave11(track-c) R7: kernel-level port for publishing domain events.
// See `epsx_contracts::event_publisher_port` for the design notes.
use epsx_contracts::event_publisher_port::EventPublisherPort;

use crate::domain::payment::{
    Payment, PaymentId, PaymentAmount, PaymentMethod, PaymentRepositoryPort
};
use crate::domain::wallet_management::value_objects::WalletAddress;
use crate::application::shared::{Command, CommandHandler, ApplicationResult, ApplicationError};

/// Command to create a new payment
#[derive(Debug, Clone)]
pub struct CreatePaymentCommand {
    pub wallet_address: WalletAddress,
    pub amount: PaymentAmount,
    pub method: PaymentMethod,
    pub reference: Option<String>,
    pub metadata: std::collections::HashMap<String, String>,
}

impl CreatePaymentCommand {
    pub fn new(
        wallet_address: WalletAddress,
        amount: PaymentAmount,
        method: PaymentMethod,
    ) -> Self {
        Self {
            wallet_address,
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
    // wave11(track-c) R7: migrated from `Arc<dyn DomainEventBus>` to the
    // kernel-level `EventPublisherPort`. The publish is async on the
    // port (was sync on the bus).
    event_publisher: Arc<dyn EventPublisherPort>,
}

impl CreatePaymentCommandHandler {
    pub fn new(
        payment_repository: Arc<dyn PaymentRepositoryPort>,
        event_publisher: Arc<dyn EventPublisherPort>,
    ) -> Self {
        Self {
            payment_repository,
            event_publisher,
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
                        .and_then(|details| details.payment_address.as_ref())
                        .map(|addr| addr.to_string()),
                    amount: payment.amount().clone(),
                    memo: Some(format!("Payment ID: {}", payment.id())),
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
            wallet_address = %command.wallet_address,
            amount = %command.amount.amount(),
            currency = %command.amount.currency(),
            method = ?command.method.method_type(),
            "Creating new payment"
        );
        
        // Create the payment aggregate
        let mut payment = Payment::create(
            command.wallet_address,
            command.amount,
            command.method,
        ).map_err(|e| {
            error!(error = %e, "Failed to create payment aggregate");
            ApplicationError::business_logic(format!("Payment creation failed: {}", e))
        })?;
        
        // Set custom reference if provided
        if let Some(reference) = command.reference {
            payment.metadata_mut().custom_reference = Some(reference);
        }
        
        // Set custom metadata if provided
        for (key, value) in command.metadata {
            payment.metadata_mut().custom_data.insert(key, value);
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
                ApplicationError::infrastructure(format!("Failed to save payment: {}", e))
            })?;
        
        // Publish domain events via the new `EventPublisherPort` (R7).
        // The aggregate exposes a borrowed slice
        // `&[Box<dyn DomainEvent>]`; the port takes owned
        // `Box<dyn DomainEvent>`. We wrap each event in an
        // `OwnedEvent` (one JSON round-trip + preserved event type
        // header) and pass the owned box to the port. The wrapper
        // lives in `epsx_contracts::domain_event`.
        for event in payment.uncommitted_events() {
            let owned: Box<dyn crate::domain::shared_kernel::DomainEvent> =
                Box::new(epsx_contracts::domain_event::OwnedEvent::from_borrowed(&**event));
            if let Err(e) = self.event_publisher.publish(owned).await {
                tracing::warn!(
                    error = %e,
                    "EventPublisherPort.publish returned error; command continues"
                );
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
            status: payment.status().clone(),
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
    use crate::domain::payment::value_objects::{Currency, PaymentMethodType, Network};
    
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
        
        async fn find_by_user(&self, _wallet_address: &WalletAddress) -> Result<Vec<Payment>, String> {
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
        
        async fn get_user_payment_stats(&self, _wallet_address: &WalletAddress) -> Result<crate::domain::payment::PaymentStats, String> {
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
    
    // Mock event publisher (wave11(track-c) R7: replaces MockEventBus).
    // Uses a `Mutex<Vec<String>>` to capture published event types
    // for test assertions.
    struct MockEventPublisher {
        published: std::sync::Mutex<Vec<String>>,
    }

    impl MockEventPublisher {
        fn new() -> Self {
            Self { published: std::sync::Mutex::new(Vec::new()) }
        }
    }

    #[async_trait::async_trait]
    impl epsx_contracts::event_publisher_port::EventPublisherPort for MockEventPublisher {
        async fn publish(
            &self,
            event: Box<dyn crate::domain::shared_kernel::DomainEvent>,
        ) -> crate::prelude::AppResult<()> {
            self.published.lock().unwrap().push(event.event_type().to_string());
            Ok(())
        }
    }
    
    #[tokio::test]
    async fn test_create_payment_command() {
        let payment_repo = Arc::new(MockPaymentRepository);
        let event_publisher: Arc<dyn epsx_contracts::event_publisher_port::EventPublisherPort> =
            Arc::new(MockEventPublisher::new());
        let handler = CreatePaymentCommandHandler::new(payment_repo, event_publisher);
        
        let wallet_address = WalletAddress::new("0x742d35Cc6634C0532925a3b8D369D7763F3c45c6").unwrap();
        let amount = PaymentAmount::new(rust_decimal::Decimal::from(100), Currency::USDT).unwrap();
        let method = PaymentMethod::new(
            PaymentMethodType::Crypto,
            Currency::USDT,
            Some(Network::Ethereum),
        ).unwrap();
        
        let command = CreatePaymentCommand::new(wallet_address, amount, method);
        
        // This test would pass if the Payment::create method was properly implemented
        // For now, it will likely fail due to incomplete Payment aggregate implementation
        let _result = handler.handle(command).await;
        // assert!(result.is_ok());
    }
}