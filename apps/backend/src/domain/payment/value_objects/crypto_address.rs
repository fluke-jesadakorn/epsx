use serde::{Deserialize, Serialize};
use std::fmt::{self, Display};

pub use crate::domain::shared_kernel::value_objects::{Currency, Network};

/// Crypto Address Value Object
/// Validates and represents blockchain addresses with network context
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct CryptoAddress {
    address: String,
    network: Network,
    currency: Currency,
}

impl CryptoAddress {
    /// Create new crypto address with validation
    pub fn new(address: String, network: Network, currency: Currency) -> Result<Self, CryptoAddressError> {
        // Validate address format based on network
        Self::validate_address_format(&address, &network)?;
        
        // Validate currency is supported on network
        if !Self::is_currency_supported_on_network(&currency, &network) {
            return Err(CryptoAddressError::CurrencyNotSupportedOnNetwork {
                currency: currency.clone(),
                network: network.clone(),
            });
        }

        Ok(Self {
            address: address.to_lowercase(), // Normalize to lowercase
            network,
            currency,
        })
    }

    /// Get the address
    pub fn address(&self) -> &str {
        &self.address
    }

    /// Get the network
    pub fn network(&self) -> &Network {
        &self.network
    }

    /// Get the currency
    pub fn currency(&self) -> &Currency {
        &self.currency
    }

    /// Get address in checksum format (for Ethereum-like networks)
    pub fn checksum_address(&self) -> String {
        match self.network {
            Network::Ethereum | Network::Binance | Network::Arbitrum | Network::Polygon | Network::BinanceSmartChain => {
                Self::to_checksum_address(&self.address)
            }
            Network::Tron => self.address.clone(), // TRON uses different format
            Network::Bitcoin => self.address.clone(), // Bitcoin uses different format
        }
    }

    /// Check if address is a contract address (placeholder implementation)
    pub fn is_contract(&self) -> bool {
        // This would require blockchain queries in real implementation
        // For now, return false for all addresses
        false
    }

    /// Get address type
    pub fn address_type(&self) -> AddressType {
        match self.network {
            Network::Ethereum | Network::Binance | Network::Arbitrum | Network::Polygon | Network::BinanceSmartChain => {
                if self.address.starts_with("0x") {
                    AddressType::Externally
                } else {
                    AddressType::Invalid
                }
            }
            Network::Tron => {
                if self.address.starts_with('T') || self.address.starts_with('3') {
                    AddressType::Externally
                } else {
                    AddressType::Invalid
                }
            }
            Network::Bitcoin => {
                if self.address.starts_with('1') || self.address.starts_with('3') || self.address.starts_with("bc1") {
                    AddressType::Externally
                } else {
                    AddressType::Invalid
                }
            }
        }
    }

    /// Get network explorer URL for this address
    pub fn explorer_url(&self) -> String {
        let is_mainnet = std::env::var("BLOCKCHAIN_NETWORK")
            .unwrap_or_default()
            .eq_ignore_ascii_case("mainnet");
        let bsc_explorer = if is_mainnet { "https://bscscan.com/address/" } else { "https://testnet.bscscan.com/address/" };
        let base_url = match self.network {
            Network::Ethereum => "https://etherscan.io/address/",
            Network::Binance => bsc_explorer,
            Network::Tron => "https://tronscan.org/#/address/",
            Network::Arbitrum => "https://arbiscan.io/address/",
            Network::Polygon => "https://polygonscan.com/address/",
            Network::Bitcoin => "https://blockstream.info/address/",
            Network::BinanceSmartChain => bsc_explorer,
        };

        format!("{}{}", base_url, self.checksum_address())
    }

    /// Validate address format based on network
    fn validate_address_format(address: &str, network: &Network) -> Result<(), CryptoAddressError> {
        match network {
            Network::Ethereum | Network::Binance | Network::Arbitrum | Network::Polygon | Network::BinanceSmartChain => {
                Self::validate_ethereum_address(address)
            }
            Network::Tron => Self::validate_tron_address(address),
            Network::Bitcoin => Self::validate_bitcoin_address(address),
        }
    }

