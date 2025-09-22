use async_trait::async_trait;
use crate::application::ports::outbound::service_ports::NotificationServicePort;

#[derive(Debug, thiserror::Error)]
pub enum NotificationError {
  #[error("Failed to send notification: {0}")] SendError(String),
  #[error("Invalid device token: {0}")] InvalidToken(String),
  #[error("Service unavailable: {0}")] ServiceError(String),
}

/// Notification service adapter for handling push notifications
#[derive(Clone)]
pub struct NotificationServiceAdapter {
  // In a real implementation, this would contain FCM service, etc.
}

impl NotificationServiceAdapter {
  pub fn new() -> Self {
    Self {}
  }
}

#[async_trait]
impl NotificationServicePort for NotificationServiceAdapter {
  type Error = NotificationError;

  async fn send_push_notification(
    &self,
    device_token: &str,
    message: &str
  ) -> Result<(), Self::Error> {
    tracing::info!(
      "Sending push notification to token: {} with message: {}",
      device_token,
      message
    );

    // In full implementation, would send actual push notification
    // For now, just log the attempt
    Ok(())
  }
}

impl Default for NotificationServiceAdapter {
  fn default() -> Self {
    Self::new()
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[tokio::test]
  async fn test_send_push_notification() {
    let service = NotificationServiceAdapter::new();
    let result = service.send_push_notification(
      "test_token",
      "test message"
    ).await;
    assert!(result.is_ok());
  }
}
