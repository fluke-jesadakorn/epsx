// Diesel-based Notification Repository Implementation
use diesel::prelude::*;
use chrono::{DateTime, Utc};
use uuid::Uuid;
use tracing::{info, error, debug, warn};

use crate::{
    dom::{
        ports::notification::{DomainNotification, NotificationError, NotificationRecipient},
        values::identifiers::UserId,
    },
    infra::{
        cache::Cache,
        db::diesel::{
            models::{DieselNotification, NewDieselNotification, NotificationStats},
            pool::DbPool,
            schema::notifications::dsl::*,
        },
    },
};

pub struct PostgresNotificationRepo {
    pool: DbPool,
    cache: Option<Box<dyn Cache>>,
}

impl PostgresNotificationRepo {
    pub fn new(pool: DbPool) -> Self {
        Self {
            pool,
            cache: None,
        }
    }

    pub fn with_cache(pool: DbPool, cache_instance: Box<dyn Cache>) -> Self {
        Self {
            pool,
            cache: Some(cache_instance),
        }
    }

    // Create a new notification
    pub async fn create_notification(&self, notification: &DomainNotification) -> Result<Uuid, NotificationError> {
        let mut conn = self.pool.get()
            .await
            .map_err(|e| NotificationError::DatabaseError(format!("Connection failed: {}", e)))?;

        // Generate ID if not provided
        let notification_id = match &notification.id {
            Some(id_str) => Uuid::parse_str(id_str)
                .map_err(|_| NotificationError::DatabaseError("Invalid UUID format".to_string()))?,
            None => Uuid::new_v4(),
        };

        // Extract user info from recipient
        let (user_id, firebase_uid) = match &notification.recipient {
            NotificationRecipient::User(user_id) => (
                user_id.0, // Access the inner Uuid from UserId
                None // For now, we don't have firebase_uid in the domain model
            ),
            NotificationRecipient::Email(_) => {
                return Err(NotificationError::ValidationError("Email-only recipients not supported yet".to_string()));
            },
            _ => {
                return Err(NotificationError::ValidationError("Unsupported recipient type".to_string()));
            },
        };

        let new_notification = NewDieselNotification {
            id: notification_id,
            user_id,
            user_firebase_uid: firebase_uid,
            title: notification.title.clone(),
            message: notification.message.clone(),
            notification_type: notification.notification_type.to_string(),
            priority: notification.priority.to_string(),
            is_read: false,
            delivery_status: Some("pending".to_string()),
            created_at: Utc::now(),
            expires_at: notification.expires_at,
            metadata: notification.data.clone(),
        };

        let result = diesel::insert_into(notifications)
            .values(&new_notification)
            .execute(&mut conn)
            .await
            .map_err(|e| {
                error!("Failed to create notification: {}", e);
                NotificationError::DatabaseError(format!("Insert failed: {}", e))
            })?;

        if result > 0 {
            info!("Created notification {} for user {}", notification_id, user_id);
            Ok(notification_id)
        } else {
            Err(NotificationError::DatabaseError("No rows inserted".to_string()))
        }
    }

    // Get a notification by ID  
    pub async fn get_notification(&self, notification_id: &Uuid) -> Result<Option<DieselNotification>, NotificationError> {
        let mut conn = self.pool.get()
            .await
            .map_err(|e| NotificationError::DatabaseError(format!("Connection failed: {}", e)))?;

        let result = notifications
            .filter(id.eq(notification_id))
            .first::<DieselNotification>(&mut conn)
            .await
            .optional()
            .map_err(|e| {
                error!("Failed to get notification {}: {}", notification_id, e);
                NotificationError::DatabaseError(format!("Query failed: {}", e))
            })?;

        Ok(result)
    }

    // Get notifications for a user with pagination
    pub async fn get_user_notifications(
        &self, 
        user_id_filter: &Uuid, 
        offset_val: i32, 
        limit_val: i32
    ) -> Result<Vec<DieselNotification>, NotificationError> {
        let mut conn = self.pool.get()
            .await
            .map_err(|e| NotificationError::DatabaseError(format!("Connection failed: {}", e)))?;

        let results = notifications
            .filter(user_id.eq(user_id_filter))
            .order(created_at.desc())
            .offset(offset_val as i64)
            .limit(limit_val as i64)
            .load::<DieselNotification>(&mut conn)
            .await
            .map_err(|e| {
                error!("Failed to get notifications for user {}: {}", user_id_filter, e);
                NotificationError::DatabaseError(format!("Query failed: {}", e))
            })?;

        Ok(results)
    }

    // Mark a notification as read
    pub async fn mark_as_read(&self, notification_id: &Uuid, user_id_filter: &Uuid) -> Result<bool, NotificationError> {
        let mut conn = self.pool.get()
            .await
            .map_err(|e| NotificationError::DatabaseError(format!("Connection failed: {}", e)))?;

        let rows_updated = diesel::update(
            notifications
                .filter(id.eq(notification_id))
                .filter(user_id.eq(user_id_filter))
        )
        .set(is_read.eq(true))
        .execute(&mut conn)
        .await
        .map_err(|e| {
            error!("Failed to mark notification {} as read: {}", notification_id, e);
            NotificationError::DatabaseError(format!("Update failed: {}", e))
        })?;

        Ok(rows_updated > 0)
    }