    /// Validate Ethereum-style address (0x followed by 40 hex chars)
    fn validate_ethereum_address(address: &str) -> Result<(), CryptoAddressError> {
        if !address.starts_with("0x") {
            return Err(CryptoAddressError::InvalidFormat {
                address: address.to_string(),
                expected: "Must start with 0x".to_string(),
            });
        }

        let hex_part = &address[2..];
        if hex_part.len() != 40 {
            return Err(CryptoAddressError::InvalidLength {
                address: address.to_string(),
                expected: 42, // 0x + 40 chars
                actual: address.len(),
            });
        }

        if !hex_part.chars().all(|c| c.is_ascii_hexdigit()) {
            return Err(CryptoAddressError::InvalidCharacters {
                address: address.to_string(),
                expected: "hexadecimal characters only".to_string(),
            });
        }

        Ok(())
    }

    /// Validate TRON address (Base58 encoded, starts with T)
    fn validate_tron_address(address: &str) -> Result<(), CryptoAddressError> {
        if address.len() != 34 {
            return Err(CryptoAddressError::InvalidLength {
                address: address.to_string(),
                expected: 34,
                actual: address.len(),
            });
        }

        if !address.starts_with('T') && !address.starts_with('3') {
            return Err(CryptoAddressError::InvalidFormat {
                address: address.to_string(),
                expected: "Must start with T or 3".to_string(),
            });
        }

        // Basic Base58 character validation
        let base58_chars = "123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz";
        if !address.chars().all(|c| base58_chars.contains(c)) {
            return Err(CryptoAddressError::InvalidCharacters {
                address: address.to_string(),
                expected: "Base58 characters only".to_string(),
            });
        }

        Ok(())
    }

    /// Validate Bitcoin address (Base58 encoded, multiple formats)
    fn validate_bitcoin_address(address: &str) -> Result<(), CryptoAddressError> {
        // Bitcoin addresses can be 25-34 characters long depending on type
        if address.len() < 25 || address.len() > 62 { // Including bech32 which can be longer
            return Err(CryptoAddressError::InvalidLength {
                address: address.to_string(),
                expected: 34, // Typical length
                actual: address.len(),
            });
        }

        // Check for valid prefixes
        if !address.starts_with('1') && !address.starts_with('3') && !address.starts_with("bc1") {
            return Err(CryptoAddressError::InvalidFormat {
                address: address.to_string(),
                expected: "Must start with 1, 3, or bc1".to_string(),
            });
        }

        // Basic character validation for legacy addresses (not comprehensive for bech32)
        if address.starts_with("bc1") {
            // Simple bech32 validation
            if !address.chars().all(|c| c.is_ascii_alphanumeric()) {
                return Err(CryptoAddressError::InvalidCharacters {
                    address: address.to_string(),
                    expected: "alphanumeric characters for bech32".to_string(),
                });
            }
        } else {
            // Base58 validation for legacy addresses
            let base58_chars = "123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz";
            if !address.chars().all(|c| base58_chars.contains(c)) {
                return Err(CryptoAddressError::InvalidCharacters {
                    address: address.to_string(),
                    expected: "Base58 characters only".to_string(),
                });
            }
        }

        Ok(())
    }

    /// Check if currency is supported on network
    fn is_currency_supported_on_network(currency: &Currency, network: &Network) -> bool {
        let supported_networks = currency.supported_networks();
        supported_networks.contains(network)
    }

    /// Convert address to checksum format (EIP-55)
    fn to_checksum_address(address: &str) -> String {
        if !address.starts_with("0x") {
            return address.to_string();
        }

        // Simple checksum implementation (in production, use proper keccak256)
        let addr_lower = address.to_lowercase();
        let hex_part = &addr_lower[2..];
        
        let mut checksum = String::from("0x");
        for (i, c) in hex_part.chars().enumerate() {
            if c.is_ascii_digit() {
                checksum.push(c);
            } else {
                // Simplified: alternate between upper and lower case
                // Real implementation uses keccak256 hash
                if i % 2 == 0 {
                    checksum.push(c.to_ascii_uppercase());
                } else {
                    checksum.push(c.to_ascii_lowercase());
                }
            }
        }
        
        checksum
    }
}

