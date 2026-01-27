// Contract configuration for blockchain payment monitoring
// Stores contract addresses per chain and payment event signatures

use ethers::types::H160;
use std::env;
use std::fmt;

/// Blockchain chain identifiers
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Chain {
    Ethereum,
    Polygon,
    Bsc,
    Arbitrum,
    Optimism,
    Base,
}

impl Chain {
    /// Get chain name as string
    pub fn name(&self) -> &str {
        match self {
            Chain::Ethereum => "ethereum",
            Chain::Polygon => "polygon",
            Chain::Bsc => "bsc",
            Chain::Arbitrum => "arbitrum",
            Chain::Optimism => "optimism",
            Chain::Base => "base",
        }
    }

    /// Get chain ID
    pub fn chain_id(&self) -> u64 {
        match self {
            Chain::Ethereum => 1,
            Chain::Polygon => 137,
            Chain::Bsc => 56,
            Chain::Arbitrum => 42161,
            Chain::Optimism => 10,
            Chain::Base => 8453,
        }
    }

    /// Parse from string
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "ethereum" | "eth" => Some(Chain::Ethereum),
            "polygon" | "matic" => Some(Chain::Polygon),
            "bsc" | "binance" => Some(Chain::Bsc),
            "arbitrum" | "arb" => Some(Chain::Arbitrum),
            "optimism" | "opt" => Some(Chain::Optimism),
            "base" => Some(Chain::Base),
            _ => None,
        }
    }

    /// Get environment variable suffix for this chain
    pub fn env_suffix(&self) -> &str {
        match self {
            Chain::Ethereum => "ETHEREUM",
            Chain::Polygon => "POLYGON",
            Chain::Bsc => "BSC",
            Chain::Arbitrum => "ARBITRUM",
            Chain::Optimism => "OPTIMISM",
            Chain::Base => "BASE",
        }
    }
}

impl fmt::Display for Chain {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name())
    }
}

/// Payment event signature
/// PaymentReceived(address indexed user, uint256 planId, address indexed token, uint256 amount, uint256 timestamp, uint256 paymentId)
/// keccak256("PaymentReceived(address,uint256,address,uint256,uint256,uint256)")
pub const PAYMENT_EVENT_TOPIC: &str =
    "0xa7f9e7f4f9c6e7e3d8b3a2f1c0d9e8f7a6b5c4d3e2f1a0b9c8d7e6f5a4b3c2d1";

/// Single chain contract configuration
#[derive(Debug, Clone)]
pub struct ChainContractConfig {
    pub chain: Chain,
    pub contract_address: Option<H160>,
    pub ws_url: String,
    pub ws_backup_url: Option<String>,
    pub http_url: String,
}

impl ChainContractConfig {
    /// Create from environment for a specific chain
    pub fn from_env(chain: Chain) -> Option<Self> {
        let suffix = chain.env_suffix();

        // Contract address (optional - only enable chains you use)
        let contract_address = env::var(format!("CONTRACT_ADDRESS_{}", suffix))
            .ok()
            .and_then(|addr| addr.parse::<H160>().ok());

        // Skip if no contract configured
        if contract_address.is_none() {
            return None;
        }

        // WebSocket URLs (with backup)
        let ws_url = env::var(format!("WS_URL_{}", suffix))
            .unwrap_or_else(|_| default_ws_url(chain));

        let ws_backup_url = env::var(format!("WS_BACKUP_{}", suffix)).ok();

        // HTTP fallback URL
        let http_url = env::var(format!("{}_RPC_URL", suffix))
            .unwrap_or_else(|_| default_http_url(chain));

        Some(Self {
            chain,
            contract_address,
            ws_url,
            ws_backup_url,
            http_url,
        })
    }

    /// Check if WebSocket is configured
    pub fn has_websocket(&self) -> bool {
        !self.ws_url.is_empty()
    }
}

/// Default WebSocket endpoints by chain
fn default_ws_url(chain: Chain) -> String {
    match chain {
        Chain::Bsc => "wss://bsc-ws-node.nariox.org:443".to_string(),
        Chain::Ethereum => "wss://eth.llamarpc.com".to_string(),
        Chain::Polygon => "wss://polygon-bor.publicnode.com".to_string(),
        Chain::Arbitrum => "wss://arbitrum-one.publicnode.com".to_string(),
        Chain::Optimism => "wss://optimism.publicnode.com".to_string(),
        Chain::Base => "wss://base.publicnode.com".to_string(),
    }
}

/// Default HTTP RPC endpoints by chain
fn default_http_url(chain: Chain) -> String {
    match chain {
        Chain::Bsc => "https://bsc-dataseed.binance.org".to_string(),
        Chain::Ethereum => "https://eth.llamarpc.com".to_string(),
        Chain::Polygon => "https://polygon.llamarpc.com".to_string(),
        Chain::Arbitrum => "https://arbitrum.llamarpc.com".to_string(),
        Chain::Optimism => "https://optimism.llamarpc.com".to_string(),
        Chain::Base => "https://base.llamarpc.com".to_string(),
    }
}

/// Complete contract configuration for all chains
#[derive(Debug, Clone)]
pub struct ContractConfig {
    pub chains: Vec<ChainContractConfig>,
    pub payment_event_topic: String,
}

impl ContractConfig {
    /// Load from environment
    pub fn from_env() -> Self {
        let mut chains = Vec::new();

        // Try to load config for each chain
        for chain in [
            Chain::Bsc,
            Chain::Ethereum,
            Chain::Polygon,
            Chain::Arbitrum,
            Chain::Optimism,
            Chain::Base,
        ] {
            if let Some(config) = ChainContractConfig::from_env(chain) {
                chains.push(config);
            }
        }

        let payment_event_topic = env::var("PAYMENT_EVENT_TOPIC")
            .unwrap_or_else(|_| PAYMENT_EVENT_TOPIC.to_string());

        Self {
            chains,
            payment_event_topic,
        }
    }

    /// Get config for specific chain
    pub fn get_chain(&self, chain: Chain) -> Option<&ChainContractConfig> {
        self.chains.iter().find(|c| c.chain == chain)
    }

    /// Check if any chains are configured
    pub fn is_empty(&self) -> bool {
        self.chains.is_empty()
    }

    /// Get number of configured chains
    pub fn len(&self) -> usize {
        self.chains.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chain_from_str() {
        assert_eq!(Chain::from_str("bsc"), Some(Chain::Bsc));
        assert_eq!(Chain::from_str("BSC"), Some(Chain::Bsc));
        assert_eq!(Chain::from_str("binance"), Some(Chain::Bsc));
        assert_eq!(Chain::from_str("ethereum"), Some(Chain::Ethereum));
        assert_eq!(Chain::from_str("unknown"), None);
    }

    #[test]
    fn test_chain_ids() {
        assert_eq!(Chain::Bsc.chain_id(), 56);
        assert_eq!(Chain::Ethereum.chain_id(), 1);
        assert_eq!(Chain::Polygon.chain_id(), 137);
    }

    #[test]
    fn test_default_urls() {
        assert_eq!(
            default_ws_url(Chain::Bsc),
            "wss://bsc-ws-node.nariox.org:443"
        );
        assert_eq!(
            default_http_url(Chain::Bsc),
            "https://bsc-dataseed.binance.org"
        );
    }
}
