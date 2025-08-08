use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::{PgPool, FromRow};
use uuid::Uuid;

use crate::app::ports::repositories::{TemporaryPermissionRepo, TemporaryPermissionQuery, RepoError};
use crate::dom::entities::TemporaryPermission;
use crate::dom::values::UserId;

/// PostgreSQL implementation of TemporaryPermissionRepo
#[derive(Debug)]
pub struct PostgresTemporaryPermissionRepo {
    pool: PgPool,
}

impl PostgresTemporaryPermissionRepo {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl TemporaryPermissionRepo for PostgresTemporaryPermissionRepo {
    async fn create(&self, permission: &TemporaryPermission) -> Result<TemporaryPermission, RepoError> {
        let query = r#"
            INSERT INTO temporary_permissions (
                id, user_id, permission, resource, action,
                granted_at, expires_at, auto_revoke,
                granted_by, reason, conditions,
                status, revoked_at, revoked_by, revocation_reason,
                created_at, updated_at
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17)
            RETURNING *
        "#;
        
        let row = sqlx::query(query)
            .bind(&permission.id)
            .bind(permission.user_id())
            .bind(&permission.permission)
            .bind(&permission.resource)
            .bind(&permission.action)
            .bind(&permission.granted_at)
            .bind(&permission.expires_at)
            .bind(&permission.auto_revoke)
            .bind(permission.granted_by())
            .bind(&permission.reason)
            .bind(&permission.conditions)
            .bind(&permission.status)
            .bind(&permission.revoked_at)
            .bind(permission.revoked_by())
            .bind(&permission.revocation_reason)
            .bind(&permission.created_at)
            .bind(&permission.updated_at)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| RepoError::QueryError(format!("Failed to create temporary permission: {}", e)))?;

        Ok(TemporaryPermission::from_row(&row)
            .map_err(|e| RepoError::SerializationError(format!("Failed to parse temporary permission: {}", e)))?)
    }

    async fn find_by_id(&self, id: &Uuid) -> Result<Option<TemporaryPermission>, RepoError> {
        let query = "SELECT * FROM temporary_permissions WHERE id = $1";
        
        let row = sqlx::query(query)
            .bind(id)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| RepoError::QueryError(format!("Failed to find temporary permission: {}", e)))?;

