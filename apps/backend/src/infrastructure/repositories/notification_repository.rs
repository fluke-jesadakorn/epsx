use async_trait::async_trait;
// use diesel::prelude::*;
// use diesel_async::RunQueryDsl;
use std::sync::Arc;
use uuid::Uuid;

use super::notification_record::NotificationRecord;
use crate::domain::notification::*;
use crate::prelude::*;
use epsx_contracts::errors::{AppError, ErrorKind};

pub struct NotificationRepository {
    pool: Arc<TlsPool>,
}

impl NotificationRepository {
    pub fn new(pool: Arc<TlsPool>) -> Self {
        Self { pool }
    }

    fn diesel_row_to_record(row: NotificationQueryRow) -> Result<NotificationRecord, AppError> {
        Ok(NotificationRecord {
            id: row.id,
            recipient_wallet_address: row.recipient_wallet_address.clone(),
            topic_name: row.topic_name,
            title: row.title,
            body: row.body,
            urgency: row.urgency,
            notification_type: row.notification_type,
            priority: row.priority,
            channels: row.channels,
            schedule_type: row.schedule_type,
            scheduled_at: row.scheduled_at,
            expires_at: row.expires_at,
            status: row.status,
            send_started_at: row.send_started_at,
            channel_status: row.channel_status,
            total_attempts: row.total_attempts,
            created_by: row.created_by,
            image_url: row.image_url,
            action_url: row.action_url,
            data_payload: row.data_payload,
            tags: row.tags.unwrap_or_default(),
            notes: row.notes.map(|n| vec![n]).unwrap_or_default(),
            version: row.version as i64,
            created_at: row.created_at,
            updated_at: row.updated_at,
        })
    }
}

// Diesel row struct for notification queries
#[derive(sqlx::FromRow)]
struct NotificationQueryRow {
    id: Uuid,
    recipient_wallet_address: Option<String>,
    topic_name: Option<String>,
    title: String,
    body: String,
    urgency: String,
    notification_type: String,
    priority: String,
    channels: serde_json::Value,
    schedule_type: String,
    scheduled_at: Option<chrono::DateTime<chrono::Utc>>,
    expires_at: Option<chrono::DateTime<chrono::Utc>>,
    status: String,
    send_started_at: Option<chrono::DateTime<chrono::Utc>>,
    channel_status: serde_json::Value,
    total_attempts: i32,
    created_by: Option<String>,
    image_url: Option<String>,
    action_url: Option<String>,
    data_payload: Option<serde_json::Value>,
    tags: Option<Vec<String>>,
    notes: Option<String>,
    version: i32,
    created_at: chrono::DateTime<chrono::Utc>,
    updated_at: chrono::DateTime<chrono::Utc>,
}

