// Wallet Verification and User Lifecycle
// Handles: get_or_create_user, emit_new_wallet_event, assign_free_plan_to_wallet
// Also handles explicitly configured blockchain permission queries: NFT and DAO
//
// MIGRATED TO SQLX (real): no stubs.

use chrono::Utc;
use ethers::{
    abi::Abi,
    contract::Contract,
    providers::{Http, Provider},
    types::{Address, U256},
};
use std::str::FromStr;
use std::sync::Arc;
use tracing::{debug, info, warn};
use uuid::Uuid;

use super::auth_service::{UnifiedWeb3AuthService, Web3AuthError};
use crate::config::get_bsc_chain_id;

impl UnifiedWeb3AuthService {
    /// Get or create user for wallet
    pub(super) async fn get_or_create_user(
        &self,
        wallet_address: &str,
    ) -> Result<(String, bool), Web3AuthError> {
        let wallet_address = wallet_address.trim().to_lowercase();
        let wallet_address = wallet_address.as_str();

        let exists: Option<(String,)> = sqlx::query_as(
            "SELECT wallet_address FROM wallet_users WHERE wallet_address = $1",
        )
        .bind(wallet_address)
        .fetch_optional(&*self.db_pool)
        .await
        .map_err(|e| Web3AuthError::DatabaseError(e.to_string()))?;

        if exists.is_some() {
            let now = Utc::now();
            sqlx::query(
                "UPDATE wallet_users SET last_auth_at = $1, updated_at = $1 WHERE wallet_address = $2",
            )
            .bind(now)
            .bind(wallet_address)
            .execute(&*self.db_pool)
            .await
            .map_err(|e| Web3AuthError::DatabaseError(e.to_string()))?;

            debug!("Updated existing wallet user activity: {}", wallet_address);
            return Ok((wallet_address.to_string(), false));
        }

        let blockchain_network = std::env::var("NEXT_PUBLIC_BLOCKCHAIN_NETWORK")
            .unwrap_or_else(|_| "testnet".to_string());
        let chain_id = get_bsc_chain_id(&blockchain_network);

        let connection_metadata = serde_json::json!({
            "first_connection_at": Utc::now().to_rfc3339(),
            "connection_source": "web3_siwe",
            "domain": self.domain,
            "initial_tier": "Bronze",
            "auto_created": true,
            "chain_id": chain_id,
            "blockchain_network": blockchain_network
        });

        let now = chrono::Utc::now();
        sqlx::query(
            "INSERT INTO wallet_users (wallet_address, is_active, tier_level, wallet_metadata, created_at, updated_at, last_auth_at) VALUES ($1, true, 'Bronze', $2, $3, $3, $3)",
        )
        .bind(wallet_address)
        .bind(&connection_metadata)
        .bind(now)
        .execute(&*self.db_pool)
        .await
        .map_err(|e| Web3AuthError::DatabaseError(e.to_string()))?;

        info!(
            wallet_address = %wallet_address,
            domain = %self.domain,
            connection_type = "new_wallet_creation",
            metadata = %connection_metadata,
            "New wallet user created successfully"
        );

        self.emit_new_wallet_event(wallet_address, &connection_metadata)
            .await;
        self.assign_free_plan_to_wallet(wallet_address).await;

        Ok((wallet_address.to_string(), true))
    }

    /// Emit new wallet creation event for admin notifications
    pub(super) async fn emit_new_wallet_event(
        &self,
        wallet_address: &str,
        metadata: &serde_json::Value,
    ) {
        let event_payload = serde_json::json!({
            "event_type": "new_wallet_connected",
            "timestamp": Utc::now().to_rfc3339(),
            "wallet_address": wallet_address,
            "metadata": metadata,
            "notification": {
                "title": "New Wallet Connected",
                "message": format!("Wallet {} just connected to the platform", wallet_address),
                "severity": "info",
                "category": "user_activity"
            }
        });

        info!(
            event_type = "new_wallet_connected",
            wallet_address = %wallet_address,
            payload = %event_payload,
            "New wallet event emitted for admin notification"
        );
    }

