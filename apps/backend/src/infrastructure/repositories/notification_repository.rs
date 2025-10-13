use sqlx::{PgPool, Row};
use std::sync::Arc;
use uuid::Uuid;
use async_trait::async_trait;

use crate::prelude::*;
use crate::domain::notification::*;
use crate::core::errors::{AppError, ErrorKind};
use super::notification_record::NotificationRecord;

pub struct NotificationRepository {
    pool: Arc<PgPool>,
}

impl NotificationRepository {
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }

    async fn row_to_record(row: sqlx::postgres::PgRow) -> Result<NotificationRecord, AppError> {
        Ok(NotificationRecord {
            id: row.try_get("id").map_err(|e| AppError::new(ErrorKind::DatabaseError, e.to_string()))?,
            recipient_wallet_id: row.try_get("recipient_wallet_id").map_err(|e| AppError::new(ErrorKind::DatabaseError, e.to_string()))?,
            topic_name: row.try_get("topic_name").map_err(|e| AppError::new(ErrorKind::DatabaseError, e.to_string()))?,
            title: row.try_get("title").map_err(|e| AppError::new(ErrorKind::DatabaseError, e.to_string()))?,
            body: row.try_get("body").map_err(|e| AppError::new(ErrorKind::DatabaseError, e.to_string()))?,
            urgency: row.try_get("urgency").map_err(|e| AppError::new(ErrorKind::DatabaseError, e.to_string()))?,
            notification_type: row.try_get("notification_type").map_err(|e| AppError::new(ErrorKind::DatabaseError, e.to_string()))?,
            priority: row.try_get("priority").map_err(|e| AppError::new(ErrorKind::DatabaseError, e.to_string()))?,
            channels: row.try_get("channels").map_err(|e| AppError::new(ErrorKind::DatabaseError, e.to_string()))?,
            schedule_type: row.try_get("schedule_type").map_err(|e| AppError::new(ErrorKind::DatabaseError, e.to_string()))?,
            scheduled_at: row.try_get("scheduled_at").map_err(|e| AppError::new(ErrorKind::DatabaseError, e.to_string()))?,
            expires_at: row.try_get("expires_at").map_err(|e| AppError::new(ErrorKind::DatabaseError, e.to_string()))?,
            status: row.try_get("status").map_err(|e| AppError::new(ErrorKind::DatabaseError, e.to_string()))?,
            send_started_at: row.try_get("send_started_at").map_err(|e| AppError::new(ErrorKind::DatabaseError, e.to_string()))?,
            channel_status: row.try_get("channel_status").map_err(|e| AppError::new(ErrorKind::DatabaseError, e.to_string()))?,
            total_attempts: row.try_get("total_attempts").map_err(|e| AppError::new(ErrorKind::DatabaseError, e.to_string()))?,
            created_by_wallet_id: row.try_get("created_by_wallet_id").map_err(|e| AppError::new(ErrorKind::DatabaseError, e.to_string()))?,
            image_url: row.try_get("image_url").map_err(|e| AppError::new(ErrorKind::DatabaseError, e.to_string()))?,
            action_url: row.try_get("action_url").map_err(|e| AppError::new(ErrorKind::DatabaseError, e.to_string()))?,
            data_payload: row.try_get("data_payload").map_err(|e| AppError::new(ErrorKind::DatabaseError, e.to_string()))?,
            tags: row.try_get("tags").map_err(|e| AppError::new(ErrorKind::DatabaseError, e.to_string()))?,
            notes: row.try_get("notes").map_err(|e| AppError::new(ErrorKind::DatabaseError, e.to_string()))?,
            version: row.try_get("version").map_err(|e| AppError::new(ErrorKind::DatabaseError, e.to_string()))?,
            created_at: row.try_get("created_at").map_err(|e| AppError::new(ErrorKind::DatabaseError, e.to_string()))?,
            updated_at: row.try_get("updated_at").map_err(|e| AppError::new(ErrorKind::DatabaseError, e.to_string()))?,
        })
    }
}

