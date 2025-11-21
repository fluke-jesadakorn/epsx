/**
 * Diesel Models for Wallet Users
 *
 * Database models for wallet_users table using Diesel ORM
 */

use chrono::{DateTime, Utc};
use diesel::{Queryable, Selectable, Insertable, AsChangeset};
use serde::Deserialize;

/// Diesel Queryable model for wallet_users table
#[derive(Debug, Clone, Queryable, Selectable)]
#[diesel(table_name = crate::schema::wallet_users)]
pub struct WalletUserDb {
    pub wallet_address: String,
    pub is_active: bool,
    pub tier_level: Option<String>,
    pub wallet_metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub last_auth_at: Option<DateTime<Utc>>,
}

/// Diesel Insertable model for creating new wallet users
#[derive(Debug, Clone, Insertable)]
#[diesel(table_name = crate::schema::wallet_users)]
pub struct NewWalletUserDb {
    pub wallet_address: String,
    pub is_active: bool,
    pub wallet_metadata: serde_json::Value,
}

/// Diesel AsChangeset model for updating wallet users
#[derive(Debug, Clone, AsChangeset)]
#[diesel(table_name = crate::schema::wallet_users)]
pub struct UpdateWalletUserDb {
    pub is_active: Option<bool>,
    pub wallet_metadata: Option<serde_json::Value>,
    pub updated_at: Option<DateTime<Utc>>,
}

/// Form data for wallet user updates from API requests
#[derive(Debug, Deserialize)]
pub struct UpdateWalletUserRequest {
    pub is_active: Option<bool>,
    pub wallet_metadata: Option<serde_json::Value>,
}