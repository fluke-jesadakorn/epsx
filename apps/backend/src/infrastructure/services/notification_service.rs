//! Reusable notification service
//!
//! Encapsulates DB insert + Redis publish for sending notifications.
//! Used by automatic triggers (plan changes, chat, payments) to avoid
//! duplicating notification logic across handlers.

use chrono::Utc;
use std::sync::Arc;
use uuid::Uuid;

use crate::core::errors::AppError;
use crate::web::auth::AppState;
use crate::web::admin::wallet_notification_repository::WalletNotificationRepository;
use crate::web::notifications::{NotificationType, NotificationPriority, SSENotification};

pub struct NotificationService;

impl NotificationService {
    /// Send a notification to a specific wallet address.
    /// Persists to DB and publishes via Redis for real-time delivery.
    pub async fn send(
        app_state: &AppState,
        wallet_address: &str,
        notification_type: NotificationType,
        priority: NotificationPriority,
        title: &str,
        message: &str,
        data: Option<serde_json::Value>,
        action_url: Option<String>,
    ) -> Result<String, AppError> {
        let id = Uuid::new_v4();
        let notif_type = format!("{:?}", notification_type).to_lowercase();
        let notif_priority = format!("{:?}", priority).to_lowercase();
        let wallet = wallet_address.to_lowercase();

        // Get notifications DB pool
        let pool = if let Ok(p) = crate::infrastructure::database::get_notifications_pool().await {
            Arc::new(p)
        } else {
            app_state.db_pool.clone()
        };
        let repo = WalletNotificationRepository::new(pool);

        // Persist to database
        repo.create(
            id,
            &wallet,
            &notif_type,
            title,
            message,
            data.clone(),
            &notif_priority,
            None,
            action_url.clone(),
            None,
        ).await?;

        // Build SSE notification
        let sse = SSENotification {
            id: id.to_string(),
            wallet_address: wallet.clone(),
            notification_type,
            title: title.to_string(),
            message: message.to_string(),
            data,
            priority,
            timestamp: Utc::now(),
            expires_at: None,
        };

        // Publish via Redis (fire-and-forget, don't fail the parent operation)
        if let Some(broadcaster) = &app_state.redis_broadcaster {
            if let Err(e) = broadcaster.publish_to_wallet(&wallet, &sse).await {
                tracing::warn!("Failed to publish notification via Redis: {}", e);
            }
        }

        Ok(id.to_string())
    }

    /// Broadcast a notification to all users.
    /// Persists to DB with wallet_address='all' and publishes via Redis broadcast channel.
    pub async fn broadcast(
        app_state: &AppState,
        notification_type: NotificationType,
        priority: NotificationPriority,
        title: &str,
        message: &str,
        data: Option<serde_json::Value>,
    ) -> Result<String, AppError> {
        let id = Uuid::new_v4();
        let notif_type = format!("{:?}", notification_type).to_lowercase();
        let notif_priority = format!("{:?}", priority).to_lowercase();

        // Get notifications DB pool
        let pool = if let Ok(p) = crate::infrastructure::database::get_notifications_pool().await {
            Arc::new(p)
        } else {
            app_state.db_pool.clone()
        };
        let repo = WalletNotificationRepository::new(pool);

        // Persist to database
        repo.create(
            id,
            "all",
            &notif_type,
            title,
            message,
            data.clone(),
            &notif_priority,
            None,
            None,
            None,
        ).await?;

        // Build SSE notification
        let sse = SSENotification {
            id: id.to_string(),
            wallet_address: "all".to_string(),
            notification_type,
            title: title.to_string(),
            message: message.to_string(),
            data,
            priority,
            timestamp: Utc::now(),
            expires_at: None,
        };

        // Publish via Redis broadcast
        if let Some(broadcaster) = &app_state.redis_broadcaster {
            if let Err(e) = broadcaster.publish_to_all(&sse).await {
                tracing::warn!("Failed to broadcast notification via Redis: {}", e);
            }
        }

        Ok(id.to_string())
    }
}
