// Web3 Configuration Management Service
// Handles NFT configs, token configs, DAO proposals, and automatic permission processing

use anyhow::{anyhow, Result};
use chrono::{DateTime, Utc};
use ethers::types::Address;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::str::FromStr;
use tracing::{debug, error, info, warn};
use uuid::Uuid;

use super::web3_shared_types::{
    DAOProposal, NFTConfig, TokenConfig, Web3PermissionError, Web3PermissionResult
};

/// Configuration management service for Web3 permissions
#[derive(Clone)]
pub struct Web3ConfigService {
    db_pool: PgPool,
}

impl Web3ConfigService {
    /// Create new configuration service
    pub fn new(db_pool: PgPool) -> Self {
        Self { db_pool }
    }

    /// Configure NFT-gated permission
    pub async fn configure_nft_permission(
        &self,
        config: NFTConfig,
        created_by: Uuid,
    ) -> Web3PermissionResult<Uuid> {
        info!("🖼️ Configuring NFT permission for contract: {}", config.contract_address);
        
        let address = Address::from_str(&config.contract_address)
            .map_err(|_| Web3PermissionError::Configuration("Invalid contract address format".to_string()))?;
        let contract_address = address.to_string().to_lowercase();

        let config_id = Uuid::new_v4();

        sqlx::query!(
            r#"
            INSERT INTO nft_permission_configs
            (id, contract_address, network, permission, collection_name,
             require_specific_token, specific_token_ids, minimum_tokens,
             check_ownership_live, created_by)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
            "#,
            config_id,
            &contract_address,
            &config.network,
            config.permission,
            config.collection_name,
            config.require_specific_token,
            &config.specific_token_ids,
            config.minimum_tokens,
            config.check_ownership_live,
            created_by
        )
        .execute(&self.db_pool)
        .await?;

        info!("✅ Configured NFT permission for contract {} on {}", contract_address, config.network);
        Ok(config_id)
    }

    /// Configure token-gated permission
    pub async fn configure_token_permission(
        &self,
        config: TokenConfig,
        created_by: Uuid,
    ) -> Web3PermissionResult<Uuid> {
        info!("🪙 Configuring token permission for contract: {}", config.contract_address);
        
        let address = Address::from_str(&config.contract_address)
            .map_err(|_| Web3PermissionError::Configuration("Invalid contract address format".to_string()))?;
        let contract_address = address.to_string().to_lowercase();

        let config_id = Uuid::new_v4();

        // Parse minimum balance as BigDecimal for database storage
        let minimum_balance = sqlx::types::BigDecimal::from_str(&config.minimum_balance)
            .map_err(|_| Web3PermissionError::Configuration("Invalid balance format".to_string()))?;

        sqlx::query!(
            r#"
            INSERT INTO token_permission_configs
            (id, contract_address, network, permission, token_name, token_symbol,
             token_decimals, minimum_balance, check_balance_live, created_by)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
            "#,
            config_id,
            &contract_address,
            &config.network,
            config.permission,
            config.token_name,
            config.token_symbol,
            config.token_decimals,
            minimum_balance,
            config.check_balance_live,
            created_by
        )
        .execute(&self.db_pool)
        .await?;

        info!("✅ Configured token permission for contract {} on {}", contract_address, config.network);
        Ok(config_id)
    }

    /// Create DAO permission proposal
    pub async fn create_dao_proposal(&self, proposal: DAOProposal) -> Web3PermissionResult<Uuid> {
        info!("🏛️ Creating DAO proposal: {}", proposal.title);
        
        let dao_address = Address::from_str(&proposal.dao_contract_address)
            .map_err(|_| Web3PermissionError::Configuration("Invalid DAO contract address format".to_string()))?;
        let dao_contract = dao_address.to_string().to_lowercase();

        let target_address = Address::from_str(&proposal.target_wallet_address)
            .map_err(|_| Web3PermissionError::Configuration("Invalid target wallet address format".to_string()))?;
        let target_wallet = target_address.to_string().to_lowercase();

        let proposal_id = Uuid::new_v4();

        sqlx::query!(
            r#"
            INSERT INTO dao_permission_proposals
            (id, dao_contract_address, network, proposal_id, title, description,
             proposer_address, target_wallet_address, permission, proposal_status, voting_end)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, 'active', $10)
            "#,
            proposal_id,
            &dao_contract,
            proposal.network,
            &proposal.proposal_id,
            proposal.title,
            proposal.description,
            &dao_contract, // Assuming proposer is the DAO contract for now
            target_wallet,
            &proposal.permission,
            proposal.voting_end
        )
        .execute(&self.db_pool)
        .await?;

