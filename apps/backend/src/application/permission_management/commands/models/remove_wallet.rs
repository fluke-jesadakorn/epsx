use crate::prelude::*;
use crate::application::shared::Command;

/// Command to remove a wallet from a permission group
#[derive(Debug, Clone)]
pub struct RemoveWalletFromGroupCommand {
    pub group_id: String,
    pub wallet_address: String,
}

impl Command for RemoveWalletFromGroupCommand {
    type Response = RemoveWalletFromGroupResponse;
}

/// Response for remove wallet from group command
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemoveWalletFromGroupResponse {
    pub group_id: String,
    pub wallet_address: String,
    pub removed: bool,
}
