// Crypto Network Value Object  
// Represents blockchain networks for cryptocurrency transactions

use serde::{Deserialize, Serialize};
use std::fmt::{self, Display};

use crate::domain::shared_kernel::{ValueObject, value_object::ValueObjectError};

/// Supported cryptocurrency networks
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CryptoNetwork {
    /// Bitcoin mainnet
    Bitcoin,
    /// Bitcoin testnet
    BitcoinTestnet,
    /// Ethereum mainnet
    Ethereum,
    /// Ethereum Goerli testnet
    EthereumGoerli,
    /// Ethereum Sepolia testnet  
    EthereumSepolia,
    /// Binance Smart Chain
    BinanceSmartChain,
    /// Binance Smart Chain testnet
    BinanceSmartChainTestnet,
    /// Polygon (Matic) mainnet
    Polygon,
    /// Polygon Mumbai testnet
    PolygonMumbai,
    /// Avalanche C-Chain
    Avalanche,
    /// Avalanche Fuji testnet
    AvalancheFuji,
    /// Arbitrum One
    Arbitrum,
    /// Arbitrum Goerli testnet
    ArbitrumGoerli,
    /// Optimism
    Optimism,
    /// Optimism Goerli testnet
    OptimismGoerli,
    /// Base
    Base,
    /// Base Goerli testnet
    BaseGoerli,
    /// Solana mainnet
    Solana,
    /// Solana devnet
    SolanaDevnet,
    /// Solana testnet
    SolanaTestnet,
    /// Custom/Unknown network
    Custom(String),
}

impl CryptoNetwork {
    /// Create from string representation
    pub fn from_string(network: &str) -> Result<Self, CryptoNetworkError> {
        let network_lower = network.to_lowercase();
        
        match network_lower.as_str() {
            "bitcoin" | "btc" => Ok(CryptoNetwork::Bitcoin),
            "bitcoin_testnet" | "btc_testnet" => Ok(CryptoNetwork::BitcoinTestnet),
            "ethereum" | "eth" => Ok(CryptoNetwork::Ethereum),
            "ethereum_goerli" | "eth_goerli" | "goerli" => Ok(CryptoNetwork::EthereumGoerli),
            "ethereum_sepolia" | "eth_sepolia" | "sepolia" => Ok(CryptoNetwork::EthereumSepolia),
            "binance_smart_chain" | "bsc" => Ok(CryptoNetwork::BinanceSmartChain),
            "binance_smart_chain_testnet" | "bsc_testnet" => Ok(CryptoNetwork::BinanceSmartChainTestnet),
            "polygon" | "matic" => Ok(CryptoNetwork::Polygon),
            "polygon_mumbai" | "mumbai" => Ok(CryptoNetwork::PolygonMumbai),
            "avalanche" | "avax" => Ok(CryptoNetwork::Avalanche),
            "avalanche_fuji" | "fuji" => Ok(CryptoNetwork::AvalancheFuji),
            "arbitrum" | "arb" => Ok(CryptoNetwork::Arbitrum),
            "arbitrum_goerli" | "arb_goerli" => Ok(CryptoNetwork::ArbitrumGoerli),
            "optimism" | "op" => Ok(CryptoNetwork::Optimism),
            "optimism_goerli" | "op_goerli" => Ok(CryptoNetwork::OptimismGoerli),
            "base" => Ok(CryptoNetwork::Base),
            "base_goerli" => Ok(CryptoNetwork::BaseGoerli),
            "solana" | "sol" => Ok(CryptoNetwork::Solana),
            "solana_devnet" | "sol_devnet" => Ok(CryptoNetwork::SolanaDevnet),
            "solana_testnet" | "sol_testnet" => Ok(CryptoNetwork::SolanaTestnet),
            _ => {
                if network.trim().is_empty() {
                    Err(CryptoNetworkError::EmptyNetwork)
                } else {
                    Ok(CryptoNetwork::Custom(network.to_string()))
                }
            }
        }
    }
    
