use crate::prelude::*;
use crate::application::shared::{CommandHandler, ApplicationResult, ApplicationError};
use crate::application::notification::commands::{
    CancelNotificationCommand, CancelNotificationResponse
};
use crate::domain::notification::NotificationRepositoryPort;
use crate::domain::shared_kernel::DomainEventBus;

/// Command handler for cancelling notifications
pub struct CancelNotificationCommandHandler {
    notification_repository: Arc<dyn NotificationRepositoryPort>,
    event_bus: Arc<dyn DomainEventBus>,
}

impl CancelNotificationCommandHandler {
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
impl CommandHandler<CancelNotificationCommand> for CancelNotificationCommandHandler {
    async fn handle(&self, command: CancelNotificationCommand) -> ApplicationResult<CancelNotificationResponse> {
        // 1. Find notification
        let mut notification = self.notification_repository.find_by_id(&command.notification_id).await
            .map_err(|e| ApplicationError::infrastructure(e.to_string()))?
            .ok_or_else(|| ApplicationError::not_found("notification_id", "Notification not found"))?;

        // 2. Cancel notification (domain logic validates state transition)
        notification.cancel(command.reason.clone())
            .map_err(|e| ApplicationError::business_logic(e))?;

        // 3. Save cancelled notification
        self.notification_repository.save(&notification).await
            .map_err(|e| ApplicationError::infrastructure(e.to_string()))?;

        // 4. Publish domain events
        for event in notification.uncommitted_events() {
            self.event_bus.publish(&**event);
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