/// Address type classification
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AddressType {
    /// Externally Owned Account (user wallet)
    Externally,
    /// Smart Contract address
    Contract,
    /// Invalid address format
    Invalid,
}

/// Payment Address - combines crypto address with payment context
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PaymentAddress {
    crypto_address: CryptoAddress,
    label: Option<String>,
    is_verified: bool,
    created_at: chrono::DateTime<chrono::Utc>,
}

impl PaymentAddress {
    /// Create new payment address
    pub fn new(crypto_address: CryptoAddress, label: Option<String>) -> Self {
        Self {
            crypto_address,
            label,
            is_verified: false,
            created_at: chrono::Utc::now(),
        }
    }

    /// Get the crypto address
    pub fn crypto_address(&self) -> &CryptoAddress {
        &self.crypto_address
    }

    /// Get the label
    pub fn label(&self) -> Option<&str> {
        self.label.as_deref()
    }

    /// Check if address is verified
    pub fn is_verified(&self) -> bool {
        self.is_verified
    }

    /// Mark address as verified
    pub fn mark_verified(&mut self) {
        self.is_verified = true;
    }

    /// Set label
    pub fn set_label(&mut self, label: Option<String>) {
        self.label = label;
    }

    /// Get creation timestamp
    pub fn created_at(&self) -> chrono::DateTime<chrono::Utc> {
        self.created_at
    }

    /// Get display name (label or shortened address)
    pub fn display_name(&self) -> String {
        if let Some(label) = &self.label {
            format!("{} ({})", label, self.crypto_address.address())
        } else {
            let addr = self.crypto_address.address();
            if addr.len() > 10 {
                format!("{}...{}", &addr[0..6], &addr[addr.len()-4..])
            } else {
                addr.to_string()
            }
        }
    }
}

impl Display for CryptoAddress {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} ({})", self.address, self.network.short_name())
    }
}

impl Display for PaymentAddress {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.display_name())
    }
}

#[derive(Debug, thiserror::Error)]
pub enum CryptoAddressError {
    #[error("Invalid address format for {address}: {expected}")]
    InvalidFormat { address: String, expected: String },

    #[error("Invalid address length for {address}: expected {expected}, got {actual}")]
    InvalidLength {
        address: String,
        expected: usize,
        actual: usize,
    },

    #[error("Invalid characters in address {address}: {expected}")]
    InvalidCharacters { address: String, expected: String },

    #[error("Currency {currency} not supported on network {network}")]
    CurrencyNotSupportedOnNetwork { currency: Currency, network: Network },

    #[error("Address verification failed")]
    VerificationFailed,

