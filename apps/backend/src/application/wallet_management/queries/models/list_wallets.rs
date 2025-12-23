// List Wallets Query
// Query to retrieve multiple wallets with filtering and pagination

use crate::application::shared::{Query, ApplicationResult};
use crate::domain::wallet_management::value_objects::Permission;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListWalletsQuery {
    pub limit: usize,
    pub offset: usize,
    pub permission_filter: Option<Vec<String>>,
    pub wallet_pattern_filter: Option<String>,
}

impl Default for ListWalletsQuery {
    fn default() -> Self {
        Self::new()
    }
}

impl ListWalletsQuery {
    pub fn new() -> Self {
        Self {
            limit: 50,
            offset: 0,
            permission_filter: None,
            wallet_pattern_filter: None,
        }
    }
    
    pub fn with_limit(mut self, limit: usize) -> Self {
        self.limit = limit;
        self
    }
    
    pub fn with_offset(mut self, offset: usize) -> Self {
        self.offset = offset;
        self
    }
    
    pub fn with_permission_filter(mut self, permissions: Vec<String>) -> Self {
        self.permission_filter = Some(permissions);
        self
    }
    
    pub fn with_wallet_pattern_filter(mut self, pattern: String) -> Self {
        self.wallet_pattern_filter = Some(pattern);
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListWalletsResponse {
    pub users: Vec<WalletSummary>,
    pub total_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalletSummary {
    // Identity fields (Web3-first: wallet-based)
    pub id: String,                        // Wallet address for frontend
    pub display_name: Option<String>,      // Display name from database

    // Status and group fields
    pub group: String,                     // Derived from permissions (admin/user/premium)
    pub status: String,                    // Derived from is_active (active/inactive)
    pub is_active: bool,

    // Permission and tier fields
    pub permissions: HashSet<Permission>,
    pub permission_group: String,          // Permission group from permissions

    // Timestamp fields
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub last_login_at: Option<DateTime<Utc>>,
}

impl ListWalletsResponse {
    pub fn new(users: Vec<WalletSummary>, total_count: usize) -> Self {
        Self { users, total_count }
    }
}

impl Query for ListWalletsQuery {
    type Response = ListWalletsResponse;

    fn validate(&self) -> ApplicationResult<()> {
        // Validate limit is reasonable
        if self.limit > 1000 {
            return Err(crate::application::ApplicationError::validation(
                "limit",
                "Limit cannot exceed 1000"
            ));
        }
        Ok(())
    }
}