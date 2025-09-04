use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use chrono::{DateTime, Utc};
use uuid::Uuid;
use crate::infra::db::diesel::{DbPool, schema::{user_notifications, notifications}};
use crate::core::errors::AppError;
use std::sync::Arc;

pub struct UserNotificationRepository {
    pool: Arc<DbPool>,
}

impl UserNotificationRepository {
    pub fn new(pool: Arc<DbPool>) -> Self {
        Self { pool }
    }

    /// Get user notifications with join to notifications table
    pub async fn get_user_notifications(&self, user_firebase_uid: &str, limit: Option<i64>, offset: Option<i64>) -> Result<Vec<UserNotificationWithDetails>, AppError> {
        let mut conn = self.pool.get().await
            .map_err(|e| AppError {
                kind: crate::core::errors::ErrorKind::DatabaseError,
                message: format!("Failed to get database connection: {}", e),
                context: crate::core::errors::ErrorContext::default(),
                correlation_id: Uuid::new_v4().to_string(),
                timestamp: chrono::Utc::now(),
                stack_trace: None,
            })?;

        let mut query = notifications::table
            .inner_join(user_notifications::table.on(notifications::id.eq(user_notifications::notification_id)))
            .filter(user_notifications::user_id.eq(user_firebase_uid))
            .order(notifications::created_at.desc())
            .into_boxed();

        if let Some(limit) = limit {
            query = query.limit(limit);
        }
        
        if let Some(offset) = offset {
            query = query.offset(offset);
        }

        let results = query
            .select((
                notifications::id,
                notifications::title,
                notifications::body,
                notifications::notification_type,
                notifications::priority,
                notifications::image_url,
                notifications::action_url,
                notifications::data_payload,
                notifications::created_at,
                user_notifications::read_at,
                user_notifications::clicked_at,
            ))
            .load::<UserNotificationJoin>(&mut conn)
            .await
            .map_err(|e| AppError {
                kind: crate::core::errors::ErrorKind::DatabaseError,
                message: format!("Failed to fetch user notifications: {}", e),
                context: crate::core::errors::ErrorContext::default(),
                correlation_id: Uuid::new_v4().to_string(),
                timestamp: chrono::Utc::now(),
                stack_trace: None,
            })?;

        Ok(results.into_iter().map(|r| UserNotificationWithDetails {
            id: r.id,
            title: r.title,
            body: r.body,
            notification_type: format!("{:?}", r.notification_type).to_lowercase(),
            priority: format!("{:?}", r.priority).to_lowercase(),
            image_url: r.image_url,
            action_url: r.action_url,
            data_payload: r.data_payload,
            created_at: r.created_at.unwrap_or_else(|| Utc::now()),
            read_at: r.read_at,
            clicked_at: r.clicked_at,
        }).collect())
    }

    /// Get unread notifications for user
    pub async fn get_unread_notifications(&self, user_firebase_uid: &str) -> Result<Vec<UserNotificationWithDetails>, AppError> {
        let mut conn = self.pool.get().await
            .map_err(|e| AppError {
                kind: crate::core::errors::ErrorKind::DatabaseError,
                message: format!("Failed to get database connection: {}", e),
                context: crate::core::errors::ErrorContext::default(),
                correlation_id: Uuid::new_v4().to_string(),
                timestamp: chrono::Utc::now(),
                stack_trace: None,
            })?;

        let results = notifications::table
            .inner_join(user_notifications::table.on(notifications::id.eq(user_notifications::notification_id)))
            .filter(user_notifications::user_id.eq(user_firebase_uid))
            .filter(user_notifications::read_at.is_null())
            .order(notifications::created_at.desc())
            .select((
                notifications::id,
                notifications::title,
                notifications::body,
                notifications::notification_type,
                notifications::priority,
                notifications::image_url,
                notifications::action_url,
                notifications::data_payload,
                notifications::created_at,
                user_notifications::read_at,
                user_notifications::clicked_at,
            ))
            .load::<UserNotificationJoin>(&mut conn)
            .await
            .map_err(|e| AppError {
                kind: crate::core::errors::ErrorKind::DatabaseError,
                message: format!("Failed to fetch unread notifications: {}", e),
                context: crate::core::errors::ErrorContext::default(),
                correlation_id: Uuid::new_v4().to_string(),
                timestamp: chrono::Utc::now(),
                stack_trace: None,
            })?;

        Ok(results.into_iter().map(|r| UserNotificationWithDetails {
            id: r.id,
            title: r.title,
            body: r.body,
            notification_type: format!("{:?}", r.notification_type).to_lowercase(),
            priority: format!("{:?}", r.priority).to_lowercase(),
            image_url: r.image_url,
            action_url: r.action_url,
            data_payload: r.data_payload,
            created_at: r.created_at.unwrap_or_else(|| Utc::now()),
            read_at: r.read_at,
            clicked_at: r.clicked_at,
        }).collect())
    }

