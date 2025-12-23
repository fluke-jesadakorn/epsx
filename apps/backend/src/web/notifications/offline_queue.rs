use diesel::prelude::*;
use diesel_async::{AsyncPgConnection, RunQueryDsl, pooled_connection::deadpool::Pool};
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
    db_pool: &Pool<AsyncPgConnection>,
    wallet_address: &str,
) -> Result<Vec<SSENotification>, AppError> {
    let mut conn = db_pool.get().await
        .map_err(|e| AppError::database_error(format!("Connection pool error: {}", e)))?;

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
        timestamp: chrono::DateTime<chrono::Utc>,
        #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Timestamptz>)]
        expires_at: Option<chrono::DateTime<chrono::Utc>>,
    }

    let records = diesel::sql_query(
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
        "#
    )
    .bind::<diesel::sql_types::Text, _>(wallet_address.to_lowercase())
    .load::<NotificationRow>(&mut conn)
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
    db_pool: &Pool<AsyncPgConnection>,
    notification_id: &str,
) -> Result<(), AppError> {
    let id = Uuid::parse_str(notification_id)
        .map_err(|e| AppError::from(Box::new(e) as Box<dyn std::error::Error>))?;

    let mut conn = db_pool.get().await
        .map_err(|e| AppError::database_error(format!("Connection pool error: {}", e)))?;

    diesel::sql_query(
        "UPDATE wallet_notifications SET delivered_at = NOW(), delivery_attempts = delivery_attempts + 1 WHERE id = $1"
    )
    .bind::<diesel::sql_types::Uuid, _>(id)
    .execute(&mut conn)
    .await?;

    Ok(())
}

/// Mark notification as acknowledged by client
pub async fn mark_as_acknowledged(
    db_pool: &Pool<AsyncPgConnection>,
    notification_id: &str,
) -> Result<(), AppError> {
    let id = Uuid::parse_str(notification_id)
        .map_err(|e| AppError::from(Box::new(e) as Box<dyn std::error::Error>))?;

    let mut conn = db_pool.get().await
        .map_err(|e| AppError::database_error(format!("Connection pool error: {}", e)))?;

    diesel::sql_query(
        "UPDATE wallet_notifications SET acknowledged_at = NOW() WHERE id = $1"
    )
    .bind::<diesel::sql_types::Uuid, _>(id)
    .execute(&mut conn)
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
    db_pool: &Pool<AsyncPgConnection>,
    _days: i64,
) -> Result<u64, AppError> {
    let mut conn = db_pool.get().await.map_err(|e| {
        AppError::database_error(format!("Failed to get database connection: {}", e))
    })?;

    // Delete soft-deleted notifications after grace period (7 days)
    let soft_deleted_result = diesel::sql_query(
        "DELETE FROM wallet_notifications WHERE deleted_at IS NOT NULL AND deleted_at < NOW() - INTERVAL '7 days'"
    )
    .execute(&mut conn)
    .await?;

    // Delete old read notifications (90 days)
    let read_result = diesel::sql_query(
        "DELETE FROM wallet_notifications WHERE read_at IS NOT NULL AND deleted_at IS NULL AND created_at < NOW() - INTERVAL '90 days'"
    )
    .execute(&mut conn)
    .await?;

    // Delete expired notifications immediately
    let expired_result = diesel::sql_query(
        "DELETE FROM wallet_notifications WHERE expires_at IS NOT NULL AND expires_at < NOW()"
    )
    .execute(&mut conn)
    .await?;

    let total_cleaned = soft_deleted_result
        + read_result
        + expired_result;

    tracing::info!(
        "🧹 Cleaned up {} notifications (soft-deleted: {}, read: {}, expired: {})",
        total_cleaned,
        soft_deleted_result,
        read_result,
        expired_result
    );

    Ok(total_cleaned as u64)
}

/// Get notification statistics for monitoring (excludes soft-deleted)
pub async fn get_notification_stats(
    db_pool: &Pool<AsyncPgConnection>,
) -> Result<NotificationStats, AppError> {
    let mut conn = db_pool.get().await.map_err(|e| {
        AppError::database_error(format!("Failed to get database connection: {}", e))
    })?;

    use diesel::sql_types::BigInt;

    #[derive(diesel::QueryableByName)]
    struct CountRow {
        #[diesel(sql_type = BigInt)]
        count: i64,
    }

    let total: CountRow = diesel::sql_query("SELECT COUNT(*) as count FROM wallet_notifications WHERE deleted_at IS NULL")
        .get_result(&mut conn)
        .await?;

    let queued: CountRow = diesel::sql_query("SELECT COUNT(*) as count FROM wallet_notifications WHERE delivered_at IS NULL AND deleted_at IS NULL")
        .get_result(&mut conn)
        .await?;

    let delivered: CountRow = diesel::sql_query("SELECT COUNT(*) as count FROM wallet_notifications WHERE delivered_at IS NOT NULL AND deleted_at IS NULL")
        .get_result(&mut conn)
        .await?;

    let acknowledged: CountRow = diesel::sql_query("SELECT COUNT(*) as count FROM wallet_notifications WHERE acknowledged_at IS NOT NULL AND deleted_at IS NULL")
        .get_result(&mut conn)
        .await?;

    Ok(NotificationStats {
        total: total.count as usize,
        queued: queued.count as usize,
        delivered: delivered.count as usize,
        acknowledged: acknowledged.count as usize,
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
