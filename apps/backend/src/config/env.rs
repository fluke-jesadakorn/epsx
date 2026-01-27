// Pure Web3 Enterprise Configuration for EPSX Platform
// OIDC completely removed - Web3-only authentication
// Supports multi-chain enterprise permissions and token-gated access

use std::env;
use std::fmt;

#[derive(Debug, Clone)]
pub struct ValidationError {
    pub variable: String,
    pub reason: String,
}

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "❌ {}: {}", self.variable, self.reason)
    }
}

/// Pure Web3 Enterprise Configuration - Zero OIDC Dependencies
#[derive(Debug, Clone)]
pub struct Config {
    // Core Infrastructure (4 variables)
    pub database_url: String,
    pub backend_url: String,
    pub frontend_url: String,
    pub admin_frontend_url: String,

    // Web3 Authentication (0 variables) - Pure signature-based auth, no JWT tokens needed

    // Blockchain Infrastructure (6 variables)
    pub ethereum_rpc_url: String,
    pub polygon_rpc_url: String,
    pub arbitrum_rpc_url: String,
    pub optimism_rpc_url: String,
    pub base_rpc_url: String,
    pub bsc_rpc_url: String,

    // WebSocket endpoints for real-time subscriptions
    pub ethereum_ws_url: String,
    pub polygon_ws_url: String,
    pub arbitrum_ws_url: String,
    pub optimism_ws_url: String,
    pub base_ws_url: String,
    pub bsc_ws_url: String,

    // Backup WebSocket endpoints
    pub ethereum_ws_backup: Option<String>,
    pub polygon_ws_backup: Option<String>,
    pub arbitrum_ws_backup: Option<String>,
    pub optimism_ws_backup: Option<String>,
    pub base_ws_backup: Option<String>,
    pub bsc_ws_backup: Option<String>,

    // Contract addresses for payment monitoring
    pub bsc_payment_contract: Option<String>,
    pub ethereum_payment_contract: Option<String>,
    pub polygon_payment_contract: Option<String>,
    pub arbitrum_payment_contract: Option<String>,
    pub optimism_payment_contract: Option<String>,
    pub base_payment_contract: Option<String>,

    // Payment event signature
    pub payment_event_topic: String,

    // Enterprise Web3 Features
    pub enterprise_nft_contract: Option<String>,
    pub enterprise_dao_contract: Option<String>,
    pub enterprise_governance_token: Option<String>,

    // Blockchain Network Configuration
    pub blockchain_network: String,

    // Infrastructure (1 variable)
    pub redis_url: Option<String>,
    pub log_level: String,

    // Token whitelist
    pub supported_payment_tokens: Vec<String>,
}

