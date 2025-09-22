// Web3 Admin Permission Management Handlers
// Leverages existing Web3PermissionService for blockchain-based permission management

use axum::{ extract::{ Query, State }, http::StatusCode, response::Json };
use serde::{ Deserialize, Serialize };
use tracing::{ error, info, warn };
use uuid::Uuid;

// Import Web3 permission types
use crate::auth::{ NFTConfig, TokenConfig, DAOProposal };
use crate::web::auth::routes::AppState;

// Using structs from crate::auth module - no local duplicates needed

// Request/Response DTOs
#[derive(Debug, Deserialize)]
pub struct GrantPermissionRequest {
  pub wallet_address: String,
  pub permissions: Vec<String>,
  pub expires_at: Option<String>, // ISO 8601 datetime
  pub grant_reason: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct NFTGateRequest {
  pub contract_address: String,
  pub contract_name: String,
  pub permissions: Vec<String>,
  pub min_token_count: i32,
  pub network: Option<String>, // Default to "ethereum"
}

#[derive(Debug, Deserialize)]
pub struct TokenGateRequest {
  pub contract_address: String,
  pub token_symbol: String,
  pub permissions: Vec<String>,
  pub min_amount: String,
  pub token_decimals: Option<i32>, // Default to 18
  pub network: Option<String>, // Default to "ethereum"
}

#[derive(Debug, Deserialize)]
pub struct DAOProposalRequest {
  pub title: String,
  pub description: String,
  pub target_wallet: String,
  pub permissions: Vec<String>,
  pub votes_required: i32,
  pub expires_at: String, // ISO 8601 datetime
  pub dao_contract_address: Option<String>,
  pub network: Option<String>, // Default to "ethereum"
}

#[derive(Debug, Serialize)]
pub struct PermissionsResponse {
  pub permissions: Vec<WalletPermissionResponse>,
  pub total_count: usize,
}

#[derive(Debug, Serialize)]
pub struct WalletPermissionResponse {
  pub id: String,
  pub wallet_address: String,
  pub permission: String,
  pub source: String, // "manual", "nft", "token", "dao"
  pub expires_at: Option<String>,
  pub granted_at: String,
  pub granted_by: String,
  pub is_active: bool,
  pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Serialize)]
pub struct NFTGateResponse {
  pub id: String,
  pub contract_address: String,
  pub contract_name: String,
  pub permissions: Vec<String>,
  pub min_token_count: i32,
  pub network: String,
  pub is_active: bool,
  pub created_at: String,
}

#[derive(Debug, Serialize)]
pub struct TokenGateResponse {
  pub id: String,
  pub contract_address: String,
  pub token_symbol: String,
  pub permissions: Vec<String>,
  pub min_amount: String,
  pub token_decimals: i32,
  pub network: String,
  pub is_active: bool,
  pub created_at: String,
}

#[derive(Debug, Serialize)]
pub struct DAOProposalResponse {
  pub id: String,
  pub title: String,
  pub description: Option<String>,
  pub target_wallet: String,
  pub permissions: Vec<String>,
  pub votes_for: i32,
  pub votes_against: i32,
  pub votes_required: i32,
  pub status: String, // "pending", "approved", "rejected", "executed"
  pub created_at: String,
  pub expires_at: String,
}

#[derive(Debug, Deserialize)]
pub struct PermissionsQuery {
  pub wallet_address: Option<String>,
  pub permission: Option<String>,
  pub source: Option<String>,
  pub limit: Option<usize>,
  pub offset: Option<usize>,
}