#[async_trait]
#[async_trait]
impl NotificationRepositoryPort for NotificationRepository {
    async fn find_by_id(&self, notification_id: &str) -> AppResult<Option<Notification>> {
        let id = Uuid::parse_str(notification_id).map_err(|e| {
            AppError::new(
                ErrorKind::ValidationError,
                format!("Invalid notification ID: {}", e),
            )
        })?;

        let mut conn = self.pool.acquire().await.map_err(|e| {
            AppError::new(
                ErrorKind::DatabaseError,
                format!("Failed to get connection: {}", e),
            )
        })?;

        let row: Option<NotificationQueryRow> = sqlx::query_as::<_, NotificationQueryRow>(
            r#"
            SELECT id, recipient_wallet_address, topic_name, title, body, urgency,
                   notification_type, priority, channels, schedule_type, scheduled_at, expires_at,
                   status, send_started_at, channel_status, total_attempts,
                   created_by, image_url, action_url, data_payload, tags, notes,
                   version, created_at, updated_at
            FROM wallet_notifications WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(&mut *conn)
        .await
        .map_err(|e| AppError::new(ErrorKind::DatabaseError, format!("Database error: {}", e)))?;

        match row {
            Some(r) => {
                let record = Self::diesel_row_to_record(r)?;
                Ok(Some(
                    record
                        .to_domain()
                        .map_err(|e| AppError::new(ErrorKind::InternalError, e))?,
                ))
            }
            None => Ok(None),
        }
    }

    async fn find_all(&self, criteria: NotificationSearchCriteria) -> AppResult<Vec<Notification>> {
        let mut conn = self.pool.acquire().await.map_err(|e| {
            AppError::new(
                ErrorKind::DatabaseError,
                format!("Failed to get connection: {}", e),
            )
        })?;

        let rows: Vec<NotificationQueryRow> = sqlx::query_as::<_, NotificationQueryRow>(
            r#"
            SELECT id, recipient_wallet_address, topic_name, title, body, urgency,
                   notification_type, priority, channels, schedule_type, scheduled_at, expires_at,
                   status, send_started_at, channel_status, total_attempts,
                   created_by, image_url, action_url, data_payload, tags, notes,
                   version, created_at, updated_at
            FROM wallet_notifications
            WHERE ($1::text IS NULL OR recipient_wallet_address = $1)
              AND ($2::text IS NULL OR topic_name = $2)
              AND ($3::text IS NULL OR status = $3)
              AND ($4::text IS NULL OR notification_type = $4)
              AND ($5::text IS NULL OR priority = $5)
              AND ($6::timestamptz IS NULL OR created_at >= $6)
              AND ($7::timestamptz IS NULL OR created_at <= $7)
            ORDER BY created_at DESC
            LIMIT $8 OFFSET $9
            "#,
        )
        .bind(criteria.recipient_wallet_address.as_deref())
        .bind(criteria.topic.as_deref())
        .bind(criteria.status.as_ref().map(|s| s.as_str()))
        .bind(criteria.notification_type.as_ref().map(|t| t.as_str()))
        .bind(criteria.priority.as_deref())
        .bind(criteria.created_after)
        .bind(criteria.created_before)
        .bind(criteria.limit.unwrap_or(100))
        .bind(criteria.offset.unwrap_or(0))
        .fetch_all(&mut *conn)
        .await
        .map_err(|e| AppError::new(ErrorKind::DatabaseError, format!("Database error: {}", e)))?;

        let mut notifications = Vec::new();
        for row in rows {
            let record = Self::diesel_row_to_record(row)?;
            notifications.push(
                record
                    .to_domain()
                    .map_err(|e| AppError::new(ErrorKind::InternalError, e))?,
            );
        }
        Ok(notifications)
    }

    async fn save(&self, notification: &Notification) -> AppResult<()> {
        let record = NotificationRecord::from_domain(notification);

        let mut conn = self.pool.acquire().await.map_err(|e| {
            AppError::new(
                ErrorKind::DatabaseError,
                format!("Failed to get connection: {}", e),
            )
        })?;

        sqlx::query(
            r#"
            INSERT INTO wallet_notifications (
                id, recipient_wallet_address, topic_name, title, body, urgency,
                notification_type, priority, channels, schedule_type, scheduled_at, expires_at,
                status, send_started_at, channel_status, total_attempts,
                created_by, image_url, action_url, data_payload, tags, notes,
                version, created_at, updated_at
            ) VALUES (
                $1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16,
                $17, $18, $19, $20, $21, $22, $23, $24, $25
            )
            ON CONFLICT (id) DO UPDATE SET
                recipient_wallet_address = EXCLUDED.recipient_wallet_address,
                topic_name = EXCLUDED.topic_name,
                title = EXCLUDED.title,
                body = EXCLUDED.body,
                urgency = EXCLUDED.urgency,
                notification_type = EXCLUDED.notification_type,
                priority = EXCLUDED.priority,
                channels = EXCLUDED.channels,
                schedule_type = EXCLUDED.schedule_type,
                scheduled_at = EXCLUDED.scheduled_at,
                expires_at = EXCLUDED.expires_at,
                status = EXCLUDED.status,
                send_started_at = EXCLUDED.send_started_at,
                channel_status = EXCLUDED.channel_status,
                total_attempts = EXCLUDED.total_attempts,
                image_url = EXCLUDED.image_url,
                action_url = EXCLUDED.action_url,
                data_payload = EXCLUDED.data_payload,
                tags = EXCLUDED.tags,
                notes = EXCLUDED.notes,
                version = EXCLUDED.version,
                updated_at = EXCLUDED.updated_at
            "#,
        )
        .bind(record.id)
        .bind(
            record.recipient_wallet_address.as_deref(),
        )
        .bind(
            record.topic_name.as_deref(),
        )
        .bind(&record.title)
        .bind(&record.body)
        .bind(&record.urgency)
        .bind(&record.notification_type)
        .bind(&record.priority)
        .bind(&record.channels)
        .bind(&record.schedule_type)
        .bind(record.scheduled_at)
        .bind(record.expires_at)
        .bind(&record.status)
        .bind(
            record.send_started_at,
        )
        .bind(&record.channel_status)
        .bind(record.total_attempts)
        .bind(
            record.created_by.as_deref(),
        )
        .bind(
            record.image_url.as_deref(),
        )
        .bind(
            record.action_url.as_deref(),
        )
        .bind(&record.data_payload)
        .bind(
            if record.tags.is_empty() {
                None
            } else {
                Some(&record.tags)
            },
        )
        .bind(
            if record.notes.is_empty() {
                None
            } else {
                Some(serde_json::to_string(&record.notes).unwrap_or_default())
            }
            .as_deref(),
        )
        .bind(record.version)
        .bind(record.created_at)
        .bind(record.updated_at)
        .execute(&mut *conn)
        .await
        .map_err(|e| {
            AppError::new(
                ErrorKind::DatabaseError,
                format!("Failed to save notification: {}", e),
            )
        })?;

        Ok(())
    }

    async fn delete(&self, notification_id: &str) -> AppResult<()> {
        let id = Uuid::parse_str(notification_id).map_err(|e| {
            AppError::new(
                ErrorKind::ValidationError,
                format!("Invalid notification ID: {}", e),
            )
        })?;

        let mut conn = self.pool.acquire().await.map_err(|e| {
            AppError::new(
                ErrorKind::DatabaseError,
                format!("Failed to get connection: {}", e),
            )
        })?;

        sqlx::query("DELETE FROM wallet_notifications WHERE id = $1")
            .bind(id)
            .execute(&mut *conn)
            .await
            .map_err(|e| {
                AppError::new(
                    ErrorKind::DatabaseError,
                    format!("Failed to delete notification: {}", e),
                )
            })?;

        Ok(())
    }

    async fn count(&self, criteria: NotificationSearchCriteria) -> AppResult<i64> {
        let mut conn = self.pool.acquire().await.map_err(|e| {
            AppError::new(
                ErrorKind::DatabaseError,
                format!("Failed to get connection: {}", e),
            )
        })?;

        #[derive(sqlx::FromRow)]
        struct CountRow {
            count: i64,
        }

        let row: CountRow = sqlx::query_as::<_, CountRow>(
            r#"
            SELECT COUNT(*) as count
            FROM wallet_notifications
            WHERE ($1::text IS NULL OR recipient_wallet_address = $1)
              AND ($2::text IS NULL OR topic_name = $2)
              AND ($3::text IS NULL OR status = $3)
              AND ($4::text IS NULL OR notification_type = $4)
            "#,
        )
        .bind(
            criteria.recipient_wallet_address.as_deref(),
        )
        .bind(criteria.topic.as_deref())
        .bind(
            criteria.status.as_ref().map(|s| s.as_str()),
        )
        .bind(
            criteria.notification_type.as_ref().map(|t| t.as_str()),
        )
        .fetch_one(&mut *conn)
        .await
        .map_err(|e| {
            AppError::new(
                ErrorKind::DatabaseError,
                format!("Failed to count notifications: {}", e),
            )
        })?;

        Ok(row.count)
    }

    async fn notification_exists(&self, notification_id: &str) -> AppResult<bool> {
        let id = Uuid::parse_str(notification_id).map_err(|e| {
            AppError::new(
                ErrorKind::ValidationError,
                format!("Invalid notification ID: {}", e),
            )
        })?;

        let mut conn = self.pool.acquire().await.map_err(|e| {
            AppError::new(
                ErrorKind::DatabaseError,
                format!("Failed to get connection: {}", e),
            )
        })?;

        #[derive(sqlx::FromRow)]
        struct ExistsRow {
            exists: bool,
        }

        let row: ExistsRow = sqlx::query_as::<_, ExistsRow>(
            "SELECT EXISTS(SELECT 1 FROM wallet_notifications WHERE id = $1) as exists",
        )
        .bind(id)
        .fetch_one(&mut *conn)
        .await
        .map_err(|e| AppError::new(ErrorKind::DatabaseError, format!("Database error: {}", e)))?;

        Ok(row.exists)
    }

    async fn find_pending(&self, limit: u32) -> AppResult<Vec<Notification>> {
        let mut conn = self.pool.acquire().await.map_err(|e| {
            AppError::new(
                ErrorKind::DatabaseError,
                format!("Failed to get connection: {}", e),
            )
        })?;

        let rows: Vec<NotificationQueryRow> = sqlx::query_as::<_, NotificationQueryRow>(
            r#"
            SELECT id, recipient_wallet_address, topic_name, title, body, urgency,
                   notification_type, priority, channels, schedule_type, scheduled_at, expires_at,
                   status, send_started_at, channel_status, total_attempts,
                   created_by, image_url, action_url, data_payload, tags, notes,
                   version, created_at, updated_at
            FROM wallet_notifications
            WHERE status IN ('scheduled', 'queued', 'created')
              AND (scheduled_at IS NULL OR scheduled_at <= NOW())
              AND (expires_at IS NULL OR expires_at > NOW())
            ORDER BY priority DESC, created_at ASC
            LIMIT $1
            "#,
        )
        .bind(limit as i64)
        .fetch_all(&mut *conn)
        .await
        .map_err(|e| AppError::new(ErrorKind::DatabaseError, format!("Database error: {}", e)))?;

        let mut notifications = Vec::new();
        for row in rows {
            let record = Self::diesel_row_to_record(row)?;
            notifications.push(
                record
                    .to_domain()
                    .map_err(|e| AppError::new(ErrorKind::InternalError, e))?,
            );
        }
        Ok(notifications)
    }

    async fn find_by_wallet(&self, wallet_address: String) -> AppResult<Vec<Notification>> {
        let criteria = NotificationSearchCriteria {
            recipient_wallet_address: Some(wallet_address),
            ..Default::default()
        };
        self.find_all(criteria).await
    }

    async fn find_by_topic(&self, topic: &str) -> AppResult<Vec<Notification>> {
        let criteria = NotificationSearchCriteria {
            topic: Some(topic.to_string()),
            ..Default::default()
        };
        self.find_all(criteria).await
    }

    async fn find_by_status(&self, status: NotificationStatus) -> AppResult<Vec<Notification>> {
        let criteria = NotificationSearchCriteria {
            status: Some(status),
            ..Default::default()
        };
        self.find_all(criteria).await
    }

    async fn find_expired(&self) -> AppResult<Vec<Notification>> {
        let mut conn = self.pool.acquire().await.map_err(|e| {
            AppError::new(
                ErrorKind::DatabaseError,
                format!("Failed to get connection: {}", e),
            )
        })?;

        let rows: Vec<NotificationQueryRow> = sqlx::query_as::<_, NotificationQueryRow>(
            r#"
            SELECT id, recipient_wallet_address, topic_name, title, body, urgency,
                   notification_type, priority, channels, schedule_type, scheduled_at, expires_at,
                   status, send_started_at, channel_status, total_attempts,
                   created_by, image_url, action_url, data_payload, tags, notes,
                   version, created_at, updated_at
            FROM wallet_notifications
            WHERE expires_at IS NOT NULL
              AND expires_at <= NOW()
              AND status NOT IN ('expired', 'delivered', 'cancelled')
            "#,
        )
        .fetch_all(&mut *conn)
        .await
        .map_err(|e| AppError::new(ErrorKind::DatabaseError, format!("Database error: {}", e)))?;

        let mut notifications = Vec::new();
        for row in rows {
            let record = Self::diesel_row_to_record(row)?;
            notifications.push(
                record
                    .to_domain()
                    .map_err(|e| AppError::new(ErrorKind::InternalError, e))?,
            );
        }
        Ok(notifications)
    }
}
