//! API Key Repository
//!
//! Handles database operations for API keys.
//!
//! BIG-BANG: migrated to sqlx (real).

use chrono::Utc;
use sqlx::PgPool;
use std::sync::Arc;
use uuid::Uuid;

use crate::domain::developer_portal::{
    ApiKey, ApiKeyCreatedResponse, ApiKeyStatus, CreateApiKeyRequest, RateLimits,
    RevokeApiKeyRequest,
};
use crate::prelude::*;

// Exclude hashes and plaintext secrets before materializing a read response.
const KEY_VIEW: &str = "SELECT (to_jsonb(k)-'key_hash'-'full_key') || jsonb_build_object(
  'full_key',NULL,'ip_restrictions',COALESCE(to_jsonb(k.ip_restrictions),'[]'::jsonb),
  'rate_limits',jsonb_build_object('per_minute',k.rate_limit_per_minute,'per_day',k.rate_limit_per_day),
  'allowed_modules',COALESCE((SELECT jsonb_agg(jsonb_build_object('module_id',m.id,'module_name',m.name,'access_level',a.access_level,'custom_rate_limit',a.custom_rate_limit,'custom_quotas',a.custom_quotas) ORDER BY m.id) FROM api_key_module_access a JOIN api_modules m ON m.id=a.module_id WHERE a.api_key_id=k.id),'[]'::jsonb),
  'permission_plans',COALESCE((SELECT jsonb_agg(jsonb_build_object('id',p.id,'name',p.name,'slug',p.slug,'description',p.description,'plan_type',p.plan_type) ORDER BY p.id) FROM api_key_permissions a JOIN plans p ON p.id=a.plan_id WHERE a.api_key_id=k.id AND a.is_active AND (a.expires_at IS NULL OR a.expires_at>NOW())),'[]'::jsonb)) FROM api_keys k";

