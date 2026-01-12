// List Wallet Sessions Query Model
use serde::{ Deserialize, Serialize };

use crate::application::shared::{ Query, ApplicationResult };
use crate::domain::shared_kernel::value_objects::UserId;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListWalletSessionsQuery {
  pub wallet_address: UserId,
  pub include_expired: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalletSessionInfo {
  pub sid: String,
  pub created_at: String,
  pub expires_at: String,
  pub is_active: bool,
  pub device_info: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListWalletSessionsResponse {
  pub sessions: Vec<WalletSessionInfo>,
  pub total_count: usize,
}

impl Query for ListWalletSessionsQuery {
  type Response = ListWalletSessionsResponse;

  fn validate(&self) -> ApplicationResult<()> {
    // Wallet address validation handled by UserId type
    Ok(())
  }
}