    /// Get notification statistics for admin dashboard
    pub async fn get_admin_notification_stats(&self) -> Result<AdminNotificationStats, AppError> {
        let mut conn = self.pool.get().await
            .map_err(|e| AppError {
                kind: crate::core::errors::ErrorKind::DatabaseError,
                message: format!("Failed to get database connection: {}", e),
                context: crate::core::errors::ErrorContext::default(),
                correlation_id: Uuid::new_v4().to_string(),
                timestamp: chrono::Utc::now(),
                stack_trace: None,
            })?;

        // Get total counts
        let total_sent = notifications::table.count().get_result::<i64>(&mut conn).await.unwrap_or(0);
        let total_delivered = user_notifications::table
            .filter(user_notifications::delivery_status.eq("delivered"))
            .count().get_result::<i64>(&mut conn).await.unwrap_or(0);
        let total_failed = user_notifications::table
            .filter(user_notifications::delivery_status.eq("failed"))
            .count().get_result::<i64>(&mut conn).await.unwrap_or(0);
        let total_pending = user_notifications::table
            .filter(user_notifications::delivery_status.eq("pending"))
            .count().get_result::<i64>(&mut conn).await.unwrap_or(0);

        // Get today's stats
        let today_start = Utc::now().date_naive().and_hms_opt(0, 0, 0).unwrap().and_utc();
        let todays_sent = notifications::table
            .filter(notifications::created_at.ge(today_start))
            .count().get_result::<i64>(&mut conn).await.unwrap_or(0);
        let todays_delivered = user_notifications::table
            .filter(user_notifications::delivery_status.eq("delivered"))
            .filter(user_notifications::delivered_at.ge(today_start))
            .count().get_result::<i64>(&mut conn).await.unwrap_or(0);

        let success_rate = if total_sent > 0 {
            (total_delivered as f64 / total_sent as f64) * 100.0
        } else {
            0.0
        };

        Ok(AdminNotificationStats {
            total_sent,
            delivered: total_delivered,
            failed: total_failed,
            pending: total_pending,
            success_rate,
            todays_sent,
            todays_delivered,
            avg_delivery_time: 1250, // Mock for now
            peak_hour: "14:00-15:00".to_string(), // Mock for now
        })
    }

    /// Mark notification as read
    pub async fn mark_as_read(&self, user_firebase_uid: &str, notification_id: Uuid) -> Result<(), AppError> {
        let mut conn = self.pool.get().await
            .map_err(|e| AppError {
                kind: crate::core::errors::ErrorKind::DatabaseError,
                message: format!("Failed to get database connection: {}", e),
                context: crate::core::errors::ErrorContext::default(),
                correlation_id: Uuid::new_v4().to_string(),
                timestamp: chrono::Utc::now(),
                stack_trace: None,
            })?;

        diesel::update(user_notifications::table)
            .filter(user_notifications::user_id.eq(user_firebase_uid))
            .filter(user_notifications::notification_id.eq(notification_id))
            .set(user_notifications::read_at.eq(Some(Utc::now())))
            .execute(&mut conn)
            .await
            .map_err(|e| AppError {
                kind: crate::core::errors::ErrorKind::DatabaseError,
                message: format!("Failed to mark notification as read: {}", e),
                context: crate::core::errors::ErrorContext::default(),
                correlation_id: Uuid::new_v4().to_string(),
                timestamp: chrono::Utc::now(),
                stack_trace: None,
            })?;

        Ok(())
    }

