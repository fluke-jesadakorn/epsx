use diesel::prelude::*;
use chrono::{DateTime, Utc};
use uuid::Uuid;

use serde::{Deserialize, Serialize};



use crate::infra::db::diesel::schema::revoked_tokens;


/// Diesel model for revoked_tokens table (JTI blacklist)
#[derive(Queryable, Selectable, Identifiable, Serialize, Deserialize, Debug, Clone)]
#[diesel(table_name = revoked_tokens)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct RevokedToken {
    pub id: Uuid,
    pub jti: String,
    pub user_id: String,
    pub token_type: String,
    pub expires_at: DateTime<Utc>,
    pub revoked_at: DateTime<Utc>,
    pub revoked_by: Option<String>,
    pub revoked_reason: String,
    pub created_at: DateTime<Utc>,
}

/// Diesel model for inserting new revoked tokens
#[derive(Insertable, Serialize, Deserialize, Debug, Clone)]
#[diesel(table_name = revoked_tokens)]
pub struct NewRevokedToken {
    pub jti: String,
    pub user_id: String,
    pub token_type: String,
    pub expires_at: DateTime<Utc>,
    pub revoked_by: Option<String>,
    pub revoked_reason: String,
}

/// Token types for JTI blacklist
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TokenType {
    AccessToken,
    RefreshToken,
    IdToken,
}

impl ToString for TokenType {
    fn to_string(&self) -> String {
        match self {
            TokenType::AccessToken => "access_token".to_string(),
            TokenType::RefreshToken => "refresh_token".to_string(),
            TokenType::IdToken => "id_token".to_string(),
        }
    }
}

impl From<&str> for TokenType {
    fn from(value: &str) -> Self {
        match value {
            "access_token" => TokenType::AccessToken,
            "refresh_token" => TokenType::RefreshToken,
            "id_token" => TokenType::IdToken,
            _ => TokenType::AccessToken, // Default
        }
    }
}

impl NewRevokedToken {
    pub fn new(
        jti: String,
        user_id: String,
        token_type: TokenType,
        expires_at: DateTime<Utc>,
        revoked_by: Option<String>,
        revoked_reason: String,
    ) -> Self {
        Self {
            jti,
            user_id,
            token_type: token_type.to_string(),
            expires_at,
            revoked_by,
            revoked_reason,
        }
    }

    pub fn revoke_access_token(
        jti: String,
        user_id: String,
        expires_at: DateTime<Utc>,
        revoked_by: Option<String>,
        reason: String,
    ) -> Self {
        Self::new(jti, user_id, TokenType::AccessToken, expires_at, revoked_by, reason)
    }

    pub fn revoke_refresh_token(
        jti: String,
        user_id: String,
        expires_at: DateTime<Utc>,
        revoked_by: Option<String>,
        reason: String,
    ) -> Self {
        Self::new(jti, user_id, TokenType::RefreshToken, expires_at, revoked_by, reason)
    }
}