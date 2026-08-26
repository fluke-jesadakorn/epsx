use crate::prelude::TlsPool;
use crate::web::notifications::{NotificationPriority, NotificationType, SSENotification};
use epsx_contracts::errors::AppError;
use uuid::Uuid;

/// Fetch all active notifications for a wallet (offline queue)
/// Returns notifications that persist until user explicitly deletes them
/// Includes both read and unread notifications from the last 30 days
///
/// Behavior:
/// - Notifications persist across login sessions until user deletes
/// - Shows all notifications (read and unread) for continuity
/// - Filters out soft-deleted notifications (status = 'deleted')
/// - Limits to last 30 days to prevent fetching excessive old data
/// - Excludes expired notifications
pub async fn fetch_queued_notifications(
    db_pool: &TlsPool,
    wallet_address: &str,
) -> Result<Vec<SSENotification>, AppError> {
    let mut conn = db_pool
        .acquire()
        .await
        .map_err(|e| AppError::database_error(format!("Connection pool error: {}", e)))?;

    #[derive(sqlx::FromRow)]
    struct NotificationRow {
        id: Uuid,
        wallet_address: String,
        notification_type: String,
        title: String,
        message: String,
        data: Option<serde_json::Value>,
        priority: String,
        timestamp: chrono::DateTime<chrono::Utc>,
        expires_at: Option<chrono::DateTime<chrono::Utc>>,
    }

    let records: Vec<NotificationRow> = sqlx::query_as::<_, NotificationRow>(
        r#"
        SELECT
            id, recipient_wallet_address as wallet_address, notification_type, title, body as message,
            data_payload as data, priority, created_at as timestamp, expires_at
        FROM wallet_notifications
        WHERE (recipient_wallet_address = $1 OR recipient_wallet_address = 'all')
          AND status != 'deleted'
          AND created_at > NOW() - INTERVAL '30 days'
          AND (expires_at IS NULL OR expires_at > NOW())
        ORDER BY created_at DESC
        LIMIT 100
        "#,
    )
    .bind(wallet_address.to_lowercase())
    .fetch_all(&mut *conn)
    .await
    .map_err(|e| AppError::database_error(format!("Failed to fetch notifications: {}", e)))?;

    let notifications: Vec<_> = records
        .into_iter()
        .map(|r| SSENotification {
            id: r.id.to_string(),
            wallet_address: r.wallet_address,
            notification_type: parse_notification_type(&r.notification_type, &r.id),
            title: r.title,
            message: r.message,
            data: r.data,
            priority: parse_priority(&r.priority, &r.id),
            timestamp: r.timestamp,
            expires_at: r.expires_at,
        })
        .collect();

    tracing::info!(
        "Fetched {} active notifications (last 30 days) for notification stream",
        notifications.len()
    );

    Ok(notifications)
}

/// Mark notification as delivered (SSE stream sent it to the client).
/// Only transitions from undelivered states (created/queued/sent) so it never
/// clobbers a user-explicitly set state like 'read' or 'unread'.
pub async fn mark_as_delivered(db_pool: &TlsPool, notification_id: &str) -> Result<(), AppError> {
    let id = Uuid::parse_str(notification_id)
        .map_err(|e| AppError::from(Box::new(e) as Box<dyn std::error::Error>))?;

    let mut conn = db_pool
        .acquire()
        .await
        .map_err(|e| AppError::database_error(format!("Connection pool error: {}", e)))?;

    sqlx::query(
        "UPDATE wallet_notifications \
         SET status = 'delivered', total_attempts = total_attempts + 1, updated_at = NOW() \
         WHERE id = $1 AND status IN ('created', 'queued', 'sent')",
    )
    .bind(id)
    .execute(&mut *conn)
    .await
    .map_err(|e| AppError::database_error(format!("Failed to mark notification as delivered: {}", e)))?;

    Ok(())
}

/// Mark notification as acknowledged by client (called automatically on SSE receipt).
/// Only marks as 'delivered' when currently in an undelivered state.
/// Never overrides user-explicit 'read' or 'unread' states — those are set
/// only through the explicit mark-as-read / mark-as-unread API endpoints.
pub async fn mark_as_acknowledged(
    db_pool: &TlsPool,
    notification_id: &str,
) -> Result<(), AppError> {
    let id = Uuid::parse_str(notification_id)
        .map_err(|e| AppError::from(Box::new(e) as Box<dyn std::error::Error>))?;

    let mut conn = db_pool
        .acquire()
        .await
        .map_err(|e| AppError::database_error(format!("Connection pool error: {}", e)))?;

    // Only update if not already in a user-controlled state ('read' or 'unread').
    // This prevents the SSE auto-acknowledge from undoing an explicit markAsUnread.
    sqlx::query(
        "UPDATE wallet_notifications \
         SET status = 'delivered', updated_at = NOW() \
         WHERE id = $1 AND status NOT IN ('read', 'unread', 'deleted')",
    )
    .bind(id)
    .execute(&mut *conn)
    .await
    .map_err(|e| AppError::database_error(format!("Failed to ack notification: {}", e)))?;

    tracing::debug!(
        "Notification acknowledged (delivery confirmed): id={}",
        notification_id
    );

    Ok(())
}

