use axum::{
    response::IntoResponse,
    Json,
    Extension,
};
use std::sync::Arc;
use uuid::Uuid;
use tracing::info;

use crate::core::errors::AppError;
use crate::infra::services::{FcmService, FcmTopicService, FcmNotification};
use crate::infra::db::diesel::repos::{UserNotificationRepository, UserNotificationWithDetails};
use crate::web::notifications::dto::*;
use crate::web::middleware::AuthenticatedUser;

#[derive(Debug, serde::Deserialize)]
pub struct PaginationQuery {
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

/// Admin notification sending handler
pub async fn admin_send_notification(
    Extension(fcm_service): Extension<Arc<FcmService>>,
    Extension(auth_user): Extension<AuthenticatedUser>,
    Json(request): Json<SendNotificationRequest>,
) -> Result<impl IntoResponse, AppError> {
    // Check admin permissions
    if !auth_user.valid_permissions.iter().any(|p| p.starts_with("admin:")) {
        return Err(AppError {
            kind: crate::core::errors::ErrorKind::AuthorizationError,
            message: "Admin access required".to_string(),
            context: crate::core::errors::ErrorContext::default(),
            correlation_id: Uuid::new_v4().to_string(),
            timestamp: chrono::Utc::now(),
            stack_trace: None,
        });
    }

    info!("Admin {} sending notification: {}", auth_user.user_id, request.title);

    // Create FCM notification
    let notification = FcmNotification {
        title: Some(request.title.clone()),
        body: Some(request.body.clone()),
        image: request.image_url.clone(),
    };

    let notification_id = Uuid::new_v4();

    // Send to specific user or topic
    if let Some(topic_name) = &request.fcm_topic_id {
        let message_id = fcm_service
            .send_to_topic(topic_name.clone(), notification, request.data_payload)
            .await?;

        let response = SendNotificationResponse {
            id: notification_id,
            message: "Notification sent to topic successfully".to_string(),
            recipient_count: None,
            delivery_ids: vec![],
        };

        Ok(Json(response))
    } else if let Some(user_id) = request.recipient_user_id {
        // For specific user notifications, we would need FCM token lookup
        // For now, return an error indicating this feature needs implementation
        Err(AppError {
            kind: crate::core::errors::ErrorKind::InternalError,
            message: "Direct user notification not yet implemented".to_string(),
            context: crate::core::errors::ErrorContext::default(),
            correlation_id: Uuid::new_v4().to_string(),
            timestamp: chrono::Utc::now(),
            stack_trace: None,
        })
    } else {
        Err(AppError {
            kind: crate::core::errors::ErrorKind::ValidationError,
            message: "Either fcm_topic_id or recipient_user_id must be specified".to_string(),
            context: crate::core::errors::ErrorContext::default(),
            correlation_id: Uuid::new_v4().to_string(),
            timestamp: chrono::Utc::now(),
            stack_trace: None,
        })
    }
}

/// Admin topic broadcast handler
pub async fn admin_broadcast_to_topic(
    Extension(fcm_topic_service): Extension<Arc<FcmTopicService>>,
    Extension(auth_user): Extension<AuthenticatedUser>,
    Json(request): Json<BroadcastRequest>,
) -> Result<impl IntoResponse, AppError> {
    // Check admin permissions
    if !auth_user.valid_permissions.iter().any(|p| p.starts_with("admin:")) {
        return Err(AppError {
            kind: crate::core::errors::ErrorKind::AuthorizationError,
            message: "Admin access required".to_string(),
            context: crate::core::errors::ErrorContext::default(),
            correlation_id: Uuid::new_v4().to_string(),
            timestamp: chrono::Utc::now(),
            stack_trace: None,
        });
    }

    info!("Admin {} broadcasting to topic: {}", auth_user.user_id, request.topic);

    let message_id = fcm_topic_service
        .broadcast_to_topic(&request.topic, &request.title, &request.body, request.data)
        .await?;

    let response = BroadcastResponse {
        message_id,
        topic: request.topic,
        sent_at: chrono::Utc::now(),
    };

    Ok(Json(response))
}

/// Admin notification stats handler
pub async fn admin_get_notification_stats(
    Extension(auth_user): Extension<AuthenticatedUser>,
    Extension(repo): Extension<Arc<UserNotificationRepository>>,
) -> Result<impl IntoResponse, AppError> {
    // Check admin permissions
    if !auth_user.valid_permissions.iter().any(|p| p.starts_with("admin:")) {
        return Err(AppError {
            kind: crate::core::errors::ErrorKind::AuthorizationError,
            message: "Admin access required".to_string(),
            context: crate::core::errors::ErrorContext::default(),
            correlation_id: Uuid::new_v4().to_string(),
            timestamp: chrono::Utc::now(),
            stack_trace: None,
        });
    }

    info!("Admin {} fetching notification stats", auth_user.user_id);

    let stats = repo.get_admin_notification_stats().await?;

    // Convert to the specific format expected by the frontend
    #[derive(serde::Serialize)]
    struct AdminNotificationStatsResponse {
        #[serde(rename = "totalSent")]
        total_sent: i64,
        delivered: i64,
        failed: i64,
        pending: i64,
        #[serde(rename = "successRate")]
        success_rate: f64,
        #[serde(rename = "todaysSent")]
        todays_sent: i64,
        #[serde(rename = "todaysDelivered")]
        todays_delivered: i64,
        #[serde(rename = "avgDeliveryTime")]
        avg_delivery_time: i64,
        #[serde(rename = "peakHour")]
        peak_hour: String,
    }

    let response = AdminNotificationStatsResponse {
        total_sent: stats.total_sent,
        delivered: stats.delivered,
        failed: stats.failed,
        pending: stats.pending,
        success_rate: stats.success_rate,
        todays_sent: stats.todays_sent,
        todays_delivered: stats.todays_delivered,
        avg_delivery_time: stats.avg_delivery_time,
        peak_hour: stats.peak_hour,
    };

    Ok(Json(response))
}

/// Get recent notifications for admin dashboard
pub async fn admin_get_recent_notifications(
    Extension(auth_user): Extension<AuthenticatedUser>,
    Extension(repo): Extension<Arc<UserNotificationRepository>>,
    axum::extract::Query(pagination): axum::extract::Query<PaginationQuery>,
) -> Result<impl IntoResponse, AppError> {
    // Check admin permissions
    if !auth_user.valid_permissions.iter().any(|p| p.starts_with("admin:")) {
        return Err(AppError {
            kind: crate::core::errors::ErrorKind::AuthorizationError,
            message: "Admin access required".to_string(),
            context: crate::core::errors::ErrorContext::default(),
            correlation_id: Uuid::new_v4().to_string(),
            timestamp: chrono::Utc::now(),
            stack_trace: None,
        });
    }

    info!("Admin {} fetching recent notifications", auth_user.user_id);

    #[derive(serde::Serialize)]
    struct RecentNotification {
        id: String,
        title: String,
        body: String,
        target: String,
        #[serde(rename = "sentAt")]
        sent_at: chrono::DateTime<chrono::Utc>,
        #[serde(rename = "recipientCount")]
        recipient_count: i32,
        #[serde(rename = "deliveryStatus")]
        delivery_status: String,
        priority: String,
        #[serde(rename = "type")]
        notification_type: String,
    }

    #[derive(serde::Serialize)]
    struct RecentNotificationsResponse {
        notifications: Vec<RecentNotification>,
    }

    // Get recent notifications from database
    let recent_notifications_from_db = repo.get_recent_notifications(
        pagination.limit.unwrap_or(15)
    ).await?;
    
    let notifications = recent_notifications_from_db.into_iter().map(|n| RecentNotification {
        id: n.id.to_string(),
        title: n.title,
        body: n.body,
        target: format!("{} Users", n.notification_type),
        sent_at: n.created_at,
        recipient_count: 1, // Default recipient count - could be enhanced with actual data
        delivery_status: if n.read_at.is_some() { "delivered" } else { "sent" }.to_string(),
        priority: n.priority,
        notification_type: n.notification_type,
    }).collect();

    Ok(Json(RecentNotificationsResponse { notifications }))
}

/// Get admin unread notifications
pub async fn admin_get_unread_notifications(
    Extension(auth_user): Extension<AuthenticatedUser>,
    Extension(repo): Extension<Arc<UserNotificationRepository>>,
) -> Result<impl IntoResponse, AppError> {
    // Check admin permissions
    if !auth_user.valid_permissions.iter().any(|p| p.starts_with("admin:")) {
        return Err(AppError {
            kind: crate::core::errors::ErrorKind::AuthorizationError,
            message: "Admin access required".to_string(),
            context: crate::core::errors::ErrorContext::default(),
            correlation_id: Uuid::new_v4().to_string(),
            timestamp: chrono::Utc::now(),
            stack_trace: None,
        });
    }

    info!("Admin {} fetching unread notifications", auth_user.user_id);

    let notifications = repo.get_unread_notifications(&auth_user.user_id).await?;
    
    let response = UserNotificationsResponse {
        notifications: notifications.iter().map(|n| UserNotification {
            id: n.id,
            title: n.title.clone(),
            body: n.body.clone(),
            notification_type: n.notification_type.clone(),
            priority: n.priority.clone(),
            image_url: n.image_url.clone(),
            action_url: n.action_url.clone(),
            data_payload: n.data_payload.clone(),
            created_at: n.created_at,
            read_at: n.read_at,
            clicked_at: n.clicked_at,
        }).collect(),
        total_count: notifications.len() as i64,
        unread_count: notifications.len() as i64,
    };

    Ok(Json(response))
}

/// Get notification history for admin
pub async fn admin_get_notification_history(
    Extension(auth_user): Extension<AuthenticatedUser>,
    Extension(repo): Extension<Arc<UserNotificationRepository>>,
    axum::extract::Query(pagination): axum::extract::Query<PaginationQuery>,
) -> Result<impl IntoResponse, AppError> {
    // Check admin permissions
    if !auth_user.valid_permissions.iter().any(|p| p.starts_with("admin:")) {
        return Err(AppError {
            kind: crate::core::errors::ErrorKind::AuthorizationError,
            message: "Admin access required".to_string(),
            context: crate::core::errors::ErrorContext::default(),
            correlation_id: Uuid::new_v4().to_string(),
            timestamp: chrono::Utc::now(),
            stack_trace: None,
        });
    }

    info!("Admin {} fetching notification history", auth_user.user_id);

    #[derive(serde::Serialize)]
    struct NotificationHistoryResponse {
        notifications: Vec<UserNotification>,
        #[serde(rename = "totalCount")]
        total_count: i64,
    }

    // Get notification history from database
    let history_notifications = repo.get_notification_history(
        pagination.limit.unwrap_or(50),
        pagination.offset.unwrap_or(0)
    ).await?;
    
    let total_count = repo.get_notification_count().await.unwrap_or(0);
    
    let notifications = history_notifications.into_iter().map(|n| UserNotification {
        id: n.id,
        title: n.title,
        body: n.body,
        notification_type: n.notification_type,
        priority: n.priority,
        image_url: n.image_url,
        action_url: n.action_url,
        data_payload: n.data_payload,
        created_at: n.created_at,
        read_at: n.read_at,
        clicked_at: n.clicked_at,
    }).collect();

    Ok(Json(NotificationHistoryResponse {
        notifications,
        total_count,
    }))
}

/// Admin security alert handler
pub async fn admin_send_security_alert(
    Extension(fcm_topic_service): Extension<Arc<FcmTopicService>>,
    Extension(auth_user): Extension<AuthenticatedUser>,
    Json(request): Json<BroadcastRequest>,
) -> Result<impl IntoResponse, AppError> {
    // Check admin permissions
    if !auth_user.valid_permissions.iter().any(|p| p.starts_with("admin:")) {
        return Err(AppError {
            kind: crate::core::errors::ErrorKind::AuthorizationError,
            message: "Admin access required".to_string(),
            context: crate::core::errors::ErrorContext::default(),
            correlation_id: Uuid::new_v4().to_string(),
            timestamp: chrono::Utc::now(),
            stack_trace: None,
        });
    }

    info!("Admin {} sending security alert: {}", auth_user.user_id, request.title);

    let message_id = fcm_topic_service
        .broadcast_security_alert(&request.title, &request.body, &request.priority)
        .await?;

    let response = BroadcastResponse {
        message_id,
        topic: "epsx_security_alerts".to_string(),
        sent_at: chrono::Utc::now(),
    };

    Ok(Json(response))
}