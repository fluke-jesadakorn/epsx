use crate::prelude::*;
use crate::application::shared::{QueryHandler, ApplicationResult, ApplicationError};
use crate::application::notification::queries::{
    ListPendingNotificationsQuery, ListPendingNotificationsResponse, PendingNotificationDTO
};
use crate::domain::notification::NotificationRepositoryPort;

/// Query handler for listing pending notifications ready for delivery
pub struct ListPendingNotificationsQueryHandler {
    notification_repository: Arc<dyn NotificationRepositoryPort>,
}

impl ListPendingNotificationsQueryHandler {
    pub fn new(notification_repository: Arc<dyn NotificationRepositoryPort>) -> Self {
        Self {
            notification_repository,
        }
    }
}

#[async_trait]
impl QueryHandler<ListPendingNotificationsQuery> for ListPendingNotificationsQueryHandler {
    async fn handle(&self, query: ListPendingNotificationsQuery) -> ApplicationResult<ListPendingNotificationsResponse> {
        // Load pending notifications from repository
        let notifications = self.notification_repository.find_pending(query.limit).await
            .map_err(|e| ApplicationError::infrastructure(e.to_string()))?;

        // Filter by before timestamp if provided
        let filtered_notifications: Vec<_> = if let Some(before) = query.before {
            notifications.into_iter()
                .filter(|n| {
                    if let Some(scheduled_at) = n.schedule().scheduled_at() {
                        scheduled_at <= before
                    } else {
                        true
                    }
                })
                .collect()
        } else {
            notifications
        };

        let total = filtered_notifications.len() as u32;

        // Map domain aggregates to pending DTOs
        let pending_dtos: Vec<PendingNotificationDTO> = filtered_notifications
            .into_iter()
            .map(|notification| {
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

                PendingNotificationDTO {
                    notification_id: notification.id().as_str().to_string(),
                    recipient_type,
                    recipient_id,
                    title: notification.content().title().to_string(),
                    message: notification.content().body().to_string(),
                    notification_type: notification.notification_type().as_str().to_string(),
                    priority: notification.priority().as_str().to_string(),
                    channels,
                    scheduled_at: notification.schedule().scheduled_at().unwrap_or_else(|| notification.created_at()),
                    delivery_attempts: notification.delivery_tracking().total_attempts(),
                    created_at: notification.created_at(),
                }
            })
            .collect();

        Ok(ListPendingNotificationsResponse {
            notifications: pending_dtos,
            total,
        })
    }
}