    /// Assign Free Plan to a newly created wallet
    pub(super) async fn assign_free_plan_to_wallet(&self, wallet_address: &str) {
        use crate::constants::{
            FREE_PLAN_NAME, FREE_PLAN_RANKINGS_LIMIT, FREE_PLAN_RANKING_OFFSET, FREE_PLAN_SLUG,
        };

        let wallet_address = wallet_address.trim().to_lowercase();

        #[derive(sqlx::FromRow)]
        struct PlanIdResult {
            id: Uuid,
        }

        let plan_id = match sqlx::query_as::<_, PlanIdResult>(
            "SELECT id FROM plans WHERE slug = $1",
        )
        .bind(FREE_PLAN_SLUG)
        .fetch_optional(&*self.db_pool)
        .await
        {
            Ok(Some(result)) => result.id,
            Ok(None) => {
                info!("Free Plan not found, creating it automatically...");

                let free_plan_metadata = serde_json::json!({
                    "permissions": [
                        format!("epsx:rankings:view:{}", FREE_PLAN_RANKINGS_LIMIT),
                        format!("epsx:rankings:offset:{}", FREE_PLAN_RANKING_OFFSET)
                    ],
                    "features": [
                        format!("View top {} stock rankings", FREE_PLAN_RANKINGS_LIMIT),
                        "Basic market overview".to_string(),
                        "Community access".to_string()
                    ],
                    "ranking_offset": FREE_PLAN_RANKING_OFFSET,
                    "rankings_limit": FREE_PLAN_RANKINGS_LIMIT,
                    "limits": {
                        "analytics_queries_per_day": 5,
                        "stocks_tracked": 5,
                        "historical_data_months": 1
                    }
                });

                match sqlx::query_as::<_, PlanIdResult>(
                    r#"
                    INSERT INTO plans (
                        id, name, slug, description, plan_type, plan_metadata,
                        price, currency, is_active, is_promoted, display_order,
                        created_by, tier_level, is_public,
                        rate_limit_per_minute, rate_limit_per_hour, rate_limit_per_day, burst_capacity
                    ) VALUES (
                        gen_random_uuid(),
                        $1, $2, $3, 'subscription',
                        $4::jsonb,
                        0, 'USD', true, true, 1,
                        'system:auto_create', 0, true,
                        10, 100, 500, 5
                    )
                    RETURNING id
                    "#,
                )
                .bind(FREE_PLAN_NAME)
                .bind(FREE_PLAN_SLUG)
                .bind("Get started with basic analytics and stock rankings")
                .bind(&free_plan_metadata)
                .fetch_one(&*self.db_pool)
                .await
                {
                    Ok(result) => {
                        info!(plan_id = %result.id, "Free Plan created automatically");
                        result.id
                    }
                    Err(e) => {
                        warn!(error = %e, "Failed to create Free Plan automatically");
                        return;
                    }
                }
            }
            Err(e) => {
                warn!(wallet_address = %wallet_address, error = %e, "Error looking up Free Plan - skipping auto-assignment");
                return;
            }
        };

        #[derive(sqlx::FromRow)]
        struct CountResult {
            count: i64,
        }

        let existing = sqlx::query_as::<_, CountResult>(
            "SELECT COUNT(*) as count FROM wallet_plan_assignments WHERE wallet_address = $1 AND plan_id = $2",
        )
        .bind(&wallet_address)
        .bind(plan_id)
        .fetch_optional(&*self.db_pool)
        .await
        .map(|opt| opt.map(|r| r.count > 0).unwrap_or(false))
        .unwrap_or(false);

        if existing {
            debug!(wallet_address = %wallet_address, "Wallet already has Free Plan assigned");
            return;
        }

        let now = Utc::now();
        if let Err(e) = sqlx::query(
            r#"
            INSERT INTO wallet_plan_assignments (id, wallet_address, plan_id, is_active, assigned_at, assigned_by)
            VALUES (gen_random_uuid(), $1, $2, true, $3, 'system:auto_assign')
            ON CONFLICT (wallet_address, plan_id) DO NOTHING
            "#,
        )
        .bind(&wallet_address)
        .bind(plan_id)
        .bind(now)
        .execute(&*self.db_pool)
        .await
        {
            warn!(wallet_address = %wallet_address, error = %e, "Failed to assign Free Plan to wallet");
            return;
        }

        info!(wallet_address = %wallet_address, plan_id = %plan_id, "Free Plan auto-assigned to new wallet");
    }

