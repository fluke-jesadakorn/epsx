// Response DTOs for Wallet Management

use serde::Serialize;
use utoipa::ToSchema;

// ============================================================================
// AUTH RESPONSE DTOS
// ============================================================================

#[derive(Debug, Serialize, ToSchema)]
pub struct SessionVerificationResponse {
    /// Whether the verification was successful
    pub success: bool,
    /// Whether the user is authenticated
    pub authenticated: Option<bool>,
    /// User's wallet address (if authenticated)
    pub wallet_address: Option<String>,
    /// User's ID (if authenticated)
    pub user_id: Option<String>,
    /// User's permissions (if authenticated)
    pub permissions: Option<Vec<String>>,
    /// Whether user has admin permissions
    pub is_admin: Option<bool>,
    /// Session expiry (if authenticated)
    pub expires: Option<String>,
    /// Error message (if verification failed)
    pub error: Option<String>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct Web3ChallengeResponse {
    pub success: bool,
    pub nonce: String,
    pub message: String,
    pub expires_at: String,
    pub wallet_address: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct Web3VerifyResponse {
    pub success: bool,
    pub wallet_address: String,
    pub token: String,
    pub permissions: Vec<String>,
    pub groups: Vec<String>,
    pub expires_at: String,
}

// ============================================================================
// WALLET MANAGEMENT RESPONSE DTOS
// ============================================================================

#[derive(Debug, Serialize, ToSchema)]
pub struct WalletResponse {
    pub id: String,
    pub wallet_address: String,
    pub created_at: String,
    pub updated_at: String,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct PermissionGrantResponse {
    pub success: bool,
    pub wallet_address: String,
    pub permission: String,
    pub granted_at: String,
    pub expires_at: Option<String>,
}
