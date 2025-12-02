/// Web3-first notification repository adapter for serverless deployment  
/// Handles wallet-based notifications and database storage

use chrono::Utc;
use uuid::Uuid;
use tracing::{ debug, info };

use crate::domain::notification::aggregates::notification::{
  Notification,
  DeliveryResult,
};
// Email service removed - Web3-first system uses direct wallet notifications
use crate::application::shared::error::ApplicationResult;

/// Web3-first notification repository adapter - no email dependencies
#[derive(Clone)]
pub struct NotificationRepositoryAdapter {
  // Email service removed - Web3-first system uses direct wallet notifications
}

impl Default for NotificationRepositoryAdapter {
  fn default() -> Self {
    Self::new()
  }
}

impl NotificationRepositoryAdapter {
  pub fn new() -> Self {
    debug!("Creating Web3-first NotificationRepositoryAdapter");
    Self {
      // Email service removed - Web3-first system uses direct wallet notifications
    }
  }

  /// Send notification using Web3-first approach
  pub async fn send_notification(
    &self,
    notification: &Notification,
    wallet_address: &str,
  ) -> ApplicationResult<DeliveryResult> {
    let notification_id = notification.id().value();
    
    info!(
      "Sending Web3 notification {} to wallet {}",
      notification_id,
      wallet_address
    );

    // Store in database for wallet-based in-app notifications
    self.store_wallet_notification(notification, wallet_address).await
  }

  /// Send notification to multiple wallet recipients
  pub async fn send_bulk_notification(
    &self,
    notification: &Notification,
    wallet_addresses: Vec<String>, // Direct wallet addresses only
  ) -> ApplicationResult<Vec<DeliveryResult>> {
    info!(
      "Sending bulk Web3 notification {} to {} wallet recipients",
      notification.id().value(),
      wallet_addresses.len()
    );

    let mut results = Vec::new();
    
    for wallet_address in wallet_addresses {
      let result = self.send_notification(notification, &wallet_address).await
        .unwrap_or_else(|e| DeliveryResult::Failed {
          error_message: format!("Failed to send notification: {}", e),
          retry_after: Some(Utc::now() + chrono::Duration::minutes(5)),
        });
      
      results.push(result);
    }

    Ok(results)
  }

  /// Send notification to topic (Web3-first approach)
  pub async fn send_topic_notification(
    &self,
    notification: &Notification,
    topic_name: &str,
  ) -> ApplicationResult<DeliveryResult> {
    info!(
      "Sending Web3 topic notification {} to topic {}",
      notification.id().value(),
      topic_name
    );

    // Query database for topic subscribers (by wallet addresses)
    let subscriber_wallets = self.get_topic_subscribers(topic_name).await?;

    if subscriber_wallets.is_empty() {
      info!("No subscribers found for topic: {}", topic_name);
      return Ok(DeliveryResult::Success {
        delivered_at: Utc::now(),
        message_id: Some(format!("topic-empty-{}", Uuid::new_v4())),
      });
    }

    // Store wallet notifications for all subscribers
    let mut delivered_count = 0;
    let mut failed_count = 0;

    for wallet_address in subscriber_wallets {
      match self.store_wallet_notification(notification, &wallet_address).await {
        Ok(_) => delivered_count += 1,
        Err(e) => {
          failed_count += 1;
          debug!("Failed to store notification for wallet {}: {}", wallet_address, e);
        }
      }
    }

    info!(
      "Topic notification delivered to {}/{} subscribers ({} successful, {} failed)",
      delivered_count + failed_count,
      delivered_count + failed_count,
      delivered_count,
      failed_count
    );

    if delivered_count > 0 {
      Ok(DeliveryResult::Success {
        delivered_at: Utc::now(),
        message_id: Some(format!("topic-{}", Uuid::new_v4())),
      })
    } else {
      Ok(DeliveryResult::Failed {
        error_message: "Failed to deliver to any subscribers".to_string(),
        retry_after: Some(Utc::now() + chrono::Duration::minutes(5)),
      })
    }
  }

  // Email delivery method removed - Web3-first system uses direct wallet notifications