    /// Check if this is a testnet
    pub fn is_testnet(&self) -> bool {
        match self {
            CryptoNetwork::Bitcoin => false,
            CryptoNetwork::BitcoinTestnet => true,
            CryptoNetwork::Ethereum => false,
            CryptoNetwork::EthereumGoerli => true,
            CryptoNetwork::EthereumSepolia => true,
            CryptoNetwork::BinanceSmartChain => false,
            CryptoNetwork::BinanceSmartChainTestnet => true,
            CryptoNetwork::Polygon => false,
            CryptoNetwork::PolygonMumbai => true,
            CryptoNetwork::Avalanche => false,
            CryptoNetwork::AvalancheFuji => true,
            CryptoNetwork::Arbitrum => false,
            CryptoNetwork::ArbitrumGoerli => true,
            CryptoNetwork::Optimism => false,
            CryptoNetwork::OptimismGoerli => true,
            CryptoNetwork::Base => false,
            CryptoNetwork::BaseGoerli => true,
            CryptoNetwork::Solana => false,
            CryptoNetwork::SolanaDevnet => true,
            CryptoNetwork::SolanaTestnet => true,
            CryptoNetwork::Custom(_) => false, // Assume mainnet by default
        }
    }
    
    /// Get the chain ID for EVM-compatible networks
    pub fn chain_id(&self) -> Option<u64> {
        match self {
            CryptoNetwork::Ethereum => Some(1),
            CryptoNetwork::EthereumGoerli => Some(5),
            CryptoNetwork::EthereumSepolia => Some(11155111),
            CryptoNetwork::BinanceSmartChain => Some(56),
            CryptoNetwork::BinanceSmartChainTestnet => Some(97),
            CryptoNetwork::Polygon => Some(137),
            CryptoNetwork::PolygonMumbai => Some(80001),
            CryptoNetwork::Avalanche => Some(43114),
            CryptoNetwork::AvalancheFuji => Some(43113),
            CryptoNetwork::Arbitrum => Some(42161),
            CryptoNetwork::ArbitrumGoerli => Some(421613),
            CryptoNetwork::Optimism => Some(10),
            CryptoNetwork::OptimismGoerli => Some(420),
            CryptoNetwork::Base => Some(8453),
            CryptoNetwork::BaseGoerli => Some(84531),
            _ => None, // Non-EVM networks or custom networks
        }
    }
    
    /// Get the native currency symbol
    pub fn native_currency(&self) -> &str {
        match self {
            CryptoNetwork::Bitcoin | CryptoNetwork::BitcoinTestnet => "BTC",
            CryptoNetwork::Ethereum | CryptoNetwork::EthereumGoerli | CryptoNetwork::EthereumSepolia => "ETH",
            CryptoNetwork::BinanceSmartChain | CryptoNetwork::BinanceSmartChainTestnet => "BNB",
            CryptoNetwork::Polygon | CryptoNetwork::PolygonMumbai => "MATIC",
            CryptoNetwork::Avalanche | CryptoNetwork::AvalancheFuji => "AVAX",
            CryptoNetwork::Arbitrum | CryptoNetwork::ArbitrumGoerli => "ETH",
            CryptoNetwork::Optimism | CryptoNetwork::OptimismGoerli => "ETH",
            CryptoNetwork::Base | CryptoNetwork::BaseGoerli => "ETH",
            CryptoNetwork::Solana | CryptoNetwork::SolanaDevnet | CryptoNetwork::SolanaTestnet => "SOL",
            CryptoNetwork::Custom(_) => "UNKNOWN",
        }
    }
    
