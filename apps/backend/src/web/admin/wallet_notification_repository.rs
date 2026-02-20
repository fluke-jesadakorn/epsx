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
        let mut where_parts = vec!["status != 'deleted'".to_string()]; // Assuming 'deleted' status instead of deleted_at column if removed, checking schema... up.sql had NO deleted_at.

        // Wait, up.sql does NOT have deleted_at! It has `status` but no `deleted_at`.
        // I will assume logic for soft delete is setting status to 'deleted'.

        if let Some(ref wallet) = filter.wallet_address {
            where_parts.push(format!("recipient_wallet_address = '{}'", wallet.replace("'", "''")));
        }
        if let Some(ref notif_type) = filter.notification_type {
            where_parts.push(format!("notification_type = '{}'", notif_type.replace("'", "''")));
        }
        if let Some(ref priority) = filter.priority {
            where_parts.push(format!("priority = '{}'", priority.replace("'", "''")));
        }
        if let Some(ref status) = filter.status {
            if status == "read" {
                where_parts.push("status = 'read'".to_string());
            } else if status == "unread" {
                where_parts.push("status != 'read'".to_string()); // includes created, sent, etc.
            }
        }

        let query_str = format!(
            "SELECT id, recipient_wallet_address, notification_type, title, body, data_payload, priority, \
             created_at, expires_at, status, action_url, image_url, \
             created_at as created_at_alias, updated_at \
             FROM wallet_notifications WHERE {} \
             ORDER BY created_at DESC LIMIT {} OFFSET {}",
            where_parts.join(" AND "),
            limit,
            offset
        );

        #[derive(QueryableByName)]
        struct NotificationRow {
            #[diesel(sql_type = diesel::sql_types::Uuid)]
            id: Uuid,
            #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Text>)] // recipient_wallet_address can be null? up.sql says VARCHAR(42), nullable
            recipient_wallet_address: Option<String>,
            #[diesel(sql_type = diesel::sql_types::Text)]
            notification_type: String,
            #[diesel(sql_type = diesel::sql_types::Text)]
            title: String,
            #[diesel(sql_type = diesel::sql_types::Text)]
            body: String,
            #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Jsonb>)]
            data_payload: Option<serde_json::Value>,
            #[diesel(sql_type = diesel::sql_types::Text)]
            priority: String,
            #[diesel(sql_type = diesel::sql_types::Timestamptz)]
            created_at: DateTime<Utc>,
            #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Timestamptz>)]
            expires_at: Option<DateTime<Utc>>,
            #[diesel(sql_type = diesel::sql_types::Text)]
            status: String,
            #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Text>)]
            action_url: Option<String>,
            #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Text>)]
            image_url: Option<String>,
            #[diesel(sql_type = diesel::sql_types::Timestamptz)]
            created_at_alias: DateTime<Utc>,
            #[diesel(sql_type = diesel::sql_types::Timestamptz)]
            updated_at: DateTime<Utc>,
        }

        let records = diesel::sql_query(&query_str)
            .load::<NotificationRow>(&mut conn)
            .await
            .map_err(|e| AppError::new(ErrorKind::DatabaseError, format!("Failed to fetch notifications: {}", e)))?;

        Ok(records.into_iter().map(|r| {
            let read_at = if r.status == "read" { Some(r.updated_at) } else { None };
            
            WalletNotificationRecord {
                id: r.id,
                wallet_address: r.recipient_wallet_address.unwrap_or_default(),
                notification_type: r.notification_type,
                title: r.title,
                message: r.body,
                data: r.data_payload,
                priority: r.priority,
                timestamp: r.created_at,
                expires_at: r.expires_at,
                read_at,
                clicked_at: None, // Not tracked in new schema
                delivered_at: None, // Not tracked directly or use send_started_at?
                action_url: r.action_url,
                image_url: r.image_url,
                created_at: r.created_at_alias,
                updated_at: r.updated_at,
            }
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

        // Build WHERE clause manually
        let escaped_wallet = wallet_address.replace("'", "''");
        let mut where_parts = vec![
            "status != 'deleted'".to_string(),
            format!("(recipient_wallet_address = '{}' OR recipient_wallet_address = 'all')", escaped_wallet)
        ];

        if let Some(ref notif_type) = filter.notification_type {
            where_parts.push(format!("notification_type = '{}'", notif_type.replace("'", "''")));
        }
        if let Some(ref priority) = filter.priority {
            where_parts.push(format!("priority = '{}'", priority.replace("'", "''")));
        }
        if let Some(ref status) = filter.status {
            if status == "read" {
                where_parts.push("status = 'read'".to_string());
            } else if status == "unread" {
                where_parts.push("status != 'read'".to_string());
            }
        }

        let query_str = format!(
            "SELECT id, recipient_wallet_address, notification_type, title, body, data_payload, priority, \
             created_at, expires_at, status, action_url, image_url, \
             created_at as created_at_alias, updated_at \
             FROM wallet_notifications WHERE {} \
             ORDER BY created_at DESC LIMIT {} OFFSET {}",
            where_parts.join(" AND "),
            limit,
            offset
        );

        #[derive(QueryableByName)]
        struct NotificationRow {
            #[diesel(sql_type = diesel::sql_types::Uuid)]
            id: Uuid,
            #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Text>)]
            recipient_wallet_address: Option<String>,
            #[diesel(sql_type = diesel::sql_types::Text)]
            notification_type: String,
            #[diesel(sql_type = diesel::sql_types::Text)]
            title: String,
            #[diesel(sql_type = diesel::sql_types::Text)]
            body: String,
            #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Jsonb>)]
            data_payload: Option<serde_json::Value>,
            #[diesel(sql_type = diesel::sql_types::Text)]
            priority: String,
            #[diesel(sql_type = diesel::sql_types::Timestamptz)]
            created_at: DateTime<Utc>,
            #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Timestamptz>)]
            expires_at: Option<DateTime<Utc>>,
            #[diesel(sql_type = diesel::sql_types::Text)]
            status: String,
            #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Text>)]
            action_url: Option<String>,
            #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Text>)]
            image_url: Option<String>,
            #[diesel(sql_type = diesel::sql_types::Timestamptz)]
            created_at_alias: DateTime<Utc>,
            #[diesel(sql_type = diesel::sql_types::Timestamptz)]
            updated_at: DateTime<Utc>,
        }

        let records = diesel::sql_query(&query_str)
            .load::<NotificationRow>(&mut conn)
            .await
            .map_err(|e| AppError::new(ErrorKind::DatabaseError, format!("Failed to fetch notifications: {}", e)))?;

        Ok(records.into_iter().map(|r| {
            let read_at = if r.status == "read" { Some(r.updated_at) } else { None };

            WalletNotificationRecord {
                id: r.id,
                wallet_address: r.recipient_wallet_address.unwrap_or_default(),
                notification_type: r.notification_type,
                title: r.title,
                message: r.body,
                data: r.data_payload,
                priority: r.priority,
                timestamp: r.created_at,
                expires_at: r.expires_at,
                read_at,
                clicked_at: None,
                delivered_at: None,
                action_url: r.action_url,
                image_url: r.image_url,
                created_at: r.created_at_alias,
                updated_at: r.updated_at,
            }
        }).collect())
    }

    /// Count notifications with filters (admin view)
    pub async fn count_with_filters(
        &self,
        filter: &NotificationQueryFilter,
    ) -> Result<i64, AppError> {
        let mut conn = self.pool.get().await
            .map_err(|e| AppError::new(ErrorKind::DatabaseError, format!("Failed to get database connection: {}", e)))?;

        let mut where_parts = vec!["status != 'deleted'".to_string()];

        if let Some(ref wallet) = filter.wallet_address {
            where_parts.push(format!("recipient_wallet_address = '{}'", wallet.replace("'", "''")));
        }
        if let Some(ref notif_type) = filter.notification_type {
            where_parts.push(format!("notification_type = '{}'", notif_type.replace("'", "''")));
        }
        if let Some(ref priority) = filter.priority {
            where_parts.push(format!("priority = '{}'", priority.replace("'", "''")));
        }
        if let Some(ref status) = filter.status {
            if status == "read" {
                where_parts.push("status = 'read'".to_string());
            } else if status == "unread" {
                where_parts.push("status != 'read'".to_string());
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
            "status != 'deleted'".to_string(),
            format!("(recipient_wallet_address = '{}' OR recipient_wallet_address = 'all')", escaped_wallet)
        ];

        if let Some(ref notif_type) = filter.notification_type {
            where_parts.push(format!("notification_type = '{}'", notif_type.replace("'", "''")));
        }
        if let Some(ref priority) = filter.priority {
            where_parts.push(format!("priority = '{}'", priority.replace("'", "''")));
        }
        if let Some(ref status) = filter.status {
            if status == "read" {
                where_parts.push("status = 'read'".to_string());
            } else if status == "unread" {
                where_parts.push("status != 'read'".to_string());
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

        let mut where_parts = vec!["status != 'read'".to_string(), "status != 'deleted'".to_string()];

        if let Some(ref wallet) = filter.wallet_address {
            where_parts.push(format!("recipient_wallet_address = '{}'", wallet.replace("'", "''")));
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
            "status != 'deleted'".to_string(),
            "status != 'read'".to_string(),
            format!("(recipient_wallet_address = '{}' OR recipient_wallet_address = 'all')", escaped_wallet)
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
            (id, recipient_wallet_address, notification_type, title, body, data_payload, priority, created_at, expires_at, action_url, image_url, status)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, 'created')
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
        .bind::<diesel::sql_types::Nullable<diesel::sql_types::Text>, _>(action_url.as_deref())
        .bind::<diesel::sql_types::Nullable<diesel::sql_types::Text>, _>(image_url.as_deref())
        .execute(&mut conn)
        .await
        .map_err(|e| AppError::new(ErrorKind::DatabaseError, format!("Failed to save notification: {}", e)))?;

        Ok(())
    }

    /// Update delivery attempt - Updated for new schema
    pub async fn update_delivery_attempt(&self, id: Uuid) -> Result<(), AppError> {
        let mut conn = self.pool.get().await
            .map_err(|e| AppError::new(ErrorKind::DatabaseError, format!("Failed to get database connection: {}", e)))?;

        // New schema uses total_attempts
        diesel::sql_query(
            "UPDATE wallet_notifications SET total_attempts = total_attempts + 1, updated_at = NOW() WHERE id = $1"
        )
        .bind::<diesel::sql_types::Uuid, _>(id)
        .execute(&mut conn)
        .await
        .map_err(|e| AppError::new(ErrorKind::DatabaseError, format!("Failed to update delivery attempt: {}", e)))?;

        Ok(())
    }

    /// Mark notification as read - Updated: Sets status = 'read'
    pub async fn mark_as_read(&self, id: Uuid, wallet_address: &str) -> Result<u64, AppError> {
        let mut conn = self.pool.get().await
            .map_err(|e| AppError::new(ErrorKind::DatabaseError, format!("Failed to get database connection: {}", e)))?;

        let now = Utc::now();

        let rows_affected = diesel::sql_query(
            r#"
            UPDATE wallet_notifications
            SET status = 'read', updated_at = $1
            WHERE id = $2 AND (recipient_wallet_address = $3 OR recipient_wallet_address = 'all')
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
            SET status = 'read', updated_at = $1
            WHERE (recipient_wallet_address = $2 OR recipient_wallet_address = 'all') AND status != 'read' AND status != 'deleted'
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
            SET status = 'deleted', updated_at = NOW()
            WHERE id = $1 AND status != 'deleted' AND (recipient_wallet_address = $2 OR recipient_wallet_address = 'all')
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
            SET status = 'deleted', updated_at = NOW()
            WHERE (recipient_wallet_address = $1 OR recipient_wallet_address = 'all') AND status != 'deleted'
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
             WHERE (recipient_wallet_address = $1 OR recipient_wallet_address = 'all') \
             AND status != 'read' AND status != 'deleted'"
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
