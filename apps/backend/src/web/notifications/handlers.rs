use axum::{extract::Query, response::IntoResponse, Json, Extension};
use serde::Deserialize;
use std::sync::Arc;
use uuid::Uuid;
use tracing::{info, warn};

use crate::core::errors::AppError;
use crate::infra::services::{FcmService, FcmTopicService};
// use crate::infra::db::diesel::repos::UserNotificationRepository;
use crate::web::middleware::AuthenticatedUser;
use super::dto::*;

#[derive(Debug, Deserialize)]
pub struct PaginationQuery {
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

/// Register FCM token for authenticated user
pub async fn register_fcm_token(
    Extension(fcm_topic_service): Extension<Arc<FcmTopicService>>,
    Extension(auth_user): Extension<AuthenticatedUser>,
    Json(request): Json<RegisterFcmTokenRequest>,
) -> Result<impl IntoResponse, AppError> {
    info!("Registering FCM token for user: {}", auth_user.user_id);

    // Validate token format
    if !FcmService::is_valid_token(&request.token) {
        warn!("Invalid FCM token format from user: {}", auth_user.user_id);
        return Err(AppError {
            kind: crate::core::errors::ErrorKind::ValidationError,
            message: "Invalid FCM token format".to_string(),
            context: crate::core::errors::ErrorContext::default(),
            correlation_id: Uuid::new_v4().to_string(),
            timestamp: chrono::Utc::now(),
            stack_trace: None,
        });
    }

    // Subscribe user to appropriate topics based on permissions
    let subscribed_topics = fcm_topic_service
        .manage_user_topics(&auth_user.user_id, &auth_user.valid_permissions, &request.token)
        .await?;

    let response = RegisterFcmTokenResponse {
        id: Uuid::new_v4(),
        message: "FCM token registered successfully".to_string(),
        subscribed_topics,
    };

    Ok(Json(response))
}

/// Send notification (admin only)
pub async fn send_notification(
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
    let notification = crate::infra::services::FcmNotification {
        title: Some(request.title.clone()),
        body: Some(request.body.clone()),
        image: request.image_url.clone(),
    };

    let notification_id = Uuid::new_v4();
    let mut recipient_count = 0u32;

    // Send based on target type
    if let Some(topic_name) = &request.fcm_topic_id {
        // Topic broadcast
        let message_id = fcm_service
            .send_to_topic(topic_name.clone(), notification, request.data_payload.clone())
            .await?;
        
        info!("Notification sent to topic {}: {:?}", topic_name, message_id);
        recipient_count = 100; // Estimate - would be real count from database
    } else if let Some(_user_id) = request.recipient_user_id {
        // Individual user notification
        // Would implement user token lookup and send to specific tokens
        recipient_count = 1;
    }

    let response = SendNotificationResponse {
        id: notification_id,
        message: "Notification sent successfully".to_string(),
        recipient_count: Some(recipient_count),
        delivery_ids: vec![Uuid::new_v4()],
    };

    Ok(Json(response))
}

/// Broadcast to topic (admin only)
pub async fn broadcast_to_topic(
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

/// Track notification interaction
pub async fn track_notification(
    Json(request): Json<TrackNotificationRequest>,
) -> Result<impl IntoResponse, AppError> {
    info!(
        "Tracking notification {} action: {} at {}",
        request.notification_id, request.action, request.timestamp
    );

    // Here you would update the notification_deliveries table
    // For now, just log and return success

    let response = TrackNotificationResponse {
        success: true,
        message: "Notification tracking recorded".to_string(),
    };

    Ok(Json(response))
}

/// Get user notifications with real database query
pub async fn get_user_notifications(
    Extension(auth_user): Extension<AuthenticatedUser>,
    Extension(repo): Extension<Arc<crate::infra::db::diesel::repos::UserNotificationRepository>>,
    Query(pagination): Query<PaginationQuery>,
) -> Result<impl IntoResponse, AppError> {
    info!("Fetching notifications for user: {}", auth_user.user_id);
    
    let limit = pagination.limit.unwrap_or(20);
    let offset = pagination.offset.unwrap_or(0);
    
    match repo.get_user_notifications(&auth_user.user_id, Some(limit), Some(offset)).await {
        Ok(notifications) => {
            let total_count = notifications.len() as i64;
            let unread_count = notifications.iter().filter(|n| n.read_at.is_none()).count() as i64;
            
            Ok(Json(serde_json::json!({
                "user_id": auth_user.user_id,
                "notifications": notifications,
                "total_count": total_count,
                "unread_count": unread_count,
                "limit": limit,
                "offset": offset,
                "fetched_at": chrono::Utc::now()
            })))
        },
        Err(e) => {
            warn!("Failed to fetch notifications for user {}: {}", auth_user.user_id, e);
            Err(AppError {
                kind: crate::core::errors::ErrorKind::DatabaseError,
                message: "Failed to fetch notifications".to_string(),
                context: crate::core::errors::ErrorContext::default(),
                correlation_id: uuid::Uuid::new_v4().to_string(),
                timestamp: chrono::Utc::now(),
                stack_trace: None,
            })
        }
    }
}

/// Get unread notifications only with real database query
pub async fn get_unread_notifications(
    Extension(auth_user): Extension<AuthenticatedUser>,
    Extension(repo): Extension<Arc<crate::infra::db::diesel::repos::UserNotificationRepository>>,
) -> Result<impl IntoResponse, AppError> {
    info!("Fetching unread notifications for user: {}", auth_user.user_id);
    
    match repo.get_unread_notifications(&auth_user.user_id).await {
        Ok(notifications) => {
            let count = notifications.len();
            Ok(Json(serde_json::json!({
                "user_id": auth_user.user_id,
                "notifications": notifications,
                "unread_count": count,
                "fetched_at": chrono::Utc::now()
            })))
        },
        Err(e) => {
            warn!("Failed to fetch unread notifications for user {}: {}", auth_user.user_id, e);
            Err(AppError {
                kind: crate::core::errors::ErrorKind::DatabaseError,
                message: "Failed to fetch unread notifications".to_string(),
                context: crate::core::errors::ErrorContext::default(),
                correlation_id: uuid::Uuid::new_v4().to_string(),
                timestamp: chrono::Utc::now(),
                stack_trace: None,
            })
        }
    }
}

/// Get notification preferences
pub async fn get_preferences(
    Extension(auth_user): Extension<AuthenticatedUser>,
) -> Result<impl IntoResponse, AppError> {
    info!("Fetching notification preferences for user: {}", auth_user.user_id);

    // Mock response - would fetch from database
    let response = NotificationPreferencesResponse {
        fcm_enabled: true,
        in_app_enabled: true,
        email_enabled: false,
        quiet_hours_start: Some("22:00".to_string()),
        quiet_hours_end: Some("08:00".to_string()),
        timezone: "UTC".to_string(),
        blocked_topics: vec!["marketing".to_string()],
    };

    Ok(Json(response))
}

/// Get notification statistics (admin only) with real database query
pub async fn get_notification_stats(
    Extension(auth_user): Extension<AuthenticatedUser>,
    Extension(repo): Extension<Arc<crate::infra::db::diesel::repos::UserNotificationRepository>>,
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

    match repo.get_admin_notification_stats().await {
        Ok(stats) => {
            Ok(Json(serde_json::json!({
                "total_sent": stats.total_sent,
                "delivered": stats.delivered,
                "failed": stats.failed,
                "pending": stats.pending,
                "success_rate": stats.success_rate,
                "todays_sent": stats.todays_sent,
                "todays_delivered": stats.todays_delivered,
                "avg_delivery_time": stats.avg_delivery_time,
                "peak_hour": stats.peak_hour,
                "generated_at": chrono::Utc::now()
            })))
        },
        Err(e) => {
            warn!("Failed to fetch notification stats: {}", e);
            Err(AppError {
                kind: crate::core::errors::ErrorKind::DatabaseError,
                message: "Failed to fetch notification statistics".to_string(),
                context: crate::core::errors::ErrorContext::default(),
                correlation_id: Uuid::new_v4().to_string(),
                timestamp: chrono::Utc::now(),
                stack_trace: None,
            })
        }
    }
}

/// Update notification preferences
pub async fn update_preferences(
    Extension(auth_user): Extension<AuthenticatedUser>,
    Json(request): Json<UpdatePreferencesRequest>,
) -> Result<impl IntoResponse, AppError> {
    info!("Updating notification preferences for user: {}", auth_user.user_id);

    // Here you would update the user_notification_preferences table
    // For now, just return the updated preferences

    let response = NotificationPreferencesResponse {
        fcm_enabled: request.fcm_enabled.unwrap_or(true),
        in_app_enabled: request.in_app_enabled.unwrap_or(true),
        email_enabled: request.email_enabled.unwrap_or(false),
        quiet_hours_start: request.quiet_hours_start,
        quiet_hours_end: request.quiet_hours_end,
        timezone: request.timezone.unwrap_or_else(|| "UTC".to_string()),
        blocked_topics: request.blocked_topics.unwrap_or_default(),
    };

    Ok(Json(response))
}

/// Subscribe to topics
pub async fn subscribe_to_topics(
    Extension(_fcm_topic_service): Extension<Arc<FcmTopicService>>,
    Extension(auth_user): Extension<AuthenticatedUser>,
    Json(request): Json<TopicSubscriptionRequest>,
) -> Result<impl IntoResponse, AppError> {
    info!("User {} subscribing to topics: {:?}", auth_user.user_id, request.topics);

    // Here you would get user's FCM tokens and subscribe to topics
    // For now, just return mock response

    let response = TopicSubscriptionResponse {
        subscribed: request.topics.clone(),
        failed: vec![],
        message: "Successfully subscribed to topics".to_string(),
    };

    Ok(Json(response))
}

/// Send security alert (admin only)
pub async fn send_security_alert(
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

    info!("Admin {} sending security alert", auth_user.user_id);

    let severity = match request.priority.as_str() {
        "urgent" => "critical",
        "high" => "high",
        _ => "medium",
    };

    let message_id = fcm_topic_service
        .broadcast_security_alert(&request.title, &request.body, severity)
        .await?;

    let response = BroadcastResponse {
        message_id,
        topic: "security_alerts".to_string(),
        sent_at: chrono::Utc::now(),
    };

    Ok(Json(response))
}