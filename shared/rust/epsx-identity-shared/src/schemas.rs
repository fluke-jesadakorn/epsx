//! Schema row types — sqlx canonical (Diesel removed).
//!
//! Previously held `diesel::table!` macros. Now plain `sqlx::FromRow` structs
//! for the three tables used in `token_service` and `unified_permission_service`.
//! Kept for import compatibility; new code should query via raw sqlx.

use chrono::{DateTime, Utc};
use serde_json::Value as JsonValue;
use uuid::Uuid;

pub mod primary {
    use super::*;

    #[derive(Debug, Clone, sqlx::FromRow)]
    pub struct OpenIdRefreshTokenRow {
        pub token_id: String,
        pub wallet_address: String,
        pub client_id: Option<String>,
        pub family_id: Option<Uuid>,
        pub token_digest: Option<Vec<u8>>,
        pub digest_key_id: Option<String>,
        pub digest_version: Option<i16>,
        pub storage_version: Option<i16>,
        pub expires_at: DateTime<Utc>,
        pub created_at: DateTime<Utc>,
        pub is_revoked: bool,
        pub consumed_at: Option<DateTime<Utc>>,
        pub revoked_at: Option<DateTime<Utc>>,
        pub replay_detected_at: Option<DateTime<Utc>>,
    }

    #[derive(Debug, Clone, sqlx::FromRow)]
    pub struct WalletUserRow {
        pub wallet_address: String,
        pub is_active: bool,
        pub tier_level: String,
        pub wallet_metadata: Option<JsonValue>,
        pub last_auth_at: Option<DateTime<Utc>>,
        pub updated_at: DateTime<Utc>,
        pub created_at: DateTime<Utc>,
    }

    #[derive(Debug, Clone, sqlx::FromRow)]
    pub struct Web3AuthNonceRow {
        pub wallet_address: String,
        pub nonce: String,
        pub message: String,
        pub expires_at: DateTime<Utc>,
        pub created_at: DateTime<Utc>,
    }
}