fn decode_key(value: serde_json::Value) -> AppResult<ApiKey> {
    serde_json::from_value(value)
        .map_err(|_| AppError::database_error("API key projection is malformed"))
}

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
#[allow(dead_code)]
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

        let total: (i64,) = sqlx::query_as("SELECT COUNT(*)::BIGINT FROM api_keys")
            .fetch_one(pool)
            .await
            .map_err(|e| AppError::database_error(format!("Failed to count API keys: {}", e)))?;

        let active: (i64,) = sqlx::query_as(
            "SELECT COUNT(*)::BIGINT FROM api_keys \
             WHERE status = 'active' AND (expires_at IS NULL OR expires_at >= $1)",
        )
        .bind(now)
        .fetch_one(pool)
        .await
        .map_err(|e| AppError::database_error(format!("Failed to count active API keys: {}", e)))?;

        let revoked: (i64,) =
            sqlx::query_as("SELECT COUNT(*)::BIGINT FROM api_keys WHERE status = 'revoked'")
                .fetch_one(pool)
                .await
                .map_err(|e| {
                    AppError::database_error(format!("Failed to count revoked API keys: {}", e))
                })?;

        let expired: (i64,) = sqlx::query_as(
            "SELECT COUNT(*)::BIGINT FROM api_keys \
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
        .map_err(OwnerMutationError::Database)?;

        // Check claim
        #[derive(sqlx::FromRow)]
        struct ClaimRow {
            payload_hash: String,
            resource_id: Option<Uuid>,
        }

        let claim: ClaimRow = sqlx::query_as(
            "SELECT payload_hash, resource_id
             FROM developer_api_key_idempotency
             WHERE principal = $1 AND operation = 'create' AND idempotency_key = $2 FOR UPDATE",
        )
        .bind(&request.wallet_address)
        .bind(&idempotency_key)
        .fetch_one(&mut *tx)
        .await
        .map_err(OwnerMutationError::Database)?;

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
            "INSERT INTO api_keys (
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
        .bind(&request.scopes)
        .bind(request.expires_at)
        .bind(now)
        .execute(&mut *tx)
        .await
        .map_err(OwnerMutationError::Database)?;

        // Update idempotency to record resource_id
        sqlx::query(
            "UPDATE developer_api_key_idempotency SET resource_id = $1, completed_at=NOW() WHERE principal = $2 AND operation = 'create' AND idempotency_key = $3",
        )
        .bind(generated_id)
        .bind(&request.wallet_address)
        .bind(&idempotency_key)
        .execute(&mut *tx)
        .await
        .map_err(OwnerMutationError::Database)?;

        sqlx::query("INSERT INTO developer_api_key_audit(actor,action,api_key_id,idempotency_key) VALUES($1,'created',$2,$3)")
            .bind(&request.wallet_address).bind(generated_id).bind(&idempotency_key)
            .execute(&mut *tx).await.map_err(OwnerMutationError::Database)?;
        tx.commit().await.map_err(OwnerMutationError::Database)?;
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
            ip_restrictions: Option<Vec<String>>,
            rate_limit_per_minute: i32,
            rate_limit_per_day: i32,
            selected_permissions: Vec<String>,
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
             FROM api_keys \
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
            _ => return Err(AppError::database_error("Unknown persisted API key status")),
        };

        let selected_permissions = row.selected_permissions;
        let ip_restrictions = row.ip_restrictions.unwrap_or_default();

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

    pub async fn revoke(&self, id: Uuid, request: RevokeApiKeyRequest) -> AppResult<()> {
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(OwnerMutationError::Database)?;
        let status: Option<String> =
            sqlx::query_scalar("SELECT status FROM api_keys WHERE id=$1 FOR UPDATE")
                .bind(id)
                .fetch_optional(&mut *tx)
                .await
                .map_err(OwnerMutationError::Database)?;
        let status = status.ok_or_else(|| AppError::not_found("API key"))?;
        if status != "revoked" {
            sqlx::query("UPDATE api_keys SET status='revoked',revoked_at=NOW(),revoked_by=$2,revocation_reason=$3,updated_at=NOW() WHERE id=$1")
                .bind(id).bind(&request.revoked_by).bind(&request.reason).execute(&mut *tx).await.map_err(OwnerMutationError::Database)?;
            sqlx::query("INSERT INTO developer_api_key_audit(actor,action,api_key_id,idempotency_key,metadata) VALUES($1,'revoked',$2,$3,jsonb_build_object('reason',$4::text))")
                .bind(&request.revoked_by).bind(id).bind(format!("admin-revoke-{id}")).bind(&request.reason)
                .execute(&mut *tx).await.map_err(OwnerMutationError::Database)?;
        }
        tx.commit().await.map_err(OwnerMutationError::Database)?;
        Ok(())
    }

    pub async fn list_by_wallet(
        &self,
        wallet_address: &str,
        limit: Option<i64>,
        offset: Option<i64>,
        status: Option<&str>,
    ) -> AppResult<(Vec<ApiKey>, i64)> {
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(OwnerMutationError::Database)?;
        sqlx::query("SET TRANSACTION ISOLATION LEVEL REPEATABLE READ READ ONLY")
            .execute(&mut *tx)
            .await
            .map_err(OwnerMutationError::Database)?;
        let total: i64 = sqlx::query_scalar("SELECT count(*) FROM api_keys WHERE wallet_address=$1 AND ($2::text IS NULL OR CASE WHEN status='active' AND expires_at<=NOW() THEN 'expired' ELSE status END=$2)")
            .bind(wallet_address.to_ascii_lowercase()).bind(status).fetch_one(&mut *tx).await.map_err(OwnerMutationError::Database)?;
        let sql = format!("{KEY_VIEW} WHERE k.wallet_address=$1 AND ($2::text IS NULL OR CASE WHEN k.status='active' AND k.expires_at<=NOW() THEN 'expired' ELSE k.status END=$2) ORDER BY k.created_at DESC,k.id LIMIT $3 OFFSET $4");
        let values: Vec<serde_json::Value> = sqlx::query_scalar(&sql)
            .bind(wallet_address.to_ascii_lowercase())
            .bind(status)
            .bind(limit.unwrap_or(20).clamp(1, 1000))
            .bind(offset.unwrap_or(0).max(0))
            .fetch_all(&mut *tx)
            .await
            .map_err(OwnerMutationError::Database)?;
        let keys = values
            .into_iter()
            .map(decode_key)
            .collect::<AppResult<Vec<_>>>()?;
        tx.commit().await.map_err(OwnerMutationError::Database)?;
        Ok((keys, total))
    }

    pub async fn list_all(
        &self,
        limit: i64,
        offset: i64,
        status: Option<&str>,
    ) -> AppResult<Vec<ApiKey>> {
        let sql = format!("{KEY_VIEW} WHERE $1::text IS NULL OR CASE WHEN k.status='active' AND k.expires_at<=NOW() THEN 'expired' ELSE k.status END=$1 ORDER BY k.created_at DESC,k.id LIMIT $2 OFFSET $3");
        let values: Vec<serde_json::Value> = sqlx::query_scalar(&sql)
            .bind(status)
            .bind(limit.clamp(1, 1000))
            .bind(offset.max(0))
            .fetch_all(self.pool.as_ref())
            .await
            .map_err(OwnerMutationError::Database)?;
        values.into_iter().map(decode_key).collect()
    }

    pub async fn get_by_id(&self, id: Uuid) -> AppResult<Option<ApiKey>> {
        let value: Option<serde_json::Value> =
            sqlx::query_scalar(&format!("{KEY_VIEW} WHERE k.id=$1"))
                .bind(id)
                .fetch_optional(self.pool.as_ref())
                .await
                .map_err(OwnerMutationError::Database)?;
        value.map(decode_key).transpose()
    }

    pub async fn revoke_for_owner(
        &self,
        id: Uuid,
        wallet_address: &str,
        reason: &str,
        idempotency_key: &str,
        payload_hash: &str,
    ) -> AppResult<IdempotentMutation> {
        let owner = wallet_address.to_ascii_lowercase();
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(OwnerMutationError::Database)?;
        let status: Option<String> = sqlx::query_scalar(
            "SELECT status FROM api_keys WHERE id=$1 AND wallet_address=$2 FOR UPDATE",
        )
        .bind(id)
        .bind(&owner)
        .fetch_optional(&mut *tx)
        .await
        .map_err(OwnerMutationError::Database)?;
        let status = status.ok_or_else(|| AppError::not_found("API key"))?;
        sqlx::query("INSERT INTO developer_api_key_idempotency(principal,operation,idempotency_key,payload_hash) VALUES($1,'revoke',$2,$3) ON CONFLICT DO NOTHING")
            .bind(&owner).bind(idempotency_key).bind(payload_hash).execute(&mut *tx).await.map_err(OwnerMutationError::Database)?;
        let (claimed_hash, resource): (String,Option<Uuid>) = sqlx::query_as("SELECT payload_hash,resource_id FROM developer_api_key_idempotency WHERE principal=$1 AND operation='revoke' AND idempotency_key=$2 FOR UPDATE")
            .bind(&owner).bind(idempotency_key).fetch_one(&mut *tx).await.map_err(OwnerMutationError::Database)?;
        if claimed_hash != payload_hash {
            return Err(OwnerMutationError::Conflict.into());
        }
        if resource.is_some() {
            return Ok(IdempotentMutation::Replayed(id));
        }
        if status != "revoked" {
            sqlx::query("UPDATE api_keys SET status='revoked',revoked_at=NOW(),revoked_by=$2,revocation_reason=$3,updated_at=NOW() WHERE id=$1 AND wallet_address=$2")
                .bind(id).bind(&owner).bind(reason).execute(&mut *tx).await.map_err(OwnerMutationError::Database)?;
            sqlx::query("INSERT INTO developer_api_key_audit(actor,action,api_key_id,idempotency_key,metadata) VALUES($1,'revoked',$2,$3,jsonb_build_object('reason',$4::text))")
                .bind(&owner).bind(id).bind(idempotency_key).bind(reason).execute(&mut *tx).await.map_err(OwnerMutationError::Database)?;
        }
        sqlx::query("UPDATE developer_api_key_idempotency SET resource_id=$3,completed_at=NOW() WHERE principal=$1 AND operation='revoke' AND idempotency_key=$2")
            .bind(&owner).bind(idempotency_key).bind(id).execute(&mut *tx).await.map_err(OwnerMutationError::Database)?;
        tx.commit().await.map_err(OwnerMutationError::Database)?;
        Ok(if status == "revoked" {
            IdempotentMutation::Replayed(id)
        } else {
            IdempotentMutation::Applied(id)
        })
    }

    pub async fn create(
        &self,
        request: CreateApiKeyRequest,
        idempotency_key: &str,
    ) -> AppResult<ApiKeyCreatedResponse> {
        use sha2::{Digest, Sha256};
        let payload_hash = hex::encode(Sha256::digest(
            serde_json::to_vec(&request)
                .map_err(|_| AppError::bad_request("Invalid API key request"))?,
        ));
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(OwnerMutationError::Database)?;
        sqlx::query("INSERT INTO developer_api_key_idempotency(principal,operation,idempotency_key,payload_hash) VALUES($1,'create',$2,$3) ON CONFLICT DO NOTHING")
            .bind(&request.created_by).bind(idempotency_key).bind(&payload_hash).execute(&mut *tx).await.map_err(OwnerMutationError::Database)?;
        let (claim,resource): (String,Option<Uuid>) = sqlx::query_as("SELECT payload_hash,resource_id FROM developer_api_key_idempotency WHERE principal=$1 AND operation='create' AND idempotency_key=$2 FOR UPDATE")
            .bind(&request.created_by).bind(idempotency_key).fetch_one(&mut *tx).await.map_err(OwnerMutationError::Database)?;
        if claim != payload_hash {
            return Err(OwnerMutationError::Conflict.into());
        }
        if resource.is_some() {
            return Err(AppError::conflict(
                "API key already created; plaintext secrets cannot be replayed",
            ));
        }
        let id = Uuid::new_v4();
        let (secret, prefix) = Self::generate_api_key();
        sqlx::query("INSERT INTO api_keys(id,key_hash,key_prefix,client_name,client_description,client_contact_email,wallet_address,ip_restrictions,rate_limit_per_minute,rate_limit_per_day,selected_permissions,expires_at,created_by) VALUES($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13)")
            .bind(id).bind(Self::hash_api_key(&secret)).bind(prefix).bind(&request.client_name).bind(&request.client_description).bind(&request.client_contact_email)
            .bind(&request.wallet_address).bind(&request.ip_restrictions).bind(request.rate_limit_per_minute.unwrap_or(60)).bind(request.rate_limit_per_day.unwrap_or(10000))
            .bind(&request.permissions).bind(request.expires_at).bind(&request.created_by).execute(&mut *tx).await.map_err(OwnerMutationError::Database)?;
        for module in &request.allowed_modules {
            sqlx::query("INSERT INTO api_key_module_access(api_key_id,module_id,access_level,custom_quotas,granted_by) VALUES($1,$2,$3,COALESCE($4,'{}'::jsonb),$5)")
                .bind(id).bind(module.module_id).bind(&module.access_level).bind(&module.custom_quotas).bind(&request.created_by)
                .execute(&mut *tx).await.map_err(OwnerMutationError::Database)?;
        }
        for plan in &request.plan_ids {
            sqlx::query(
                "INSERT INTO api_key_permissions(api_key_id,plan_id,granted_by) VALUES($1,$2,$3)",
            )
            .bind(id)
            .bind(plan)
            .bind(&request.created_by)
            .execute(&mut *tx)
            .await
            .map_err(OwnerMutationError::Database)?;
        }
        sqlx::query("INSERT INTO developer_api_key_audit(actor,action,api_key_id,idempotency_key) VALUES($1,'created',$2,$3)")
            .bind(&request.created_by).bind(id).bind(idempotency_key).execute(&mut *tx).await.map_err(OwnerMutationError::Database)?;
        sqlx::query("UPDATE developer_api_key_idempotency SET resource_id=$3,completed_at=NOW() WHERE principal=$1 AND operation='create' AND idempotency_key=$2")
            .bind(&request.created_by).bind(idempotency_key).bind(id).execute(&mut *tx).await.map_err(OwnerMutationError::Database)?;
        let value: serde_json::Value = sqlx::query_scalar(&format!("{KEY_VIEW} WHERE k.id=$1"))
            .bind(id)
            .fetch_one(&mut *tx)
            .await
            .map_err(OwnerMutationError::Database)?;
        let api_key = decode_key(value)?;
        tx.commit().await.map_err(OwnerMutationError::Database)?;
        Ok(ApiKeyCreatedResponse {
            api_key,
            full_key: secret,
        })
    }

    pub async fn update_expiration(
        &self,
        id: Uuid,
        expires_at: Option<chrono::DateTime<Utc>>,
    ) -> AppResult<()> {
        let rows = sqlx::query("UPDATE api_keys SET expires_at=$2,updated_at=NOW() WHERE id=$1")
            .bind(id)
            .bind(expires_at)
            .execute(self.pool.as_ref())
            .await
            .map_err(OwnerMutationError::Database)?
            .rows_affected();
        if rows == 0 {
            return Err(AppError::not_found("API key"));
        }
        Ok(())
    }

    pub async fn list_expiring_keys(&self, within_days: i64) -> AppResult<Vec<ApiKey>> {
        let sql = format!("{KEY_VIEW} WHERE k.status='active' AND k.expires_at>NOW() AND k.expires_at<=NOW()+make_interval(days=>$1) ORDER BY k.expires_at,k.id LIMIT 1000");
        let values: Vec<serde_json::Value> = sqlx::query_scalar(&sql)
            .bind(within_days.clamp(1, 3650) as i32)
            .fetch_all(self.pool.as_ref())
            .await
            .map_err(OwnerMutationError::Database)?;
        values.into_iter().map(decode_key).collect()
    }
}

