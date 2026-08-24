// Wallet Notification Repository - Lightweight repository for wallet_notifications table
// Eliminates duplicate database logic from handlers
//
// BIG-BANG: migrated to sqlx (real). All diesel DSL replaced with sqlx queries.

use std::sync::Arc;

use chrono::{DateTime, Utc};
use sqlx::{PgPool, QueryBuilder};
use uuid::Uuid;

use epsx_contracts::errors::{AppError, ErrorKind};

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

/// The immutable fields used to verify a producer event retry.
#[derive(Debug, Clone, PartialEq)]
pub struct WalletNotificationIdentity {
    pub wallet_address: String,
    pub notification_type: String,
    pub title: String,
    pub message: String,
    pub data: Option<serde_json::Value>,
    pub priority: String,
    pub action_url: Option<String>,
    pub expires_at: Option<DateTime<Utc>>,
}

impl WalletNotificationIdentity {
    pub fn matches_payload(
        &self,
        wallet_address: &str,
        notification_type: &str,
        priority: &str,
        title: &str,
        message: &str,
        data: Option<&serde_json::Value>,
        action_url: Option<&str>,
        expires_at: Option<DateTime<Utc>>,
    ) -> bool {
        self.wallet_address == wallet_address
            && self.notification_type == notification_type
            && self.priority == priority
            && self.title == title
            && self.message == message
            && self.data.as_ref() == data
            && self.action_url.as_deref() == action_url
            && self.expires_at == expires_at
    }
}

/// Row for sqlx::query_as
#[derive(sqlx::FromRow)]
struct NotificationRow {
    id: Uuid,
    recipient_wallet_address: Option<String>,
    notification_type: String,
    title: String,
    body: String,
    data_payload: Option<serde_json::Value>,
    priority: String,
    created_at: DateTime<Utc>,
    expires_at: Option<DateTime<Utc>>,
    status: String,
    action_url: Option<String>,
    image_url: Option<String>,
    updated_at: DateTime<Utc>,
}

/// Repository for wallet_notifications table operations
pub struct WalletNotificationRepository {
    pool: Arc<PgPool>,
}

