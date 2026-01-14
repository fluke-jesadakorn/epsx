use crate::prelude::*;
use chrono::Utc;

use crate::application::shared::{Query, QueryHandler, ApplicationResult, ApplicationError};
use crate::application::wallet_management::queries::models::{
    GetWalletQuery, GetWalletResponse, WalletStats
};
use crate::domain::wallet_management::{
    WalletUserRepositoryPort, WalletAddress
};

/// Query handler for retrieving wallet information
pub struct GetWalletQueryHandler {
    wallet_repository: Arc<dyn WalletUserRepositoryPort>,
}

impl GetWalletQueryHandler {
    pub fn new(
        wallet_repository: Arc<dyn WalletUserRepositoryPort>,
    ) -> Self {
        Self {
            wallet_repository,
        }
    }
}

#[async_trait]
impl QueryHandler<GetWalletQuery> for GetWalletQueryHandler {
    async fn handle(&self, query: GetWalletQuery) -> ApplicationResult<GetWalletResponse> {
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

        // 4. Calculate statistics
        let total_permissions = wallet.permissions().len() as u32;
        let active_permissions = wallet.permissions()
            .iter()
            .filter(|p| p.is_active())
            .count() as u32;
        let expired_permissions = total_permissions - active_permissions;

        let account_age_days = (Utc::now() - wallet.created_at()).num_days();

        let stats = WalletStats {
            total_permissions,
            active_permissions,
            expired_permissions,
            account_age_days,
        };

        // 5. Optionally include permissions
        let permissions = if query.include_permissions {
            Some(
                wallet.permissions()
                    .iter()
                    .filter(|p| p.is_active())
                    .map(|p| p.as_str().to_string())
                    .collect()
            )
        } else {
            None
        };

        // 6. Optionally include session count
        // Stateless: Always return None or 0. Since we don't store sessions anymore, we can't count them.
        let active_session_count = None;

        // 7. Build response
        Ok(GetWalletResponse {
            wallet_address: wallet.wallet_address().to_user_id(),
            is_active: wallet.is_active(),
            created_at: wallet.created_at(),
            updated_at: wallet.updated_at(),
            last_login_at: wallet.last_auth_at(),
            permissions,
            active_session_count,
            stats,
        })
    }
}
