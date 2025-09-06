use rust_decimal::Decimal;
use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use std::fmt::{self, Display};

pub use crate::domain::shared_kernel::value_objects::{Currency, Network};

/// Transaction Hash Value Object
/// Represents blockchain transaction identifiers with network-specific validation
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TransactionHash {
    hash: String,
    network: Network,
    block_number: Option<u64>,
    confirmations: u32,
}

impl TransactionHash {
    /// Create new transaction hash with validation
    pub fn new(hash: String, network: Network) -> Result<Self, TransactionHashError> {
        let normalized_hash = Self::normalize_hash(&hash, &network)?;
        Self::validate_hash_format(&normalized_hash, &network)?;

        Ok(Self {
            hash: normalized_hash,
            network,
            block_number: None,
            confirmations: 0,
        })
    }

    /// Create transaction hash with block information
    pub fn with_block_info(
        hash: String,
        network: Network,
        block_number: u64,
        confirmations: u32,
    ) -> Result<Self, TransactionHashError> {
        let mut tx_hash = Self::new(hash, network)?;
        tx_hash.block_number = Some(block_number);
        tx_hash.confirmations = confirmations;
        Ok(tx_hash)
    }

    /// Get the hash value
    pub fn hash(&self) -> &str {
        &self.hash
    }

    /// Get the network
    pub fn network(&self) -> &Network {
        &self.network
    }

    /// Get block number if known
    pub fn block_number(&self) -> Option<u64> {
        self.block_number
    }

    /// Get confirmation count
    pub fn confirmations(&self) -> u32 {
        self.confirmations
    }

    /// Update confirmation count
    pub fn update_confirmations(&mut self, confirmations: u32) {
        self.confirmations = confirmations;
    }

    /// Set block number
    pub fn set_block_number(&mut self, block_number: u64) {
        self.block_number = Some(block_number);
    }

    /// Check if transaction is confirmed (has minimum confirmations)
    pub fn is_confirmed(&self) -> bool {
        let min_confirmations = self.minimum_confirmations();
        self.confirmations >= min_confirmations
    }

    /// Check if transaction is finalized (has safe confirmations)
    pub fn is_finalized(&self) -> bool {
        let safe_confirmations = self.safe_confirmations();
        self.confirmations >= safe_confirmations
    }

    /// Get minimum confirmations required for this network
    pub fn minimum_confirmations(&self) -> u32 {
        match self.network {
            Network::Ethereum => 1,     // 1 confirmation for Ethereum
            Network::Binance => 1,      // 1 confirmation for BSC
            Network::Tron => 1,         // 1 confirmation for TRON
            Network::Arbitrum => 1,     // 1 confirmation for Arbitrum
            Network::Polygon => 1,      // 1 confirmation for Polygon
            Network::Bitcoin => 3,      // 3 confirmations for Bitcoin
            Network::BinanceSmartChain => 1, // 1 confirmation for BSC
        }
    }

    /// Get safe confirmations for this network (recommended for large amounts)
    pub fn safe_confirmations(&self) -> u32 {
        match self.network {
            Network::Ethereum => 12,    // 12 confirmations for Ethereum (~3 minutes)
            Network::Binance => 20,     // 20 confirmations for BSC (~1 minute)
            Network::Tron => 20,        // 20 confirmations for TRON (~1 minute)
            Network::Arbitrum => 1,     // 1 confirmation for Arbitrum (L2)
            Network::Polygon => 50,     // 50 confirmations for Polygon (~2 minutes)
            Network::Bitcoin => 6,      // 6 confirmations for Bitcoin (~1 hour)
            Network::BinanceSmartChain => 20, // 20 confirmations for BSC
        }
    }

    /// Get transaction explorer URL
    pub fn explorer_url(&self) -> String {
        let base_url = match self.network {
            Network::Ethereum => "https://etherscan.io/tx/",
            Network::Binance => "https://bscscan.com/tx/",
            Network::Tron => "https://tronscan.org/#/transaction/",
            Network::Arbitrum => "https://arbiscan.io/tx/",
            Network::Polygon => "https://polygonscan.com/tx/",
            Network::Bitcoin => "https://blockstream.info/tx/",
            Network::BinanceSmartChain => "https://bscscan.com/tx/",
        };
        
        format!("{}{}", base_url, self.hash)
    }