impl WalletNotificationRepository {
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }

    /// Helper to map a NotificationRow to WalletNotificationRecord
    fn row_to_record(r: NotificationRow) -> WalletNotificationRecord {
        let read_at = if r.status == "read" {
            Some(r.updated_at)
        } else {
            None
        };
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
            created_at: r.created_at,
            updated_at: r.updated_at,
        }
    }

    /// Build WHERE clause prefix shared by find/count queries.
    fn apply_filter_to_qb(
        mut qb: QueryBuilder<sqlx::Postgres>,
        filter: &NotificationQueryFilter,
    ) -> QueryBuilder<sqlx::Postgres> {
        qb.push(" WHERE status != 'deleted'");
        if let Some(ref wallet) = filter.wallet_address {
            qb.push(" AND recipient_wallet_address = ").push_bind(wallet.clone());
        }
        if let Some(ref notif_type) = filter.notification_type {
            qb.push(" AND notification_type = ").push_bind(notif_type.clone());
        }
        if let Some(ref priority) = filter.priority {
            qb.push(" AND priority = ").push_bind(priority.clone());
        }
        if let Some(ref status) = filter.status {
            match status.as_str() {
                "read" => qb.push(" AND status = 'read'"),
                "unread" => qb.push(" AND status != 'read'"),
                _ => {}
            }
        }
        qb
    }

    /// Load only the immutable fields needed to validate a stable producer
    /// event retry.
    pub async fn find_identity_by_id(
        &self,
        id: Uuid,
    ) -> Result<Option<WalletNotificationIdentity>, AppError> {
        #[derive(sqlx::FromRow)]
        struct IdentityRow {
            recipient_wallet_address: Option<String>,
            notification_type: String,
            title: String,
            body: String,
            data_payload: Option<serde_json::Value>,
            priority: String,
            action_url: Option<String>,
            expires_at: Option<DateTime<Utc>>,
        }

        let rows: Vec<IdentityRow> = sqlx::query_as(
            "SELECT recipient_wallet_address, notification_type, title, body, data_payload, priority, action_url, expires_at \
             FROM wallet_notifications WHERE id = $1",
        )
        .bind(id)
        .fetch_all(self.pool.as_ref())
        .await
        .map_err(|e| {
            AppError::new(
                ErrorKind::DatabaseError,
                format!("Failed to fetch notification identity: {}", e),
            )
        })?;

        Ok(rows.into_iter().next().map(|row| WalletNotificationIdentity {
            wallet_address: row.recipient_wallet_address.unwrap_or_default(),
            notification_type: row.notification_type,
            title: row.title,
            message: row.body,
            data: row.data_payload,
            priority: row.priority,
            action_url: row.action_url,
            expires_at: row.expires_at,
        }))
    }

    /// Find notifications with filters and pagination (admin view)
    pub async fn find_with_filters(
        &self,
        filter: &NotificationQueryFilter,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<WalletNotificationRecord>, AppError> {
        let qb = QueryBuilder::new(
            "SELECT id, recipient_wallet_address, notification_type, title, body, data_payload, \
                    priority, created_at, expires_at, status, action_url, image_url, updated_at \
             FROM wallet_notifications",
        );
        let qb = Self::apply_filter_to_qb(qb, filter);
        let qb = qb
            .push(" ORDER BY created_at DESC LIMIT ")
            .push_bind(limit)
            .push(" OFFSET ")
            .push_bind(offset);

        let rows: Vec<NotificationRow> = qb
            .build_query_as()
            .fetch_all(self.pool.as_ref())
            .await
            .map_err(|e| {
                AppError::new(
                    ErrorKind::DatabaseError,
                    format!("Failed to fetch notifications: {}", e),
                )
            })?;

        Ok(rows.into_iter().map(Self::row_to_record).collect())
    }

    /// Find notifications for specific wallet (user view)
    pub async fn find_for_wallet(
        &self,
        wallet_address: &str,
        filter: &NotificationQueryFilter,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<WalletNotificationRecord>, AppError> {
        let escaped_wallet = wallet_address.to_lowercase();
        let mut qb = QueryBuilder::new(
            "SELECT id, recipient_wallet_address, notification_type, title, body, data_payload, \
                    priority, created_at, expires_at, status, action_url, image_url, updated_at \
             FROM wallet_notifications",
        );
        qb.push(" WHERE status != 'deleted'");
        qb.push(" AND (LOWER(recipient_wallet_address) = ").push_bind(escaped_wallet);
        qb.push(" OR recipient_wallet_address = 'all')");

        if let Some(ref notif_type) = filter.notification_type {
            qb.push(" AND notification_type = ").push_bind(notif_type.clone());
        }
        if let Some(ref priority) = filter.priority {
            qb.push(" AND priority = ").push_bind(priority.clone());
        }
        if let Some(ref status) = filter.status {
            match status.as_str() {
                "read" => qb.push(" AND status = 'read'"),
                "unread" => qb.push(" AND status != 'read'"),
                _ => {}
            }
        }

        qb.push(" ORDER BY created_at DESC LIMIT ")
            .push_bind(limit)
            .push(" OFFSET ")
            .push_bind(offset);

        let rows: Vec<NotificationRow> = qb
            .build_query_as()
            .fetch_all(self.pool.as_ref())
            .await
            .map_err(|e| {
                AppError::new(
                    ErrorKind::DatabaseError,
                    format!("Failed to fetch notifications: {}", e),
                )
            })?;

        Ok(rows.into_iter().map(Self::row_to_record).collect())
    }

    /// Count notifications with filters (admin view)
    pub async fn count_with_filters(
        &self,
        filter: &NotificationQueryFilter,
    ) -> Result<i64, AppError> {
        let qb = QueryBuilder::new("SELECT COUNT(*) as count FROM wallet_notifications");
        let qb = Self::apply_filter_to_qb(qb, filter);
        let row: (i64,) = qb
            .build_query_as()
            .fetch_one(self.pool.as_ref())
            .await
            .map_err(|e| {
                AppError::new(
                    ErrorKind::DatabaseError,
                    format!("Failed to count notifications: {}", e),
                )
            })?;
        Ok(row.0)
    }

    /// Count notifications for specific wallet
    pub async fn count_for_wallet(
        &self,
        wallet_address: &str,
        filter: &NotificationQueryFilter,
    ) -> Result<i64, AppError> {
        let escaped_wallet = wallet_address.to_lowercase();
        let mut qb = QueryBuilder::new("SELECT COUNT(*) as count FROM wallet_notifications");
        qb.push(" WHERE status != 'deleted'");
        qb.push(" AND (LOWER(recipient_wallet_address) = ").push_bind(escaped_wallet);
        qb.push(" OR recipient_wallet_address = 'all')");

        if let Some(ref notif_type) = filter.notification_type {
            qb.push(" AND notification_type = ").push_bind(notif_type.clone());
        }
        if let Some(ref priority) = filter.priority {
            qb.push(" AND priority = ").push_bind(priority.clone());
        }
        if let Some(ref status) = filter.status {
            match status.as_str() {
                "read" => qb.push(" AND status = 'read'"),
                "unread" => qb.push(" AND status != 'read'"),
                _ => {}
            }
        }

        let row: (i64,) = qb
            .build_query_as()
            .fetch_one(self.pool.as_ref())
            .await
            .map_err(|e| {
                AppError::new(
                    ErrorKind::DatabaseError,
                    format!("Failed to count notifications: {}", e),
                )
            })?;
        Ok(row.0)
    }

    /// Count unread notifications with filters (admin view)
    pub async fn count_unread_with_filters(
        &self,
        filter: &NotificationQueryFilter,
    ) -> Result<i64, AppError> {
        let mut qb = QueryBuilder::new(
            "SELECT COUNT(*) as count FROM wallet_notifications WHERE status != 'read' AND status != 'deleted'",
        );
        if let Some(ref wallet) = filter.wallet_address {
            qb.push(" AND recipient_wallet_address = ").push_bind(wallet.clone());
        }
        if let Some(ref notif_type) = filter.notification_type {
            qb.push(" AND notification_type = ").push_bind(notif_type.clone());
        }
        if let Some(ref priority) = filter.priority {
            qb.push(" AND priority = ").push_bind(priority.clone());
        }

        let row: (i64,) = qb
            .build_query_as()
            .fetch_one(self.pool.as_ref())
            .await
            .map_err(|e| {
                AppError::new(
                    ErrorKind::DatabaseError,
                    format!("Failed to count unread: {}", e),
                )
            })?;
        Ok(row.0)
    }

    /// Count unread notifications for specific wallet
    pub async fn count_unread_for_wallet(
        &self,
        wallet_address: &str,
        filter: &NotificationQueryFilter,
    ) -> Result<i64, AppError> {
        let escaped_wallet = wallet_address.to_lowercase();
        let mut qb = QueryBuilder::new(
            "SELECT COUNT(*) as count FROM wallet_notifications \
             WHERE status != 'deleted' AND status != 'read' \
               AND (LOWER(recipient_wallet_address) = ",
        );
        qb.push_bind(escaped_wallet);
        qb.push(" OR recipient_wallet_address = 'all')");

        if let Some(ref notif_type) = filter.notification_type {
            qb.push(" AND notification_type = ").push_bind(notif_type.clone());
        }
        if let Some(ref priority) = filter.priority {
            qb.push(" AND priority = ").push_bind(priority.clone());
        }

        let row: (i64,) = qb
            .build_query_as()
            .fetch_one(self.pool.as_ref())
            .await
            .map_err(|e| {
                AppError::new(
                    ErrorKind::DatabaseError,
                    format!("Failed to count unread: {}", e),
                )
            })?;
        Ok(row.0)
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
        let now = Utc::now();

        sqlx::query(
            r#"
            INSERT INTO wallet_notifications
            (id, recipient_wallet_address, notification_type, title, body, data_payload, priority, created_at, expires_at, action_url, image_url, status)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, 'created')
            "#,
        )
        .bind(id)
        .bind(wallet_address)
        .bind(notification_type)
        .bind(title)
        .bind(message)
        .bind(data.unwrap_or(serde_json::Value::Null))
        .bind(priority)
        .bind(now)
        .bind(expires_at)
        .bind(action_url)
        .bind(image_url)
        .execute(self.pool.as_ref())
        .await
        .map_err(|e| {
            AppError::new(
                ErrorKind::DatabaseError,
                format!("Failed to save notification: {}", e),
            )
        })?;

        Ok(())
    }

    /// Update delivery attempt - Updated for new schema
    pub async fn update_delivery_attempt(&self, id: Uuid) -> Result<(), AppError> {
        sqlx::query(
            "UPDATE wallet_notifications SET total_attempts = total_attempts + 1, updated_at = NOW() WHERE id = $1",
        )
        .bind(id)
        .execute(self.pool.as_ref())
        .await
        .map_err(|e| {
            AppError::new(
                ErrorKind::DatabaseError,
                format!("Failed to update delivery attempt: {}", e),
            )
        })?;
        Ok(())
    }

    /// Mark notification as read
    pub async fn mark_as_read(&self, id: Uuid, wallet_address: &str) -> Result<u64, AppError> {
        let now = Utc::now();

        let result = sqlx::query(
            r#"
            UPDATE wallet_notifications
            SET status = 'read', updated_at = $1
            WHERE id = $2 AND (LOWER(recipient_wallet_address) = LOWER($3) OR recipient_wallet_address = 'all')
            "#,
        )
        .bind(now)
        .bind(id)
        .bind(wallet_address)
        .execute(self.pool.as_ref())
        .await
        .map_err(|e| {
            AppError::new(
                ErrorKind::DatabaseError,
                format!("Failed to mark notification as read: {}", e),
            )
        })?;

        Ok(result.rows_affected())
    }

    /// Mark all notifications as read for wallet
    pub async fn mark_all_as_read(&self, wallet_address: &str) -> Result<u64, AppError> {
        let now = Utc::now();

        let result = sqlx::query(
            r#"
            UPDATE wallet_notifications
            SET status = 'read', updated_at = $1
            WHERE (LOWER(recipient_wallet_address) = LOWER($2) OR recipient_wallet_address = 'all') AND status != 'read' AND status != 'deleted'
            "#,
        )
        .bind(now)
        .bind(wallet_address)
        .execute(self.pool.as_ref())
        .await
        .map_err(|e| {
            AppError::new(
                ErrorKind::DatabaseError,
                format!("Failed to mark all notifications as read: {}", e),
            )
        })?;

        Ok(result.rows_affected())
    }

    /// Soft delete notification
    pub async fn soft_delete(&self, id: Uuid, wallet_address: &str) -> Result<u64, AppError> {
        let result = sqlx::query(
            r#"
            UPDATE wallet_notifications
            SET status = 'deleted', updated_at = NOW()
            WHERE id = $1 AND status != 'deleted' AND (LOWER(recipient_wallet_address) = LOWER($2) OR recipient_wallet_address = 'all')
            "#,
        )
        .bind(id)
        .bind(wallet_address)
        .execute(self.pool.as_ref())
        .await
        .map_err(|e| {
            AppError::new(
                ErrorKind::DatabaseError,
                format!("Failed to delete notification: {}", e),
            )
        })?;

        Ok(result.rows_affected())
    }

    /// Soft delete all notifications for wallet
    pub async fn soft_delete_all(&self, wallet_address: &str) -> Result<u64, AppError> {
        let result = sqlx::query(
            r#"
            UPDATE wallet_notifications
            SET status = 'deleted', updated_at = NOW()
            WHERE (LOWER(recipient_wallet_address) = LOWER($1) OR recipient_wallet_address = 'all') AND status != 'deleted'
            "#,
        )
        .bind(wallet_address)
        .execute(self.pool.as_ref())
        .await
        .map_err(|e| {
            AppError::new(
                ErrorKind::DatabaseError,
                format!("Failed to clear all notifications: {}", e),
            )
        })?;

        Ok(result.rows_affected())
    }

    /// Hard delete notification (admin only)
    pub async fn hard_delete(&self, id: Uuid) -> Result<u64, AppError> {
        let result = sqlx::query("DELETE FROM wallet_notifications WHERE id = $1")
            .bind(id)
            .execute(self.pool.as_ref())
            .await
            .map_err(|e| {
                AppError::new(
                    ErrorKind::DatabaseError,
                    format!("Failed to delete notification: {}", e),
                )
            })?;

        Ok(result.rows_affected())
    }

    /// Get simple unread count for wallet
    pub async fn get_unread_count(&self, wallet_address: &str) -> Result<i64, AppError> {
        let row: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) as count FROM wallet_notifications \
             WHERE (LOWER(recipient_wallet_address) = LOWER($1) OR recipient_wallet_address = 'all') \
               AND status != 'read' AND status != 'deleted'",
        )
        .bind(wallet_address)
        .fetch_one(self.pool.as_ref())
        .await
        .map_err(|e| {
            AppError::new(
                ErrorKind::DatabaseError,
                format!("Failed to count unread notifications: {}", e),
            )
        })?;

        Ok(row.0)
    }
}

#[cfg(test)]
mod tests {
    use super::WalletNotificationIdentity;

    #[test]
    fn test_repository_creation() {
        // Placeholder test to ensure module compiles
    }

    #[test]
    fn stable_event_identity_requires_an_exact_payload_match() {
        let identity = WalletNotificationIdentity {
            wallet_address: "0xabc".into(),
            notification_type: "payment".into(),
            title: "Paid".into(),
            message: "Complete".into(),
            data: Some(serde_json::json!({"reference": "p-1"})),
            priority: "normal".into(),
            action_url: Some("/payments".into()),
            expires_at: None,
        };
        let data = serde_json::json!({"reference": "p-1"});
        assert!(identity.matches_payload(
            "0xabc",
            "payment",
            "normal",
            "Paid",
            "Complete",
            Some(&data),
            Some("/payments"),
            None,
        ));
        assert!(!identity.matches_payload(
            "0xdef",
            "payment",
            "normal",
            "Paid",
            "Complete",
            Some(&data),
            Some("/payments"),
            None,
        ));
        assert!(!identity.matches_payload(
            "0xabc",
            "payment",
            "normal",
            "Paid",
            "Changed",
            Some(&data),
            Some("/payments"),
            None,
        ));
    }
}
