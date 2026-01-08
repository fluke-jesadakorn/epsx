//! API Key Repository
//!
//! Handles database operations for API keys.

use chrono::Utc;
use diesel::prelude::*;
use diesel_async::{AsyncPgConnection, RunQueryDsl, pooled_connection::deadpool::Pool};
use tracing::info;
use uuid::Uuid;

use crate::domain::developer_portal::{
    ApiKey, ApiKeyStatus, ModuleAccess, RateLimits, CreateApiKeyRequest, 
    RevokeApiKeyRequest, ApiKeyCreatedResponse, AccessLevel,
    PermissionGroupInfo,
};
use crate::prelude::*;
use crate::schemas::primary::{api_keys, api_key_module_access, api_key_permissions, api_modules};

/// API Key Repository for database operations
pub struct ApiKeyRepository {
    pool: &'static Pool<AsyncPgConnection>,
}

impl ApiKeyRepository {
    pub fn new(pool: &'static Pool<AsyncPgConnection>) -> Self {
        Self { pool }
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
        use sha2::{Sha256, Digest};
        let mut hasher = Sha256::new();
        hasher.update(key.as_bytes());
        hex::encode(hasher.finalize())
    }

    /// Create a new API key
    pub async fn create(&self, request: CreateApiKeyRequest) -> AppResult<ApiKeyCreatedResponse> {
        let mut conn = self.pool.get().await
            .map_err(|e| AppError::database_error(format!("Pool error: {}", e)))?;

        let (full_key, prefix) = Self::generate_api_key();
        let key_hash = Self::hash_api_key(&full_key);
        let now = Utc::now();
        let id = Uuid::new_v4();

        // Insert API key (including full_key for user to copy later)
        diesel::insert_into(api_keys::table)
            .values((
                api_keys::id.eq(&id),
                api_keys::key_hash.eq(&key_hash),
                api_keys::key_prefix.eq(&prefix),
                api_keys::full_key.eq(&full_key),
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
                    api_key_module_access::custom_quotas.eq(module_access.custom_quotas.clone().unwrap_or(serde_json::json!({}))),
                    api_key_module_access::granted_at.eq(&now),
                    api_key_module_access::granted_by.eq(&request.created_by),
                ))
                .execute(&mut conn)
                .await
                .map_err(|e| AppError::database_error(format!("Failed to add module access: {}", e)))?;
        }

        // Insert permission group assignments (new group-based system)
        for group_id in &request.group_ids {
            diesel::insert_into(api_key_permissions::table)
                .values((
                    api_key_permissions::api_key_id.eq(&id),
                    api_key_permissions::permission_group_id.eq(group_id),
                    api_key_permissions::granted_at.eq(&now),
                    api_key_permissions::granted_by.eq(&request.created_by),
                    api_key_permissions::is_active.eq(true),
                    api_key_permissions::metadata.eq(serde_json::json!({})),
                ))
                .execute(&mut conn)
                .await
                .map_err(|e| AppError::database_error(format!("Failed to add permission group: {}", e)))?;
        }

        info!("Created API key {} for wallet {} with {} groups", id, request.wallet_address, request.group_ids.len());

        // Fetch the created key with modules
        let api_key = self.get_by_id(id).await?
            .ok_or_else(|| AppError::not_found("API key not found after creation"))?;

