// Blockchain RPC configuration for Web3 validation

use std::collections::HashMap;

/// Configuration for blockchain RPC connections
#[derive(Debug, Clone)]
pub struct BlockchainCfg {
    /// RPC endpoints for different chains
    pub rpc_endpoints: HashMap<u64, String>,
    /// Request timeout for blockchain queries (milliseconds)
    pub request_timeout_ms: u64,
    /// Maximum retries for failed requests
    pub max_retries: u32,
    /// Rate limiting (requests per second)
    pub rate_limit_per_second: u32,
}

impl Default for BlockchainCfg {
    fn default() -> Self {
        let mut rpc_endpoints = HashMap::new();
        rpc_endpoints.insert(1, "https://eth-mainnet.g.alchemy.com/v2/YOUR_KEY".to_string());
        rpc_endpoints.insert(56, "https://bsc-dataseed.binance.org".to_string());
        rpc_endpoints.insert(137, "https://polygon-rpc.com".to_string());

        Self {
            rpc_endpoints,
            request_timeout_ms: 30000, // 30 seconds
            max_retries: 3,
            rate_limit_per_second: 10,
        }
    }
}