#[async_trait]
impl NotificationRepositoryPort for NotificationRepository {
    async fn find_by_id(&self, notification_id: &str) -> AppResult<Option<Notification>> {
        let id = Uuid::parse_str(notification_id)
            .map_err(|e| AppError::new(ErrorKind::ValidationError, format!("Invalid notification ID: {}", e)))?;

        let row = sqlx::query(
            r#"
            SELECT id, recipient_wallet_id, topic_name, title, body, urgency,
                   notification_type, priority, channels, schedule_type, scheduled_at, expires_at,
                   status, send_started_at, channel_status, total_attempts,
                   created_by_wallet_id, image_url, action_url, data_payload, tags, notes,
                   version, created_at, updated_at
            FROM notifications WHERE id = $1
            "#
        )
        .bind(id)
        .fetch_optional(&*self.pool)
        .await
        .map_err(|e| AppError::new(ErrorKind::DatabaseError, format!("Database error: {}", e)))?;

        match row {
            Some(r) => {
                let record = Self::row_to_record(r).await?;
                Ok(Some(record.to_domain().map_err(|e| AppError::new(ErrorKind::InternalError, e))?))
            },
            None => Ok(None),
        }
    }

    async fn find_all(&self, criteria: NotificationSearchCriteria) -> AppResult<Vec<Notification>> {
        let rows = sqlx::query(
            r#"
            SELECT id, recipient_wallet_id, topic_name, title, body, urgency,
                   notification_type, priority, channels, schedule_type, scheduled_at, expires_at,
                   status, send_started_at, channel_status, total_attempts,
                   created_by_wallet_id, image_url, action_url, data_payload, tags, notes,
                   version, created_at, updated_at
            FROM notifications
            WHERE ($1::uuid IS NULL OR recipient_wallet_id = $1)
              AND ($2::text IS NULL OR topic_name = $2)
              AND ($3::text IS NULL OR status = $3)
              AND ($4::text IS NULL OR notification_type = $4)
              AND ($5::text IS NULL OR priority = $5)
              AND ($6::timestamptz IS NULL OR created_at >= $6)
              AND ($7::timestamptz IS NULL OR created_at <= $7)
            ORDER BY created_at DESC
            LIMIT $8 OFFSET $9
            "#
        )
        .bind(criteria.recipient_wallet_address)
        .bind(criteria.topic)
        .bind(criteria.status.as_ref().map(|s| s.as_str()))
        .bind(criteria.notification_type.as_ref().map(|t| t.as_str()))
        .bind(criteria.priority)
        .bind(criteria.created_after)
        .bind(criteria.created_before)
        .bind(criteria.limit.unwrap_or(100))
        .bind(criteria.offset.unwrap_or(0))
        .fetch_all(&*self.pool)
        .await
        .map_err(|e| AppError::new(ErrorKind::DatabaseError, format!("Database error: {}", e)))?;

        let mut notifications = Vec::new();
        for row in rows {
            let record = Self::row_to_record(row).await?;
            notifications.push(record.to_domain().map_err(|e| AppError::new(ErrorKind::InternalError, e))?);
        }
        Ok(notifications)
    }

    async fn save(&self, notification: &Notification) -> AppResult<()> {
        let record = NotificationRecord::from_domain(notification);

        sqlx::query(
            r#"
            INSERT INTO notifications (
                id, recipient_wallet_id, topic_name, title, body, urgency,
                notification_type, priority, channels, schedule_type, scheduled_at, expires_at,
                status, send_started_at, channel_status, total_attempts,
                created_by_wallet_id, image_url, action_url, data_payload, tags, notes,
                version, created_at, updated_at
            ) VALUES (
                $1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16,
                $17, $18, $19, $20, $21, $22, $23, $24, $25
            )
            ON CONFLICT (id) DO UPDATE SET
                recipient_wallet_id = EXCLUDED.recipient_wallet_id,
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
            "#
        )
        .bind(record.id)
        .bind(record.recipient_wallet_id)
        .bind(record.topic_name)
        .bind(record.title)
        .bind(record.body)
        .bind(record.urgency)
        .bind(record.notification_type)
        .bind(record.priority)
        .bind(record.channels)
        .bind(record.schedule_type)
        .bind(record.scheduled_at)
        .bind(record.expires_at)
        .bind(record.status)
        .bind(record.send_started_at)
        .bind(record.channel_status)
        .bind(record.total_attempts)
        .bind(record.created_by_wallet_id)
        .bind(record.image_url)
        .bind(record.action_url)
        .bind(record.data_payload)
        .bind(&record.tags)
        .bind(&record.notes)
        .bind(record.version)
        .bind(record.created_at)
        .bind(record.updated_at)
        .execute(&*self.pool)
        .await
        .map_err(|e| AppError::new(ErrorKind::DatabaseError, format!("Failed to save notification: {}", e)))?;

        Ok(())
    }

