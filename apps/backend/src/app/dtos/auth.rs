// Authentication DTOs

use serde::{Serialize, Deserialize};
use crate::dom::values::{UserId, Role};

#[derive(Debug, Serialize, Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
    pub firebase_token: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LoginResponse {
    pub user_id: UserId,
    pub role: Role,
    pub access_token: String,
    pub expires_in: i64,
    pub sess_id: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RefreshTokenRequest {
    pub refresh_token: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserSession {
    pub user_id: UserId,
    pub role: Role,
    pub permissions: Vec<String>,
    pub expires_at: chrono::DateTime<chrono::Utc>,
}

// Aliases for compatibility
pub type LoginReq = LoginRequest;
pub type LoginRes = LoginResponse;
pub type RefreshReq = RefreshTokenRequest;

#[derive(Debug, Serialize, Deserialize)]
pub struct LogoutReq {
    pub session_id: String,
    pub sess_id: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ValidateReq {
    pub token: String,
    pub sess_id: String,
}