impl From<OwnerMutationError> for AppError {
    fn from(error: OwnerMutationError) -> Self {
        match error {
            OwnerMutationError::Database(e) => {
                AppError::database_error(format!("Database error: {}", e))
            }
            OwnerMutationError::Conflict => AppError::conflict(
                "Idempotency payload hash mismatch (replay with different body)".to_string(),
            ),
            OwnerMutationError::NotFound => AppError::not_found("API key"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    #[ignore = "requires migrated isolated EPSX_MERCHANT_CORE database"]
    async fn restored_api_keys_preserve_owner_boundaries_replays_and_audit() {
        let pool = Arc::new(
            PgPool::connect(&std::env::var("EPSX_MERCHANT_CORE").unwrap())
                .await
                .unwrap(),
        );
        let db: String = sqlx::query_scalar("SELECT current_database()")
            .fetch_one(pool.as_ref())
            .await
            .unwrap();
        assert!(db.starts_with("epsx_merchant_check_"));
        let owner = format!("0x00000000{}", Uuid::new_v4().simple());
        sqlx::query("INSERT INTO wallet_users(wallet_address,is_active) VALUES($1,true)")
            .bind(&owner)
            .execute(pool.as_ref())
            .await
            .unwrap();
        let repo = ApiKeyRepository::new(pool.clone());
        let request = OwnerApiKeyCreateRequest {
            client_name: "Owner boundary rehearsal".into(),
            client_description: None,
            wallet_address: owner.clone(),
            scopes: vec!["epsx:analytics:view".into()],
            rate_limit_per_minute: 5,
            rate_limit_per_day: 100,
            expires_at: None,
        };
        let claim = Uuid::new_v4().to_string();
        let hash = "a".repeat(64);
        let (mutation, secret) = repo
            .create_for_owner(request.clone(), &claim, &hash)
            .await
            .unwrap();
        let IdempotentMutation::Applied(id) = mutation else {
            panic!("first create must apply")
        };
        let read = repo.get_by_id(id).await.unwrap().unwrap();
        assert_eq!(read.selected_permissions, request.scopes);
        assert!(read.full_key.is_none());
        assert_eq!(
            repo.validate_key(secret.as_deref().unwrap())
                .await
                .unwrap()
                .unwrap()
                .id,
            id
        );
        let (rows, total) = repo
            .list_by_wallet(&owner, Some(10), Some(0), Some("active"))
            .await
            .unwrap();
        assert_eq!((rows.len(), total), (1, 1));
        assert!(rows[0].full_key.is_none());
        let (replay, replayed_secret) = repo
            .create_for_owner(request.clone(), &claim, &hash)
            .await
            .unwrap();
        assert_eq!(replay, IdempotentMutation::Replayed(id));
        assert!(replayed_secret.is_none());
        assert!(repo
            .create_for_owner(request.clone(), &claim, &"b".repeat(64))
            .await
            .is_err());
        let revoke_claim = Uuid::new_v4().to_string();
        assert!(repo
            .revoke_for_owner(
                id,
                "0x1111111111111111111111111111111111111111",
                "Foreign owner",
                &revoke_claim,
                &hash
            )
            .await
            .is_err());
        assert_eq!(
            repo.get_by_id(id).await.unwrap().unwrap().status,
            ApiKeyStatus::Active
        );
        assert_eq!(
            repo.revoke_for_owner(id, &owner, "Owner request", &revoke_claim, &hash)
                .await
                .unwrap(),
            IdempotentMutation::Applied(id)
        );
        assert_eq!(
            repo.revoke_for_owner(id, &owner, "Owner request", &revoke_claim, &hash)
                .await
                .unwrap(),
            IdempotentMutation::Replayed(id)
        );
        let revoked = repo.get_by_id(id).await.unwrap().unwrap();
        assert_eq!(revoked.revoked_by.as_deref(), Some(owner.as_str()));
        assert_eq!(revoked.revocation_reason.as_deref(), Some("Owner request"));
        let count: i64 =
            sqlx::query_scalar("SELECT count(*) FROM developer_api_key_audit WHERE api_key_id=$1")
                .bind(id)
                .fetch_one(pool.as_ref())
                .await
                .unwrap();
        assert_eq!(
            count, 2,
            "create and revoke each have one durable audit row"
        );
        assert_eq!(
            repo.list_by_wallet(&owner, Some(10), Some(0), Some("active"))
                .await
                .unwrap()
                .1,
            0
        );

        // Audit failure must undo both the key insert and its idempotency claim.
        sqlx::query("CREATE OR REPLACE FUNCTION rehearsal_reject_key_audit() RETURNS trigger LANGUAGE plpgsql AS $$ BEGIN RAISE EXCEPTION 'rehearsal audit failure'; END $$").execute(pool.as_ref()).await.unwrap();
        sqlx::query("CREATE TRIGGER rehearsal_key_audit BEFORE INSERT ON developer_api_key_audit FOR EACH ROW EXECUTE FUNCTION rehearsal_reject_key_audit()")
            .execute(pool.as_ref()).await.unwrap();
        let failed_claim = Uuid::new_v4().to_string();
        let result = repo.create_for_owner(request, &failed_claim, &hash).await;
        sqlx::query("DROP TRIGGER rehearsal_key_audit ON developer_api_key_audit")
            .execute(pool.as_ref())
            .await
            .unwrap();
        assert!(result.is_err());
        let claims:i64=sqlx::query_scalar("SELECT count(*) FROM developer_api_key_idempotency WHERE principal=$1 AND idempotency_key=$2")
            .bind(&owner).bind(failed_claim).fetch_one(pool.as_ref()).await.unwrap();
        assert_eq!(claims, 0);
        assert_eq!(
            repo.list_by_wallet(&owner, None, None, None)
                .await
                .unwrap()
                .1,
            1
        );
        let module: Uuid = sqlx::query_scalar("INSERT INTO api_modules(name,display_name,category,base_path) VALUES($1,'Rehearsal','analytics','/rehearsal') RETURNING id")
            .bind(format!("rehearsal-{id}")).fetch_one(pool.as_ref()).await.unwrap();
        let plan: Uuid = sqlx::query_scalar("SELECT id FROM plans ORDER BY id LIMIT 1")
            .fetch_one(pool.as_ref())
            .await
            .unwrap();
        let admin_request = CreateApiKeyRequest {
            client_name: "Admin creation rehearsal".into(),
            client_description: Some("Preserved metadata".into()),
            client_contact_email: None,
            wallet_address: owner.clone(),
            allowed_modules: vec![crate::domain::developer_portal::ModuleAccessRequest {
                module_id: module,
                access_level: "bronze".into(),
                custom_quotas: None,
            }],
            plan_ids: vec![plan],
            permissions: vec!["epsx:analytics:view".into()],
            ip_restrictions: None,
            rate_limit_per_minute: Some(5),
            rate_limit_per_day: Some(100),
            expires_at: None,
            created_by: owner.clone(),
        };
        let admin_claim = Uuid::new_v4().to_string();
        let created = repo
            .create(admin_request.clone(), &admin_claim)
            .await
            .unwrap();
        assert_eq!(created.api_key.allowed_modules.len(), 1);
        assert_eq!(created.api_key.permission_plans[0].id, plan);
        assert!(created.api_key.full_key.is_none());
        assert!(repo.create(admin_request, &admin_claim).await.is_err());
        let expiry = Utc::now() + chrono::Duration::days(1);
        repo.update_expiration(created.api_key.id, Some(expiry))
            .await
            .unwrap();
        assert!(repo
            .list_expiring_keys(2)
            .await
            .unwrap()
            .iter()
            .any(|key| key.id == created.api_key.id));
        repo.revoke(
            created.api_key.id,
            RevokeApiKeyRequest {
                reason: "Admin request".into(),
                revoked_by: owner.clone(),
            },
        )
        .await
        .unwrap();
        assert_eq!(
            repo.get_by_id(created.api_key.id)
                .await
                .unwrap()
                .unwrap()
                .status,
            ApiKeyStatus::Revoked
        );
        assert_eq!(
            repo.get_by_id(id)
                .await
                .unwrap()
                .unwrap()
                .revocation_reason
                .as_deref(),
            Some("Owner request"),
            "admin revocation targets only its explicit key ID"
        );
        pool.close().await;
    }
}
