// List User Sessions Query Model
use serde::{Deserialize, Serialize};

use crate::domain::shared_kernel::value_objects::UserId;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListUserSessionsQuery {
    pub user_id: UserId,
    pub include_expired: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserSessionInfo {
    pub session_id: String,
    pub created_at: String,
    pub expires_at: String,
    pub is_active: bool,
    pub device_info: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListUserSessionsResponse {
    pub sessions: Vec<UserSessionInfo>,
    pub total_count: usize,
}