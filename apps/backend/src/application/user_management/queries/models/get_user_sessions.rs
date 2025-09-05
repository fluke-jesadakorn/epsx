use serde::{Serialize, Deserialize};
use crate::application::shared::{Query, ApplicationResult};

/// Query to get all sessions for a user
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetUserSessionsQuery {
    pub user_id: String,
    pub include_expired: bool,
}

/// User sessions response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetUserSessionsResponse {
    pub user_id: String,
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

impl Query for GetUserSessionsQuery {
    type Response = GetUserSessionsResponse;
    
    fn validate(&self) -> ApplicationResult<()> {
        // TODO: Implement validation
        Ok(())
    }
}