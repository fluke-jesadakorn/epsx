use serde::{Deserialize, Serialize};
use std::fmt::{self, Display};
use std::collections::HashMap;

pub use crate::dom::values::payments::{Currency, Network};

/// Payment Method Value Object
/// Represents different ways users can make payments
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PaymentMethod {
    method_type: PaymentMethodType,
    currency: Currency,
    network: Option<Network>,
    configuration: PaymentMethodConfig,
}

impl PaymentMethod {
    /// Create new payment method
    pub fn new(
        method_type: PaymentMethodType,
        currency: Currency,
        network: Option<Network>,
    ) -> Result<Self, PaymentMethodError> {
        // Validate currency is supported for this method
        if !method_type.supports_currency(&currency) {
            return Err(PaymentMethodError::UnsupportedCurrency {
                method: method_type,
                currency,
            });
        }

        // Validate network requirements
        match &method_type {
            PaymentMethodType::Crypto => {
                if network.is_none() {
                    return Err(PaymentMethodError::NetworkRequired(method_type));
                }
                
                let network = network.as_ref().unwrap();
                if !currency.supported_networks().contains(network) {
                    return Err(PaymentMethodError::UnsupportedNetwork {
                        currency: currency.clone(),
                        network: network.clone(),
                    });
                }
            }
            PaymentMethodType::BankTransfer | PaymentMethodType::CreditCard => {
                if network.is_some() {
                    return Err(PaymentMethodError::NetworkNotAllowed(method_type));
                }
            }
        }

        let configuration = PaymentMethodConfig::default_for_method(&method_type, &currency);

        Ok(Self {
            method_type,
            currency,
            network,
            configuration,
        })
    }

    /// Get payment method type
    pub fn method_type(&self) -> &PaymentMethodType {
        &self.method_type
    }

    /// Get currency
    pub fn currency(&self) -> &Currency {
        &self.currency
    }

    /// Get network (for crypto payments)
    pub fn network(&self) -> Option<&Network> {
        self.network.as_ref()
    }

    /// Get configuration
    pub fn configuration(&self) -> &PaymentMethodConfig {
        &self.configuration
    }

    /// Check if method is available
    pub fn is_available(&self) -> bool {
        self.configuration.is_enabled
    }

    /// Get processing time estimate in seconds
    pub fn processing_time_seconds(&self) -> u32 {
        self.configuration.processing_time_seconds
    }

    /// Get processing fee rate (as decimal, e.g., 0.025 = 2.5%)
    pub fn processing_fee_rate(&self) -> rust_decimal::Decimal {
        self.configuration.processing_fee_rate
    }

    /// Get minimum payment amount
    pub fn minimum_amount(&self) -> rust_decimal::Decimal {
        self.configuration.minimum_amount
    }

    /// Get maximum payment amount
    pub fn maximum_amount(&self) -> rust_decimal::Decimal {
        self.configuration.maximum_amount
    }

    /// Check if amount is within limits
    pub fn is_amount_valid(&self, amount: rust_decimal::Decimal) -> bool {
        amount >= self.minimum_amount() && amount <= self.maximum_amount()
    }

    /// Get user-friendly display name
    pub fn display_name(&self) -> String {
        match (&self.method_type, &self.network) {
            (PaymentMethodType::Crypto, Some(network)) => {
                format!("{} ({})", self.currency.symbol(), network.short_name())
            }
            (PaymentMethodType::BankTransfer, None) => {
                format!("Bank Transfer ({})", self.currency.symbol())
            }
            (PaymentMethodType::CreditCard, None) => {
                format!("Credit Card ({})", self.currency.symbol())
            }
            _ => format!("{} ({})", self.method_type, self.currency.symbol()),
        }
    }

