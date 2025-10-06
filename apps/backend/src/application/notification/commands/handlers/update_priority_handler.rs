use crate::prelude::*;
use crate::application::shared::{CommandHandler, ApplicationResult, ApplicationError};
use crate::application::notification::commands::{
    UpdateNotificationPriorityCommand, UpdateNotificationPriorityResponse
};
use crate::domain::notification::{NotificationRepositoryPort, NotificationPriority};
use crate::domain::shared_kernel::DomainEventBus;

/// Command handler for updating notification priority
pub struct UpdateNotificationPriorityCommandHandler {
    notification_repository: Arc<dyn NotificationRepositoryPort>,
    event_bus: Arc<dyn DomainEventBus>,
}

impl UpdateNotificationPriorityCommandHandler {
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
impl CommandHandler<UpdateNotificationPriorityCommand> for UpdateNotificationPriorityCommandHandler {
    async fn handle(&self, command: UpdateNotificationPriorityCommand) -> ApplicationResult<UpdateNotificationPriorityResponse> {
        // 1. Find notification
        let mut notification = self.notification_repository.find_by_id(&command.notification_id).await
            .map_err(|e| ApplicationError::infrastructure(e.to_string()))?
            .ok_or_else(|| ApplicationError::not_found("notification_id", "Notification not found"))?;

        // 2. Store old priority
        let old_priority = notification.priority().as_str().to_string();

        // 3. Parse new priority
        let new_priority = NotificationPriority::from_str(&command.new_priority)
            .map_err(|e| ApplicationError::validation("new_priority", e))?;

        // 4. Update priority (domain logic validates state transition)
        notification.update_priority(new_priority)
            .map_err(|e| ApplicationError::business_logic(e))?;

        // 5. Save updated notification
        self.notification_repository.save(&notification).await
            .map_err(|e| ApplicationError::infrastructure(e.to_string()))?;

        // 6. Publish domain events
        for event in notification.uncommitted_events() {
            self.event_bus.publish(&**event);
        }

        // 7. Return response
        Ok(UpdateNotificationPriorityResponse {
            notification_id: command.notification_id,
            old_priority,
            new_priority: notification.priority().as_str().to_string(),
            updated_at: Utc::now(),
        })
    }
}