    /// Get network display name
    pub fn display_name(&self) -> &str {
        match self {
            CryptoNetwork::Bitcoin => "Bitcoin",
            CryptoNetwork::BitcoinTestnet => "Bitcoin Testnet",
            CryptoNetwork::Ethereum => "Ethereum",
            CryptoNetwork::EthereumGoerli => "Ethereum Goerli",
            CryptoNetwork::EthereumSepolia => "Ethereum Sepolia",
            CryptoNetwork::BinanceSmartChain => "Binance Smart Chain",
            CryptoNetwork::BinanceSmartChainTestnet => "BSC Testnet",
            CryptoNetwork::Polygon => "Polygon",
            CryptoNetwork::PolygonMumbai => "Polygon Mumbai",
            CryptoNetwork::Avalanche => "Avalanche",
            CryptoNetwork::AvalancheFuji => "Avalanche Fuji",
            CryptoNetwork::Arbitrum => "Arbitrum One",
            CryptoNetwork::ArbitrumGoerli => "Arbitrum Goerli",
            CryptoNetwork::Optimism => "Optimism",
            CryptoNetwork::OptimismGoerli => "Optimism Goerli",
            CryptoNetwork::Base => "Base",
            CryptoNetwork::BaseGoerli => "Base Goerli",
            CryptoNetwork::Solana => "Solana",
            CryptoNetwork::SolanaDevnet => "Solana Devnet",
            CryptoNetwork::SolanaTestnet => "Solana Testnet",
            CryptoNetwork::Custom(name) => name,
        }
    }
}

impl ValueObject for CryptoNetwork {
    type Error = CryptoNetworkError;
    
    fn validate(&self) -> Result<(), Self::Error> {
        match self {
            CryptoNetwork::Custom(name) => {
                if name.trim().is_empty() {
                    return Err(CryptoNetworkError::EmptyNetwork);
                }
                if name.len() > 100 {
                    return Err(CryptoNetworkError::NetworkNameTooLong);
                }
            }
            _ => {} // Predefined networks are always valid
        }
        Ok(())
    }
}

impl Display for CryptoNetwork {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.display_name())
    }
}

impl std::str::FromStr for CryptoNetwork {
    type Err = CryptoNetworkError;
    
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::from_string(s)
    }
}

/// Errors that can occur when working with crypto networks
#[derive(Debug, thiserror::Error)]
pub enum CryptoNetworkError {
    #[error("Network name cannot be empty")]
    EmptyNetwork,
    
    #[error("Network name too long (max 100 characters)")]
    NetworkNameTooLong,
    
    #[error("Unsupported network: {0}")]
    UnsupportedNetwork(String),
    
    #[error("Network validation failed: {0}")]
    ValidationFailed(String),
}

impl From<CryptoNetworkError> for ValueObjectError {
    fn from(error: CryptoNetworkError) -> Self {
        ValueObjectError::ValidationFailed(error.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_network_from_string() {
        assert_eq!(CryptoNetwork::from_string("ethereum").unwrap(), CryptoNetwork::Ethereum);
        assert_eq!(CryptoNetwork::from_string("btc").unwrap(), CryptoNetwork::Bitcoin);
        assert_eq!(CryptoNetwork::from_string("polygon").unwrap(), CryptoNetwork::Polygon);
    }
    
    #[test]
    fn test_testnet_detection() {
        assert!(!CryptoNetwork::Bitcoin.is_testnet());
        assert!(CryptoNetwork::BitcoinTestnet.is_testnet());
        assert!(!CryptoNetwork::Ethereum.is_testnet());
        assert!(CryptoNetwork::EthereumGoerli.is_testnet());
    }
    
    #[test]
    fn test_chain_id() {
        assert_eq!(CryptoNetwork::Ethereum.chain_id(), Some(1));
        assert_eq!(CryptoNetwork::Polygon.chain_id(), Some(137));
        assert_eq!(CryptoNetwork::Bitcoin.chain_id(), None);
    }
    
    #[test]
    fn test_native_currency() {
        assert_eq!(CryptoNetwork::Bitcoin.native_currency(), "BTC");
        assert_eq!(CryptoNetwork::Ethereum.native_currency(), "ETH");
        assert_eq!(CryptoNetwork::Polygon.native_currency(), "MATIC");
    }
    
    #[test]
    fn test_custom_network() {
        let custom = CryptoNetwork::Custom("MyCustomNetwork".to_string());
        assert!(custom.is_valid());
        assert_eq!(custom.display_name(), "MyCustomNetwork");
        assert!(!custom.is_testnet());
    }
    
    #[test]
    fn test_empty_network_error() {
        let result = CryptoNetwork::from_string("");
        assert!(result.is_err());
        
        let custom = CryptoNetwork::Custom("".to_string());
        assert!(!custom.is_valid());
    }
}