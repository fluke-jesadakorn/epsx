//! API Key Repository
//!
//! Handles database operations for API keys.

use chrono::Utc;
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use tracing::info;
use uuid::Uuid;

use crate::domain::developer_portal::{
    AccessLevel, ApiKey, ApiKeyCreatedResponse, ApiKeyStatus, CreateApiKeyRequest, ModuleAccess,
    PlanInfo, RateLimits, RevokeApiKeyRequest,
};
use crate::prelude::*;
use crate::schemas::primary::{api_key_module_access, api_key_permissions, api_keys, api_modules};

/// API Key Repository for database operations
pub struct ApiKeyRepository {
    pool: &'static TlsPool,
}

impl ApiKeyRepository {
    pub fn new(pool: &'static TlsPool) -> Self {
        Self { pool }
    }

    /// Return authoritative counts without loading or classifying a bounded
    /// page of keys. Expired keys are active records whose persisted expiry is
    /// in the past; this matches the effective status exposed by the domain.
    pub async fn counts(&self) -> AppResult<(i64, i64, i64, i64)> {
        let mut conn = self
            .pool
            .get()
            .await
            .map_err(|e| AppError::database_error(format!("Pool error: {}", e)))?;
        let now = Utc::now();
        let total = api_keys::table
            .count()
            .get_result::<i64>(&mut conn)
            .await
            .map_err(|e| AppError::database_error(format!("Failed to count API keys: {}", e)))?;
        let active = api_keys::table
            .filter(api_keys::status.eq("active"))
            .filter(
                api_keys::expires_at
                    .is_null()
                    .or(api_keys::expires_at.ge(now)),
            )
            .count()
            .get_result::<i64>(&mut conn)
            .await
            .map_err(|e| {
                AppError::database_error(format!("Failed to count active API keys: {}", e))
            })?;
        let revoked = api_keys::table
            .filter(api_keys::status.eq("revoked"))
            .count()
            .get_result::<i64>(&mut conn)
            .await
            .map_err(|e| {
                AppError::database_error(format!("Failed to count revoked API keys: {}", e))
            })?;
        let expired = api_keys::table
            .filter(api_keys::status.eq("active"))
            .filter(api_keys::expires_at.lt(now))
            .count()
            .get_result::<i64>(&mut conn)
            .await
            .map_err(|e| {
                AppError::database_error(format!("Failed to count expired API keys: {}", e))
            })?;

        Ok((total, active, revoked, expired))
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

    /// Create a new API key
    pub async fn create(&self, request: CreateApiKeyRequest) -> AppResult<ApiKeyCreatedResponse> {
        let mut conn = self
            .pool
            .get()
            .await
            .map_err(|e| AppError::database_error(format!("Pool error: {}", e)))?;

        let (full_key, prefix) = Self::generate_api_key();
        let key_hash = Self::hash_api_key(&full_key);
        let now = Utc::now();
        let id = Uuid::new_v4();

        // Store only the hash.  The plaintext secret is returned by the
        // create handler once and is never persisted or returned by reads.
        diesel::insert_into(api_keys::table)
            .values((
                api_keys::id.eq(&id),
                api_keys::key_hash.eq(&key_hash),
                api_keys::key_prefix.eq(&prefix),
                api_keys::client_name.eq(&request.client_name),
                api_keys::client_description.eq(&request.client_description),
                api_keys::client_contact_email.eq(&request.client_contact_email),
                api_keys::wallet_address.eq(&request.wallet_address),
                api_keys::status.eq("active"),
                api_keys::total_requests.eq(0_i64),
                api_keys::ip_restrictions.eq(&request.ip_restrictions),
                api_keys::rate_limit_per_minute.eq(request.rate_limit_per_minute.unwrap_or(60)),
                api_keys::rate_limit_per_day.eq(request.rate_limit_per_day.unwrap_or(10000)),
                api_keys::expires_at.eq(&request.expires_at),
                api_keys::created_at.eq(&now),
                api_keys::created_by.eq(&request.created_by),
                api_keys::updated_at.eq(&now),
                api_keys::selected_permissions.eq(&request.permissions),
            ))
            .execute(&mut conn)
            .await
            .map_err(|e| AppError::database_error(format!("Failed to create API key: {}", e)))?;

        // Insert module access entries (legacy, for backwards compatibility)
        for module_access in &request.allowed_modules {
            diesel::insert_into(api_key_module_access::table)
                .values((
                    api_key_module_access::api_key_id.eq(&id),
                    api_key_module_access::module_id.eq(&module_access.module_id),
                    api_key_module_access::access_level.eq(&module_access.access_level),
                    api_key_module_access::custom_quotas.eq(module_access
                        .custom_quotas
                        .clone()
                        .unwrap_or(serde_json::json!({}))),
                    api_key_module_access::granted_at.eq(&now),
                    api_key_module_access::granted_by.eq(&request.created_by),
                ))
                .execute(&mut conn)
                .await
                .map_err(|e| {
                    AppError::database_error(format!("Failed to add module access: {}", e))
                })?;
        }

        // Insert permission plan assignments (new plan-based system)
        for plan_id in &request.plan_ids {
            diesel::insert_into(api_key_permissions::table)
                .values((
                    api_key_permissions::api_key_id.eq(&id),
                    api_key_permissions::plan_id.eq(plan_id),
                    api_key_permissions::granted_at.eq(&now),
                    api_key_permissions::granted_by.eq(&request.created_by),
                    api_key_permissions::is_active.eq(true),
                    api_key_permissions::metadata.eq(serde_json::json!({})),
                ))
                .execute(&mut conn)
                .await
                .map_err(|e| {
                    AppError::database_error(format!("Failed to add permission plan: {}", e))
                })?;
        }

        info!(
            "Created API key {} for wallet {} with {} plans",
            id,
            request.wallet_address,
            request.plan_ids.len()
        );

        // Fetch the created key with modules
        let api_key = self
            .get_by_id(id)
            .await?
            .ok_or_else(|| AppError::not_found("API key not found after creation"))?;

        Ok(ApiKeyCreatedResponse { api_key, full_key })
    }

    /// Get an API key by ID
    pub async fn get_by_id(&self, id: Uuid) -> AppResult<Option<ApiKey>> {
        let mut conn = self
            .pool
            .get()
            .await
            .map_err(|e| AppError::database_error(format!("Pool error: {}", e)))?;

        // Core API key data (16 fields - Diesel's default tuple limit)
        #[derive(Queryable)]
        struct ApiKeyCoreRow {
            id: Uuid,
            key_prefix: String,
            client_name: String,
            client_description: Option<String>,
            client_contact_email: Option<String>,
            wallet_address: String,
            status: String,
            total_requests: i64,
            ip_restrictions: Option<Vec<Option<String>>>,
            rate_limit_per_minute: i32,
            rate_limit_per_day: i32,
            expires_at: Option<chrono::DateTime<Utc>>,
            created_at: chrono::DateTime<Utc>,
            created_by: String,
        }

        // Revocation and timestamp data (separate query)
        #[derive(Queryable)]
        struct ApiKeyMetaRow {
            last_used_at: Option<chrono::DateTime<Utc>>,
            revoked_at: Option<chrono::DateTime<Utc>>,
            revoked_by: Option<String>,
            revocation_reason: Option<String>,
            updated_at: chrono::DateTime<Utc>,
        }

        let core_row: Option<ApiKeyCoreRow> = api_keys::table
            .filter(api_keys::id.eq(&id))
            .select((
                api_keys::id,
                api_keys::key_prefix,
                api_keys::client_name,
                api_keys::client_description,
                api_keys::client_contact_email,
                api_keys::wallet_address,
                api_keys::status,
                api_keys::total_requests,
                api_keys::ip_restrictions,
                api_keys::rate_limit_per_minute,
                api_keys::rate_limit_per_day,
                api_keys::expires_at,
                api_keys::created_at,
                api_keys::created_by,
            ))
            .first(&mut conn)
            .await
            .optional()
            .map_err(|e| AppError::database_error(format!("Failed to fetch API key: {}", e)))?;

        let meta_row: Option<ApiKeyMetaRow> = api_keys::table
            .filter(api_keys::id.eq(&id))
            .select((
                api_keys::last_used_at,
                api_keys::revoked_at,
                api_keys::revoked_by,
                api_keys::revocation_reason,
                api_keys::updated_at,
            ))
            .first(&mut conn)
            .await
            .optional()
            .map_err(|e| {
                AppError::database_error(format!("Failed to fetch API key metadata: {}", e))
            })?;

        if let (Some(core), Some(meta)) = (core_row, meta_row) {
            // Fetch module access (legacy)
            let modules = self.get_module_access_for_key(&mut conn, id).await?;
            // Fetch permission plans (new system)
            let permission_plans = self.get_permission_plans_for_key(&mut conn, id).await?;

            Ok(Some(ApiKey {
                id: core.id,
                key_prefix: core.key_prefix,
                full_key: None,
                client_name: core.client_name,
                client_description: core.client_description,
                client_contact_email: core.client_contact_email,
                wallet_address: core.wallet_address,
                status: Self::effective_status(&core.status, core.expires_at),
                total_requests: core.total_requests,
                ip_restrictions: core
                    .ip_restrictions
                    .unwrap_or_default()
                    .into_iter()
                    .flatten()
                    .collect(),
                rate_limits: RateLimits {
                    per_minute: core.rate_limit_per_minute,
                    per_day: core.rate_limit_per_day,
                },
                allowed_modules: modules,
                permission_plans,
                selected_permissions: self.get_selected_permissions_for_key(&mut conn, id).await?,
                expires_at: core.expires_at,
                last_used_at: meta.last_used_at,
                revoked_at: meta.revoked_at,
                revoked_by: meta.revoked_by,
                revocation_reason: meta.revocation_reason,
                created_at: core.created_at,
                created_by: core.created_by,
                updated_at: meta.updated_at,
            }))
        } else {
            Ok(None)
        }
    }

    fn effective_status(status: &str, expires_at: Option<chrono::DateTime<Utc>>) -> ApiKeyStatus {
        if status.eq_ignore_ascii_case("active")
            && expires_at.is_some_and(|expires_at| expires_at < Utc::now())
        {
            ApiKeyStatus::Expired
        } else {
            ApiKeyStatus::from(status)
        }
    }

    async fn load_authoritative_ids(&self, ids: Vec<Uuid>) -> AppResult<Vec<ApiKey>> {
        let mut keys = Vec::with_capacity(ids.len());
        for id in ids {
            let key = self
                .get_by_id(id)
                .await?
                .ok_or_else(|| AppError::database_error("API key disappeared during list read"))?;
            keys.push(key);
        }
        Ok(keys)
    }

    /// Get module access for an API key
    async fn get_module_access_for_key(
        &self,
        conn: &mut diesel_async::AsyncPgConnection,
        api_key_id: Uuid,
    ) -> AppResult<Vec<ModuleAccess>> {
        #[derive(Queryable)]
        struct ModuleAccessRow {
            module_id: Uuid,
            access_level: String,
            custom_rate_limit: Option<i32>,
            custom_quotas: serde_json::Value,
            module_name: String,
        }

        let rows = api_key_module_access::table
            .inner_join(api_modules::table.on(api_key_module_access::module_id.eq(api_modules::id)))
            .filter(api_key_module_access::api_key_id.eq(&api_key_id))
            .select((
                api_key_module_access::module_id,
                api_key_module_access::access_level,
                api_key_module_access::custom_rate_limit,
                api_key_module_access::custom_quotas,
                api_modules::name,
            ))
            .load::<ModuleAccessRow>(conn)
            .await
            .map_err(|e| {
                AppError::database_error(format!("Failed to fetch module access: {}", e))
            })?;

        Ok(rows
            .into_iter()
            .map(|row| ModuleAccess {
                module_id: row.module_id,
                module_name: row.module_name,
                access_level: AccessLevel::from(row.access_level.as_str()),
                custom_rate_limit: row.custom_rate_limit,
                custom_quotas: row.custom_quotas,
            })
            .collect())
    }

    /// Get permission plans for an API key
    async fn get_permission_plans_for_key(
        &self,
        conn: &mut diesel_async::AsyncPgConnection,
        api_key_id: Uuid,
    ) -> AppResult<Vec<PlanInfo>> {
        use crate::schemas::primary::api_key_permissions;
        use crate::schemas::primary::plans;

        // 1. Get plan IDs from permissions table
        let plan_ids: Vec<Uuid> = api_key_permissions::table
            .filter(api_key_permissions::api_key_id.eq(&api_key_id))
            .filter(api_key_permissions::is_active.eq(true))
            .select(api_key_permissions::plan_id)
            .load::<Uuid>(conn)
            .await
            .map_err(|e| {
                AppError::database_error(format!("Failed to fetch permission IDs: {}", e))
            })?;

        if plan_ids.is_empty() {
            return Ok(Vec::new());
        }

        // 2. Fetch plans details
        #[derive(Queryable)]
        struct PlanRow {
            id: Uuid,
            name: String,
            slug: String,
            description: String,
            plan_type: String,
        }

        let rows = plans::table
            .filter(plans::id.eq_any(plan_ids))
            .select((
                plans::id,
                plans::name,
                plans::slug,
                plans::description,
                plans::plan_type,
            ))
            .load::<PlanRow>(conn)
            .await
            .map_err(|e| AppError::database_error(format!("Failed to fetch plans: {}", e)))?;

        Ok(rows
            .into_iter()
            .map(|row| PlanInfo {
                id: row.id,
                name: row.name,
                slug: row.slug,
                description: Some(row.description),
                plan_type: row.plan_type,
            })
            .collect())
    }

    /// Get selected permissions for an API key
    /// Uses raw SQL to handle Nullable<Array<Nullable<Text>>> column type
    async fn get_selected_permissions_for_key(
        &self,
        conn: &mut diesel_async::AsyncPgConnection,
        api_key_id: Uuid,
    ) -> AppResult<Vec<String>> {
        use diesel_async::RunQueryDsl;

        #[derive(diesel::QueryableByName)]
        struct PermissionsRow {
            #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Array<diesel::sql_types::Nullable<diesel::sql_types::Text>>>)]
            selected_permissions: Option<Vec<Option<String>>>,
        }

        let result = diesel::sql_query("SELECT selected_permissions FROM api_keys WHERE id = $1")
            .bind::<diesel::sql_types::Uuid, _>(&api_key_id)
            .get_result::<PermissionsRow>(conn)
            .await
            .map_err(|e| {
                AppError::database_error(format!("Failed to fetch selected permissions: {}", e))
            })?;

        Ok(result
            .selected_permissions
            .unwrap_or_default()
            .into_iter()
            .flatten()
            .collect())
    }

    /// List all API keys with optional filters
    pub async fn list_all(
        &self,
        limit: Option<i64>,
        offset: Option<i64>,
        status_filter: Option<&str>,
    ) -> AppResult<(Vec<ApiKey>, i64)> {
        let mut conn = self
            .pool
            .get()
            .await
            .map_err(|e| AppError::database_error(format!("Pool error: {}", e)))?;
        let mut count_query = api_keys::table.into_boxed();
        match status_filter {
            None => {}
            Some("active") => {
                count_query = count_query.filter(
                    api_keys::status.eq("active").and(
                        api_keys::expires_at
                            .is_null()
                            .or(api_keys::expires_at.ge(Utc::now())),
                    ),
                );
            }
            Some("revoked") => count_query = count_query.filter(api_keys::status.eq("revoked")),
            Some("expired") => {
                count_query = count_query.filter(
                    api_keys::status
                        .eq("active")
                        .and(api_keys::expires_at.lt(Utc::now())),
                )
            }
            Some(_) => return Err(AppError::validation_error("Unsupported API key status")),
        }

        let total: i64 = count_query
            .count()
            .get_result(&mut conn)
            .await
            .map_err(|e| AppError::database_error(format!("Failed to count API keys: {}", e)))?;

        let mut query = api_keys::table.into_boxed();
        match status_filter {
            None => {}
            Some("active") => {
                query = query.filter(
                    api_keys::status.eq("active").and(
                        api_keys::expires_at
                            .is_null()
                            .or(api_keys::expires_at.ge(Utc::now())),
                    ),
                );
            }
            Some("revoked") => query = query.filter(api_keys::status.eq("revoked")),
            Some("expired") => {
                query = query.filter(
                    api_keys::status
                        .eq("active")
                        .and(api_keys::expires_at.lt(Utc::now())),
                )
            }
            Some(_) => unreachable!("status filter was validated for the count query"),
        }
        let ids: Vec<Uuid> = query
            .order(api_keys::created_at.desc())
            .limit(limit.unwrap_or(50))
            .offset(offset.unwrap_or(0))
            .select(api_keys::id)
            .load(&mut conn)
            .await
            .map_err(|e| AppError::database_error(format!("Failed to list API keys: {}", e)))?;
        drop(conn);
        Ok((self.load_authoritative_ids(ids).await?, total))
    }

    /// List API keys for a specific wallet address
    pub async fn list_by_wallet(
        &self,
        wallet_address: &str,
        limit: Option<i64>,
        offset: Option<i64>,
        status_filter: Option<&str>,
    ) -> AppResult<(Vec<ApiKey>, i64)> {
        let mut conn = self
            .pool
            .get()
            .await
            .map_err(|e| AppError::database_error(format!("Pool error: {}", e)))?;
        let mut count_query = api_keys::table.into_boxed();
        count_query = count_query.filter(api_keys::wallet_address.ilike(wallet_address));
        match status_filter {
            None => {}
            Some("active") => {
                count_query = count_query.filter(
                    api_keys::status.eq("active").and(
                        api_keys::expires_at
                            .is_null()
                            .or(api_keys::expires_at.ge(Utc::now())),
                    ),
                );
            }
            Some("revoked") => count_query = count_query.filter(api_keys::status.eq("revoked")),
            Some("expired") => {
                count_query = count_query.filter(
                    api_keys::status
                        .eq("active")
                        .and(api_keys::expires_at.lt(Utc::now())),
                )
            }
            Some(_) => return Err(AppError::validation_error("Unsupported API key status")),
        }

        let total: i64 = count_query
            .count()
            .get_result(&mut conn)
            .await
            .map_err(|e| AppError::database_error(format!("Failed to count API keys: {}", e)))?;

        let mut query = api_keys::table.into_boxed();
        query = query.filter(api_keys::wallet_address.ilike(wallet_address));
        match status_filter {
            None => {}
            Some("active") => {
                query = query.filter(
                    api_keys::status.eq("active").and(
                        api_keys::expires_at
                            .is_null()
                            .or(api_keys::expires_at.ge(Utc::now())),
                    ),
                );
            }
            Some("revoked") => query = query.filter(api_keys::status.eq("revoked")),
            Some("expired") => {
                query = query.filter(
                    api_keys::status
                        .eq("active")
                        .and(api_keys::expires_at.lt(Utc::now())),
                )
            }
            Some(_) => unreachable!("status filter was validated for the count query"),
        }
        let ids: Vec<Uuid> = query
            .order(api_keys::created_at.desc())
            .limit(limit.unwrap_or(50))
            .offset(offset.unwrap_or(0))
            .select(api_keys::id)
            .load(&mut conn)
            .await
            .map_err(|e| AppError::database_error(format!("Failed to list API keys: {}", e)))?;

        drop(conn);
        Ok((self.load_authoritative_ids(ids).await?, total))
    }

    /// Revoke an API key
    pub async fn revoke(&self, id: Uuid, request: RevokeApiKeyRequest) -> AppResult<ApiKey> {
        let mut conn = self
            .pool
            .get()
            .await
            .map_err(|e| AppError::database_error(format!("Pool error: {}", e)))?;

        let now = Utc::now();

        diesel::update(api_keys::table)
            .filter(api_keys::id.eq(&id))
            .set((
                api_keys::status.eq("revoked"),
                api_keys::revoked_at.eq(&now),
                api_keys::revoked_by.eq(&request.revoked_by),
                api_keys::revocation_reason.eq(&request.reason),
                api_keys::updated_at.eq(&now),
            ))
            .execute(&mut conn)
            .await
            .map_err(|e| AppError::database_error(format!("Failed to revoke API key: {}", e)))?;

        info!(
            "Revoked API key {} by {}: {}",
            id, request.revoked_by, request.reason
        );

        self.get_by_id(id)
            .await?
            .ok_or_else(|| AppError::not_found("API key not found"))
    }

    /// Validate an API key by its raw value
    pub async fn validate_key(&self, raw_key: &str) -> AppResult<Option<ApiKey>> {
        let key_hash = Self::hash_api_key(raw_key);

        let mut conn = self
            .pool
            .get()
            .await
            .map_err(|e| AppError::database_error(format!("Pool error: {}", e)))?;

        let id: Option<Uuid> = api_keys::table
            .filter(api_keys::key_hash.eq(&key_hash))
            .filter(api_keys::status.eq("active"))
            .select(api_keys::id)
            .first(&mut conn)
            .await
            .optional()
            .map_err(|e| AppError::database_error(format!("Failed to validate key: {}", e)))?;

        if let Some(id) = id {
            // Update last_used_at
            diesel::update(api_keys::table)
                .filter(api_keys::id.eq(&id))
                .set((
                    api_keys::last_used_at.eq(Utc::now()),
                    api_keys::total_requests.eq(api_keys::total_requests + 1),
                ))
                .execute(&mut conn)
                .await
                .ok(); // Don't fail if update fails

            self.get_by_id(id).await
        } else {
            Ok(None)
        }
    }

    /// Update expiration date for an API key
    pub async fn update_expiration(
        &self,
        id: Uuid,
        expires_at: Option<chrono::DateTime<Utc>>,
    ) -> AppResult<ApiKey> {
        let mut conn = self
            .pool
            .get()
            .await
            .map_err(|e| AppError::database_error(format!("Pool error: {}", e)))?;

        let now = Utc::now();

        diesel::update(api_keys::table)
            .filter(api_keys::id.eq(&id))
            .set((
                api_keys::expires_at.eq(&expires_at),
                api_keys::updated_at.eq(&now),
            ))
            .execute(&mut conn)
            .await
            .map_err(|e| AppError::database_error(format!("Failed to update expiration: {}", e)))?;

        info!("Updated API key {} expiration to {:?}", id, expires_at);

        self.get_by_id(id)
            .await?
            .ok_or_else(|| AppError::not_found("API key not found"))
    }

    /// List API keys expiring within the specified number of days
    /// Returns keys planed by wallet address for admin tracking
    pub async fn list_expiring_keys(
        &self,
        days: i64,
        limit: Option<i64>,
        offset: Option<i64>,
    ) -> AppResult<(Vec<ApiKey>, i64)> {
        let mut conn = self
            .pool
            .get()
            .await
            .map_err(|e| AppError::database_error(format!("Pool error: {}", e)))?;
        let now = Utc::now();
        let expiry_threshold = now + chrono::Duration::days(days);

        let count_query = api_keys::table
            .filter(api_keys::expires_at.is_not_null())
            .filter(api_keys::expires_at.le(&expiry_threshold))
            .filter(api_keys::expires_at.gt(&now))
            .filter(api_keys::status.eq("active"));
        let total: i64 = count_query
            .count()
            .get_result(&mut conn)
            .await
            .map_err(|e| {
                AppError::database_error(format!("Failed to count expiring keys: {}", e))
            })?;

        let ids: Vec<Uuid> = api_keys::table
            .filter(api_keys::expires_at.is_not_null())
            .filter(api_keys::expires_at.le(&expiry_threshold))
            .filter(api_keys::expires_at.gt(&now))
            .filter(api_keys::status.eq("active"))
            .order(api_keys::expires_at.asc())
            .limit(limit.unwrap_or(50))
            .offset(offset.unwrap_or(0))
            .select(api_keys::id)
            .load(&mut conn)
            .await
            .map_err(|e| {
                AppError::database_error(format!("Failed to list expiring keys: {}", e))
            })?;
        drop(conn);
        let api_keys_result = self.load_authoritative_ids(ids).await?;
        info!(
            "Found {} expiring API keys within {} days",
            api_keys_result.len(),
            days
        );
        Ok((api_keys_result, total))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn effective_status_exposes_persisted_expiration_without_fabrication() {
        assert_eq!(
            ApiKeyRepository::effective_status("active", None),
            ApiKeyStatus::Active
        );
        assert_eq!(
            ApiKeyRepository::effective_status(
                "active",
                Some(Utc::now() - chrono::Duration::seconds(1))
            ),
            ApiKeyStatus::Expired
        );
        assert_eq!(
            ApiKeyRepository::effective_status(
                "revoked",
                Some(Utc::now() - chrono::Duration::seconds(1))
            ),
            ApiKeyStatus::Revoked
        );
    }
}
