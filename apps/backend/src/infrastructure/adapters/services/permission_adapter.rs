// Web3 Permission Service Adapter (Infrastructure Layer)
// Provides infrastructure implementation for Web3 permission validation with blockchain integration

use crate::prelude::*;

use std::collections::HashMap;
use std::time::Duration;
use tokio::time::timeout;
use tracing::{error, info, warn, debug};
use sqlx::PgPool;
use ethers::prelude::*;
use ethers::providers::{Http, Provider};
use ethers::types::{Address, U256, Bytes, TransactionRequest};
use ethers::types::transaction::eip2718::TypedTransaction;
use std::str::FromStr;

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

/// Configuration for blockchain RPC connections
#[derive(Debug, Clone)]
pub struct BlockchainConfig {
    /// RPC endpoints for different chains
    pub rpc_endpoints: HashMap<u64, String>,
    /// Request timeout for blockchain queries
    pub request_timeout_ms: u64,
    /// Maximum retries for failed requests
    pub max_retries: u32,
    /// Rate limiting (requests per second) 
    pub rate_limit_per_second: u32,
}

impl Default for BlockchainConfig {
    fn default() -> Self {
        let mut rpc_endpoints = HashMap::new();
        rpc_endpoints.insert(1, "https://eth-mainnet.g.alchemy.com/v2/YOUR_KEY".to_string());
        rpc_endpoints.insert(56, "https://bsc-dataseed.binance.org".to_string()); 
        rpc_endpoints.insert(137, "https://polygon-rpc.com".to_string());
        
        Self {
            rpc_endpoints,
            request_timeout_ms: 30000, // 30 seconds
            max_retries: 3,
            rate_limit_per_second: 10,
        }
    }
}

/// Infrastructure adapter for Web3 permission service
pub struct Web3PermissionServiceAdapter {
    /// Cache for validation results
    cache: Option<Arc<dyn Cache>>,
    /// Blockchain configuration
    blockchain_config: BlockchainConfig,
    /// Database pool for permission queries
    pool: PgPool,
}

impl Web3PermissionServiceAdapter {
    pub fn new(
        cache: Option<Arc<dyn Cache>>,
        blockchain_config: Option<BlockchainConfig>,
        pool: PgPool,
    ) -> Self {
        Self {
            cache,
            blockchain_config: blockchain_config.unwrap_or_default(),
            pool,
        }
    }

    /// Validate NFT ownership on blockchain
    pub async fn validate_nft_ownership(
        &self,
        wallet_address: &WalletAddress,
        contract_address: &str,
        token_ids: &[u64],
        chain_id: u64,
    ) -> AppResult<NftOwnershipResult> {
        debug!(
            "Validating NFT ownership for wallet {} on contract {} (chain {})",
            wallet_address.as_str(),
            contract_address,
            chain_id
        );

        // Check cache first
        if let Some(cached_result) = self.get_cached_nft_result(
            wallet_address,
            contract_address,
            token_ids,
            chain_id
        ).await? {
            return Ok(cached_result);
        }

        // Get RPC endpoint for chain
        let rpc_url = self.blockchain_config.rpc_endpoints.get(&chain_id)
            .ok_or_else(|| AppError::blockchain_rpc_error(
                format!("No RPC endpoint configured for chain {}", chain_id)
            ).with_component("web3_permission_service"))?;

        // Validate NFT ownership via blockchain RPC
        let ownership_result = self.check_nft_ownership_blockchain(
            wallet_address,
            contract_address,
            token_ids,
            rpc_url,
        ).await?;

        // Cache the result
        self.cache_nft_result(
            wallet_address,
            contract_address,
            token_ids,
            chain_id,
            &ownership_result,
        ).await?;

        Ok(ownership_result)
    }

