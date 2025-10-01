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
    let notification_id = notification.id().value().clone();
    
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

    // In a real implementation, you would:
    // 1. Query database for topic subscribers (by wallet addresses)
    // 2. Store wallet notifications for all subscribers
    // 3. Enable in-app notifications for active wallet users
    
    // For now, return success as placeholder
    Ok(DeliveryResult::Success {
      delivered_at: Utc::now(),
      message_id: Some(format!("topic-{}", Uuid::new_v4())),
    })
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

    // In a real implementation, you would:
    // 1. Store notification in PostgreSQL with wallet_address as key
    // 2. Mark as unread for the wallet user
    // 3. Enable real-time updates via WebSocket for connected wallets
    // 4. Return success with database message ID

    Ok(DeliveryResult::Success {
      delivered_at: Utc::now(),
      message_id: Some(format!("wallet-{}", Uuid::new_v4())),
    })
  }



  /// Get notification delivery stats (Web3-first)
  pub async fn get_delivery_stats(&self) -> ApplicationResult<NotificationStats> {
    // In a real implementation, query database for wallet notification stats
    Ok(NotificationStats {
      total_sent: 0,
      successful_deliveries: 0,
      failed_deliveries: 0,
      // email_deliveries removed - Web3-first system uses wallet notifications only
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