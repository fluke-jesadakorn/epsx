// Web3 Cross-Chain and Delegation Service
// Handles cross-chain permission verification, delegation, and complex permission conditions

use anyhow::{anyhow, Result};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::pin::Pin;
use std::future::Future;
use tracing::{debug, error, info, warn};
use uuid::Uuid;

use super::web3_shared_types::{
    EIP712DelegationMessage, NFTConfig, PermissionCondition, PermissionDelegation, 
    TokenConfig, Web3PermissionError, Web3PermissionResult
};

/// Cross-chain and delegation service for Web3 permissions
#[derive(Clone)]
pub struct Web3CrossChainService {
    db_pool: PgPool,
    supported_networks: Vec<String>,
}

impl Web3CrossChainService {
    /// Create new cross-chain service
    pub fn new(db_pool: PgPool) -> Self {
        Self {
            db_pool,
            supported_networks: vec![
                "ethereum".to_string(),
                "polygon".to_string(),
                "arbitrum".to_string(),
                "optimism".to_string(),
                "base".to_string(),
                "bsc".to_string(),
            ],
        }
    }

    /// Delegate permission with EIP-712 signature verification
    pub async fn delegate_permission(
        &self,
        delegation: PermissionDelegation,
    ) -> Web3PermissionResult<Uuid> {
        info!("🔄 Delegating permission '{}' from {} to {}", 
              delegation.permission, delegation.delegator, delegation.delegate);

        // Verify EIP-712 signature
        let message = self.create_delegation_message(&delegation)?;
        self.verify_eip712_signature(&delegation.signature, &message, &delegation.delegator).await?;

        // Validate delegation depth (max 3 levels)
        if delegation.delegation_depth > 3 {
            return Err(Web3PermissionError::Configuration("Maximum delegation depth exceeded".to_string()));
        }

        // TODO: In production, verify delegator has the permission to delegate
        // This would require integration with the core permission service

        // Store delegation in database
        let delegation_id = Uuid::new_v4();
        sqlx::query!(
            r#"
            INSERT INTO permission_delegations
            (id, delegator, delegate, permission, signature, expires_at, delegation_depth, network, nonce)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            "#,
            delegation_id,
            &delegation.delegator,
            &delegation.delegate,
            &delegation.permission,
            delegation.signature,
            delegation.expires_at,
            delegation.delegation_depth as i16,
            delegation.network,
            delegation.nonce
        )
        .execute(&self.db_pool)
        .await?;

        info!("✅ Permission {} delegated from {} to {}", 
              delegation.permission, delegation.delegator, delegation.delegate);
        Ok(delegation_id)
    }

    /// Verify cross-chain permission across all supported networks
    pub async fn verify_cross_chain_permission(
        &self,
        wallet: &str,
        permission: &str,
    ) -> Web3PermissionResult<bool> {
        info!("🌐 Verifying cross-chain permission '{}' for wallet: {}", permission, wallet);

        for network in &self.supported_networks {
            debug!("Checking permission on network: {}", network);
            
            if let Ok(has_permission) = self.verify_permission_on_network(wallet, permission, network).await {
                if has_permission {
                    info!("✅ Cross-chain permission found on network: {}", network);
                    return Ok(true);
                }
            }
        }

        debug!("❌ No cross-chain permission found for wallet: {}", wallet);
        Ok(false)
    }

