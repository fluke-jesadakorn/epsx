use chrono::{Utc, Duration};
use std::collections::{HashSet, HashMap};
use std::sync::Arc;

use crate::core::errors::AppResult;
use crate::domain::shared_kernel::Specification;
use crate::domain::wallet_management::{
    aggregates::{WalletUser},
    value_objects::{Permission, WalletAddress, PermissionType},
};
use crate::infrastructure::blockchain::{BlockchainValidationClient, NftValidationResult, TokenValidationResult, DaoValidationResult};

/// Domain service for managing wallet-based user permissions
/// This service contains business logic for permission validation,
/// blockchain state synchronization, and cross-chain permission management
pub struct WalletPermissionService {
    blockchain_client: Arc<BlockchainValidationClient>,
}

/// @deprecated Use WalletPermissionService instead
pub struct Web3WalletPermissionService;

impl WalletPermissionService {
    pub fn new() -> Self {
        Self {
            blockchain_client: Arc::new(BlockchainValidationClient::new()),
        }
    }

    pub fn with_client(blockchain_client: Arc<BlockchainValidationClient>) -> Self {
        Self { blockchain_client }
    }

    pub fn with_timeout(blockchain_client: Arc<BlockchainValidationClient>, timeout_ms: u64) -> Self {
        let client = (*blockchain_client).clone().with_timeout(timeout_ms);
        Self {
            blockchain_client: Arc::new(client)
        }
    }
    /// Calculate effective permissions for a wallet user
    /// Includes direct permissions, group-based permissions, and Web3-validated permissions
    pub async fn calculate_effective_permissions(
        &self,
        wallet: &WalletUser,
        context: &Web3PermissionContext
    ) -> AppResult<HashSet<Permission>> {
        let mut effective_permissions = HashSet::new();
        
        // Start with direct manual permissions
        for permission in wallet.permissions() {
            if permission.is_active() && permission.is_manual() {
                effective_permissions.insert(permission.clone());
            }
        }
        
        // Add group-based permissions
        let groups: Vec<String> = wallet.groups().iter().cloned().collect();
        let group_permissions = self.get_group_permissions(&groups)?;
        effective_permissions.extend(group_permissions);
        
        // Add Web3-validated permissions (NFT, token, DAO)
        effective_permissions.extend(
            self.get_web3_validated_permissions(wallet, context).await?
        );
        
        Ok(effective_permissions)
    }
    
    /// Check if a wallet can perform an action on a resource in a given Web3 context
    pub async fn can_wallet_access(
        &self,
        wallet: &WalletUser,
        platform: &str,
        resource: &str,
        action: &str,
        context: &Web3PermissionContext
    ) -> AppResult<bool> {
        // Wallet must be active
        if !wallet.is_active() {
            return Ok(false);
        }
        
        // Calculate effective permissions
        let effective_permissions = self.calculate_effective_permissions(wallet, context).await?;
        
        // Check if any permission grants access
        Ok(effective_permissions
            .iter()
            .any(|p| p.grants_access(platform, resource, action)))
    }
    
    /// Generate default permissions for a new wallet user based on their characteristics
    pub async fn generate_default_permissions(
        &self,
        wallet: &WalletUser,
        context: &Web3PermissionContext
    ) -> AppResult<HashSet<Permission>> {
        let mut permissions = HashSet::new();
        
        // All wallet users get basic read access
        permissions.insert(Permission::new("epsx:analytics:view")?);
        permissions.insert(Permission::new("epsx:user:read")?);
        
        // Group-based permissions
        for group in wallet.groups() {
            permissions.extend(self.get_group_permissions(&[group.clone()])?);
        }
        
        // Chain-specific permissions based on user's primary chain
        if let Some(chain_id) = context.primary_chain_id {
            permissions.extend(self.get_chain_specific_permissions(chain_id)?);
        }
        
        Ok(permissions)
    }
    
    /// Validate Web3 permissions against blockchain state
    pub async fn validate_web3_permissions(
        &self,
        wallet: &WalletUser,
        permissions: &[Permission],
        context: &Web3PermissionContext
    ) -> AppResult<Vec<Web3ValidationResult>> {
        let mut results = Vec::new();
        
        for permission in permissions {
            if permission.requires_web3_validation() {
                let result = self.validate_single_web3_permission(
                    wallet.wallet_address(),
                    permission,
                    context
                ).await?;
                results.push(result);
            } else {
                // Manual permissions are always valid
                results.push(Web3ValidationResult {
                    permission: permission.clone(),
                    is_valid: true,
                    validation_type: Web3ValidationType::Manual,
                    blockchain_data: None,
                    error_details: None,
                });
            }
        }
        
        Ok(results)
    }
    
