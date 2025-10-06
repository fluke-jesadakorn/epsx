use crate::prelude::*;
use crate::application::shared::Command;

/// Command to delete a permission group
#[derive(Debug, Clone)]
pub struct DeletePermissionGroupCommand {
    pub group_id: String,
}

impl Command for DeletePermissionGroupCommand {
    type Response = DeletePermissionGroupResponse;
}

/// Response for delete permission group command
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeletePermissionGroupResponse {
    pub group_id: String,
    pub deleted: bool,
}