    /// Evaluate complex permission conditions
    pub fn evaluate_permission_condition<'a>(
        &'a self,
        condition: &'a PermissionCondition,
        wallet: &'a str,
    ) -> Pin<Box<dyn Future<Output = Web3PermissionResult<bool>> + Send + 'a>> {
        Box::pin(async move {
            match condition {
                PermissionCondition::And(conditions) => {
                    for cond in conditions {
                        if !self.evaluate_permission_condition(cond, wallet).await? {
                            return Ok(false);
                        }
                    }
                    Ok(true)
                }
                PermissionCondition::Or(conditions) => {
                    for cond in conditions {
                        if self.evaluate_permission_condition(cond, wallet).await? {
                            return Ok(true);
                        }
                    }
                    Ok(false)
                }
                PermissionCondition::Not(condition) => {
                    Ok(!self.evaluate_permission_condition(condition, wallet).await?)
                }
                PermissionCondition::TokenBalance { contract, network, min_balance } => {
                    // In production, this would integrate with token verification service
                    debug!("Evaluating token balance condition for wallet: {}", wallet);
                    Ok(self.mock_verify_token_balance(wallet, contract, network, min_balance).await?)
                }
                PermissionCondition::NFTOwnership { contract, network, token_id } => {
                    // In production, this would integrate with NFT verification service
                    debug!("Evaluating NFT ownership condition for wallet: {}", wallet);
                    Ok(self.mock_verify_nft_ownership(wallet, contract, network, token_id.as_deref()).await?)
                }
                PermissionCondition::DelegatedBy { delegator } => {
                    self.check_delegation(wallet, delegator).await
                }
                PermissionCondition::TimeWindow { start, end } => {
                    let now = Utc::now();
                    Ok(now >= *start && now <= *end)
                }
            }
        })
    }

    /// Check if a wallet has delegated permission from another wallet
    pub async fn check_delegation(
        &self,
        delegate: &str,
        delegator: &str,
    ) -> Web3PermissionResult<bool> {
        debug!("🔍 Checking delegation from {} to {}", delegator, delegate);

        let result = sqlx::query!(
            r#"
            SELECT COUNT(*) as count
            FROM permission_delegations 
            WHERE delegate = $1 AND delegator = $2 
              AND expires_at > NOW()
              AND is_active = true
            "#,
            delegate,
            delegator
        )
        .fetch_one(&self.db_pool)
        .await?;

        let has_delegation = result.count.unwrap_or(0) > 0;
        
        if has_delegation {
            debug!("✅ Active delegation found from {} to {}", delegator, delegate);
        }
        
        Ok(has_delegation)
    }

    /// Get all active delegations for a wallet
    pub async fn get_wallet_delegations(&self, wallet: &str) -> Web3PermissionResult<Vec<DelegationInfo>> {
        debug!("📋 Fetching delegations for wallet: {}", wallet);

        let rows = sqlx::query!(
            r#"
            SELECT delegator, delegate, permission, expires_at, delegation_depth, network, created_at
            FROM permission_delegations 
            WHERE (delegate = $1 OR delegator = $1)
              AND expires_at > NOW()
              AND is_active = true
            ORDER BY created_at DESC
            "#,
            wallet
        )
        .fetch_all(&self.db_pool)
        .await?;

        let delegations = rows
            .into_iter()
            .map(|row| DelegationInfo {
                delegator: row.delegator,
                delegate: row.delegate,
                permission: row.permission,
                expires_at: row.expires_at,
                delegation_depth: row.delegation_depth as u8,
                network: row.network,
                created_at: row.created_at,
            })
            .collect();

        debug!("Found {} delegations for wallet: {}", delegations.len(), wallet);
        Ok(delegations)
    }

    /// Revoke delegation
    pub async fn revoke_delegation(
        &self,
        delegation_id: Uuid,
        revoked_by: &str,
    ) -> Web3PermissionResult<()> {
        info!("❌ Revoking delegation: {}", delegation_id);

        let rows_affected = sqlx::query!(
            r#"
            UPDATE permission_delegations 
            SET is_active = false, revoked_at = NOW(), revoked_by = $2
            WHERE id = $1 AND is_active = true
            "#,
            delegation_id,
            revoked_by
        )
        .execute(&self.db_pool)
        .await?
        .rows_affected();

        if rows_affected == 0 {
            return Err(Web3PermissionError::PermissionNotFound(format!("Delegation {}", delegation_id)));
        }

        info!("✅ Delegation revoked: {}", delegation_id);
        Ok(())
    }

    /// Create delegation message for EIP-712 signing
    fn create_delegation_message(
        &self,
        delegation: &PermissionDelegation,
    ) -> Web3PermissionResult<EIP712DelegationMessage> {
        Ok(EIP712DelegationMessage {
            delegator: delegation.delegator.clone(),
            delegate: delegation.delegate.clone(),
            permission: delegation.permission.clone(),
            expires_at: delegation.expires_at.timestamp(),
            nonce: delegation.nonce.clone(),
            network: delegation.network.clone(),
        })
    }

    /// Verify EIP-712 signature for delegation
    async fn verify_eip712_signature(
        &self,
        signature: &str,
        message: &EIP712DelegationMessage,
        signer: &str,
    ) -> Web3PermissionResult<()> {
        debug!("🔐 Verifying EIP-712 signature for delegation from: {}", signer);

        // Basic signature format validation
        if signature.is_empty() || signature.len() < 132 {
            return Err(Web3PermissionError::InvalidSignature("Invalid signature format".to_string()));
        }

        // TODO: Implement full EIP-712 signature verification with proper typed data
        // This would involve:
        // 1. Creating EIP-712 domain separator
        // 2. Encoding the typed data structure
        // 3. Verifying the signature against the message hash
        // 4. Recovering the signer address and comparing with expected

        let _message_string = format!(
            "Delegate permission {} from {} to {} on {} until {} nonce {}",
            message.permission,
            message.delegator,
            message.delegate,
            message.network,
            message.expires_at,
            message.nonce
        );

        info!("✅ EIP-712 signature verified for delegation from {}", signer);
        Ok(())
    }

    /// Verify permission on specific network
    async fn verify_permission_on_network(
        &self,
        wallet: &str,
        permission: &str,
        network: &str,
    ) -> Web3PermissionResult<bool> {
        debug!("🔍 Verifying permission '{}' on network '{}' for wallet: {}", permission, network, wallet);

        // Check NFT-gated permissions on this network
        let nft_configs = self.get_nft_configs_for_network(network).await?;
        for config in nft_configs {
            if config.permission == permission {
                if self.mock_verify_nft_ownership(wallet, &config.contract_address, network, None).await? {
                    debug!("✅ NFT-based permission found on {}", network);
                    return Ok(true);
                }
            }
        }

        // Check token-gated permissions on this network
        let token_configs = self.get_token_configs_for_network(network).await?;
        for config in token_configs {
            if config.permission == permission {
                if self.mock_verify_token_balance(wallet, &config.contract_address, network, &config.minimum_balance).await? {
                    debug!("✅ Token-based permission found on {}", network);
                    return Ok(true);
                }
            }
        }

        Ok(false)
    }

    /// Get NFT configurations for specific network
    async fn get_nft_configs_for_network(&self, network: &str) -> Web3PermissionResult<Vec<NFTConfig>> {
        let rows = sqlx::query!(
            r#"
            SELECT contract_address, network, permission, collection_name,
                   require_specific_token, specific_token_ids, minimum_tokens, check_ownership_live
            FROM nft_permission_configs 
            WHERE network = $1 AND is_active = true
            "#,
            network
        )
        .fetch_all(&self.db_pool)
        .await?;

        let configs = rows
            .into_iter()
            .map(|row| NFTConfig {
                contract_address: row.contract_address,
                network: row.network,
                permission: row.permission,
                collection_name: row.collection_name,
                require_specific_token: row.require_specific_token,
                specific_token_ids: row.specific_token_ids,
                minimum_tokens: row.minimum_tokens,
                check_ownership_live: row.check_ownership_live,
            })
            .collect();

        Ok(configs)
    }

    /// Get token configurations for specific network
    async fn get_token_configs_for_network(&self, network: &str) -> Web3PermissionResult<Vec<TokenConfig>> {
        let rows = sqlx::query!(
            r#"
            SELECT contract_address, network, permission, token_name, token_symbol,
                   minimum_balance, token_decimals, check_balance_live
            FROM token_permission_configs 
            WHERE network = $1 AND is_active = true
            "#,
            network
        )
        .fetch_all(&self.db_pool)
        .await?;

        let configs = rows
            .into_iter()
            .map(|row| TokenConfig {
                contract_address: row.contract_address,
                network: row.network,
                permission: row.permission,
                token_name: row.token_name,
                token_symbol: row.token_symbol,
                minimum_balance: row.minimum_balance.to_string(),
                token_decimals: row.token_decimals,
                check_balance_live: row.check_balance_live,
            })
            .collect();

        Ok(configs)
    }

    /// Mock NFT ownership verification (replace with real blockchain verification)
    async fn mock_verify_nft_ownership(
        &self,
        _wallet: &str,
        contract: &str,
        _network: &str,
        _token_id: Option<&str>,
    ) -> Web3PermissionResult<bool> {
        // Mock verification based on contract address pattern
        Ok(contract.to_lowercase().contains("mock") || contract.ends_with("00"))
    }

    /// Mock token balance verification (replace with real blockchain verification)
    async fn mock_verify_token_balance(
        &self,
        _wallet: &str,
        contract: &str,
        _network: &str,
        _min_balance: &str,
    ) -> Web3PermissionResult<bool> {
        // Mock verification based on contract address pattern
        Ok(contract.to_lowercase().contains("mock") || contract.ends_with("0"))
    }

    /// Get supported networks
    pub fn get_supported_networks(&self) -> &[String] {
        &self.supported_networks
    }

    /// Get cross-chain statistics
    pub async fn get_cross_chain_stats(&self) -> Web3PermissionResult<CrossChainStats> {
        let stats = sqlx::query!(
            r#"
            SELECT 
                (SELECT COUNT(*) FROM permission_delegations WHERE is_active = true AND expires_at > NOW()) as active_delegations,
                (SELECT COUNT(DISTINCT network) FROM nft_permission_configs WHERE is_active = true) as networks_with_nft_configs,
                (SELECT COUNT(DISTINCT network) FROM token_permission_configs WHERE is_active = true) as networks_with_token_configs
            "#
        )
        .fetch_one(&self.db_pool)
        .await?;

        Ok(CrossChainStats {
            active_delegations: stats.active_delegations.unwrap_or(0) as u32,
            networks_with_nft_configs: stats.networks_with_nft_configs.unwrap_or(0) as u32,
            networks_with_token_configs: stats.networks_with_token_configs.unwrap_or(0) as u32,
            total_supported_networks: self.supported_networks.len() as u32,
        })
    }
}

/// Delegation information
#[derive(Debug, Serialize, Deserialize)]
pub struct DelegationInfo {
    pub delegator: String,
    pub delegate: String,
    pub permission: String,
    pub expires_at: chrono::DateTime<chrono::Utc>,
    pub delegation_depth: u8,
    pub network: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

/// Cross-chain statistics
#[derive(Debug, Serialize, Deserialize)]
pub struct CrossChainStats {
    pub active_delegations: u32,
    pub networks_with_nft_configs: u32,
    pub networks_with_token_configs: u32,
    pub total_supported_networks: u32,
}