    /// Mark notification as read for all users (admin operation)
    pub async fn mark_notification_as_read_for_all(&self, notification_id: Uuid) -> Result<usize, AppError> {
        let mut conn = self.pool.get().await
            .map_err(|e| AppError {
                kind: crate::core::errors::ErrorKind::DatabaseError,
                message: format!("Failed to get database connection: {}", e),
                context: crate::core::errors::ErrorContext::default(),
                correlation_id: Uuid::new_v4().to_string(),
                timestamp: chrono::Utc::now(),
                stack_trace: None,
            })?;

        let rows_affected = diesel::update(user_notifications::table)
            .filter(user_notifications::notification_id.eq(notification_id))
            .filter(user_notifications::read_at.is_null())
            .set(user_notifications::read_at.eq(Some(Utc::now())))
            .execute(&mut conn)
            .await
            .map_err(|e| AppError {
                kind: crate::core::errors::ErrorKind::DatabaseError,
                message: format!("Failed to mark notification as read for all: {}", e),
                context: crate::core::errors::ErrorContext::default(),
                correlation_id: Uuid::new_v4().to_string(),
                timestamp: chrono::Utc::now(),
                stack_trace: None,
            })?;

        Ok(rows_affected)
    }

    /// Get recent notifications for admin dashboard  
    pub async fn get_recent_notifications(&self, limit: i64) -> Result<Vec<UserNotificationWithDetails>, AppError> {
        let mut conn = self.pool.get().await
            .map_err(|e| AppError {
                kind: crate::core::errors::ErrorKind::DatabaseError,
                message: format!("Failed to get database connection: {}", e),
                context: crate::core::errors::ErrorContext::default(),
                correlation_id: Uuid::new_v4().to_string(),
                timestamp: chrono::Utc::now(),
                stack_trace: None,
            })?;

        let results = notifications::table
            .inner_join(user_notifications::table.on(notifications::id.eq(user_notifications::notification_id)))
            .order(notifications::created_at.desc())
            .limit(limit)
            .select((
                notifications::id,
                notifications::title,
                notifications::body,
                notifications::notification_type,
                notifications::priority,
                notifications::image_url,
                notifications::action_url,
                notifications::data_payload,
                notifications::created_at,
                user_notifications::read_at,
                user_notifications::clicked_at,
            ))
            .load::<UserNotificationJoin>(&mut conn)
            .await
            .map_err(|e| AppError {
                kind: crate::core::errors::ErrorKind::DatabaseError,
                message: format!("Failed to fetch recent notifications: {}", e),
                context: crate::core::errors::ErrorContext::default(),
                correlation_id: Uuid::new_v4().to_string(),
                timestamp: chrono::Utc::now(),
                stack_trace: None,
            })?;

        Ok(results.into_iter().map(|r| UserNotificationWithDetails {
            id: r.id,
            title: r.title,
            body: r.body,
            notification_type: format!("{:?}", r.notification_type).to_lowercase(),
            priority: format!("{:?}", r.priority).to_lowercase(),
            image_url: r.image_url,
            action_url: r.action_url,
            data_payload: r.data_payload,
            created_at: r.created_at.unwrap_or_else(|| Utc::now()),
            read_at: r.read_at,
            clicked_at: r.clicked_at,
        }).collect())
    }

