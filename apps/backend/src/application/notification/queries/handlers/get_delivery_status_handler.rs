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

        // Map channel delivery statuses
        // TODO: Get actual delivery status from DeliveryTracking
        let channel_statuses: Vec<ChannelDeliveryStatus> = notification.channels()
            .enabled_channels()
            .iter()
            .map(|channel| {
                ChannelDeliveryStatus {
                    channel: channel.channel_type().as_str().to_string(),
                    delivered: false, // TODO: Get from DeliveryTracking
                    attempts: 0, // TODO: Get from DeliveryTracking
                    last_attempt_at: None, // TODO: Get from DeliveryTracking
                    last_error: None, // TODO: Get from DeliveryTracking
                }
            })
            .collect();

        // Check if notification is expired
        let is_expired = if let Some(expires_at) = notification.schedule().expires_at() {
            Utc::now() > expires_at
        } else {
            false
        };

        Ok(GetDeliveryStatusResponse {
            notification_id: notification.id().as_str().to_string(),
            status: notification.status().as_str().to_string(),
            delivery_attempts: notification.delivery_tracking().total_attempts(),
            last_attempt_at: None, // TODO: Add to domain model if needed
            delivered_at: None, // TODO: Add to domain model if needed
            channels: channel_statuses,
            is_expired,
            expires_at: notification.schedule().expires_at(),
        })
    }
}