    // Delete a notification
    pub async fn delete_notification(&self, notification_id: &Uuid, user_id_filter: &Uuid) -> Result<bool, NotificationError> {
        let mut conn = self.pool.get()
            .await
            .map_err(|e| NotificationError::DatabaseError(format!("Connection failed: {}", e)))?;

        let rows_deleted = diesel::delete(
            notifications
                .filter(id.eq(notification_id))
                .filter(user_id.eq(user_id_filter))
        )
        .execute(&mut conn)
        .await
        .map_err(|e| {
            error!("Failed to delete notification {}: {}", notification_id, e);
            NotificationError::DatabaseError(format!("Delete failed: {}", e))
        })?;

        Ok(rows_deleted > 0)
    }

    // Get unread notification count for a user
    pub async fn get_unread_count(&self, user_id_filter: &Uuid) -> Result<i64, NotificationError> {
        let mut conn = self.pool.get()
            .await
            .map_err(|e| NotificationError::DatabaseError(format!("Connection failed: {}", e)))?;

        let count = notifications
            .filter(user_id.eq(user_id_filter))
            .filter(is_read.eq(false))
            .count()
            .get_result::<i64>(&mut conn)
            .await
            .map_err(|e| {
                error!("Failed to get unread count for user {}: {}", user_id_filter, e);
                NotificationError::DatabaseError(format!("Count query failed: {}", e))
            })?;

        Ok(count)
    }

    // Get notification statistics for a user
    pub async fn get_notification_stats(&self, user_id_filter: &Uuid) -> Result<NotificationStats, NotificationError> {
        let mut conn = self.pool.get()
            .await
            .map_err(|e| NotificationError::DatabaseError(format!("Connection failed: {}", e)))?;

        let total_count = notifications
            .filter(user_id.eq(user_id_filter))
            .count()
            .get_result::<i64>(&mut conn)
            .await
            .map_err(|e| NotificationError::DatabaseError(format!("Total count query failed: {}", e)))?;

        let unread_count = notifications
            .filter(user_id.eq(user_id_filter))
            .filter(is_read.eq(false))
            .count()
            .get_result::<i64>(&mut conn)
            .await
            .map_err(|e| NotificationError::DatabaseError(format!("Unread count query failed: {}", e)))?;

        let critical_count = notifications
            .filter(user_id.eq(user_id_filter))
            .filter(priority.eq("critical"))
            .count()
            .get_result::<i64>(&mut conn)
            .await
            .map_err(|e| NotificationError::DatabaseError(format!("Critical count query failed: {}", e)))?;

        let today_start = Utc::now().date_naive().and_hms_opt(0, 0, 0).unwrap();
        let today_count = notifications
            .filter(user_id.eq(user_id_filter))
            .filter(created_at.gt(today_start))
            .count()
            .get_result::<i64>(&mut conn)
            .await
            .map_err(|e| NotificationError::DatabaseError(format!("Today count query failed: {}", e)))?;

        let last_notification_at = notifications
            .filter(user_id.eq(user_id_filter))
            .order(created_at.desc())
            .select(created_at)
            .first::<DateTime<Utc>>(&mut conn)
            .await
            .optional()
            .map_err(|e| NotificationError::DatabaseError(format!("Last notification query failed: {}", e)))?;

        Ok(NotificationStats {
            total_notifications: total_count,
            unread_count,
            critical_count,
            today_count,
            last_notification_at,
        })
    }

    // Bulk mark notifications as read
    pub async fn bulk_mark_as_read(&self, user_id_filter: &Uuid, notification_ids: &[Uuid]) -> Result<usize, NotificationError> {
        let mut conn = self.pool.get()
            .await
            .map_err(|e| NotificationError::DatabaseError(format!("Connection failed: {}", e)))?;

        let rows_updated = diesel::update(
            notifications
                .filter(user_id.eq(user_id_filter))
                .filter(id.eq_any(notification_ids))
        )
        .set(is_read.eq(true))
        .execute(&mut conn)
        .await
        .map_err(|e| {
            error!("Failed to bulk mark notifications as read: {}", e);
            NotificationError::DatabaseError(format!("Bulk update failed: {}", e))
        })?;

        if rows_updated > 0 {
            info!("Bulk marked {} notifications as read for user {}", rows_updated, user_id_filter);
        }

        Ok(rows_updated)
    }

    // Clean up expired notifications
    pub async fn cleanup_expired_notifications(&self) -> Result<usize, NotificationError> {
        let mut conn = self.pool.get()
            .await
            .map_err(|e| NotificationError::DatabaseError(format!("Connection failed: {}", e)))?;

        let rows_deleted = diesel::delete(
            notifications.filter(expires_at.lt(Utc::now()))
        )
        .execute(&mut conn)
        .await
        .map_err(|e| {
            error!("Failed to cleanup expired notifications: {}", e);
            NotificationError::DatabaseError(format!("Cleanup failed: {}", e))
        })?;

        if rows_deleted > 0 {
            info!("Cleaned up {} expired notifications", rows_deleted);
        }

        Ok(rows_deleted)
    }

    // Helper method for cache cleanup - currently stubbed
    async fn _invalidate_user_cache(&self, user_id: &Uuid) {
        if let Some(_cache) = &self.cache {
            warn!("Cache invalidation stubbed - implement when cache trait is ready");
            debug!("Would invalidate cache for user: {}", user_id);
        }
    }
}