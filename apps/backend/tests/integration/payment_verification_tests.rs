use std::sync::Arc;
use std::time::Duration;
use chrono::Utc;
use rust_decimal::Decimal;
use serde_json::json;
use sqlx::{PgPool, Row};
use tokio::time::sleep;
use uuid::Uuid;

use epsx::domain::payment::value_objects::{PaymentId, CryptoNetwork};
use epsx::domain::shared_kernel::value_objects::UserId;
use epsx::infrastructure::adapters::services::{
    SmartPaymentVerifier, AlloyBlockchainRpcService, BlockchainConfig,
    PaymentVerificationError, VerificationStatus
};

/// Integration tests for the complete payment verification workflow
/// Tests the end-to-end process from payment creation to subscription activation
pub struct PaymentVerificationIntegrationTests {
    db_pool: Arc<PgPool>,
    verifier: SmartPaymentVerifier,
}

impl PaymentVerificationIntegrationTests {
    pub async fn new(db_pool: Arc<PgPool>) -> Result<Self, Box<dyn std::error::Error>> {
        let blockchain_config = BlockchainConfig::testnet(); // Use testnet for integration tests
        let blockchain_service = AlloyBlockchainRpcService::new(blockchain_config)?;
        
        let verifier = SmartPaymentVerifier::new(
            db_pool.clone(),
            Arc::new(blockchain_service),
        );

        Ok(Self {
            db_pool,
            verifier,
        })
    }

    /// Test complete successful payment verification flow
    pub async fn test_successful_verification_flow(&self) -> Result<(), Box<dyn std::error::Error>> {
        println!("🧪 Testing successful payment verification flow...");

        // 1. Create test payment record
        let payment_id = self.create_test_payment_record().await?;
        
        // 2. Simulate transaction hash submission (this would come from user)
        let test_tx_hash = "0x742d35cc6d65c0532b86d1b0d8d2b5d4d7e8f9a0b1c2d3e4f5a6b7c8d9e0f1a2";
        self.update_payment_with_tx_hash(&payment_id, test_tx_hash).await?;

        // 3. Attempt verification (should succeed with mock blockchain service)
        let result = self.verifier.verify_and_auto_confirm(&payment_id).await?;

        // 4. Verify results
        assert_eq!(result.verification_result, VerificationStatus::ConfirmedAndActivated);
        assert!(result.blockchain_details.is_some());
        assert!(result.subscription_status.is_some());
        assert!(result.error_details.is_none());

        println!("✅ Successful verification flow test passed");
        Ok(())
    }

    /// Test payment not found scenario
    pub async fn test_payment_not_found(&self) -> Result<(), Box<dyn std::error::Error>> {
        println!("🧪 Testing payment not found scenario...");

        let non_existent_payment_id = PaymentId::generate();
        
        match self.verifier.verify_and_auto_confirm(&non_existent_payment_id).await {
            Err(PaymentVerificationError::PaymentNotFound { .. }) => {
                println!("✅ Payment not found error handled correctly");
                Ok(())
            },
            result => {
                panic!("Expected PaymentNotFound error, got: {:?}", result);
            }
        }
    }

    /// Test already processed payment scenario
    pub async fn test_already_processed_payment(&self) -> Result<(), Box<dyn std::error::Error>> {
        println!("🧪 Testing already processed payment scenario...");

        // 1. Create and process payment
        let payment_id = self.create_test_payment_record().await?;
        self.mark_payment_as_processed(&payment_id).await?;

        // 2. Attempt verification again
        let result = self.verifier.verify_and_auto_confirm(&payment_id).await?;

        // 3. Should return already processed status, not error
        assert_eq!(result.verification_result, VerificationStatus::AlreadyProcessed);
        assert!(result.error_details.is_none());

        println!("✅ Already processed payment test passed");
        Ok(())
    }

    /// Test pending confirmations scenario
    pub async fn test_pending_confirmations(&self) -> Result<(), Box<dyn std::error::Error>> {
        println!("🧪 Testing pending confirmations scenario...");

        // 1. Create payment with insufficient confirmations
        let payment_id = self.create_test_payment_record().await?;
        let test_tx_hash = "0x742d35cc6d65c0532b86d1b0d8d2b5d4d7e8f9a0b1c2d3e4f5a6b7c8d9e0f1a3";
        self.update_payment_with_tx_hash(&payment_id, test_tx_hash).await?;
        self.set_payment_confirmations(&payment_id, 5).await?; // Less than required 12

        // 2. Attempt verification
        let result = self.verifier.verify_and_auto_confirm(&payment_id).await?;

        // 3. Should return pending status with progress info
        assert_eq!(result.verification_result, VerificationStatus::PendingConfirmations);
        assert!(result.next_verification.is_some());
        assert_eq!(result.current_confirmations, 5);
        assert_eq!(result.required_confirmations, 12);

        println!("✅ Pending confirmations test passed");
        Ok(())
    }