    /// Get confirmation status description
    pub fn confirmation_status(&self) -> ConfirmationStatus {
        if self.confirmations == 0 {
            ConfirmationStatus::Pending
        } else if self.confirmations < self.minimum_confirmations() {
            ConfirmationStatus::Confirming
        } else if self.confirmations < self.safe_confirmations() {
            ConfirmationStatus::Confirmed
        } else {
            ConfirmationStatus::Finalized
        }
    }

    /// Get short hash for display (first 8 and last 8 characters)
    pub fn short_hash(&self) -> String {
        if self.hash.len() > 16 {
            format!("{}...{}", &self.hash[0..8], &self.hash[self.hash.len()-8..])
        } else {
            self.hash.clone()
        }
    }

    /// Validate hash format based on network
    fn validate_hash_format(hash: &str, network: &Network) -> Result<(), TransactionHashError> {
        match network {
            Network::Ethereum | Network::Binance | Network::Arbitrum | Network::Polygon | Network::BinanceSmartChain => {
                Self::validate_ethereum_tx_hash(hash)
            }
            Network::Tron => Self::validate_tron_tx_hash(hash),
            Network::Bitcoin => Self::validate_bitcoin_tx_hash(hash),
        }
    }

    /// Validate Ethereum-style transaction hash
    fn validate_ethereum_tx_hash(hash: &str) -> Result<(), TransactionHashError> {
        if !hash.starts_with("0x") {
            return Err(TransactionHashError::InvalidFormat {
                hash: hash.to_string(),
                expected: "Must start with 0x".to_string(),
            });
        }

        let hex_part = &hash[2..];
        if hex_part.len() != 64 {
            return Err(TransactionHashError::InvalidLength {
                hash: hash.to_string(),
                expected: 66, // 0x + 64 chars
                actual: hash.len(),
            });
        }

        if !hex_part.chars().all(|c| c.is_ascii_hexdigit()) {
            return Err(TransactionHashError::InvalidCharacters {
                hash: hash.to_string(),
                expected: "hexadecimal characters only".to_string(),
            });
        }

        Ok(())
    }

    /// Validate TRON transaction hash
    fn validate_tron_tx_hash(hash: &str) -> Result<(), TransactionHashError> {
        if hash.len() != 64 {
            return Err(TransactionHashError::InvalidLength {
                hash: hash.to_string(),
                expected: 64,
                actual: hash.len(),
            });
        }

        if !hash.chars().all(|c| c.is_ascii_hexdigit()) {
            return Err(TransactionHashError::InvalidCharacters {
                hash: hash.to_string(),
                expected: "hexadecimal characters only".to_string(),
            });
        }

        Ok(())
    }

    /// Validate Bitcoin transaction hash
    fn validate_bitcoin_tx_hash(hash: &str) -> Result<(), TransactionHashError> {
        if hash.len() != 64 {
            return Err(TransactionHashError::InvalidLength {
                hash: hash.to_string(),
                expected: 64,
                actual: hash.len(),
            });
        }

        if !hash.chars().all(|c| c.is_ascii_hexdigit()) {
            return Err(TransactionHashError::InvalidCharacters {
                hash: hash.to_string(),
                expected: "hexadecimal characters only".to_string(),
            });
        }

        Ok(())
    }

    /// Normalize hash format
    fn normalize_hash(hash: &str, network: &Network) -> Result<String, TransactionHashError> {
        let trimmed = hash.trim();
        
        match network {
            Network::Ethereum | Network::Binance | Network::Arbitrum | Network::Polygon | Network::BinanceSmartChain => {
                // Ensure 0x prefix for Ethereum-like networks
                if trimmed.starts_with("0x") {
                    Ok(trimmed.to_lowercase())
                } else if trimmed.len() == 64 && trimmed.chars().all(|c| c.is_ascii_hexdigit()) {
                    Ok(format!("0x{}", trimmed.to_lowercase()))
                } else {
                    Err(TransactionHashError::InvalidFormat {
                        hash: hash.to_string(),
                        expected: "64 character hex string with or without 0x prefix".to_string(),
                    })
                }
            }
            Network::Tron => {
                // TRON hashes don't use 0x prefix
                if trimmed.starts_with("0x") {
                    Ok(trimmed[2..].to_lowercase())
                } else {
                    Ok(trimmed.to_lowercase())
                }
            }
            Network::Bitcoin => {
                // Bitcoin hashes don't use 0x prefix, use little-endian format
                if trimmed.starts_with("0x") {
                    Ok(trimmed[2..].to_lowercase())
                } else {
                    Ok(trimmed.to_lowercase())
                }
            }
        }
    }
}

