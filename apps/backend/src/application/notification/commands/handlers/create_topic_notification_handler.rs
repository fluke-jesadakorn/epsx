use crate::prelude::*;
use crate::application::shared::{CommandHandler, ApplicationResult, ApplicationError};
use crate::application::notification::commands::{
    CreateTopicNotificationCommand, CreateTopicNotificationResponse
};
use crate::domain::notification::{
    NotificationRepositoryPort, Notification, NotificationContent, NotificationTopic,
    NotificationPriority, MultiChannelConfig, ScheduleInfo, DeliveryChannel, DeliveryChannelType
};
use crate::domain::notification::value_objects::user_preferences::NotificationType;
use crate::domain::notification::value_objects::schedule_info::ScheduleType;
use crate::domain::notification::value_objects::notification_topic::TopicCategory;
use crate::domain::shared_kernel::DomainEventBus;

/// Command handler for creating topic notifications
pub struct CreateTopicNotificationCommandHandler {
    notification_repository: Arc<dyn NotificationRepositoryPort>,
    event_bus: Arc<dyn DomainEventBus>,
}

impl CreateTopicNotificationCommandHandler {
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
impl CommandHandler<CreateTopicNotificationCommand> for CreateTopicNotificationCommandHandler {
    async fn handle(&self, command: CreateTopicNotificationCommand) -> ApplicationResult<CreateTopicNotificationResponse> {
        // 1. Create notification content
        let content = NotificationContent::new(command.title.clone(), command.message.clone())
            .map_err(|e| ApplicationError::validation("content", e))?;

        // 2. Create notification topic (using General category as default)
        let topic = NotificationTopic::new(
            command.topic.clone(),
            command.topic.clone(),
            None,
            TopicCategory::General,
        ).map_err(|e| ApplicationError::validation("topic", e))?;

        // 3. Parse notification type
        let notification_type = NotificationType::from_str(&command.notification_type)
            .map_err(|e| ApplicationError::validation("notification_type", e))?;

        // 4. Parse priority
        let priority = NotificationPriority::from_str(&command.priority)
            .map_err(|e| ApplicationError::validation("priority", e))?;

        // 5. Parse delivery channels
        let channels: Result<Vec<DeliveryChannel>, String> = command.channels.iter()
            .map(|ch| DeliveryChannelType::from_str(ch).map(DeliveryChannel::new))
            .collect();

        let channels = channels.map_err(|e| ApplicationError::validation("channels", e))?;
        let multi_channel_config = MultiChannelConfig::new(channels);

        // 6. Create schedule info
        let schedule = if let Some(schedule_type_str) = command.schedule_type {
            let schedule_type = ScheduleType::from_str(&schedule_type_str)
                .map_err(|e| ApplicationError::validation("schedule_type", e))?;

            match schedule_type {
                ScheduleType::Immediate => {
                    if let Some(expires_at) = command.expires_at {
                        ScheduleInfo::with_expiry(expires_at)
                            .map_err(|e| ApplicationError::validation("expires_at", e))?
                    } else {
                        ScheduleInfo::immediate()
                    }
                },
                ScheduleType::Scheduled => {
                    let scheduled_at = command.scheduled_at
                        .ok_or_else(|| ApplicationError::validation("scheduled_at", "Scheduled delivery requires scheduled_at timestamp"))?;

                    if let Some(expires_at) = command.expires_at {
                        ScheduleInfo::scheduled_with_expiry(scheduled_at, expires_at)
                            .map_err(|e| ApplicationError::validation("schedule", e))?
                    } else {
                        ScheduleInfo::scheduled(scheduled_at)
                            .map_err(|e| ApplicationError::validation("scheduled_at", e))?
                    }
                }
            }
        } else {
            ScheduleInfo::immediate()
        };

        // 7. Create notification aggregate for topic
        let mut notification = Notification::create_for_topic(
            topic,
            content,
            notification_type,
            priority,
            multi_channel_config,
            schedule,
            None, // created_by - no specific creator for topic broadcasts
        ).map_err(ApplicationError::business_logic)?;

        // 8. Add optional metadata
        if let Some(image_url) = command.image_url {
            notification.metadata_mut().set_image_url(image_url);
        }
        if let Some(action_url) = command.action_url {
            notification.metadata_mut().set_action_url(action_url);
        }
        if let Some(tags) = command.tags {
            for tag in tags {
                notification.metadata_mut().add_tag(tag);
            }
        }

        // 9. Save notification
        self.notification_repository.save(&notification).await
            .map_err(|e| ApplicationError::infrastructure(e.to_string()))?;

        // 10. Publish domain events
        for event in notification.uncommitted_events() {
            self.event_bus.publish(&**event);
        }

        // 11. Return response
        Ok(CreateTopicNotificationResponse {
            notification_id: notification.id().as_str().to_string(),
            topic: command.topic,
            status: notification.status().as_str().to_string(),
            scheduled_at: notification.schedule().scheduled_at(),
            created_at: notification.created_at(),
        })
    }
}