    /// Test maximum attempts exceeded scenario
    pub async fn test_max_attempts_exceeded(&self) -> Result<(), Box<dyn std::error::Error>> {
        println!("🧪 Testing maximum attempts exceeded scenario...");

        // 1. Create payment with max attempts already reached
        let payment_id = self.create_test_payment_record().await?;
        self.set_payment_verification_attempts(&payment_id, 10).await?; // Max is typically 5-10

        // 2. Attempt verification
        match self.verifier.verify_and_auto_confirm(&payment_id).await {
            Err(PaymentVerificationError::MaxAttemptsExceeded { .. }) => {
                println!("✅ Max attempts exceeded error handled correctly");
                Ok(())
            },
            result => {
                panic!("Expected MaxAttemptsExceeded error, got: {:?}", result);
            }
        }
    }

    /// Test batch verification functionality
    pub async fn test_batch_verification(&self) -> Result<(), Box<dyn std::error::Error>> {
        println!("🧪 Testing batch verification functionality...");

        // 1. Create multiple test payments
        let payment_ids = vec![
            self.create_test_payment_record().await?,
            self.create_test_payment_record().await?,
            self.create_test_payment_record().await?,
        ];

        // 2. Set them as ready for verification
        for (i, payment_id) in payment_ids.iter().enumerate() {
            let tx_hash = format!("0x742d35cc6d65c0532b86d1b0d8d2b5d4d7e8f9a0b1c2d3e4f5a6b7c8d9e0f1a{}", i);
            self.update_payment_with_tx_hash(payment_id, &tx_hash).await?;
        }

        // 3. Run batch verification
        let results = self.verifier.batch_verify_ready_payments(Some(5)).await?;

        // 4. Verify results
        assert!(results.len() >= 3);
        println!("✅ Batch verification test passed with {} results", results.len());
        Ok(())
    }

    /// Test database transaction integrity
    pub async fn test_transaction_integrity(&self) -> Result<(), Box<dyn std::error::Error>> {
        println!("🧪 Testing database transaction integrity...");

        let payment_id = self.create_test_payment_record().await?;
        
        // Verify initial state
        let initial_attempts = self.get_payment_verification_attempts(&payment_id).await?;
        
        // Attempt verification (may fail, that's okay for this test)
        let _ = self.verifier.verify_and_auto_confirm(&payment_id).await;
        
        // Verify state was updated correctly
        let final_attempts = self.get_payment_verification_attempts(&payment_id).await?;
        assert!(final_attempts > initial_attempts, "Verification attempts should be incremented");
        
        println!("✅ Transaction integrity test passed");
        Ok(())
    }

    /// Test error recovery and retry logic
    pub async fn test_error_recovery(&self) -> Result<(), Box<dyn std::error::Error>> {
        println!("🧪 Testing error recovery and retry logic...");

        // This test would typically use a mock blockchain service that fails first, then succeeds
        // For now, we'll test the basic retry structure
        
        let payment_id = self.create_test_payment_record().await?;
        let test_tx_hash = "0x742d35cc6d65c0532b86d1b0d8d2b5d4d7e8f9a0b1c2d3e4f5a6b7c8d9e0f1a4";
        self.update_payment_with_tx_hash(&payment_id, test_tx_hash).await?;

        // Multiple attempts should be handled gracefully
        for i in 1..=3 {
            println!("  Attempt {}/3", i);
            let _ = self.verifier.verify_and_auto_confirm(&payment_id).await;
            sleep(Duration::from_millis(100)).await; // Small delay between attempts
        }

        println!("✅ Error recovery test completed");
        Ok(())
    }

