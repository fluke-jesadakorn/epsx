// Web3 Permission Service (Domain Service)
// Handles blockchain-based permission verification (NFT, Token, DAO, Manual)
// Pure domain logic following DDD principles

use std::{sync::Arc, collections::HashMap};
use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::domain::{
    shared_kernel::{
        domain_error::DomainError,
        value_objects::UserId,
    },
};

/// Web3 Permission Domain Service
/// Verifies permissions based on blockchain assets (NFTs, tokens, DAO membership)
pub struct Web3PermissionService {
    permission_repository: Arc<dyn Web3PermissionRepositoryPort>,
    blockchain_service: Arc<dyn BlockchainServicePort>,
}

/// Web3 Permission types supported
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Web3PermissionType {
    NftGated {
        contract_address: String,
        network: String,
        required_token_ids: Option<Vec<String>>,
        minimum_count: u32,
    },
    TokenGated {
        contract_address: String,
        network: String,
        minimum_balance: String, // Wei/smallest unit
    },
    DaoGoverned {
        dao_address: String,
        network: String,
        proposal_id: Option<String>,
        minimum_voting_power: Option<String>,
    },
    Manual {
        granted_by: UserId,
        reason: String,
    },
}

/// Web3 Permission (Domain Entity)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Web3Permission {
    pub permission_name: String,
    pub permission_type: Web3PermissionType,
    pub user_id: UserId,
    pub wallet_address: String,
    pub granted_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
    pub is_active: bool,
    pub last_verified_at: Option<DateTime<Utc>>,
    pub verification_metadata: HashMap<String, String>,
}

/// Permission verification request (Value Object)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PermissionVerificationRequest {
    pub user_id: UserId,
    pub wallet_address: String,
    pub permission_name: String,
    pub force_blockchain_check: bool,
}

/// Permission verification result (Value Object)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PermissionVerificationResult {
    pub has_permission: bool,
    pub permission_source: Option<Web3PermissionType>,
    pub verified_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
    pub metadata: HashMap<String, String>,
    pub verification_method: VerificationMethod,
}

/// How the permission was verified
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum VerificationMethod {
    DatabaseCache,
    BlockchainVerification,
    ManualGrant,
}

/// Web3 Permission Errors
#[derive(Debug, Error)]
pub enum Web3PermissionError {
    #[error("Permission not found: {permission_name}")]
    PermissionNotFound { permission_name: String },
    
    #[error("Blockchain verification failed: {reason}")]
    BlockchainVerificationFailed { reason: String },
    
    #[error("Permission expired")]
    PermissionExpired,
    
    #[error("Invalid permission configuration")]
    InvalidConfiguration,
    
    #[error("Domain error: {0}")]
    DomainError(#[from] DomainError),
}

impl Web3PermissionService {
    pub fn new(
        permission_repository: Arc<dyn Web3PermissionRepositoryPort>,
        blockchain_service: Arc<dyn BlockchainServicePort>,
    ) -> Self {
        Self {
            permission_repository,
            blockchain_service,
        }
    }

    /// Verify if user has a specific permission
    pub async fn verify_permission(&self, request: PermissionVerificationRequest) -> Result<PermissionVerificationResult, Web3PermissionError> {
        // First check cache/database
        if let Some(cached_permission) = self.permission_repository
            .get_user_permission(&request.user_id, &request.permission_name)
            .await? 
        {
            if !request.force_blockchain_check && self.is_cache_valid(&cached_permission) {
                return Ok(PermissionVerificationResult {
                    has_permission: cached_permission.is_active && !self.is_expired(&cached_permission),
                    permission_source: Some(cached_permission.permission_type),
                    verified_at: cached_permission.last_verified_at.unwrap_or(cached_permission.granted_at),
                    expires_at: cached_permission.expires_at,
                    metadata: cached_permission.verification_metadata,
                    verification_method: VerificationMethod::DatabaseCache,
                });
            }
        }

        // Get permission configuration
        let permission_config = self.permission_repository
            .get_permission_config(&request.permission_name)
            .await?
            .ok_or_else(|| Web3PermissionError::PermissionNotFound {
                permission_name: request.permission_name.clone(),
            })?;

        // Verify on blockchain
        let verification_result = self.verify_on_blockchain(&request, &permission_config).await?;

        // Update cache
        if verification_result.has_permission {
            self.update_permission_cache(&request, &permission_config, &verification_result).await?;
        }

        Ok(verification_result)
    }

