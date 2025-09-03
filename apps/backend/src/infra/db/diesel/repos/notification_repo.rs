use async_trait::async_trait;
use chrono::{DateTime, Utc};
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use std::sync::Arc;
use uuid::Uuid;

use crate::dom::notification::{
    CreateNotificationRequest, Notification, NotificationStats, 
    NotificationRepository
};
use crate::infra::db::diesel::types::{DeliveryStatus}; 
use crate::core::errors::AppError;
use crate::infra::db::diesel::{DbPool, schema::{notifications, notification_deliveries}, models::{DieselNotification, NewDieselNotification}};

pub struct DieselNotificationRepository {
    pool: Arc<DbPool>,
}

impl DieselNotificationRepository {
    pub fn new(pool: Arc<DbPool>) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl NotificationRepository for DieselNotificationRepository {
    async fn create_notification(&self, req: CreateNotificationRequest, created_by: Option<Uuid>) -> Result<Notification, AppError> {
        let mut conn = self.pool.get().await.map_err(|e| crate::infra::error::AppError::DatabaseError(e.to_string()))?;
        
        let id = Uuid::new_v4();
        let now = Utc::now();
        
        let new_notification = NewDieselNotification {
            id,
            recipient_user_id: req.recipient_user_id,
            fcm_topic_id: req.fcm_topic_id,
            title: req.title.clone(),
            body: req.body.clone(),
            notification_type: req.notification_type,
            priority: req.priority,
            channels: req.channels.clone(),
            data_payload: req.data_payload.clone(),
            image_url: req.image_url.clone(),
            action_url: req.action_url.clone(),
            scheduled_at: req.scheduled_at,
            expires_at: req.expires_at,
            created_by,
            created_at: now,
        };

        diesel::insert_into(notifications::table)
            .values(&new_notification)
            .execute(&mut conn)
            .await
            .map_err(|e| crate::infra::error::AppError::DatabaseError(e.to_string()))?;

        Ok(Notification {
            id,
            recipient_user_id: req.recipient_user_id,
            fcm_topic_id: req.fcm_topic_id,
            title: req.title,
            body: req.body,
            notification_type: req.notification_type,
            priority: req.priority,
            channels: req.channels,
            data_payload: req.data_payload,
            image_url: req.image_url,
            action_url: req.action_url,
            scheduled_at: req.scheduled_at,
            expires_at: req.expires_at,
            created_by,
            created_at: now,
        })
    }

    async fn get_notification(&self, id: Uuid) -> Result<Option<Notification>, AppError> {
        let mut conn = self.pool.get().await.map_err(|e| crate::infra::error::AppError::DatabaseError(e.to_string()))?;
        
        let result = notifications::table
            .filter(notifications::id.eq(id))
            .select(DieselNotification::as_select())
            .first::<DieselNotification>(&mut conn)
            .await
            .optional()
            .map_err(|e| crate::infra::error::AppError::DatabaseError(e.to_string()))?;

        Ok(result.map(|n| Notification {
            id: n.id,
            recipient_user_id: n.recipient_user_id,
            fcm_topic_id: n.fcm_topic_id,
            title: n.title,
            body: n.body,
            notification_type: n.notification_type,
            priority: n.priority,
            channels: n.channels,
            data_payload: n.data_payload,
            image_url: n.image_url,
            action_url: n.action_url,
            scheduled_at: n.scheduled_at,
            expires_at: n.expires_at,
            created_by: n.created_by,
            created_at: n.created_at,
        }))
    }

    async fn get_user_notifications(&self, user_id: Uuid, limit: Option<i64>, offset: Option<i64>) -> Result<Vec<Notification>, AppError> {
        let mut conn = self.pool.get().await.map_err(|e| crate::infra::error::AppError::DatabaseError(e.to_string()))?;
        
        let mut query = notifications::table
            .filter(notifications::recipient_user_id.eq(user_id))
            .order(notifications::created_at.desc())
            .into_boxed();

        if let Some(limit) = limit {
            query = query.limit(limit);
        }
        
        if let Some(offset) = offset {
            query = query.offset(offset);
        }

        let results = query
            .select(DieselNotification::as_select())
            .load::<DieselNotification>(&mut conn)
            .await
            .map_err(|e| crate::infra::error::AppError::DatabaseError(e.to_string()))?;

        Ok(results.into_iter().map(|n| Notification {
            id: n.id,
            recipient_user_id: n.recipient_user_id,
            fcm_topic_id: n.fcm_topic_id,
            title: n.title,
            body: n.body,
            notification_type: n.notification_type,
            priority: n.priority,
            channels: n.channels,
            data_payload: n.data_payload,
            image_url: n.image_url,
            action_url: n.action_url,
            scheduled_at: n.scheduled_at,
            expires_at: n.expires_at,
            created_by: n.created_by,
            created_at: n.created_at,
        }).collect())
    }

    async fn update_notification_delivery_status(&self, delivery_id: Uuid, status: DeliveryStatus, error_message: Option<String>) -> Result<(), AppError> {
        let mut conn = self.pool.get().await.map_err(|e| crate::infra::error::AppError::DatabaseError(e.to_string()))?;
        
        let now = Utc::now();
        
        diesel::update(notification_deliveries::table)
            .filter(notification_deliveries::id.eq(delivery_id))
            .set((
                notification_deliveries::status.eq(status),
                notification_deliveries::error_message.eq(error_message),
                notification_deliveries::delivered_at.eq(if matches!(status, DeliveryStatus::Delivered) { Some(now) } else { None::<DateTime<Utc>> })
            ))
            .execute(&mut conn)
            .await
            .map_err(|e| crate::infra::error::AppError::DatabaseError(e.to_string()))?;

        Ok(())
    }

    async fn mark_notification_read(&self, delivery_id: Uuid, read_at: DateTime<Utc>) -> Result<(), AppError> {
        let mut conn = self.pool.get().await.map_err(|e| crate::infra::error::AppError::DatabaseError(e.to_string()))?;
        
        diesel::update(notification_deliveries::table)
            .filter(notification_deliveries::id.eq(delivery_id))
            .set(notification_deliveries::read_at.eq(read_at))
            .execute(&mut conn)
            .await
            .map_err(|e| crate::infra::error::AppError::DatabaseError(e.to_string()))?;

        Ok(())
    }

    async fn mark_notification_clicked(&self, delivery_id: Uuid, clicked_at: DateTime<Utc>) -> Result<(), AppError> {
        let mut conn = self.pool.get().await.map_err(|e| crate::infra::error::AppError::DatabaseError(e.to_string()))?;
        
        diesel::update(notification_deliveries::table)
            .filter(notification_deliveries::id.eq(delivery_id))
            .set(notification_deliveries::clicked_at.eq(clicked_at))
            .execute(&mut conn)
            .await
            .map_err(|e| crate::infra::error::AppError::DatabaseError(e.to_string()))?;

        Ok(())
    }

    async fn get_scheduled_notifications(&self, before: DateTime<Utc>) -> Result<Vec<Notification>, AppError> {
        let mut conn = self.pool.get().await.map_err(|e| crate::infra::error::AppError::DatabaseError(e.to_string()))?;
        
        let results = notifications::table
            .filter(notifications::scheduled_at.le(before))
            .filter(notifications::scheduled_at.is_not_null())
            .select(DieselNotification::as_select())
            .load::<DieselNotification>(&mut conn)
            .await
            .map_err(|e| crate::infra::error::AppError::DatabaseError(e.to_string()))?;

        Ok(results.into_iter().map(|n| Notification {
            id: n.id,
            recipient_user_id: n.recipient_user_id,
            fcm_topic_id: n.fcm_topic_id,
            title: n.title,
            body: n.body,
            notification_type: n.notification_type,
            priority: n.priority,
            channels: n.channels,
            data_payload: n.data_payload,
            image_url: n.image_url,
            action_url: n.action_url,
            scheduled_at: n.scheduled_at,
            expires_at: n.expires_at,
            created_by: n.created_by,
            created_at: n.created_at,
        }).collect())
    }

    async fn get_notification_stats(&self, user_id: Option<Uuid>, start_date: Option<DateTime<Utc>>, end_date: Option<DateTime<Utc>>) -> Result<NotificationStats, AppError> {
        let mut conn = self.pool.get().await.map_err(|e| crate::infra::error::AppError::DatabaseError(e.to_string()))?;
        
        let mut query = notification_deliveries::table.into_boxed();
        
        if let Some(user_id) = user_id {
            query = query.filter(notification_deliveries::user_id.eq(user_id));
        }
        
        if let Some(start_date) = start_date {
            query = query.filter(notification_deliveries::created_at.ge(start_date));
        }
        
        if let Some(end_date) = end_date {
            query = query.filter(notification_deliveries::created_at.le(end_date));
        }

        let total_sent = query.clone().filter(notification_deliveries::status.ne(DeliveryStatus::Pending)).count().get_result::<i64>(&mut conn).await.map_err(|e| crate::infra::error::AppError::DatabaseError(e.to_string()))?;
        let total_delivered = query.clone().filter(notification_deliveries::status.eq(DeliveryStatus::Delivered)).count().get_result::<i64>(&mut conn).await.map_err(|e| crate::infra::error::AppError::DatabaseError(e.to_string()))?;
        let total_failed = query.clone().filter(notification_deliveries::status.eq(DeliveryStatus::Failed)).count().get_result::<i64>(&mut conn).await.map_err(|e| crate::infra::error::AppError::DatabaseError(e.to_string()))?;
        let total_pending = query.filter(notification_deliveries::status.eq(DeliveryStatus::Pending)).count().get_result::<i64>(&mut conn).await.map_err(|e| crate::infra::error::AppError::DatabaseError(e.to_string()))?;

        Ok(NotificationStats {
            total_sent,
            total_delivered,
            total_failed,
            total_pending,
        })
    }
}