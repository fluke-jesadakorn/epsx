// WalletReadModelProjection
// Projects WalletUser events into read_model.wallet_details

use crate::prelude::*;
use crate::infrastructure::cqrs::projection::{Projection, ProjectionEvent, ProjectionCheckpoint};
use sqlx::{PgPool, Postgres, Transaction};
use async_trait::async_trait;
use chrono::Utc;

pub struct WalletReadModelProjection {
    _pool: Arc<PgPool>,
}

impl WalletReadModelProjection {
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { _pool: pool }
    }
}

#[async_trait]
impl Projection for WalletReadModelProjection {
    fn projection_name(&self) -> &'static str {
        "WalletReadModel"
    }

    fn handles_event_types(&self) -> Vec<&'static str> {
        vec![
            "WalletUserCreated",
            "WalletUserActivated",
            "WalletUserDeactivated",
            "WalletPermissionsUpdated",
            "SessionCreated",
            "SessionInvalidated",
        ]
    }

    async fn project_event(
        &self,
        tx: &mut Transaction<'_, Postgres>,
        event: &ProjectionEvent,
    ) -> AppResult<()> {
        match event.event_type.as_str() {
            "WalletUserCreated" => self.handle_wallet_created(tx, event).await,
            "WalletUserActivated" => self.handle_wallet_activated(tx, event).await,
            "WalletUserDeactivated" => self.handle_wallet_deactivated(tx, event).await,
            "WalletPermissionsUpdated" => self.handle_permissions_updated(tx, event).await,
            "SessionCreated" => self.handle_session_created(tx, event).await,
            "SessionInvalidated" => self.handle_session_invalidated(tx, event).await,
            _ => {
                tracing::warn!("Unhandled event type: {}", event.event_type);
                Ok(())
            }
        }
    }

    async fn get_checkpoint(&self, pool: &PgPool) -> AppResult<Option<ProjectionCheckpoint>> {
        let result = sqlx::query!(
            r#"
            SELECT
                projection_name,
                last_processed_event_id,
                last_processed_sequence,
                events_processed_count,
                processed_at,
                is_healthy
            FROM read_model.projection_checkpoints
            WHERE projection_name = $1
            "#,
            self.projection_name()
        )
        .fetch_optional(pool)
        .await
        .map_err(|e| {
            AppError::database_error(format!("Failed to get checkpoint: {}", e))
        })?;

        Ok(result.map(|row| ProjectionCheckpoint {
            projection_name: row.projection_name,
            last_processed_event_id: row.last_processed_event_id,
            last_processed_sequence: row.last_processed_sequence,
            events_processed_count: row.events_processed_count,
            processed_at: row.processed_at,
            is_healthy: row.is_healthy,
        }))
    }

    async fn save_checkpoint(
        &self,
        tx: &mut Transaction<'_, Postgres>,
        checkpoint: &ProjectionCheckpoint,
    ) -> AppResult<()> {
        sqlx::query!(
            r#"
            INSERT INTO read_model.projection_checkpoints (
                projection_name,
                last_processed_event_id,
                last_processed_sequence,
                events_processed_count,
                processed_at,
                is_healthy
            ) VALUES ($1, $2, $3, $4, $5, $6)
            ON CONFLICT (projection_name) DO UPDATE SET
                last_processed_event_id = $2,
                last_processed_sequence = $3,
                events_processed_count = $4,
                processed_at = $5,
                is_healthy = $6
            "#,
            checkpoint.projection_name,
            checkpoint.last_processed_event_id,
            checkpoint.last_processed_sequence,
            checkpoint.events_processed_count,
            checkpoint.processed_at,
            checkpoint.is_healthy
        )
        .execute(&mut **tx)
        .await
        .map_err(|e| {
            AppError::database_error(format!("Failed to save checkpoint: {}", e))
        })?;

        Ok(())
    }
}

