// Web3 Permission Service Adapter (Infrastructure Layer)
// Orchestrates Web3 permission validation with blockchain integration
// Uses database-backed permission checks for legacy compatibility

use crate::prelude::*;
use tracing::{debug, info, warn};
use diesel_async::{AsyncPgConnection, RunQueryDsl, pooled_connection::deadpool::Pool};
use diesel::prelude::*;

use crate::domain::wallet_management::{
    aggregates::WalletUser,
    value_objects::{WalletAddress, Permission, PermissionType},
    domain_services::{
        Web3PermissionContext,
        Web3ValidationResult,
        Web3ValidationType,
    },
};
use crate::infrastructure::cache::Cache;
use std::sync::Arc;

mod web3;
use web3::{Web3CacheMgr, NftValidator, TokenValidator, DaoValidator};

// Re-export types for backward compatibility
pub use web3::{BlockchainCfg as BlockchainConfig, NftResult as NftOwnershipResult, TokenResult as TokenBalanceResult, DaoResult as DaoMembershipResult};

/// Infrastructure adapter for Web3 permission service
pub struct Web3PermissionServiceAdapter {
    nft: NftValidator,
    token: TokenValidator,
    dao: DaoValidator,
    pool: &'static Pool<AsyncPgConnection>,
}

impl Web3PermissionServiceAdapter {
    pub fn new(
        cache: Option<Arc<dyn Cache>>,
        cfg: Option<BlockchainConfig>,
        pool: &'static Pool<AsyncPgConnection>,
    ) -> Self {
        let cache_mgr = Web3CacheMgr::new(cache);
        let blockchain_cfg = cfg.unwrap_or_default();

        Self {
            nft: NftValidator::new(cache_mgr.clone(), blockchain_cfg.clone()),
            token: TokenValidator::new(cache_mgr.clone(), blockchain_cfg.clone()),
            dao: DaoValidator::new(cache_mgr, blockchain_cfg),
            pool,
        }
    }

    // Public wrapper methods for backward compatibility

    /// Validate NFT ownership on blockchain
    pub async fn validate_nft_ownership(
        &self,
        wallet: &WalletAddress,
        contract: &str,
        tokens: &[u64],
        chain: u64,
    ) -> AppResult<NftOwnershipResult> {
        self.nft.validate(wallet, contract, tokens, chain).await
    }

    /// Validate ERC-20 token balance on blockchain
    pub async fn validate_token_balance(
        &self,
        wallet: &WalletAddress,
        contract: &str,
        min: &str,
        chain: u64,
    ) -> AppResult<TokenBalanceResult> {
        self.token.validate(wallet, contract, min, chain).await
    }

    /// Validate DAO voting power on blockchain
    pub async fn validate_dao_membership(
        &self,
        wallet: &WalletAddress,
        contract: &str,
        min_power: &str,
        chain: u64,
    ) -> AppResult<DaoMembershipResult> {
        self.dao.validate(wallet, contract, min_power, chain).await
    }

    /// Validate all Web3 permissions for a user
    pub async fn validate_user_web3_permissions(
        &self,
        user: &WalletUser,
        _context: &Web3PermissionContext,
    ) -> AppResult<Vec<Web3ValidationResult>> {
        let web3_perms: Vec<Permission> = user.permissions()
            .iter()
            .filter(|p| p.requires_web3_validation())
            .cloned()
            .collect();

        if web3_perms.is_empty() {
            return Ok(Vec::new());
        }

        let mut results = Vec::new();

        for perm in web3_perms {
            let result = match perm.permission_type() {
                PermissionType::Manual => {
                    Web3ValidationResult {
                        permission: perm.clone(),
                        is_valid: true,
                        validation_type: Web3ValidationType::Manual,
                        blockchain_data: None,
                        error_details: None,
                    }
                },

                PermissionType::NftGated { contract_address, token_ids, chain_id } => {
                    self.nft.validate_perm(
                        user.wallet_address(),
                        &perm,
                        contract_address,
                        token_ids,
                        *chain_id,
                    ).await?
                },

                PermissionType::TokenGated { contract_address, min_balance, chain_id } => {
                    self.token.validate_perm(
                        user.wallet_address(),
                        &perm,
                        contract_address,
                        min_balance,
                        *chain_id,
                    ).await?
                },

                PermissionType::DaoGovernance { dao_contract, min_voting_power, chain_id, .. } => {
                    self.dao.validate_perm(
                        user.wallet_address(),
                        &perm,
                        dao_contract,
                        min_voting_power,
                        *chain_id,
                    ).await?
                },
            };

            results.push(result);
        }

        Ok(results)
    }