    /// Grant manual permission (admin function)
    pub async fn grant_manual_permission(
        &self,
        user_id: UserId,
        wallet_address: String,
        permission_name: String,
        granted_by: UserId,
        reason: String,
        expires_at: Option<DateTime<Utc>>,
    ) -> Result<Web3Permission, Web3PermissionError> {
        let permission = Web3Permission {
            permission_name,
            permission_type: Web3PermissionType::Manual { granted_by, reason },
            user_id,
            wallet_address,
            granted_at: Utc::now(),
            expires_at,
            is_active: true,
            last_verified_at: Some(Utc::now()),
            verification_metadata: HashMap::new(),
        };

        self.permission_repository.store_permission(&permission).await?;
        Ok(permission)
    }

    /// Revoke permission
    pub async fn revoke_permission(&self, user_id: UserId, permission_name: String) -> Result<(), Web3PermissionError> {
        self.permission_repository.revoke_permission(&user_id, &permission_name).await?;
        Ok(())
    }

    /// Get all permissions for a user
    pub async fn get_user_permissions(&self, user_id: UserId) -> Result<Vec<Web3Permission>, Web3PermissionError> {
        let permissions = self.permission_repository.get_user_permissions(&user_id).await?;
        Ok(permissions.into_iter().filter(|p| !self.is_expired(p)).collect())
    }

    // Private helper methods
    fn is_cache_valid(&self, permission: &Web3Permission) -> bool {
        if let Some(last_verified) = permission.last_verified_at {
            let cache_duration = chrono::Duration::hours(1); // 1 hour cache
            Utc::now() - last_verified < cache_duration
        } else {
            false
        }
    }

    fn is_expired(&self, permission: &Web3Permission) -> bool {
        if let Some(expires_at) = permission.expires_at {
            Utc::now() > expires_at
        } else {
            false
        }
    }

    async fn verify_on_blockchain(
        &self,
        request: &PermissionVerificationRequest,
        config: &Web3PermissionType,
    ) -> Result<PermissionVerificationResult, Web3PermissionError> {
        match config {
            Web3PermissionType::NftGated { contract_address, network, required_token_ids, minimum_count } => {
                let nft_balance = self.blockchain_service
                    .get_nft_balance(&request.wallet_address, contract_address, network)
                    .await?;

                let has_permission = if let Some(token_ids) = required_token_ids {
                    // Check for specific token IDs
                    token_ids.iter().any(|id| nft_balance.owned_tokens.contains(id))
                } else {
                    // Check minimum count
                    nft_balance.total_count >= *minimum_count
                };

                Ok(PermissionVerificationResult {
                    has_permission,
                    permission_source: Some(config.clone()),
                    verified_at: Utc::now(),
                    expires_at: None, // NFT permissions don't expire unless the NFT is transferred
                    metadata: [
                        ("nft_count".to_string(), nft_balance.total_count.to_string()),
                        ("contract".to_string(), contract_address.clone()),
                        ("network".to_string(), network.clone()),
                    ].into_iter().collect(),
                    verification_method: VerificationMethod::BlockchainVerification,
                })
            },
            
            Web3PermissionType::TokenGated { contract_address, network, minimum_balance } => {
                let token_balance = self.blockchain_service
                    .get_token_balance(&request.wallet_address, contract_address, network)
                    .await?;

                let min_balance: u128 = minimum_balance.parse()
                    .map_err(|_| Web3PermissionError::InvalidConfiguration)?;

                Ok(PermissionVerificationResult {
                    has_permission: token_balance.balance >= min_balance,
                    permission_source: Some(config.clone()),
                    verified_at: Utc::now(),
                    expires_at: None,
                    metadata: [
                        ("token_balance".to_string(), token_balance.balance.to_string()),
                        ("minimum_required".to_string(), minimum_balance.clone()),
                        ("contract".to_string(), contract_address.clone()),
                        ("network".to_string(), network.clone()),
                    ].into_iter().collect(),
                    verification_method: VerificationMethod::BlockchainVerification,
                })
            },
            
            Web3PermissionType::DaoGoverned { dao_address, network, proposal_id: _, minimum_voting_power } => {
                let dao_membership = self.blockchain_service
                    .get_dao_membership(&request.wallet_address, dao_address, network)
                    .await?;

                let has_permission = if let Some(min_power) = minimum_voting_power {
                    let min_power: u128 = min_power.parse()
                        .map_err(|_| Web3PermissionError::InvalidConfiguration)?;
                    dao_membership.voting_power >= min_power
                } else {
                    dao_membership.is_member
                };

                Ok(PermissionVerificationResult {
                    has_permission,
                    permission_source: Some(config.clone()),
                    verified_at: Utc::now(),
                    expires_at: None,
                    metadata: [
                        ("voting_power".to_string(), dao_membership.voting_power.to_string()),
                        ("is_member".to_string(), dao_membership.is_member.to_string()),
                        ("dao_address".to_string(), dao_address.clone()),
                        ("network".to_string(), network.clone()),
                    ].into_iter().collect(),
                    verification_method: VerificationMethod::BlockchainVerification,
                })
            },
            
            Web3PermissionType::Manual { .. } => {
                Ok(PermissionVerificationResult {
                    has_permission: true,
                    permission_source: Some(config.clone()),
                    verified_at: Utc::now(),
                    expires_at: None,
                    metadata: HashMap::new(),
                    verification_method: VerificationMethod::ManualGrant,
                })
            },
        }
    }