    /// Get NFT-based permissions
    pub(super) async fn get_nft_permissions(
        &self,
        wallet_address: &str,
    ) -> Result<Vec<String>, Web3AuthError> {
        let mut permissions = Vec::new();

        let nft_contract = match std::env::var("ENTERPRISE_NFT_CONTRACT") {
            Ok(contract) if !contract.is_empty() => contract,
            _ => {
                debug!("No enterprise NFT contract configured, skipping NFT permissions");
                return Ok(permissions);
            }
        };

        let provider = match self.bsc_provider() {
            Ok(p) => p,
            Err(e) => {
                warn!("Failed to create BSC provider for NFT check: {}", e);
                return Ok(permissions);
            }
        };

        let wallet_addr = match Address::from_str(wallet_address) {
            Ok(addr) => addr,
            Err(e) => {
                warn!("Invalid wallet address {}: {}", wallet_address, e);
                return Ok(permissions);
            }
        };

        let contract_addr = match Address::from_str(&nft_contract) {
            Ok(addr) => addr,
            Err(e) => {
                warn!("Invalid NFT contract address {}: {}", nft_contract, e);
                return Ok(permissions);
            }
        };

        let balance_of_abi = r#"[{"inputs":[{"name":"owner","type":"address"}],"name":"balanceOf","outputs":[{"name":"","type":"uint256"}],"type":"function"}]"#;

        if let Ok(abi) = serde_json::from_str::<Abi>(balance_of_abi) {
            let contract = Contract::new(contract_addr, abi, Arc::new(provider));
            match contract.method::<_, U256>("balanceOf", wallet_addr) {
                Ok(call) => match call.call().await {
                    Ok(balance) if balance > U256::zero() => {
                        permissions.push("epsx:premium:nft_holder".to_string());
                        permissions.push("epsx:analytics:exclusive".to_string());
                        info!(
                            "Wallet {} owns {} NFTs, granted premium NFT permissions",
                            wallet_address, balance
                        );
                    }
                    Ok(_) => debug!(
                        "Wallet {} owns no NFTs from contract {}",
                        wallet_address, nft_contract
                    ),
                    Err(e) => warn!("Failed to check NFT balance for {}: {}", wallet_address, e),
                },
                Err(e) => warn!("Failed to create NFT contract call: {}", e),
            }
        } else {
            warn!("Failed to parse NFT contract ABI");
        }

        Ok(permissions)
    }

    /// Get DAO governance permissions based on governance token holdings
    pub(super) async fn get_dao_permissions(
        &self,
        wallet_address: &str,
    ) -> Result<Vec<String>, Web3AuthError> {
        let mut permissions = Vec::new();

        let governance_token = match std::env::var("ENTERPRISE_GOVERNANCE_TOKEN") {
            Ok(contract) if !contract.is_empty() => contract,
            _ => {
                debug!("No enterprise governance token configured, skipping DAO permissions");
                return Ok(permissions);
            }
        };

        let provider = match self.bsc_provider() {
            Ok(p) => p,
            Err(e) => {
                warn!("Failed to create BSC provider for DAO check: {}", e);
                return Ok(permissions);
            }
        };

        let wallet_addr = match Address::from_str(wallet_address) {
            Ok(addr) => addr,
            Err(e) => {
                warn!("Invalid wallet address {}: {}", wallet_address, e);
                return Ok(permissions);
            }
        };

        let token_addr = match Address::from_str(&governance_token) {
            Ok(addr) => addr,
            Err(e) => {
                warn!(
                    "Invalid governance token address {}: {}",
                    governance_token, e
                );
                return Ok(permissions);
            }
        };

        let balance_of_abi = r#"[{"inputs":[{"name":"account","type":"address"}],"name":"balanceOf","outputs":[{"name":"","type":"uint256"}],"type":"function"}]"#;

        if let Ok(abi) = serde_json::from_str::<Abi>(balance_of_abi) {
            let contract = Contract::new(token_addr, abi, Arc::new(provider));
            match contract.method::<_, U256>("balanceOf", wallet_addr) {
                Ok(call) => match call.call().await {
                    Ok(balance) if balance > U256::zero() => {
                        let token_bal = balance.as_u128() as f64 / 1e18;
                        if token_bal >= 1000.0 {
                            permissions.push("epsx:dao:executive".to_string());
                            permissions.push("epsx:dao:vote".to_string());
                            permissions.push("epsx:dao:propose".to_string());
                        } else if token_bal >= 100.0 {
                            permissions.push("epsx:dao:member".to_string());
                            permissions.push("epsx:dao:vote".to_string());
                        } else if token_bal >= 10.0 {
                            permissions.push("epsx:dao:participant".to_string());
                        }
                        info!(
                            "Wallet {} holds {} governance tokens, granted {} DAO permissions",
                            wallet_address,
                            token_bal,
                            permissions.len()
                        );
                    }
                    Ok(_) => debug!("Wallet {} holds no governance tokens", wallet_address),
                    Err(e) => warn!(
                        "Failed to check governance token balance for {}: {}",
                        wallet_address, e
                    ),
                },
                Err(e) => warn!("Failed to create governance token contract call: {}", e),
            }
        } else {
            warn!("Failed to parse governance token ABI");
        }

        Ok(permissions)
    }

    /// Create a BSC provider from environment config
    fn bsc_provider(&self) -> Result<Provider<Http>, String> {
        let network = std::env::var("NEXT_PUBLIC_BLOCKCHAIN_NETWORK")
            .unwrap_or_else(|_| "testnet".to_string());
        let rpc_url = std::env::var("BSC_RPC_URL").unwrap_or_else(|_| match network.as_str() {
            "mainnet" => "https://bsc-dataseed.binance.org".to_string(),
            _ => "https://data-seed-prebsc-1-s1.binance.org:8545".to_string(),
        });
        Provider::<Http>::try_from(&rpc_url).map_err(|e| e.to_string())
    }
}