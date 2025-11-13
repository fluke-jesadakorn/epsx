// WalletReadModelProjection
// Projects WalletUser events into read_model.wallet_details

use crate::prelude::*;
use crate::infrastructure::cqrs::projection::{Projection, ProjectionEvent, ProjectionCheckpoint};
use diesel::prelude::*;
use diesel_async::{AsyncPgConnection, RunQueryDsl, pooled_connection::deadpool::Pool};
use async_trait::async_trait;
use chrono::Utc;

pub struct WalletReadModelProjection {
    _pool: Arc<&'static Pool<AsyncPgConnection>>,
}

impl WalletReadModelProjection {
    pub fn new(pool: Arc<&'static Pool<AsyncPgConnection>>) -> Self {
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
        conn: &mut AsyncPgConnection,
        event: &ProjectionEvent,
    ) -> AppResult<()> {
        match event.event_type.as_str() {
            "WalletUserCreated" => self.handle_wallet_created(conn, event).await,
            "WalletUserActivated" => self.handle_wallet_activated(conn, event).await,
            "WalletUserDeactivated" => self.handle_wallet_deactivated(conn, event).await,
            "WalletPermissionsUpdated" => self.handle_permissions_updated(conn, event).await,
            "SessionCreated" => self.handle_session_created(conn, event).await,
            "SessionInvalidated" => self.handle_session_invalidated(conn, event).await,
            _ => {
                tracing::warn!("Unhandled event type: {}", event.event_type);
                Ok(())
            }
        }
    }

