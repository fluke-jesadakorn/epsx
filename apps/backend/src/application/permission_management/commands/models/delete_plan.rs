use crate::prelude::*;
use crate::application::shared::Command;

/// Command to delete a permission plan
#[derive(Debug, Clone)]
pub struct DeletePermissionPlanCommand {
    pub plan_id: String,
}

impl Command for DeletePermissionPlanCommand {
    type Response = DeletePermissionPlanResponse;
}

/// Response for delete permission plan command
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeletePermissionPlanResponse {
    pub plan_id: String,
    pub deleted: bool,
}