    async fn update_permission_cache(
        &self,
        request: &PermissionVerificationRequest,
        config: &Web3PermissionType,
        result: &PermissionVerificationResult,
    ) -> Result<(), Web3PermissionError> {
        let permission = Web3Permission {
            permission_name: request.permission_name.clone(),
            permission_type: config.clone(),
            user_id: request.user_id.clone(),
            wallet_address: request.wallet_address.clone(),
            granted_at: Utc::now(),
            expires_at: result.expires_at,
            is_active: result.has_permission,
            last_verified_at: Some(result.verified_at),
            verification_metadata: result.metadata.clone(),
        };

        self.permission_repository.store_permission(&permission).await?;
        Ok(())
    }
}

/// Repository port for Web3 permissions (Hexagonal Architecture)
#[async_trait::async_trait]
pub trait Web3PermissionRepositoryPort: Send + Sync {
    async fn get_user_permission(&self, user_id: &UserId, permission_name: &str) -> Result<Option<Web3Permission>, DomainError>;
    async fn get_user_permissions(&self, user_id: &UserId) -> Result<Vec<Web3Permission>, DomainError>;
    async fn get_permission_config(&self, permission_name: &str) -> Result<Option<Web3PermissionType>, DomainError>;
    async fn store_permission(&self, permission: &Web3Permission) -> Result<(), DomainError>;
    async fn revoke_permission(&self, user_id: &UserId, permission_name: &str) -> Result<(), DomainError>;
}

/// Blockchain service port for Web3 verification (Hexagonal Architecture)
#[async_trait::async_trait]
pub trait BlockchainServicePort: Send + Sync {
    async fn get_nft_balance(&self, wallet_address: &str, contract_address: &str, network: &str) -> Result<NftBalance, DomainError>;
    async fn get_token_balance(&self, wallet_address: &str, contract_address: &str, network: &str) -> Result<TokenBalance, DomainError>;
    async fn get_dao_membership(&self, wallet_address: &str, dao_address: &str, network: &str) -> Result<DaoMembership, DomainError>;
}

/// NFT balance information
#[derive(Debug, Clone)]
pub struct NftBalance {
    pub total_count: u32,
    pub owned_tokens: Vec<String>,
}

/// Token balance information  
#[derive(Debug, Clone)]
pub struct TokenBalance {
    pub balance: u128,
    pub decimals: u8,
}

/// DAO membership information
#[derive(Debug, Clone)]
pub struct DaoMembership {
    pub is_member: bool,
    pub voting_power: u128,
}