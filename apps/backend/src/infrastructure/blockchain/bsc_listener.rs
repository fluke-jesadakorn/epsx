use ethers::prelude::*;
use std::sync::Arc;
use tokio::time::{sleep, Duration};
use tracing::{debug, error, info, warn};

use super::{PaymentEvent, parse_payment_event, PaymentVerifier};
use crate::domain::shared_kernel::app_error::AppError;

/// BSC blockchain event listener for PaymentReceived events
pub struct BscEventListener {
    provider: Arc<Provider<Http>>,
    contract_address: H160,
    payment_verifier: Arc<PaymentVerifier>,
    last_checked_block: u64,
    poll_interval_secs: u64,
    event_topic: H256,
}

impl BscEventListener {
    /// Create new BSC event listener
    pub fn new(
        rpc_url: String,
        contract_address: String,
        start_block: u64,
        poll_interval_secs: u64,
        supported_tokens: Vec<String>,
    ) -> Result<Self, AppError> {
        let provider = Provider::<Http>::try_from(&rpc_url)
            .map_err(|e| AppError::infrastructure_error(format!("Failed to create provider: {}", e)))?;

        let contract_address = contract_address.parse::<H160>()
            .map_err(|e| AppError::infrastructure_error(format!("Invalid contract address: {}", e)))?;

        let payment_verifier = Arc::new(PaymentVerifier::new(
            rpc_url.clone(),
            format!("{:#x}", contract_address),
            supported_tokens
        )?);

        // PaymentWithContext event topic (V2)
        // keccak256("PaymentWithContext(address,uint8,uint256,address,uint256,uint256,uint256,bytes32)")
        let event_topic = H256::from_slice(&hex::decode(
            "842de788230478cf96f2c9139ce2cedad856220f7accbee6cd941b420224a770"
        ).unwrap_or_default());

        Ok(Self {
            provider: Arc::new(provider),
            contract_address,
            payment_verifier,
            last_checked_block: start_block,
            poll_interval_secs,
            event_topic,
        })
    }

    /// Start listening for events
    pub async fn start_listening<F>(&mut self, callback: F) -> Result<(), AppError>
    where
        F: Fn(PaymentEvent) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<(), AppError>> + Send>> + Send + Sync,
    {
        info!("Starting BSC event listener on contract: {:?}", self.contract_address);
        info!("Starting from block: {}", self.last_checked_block);

        loop {
            match self.check_new_events().await {
                Ok(events) => {
                    if !events.is_empty() {
                        info!("Found {} new payment events", events.len());

                        for event in events {
                            // Verify payment before processing
                            match self.payment_verifier.verify_payment(&event).await {
                                Ok(verification) => {
                                    if verification.is_verified() {
                                        info!("Payment verified: {}", event.unique_id());

                                        for warning in &verification.warnings {
                                            warn!("{}", warning);
                                        }

                                        // Process event
                                        if let Err(e) = callback(event.clone()).await {
                                            error!("Failed to process event {}: {}", event.unique_id(), e);
                                        }
                                    } else {
                                        error!("Payment verification failed: {:?}", verification.errors);
                                    }
                                }
                                Err(e) => {
                                    error!("Payment verification error: {}", e);
                                }
                            }
                        }
                    }
                }
                Err(e) => {
                    error!("Error checking events: {}", e);
                }
            }

            // Wait before next poll
            sleep(Duration::from_secs(self.poll_interval_secs)).await;
        }
    }

    /// Check for new events since last checked block
    async fn check_new_events(&mut self) -> Result<Vec<PaymentEvent>, AppError> {
        let current_block = self.provider.get_block_number().await
            .map_err(|e| AppError::infrastructure_error(format!("Failed to get block number: {}", e)))?
            .as_u64();

        if current_block <= self.last_checked_block {
            debug!("No new blocks. Current: {}, Last checked: {}", current_block, self.last_checked_block);
            return Ok(Vec::new());
        }

        let from_block = self.last_checked_block + 1;
        let to_block = current_block;

        debug!("Checking blocks {} to {} for events", from_block, to_block);

        let events = self.fetch_payment_events(from_block, to_block).await?;

        // Update last checked block
        self.last_checked_block = current_block;

        Ok(events)
    }

    /// Fetch PaymentReceived events from blockchain
    async fn fetch_payment_events(
        &self,
        from_block: u64,
        to_block: u64,
    ) -> Result<Vec<PaymentEvent>, AppError> {
        let filter = Filter::new()
            .address(self.contract_address)
            .topic0(self.event_topic)
            .from_block(from_block)
            .to_block(to_block);

        let logs = self.provider.get_logs(&filter).await
            .map_err(|e| AppError::infrastructure_error(format!("Failed to get logs: {}", e)))?;

        let mut events = Vec::new();

        for log in logs {
            match parse_payment_event(&log) {
                Ok(event) => {
                    if event.is_valid() {
                        events.push(event);
                    } else {
                        warn!("Invalid event parsed: {:?}", event);
                    }
                }
                Err(e) => {
                    error!("Failed to parse event: {}", e);
                }
            }
        }

        Ok(events)
    }

    /// Get current block number
    pub async fn get_current_block(&self) -> Result<u64, AppError> {
        let block_number = self.provider.get_block_number().await
            .map_err(|e| AppError::infrastructure_error(format!("Failed to get block number: {}", e)))?;

        Ok(block_number.as_u64())
    }

    /// Update last checked block (useful for resuming from a specific block)
    pub fn set_last_checked_block(&mut self, block: u64) {
        self.last_checked_block = block;
        info!("Updated last checked block to: {}", block);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_bsc_listener_creation() {
        let listener = BscEventListener::new(
            "https://data-seed-prebsc-1-s1.binance.org:8545/".to_string(),
            "0x1234567890123456789012345678901234567890".to_string(),
            0,
            3,
            vec![], // Empty tokens list for testing
        );

        assert!(listener.is_ok(), "BscEventListener::new failed: {:?}", listener.err());
    }
}
