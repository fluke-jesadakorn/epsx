use sqlx::PgPool;
use uuid::Uuid;
use crate::web::notifications::{SSENotification, NotificationType, NotificationPriority};
use crate::core::errors::AppError;

/// Fetch all active notifications for a wallet (offline queue)
/// Returns notifications that persist until user explicitly deletes them
/// Includes both read and unread notifications from the last 30 days
///
/// Behavior:
/// - Notifications persist across login sessions until user deletes
/// - Shows all notifications (read and unread) for continuity
/// - Filters out soft-deleted notifications (deleted_at IS NOT NULL)
/// - Limits to last 30 days to prevent fetching excessive old data
/// - Excludes expired notifications
pub async fn fetch_queued_notifications(
    db_pool: &PgPool,
    wallet_address: &str,
) -> Result<Vec<SSENotification>, AppError> {
    let records = sqlx::query!(
        r#"
        SELECT
            id, wallet_address, notification_type, title, message,
            data, priority, timestamp, expires_at
        FROM wallet_notifications
        WHERE (wallet_address = $1 OR wallet_address = 'all')
          AND deleted_at IS NULL
          AND created_at > NOW() - INTERVAL '30 days'
          AND (expires_at IS NULL OR expires_at > NOW())
        ORDER BY timestamp DESC
        LIMIT 100
        "#,
        wallet_address.to_lowercase()
    )
    .fetch_all(db_pool)
    .await?;

    let notifications: Vec<_> = records
        .into_iter()
        .map(|r| {
            SSENotification {
                id: r.id.to_string(),
                wallet_address: r.wallet_address,
                notification_type: parse_notification_type(&r.notification_type, &r.id),
                title: r.title,
                message: r.message,
                data: r.data,
                priority: parse_priority(&r.priority, &r.id),
                timestamp: r.timestamp,
                expires_at: r.expires_at,
            }
        })
        .collect();

    tracing::info!(
        "📦 Fetched {} active notifications (last 30 days) for wallet: {}",
        notifications.len(),
        wallet_address
    );

    Ok(notifications)
}

/// Mark notification as delivered via Redis
pub async fn mark_as_delivered(
    db_pool: &PgPool,
    notification_id: &str,
) -> Result<(), AppError> {
    let id = Uuid::parse_str(notification_id)
        .map_err(|e| AppError::from(Box::new(e) as Box<dyn std::error::Error>))?;

    sqlx::query!(
        "UPDATE wallet_notifications SET delivered_at = NOW(), delivery_attempts = delivery_attempts + 1 WHERE id = $1",
        id
    )
    .execute(db_pool)
    .await?;

    Ok(())
}

/// Mark notification as acknowledged by client
pub async fn mark_as_acknowledged(
    db_pool: &PgPool,
    notification_id: &str,
) -> Result<(), AppError> {
    let id = Uuid::parse_str(notification_id)
        .map_err(|e| AppError::from(Box::new(e) as Box<dyn std::error::Error>))?;

    sqlx::query!(
        "UPDATE wallet_notifications SET acknowledged_at = NOW() WHERE id = $1",
        id
    )
    .execute(db_pool)
    .await?;

    tracing::debug!("✅ Notification acknowledged: id={}", notification_id);

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
/// This function is designed to be called by a cron job (not implemented)
pub async fn cleanup_old_notifications(
    db_pool: &PgPool,
    _days: i64,
) -> Result<u64, AppError> {
    // Delete soft-deleted notifications after grace period (7 days)
    let soft_deleted_result = sqlx::query!(
        "DELETE FROM wallet_notifications WHERE deleted_at IS NOT NULL AND deleted_at < NOW() - INTERVAL '7 days'"
    )
    .execute(db_pool)
    .await?;

    // Delete old read notifications (90 days)
    let read_result = sqlx::query!(
        "DELETE FROM wallet_notifications WHERE read_at IS NOT NULL AND deleted_at IS NULL AND created_at < NOW() - INTERVAL '90 days'"
    )
    .execute(db_pool)
    .await?;

    // Delete expired notifications immediately
    let expired_result = sqlx::query!(
        "DELETE FROM wallet_notifications WHERE expires_at IS NOT NULL AND expires_at < NOW()"
    )
    .execute(db_pool)
    .await?;

    let total_cleaned = soft_deleted_result.rows_affected()
        + read_result.rows_affected()
        + expired_result.rows_affected();

    tracing::info!(
        "🧹 Cleaned up {} notifications (soft-deleted: {}, read: {}, expired: {})",
        total_cleaned,
        soft_deleted_result.rows_affected(),
        read_result.rows_affected(),
        expired_result.rows_affected()
    );

    Ok(total_cleaned)
}

/// Get notification statistics for monitoring (excludes soft-deleted)
pub async fn get_notification_stats(
    db_pool: &PgPool,
) -> Result<NotificationStats, AppError> {
    let total: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM wallet_notifications WHERE deleted_at IS NULL")
        .fetch_one(db_pool)
        .await?;

    let queued: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM wallet_notifications WHERE delivered_at IS NULL AND deleted_at IS NULL")
        .fetch_one(db_pool)
        .await?;

    let delivered: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM wallet_notifications WHERE delivered_at IS NOT NULL AND deleted_at IS NULL")
        .fetch_one(db_pool)
        .await?;

    let acknowledged: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM wallet_notifications WHERE acknowledged_at IS NOT NULL AND deleted_at IS NULL")
        .fetch_one(db_pool)
        .await?;

    Ok(NotificationStats {
        total: total.0 as usize,
        queued: queued.0 as usize,
        delivered: delivered.0 as usize,
        acknowledged: acknowledged.0 as usize,
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
        _ => {
            tracing::warn!(
                "⚠️ Data quality issue: Invalid notification_type '{}' for notification id={}, defaulting to System",
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
                "⚠️ Data quality issue: Invalid priority '{}' for notification id={}, defaulting to Normal",
                s,
                notification_id
            );
            NotificationPriority::Normal
        }
    }
}