/// Transaction confirmation status
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConfirmationStatus {
    /// Transaction is in mempool, not yet mined
    Pending,
    /// Transaction is mined but has insufficient confirmations
    Confirming,
    /// Transaction has minimum confirmations
    Confirmed,
    /// Transaction has safe number of confirmations
    Finalized,
}

impl ConfirmationStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            ConfirmationStatus::Pending => "pending",
            ConfirmationStatus::Confirming => "confirming",
            ConfirmationStatus::Confirmed => "confirmed",
            ConfirmationStatus::Finalized => "finalized",
        }
    }

    /// Check if status indicates successful confirmation
    pub fn is_success(&self) -> bool {
        matches!(self, ConfirmationStatus::Confirmed | ConfirmationStatus::Finalized)
    }

    /// Check if status is final (won't change)
    pub fn is_final(&self) -> bool {
        matches!(self, ConfirmationStatus::Finalized)
    }
}

impl Display for ConfirmationStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl Display for TransactionHash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} ({})", self.short_hash(), self.network.short_name())
    }
}

/// Transaction Receipt - Complete transaction information
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TransactionReceipt {
    transaction_hash: TransactionHash,
    from_address: String,
    to_address: String,
    gas_used: Option<u64>,
    gas_price: Option<u64>,
    status: TransactionStatus,
    timestamp: Option<chrono::DateTime<chrono::Utc>>,
}

impl TransactionReceipt {
    /// Create new transaction receipt
    pub fn new(
        transaction_hash: TransactionHash,
        from_address: String,
        to_address: String,
        status: TransactionStatus,
    ) -> Self {
        Self {
            transaction_hash,
            from_address,
            to_address,
            gas_used: None,
            gas_price: None,
            status,
            timestamp: None,
        }
    }

    /// Get transaction hash
    pub fn transaction_hash(&self) -> &TransactionHash {
        &self.transaction_hash
    }

    /// Get from address
    pub fn from_address(&self) -> &str {
        &self.from_address
    }

    /// Get to address
    pub fn to_address(&self) -> &str {
        &self.to_address
    }

    /// Get transaction status
    pub fn status(&self) -> &TransactionStatus {
        &self.status
    }

    /// Set gas information
    pub fn set_gas_info(&mut self, gas_used: u64, gas_price: u64) {
        self.gas_used = Some(gas_used);
        self.gas_price = Some(gas_price);
    }

    /// Set timestamp
    pub fn set_timestamp(&mut self, timestamp: chrono::DateTime<chrono::Utc>) {
        self.timestamp = Some(timestamp);
    }

    /// Calculate transaction fee (if gas info available)
    pub fn transaction_fee(&self) -> Option<u64> {
        if let (Some(gas_used), Some(gas_price)) = (self.gas_used, self.gas_price) {
            Some(gas_used * gas_price)
        } else {
            None
        }
    }

    /// Check if transaction was successful
    pub fn is_successful(&self) -> bool {
        matches!(self.status, TransactionStatus::Success)
    }
}

/// Transaction execution status
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TransactionStatus {
    /// Transaction executed successfully
    Success,
    /// Transaction failed during execution
    Failed,
    /// Transaction was reverted
    Reverted,
    /// Transaction is still being processed
    Processing,
}

impl TransactionStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            TransactionStatus::Success => "success",
            TransactionStatus::Failed => "failed",
            TransactionStatus::Reverted => "reverted",
            TransactionStatus::Processing => "processing",
        }
    }
}

impl Display for TransactionStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

#[derive(Debug, thiserror::Error)]
pub enum TransactionHashError {
    #[error("Invalid hash format for {hash}: {expected}")]
    InvalidFormat { hash: String, expected: String },

    #[error("Invalid hash length for {hash}: expected {expected}, got {actual}")]
    InvalidLength {
        hash: String,
        expected: usize,
        actual: usize,
    },

    #[error("Invalid characters in hash {hash}: {expected}")]
    InvalidCharacters { hash: String, expected: String },

    #[error("Transaction not found on blockchain")]
    TransactionNotFound,