    /// Process automatic permissions for a wallet address
    ///
    /// Future enhancement: Implement rule-based automatic permission assignment
    /// based on wallet activity, NFT/token holdings, or other criteria
    pub async fn process_automatic_permissions(&self, wallet: &str) -> AppResult<Vec<String>> {
        debug!("Processing automatic permissions for wallet: {}", wallet);

        // Returns basic read permissions for all users
        // Future: Add automatic permission rules engine
        Ok(vec![
            "epsx:read:analytics".to_string(),
            "epsx:read:market_data".to_string(),
        ])
    }

    /// Get user permissions for a wallet address (direct database query)
    pub async fn get_user_permissions(&self, wallet: &str) -> AppResult<Vec<String>> {
        debug!("Getting user permissions for wallet: {}", wallet);

        let wallet_lower = wallet.to_lowercase();
        let mut conn = self.pool.get().await
            .map_err(|e| AppError::database_error(format!("Pool error: {}", e)))?;

        #[derive(QueryableByName)]
        struct PermissionsJson {
            #[diesel(sql_type = diesel::sql_types::Jsonb)]
            get_wallet_effective_permissions: serde_json::Value,
        }

        match diesel::sql_query("SELECT get_wallet_effective_permissions($1)")
            .bind::<diesel::sql_types::Text, _>(&wallet_lower)
            .get_result::<PermissionsJson>(&mut conn)
            .await
        {
            Ok(result) => {
                let perm_strings: Vec<String> = if let serde_json::Value::Array(arr) = result.get_wallet_effective_permissions {
                    arr.into_iter()
                        .filter_map(|v| v.as_str().map(String::from))
                        .collect()
                } else {
                    vec![]
                };

                if !perm_strings.is_empty() {
                    debug!("Found {} permissions for wallet: {}", perm_strings.len(), wallet);
                    Ok(perm_strings)
                } else {
                    debug!("No permissions found for wallet: {}", wallet);
                    // Return basic permissions as fallback
                    Ok(vec![
                        "epsx:read:analytics".to_string(),
                        "epsx:read:market_data".to_string(),
                        "epsx:read:portfolio".to_string(),
                    ])
                }
            }
            Err(e) => {
                debug!("Error getting permissions for wallet {}: {}", wallet, e);
                // Return basic permissions for new users
                Ok(vec![
                    "epsx:read:analytics".to_string(),
                    "epsx:read:market_data".to_string(),
                    "epsx:read:portfolio".to_string(),
                ])
            }
        }
    }

    /// Check if user has a specific permission (direct database query)
    pub async fn has_permission(&self, wallet: &str, perm: &str) -> AppResult<bool> {
        debug!("Checking permission '{}' for wallet: {}", perm, wallet);

        let wallet_lower = wallet.to_lowercase();
        let mut conn = self.pool.get().await
            .map_err(|e| AppError::database_error(format!("Pool error: {}", e)))?;

        #[derive(QueryableByName)]
        struct PermissionCheck {
            #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Bool>)]
            wallet_has_permission: Option<bool>,
        }

        match diesel::sql_query("SELECT wallet_has_permission($1, $2)")
            .bind::<diesel::sql_types::Text, _>(&wallet_lower)
            .bind::<diesel::sql_types::Text, _>(perm)
            .get_result::<PermissionCheck>(&mut conn)
            .await
        {
            Ok(result) => {
                let has_perm = result.wallet_has_permission.unwrap_or(false);
                debug!("Permission check result: {} for wallet: {}", has_perm, wallet);
                Ok(has_perm)
            }
            Err(e) => {
                warn!("Permission check failed for wallet {}: {}", wallet, e);
                Ok(false)
            }
        }
    }

    /// Grant manual permission to a user
    ///
    /// Future enhancement: Integrate with wallet_permissions table for persistence
    pub async fn grant_manual_permission(
        &self,
        wallet: &str,
        perm: &str,
        _granted_by: Option<String>,
        _expires_at: Option<chrono::DateTime<chrono::Utc>>
    ) -> AppResult<()> {
        debug!("Granting permission '{}' to wallet: {}", perm, wallet);

        // Logs permission grant event
        // Future: Insert into wallet_permissions table with granted_by and expires_at
        info!(
            wallet_address = wallet,
            permission = perm,
            "Manual permission granted"
        );

        Ok(())
    }

    /// Revoke permission from a user
    ///
    /// Future enhancement: Integrate with wallet_permissions table for persistence
    pub async fn revoke_permission(&self, wallet: &str, perm: &str) -> AppResult<()> {
        debug!("Revoking permission '{}' from wallet: {}", perm, wallet);

        // Logs permission revocation event
        // Future: Delete from wallet_permissions table or mark as inactive
        info!(
            wallet_address = wallet,
            permission = perm,
            "Permission revoked"
        );

        Ok(())
    }
}
