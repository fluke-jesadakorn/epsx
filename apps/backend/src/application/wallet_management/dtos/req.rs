// Request DTOs for Wallet Management

use serde::{Deserialize, Serialize};
use utoipa::{ToSchema, IntoParams};

// ============================================================================
// AUTH REQUEST DTOS
// ============================================================================

#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct ChallengeRequest {
    /// Ethereum wallet address
    #[schema(example = "0x1234567890123456789012345678901234567890")]
    pub wallet_address: String,
}

#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct SignatureVerificationRequest {
    /// SIWE message that was signed
    pub message: String,
    /// Cryptographic signature from wallet
    pub signature: String,
    /// Ethereum wallet address
    #[schema(example = "0x1234567890123456789012345678901234567890")]
    pub wallet_address: String,
    /// Challenge nonce
    #[schema(example = "abc123def456")]
    pub nonce: String,
}

#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct LogoutRequest {
    /// Ethereum wallet address to logout
    #[schema(example = "0x1234567890123456789012345678901234567890")]
    pub wallet_address: String,
}

#[derive(Debug, Deserialize, Serialize, ToSchema, IntoParams)]
pub struct PermissionCheckQuery {
    /// Permission to check (format: platform:resource:action)
    #[schema(example = "epsx:analytics:read")]
    pub permission: String,
}

#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct SessionVerificationRequest {
    /// Whether to check for admin context (admin permissions required)
    #[schema(example = true)]
    pub admin_context: Option<bool>,
}

// ============================================================================
// WALLET MANAGEMENT REQUEST DTOS
// ============================================================================

#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct UpdateWalletRequest {
    pub wallet_address: String,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct GrantPermissionRequest {
    pub wallet_address: String,
    pub permission: String,
    pub expires_at: Option<chrono::DateTime<chrono::Utc>>,
}
