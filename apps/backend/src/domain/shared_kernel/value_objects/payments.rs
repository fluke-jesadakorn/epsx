// Payment-related shared value objects

use serde::{Deserialize, Serialize};
use std::fmt::{self, Display};

/// Supported cryptocurrencies
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Currency {
    Bitcoin,
    Ethereum,
    Usdt,
    Usdc,
    Bnb,
    USD,
    // Uppercase variants for compatibility
    USDT,
    USDC,
    ETH,
    BTC,
    BNB,
    TRX,
}

impl Display for Currency {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Currency::Bitcoin => write!(f, "BTC"),
            Currency::Ethereum => write!(f, "ETH"),
            Currency::Usdt => write!(f, "USDT"),
            Currency::Usdc => write!(f, "USDC"),
            Currency::Bnb => write!(f, "BNB"),
            Currency::USD => write!(f, "USD"),
            Currency::USDT => write!(f, "USDT"),
            Currency::USDC => write!(f, "USDC"),
            Currency::ETH => write!(f, "ETH"),
            Currency::BTC => write!(f, "BTC"),
            Currency::BNB => write!(f, "BNB"),
            Currency::TRX => write!(f, "TRX"),
        }
    }
}

/// Supported blockchain networks
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Network {
    Bitcoin,
    Ethereum,
    BinanceSmartChain,
    Polygon,
    Arbitrum,
    Tron,
    Binance,
}

impl Display for Network {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Network::Bitcoin => write!(f, "bitcoin"),
            Network::Ethereum => write!(f, "ethereum"),
            Network::BinanceSmartChain => write!(f, "bsc"),
            Network::Polygon => write!(f, "polygon"),
            Network::Arbitrum => write!(f, "arbitrum"),
            Network::Tron => write!(f, "tron"),
            Network::Binance => write!(f, "binance"),
        }
    }
}

impl Network {
    /// Get short name for this network
    pub fn short_name(&self) -> &'static str {
        match self {
            Network::Bitcoin => "bitcoin",
            Network::Ethereum => "ethereum",
            Network::BinanceSmartChain => "bsc",
            Network::Polygon => "polygon",
            Network::Arbitrum => "arbitrum",
            Network::Tron => "tron",
            Network::Binance => "binance",
        }
    }
    
    /// Get full name for this network
    pub fn name(&self) -> &'static str {
        match self {
            Network::Bitcoin => "Bitcoin",
            Network::Ethereum => "Ethereum",
            Network::BinanceSmartChain => "Binance Smart Chain",
            Network::Polygon => "Polygon",
            Network::Arbitrum => "Arbitrum",
            Network::Tron => "Tron",
            Network::Binance => "Binance",
        }
    }
}

impl Currency {
    /// Get supported networks for this currency
    pub fn supported_networks(&self) -> Vec<Network> {
        match self {
            Currency::Bitcoin | Currency::BTC => vec![Network::Bitcoin],
            Currency::Ethereum | Currency::ETH => vec![Network::Ethereum, Network::Arbitrum],
            Currency::Usdt | Currency::Usdc | Currency::USDT | Currency::USDC => vec![
                Network::Ethereum,
                Network::BinanceSmartChain,
                Network::Polygon,
                Network::Arbitrum,
            ],
            Currency::Bnb | Currency::BNB => vec![Network::BinanceSmartChain],
            Currency::TRX => vec![Network::Tron],
            Currency::USD => vec![], // USD is not on any blockchain network
        }
    }
    
    /// Get decimal places for this currency
    pub fn decimals(&self) -> u8 {
        match self {
            Currency::Bitcoin | Currency::BTC => 8,
            Currency::Ethereum | Currency::ETH => 18,
            Currency::Usdt | Currency::USDT => 6,
            Currency::Usdc | Currency::USDC => 6,
            Currency::Bnb | Currency::BNB => 18,
            Currency::TRX => 6,
            Currency::USD => 2,
        }
    }
    
    /// Get symbol for this currency
    pub fn symbol(&self) -> &'static str {
        match self {
            Currency::Bitcoin | Currency::BTC => "BTC",
            Currency::Ethereum | Currency::ETH => "ETH", 
            Currency::Usdt | Currency::USDT => "USDT",
            Currency::Usdc | Currency::USDC => "USDC",
            Currency::Bnb | Currency::BNB => "BNB",
            Currency::TRX => "TRX",
            Currency::USD => "USD",
        }
    }
    
    /// Check if this currency is a cryptocurrency
    pub fn is_crypto(&self) -> bool {
        match self {
            Currency::USD => false,
            _ => true, // All other currencies are crypto
        }
    }
}