use crate::prelude::*;
use crate::application::shared::Command;

/// Command to assign a wallet to a permission group
#[derive(Debug, Clone)]
pub struct AssignWalletToGroupCommand {
    pub group_id: String,
    pub wallet_address: String,
    pub assigned_by: Option<String>,
    pub expires_at: Option<chrono::DateTime<chrono::Utc>>,
}

impl Command for AssignWalletToGroupCommand {
    type Response = AssignWalletToGroupResponse;
}

/// Response for assign wallet to group command
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssignWalletToGroupResponse {
    pub group_id: String,
    pub wallet_address: String,
    pub assigned_at: chrono::DateTime<chrono::Utc>,
}