    async fn get_checkpoint(&self, pool: &Pool<AsyncPgConnection>) -> AppResult<Option<ProjectionCheckpoint>> {
        let mut conn = pool.get().await.map_err(|e| {
            AppError::database_error(format!("Failed to get connection: {}", e))
        })?;

        #[derive(QueryableByName)]
        struct CheckpointRow {
            #[diesel(sql_type = diesel::sql_types::Text)]
            projection_name: String,
            #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Uuid>)]
            last_processed_event_id: Option<uuid::Uuid>,
            #[diesel(sql_type = diesel::sql_types::BigInt)]
            last_processed_sequence: i64,
            #[diesel(sql_type = diesel::sql_types::BigInt)]
            events_processed_count: i64,
            #[diesel(sql_type = diesel::sql_types::Timestamptz)]
            processed_at: chrono::DateTime<chrono::Utc>,
            #[diesel(sql_type = diesel::sql_types::Bool)]
            is_healthy: bool,
        }

        let result = diesel::sql_query(
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
            "#
        )
        .bind::<diesel::sql_types::Text, _>(self.projection_name())
        .get_result::<CheckpointRow>(&mut conn)
        .await
        .optional()
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
        conn: &mut AsyncPgConnection,
        checkpoint: &ProjectionCheckpoint,
    ) -> AppResult<()> {
        diesel::sql_query(
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
            "#
        )
        .bind::<diesel::sql_types::Text, _>(&checkpoint.projection_name)
        .bind::<diesel::sql_types::Nullable<diesel::sql_types::Uuid>, _>(checkpoint.last_processed_event_id)
        .bind::<diesel::sql_types::BigInt, _>(checkpoint.last_processed_sequence)
        .bind::<diesel::sql_types::BigInt, _>(checkpoint.events_processed_count)
        .bind::<diesel::sql_types::Timestamptz, _>(checkpoint.processed_at)
        .bind::<diesel::sql_types::Bool, _>(checkpoint.is_healthy)
        .execute(conn)
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
        conn: &mut AsyncPgConnection,
        event: &ProjectionEvent,
    ) -> AppResult<()> {
        let payload = &event.event_payload;

        // Extract data from event payload
        let wallet_address = payload["wallet_address"]
            .as_str()
            .ok_or_else(|| AppError::internal_error("Missing wallet_address".to_string()))?;

        let created_at = event.occurred_at;
        let now = Utc::now();

        // Insert into read model (schema optimized - removed denormalized columns)
        diesel::sql_query(
            r#"
            INSERT INTO read_model.wallet_details (
                wallet_address,
                is_active,
                created_at,
                updated_at,
                projection_version,
                last_event_id,
                last_projected_at
            ) VALUES ($1, $2, $3, $4, $5, $6, $7)
            ON CONFLICT (wallet_address) DO UPDATE SET
                updated_at = $4,
                projection_version = read_model.wallet_details.projection_version + 1,
                last_event_id = $6,
                last_projected_at = $7
            "#
        )
        .bind::<diesel::sql_types::Text, _>(wallet_address)
        .bind::<diesel::sql_types::Bool, _>(true)
        .bind::<diesel::sql_types::Timestamptz, _>(created_at)
        .bind::<diesel::sql_types::Timestamptz, _>(now)
        .bind::<diesel::sql_types::BigInt, _>(1i64)
        .bind::<diesel::sql_types::Uuid, _>(event.event_id)
        .bind::<diesel::sql_types::Timestamptz, _>(now)
        .execute(conn)
        .await
        .map_err(|e| {
            AppError::database_error(format!("Failed to project WalletUserCreated: {}", e))
        })?;

        // Update wallet details with current stats (calls update_wallet_details function)
        diesel::sql_query(r#"SELECT update_wallet_details($1) as updated"#)
            .bind::<diesel::sql_types::Text, _>(wallet_address)
            .execute(conn)
            .await
            .map_err(|e| {
                AppError::database_error(format!("Failed to update wallet details: {}", e))
            })?;

        tracing::debug!("Projected WalletUserCreated for {}", wallet_address);
        Ok(())
    }

    async fn handle_wallet_activated(
        &self,
        conn: &mut AsyncPgConnection,
        event: &ProjectionEvent,
    ) -> AppResult<()> {
        diesel::sql_query(
            r#"
            UPDATE read_model.wallet_details
            SET
                is_active = true,
                updated_at = NOW(),
                projection_version = projection_version + 1,
                last_event_id = $1,
                last_projected_at = NOW()
            WHERE wallet_address = $2
            "#
        )
        .bind::<diesel::sql_types::Uuid, _>(event.event_id)
        .bind::<diesel::sql_types::Text, _>(&event.aggregate_id)
        .execute(conn)
        .await
        .map_err(|e| {
            AppError::database_error(format!("Failed to project WalletUserActivated: {}", e))
        })?;

        tracing::debug!("Projected WalletUserActivated for {}", event.aggregate_id);
        Ok(())
    }

    async fn handle_wallet_deactivated(
        &self,
        conn: &mut AsyncPgConnection,
        event: &ProjectionEvent,
    ) -> AppResult<()> {
        diesel::sql_query(
            r#"
            UPDATE read_model.wallet_details
            SET
                is_active = false,
                updated_at = NOW(),
                projection_version = projection_version + 1,
                last_event_id = $1,
                last_projected_at = NOW()
            WHERE wallet_address = $2
            "#
        )
        .bind::<diesel::sql_types::Uuid, _>(event.event_id)
        .bind::<diesel::sql_types::Text, _>(&event.aggregate_id)
        .execute(conn)
        .await
        .map_err(|e| {
            AppError::database_error(format!("Failed to project WalletUserDeactivated: {}", e))
        })?;

        tracing::debug!("Projected WalletUserDeactivated for {}", event.aggregate_id);
        Ok(())
    }

    async fn handle_permissions_updated(
        &self,
        conn: &mut AsyncPgConnection,
        event: &ProjectionEvent,
    ) -> AppResult<()> {
        // Schema optimized - permissions now tracked in user_effective_permissions table
        // and computed dynamically via triggers. Just update metadata.
        diesel::sql_query(
            r#"
            UPDATE read_model.wallet_details
            SET
                permissions_last_updated = NOW(),
                groups_last_updated = NOW(),
                updated_at = NOW(),
                projection_version = projection_version + 1,
                last_event_id = $1,
                last_projected_at = NOW()
            WHERE wallet_address = $2
            "#
        )
        .bind::<diesel::sql_types::Uuid, _>(event.event_id)
        .bind::<diesel::sql_types::Text, _>(&event.aggregate_id)
        .execute(conn)
        .await
        .map_err(|e| {
            AppError::database_error(format!("Failed to project WalletPermissionsUpdated: {}", e))
        })?;

        // Update computed stats (calls update_wallet_details function)
        diesel::sql_query(r#"SELECT update_wallet_details($1) as updated"#)
            .bind::<diesel::sql_types::Text, _>(&event.aggregate_id)
            .execute(conn)
            .await
            .map_err(|e| {
                AppError::database_error(format!("Failed to update wallet details: {}", e))
            })?;

        tracing::debug!(
            "Projected WalletPermissionsUpdated for {}",
            event.aggregate_id
        );
        Ok(())
    }

    async fn handle_session_created(
        &self,
        conn: &mut AsyncPgConnection,
        event: &ProjectionEvent,
    ) -> AppResult<()> {
        let payload = &event.event_payload;

        // Extract wallet_address from session event
        let wallet_address = payload["wallet_address"]
            .as_str()
            .ok_or_else(|| AppError::internal_error("Missing wallet_address in session event".to_string()))?;

        diesel::sql_query(
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
            "#
        )
        .bind::<diesel::sql_types::Timestamptz, _>(event.occurred_at)
        .bind::<diesel::sql_types::Uuid, _>(event.event_id)
        .bind::<diesel::sql_types::Text, _>(wallet_address)
        .execute(conn)
        .await
        .map_err(|e| {
            AppError::database_error(format!("Failed to project SessionCreated: {}", e))
        })?;

        tracing::debug!("Projected SessionCreated for {}", wallet_address);
        Ok(())
    }

    async fn handle_session_invalidated(
        &self,
        conn: &mut AsyncPgConnection,
        event: &ProjectionEvent,
    ) -> AppResult<()> {
        let payload = &event.event_payload;

        let wallet_address = payload["wallet_address"]
            .as_str()
            .ok_or_else(|| AppError::internal_error("Missing wallet_address in session event".to_string()))?;

        diesel::sql_query(
            r#"
            UPDATE read_model.wallet_details
            SET
                active_session_count = GREATEST(active_session_count - 1, 0),
                updated_at = NOW(),
                projection_version = projection_version + 1,
                last_event_id = $1,
                last_projected_at = NOW()
            WHERE wallet_address = $2
            "#
        )
        .bind::<diesel::sql_types::Uuid, _>(event.event_id)
        .bind::<diesel::sql_types::Text, _>(wallet_address)
        .execute(conn)
        .await
        .map_err(|e| {
            AppError::database_error(format!("Failed to project SessionInvalidated: {}", e))
        })?;

        tracing::debug!("Projected SessionInvalidated for {}", wallet_address);
        Ok(())
    }
}
