use crate::prelude::*;
use crate::application::shared::{CommandHandler, ApplicationResult, ApplicationError};
use crate::application::notification::commands::{
    RecordDeliveryAttemptCommand, RecordDeliveryAttemptResponse
};
use crate::domain::notification::{NotificationRepositoryPort, DeliveryChannelType, DeliveryResult};
use crate::domain::shared_kernel::DomainEventBus;

/// Command handler for recording notification delivery attempts
pub struct RecordDeliveryAttemptCommandHandler {
    notification_repository: Arc<dyn NotificationRepositoryPort>,
    event_bus: Arc<dyn DomainEventBus>,
}

impl RecordDeliveryAttemptCommandHandler {
    pub fn new(
        notification_repository: Arc<dyn NotificationRepositoryPort>,
        event_bus: Arc<dyn DomainEventBus>,
    ) -> Self {
        Self {
            notification_repository,
            event_bus,
        }
    }
}

#[async_trait]
impl CommandHandler<RecordDeliveryAttemptCommand> for RecordDeliveryAttemptCommandHandler {
    async fn handle(&self, command: RecordDeliveryAttemptCommand) -> ApplicationResult<RecordDeliveryAttemptResponse> {
        // 1. Find notification
        let mut notification = self.notification_repository.find_by_id(&command.notification_id).await
            .map_err(|e| ApplicationError::infrastructure(e.to_string()))?
            .ok_or_else(|| ApplicationError::not_found("notification_id", "Notification not found"))?;

        // 2. Parse delivery channel
        let channel = DeliveryChannelType::from_str(&command.channel)
            .map_err(|e| ApplicationError::validation("channel", e))?;

        // 3. Build delivery result based on success/failure
        let delivery_result = if command.success {
            DeliveryResult::Success {
                delivered_at: Utc::now(),
                message_id: command.response_details.clone(),
            }
        } else {
            DeliveryResult::Failed {
                error_message: command.error_message.clone().unwrap_or_else(|| "Delivery failed".to_string()),
                retry_after: None,
            }
        };

        // 4. Record delivery attempt (domain logic validates state and tracks attempts)
        notification.record_delivery_attempt(&channel, delivery_result)
            .map_err(ApplicationError::business_logic)?;

        // 5. Get attempt count for response
        let attempt_count = notification.delivery_tracking().total_attempts();

        // 6. Save updated notification
        self.notification_repository.save(&notification).await
            .map_err(|e| ApplicationError::infrastructure(e.to_string()))?;

        // 7. Publish domain events
        for event in notification.uncommitted_events() {
            self.event_bus.publish(&**event);
        }

        // 8. Return response
        Ok(RecordDeliveryAttemptResponse {
            notification_id: command.notification_id,
            channel: command.channel,
            success: command.success,
            attempt_count,
            recorded_at: Utc::now(),
        })
    }
}
