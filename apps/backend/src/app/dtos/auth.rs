// Authentication DTOs

use serde::{Serialize, Deserialize};
use crate::dom::values::UserId;

#[derive(Debug, Serialize, Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LoginResponse {
    pub user_id: UserId,
    pub package_tier: String,
    pub permissions: Vec<String>,
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
    pub package_tier: String,
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

#[derive(Debug, Serialize, Deserialize)]
pub struct AutoRegistrationRequest {
    pub email: String,
    pub password: String,
    pub package_tier: String,
    pub referral_code: Option<String>,
    pub source: String,
    pub region: Option<String>,
    pub utm_source: Option<String>,
    pub utm_campaign: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RegistrationResponse {
    pub user_id: UserId,
    pub access_token: String,
    pub expires_in: i64,
    pub features_unlocked: Vec<String>,
    pub total_features_assigned: u32,
    pub assignment_results: Vec<FeatureAssignmentResult>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FeatureAssignmentResult {
    pub feature_id: String,
    pub profile_name: String,
    pub success: bool,
    pub reason: String,
    pub expires_at: Option<chrono::DateTime<chrono::Utc>>,
}