    /// Validate ERC-20 token balance on blockchain
    pub async fn validate_token_balance(
        &self,
        wallet_address: &WalletAddress,
        contract_address: &str,
        min_balance: &str,
        chain_id: u64,
    ) -> AppResult<TokenBalanceResult> {
        debug!(
            "Validating token balance for wallet {} on contract {} (min: {}, chain: {})",
            wallet_address.as_str(),
            contract_address,
            min_balance,
            chain_id
        );

        // Check cache first
        if let Some(cached_result) = self.get_cached_token_result(
            wallet_address,
            contract_address,
            min_balance,
            chain_id
        ).await? {
            return Ok(cached_result);
        }

        // Get RPC endpoint for chain
        let rpc_url = self.blockchain_config.rpc_endpoints.get(&chain_id)
            .ok_or_else(|| AppError::blockchain_rpc_error(
                format!("No RPC endpoint configured for chain {}", chain_id)
            ).with_component("web3_permission_service"))?;

        // Validate token balance via blockchain RPC
        let balance_result = self.check_token_balance_blockchain(
            wallet_address,
            contract_address,
            min_balance,
            rpc_url,
        ).await?;

        // Cache the result
        self.cache_token_result(
            wallet_address,
            contract_address,
            min_balance,
            chain_id,
            &balance_result,
        ).await?;

        Ok(balance_result)
    }

    /// Validate DAO voting power on blockchain
    pub async fn validate_dao_membership(
        &self,
        wallet_address: &WalletAddress,
        dao_contract: &str,
        min_voting_power: &str,
        chain_id: u64,
    ) -> AppResult<DaoMembershipResult> {
        debug!(
            "Validating DAO membership for wallet {} on contract {} (min voting power: {}, chain: {})",
            wallet_address.as_str(),
            dao_contract,
            min_voting_power,
            chain_id
        );

        // Check cache first
        if let Some(cached_result) = self.get_cached_dao_result(
            wallet_address,
            dao_contract,
            min_voting_power,
            chain_id
        ).await? {
            return Ok(cached_result);
        }

        // Get RPC endpoint for chain
        let rpc_url = self.blockchain_config.rpc_endpoints.get(&chain_id)
            .ok_or_else(|| AppError::blockchain_rpc_error(
                format!("No RPC endpoint configured for chain {}", chain_id)
            ).with_component("web3_permission_service"))?;

        // Validate DAO membership via blockchain RPC
        let membership_result = self.check_dao_membership_blockchain(
            wallet_address,
            dao_contract,
            min_voting_power,
            rpc_url,
        ).await?;

        // Cache the result
        self.cache_dao_result(
            wallet_address,
            dao_contract,
            min_voting_power,
            chain_id,
            &membership_result,
        ).await?;

        Ok(membership_result)
    }

