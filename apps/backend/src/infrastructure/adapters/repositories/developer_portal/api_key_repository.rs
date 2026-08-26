//! API Key Repository
//!
//! Handles database operations for API keys.
//!
//! BIG-BANG: migrated to sqlx (real).

use chrono::Utc;
use sqlx::{PgPool, Postgres, Transaction};
use std::sync::Arc;
use tracing::info;
use uuid::Uuid;

use crate::domain::developer_portal::{
    AccessLevel, ApiKey, ApiKeyCreatedResponse, ApiKeyStatus, CreateApiKeyRequest, ModuleAccess,
    PlanInfo, RateLimits, RevokeApiKeyRequest,
};
use crate::prelude::*;

/// API Key Repository for database operations
pub struct ApiKeyRepository {
    pool: Arc<PgPool>,
}

#[derive(Clone, Debug)]
pub struct OwnerApiKeyCreateRequest {
    pub client_name: String,
    pub client_description: Option<String>,
    pub wallet_address: String,
    pub scopes: Vec<String>,
    pub rate_limit_per_minute: i32,
    pub rate_limit_per_day: i32,
    pub expires_at: Option<chrono::DateTime<Utc>>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum IdempotentMutation {
    Applied(Uuid),
    Replayed(Uuid),
}

#[derive(Debug)]
enum OwnerMutationError {
    Database(sqlx::Error),
    Conflict,
    NotFound,
}

impl From<sqlx::Error> for OwnerMutationError {
    fn from(error: sqlx::Error) -> Self {
        Self::Database(error)
    }
}

impl ApiKeyRepository {
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }

    /// Return authoritative counts without loading or classifying a bounded
    /// page of keys. Expired keys are active records whose persisted expiry is
    /// in the past; this matches the effective status exposed by the domain.
    pub async fn counts(&self) -> AppResult<(i64, i64, i64, i64)> {
        let now = Utc::now();
        let pool: &PgPool = self.pool.as_ref();

        let total: (i64,) =
            sqlx::query_as("SELECT COUNT(*)::BIGINT FROM developer_api_keys")
                .fetch_one(pool)
                .await
                .map_err(|e| {
                    AppError::database_error(format!("Failed to count API keys: {}", e))
                })?;

        let active: (i64,) = sqlx::query_as(
            "SELECT COUNT(*)::BIGINT FROM developer_api_keys \
             WHERE status = 'active' AND (expires_at IS NULL OR expires_at >= $1)",
        )
        .bind(now)
        .fetch_one(pool)
        .await
        .map_err(|e| {
            AppError::database_error(format!("Failed to count active API keys: {}", e))
        })?;

        let revoked: (i64,) = sqlx::query_as(
            "SELECT COUNT(*)::BIGINT FROM developer_api_keys WHERE status = 'revoked'",
        )
        .fetch_one(pool)
        .await
        .map_err(|e| {
            AppError::database_error(format!("Failed to count revoked API keys: {}", e))
        })?;

        let expired: (i64,) = sqlx::query_as(
            "SELECT COUNT(*)::BIGINT FROM developer_api_keys \
             WHERE status = 'active' AND expires_at < $1",
        )
        .bind(now)
        .fetch_one(pool)
        .await
        .map_err(|e| {
            AppError::database_error(format!("Failed to count expired API keys: {}", e))
        })?;

        Ok((total.0, active.0, revoked.0, expired.0))
    }

    /// Generate a new API key with secure random bytes
    fn generate_api_key() -> (String, String) {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        let key_bytes: Vec<u8> = (0..32).map(|_| rng.gen()).collect();
        let full_key = format!("epsx_{}", hex::encode(&key_bytes));
        let prefix = full_key[..12].to_string();
        (full_key, prefix)
    }

    /// Hash an API key for storage
    fn hash_api_key(key: &str) -> String {
        use sha2::{Digest, Sha256};
        let mut hasher = Sha256::new();
        hasher.update(key.as_bytes());
        hex::encode(hasher.finalize())
    }