// Handler: Get all wallet permissions with filtering
pub async fn get_wallet_permissions(
  Query(params): Query<PermissionsQuery>,
  State(app_state): State<AppState>
) -> Result<Json<PermissionsResponse>, StatusCode> {
  info!("🔍 Admin: Fetching wallet permissions with filters: {:?}", params);

  // If specific wallet is requested, get permissions for that wallet
  if let Some(wallet_address) = &params.wallet_address {
    match
      app_state.web3_permission_service.get_wallet_permissions(
        wallet_address
      ).await
    {
      Ok(permissions) => {
        let response_permissions: Vec<WalletPermissionResponse> = permissions
          .into_iter()
          .map(|p| {
            WalletPermissionResponse {
              id: Uuid::new_v4().to_string(), // Generate ID for frontend
              wallet_address: wallet_address.clone(),
              permission: p.permission,
              source: p.permission_type,
              expires_at: p.expires_at.map(|dt| dt.to_rfc3339()),
              granted_at: p.granted_at.to_rfc3339(),
              granted_by: "admin".to_string(), // TODO: Track actual granter
              is_active: p.is_active,
              metadata: p.verification_data,
            }
          })
          .collect();

        let response = PermissionsResponse {
          total_count: response_permissions.len(),
          permissions: response_permissions,
        };

        info!(
          "✅ Admin: Retrieved {} permissions for wallet {}",
          response.total_count,
          wallet_address
        );
        Ok(Json(response))
      }
      Err(e) => {
        error!(
          "❌ Admin: Failed to get permissions for wallet {}: {}",
          wallet_address,
          e
        );
        Err(StatusCode::INTERNAL_SERVER_ERROR)
      }
    }
  } else {
    // TODO: Implement database query to get all permissions across all wallets
    // For now, return empty response
    warn!("⚠️ Admin: Bulk permission listing not yet implemented");
    let response = PermissionsResponse {
      permissions: vec![],
      total_count: 0,
    };
    Ok(Json(response))
  }
}

// Handler: Grant manual permission to wallet
pub async fn grant_manual_permission(
  State(app_state): State<AppState>,
  Json(request): Json<GrantPermissionRequest>
) -> Result<Json<serde_json::Value>, StatusCode> {
  info!(
    "🎯 Admin: Granting manual permissions to wallet: {}",
    request.wallet_address
  );

  // Parse expiration date if provided
  let expires_at = if let Some(expires_str) = &request.expires_at {
    match chrono::DateTime::parse_from_rfc3339(expires_str) {
      Ok(dt) => Some(dt.with_timezone(&chrono::Utc)),
      Err(e) => {
        error!("❌ Admin: Invalid expires_at format: {}", e);
        return Err(StatusCode::BAD_REQUEST);
      }
    }
  } else {
    None
  };

  // Grant each permission
  let mut granted_permissions = Vec::new();
  for permission in &request.permissions {
    match
      app_state.web3_permission_service.grant_manual_permission(
        &request.wallet_address,
        permission,
        Some(Uuid::new_v4()), // TODO: Use actual admin user ID (granted_by)
        expires_at // expires_at
      ).await
    {
      Ok(_permission_id) => {
        info!(
          "✅ Admin: Granted permission '{}' to wallet {}",
          permission,
          request.wallet_address
        );
        granted_permissions.push(permission.clone());
      }
      Err(e) => {
        error!(
          "❌ Admin: Failed to grant permission '{}' to wallet {}: {}",
          permission,
          request.wallet_address,
          e
        );
        // Continue with other permissions instead of failing completely
      }
    }
  }

  if granted_permissions.is_empty() {
    error!("❌ Admin: No permissions were granted successfully");
    return Err(StatusCode::INTERNAL_SERVER_ERROR);
  }

  let response =
    serde_json::json!({
        "success": true,
        "granted_permissions": granted_permissions,
        "wallet_address": request.wallet_address,
        "message": format!("Successfully granted {} permission(s)", granted_permissions.len())
    });

  info!(
    "🎉 Admin: Successfully granted {} permission(s) to wallet {}",
    granted_permissions.len(),
    request.wallet_address
  );
  Ok(Json(response))
}

