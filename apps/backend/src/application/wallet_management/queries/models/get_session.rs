use serde::{Deserialize, Serialize};
use crate::application::shared::{Query, ApplicationResult};

/// Query to get session information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetSessionQuery {
    pub session_id: String,
    pub include_security_info: bool,
}

/// Session information response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetSessionResponse {
    pub session_id: String,
    pub wallet_address: String,
    pub is_valid: bool,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub expires_at: chrono::DateTime<chrono::Utc>,
}

impl Query for GetSessionQuery {
    type Response = GetSessionResponse;

    fn validate(&self) -> ApplicationResult<()> {
        // Minimal validation for read queries
        // Session ID format and ownership validated in handler
        Ok(())
    }
}