        Ok(ApiKeyCreatedResponse {
            api_key,
            full_key,
        })
    }

    /// Get an API key by ID
    pub async fn get_by_id(&self, id: Uuid) -> AppResult<Option<ApiKey>> {
        let mut conn = self.pool.get().await
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
            full_key: Option<String>,
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
                api_keys::full_key,
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
            .map_err(|e| AppError::database_error(format!("Failed to fetch API key metadata: {}", e)))?;

        if let (Some(core), Some(meta)) = (core_row, meta_row) {
            // Fetch module access (legacy)
            let modules = self.get_module_access_for_key(&mut conn, id).await?;
            // Fetch permission groups (new system)
            let permission_groups = self.get_permission_groups_for_key(&mut conn, id).await?;

            Ok(Some(ApiKey {
                id: core.id,
                key_prefix: core.key_prefix,
                full_key: core.full_key,
                client_name: core.client_name,
                client_description: core.client_description,
                client_contact_email: core.client_contact_email,
                wallet_address: core.wallet_address,
                status: ApiKeyStatus::from(core.status.as_str()),
                total_requests: core.total_requests,
                ip_restrictions: core.ip_restrictions
                    .unwrap_or_default()
                    .into_iter()
                    .flatten()
                    .collect(),
                rate_limits: RateLimits {
                    per_minute: core.rate_limit_per_minute,
                    per_day: core.rate_limit_per_day,
                },
                allowed_modules: modules,
                permission_groups,
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

    /// Get module access for an API key
    async fn get_module_access_for_key(
        &self,
        conn: &mut AsyncPgConnection,
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
            .inner_join(api_modules::table.on(api_modules::id.eq(api_key_module_access::module_id)))
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
            .map_err(|e| AppError::database_error(format!("Failed to fetch module access: {}", e)))?;

        Ok(rows.into_iter().map(|row| ModuleAccess {
            module_id: row.module_id,
            module_name: row.module_name,
            access_level: AccessLevel::from(row.access_level.as_str()),
            custom_rate_limit: row.custom_rate_limit,
            custom_quotas: row.custom_quotas,
        }).collect())
    }

    /// Get permission groups for an API key
    async fn get_permission_groups_for_key(
        &self,
        conn: &mut AsyncPgConnection,
        api_key_id: Uuid,
    ) -> AppResult<Vec<PermissionGroupInfo>> {
        use crate::schemas::primary::{api_key_permissions, groups};

        #[derive(Queryable)]
        struct GroupRow {
            id: Uuid,
            name: String,
            slug: String,
            description: String,
            group_type: String,
        }

        let rows = api_key_permissions::table
            .inner_join(groups::table.on(groups::id.eq(api_key_permissions::permission_group_id)))
            .filter(api_key_permissions::api_key_id.eq(&api_key_id))
            .filter(api_key_permissions::is_active.eq(true))
            .select((
                groups::id,
                groups::name,
                groups::slug,
                groups::description,
                groups::group_type,
            ))
            .load::<GroupRow>(conn)
            .await
            .map_err(|e| AppError::database_error(format!("Failed to fetch permission groups: {}", e)))?;

        Ok(rows.into_iter().map(|row| PermissionGroupInfo {
            id: row.id,
            name: row.name,
            slug: row.slug,
            description: Some(row.description),
            group_type: row.group_type,
        }).collect())
    }

    /// Get selected permissions for an API key
    /// Uses raw SQL to handle Nullable<Array<Nullable<Text>>> column type
    async fn get_selected_permissions_for_key(
        &self,
        conn: &mut AsyncPgConnection,
        api_key_id: Uuid,
    ) -> AppResult<Vec<String>> {
        use diesel_async::RunQueryDsl;
        
        #[derive(diesel::QueryableByName)]
        struct PermissionsRow {
            #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Array<diesel::sql_types::Nullable<diesel::sql_types::Text>>>)]
            selected_permissions: Option<Vec<Option<String>>>,
        }

        let result = diesel::sql_query(
            "SELECT selected_permissions FROM api_keys WHERE id = $1"
        )
        .bind::<diesel::sql_types::Uuid, _>(&api_key_id)
        .get_result::<PermissionsRow>(conn)
        .await
        .ok();

        Ok(result
            .and_then(|r| r.selected_permissions)
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
        let mut conn = self.pool.get().await
            .map_err(|e| AppError::database_error(format!("Pool error: {}", e)))?;

        // Build query
        let mut query = api_keys::table.into_boxed();

        if let Some(status) = status_filter {
            query = query.filter(api_keys::status.eq(status));
        }

        // Get total count
        let total: i64 = api_keys::table
            .count()
            .get_result(&mut conn)
            .await
            .map_err(|e| AppError::database_error(format!("Failed to count API keys: {}", e)))?;

        // Apply pagination
        let query = query
            .order(api_keys::created_at.desc())
            .limit(limit.unwrap_or(50))
            .offset(offset.unwrap_or(0));

        // API key row with 16 fields max (Diesel's default tuple limit)
        #[derive(Queryable)]
        struct ApiKeyListRow {
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
            last_used_at: Option<chrono::DateTime<Utc>>,
            created_at: chrono::DateTime<Utc>,
            created_by: String,
            full_key: Option<String>,
        }

        let rows: Vec<ApiKeyListRow> = query
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
                api_keys::last_used_at,
                api_keys::created_at,
                api_keys::created_by,
                api_keys::full_key,
            ))
            .load(&mut conn)
            .await
            .map_err(|e| AppError::database_error(format!("Failed to list API keys: {}", e)))?;

        let mut keys = Vec::new();
        for row in rows {
            let modules = self.get_module_access_for_key(&mut conn, row.id).await?;
            let permission_groups = self.get_permission_groups_for_key(&mut conn, row.id).await?;
            keys.push(ApiKey {
                id: row.id,
                key_prefix: row.key_prefix,
                full_key: row.full_key,
                client_name: row.client_name,
                client_description: row.client_description,
                client_contact_email: row.client_contact_email,
                wallet_address: row.wallet_address,
                status: ApiKeyStatus::from(row.status.as_str()),
                total_requests: row.total_requests,
                ip_restrictions: row.ip_restrictions
                    .unwrap_or_default()
                    .into_iter()
                    .flatten()
                    .collect(),
                rate_limits: RateLimits {
                    per_minute: row.rate_limit_per_minute,
                    per_day: row.rate_limit_per_day,
                },
                allowed_modules: modules,
                permission_groups,
                // For list operations, we don't fetch selected_permissions to avoid N+1 queries
                // Use get_by_id if you need the full permissions list
                selected_permissions: vec![],
                expires_at: row.expires_at,
                last_used_at: row.last_used_at,
                revoked_at: None, // Not fetched in list for performance
                revoked_by: None,
                revocation_reason: None,
                created_at: row.created_at,
                created_by: row.created_by,
                updated_at: row.created_at, // Use created_at as fallback
            });
        }

        Ok((keys, total))
    }

    /// List API keys for a specific wallet address
    pub async fn list_by_wallet(
        &self,
        wallet_address: &str,
        limit: Option<i64>,
        offset: Option<i64>,
        status_filter: Option<&str>,
    ) -> AppResult<(Vec<ApiKey>, i64)> {
        let mut conn = self.pool.get().await
            .map_err(|e| AppError::database_error(format!("Pool error: {}", e)))?;

        // Build query with wallet filter
        let mut query = api_keys::table.into_boxed();
        query = query.filter(api_keys::wallet_address.ilike(wallet_address));

        if let Some(status) = status_filter {
            query = query.filter(api_keys::status.eq(status));
        }

        // Get total count for this wallet
        let total: i64 = api_keys::table
            .filter(api_keys::wallet_address.ilike(wallet_address))
            .count()
            .get_result(&mut conn)
            .await
            .map_err(|e| AppError::database_error(format!("Failed to count API keys: {}", e)))?;

        // Apply pagination
        let query = query
            .order(api_keys::created_at.desc())
            .limit(limit.unwrap_or(50))
            .offset(offset.unwrap_or(0));

        // API key row with 16 fields max (Diesel's default tuple limit)
        #[derive(Queryable)]
        struct ApiKeyListRow {
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
            last_used_at: Option<chrono::DateTime<Utc>>,
            created_at: chrono::DateTime<Utc>,
            created_by: String,
            full_key: Option<String>,
            selected_permissions: Vec<Option<String>>,
        }

        let rows: Vec<ApiKeyListRow> = query
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
                api_keys::last_used_at,
                api_keys::created_at,
                api_keys::created_by,
                api_keys::full_key,
                api_keys::selected_permissions,
            ))
            .load(&mut conn)
            .await
            .map_err(|e| AppError::database_error(format!("Failed to list API keys: {}", e)))?;

        let mut keys = Vec::new();
        for row in rows {
            let modules = self.get_module_access_for_key(&mut conn, row.id).await?;
            let permission_groups = self.get_permission_groups_for_key(&mut conn, row.id).await?;
            keys.push(ApiKey {
                id: row.id,
                key_prefix: row.key_prefix,
                full_key: row.full_key,
                client_name: row.client_name,
                client_description: row.client_description,
                client_contact_email: row.client_contact_email,
                wallet_address: row.wallet_address,
                status: ApiKeyStatus::from(row.status.as_str()),
                total_requests: row.total_requests,
                ip_restrictions: row.ip_restrictions
                    .unwrap_or_default()
                    .into_iter()
                    .flatten()
                    .collect(),
                rate_limits: RateLimits {
                    per_minute: row.rate_limit_per_minute,
                    per_day: row.rate_limit_per_day,
                },
                allowed_modules: modules,
                permission_groups,
                selected_permissions: row.selected_permissions.into_iter().flatten().collect(),
                expires_at: row.expires_at,
                last_used_at: row.last_used_at,
                revoked_at: None,
                revoked_by: None,
                revocation_reason: None,
                created_at: row.created_at,
                created_by: row.created_by,
                updated_at: row.created_at,
            });
        }

        Ok((keys, total))
    }

    /// Revoke an API key
    pub async fn revoke(&self, id: Uuid, request: RevokeApiKeyRequest) -> AppResult<ApiKey> {
        let mut conn = self.pool.get().await
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

        info!("Revoked API key {} by {}: {}", id, request.revoked_by, request.reason);

        self.get_by_id(id).await?
            .ok_or_else(|| AppError::not_found("API key not found"))
    }

    /// Validate an API key by its raw value
    pub async fn validate_key(&self, raw_key: &str) -> AppResult<Option<ApiKey>> {
        let key_hash = Self::hash_api_key(raw_key);
        
        let mut conn = self.pool.get().await
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
        let mut conn = self.pool.get().await
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

        self.get_by_id(id).await?
            .ok_or_else(|| AppError::not_found("API key not found"))
    }

    /// List API keys expiring within the specified number of days
    /// Returns keys grouped by wallet address for admin tracking
    pub async fn list_expiring_keys(
        &self,
        days: i64,
        limit: Option<i64>,
        offset: Option<i64>,
    ) -> AppResult<(Vec<ApiKey>, i64)> {
        let mut conn = self.pool.get().await
            .map_err(|e| AppError::database_error(format!("Pool error: {}", e)))?;

        // API key row with 16 fields max (Diesel's default tuple limit)
        #[derive(Queryable)]
        struct ApiKeyListRow {
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
            last_used_at: Option<chrono::DateTime<Utc>>,
            created_at: chrono::DateTime<Utc>,
            created_by: String,
            full_key: Option<String>,
        }

        let now = Utc::now();
        let expiry_threshold = now + chrono::Duration::days(days);

        // Count total
        let total: i64 = api_keys::table
            .filter(api_keys::expires_at.is_not_null())
            .filter(api_keys::expires_at.le(&expiry_threshold))
            .filter(api_keys::expires_at.gt(&now)) // Not yet expired
            .filter(api_keys::status.eq("active"))
            .count()
            .get_result(&mut conn)
            .await
            .map_err(|e| AppError::database_error(format!("Failed to count expiring keys: {}", e)))?;

        // Fetch keys ordered by expiration date (soonest first)
        let rows: Vec<ApiKeyListRow> = api_keys::table
            .filter(api_keys::expires_at.is_not_null())
            .filter(api_keys::expires_at.le(&expiry_threshold))
            .filter(api_keys::expires_at.gt(&now))
            .filter(api_keys::status.eq("active"))
            .order(api_keys::expires_at.asc())
            .limit(limit.unwrap_or(50))
            .offset(offset.unwrap_or(0))
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
                api_keys::last_used_at,
                api_keys::created_at,
                api_keys::created_by,
                api_keys::full_key,
            ))
            .load(&mut conn)
            .await
            .map_err(|e| AppError::database_error(format!("Failed to list expiring keys: {}", e)))?;

        // Build full ApiKey objects with modules and groups
        let mut api_keys_result = Vec::new();
        for row in rows {
            let modules = self.get_module_access_for_key(&mut conn, row.id).await?;
            let permission_groups = self.get_permission_groups_for_key(&mut conn, row.id).await?;
            
            api_keys_result.push(ApiKey {
                id: row.id,
                key_prefix: row.key_prefix,
                full_key: row.full_key,
                client_name: row.client_name,
                client_description: row.client_description,
                client_contact_email: row.client_contact_email,
                wallet_address: row.wallet_address,
                status: ApiKeyStatus::from(row.status.as_str()),
                total_requests: row.total_requests,
                ip_restrictions: row.ip_restrictions
                    .unwrap_or_default()
                    .into_iter()
                    .flatten()
                    .collect(),
                rate_limits: RateLimits {
                    per_minute: row.rate_limit_per_minute,
                    per_day: row.rate_limit_per_day,
                },
                allowed_modules: modules,
                permission_groups,
                selected_permissions: vec![],
                expires_at: row.expires_at,
                last_used_at: row.last_used_at,
                revoked_at: None,
                revoked_by: None,
                revocation_reason: None,
                created_at: row.created_at,
                created_by: row.created_by,
                updated_at: row.created_at,
            });
        }

        info!("Found {} expiring API keys within {} days", api_keys_result.len(), days);
        Ok((api_keys_result, total))
    }
}
