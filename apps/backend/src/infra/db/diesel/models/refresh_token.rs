use diesel::prelude::*;
use std::net::IpAddr;
use uuid::Uuid;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::infra::db::diesel::schema::refresh_tokens;
use crate::infra::db::diesel::types::DieselIpAddr;

/// Diesel model for refresh_tokens table
#[derive(Queryable, Selectable, Identifiable, Serialize, Deserialize, Debug, Clone)]
#[diesel(table_name = refresh_tokens)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct RefreshToken {
    pub id: Uuid,
    pub user_id: String,
    pub token_hash: String,
    pub family_id: Uuid,
    pub expires_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub last_used_at: Option<DateTime<Utc>>,
    pub device_info: Option<serde_json::Value>,
    pub ip_address: Option<DieselIpAddr>,
    pub user_agent: Option<String>,
    pub is_revoked: bool,
    pub revoked_at: Option<DateTime<Utc>>,
    pub revoked_reason: Option<String>,
}

/// Diesel model for inserting new refresh tokens
#[derive(Insertable, Serialize, Deserialize, Debug, Clone)]
#[diesel(table_name = refresh_tokens)]
pub struct NewRefreshToken {
    pub user_id: String,
    pub token_hash: String,
    pub family_id: Uuid,
    pub expires_at: DateTime<Utc>,
    pub device_info: Option<serde_json::Value>,
    pub ip_address: Option<DieselIpAddr>,
    pub user_agent: Option<String>,
}

/// Diesel model for updating refresh tokens
#[derive(AsChangeset, Serialize, Deserialize, Debug, Clone)]
#[diesel(table_name = refresh_tokens)]
pub struct UpdateRefreshToken {
    pub updated_at: Option<DateTime<Utc>>,
    pub last_used_at: Option<DateTime<Utc>>,
    pub is_revoked: Option<bool>,
    pub revoked_at: Option<DateTime<Utc>>,
    pub revoked_reason: Option<String>,
}

impl NewRefreshToken {
    pub fn new(
        user_id: String,
        token_hash: String,
        family_id: Uuid,
        expires_at: DateTime<Utc>,
        device_info: Option<serde_json::Value>,
        ip_address: Option<IpAddr>,
        user_agent: Option<String>,
    ) -> Self {
        Self {
            user_id,
            token_hash,
            family_id,
            expires_at,
            device_info,
            ip_address: ip_address.map(|ip| DieselIpAddr(ip)),
            user_agent,
        }
    }
}

impl UpdateRefreshToken {
    pub fn mark_used(last_used_at: DateTime<Utc>) -> Self {
        Self {
            updated_at: Some(Utc::now()),
            last_used_at: Some(last_used_at),
            is_revoked: None,
            revoked_at: None,
            revoked_reason: None,
        }
    }

    pub fn mark_revoked(reason: String) -> Self {
        Self {
            updated_at: Some(Utc::now()),
            last_used_at: None,
            is_revoked: Some(true),
            revoked_at: Some(Utc::now()),
            revoked_reason: Some(reason),
        }
    }
}