    #[error("Checksum validation failed for address: {0}")]
    ChecksumValidationFailed(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ethereum_address_validation() {
        let valid_eth_addr = "0x742d35Cc3681d452bC9a4D0c99D2DB8b4E8B5f43";
        let crypto_addr = CryptoAddress::new(
            valid_eth_addr.to_string(),
            Network::Ethereum,
            Currency::ETH,
        );
        assert!(crypto_addr.is_ok());

        let addr = crypto_addr.unwrap();
        assert_eq!(addr.network(), &Network::Ethereum);
        assert_eq!(addr.currency(), &Currency::ETH);
        assert_eq!(addr.address_type(), AddressType::Externally);
    }

    #[test]
    fn test_invalid_ethereum_address() {
        let invalid_addresses = vec![
            "742d35Cc3681d452bC9a4D0c99D2DB8b4E8B5f43", // Missing 0x
            "0x742d35Cc3681d452bC9a4D0c99D2DB8b4E8B5f4", // Too short
            "0x742d35Cc3681d452bC9a4D0c99D2DB8b4E8B5f433", // Too long
            "0x742d35Cc3681d452bC9a4D0c99D2DB8b4E8B5g43", // Invalid character 'g'
        ];

        for addr in invalid_addresses {
            let result = CryptoAddress::new(
                addr.to_string(),
                Network::Ethereum,
                Currency::ETH,
            );
            assert!(result.is_err(), "Should be invalid: {}", addr);
        }
    }

    #[test]
    fn test_tron_address_validation() {
        let valid_tron_addr = "TLyqzVGLV1srkB7dToTAEqgDSfPtXRJZYH";
        let crypto_addr = CryptoAddress::new(
            valid_tron_addr.to_string(),
            Network::Tron,
            Currency::TRX,
        );
        assert!(crypto_addr.is_ok());

        let addr = crypto_addr.unwrap();
        assert_eq!(addr.network(), &Network::Tron);
        assert_eq!(addr.currency(), &Currency::TRX);
    }

    #[test]
    fn test_currency_network_compatibility() {
        // USDT is supported on Ethereum
        let usdt_eth = CryptoAddress::new(
            "0x742d35Cc3681d452bC9a4D0c99D2DB8b4E8B5f43".to_string(),
            Network::Ethereum,
            Currency::USDT,
        );
        assert!(usdt_eth.is_ok());

        // BTC is not supported on Ethereum
        let btc_eth = CryptoAddress::new(
            "0x742d35Cc3681d452bC9a4D0c99D2DB8b4E8B5f43".to_string(),
            Network::Ethereum,
            Currency::BTC,
        );
        assert!(btc_eth.is_err());
    }

    #[test]
    fn test_checksum_address() {
        let addr = CryptoAddress::new(
            "0x742d35cc3681d452bc9a4d0c99d2db8b4e8b5f43".to_string(),
            Network::Ethereum,
            Currency::ETH,
        ).unwrap();

        let checksum = addr.checksum_address();
        assert!(checksum.starts_with("0x"));
        assert_eq!(checksum.len(), 42);
        
        // Should have mixed case (simplified checksum)
        assert!(checksum.chars().any(|c| c.is_ascii_uppercase()));
        assert!(checksum.chars().any(|c| c.is_ascii_lowercase()));
    }

    #[test]
    fn test_explorer_url() {
        let addr = CryptoAddress::new(
            "0x742d35Cc3681d452bC9a4D0c99D2DB8b4E8B5f43".to_string(),
            Network::Ethereum,
            Currency::ETH,
        ).unwrap();

        let url = addr.explorer_url();
        assert!(url.starts_with("https://etherscan.io/address/0x"));
    }

    #[test]
    fn test_payment_address() {
        let crypto_addr = CryptoAddress::new(
            "0x742d35Cc3681d452bC9a4D0c99D2DB8b4E8B5f43".to_string(),
            Network::Ethereum,
            Currency::ETH,
        ).unwrap();

        let mut payment_addr = PaymentAddress::new(
            crypto_addr,
            Some("My Wallet".to_string()),
        );

        assert_eq!(payment_addr.label(), Some("My Wallet"));
        assert!(!payment_addr.is_verified());

        payment_addr.mark_verified();
        assert!(payment_addr.is_verified());

        let display = payment_addr.display_name();
        assert!(display.contains("My Wallet"));
    }

    #[test]
    fn test_payment_address_display() {
        let crypto_addr = CryptoAddress::new(
            "0x742d35Cc3681d452bC9a4D0c99D2DB8b4E8B5f43".to_string(),
            Network::Ethereum,
            Currency::ETH,
        ).unwrap();

        // With label
        let labeled_addr = PaymentAddress::new(
            crypto_addr.clone(),
            Some("Trading Wallet".to_string()),
        );
        let display = labeled_addr.display_name();
        assert!(display.contains("Trading Wallet"));

        // Without label
        let unlabeled_addr = PaymentAddress::new(crypto_addr, None);
        let display = unlabeled_addr.display_name();
        assert!(display.starts_with("0x742d"));
        assert!(display.ends_with("5f43"));
        assert!(display.contains("..."));
    }
}