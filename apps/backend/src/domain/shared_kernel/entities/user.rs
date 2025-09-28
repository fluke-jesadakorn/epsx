// User entity for shared use
// Web3-first user information for lightweight operations
use serde::{Deserialize, Serialize};

/// Basic wallet user information for lightweight operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalletUserInfo {
    pub wallet_address: String,
    pub display_name: Option<String>,
    pub tier_level: String,
    pub is_active: bool,
}

impl WalletUserInfo {
    pub fn new(wallet_address: String, tier_level: String) -> Self {
        Self {
            wallet_address,
            display_name: None,
            tier_level,
            is_active: true,
        }
    }
}