// Handler: Create NFT permission gate
pub async fn create_nft_gate(
  State(app_state): State<AppState>,
  Json(request): Json<NFTGateRequest>
) -> Result<Json<serde_json::Value>, StatusCode> {
  info!(
    "🎨 Admin: Creating NFT gate for contract: {}",
    request.contract_address
  );

  let network = request.network.unwrap_or_else(|| "ethereum".to_string());

  // Create NFT config for each permission
  let mut created_gates = Vec::new();
  for permission in &request.permissions {
    let nft_config = NFTConfig {
      contract_address: request.contract_address.clone(),
      network: network.clone(),
      permission: permission.clone(),
      collection_name: Some(request.contract_name.clone()),
      require_specific_token: false,
      specific_token_ids: vec![],
      minimum_tokens: request.min_token_count as i32,
      check_ownership_live: true, // Enable live blockchain verification
    };

    match
      app_state.web3_permission_service.configure_nft_permission(
        nft_config,
        Uuid::new_v4() // TODO: Use actual admin user ID
      ).await
    {
      Ok(_gate_id) => {
        info!(
          "✅ Admin: Created NFT gate for permission '{}' on {}",
          permission,
          network
        );
        created_gates.push(permission.clone());
      }
      Err(e) => {
        error!(
          "❌ Admin: Failed to create NFT gate for permission '{}': {}",
          permission,
          e
        );
      }
    }
  }

  if created_gates.is_empty() {
    error!("❌ Admin: No NFT gates were created successfully");
    return Err(StatusCode::INTERNAL_SERVER_ERROR);
  }

  let response =
    serde_json::json!({
        "success": true,
        "created_permissions": created_gates,
        "contract_address": request.contract_address,
        "network": network,
        "message": format!("Successfully created {} NFT gate(s)", created_gates.len())
    });

  info!(
    "🎉 Admin: Successfully created {} NFT gate(s) for contract {}",
    created_gates.len(),
    request.contract_address
  );
  Ok(Json(response))
}

// Handler: Create token permission gate
pub async fn create_token_gate(
  State(app_state): State<AppState>,
  Json(request): Json<TokenGateRequest>
) -> Result<Json<serde_json::Value>, StatusCode> {
  info!(
    "🪙 Admin: Creating token gate for contract: {}",
    request.contract_address
  );

  let network = request.network.unwrap_or_else(|| "ethereum".to_string());
  let decimals = request.token_decimals.unwrap_or(18);

  // Create token config for each permission
  let mut created_gates = Vec::new();
  for permission in &request.permissions {
    let token_config = TokenConfig {
      contract_address: request.contract_address.clone(),
      network: network.clone(),
      permission: permission.clone(),
      token_name: None,
      token_symbol: Some(request.token_symbol.clone()),
      minimum_balance: request.min_amount.clone(),
      token_decimals: decimals as i32,
      check_balance_live: true, // Enable live blockchain verification
    };

    match
      app_state.web3_permission_service.configure_token_permission(
        token_config,
        Uuid::new_v4() // TODO: Use actual admin user ID
      ).await
    {
      Ok(_gate_id) => {
        info!(
          "✅ Admin: Created token gate for permission '{}' on {}",
          permission,
          network
        );
        created_gates.push(permission.clone());
      }
      Err(e) => {
        error!(
          "❌ Admin: Failed to create token gate for permission '{}': {}",
          permission,
          e
        );
      }
    }
  }

  if created_gates.is_empty() {
    error!("❌ Admin: No token gates were created successfully");
    return Err(StatusCode::INTERNAL_SERVER_ERROR);
  }

  let response =
    serde_json::json!({
        "success": true,
        "created_permissions": created_gates,
        "contract_address": request.contract_address,
        "token_symbol": request.token_symbol,
        "network": network,
        "message": format!("Successfully created {} token gate(s)", created_gates.len())
    });

  info!(
    "🎉 Admin: Successfully created {} token gate(s) for contract {}",
    created_gates.len(),
    request.contract_address
  );
  Ok(Json(response))
}

