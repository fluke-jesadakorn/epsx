use async_trait::async_trait;
use chrono::{DateTime, Utc};
use uuid::Uuid;

use super::models::*;
use crate::infra::db::diesel::types::{DeliveryStatus, DeliveryChannel};
use crate::core::errors::AppError;

#[async_trait]
pub trait NotificationRepository: Send + Sync {
    async fn create_notification(&self, req: CreateNotificationRequest, created_by: Option<Uuid>) -> Result<Notification, AppError>;
    async fn get_notification(&self, id: Uuid) -> Result<Option<Notification>, AppError>;
    async fn get_user_notifications(&self, user_id: Uuid, limit: Option<i64>, offset: Option<i64>) -> Result<Vec<Notification>, AppError>;
    async fn update_notification_delivery_status(&self, delivery_id: Uuid, status: DeliveryStatus, error_message: Option<String>) -> Result<(), AppError>;
    async fn mark_notification_read(&self, delivery_id: Uuid, read_at: DateTime<Utc>) -> Result<(), AppError>;
    async fn mark_notification_clicked(&self, delivery_id: Uuid, clicked_at: DateTime<Utc>) -> Result<(), AppError>;
    async fn get_scheduled_notifications(&self, before: DateTime<Utc>) -> Result<Vec<Notification>, AppError>;
    async fn get_notification_stats(&self, user_id: Option<Uuid>, start_date: Option<DateTime<Utc>>, end_date: Option<DateTime<Utc>>) -> Result<NotificationStats, AppError>;
}

#[async_trait]
pub trait FcmTokenRepository: Send + Sync {
    async fn create_token(&self, user_id: Uuid, req: CreateFcmTokenRequest) -> Result<FcmToken, AppError>;
    async fn get_user_tokens(&self, user_id: Uuid) -> Result<Vec<FcmToken>, AppError>;
    async fn get_active_user_tokens(&self, user_id: Uuid) -> Result<Vec<FcmToken>, AppError>;
    async fn update_token_last_used(&self, token_id: Uuid) -> Result<(), AppError>;
    async fn deactivate_token(&self, token_id: Uuid) -> Result<(), AppError>;
    async fn subscribe_to_topics(&self, token_id: Uuid, topic_names: Vec<String>) -> Result<(), AppError>;
    async fn unsubscribe_from_topics(&self, token_id: Uuid, topic_names: Vec<String>) -> Result<(), AppError>;
    async fn get_tokens_by_topic(&self, topic_name: String) -> Result<Vec<FcmToken>, AppError>;
    async fn cleanup_expired_tokens(&self, older_than: DateTime<Utc>) -> Result<i64, AppError>;
}

#[async_trait]
pub trait FcmTopicRepository: Send + Sync {
    async fn get_all_topics(&self) -> Result<Vec<FcmTopic>, AppError>;
    async fn get_topic_by_name(&self, name: String) -> Result<Option<FcmTopic>, AppError>;
    async fn create_topic(&self, name: String, display_name: String, description: Option<String>, target_permissions: Option<serde_json::Value>, created_by: Option<Uuid>) -> Result<FcmTopic, AppError>;
    async fn update_topic(&self, id: Uuid, display_name: Option<String>, description: Option<String>, target_permissions: Option<serde_json::Value>) -> Result<(), AppError>;
    async fn deactivate_topic(&self, id: Uuid) -> Result<(), AppError>;
}

#[async_trait]
pub trait UserNotificationPreferencesRepository: Send + Sync {
    async fn get_user_preferences(&self, user_id: Uuid) -> Result<Option<UserNotificationPreferences>, AppError>;
    async fn create_or_update_preferences(&self, preferences: UserNotificationPreferences) -> Result<UserNotificationPreferences, AppError>;
    async fn block_topics(&self, user_id: Uuid, topic_names: Vec<String>) -> Result<(), AppError>;
    async fn unblock_topics(&self, user_id: Uuid, topic_names: Vec<String>) -> Result<(), AppError>;
}