/// Cleanup old notifications with smart deletion rules
///
/// Deletion Strategy:
/// - Soft-deleted notifications: Remove after 7 days (allows undo within grace period)
/// - Read notifications: Remove after 90 days (archived)
/// - Unread notifications: Keep indefinitely (user might still want to see them)
/// - Expired notifications: Remove immediately
///
/// Called every hour by `PlanExpirationService` background task (main.rs).
pub async fn cleanup_old_notifications(db_pool: &TlsPool, _days: i64) -> Result<u64, AppError> {
    let mut conn = db_pool.acquire().await.map_err(|e| {
        AppError::database_error(format!("Failed to get database connection: {}", e))
    })?;

    // Delete soft-deleted notifications after grace period (7 days)
    let soft_deleted_result = sqlx::query(
        "DELETE FROM wallet_notifications WHERE status = 'deleted' AND updated_at < NOW() - INTERVAL '7 days'"
    )
    .execute(&mut *conn)
    .await
    .map_err(|e| AppError::database_error(format!("Failed to delete soft-deleted: {}", e)))?;

    // Delete old read notifications (90 days)
    let read_result = sqlx::query(
        "DELETE FROM wallet_notifications WHERE status = 'read' AND created_at < NOW() - INTERVAL '90 days'"
    )
    .execute(&mut *conn)
    .await
    .map_err(|e| AppError::database_error(format!("Failed to delete read: {}", e)))?;

    // Delete expired notifications immediately
    let expired_result = sqlx::query(
        "DELETE FROM wallet_notifications WHERE expires_at IS NOT NULL AND expires_at < NOW()",
    )
    .execute(&mut *conn)
    .await
    .map_err(|e| AppError::database_error(format!("Failed to delete expired: {}", e)))?;

    let total_cleaned = soft_deleted_result.rows_affected()
        + read_result.rows_affected()
        + expired_result.rows_affected();

    tracing::info!(
        "Cleaned up {} notifications (soft-deleted: {}, read: {}, expired: {})",
        total_cleaned,
        soft_deleted_result.rows_affected(),
        read_result.rows_affected(),
        expired_result.rows_affected()
    );

    Ok(total_cleaned as u64)
}

/// Get notification statistics for monitoring (excludes soft-deleted)
pub async fn get_notification_stats(db_pool: &TlsPool) -> Result<NotificationStats, AppError> {
    let mut conn = db_pool.acquire().await.map_err(|e| {
        AppError::database_error(format!("Failed to get database connection: {}", e))
    })?;

    #[derive(sqlx::FromRow)]
    struct CountRow {
        count: i64,
    }

    let total: i64 = sqlx::query_as::<_, CountRow>(
        "SELECT COUNT(*) as count FROM wallet_notifications WHERE status != 'deleted'",
    )
    .fetch_one(&mut *conn)
    .await
    .map_err(|e| AppError::database_error(format!("Failed to count: {}", e)))?
    .count;

    let queued: i64 = sqlx::query_as::<_, CountRow>("SELECT COUNT(*) as count FROM wallet_notifications WHERE status IN ('created', 'queued') AND status != 'deleted'")
        .fetch_one(&mut *conn)
        .await
        .map_err(|e| AppError::database_error(format!("Failed to count queued: {}", e)))?
        .count;

    let delivered: i64 = sqlx::query_as::<_, CountRow>("SELECT COUNT(*) as count FROM wallet_notifications WHERE status IN ('sent', 'delivered') AND status != 'deleted'")
        .fetch_one(&mut *conn)
        .await
        .map_err(|e| AppError::database_error(format!("Failed to count delivered: {}", e)))?
        .count;

    let acknowledged: i64 = sqlx::query_as::<_, CountRow>("SELECT COUNT(*) as count FROM wallet_notifications WHERE status = 'read' AND status != 'deleted'")
        .fetch_one(&mut *conn)
        .await
        .map_err(|e| AppError::database_error(format!("Failed to count acknowledged: {}", e)))?
        .count;

    Ok(NotificationStats {
        total: total as usize,
        queued: queued as usize,
        delivered: delivered as usize,
        acknowledged: acknowledged as usize,
    })
}

#[derive(Debug, serde::Serialize)]
pub struct NotificationStats {
    pub total: usize,
    pub queued: usize,
    pub delivered: usize,
    pub acknowledged: usize,
}

// Helper functions to parse database values with logging

fn parse_notification_type(s: &str, notification_id: &Uuid) -> NotificationType {
    match s {
        "security" => NotificationType::Security,
        "permission" => NotificationType::Permission,
        "wallet_management" => NotificationType::WalletManagement,
        "wallet" => NotificationType::Wallet,
        "payment" => NotificationType::Payment,
        "general" => NotificationType::General,
        "system" => NotificationType::System,
        "announcement" => NotificationType::Announcement,
        "advertisement" => NotificationType::Advertisement,
        "chat" => NotificationType::Chat,
        _ => {
            tracing::warn!(
                "Data quality issue: Invalid notification_type '{}' for notification id={}, defaulting to System",
                s,
                notification_id
            );
            NotificationType::System
        }
    }
}

fn parse_priority(s: &str, notification_id: &Uuid) -> NotificationPriority {
    match s {
        "low" => NotificationPriority::Low,
        "normal" => NotificationPriority::Normal,
        "high" => NotificationPriority::High,
        "critical" => NotificationPriority::Critical,
        "urgent" => {
            tracing::debug!(
                "Mapping deprecated priority 'urgent' to 'critical' for notification id={}",
                notification_id
            );
            NotificationPriority::Critical
        }
        _ => {
            tracing::warn!(
                "Data quality issue: Invalid priority '{}' for notification id={}, defaulting to Normal",
                s,
                notification_id
            );
            NotificationPriority::Normal
        }
    }
}
