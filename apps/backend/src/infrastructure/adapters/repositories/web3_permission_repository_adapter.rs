// Web3 Permission Repository Adapter (Infrastructure Layer)
// Implements the Web3PermissionRepositoryPort using SQLx and PostgreSQL

use std::{sync::Arc, collections::HashMap};
use anyhow::Result;
use sqlx::PgPool;
use tracing::{error, info};

use crate::domain::{
    shared_kernel::{domain_error::DomainError, value_objects::UserId},
    authentication::{
        Web3Permission, Web3PermissionType, Web3PermissionRepositoryPort,
    },
};

/// PostgreSQL implementation of Web3PermissionRepositoryPort
pub struct Web3PermissionRepositoryAdapter {
    db_pool: Arc<PgPool>,
}

impl Web3PermissionRepositoryAdapter {
    pub fn new(db_pool: Arc<PgPool>) -> Self {
        Self { db_pool }
    }

    // Helper method to serialize permission type to JSON
    fn serialize_permission_type(permission_type: &Web3PermissionType) -> Result<serde_json::Value, DomainError> {
        serde_json::to_value(permission_type)
            .map_err(|e| DomainError::InfrastructureError(format!("Serialization error: {}", e)))
    }

    // Helper method to deserialize permission type from JSON
    fn deserialize_permission_type(value: &serde_json::Value) -> Result<Web3PermissionType, DomainError> {
        serde_json::from_value(value.clone())
            .map_err(|e| DomainError::InfrastructureError(format!("Deserialization error: {}", e)))
    }

    // Helper method to serialize metadata to JSON
    fn serialize_metadata(metadata: &HashMap<String, String>) -> Result<serde_json::Value, DomainError> {
        serde_json::to_value(metadata)
            .map_err(|e| DomainError::InfrastructureError(format!("Metadata serialization error: {}", e)))
    }

    // Helper method to deserialize metadata from JSON
    fn deserialize_metadata(value: &serde_json::Value) -> Result<HashMap<String, String>, DomainError> {
        serde_json::from_value(value.clone())
            .map_err(|e| DomainError::InfrastructureError(format!("Metadata deserialization error: {}", e)))
    }
}

#[async_trait::async_trait]
impl Web3PermissionRepositoryPort for Web3PermissionRepositoryAdapter {
    async fn get_user_permission(&self, user_id: &UserId, permission_name: &str) -> Result<Option<Web3Permission>, DomainError> {
        let permission = sqlx::query!(
            r#"
            SELECT permission_name, permission_type, wallet_address, granted_at, 
                   expires_at, is_active, last_verified_at, verification_metadata
            FROM web3_permissions
            WHERE user_id = $1 AND permission_name = $2
            "#,
            user_id.value(),
            permission_name
        )
        .fetch_optional(&**self.db_pool)
        .await
        .map_err(|e| {
            error!("Failed to get user permission: {}", e);
            DomainError::InfrastructureError(format!("Database error: {}", e))
        })?;

        if let Some(row) = permission {
            let permission_type = Self::deserialize_permission_type(&row.permission_type)?;
            let metadata = Self::deserialize_metadata(&row.verification_metadata)?;

            Ok(Some(Web3Permission {
                permission_name: row.permission_name,
                permission_type,
                user_id: *user_id,
                wallet_address: row.wallet_address,
                granted_at: row.granted_at,
                expires_at: row.expires_at,
                is_active: row.is_active,
                last_verified_at: row.last_verified_at,
                verification_metadata: metadata,
            }))
        } else {
            Ok(None)
        }
    }

    async fn get_user_permissions(&self, user_id: &UserId) -> Result<Vec<Web3Permission>, DomainError> {
        let permissions = sqlx::query!(
            r#"
            SELECT permission_name, permission_type, wallet_address, granted_at, 
                   expires_at, is_active, last_verified_at, verification_metadata
            FROM web3_permissions
            WHERE user_id = $1
            ORDER BY granted_at DESC
            "#,
            user_id.value()
        )
        .fetch_all(&**self.db_pool)
        .await
        .map_err(|e| {
            error!("Failed to get user permissions: {}", e);
            DomainError::InfrastructureError(format!("Database error: {}", e))
        })?;

        let mut result = Vec::new();
        for row in permissions {
            let permission_type = Self::deserialize_permission_type(&row.permission_type)?;
            let metadata = Self::deserialize_metadata(&row.verification_metadata)?;

            result.push(Web3Permission {
                permission_name: row.permission_name,
                permission_type,
                user_id: *user_id,
                wallet_address: row.wallet_address,
                granted_at: row.granted_at,
                expires_at: row.expires_at,
                is_active: row.is_active,
                last_verified_at: row.last_verified_at,
                verification_metadata: metadata,
            });
        }

        Ok(result)
    }

    async fn get_permission_config(&self, permission_name: &str) -> Result<Option<Web3PermissionType>, DomainError> {
        let config = sqlx::query!(
            r#"
            SELECT permission_type
            FROM web3_permission_configs
            WHERE permission_name = $1
            "#,
            permission_name
        )
        .fetch_optional(&**self.db_pool)
        .await
        .map_err(|e| {
            error!("Failed to get permission config: {}", e);
            DomainError::InfrastructureError(format!("Database error: {}", e))
        })?;

        if let Some(row) = config {
            let permission_type = Self::deserialize_permission_type(&row.permission_type)?;
            Ok(Some(permission_type))
        } else {
            Ok(None)
        }
    }

    async fn store_permission(&self, permission: &Web3Permission) -> Result<(), DomainError> {
        let permission_type_json = Self::serialize_permission_type(&permission.permission_type)?;
        let metadata_json = Self::serialize_metadata(&permission.verification_metadata)?;

        sqlx::query!(
            r#"
            INSERT INTO web3_permissions (
                user_id, permission_name, permission_type, wallet_address,
                granted_at, expires_at, is_active, last_verified_at, verification_metadata
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            ON CONFLICT (user_id, permission_name) DO UPDATE SET
                permission_type = EXCLUDED.permission_type,
                wallet_address = EXCLUDED.wallet_address,
                granted_at = EXCLUDED.granted_at,
                expires_at = EXCLUDED.expires_at,
                is_active = EXCLUDED.is_active,
                last_verified_at = EXCLUDED.last_verified_at,
                verification_metadata = EXCLUDED.verification_metadata
            "#,
            permission.user_id.value(),
            permission.permission_name,
            permission_type_json,
            permission.wallet_address,
            permission.granted_at,
            permission.expires_at,
            permission.is_active,
            permission.last_verified_at,
            metadata_json
        )
        .execute(&**self.db_pool)
        .await
        .map_err(|e| {
            error!("Failed to store permission: {}", e);
            DomainError::InfrastructureError(format!("Database error: {}", e))
        })?;

        info!("Stored Web3 permission {} for user {}", permission.permission_name, permission.user_id.value());
        Ok(())
    }

    async fn revoke_permission(&self, user_id: &UserId, permission_name: &str) -> Result<(), DomainError> {
        let result = sqlx::query!(
            r#"
            UPDATE web3_permissions
            SET is_active = false
            WHERE user_id = $1 AND permission_name = $2
            "#,
            user_id.value(),
            permission_name
        )
        .execute(&**self.db_pool)
        .await
        .map_err(|e| {
            error!("Failed to revoke permission: {}", e);
            DomainError::InfrastructureError(format!("Database error: {}", e))
        })?;

        if result.rows_affected() == 0 {
            return Err(DomainError::NotFound("Permission not found".to_string()));
        }

        info!("Revoked permission {} for user {}", permission_name, user_id.value());
        Ok(())
    }
}