    #[error("Network mismatch: transaction belongs to different network")]
    NetworkMismatch,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ethereum_transaction_hash() {
        let eth_tx = "0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef12";
        let tx_hash = TransactionHash::new(eth_tx.to_string(), Network::Ethereum).unwrap();

        assert_eq!(tx_hash.network(), &Network::Ethereum);
        assert_eq!(tx_hash.confirmations(), 0);
        assert!(!tx_hash.is_confirmed());
        assert_eq!(tx_hash.minimum_confirmations(), 1);
        assert_eq!(tx_hash.safe_confirmations(), 12);
    }

    #[test]
    fn test_transaction_hash_normalization() {
        // Test with 0x prefix
        let with_prefix = "0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef12";
        let tx1 = TransactionHash::new(with_prefix.to_string(), Network::Ethereum).unwrap();

        // Test without 0x prefix
        let without_prefix = "1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef12";
        let tx2 = TransactionHash::new(without_prefix.to_string(), Network::Ethereum).unwrap();

        assert_eq!(tx1.hash(), tx2.hash()); // Should be normalized to same format
    }

    #[test]
    fn test_tron_transaction_hash() {
        let tron_tx = "1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef12";
        let tx_hash = TransactionHash::new(tron_tx.to_string(), Network::Tron).unwrap();

        assert_eq!(tx_hash.network(), &Network::Tron);
        assert!(!tx_hash.hash().starts_with("0x")); // TRON doesn't use 0x prefix
    }

    #[test]
    fn test_invalid_transaction_hashes() {
        let invalid_hashes = vec![
            ("0x123", Network::Ethereum),                    // Too short
            ("0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef123", Network::Ethereum), // Too long
            ("0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdefzz", Network::Ethereum), // Invalid character
            ("12345", Network::Tron),                        // Too short for TRON
        ];

        for (hash, network) in invalid_hashes {
            let result = TransactionHash::new(hash.to_string(), network);
            assert!(result.is_err(), "Should be invalid: {}", hash);
        }
    }

    #[test]
    fn test_confirmation_updates() {
        let tx_hash = TransactionHash::new(
            "0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef12".to_string(),
            Network::Ethereum,
        ).unwrap();

        assert_eq!(tx_hash.confirmation_status(), ConfirmationStatus::Pending);

        let mut tx_hash = tx_hash;
        tx_hash.update_confirmations(1);
        assert!(tx_hash.is_confirmed());
        assert_eq!(tx_hash.confirmation_status(), ConfirmationStatus::Confirmed);

        tx_hash.update_confirmations(15);
        assert!(tx_hash.is_finalized());
        assert_eq!(tx_hash.confirmation_status(), ConfirmationStatus::Finalized);
    }

    #[test]
    fn test_explorer_url() {
        let tx_hash = TransactionHash::new(
            "0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef12".to_string(),
            Network::Ethereum,
        ).unwrap();

        let url = tx_hash.explorer_url();
        assert!(url.starts_with("https://etherscan.io/tx/0x"));
        assert!(url.contains("1234567890abcdef"));
    }

    #[test]
    fn test_short_hash() {
        let tx_hash = TransactionHash::new(
            "0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef12".to_string(),
            Network::Ethereum,
        ).unwrap();

        let short = tx_hash.short_hash();
        assert_eq!(short, "0x123456...cdef12");
        assert!(short.len() < tx_hash.hash().len());
    }

    #[test]
    fn test_transaction_receipt() {
        let tx_hash = TransactionHash::new(
            "0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef12".to_string(),
            Network::Ethereum,
        ).unwrap();

        let mut receipt = TransactionReceipt::new(
            tx_hash,
            "0xfrom123456789".to_string(),
            "0xto987654321".to_string(),
            TransactionStatus::Success,
        );

        assert!(receipt.is_successful());
        assert!(receipt.transaction_fee().is_none());

        receipt.set_gas_info(21000, 20000000000); // 21k gas at 20 gwei
        let fee = receipt.transaction_fee().unwrap();
        assert_eq!(fee, 420000000000000); // 21k * 20 gwei
    }

    #[test]
    fn test_confirmation_status_methods() {
        assert!(ConfirmationStatus::Confirmed.is_success());
        assert!(ConfirmationStatus::Finalized.is_success());
        assert!(!ConfirmationStatus::Pending.is_success());
        assert!(!ConfirmationStatus::Confirming.is_success());

        assert!(ConfirmationStatus::Finalized.is_final());
        assert!(!ConfirmationStatus::Confirmed.is_final());
    }
}