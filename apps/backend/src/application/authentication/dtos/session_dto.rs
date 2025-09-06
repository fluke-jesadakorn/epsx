use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionDto {
    pub session_id: String,
    pub user_id: String,
    pub is_active: bool,
    pub created_at: String,
    pub expires_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateSessionDto {
    pub user_id: String,
    pub device_info: Option<String>,
}