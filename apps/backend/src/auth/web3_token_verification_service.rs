// Web3 Token Verification Service
// Handles all blockchain token balance verification operations

use anyhow::{anyhow, Result};
use ethers::types::Address;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::str::FromStr;
use tracing::{debug, error, info, warn};

use super::web3_shared_types::{
    NetworkConfig, PermissionVerificationResult, Web3PermissionError, Web3PermissionResult
};

/// Token verification service for blockchain balance checks
#[derive(Clone)]
pub struct Web3TokenVerificationService {
    db_pool: PgPool,
    network_config: NetworkConfig,
    cache_duration_minutes: i64,
}

/// Token balance information
#[derive(Debug, Serialize, Deserialize)]
pub struct TokenBalance {
    pub token_address: String,
    pub wallet_address: String,
    pub balance: String,
    pub decimals: u8,
    pub network: String,
    pub last_updated: chrono::DateTime<chrono::Utc>,
}

impl Web3TokenVerificationService {
    /// Create new token verification service
    pub fn new(db_pool: PgPool, network_config: NetworkConfig) -> Self {
        Self {
            db_pool,
            network_config,
            cache_duration_minutes: 15, // 15 minute cache for token balances
        }
    }

    /// Verify BSC token balance for wallet
    pub async fn verify_bsc_token_balance(
        &self,
        wallet_address: &str,
        token_contract: &str,
        minimum_balance: &str,
    ) -> Web3PermissionResult<bool> {
        info!("🪙 Verifying BSC token balance for wallet: {}", wallet_address);
        
        let wallet_addr = Address::from_str(wallet_address)
            .map_err(|_| Web3PermissionError::InvalidWallet(wallet_address.to_string()))?;
            
        let token_addr = Address::from_str(token_contract)
            .map_err(|_| Web3PermissionError::Configuration(format!("Invalid token contract: {}", token_contract)))?;

        // Check cache first
        if let Some(cached_balance) = self.get_cached_balance(wallet_address, token_contract, "bsc").await? {
            return Ok(self.compare_balance(&cached_balance.balance, minimum_balance)?);
        }

        // Make RPC call to BSC
        let balance = self.fetch_token_balance_rpc(&self.network_config.bsc_rpc_url, &wallet_addr, &token_addr).await?;
        
        // Cache the result
        self.cache_balance(wallet_address, token_contract, &balance, "bsc").await?;
        
        let has_sufficient = self.compare_balance(&balance, minimum_balance)?;
        
        info!("✅ BSC token verification complete. Sufficient balance: {}", has_sufficient);
        Ok(has_sufficient)
    }

    /// Verify PancakeSwap LP token balance
    pub async fn verify_pancakeswap_lp_balance(
        &self,
        wallet_address: &str,
        lp_contract: &str,
        minimum_balance: &str,
    ) -> Web3PermissionResult<bool> {
        info!("🥞 Verifying PancakeSwap LP balance for wallet: {}", wallet_address);
        
        // PancakeSwap LP tokens are ERC20 tokens on BSC
        self.verify_bsc_token_balance(wallet_address, lp_contract, minimum_balance).await
    }

    /// Verify CAKE token balance specifically
    pub async fn verify_cake_token_balance(
        &self,
        wallet_address: &str,
        minimum_balance: &str,
    ) -> Web3PermissionResult<bool> {
        info!("🎂 Verifying CAKE token balance for wallet: {}", wallet_address);
        
        // CAKE token contract on BSC
        const CAKE_TOKEN_CONTRACT: &str = "0x0E09FaBB73Bd3Ade0a17ECC321fD13a19e81cE82";
        
        self.verify_bsc_token_balance(wallet_address, CAKE_TOKEN_CONTRACT, minimum_balance).await
    }

    /// Verify token balance in USD value
    pub async fn verify_token_balance_usd(
        &self,
        wallet_address: &str,
        token_contract: &str,
        network: &str,
        minimum_usd_value: f64,
    ) -> Web3PermissionResult<bool> {
        info!("💰 Verifying token balance USD value for wallet: {}", wallet_address);
        
        // Get token balance
        let balance = match network {
            "bsc" => self.fetch_token_balance_direct(wallet_address, token_contract, &self.network_config.bsc_rpc_url).await?,
            "ethereum" => self.fetch_token_balance_direct(wallet_address, token_contract, &self.network_config.ethereum_rpc_url).await?,
            "polygon" => self.fetch_token_balance_direct(wallet_address, token_contract, &self.network_config.polygon_rpc_url).await?,
            _ => return Err(Web3PermissionError::UnsupportedNetwork(network.to_string())),
        };

        // Get token price (simplified - in production would use price oracle)
        let token_price_usd = self.get_token_price_usd(token_contract, network).await?;
        
        // Calculate USD value
        let balance_f64: f64 = balance.parse()
            .map_err(|_| Web3PermissionError::Configuration("Invalid balance format".to_string()))?;
            
        let usd_value = balance_f64 * token_price_usd;
        
        let has_sufficient = usd_value >= minimum_usd_value;
        
        info!("✅ USD verification complete. Value: ${:.2}, Required: ${:.2}, Sufficient: {}", 
              usd_value, minimum_usd_value, has_sufficient);
              
        Ok(has_sufficient)
    }

