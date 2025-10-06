use crate::application::shared::{Query, ApplicationResult};
use serde::{Deserialize, Serialize};

/// Query to get all sessions for a user
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetWalletSessionsQuery {
    pub wallet_address: String,
    pub include_expired: bool,
}

/// User sessions response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetWalletSessionsResponse {
    pub wallet_address: String,
    pub active_sessions: Vec<SessionSummary>,
    pub expired_sessions: Vec<SessionSummary>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionSummary {
    pub session_id: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub expires_at: chrono::DateTime<chrono::Utc>,
    pub is_valid: bool,
    pub ip_address: Option<String>,
}

impl Query for GetWalletSessionsQuery {
    type Response = GetWalletSessionsResponse;
    
    fn validate(&self) -> ApplicationResult<()> {
        // TODO: Implement validation
        Ok(())
    }
}