        info!("✅ Created DAO proposal {} for permission {}", proposal.proposal_id, proposal.permission);
        Ok(proposal_id)
    }

    /// Process automatic permission granting based on configurations
    pub async fn process_automatic_permissions(
        &self,
        wallet_address: &str,
    ) -> Web3PermissionResult<Vec<String>> {
        info!("⚡ Processing automatic permissions for wallet: {}", wallet_address);
        
        let address = Address::from_str(wallet_address)
            .map_err(|_| Web3PermissionError::InvalidWallet(wallet_address.to_string()))?;
        let normalized_wallet = address.to_string().to_lowercase();

        let mut granted_permissions = Vec::new();

        // Check NFT-gated permissions
        let nft_configs = self.get_active_nft_configs().await?;
        for config in nft_configs {
            // In production, this would verify actual NFT ownership via blockchain RPC
            if self.verify_nft_ownership_mock(&normalized_wallet, &config).await? {
                self.grant_nft_permission(&normalized_wallet, &config).await?;
                granted_permissions.push(config.permission);
            }
        }

        // Check token-gated permissions
        let token_configs = self.get_active_token_configs().await?;
        for config in token_configs {
            // In production, this would verify actual token balance via blockchain RPC
            if self.verify_token_balance_mock(&normalized_wallet, &config).await? {
                self.grant_token_permission(&normalized_wallet, &config).await?;
                granted_permissions.push(config.permission);
            }
        }

        if !granted_permissions.is_empty() {
            info!("✅ Automatically granted {} permissions to wallet {}", granted_permissions.len(), wallet_address);
        } else {
            debug!("No automatic permissions granted for wallet: {}", wallet_address);
        }

        Ok(granted_permissions)
    }

    /// Get all active NFT permission configurations
    pub async fn get_active_nft_configs(&self) -> Web3PermissionResult<Vec<NFTConfig>> {
        debug!("📋 Fetching active NFT configurations");
        
        let rows = sqlx::query!(
            r#"
            SELECT contract_address, network, permission, collection_name,
                   require_specific_token, specific_token_ids, minimum_tokens, check_ownership_live
            FROM nft_permission_configs 
            WHERE is_active = true
            "#
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

        debug!("Found {} active NFT configurations", configs.len());
        Ok(configs)
    }

    /// Get all active token permission configurations
    pub async fn get_active_token_configs(&self) -> Web3PermissionResult<Vec<TokenConfig>> {
        debug!("📋 Fetching active token configurations");
        
        let rows = sqlx::query!(
            r#"
            SELECT contract_address, network, permission, token_name, token_symbol,
                   minimum_balance, token_decimals, check_balance_live
            FROM token_permission_configs 
            WHERE is_active = true
            "#
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

        debug!("Found {} active token configurations", configs.len());
        Ok(configs)
    }

    /// Grant NFT-based permission to wallet
    async fn grant_nft_permission(
        &self,
        wallet_address: &str,
        config: &NFTConfig,
    ) -> Web3PermissionResult<()> {
        debug!("🖼️ Granting NFT permission {} to wallet: {}", config.permission, wallet_address);
        
        sqlx::query!(
            r#"
            INSERT INTO wallet_permissions 
            (wallet_address, permission, permission_type, nft_contract_address, nft_network)
            VALUES ($1, $2, $3, $4, $5)
            ON CONFLICT DO NOTHING
            "#,
            wallet_address,
            &config.permission,
            "nft_gated",
            &config.contract_address,
            &config.network
        )
        .execute(&self.db_pool)
        .await?;

        Ok(())
    }

    /// Grant token-based permission to wallet
    async fn grant_token_permission(
        &self,
        wallet_address: &str,
        config: &TokenConfig,
    ) -> Web3PermissionResult<()> {
        debug!("🪙 Granting token permission {} to wallet: {}", config.permission, wallet_address);
        
        let minimum_balance = sqlx::types::BigDecimal::from_str(&config.minimum_balance)
            .map_err(|_| Web3PermissionError::Configuration("Invalid balance format".to_string()))?;

        sqlx::query!(
            r#"
            INSERT INTO wallet_permissions 
            (wallet_address, permission, permission_type, token_contract_address, token_network, 
             required_balance, token_decimals)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            ON CONFLICT DO NOTHING
            "#,
            wallet_address,
            &config.permission,
            "token_gated",
            &config.contract_address,
            &config.network,
            minimum_balance,
            config.token_decimals
        )
        .execute(&self.db_pool)
        .await?;

        Ok(())
    }

    /// Mock NFT ownership verification (replace with real blockchain verification)
    async fn verify_nft_ownership_mock(
        &self,
        _wallet_address: &str,
        config: &NFTConfig,
    ) -> Web3PermissionResult<bool> {
        debug!("🔍 Mock NFT verification for contract: {}", config.contract_address);
        
        // In production, this would:
        // 1. Call the NFT contract's balanceOf or ownerOf functions
        // 2. Verify the wallet owns required number of tokens
        // 3. Check specific token IDs if required
        
        // For now, return mock result based on contract address pattern
        let mock_owns_nft = config.contract_address.to_lowercase().contains("mock") 
            || config.contract_address.ends_with("00");
            
        Ok(mock_owns_nft)
    }

    /// Mock token balance verification (replace with real blockchain verification)
    async fn verify_token_balance_mock(
        &self,
        _wallet_address: &str,
        config: &TokenConfig,
    ) -> Web3PermissionResult<bool> {
        debug!("🔍 Mock token verification for contract: {}", config.contract_address);
        
        // In production, this would:
        // 1. Call the ERC20 contract's balanceOf function
        // 2. Compare balance against minimum required
        // 3. Account for token decimals
        
        // For now, return mock result based on contract address pattern
        let mock_has_balance = config.contract_address.to_lowercase().contains("mock")
            || config.contract_address.ends_with("0");
            
        Ok(mock_has_balance)
    }

    /// Disable NFT configuration
    pub async fn disable_nft_config(&self, config_id: Uuid) -> Web3PermissionResult<()> {
        info!("❌ Disabling NFT configuration: {}", config_id);
        
        let rows_affected = sqlx::query!(
            "UPDATE nft_permission_configs SET is_active = false WHERE id = $1",
            config_id
        )
        .execute(&self.db_pool)
        .await?
        .rows_affected();

        if rows_affected == 0 {
            return Err(Web3PermissionError::PermissionNotFound(format!("NFT config {}", config_id)));
        }

        info!("✅ NFT configuration disabled: {}", config_id);
        Ok(())
    }

    /// Disable token configuration
    pub async fn disable_token_config(&self, config_id: Uuid) -> Web3PermissionResult<()> {
        info!("❌ Disabling token configuration: {}", config_id);
        
        let rows_affected = sqlx::query!(
            "UPDATE token_permission_configs SET is_active = false WHERE id = $1",
            config_id
        )
        .execute(&self.db_pool)
        .await?
        .rows_affected();

        if rows_affected == 0 {
            return Err(Web3PermissionError::PermissionNotFound(format!("Token config {}", config_id)));
        }

        info!("✅ Token configuration disabled: {}", config_id);
        Ok(())
    }

    /// Get configuration statistics
    pub async fn get_config_stats(&self) -> Web3PermissionResult<ConfigStats> {
        let stats = sqlx::query!(
            r#"
            SELECT 
                (SELECT COUNT(*) FROM nft_permission_configs WHERE is_active = true) as active_nft_configs,
                (SELECT COUNT(*) FROM token_permission_configs WHERE is_active = true) as active_token_configs,
                (SELECT COUNT(*) FROM dao_permission_proposals WHERE proposal_status = 'active') as active_dao_proposals
            "#
        )
        .fetch_one(&self.db_pool)
        .await?;

        Ok(ConfigStats {
            active_nft_configs: stats.active_nft_configs.unwrap_or(0) as u32,
            active_token_configs: stats.active_token_configs.unwrap_or(0) as u32,
            active_dao_proposals: stats.active_dao_proposals.unwrap_or(0) as u32,
        })
    }
}

/// Configuration statistics
#[derive(Debug, Serialize, Deserialize)]
pub struct ConfigStats {
    pub active_nft_configs: u32,
    pub active_token_configs: u32,
    pub active_dao_proposals: u32,
}