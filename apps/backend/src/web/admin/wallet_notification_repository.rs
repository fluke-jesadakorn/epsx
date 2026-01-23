use crate::prelude::TlsPool;
// Wallet Notification Repository - Lightweight repository for wallet_notifications table
// Eliminates duplicate database logic from handlers

use diesel::prelude::*;
use diesel_async::{RunQueryDsl};
use chrono::{DateTime, Utc};
use uuid::Uuid;
use std::sync::Arc;

use crate::core::errors::{AppError, ErrorKind};
use super::notification_query_helper::NotificationQueryFilter;

/// DTO for wallet notification records
#[derive(Debug, Clone)]
pub struct WalletNotificationRecord {
    pub id: Uuid,
    pub wallet_address: String,
    pub notification_type: String,
    pub title: String,
    pub message: String,
    pub data: Option<serde_json::Value>,
    pub priority: String,
    pub timestamp: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
    pub read_at: Option<DateTime<Utc>>,
    pub clicked_at: Option<DateTime<Utc>>,
    pub delivered_at: Option<DateTime<Utc>>,
    pub action_url: Option<String>,
    pub image_url: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Repository for wallet_notifications table operations
pub struct WalletNotificationRepository {
    pool: Arc<&'static TlsPool>,
}

impl WalletNotificationRepository {
    pub fn new(pool: Arc<&'static TlsPool>) -> Self {
        Self { pool }
    }