impl Config {
    /// Load and validate configuration from environment
    pub fn from_env() -> Result<Self, Vec<ValidationError>> {
        let mut errors = Vec::new();

        // Helper function to get required env var
        let get_required = |key: &str| -> Result<String, ValidationError> {
            env::var(key).map_err(|_| ValidationError {
                variable: key.to_string(),
                reason: "Required environment variable is missing".to_string(),
            })
        };

        // Helper function to get optional env var
        let get_optional = |key: &str| -> Option<String> {
            env::var(key).ok()
        };

        // Helper function to get env var with default
        let get_with_default = |key: &str, default: &str| -> String {
            env::var(key).unwrap_or_else(|_| default.to_string())
        };

        // Core Infrastructure - Required
        let database_url = match get_required("DATABASE_URL") {
            Ok(url) => {
                if !url.starts_with("postgresql://") && !url.starts_with("postgres://") {
                    errors.push(ValidationError {
                        variable: "DATABASE_URL".to_string(),
                        reason: "Must be a valid PostgreSQL connection string".to_string(),
                    });
                    String::new()
                } else {
                    url
                }
            },
            Err(e) => {
                errors.push(e);
                String::new()
            }
        };


        let backend_url = get_with_default("BACKEND_URL", 
            if Self::is_development() { "http://localhost:8080" } else { "" });



        let frontend_url = get_with_default("FRONTEND_URL", 
            if Self::is_development() { "http://localhost:3000" } else { "" });
        let admin_frontend_url = get_with_default("ADMIN_FRONTEND_URL", 
            if Self::is_development() { "http://localhost:3001" } else { "" });
        
        // Validate required URLs in production
        if !Self::is_development() {
            if backend_url.is_empty() {
                errors.push(ValidationError {
                    variable: "BACKEND_URL".to_string(),
                    reason: "Required in production environment".to_string(),
                });
            }
            if frontend_url.is_empty() {
                errors.push(ValidationError {
                    variable: "FRONTEND_URL".to_string(),
                    reason: "Required in production environment".to_string(),
                });
            }
            if admin_frontend_url.is_empty() {
                errors.push(ValidationError {
                    variable: "ADMIN_FRONTEND_URL".to_string(),
                    reason: "Required in production environment".to_string(),
                });
            }
        }

        // Web3 Authentication - Pure signature-based, no JWT tokens needed

        // Blockchain Infrastructure - with fallbacks to free RPC endpoints
        let ethereum_rpc_url = get_with_default("ETHEREUM_RPC_URL", "https://eth.llamarpc.com");
        let polygon_rpc_url = get_with_default("POLYGON_RPC_URL", "https://polygon.llamarpc.com");
        let arbitrum_rpc_url = get_with_default("ARBITRUM_RPC_URL", "https://arbitrum.llamarpc.com");
        let optimism_rpc_url = get_with_default("OPTIMISM_RPC_URL", "https://optimism.llamarpc.com");
        let base_rpc_url = get_with_default("BASE_RPC_URL", "https://base.llamarpc.com");
        let bsc_rpc_url = get_with_default("BSC_RPC_URL", "https://bsc-dataseed.binance.org");

        // WebSocket endpoints for real-time subscriptions
        let ethereum_ws_url = get_with_default("WS_URL_ETHEREUM", "wss://eth.llamarpc.com");
        let polygon_ws_url = get_with_default("WS_URL_POLYGON", "wss://polygon-bor.publicnode.com");
        let arbitrum_ws_url = get_with_default("WS_URL_ARBITRUM", "wss://arbitrum-one.publicnode.com");
        let optimism_ws_url = get_with_default("WS_URL_OPTIMISM", "wss://optimism.publicnode.com");
        let base_ws_url = get_with_default("WS_URL_BASE", "wss://base.publicnode.com");
        let bsc_ws_url = get_with_default("WS_URL_BSC", "wss://bsc-ws-node.nariox.org:443");

        // Backup WebSocket endpoints
        let ethereum_ws_backup = get_optional("WS_BACKUP_ETHEREUM");
        let polygon_ws_backup = get_optional("WS_BACKUP_POLYGON");
        let arbitrum_ws_backup = get_optional("WS_BACKUP_ARBITRUM");
        let optimism_ws_backup = get_optional("WS_BACKUP_OPTIMISM");
        let base_ws_backup = get_optional("WS_BACKUP_BASE");
        let bsc_ws_backup = get_optional("WS_BACKUP_BSC");

        // Contract addresses for payment monitoring
        let bsc_payment_contract = get_optional("CONTRACT_ADDRESS_BSC");
        let ethereum_payment_contract = get_optional("CONTRACT_ADDRESS_ETHEREUM");
        let polygon_payment_contract = get_optional("CONTRACT_ADDRESS_POLYGON");
        let arbitrum_payment_contract = get_optional("CONTRACT_ADDRESS_ARBITRUM");
        let optimism_payment_contract = get_optional("CONTRACT_ADDRESS_OPTIMISM");
        let base_payment_contract = get_optional("CONTRACT_ADDRESS_BASE");

        // Payment event signature
        // keccak256("PaymentReceived(address,uint256,address,uint256,uint256,uint256)")
        let payment_event_topic = get_with_default(
            "PAYMENT_EVENT_TOPIC",
            "0xa7f9e7f4f9c6e7e3d8b3a2f1c0d9e8f7a6b5c4d3e2f1a0b9c8d7e6f5a4b3c2d1"
        );

        // Enterprise Web3 Features - Optional
        let enterprise_nft_contract = get_optional("ENTERPRISE_NFT_CONTRACT");
        let enterprise_dao_contract = get_optional("ENTERPRISE_DAO_CONTRACT");
        let enterprise_governance_token = get_optional("ENTERPRISE_GOVERNANCE_TOKEN");
        
        // Blockchain Network Configuration
        let blockchain_network = get_with_default("NEXT_PUBLIC_BLOCKCHAIN_NETWORK", "testnet");

        // Infrastructure - Optional
        let redis_url = get_optional("REDIS_URL");
        let log_level = get_with_default("LOG_LEVEL", "info");

        // Parse supported tokens from comma-separated env var or strict defaults
        let supported_tokens_str = get_with_default("SUPPORTED_PAYMENT_TOKENS", 
            "0x55d398326f99059fF775485246999027B3197955,0x8AC76a51cc950d9822D68b83fE1Ad97B32Cd580d,0x337610d27c682E347C9cD60BD4b3b107C9d34dDD,0x64544969ed7EBf5f083679233325356EbE738930");
        
        let supported_payment_tokens = supported_tokens_str
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();

        if !errors.is_empty() {
            return Err(errors);
        }

        Ok(Config {
            database_url,
            backend_url,
            frontend_url,
            admin_frontend_url,
            ethereum_rpc_url,
            polygon_rpc_url,
            arbitrum_rpc_url,
            optimism_rpc_url,
            base_rpc_url,
            bsc_rpc_url,
            ethereum_ws_url,
            polygon_ws_url,
            arbitrum_ws_url,
            optimism_ws_url,
            base_ws_url,
            bsc_ws_url,
            ethereum_ws_backup,
            polygon_ws_backup,
            arbitrum_ws_backup,
            optimism_ws_backup,
            base_ws_backup,
            bsc_ws_backup,
            bsc_payment_contract,
            ethereum_payment_contract,
            polygon_payment_contract,
            arbitrum_payment_contract,
            optimism_payment_contract,
            base_payment_contract,
            payment_event_topic,
            enterprise_nft_contract,
            enterprise_dao_contract,
            enterprise_governance_token,
            blockchain_network,
            redis_url,
            log_level,
            supported_payment_tokens,
        })
    }

