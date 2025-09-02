// Diesel models for OIDC tables
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use uuid::Uuid;

// OIDC Refresh Token model
#[derive(Debug, Clone, Queryable, Identifiable, Serialize, Deserialize)]
#[diesel(table_name = crate::infra::db::diesel::schema::oidc_refresh_tokens)]
pub struct OidcRefreshToken {
    pub id: Uuid,
    pub jti: String,
    pub user_id: String,
    pub token_hash: String,
    pub expires_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub last_used_at: Option<DateTime<Utc>>,
    pub revoked: bool,
    pub revoked_at: Option<DateTime<Utc>>,
    pub revoked_reason: Option<String>,
    pub client_id: Option<String>,
    pub scope: Option<String>,
    pub metadata: Option<serde_json::Value>,
}

// New OIDC Refresh Token for insertion
#[derive(Debug, Clone, Insertable, Serialize, Deserialize)]
#[diesel(table_name = crate::infra::db::diesel::schema::oidc_refresh_tokens)]
pub struct NewOidcRefreshToken {
    pub jti: String,
    pub user_id: String,
    pub token_hash: String,
    pub expires_at: DateTime<Utc>,
    pub client_id: Option<String>,
    pub scope: Option<String>,
    pub metadata: Option<serde_json::Value>,
}

// OIDC Token Audit model
#[derive(Debug, Clone, Queryable, Identifiable, Serialize, Deserialize)]
#[diesel(table_name = crate::infra::db::diesel::schema::oidc_token_audit)]
pub struct OidcTokenAudit {
    pub id: Uuid,
    pub operation: String,
    pub table_name: String,
    pub record_id: Option<Uuid>,
    pub jti: Option<String>,
    pub user_id: Option<String>,
    pub old_values: Option<serde_json::Value>,
    pub new_values: Option<serde_json::Value>,
    pub changed_by: Option<String>,
    pub changed_at: DateTime<Utc>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub session_id: Option<String>,
    pub additional_data: Option<serde_json::Value>,
}

// New OIDC Token Audit for insertion
#[derive(Debug, Clone, Insertable, Serialize, Deserialize)]
#[diesel(table_name = crate::infra::db::diesel::schema::oidc_token_audit)]
pub struct NewOidcTokenAudit {
    pub operation: String,
    pub table_name: String,
    pub record_id: Option<Uuid>,
    pub jti: Option<String>,
    pub user_id: Option<String>,
    pub old_values: Option<serde_json::Value>,
    pub new_values: Option<serde_json::Value>,
    pub changed_by: Option<String>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub session_id: Option<String>,
    pub additional_data: Option<serde_json::Value>,
}

// Update struct for OIDC Refresh Token
#[derive(Debug, Clone, AsChangeset, Serialize, Deserialize)]
#[diesel(table_name = crate::infra::db::diesel::schema::oidc_refresh_tokens)]
pub struct UpdateOidcRefreshToken {
    pub last_used_at: Option<Option<DateTime<Utc>>>,
    pub revoked: Option<bool>,
    pub revoked_at: Option<Option<DateTime<Utc>>>,
    pub revoked_reason: Option<Option<String>>,
}

// Active OIDC Token view model
#[derive(Debug, Clone, Queryable, Serialize, Deserialize)]
pub struct ActiveOidcToken {
    pub id: Uuid,
    pub jti: String,
    pub user_id: String,
    pub expires_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub last_used_at: Option<DateTime<Utc>>,
    pub scope: Option<String>,
    pub metadata: Option<serde_json::Value>,
    pub seconds_until_expiry: Option<f64>,
}

// Token statistics view model  
#[derive(Debug, Clone, Queryable, Serialize, Deserialize)]
pub struct OidcTokenStats {
    pub total_tokens: Option<i64>,
    pub active_tokens: Option<i64>,
    pub revoked_tokens: Option<i64>,
    pub expired_tokens: Option<i64>,
    pub unique_users: Option<i64>,
    pub avg_token_lifetime_seconds: Option<f64>,
    pub oldest_token_created: Option<DateTime<Utc>>,
    pub newest_token_created: Option<DateTime<Utc>>,
    pub operations_last_hour: Option<i64>,
    pub operations_last_day: Option<i64>,
    pub tokens_created_today: Option<i64>,
    pub tokens_deleted_today: Option<i64>,
    pub stats_generated_at: DateTime<Utc>,
}

impl OidcRefreshToken {
    /// Check if token is expired
    pub fn is_expired(&self) -> bool {
        self.expires_at <= Utc::now()
    }

    /// Check if token is active (not revoked and not expired)
    pub fn is_active(&self) -> bool {
        !self.revoked && !self.is_expired()
    }

    /// Get remaining lifetime in seconds
    pub fn remaining_seconds(&self) -> i64 {
        if self.is_expired() {
            0
        } else {
            (self.expires_at - Utc::now()).num_seconds()
        }
    }

    /// Mark token as used
    pub fn mark_as_used(&self) -> UpdateOidcRefreshToken {
        UpdateOidcRefreshToken {
            last_used_at: Some(Some(Utc::now())),
            revoked: None,
            revoked_at: None,
            revoked_reason: None,
        }
    }