    /// Validate a single Web3 permission against blockchain state
    pub async fn validate_single_web3_permission(
        &self,
        wallet_address: &WalletAddress,
        permission: &Permission,
        context: &Web3PermissionContext
    ) -> AppResult<Web3ValidationResult> {
        match permission.permission_type() {
            PermissionType::Manual => {
                Ok(Web3ValidationResult {
                    permission: permission.clone(),
                    is_valid: true,
                    validation_type: Web3ValidationType::Manual,
                    blockchain_data: None,
                    error_details: None,
                })
            },
            
            PermissionType::NftGated { contract_address, token_ids, chain_id } => {
                self.validate_nft_permission(
                    wallet_address,
                    permission,
                    contract_address,
                    token_ids,
                    *chain_id,
                    context
                ).await
            },
            
            PermissionType::TokenGated { contract_address, min_balance, chain_id } => {
                self.validate_token_permission(
                    wallet_address,
                    permission,
                    contract_address,
                    min_balance,
                    *chain_id,
                    context
                ).await
            },
            
            PermissionType::DaoGovernance { dao_contract, min_voting_power, chain_id, .. } => {
                self.validate_dao_permission(
                    wallet_address,
                    permission,
                    dao_contract,
                    min_voting_power,
                    *chain_id,
                    context
                ).await
            },
        }
    }
    
    /// Create time-limited Web3 permissions for temporary access
    pub fn create_temporary_web3_permissions(
        &self,
        base_permissions: &HashSet<Permission>,
        duration: Duration
    ) -> AppResult<HashSet<Permission>> {
        let expires_at = Utc::now() + duration;
        let mut temporary_permissions = HashSet::new();
        
        for permission in base_permissions {
            let temp_permission = match permission.permission_type() {
                PermissionType::Manual => {
                    Permission::new(permission.as_str())?.with_expiration(expires_at)
                },
                PermissionType::NftGated { contract_address, token_ids, chain_id } => {
                    Permission::new_nft_gated(
                        permission.as_str(),
                        contract_address.clone(),
                        token_ids.clone(),
                        *chain_id
                    )?.with_expiration(expires_at)
                },
                PermissionType::TokenGated { contract_address, min_balance, chain_id } => {
                    Permission::new_token_gated(
                        permission.as_str(),
                        contract_address.clone(),
                        min_balance.clone(),
                        *chain_id
                    )?.with_expiration(expires_at)
                },
                PermissionType::DaoGovernance { dao_contract, min_voting_power, chain_id, governance_metadata } => {
                    Permission::new_dao_governance(
                        permission.as_str(),
                        dao_contract.clone(),
                        min_voting_power.clone(),
                        *chain_id,
                        governance_metadata.clone()
                    )?.with_expiration(expires_at)
                },
            };
            temporary_permissions.insert(temp_permission);
        }
        
        Ok(temporary_permissions)
    }
    
    /// Get permissions that are about to expire
    pub fn get_expiring_permissions(
        &self,
        wallet: &WalletUser,
        within: Duration
    ) -> Vec<Permission> {
        let cutoff = Utc::now() + within;
        
        wallet.permissions()
            .iter()
            .filter(|p| {
                if let Some(expires_at) = p.expires_at() {
                    expires_at <= cutoff
                } else {
                    false
                }
            })
            .cloned()
            .collect()
    }
    
    /// Synchronize user permissions with current blockchain state
    pub async fn sync_permissions_with_blockchain(
        &self,
        wallet: &mut WalletUser,
        context: &Web3PermissionContext
    ) -> AppResult<PermissionSyncResult> {
        let web3_permissions: Vec<Permission> = wallet.permissions()
            .iter()
            .filter(|p| p.requires_web3_validation())
            .cloned()
            .collect();
        
        let validation_results = self.validate_web3_permissions(
            wallet,
            &web3_permissions,
            context
        ).await?;
        
        let mut sync_result = PermissionSyncResult::new();
        
        for result in validation_results {
            if !result.is_valid {
                // Remove invalid permissions
                wallet.revoke_permission(&result.permission)?;
                sync_result.revoked_permissions.push(result.permission);
            } else {
                sync_result.validated_permissions.push(result.permission);
            }
        }
        
        Ok(sync_result)
    }
    
