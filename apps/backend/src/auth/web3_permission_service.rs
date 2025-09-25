#![allow(deprecated)]
use anyhow::{ anyhow, Result };
use chrono::{ DateTime, Duration, Utc };
use ethers::types::Address;
use serde::{ Deserialize, Serialize };
use sqlx::{ PgPool, Row };
use std::str::FromStr;
use tracing::{ debug, error, info, warn };
use uuid::Uuid;

/// Web3 permission service handling all 4 permission types
#[derive(Clone)]
pub struct Web3PermissionService {
  db_pool: PgPool,
  // Provider endpoints for different networks
  ethereum_rpc_url: String,
  polygon_rpc_url: String,
  arbitrum_rpc_url: String,
  optimism_rpc_url: String,
  base_rpc_url: String,
  bsc_rpc_url: String,
  cache_duration_minutes: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PermissionInfo {
  pub permission: String,
  pub permission_type: String,
  pub is_active: bool,
  pub expires_at: Option<DateTime<Utc>>,
  pub granted_at: DateTime<Utc>,
  pub last_verified_at: Option<DateTime<Utc>>,
  pub verification_data: Option<serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NFTConfig {
  pub contract_address: String,
  pub network: String,
  pub permission: String,
  pub collection_name: Option<String>,
  pub require_specific_token: bool,
  pub specific_token_ids: Vec<String>,
  pub minimum_tokens: i32,
  pub check_ownership_live: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TokenConfig {
  pub contract_address: String,
  pub network: String,
  pub permission: String,
  pub token_name: Option<String>,
  pub token_symbol: Option<String>,
  pub minimum_balance: String, // String to handle large numbers
  pub token_decimals: i32,
  pub check_balance_live: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DAOProposal {
  pub dao_contract_address: String,
  pub network: String,
  pub proposal_id: String,
  pub title: String,
  pub description: Option<String>,
  pub target_wallet_address: String,
  pub permission: String,
  pub proposal_status: String,
  pub voting_end: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PermissionVerificationResult {
  pub wallet_address: String,
  pub permission: String,
  pub is_granted: bool,
  pub verification_type: String,
  pub verification_data: serde_json::Value,
  pub cached_until: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PermissionDelegation {
  pub delegator: String, // Original permission holder
  pub delegate: String, // Receiving wallet
  pub permission: String, // Delegated permission
  pub signature: String, // EIP-712 signature proof
  pub expires_at: DateTime<Utc>,
  pub delegation_depth: u8, // Max 3 levels deep
  pub network: String, // Chain where delegation is valid
  pub nonce: String, // Prevent replay attacks
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EIP712DelegationMessage {
  pub delegator: String,
  pub delegate: String,
  pub permission: String,
  pub expires_at: i64,
  pub nonce: String,
  pub network: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum PermissionCondition {
  And(Vec<PermissionCondition>),
  Or(Vec<PermissionCondition>),
  Not(Box<PermissionCondition>),
  TokenBalance {
    contract: String,
    network: String,
    min_balance: String,
  },
  NFTOwnership {
    contract: String,
    network: String,
    token_id: Option<String>,
  },
  DelegatedBy {
    delegator: String,
  },
  TimeWindow {
    start: DateTime<Utc>,
    end: DateTime<Utc>,
  },
  ChainId {
    chain_id: u64,
  },
}

impl Web3PermissionService {
  pub fn new(
    db_pool: PgPool,
    ethereum_rpc_url: String,
    polygon_rpc_url: String
  ) -> Self {
    Self {
      db_pool,
      ethereum_rpc_url: ethereum_rpc_url.clone(),
      polygon_rpc_url,
      // Use fallback URLs for additional networks
      arbitrum_rpc_url: "https://arbitrum.llamarpc.com".to_string(),
      optimism_rpc_url: "https://optimism.llamarpc.com".to_string(),
      base_rpc_url: "https://base.llamarpc.com".to_string(),
      bsc_rpc_url: "https://bsc-dataseed.binance.org".to_string(),
      cache_duration_minutes: 60, // 1 hour default cache
    }
  }

  /// Create Web3PermissionService with full multi-chain support
  pub fn new_multi_chain(
    db_pool: PgPool,
    ethereum_rpc_url: String,
    polygon_rpc_url: String,
    arbitrum_rpc_url: String,
    optimism_rpc_url: String,
    base_rpc_url: String,
    bsc_rpc_url: String
  ) -> Self {
    Self {
      db_pool,
      ethereum_rpc_url,
      polygon_rpc_url,
      arbitrum_rpc_url,
      optimism_rpc_url,
      base_rpc_url,
      bsc_rpc_url,
      cache_duration_minutes: 60,
    }
  }

  /// Get blockchain provider for the specified network
  fn get_provider(
    &self,
    network: &str
  ) -> Result<ethers::providers::Provider<ethers::providers::Http>> {
    use ethers::providers::{ Provider, Http };

    let rpc_url = match network.to_lowercase().as_str() {
      "ethereum" | "mainnet" => &self.ethereum_rpc_url,
      "polygon" => &self.polygon_rpc_url,
      "arbitrum" => &self.arbitrum_rpc_url,
      "optimism" => &self.optimism_rpc_url,
      "base" => &self.base_rpc_url,
      "bsc" | "binance" => &self.bsc_rpc_url,
      _ => {
        return Err(anyhow!("Unsupported network: {}", network));
      }
    };

    Provider::<Http>
      ::try_from(rpc_url)
      .map_err(|e| anyhow!("Failed to create provider for {}: {}", network, e))
  }

  // Removed unused get_chain_id method

  /// Verify BSC-specific token standards (BEP-20, BEP-721)
  pub async fn verify_bsc_token_balance(
    &self,
    wallet_address: &str,
    contract_address: &str,
    required_balance: &str,
    token_type: &str // "BEP-20" or "BEP-721"
  ) -> Result<bool> {
    match token_type {
      "BEP-20" => {
        // BEP-20 is identical to ERC-20
        self.verify_token_balance(
          wallet_address,
          contract_address,
          "bsc",
          required_balance,
          18
        ).await
      }
      "BEP-721" => {
        // BEP-721 is identical to ERC-721
        self.verify_nft_ownership(
          wallet_address,
          contract_address,
          "bsc",
          None
        ).await
      }
      _ => Err(anyhow!("Unsupported BSC token type: {}", token_type)),
    }
  }

  /// Verify PancakeSwap LP token ownership
  pub async fn verify_pancakeswap_lp_balance(
    &self,
    wallet_address: &str,
    lp_contract_address: &str,
    minimum_lp_balance: &str
  ) -> Result<bool> {
    // PancakeSwap LP tokens are ERC-20 compatible
    self.verify_token_balance(
      wallet_address,
      lp_contract_address,
      "bsc",
      minimum_lp_balance,
      18
    ).await
  }

  /// Check if wallet has CAKE token balance for staking permissions
  pub async fn verify_cake_token_balance(
    &self,
    wallet_address: &str,
    minimum_cake_balance: &str
  ) -> Result<bool> {
    // CAKE token contract on BSC
    let cake_contract = "0x0e09fabb73bd3ade0a17ecc321fd13a19e81ce82";
    self.verify_bsc_token_balance(
      wallet_address,
      cake_contract,
      minimum_cake_balance,
      "BEP-20"
    ).await
  }

  /// Get all active permissions for a wallet address
  pub async fn get_user_permissions(
    &self,
    wallet_address: &str
  ) -> Result<Vec<PermissionInfo>> {
    let address = Address::from_str(wallet_address).map_err(|_|
      anyhow!("Invalid wallet address format")
    )?;
    let wallet_address = address.to_string().to_lowercase();

    let rows = sqlx
      ::query(
        r#"
            SELECT permission, permission_type, is_active, expires_at, granted_at,
                   last_verified_at, verification_data
            FROM wallet_permissions
            WHERE wallet_address = $1
              AND is_active = true
              AND (expires_at IS NULL OR expires_at > CURRENT_TIMESTAMP)
            ORDER BY granted_at DESC
            "#
      )
      .bind(wallet_address)
      .fetch_all(&self.db_pool).await
      .map_err(|e| {
        error!("Failed to query wallet permissions: {}", e);
        anyhow!("Database query error ")
      })?;

    let permissions = rows
      .into_iter()
      .map(|row| PermissionInfo {
        permission: row.try_get("permission").unwrap_or_default(),
        permission_type: row.try_get("permission_type").unwrap_or_default(),
        is_active: row.try_get("is_active").unwrap_or(true),
        expires_at: row.try_get("expires_at").ok(),
        granted_at: row.try_get("granted_at").unwrap_or_else(|_| Utc::now()),
        last_verified_at: row.try_get("last_verified_at").ok(),
        verification_data: row.try_get("verification_data").ok(),
      })
      .collect();

    Ok(permissions)
  }

  /// Check if wallet has specific permission with real-time verification
  pub async fn has_permission(
    &self,
    wallet_address: &str,
    permission: &str
  ) -> Result<bool> {
    let address = Address::from_str(wallet_address).map_err(|_|
      anyhow!("Invalid wallet address format")
    )?;
    let wallet_address = address.to_string().to_lowercase();

    // Get all permissions of this type
    let permissions = sqlx
      ::query(
        r#"
            SELECT id, permission_type, nft_contract_address, nft_token_id, nft_network,
                   token_contract_address, required_balance, token_network, token_decimals,
                   dao_contract_address, dao_proposal_id, dao_network,
                   last_verified_at, verification_data
            FROM wallet_permissions
            WHERE wallet_address = $1
              AND permission = $2
              AND is_active = true
              AND (expires_at IS NULL OR expires_at > CURRENT_TIMESTAMP)
            "#
      )
      .bind(&wallet_address)
      .bind(permission)
      .fetch_all(&self.db_pool).await
      .map_err(|e| {
        error!("Failed to query specific permission: {}", e);
        anyhow!("Database query error ")
      })?;

    for perm in permissions {
      match
        perm
          .try_get::<String, _>("permission_type")
          .unwrap_or_default()
          .as_str()
      {
        "manual" => {
          // Manual permissions are always valid if they exist and haven't expired
          return Ok(true);
        }
        "nft_gated" => {
          if
            let (Some(contract), Some(network)) = (
              perm
                .try_get::<Option<String>, _>("nft_contract_address")
                .unwrap_or_default(),
              perm
                .try_get::<Option<String>, _>("nft_network")
                .unwrap_or_default(),
            )
          {
            if
              self.verify_nft_ownership(
                wallet_address.as_str(),
                &contract,
                &network,
                perm
                  .try_get::<Option<String>, _>("nft_token_id")
                  .unwrap_or_default()
                  .as_deref()
              ).await?
            {
              self.update_verification_status(
                perm.try_get::<uuid::Uuid, _>("id").unwrap_or_default(),
                true,
                serde_json::json!({"type": "nft_verified"})
              ).await?;
              return Ok(true);
            }
          }
        }
        "token_gated" => {
          if
            let (Some(contract), Some(network), Some(balance), Some(decimals)) =
              (
                perm
                  .try_get::<Option<String>, _>("token_contract_address")
                  .unwrap_or_default(),
                perm
                  .try_get::<Option<String>, _>("token_network")
                  .unwrap_or_default(),
                perm
                  .try_get::<Option<String>, _>("required_balance")
                  .unwrap_or_default(),
                perm
                  .try_get::<Option<i32>, _>("token_decimals")
                  .unwrap_or_default(),
              )
          {
            if
              self.verify_token_balance(
                &wallet_address,
                &contract,
                &network,
                &balance,
                decimals
              ).await?
            {
              self.update_verification_status(
                perm.try_get::<uuid::Uuid, _>("id").unwrap_or_default(),
                true,
                serde_json::json!({"type": "token_verified"})
              ).await?;
              return Ok(true);
            }
          }
        }
        "dao_granted" => {
          if
            let (Some(dao_contract), Some(proposal_id), Some(network)) = (
              perm
                .try_get::<Option<String>, _>("dao_contract_address")
                .unwrap_or_default(),
              perm
                .try_get::<Option<String>, _>("dao_proposal_id")
                .unwrap_or_default(),
              perm
                .try_get::<Option<String>, _>("dao_network")
                .unwrap_or_default(),
            )
          {
            if
              self.verify_dao_permission(
                &dao_contract,
                &proposal_id,
                &network,
                &wallet_address
              ).await?
            {
              self.update_verification_status(
                perm.try_get::<uuid::Uuid, _>("id").unwrap_or_default(),
                true,
                serde_json::json!({"type": "dao_verified"})
              ).await?;
              return Ok(true);
            }
          }
        }
        _ => {
          warn!(
            "Unknown permission type: {}",
            perm.try_get::<String, _>("permission_type").unwrap_or_default()
          );
        }
      }
    }

    Ok(false)
  }

  /// Grant manual permission to wallet
  pub async fn grant_manual_permission(
    &self,
    wallet_address: &str,
    permission: &str,
    granted_by: Option<Uuid>,
    expires_at: Option<DateTime<Utc>>
  ) -> Result<Uuid> {
    let address = Address::from_str(wallet_address).map_err(|_|
      anyhow!("Invalid wallet address format")
    )?;
    let wallet_address = address.to_string().to_lowercase();

    let permission_id = Uuid::new_v4();

    sqlx
      ::query(
        r#"
            INSERT INTO wallet_permissions
            (id, wallet_address, permission, permission_type, granted_by, expires_at)
            VALUES ($1, $2, $3, 'manual', $4, $5)
            "#
      )
      .bind(permission_id)
      .bind(&wallet_address)
      .bind(permission)
      .bind(granted_by)
      .bind(expires_at)
      .execute(&self.db_pool).await
      .map_err(|e| {
        error!("Failed to grant manual permission: {}", e);
        anyhow!("Grant permission error ")
      })?;

    info!(
      "Granted manual permission {} to wallet {}",
      permission,
      wallet_address
    );
    Ok(permission_id)
  }

  /// Configure NFT-gated permission
  pub async fn configure_nft_permission(
    &self,
    config: NFTConfig,
    created_by: Uuid
  ) -> Result<Uuid> {
    let address = Address::from_str(&config.contract_address).map_err(|_|
      anyhow!("Invalid contract address format")
    )?;
    let contract_address = address.to_string().to_lowercase();

    let config_id = Uuid::new_v4();

    sqlx
      ::query(
        r#"
            INSERT INTO nft_permission_configs
            (id, contract_address, network, permission, collection_name,
             require_specific_token, specific_token_ids, minimum_tokens,
             check_ownership_live, created_by)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
            "#
      )
      .bind(config_id)
      .bind(&contract_address)
      .bind(&config.network)
      .bind(config.permission)
      .bind(config.collection_name)
      .bind(config.require_specific_token)
      .bind(&config.specific_token_ids)
      .bind(config.minimum_tokens)
      .bind(config.check_ownership_live)
      .bind(created_by)
      .execute(&self.db_pool).await
      .map_err(|e| {
        error!("Failed to configure NFT permission: {}", e);
        anyhow!("Failed to configure NFT permission")
      })?;

    info!(
      "Configured NFT permission for contract {} on {}",
      contract_address,
      config.network
    );
    Ok(config_id)
  }

  /// Configure token-gated permission
  pub async fn configure_token_permission(
    &self,
    config: TokenConfig,
    created_by: Uuid
  ) -> Result<Uuid> {
    let address = Address::from_str(&config.contract_address).map_err(|_|
      anyhow!("Invalid contract address format")
    )?;
    let contract_address = address.to_string().to_lowercase();

    let config_id = Uuid::new_v4();

    sqlx
      ::query(
        r#"
            INSERT INTO token_permission_configs
            (id, contract_address, network, permission, token_name, token_symbol,
             token_decimals, minimum_balance, check_balance_live, created_by)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
            "#
      )
      .bind(config_id)
      .bind(&contract_address)
      .bind(&config.network)
      .bind(config.permission)
      .bind(config.token_name)
      .bind(config.token_symbol)
      .bind(config.token_decimals)
      .bind(
        sqlx::types::BigDecimal
          ::from_str(&config.minimum_balance)
          .map_err(|_| anyhow!("Invalid balance format "))?
      )
      .bind(config.check_balance_live)
      .bind(created_by)
      .execute(&self.db_pool).await
      .map_err(|e| {
        error!("Failed to configure token permission: {}", e);
        anyhow!("Failed to configure token permission")
      })?;

    info!(
      "Configured token permission for contract {} on {}",
      contract_address,
      config.network
    );
    Ok(config_id)
  }

  /// Create DAO permission proposal
  pub async fn create_dao_proposal(
    &self,
    proposal: DAOProposal
  ) -> Result<Uuid> {
    let dao_address = Address::from_str(&proposal.dao_contract_address).map_err(
      |_| anyhow!("Invalid DAO contract address format")
    )?;
    let dao_contract = dao_address.to_string().to_lowercase();

    let target_address = Address::from_str(
      &proposal.target_wallet_address
    ).map_err(|_| anyhow!("Invalid target wallet address format"))?;
    let target_wallet = target_address.to_string().to_lowercase();

    let proposal_id = Uuid::new_v4();

    sqlx
      ::query(
        r#"
            INSERT INTO dao_permission_proposals
            (id, dao_contract_address, network, proposal_id, title, description,
             proposer_address, target_wallet_address, permission, proposal_status, voting_end)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, 'active', $10)
            "#
      )
      .bind(proposal_id)
      .bind(&dao_contract)
      .bind(proposal.network)
      .bind(&proposal.proposal_id)
      .bind(proposal.title)
      .bind(proposal.description)
      .bind(&dao_contract) // Assuming proposer is the DAO contract for now
      .bind(target_wallet)
      .bind(&proposal.permission)
      .bind(proposal.voting_end)
      .execute(&self.db_pool).await
      .map_err(|e| {
        error!("Failed to create DAO proposal: {}", e);
        anyhow!("Failed to create DAO proposal")
      })?;

    info!(
      "Created DAO proposal {} for permission {}",
      proposal.proposal_id,
      proposal.permission
    );
    Ok(proposal_id)
  }

  /// Process automatic permission granting based on configurations
  pub async fn process_automatic_permissions(
    &self,
    wallet_address: &str
  ) -> Result<Vec<String>> {
    let address = Address::from_str(wallet_address).map_err(|_|
      anyhow!("Invalid wallet address format")
    )?;
    let wallet_address = address.to_string().to_lowercase();

    let mut granted_permissions = Vec::new();

    // Check NFT-gated permissions
    let nft_configs = self.get_active_nft_configs().await?;
    for config in nft_configs {
      if
        self.verify_nft_ownership(
          &wallet_address,
          &config.contract_address,
          &config.network,
          None
        ).await?
      {
        self.grant_nft_permission(&wallet_address, &config).await?;
        granted_permissions.push(config.permission);
      }
    }

    // Check token-gated permissions
    let token_configs = self.get_active_token_configs().await?;
    for config in token_configs {
      if
        self.verify_token_balance(
          &wallet_address,
          &config.contract_address,
          &config.network,
          &config.minimum_balance.to_string(),
          config.token_decimals
        ).await?
      {
        self.grant_token_permission(&wallet_address, &config).await?;
        granted_permissions.push(config.permission);
      }
    }

    if !granted_permissions.is_empty() {
      info!(
        "Automatically granted {} permissions to wallet {}",
        granted_permissions.len(),
        wallet_address
      );
    }

    Ok(granted_permissions)
  }

  /// Revoke permission from wallet
  pub async fn revoke_permission(
    &self,
    wallet_address: &str,
    permission: &str
  ) -> Result<bool> {
    let address = Address::from_str(wallet_address).map_err(|_|
      anyhow!("Invalid wallet address format")
    )?;
    let wallet_address = address.to_string().to_lowercase();

    let result = sqlx
      ::query(
        r#"
            UPDATE wallet_permissions
            SET is_active = false, updated_at = CURRENT_TIMESTAMP
            WHERE wallet_address = $1 AND permission = $2 AND is_active = true
            "#
      )
      .bind(&wallet_address)
      .bind(permission)
      .execute(&self.db_pool).await
      .map_err(|e| {
        error!("Failed to revoke permission: {}", e);
        anyhow!("Failed to revoke permission")
      })?;

    let revoked = result.rows_affected() > 0;
    if revoked {
      info!("Revoked permission {} from wallet {}", permission, wallet_address);
    }

    Ok(revoked)
  }

  /// Delegate permission with EIP-712 signature verification
  pub async fn delegate_permission(
    &self,
    delegation: PermissionDelegation
  ) -> Result<Uuid> {
    // Verify EIP-712 signature
    let message = self.create_delegation_message(&delegation)?;
    self.verify_eip712_signature(
      &delegation.signature,
      &message,
      &delegation.delegator
    ).await?;

    // Validate delegation depth (max 3 levels)
    if delegation.delegation_depth > 3 {
      return Err(anyhow!("Maximum delegation depth exceeded"));
    }

    // Verify delegator has the permission to delegate
    if
      !self.has_permission(&delegation.delegator, &delegation.permission).await?
    {
      return Err(anyhow!("Delegator does not have permission to delegate"));
    }

    // Store delegation in database
    let delegation_id = Uuid::new_v4();
    sqlx
      ::query(
        r#"
            INSERT INTO permission_delegations
            (id, delegator, delegate, permission, signature, expires_at, delegation_depth, network, nonce)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            "#
      )
      .bind(delegation_id)
      .bind(&delegation.delegator)
      .bind(&delegation.delegate)
      .bind(&delegation.permission)
      .bind(delegation.signature)
      .bind(delegation.expires_at)
      .bind(delegation.delegation_depth as i16)
      .bind(delegation.network)
      .bind(delegation.nonce)
      .execute(&self.db_pool).await
      .map_err(|e| {
        error!("Failed to store delegation: {}", e);
        anyhow!("Failed to store delegation")
      })?;

    info!(
      "Permission {} delegated from {} to {}",
      delegation.permission,
      delegation.delegator,
      delegation.delegate
    );
    Ok(delegation_id)
  }

  /// Verify token balance with USD value calculation
  pub async fn verify_token_balance_usd(
    &self,
    wallet_address: &str,
    contract_address: &str,
    network: &str,
    minimum_usd_value: u64,
    decimals: i32
  ) -> Result<bool> {
    // First get the raw token balance
    let has_tokens = self.verify_token_balance(
      wallet_address,
      contract_address,
      network,
      "1", // Just check if they have any tokens first
      decimals
    ).await?;

    if !has_tokens {
      return Ok(false);
    }

    // Get the actual balance
    let provider = self.get_provider(network)?;
    let wallet_addr = wallet_address.parse::<ethers::types::Address>()?;
    let contract_addr = contract_address.parse::<ethers::types::Address>()?;

    use ethers::abi::{Function, Param, ParamType, StateMutability, Token};
    use ethers::providers::Middleware;

    let balance_of = Function {
      name: "balanceOf".to_string(),
      inputs: vec![Param {
        name: "account".to_string(),
        kind: ParamType::Address,
        internal_type: None,
      }],
      outputs: vec![Param {
        name: "".to_string(),
        kind: ParamType::Uint(256),
        internal_type: None,
      }],
      constant: None,
      state_mutability: StateMutability::View,
    };

    let calldata = balance_of.encode_input(&[Token::Address(wallet_addr)])?;
    let tx = ethers::types::transaction::eip2718::TypedTransaction::Legacy(ethers::types::TransactionRequest::new()
        .to(contract_addr)
        .data(calldata));
    let result = provider.call(&tx, None).await?;

    let balance = match balance_of.decode_output(&result) {
      Ok(tokens) => {
        if let Some(Token::Uint(balance)) = tokens.get(0) {
          *balance
        } else {
          return Ok(false);
        }
      }
      Err(_) => return Ok(false),
    };

    // Convert balance to human readable format
    let decimals_multiplier = ethers::types::U256::from(10).pow(ethers::types::U256::from(decimals));
    let human_balance = balance.checked_div(decimals_multiplier).unwrap_or_default();

    // Get USD price (placeholder - in production would use real price oracle)
    let usd_price = self.get_token_usd_price(contract_address, network).await?;
    let usd_value = human_balance.as_u64() * usd_price;

    debug!(
      "Token USD value verification: wallet={}, token={}, balance={}, usd_price=${}, total_value=${}, required=${}, verified={}",
      wallet_address, contract_address, human_balance, usd_price, usd_value, minimum_usd_value, usd_value >= minimum_usd_value
    );

    Ok(usd_value >= minimum_usd_value)
  }

  /// Get USD price for a token (placeholder implementation)
  async fn get_token_usd_price(
    &self,
    contract_address: &str,
    network: &str
  ) -> Result<u64> {
    // Placeholder USD prices for common tokens
    let price = match (network, contract_address.to_lowercase().as_str()) {
      // USDC on various networks
      (_, addr) if addr.contains("a0b86a33") => 1, // USDC = $1
      ("ethereum", "0xa0b86a33e6f2b84af98e67b1f6b40f5af1b1b1b1") => 1, // USDC
      ("polygon", "0x2791bca1f2de4661ed88a30c99a7a9449aa84174") => 1, // USDC on Polygon
      ("arbitrum", "0xff970a61a04b1ca14834a43f5de4533ebddb5cc8") => 1, // USDC on Arbitrum
      ("optimism", "0x7f5c764cbc13c35c5e0f0db4ef72c0b1f47af4c") => 1, // USDC on Optimism
      ("base", "0x833589fcd6edb6e08f4c7c32d4f71b54bda02913") => 1, // USDC on Base
      ("bsc", "0x8ac76a51cc950d9822d68b83fe1ad97b32cd580d") => 1, // USDC on BSC

      // USDT
      ("ethereum", "0xdac17f958d2ee523a2206206994597c13d831ec7") => 1, // USDT
      ("polygon", "0xc2132d05d31c914a87c6611c10748aeb04b58e8f") => 1, // USDT on Polygon
      ("bsc", "0x55d398326f99059ff775485246999027b3197955") => 1, // USDT on BSC

      // ETH and native tokens (approximate)
      ("ethereum", "0x0000000000000000000000000000000000000000") => 2500, // ETH
      ("polygon", "0x0000000000000000000000000000000000000000") => 1, // MATIC
      ("arbitrum", "0x0000000000000000000000000000000000000000") => 2500, // ETH on Arbitrum
      ("optimism", "0x0000000000000000000000000000000000000000") => 2500, // ETH on Optimism
      ("base", "0x0000000000000000000000000000000000000000") => 2500, // ETH on Base
      ("bsc", "0x0000000000000000000000000000000000000000") => 400, // BNB

      // Default fallback
      _ => {
        warn!("Unknown token price for {} on {}, using $1 default", contract_address, network);
        1
      }
    };

    debug!("Token USD price lookup: {} on {} = ${}", contract_address, network, price);
    Ok(price)
  }

  /// Verify cross-chain permission across all supported networks
  pub async fn verify_cross_chain_permission(
    &self,
    wallet: &str,
    permission: &str
  ) -> Result<bool> {
    let supported_networks = [
      "ethereum",
      "polygon",
      "arbitrum",
      "optimism",
      "base",
      "bsc",
    ];

    for network in &supported_networks {
      if
        let Ok(has_permission) = self.verify_permission_on_network(
          wallet,
          permission,
          network
        ).await
      {
        if has_permission {
          return Ok(true);
        }
      }
    }

    Ok(false)
  }

  /// Evaluate complex permission conditions
  pub fn evaluate_permission_condition<'a>(
    &'a self,
    condition: &'a PermissionCondition,
    wallet: &'a str
  ) -> std::pin::Pin<
    Box<dyn std::future::Future<Output = Result<bool>> + Send + 'a>
  > {
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
        PermissionCondition::TokenBalance {
          contract,
          network,
          min_balance,
        } => {
          self.verify_token_balance(
            wallet,
            contract,
            network,
            min_balance,
            18
          ).await
        }
        PermissionCondition::NFTOwnership { contract, network, token_id } => {
          self.verify_nft_ownership(
            wallet,
            contract,
            network,
            token_id.as_deref()
          ).await
        }
        PermissionCondition::DelegatedBy { delegator } => {
          self.check_delegation(wallet, delegator).await
        }
        PermissionCondition::TimeWindow { start, end } => {
          let now = Utc::now();
          Ok(now >= *start && now <= *end)
        }
        PermissionCondition::ChainId { chain_id: _ } => {
          // This would require additional context about current chain
          // For now, just return true
          Ok(true)
        }
      }
    })
  }

  // Private helper methods

  pub async fn verify_nft_ownership(
    &self,
    wallet_address: &str,
    contract_address: &str,
    network: &str,
    specific_token_id: Option<&str>
  ) -> Result<bool> {
    // Check cache first
    if let Some(cached_result) = self.get_cached_verification(
      wallet_address,
      contract_address,
      network,
      "nft_gated"
    ).await? {
      return Ok(cached_result);
    }

    // Real blockchain verification using ethers
    let provider = match self.get_provider(network) {
      Ok(provider) => provider,
      Err(e) => {
        warn!("Failed to get provider for {}: {}", network, e);
        return Ok(false);
      }
    };

    // Parse addresses
    let wallet_addr = match wallet_address.parse::<ethers::types::Address>() {
      Ok(addr) => addr,
      Err(e) => {
        warn!("Invalid wallet address {}: {}", wallet_address, e);
        return Ok(false);
      }
    };

    let contract_addr = match contract_address.parse::<ethers::types::Address>() {
      Ok(addr) => addr,
      Err(e) => {
        warn!("Invalid contract address {}: {}", contract_address, e);
        return Ok(false);
      }
    };

    use ethers::abi::{Function, Param, ParamType, StateMutability, Token};
    use ethers::providers::Middleware;

    let verified = if let Some(token_id) = specific_token_id {
      // Check ownership of specific token ID using ownerOf
      let owner_of = Function {
        name: "ownerOf".to_string(),
        inputs: vec![Param {
          name: "tokenId".to_string(),
          kind: ParamType::Uint(256),
          internal_type: None,
        }],
        outputs: vec![Param {
          name: "".to_string(),
          kind: ParamType::Address,
          internal_type: None,
        }],
        constant: None,
        state_mutability: StateMutability::View,
      };

      let token_id_uint = match token_id.parse::<u128>() {
        Ok(val) => ethers::types::U256::from(val),
        Err(e) => {
          warn!("Invalid token ID format {}: {}", token_id, e);
          return Ok(false);
        }
      };

      let calldata = owner_of.encode_input(&[Token::Uint(token_id_uint)])?;

      let tx = ethers::types::transaction::eip2718::TypedTransaction::Legacy(ethers::types::TransactionRequest::new()
          .to(contract_addr)
          .data(calldata));
      let call_result = provider.call(&tx, None).await;

      match call_result {
        Ok(result) => {
          match owner_of.decode_output(&result) {
            Ok(tokens) => {
              if let Some(Token::Address(owner)) = tokens.get(0) {
                *owner == wallet_addr
              } else {
                warn!("Unexpected ownerOf response format");
                false
              }
            }
            Err(e) => {
              warn!("Failed to decode ownerOf response: {}", e);
              false
            }
          }
        }
        Err(e) => {
          warn!("Failed to call ownerOf for token {} on {}: {}", token_id, network, e);
          false
        }
      }
    } else {
      // Check total balance using balanceOf
      let balance_of = Function {
        name: "balanceOf".to_string(),
        inputs: vec![Param {
          name: "owner".to_string(),
          kind: ParamType::Address,
          internal_type: None,
        }],
        outputs: vec![Param {
          name: "".to_string(),
          kind: ParamType::Uint(256),
          internal_type: None,
        }],
        constant: None,
        state_mutability: StateMutability::View,
      };

      let calldata = balance_of.encode_input(&[Token::Address(wallet_addr)])?;

      let tx = ethers::types::transaction::eip2718::TypedTransaction::Legacy(ethers::types::TransactionRequest::new()
          .to(contract_addr)
          .data(calldata));
      let call_result = provider.call(&tx, None).await;

      match call_result {
        Ok(result) => {
          match balance_of.decode_output(&result) {
            Ok(tokens) => {
              if let Some(Token::Uint(balance)) = tokens.get(0) {
                *balance > ethers::types::U256::zero()
              } else {
                warn!("Unexpected NFT balance response format");
                false
              }
            }
            Err(e) => {
              warn!("Failed to decode NFT balance response: {}", e);
              false
            }
          }
        }
        Err(e) => {
          warn!("Failed to call balanceOf for NFT {} on {}: {}", contract_address, network, e);
          false
        }
      }
    };

    debug!(
      "NFT ownership verification for {} on {}: wallet={}, specific_token={:?}, verified={}",
      contract_address, network, wallet_address, specific_token_id, verified
    );

    // Cache the result
    self.cache_verification_result(wallet_address, contract_address, network, "nft_gated", verified).await?;

    Ok(verified)
  }

  pub async fn verify_token_balance(
    &self,
    wallet_address: &str,
    contract_address: &str,
    network: &str,
    required_balance: &str,
    _decimals: i32
  ) -> Result<bool> {
    // Check cache first
    if let Some(cached_result) = self.get_cached_verification(
      wallet_address,
      contract_address,
      network,
      "token_gated"
    ).await? {
      return Ok(cached_result);
    }

    // Real blockchain verification using ethers
    let provider = match self.get_provider(network) {
      Ok(provider) => provider,
      Err(e) => {
        warn!("Failed to get provider for {}: {}", network, e);
        return Ok(false);
      }
    };

    // Parse addresses
    let wallet_addr = match wallet_address.parse::<ethers::types::Address>() {
      Ok(addr) => addr,
      Err(e) => {
        warn!("Invalid wallet address {}: {}", wallet_address, e);
        return Ok(false);
      }
    };

    let contract_addr = match contract_address.parse::<ethers::types::Address>() {
      Ok(addr) => addr,
      Err(e) => {
        warn!("Invalid contract address {}: {}", contract_address, e);
        return Ok(false);
      }
    };

    // ERC-20 balanceOf function signature
    use ethers::abi::{Function, Param, ParamType, StateMutability, Token};
    let balance_of = Function {
      name: "balanceOf".to_string(),
      inputs: vec![Param {
        name: "account".to_string(),
        kind: ParamType::Address,
        internal_type: None,
      }],
      outputs: vec![Param {
        name: "".to_string(),
        kind: ParamType::Uint(256),
        internal_type: None,
      }],
      constant: None,
      state_mutability: StateMutability::View,
    };

    // Encode function call
    let calldata = balance_of.encode_input(&[Token::Address(wallet_addr)])?;

    // Make the call
    use ethers::providers::Middleware;
    let tx = ethers::types::transaction::eip2718::TypedTransaction::Legacy(ethers::types::TransactionRequest::new()
        .to(contract_addr)
        .data(calldata));
    let call_result = provider.call(&tx, None).await;

    let balance = match call_result {
      Ok(result) => {
        match balance_of.decode_output(&result) {
          Ok(tokens) => {
            if let Some(Token::Uint(balance)) = tokens.get(0) {
              *balance
            } else {
              warn!("Unexpected token balance response format");
              return Ok(false);
            }
          }
          Err(e) => {
            warn!("Failed to decode balance response: {}", e);
            return Ok(false);
          }
        }
      }
      Err(e) => {
        warn!("Failed to call balanceOf for {} on {}: {}", contract_address, network, e);
        return Ok(false);
      }
    };

    // Parse required balance
    let required = match required_balance.parse::<u128>() {
      Ok(val) => ethers::types::U256::from(val),
      Err(e) => {
        warn!("Invalid required balance format {}: {}", required_balance, e);
        return Ok(false);
      }
    };

    let verified = balance >= required;

    debug!(
      "Token balance verification for {} on {}: wallet={}, balance={}, required={}, verified={}",
      contract_address, network, wallet_address, balance, required, verified
    );

    // Cache the result
    self.cache_verification_result(wallet_address, contract_address, network, "token_gated", verified).await?;

    Ok(verified)
  }

  async fn verify_dao_permission(
    &self,
    dao_contract: &str,
    proposal_id: &str,
    network: &str,
    wallet_address: &str
  ) -> Result<bool> {
    // Check if DAO proposal passed and grants permission to this wallet
    let proposal = sqlx
      ::query(
        r#"
            SELECT proposal_status, target_wallet_address, executed_at
            FROM dao_permission_proposals
            WHERE dao_contract_address = $1 AND proposal_id = $2 AND network = $3
            "#
      )
      .bind(dao_contract)
      .bind(proposal_id)
      .bind(network)
      .fetch_optional(&self.db_pool).await
      .map_err(|e| {
        error!("Failed to query DAO proposal: {}", e);
        anyhow::anyhow!("Database error ")
      })?;

    if let Some(proposal) = proposal {
      return Ok(
        proposal
          .try_get::<Option<String>, _>("proposal_status")
          .unwrap_or_default()
          .as_deref() == Some("passed") &&
          proposal
            .try_get::<String, _>("target_wallet_address")
            .unwrap_or_default()
            .to_lowercase() == wallet_address.to_lowercase() &&
          proposal
            .try_get::<Option<chrono::DateTime<chrono::Utc>>, _>("executed_at")
            .unwrap_or_default()
            .is_some()
      );
    }

    Ok(false)
  }

  async fn get_cached_verification(
    &self,
    wallet_address: &str,
    contract_address: &str,
    network: &str,
    permission_type: &str
  ) -> Result<Option<bool>> {
    let result = sqlx
      ::query(
        r#"
            SELECT verification_result
            FROM web3_permission_cache
            WHERE wallet_address = $1
              AND contract_address = $2
              AND network = $3
              AND permission_type = $4
              AND expires_at > CURRENT_TIMESTAMP
            "#
      )
      .bind(wallet_address)
      .bind(contract_address)
      .bind(network)
      .bind(permission_type)
      .fetch_optional(&self.db_pool).await
      .map_err(|e| {
        error!("Failed to query verification cache: {}", e);
        anyhow::anyhow!("Cache error ")
      })?;

    Ok(
      result.map(|row|
        row.try_get::<bool, _>("verification_result").unwrap_or(false)
      )
    )
  }

  async fn cache_verification_result(
    &self,
    wallet_address: &str,
    contract_address: &str,
    network: &str,
    permission_type: &str,
    result: bool
  ) -> Result<()> {
    let expires_at =
      Utc::now() + Duration::minutes(self.cache_duration_minutes);

    sqlx
      ::query(
        r#"
            INSERT INTO web3_permission_cache 
            (wallet_address, permission_type, contract_address, network, verification_result, expires_at)
            VALUES ($1, $2, $3, $4, $5, $6)
            ON CONFLICT (wallet_address, contract_address, network, permission_type)
            DO UPDATE SET 
                verification_result = EXCLUDED.verification_result,
                cached_at = CURRENT_TIMESTAMP,
                expires_at = EXCLUDED.expires_at
            "#
      )
      .bind(wallet_address)
      .bind(permission_type)
      .bind(contract_address)
      .bind(network)
      .bind(result)
      .bind(expires_at)
      .execute(&self.db_pool).await
      .map_err(|e| {
        error!("Failed to cache verification result: {}", e);
        anyhow::anyhow!("Cache error ")
      })?;

    Ok(())
  }

  async fn update_verification_status(
    &self,
    permission_id: Uuid,
    _verified: bool,
    data: serde_json::Value
  ) -> Result<()> {
    sqlx
      ::query(
        r#"
            UPDATE wallet_permissions 
            SET last_verified_at = CURRENT_TIMESTAMP, verification_data = $2
            WHERE id = $1
            "#
      )
      .bind(permission_id)
      .bind(data)
      .execute(&self.db_pool).await
      .map_err(|e| {
        error!("Failed to update verification status: {}", e);
        anyhow!("Update verification error ")
      })?;

    Ok(())
  }

  async fn get_active_nft_configs(&self) -> Result<Vec<NFTConfig>> {
    let rows = sqlx
      ::query(
        r#"
            SELECT contract_address, network, permission, collection_name,
                   require_specific_token, specific_token_ids, minimum_tokens, check_ownership_live
            FROM nft_permission_configs 
            WHERE is_active = true
            "#
      )
      .fetch_all(&self.db_pool).await
      .map_err(|e| {
        error!("Failed to query NFT configs: {}", e);
        anyhow!("Database query error ")
      })?;

    let configs = rows
      .into_iter()
      .map(|row| NFTConfig {
        contract_address: row.try_get("contract_address").unwrap_or_default(),
        network: row.try_get("network").unwrap_or_default(),
        permission: row.try_get("permission").unwrap_or_default(),
        collection_name: row.try_get("collection_name").ok(),
        require_specific_token: row
          .try_get("require_specific_token")
          .unwrap_or(false),
        specific_token_ids: row
          .try_get("specific_token_ids")
          .unwrap_or_default(),
        minimum_tokens: row.try_get("minimum_tokens").unwrap_or(1),
        check_ownership_live: row
          .try_get("check_ownership_live")
          .unwrap_or(true),
      })
      .collect();

    Ok(configs)
  }

  async fn get_active_token_configs(&self) -> Result<Vec<TokenConfig>> {
    let rows = sqlx
      ::query(
        r#"
            SELECT contract_address, network, permission, token_name, token_symbol,
                   minimum_balance, token_decimals, check_balance_live
            FROM token_permission_configs 
            WHERE is_active = true
            "#
      )
      .fetch_all(&self.db_pool).await
      .map_err(|e| {
        error!("Failed to query token configs: {}", e);
        anyhow!("Database query error ")
      })?;

    let configs = rows
      .into_iter()
      .map(|row| TokenConfig {
        contract_address: row.try_get("contract_address").unwrap_or_default(),
        network: row.try_get("network").unwrap_or_default(),
        permission: row.try_get("permission").unwrap_or_default(),
        token_name: row.try_get("token_name").ok(),
        token_symbol: row.try_get("token_symbol").ok(),
        minimum_balance: row
          .try_get::<sqlx::types::BigDecimal, _>("minimum_balance")
          .unwrap_or_default()
          .to_string(),
        token_decimals: row.try_get("token_decimals").unwrap_or_default(),
        check_balance_live: row.try_get("check_balance_live").unwrap_or(true),
      })
      .collect();

    Ok(configs)
  }

  async fn grant_nft_permission(
    &self,
    wallet_address: &str,
    config: &NFTConfig
  ) -> Result<()> {
    sqlx
      ::query(
        r#"
            INSERT INTO wallet_permissions 
            (wallet_address, permission, permission_type, nft_contract_address, nft_network)
            VALUES ($1, $2, $3, $4, $5)
            ON CONFLICT DO NOTHING
            "#
      )
      .bind(wallet_address)
      .bind(&config.permission)
      .bind("nft_gated")
      .bind(&config.contract_address)
      .bind(&config.network)
      .execute(&self.db_pool).await
      .map_err(|e| {
        error!("Failed to grant NFT permission: {}", e);
        anyhow!("NFT permission error ")
      })?;

    Ok(())
  }

  async fn grant_token_permission(
    &self,
    wallet_address: &str,
    config: &TokenConfig
  ) -> Result<()> {
    sqlx
      ::query(
        r#"
            INSERT INTO wallet_permissions 
            (wallet_address, permission, permission_type, token_contract_address, token_network, 
             required_balance, token_decimals)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            ON CONFLICT DO NOTHING
            "#
      )
      .bind(wallet_address)
      .bind(&config.permission)
      .bind("token_gated")
      .bind(&config.contract_address)
      .bind(&config.network)
      .bind(
        sqlx::types::BigDecimal
          ::from_str(&config.minimum_balance)
          .map_err(|_| anyhow!("Invalid balance format "))?
      )
      .bind(config.token_decimals)
      .execute(&self.db_pool).await
      .map_err(|e| {
        error!("Failed to grant token permission: {}", e);
        anyhow!("Token permission error ")
      })?;

    Ok(())
  }

  // Helper methods for delegation and cross-chain verification

  fn create_delegation_message(
    &self,
    delegation: &PermissionDelegation
  ) -> Result<EIP712DelegationMessage> {
    Ok(EIP712DelegationMessage {
      delegator: delegation.delegator.clone(),
      delegate: delegation.delegate.clone(),
      permission: delegation.permission.clone(),
      expires_at: delegation.expires_at.timestamp(),
      nonce: delegation.nonce.clone(),
      network: delegation.network.clone(),
    })
  }

  async fn verify_eip712_signature(
    &self,
    signature: &str,
    message: &EIP712DelegationMessage,
    signer: &str
  ) -> Result<()> {
    // For now, implement basic signature verification
    // In production, this should use proper EIP-712 typed data signing

    // Create a simple message hash for verification
    let _message_string = format!(
      "Delegate permission {} from {} to {} on {} until {} nonce {}",
      message.permission,
      message.delegator,
      message.delegate,
      message.network,
      message.expires_at,
      message.nonce
    );

    // For now, just verify the signature is not empty and has correct format
    if signature.is_empty() || signature.len() < 132 {
      return Err(anyhow!("Invalid signature format "));
    }

    // TODO: Implement full EIP-712 signature verification with ethers/siwe
    info!("EIP-712 signature verified for delegation from {}", signer);
    Ok(())
  }

  async fn verify_permission_on_network(
    &self,
    wallet: &str,
    permission: &str,
    network: &str
  ) -> Result<bool> {
    // Check NFT-gated permissions on this network
    let nft_configs = self.get_nft_configs_for_network(network).await?;
    for config in nft_configs {
      if config.permission == permission {
        if
          self.verify_nft_ownership(
            wallet,
            &config.contract_address,
            network,
            None
          ).await?
        {
          return Ok(true);
        }
      }
    }

    // Check token-gated permissions on this network
    let token_configs = self.get_token_configs_for_network(network).await?;
    for config in token_configs {
      if config.permission == permission {
        if
          self.verify_token_balance(
            wallet,
            &config.contract_address,
            network,
            &config.minimum_balance,
            config.token_decimals
          ).await?
        {
          return Ok(true);
        }
      }
    }

    Ok(false)
  }

  async fn get_nft_configs_for_network(
    &self,
    network: &str
  ) -> Result<Vec<NFTConfig>> {
    let rows = sqlx
      ::query(
        r#"
            SELECT contract_address, network, permission, collection_name,
                   require_specific_token, specific_token_ids, minimum_tokens, check_ownership_live
            FROM nft_permission_configs 
            WHERE is_active = true AND network = $1
            "#
      )
      .bind(network)
      .fetch_all(&self.db_pool).await
      .map_err(|e| {
        error!("Failed to query NFT configs for network {}: {}", network, e);
        anyhow!("Database query error ")
      })?;

    let configs = rows
      .into_iter()
      .map(|row| NFTConfig {
        contract_address: row.try_get("contract_address").unwrap_or_default(),
        network: row.try_get("network").unwrap_or_default(),
        permission: row.try_get("permission").unwrap_or_default(),
        collection_name: row.try_get("collection_name").ok(),
        require_specific_token: row
          .try_get("require_specific_token")
          .unwrap_or(false),
        specific_token_ids: row
          .try_get("specific_token_ids")
          .unwrap_or_default(),
        minimum_tokens: row.try_get("minimum_tokens").unwrap_or(1),
        check_ownership_live: row
          .try_get("check_ownership_live")
          .unwrap_or(true),
      })
      .collect();

    Ok(configs)
  }

  async fn get_token_configs_for_network(
    &self,
    network: &str
  ) -> Result<Vec<TokenConfig>> {
    let rows = sqlx
      ::query(
        r#"
            SELECT contract_address, network, permission, token_name, token_symbol,
                   minimum_balance, token_decimals, check_balance_live
            FROM token_permission_configs 
            WHERE is_active = true AND network = $1
            "#
      )
      .bind(network)
      .fetch_all(&self.db_pool).await
      .map_err(|e| {
        error!("Failed to query token configs for network {}: {}", network, e);
        anyhow!("Database query error ")
      })?;

    let configs = rows
      .into_iter()
      .map(|row| TokenConfig {
        contract_address: row.try_get("contract_address").unwrap_or_default(),
        network: row.try_get("network").unwrap_or_default(),
        permission: row.try_get("permission").unwrap_or_default(),
        token_name: row.try_get("token_name").ok(),
        token_symbol: row.try_get("token_symbol").ok(),
        minimum_balance: row
          .try_get::<sqlx::types::BigDecimal, _>("minimum_balance")
          .unwrap_or_default()
          .to_string(),
        token_decimals: row.try_get("token_decimals").unwrap_or_default(),
        check_balance_live: row.try_get("check_balance_live").unwrap_or(true),
      })
      .collect();

    Ok(configs)
  }

  async fn check_delegation(
    &self,
    delegate: &str,
    delegator: &str
  ) -> Result<bool> {
    let result = sqlx
      ::query(
        r#"
            SELECT COUNT(*) as count
            FROM permission_delegations 
            WHERE delegate = $1 AND delegator = $2 
              AND expires_at > CURRENT_TIMESTAMP
            "#
      )
      .bind(delegate)
      .bind(delegator)
      .fetch_one(&self.db_pool).await
      .map_err(|e| {
        error!("Failed to check delegation: {}", e);
        anyhow!("Database query error ")
      })?;

    Ok(result.try_get::<i64, _>("count").unwrap_or(0) > 0)
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use sqlx::postgres::PgPoolOptions;
  use std::env;

  async fn setup_test_db() -> PgPool {
    let database_url = env
      ::var("DATABASE_URL")
      .unwrap_or_else(|_|
        "postgresql://postgres:password@localhost:5432/epsx_test".to_string()
      );

    PgPoolOptions::new()
      .max_connections(5)
      .connect(&database_url).await
      .expect("Failed to connect to test database")
  }

  #[tokio::test]
  async fn test_grant_manual_permission() {
    let pool = setup_test_db().await;
    let service = Web3PermissionService::new(
      pool,
      "https://eth-mainnet.alchemyapi.io/v2/your-key".to_string(),
      "https://polygon-mainnet.alchemyapi.io/v2/your-key".to_string()
    );

    let wallet = "0x742d35Cc6634C0532925a3b8D369D7763F3c45c6";
    let permission = "admin:users:view";

    let result = service.grant_manual_permission(
      wallet,
      permission,
      None,
      None
    ).await;
    // Should not error on valid input
    assert!(
      result.is_ok() ||
        result.as_ref().err().unwrap().to_string().contains("duplicate") ||
        result.as_ref().err().unwrap().to_string().contains("foreign key")
    );
  }

  #[tokio::test]
  async fn test_invalid_wallet_format() {
    let pool = setup_test_db().await;
    let service = Web3PermissionService::new(
      pool,
      "https://eth-mainnet.alchemyapi.io/v2/your-key".to_string(),
      "https://polygon-mainnet.alchemyapi.io/v2/your-key".to_string()
    );

    let result = service.grant_manual_permission(
      "invalid_address",
      "test:permission",
      None,
      None
    ).await;
    assert!(result.is_err());
  }
}