    /// Get notification history for admin dashboard
    pub async fn get_notification_history(&self, limit: i64, offset: i64) -> Result<Vec<UserNotificationWithDetails>, AppError> {
        let mut conn = self.pool.get().await
            .map_err(|e| AppError {
                kind: crate::core::errors::ErrorKind::DatabaseError,
                message: format!("Failed to get database connection: {}", e),
                context: crate::core::errors::ErrorContext::default(),
                correlation_id: Uuid::new_v4().to_string(),
                timestamp: chrono::Utc::now(),
                stack_trace: None,
            })?;

        let results = notifications::table
            .inner_join(user_notifications::table.on(notifications::id.eq(user_notifications::notification_id)))
            .order(notifications::created_at.desc())
            .limit(limit)
            .offset(offset)
            .select((
                notifications::id,
                notifications::title,
                notifications::body,
                notifications::notification_type,
                notifications::priority,
                notifications::image_url,
                notifications::action_url,
                notifications::data_payload,
                notifications::created_at,
                user_notifications::read_at,
                user_notifications::clicked_at,
            ))
            .load::<UserNotificationJoin>(&mut conn)
            .await
            .map_err(|e| AppError {
                kind: crate::core::errors::ErrorKind::DatabaseError,
                message: format!("Failed to fetch notification history: {}", e),
                context: crate::core::errors::ErrorContext::default(),
                correlation_id: Uuid::new_v4().to_string(),
                timestamp: chrono::Utc::now(),
                stack_trace: None,
            })?;

        Ok(results.into_iter().map(|r| UserNotificationWithDetails {
            id: r.id,
            title: r.title,
            body: r.body,
            notification_type: format!("{:?}", r.notification_type).to_lowercase(),
            priority: format!("{:?}", r.priority).to_lowercase(),
            image_url: r.image_url,
            action_url: r.action_url,
            data_payload: r.data_payload,
            created_at: r.created_at.unwrap_or_else(|| Utc::now()),
            read_at: r.read_at,
            clicked_at: r.clicked_at,
        }).collect())
    }

    /// Get total notification count for pagination
    pub async fn get_notification_count(&self) -> Result<i64, AppError> {
        let mut conn = self.pool.get().await
            .map_err(|e| AppError {
                kind: crate::core::errors::ErrorKind::DatabaseError,
                message: format!("Failed to get database connection: {}", e),
                context: crate::core::errors::ErrorContext::default(),
                correlation_id: Uuid::new_v4().to_string(),
                timestamp: chrono::Utc::now(),
                stack_trace: None,
            })?;

        notifications::table.count().get_result::<i64>(&mut conn).await
            .map_err(|e| AppError {
                kind: crate::core::errors::ErrorKind::DatabaseError,
                message: format!("Failed to get notification count: {}", e),
                context: crate::core::errors::ErrorContext::default(),
                correlation_id: Uuid::new_v4().to_string(),
                timestamp: chrono::Utc::now(),
                stack_trace: None,
            })
    }

