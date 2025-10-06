use crate::prelude::*;

use crate::application::shared::{Query, QueryHandler, ApplicationResult, ApplicationError};
use crate::application::wallet_management::queries::models::{
    SearchWalletsQuery, SearchWalletsResponse
};
use crate::domain::wallet_management::WalletUserRepositoryPort;

/// Query handler for searching wallets
pub struct SearchWalletsQueryHandler {
    wallet_repository: Arc<dyn WalletUserRepositoryPort>,
}

impl SearchWalletsQueryHandler {
    pub fn new(wallet_repository: Arc<dyn WalletUserRepositoryPort>) -> Self {
        Self { wallet_repository }
    }
}

#[async_trait]
impl QueryHandler<SearchWalletsQuery> for SearchWalletsQueryHandler {
    async fn handle(&self, query: SearchWalletsQuery) -> ApplicationResult<SearchWalletsResponse> {
        // 1. Validate query
        query.validate()?;

        // 2. Create search criteria
        let has_permissions = query.has_permissions.iter()
            .filter_map(|p| crate::domain::wallet_management::value_objects::Permission::new(p).ok())
            .collect();

        let criteria = crate::domain::wallet_management::WalletUserSearchCriteria {
            wallet_pattern: query.wallet_pattern,
            is_active: query.is_active,
            permission_group: None,
            has_permissions,
            permission_type: None,
            chain_id: None,
            created_after: query.created_after,
            created_before: query.created_before,
            last_auth_after: query.last_login_after,
            last_auth_before: None,
            interacted_with_contract: None,
            custom_filters: std::collections::HashMap::new(),
        };

        // 3. Search repository
        let result = self.wallet_repository
            .find_by_criteria(&criteria, query.pagination.page_size, query.pagination.offset())
            .await
            .map_err(|e| ApplicationError::infrastructure(e.to_string()))?;

        // 4. Map to response
        let total_pages = (result.total_count as f64 / query.pagination.page_size as f64).ceil() as u32;
        let current_page = query.pagination.page;

        // Map WalletUser to WalletSummary
        let users: Vec<crate::application::wallet_management::queries::models::search_wallets::WalletSummary> = result.users
            .into_iter()
            .map(|wallet| crate::application::wallet_management::queries::models::search_wallets::WalletSummary {
                wallet_address: wallet.wallet_address().to_user_id(),
                is_active: wallet.is_active(),
                created_at: wallet.created_at(),
                last_login_at: wallet.last_auth_at(),
                permission_count: wallet.permissions().len() as u32,
                active_session_count: 0, // TODO: Add session count tracking
            })
            .collect();

        Ok(SearchWalletsResponse {
            users,
            pagination: crate::application::wallet_management::queries::models::search_wallets::PaginationResult {
                page: current_page,
                page_size: query.pagination.page_size,
                total_pages,
                total_count: result.total_count,
                has_next: current_page < total_pages,
                has_previous: current_page > 1,
            },
            search_metadata: crate::application::wallet_management::queries::models::search_wallets::SearchMetadata {
                execution_time_ms: 0, // TODO: Add timing
                filters_applied: vec![],
                sort_applied: None,
                cache_hit: false,
            },
        })
    }
}