    /// Create revocation update
    pub fn create_revocation_update(reason: Option<String>) -> UpdateOidcRefreshToken {
        UpdateOidcRefreshToken {
            last_used_at: None,
            revoked: Some(true),
            revoked_at: Some(Some(Utc::now())),
            revoked_reason: Some(reason),
        }
    }
}

impl NewOidcRefreshToken {
    /// Create new refresh token model from token data
    pub fn new(
        jti: String,
        user_id: String,
        token_hash: String,
        expires_at: DateTime<Utc>,
        scope: Option<String>,
    ) -> Self {
        Self {
            jti,
            user_id,
            token_hash,
            expires_at,
            client_id: None, // Could be set later if needed
            scope,
            metadata: None,
        }
    }

    /// Add metadata to the token
    pub fn with_metadata(mut self, metadata: serde_json::Value) -> Self {
        self.metadata = Some(metadata);
        self
    }

    /// Add client ID to the token
    pub fn with_client_id(mut self, client_id: String) -> Self {
        self.client_id = Some(client_id);
        self
    }
}

impl NewOidcTokenAudit {
    /// Create audit log for token operation
    pub fn new_token_operation(
        operation: &str,
        table_name: &str,
        jti: Option<String>,
        user_id: Option<String>,
        additional_data: Option<serde_json::Value>,
    ) -> Self {
        Self {
            operation: operation.to_string(),
            table_name: table_name.to_string(),
            record_id: None,
            jti,
            user_id,
            old_values: None,
            new_values: None,
            changed_by: Some("system".to_string()),
            ip_address: None,
            user_agent: None,
            session_id: None,
            additional_data,
        }
    }

    /// Add request context to audit log
    pub fn with_context(
        mut self,
        changed_by: Option<String>,
        ip_address: Option<String>,
        user_agent: Option<String>,
        session_id: Option<String>,
    ) -> Self {
        self.changed_by = changed_by;
        self.ip_address = ip_address;
        self.user_agent = user_agent;
        self.session_id = session_id;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Duration;

    #[test]
    fn test_token_expiry_check() {
        let expired_token = OidcRefreshToken {
            id: Uuid::new_v4(),
            jti: "test-jti".to_string(),
            user_id: "test-user".to_string(),
            token_hash: "hash".to_string(),
            expires_at: Utc::now() - Duration::hours(1), // Expired 1 hour ago
            created_at: Utc::now() - Duration::hours(2),
            last_used_at: None,
            revoked: false,
            revoked_at: None,
            revoked_reason: None,
            client_id: None,
            scope: None,
            metadata: None,
        };

        assert!(expired_token.is_expired());
        assert!(!expired_token.is_active());
        assert_eq!(expired_token.remaining_seconds(), 0);
    }

    #[test]
    fn test_active_token() {
        let active_token = OidcRefreshToken {
            id: Uuid::new_v4(),
            jti: "test-jti".to_string(),
            user_id: "test-user".to_string(),
            token_hash: "hash".to_string(),
            expires_at: Utc::now() + Duration::hours(1), // Expires in 1 hour
            created_at: Utc::now() - Duration::minutes(30),
            last_used_at: None,
            revoked: false,
            revoked_at: None,
            revoked_reason: None,
            client_id: None,
            scope: None,
            metadata: None,
        };

        assert!(!active_token.is_expired());
        assert!(active_token.is_active());
        assert!(active_token.remaining_seconds() > 3500); // Should be close to 3600 seconds
    }

    #[test]
    fn test_revoked_token() {
        let revoked_token = OidcRefreshToken {
            id: Uuid::new_v4(),
            jti: "test-jti".to_string(),
            user_id: "test-user".to_string(),
            token_hash: "hash".to_string(),
            expires_at: Utc::now() + Duration::hours(1), // Not expired
            created_at: Utc::now() - Duration::minutes(30),
            last_used_at: None,
            revoked: true, // But revoked
            revoked_at: Some(Utc::now()),
            revoked_reason: Some("User requested".to_string()),
            client_id: None,
            scope: None,
            metadata: None,
        };

        assert!(!revoked_token.is_expired());
        assert!(!revoked_token.is_active()); // Not active because revoked
    }

    #[test]
    fn test_new_token_creation() {
        let new_token = NewOidcRefreshToken::new(
            "test-jti".to_string(),
            "test-user".to_string(),
            "token-hash".to_string(),
            Utc::now() + Duration::days(7),
            Some("openid profile".to_string()),
        );

        assert_eq!(new_token.jti, "test-jti");
        assert_eq!(new_token.user_id, "test-user");
        assert_eq!(new_token.scope, Some("openid profile".to_string()));
    }

    #[test]
    fn test_audit_log_creation() {
        let audit = NewOidcTokenAudit::new_token_operation(
            "VALIDATE",
            "oidc_refresh_tokens",
            Some("test-jti".to_string()),
            Some("test-user".to_string()),
            Some(serde_json::json!({"action": "token_validated", "success": true})),
        );

        assert_eq!(audit.operation, "VALIDATE");
        assert_eq!(audit.table_name, "oidc_refresh_tokens");
        assert_eq!(audit.jti, Some("test-jti".to_string()));
    }
}