use crate::application::shared::{ApplicationResult, Command};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub struct DisableWalletCommand {
    #[serde(default)]
    pub wallet_address: String,
    #[serde(default)]
    pub admin_wallet_address: String, // Who performed the action
    pub duration_days: Option<i32>, // None = until manual re-enable
    pub reason_category: String,
    pub reason_details: String,
    pub affected_platforms: Vec<String>,
    pub block_login: bool,
    pub pause_subscriptions: bool,
    pub notify_user: bool,
}

impl Command for DisableWalletCommand {
    type Response = DisableWalletResponse;

    fn validate(&self) -> ApplicationResult<()> {
        if self.wallet_address.trim().is_empty() {
             return Err(crate::application::shared::ApplicationError::validation(
                "wallet_address",
                "Wallet address cannot be empty",
            ));
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct DisableWalletResponse {
    pub success: bool,
    pub message: String,
    pub disabled_until: Option<chrono::DateTime<chrono::Utc>>,
}
