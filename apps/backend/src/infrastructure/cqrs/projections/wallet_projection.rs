// WalletReadModelProjection
// Projects WalletUser events into read_model.wallet_details
//
// BIG-BANG: migrated to sqlx (real).

use crate::infrastructure::cqrs::projection::{Projection, ProjectionCheckpoint, ProjectionEvent};
use crate::prelude::*;
use async_trait::async_trait;
use chrono::Utc;
use sqlx::{PgPool, Postgres, Transaction};
use std::sync::Arc;
use uuid::Uuid;

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
        let row: Option<(i64, Option<String>, i64, i64, bool)> = sqlx::query_as(
            "SELECT last_processed_sequence, last_processed_event_id::text, \
                    events_processed_count, (last_processed_sequence)::bigint, is_healthy \
             FROM projection_checkpoints WHERE projection_name = $1",
        )
        .bind(self.projection_name())
        .fetch_optional(pool)
        .await
        .map_err(|e| AppError::database_error(format!("Failed to load checkpoint: {}", e)))?;

        Ok(row.map(
            |(seq, event_id_text, count, _seq_big, healthy)| ProjectionCheckpoint {
                projection_name: self.projection_name().to_string(),
                last_processed_event_id: event_id_text.and_then(|s| Uuid::parse_str(&s).ok()),
                last_processed_sequence: seq,
                events_processed_count: count,
                processed_at: Utc::now(),
                is_healthy: healthy,
            },
        ))
    }

    async fn save_checkpoint(
        &self,
        tx: &mut Transaction<'_, Postgres>,
        checkpoint: &ProjectionCheckpoint,
    ) -> AppResult<()> {
        sqlx::query(
            "INSERT INTO projection_checkpoints \
             (projection_name, last_processed_event_id, last_processed_sequence, events_processed_count, is_healthy, processed_at) \
             VALUES ($1, $2, $3, $4, $5, NOW()) \
             ON CONFLICT (projection_name) DO UPDATE SET \
                last_processed_event_id = EXCLUDED.last_processed_event_id, \
                last_processed_sequence = EXCLUDED.last_processed_sequence, \
                events_processed_count = EXCLUDED.events_processed_count, \
                is_healthy = EXCLUDED.is_healthy, \
                processed_at = NOW()",
        )
        .bind(checkpoint.projection_name.clone())
        .bind(checkpoint.last_processed_event_id)
        .bind(checkpoint.last_processed_sequence)
        .bind(checkpoint.events_processed_count)
        .bind(checkpoint.is_healthy)
        .execute(&mut **tx)
        .await
        .map_err(|e| AppError::database_error(format!("Failed to save checkpoint: {}", e)))?;
        Ok(())
    }

    async fn rebuild(&self, _pool: &PgPool) -> AppResult<()> {
        Err(AppError::internal_error(
            "WalletReadModelProjection rebuild not implemented".to_string(),
        ))
    }
}

// Internal handlers using sqlx::query directly

impl WalletReadModelProjection {
    async fn handle_wallet_created(
        &self,
        tx: &mut Transaction<'_, Postgres>,
        event: &ProjectionEvent,
    ) -> AppResult<()> {
        let wallet_address = event
            .event_payload
            .get("wallet_address")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
            .ok_or_else(|| AppError::internal_error("Missing wallet_address".to_string()))?;
        let tier = event
            .event_payload
            .get("tier")
            .and_then(|v| v.as_str())
            .unwrap_or("Bronze")
            .to_string();
        let is_active = event
            .event_payload
            .get("is_active")
            .and_then(|v| v.as_bool())
            .unwrap_or(true);

        sqlx::query(
            r#"
            INSERT INTO wallet_details (wallet_address, tier, is_active, permissions, created_at, updated_at)
            VALUES ($1, $2, $3, '{}'::jsonb, NOW(), NOW())
            ON CONFLICT (wallet_address) DO UPDATE SET
                tier = EXCLUDED.tier,
                is_active = EXCLUDED.is_active,
                updated_at = NOW()
            "#,
        )
        .bind(&wallet_address)
        .bind(&tier)
        .bind(is_active)
        .execute(&mut **tx)
        .await
        .map_err(|e| AppError::database_error(format!("Failed to insert wallet: {}", e)))?;

        Ok(())
    }

