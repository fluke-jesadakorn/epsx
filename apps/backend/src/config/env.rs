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

    // Enterprise Web3 Features
    pub enterprise_nft_contract: Option<String>,
    pub enterprise_dao_contract: Option<String>,
    pub enterprise_governance_token: Option<String>,
    
    // Blockchain Network Configuration
    pub blockchain_network: String,

    // Infrastructure (1 variable)
    pub redis_url: Option<String>,
    pub log_level: String,
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

        // Enterprise Web3 Features - Optional
        let enterprise_nft_contract = get_optional("ENTERPRISE_NFT_CONTRACT");
        let enterprise_dao_contract = get_optional("ENTERPRISE_DAO_CONTRACT");
        let enterprise_governance_token = get_optional("ENTERPRISE_GOVERNANCE_TOKEN");
        
        // Blockchain Network Configuration
        let blockchain_network = get_with_default("NEXT_PUBLIC_BLOCKCHAIN_NETWORK", "testnet");

        // Infrastructure - Optional
        let redis_url = get_optional("REDIS_URL");
        let log_level = get_with_default("LOG_LEVEL", "info");

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
            enterprise_nft_contract,
            enterprise_dao_contract,
            enterprise_governance_token,
            blockchain_network,
            redis_url,
            log_level,
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
        format!("{}/api/v1/enterprise", self.backend_url)
    }
}

/// Load environment from .env file
pub fn load_env() {
    // In a monorepo setup with TurboRepo, environment variables are injected via dotenv-cli
    // before the process starts. We don't need to manually load the .env file here.
    // However, we keep this as a no-op or fallback if needed in the future.
    // if let Err(e) = dotenv::dotenv() {
    //     eprintln!("Warning: Failed to load .env file: {}", e);
    // }
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
        enterprise_nft_contract: None,
        enterprise_dao_contract: None,
        enterprise_governance_token: None,
        blockchain_network: "testnet".to_string(),
        redis_url: None,
        log_level: "info".to_string(),
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