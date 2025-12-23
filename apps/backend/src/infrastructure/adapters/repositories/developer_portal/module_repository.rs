//! Module Repository
//!
//! Handles database operations for API modules.

use chrono::Utc;
use diesel::prelude::*;
use diesel_async::{AsyncPgConnection, RunQueryDsl, pooled_connection::deadpool::Pool};
use tracing::info;
use uuid::Uuid;

use crate::domain::developer_portal::{
    ApiModule, ModuleStatus, ModuleEndpoint, ModuleListResponse,
    CreateModuleRequest, UpdateModuleRequest,
};
use crate::prelude::*;
use crate::schema::api_modules;

/// Module Repository for database operations
pub struct ModuleRepository {
    pool: &'static Pool<AsyncPgConnection>,
}

impl ModuleRepository {
    pub fn new(pool: &'static Pool<AsyncPgConnection>) -> Self {
        Self { pool }
    }

    /// List all modules with optional filters
    pub async fn list(
        &self,
        status_filter: Option<&str>,
        category_filter: Option<&str>,
    ) -> AppResult<ModuleListResponse> {
        let mut conn = self.pool.get().await
            .map_err(|e| AppError::database_error(format!("Pool error: {}", e)))?;

        let mut query = api_modules::table.into_boxed();

        if let Some(status) = status_filter {
            query = query.filter(api_modules::status.eq(status));
        }

        if let Some(category) = category_filter {
            query = query.filter(api_modules::category.eq(category));
        }

        #[derive(Queryable)]
        struct ModuleRow {
            id: Uuid,
            name: String,
            display_name: String,
            description: Option<String>,
            category: String,
            status: String,
            base_path: String,
            default_rate_limit: i32,
            access_levels: serde_json::Value,
            endpoints: serde_json::Value,
            created_at: chrono::DateTime<Utc>,
            updated_at: chrono::DateTime<Utc>,
        }

        let rows = query
            .order(api_modules::display_name.asc())
            .load::<ModuleRow>(&mut conn)
            .await
            .map_err(|e| AppError::database_error(format!("Failed to list modules: {}", e)))?;

        let total = rows.len() as i64;
        let modules: Vec<ApiModule> = rows.into_iter().map(|row| {
            let endpoints: Vec<ModuleEndpoint> = serde_json::from_value(row.endpoints.clone())
                .unwrap_or_default();
            
            ApiModule {
                id: row.id,
                name: row.name,
                display_name: row.display_name,
                description: row.description,
                category: row.category,
                status: ModuleStatus::from(row.status.as_str()),
                base_path: row.base_path,
                default_rate_limit: row.default_rate_limit,
                access_levels: row.access_levels,
                endpoints,
                created_at: row.created_at,
                updated_at: row.updated_at,
            }
        }).collect();

        Ok(ModuleListResponse { modules, total })
    }

    /// Get a module by ID
    pub async fn get_by_id(&self, id: Uuid) -> AppResult<Option<ApiModule>> {
        let mut conn = self.pool.get().await
            .map_err(|e| AppError::database_error(format!("Pool error: {}", e)))?;

        #[derive(Queryable)]
        struct ModuleRow {
            id: Uuid,
            name: String,
            display_name: String,
            description: Option<String>,
            category: String,
            status: String,
            base_path: String,
            default_rate_limit: i32,
            access_levels: serde_json::Value,
            endpoints: serde_json::Value,
            created_at: chrono::DateTime<Utc>,
            updated_at: chrono::DateTime<Utc>,
        }

        let row = api_modules::table
            .filter(api_modules::id.eq(&id))
            .first::<ModuleRow>(&mut conn)
            .await
            .optional()
            .map_err(|e| AppError::database_error(format!("Failed to fetch module: {}", e)))?;

        Ok(row.map(|row| {
            let endpoints: Vec<ModuleEndpoint> = serde_json::from_value(row.endpoints.clone())
                .unwrap_or_default();
            
            ApiModule {
                id: row.id,
                name: row.name,
                display_name: row.display_name,
                description: row.description,
                category: row.category,
                status: ModuleStatus::from(row.status.as_str()),
                base_path: row.base_path,
                default_rate_limit: row.default_rate_limit,
                access_levels: row.access_levels,
                endpoints,
                created_at: row.created_at,
                updated_at: row.updated_at,
            }
        }))
    }