    async fn handle_wallet_activated(
        &self,
        tx: &mut Transaction<'_, Postgres>,
        event: &ProjectionEvent,
    ) -> AppResult<()> {
        let wallet_address = event
            .event_payload
            .get("wallet_address")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
            .ok_or_else(|| AppError::internal_error("Missing wallet_address".to_string()))?;
        sqlx::query(
            "UPDATE wallet_details SET is_active = TRUE, updated_at = NOW() WHERE wallet_address = $1",
        )
        .bind(&wallet_address)
        .execute(&mut **tx)
        .await
        .map_err(|e| AppError::database_error(format!("Failed to activate wallet: {}", e)))?;
        Ok(())
    }

    async fn handle_wallet_deactivated(
        &self,
        tx: &mut Transaction<'_, Postgres>,
        event: &ProjectionEvent,
    ) -> AppResult<()> {
        let wallet_address = event
            .event_payload
            .get("wallet_address")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
            .ok_or_else(|| AppError::internal_error("Missing wallet_address".to_string()))?;
        sqlx::query(
            "UPDATE wallet_details SET is_active = FALSE, updated_at = NOW() WHERE wallet_address = $1",
        )
        .bind(&wallet_address)
        .execute(&mut **tx)
        .await
        .map_err(|e| AppError::database_error(format!("Failed to deactivate wallet: {}", e)))?;
        Ok(())
    }

    async fn handle_permissions_updated(
        &self,
        tx: &mut Transaction<'_, Postgres>,
        event: &ProjectionEvent,
    ) -> AppResult<()> {
        let wallet_address = event
            .event_payload
            .get("wallet_address")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
            .ok_or_else(|| AppError::internal_error("Missing wallet_address".to_string()))?;
        let permissions = event
            .event_payload
            .get("permissions")
            .cloned()
            .unwrap_or(serde_json::json!([]));
        sqlx::query(
            "UPDATE wallet_details SET permissions = $1, updated_at = NOW() WHERE wallet_address = $2",
        )
        .bind(permissions)
        .bind(&wallet_address)
        .execute(&mut **tx)
        .await
        .map_err(|e| AppError::database_error(format!("Failed to update permissions: {}", e)))?;
        Ok(())
    }

    async fn handle_session_created(
        &self,
        tx: &mut Transaction<'_, Postgres>,
        event: &ProjectionEvent,
    ) -> AppResult<()> {
        let wallet_address = event
            .event_payload
            .get("wallet_address")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
            .ok_or_else(|| AppError::internal_error("Missing wallet_address".to_string()))?;
        let session_id = event
            .event_payload
            .get("session_id")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        let expires_at = event
            .event_payload
            .get("expires_at")
            .and_then(|v| v.as_str())
            .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
            .map(|dt| dt.with_timezone(&Utc))
            .unwrap_or_else(Utc::now);

        sqlx::query(
            "INSERT INTO sessions (id, wallet_address, expires_at, created_at, is_valid) \
             VALUES ($1, $2, $3, NOW(), TRUE) \
             ON CONFLICT (id) DO UPDATE SET expires_at = EXCLUDED.expires_at, is_valid = TRUE",
        )
        .bind(session_id)
        .bind(&wallet_address)
        .bind(expires_at)
        .execute(&mut **tx)
        .await
        .map_err(|e| AppError::database_error(format!("Failed to upsert session: {}", e)))?;
        Ok(())
    }

    async fn handle_session_invalidated(
        &self,
        tx: &mut Transaction<'_, Postgres>,
        event: &ProjectionEvent,
    ) -> AppResult<()> {
        let session_id = event
            .event_payload
            .get("session_id")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
            .ok_or_else(|| AppError::internal_error("Missing session_id".to_string()))?;
        sqlx::query("UPDATE sessions SET is_valid = FALSE WHERE id = $1")
            .bind(session_id)
            .execute(&mut **tx)
            .await
            .map_err(|e| {
                AppError::database_error(format!("Failed to invalidate session: {}", e))
            })?;
        Ok(())
    }
}
