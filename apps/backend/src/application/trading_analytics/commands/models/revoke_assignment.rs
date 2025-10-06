// Revoke Assignment Command
// Revoke stock ranking assignment

use serde::{Deserialize, Serialize};
use crate::application::shared::Command;

#[derive(Debug, Clone)]
pub struct RevokeAssignmentCommand {
    pub assignment_id: String,
    pub reason: String,
    pub performed_by: String,
}

impl Command for RevokeAssignmentCommand {
    type Response = RevokeAssignmentResponse;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RevokeAssignmentResponse {
    pub success: bool,
    pub assignment_id: String,
    pub wallet_address: String,
    pub package_name: String,
    pub revoked_at: chrono::DateTime<chrono::Utc>,
    pub reason: String,
    pub message: String,
}