    /// Create a new module
    pub async fn create(&self, request: CreateModuleRequest) -> AppResult<ApiModule> {
        let mut conn = self.pool.get().await
            .map_err(|e| AppError::database_error(format!("Pool error: {}", e)))?;

        let id = Uuid::new_v4();
        let now = Utc::now();
        let endpoints_json = serde_json::to_value(&request.endpoints.unwrap_or_default())
            .map_err(|e| AppError::internal_error(format!("Failed to serialize endpoints: {}", e)))?;
        let access_levels = request.access_levels.unwrap_or(serde_json::json!({}));

        diesel::insert_into(api_modules::table)
            .values((
                api_modules::id.eq(&id),
                api_modules::name.eq(&request.name),
                api_modules::display_name.eq(&request.display_name),
                api_modules::description.eq(&request.description),
                api_modules::category.eq(&request.category),
                api_modules::status.eq("active"),
                api_modules::base_path.eq(&request.base_path),
                api_modules::default_rate_limit.eq(request.default_rate_limit.unwrap_or(60)),
                api_modules::access_levels.eq(&access_levels),
                api_modules::endpoints.eq(&endpoints_json),
                api_modules::created_at.eq(&now),
                api_modules::updated_at.eq(&now),
            ))
            .execute(&mut conn)
            .await
            .map_err(|e| AppError::database_error(format!("Failed to create module: {}", e)))?;

        info!("Created module {} ({})", request.display_name, request.name);

        self.get_by_id(id).await?
            .ok_or_else(|| AppError::not_found("Module not found after creation"))
    }

    /// Update a module
    pub async fn update(&self, id: Uuid, request: UpdateModuleRequest) -> AppResult<ApiModule> {
        let mut conn = self.pool.get().await
            .map_err(|e| AppError::database_error(format!("Pool error: {}", e)))?;

        let now = Utc::now();

        // Build dynamic update
        if let Some(display_name) = &request.display_name {
            diesel::update(api_modules::table)
                .filter(api_modules::id.eq(&id))
                .set((
                    api_modules::display_name.eq(display_name),
                    api_modules::updated_at.eq(&now),
                ))
                .execute(&mut conn)
                .await
                .map_err(|e| AppError::database_error(format!("Failed to update module: {}", e)))?;
        }

        if let Some(description) = &request.description {
            diesel::update(api_modules::table)
                .filter(api_modules::id.eq(&id))
                .set((
                    api_modules::description.eq(description),
                    api_modules::updated_at.eq(&now),
                ))
                .execute(&mut conn)
                .await
                .map_err(|e| AppError::database_error(format!("Failed to update module: {}", e)))?;
        }

        if let Some(status) = &request.status {
            diesel::update(api_modules::table)
                .filter(api_modules::id.eq(&id))
                .set((
                    api_modules::status.eq(status),
                    api_modules::updated_at.eq(&now),
                ))
                .execute(&mut conn)
                .await
                .map_err(|e| AppError::database_error(format!("Failed to update module: {}", e)))?;
        }

        if let Some(rate_limit) = request.default_rate_limit {
            diesel::update(api_modules::table)
                .filter(api_modules::id.eq(&id))
                .set((
                    api_modules::default_rate_limit.eq(rate_limit),
                    api_modules::updated_at.eq(&now),
                ))
                .execute(&mut conn)
                .await
                .map_err(|e| AppError::database_error(format!("Failed to update module: {}", e)))?;
        }

        if let Some(access_levels) = &request.access_levels {
            diesel::update(api_modules::table)
                .filter(api_modules::id.eq(&id))
                .set((
                    api_modules::access_levels.eq(access_levels),
                    api_modules::updated_at.eq(&now),
                ))
                .execute(&mut conn)
                .await
                .map_err(|e| AppError::database_error(format!("Failed to update module: {}", e)))?;
        }

        if let Some(endpoints) = &request.endpoints {
            let endpoints_json = serde_json::to_value(endpoints)
                .map_err(|e| AppError::internal_error(format!("Failed to serialize endpoints: {}", e)))?;
            diesel::update(api_modules::table)
                .filter(api_modules::id.eq(&id))
                .set((
                    api_modules::endpoints.eq(&endpoints_json),
                    api_modules::updated_at.eq(&now),
                ))
                .execute(&mut conn)
                .await
                .map_err(|e| AppError::database_error(format!("Failed to update module: {}", e)))?;
        }

        self.get_by_id(id).await?
            .ok_or_else(|| AppError::not_found("Module not found"))
    }
}
