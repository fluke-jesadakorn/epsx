use std::sync::Arc;

use crate::application::shared::{ApplicationResult, ApplicationError};
use crate::domain::user_management::WalletUserSearchCriteria;
use crate::application::user_management::{
    ListUsersQuery,
    ListUsersResponse,
    UserSummary,
};
use crate::domain::user_management::WalletUserRepositoryPort;
// use crate::domain::shared_kernel::AggregateRoot; // REMOVED - using direct accessor methods now

/// Wallet Query Service
/// Handles read-only operations for wallet-based user data
#[derive(Clone)]
pub struct WalletQueryService {
    wallet_repository: Arc<dyn WalletUserRepositoryPort>,
}

impl WalletQueryService {
    /// Create a new WalletQueryService
    pub fn new(wallet_repository: Arc<dyn WalletUserRepositoryPort>) -> Self {
        Self { 
            wallet_repository,
        }
    }
    
    // Firebase UID lookup removed - migrated to Web3
    
    /// List wallets with filtering and pagination
    pub async fn list_wallets(
        &self,
        query: ListUsersQuery,
    ) -> ApplicationResult<ListUsersResponse> {
        tracing::info!("Processing ListWalletsQuery with limit: {}, offset: {}", query.limit, query.offset);
        
        // Create search criteria from query
        let mut search_criteria = WalletUserSearchCriteria::default();
        
        // Apply wallet pattern filter if specified
        if let Some(wallet_pattern) = &query.wallet_pattern_filter {
            search_criteria.wallet_pattern = Some(wallet_pattern.clone());
        }
        
        // Apply permission filter if specified
        if let Some(permissions) = &query.permission_filter {
            for permission_str in permissions {
                if let Ok(permission) = crate::domain::user_management::value_objects::Permission::new(permission_str.clone()) {
                    search_criteria.has_permissions.push(permission);
                }
            }
        }
        
        let users = self.wallet_repository
            .find_by_criteria(&search_criteria, query.limit as u32, query.offset as u32)
            .await
            .map_err(|e| ApplicationError::infrastructure(e.to_string()))?;
        
        let total_count = users.users.len(); // Use actual result count
        
        // Convert domain wallets to summary format using pure domain data
        let mut user_summaries = Vec::new();
        
        for wallet_user in users.users {
            let permissions_vec: Vec<String> = wallet_user.permissions().iter()
                .map(|p| p.to_string())
                .collect();
            
            // Derive role from permissions
            let role = if permissions_vec.iter().any(|p| p.contains("admin")) {
                "admin".to_string()
            } else if permissions_vec.iter().any(|p| p.contains("premium")) {
                "premium".to_string()
            } else {
                "user".to_string()
            };
            
            // Derive status from is_active
            let status = if wallet_user.is_active() {
                "active".to_string()
            } else {
                "inactive".to_string()
            };
            
            user_summaries.push(UserSummary {
                id: wallet_user.wallet_address().to_string(),  // Use wallet address as ID
                email: format!("{}@wallet.web3", wallet_user.wallet_address().to_string()), // Web3-first: no email, use wallet address
                display_name: None, // Web3-first: display name from wallet metadata if needed
                role,
                status,
                is_active: wallet_user.is_active(),
                email_verified: false, // Web3-first: no email verification needed
                permissions: wallet_user.permissions().clone(),
                package_tier: wallet_user.permission_groups().iter().next().unwrap_or(&"basic".to_string()).clone(), // Use primary permission group
                created_at: wallet_user.created_at(),
                updated_at: wallet_user.updated_at(),
                last_login_at: wallet_user.last_auth_at(),
            });
        }
        
        Ok(ListUsersResponse::new(user_summaries, total_count))
    }

    /// Update wallet (placeholder)
    pub async fn update_wallet(
        &self,
        _wallet_address: &str,
        _update_data: WalletUpdateData,
    ) -> ApplicationResult<()> {
        // TODO: Implement wallet update with SQLx
        Ok(())
    }

    /// Delete wallet (placeholder)
    pub async fn delete_wallet(&self, _wallet_address: &str) -> ApplicationResult<()> {
        // TODO: Implement wallet deletion with SQLx
        Ok(())
    }

    /// Create wallet (placeholder)
    pub async fn create_wallet(&self, _wallet_data: CreateWalletData) -> ApplicationResult<String> {
        // TODO: Implement wallet creation with SQLx
        Ok("wallet_address_placeholder".to_string())
    }
}

/// Placeholder structs for wallet operations
#[derive(Debug)]
pub struct WalletUpdateData {
    pub is_active: Option<bool>,
    pub permission_groups: Option<Vec<String>>,
}

#[derive(Debug)]
pub struct CreateWalletData {
    pub wallet_address: String,
    pub initial_permissions: Option<Vec<String>>,
}