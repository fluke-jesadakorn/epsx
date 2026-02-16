use redis::AsyncCommands;
use std::sync::Arc;
use crate::web::notifications::SSENotification;
use crate::infrastructure::redis::RedisPool;
use crate::core::errors::{AppError, ErrorKind};

#[derive(Clone)]
pub struct RedisNotificationBroadcaster {
    pool: Arc<RedisPool>,
}

impl RedisNotificationBroadcaster {
    pub fn new(pool: Arc<RedisPool>) -> Self {
        Self { pool }
    }

    /// Publish notification to specific wallet via Redis pub/sub
    pub async fn publish_to_wallet(
        &self,
        wallet_address: &str,
        notification: &SSENotification,
    ) -> Result<usize, AppError> {
        let channel = format!("notifications:wallet:{}", wallet_address.to_lowercase());
        let payload = serde_json::to_string(notification)
            .map_err(|e| AppError::new(
                ErrorKind::InternalError,
                format!("Failed to serialize notification: {}", e)
            ))?;

        let mut conn = self.pool.get_connection();

        let subscriber_count: i32 = conn.publish(&channel, &payload).await
            .map_err(|e| AppError::new(
                ErrorKind::InternalError,
                format!("Redis publish failed: {}", e)
            ))?;

        tracing::info!(
            "Published notification to Redis: wallet={}, channel={}, subscribers={}, id={}",
            wallet_address,
            channel,
            subscriber_count,
            notification.id
        );

        Ok(subscriber_count as usize)
    }

    /// Publish notification to all users (broadcast via Redis pub/sub)
    pub async fn publish_to_all(
        &self,
        notification: &SSENotification,
    ) -> Result<usize, AppError> {
        let channel = "notifications:all";
        let payload = serde_json::to_string(notification)
            .map_err(|e| AppError::new(
                ErrorKind::InternalError,
                format!("Failed to serialize notification: {}", e)
            ))?;

        let mut conn = self.pool.get_connection();

        let subscriber_count: i32 = conn.publish(channel, &payload).await
            .map_err(|e| AppError::new(
                ErrorKind::InternalError,
                format!("Redis publish failed: {}", e)
            ))?;

        tracing::info!(
            "Broadcast notification to Redis: channel={}, subscribers={}, id={}",
            channel,
            subscriber_count,
            notification.id
        );

        Ok(subscriber_count as usize)
    }

    /// Subscribe to wallet-specific notifications via Redis pub/sub
    pub async fn subscribe_to_wallet(
        &self,
        wallet_address: &str,
    ) -> Result<redis::aio::PubSub, AppError> {
        let mut pubsub = self.pool.get_pubsub().await
            .map_err(|e| AppError::new(
                ErrorKind::InternalError,
                format!("Redis pubsub connection failed: {}", e)
            ))?;

        // Subscribe to wallet-specific channel
        let wallet_channel = format!("notifications:wallet:{}", wallet_address.to_lowercase());
        pubsub.subscribe(&wallet_channel).await
            .map_err(|e| AppError::new(
                ErrorKind::InternalError,
                format!("Redis subscribe to {} failed: {}", wallet_channel, e)
            ))?;

        // Also subscribe to broadcast channel
        pubsub.subscribe("notifications:all").await
            .map_err(|e| AppError::new(
                ErrorKind::InternalError,
                format!("Redis subscribe to notifications:all failed: {}", e)
            ))?;

        tracing::info!(
            "Subscribed to Redis channels: wallet={}, channels=[{}, notifications:all]",
            wallet_address,
            wallet_channel
        );

        Ok(pubsub)
    }

    /// Get active subscriber count for a wallet channel
    pub async fn get_subscriber_count(&self, wallet_address: &str) -> Result<usize, AppError> {
        use redis::cmd;

        let channel = format!("notifications:wallet:{}", wallet_address.to_lowercase());
        let mut conn = self.pool.get_connection();

        // PUBSUB NUMSUB returns [channel, count, channel, count, ...]
        let result: Vec<redis::Value> = cmd("PUBSUB")
            .arg("NUMSUB")
            .arg(&channel)
            .query_async(&mut conn)
            .await
            .unwrap_or_default();

        // Extract count from result (second element)
        let count = if result.len() >= 2 {
            match &result[1] {
                redis::Value::Int(n) => *n as usize,
                _ => 0,
            }
        } else {
            0
        };

        Ok(count)
    }
}
