use crate::prelude::*;
use crate::application::shared::{CommandHandler, ApplicationResult, ApplicationError};
use crate::application::notification::commands::{
    CancelNotificationCommand, CancelNotificationResponse
};
use crate::domain::notification::NotificationRepositoryPort;
use epsx_contracts::event_publisher_port::EventPublisherPort;

/// Command handler for cancelling notifications
pub struct CancelNotificationCommandHandler {
    notification_repository: Arc<dyn NotificationRepositoryPort>,
    event_publisher: Arc<dyn EventPublisherPort>,
}

impl CancelNotificationCommandHandler {
    pub fn new(
        notification_repository: Arc<dyn NotificationRepositoryPort>,
        event_publisher: Arc<dyn EventPublisherPort>,
    ) -> Self {
        Self {
            notification_repository,
            event_publisher,
        }
    }
}

#[async_trait]
impl CommandHandler<CancelNotificationCommand> for CancelNotificationCommandHandler {
    async fn handle(&self, command: CancelNotificationCommand) -> ApplicationResult<CancelNotificationResponse> {
        // 1. Find notification
        let mut notification = self.notification_repository.find_by_id(&command.notification_id).await
            .map_err(|e| ApplicationError::infrastructure(e.to_string()))?
            .ok_or_else(|| ApplicationError::not_found("notification_id", "Notification not found"))?;

        // 2. Cancel notification (domain logic validates state transition)
        notification.cancel(command.reason.clone())
            .map_err(ApplicationError::business_logic)?;

        // 3. Save cancelled notification
        self.notification_repository.save(&notification).await
            .map_err(|e| ApplicationError::infrastructure(e.to_string()))?;

        // 4. Publish domain events
        for event in notification.uncommitted_events() {
            let owned: Box<dyn crate::domain::shared_kernel::DomainEvent> = Box::new(epsx_contracts::domain_event::OwnedEvent::from_borrowed(&**event));
            if let Err(e) = self.event_publisher.publish(owned).await {
                tracing::warn!(
                    error = %e,
                    "EventPublisherPort.publish returned error; command continues"
                );
            }
        }

        // 5. Return response
        Ok(CancelNotificationResponse {
            notification_id: command.notification_id,
            status: notification.status().as_str().to_string(),
            reason: command.reason,
            cancelled_at: Utc::now(),
        })
    }
}