// Event handlers
impl WalletReadModelProjection {
    async fn handle_wallet_created(
        &self,
        tx: &mut Transaction<'_, Postgres>,
        event: &ProjectionEvent,
    ) -> AppResult<()> {
        let payload = &event.event_payload;

        // Extract data from event payload
        let wallet_address = payload["wallet_address"]
            .as_str()
            .ok_or_else(|| AppError::internal_error("Missing wallet_address".to_string()))?;

        let created_at = event.occurred_at;
        let permission_groups = payload["permission_groups"]
            .as_array()
            .map(|arr| serde_json::Value::Array(arr.clone()))
            .unwrap_or_else(|| serde_json::json!([]));

        // Insert into read model
        sqlx::query!(
            r#"
            INSERT INTO read_model.wallet_details (
                wallet_address,
                is_active,
                created_at,
                updated_at,
                permission_groups,
                projection_version,
                last_event_id,
                last_projected_at
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            ON CONFLICT (wallet_address) DO UPDATE SET
                updated_at = $4,
                projection_version = read_model.wallet_details.projection_version + 1,
                last_event_id = $7,
                last_projected_at = $8
            "#,
            wallet_address,
            true, // is_active
            created_at,
            Utc::now(),
            permission_groups,
            1i64, // projection_version
            event.event_id,
            Utc::now()
        )
        .execute(&mut **tx)
        .await
        .map_err(|e| {
            AppError::database_error(format!("Failed to project WalletUserCreated: {}", e))
        })?;

        tracing::debug!("Projected WalletUserCreated for {}", wallet_address);
        Ok(())
    }

    async fn handle_wallet_activated(
        &self,
        tx: &mut Transaction<'_, Postgres>,
        event: &ProjectionEvent,
    ) -> AppResult<()> {
        sqlx::query!(
            r#"
            UPDATE read_model.wallet_details
            SET
                is_active = true,
                updated_at = NOW(),
                projection_version = projection_version + 1,
                last_event_id = $1,
                last_projected_at = NOW()
            WHERE wallet_address = $2
            "#,
            event.event_id,
            event.aggregate_id
        )
        .execute(&mut **tx)
        .await
        .map_err(|e| {
            AppError::database_error(format!("Failed to project WalletUserActivated: {}", e))
        })?;

        tracing::debug!("Projected WalletUserActivated for {}", event.aggregate_id);
        Ok(())
    }

    async fn handle_wallet_deactivated(
        &self,
        tx: &mut Transaction<'_, Postgres>,
        event: &ProjectionEvent,
    ) -> AppResult<()> {
        sqlx::query!(
            r#"
            UPDATE read_model.wallet_details
            SET
                is_active = false,
                updated_at = NOW(),
                projection_version = projection_version + 1,
                last_event_id = $1,
                last_projected_at = NOW()
            WHERE wallet_address = $2
            "#,
            event.event_id,
            event.aggregate_id
        )
        .execute(&mut **tx)
        .await
        .map_err(|e| {
            AppError::database_error(format!("Failed to project WalletUserDeactivated: {}", e))
        })?;

        tracing::debug!("Projected WalletUserDeactivated for {}", event.aggregate_id);
        Ok(())
    }

    async fn handle_permissions_updated(
        &self,
        tx: &mut Transaction<'_, Postgres>,
        event: &ProjectionEvent,
    ) -> AppResult<()> {
        let payload = &event.event_payload;

        // Extract permissions from payload
        let active_permissions = payload["active_permissions"]
            .as_array()
            .map(|arr| serde_json::Value::Array(arr.clone()))
            .unwrap_or_else(|| serde_json::json!([]));

        let permission_groups = payload["permission_groups"]
            .as_array()
            .map(|arr| serde_json::Value::Array(arr.clone()))
            .unwrap_or_else(|| serde_json::json!([]));

        let total_permissions = active_permissions.as_array().map(|arr| arr.len() as i32).unwrap_or(0);

        sqlx::query!(
            r#"
            UPDATE read_model.wallet_details
            SET
                active_permissions = $1,
                permission_groups = $2,
                total_permissions = $3,
                updated_at = NOW(),
                projection_version = projection_version + 1,
                last_event_id = $4,
                last_projected_at = NOW()
            WHERE wallet_address = $5
            "#,
            active_permissions,
            permission_groups,
            total_permissions,
            event.event_id,
            event.aggregate_id
        )
        .execute(&mut **tx)
        .await
        .map_err(|e| {
            AppError::database_error(format!("Failed to project WalletPermissionsUpdated: {}", e))
        })?;

        tracing::debug!(
            "Projected WalletPermissionsUpdated for {} ({} permissions)",
            event.aggregate_id,
            total_permissions
        );
        Ok(())
    }

    async fn handle_session_created(
        &self,
        tx: &mut Transaction<'_, Postgres>,
        event: &ProjectionEvent,
    ) -> AppResult<()> {
        let payload = &event.event_payload;

        // Extract wallet_address from session event
        let wallet_address = payload["wallet_address"]
            .as_str()
            .ok_or_else(|| AppError::internal_error("Missing wallet_address in session event".to_string()))?;

        sqlx::query!(
            r#"
            UPDATE read_model.wallet_details
            SET
                total_sessions = total_sessions + 1,
                active_session_count = active_session_count + 1,
                total_logins = total_logins + 1,
                last_auth_at = $1,
                last_activity_at = $1,
                updated_at = NOW(),
                projection_version = projection_version + 1,
                last_event_id = $2,
                last_projected_at = NOW()
            WHERE wallet_address = $3
            "#,
            event.occurred_at,
            event.event_id,
            wallet_address
        )
        .execute(&mut **tx)
        .await
        .map_err(|e| {
            AppError::database_error(format!("Failed to project SessionCreated: {}", e))
        })?;

        tracing::debug!("Projected SessionCreated for {}", wallet_address);
        Ok(())
    }

    async fn handle_session_invalidated(
        &self,
        tx: &mut Transaction<'_, Postgres>,
        event: &ProjectionEvent,
    ) -> AppResult<()> {
        let payload = &event.event_payload;

        let wallet_address = payload["wallet_address"]
            .as_str()
            .ok_or_else(|| AppError::internal_error("Missing wallet_address in session event".to_string()))?;

        sqlx::query!(
            r#"
            UPDATE read_model.wallet_details
            SET
                active_session_count = GREATEST(active_session_count - 1, 0),
                updated_at = NOW(),
                projection_version = projection_version + 1,
                last_event_id = $1,
                last_projected_at = NOW()
            WHERE wallet_address = $2
            "#,
            event.event_id,
            wallet_address
        )
        .execute(&mut **tx)
        .await
        .map_err(|e| {
            AppError::database_error(format!("Failed to project SessionInvalidated: {}", e))
        })?;

        tracing::debug!("Projected SessionInvalidated for {}", wallet_address);
        Ok(())
    }
}