// Handler: Create DAO proposal
pub async fn create_dao_proposal(
  State(app_state): State<AppState>,
  Json(request): Json<DAOProposalRequest>
) -> Result<Json<serde_json::Value>, StatusCode> {
  info!("🗳️ Admin: Creating DAO proposal: {}", request.title);

  let network = request.network.unwrap_or_else(|| "ethereum".to_string());
  let dao_contract = request.dao_contract_address.unwrap_or_else(|| {
    // Default DAO contract addresses per network
    match network.as_str() {
      "ethereum" => "0x0000000000000000000000000000000000000000".to_string(),
      "polygon" => "0x0000000000000000000000000000000000000000".to_string(),
      "bsc" => "0x0000000000000000000000000000000000000000".to_string(),
      _ => "0x0000000000000000000000000000000000000000".to_string(),
    }
  });

  // Parse expiration date
  let _expires_at = match
    chrono::DateTime::parse_from_rfc3339(&request.expires_at)
  {
    Ok(dt) => dt.with_timezone(&chrono::Utc),
    Err(e) => {
      error!("❌ Admin: Invalid expires_at format: {}", e);
      return Err(StatusCode::BAD_REQUEST);
    }
  };

  // Create DAO proposal for each permission
  let mut created_proposals = Vec::new();
  for permission in &request.permissions {
    let dao_proposal = DAOProposal {
      dao_contract_address: dao_contract.clone(),
      network: network.clone(),
      proposal_id: Uuid::new_v4().to_string(),
      title: request.title.clone(),
      description: Some(request.description.clone()),
      target_wallet_address: request.target_wallet.clone(),
      permission: permission.clone(),
      proposal_status: "pending".to_string(),
      voting_end: None, // No voting deadline specified
    };

    match
      app_state.web3_permission_service.create_dao_proposal(dao_proposal).await
    {
      Ok(_proposal_id) => {
        info!(
          "✅ Admin: Created DAO proposal for permission '{}' on {}",
          permission,
          network
        );
        created_proposals.push(permission.clone());
      }
      Err(e) => {
        error!(
          "❌ Admin: Failed to create DAO proposal for permission '{}': {}",
          permission,
          e
        );
      }
    }
  }

  if created_proposals.is_empty() {
    error!("❌ Admin: No DAO proposals were created successfully");
    return Err(StatusCode::INTERNAL_SERVER_ERROR);
  }

  let response =
    serde_json::json!({
        "success": true,
        "created_permissions": created_proposals,
        "title": request.title,
        "target_wallet": request.target_wallet,
        "network": network,
        "message": format!("Successfully created {} DAO proposal(s)", created_proposals.len())
    });

  info!(
    "🎉 Admin: Successfully created {} DAO proposal(s): {}",
    created_proposals.len(),
    request.title
  );
  Ok(Json(response))
}

// Handler: Get NFT gates (placeholder for now)
pub async fn get_nft_gates(State(_app_state): State<AppState>) -> Result<
  Json<serde_json::Value>,
  StatusCode
> {
  info!("🎨 Admin: Fetching NFT gates");

  // TODO: Implement database query to get all NFT gate configurations
  let response =
    serde_json::json!({
        "nft_gates": [],
        "total_count": 0
    });

  Ok(Json(response))
}

// Handler: Get token gates (placeholder for now)
pub async fn get_token_gates(State(_app_state): State<AppState>) -> Result<
  Json<serde_json::Value>,
  StatusCode
> {
  info!("🪙 Admin: Fetching token gates");

  // TODO: Implement database query to get all token gate configurations
  let response =
    serde_json::json!({
        "token_gates": [],
        "total_count": 0
    });

  Ok(Json(response))
}

// Handler: Get DAO proposals (placeholder for now)
pub async fn get_dao_proposals(State(_app_state): State<AppState>) -> Result<
  Json<serde_json::Value>,
  StatusCode
> {
  info!("🗳️ Admin: Fetching DAO proposals");

  // TODO: Implement database query to get all DAO proposals
  let response =
    serde_json::json!({
        "proposals": [],
        "total_count": 0
    });

  Ok(Json(response))
}