    /// Get all notifications for admin dashboard
    pub async fn get_all_notifications(&self, limit: Option<i64>, offset: Option<i64>) -> Result<Vec<UserNotificationWithDetails>, AppError> {
        let mut conn = self.pool.get().await
            .map_err(|e| AppError {
                kind: crate::core::errors::ErrorKind::DatabaseError,
                message: format!("Failed to get database connection: {}", e),
                context: crate::core::errors::ErrorContext::default(),
                correlation_id: Uuid::new_v4().to_string(),
                timestamp: chrono::Utc::now(),
                stack_trace: None,
            })?;

        let mut query = notifications::table
            .inner_join(user_notifications::table.on(notifications::id.eq(user_notifications::notification_id)))
            .filter(user_notifications::dismissed_at.is_null())
            .order(notifications::created_at.desc())
            .into_boxed();

        if let Some(limit) = limit {
            query = query.limit(limit);
        }
        
        if let Some(offset) = offset {
            query = query.offset(offset);
        }

        let results = query
            .select((
                notifications::id,
                notifications::title,
                notifications::body,
                notifications::notification_type,
                notifications::priority,
                notifications::image_url,
                notifications::action_url,
                notifications::data_payload,
                notifications::created_at,
                user_notifications::read_at,
                user_notifications::clicked_at,
            ))
            .load::<UserNotificationJoin>(&mut conn)
            .await
            .map_err(|e| AppError {
                kind: crate::core::errors::ErrorKind::DatabaseError,
                message: format!("Failed to fetch all notifications: {}", e),
                context: crate::core::errors::ErrorContext::default(),
                correlation_id: Uuid::new_v4().to_string(),
                timestamp: chrono::Utc::now(),
                stack_trace: None,
            })?;

        Ok(results.into_iter().map(|r| UserNotificationWithDetails {
            id: r.id,
            title: r.title,
            body: r.body,
            notification_type: format!("{:?}", r.notification_type).to_lowercase(),
            priority: format!("{:?}", r.priority).to_lowercase(),
            image_url: r.image_url,
            action_url: r.action_url,
            data_payload: r.data_payload,
            created_at: r.created_at.unwrap_or_else(|| Utc::now()),
            read_at: r.read_at,
            clicked_at: r.clicked_at,
        }).collect())
    }

    /// Delete notification (soft delete by setting dismissed_at)
    pub async fn delete_notification(&self, notification_id: Uuid) -> Result<bool, AppError> {
        let mut conn = self.pool.get().await
            .map_err(|e| AppError {
                kind: crate::core::errors::ErrorKind::DatabaseError,
                message: format!("Failed to get database connection: {}", e),
                context: crate::core::errors::ErrorContext::default(),
                correlation_id: Uuid::new_v4().to_string(),
                timestamp: chrono::Utc::now(),
                stack_trace: None,
            })?;

        let rows_affected = diesel::update(user_notifications::table)
            .filter(user_notifications::notification_id.eq(notification_id))
            .filter(user_notifications::dismissed_at.is_null())
            .set(user_notifications::dismissed_at.eq(Some(Utc::now())))
            .execute(&mut conn)
            .await
            .map_err(|e| AppError {
                kind: crate::core::errors::ErrorKind::DatabaseError,
                message: format!("Failed to delete notification: {}", e),
                context: crate::core::errors::ErrorContext::default(),
                correlation_id: Uuid::new_v4().to_string(),
                timestamp: chrono::Utc::now(),
                stack_trace: None,
            })?;

        Ok(rows_affected > 0)
    }
}

// Define the result type for joined queries
#[derive(Queryable, Debug)]
pub struct UserNotificationJoin {
    pub id: Uuid,
    pub title: String,
    pub body: String,
    pub notification_type: crate::infra::db::diesel::types::NotificationType,
    pub priority: crate::infra::db::diesel::types::NotificationPriority,
    pub image_url: Option<String>,
    pub action_url: Option<String>,
    pub data_payload: Option<serde_json::Value>,
    pub created_at: Option<DateTime<Utc>>,
    pub read_at: Option<DateTime<Utc>>,
    pub clicked_at: Option<DateTime<Utc>>,
}

// DTO for API responses
#[derive(Debug, Clone, serde::Serialize)]
pub struct UserNotificationWithDetails {
    pub id: Uuid,
    pub title: String,
    pub body: String,
    pub notification_type: String,
    pub priority: String,
    pub image_url: Option<String>,
    pub action_url: Option<String>,
    pub data_payload: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
    pub read_at: Option<DateTime<Utc>>,
    pub clicked_at: Option<DateTime<Utc>>,
}

#[derive(Debug, serde::Serialize)]
pub struct AdminNotificationStats {
    pub total_sent: i64,
    pub delivered: i64,
    pub failed: i64,
    pub pending: i64,
    pub success_rate: f64,
    pub todays_sent: i64,
    pub todays_delivered: i64,
    pub avg_delivery_time: i64,
    pub peak_hour: String,
}