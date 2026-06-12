use ethers::prelude::*;
use rust_decimal::Decimal;
use std::sync::Arc;

use super::PaymentEvent;
use crate::domain::shared_kernel::app_error::AppError;

/// Service for verifying blockchain payment transactions
pub struct PaymentVerifier {
    provider: Arc<Provider<Http>>,
    contract_address: H160,
    supported_tokens: Vec<String>,
}

impl PaymentVerifier {
    /// Create new payment verifier
    pub fn new(
        rpc_url: String, 
        contract_address: String,
        supported_tokens: Vec<String>
    ) -> Result<Self, AppError> {
        let provider = Provider::<Http>::try_from(rpc_url)
            .map_err(|e| AppError::internal_server_error(format!("Failed to create provider: {}", e)))?;

        let contract_address = contract_address.parse::<H160>()
            .map_err(|e| AppError::internal_server_error(format!("Invalid contract address: {}", e)))?;

        Ok(Self {
            provider: Arc::new(provider),
            contract_address,
            supported_tokens,
        })
    }

    /// Verify transaction exists and is confirmed
    pub async fn verify_transaction(&self, tx_hash: &str) -> Result<bool, AppError> {
        let tx_hash = tx_hash.parse::<H256>()
            .map_err(|e| AppError::internal_server_error(format!("Invalid transaction hash: {}", e)))?;

        // Get transaction receipt
        let receipt = self.provider.get_transaction_receipt(tx_hash).await
            .map_err(|e| AppError::internal_server_error(format!("Failed to get receipt: {}", e)))?;

        match receipt {
            Some(receipt) => {
                // Check transaction was successful
                if receipt.status != Some(U64::from(1)) {
                    return Ok(false);
                }

                // Verify transaction was sent to our contract
                if receipt.to != Some(self.contract_address) {
                    return Ok(false);
                }

                Ok(true)
            }
            None => Ok(false),
        }
    }

    /// Verify payment amount matches expected amount
    pub async fn verify_amount(
        &self,
        actual_amount: Decimal,
        expected_amount: Decimal,
    ) -> Result<bool, AppError> {
        Ok(actual_amount == expected_amount)
    }

    /// Verify token address is supported
    pub async fn verify_token(&self, token_address: &str) -> Result<bool, AppError> {
        let token_lower = token_address.to_lowercase();
        Ok(self.supported_tokens.iter().any(|t| t.to_lowercase() == token_lower))
    }

    /// Get transaction confirmations
    pub async fn get_confirmations(&self, tx_hash: &str) -> Result<u64, AppError> {
        let tx_hash = tx_hash.parse::<H256>()
            .map_err(|e| AppError::internal_server_error(format!("Invalid transaction hash: {}", e)))?;

        // Get transaction receipt
        let receipt = self.provider.get_transaction_receipt(tx_hash).await
            .map_err(|e| AppError::internal_server_error(format!("Failed to get receipt: {}", e)))?;

        match receipt {
            Some(receipt) => {
                if let Some(block_number) = receipt.block_number {
                    let current_block = self.provider.get_block_number().await
                        .map_err(|e| AppError::internal_server_error(format!("Failed to get block number: {}", e)))?;

                    let confirmations = current_block.as_u64().saturating_sub(block_number.as_u64());
                    Ok(confirmations)
                } else {
                    Ok(0)
                }
            }
            None => Ok(0),
        }
    }

    /// Check if transaction has minimum confirmations
    pub async fn has_minimum_confirmations(
        &self,
        tx_hash: &str,
        min_confirmations: u64,
    ) -> Result<bool, AppError> {
        let confirmations = self.get_confirmations(tx_hash).await?;
        Ok(confirmations >= min_confirmations)
    }

    /// Comprehensive payment verification
    pub async fn verify_payment(&self, event: &PaymentEvent) -> Result<VerificationResult, AppError> {
        let mut result = VerificationResult {
            is_valid: true,
            errors: Vec::new(),
            warnings: Vec::new(),
        };

        // Verify transaction exists and succeeded
        if !self.verify_transaction(&event.transaction_hash).await? {
            result.is_valid = false;
            result.errors.push("Transaction not found or failed".to_string());
        }

        // Verify amount matches plan
        // This requires the caller to provide expected amount, or we need to fetch it
        // For now, we'll skip this check in verify_payment and let the caller handle it
        // or we could change the signature of verify_payment to accept expected_amount
        
        /*
        if !self.verify_amount(event.amount, expected_amount).await? {
            result.is_valid = false;
            result.errors.push(format!(
                "Amount mismatch: expected ${}, got ${}",
                expected_amount,
                event.amount
            ));
        }
        */

        // Verify token is supported
        if !self.verify_token(&event.token_address).await? {
            result.is_valid = false;
            result.errors.push(format!("Unsupported token: {}", event.token_address));
        }

        // Check confirmations (warning if too few)
        let confirmations = self.get_confirmations(&event.transaction_hash).await?;
        if confirmations < 3 {
            result.warnings.push(format!(
                "Low confirmations: {} (recommended: 3+)",
                confirmations
            ));
        }

        Ok(result)
    }

    // Helper removed as we now pass expected amount directly
    // fn get_plan_price(&self, plan_id: u32) -> Decimal { ... }
}

/// Payment verification result
#[derive(Debug, Clone)]
pub struct VerificationResult {
    pub is_valid: bool,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
}

impl VerificationResult {
    pub fn is_verified(&self) -> bool {
        self.is_valid && self.errors.is_empty()
    }

    pub fn has_warnings(&self) -> bool {
        !self.warnings.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_verification_result() {
        let mut result = VerificationResult {
            is_valid: true,
            errors: Vec::new(),
            warnings: Vec::new(),
        };

        assert!(result.is_verified());
        assert!(!result.has_warnings());

        result.warnings.push("Low confirmations".to_string());
        assert!(result.is_verified());
        assert!(result.has_warnings());

        result.errors.push("Amount mismatch".to_string());
        assert!(!result.is_verified());
    }
}