    // Private helper methods
    
    fn get_group_permissions(&self, groups: &[String]) -> AppResult<HashSet<Permission>> {
        let mut permissions = HashSet::new();
        
        for group in groups {
            match group.as_str() {
                "basic" => {
                    permissions.insert(Permission::new("epsx:basic:access")?);
                },
                "premium" => {
                    permissions.insert(Permission::new("epsx:premium:access")?);
                    permissions.insert(Permission::new("epsx:analytics:*")?);
                },
                "vip" => {
                    permissions.insert(Permission::new("epsx:vip:access")?);
                    permissions.insert(Permission::new("admin:users:view")?);
                },
                "admin" => {
                    permissions.insert(Permission::new("admin:*:*")?);
                },
                "enhanced" => {
                    permissions.insert(Permission::new("epsx:enhanced:access")?);
                    permissions.insert(Permission::new("epsx:notifications:manage")?);
                },
                _ => {
                    // Custom group - derive permissions from group name
                    permissions.insert(Permission::new(format!("epsx:group:{}", group))?);
                }
            }
        }
        
        Ok(permissions)
    }
    
    async fn get_web3_validated_permissions(
        &self,
        wallet: &WalletUser,
        context: &Web3PermissionContext
    ) -> AppResult<HashSet<Permission>> {
        let mut validated_permissions = HashSet::new();
        
        let web3_permissions: Vec<Permission> = wallet.permissions()
            .iter()
            .filter(|p| p.requires_web3_validation())
            .cloned()
            .collect();
        
        let validation_results = self.validate_web3_permissions(
            wallet,
            &web3_permissions,
            context
        ).await?;
        
        for result in validation_results {
            if result.is_valid {
                validated_permissions.insert(result.permission);
            }
        }
        
        Ok(validated_permissions)
    }
    
    fn get_chain_specific_permissions(&self, chain_id: u64) -> AppResult<HashSet<Permission>> {
        let mut permissions = HashSet::new();
        
        match chain_id {
            1 => { // Ethereum Mainnet
                permissions.insert(Permission::new("epsx:ethereum:access")?);
            },
            56 => { // BSC
                permissions.insert(Permission::new("epsx:bsc:access")?);
            },
            137 => { // Polygon
                permissions.insert(Permission::new("epsx:polygon:access")?);
            },
            _ => {
                // Generic chain access
                permissions.insert(Permission::new("epsx:chain:access")?);
            }
        }
        
        Ok(permissions)
    }
    
    async fn validate_nft_permission(
        &self,
        wallet_address: &WalletAddress,
        permission: &Permission,
        contract_address: &str,
        token_ids: &[u64],
        chain_id: u64,
        _context: &Web3PermissionContext
    ) -> AppResult<Web3ValidationResult> {
        match self.blockchain_client.validate_nft_ownership(
            chain_id,
            wallet_address.as_str(),
            contract_address,
            token_ids
        ).await {
            Ok(nft_result) => Ok(Web3ValidationResult {
                permission: permission.clone(),
                is_valid: nft_result.is_valid,
                validation_type: Web3ValidationType::NftGated,
                blockchain_data: Some(format!(
                    "NFT validation for wallet {} on contract {} (chain {}): owns {} tokens, total balance {}",
                    wallet_address.as_str(),
                    contract_address,
                    chain_id,
                    nft_result.owned_tokens.len(),
                    nft_result.balance
                )),
                error_details: None,
            }),
            Err(e) => Ok(Web3ValidationResult {
                permission: permission.clone(),
                is_valid: false,
                validation_type: Web3ValidationType::NftGated,
                blockchain_data: None,
                error_details: Some(format!("Blockchain validation failed: {}", e)),
            })
        }
    }
    
