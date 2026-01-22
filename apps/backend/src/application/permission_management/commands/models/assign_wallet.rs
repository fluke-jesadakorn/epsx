use crate::prelude::*;
use crate::application::shared::Command;

/// Command to assign a wallet to a permission plan
#[derive(Debug, Clone)]
pub struct AssignWalletToPlanCommand {
    pub plan_id: String,
    pub wallet_address: String,
    pub assigned_by: Option<String>,
    pub expires_at: Option<chrono::DateTime<chrono::Utc>>,
}

impl Command for AssignWalletToPlanCommand {
    type Response = AssignWalletToPlanResponse;
}

/// Response for assign wallet to plan command
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssignWalletToPlanResponse {
    pub plan_id: String,
    pub wallet_address: String,
    pub assigned_at: chrono::DateTime<chrono::Utc>,
}
