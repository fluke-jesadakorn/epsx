use crate::prelude::*;

use crate::application::shared::{Query, QueryHandler, ApplicationResult, ApplicationError};
use crate::application::wallet_management::queries::models::{
    ListWalletsQuery, ListWalletsResponse, WalletSummary
};
use crate::domain::wallet_management::WalletUserRepositoryPort;

/// Query handler for listing wallets with filtering
pub struct ListWalletsQueryHandler {
    wallet_repository: Arc<dyn WalletUserRepositoryPort>,
}

impl ListWalletsQueryHandler {
    pub fn new(wallet_repository: Arc<dyn WalletUserRepositoryPort>) -> Self {
        Self { wallet_repository }
    }
}

#[async_trait]
impl QueryHandler<ListWalletsQuery> for ListWalletsQueryHandler {
    async fn handle(&self, query: ListWalletsQuery) -> ApplicationResult<ListWalletsResponse> {
        // 1. Validate query
        query.validate()?;

        // 2. Create search criteria from query
        let has_permissions = query.permission_filter
            .map(|perms| perms.iter()
                .filter_map(|p| crate::domain::wallet_management::value_objects::Permission::new(p).ok())
                .collect())
            .unwrap_or_default();

        let criteria = crate::domain::wallet_management::WalletUserSearchCriteria {
            wallet_pattern: query.wallet_pattern_filter,
            is_active: None,
            permission_group: None,
            has_permissions,
            permission_type: None,
            chain_id: None,
            created_after: None,
            created_before: None,
            last_auth_after: None,
            last_auth_before: None,
            interacted_with_contract: None,
            custom_filters: std::collections::HashMap::new(),
        };

        // 3. Query repository
        let result = self.wallet_repository
            .find_by_criteria(&criteria, query.limit as u32, query.offset as u32)
            .await
            .map_err(|e| ApplicationError::infrastructure(e.to_string()))?;

        // 4. Map domain models to DTOs
        let summaries: Vec<WalletSummary> = result.users
            .into_iter()
            .map(|wallet| {
                // Derive group from permissions
                let group = if wallet.permissions().iter().any(|p| p.platform() == "admin") {
                    "admin".to_string()
                } else if wallet.permissions().iter().any(|p| p.platform() == "premium") {
                    "premium".to_string()
                } else {
                    "user".to_string()
                };

                // Status from is_active
                let status = if wallet.is_active() {
                    "active".to_string()
                } else {
                    "inactive".to_string()
                };

                // Permission group - use first group or "none"
                let permission_group = wallet.groups()
                    .iter()
                    .next().cloned()
                    .unwrap_or_else(|| "none".to_string());

                WalletSummary {
                    id: wallet.wallet_address().as_str().to_string(),
                    display_name: None, // TODO: Add display name support
                    group,
                    status,
                    is_active: wallet.is_active(),
                    permissions: wallet.permissions().clone(),
                    permission_group,
                    created_at: wallet.created_at(),
                    updated_at: wallet.updated_at(),
                    last_login_at: wallet.last_auth_at(),
                }
            })
            .collect();

        Ok(ListWalletsResponse::new(summaries, result.total_count as usize))
    }
}
