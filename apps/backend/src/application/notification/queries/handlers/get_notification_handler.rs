use crate::prelude::*;
use crate::application::shared::{QueryHandler, ApplicationResult, ApplicationError};
use crate::application::notification::queries::{GetNotificationQuery, GetNotificationResponse};
use crate::domain::notification::NotificationRepositoryPort;

/// Query handler for getting a single notification
pub struct GetNotificationQueryHandler {
    notification_repository: Arc<dyn NotificationRepositoryPort>,
}

impl GetNotificationQueryHandler {
    pub fn new(notification_repository: Arc<dyn NotificationRepositoryPort>) -> Self {
        Self {
            notification_repository,
        }
    }
}

#[async_trait]
impl QueryHandler<GetNotificationQuery> for GetNotificationQueryHandler {
    async fn handle(&self, query: GetNotificationQuery) -> ApplicationResult<GetNotificationResponse> {
        // Load notification from repository
        let notification = self.notification_repository.find_by_id(&query.notification_id).await
            .map_err(|e| ApplicationError::infrastructure(e.to_string()))?
            .ok_or_else(|| ApplicationError::not_found("notification_id", "Notification not found"))?;

        // Map domain aggregate to response DTO
        let (recipient_type, recipient_id) = if let Some(wallet_address) = notification.recipient_wallet_address() {
            ("user".to_string(), wallet_address.to_string())
        } else if let Some(topic) = notification.topic() {
            ("topic".to_string(), topic.name().to_string())
        } else {
            ("unknown".to_string(), "".to_string())
        };

        let channels: Vec<String> = notification.channels()
            .enabled_channels()
            .iter()
            .map(|ch| ch.channel_type().as_str().to_string())
            .collect();

        // Get delivery tracking information from actual channel statuses
        let delivery_tracking = notification.delivery_tracking();

        // Get last attempt time across all channels
        let last_delivery_attempt = notification.channels()
            .enabled_channels()
            .iter()
            .filter_map(|ch| {
                match delivery_tracking.get_channel_status(ch.channel_type().as_str()) {
                    Some(crate::domain::notification::aggregates::notification::ChannelDeliveryStatus::Delivered { delivered_at, .. }) => {
                        Some(*delivered_at)
                    }
                    Some(crate::domain::notification::aggregates::notification::ChannelDeliveryStatus::Failed(err)) => {
                        Some(err.attempted_at)
                    }
                    None => None,
                }
            })
            .max();

        // Get delivered_at from first successful delivery
        let delivered_at = if delivery_tracking.has_successful_delivery() {
            notification.channels()
                .enabled_channels()
                .iter()
                .filter_map(|ch| {
                    match delivery_tracking.get_channel_status(ch.channel_type().as_str()) {
                        Some(crate::domain::notification::aggregates::notification::ChannelDeliveryStatus::Delivered { delivered_at, .. }) => {
                            Some(*delivered_at)
                        }
                        _ => None,
                    }
                })
                .min()
        } else {
            None
        };

        Ok(GetNotificationResponse {
            notification_id: notification.id().as_str().to_string(),
            recipient_type,
            recipient_id,
            title: notification.content().title().to_string(),
            message: notification.content().body().to_string(),
            notification_type: notification.notification_type().as_str().to_string(),
            priority: notification.priority().as_str().to_string(),
            status: notification.status().as_str().to_string(),
            channels,
            schedule_type: notification.schedule().schedule_type().as_str().to_string(),
            scheduled_at: notification.schedule().scheduled_at(),
            expires_at: notification.schedule().expires_at(),
            delivery_attempts: delivery_tracking.total_attempts(),
            last_delivery_attempt,
            delivered_at,
            image_url: notification.metadata().image_url().map(|s| s.to_string()),
            action_url: notification.metadata().action_url().map(|s| s.to_string()),
            tags: notification.metadata().tags().iter().map(|s| s.to_string()).collect(),
            created_at: notification.created_at(),
            updated_at: notification.updated_at(),
        })
    }
}
