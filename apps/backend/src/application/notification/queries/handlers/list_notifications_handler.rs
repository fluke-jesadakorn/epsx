use crate::prelude::*;
use crate::application::shared::{QueryHandler, ApplicationResult, ApplicationError};
use crate::application::notification::queries::{
    ListNotificationsQuery, ListNotificationsResponse, NotificationSummaryDTO
};
use crate::domain::notification::{NotificationRepositoryPort, NotificationSearchCriteria, NotificationStatus, NotificationType};
use uuid::Uuid;

/// Query handler for listing notifications with filters
pub struct ListNotificationsQueryHandler {
    notification_repository: Arc<dyn NotificationRepositoryPort>,
}

impl ListNotificationsQueryHandler {
    pub fn new(notification_repository: Arc<dyn NotificationRepositoryPort>) -> Self {
        Self {
            notification_repository,
        }
    }
}

#[async_trait]
impl QueryHandler<ListNotificationsQuery> for ListNotificationsQueryHandler {
    async fn handle(&self, query: ListNotificationsQuery) -> ApplicationResult<ListNotificationsResponse> {
        // Parse wallet address if provided
        let recipient_wallet_address = if let Some(addr) = &query.wallet_address {
            Some(Uuid::parse_str(addr)
                .map_err(|e| ApplicationError::validation("wallet_address", e.to_string()))?)
        } else {
            None
        };

        // Parse status if provided
        let status = if let Some(s) = &query.status {
            Some(NotificationStatus::from_str(s)
                .map_err(|e| ApplicationError::validation("status", e))?)
        } else {
            None
        };

        // Parse notification type if provided
        let notification_type = if let Some(nt) = &query.notification_type {
            Some(NotificationType::from_str(nt)
                .map_err(|e| ApplicationError::validation("notification_type", e))?)
        } else {
            None
        };

        // Build search criteria from query
        let criteria = NotificationSearchCriteria {
            recipient_wallet_address,
            topic: query.topic.clone(),
            status,
            notification_type,
            priority: query.priority.clone(),
            created_after: None,
            created_before: None,
            limit: Some(query.limit as i64),
            offset: Some(query.offset as i64),
        };

        // Load notifications from repository
        let notifications = self.notification_repository.find_all(criteria.clone()).await
            .map_err(|e| ApplicationError::infrastructure(e.to_string()))?;

        // Get total count
        let total = self.notification_repository.count(criteria).await
            .map_err(|e| ApplicationError::infrastructure(e.to_string()))?;

        // Map domain aggregates to summary DTOs
        let notification_summaries: Vec<NotificationSummaryDTO> = notifications
            .into_iter()
            .map(|notification| {
                let (recipient_type, recipient_id) = if let Some(wallet_address) = notification.recipientwallet_address() {
                    ("user".to_string(), wallet_address.to_string())
                } else if let Some(topic) = notification.topic() {
                    ("topic".to_string(), topic.name().to_string())
                } else {
                    ("unknown".to_string(), "".to_string())
                };

                NotificationSummaryDTO {
                    notification_id: notification.id().as_str().to_string(),
                    recipient_type,
                    recipient_id,
                    title: notification.content().title().to_string(),
                    notification_type: notification.notification_type().as_str().to_string(),
                    priority: notification.priority().as_str().to_string(),
                    status: notification.status().as_str().to_string(),
                    scheduled_at: notification.schedule().scheduled_at(),
                    delivery_attempts: notification.delivery_tracking().total_attempts(),
                    created_at: notification.created_at(),
                }
            })
            .collect();

        Ok(ListNotificationsResponse {
            notifications: notification_summaries,
            total: total as u32,
            offset: query.offset,
            limit: query.limit,
        })
    }
}