    /// Get payment instructions for users
    pub fn get_instructions(&self) -> PaymentInstructions {
        match &self.method_type {
            PaymentMethodType::Crypto => PaymentInstructions::Crypto {
                currency: self.currency.clone(),
                network: self.network.clone().unwrap(),
                estimated_confirmations: self.network.as_ref().map(|n| match n {
                    Network::Ethereum => 12,
                    Network::Binance => 20,
                    Network::Tron => 20,
                    Network::Arbitrum => 1,
                    Network::Polygon => 50,
                }).unwrap_or(1),
            },
            PaymentMethodType::BankTransfer => PaymentInstructions::BankTransfer {
                currency: self.currency.clone(),
                processing_days: 1..=3,
            },
            PaymentMethodType::CreditCard => PaymentInstructions::CreditCard {
                currency: self.currency.clone(),
                instant_processing: true,
            },
        }
    }

    /// Update configuration
    pub fn update_config(&mut self, config: PaymentMethodConfig) -> Result<(), PaymentMethodError> {
        // Validate configuration
        if config.minimum_amount >= config.maximum_amount {
            return Err(PaymentMethodError::InvalidConfiguration(
                "Minimum amount must be less than maximum amount".to_string(),
            ));
        }

        if config.processing_fee_rate < rust_decimal::Decimal::ZERO 
            || config.processing_fee_rate > rust_decimal::Decimal::ONE {
            return Err(PaymentMethodError::InvalidConfiguration(
                "Processing fee rate must be between 0 and 1".to_string(),
            ));
        }

        self.configuration = config;
        Ok(())
    }
}

/// Payment method types
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PaymentMethodType {
    /// Cryptocurrency payment (requires network)
    Crypto,
    /// Traditional bank transfer
    BankTransfer,
    /// Credit/debit card payment
    CreditCard,
}

impl std::str::FromStr for PaymentMethodType {
    type Err = ValueObjectError;
    
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "crypto" | "cryptocurrency" => Ok(PaymentMethodType::Crypto),
            "bank" | "banktransfer" | "bank_transfer" => Ok(PaymentMethodType::BankTransfer),
            "card" | "creditcard" | "credit_card" => Ok(PaymentMethodType::CreditCard),
            _ => Err(ValueObjectError::InvalidFormat(format!("Unknown payment method type: {}", s))),
        }
    }
}

impl PaymentMethodType {
    /// Check if this method supports a currency
    pub fn supports_currency(&self, currency: &Currency) -> bool {
        match self {
            PaymentMethodType::Crypto => currency.is_crypto(),
            PaymentMethodType::BankTransfer => !currency.is_crypto(),
            PaymentMethodType::CreditCard => !currency.is_crypto(),
        }
    }

    /// Get supported currencies for this method
    pub fn supported_currencies(&self) -> Vec<Currency> {
        match self {
            PaymentMethodType::Crypto => vec![
                Currency::USDT, Currency::USDC, Currency::ETH, 
                Currency::BTC, Currency::BNB, Currency::TRX
            ],
            PaymentMethodType::BankTransfer | PaymentMethodType::CreditCard => vec![
                Currency::USD
            ],
        }
    }

    /// Check if method requires network specification
    pub fn requires_network(&self) -> bool {
        matches!(self, PaymentMethodType::Crypto)
    }

    /// Get default processing time in seconds
    pub fn default_processing_time(&self) -> u32 {
        match self {
            PaymentMethodType::Crypto => 600,        // 10 minutes (depends on network)
            PaymentMethodType::BankTransfer => 86400, // 1 day
            PaymentMethodType::CreditCard => 30,     // 30 seconds
        }
    }
}

impl Display for PaymentMethodType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PaymentMethodType::Crypto => write!(f, "Cryptocurrency"),
            PaymentMethodType::BankTransfer => write!(f, "Bank Transfer"),
            PaymentMethodType::CreditCard => write!(f, "Credit Card"),
        }
    }
}

