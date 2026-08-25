//! Database models for wallet_users (sqlx-friendly).
//!
//! BIG-BANG: migrated from diesel to plain sqlx structs.

use chrono::{DateTime, Utc};
use serde::Deserialize;

/// Row model for wallet_users table (sqlx).
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, sqlx::FromRow)]
pub struct WalletUserDb {
    pub wallet_address: String,
    pub is_active: bool,
    pub tier_level: String,
    pub wallet_metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub last_auth_at: Option<DateTime<Utc>>,
    pub permission_plans: Option<serde_json::Value>,
    pub disable_info: Option<serde_json::Value>,
}

/// Insert model for creating new wallet users.
#[derive(Debug, Clone, serde::Deserialize)]
pub struct NewWalletUserDb {
    pub wallet_address: String,
    pub is_active: bool,
    pub tier_level: String,
    pub wallet_metadata: serde_json::Value,
}

/// Update model for wallet users.
#[derive(Debug, Clone, serde::Deserialize, Default)]
pub struct UpdateWalletUserDb {
    pub is_active: Option<bool>,
    pub wallet_metadata: Option<serde_json::Value>,
    pub updated_at: Option<DateTime<Utc>>,
}

/// Form data for wallet user updates from API requests.
#[derive(Debug, Deserialize)]
pub struct UpdateWalletUserRequest {
    pub is_active: Option<bool>,
    pub wallet_metadata: Option<serde_json::Value>,
}