    async fn delete(&self, notification_id: &str) -> AppResult<()> {
        let id = Uuid::parse_str(notification_id)
            .map_err(|e| AppError::new(ErrorKind::ValidationError, format!("Invalid notification ID: {}", e)))?;

        sqlx::query("DELETE FROM notifications WHERE id = $1")
            .bind(id)
            .execute(&*self.pool)
            .await
            .map_err(|e| AppError::new(ErrorKind::DatabaseError, format!("Failed to delete notification: {}", e)))?;

        Ok(())
    }

    async fn count(&self, criteria: NotificationSearchCriteria) -> AppResult<i64> {
        let row = sqlx::query(
            r#"
            SELECT COUNT(*) as count
            FROM notifications
            WHERE ($1::uuid IS NULL OR recipient_wallet_id = $1)
              AND ($2::text IS NULL OR topic_name = $2)
              AND ($3::text IS NULL OR status = $3)
              AND ($4::text IS NULL OR notification_type = $4)
            "#
        )
        .bind(criteria.recipient_wallet_address)
        .bind(criteria.topic)
        .bind(criteria.status.as_ref().map(|s| s.as_str()))
        .bind(criteria.notification_type.as_ref().map(|t| t.as_str()))
        .fetch_one(&*self.pool)
        .await
        .map_err(|e| AppError::new(ErrorKind::DatabaseError, format!("Failed to count notifications: {}", e)))?;

        let count: i64 = row.try_get("count")
            .map_err(|e| AppError::new(ErrorKind::DatabaseError, e.to_string()))?;
        Ok(count)
    }

    async fn notification_exists(&self, notification_id: &str) -> AppResult<bool> {
        let id = Uuid::parse_str(notification_id)
            .map_err(|e| AppError::new(ErrorKind::ValidationError, format!("Invalid notification ID: {}", e)))?;

        let row = sqlx::query("SELECT EXISTS(SELECT 1 FROM notifications WHERE id = $1) as exists")
            .bind(id)
            .fetch_one(&*self.pool)
            .await
            .map_err(|e| AppError::new(ErrorKind::DatabaseError, format!("Database error: {}", e)))?;

        let exists: bool = row.try_get("exists")
            .map_err(|e| AppError::new(ErrorKind::DatabaseError, e.to_string()))?;
        Ok(exists)
    }

    async fn find_pending(&self, limit: u32) -> AppResult<Vec<Notification>> {
        let rows = sqlx::query(
            r#"
            SELECT id, recipient_wallet_id, topic_name, title, body, urgency,
                   notification_type, priority, channels, schedule_type, scheduled_at, expires_at,
                   status, send_started_at, channel_status, total_attempts,
                   created_by_wallet_id, image_url, action_url, data_payload, tags, notes,
                   version, created_at, updated_at
            FROM notifications
            WHERE status IN ('scheduled', 'queued', 'created')
              AND (scheduled_at IS NULL OR scheduled_at <= NOW())
              AND (expires_at IS NULL OR expires_at > NOW())
            ORDER BY priority DESC, created_at ASC
            LIMIT $1
            "#
        )
        .bind(limit as i64)
        .fetch_all(&*self.pool)
        .await
        .map_err(|e| AppError::new(ErrorKind::DatabaseError, format!("Database error: {}", e)))?;

        let mut notifications = Vec::new();
        for row in rows {
            let record = Self::row_to_record(row).await?;
            notifications.push(record.to_domain().map_err(|e| AppError::new(ErrorKind::InternalError, e))?);
        }
        Ok(notifications)
    }

    async fn find_by_wallet(&self, wallet_address: Uuid) -> AppResult<Vec<Notification>> {
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
        let rows = sqlx::query(
            r#"
            SELECT id, recipient_wallet_id, topic_name, title, body, urgency,
                   notification_type, priority, channels, schedule_type, scheduled_at, expires_at,
                   status, send_started_at, channel_status, total_attempts,
                   created_by_wallet_id, image_url, action_url, data_payload, tags, notes,
                   version, created_at, updated_at
            FROM notifications
            WHERE expires_at IS NOT NULL
              AND expires_at <= NOW()
              AND status NOT IN ('expired', 'delivered', 'cancelled')
            "#
        )
        .fetch_all(&*self.pool)
        .await
        .map_err(|e| AppError::new(ErrorKind::DatabaseError, format!("Database error: {}", e)))?;

        let mut notifications = Vec::new();
        for row in rows {
            let record = Self::row_to_record(row).await?;
            notifications.push(record.to_domain().map_err(|e| AppError::new(ErrorKind::InternalError, e))?);
        }
        Ok(notifications)
    }
}