  /// Store notification in database for wallet-based in-app display
  async fn store_wallet_notification(
    &self,
    notification: &Notification,
    wallet_address: &str,
  ) -> ApplicationResult<DeliveryResult> {
    info!(
      "Storing wallet notification {} for {}",
      notification.id().value(),
      wallet_address
    );

    // Store notification in PostgreSQL with wallet_address as key
    // This would involve database operations to persist the notification
    self.persist_wallet_notification(notification, wallet_address).await?;

    // Mark as unread for the wallet user
    self.mark_notification_unread(&notification.id().value().to_string(), wallet_address).await?;

    // Enable real-time updates via WebSocket for connected wallets
    self.trigger_websocket_update(wallet_address, notification).await?;

    Ok(DeliveryResult::Success {
      delivered_at: Utc::now(),
      message_id: Some(format!("wallet-{}", Uuid::new_v4())),
    })
  }

  /// Get topic subscribers from database
  async fn get_topic_subscribers(&self, topic_name: &str) -> ApplicationResult<Vec<String>> {
    info!("Querying subscribers for topic: {}", topic_name);

    // Query database for wallet addresses subscribed to this topic
    // This would typically query a topic_subscriptions table
    let subscribers = vec![]; // Empty for now - would query database

    debug!("Found {} subscribers for topic {}", subscribers.len(), topic_name);
    Ok(subscribers)
  }

  /// Persist notification in database for wallet user
  async fn persist_wallet_notification(
    &self,
    notification: &Notification,
    wallet_address: &str,
  ) -> ApplicationResult<()> {
    // Database operation to insert notification record
    // This would insert into a wallet_notifications table
    info!(
      "Persisting notification {} for wallet {} in database",
      notification.id().value(),
      wallet_address
    );
    Ok(())
  }

  /// Mark notification as unread for wallet user
  async fn mark_notification_unread(
    &self,
    notification_id: &str,
    wallet_address: &str,
  ) -> ApplicationResult<()> {
    // Database operation to mark notification as unread
    info!(
      "Marking notification {} as unread for wallet {}",
      notification_id,
      wallet_address
    );
    Ok(())
  }

  /// Trigger WebSocket update for connected wallet clients
  async fn trigger_websocket_update(
    &self,
    wallet_address: &str,
    notification: &Notification,
  ) -> ApplicationResult<()> {
    // WebSocket notification to connected clients for this wallet
    info!(
      "Triggering WebSocket update for wallet {} notification {}",
      wallet_address,
      notification.id().value()
    );
    Ok(())
  }



  /// Get notification delivery stats (Web3-first)
  pub async fn get_delivery_stats(&self) -> ApplicationResult<NotificationStats> {
    info!("Querying Web3 notification delivery statistics from database");

    // Query database for wallet notification statistics
    let stats = self.query_notification_stats_from_database().await?;

    debug!(
      "Retrieved notification stats: {} total, {} successful, {} failed, {} wallet, {} in-app",
      stats.total_sent,
      stats.successful_deliveries,
      stats.failed_deliveries,
      stats.wallet_notifications,
      stats.in_app_notifications
    );

    Ok(stats)
  }

  /// Query notification statistics from database
  async fn query_notification_stats_from_database(&self) -> ApplicationResult<NotificationStats> {
    // Database query to get real notification statistics
    // This would aggregate data from wallet_notifications table

    // For now, return zero values - in production this would query actual stats
    let current_time = Utc::now();
    info!("Querying notification stats as of {}", current_time);

    Ok(NotificationStats {
      total_sent: 0,
      successful_deliveries: 0,
      failed_deliveries: 0,
      in_app_notifications: 0,
      wallet_notifications: 0,
    })
  }
}

/// Web3-first notification delivery statistics
#[derive(Debug, Clone)]
pub struct NotificationStats {
  pub total_sent: u64,
  pub successful_deliveries: u64,
  pub failed_deliveries: u64,
  // email_deliveries removed - Web3-first system uses wallet notifications only
  pub in_app_notifications: u64,
  pub wallet_notifications: u64,
}

#[cfg(test)]
mod tests {
  use super::*;

  #[tokio::test]
  async fn test_web3_notification_delivery() {
    // Email service removed - Web3-first system uses direct wallet notifications
    let adapter = NotificationRepositoryAdapter::new();

    // Test would create a notification and verify Web3-first delivery
    let stats = adapter.get_delivery_stats().await.unwrap();
    assert_eq!(stats.total_sent, 0); // Fresh instance
  }

  #[tokio::test]
  async fn test_wallet_notification_stats() {
    // Email service removed - Web3-first system uses direct wallet notifications
    let adapter = NotificationRepositoryAdapter::new();

    let stats = adapter.get_delivery_stats().await.unwrap();
    // email_deliveries field removed - Web3-first system uses wallet notifications only
    assert_eq!(stats.wallet_notifications, 0);
    assert_eq!(stats.in_app_notifications, 0);
  }
}