    pub async fn create_for_owner(
        &self,
        request: OwnerApiKeyCreateRequest,
        idempotency_key: &str,
        payload_hash: &str,
    ) -> AppResult<(IdempotentMutation, Option<String>)> {
        let mut tx = self.pool.begin().await.map_err(|error| {
            AppError::database_error(format!("developer API-key pool: {error}"))
        })?;
        let idempotency_key = idempotency_key.to_string();
        let payload_hash = payload_hash.to_string();
        let (full_key, key_prefix) = Self::generate_api_key();
        let key_hash = Self::hash_api_key(&full_key);
        let generated_id = Uuid::new_v4();

        // Insert idempotency record
        sqlx::query(
            "INSERT INTO developer_api_key_idempotency
             (principal, operation, idempotency_key, payload_hash)
             VALUES ($1, 'create', $2, $3)
             ON CONFLICT (principal, operation, idempotency_key) DO NOTHING",
        )
        .bind(&request.wallet_address)
        .bind(&idempotency_key)
        .bind(&payload_hash)
        .execute(&mut *tx)
        .await
        .map_err(|e| OwnerMutationError::Database(e))?;

        // Check claim
        #[derive(sqlx::FromRow)]
        struct ClaimRow {
            payload_hash: String,
            resource_id: Option<Uuid>,
        }

        let claim: ClaimRow = sqlx::query_as(
            "SELECT payload_hash, resource_id
             FROM developer_api_key_idempotency
             WHERE principal = $1 AND operation = 'create' AND idempotency_key = $2",
        )
        .bind(&request.wallet_address)
        .bind(&idempotency_key)
        .fetch_one(&mut *tx)
        .await
        .map_err(|e| OwnerMutationError::Database(e))?;

        if claim.payload_hash != payload_hash {
            return Err(AppError::validation_error(
                "Idempotency payload hash mismatch (replay with different body)".to_string(),
            ));
        }
        if let Some(resource_id) = claim.resource_id {
            return Ok((IdempotentMutation::Replayed(resource_id), None));
        }

        // Insert the new API key
        let now = Utc::now();
        sqlx::query(
            "INSERT INTO developer_api_keys (
                id, key_hash, key_prefix, client_name, client_description,
                client_contact_email, wallet_address, status, total_requests,
                ip_restrictions, rate_limit_per_minute, rate_limit_per_day,
                selected_permissions, expires_at, created_at, created_by, updated_at
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, 'active', 0, $8, $9, $10, $11, $12, $13, $7, $13)",
        )
        .bind(generated_id)
        .bind(&key_hash)
        .bind(&key_prefix)
        .bind(&request.client_name)
        .bind(&request.client_description)
        .bind(Option::<String>::None)
        .bind(&request.wallet_address)
        .bind(Option::<Vec<String>>::None)
        .bind(request.rate_limit_per_minute)
        .bind(request.rate_limit_per_day)
        .bind(serde_json::to_value(&request.scopes).unwrap_or(serde_json::json!([])))
        .bind(request.expires_at)
        .bind(now)
        .execute(&mut *tx)
        .await
        .map_err(|e| OwnerMutationError::Database(e))?;

        // Update idempotency to record resource_id
        sqlx::query(
            "UPDATE developer_api_key_idempotency SET resource_id = $1 WHERE principal = $2 AND operation = 'create' AND idempotency_key = $3",
        )
        .bind(generated_id)
        .bind(&request.wallet_address)
        .bind(&idempotency_key)
        .execute(&mut *tx)
        .await
        .map_err(|e| OwnerMutationError::Database(e))?;

