// Extend Assignment Command
// Extend stock ranking assignment expiration

use serde::{Deserialize, Serialize};
use crate::application::shared::Command;

#[derive(Debug, Clone)]
pub struct ExtendAssignmentCommand {
    pub assignment_id: String,
    pub extension_days: i32,
    pub reason: Option<String>,
    pub performed_by: String,
}

impl Command for ExtendAssignmentCommand {
    type Response = ExtendAssignmentResponse;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtendAssignmentResponse {
    pub success: bool,
    pub assignment_id: String,
    pub old_expires_at: Option<chrono::DateTime<chrono::Utc>>,
    pub new_expires_at: Option<chrono::DateTime<chrono::Utc>>,
    pub extended_days: i32,
    pub message: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}
