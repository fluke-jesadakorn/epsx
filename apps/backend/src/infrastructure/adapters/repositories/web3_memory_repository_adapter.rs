// In-Memory Web3 Repository Adapters (Temporary Implementation)
// These adapters provide working implementations without database dependencies
// TODO: Replace with proper database implementations once schema is ready

use std::{sync::Arc, collections::HashMap};
use anyhow::Result;
use chrono::Utc;
use tokio::sync::RwLock;
use tracing::info;

use crate::domain::{
    shared_kernel::{domain_error::DomainError, value_objects::UserId},
    authentication::{
        Web3Challenge, Web3Permission, Web3PermissionType,
        Web3ChallengeRepositoryPort, Web3UserRepositoryPort, Web3PermissionRepositoryPort,
    },
};

/// In-memory Web3 challenge repository
pub struct InMemoryWeb3ChallengeRepository {
    challenges: Arc<RwLock<HashMap<String, Web3Challenge>>>,
}

impl InMemoryWeb3ChallengeRepository {
    pub fn new() -> Self {
        Self {
            challenges: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

#[async_trait::async_trait]
impl Web3ChallengeRepositoryPort for InMemoryWeb3ChallengeRepository {
    async fn store_challenge(&self, challenge: &Web3Challenge) -> Result<(), DomainError> {
        let mut challenges = self.challenges.write().await;
        challenges.insert(challenge.nonce.clone(), challenge.clone());
        info!("Stored Web3 challenge for wallet: {}", challenge.wallet_address);
        Ok(())
    }

    async fn get_challenge(&self, nonce: &str) -> Result<Option<Web3Challenge>, DomainError> {
        let challenges = self.challenges.read().await;
        Ok(challenges.get(nonce).cloned())
    }

    async fn mark_challenge_used(&self, nonce: &str) -> Result<(), DomainError> {
        let mut challenges = self.challenges.write().await;
        if let Some(challenge) = challenges.get_mut(nonce) {
            challenge.used = true;
            info!("Marked Web3 challenge as used: {}", nonce);
            Ok(())
        } else {
            Err(DomainError::entity_not_found("Web3Challenge", nonce))
        }
    }

    async fn cleanup_expired_challenges(&self) -> Result<u64, DomainError> {
        let mut challenges = self.challenges.write().await;
        let now = Utc::now();
        let initial_count = challenges.len();
        
        challenges.retain(|_, challenge| !challenge.used && challenge.expires_at > now);
        
        let deleted_count = (initial_count - challenges.len()) as u64;
        info!("Cleaned up {} expired Web3 challenges", deleted_count);
        Ok(deleted_count)
    }
}

/// In-memory Web3 user repository
pub struct InMemoryWeb3UserRepository {
    wallet_to_user: Arc<RwLock<HashMap<String, UserId>>>,
    user_wallets: Arc<RwLock<HashMap<String, Vec<String>>>>, // user_id -> wallets
}

impl InMemoryWeb3UserRepository {
    pub fn new() -> Self {
        Self {
            wallet_to_user: Arc::new(RwLock::new(HashMap::new())),
            user_wallets: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

#[async_trait::async_trait]
impl Web3UserRepositoryPort for InMemoryWeb3UserRepository {
    async fn find_by_wallet(&self, wallet_address: &str) -> Result<Option<UserId>, DomainError> {
        let wallet_to_user = self.wallet_to_user.read().await;
        Ok(wallet_to_user.get(&wallet_address.to_lowercase()).cloned())
    }

    async fn create_user(&self, wallet_address: &str) -> Result<UserId, DomainError> {
        let user_id = UserId::new();
        let user_id_str = user_id.to_string();
        
        {
            let mut wallet_to_user = self.wallet_to_user.write().await;
            wallet_to_user.insert(wallet_address.to_lowercase(), user_id.clone());
        }
        
        {
            let mut user_wallets = self.user_wallets.write().await;
            user_wallets.insert(user_id_str.clone(), vec![wallet_address.to_lowercase()]);
        }
        
        info!("Created new user {} for wallet: {}", user_id_str, wallet_address);
        Ok(user_id)
    }

    async fn link_wallet_to_user(&self, user_id: UserId, wallet_address: &str) -> Result<(), DomainError> {
        let user_id_str = user_id.to_string();
        let wallet_lower = wallet_address.to_lowercase();
        
        // Check if wallet is already linked to another user
        {
            let wallet_to_user = self.wallet_to_user.read().await;
            if let Some(existing_user_id) = wallet_to_user.get(&wallet_lower) {
                if existing_user_id.to_string() != user_id_str {
                    return Err(DomainError::resource_conflict(
                        "wallet",
                        "Wallet is already linked to another user"
                    ));
                }
                // Already linked to this user
                return Ok(());
            }
        }

        // Link wallet to user
        {
            let mut wallet_to_user = self.wallet_to_user.write().await;
            wallet_to_user.insert(wallet_lower.clone(), user_id.clone());
        }
        
        {
            let mut user_wallets = self.user_wallets.write().await;
            let wallets = user_wallets.entry(user_id_str.clone()).or_insert_with(Vec::new);
            if !wallets.contains(&wallet_lower) {
                wallets.push(wallet_lower);
            }
        }
        
        info!("Linked wallet {} to user {}", wallet_address, user_id_str);
        Ok(())
    }
}

/// In-memory Web3 permission repository
pub struct InMemoryWeb3PermissionRepository {
    permissions: Arc<RwLock<HashMap<String, Web3Permission>>>, // "{user_id}:{permission_name}" -> permission
    configs: Arc<RwLock<HashMap<String, Web3PermissionType>>>, // permission_name -> config
}

impl InMemoryWeb3PermissionRepository {
    pub fn new() -> Self {
        let mut configs = HashMap::new();
        
        // Add some default permission configurations for testing
        configs.insert(
            "premium_access".to_string(),
            Web3PermissionType::NftGated {
                contract_address: "0x1234567890123456789012345678901234567890".to_string(),
                network: "ethereum".to_string(),
                required_token_ids: None,
                minimum_count: 1,
            }
        );
        
        configs.insert(
            "governance_member".to_string(),
            Web3PermissionType::TokenGated {
                contract_address: "0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa".to_string(),
                network: "ethereum".to_string(),
                minimum_balance: "1000000000000000000000".to_string(), // 1000 tokens
            }
        );
        
        Self {
            permissions: Arc::new(RwLock::new(HashMap::new())),
            configs: Arc::new(RwLock::new(configs)),
        }
    }
    
    fn permission_key(user_id: &UserId, permission_name: &str) -> String {
        format!("{}:{}", user_id.to_string(), permission_name)
    }
}

#[async_trait::async_trait]
impl Web3PermissionRepositoryPort for InMemoryWeb3PermissionRepository {
    async fn get_user_permission(&self, user_id: &UserId, permission_name: &str) -> Result<Option<Web3Permission>, DomainError> {
        let permissions = self.permissions.read().await;
        let key = Self::permission_key(user_id, permission_name);
        Ok(permissions.get(&key).cloned())
    }

    async fn get_user_permissions(&self, user_id: &UserId) -> Result<Vec<Web3Permission>, DomainError> {
        let permissions = self.permissions.read().await;
        let user_id_str = user_id.to_string();
        let user_permissions: Vec<Web3Permission> = permissions
            .values()
            .filter(|p| p.user_id.to_string() == user_id_str)
            .cloned()
            .collect();
        Ok(user_permissions)
    }

    async fn get_permission_config(&self, permission_name: &str) -> Result<Option<Web3PermissionType>, DomainError> {
        let configs = self.configs.read().await;
        Ok(configs.get(permission_name).cloned())
    }

    async fn store_permission(&self, permission: &Web3Permission) -> Result<(), DomainError> {
        let mut permissions = self.permissions.write().await;
        let key = Self::permission_key(&permission.user_id, &permission.permission_name);
        permissions.insert(key, permission.clone());
        info!("Stored Web3 permission {} for user {}", permission.permission_name, permission.user_id.to_string());
        Ok(())
    }

    async fn revoke_permission(&self, user_id: &UserId, permission_name: &str) -> Result<(), DomainError> {
        let mut permissions = self.permissions.write().await;
        let key = Self::permission_key(user_id, permission_name);
        if let Some(permission) = permissions.get_mut(&key) {
            permission.is_active = false;
            info!("Revoked permission {} for user {}", permission_name, user_id.to_string());
            Ok(())
        } else {
            Err(DomainError::entity_not_found("Web3Permission", key))
        }
    }
}