/// Payment method configuration
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PaymentMethodConfig {
    pub is_enabled: bool,
    pub processing_time_seconds: u32,
    pub processing_fee_rate: rust_decimal::Decimal,
    pub minimum_amount: rust_decimal::Decimal,
    pub maximum_amount: rust_decimal::Decimal,
    pub additional_settings: HashMap<String, String>,
}

impl PaymentMethodConfig {
    /// Create default configuration for method and currency
    pub fn default_for_method(method_type: &PaymentMethodType, currency: &Currency) -> Self {
        use rust_decimal::Decimal;
        use rust_decimal_macros::dec;

        let (min_amount, max_amount, fee_rate) = match (method_type, currency) {
            // Crypto payments
            (PaymentMethodType::Crypto, Currency::USDT) => (dec!(10), dec!(100000), dec!(0.02)),
            (PaymentMethodType::Crypto, Currency::USDC) => (dec!(10), dec!(100000), dec!(0.02)),
            (PaymentMethodType::Crypto, Currency::ETH) => (dec!(0.01), dec!(100), dec!(0.025)),
            (PaymentMethodType::Crypto, Currency::BTC) => (dec!(0.001), dec!(10), dec!(0.015)),
            (PaymentMethodType::Crypto, Currency::BNB) => (dec!(0.1), dec!(1000), dec!(0.02)),
            (PaymentMethodType::Crypto, Currency::TRX) => (dec!(100), dec!(1000000), dec!(0.03)),
            
            // Fiat payments
            (PaymentMethodType::BankTransfer, Currency::USD) => (dec!(50), dec!(50000), dec!(0.01)),
            (PaymentMethodType::CreditCard, Currency::USD) => (dec!(10), dec!(10000), dec!(0.035)),
            
            // Fallback
            _ => (dec!(1), dec!(10000), dec!(0.03)),
        };

        Self {
            is_enabled: true,
            processing_time_seconds: method_type.default_processing_time(),
            processing_fee_rate: fee_rate,
            minimum_amount: min_amount,
            maximum_amount: max_amount,
            additional_settings: HashMap::new(),
        }
    }

    /// Disable this payment method
    pub fn disable(&mut self) {
        self.is_enabled = false;
    }

    /// Enable this payment method
    pub fn enable(&mut self) {
        self.is_enabled = true;
    }

    /// Set additional setting
    pub fn set_setting(&mut self, key: String, value: String) {
        self.additional_settings.insert(key, value);
    }

    /// Get additional setting
    pub fn get_setting(&self, key: &str) -> Option<&str> {
        self.additional_settings.get(key).map(|s| s.as_str())
    }
}

/// Payment instructions for users
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum PaymentInstructions {
    Crypto {
        currency: Currency,
        network: Network,
        estimated_confirmations: u32,
    },
    BankTransfer {
        currency: Currency,
        processing_days: std::ops::RangeInclusive<u32>,
    },
    CreditCard {
        currency: Currency,
        instant_processing: bool,
    },
}

impl PaymentInstructions {
    /// Get user-friendly instructions text
    pub fn instructions_text(&self) -> String {
        match self {
            PaymentInstructions::Crypto { currency, network, estimated_confirmations } => {
                format!(
                    "Send {} via {} network. Payment will be confirmed after {} network confirmations. \
                     Please ensure you're using the correct network to avoid loss of funds.",
                    currency.symbol(),
                    network.name(),
                    estimated_confirmations
                )
            }
            PaymentInstructions::BankTransfer { currency, processing_days } => {
                format!(
                    "Bank transfer in {}. Processing typically takes {:?} business days. \
                     Please include the payment reference in your transfer.",
                    currency.symbol(),
                    processing_days
                )
            }
            PaymentInstructions::CreditCard { currency, instant_processing } => {
                if *instant_processing {
                    format!("Credit/debit card payment in {}. Payment is processed instantly.", currency.symbol())
                } else {
                    format!("Credit/debit card payment in {}. Payment may take a few minutes to process.", currency.symbol())
                }
            }
        }
    }

