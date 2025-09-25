// Shared Web3 Permission Types
// Common types used across all Web3 permission services

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Core permission information structure
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PermissionInfo {
    pub permission: String,
    pub permission_type: String,
    pub is_active: bool,
    pub expires_at: Option<DateTime<Utc>>,
    pub granted_at: DateTime<Utc>,
    pub last_verified_at: Option<DateTime<Utc>>,
    pub verification_data: Option<serde_json::Value>,
}

/// NFT-based permission configuration
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct NFTConfig {
    pub contract_address: String,
    pub network: String,
    pub permission: String,
    pub collection_name: Option<String>,
    pub require_specific_token: bool,
    pub specific_token_ids: Vec<String>,
    pub minimum_tokens: i32,
    pub check_ownership_live: bool,
}

/// Token-based permission configuration
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TokenConfig {
    pub contract_address: String,
    pub network: String,
    pub permission: String,
    pub token_name: Option<String>,
    pub token_symbol: Option<String>,
    pub minimum_balance: String, // String to handle large numbers
    pub token_decimals: i32,
    pub check_balance_live: bool,
}

/// DAO proposal for governance-based permissions
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DAOProposal {
    pub dao_contract_address: String,
    pub network: String,
    pub proposal_id: String,
    pub title: String,
    pub description: Option<String>,
    pub target_wallet_address: String,
    pub permission: String,
    pub proposal_status: String,
    pub voting_end: Option<DateTime<Utc>>,
}

/// Result of permission verification
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PermissionVerificationResult {
    pub wallet_address: String,
    pub permission: String,
    pub is_granted: bool,
    pub verification_type: String,
    pub verification_data: serde_json::Value,
    pub cached_until: Option<DateTime<Utc>>,
}

/// Permission delegation between wallets
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PermissionDelegation {
    pub delegator: String, // Original permission holder
    pub delegate: String, // Receiving wallet
    pub permission: String, // Delegated permission
    pub signature: String, // EIP-712 signature proof
    pub expires_at: DateTime<Utc>,
    pub delegation_depth: u8, // Max 3 levels deep
    pub network: String, // Chain where delegation is valid
    pub nonce: String, // Prevent replay attacks
}

/// EIP-712 message structure for delegation
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct EIP712DelegationMessage {
    pub delegator: String,
    pub delegate: String,
    pub permission: String,
    pub expires_at: i64,
    pub nonce: String,
    pub network: String,
}

/// Conditional permission logic
#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum PermissionCondition {
    And(Vec<PermissionCondition>),
    Or(Vec<PermissionCondition>),
    Not(Box<PermissionCondition>),
    TokenBalance {
        contract: String,
        network: String,
        min_balance: String,
    },
    NFTOwnership {
        contract: String,
        network: String,
        token_id: Option<String>,
    },
    DelegatedBy {
        delegator: String,
    },
    TimeWindow {
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    },
}

/// Network configuration for blockchain interactions
#[derive(Debug, Clone)]
pub struct NetworkConfig {
    pub ethereum_rpc_url: String,
    pub polygon_rpc_url: String,
    pub arbitrum_rpc_url: String,
    pub optimism_rpc_url: String,
    pub base_rpc_url: String,
    pub bsc_rpc_url: String,
}

impl Default for NetworkConfig {
    fn default() -> Self {
        Self {
            ethereum_rpc_url: "https://eth.llamarpc.com".to_string(),
            polygon_rpc_url: "https://polygon.llamarpc.com".to_string(),
            arbitrum_rpc_url: "https://arbitrum.llamarpc.com".to_string(),
            optimism_rpc_url: "https://optimism.llamarpc.com".to_string(),
            base_rpc_url: "https://base.llamarpc.com".to_string(),
            bsc_rpc_url: "https://bsc.llamarpc.com".to_string(),
        }
    }
}

/// Common error types for Web3 permission operations
#[derive(Debug, thiserror::Error)]
pub enum Web3PermissionError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),
    
    #[error("RPC error: {0}")]
    Rpc(String),
    
    #[error("Invalid wallet address: {0}")]
    InvalidWallet(String),
    
    #[error("Permission not found: {0}")]
    PermissionNotFound(String),
    
    #[error("Insufficient balance: required {required}, found {found}")]
    InsufficientBalance { required: String, found: String },
    
    #[error("Invalid signature: {0}")]
    InvalidSignature(String),
    
    #[error("Network not supported: {0}")]
    UnsupportedNetwork(String),
    
    #[error("Configuration error: {0}")]
    Configuration(String),
}

pub type Web3PermissionResult<T> = Result<T, Web3PermissionError>;