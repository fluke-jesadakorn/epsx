// kernel extraction wave9 — moved from apps/backend/src/domain/shared_kernel/value_objects/common_types.rs
// Import-path adjustment: `crate::domain::shared_kernel::value_object::*` →
// `crate::value_object::*` (sibling module in this crate).
use serde::{Deserialize, Serialize};
use crate::value_object::{ValueObject, ValueObjectError};

/// Payment identifier value object
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PayId(String);

impl PayId {
    pub fn new(id: String) -> Self {
        Self(id)
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl ValueObject for PayId {
    type Error = ValueObjectError;
    
    fn validate(&self) -> Result<(), Self::Error> {
        if self.0.is_empty() {
            return Err(ValueObjectError::Required("Payment ID cannot be empty".to_string()));
        }
        Ok(())
    }
}


/// Currency enumeration
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Currency {
    USD,
    EUR,
    BTC,
    ETH,
    USDT,
    USDC,
    BNB,
    TRX,
}

impl Currency {
    /// Get currency symbol
    pub fn symbol(&self) -> &'static str {
        match self {
            Currency::USD => "USD",
            Currency::EUR => "EUR", 
            Currency::BTC => "BTC",
            Currency::ETH => "ETH",
            Currency::USDT => "USDT",
            Currency::USDC => "USDC",
            Currency::BNB => "BNB",
            Currency::TRX => "TRX",
        }
    }

    /// Get number of decimal places for the currency
    pub fn decimals(&self) -> u8 {
        match self {
            Currency::USD | Currency::EUR => 2,
            Currency::BTC => 8,
            Currency::ETH => 18,
            Currency::USDT | Currency::USDC => 6,
            Currency::BNB => 18,
            Currency::TRX => 6,
        }
    }

    /// Get supported networks for the currency
    pub fn supported_networks(&self) -> Vec<Network> {
        match self {
            Currency::USD | Currency::EUR => vec![], // Fiat currencies
            Currency::BTC => vec![Network::Bitcoin],
            Currency::ETH => vec![Network::Ethereum, Network::Arbitrum, Network::Polygon],
            Currency::USDT | Currency::USDC => vec![
                Network::Ethereum, 
                Network::Binance, 
                Network::Tron, 
                Network::Arbitrum, 
                Network::Polygon
            ],
            Currency::BNB => vec![Network::Binance],
            Currency::TRX => vec![Network::Tron],
        }
    }
}

impl ValueObject for Currency {
    type Error = ValueObjectError;
    
    fn validate(&self) -> Result<(), Self::Error> {
        // All enum variants are valid by definition
        Ok(())
    }
}

/// Payment status enumeration  
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PayStatus {
    Pending,
    Confirmed,
    Failed,
    Cancelled,
}

impl ValueObject for PayStatus {
    type Error = ValueObjectError;
    
    fn validate(&self) -> Result<(), Self::Error> {
        // All enum variants are valid by definition
        Ok(())
    }
}

/// Network enumeration for crypto payments
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Network {
    Bitcoin,
    Ethereum,
    Polygon,
    BinanceSmartChain,
    Binance,
    Tron,
    Arbitrum,
}

impl Network {
    /// Get short name for the network
    pub fn short_name(&self) -> &'static str {
        match self {
            Network::Bitcoin => "BTC",
            Network::Ethereum => "ETH",
            Network::Polygon => "MATIC",
            Network::BinanceSmartChain | Network::Binance => "BSC",
            Network::Tron => "TRX",
            Network::Arbitrum => "ARB",
        }
    }
}

impl ValueObject for Network {
    type Error = ValueObjectError;
    
    fn validate(&self) -> Result<(), Self::Error> {
        // All enum variants are valid by definition
        Ok(())
    }
}