    /// Get estimated processing time in seconds
    pub fn estimated_processing_time(&self) -> u32 {
        match self {
            PaymentInstructions::Crypto { network, estimated_confirmations, .. } => {
                let block_time = match network {
                    Network::Ethereum => 15,     // ~15 seconds per block
                    Network::Binance => 3,       // ~3 seconds per block
                    Network::Tron => 3,          // ~3 seconds per block
                    Network::Arbitrum => 1,      // ~1 second per block
                    Network::Polygon => 2,       // ~2 seconds per block
                };
                block_time * estimated_confirmations
            }
            PaymentInstructions::BankTransfer { processing_days, .. } => {
                // Convert days to seconds (use average of range)
                let avg_days = (*processing_days.start() + *processing_days.end()) / 2;
                avg_days * 24 * 60 * 60
            }
            PaymentInstructions::CreditCard { .. } => 30, // 30 seconds
        }
    }
}

impl Display for PaymentMethod {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.display_name())
    }
}

#[derive(Debug, thiserror::Error)]
pub enum PaymentMethodError {
    #[error("{method} does not support currency {currency}")]
    UnsupportedCurrency {
        method: PaymentMethodType,
        currency: Currency,
    },

    #[error("Currency {currency} not supported on network {network}")]
    UnsupportedNetwork {
        currency: Currency,
        network: Network,
    },

    #[error("{0} requires network specification")]
    NetworkRequired(PaymentMethodType),

    #[error("{0} does not allow network specification")]
    NetworkNotAllowed(PaymentMethodType),

    #[error("Invalid configuration: {0}")]
    InvalidConfiguration(String),

    #[error("Payment method is disabled")]
    MethodDisabled,

    #[error("Amount {amount} outside valid range [{min}, {max}]")]
    AmountOutOfRange {
        amount: rust_decimal::Decimal,
        min: rust_decimal::Decimal,
        max: rust_decimal::Decimal,
    },
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn test_crypto_payment_method() {
        let method = PaymentMethod::new(
            PaymentMethodType::Crypto,
            Currency::USDT,
            Some(Network::Ethereum),
        ).unwrap();

        assert_eq!(method.method_type(), &PaymentMethodType::Crypto);
        assert_eq!(method.currency(), &Currency::USDT);
        assert_eq!(method.network(), Some(&Network::Ethereum));
        assert!(method.is_available());
    }

