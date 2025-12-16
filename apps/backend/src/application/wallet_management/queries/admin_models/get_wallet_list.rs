// Admin Get Wallet List Query
// Query to retrieve wallet list for admin dashboard with filtering and pagination

use crate::application::shared::{ApplicationResult, Query};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

// ============================================================================
// QUERY
// ============================================================================

#[derive(Debug, Clone, Deserialize)]
pub struct GetWalletListQuery {
    pub page: Option<i32>,
    pub limit: Option<i32>,
    pub search: Option<String>,
    pub status: Option<String>,
    pub date_from: Option<String>,
    pub date_to: Option<String>,
    pub sort_by: Option<String>,
    pub sort_order: Option<String>,
    pub exclude_group_id: Option<String>,
}

impl Query for GetWalletListQuery {
    type Response = GetWalletListResponse;

    fn validate(&self) -> ApplicationResult<()> {
        // Validate page
        if let Some(page) = self.page {
            if page < 1 {
                return Err(crate::application::shared::ApplicationError::validation(
                    "page",
                    "Page must be >= 1",
                ));
            }
        }

        // Validate limit
        if let Some(limit) = self.limit {
            if !(1..=1000).contains(&limit) {
                return Err(crate::application::shared::ApplicationError::validation(
                    "limit",
                    "Limit must be between 1 and 1000",
                ));
            }
        }

        // Validate status
        if let Some(ref status) = self.status {
            if status != "active" && status != "inactive" {
                return Err(crate::application::shared::ApplicationError::validation(
                    "status",
                    "Status must be 'active' or 'inactive'",
                ));
            }
        }

        // Validate sort_order
        if let Some(ref order) = self.sort_order {
            let upper = order.to_uppercase();
            if upper != "ASC" && upper != "DESC" {
                return Err(crate::application::shared::ApplicationError::validation(
                    "sort_order",
                    "Sort order must be 'ASC' or 'DESC'",
                ));
            }
        }

        Ok(())
    }
}

// ============================================================================
// RESPONSE
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetWalletListResponse {
    pub success: bool,
    pub wallets: Vec<WalletSummaryDto>,
    pub pagination: PaginationDto,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalletSummaryDto {
    pub wallet_address: String,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub last_auth_at: Option<DateTime<Utc>>,
    pub permissions_count: i32,
    pub groups_count: i32,
    pub last_activity: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaginationDto {
    pub page: i32,
    pub limit: i32,
    pub total: i32,
    pub total_pages: i32,
    pub has_next_page: bool,
    pub has_previous_page: bool,
}
