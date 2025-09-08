use serde::{Deserialize, Serialize};
use crate::application::shared::{Command, ApplicationResult};

/// Command to revoke a permission from a user
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RevokePermissionCommand {
    pub user_id: String,
    pub permission: String,
    pub revoked_by: Option<String>,
    pub reason: Option<String>,
}

/// Response after successful permission revocation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RevokePermissionResponse {
    pub user_id: String,
    pub permission: String,
    pub revoked_at: chrono::DateTime<chrono::Utc>,
}

impl Command for RevokePermissionCommand {
    type Response = RevokePermissionResponse;
    
    fn validate(&self) -> ApplicationResult<()> {
        // TODO: Implement validation
        Ok(())
    }
}