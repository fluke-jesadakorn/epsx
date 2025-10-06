use ethers::types::Address;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;
use thiserror::Error;

use crate::domain::shared_kernel::ValueObject;
use crate::core::errors::AppError;

/// Wallet Address value object
/// Represents a blockchain wallet address (Ethereum-compatible)
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct WalletAddress {
    value: String,
}

#[derive(Debug, Error)]
pub enum WalletAddressError {
    #[error("Wallet address cannot be empty")]
    Empty,
    #[error("Invalid wallet address format: {0}")]
    InvalidFormat(String),
    #[error("Wallet address must be exactly 42 characters long")]
    InvalidLength,
}

impl WalletAddress {
    /// Create a new wallet address
    pub fn new(value: impl Into<String>) -> Result<Self, WalletAddressError> {
        let value = value.into();
        Self::validate_address(&value)?;
        
        // Normalize to lowercase checksummed format
        let address = Address::from_str(&value)
            .map_err(|_| WalletAddressError::InvalidFormat(value.clone()))?;
        
        Ok(Self {
            value: address.to_string().to_lowercase(),
        })
    }

    /// Create from a trusted source (skipping validation)
    /// Only use this when you're certain the address is valid (e.g., from database)
    pub fn from_trusted(value: String) -> Self {
        Self { value }
    }

    /// Get the address as a string
    pub fn as_str(&self) -> &str {
        &self.value
    }

    /// Get the address as an ethers Address type
    pub fn as_address(&self) -> Result<Address, WalletAddressError> {
        Address::from_str(&self.value)
            .map_err(|_| WalletAddressError::InvalidFormat(self.value.clone()))
    }

    /// Get the checksummed version of the address
    pub fn to_checksum(&self) -> String {
        match self.as_address() {
            Ok(addr) => format!("{:?}", addr), // ethers formats with checksum
            Err(_) => self.value.clone(),
        }
    }

    /// Check if this address is the zero address
    pub fn is_zero(&self) -> bool {
        self.value == "0x0000000000000000000000000000000000000000"
    }

    /// Check if address is valid format
    pub fn is_valid(&self) -> bool {
        Self::validate_address(&self.value).is_ok()
    }
    
    /// Convert to UserId for compatibility with legacy session system
    /// In Web3-first architecture, wallet address IS the user ID
    pub fn to_user_id(&self) -> crate::domain::shared_kernel::value_objects::UserId {
        crate::domain::shared_kernel::value_objects::UserId::from_string(self.value.clone()).unwrap()
    }

    fn validate_address(value: &str) -> Result<(), WalletAddressError> {
        if value.is_empty() {
            return Err(WalletAddressError::Empty);
        }

        if value.len() != 42 {
            return Err(WalletAddressError::InvalidLength);
        }

        if !value.starts_with("0x") {
            return Err(WalletAddressError::InvalidFormat(
                "Address must start with 0x".to_string(),
            ));
        }

        // Validate using ethers library
        Address::from_str(value)
            .map_err(|_| WalletAddressError::InvalidFormat(value.to_string()))?;

        Ok(())
    }
}

impl ValueObject for WalletAddress {
    type Error = WalletAddressError;

    fn validate(&self) -> Result<(), Self::Error> {
        Self::validate_address(&self.value)
    }
}

impl fmt::Display for WalletAddress {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.value)
    }
}

impl FromStr for WalletAddress {
    type Err = WalletAddressError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::new(s)
    }
}

impl From<WalletAddress> for String {
    fn from(wallet: WalletAddress) -> Self {
        wallet.value
    }
}

impl TryFrom<String> for WalletAddress {
    type Error = WalletAddressError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

impl From<WalletAddressError> for AppError {
    fn from(error: WalletAddressError) -> Self {
        AppError::validation_error(format!("Invalid wallet address: {}", error))
            .with_component("wallet_address_value_object")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_wallet_address_should_succeed() {
        let addr = WalletAddress::new("0x742d35Cc6634C0532925a3b8D369D7763F3c45c6").unwrap();
        assert_eq!(addr.as_str(), "0x742d35cc6634c0532925a3b8d369d7763f3c45c6");
    }

    #[test]
    fn mixed_case_address_should_normalize_to_lowercase() {
        let addr = WalletAddress::new("0x742D35Cc6634C0532925A3B8D369D7763F3C45C6").unwrap();
        assert_eq!(addr.as_str(), "0x742d35cc6634c0532925a3b8d369d7763f3c45c6");
    }

    #[test]
    fn invalid_address_should_fail() {
        assert!(WalletAddress::new("invalid").is_err());
        assert!(WalletAddress::new("0x123").is_err());
        assert!(WalletAddress::new("").is_err());
    }

    #[test]
    fn zero_address_detection() {
        let zero_addr = WalletAddress::new("0x0000000000000000000000000000000000000000").unwrap();
        assert!(zero_addr.is_zero());
        
        let normal_addr = WalletAddress::new("0x742d35Cc6634C0532925a3b8D369D7763F3c45c6").unwrap();
        assert!(!normal_addr.is_zero());
    }

    #[test]
    fn checksum_formatting() {
        let addr = WalletAddress::new("0x742d35cc6634c0532925a3b8d369d7763f3c45c6").unwrap();
        let checksum = addr.to_checksum();
        assert!(checksum.starts_with("0x"));
        assert_eq!(checksum.len(), 42);
    }
}