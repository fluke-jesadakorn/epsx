use crate::prelude::*;
use crate::application::shared::{QueryHandler, ApplicationResult, ApplicationError};
use crate::application::notification::queries::{
    GetDeliveryStatusQuery, GetDeliveryStatusResponse, ChannelDeliveryStatus
};
use crate::domain::notification::NotificationRepositoryPort;

/// Query handler for getting notification delivery status
pub struct GetDeliveryStatusQueryHandler {
    notification_repository: Arc<dyn NotificationRepositoryPort>,
}

impl GetDeliveryStatusQueryHandler {
    pub fn new(notification_repository: Arc<dyn NotificationRepositoryPort>) -> Self {
        Self {
            notification_repository,
        }
    }
}

#[async_trait]
impl QueryHandler<GetDeliveryStatusQuery> for GetDeliveryStatusQueryHandler {
    async fn handle(&self, query: GetDeliveryStatusQuery) -> ApplicationResult<GetDeliveryStatusResponse> {
        // Load notification from repository
        let notification = self.notification_repository.find_by_id(&query.notification_id).await
            .map_err(|e| ApplicationError::infrastructure(e.to_string()))?
            .ok_or_else(|| ApplicationError::not_found("notification_id", "Notification not found"))?;

        // Map channel delivery statuses from actual delivery tracking
        let delivery_tracking = notification.delivery_tracking();
        let channel_statuses: Vec<ChannelDeliveryStatus> = notification.channels()
            .enabled_channels()
            .iter()
            .map(|channel| {
                let channel_name = channel.channel_type().as_str();
                let status = delivery_tracking.get_channel_status(channel_name);

                match status {
                    Some(crate::domain::notification::aggregates::notification::ChannelDeliveryStatus::Delivered { delivered_at, .. }) => {
                        ChannelDeliveryStatus {
                            channel: channel_name.to_string(),
                            delivered: true,
                            attempts: 1, // Delivered means at least one attempt succeeded
                            last_attempt_at: Some(*delivered_at),
                            last_error: None,
                        }
                    }
                    Some(crate::domain::notification::aggregates::notification::ChannelDeliveryStatus::Failed(err)) => {
                        ChannelDeliveryStatus {
                            channel: channel_name.to_string(),
                            delivered: false,
                            attempts: 1, // Failed means at least one attempt was made
                            last_attempt_at: Some(err.attempted_at),
                            last_error: Some(err.error_message.clone()),
                        }
                    }
                    None => {
                        ChannelDeliveryStatus {
                            channel: channel_name.to_string(),
                            delivered: false,
                            attempts: 0,
                            last_attempt_at: None,
                            last_error: None,
                        }
                    }
                }
            })
            .collect();

        // Check if notification is expired
        let is_expired = if let Some(expires_at) = notification.schedule().expires_at() {
            Utc::now() > expires_at
        } else {
            false
        };

        // Get delivered_at from notification metadata or first successful delivery
        let delivered_at = if delivery_tracking.has_successful_delivery() {
            // Use the earliest successful delivery timestamp from channels
            channel_statuses.iter()
                .filter(|ch| ch.delivered)
                .filter_map(|ch| ch.last_attempt_at)
                .min()
        } else {
            None
        };

        // Get last attempt time across all channels
        let last_attempt_at = channel_statuses.iter()
            .filter_map(|ch| ch.last_attempt_at)
            .max();

        Ok(GetDeliveryStatusResponse {
            notification_id: notification.id().as_str().to_string(),
            status: notification.status().as_str().to_string(),
            delivery_attempts: delivery_tracking.total_attempts(),
            last_attempt_at,
            delivered_at,
            channels: channel_statuses,
            is_expired,
            expires_at: notification.schedule().expires_at(),
        })
    }
}
