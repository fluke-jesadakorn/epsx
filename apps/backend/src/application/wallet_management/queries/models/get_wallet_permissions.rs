use serde::{Serialize, Deserialize};
use crate::application::shared::{Query, ApplicationResult};

/// Query to get user permissions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetWalletPermissionsQuery {
    pub wallet_address: String,
    pub include_expired: bool,
}

/// User permissions response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetWalletPermissionsResponse {
    pub wallet_address: String,
    pub permissions: Vec<String>,
    pub active_permissions: Vec<String>,
    pub expired_permissions: Vec<String>,
}

impl Query for GetWalletPermissionsQuery {
    type Response = GetWalletPermissionsResponse;

    fn validate(&self) -> ApplicationResult<()> {
        // Minimal validation for read queries
        // Wallet address validation handled at authentication layer
        Ok(())
    }
}