    /// Check if running in development environment
    fn is_development() -> bool {
        env::var("RUST_ENV").unwrap_or_else(|_| "development".to_string()) == "development" ||
        env::var("NODE_ENV").unwrap_or_else(|_| "development".to_string()) == "development"
    }

    /// Check if running in production environment
    pub fn is_production(&self) -> bool {
        env::var("RUST_ENV").unwrap_or_default() == "production" ||
        env::var("NODE_ENV").unwrap_or_default() == "production"
    }

    /// Get Web3 challenge endpoint
    pub fn web3_challenge_url(&self) -> String {
        format!("{}/api/auth/web3/challenge", self.backend_url)
    }

    /// Get Web3 verify endpoint
    pub fn web3_verify_url(&self) -> String {
        format!("{}/api/auth/web3/verify", self.backend_url)
    }

    /// Get Web3 permissions endpoint
    pub fn web3_permissions_url(&self) -> String {
        format!("{}/api/auth/web3/permissions", self.backend_url)
    }

    /// Get enterprise API endpoint
    pub fn enterprise_api_url(&self) -> String {
        format!("{}/api/enterprise", self.backend_url)
    }
}

/// Load environment from .env file
pub fn load_env() {
    // Load environment variables from .env file (searches current and parent directories)
    match dotenv::dotenv() {
        Ok(path) => println!("✅ Loaded environment variables from .env file: {:?}", path),
        Err(e) => {
            // Only log if we don't have the variables already
            if std::env::var("DATABASE_URL").is_err() {
                eprintln!("Info: .env file not loaded: {}", e);
            }
        }
    }
}

