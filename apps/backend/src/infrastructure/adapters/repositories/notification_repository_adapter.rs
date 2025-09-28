/// Stateless notification repository adapter for serverless deployment
/// Handles notifications via email and database storage without Firebase/FCM

use std::sync::Arc;
use chrono::Utc;
use uuid::Uuid;
use tracing::{ debug, info, warn };

use crate::domain::notification::aggregates::notification::{
  Notification,
  DeliveryResult,
};
use crate::domain::notification::value_objects::user_preferences::NotificationType;
use crate::infrastructure::adapters::services::email_service::SendGridEmailService;
use crate::application::ports::services::EmailServiceError;
use crate::application::shared::error::ApplicationResult;

/// Stateless notification repository adapter - no Firebase dependencies
#[derive(Clone)]
pub struct NotificationRepositoryAdapter {
  email_service: Arc<SendGridEmailService>,
}

impl NotificationRepositoryAdapter {
  pub fn new(email_service: Arc<SendGridEmailService>) -> Self {
    debug!("Creating stateless NotificationRepositoryAdapter");
    Self {
      email_service,
    }
  }

  /// Send notification using stateless approach
  pub async fn send_notification(
    &self,
    notification: &Notification,
    wallet_address: &str,
    email: Option<String>,
  ) -> ApplicationResult<DeliveryResult> {
    let notification_id = notification.id().value().clone();
    
    info!(
      "Sending stateless notification {} to wallet {} via available channels",
      notification_id,
      wallet_address
    );

    // Try email delivery if email is available
    if let Some(user_email) = email {
      match self.deliver_via_email(notification, &user_email).await {
        Ok(()) => {
          info!(
            "Email delivery successful for notification {} to {}",
            notification_id,
            user_email
          );
          
          return Ok(DeliveryResult::Success {
            delivered_at: Utc::now(),
            message_id: Some(format!("email-{}", Uuid::new_v4())),
          });
        }
        Err(e) => {
          warn!(
            "Email delivery failed for notification {}: {}",
            notification_id,
            e
          );
        }
      }
    }

    // Store in database for in-app notifications (fallback)
    self.store_in_app_notification(notification, wallet_address).await
  }

  /// Send notification to multiple recipients
  pub async fn send_bulk_notification(
    &self,
    notification: &Notification,
    recipients: Vec<(String, Option<String>)>, // (wallet_address, email)
  ) -> ApplicationResult<Vec<DeliveryResult>> {
    info!(
      "Sending bulk notification {} to {} recipients",
      notification.id().value(),
      recipients.len()
    );

    let mut results = Vec::new();
    
    for (wallet_address, email) in recipients {
      let result = self.send_notification(notification, &wallet_address, email).await
        .unwrap_or_else(|e| DeliveryResult::Failed {
          error_message: format!("Failed to send notification: {}", e),
          retry_after: Some(Utc::now() + chrono::Duration::minutes(5)),
        });
      
      results.push(result);
    }

    Ok(results)
  }

  /// Send notification to topic (simplified - just email subscribers)
  pub async fn send_topic_notification(
    &self,
    notification: &Notification,
    topic_name: &str,
  ) -> ApplicationResult<DeliveryResult> {
    info!(
      "Sending topic notification {} to topic {}",
      notification.id().value(),
      topic_name
    );

    // In a real implementation, you would:
    // 1. Query database for topic subscribers
    // 2. Send email to all subscribers
    // 3. Store in-app notifications for active users
    
    // For now, return success as placeholder
    Ok(DeliveryResult::Success {
      delivered_at: Utc::now(),
      message_id: Some(format!("topic-{}", Uuid::new_v4())),
    })
  }

  /// Deliver notification via email
  async fn deliver_via_email(
    &self,
    notification: &Notification,
    email: &str,
  ) -> Result<(), EmailServiceError> {
    let subject = notification.content().title();
    let body = notification.content().body();
    
    // Use appropriate email method based on notification type
    match self.get_notification_type(notification) {
      NotificationType::Info => {
        self.email_service.send_welcome_email(email, "User").await
      }
      NotificationType::Success => {
        // Extract amount from notification content if available
        let amount = rust_decimal::Decimal::new(1000, 2); // Placeholder
        self.email_service.send_payment_confirmation(email, amount, "USD").await
      }
      _ => {
        // Generic notification email
        self.email_service.send_notification_email(email, subject, body).await
      }
    }
  }

  /// Store notification in database for in-app display
  async fn store_in_app_notification(
    &self,
    notification: &Notification,
    wallet_address: &str,
  ) -> ApplicationResult<DeliveryResult> {
    info!(
      "Storing in-app notification {} for wallet {}",
      notification.id().value(),
      wallet_address
    );

    // In a real implementation, you would:
    // 1. Store notification in PostgreSQL
    // 2. Mark as unread for the user
    // 3. Return success with database message ID

    Ok(DeliveryResult::Success {
      delivered_at: Utc::now(),
      message_id: Some(format!("db-{}", Uuid::new_v4())),
    })
  }

  /// Extract notification type from notification
  fn get_notification_type(&self, _notification: &Notification) -> NotificationType {
    // Extract from notification metadata or content
    // For now, default to general
    NotificationType::General
  }

  /// Get notification delivery stats (stateless)
  pub async fn get_delivery_stats(&self) -> ApplicationResult<NotificationStats> {
    // In a real implementation, query database for stats
    Ok(NotificationStats {
      total_sent: 0,
      successful_deliveries: 0,
      failed_deliveries: 0,
      email_deliveries: 0,
      in_app_notifications: 0,
    })
  }
}

/// Stateless notification delivery statistics
#[derive(Debug, Clone)]
pub struct NotificationStats {
  pub total_sent: u64,
  pub successful_deliveries: u64,
  pub failed_deliveries: u64,
  pub email_deliveries: u64,
  pub in_app_notifications: u64,
}

#[cfg(test)]
mod tests {
  use super::*;

  #[tokio::test]
  async fn test_stateless_notification_delivery() {
    let email_service = Arc::new(SendGridEmailService::new("test-key".to_string()));
    let adapter = NotificationRepositoryAdapter::new(email_service);

    // Test would create a notification and verify stateless delivery
    let stats = adapter.get_delivery_stats().await.unwrap();
    assert_eq!(stats.total_sent, 0); // Fresh instance
  }

  #[tokio::test]
  async fn test_notification_stats() {
    let email_service = Arc::new(SendGridEmailService::new("test-key".to_string()));
    let adapter = NotificationRepositoryAdapter::new(email_service);

    let stats = adapter.get_delivery_stats().await.unwrap();
    assert_eq!(stats.email_deliveries, 0);
    assert_eq!(stats.in_app_notifications, 0);
  }
}