    async fn validate_token_permission(
        &self,
        wallet_address: &WalletAddress,
        permission: &Permission,
        contract_address: &str,
        min_balance: &str,
        chain_id: u64,
        _context: &Web3PermissionContext
    ) -> AppResult<Web3ValidationResult> {
        match self.blockchain_client.validate_token_balance(
            chain_id,
            wallet_address.as_str(),
            contract_address,
            min_balance
        ).await {
            Ok(token_result) => Ok(Web3ValidationResult {
                permission: permission.clone(),
                is_valid: token_result.is_valid,
                validation_type: Web3ValidationType::TokenGated,
                blockchain_data: Some(format!(
                    "Token validation for wallet {} on contract {} (chain {}): balance {} / min {}",
                    wallet_address.as_str(),
                    contract_address,
                    chain_id,
                    token_result.balance,
                    token_result.min_balance
                )),
                error_details: None,
            }),
            Err(e) => Ok(Web3ValidationResult {
                permission: permission.clone(),
                is_valid: false,
                validation_type: Web3ValidationType::TokenGated,
                blockchain_data: None,
                error_details: Some(format!("Blockchain validation failed: {}", e)),
            })
        }
    }
    
    async fn validate_dao_permission(
        &self,
        wallet_address: &WalletAddress,
        permission: &Permission,
        dao_contract: &str,
        min_voting_power: &str,
        chain_id: u64,
        _context: &Web3PermissionContext
    ) -> AppResult<Web3ValidationResult> {
        match self.blockchain_client.validate_dao_voting_power(
            chain_id,
            wallet_address.as_str(),
            dao_contract,
            min_voting_power
        ).await {
            Ok(dao_result) => Ok(Web3ValidationResult {
                permission: permission.clone(),
                is_valid: dao_result.is_valid,
                validation_type: Web3ValidationType::DaoGovernance,
                blockchain_data: Some(format!(
                    "DAO validation for wallet {} on contract {} (chain {}): voting power {} / min {}",
                    wallet_address.as_str(),
                    dao_contract,
                    chain_id,
                    dao_result.voting_power,
                    dao_result.min_voting_power
                )),
                error_details: None,
            }),
            Err(e) => Ok(Web3ValidationResult {
                permission: permission.clone(),
                is_valid: false,
                validation_type: Web3ValidationType::DaoGovernance,
                blockchain_data: None,
                error_details: Some(format!("Blockchain validation failed: {}", e)),
            })
        }
    }
}

/// Context information for permission calculations
#[derive(Debug, Clone)]
pub struct PermissionContext {
    /// Primary blockchain chain ID for the user
    pub primary_chain_id: Option<u64>,
    /// Available RPC endpoints for blockchain queries
    pub rpc_endpoints: HashMap<u64, String>,
    /// Maximum time to wait for blockchain queries
    pub blockchain_timeout_ms: u64,
    /// Current block numbers for each supported chain
    pub block_numbers: HashMap<u64, u64>,
    /// Whether to use cached validation results
    pub use_cache: bool,
    /// Cache TTL in seconds
    pub cache_ttl_seconds: u64,
    /// Cross-chain bridge contracts for multi-chain validation
    pub bridge_contracts: HashMap<u64, Vec<String>>,
}

/// @deprecated Use PermissionContext instead
pub type Web3PermissionContext = PermissionContext;

impl Default for PermissionContext {
    fn default() -> Self {
        Self {
            primary_chain_id: Some(1), // Ethereum mainnet
            rpc_endpoints: HashMap::new(),
            blockchain_timeout_ms: 30000, // 30 seconds
            use_cache: true,
            cache_ttl_seconds: 300, // 5 minutes
            block_numbers: HashMap::new(),
            bridge_contracts: HashMap::new(),
        }
    }
}

/// Result of Web3 permission validation
#[derive(Debug, Clone)]
pub struct Web3ValidationResult {
    pub permission: Permission,
    pub is_valid: bool,
    pub validation_type: Web3ValidationType,
    pub blockchain_data: Option<String>,
    pub error_details: Option<String>,
}

#[derive(Debug, Clone)]
pub enum Web3ValidationType {
    Manual,
    NftGated,
    TokenGated,
    DaoGovernance,
}

/// Result of permission synchronization with blockchain
#[derive(Debug, Clone)]
pub struct PermissionSyncResult {
    pub validated_permissions: Vec<Permission>,
    pub revoked_permissions: Vec<Permission>,
    pub sync_duration_ms: u64,
    pub blockchain_queries_made: u32,
}

impl Default for PermissionSyncResult {
    fn default() -> Self {
        Self::new()
    }
}

impl PermissionSyncResult {
    pub fn new() -> Self {
        Self {
            validated_permissions: Vec::new(),
            revoked_permissions: Vec::new(),
            sync_duration_ms: 0,
            blockchain_queries_made: 0,
        }
    }
}

/// Specification for checking if a wallet user has admin privileges
pub struct IsWalletAdminSpecification;