    /// Helper: Create test payment record
    async fn create_test_payment_record(&self) -> Result<PaymentId, Box<dyn std::error::Error>> {
        let payment_id = PaymentId::generate();
        let user_id = UserId::generate();
        
        sqlx::query!(
            r#"
            INSERT INTO payment_records (
                id, user_id, wallet_address, transaction_hash, amount, network, 
                plan_id, verification_status, current_confirmations, required_confirmations,
                verification_attempts, created_at
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
            "#,
            payment_id.value(),
            user_id.value(),
            "0x742d35Cc6Df6Ce9532963E7F7d76b02e5C0d5dAa", // Test wallet address
            None::<String>, // No transaction hash initially
            json!({"amount": "100.00", "currency": "USDT"}),
            "BSC_TESTNET",
            1, // Test plan ID
            "pending_verification",
            0, // No confirmations initially
            12, // Required confirmations
            0, // No attempts initially
            Utc::now()
        )
        .execute(self.db_pool.as_ref())
        .await?;

        Ok(payment_id)
    }

    /// Helper: Update payment with transaction hash
    async fn update_payment_with_tx_hash(&self, payment_id: &PaymentId, tx_hash: &str) -> Result<(), Box<dyn std::error::Error>> {
        sqlx::query!(
            "UPDATE payment_records SET transaction_hash = $1 WHERE id = $2",
            tx_hash,
            payment_id.value()
        )
        .execute(self.db_pool.as_ref())
        .await?;
        Ok(())
    }

    /// Helper: Mark payment as already processed
    async fn mark_payment_as_processed(&self, payment_id: &PaymentId) -> Result<(), Box<dyn std::error::Error>> {
        sqlx::query!(
            "UPDATE payment_records SET verification_status = 'confirmed_and_activated' WHERE id = $1",
            payment_id.value()
        )
        .execute(self.db_pool.as_ref())
        .await?;
        Ok(())
    }

    /// Helper: Set payment confirmation count
    async fn set_payment_confirmations(&self, payment_id: &PaymentId, confirmations: i32) -> Result<(), Box<dyn std::error::Error>> {
        sqlx::query!(
            "UPDATE payment_records SET current_confirmations = $1 WHERE id = $2",
            confirmations,
            payment_id.value()
        )
        .execute(self.db_pool.as_ref())
        .await?;
        Ok(())
    }

    /// Helper: Set verification attempts count
    async fn set_payment_verification_attempts(&self, payment_id: &PaymentId, attempts: i32) -> Result<(), Box<dyn std::error::Error>> {
        sqlx::query!(
            "UPDATE payment_records SET verification_attempts = $1 WHERE id = $2",
            attempts,
            payment_id.value()
        )
        .execute(self.db_pool.as_ref())
        .await?;
        Ok(())
    }

    /// Helper: Get verification attempts count
    async fn get_payment_verification_attempts(&self, payment_id: &PaymentId) -> Result<i32, Box<dyn std::error::Error>> {
        let record = sqlx::query!(
            "SELECT verification_attempts FROM payment_records WHERE id = $1",
            payment_id.value()
        )
        .fetch_one(self.db_pool.as_ref())
        .await?;
        
        Ok(record.verification_attempts.unwrap_or(0))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    /// Test setup helper
    async fn setup_test_db() -> Arc<PgPool> {
        let database_url = env::var("DATABASE_URL")
            .unwrap_or_else(|_| "postgresql://postgres:password@localhost:5432/epsx_test".to_string());
        
        let pool = PgPool::connect(&database_url)
            .await
            .expect("Failed to connect to test database");
            
        Arc::new(pool)
    }

    #[tokio::test]
    async fn test_complete_verification_workflow() {
        let db_pool = setup_test_db().await;
        let test_suite = PaymentVerificationIntegrationTests::new(db_pool)
            .await
            .expect("Failed to create test suite");

        // Run all integration tests
        test_suite.test_successful_verification_flow().await.expect("Successful flow test failed");
        test_suite.test_payment_not_found().await.expect("Payment not found test failed");
        test_suite.test_already_processed_payment().await.expect("Already processed test failed");
        test_suite.test_pending_confirmations().await.expect("Pending confirmations test failed");
        test_suite.test_max_attempts_exceeded().await.expect("Max attempts test failed");
        test_suite.test_batch_verification().await.expect("Batch verification test failed");
        test_suite.test_transaction_integrity().await.expect("Transaction integrity test failed");
        test_suite.test_error_recovery().await.expect("Error recovery test failed");

        println!("🎉 All payment verification integration tests passed!");
    }

    #[tokio::test]
    async fn test_api_endpoint_integration() {
        // This would test the API endpoints directly using a test server
        // For now, we'll just verify the basic structure
        println!("🧪 API endpoint integration test placeholder");
        
        // TODO: Add actual API testing with test server
        // - POST /api/v1/payments/verify-and-confirm
        // - POST /api/v1/payments/create
        // - GET /api/v1/payments/:id/status
        // - POST /api/v1/payments/batch-verify
    }

    #[tokio::test] 
    async fn test_resilience_patterns() {
        println!("🧪 Testing resilience patterns integration...");
        
        // Test circuit breaker, retry policies, and rate limiting
        // These would integrate with the actual services
        
        // TODO: Add resilience pattern integration tests
        // - Circuit breaker behavior under load
        // - Retry policy effectiveness
        // - Rate limiting accuracy
        
        println!("✅ Resilience patterns integration test placeholder");
    }
}