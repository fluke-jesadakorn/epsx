//! Module Repository
//!
//! Handles database operations for API modules.
//!
//! BIG-BANG: migrated to sqlx (real).

use chrono::Utc;
use sqlx::PgPool;
use std::sync::Arc;
use tracing::info;
use uuid::Uuid;

use crate::domain::developer_portal::{
    ApiModule, CreateModuleRequest, ModuleEndpoint, ModuleListResponse, ModuleStatus,
    UpdateModuleRequest,
};
use crate::prelude::*;

#[derive(sqlx::FromRow)]
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

/// Module Repository for database operations
pub struct ModuleRepository {
    pool: Arc<PgPool>,
}

impl ModuleRepository {
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }

    /// List all modules with optional filters
    pub async fn list(
        &self,
        status_filter: Option<&str>,
        category_filter: Option<&str>,
    ) -> AppResult<ModuleListResponse> {
        let pool: &PgPool = self.pool.as_ref();

        let mut sql = String::from(
            "SELECT id, name, display_name, description, category, status, base_path, \
                    default_rate_limit, access_levels, endpoints, created_at, updated_at \
             FROM api_modules WHERE TRUE",
        );
        if let Some(_status) = status_filter {
            sql.push_str(" AND status = $1");
        }
        if let Some(_category) = category_filter {
            sql.push_str(&format!(
                " AND category = ${}",
                if status_filter.is_some() { 2 } else { 1 }
            ));
        }
        sql.push_str(" ORDER BY display_name ASC");

        let mut qb: sqlx::QueryBuilder<sqlx::Postgres> = sqlx::QueryBuilder::new(&sql);
        if let Some(status) = status_filter {
            qb.push_bind(status);
        }
        if let Some(category) = category_filter {
            qb.push_bind(category);
        }

        let rows: Vec<ModuleRow> = qb
            .build_query_as()
            .fetch_all(pool)
            .await
            .map_err(|e| AppError::database_error(format!("Failed to list modules: {}", e)))?;

        let total = rows.len() as i64;
        let modules: Vec<ApiModule> = rows
            .into_iter()
            .map(|row| {
                let endpoints: Vec<ModuleEndpoint> =
                    serde_json::from_value(row.endpoints.clone()).unwrap_or_default();
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
            })
            .collect();

        Ok(ModuleListResponse { modules, total })
    }

    /// Get a module by ID
    pub async fn get_by_id(&self, id: Uuid) -> AppResult<Option<ApiModule>> {
        let pool: &PgPool = &self.pool;

        let row: Option<ModuleRow> = sqlx::query_as(
            "SELECT id, name, display_name, description, category, status, base_path, \
                    default_rate_limit, access_levels, endpoints, created_at, updated_at \
             FROM api_modules WHERE id = $1",
        )
        .bind(id)
        .fetch_optional(pool)
        .await
        .map_err(|e| AppError::database_error(format!("Failed to fetch module: {}", e)))?;

        Ok(row.map(|r| {
            let endpoints: Vec<ModuleEndpoint> =
                serde_json::from_value(r.endpoints).unwrap_or_default();
            ApiModule {
                id: r.id,
                name: r.name,
                display_name: r.display_name,
                description: r.description,
                category: r.category,
                status: ModuleStatus::from(r.status.as_str()),
                base_path: r.base_path,
                default_rate_limit: r.default_rate_limit,
                access_levels: r.access_levels,
                endpoints,
                created_at: r.created_at,
                updated_at: r.updated_at,
            }
        }))
    }

    /// Create a new module
    pub async fn create(&self, request: CreateModuleRequest) -> AppResult<ApiModule> {
        let pool: &PgPool = &self.pool;

        let id = Uuid::new_v4();
        let now = Utc::now();
        let endpoints_json =
            serde_json::to_value(request.endpoints.unwrap_or_default()).map_err(|e| {
                AppError::internal_error(format!("Failed to serialize endpoints: {}", e))
            })?;
        let access_levels = request.access_levels.unwrap_or(serde_json::json!({}));

        sqlx::query(
            "INSERT INTO api_modules (\
                id, name, display_name, description, category, status, base_path, \
                default_rate_limit, access_levels, endpoints, created_at, updated_at\
            ) VALUES ($1, $2, $3, $4, $5, 'active', $6, $7, $8, $9, $10, $10)",
        )
        .bind(id)
        .bind(&request.name)
        .bind(&request.display_name)
        .bind(&request.description)
        .bind(&request.category)
        .bind(&request.base_path)
        .bind(request.default_rate_limit.unwrap_or(60))
        .bind(&access_levels)
        .bind(&endpoints_json)
        .bind(now)
        .execute(pool)
        .await
        .map_err(|e| AppError::database_error(format!("Failed to create module: {}", e)))?;

        info!("Created module {} ({})", request.display_name, request.name);

        self.get_by_id(id)
            .await?
            .ok_or_else(|| AppError::not_found("Module not found after creation"))
    }

    /// Update a module
    pub async fn update(&self, id: Uuid, request: UpdateModuleRequest) -> AppResult<ApiModule> {
        let pool: &PgPool = &self.pool;

        let now = Utc::now();

        // Build dynamic update via QueryBuilder for conditional sets
        let mut qb: sqlx::QueryBuilder<sqlx::Postgres> =
            sqlx::QueryBuilder::new("UPDATE api_modules SET updated_at = ");
        qb.push_bind(now);
        let mut updated = false;

        if let Some(display_name) = &request.display_name {
            qb.push(", display_name = ").push_bind(display_name.clone());
            updated = true;
        }
        if let Some(description) = &request.description {
            qb.push(", description = ").push_bind(description.clone());
            updated = true;
        }
        if let Some(status) = &request.status {
            qb.push(", status = ").push_bind(status.clone());
            updated = true;
        }
        if let Some(rate_limit) = request.default_rate_limit {
            qb.push(", default_rate_limit = ").push_bind(rate_limit);
            updated = true;
        }
        if let Some(access_levels) = &request.access_levels {
            qb.push(", access_levels = ")
                .push_bind(access_levels.clone());
            updated = true;
        }
        if let Some(endpoints) = &request.endpoints {
            let endpoints_json = serde_json::to_value(endpoints).map_err(|e| {
                AppError::internal_error(format!("Failed to serialize endpoints: {}", e))
            })?;
            qb.push(", endpoints = ").push_bind(endpoints_json);
            updated = true;
        }

        if updated {
            qb.push(" WHERE id = ").push_bind(id);
            qb.build()
                .execute(pool)
                .await
                .map_err(|e| AppError::database_error(format!("Failed to update module: {}", e)))?;
        }

        self.get_by_id(id)
            .await?
            .ok_or_else(|| AppError::not_found("Module not found"))
    }
}