    /// Find notifications with filters and pagination (admin view)
    pub async fn find_with_filters(
        &self,
        filter: &NotificationQueryFilter,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<WalletNotificationRecord>, AppError> {
        let mut conn = self.pool.get().await
            .map_err(|e| AppError::new(ErrorKind::DatabaseError, format!("Failed to get database connection: {}", e)))?;

        // Build WHERE clause manually
        let mut where_parts = vec!["deleted_at IS NULL".to_string()];

        if let Some(ref wallet) = filter.wallet_address {
            where_parts.push(format!("wallet_address = '{}'", wallet.replace("'", "''")));
        }
        if let Some(ref notif_type) = filter.notification_type {
            where_parts.push(format!("notification_type = '{}'", notif_type.replace("'", "''")));
        }
        if let Some(ref priority) = filter.priority {
            where_parts.push(format!("priority = '{}'", priority.replace("'", "''")));
        }
        if let Some(ref status) = filter.status {
            if status == "read" {
                where_parts.push("read_at IS NOT NULL".to_string());
            } else if status == "unread" {
                where_parts.push("read_at IS NULL".to_string());
            }
        }

        let query_str = format!(
            "SELECT id, wallet_address, notification_type, title, message, data, priority, \
             timestamp, expires_at, read_at, clicked_at, delivered_at, action_url, image_url, \
             created_at, updated_at \
             FROM wallet_notifications WHERE {} \
             ORDER BY timestamp DESC LIMIT {} OFFSET {}",
            where_parts.join(" AND "),
            limit,
            offset
        );

        #[derive(QueryableByName)]
        struct NotificationRow {
            #[diesel(sql_type = diesel::sql_types::Uuid)]
            id: Uuid,
            #[diesel(sql_type = diesel::sql_types::Text)]
            wallet_address: String,
            #[diesel(sql_type = diesel::sql_types::Text)]
            notification_type: String,
            #[diesel(sql_type = diesel::sql_types::Text)]
            title: String,
            #[diesel(sql_type = diesel::sql_types::Text)]
            message: String,
            #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Jsonb>)]
            data: Option<serde_json::Value>,
            #[diesel(sql_type = diesel::sql_types::Text)]
            priority: String,
            #[diesel(sql_type = diesel::sql_types::Timestamptz)]
            timestamp: DateTime<Utc>,
            #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Timestamptz>)]
            expires_at: Option<DateTime<Utc>>,
            #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Timestamptz>)]
            read_at: Option<DateTime<Utc>>,
            #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Timestamptz>)]
            clicked_at: Option<DateTime<Utc>>,
            #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Timestamptz>)]
            delivered_at: Option<DateTime<Utc>>,
            #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Text>)]
            action_url: Option<String>,
            #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Text>)]
            image_url: Option<String>,
            #[diesel(sql_type = diesel::sql_types::Timestamptz)]
            created_at: DateTime<Utc>,
            #[diesel(sql_type = diesel::sql_types::Timestamptz)]
            updated_at: DateTime<Utc>,
        }

        let records = diesel::sql_query(&query_str)
            .load::<NotificationRow>(&mut conn)
            .await
            .map_err(|e| AppError::new(ErrorKind::DatabaseError, format!("Failed to fetch notifications: {}", e)))?;

        Ok(records.into_iter().map(|r| WalletNotificationRecord {
            id: r.id,
            wallet_address: r.wallet_address,
            notification_type: r.notification_type,
            title: r.title,
            message: r.message,
            data: r.data,
            priority: r.priority,
            timestamp: r.timestamp,
            expires_at: r.expires_at,
            read_at: r.read_at,
            clicked_at: r.clicked_at,
            delivered_at: r.delivered_at,
            action_url: r.action_url,
            image_url: r.image_url,
            created_at: r.created_at,
            updated_at: r.updated_at,
        }).collect())
    }

    /// Find notifications for specific wallet (user view)
    pub async fn find_for_wallet(
        &self,
        wallet_address: &str,
        filter: &NotificationQueryFilter,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<WalletNotificationRecord>, AppError> {
        let mut conn = self.pool.get().await
            .map_err(|e| AppError::new(ErrorKind::DatabaseError, format!("Failed to get database connection: {}", e)))?;

        // Build WHERE clause manually - wallet specific (user OR broadcast 'all')
        let escaped_wallet = wallet_address.replace("'", "''");
        let mut where_parts = vec![
            "deleted_at IS NULL".to_string(),
            format!("(wallet_address = '{}' OR wallet_address = 'all')", escaped_wallet)
        ];

        if let Some(ref notif_type) = filter.notification_type {
            where_parts.push(format!("notification_type = '{}'", notif_type.replace("'", "''")));
        }
        if let Some(ref priority) = filter.priority {
            where_parts.push(format!("priority = '{}'", priority.replace("'", "''")));
        }
        if let Some(ref status) = filter.status {
            if status == "read" {
                where_parts.push("read_at IS NOT NULL".to_string());
            } else if status == "unread" {
                where_parts.push("read_at IS NULL".to_string());
            }
        }

        let query_str = format!(
            "SELECT id, wallet_address, notification_type, title, message, data, priority, \
             timestamp, expires_at, read_at, clicked_at, delivered_at, action_url, image_url, \
             created_at, updated_at \
             FROM wallet_notifications WHERE {} \
             ORDER BY timestamp DESC LIMIT {} OFFSET {}",
            where_parts.join(" AND "),
            limit,
            offset
        );

        #[derive(QueryableByName)]
        struct NotificationRow {
            #[diesel(sql_type = diesel::sql_types::Uuid)]
            id: Uuid,
            #[diesel(sql_type = diesel::sql_types::Text)]
            wallet_address: String,
            #[diesel(sql_type = diesel::sql_types::Text)]
            notification_type: String,
            #[diesel(sql_type = diesel::sql_types::Text)]
            title: String,
            #[diesel(sql_type = diesel::sql_types::Text)]
            message: String,
            #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Jsonb>)]
            data: Option<serde_json::Value>,
            #[diesel(sql_type = diesel::sql_types::Text)]
            priority: String,
            #[diesel(sql_type = diesel::sql_types::Timestamptz)]
            timestamp: DateTime<Utc>,
            #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Timestamptz>)]
            expires_at: Option<DateTime<Utc>>,
            #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Timestamptz>)]
            read_at: Option<DateTime<Utc>>,
            #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Timestamptz>)]
            clicked_at: Option<DateTime<Utc>>,
            #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Timestamptz>)]
            delivered_at: Option<DateTime<Utc>>,
            #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Text>)]
            action_url: Option<String>,
            #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Text>)]
            image_url: Option<String>,
            #[diesel(sql_type = diesel::sql_types::Timestamptz)]
            created_at: DateTime<Utc>,
            #[diesel(sql_type = diesel::sql_types::Timestamptz)]
            updated_at: DateTime<Utc>,
        }

        let records = diesel::sql_query(&query_str)
            .load::<NotificationRow>(&mut conn)
            .await
            .map_err(|e| AppError::new(ErrorKind::DatabaseError, format!("Failed to fetch notifications: {}", e)))?;

        Ok(records.into_iter().map(|r| WalletNotificationRecord {
            id: r.id,
            wallet_address: r.wallet_address,
            notification_type: r.notification_type,
            title: r.title,
            message: r.message,
            data: r.data,
            priority: r.priority,
            timestamp: r.timestamp,
            expires_at: r.expires_at,
            read_at: r.read_at,
            clicked_at: r.clicked_at,
            delivered_at: r.delivered_at,
            action_url: r.action_url,
            image_url: r.image_url,
            created_at: r.created_at,
            updated_at: r.updated_at,
        }).collect())
    }

    /// Count notifications with filters (admin view)
    pub async fn count_with_filters(
        &self,
        filter: &NotificationQueryFilter,
    ) -> Result<i64, AppError> {
        let mut conn = self.pool.get().await
            .map_err(|e| AppError::new(ErrorKind::DatabaseError, format!("Failed to get database connection: {}", e)))?;

        let mut where_parts = vec!["deleted_at IS NULL".to_string()];

        if let Some(ref wallet) = filter.wallet_address {
            where_parts.push(format!("wallet_address = '{}'", wallet.replace("'", "''")));
        }
        if let Some(ref notif_type) = filter.notification_type {
            where_parts.push(format!("notification_type = '{}'", notif_type.replace("'", "''")));
        }
        if let Some(ref priority) = filter.priority {
            where_parts.push(format!("priority = '{}'", priority.replace("'", "''")));
        }
        if let Some(ref status) = filter.status {
            if status == "read" {
                where_parts.push("read_at IS NOT NULL".to_string());
            } else if status == "unread" {
                where_parts.push("read_at IS NULL".to_string());
            }
        }

        let query_str = format!(
            "SELECT COUNT(*) as count FROM wallet_notifications WHERE {}",
            where_parts.join(" AND ")
        );

        #[derive(QueryableByName)]
        struct CountRow {
            #[diesel(sql_type = diesel::sql_types::BigInt)]
            count: i64,
        }

        let count = diesel::sql_query(&query_str)
            .get_result::<CountRow>(&mut conn)
            .await
            .map_err(|e| AppError::new(ErrorKind::DatabaseError, format!("Failed to count notifications: {}", e)))?
            .count;

        Ok(count)
    }

    /// Count notifications for specific wallet
    pub async fn count_for_wallet(
        &self,
        wallet_address: &str,
        filter: &NotificationQueryFilter,
    ) -> Result<i64, AppError> {
        let mut conn = self.pool.get().await
            .map_err(|e| AppError::new(ErrorKind::DatabaseError, format!("Failed to get database connection: {}", e)))?;

        let escaped_wallet = wallet_address.replace("'", "''");
        let mut where_parts = vec![
            "deleted_at IS NULL".to_string(),
            format!("(wallet_address = '{}' OR wallet_address = 'all')", escaped_wallet)
        ];

        if let Some(ref notif_type) = filter.notification_type {
            where_parts.push(format!("notification_type = '{}'", notif_type.replace("'", "''")));
        }
        if let Some(ref priority) = filter.priority {
            where_parts.push(format!("priority = '{}'", priority.replace("'", "''")));
        }
        if let Some(ref status) = filter.status {
            if status == "read" {
                where_parts.push("read_at IS NOT NULL".to_string());
            } else if status == "unread" {
                where_parts.push("read_at IS NULL".to_string());
            }
        }

        let query_str = format!(
            "SELECT COUNT(*) as count FROM wallet_notifications WHERE {}",
            where_parts.join(" AND ")
        );

        #[derive(QueryableByName)]
        struct CountRow {
            #[diesel(sql_type = diesel::sql_types::BigInt)]
            count: i64,
        }

        let count = diesel::sql_query(&query_str)
            .get_result::<CountRow>(&mut conn)
            .await
            .map_err(|e| AppError::new(ErrorKind::DatabaseError, format!("Failed to count notifications: {}", e)))?
            .count;

        Ok(count)
    }

    /// Count unread notifications with filters (admin view)
    pub async fn count_unread_with_filters(
        &self,
        filter: &NotificationQueryFilter,
    ) -> Result<i64, AppError> {
        let mut conn = self.pool.get().await
            .map_err(|e| AppError::new(ErrorKind::DatabaseError, format!("Failed to get database connection: {}", e)))?;

        let mut where_parts = vec!["read_at IS NULL".to_string(), "deleted_at IS NULL".to_string()];

        if let Some(ref wallet) = filter.wallet_address {
            where_parts.push(format!("wallet_address = '{}'", wallet.replace("'", "''")));
        }
        if let Some(ref notif_type) = filter.notification_type {
            where_parts.push(format!("notification_type = '{}'", notif_type.replace("'", "''")));
        }
        if let Some(ref priority) = filter.priority {
            where_parts.push(format!("priority = '{}'", priority.replace("'", "''")));
        }

        let query_str = format!(
            "SELECT COUNT(*) as count FROM wallet_notifications WHERE {}",
            where_parts.join(" AND ")
        );

        #[derive(QueryableByName)]
        struct CountRow {
            #[diesel(sql_type = diesel::sql_types::BigInt)]
            count: i64,
        }

        let count = diesel::sql_query(&query_str)
            .get_result::<CountRow>(&mut conn)
            .await
            .map_err(|e| AppError::new(ErrorKind::DatabaseError, format!("Failed to count unread: {}", e)))?
            .count;

        Ok(count)
    }

    /// Count unread notifications for specific wallet
    pub async fn count_unread_for_wallet(
        &self,
        wallet_address: &str,
        filter: &NotificationQueryFilter,
    ) -> Result<i64, AppError> {
        let mut conn = self.pool.get().await
            .map_err(|e| AppError::new(ErrorKind::DatabaseError, format!("Failed to get database connection: {}", e)))?;

        let escaped_wallet = wallet_address.replace("'", "''");
        let mut where_parts = vec![
            "deleted_at IS NULL".to_string(),
            "read_at IS NULL".to_string(),
            format!("(wallet_address = '{}' OR wallet_address = 'all')", escaped_wallet)
        ];

        if let Some(ref notif_type) = filter.notification_type {
            where_parts.push(format!("notification_type = '{}'", notif_type.replace("'", "''")));
        }
        if let Some(ref priority) = filter.priority {
            where_parts.push(format!("priority = '{}'", priority.replace("'", "''")));
        }

        let query_str = format!(
            "SELECT COUNT(*) as count FROM wallet_notifications WHERE {}",
            where_parts.join(" AND ")
        );

        #[derive(QueryableByName)]
        struct CountRow {
            #[diesel(sql_type = diesel::sql_types::BigInt)]
            count: i64,
        }

        let count = diesel::sql_query(&query_str)
            .get_result::<CountRow>(&mut conn)
            .await
            .map_err(|e| AppError::new(ErrorKind::DatabaseError, format!("Failed to count unread: {}", e)))?
            .count;

        Ok(count)
    }

    /// Create new notification
    pub async fn create(
        &self,
        id: Uuid,
        wallet_address: &str,
        notification_type: &str,
        title: &str,
        message: &str,
        data: Option<serde_json::Value>,
        priority: &str,
        expires_at: Option<DateTime<Utc>>,
        action_url: Option<String>,
        image_url: Option<String>,
    ) -> Result<(), AppError> {
        let mut conn = self.pool.get().await
            .map_err(|e| AppError::new(ErrorKind::DatabaseError, format!("Failed to get database connection: {}", e)))?;

        let now = Utc::now();

        diesel::sql_query(
            r#"
            INSERT INTO wallet_notifications
            (id, wallet_address, notification_type, title, message, data, priority, timestamp, expires_at, delivered_at, action_url, image_url)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
            "#
        )
        .bind::<diesel::sql_types::Uuid, _>(id)
        .bind::<diesel::sql_types::Text, _>(wallet_address)
        .bind::<diesel::sql_types::Text, _>(notification_type)
        .bind::<diesel::sql_types::Text, _>(title)
        .bind::<diesel::sql_types::Text, _>(message)
        .bind::<diesel::sql_types::Nullable<diesel::sql_types::Jsonb>, _>(data)
        .bind::<diesel::sql_types::Text, _>(priority)
        .bind::<diesel::sql_types::Timestamptz, _>(now)
        .bind::<diesel::sql_types::Nullable<diesel::sql_types::Timestamptz>, _>(expires_at)
        .bind::<diesel::sql_types::Nullable<diesel::sql_types::Timestamptz>, _>(Some(now))
        .bind::<diesel::sql_types::Nullable<diesel::sql_types::Text>, _>(action_url.as_deref())
        .bind::<diesel::sql_types::Nullable<diesel::sql_types::Text>, _>(image_url.as_deref())
        .execute(&mut conn)
        .await
        .map_err(|e| AppError::new(ErrorKind::DatabaseError, format!("Failed to save notification: {}", e)))?;

        Ok(())
    }

    /// Update delivery attempt
    pub async fn update_delivery_attempt(&self, id: Uuid) -> Result<(), AppError> {
        let mut conn = self.pool.get().await
            .map_err(|e| AppError::new(ErrorKind::DatabaseError, format!("Failed to get database connection: {}", e)))?;

        diesel::sql_query(
            "UPDATE wallet_notifications SET delivery_attempts = 1, last_delivery_attempt_at = NOW() WHERE id = $1"
        )
        .bind::<diesel::sql_types::Uuid, _>(id)
        .execute(&mut conn)
        .await
        .map_err(|e| AppError::new(ErrorKind::DatabaseError, format!("Failed to update delivery attempt: {}", e)))?;

        Ok(())
    }

    /// Mark notification as read
    pub async fn mark_as_read(&self, id: Uuid, wallet_address: &str) -> Result<u64, AppError> {
        let mut conn = self.pool.get().await
            .map_err(|e| AppError::new(ErrorKind::DatabaseError, format!("Failed to get database connection: {}", e)))?;

        let now = Utc::now();

        let rows_affected = diesel::sql_query(
            r#"
            UPDATE wallet_notifications
            SET read_at = $1, updated_at = $1
            WHERE id = $2 AND (wallet_address = $3 OR wallet_address = 'all')
            "#
        )
        .bind::<diesel::sql_types::Timestamptz, _>(now)
        .bind::<diesel::sql_types::Uuid, _>(id)
        .bind::<diesel::sql_types::Text, _>(wallet_address)
        .execute(&mut conn)
        .await
        .map_err(|e| AppError::new(ErrorKind::DatabaseError, format!("Failed to mark notification as read: {}", e)))?;

        Ok(rows_affected as u64)
    }

    /// Mark all notifications as read for wallet
    pub async fn mark_all_as_read(&self, wallet_address: &str) -> Result<u64, AppError> {
        let mut conn = self.pool.get().await
            .map_err(|e| AppError::new(ErrorKind::DatabaseError, format!("Failed to get database connection: {}", e)))?;

        let now = Utc::now();

        let rows_affected = diesel::sql_query(
            r#"
            UPDATE wallet_notifications
            SET read_at = $1, updated_at = $1
            WHERE (wallet_address = $2 OR wallet_address = 'all') AND read_at IS NULL AND deleted_at IS NULL
            "#
        )
        .bind::<diesel::sql_types::Timestamptz, _>(now)
        .bind::<diesel::sql_types::Text, _>(wallet_address)
        .execute(&mut conn)
        .await
        .map_err(|e| AppError::new(ErrorKind::DatabaseError, format!("Failed to mark all notifications as read: {}", e)))?;

        Ok(rows_affected as u64)
    }

    /// Soft delete notification
    pub async fn soft_delete(&self, id: Uuid, wallet_address: &str) -> Result<u64, AppError> {
        let mut conn = self.pool.get().await
            .map_err(|e| AppError::new(ErrorKind::DatabaseError, format!("Failed to get database connection: {}", e)))?;

        let rows_affected = diesel::sql_query(
            r#"
            UPDATE wallet_notifications
            SET deleted_at = NOW(), updated_at = NOW()
            WHERE id = $1 AND deleted_at IS NULL AND (wallet_address = $2 OR wallet_address = 'all')
            "#
        )
        .bind::<diesel::sql_types::Uuid, _>(id)
        .bind::<diesel::sql_types::Text, _>(wallet_address)
        .execute(&mut conn)
        .await
        .map_err(|e| AppError::new(ErrorKind::DatabaseError, format!("Failed to delete notification: {}", e)))?;

        Ok(rows_affected as u64)
    }

    /// Soft delete all notifications for wallet
    pub async fn soft_delete_all(&self, wallet_address: &str) -> Result<u64, AppError> {
        let mut conn = self.pool.get().await
            .map_err(|e| AppError::new(ErrorKind::DatabaseError, format!("Failed to get database connection: {}", e)))?;

        let rows_affected = diesel::sql_query(
            r#"
            UPDATE wallet_notifications
            SET deleted_at = NOW(), updated_at = NOW()
            WHERE (wallet_address = $1 OR wallet_address = 'all') AND deleted_at IS NULL
            "#
        )
        .bind::<diesel::sql_types::Text, _>(wallet_address)
        .execute(&mut conn)
        .await
        .map_err(|e| AppError::new(ErrorKind::DatabaseError, format!("Failed to clear all notifications: {}", e)))?;

        Ok(rows_affected as u64)
    }

    /// Hard delete notification (admin only)
    pub async fn hard_delete(&self, id: Uuid) -> Result<u64, AppError> {
        let mut conn = self.pool.get().await
            .map_err(|e| AppError::new(ErrorKind::DatabaseError, format!("Failed to get database connection: {}", e)))?;

        let rows_affected = diesel::sql_query(
            "DELETE FROM wallet_notifications WHERE id = $1"
        )
        .bind::<diesel::sql_types::Uuid, _>(id)
        .execute(&mut conn)
        .await
        .map_err(|e| AppError::new(ErrorKind::DatabaseError, format!("Failed to delete notification: {}", e)))?;

        Ok(rows_affected as u64)
    }

    /// Get simple unread count for wallet
    pub async fn get_unread_count(&self, wallet_address: &str) -> Result<i64, AppError> {
        let mut conn = self.pool.get().await
            .map_err(|e| AppError::new(ErrorKind::DatabaseError, format!("Failed to get database connection: {}", e)))?;

        #[derive(QueryableByName)]
        struct CountRow {
            #[diesel(sql_type = diesel::sql_types::BigInt)]
            count: i64,
        }

        let count = diesel::sql_query(
            "SELECT COUNT(*) as count FROM wallet_notifications \
             WHERE (wallet_address = $1 OR wallet_address = 'all') \
             AND read_at IS NULL AND deleted_at IS NULL"
        )
        .bind::<diesel::sql_types::Text, _>(wallet_address)
        .get_result::<CountRow>(&mut conn)
        .await
        .map_err(|e| AppError::new(ErrorKind::DatabaseError, format!("Failed to count unread notifications: {}", e)))?
        .count;

        Ok(count)
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_repository_creation() {
        // This is a placeholder test to ensure module compiles
        // Real tests would require a database connection
    }
}