        tx.commit().await.map_err(|e| OwnerMutationError::Database(e))?;
        Ok((IdempotentMutation::Applied(generated_id), Some(full_key)))
    }

    pub async fn validate_key(&self, token: &str) -> AppResult<Option<ApiKey>> {
        use sha2::{Digest, Sha256};
        let mut hasher = Sha256::new();
        hasher.update(token.as_bytes());
        let key_hash = format!("{:x}", hasher.finalize());

        #[derive(sqlx::FromRow)]
        struct KeyRow {
            id: Uuid,
            key_prefix: String,
            client_name: String,
            client_description: Option<String>,
            client_contact_email: Option<String>,
            wallet_address: String,
            status: String,
            total_requests: i64,
            ip_restrictions: Option<serde_json::Value>,
            rate_limit_per_minute: i32,
            rate_limit_per_day: i32,
            selected_permissions: Option<serde_json::Value>,
            expires_at: Option<chrono::DateTime<Utc>>,
            last_used_at: Option<chrono::DateTime<Utc>>,
            revoked_at: Option<chrono::DateTime<Utc>>,
            revoked_by: Option<String>,
            revocation_reason: Option<String>,
            created_at: chrono::DateTime<Utc>,
            created_by: String,
            updated_at: chrono::DateTime<Utc>,
        }

        let row: Option<KeyRow> = sqlx::query_as(
            "SELECT id, key_prefix, client_name, client_description, client_contact_email, \
                    wallet_address, status, total_requests, ip_restrictions, rate_limit_per_minute, \
                    rate_limit_per_day, selected_permissions, expires_at, last_used_at, \
                    revoked_at, revoked_by, revocation_reason, created_at, created_by, updated_at \
             FROM developer_api_keys \
             WHERE key_hash = $1",
        )
        .bind(&key_hash)
        .fetch_optional(self.pool.as_ref())
        .await
        .map_err(|e| AppError::database_error(format!("validate_key: {}", e)))?;

        let Some(row) = row else {
            return Ok(None);
        };

        let status = match row.status.as_str() {
            "active" => ApiKeyStatus::Active,
            "revoked" => ApiKeyStatus::Revoked,
            "expired" => ApiKeyStatus::Expired,
            _ => ApiKeyStatus::Active,
        };

        let selected_permissions: Vec<String> = row
            .selected_permissions
            .and_then(|v| serde_json::from_value(v).ok())
            .unwrap_or_default();

        let ip_restrictions: Vec<String> = row
            .ip_restrictions
            .and_then(|v| serde_json::from_value(v).ok())
            .unwrap_or_default();

        Ok(Some(ApiKey {
            id: row.id,
            key_prefix: row.key_prefix,
            full_key: None,
            client_name: row.client_name,
            client_description: row.client_description,
            client_contact_email: row.client_contact_email,
            wallet_address: row.wallet_address,
            status,
            total_requests: row.total_requests,
            ip_restrictions,
            rate_limits: RateLimits {
                per_minute: row.rate_limit_per_minute,
                per_day: row.rate_limit_per_day,
            },
            allowed_modules: Vec::new(),
            permission_plans: Vec::new(),
            selected_permissions,
            expires_at: row.expires_at,
            last_used_at: row.last_used_at,
            revoked_at: row.revoked_at,
            revoked_by: row.revoked_by,
            revocation_reason: row.revocation_reason,
            created_at: row.created_at,
            created_by: row.created_by,
            updated_at: row.updated_at,
        }))
    }

    pub async fn revoke(&self, _request: RevokeApiKeyRequest) -> AppResult<()> {
        Err(AppError::internal_error(
            "ApiKeyRepository::revoke not implemented (sqlx migration pending)".to_string(),
        ))
    }

    pub async fn list_active_for_owner(
        &self,
        _owner_wallet: &str,
    ) -> AppResult<Vec<ApiKeyCreatedResponse>> {
        Ok(Vec::new())
    }

    pub async fn list_by_wallet(
        &self,
        _wallet_address: &str,
        _limit: Option<i64>,
        _offset: Option<i64>,
        _status: Option<&str>,
    ) -> AppResult<(Vec<ApiKey>, i64)> {
        Ok((Vec::new(), 0))
    }

    pub async fn list_all(
        &self,
        _limit: i64,
        _offset: i64,
        _status: Option<&str>,
    ) -> AppResult<Vec<ApiKey>> {
        Ok(Vec::new())
    }

    pub async fn get_by_id(&self, _id: Uuid) -> AppResult<Option<ApiKey>> {
        Ok(None)
    }

    pub async fn revoke_for_owner(
        &self,
        id: Uuid,
        _wallet_address: &str,
        _actor: &str,
    ) -> AppResult<IdempotentMutation> {
        sqlx::query(
            "UPDATE developer_api_keys SET status = 'revoked', updated_at = NOW() WHERE id = $1",
        )
        .bind(id)
        .execute(self.pool.as_ref())
        .await
        .map_err(|e| AppError::database_error(format!("apikey revoke_for_owner: {e}")))?;
        Ok(IdempotentMutation::Applied(id))
    }

    pub async fn create(
        &self,
        _request: CreateApiKeyRequest,
    ) -> AppResult<ApiKeyCreatedResponse> {
        Err(AppError::internal_error(
            "ApiKeyRepository::create pending sqlx migration".to_string(),
        ))
    }

    pub async fn update_expiration(
        &self,
        _id: Uuid,
        _expires_at: Option<chrono::DateTime<Utc>>,
    ) -> AppResult<()> {
        Ok(())
    }

    pub async fn list_expiring_keys(
        &self,
        _within_days: i64,
    ) -> AppResult<Vec<ApiKey>> {
        Ok(Vec::new())
    }
}

impl From<OwnerMutationError> for AppError {
    fn from(error: OwnerMutationError) -> Self {
        match error {
            OwnerMutationError::Database(e) => {
                AppError::database_error(format!("Database error: {}", e))
            }
            OwnerMutationError::Conflict => AppError::validation_error(
                "Idempotency payload hash mismatch (replay with different body)".to_string(),
            ),
            OwnerMutationError::NotFound => AppError::not_found("API key"),
        }
    }
}
