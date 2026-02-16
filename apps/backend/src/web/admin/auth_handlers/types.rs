// Request/Response DTOs for Web3 Admin Permission Management

use serde::{Deserialize, Serialize};

// Request DTOs
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

// Response DTOs
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

// Query DTOs
#[derive(Debug, Deserialize)]
pub struct PermissionsQuery {
  pub wallet_address: Option<String>,
  pub permission: Option<String>,
  pub source: Option<String>,
  pub limit: Option<usize>,
  pub offset: Option<usize>,
}

#[derive(Debug, Deserialize)]
pub struct RecentWalletsQuery {
  pub limit: Option<i32>,
  pub days: Option<i32>,
}

#[derive(Debug, Deserialize)]
pub struct WalletSearchQuery {
  pub page: Option<i32>,
  pub limit: Option<i32>,
  pub search: Option<String>,
  pub tier: Option<String>,
  pub status: Option<String>,
  pub date_range: Option<String>,
  pub has_permissions: Option<String>,
  pub sort_by: Option<String>,
  pub sort_order: Option<String>,
  pub exclude_plan_id: Option<String>,
}
