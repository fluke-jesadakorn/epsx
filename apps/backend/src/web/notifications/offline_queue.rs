use sqlx::PgPool;
use uuid::Uuid;
use crate::web::notifications::{SSENotification, NotificationType, NotificationPriority};
use crate::core::errors::AppError;

/// Fetch undelivered notifications for a wallet (offline queue)
/// Returns notifications that were sent while the user was offline
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
          AND delivered_at IS NULL
          AND (expires_at IS NULL OR expires_at > NOW())
        ORDER BY timestamp ASC
        LIMIT 100
        "#,
        wallet_address.to_lowercase()
    )
    .fetch_all(db_pool)
    .await?;

    let notifications: Vec<_> = records.into_iter().map(|r| {
        SSENotification {
            id: r.id.to_string(),
            wallet_address: r.wallet_address,
            notification_type: parse_notification_type(&r.notification_type),
            title: r.title,
            message: r.message,
            data: r.data,
            priority: parse_priority(&r.priority),
            timestamp: r.timestamp,
            expires_at: r.expires_at,
        }
    }).collect();

    tracing::info!(
        "📦 Fetched {} queued notifications for wallet: {}",
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

/// Cleanup old delivered notifications (older than specified days)
pub async fn cleanup_old_notifications(
    db_pool: &PgPool,
    days: i64,
) -> Result<u64, AppError> {
    let result = sqlx::query!(
        "DELETE FROM wallet_notifications WHERE created_at < NOW() - INTERVAL '1 day' * $1 AND delivered_at IS NOT NULL",
        days as f64
    )
    .execute(db_pool)
    .await?;

    tracing::info!("🧹 Cleaned up {} old notifications (older than {} days)", result.rows_affected(), days);

    Ok(result.rows_affected())
}

/// Get notification statistics for monitoring
pub async fn get_notification_stats(
    db_pool: &PgPool,
) -> Result<NotificationStats, AppError> {
    let total: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM wallet_notifications")
        .fetch_one(db_pool)
        .await?;

    let queued: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM wallet_notifications WHERE delivered_at IS NULL")
        .fetch_one(db_pool)
        .await?;

    let delivered: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM wallet_notifications WHERE delivered_at IS NOT NULL")
        .fetch_one(db_pool)
        .await?;

    let acknowledged: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM wallet_notifications WHERE acknowledged_at IS NOT NULL")
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

// Helper functions to parse database values

fn parse_notification_type(s: &str) -> NotificationType {
    match s {
        "security" => NotificationType::Security,
        "permission" => NotificationType::Permission,
        "wallet_management" => NotificationType::WalletManagement,
        "wallet" => NotificationType::Wallet,
        "payment" => NotificationType::Payment,
        "general" => NotificationType::General,
        _ => NotificationType::System,
    }
}

fn parse_priority(s: &str) -> NotificationPriority {
    match s {
        "low" => NotificationPriority::Low,
        "high" => NotificationPriority::High,
        "critical" => NotificationPriority::Critical,
        _ => NotificationPriority::Normal,
    }
}