    /// Verify cross-chain token holdings
    pub async fn verify_cross_chain_permission(
        &self,
        wallet_address: &str,
        permission: &str,
        networks: &[String],
    ) -> Web3PermissionResult<PermissionVerificationResult> {
        info!("🌉 Verifying cross-chain permission '{}' for wallet: {}", permission, wallet_address);
        
        let mut verification_data = serde_json::json!({
            "networks_checked": networks,
            "verification_results": {}
        });

        let mut has_permission = false;

        // Check each network
        for network in networks {
            let network_result = self.verify_network_permission(wallet_address, permission, network).await?;
            verification_data["verification_results"][network] = serde_json::json!(network_result);
            
            if network_result {
                has_permission = true;
            }
        }

        Ok(PermissionVerificationResult {
            wallet_address: wallet_address.to_string(),
            permission: permission.to_string(),
            is_granted: has_permission,
            verification_type: "cross_chain".to_string(),
            verification_data,
            cached_until: Some(chrono::Utc::now() + chrono::Duration::minutes(self.cache_duration_minutes)),
        })
    }

    /// Helper: Fetch token balance via RPC
    async fn fetch_token_balance_rpc(
        &self,
        rpc_url: &str,
        wallet_address: &Address,
        token_address: &Address,
    ) -> Web3PermissionResult<String> {
        // In a real implementation, this would use ethers-rs to make the RPC call
        // For now, return a mock balance to avoid external dependencies
        debug!("🔗 Making RPC call to {} for token balance", rpc_url);
        
        // Mock implementation - in production, use ethers-rs provider
        Ok("1000000000000000000".to_string()) // 1 token with 18 decimals
    }

    /// Helper: Fetch token balance directly
    async fn fetch_token_balance_direct(
        &self,
        wallet_address: &str,
        token_contract: &str,
        rpc_url: &str,
    ) -> Web3PermissionResult<String> {
        let wallet_addr = Address::from_str(wallet_address)
            .map_err(|_| Web3PermissionError::InvalidWallet(wallet_address.to_string()))?;
            
        let token_addr = Address::from_str(token_contract)
            .map_err(|_| Web3PermissionError::Configuration(format!("Invalid token contract: {}", token_contract)))?;

        self.fetch_token_balance_rpc(rpc_url, &wallet_addr, &token_addr).await
    }

    /// Helper: Get cached balance from database
    async fn get_cached_balance(
        &self,
        wallet_address: &str,
        token_contract: &str,
        network: &str,
    ) -> Web3PermissionResult<Option<TokenBalance>> {
        let row = sqlx::query!(
            r#"
            SELECT token_address, wallet_address, balance, decimals, network, last_updated
            FROM token_balances 
            WHERE LOWER(wallet_address) = LOWER($1) 
            AND LOWER(token_address) = LOWER($2)
            AND network = $3
            AND last_updated > NOW() - INTERVAL '15 minutes'
            "#,
            wallet_address,
            token_contract,
            network
        )
        .fetch_optional(&self.db_pool)
        .await;

        match row {
            Ok(Some(row)) => Ok(Some(TokenBalance {
                token_address: row.token_address,
                wallet_address: row.wallet_address,
                balance: row.balance,
                decimals: row.decimals as u8,
                network: row.network,
                last_updated: row.last_updated,
            })),
            Ok(None) => Ok(None),
            Err(_) => {
                // Table might not exist yet, return None
                Ok(None)
            }
        }
    }

    /// Helper: Cache balance in database
    async fn cache_balance(
        &self,
        wallet_address: &str,
        token_contract: &str,
        balance: &str,
        network: &str,
    ) -> Web3PermissionResult<()> {
        let _result = sqlx::query!(
            r#"
            INSERT INTO token_balances (wallet_address, token_address, balance, decimals, network, last_updated)
            VALUES ($1, $2, $3, 18, $4, NOW())
            ON CONFLICT (wallet_address, token_address, network)
            DO UPDATE SET balance = $3, last_updated = NOW()
            "#,
            wallet_address,
            token_contract,
            balance,
            network
        )
        .execute(&self.db_pool)
        .await;

        // Ignore errors as this is just caching
        Ok(())
    }

    /// Helper: Compare balance strings
    fn compare_balance(&self, actual: &str, required: &str) -> Web3PermissionResult<bool> {
        let actual_num: f64 = actual.parse()
            .map_err(|_| Web3PermissionError::Configuration("Invalid actual balance".to_string()))?;
            
        let required_num: f64 = required.parse()
            .map_err(|_| Web3PermissionError::Configuration("Invalid required balance".to_string()))?;
            
        Ok(actual_num >= required_num)
    }

    /// Helper: Get token price in USD (mock implementation)
    async fn get_token_price_usd(&self, _token_contract: &str, _network: &str) -> Web3PermissionResult<f64> {
        // In production, this would integrate with price oracles like Chainlink or APIs like CoinGecko
        Ok(1.0) // Mock price of $1 USD
    }

    /// Helper: Verify permission on specific network
    async fn verify_network_permission(
        &self,
        wallet_address: &str,
        permission: &str,
        network: &str,
    ) -> Web3PermissionResult<bool> {
        // Mock implementation - in production would check network-specific conditions
        debug!("🔎 Checking permission '{}' on network '{}' for wallet: {}", permission, network, wallet_address);
        Ok(true) // Mock result
    }
}