        match row {
            Some(row) => {
                let permission = TemporaryPermission::from_row(&row)
                    .map_err(|e| RepoError::SerializationError(format!("Failed to parse temporary permission: {}", e)))?;
                Ok(Some(permission))
            }
            None => Ok(None),
        }
    }

    async fn find_by_query(&self, query: &TemporaryPermissionQuery) -> Result<Vec<TemporaryPermission>, RepoError> {
        let mut sql = "SELECT * FROM temporary_permissions WHERE 1=1".to_string();
        let mut bind_count = 0;

        if let Some(_user_id) = &query.user_id {
            bind_count += 1;
            sql.push_str(&format!(" AND user_id = ${}", bind_count));
        }

        if let Some(_permission) = &query.permission {
            bind_count += 1;
            sql.push_str(&format!(" AND permission = ${}", bind_count));
        }

        if let Some(_resource) = &query.resource {
            bind_count += 1;
            sql.push_str(&format!(" AND resource = ${}", bind_count));
        }

        if let Some(_action) = &query.action {
            bind_count += 1;
            sql.push_str(&format!(" AND action = ${}", bind_count));
        }

        if let Some(_status) = &query.status {
            bind_count += 1;
            sql.push_str(&format!(" AND status = ${}", bind_count));
        }

        if query.active_only == Some(true) {
            sql.push_str(" AND status = 'active' AND expires_at > NOW()");
        }

        if let Some(_expires_before) = &query.expires_before {
            bind_count += 1;
            sql.push_str(&format!(" AND expires_at < ${}", bind_count));
        }

        if let Some(_expires_after) = &query.expires_after {
            bind_count += 1;
            sql.push_str(&format!(" AND expires_at > ${}", bind_count));
        }

        if let Some(_granted_by) = &query.granted_by {
            bind_count += 1;
            sql.push_str(&format!(" AND granted_by = ${}", bind_count));
        }

        sql.push_str(" ORDER BY created_at DESC");

        if let Some(_limit) = query.limit {
            bind_count += 1;
            sql.push_str(&format!(" LIMIT ${}", bind_count));
        }

        if let Some(_offset) = query.offset {
            bind_count += 1;
            sql.push_str(&format!(" OFFSET ${}", bind_count));
        }

        let mut query_builder = sqlx::query(&sql);

        if let Some(user_id) = &query.user_id {
            query_builder = query_builder.bind(&user_id.0);
        }
        if let Some(permission) = &query.permission {
            query_builder = query_builder.bind(permission);
        }
        if let Some(resource) = &query.resource {
            query_builder = query_builder.bind(resource);
        }
        if let Some(action) = &query.action {
            query_builder = query_builder.bind(action);
        }
        if let Some(status) = &query.status {
            query_builder = query_builder.bind(status);
        }
        if let Some(expires_before) = &query.expires_before {
            query_builder = query_builder.bind(expires_before);
        }
        if let Some(expires_after) = &query.expires_after {
            query_builder = query_builder.bind(expires_after);
        }
        if let Some(granted_by) = &query.granted_by {
            query_builder = query_builder.bind(&granted_by.0);
        }
        if let Some(limit) = query.limit {
            query_builder = query_builder.bind(limit);
        }
        if let Some(offset) = query.offset {
            query_builder = query_builder.bind(offset);
        }

        let rows = query_builder
            .fetch_all(&self.pool)
            .await
            .map_err(|e| RepoError::QueryError(format!("Failed to find temporary permissions: {}", e)))?;

        let mut permissions = Vec::new();
        for row in rows {
            let permission = TemporaryPermission::from_row(&row)
                .map_err(|e| RepoError::SerializationError(format!("Failed to parse temporary permission: {}", e)))?;
            permissions.push(permission);
        }

        Ok(permissions)
    }

    async fn find_active_for_user(&self, user_id: &UserId) -> Result<Vec<TemporaryPermission>, RepoError> {
        let query = TemporaryPermissionQuery {
            user_id: Some(user_id.clone()),
            active_only: Some(true),
            ..Default::default()
        };
        self.find_by_query(&query).await
    }

    async fn update(&self, permission: &TemporaryPermission) -> Result<TemporaryPermission, RepoError> {
        let query = r#"
            UPDATE temporary_permissions SET
                permission = $2,
                resource = $3,
                action = $4,
                granted_at = $5,
                expires_at = $6,
                auto_revoke = $7,
                granted_by = $8,
                reason = $9,
                conditions = $10,
                status = $11,
                revoked_at = $12,
                revoked_by = $13,
                revocation_reason = $14,
                updated_at = $15
            WHERE id = $1
            RETURNING *
        "#;

        let row = sqlx::query(query)
            .bind(&permission.id)
            .bind(&permission.permission)
            .bind(&permission.resource)
            .bind(&permission.action)
            .bind(&permission.granted_at)
            .bind(&permission.expires_at)
            .bind(&permission.auto_revoke)
            .bind(permission.granted_by())
            .bind(&permission.reason)
            .bind(&permission.conditions)
            .bind(&permission.status)
            .bind(&permission.revoked_at)
            .bind(permission.revoked_by())
            .bind(&permission.revocation_reason)
            .bind(&permission.updated_at)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| RepoError::QueryError(format!("Failed to update temporary permission: {}", e)))?;

        Ok(TemporaryPermission::from_row(&row)
            .map_err(|e| RepoError::SerializationError(format!("Failed to parse temporary permission: {}", e)))?)
    }

    async fn delete(&self, id: &Uuid) -> Result<bool, RepoError> {
        let query = "DELETE FROM temporary_permissions WHERE id = $1";
        
        let result = sqlx::query(query)
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(|e| RepoError::QueryError(format!("Failed to delete temporary permission: {}", e)))?;

        Ok(result.rows_affected() > 0)
    }

    async fn expire_permissions(&self, before: DateTime<Utc>) -> Result<u64, RepoError> {
        let query = r#"
            UPDATE temporary_permissions 
            SET status = 'expired', updated_at = NOW()
            WHERE status = 'active' 
                AND expires_at < $1 
                AND auto_revoke = true
        "#;

        let result = sqlx::query(query)
            .bind(before)
            .execute(&self.pool)
            .await
            .map_err(|e| RepoError::QueryError(format!("Failed to expire permissions: {}", e)))?;

        Ok(result.rows_affected())
    }

    async fn cleanup_expired(&self) -> Result<u64, RepoError> {
        self.expire_permissions(Utc::now()).await
    }

    async fn count_by_query(&self, query: &TemporaryPermissionQuery) -> Result<i64, RepoError> {
        let mut sql = "SELECT COUNT(*) FROM temporary_permissions WHERE 1=1".to_string();
        let mut bind_count = 0;

        if let Some(_user_id) = &query.user_id {
            bind_count += 1;
            sql.push_str(&format!(" AND user_id = ${}", bind_count));
        }

        if let Some(_permission) = &query.permission {
            bind_count += 1;
            sql.push_str(&format!(" AND permission = ${}", bind_count));
        }

        if let Some(_resource) = &query.resource {
            bind_count += 1;
            sql.push_str(&format!(" AND resource = ${}", bind_count));
        }

        if let Some(_action) = &query.action {
            bind_count += 1;
            sql.push_str(&format!(" AND action = ${}", bind_count));
        }

        if let Some(_status) = &query.status {
            bind_count += 1;
            sql.push_str(&format!(" AND status = ${}", bind_count));
        }

        if query.active_only == Some(true) {
            sql.push_str(" AND status = 'active' AND expires_at > NOW()");
        }

        if let Some(_expires_before) = &query.expires_before {
            bind_count += 1;
            sql.push_str(&format!(" AND expires_at < ${}", bind_count));
        }

        if let Some(_expires_after) = &query.expires_after {
            bind_count += 1;
            sql.push_str(&format!(" AND expires_at > ${}", bind_count));
        }

        if let Some(_granted_by) = &query.granted_by {
            bind_count += 1;
            sql.push_str(&format!(" AND granted_by = ${}", bind_count));
        }

        let mut query_builder = sqlx::query_scalar(&sql);

        if let Some(user_id) = &query.user_id {
            query_builder = query_builder.bind(&user_id.0);
        }
        if let Some(permission) = &query.permission {
            query_builder = query_builder.bind(permission);
        }
        if let Some(resource) = &query.resource {
            query_builder = query_builder.bind(resource);
        }
        if let Some(action) = &query.action {
            query_builder = query_builder.bind(action);
        }
        if let Some(status) = &query.status {
            query_builder = query_builder.bind(status);
        }
        if let Some(expires_before) = &query.expires_before {
            query_builder = query_builder.bind(expires_before);
        }
        if let Some(expires_after) = &query.expires_after {
            query_builder = query_builder.bind(expires_after);
        }
        if let Some(granted_by) = &query.granted_by {
            query_builder = query_builder.bind(&granted_by.0);
        }

        let count: i64 = query_builder
            .fetch_one(&self.pool)
            .await
            .map_err(|e| RepoError::QueryError(format!("Failed to count temporary permissions: {}", e)))?;

        Ok(count)
    }
}