    #[test]
    fn test_crypto_method_requires_network() {
        let result = PaymentMethod::new(
            PaymentMethodType::Crypto,
            Currency::ETH,
            None, // Missing network
        );
        
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), PaymentMethodError::NetworkRequired(_)));
    }

    #[test]
    fn test_fiat_method_no_network() {
        let method = PaymentMethod::new(
            PaymentMethodType::CreditCard,
            Currency::USD,
            None,
        ).unwrap();

        assert_eq!(method.method_type(), &PaymentMethodType::CreditCard);
        assert_eq!(method.currency(), &Currency::USD);
        assert_eq!(method.network(), None);
    }

    #[test]
    fn test_fiat_method_with_network_fails() {
        let result = PaymentMethod::new(
            PaymentMethodType::CreditCard,
            Currency::USD,
            Some(Network::Ethereum), // Not allowed
        );
        
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), PaymentMethodError::NetworkNotAllowed(_)));
    }

    #[test]
    fn test_unsupported_currency() {
        // Trying to use crypto currency with bank transfer
        let result = PaymentMethod::new(
            PaymentMethodType::BankTransfer,
            Currency::ETH,
            None,
        );
        
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), PaymentMethodError::UnsupportedCurrency { .. }));
    }

    #[test]
    fn test_currency_network_compatibility() {
        // BTC doesn't support Ethereum network
        let result = PaymentMethod::new(
            PaymentMethodType::Crypto,
            Currency::BTC,
            Some(Network::Ethereum),
        );
        
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), PaymentMethodError::UnsupportedNetwork { .. }));
    }

    #[test]
    fn test_payment_method_configuration() {
        let method = PaymentMethod::new(
            PaymentMethodType::Crypto,
            Currency::USDT,
            Some(Network::Ethereum),
        ).unwrap();

        let config = method.configuration();
        assert!(config.is_enabled);
        assert!(config.minimum_amount > rust_decimal::Decimal::ZERO);
        assert!(config.maximum_amount > config.minimum_amount);
        assert!(config.processing_fee_rate >= rust_decimal::Decimal::ZERO);
    }

    #[test]
    fn test_amount_validation() {
        let method = PaymentMethod::new(
            PaymentMethodType::Crypto,
            Currency::USDT,
            Some(Network::Ethereum),
        ).unwrap();

        assert!(method.is_amount_valid(dec!(50))); // Within range
        assert!(!method.is_amount_valid(dec!(1))); // Below minimum
        assert!(!method.is_amount_valid(dec!(1000000))); // Above maximum
    }

    #[test]
    fn test_display_name() {
        let crypto_method = PaymentMethod::new(
            PaymentMethodType::Crypto,
            Currency::USDT,
            Some(Network::Ethereum),
        ).unwrap();
        assert_eq!(crypto_method.display_name(), "USDT (ETH)");

        let card_method = PaymentMethod::new(
            PaymentMethodType::CreditCard,
            Currency::USD,
            None,
        ).unwrap();
        assert_eq!(card_method.display_name(), "Credit Card (USD)");
    }

    #[test]
    fn test_payment_instructions() {
        let method = PaymentMethod::new(
            PaymentMethodType::Crypto,
            Currency::USDT,
            Some(Network::Ethereum),
        ).unwrap();

        let instructions = method.get_instructions();
        match instructions {
            PaymentInstructions::Crypto { currency, network, estimated_confirmations } => {
                assert_eq!(currency, Currency::USDT);
                assert_eq!(network, Network::Ethereum);
                assert_eq!(estimated_confirmations, 12);
            }
            _ => panic!("Expected crypto instructions"),
        }

        let text = instructions.instructions_text();
        assert!(text.contains("USDT"));
        assert!(text.contains("Ethereum"));
        assert!(text.contains("12"));
    }

    #[test]
    fn test_config_validation() {
        let mut method = PaymentMethod::new(
            PaymentMethodType::Crypto,
            Currency::USDT,
            Some(Network::Ethereum),
        ).unwrap();

        // Invalid config: min >= max
        let invalid_config = PaymentMethodConfig {
            is_enabled: true,
            processing_time_seconds: 600,
            processing_fee_rate: dec!(0.02),
            minimum_amount: dec!(100),
            maximum_amount: dec!(50), // Less than minimum
            additional_settings: HashMap::new(),
        };

        let result = method.update_config(invalid_config);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), PaymentMethodError::InvalidConfiguration(_)));
    }

    #[test]
    fn test_method_type_currency_support() {
        assert!(PaymentMethodType::Crypto.supports_currency(&Currency::ETH));
        assert!(!PaymentMethodType::Crypto.supports_currency(&Currency::USD));
        
        assert!(PaymentMethodType::CreditCard.supports_currency(&Currency::USD));
        assert!(!PaymentMethodType::CreditCard.supports_currency(&Currency::ETH));
    }

    #[test]
    fn test_processing_time_estimation() {
        let crypto_instructions = PaymentInstructions::Crypto {
            currency: Currency::ETH,
            network: Network::Ethereum,
            estimated_confirmations: 12,
        };
        
        let processing_time = crypto_instructions.estimated_processing_time();
        assert_eq!(processing_time, 15 * 12); // 15 seconds * 12 confirmations

        let card_instructions = PaymentInstructions::CreditCard {
            currency: Currency::USD,
            instant_processing: true,
        };
        
        let card_time = card_instructions.estimated_processing_time();
        assert_eq!(card_time, 30); // 30 seconds
    }
}