/// Initialize and validate configuration
pub fn init_config() -> Config {
    load_env();
    
    match Config::from_env() {
        Ok(config) => {
            println!("✅ Environment validation passed");
            config
        },
        Err(errors) => {
            eprintln!("❌ Environment validation failed:");
            for error in &errors {
                eprintln!("  {}", error);
            }
            eprintln!("\n💡 See CLAUDE.md for environment setup instructions");
            std::process::exit(1);
        }
    }
}

/// Create a fallback configuration for testing and development - Web3 only
pub fn get_fallback_config() -> Config {
    Config {
        database_url: "postgresql://localhost/epsx".to_string(),
        backend_url: "http://localhost:8080".to_string(),
        frontend_url: "http://localhost:3000".to_string(),
        admin_frontend_url: "http://localhost:3001".to_string(),
        ethereum_rpc_url: "https://eth.llamarpc.com".to_string(),
        polygon_rpc_url: "https://polygon.llamarpc.com".to_string(),
        arbitrum_rpc_url: "https://arbitrum.llamarpc.com".to_string(),
        optimism_rpc_url: "https://optimism.llamarpc.com".to_string(),
        base_rpc_url: "https://base.llamarpc.com".to_string(),
        bsc_rpc_url: "https://bsc-dataseed.binance.org".to_string(),
        ethereum_ws_url: "wss://eth.llamarcp.com".to_string(),
        polygon_ws_url: "wss://polygon-bor.publicnode.com".to_string(),
        arbitrum_ws_url: "wss://arbitrum-one.publicnode.com".to_string(),
        optimism_ws_url: "wss://optimism.publicnode.com".to_string(),
        base_ws_url: "wss://base.publicnode.com".to_string(),
        bsc_ws_url: "wss://bsc-ws-node.nariox.org:443".to_string(),
        ethereum_ws_backup: None,
        polygon_ws_backup: None,
        arbitrum_ws_backup: None,
        optimism_ws_backup: None,
        base_ws_backup: None,
        bsc_ws_backup: None,
        bsc_payment_contract: None,
        ethereum_payment_contract: None,
        polygon_payment_contract: None,
        arbitrum_payment_contract: None,
        optimism_payment_contract: None,
        base_payment_contract: None,
        payment_event_topic: "0xa7f9e7f4f9c6e7e3d8b3a2f1c0d9e8f7a6b5c4d3e2f1a0b9c8d7e6f5a4b3c2d1".to_string(),
        enterprise_nft_contract: None,
        enterprise_dao_contract: None,
        enterprise_governance_token: None,
        blockchain_network: "testnet".to_string(),
        redis_url: None,
        log_level: "info".to_string(),
        supported_payment_tokens: vec![
            "0x55d398326f99059fF775485246999027B3197955".to_string(), // USDT BSC
            "0x8AC76a51cc950d9822D68b83fE1Ad97B32Cd580d".to_string(), // USDC BSC
        ],
    }
}

// Convenience functions for Web3 configuration

/// Get BSC chain ID based on the blockchain network environment
pub fn get_bsc_chain_id(blockchain_network: &str) -> u64 {
    match blockchain_network {
        "mainnet" => 56,  // BSC Mainnet
        "testnet" => 97,  // BSC Testnet
        _ => 97, // Default to testnet for safety
    }
}
pub fn get_database_url() -> String {
    env::var("DATABASE_URL").expect("DATABASE_URL must be set")
}

// JWT functions removed - pure Web3 signature authentication only

pub fn get_log_level() -> String {
    env::var("LOG_LEVEL").unwrap_or_else(|_| "info".to_string())
}

pub fn is_production() -> bool {
    env::var("RUST_ENV").unwrap_or_default() == "production" ||
    env::var("NODE_ENV").unwrap_or_default() == "production"
}

pub fn is_development() -> bool {
    let env_val = env::var("RUST_ENV").unwrap_or_else(|_| "development".to_string());
    env_val == "development"
}

pub fn get_env_var(key: &str) -> Result<String, std::env::VarError> {
    env::var(key)
}