impl Specification<WalletUser> for IsWalletAdminSpecification {
    fn is_satisfied_by(&self, wallet: &WalletUser) -> bool {
        wallet.is_active() && wallet.permissions()
            .iter()
            .any(|p| p.platform() == "admin" && p.action() == "*")
    }
}

/// Specification for checking if a wallet user has specific platform access
pub struct HasWalletPlatformAccessSpecification {
    platform: String,
}

impl HasWalletPlatformAccessSpecification {
    pub fn new(platform: String) -> Self {
        Self { platform }
    }
}

impl Specification<WalletUser> for HasWalletPlatformAccessSpecification {
    fn is_satisfied_by(&self, wallet: &WalletUser) -> bool {
        wallet.is_active() && wallet.permissions()
            .iter()
            .any(|p| p.platform() == self.platform || p.platform() == "*")
    }
}

/// Specification for checking if a wallet user has Web3 permissions on a specific chain
pub struct HasChainAccessSpecification {
    chain_id: u64,
}

impl HasChainAccessSpecification {
    pub fn new(chain_id: u64) -> Self {
        Self { chain_id }
    }
}

impl Specification<WalletUser> for HasChainAccessSpecification {
    fn is_satisfied_by(&self, wallet: &WalletUser) -> bool {
        wallet.is_active() && wallet.permissions()
            .iter()
            .any(|p| {
                if let Some(permission_chain_id) = p.chain_id() {
                    permission_chain_id == self.chain_id
                } else {
                    false
                }
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    fn create_test_wallet_user() -> WalletUser {
        let mut groups = std::collections::HashSet::new();
        groups.insert("basic".to_string());
        WalletUser::create(
            WalletAddress::new("0x742d35Cc6634C0532925a3b8D369D7763F3c45c6").unwrap(),
            groups,
        ).unwrap()
    }
    
    #[tokio::test]
    async fn generate_default_permissions_for_basic_wallet() {
        let service = WalletPermissionService;
        let wallet = create_test_wallet_user();
        let context = Web3PermissionContext::default();
        
        let permissions = service.generate_default_permissions(&wallet, &context).await.unwrap();
        
        assert!(!permissions.is_empty());
        assert!(permissions.iter().any(|p| p.as_str() == "epsx:analytics:view"));
        assert!(permissions.iter().any(|p| p.as_str() == "epsx:basic:access"));
    }
    
    #[test]
    fn wallet_admin_specification_works() {
        let mut wallet = create_test_wallet_user();
        let admin_spec = IsWalletAdminSpecification;
        
        // Initially not admin
        assert!(!admin_spec.is_satisfied_by(&wallet));
        
        // Grant admin permission
        let admin_perm = Permission::new("admin:*:*").unwrap();
        wallet.grant_permission(admin_perm).unwrap();
        
        // Now should be admin
        assert!(admin_spec.is_satisfied_by(&wallet));
    }
    
    #[test]
    fn chain_access_specification_works() {
        let mut wallet = create_test_wallet_user();
        let ethereum_spec = HasChainAccessSpecification::new(1);
        
        // Initially no chain access
        assert!(!ethereum_spec.is_satisfied_by(&wallet));
        
        // Grant NFT permission on Ethereum
        let nft_perm = Permission::new_nft_gated(
            "epsx:nft:access",
            "0x742d35Cc6634C0532925a3b8D369D7763F3c45c6".to_string(),
            vec![1, 2, 3],
            1, // Ethereum
        ).unwrap();
        wallet.grant_permission(nft_perm).unwrap();
        
        // Now should have Ethereum access
        assert!(ethereum_spec.is_satisfied_by(&wallet));
    }
    
    #[test]
    fn temporary_web3_permissions_creation() {
        let service = WalletPermissionService;
        let mut base_permissions = HashSet::new();
        
        let nft_perm = Permission::new_nft_gated(
            "epsx:temp:nft",
            "0x742d35Cc6634C0532925a3b8D369D7763F3c45c6".to_string(),
            vec![],
            1,
        ).unwrap();
        base_permissions.insert(nft_perm);
        
        let temp_permissions = service.create_temporary_web3_permissions(
            &base_permissions,
            Duration::hours(1)
        ).unwrap();
        
        assert_eq!(temp_permissions.len(), 1);
        let temp_perm = temp_permissions.iter().next().unwrap();
        assert!(temp_perm.expires_at().is_some());
        assert!(temp_perm.is_nft_gated());
    }
}