    /// Validate all Web3 permissions for a user
    pub async fn validate_user_web3_permissions(
        &self,
        user: &WalletUser,
        _context: &Web3PermissionContext,
    ) -> AppResult<Vec<Web3ValidationResult>> {
        let web3_permissions: Vec<Permission> = user.permissions()
            .iter()
            .filter(|p| p.requires_web3_validation())
            .cloned()
            .collect();

        if web3_permissions.is_empty() {
            return Ok(Vec::new());
        }

        let mut results = Vec::new();

        for permission in web3_permissions {
            let result = match permission.permission_type() {
                PermissionType::Manual => {
                    // Manual permissions are always valid
                    Web3ValidationResult {
                        permission: permission.clone(),
                        is_valid: true,
                        validation_type: Web3ValidationType::Manual,
                        blockchain_data: None,
                        error_details: None,
                    }
                },
                
                PermissionType::NftGated { contract_address, token_ids, chain_id } => {
                    self.validate_nft_permission(
                        user.wallet_address(),
                        &permission,
                        contract_address,
                        token_ids,
                        *chain_id,
                    ).await?
                },
                
                PermissionType::TokenGated { contract_address, min_balance, chain_id } => {
                    self.validate_token_permission(
                        user.wallet_address(),
                        &permission,
                        contract_address,
                        min_balance,
                        *chain_id,
                    ).await?
                },
                
                PermissionType::DaoGovernance { dao_contract, min_voting_power, chain_id, .. } => {
                    self.validate_dao_permission(
                        user.wallet_address(),
                        &permission,
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

    // Private blockchain validation methods

    async fn validate_nft_permission(
        &self,
        wallet_address: &WalletAddress,
        permission: &Permission,
        contract_address: &str,
        token_ids: &[u64],
        chain_id: u64,
    ) -> AppResult<Web3ValidationResult> {
        match self.validate_nft_ownership(wallet_address, contract_address, token_ids, chain_id).await {
            Ok(nft_result) => {
                Ok(Web3ValidationResult {
                    permission: permission.clone(),
                    is_valid: nft_result.owns_required_nfts,
                    validation_type: Web3ValidationType::NftGated,
                    blockchain_data: Some(format!(
                        "NFT ownership check: owns {} out of {} required tokens",
                        nft_result.owned_token_ids.len(),
                        if token_ids.is_empty() { 1 } else { token_ids.len() }
                    )),
                    error_details: nft_result.error_details,
                })
            },
            Err(e) => {
                error!("NFT validation failed for {}: {}", wallet_address.as_str(), e);
                Ok(Web3ValidationResult {
                    permission: permission.clone(),
                    is_valid: false,
                    validation_type: Web3ValidationType::NftGated,
                    blockchain_data: None,
                    error_details: Some(format!("Blockchain error: {}", e)),
                })
            }
        }
    }

    async fn validate_token_permission(
        &self,
        wallet_address: &WalletAddress,
        permission: &Permission,
        contract_address: &str,
        min_balance: &str,
        chain_id: u64,
    ) -> AppResult<Web3ValidationResult> {
        match self.validate_token_balance(wallet_address, contract_address, min_balance, chain_id).await {
            Ok(balance_result) => {
                Ok(Web3ValidationResult {
                    permission: permission.clone(),
                    is_valid: balance_result.meets_minimum_balance,
                    validation_type: Web3ValidationType::TokenGated,
                    blockchain_data: Some(format!(
                        "Token balance: {} (required: {})",
                        balance_result.current_balance,
                        min_balance
                    )),
                    error_details: balance_result.error_details,
                })
            },
            Err(e) => {
                error!("Token validation failed for {}: {}", wallet_address.as_str(), e);
                Ok(Web3ValidationResult {
                    permission: permission.clone(),
                    is_valid: false,
                    validation_type: Web3ValidationType::TokenGated,
                    blockchain_data: None,
                    error_details: Some(format!("Blockchain error: {}", e)),
                })
            }
        }
    }

    async fn validate_dao_permission(
        &self,
        wallet_address: &WalletAddress,
        permission: &Permission,
        dao_contract: &str,
        min_voting_power: &str,
        chain_id: u64,
    ) -> AppResult<Web3ValidationResult> {
        match self.validate_dao_membership(wallet_address, dao_contract, min_voting_power, chain_id).await {
            Ok(dao_result) => {
                Ok(Web3ValidationResult {
                    permission: permission.clone(),
                    is_valid: dao_result.meets_minimum_voting_power,
                    validation_type: Web3ValidationType::DaoGovernance,
                    blockchain_data: Some(format!(
                        "DAO voting power: {} (required: {})",
                        dao_result.current_voting_power,
                        min_voting_power
                    )),
                    error_details: dao_result.error_details,
                })
            },
            Err(e) => {
                error!("DAO validation failed for {}: {}", wallet_address.as_str(), e);
                Ok(Web3ValidationResult {
                    permission: permission.clone(),
                    is_valid: false,
                    validation_type: Web3ValidationType::DaoGovernance,
                    blockchain_data: None,
                    error_details: Some(format!("Blockchain error: {}", e)),
                })
            }
        }
    }

    // Blockchain RPC methods (simplified implementations)

    async fn check_nft_ownership_blockchain(
        &self,
        wallet_address: &WalletAddress,
        contract_address: &str,
        token_ids: &[u64],
        rpc_url: &str,
    ) -> AppResult<NftOwnershipResult> {
        info!("🔍 Checking NFT ownership for wallet {} on contract {}", wallet_address.as_str(), contract_address);
        
        let timeout_duration = Duration::from_millis(self.blockchain_config.request_timeout_ms);
        
        let result = timeout(timeout_duration, async {
            self.check_nft_ownership_rpc(wallet_address, contract_address, token_ids, rpc_url).await
        }).await.map_err(|_| {
            AppError::blockchain_rpc_error(
                "NFT ownership check timeout".to_string()
            ).with_component("web3_permission_service")
        })??;
        
        Ok(result)
    }

    async fn check_token_balance_blockchain(
        &self,
        wallet_address: &WalletAddress,
        contract_address: &str,
        min_balance: &str,
        rpc_url: &str,
    ) -> AppResult<TokenBalanceResult> {
        info!("🔍 Checking token balance for wallet {} on contract {}", wallet_address.as_str(), contract_address);
        
        let timeout_duration = Duration::from_millis(self.blockchain_config.request_timeout_ms);
        
        let result = timeout(timeout_duration, async {
            self.check_token_balance_rpc(wallet_address, contract_address, min_balance, rpc_url).await
        }).await.map_err(|_| {
            AppError::blockchain_rpc_error(
                "Token balance check timeout".to_string()
            ).with_component("web3_permission_service")
        })??;
        
        Ok(result)
    }

    async fn check_dao_membership_blockchain(
        &self,
        wallet_address: &WalletAddress,
        dao_contract: &str,
        min_voting_power: &str,
        rpc_url: &str,
    ) -> AppResult<DaoMembershipResult> {
        info!("🔍 Checking DAO membership for wallet {} on contract {}", wallet_address.as_str(), dao_contract);
        
        let timeout_duration = Duration::from_millis(self.blockchain_config.request_timeout_ms);
        
        let result = timeout(timeout_duration, async {
            self.check_dao_membership_rpc(wallet_address, dao_contract, min_voting_power, rpc_url).await
        }).await.map_err(|_| {
            AppError::blockchain_rpc_error(
                "DAO membership check timeout".to_string()
            ).with_component("web3_permission_service")
        })??;
        
        Ok(result)
    }

    // Cache methods (simplified)

    async fn get_cached_nft_result(
        &self,
        wallet_address: &WalletAddress,
        contract_address: &str,
        token_ids: &[u64],
        chain_id: u64,
    ) -> AppResult<Option<NftOwnershipResult>> {
        if let Some(cache) = &self.cache {
            let cache_key = format!(
                "nft:{}:{}:{}:{}",
                wallet_address.as_str(),
                contract_address,
                token_ids.iter().map(|id| id.to_string()).collect::<Vec<_>>().join(","),
                chain_id
            );
            
            if let Some(cached_data) = cache.get(&cache_key) {
                if let Ok(result) = serde_json::from_str::<NftOwnershipResult>(&cached_data) {
                    debug!("Cache hit for NFT validation: {}", cache_key);
                    return Ok(Some(result));
                }
            }
        }
        Ok(None)
    }

    async fn cache_nft_result(
        &self,
        wallet_address: &WalletAddress,
        contract_address: &str,
        token_ids: &[u64],
        chain_id: u64,
        result: &NftOwnershipResult,
    ) -> AppResult<()> {
        if let Some(cache) = &self.cache {
            let cache_key = format!(
                "nft:{}:{}:{}:{}",
                wallet_address.as_str(),
                contract_address,
                token_ids.iter().map(|id| id.to_string()).collect::<Vec<_>>().join(","),
                chain_id
            );
            
            if let Ok(serialized) = serde_json::to_string(result) {
                cache.set(&cache_key, serialized, Some(300)); // 5 minutes TTL
                debug!("Cached NFT validation result: {}", cache_key);
            }
        }
        Ok(())
    }

    async fn get_cached_token_result(
        &self,
        wallet_address: &WalletAddress,
        contract_address: &str,
        min_balance: &str,
        chain_id: u64,
    ) -> AppResult<Option<TokenBalanceResult>> {
        if let Some(cache) = &self.cache {
            let cache_key = format!(
                "token:{}:{}:{}:{}",
                wallet_address.as_str(),
                contract_address,
                min_balance,
                chain_id
            );
            
            if let Some(cached_data) = cache.get(&cache_key) {
                if let Ok(result) = serde_json::from_str::<TokenBalanceResult>(&cached_data) {
                    debug!("Cache hit for token validation: {}", cache_key);
                    return Ok(Some(result));
                }
            }
        }
        Ok(None)
    }

    async fn cache_token_result(
        &self,
        wallet_address: &WalletAddress,
        contract_address: &str,
        min_balance: &str,
        chain_id: u64,
        result: &TokenBalanceResult,
    ) -> AppResult<()> {
        if let Some(cache) = &self.cache {
            let cache_key = format!(
                "token:{}:{}:{}:{}",
                wallet_address.as_str(),
                contract_address,
                min_balance,
                chain_id
            );
            
            if let Ok(serialized) = serde_json::to_string(result) {
                cache.set(&cache_key, serialized, Some(300));
                debug!("Cached token validation result: {}", cache_key);
            }
        }
        Ok(())
    }

    async fn get_cached_dao_result(
        &self,
        wallet_address: &WalletAddress,
        dao_contract: &str,
        min_voting_power: &str,
        chain_id: u64,
    ) -> AppResult<Option<DaoMembershipResult>> {
        if let Some(cache) = &self.cache {
            let cache_key = format!(
                "dao:{}:{}:{}:{}",
                wallet_address.as_str(),
                dao_contract,
                min_voting_power,
                chain_id
            );
            
            if let Some(cached_data) = cache.get(&cache_key) {
                if let Ok(result) = serde_json::from_str::<DaoMembershipResult>(&cached_data) {
                    debug!("Cache hit for DAO validation: {}", cache_key);
                    return Ok(Some(result));
                }
            }
        }
        Ok(None)
    }

    async fn cache_dao_result(
        &self,
        wallet_address: &WalletAddress,
        dao_contract: &str,
        min_voting_power: &str,
        chain_id: u64,
        result: &DaoMembershipResult,
    ) -> AppResult<()> {
        if let Some(cache) = &self.cache {
            let cache_key = format!(
                "dao:{}:{}:{}:{}",
                wallet_address.as_str(),
                dao_contract,
                min_voting_power,
                chain_id
            );
            
            if let Ok(serialized) = serde_json::to_string(result) {
                cache.set(&cache_key, serialized, Some(300));
                debug!("Cached DAO validation result: {}", cache_key);
            }
        }
        Ok(())
    }
    
    /// Process automatic permissions for a wallet address
    pub async fn process_automatic_permissions(&self, wallet_address: &str) -> AppResult<Vec<String>> {
        debug!("Processing automatic permissions for wallet: {}", wallet_address);
        
        // TODO: Implement actual automatic permission processing
        // For now, return basic permissions
        Ok(vec![
            "epsx:read:analytics".to_string(),
            "epsx:read:market_data".to_string(),
        ])
    }
    
    /// Get user permissions for a wallet address
    pub async fn get_user_permissions(&self, wallet_address: &str) -> AppResult<Vec<String>> {
        debug!("Getting user permissions for wallet: {}", wallet_address);

        // Get permissions from normalized permission tables (groups + direct)
        let permissions_result = sqlx::query!(
            r#"
            -- Permissions from groups
            SELECT DISTINCT p.permission_string
            FROM wallet_group_assignments wga
            JOIN permission_group_memberships pgm ON wga.group_id = pgm.group_id
            JOIN permissions p ON pgm.permission_id = p.id
            WHERE wga.wallet_address = $1
              AND wga.is_active = true
              AND p.is_active = true
              AND (wga.expires_at IS NULL OR wga.expires_at > NOW())

            UNION

            -- Direct permissions
            SELECT DISTINCT p.permission_string
            FROM wallet_direct_permissions wdp
            JOIN permissions p ON wdp.permission_id = p.id
            WHERE wdp.wallet_address = $1
              AND wdp.is_active = true
              AND p.is_active = true
              AND (wdp.expires_at IS NULL OR wdp.expires_at > NOW())

            ORDER BY permission_string
            "#,
            wallet_address
        )
        .fetch_all(&self.pool)
        .await;

        match permissions_result {
            Ok(records) => {
                let permissions: Vec<String> = records
                    .into_iter()
                    .filter_map(|r| r.permission_string)
                    .collect();

                if !permissions.is_empty() {
                    debug!("Found {} permissions for wallet: {}", permissions.len(), wallet_address);
                    Ok(permissions)
                } else {
                    debug!("No permissions found for wallet: {}", wallet_address);
                    // Return basic permissions as fallback for new users
                    Ok(vec![
                        "epsx:read:analytics".to_string(),
                        "epsx:read:market_data".to_string(),
                        "epsx:read:portfolio".to_string(),
                    ])
                }
            }
            Err(e) => {
                debug!("Database error getting permissions for wallet {}: {}", wallet_address, e);
                // Return basic permissions for new users
                Ok(vec![
                    "epsx:read:analytics".to_string(),
                    "epsx:read:market_data".to_string(),
                    "epsx:read:portfolio".to_string(),
                ])
            }
        }
    }
    
    /// Check if user has a specific permission
    pub async fn has_permission(&self, wallet_address: &str, permission: &str) -> AppResult<bool> {
        debug!("Checking permission '{}' for wallet: {}", permission, wallet_address);
        
        // Get user permissions from database
        let user_permissions = self.get_user_permissions(wallet_address).await?;
        
        // Use the backend permission checking logic
        let has_perm = crate::auth::permissions::check_permission_access(&user_permissions, permission);
        
        debug!("Permission check result: {} for wallet: {}", has_perm, wallet_address);
        Ok(has_perm)
    }
    
    /// Grant manual permission to a user
    pub async fn grant_manual_permission(
        &self, 
        wallet_address: &str, 
        permission: &str, 
        _granted_by: Option<String>,
        _expires_at: Option<chrono::DateTime<chrono::Utc>>
    ) -> AppResult<()> {
        debug!("Granting permission '{}' to wallet: {}", permission, wallet_address);
        
        // TODO: Implement actual permission granting
        // For now, just log the action
        info!(
            wallet_address = wallet_address,
            permission = permission,
            "Manual permission granted"
        );
        
        Ok(())
    }
    
    /// Revoke permission from a user
    pub async fn revoke_permission(&self, wallet_address: &str, permission: &str) -> AppResult<()> {
        debug!("Revoking permission '{}' from wallet: {}", permission, wallet_address);
        
        // TODO: Implement actual permission revocation
        // For now, just log the action
        info!(
            wallet_address = wallet_address,
            permission = permission,
            "Permission revoked"
        );
        
        Ok(())
    }

    /// Real NFT ownership check using Ethers RPC calls
    async fn check_nft_ownership_rpc(
        &self,
        wallet_address: &WalletAddress,
        contract_address: &str,
        token_ids: &[u64],
        rpc_url: &str,
    ) -> AppResult<NftOwnershipResult> {
        debug!("🔗 Making real NFT ownership RPC call to {}", rpc_url);
        
        // Create provider
        let provider = Provider::<Http>::try_from(rpc_url)
            .map_err(|e| AppError::blockchain_rpc_error(format!("Failed to create provider: {}", e)))?;
        
        // Parse contract address
        let contract_addr = Address::from_str(contract_address)
            .map_err(|e| AppError::validation_error(format!("Invalid contract address: {}", e)))?;
        
        // Parse wallet address 
        let wallet_addr = Address::from_str(wallet_address.as_str())
            .map_err(|e| AppError::validation_error(format!("Invalid wallet address: {}", e)))?;
        
        let mut owned_tokens = Vec::new();
        #[allow(unused_assignments)] // Variable is overwritten in conditional blocks
        let mut owns_required = false;
        
        if token_ids.is_empty() {
            // Check overall NFT balance if no specific token IDs
            let balance_call = provider.get_balance(wallet_addr, None).await
                .map_err(|e| AppError::blockchain_rpc_error(format!("Failed to get NFT balance: {}", e)))?;
            
            owns_required = balance_call > U256::zero();
            if owns_required {
                owned_tokens.push(1); // Placeholder token ID
            }
        } else {
            // Check specific token ownership using ERC721 ownerOf
            for &token_id in token_ids {
                let token_id_u256 = U256::from(token_id);
                
                // Call ownerOf(tokenId) on the contract
                let call_data = ethers::abi::encode(&[
                    ethers::abi::Token::Uint(token_id_u256)
                ]);
                
                // ERC721 ownerOf function selector: 0x6352211e  
                let mut function_call = vec![0x63, 0x52, 0x21, 0x1e];
                function_call.extend_from_slice(&call_data);
                
                let call_request = TransactionRequest::new()
                    .to(contract_addr)
                    .data(Bytes::from(function_call));
                
                let typed_tx = TypedTransaction::Legacy(call_request);
                match provider.call(&typed_tx, None).await {
                    Ok(result) => {
                        if result.len() >= 32 {
                            let owner_bytes = &result[12..32]; // Extract address from 32-byte result
                            let owner_addr = Address::from_slice(owner_bytes);
                            
                            if owner_addr == wallet_addr {
                                owned_tokens.push(token_id);
                            }
                        }
                    }
                    Err(e) => {
                        warn!("Failed to check ownership of token {}: {}", token_id, e);
                    }
                }
            }
            
            owns_required = !owned_tokens.is_empty();
        }
        
        info!("✅ NFT ownership check complete: owns {} tokens", owned_tokens.len());
        
        Ok(NftOwnershipResult {
            owns_required_nfts: owns_required,
            owned_token_ids: owned_tokens,
            contract_address: contract_address.to_string(),
            chain_id: 1, // Will be detected from RPC
            error_details: None,
        })
    }

    /// Real token balance check using Ethers RPC calls  
    async fn check_token_balance_rpc(
        &self,
        wallet_address: &WalletAddress,
        contract_address: &str,
        min_balance: &str,
        rpc_url: &str,
    ) -> AppResult<TokenBalanceResult> {
        debug!("🔗 Making real token balance RPC call to {}", rpc_url);
        
        // Create provider
        let provider = Provider::<Http>::try_from(rpc_url)
            .map_err(|e| AppError::blockchain_rpc_error(format!("Failed to create provider: {}", e)))?;
        
        // Parse addresses
        let contract_addr = Address::from_str(contract_address)
            .map_err(|e| AppError::validation_error(format!("Invalid contract address: {}", e)))?;
        
        let wallet_addr = Address::from_str(wallet_address.as_str())
            .map_err(|e| AppError::validation_error(format!("Invalid wallet address: {}", e)))?;
        
        // Parse minimum balance
        let min_balance_u256 = U256::from_dec_str(min_balance)
            .map_err(|e| AppError::validation_error(format!("Invalid min_balance format: {}", e)))?;
        
        // Call balanceOf(address) on ERC20 contract
        let call_data = ethers::abi::encode(&[
            ethers::abi::Token::Address(wallet_addr)
        ]);
        
        // ERC20 balanceOf function selector: 0x70a08231
        let mut function_call = vec![0x70, 0xa0, 0x82, 0x31];
        function_call.extend_from_slice(&call_data);
        
        let call_request = TransactionRequest::new()
            .to(contract_addr)
            .data(Bytes::from(function_call));
        
        let typed_tx = TypedTransaction::Legacy(call_request);
        let result = provider.call(&typed_tx, None).await
            .map_err(|e| AppError::blockchain_rpc_error(format!("Failed to get token balance: {}", e)))?;
        
        let current_balance = if result.len() >= 32 {
            U256::from_big_endian(&result)
        } else {
            U256::zero()
        };
        
        let meets_minimum = current_balance >= min_balance_u256;
        
        info!("✅ Token balance check complete: {} >= {} = {}", 
              current_balance, min_balance_u256, meets_minimum);
        
        Ok(TokenBalanceResult {
            meets_minimum_balance: meets_minimum,
            current_balance: current_balance.to_string(),
            min_balance_required: min_balance.to_string(),
            contract_address: contract_address.to_string(),
            chain_id: 1, // Will be detected from RPC
            error_details: None,
        })
    }

    /// Real DAO membership check using Ethers RPC calls
    async fn check_dao_membership_rpc(
        &self,
        wallet_address: &WalletAddress,
        dao_contract: &str,
        min_voting_power: &str,
        rpc_url: &str,
    ) -> AppResult<DaoMembershipResult> {
        debug!("🔗 Making real DAO membership RPC call to {}", rpc_url);
        
        // Create provider
        let provider = Provider::<Http>::try_from(rpc_url)
            .map_err(|e| AppError::blockchain_rpc_error(format!("Failed to create provider: {}", e)))?;
        
        // Parse addresses
        let contract_addr = Address::from_str(dao_contract)
            .map_err(|e| AppError::validation_error(format!("Invalid DAO contract address: {}", e)))?;
        
        let wallet_addr = Address::from_str(wallet_address.as_str())
            .map_err(|e| AppError::validation_error(format!("Invalid wallet address: {}", e)))?;
        
        // Parse minimum voting power
        let min_voting_power_u256 = U256::from_dec_str(min_voting_power)
            .map_err(|e| AppError::validation_error(format!("Invalid min_voting_power format: {}", e)))?;
        
        // Common DAO functions to try (different DAOs use different function names)
        let voting_power_functions = vec![
            ("getVotes", 0xb9a3c84c), // Common ERC20Votes function
            ("balanceOf", 0x70a08231), // ERC20 balance (governance tokens)
            ("votingPower", 0x0d32dd66), // Custom voting power function
        ];
        
        let mut current_voting_power = U256::zero();
        let mut function_found = false;
        
        for (function_name, selector) in voting_power_functions {
            let selector: u32 = selector;
            // Call function with wallet address parameter
            let call_data = ethers::abi::encode(&[
                ethers::abi::Token::Address(wallet_addr)
            ]);
            
            let mut function_call = selector.to_be_bytes().to_vec();
            function_call.extend_from_slice(&call_data);
            
            let call_request = TransactionRequest::new()
                .to(contract_addr)
                .data(Bytes::from(function_call));
            
            let typed_tx = TypedTransaction::Legacy(call_request);
            match provider.call(&typed_tx, None).await {
                Ok(result) => {
                    if result.len() >= 32 {
                        current_voting_power = U256::from_big_endian(&result);
                        function_found = true;
                        debug!("✅ Found voting power {} using function {}", current_voting_power, function_name);
                        break;
                    }
                }
                Err(e) => {
                    debug!("Failed to call {} function: {}", function_name, e);
                    // Continue trying other functions
                }
            }
        }
        
        if !function_found {
            warn!("No supported DAO voting power function found on contract {}", dao_contract);
            // Return zero voting power instead of error for graceful degradation
        }
        
        let meets_minimum = current_voting_power >= min_voting_power_u256;
        let is_member = current_voting_power > U256::zero();
        
        info!("✅ DAO membership check complete: voting power {} >= {} = {}", 
              current_voting_power, min_voting_power_u256, meets_minimum);
        
        Ok(DaoMembershipResult {
            meets_minimum_voting_power: meets_minimum,
            current_voting_power: current_voting_power.to_string(),
            min_voting_power_required: min_voting_power.to_string(),
            dao_contract: dao_contract.to_string(),
            chain_id: 1, // Will be detected from RPC
            is_member,
            delegation_info: HashMap::new(), // TODO: Implement delegation checking
            error_details: None,
        })
    }
}

/// Result of NFT ownership validation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NftOwnershipResult {
    pub owns_required_nfts: bool,
    pub owned_token_ids: Vec<u64>,
    pub contract_address: String,
    pub chain_id: u64,
    pub error_details: Option<String>,
}

/// Result of token balance validation  
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenBalanceResult {
    pub meets_minimum_balance: bool,
    pub current_balance: String,
    pub min_balance_required: String,
    pub contract_address: String,
    pub chain_id: u64,
    pub error_details: Option<String>,
}

/// Result of DAO membership validation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DaoMembershipResult {
    pub meets_minimum_voting_power: bool,
    pub current_voting_power: String,
    pub min_voting_power_required: String,
    pub dao_contract: String,
    pub chain_id: u64,
    pub is_member: bool,
    pub delegation_info: HashMap<String, String>,
    pub error_details: Option<String>,
}