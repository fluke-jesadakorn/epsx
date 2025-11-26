use crate::prelude::*;

use crate::application::shared::{Query, QueryHandler, ApplicationResult, ApplicationError};
use crate::application::wallet_management::queries::models::{
    GetWalletPermissionsQuery, GetWalletPermissionsResponse
};
use crate::domain::wallet_management::{WalletUserRepositoryPort, WalletAddress};

/// Query handler for retrieving wallet permissions
pub struct GetWalletPermissionsQueryHandler {
    wallet_repository: Arc<dyn WalletUserRepositoryPort>,
}

impl GetWalletPermissionsQueryHandler {
    pub fn new(wallet_repository: Arc<dyn WalletUserRepositoryPort>) -> Self {
        Self { wallet_repository }
    }
}

#[async_trait]
impl QueryHandler<GetWalletPermissionsQuery> for GetWalletPermissionsQueryHandler {
    async fn handle(&self, query: GetWalletPermissionsQuery) -> ApplicationResult<GetWalletPermissionsResponse> {
        // 1. Validate query
        query.validate()?;

        // 2. Parse wallet address
        let wallet_addr = WalletAddress::new(query.wallet_address.clone())
            .map_err(|e| ApplicationError::validation("wallet_address", e.to_string()))?;

        // 3. Find wallet
        let wallet = self.wallet_repository
            .find_by_wallet(&wallet_addr)
            .await
            .map_err(|e| ApplicationError::infrastructure(e.to_string()))?
            .ok_or_else(|| ApplicationError::not_found("Wallet", query.wallet_address.clone()))?;

        // 4. Extract permissions
        // 4. Filter permissions
        let all_permissions: Vec<String> = wallet.permissions()
            .iter()
            .map(|p| p.as_str().to_string())
            .collect();

        let active_permissions: Vec<String> = wallet.permissions()
            .iter()
            .filter(|p| p.is_active())
            .map(|p| p.as_str().to_string())
            .collect();

        let expired_permissions: Vec<String> = wallet.permissions()
            .iter()
            .filter(|p| !p.is_active())
            .map(|p| p.as_str().to_string())
            .collect();

        // 5. Build response
        Ok(GetWalletPermissionsResponse {
            wallet_address: wallet.wallet_address().as_str().to_string(),
            permissions: all_permissions,
            active_permissions,
            expired_permissions,
        })
    }
}
