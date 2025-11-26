use serde::{Deserialize, Serialize};
use crate::application::shared::{Command, ApplicationResult};

/// Command to invalidate a user session
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InvalidateSessionCommand {
    pub session_id: String,
    pub reason: String,
}

/// Response after successful session invalidation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InvalidateSessionResponse {
    pub session_id: String,
    pub invalidated_at: chrono::DateTime<chrono::Utc>,
}

impl Command for InvalidateSessionCommand {
    type Response = InvalidateSessionResponse;

    fn validate(&self) -> ApplicationResult<()> {
        Ok(())
    }
}