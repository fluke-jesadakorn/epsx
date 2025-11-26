// Admin Get Wallet Detail Query
// Query to retrieve detailed wallet information for admin dashboard

use crate::application::shared::{ApplicationResult, Query};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

// ============================================================================
// QUERY
// ============================================================================

#[derive(Debug, Clone, Deserialize)]
pub struct GetWalletDetailQuery {
    pub wallet_address: String,
}

impl Query for GetWalletDetailQuery {
    type Response = GetWalletDetailResponse;

    fn validate(&self) -> ApplicationResult<()> {
        // Validate wallet address is not empty
        if self.wallet_address.trim().is_empty() {
            return Err(crate::application::shared::ApplicationError::validation(
                "wallet_address",
                "Wallet address cannot be empty",
            ));
        }

        // Validate wallet address format (basic check)
        if !self.wallet_address.starts_with("0x") {
            return Err(crate::application::shared::ApplicationError::validation(
                "wallet_address",
                "Wallet address must start with 0x",
            ));
        }

        Ok(())
    }
}

// ============================================================================
// RESPONSE
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetWalletDetailResponse {
    pub success: bool,
    pub wallet: WalletDetailDto,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalletDetailDto {
    pub wallet_address: String,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub last_auth_at: Option<DateTime<Utc>>,
    pub permissions: Vec<WalletPermissionDto>,
    pub groups: Vec<WalletGroupDto>,
    pub activity_summary: WalletActivitySummaryDto,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalletPermissionDto {
    pub permission: String,
    pub source: String, // "group" or "direct"
    pub granted_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
    pub is_active: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalletGroupDto {
    pub group_id: String,
    pub group_name: String,
    pub group_type: String,
    pub assigned_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
    pub is_active: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalletActivitySummaryDto {
    pub total_logins: i32,
    pub last_30_days_logins: i32,
    pub total_permissions: i32,
    pub active_permissions: i32,
    pub expired_permissions: i32,
    pub groups_count: i32,
}
