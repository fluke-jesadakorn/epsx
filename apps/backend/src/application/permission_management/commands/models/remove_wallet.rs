use crate::prelude::*;
use crate::application::shared::Command;

/// Command to remove a wallet from a permission plan
#[derive(Debug, Clone)]
pub struct RemoveWalletFromPlanCommand {
    pub plan_id: String,
    pub wallet_address: String,
}

impl Command for RemoveWalletFromPlanCommand {
    type Response = RemoveWalletFromPlanResponse;
}

/// Response for remove wallet from plan command
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemoveWalletFromPlanResponse {
    pub plan_id: String,
    pub wallet_address: String,
    pub removed: bool,
}
