// Notification API handlers with real database integration
use uuid::Uuid;

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use chrono::Utc;
use serde_json::{json, Value};
use tracing::{info, error, warn};
use diesel::prelude::*;
use diesel_async::RunQueryDsl;

use crate::web::{auth::routes::AppState, middleware::AuthCtx};
use crate::infra::db::diesel::models::notification::{
    DieselNotification, NewDieselNotification, create_notification, get_unread_count
};
use crate::infra::db::diesel::types::{NotificationType, NotificationPriority};
use crate::infra::db::diesel::schema::notifications;
use crate::infra::cache::CacheExt;
use crate::dom::ports::notification::{
    DomainNotification, DomainNotificationType, DomainNotificationPriority,
    NotificationRecipient
};
use crate::dom::values::UserId;
use super::dto::*;


/// GET /api/v1/notifications - List user notifications with pagination and filtering
pub async fn list_notifications_handler(
    auth_ctx: AuthCtx,
    State(app_state): State<AppState>,
    Query(query): Query<NotificationQuery>,
) -> Result<Json<NotificationListResponse>, StatusCode> {
    let user_id = auth_ctx.user_id;
    
    let per_page = query.per_page.unwrap_or(20) as i64;
    let page = query.page.unwrap_or(1);
    let offset = ((page - 1) * per_page as u64) as i64;
    
    // Get database connection
    let mut conn = match app_state.db_pool.get().await {
        Ok(conn) => conn,
        Err(e) => {
            error!("Failed to get database connection: {}", e);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };
    
    // Extract UUID from UserId wrapper
    let user_uuid = user_id.0;
    
    // Get notifications from database
    let notifications = match notifications::table
        .filter(notifications::user_id.eq(user_uuid))
        .order(notifications::created_at.desc())
        .offset(offset)
        .limit(per_page)
        .select(DieselNotification::as_select())
        .load(&mut conn)
        .await {
        Ok(notifications) => {
            // Skip offset and take per_page
            notifications.into_iter().skip(offset as usize).take(per_page as usize).collect::<Vec<_>>()
        },
        Err(e) => {
            error!("Failed to get user notifications: {}", e);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };
    
    // Get unread count
    let unread_count = match get_unread_count(&mut conn, user_uuid).await {
        Ok(count) => count as u64,
        Err(e) => {
            error!("Failed to get unread count: {}", e);
            0
        }
    };
    
    // Convert to response DTOs
    let notification_responses: Vec<NotificationResponse> = notifications
        .into_iter()
        .map(|notification| convert_to_notification_response(notification))
        .collect();
    
    let total_count = notification_responses.len() as u64;
    let total_pages = if per_page > 0 { (total_count + per_page as u64 - 1) / per_page as u64 } else { 1 };
    
    let pagination = PaginationResponse {
        page,
        per_page: per_page as u64,
        total_pages,
        has_next: page < total_pages,
        has_prev: page > 1,
    };

    let response = NotificationListResponse {
        notifications: notification_responses,
        pagination,
        unread_count,
        total_count,
        fetched_at: Utc::now(),
    };

    info!("Listed {} notifications for user {}", total_count, user_id);
    Ok(Json(response))
}

/// GET /api/v1/notifications/:id - Get single notification by ID
pub async fn get_notification_handler(
    auth_ctx: AuthCtx,
    State(app_state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<NotificationResponse>, StatusCode> {
    let user_id = auth_ctx.user_id;
    
    // Parse notification ID
    let notification_uuid = match Uuid::parse_str(&id) {
        Ok(uuid) => uuid,
        Err(_) => {
            error!("Invalid notification ID format: {}", id);
            return Err(StatusCode::BAD_REQUEST);
        }
    };
    
    // Get database connection
    let mut conn = match app_state.db_pool.get().await {
        Ok(conn) => conn,
        Err(e) => {
            error!("Failed to get database connection: {}", e);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };
    
    // Extract UUID from UserId wrapper
    let user_uuid = user_id.0;
    
    // Get notification from database
    use crate::infra::db::diesel::schema::notifications;
    let notification = match notifications::table
        .filter(notifications::id.eq(notification_uuid))
        .filter(notifications::user_id.eq(user_uuid))
        .select(DieselNotification::as_select())
        .first(&mut conn)
        .await
    {
        Ok(notification) => notification,
        Err(diesel::result::Error::NotFound) => {
            warn!("Notification {} not found for user {}", id, user_id);
            return Err(StatusCode::NOT_FOUND);
        },
        Err(e) => {
            error!("Failed to get notification: {}", e);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };
    
    let response = convert_to_notification_response(notification);
    info!("Retrieved notification {} for user {}", id, user_id);
    Ok(Json(response))
}

/// POST /api/v1/notifications/read/:id - Mark notification as read
pub async fn mark_notification_read_handler(
    auth_ctx: AuthCtx,
    State(app_state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<Value>, StatusCode> {
    let user_id = auth_ctx.user_id;
    
    // Parse notification ID
    let notification_uuid = match Uuid::parse_str(&id) {
        Ok(uuid) => uuid,
        Err(_) => {
            error!("Invalid notification ID format: {}", id);
            return Err(StatusCode::BAD_REQUEST);
        }
    };
    
    // Get database connection
    let mut conn = match app_state.db_pool.get().await {
        Ok(conn) => conn,
        Err(e) => {
            error!("Failed to get database connection: {}", e);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };
    
    // Extract UUID from UserId wrapper
    let user_uuid = user_id.0;
    
    // Verify notification belongs to user
    use crate::infra::db::diesel::schema::notifications;
    let notification_exists = match notifications::table
        .filter(notifications::id.eq(notification_uuid))
        .filter(notifications::user_id.eq(user_uuid))
        .select(notifications::id)
        .first::<Uuid>(&mut conn)
        .await
    {
        Ok(_) => true,
        Err(diesel::result::Error::NotFound) => {
            warn!("Notification {} not found for user {}", id, user_id);
            return Err(StatusCode::NOT_FOUND);
        },
        Err(e) => {
            error!("Failed to verify notification ownership: {}", e);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };
    
    if !notification_exists {
        return Err(StatusCode::NOT_FOUND);
    }
    
    // Mark as read
    match diesel::update(notifications::table.filter(notifications::id.eq(notification_uuid)))
        .set(notifications::is_read.eq(true))
        .execute(&mut conn)
        .await {
        Ok(_) => {
            info!("Marked notification {} as read for user {}", id, user_id);
            Ok(Json(json!({
                "notification_id": id,
                "user_id": user_id.to_string(),
                "marked_at": Utc::now(),
                "status": "read"
            })))
        },
        Err(e) => {
            error!("Failed to mark notification as read: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// POST /api/v1/notifications/read-all - Mark all notifications as read
pub async fn mark_all_notifications_read_handler(
    auth_ctx: AuthCtx,
    State(app_state): State<AppState>,
    Json(req): Json<MarkNotificationsReadRequest>,
) -> Result<Json<MarkNotificationsReadResponse>, StatusCode> {
    let user_id = auth_ctx.user_id;
    
    // Get database connection
    let mut conn = match app_state.db_pool.get().await {
        Ok(conn) => conn,
        Err(e) => {
            error!("Failed to get database connection: {}", e);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };
    
    // Extract UUID from UserId wrapper
    let user_uuid = user_id.0;
    
    let marked_count = if req.mark_all.unwrap_or(false) {
        // Mark all notifications as read
        use crate::infra::db::diesel::schema::notifications;
        match diesel::update(notifications::table
            .filter(notifications::user_id.eq(user_uuid))
            .filter(notifications::is_read.eq(false)))
            .set(notifications::is_read.eq(true))
            .execute(&mut conn)
            .await
        {
            Ok(count) => count as u64,
            Err(e) => {
                error!("Failed to mark all notifications as read: {}", e);
                return Err(StatusCode::INTERNAL_SERVER_ERROR);
            }
        }
    } else {
        // Mark specific notifications as read
        let mut marked = 0u64;
        for notification_id in &req.notification_ids {
            if let Ok(notification_uuid) = Uuid::parse_str(notification_id) {
                if let Ok(_) = diesel::update(notifications::table.filter(notifications::id.eq(notification_uuid)))
                    .set(notifications::is_read.eq(true))
                    .execute(&mut conn)
                    .await {
                    marked += 1;
                }
            }
        }
        marked
    };

    let response = MarkNotificationsReadResponse {
        user_id: user_id.to_string(),
        marked_count,
        notification_ids: req.notification_ids,
        mark_all: req.mark_all.unwrap_or(false),
        marked_at: Utc::now(),
    };

    info!("Marked {} notifications as read for user {}", marked_count, user_id);
    Ok(Json(response))
}

/// DELETE /api/v1/notifications/:id - Delete notification
pub async fn delete_notification_handler(
    auth_ctx: AuthCtx,
    State(app_state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<Value>, StatusCode> {
    let user_id = auth_ctx.user_id;
    
    // Parse notification ID
    let notification_uuid = match Uuid::parse_str(&id) {
        Ok(uuid) => uuid,
        Err(_) => {
            error!("Invalid notification ID format: {}", id);
            return Err(StatusCode::BAD_REQUEST);
        }
    };
    
    // Get database connection
    let mut conn = match app_state.db_pool.get().await {
        Ok(conn) => conn,
        Err(e) => {
            error!("Failed to get database connection: {}", e);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };
    
    // Extract UUID from UserId wrapper
    let user_uuid = user_id.0;
    
    // Delete notification
    use crate::infra::db::diesel::schema::notifications;
    let deleted_count = match diesel::delete(notifications::table
        .filter(notifications::id.eq(notification_uuid))
        .filter(notifications::user_id.eq(user_uuid)))
        .execute(&mut conn)
        .await
    {
        Ok(count) => count,
        Err(e) => {
            error!("Failed to delete notification: {}", e);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };
    
    if deleted_count == 0 {
        warn!("Notification {} not found for user {}", id, user_id);
        return Err(StatusCode::NOT_FOUND);
    }
    
    info!("Deleted notification {} for user {}", id, user_id);
    Ok(Json(json!({
        "notification_id": id,
        "user_id": user_id.to_string(),
        "deleted_at": Utc::now(),
        "status": "deleted"
    })))
}

/// GET /api/v1/notifications/unread-count - Get unread notification count
pub async fn get_unread_count_handler(
    auth_ctx: AuthCtx,
    State(app_state): State<AppState>,
) -> Result<Json<UnreadCountResponse>, StatusCode> {
    let user_id = auth_ctx.user_id;
    
    // Get database connection
    let mut conn = match app_state.db_pool.get().await {
        Ok(conn) => conn,
        Err(e) => {
            error!("Failed to get database connection: {}", e);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };
    
    // Extract UUID from UserId wrapper
    let user_uuid = user_id.0;
    
    // Get unread count
    let unread_count = match get_unread_count(&mut conn, user_uuid).await {
        Ok(count) => count as u64,
        Err(e) => {
            error!("Failed to get unread count: {}", e);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };
    
    // Get breakdown by category and priority
    use crate::infra::db::diesel::schema::notifications;
    let notifications = match notifications::table
        .filter(notifications::user_id.eq(user_uuid))
        .filter(notifications::is_read.eq(false))
        .select((notifications::notification_type, notifications::priority))
        .load::<(NotificationType, NotificationPriority)>(&mut conn)
        .await
    {
        Ok(notifications) => notifications,
        Err(e) => {
            error!("Failed to get notification breakdown: {}", e);
            vec![]
        }
    };
    
    // Group by category
    let mut category_counts = std::collections::HashMap::new();
    let mut priority_counts = std::collections::HashMap::new();
    
    for (notification_type, priority) in notifications {
        let type_str = format!("{:?}", notification_type).to_lowercase();
        let priority_str = format!("{:?}", priority).to_lowercase();
        
        *category_counts.entry(type_str).or_insert(0u64) += 1;
        *priority_counts.entry(priority_str).or_insert(0u64) += 1;
    }
    
    let by_category: Vec<CategoryCount> = category_counts
        .into_iter()
        .map(|(category, count)| CategoryCount { category, count })
        .collect();
        
    let by_priority: Vec<PriorityCount> = priority_counts
        .into_iter()
        .map(|(priority, count)| PriorityCount { priority, count })
        .collect();

    let response = UnreadCountResponse {
        user_id: user_id.to_string(),
        unread_count,
        by_category,
        by_priority,
        last_checked: Utc::now(),
    };

    info!("Retrieved unread count {} for user {}", unread_count, user_id);
    Ok(Json(response))
}

/// POST /api/v1/notifications/device-token - Register FCM device token
pub async fn register_device_token_handler(
    auth_ctx: AuthCtx,
    State(app_state): State<AppState>,
    Json(req): Json<DeviceTokenRequest>,
) -> Result<Json<DeviceTokenResponse>, StatusCode> {
    let user_id = auth_ctx.user_id;
    
    // Get database connection
    let mut conn = match app_state.db_pool.get().await {
        Ok(conn) => conn,
        Err(e) => {
            error!("Failed to get database connection: {}", e);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };
    
    // Extract UUID from UserId wrapper
    let user_uuid = user_id.0;
    
    // Parse device_type to DevicePlatform
    let device_platform = match req.device_type.to_lowercase().as_str() {
        "android" => crate::infra::db::diesel::types::DevicePlatform::Android,
        "ios" => crate::infra::db::diesel::types::DevicePlatform::Ios,
        "web" => crate::infra::db::diesel::types::DevicePlatform::Web,
        _ => {
            error!("Invalid device type: {}", req.device_type);
            return Err(StatusCode::BAD_REQUEST);
        }
    };
    
    // Create FCM token record
    use crate::infra::db::diesel::models::fcm::{NewDieselFcmToken, create_fcm_token};
    let new_token = NewDieselFcmToken::new(
        user_uuid,
        req.token.clone(),
        device_platform,
        req.device_id.clone().map(|id| serde_json::json!({"device_id": id, "app_version": req.app_version})),
        None, // user_agent not provided in this request
    );
    
    let created_token = match create_fcm_token(&mut conn, new_token).await {
        Ok(token) => token,
        Err(e) => {
            error!("Failed to register FCM token: {}", e);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    let response = DeviceTokenResponse {
        user_id: user_id.to_string(),
        token_id: created_token.id.to_string(),
        device_type: req.device_type.clone(),
        registered_at: created_token.created_at.unwrap_or(Utc::now()),
        status: "registered".to_string(),
    };

    info!("Registered FCM token for user {} on device {}", user_id, req.device_type);
    Ok(Json(response))
}

/// POST /api/v1/notifications/preferences - Update notification preferences
pub async fn update_preferences_handler(
    auth_ctx: AuthCtx,
    State(app_state): State<AppState>,
    Json(req): Json<NotificationPreferencesRequest>,
) -> Result<Json<NotificationPreferencesResponse>, StatusCode> {
    let user_id = auth_ctx.user_id;
    
    info!("Updating notification preferences for user {}", user_id);
    
    // For now, store in cache or database - this is a basic implementation
    // In a full implementation, you'd have a notification_preferences table
    let cache_key = format!("notification_preferences:{}", user_id);
    
    let preferences_data = serde_json::json!({
        "email_enabled": req.email_enabled.unwrap_or(true),
        "push_enabled": req.push_enabled.unwrap_or(true),
        "in_app_enabled": req.in_app_enabled.unwrap_or(true),
        "categories": req.categories.clone().unwrap_or_default(),
        "quiet_hours": req.quiet_hours,
        "updated_at": Utc::now()
    });
    
    // Store in cache
    if let Err(e) = app_state.cache.set(&cache_key, &preferences_data.to_string(), Some(86400)).await {
        warn!("Failed to cache notification preferences: {}", e);
    }

    let response = NotificationPreferencesResponse {
        user_id: user_id.to_string(),
        email_enabled: req.email_enabled.unwrap_or(true),
        push_enabled: req.push_enabled.unwrap_or(true),
        in_app_enabled: req.in_app_enabled.unwrap_or(true),
        categories: req.categories.unwrap_or_default(),
        quiet_hours: req.quiet_hours,
        updated_at: Utc::now(),
    };

    Ok(Json(response))
}

/// GET /api/v1/notifications/preferences - Get notification preferences
pub async fn get_preferences_handler(
    auth_ctx: AuthCtx,
    State(app_state): State<AppState>,
) -> Result<Json<NotificationPreferencesResponse>, StatusCode> {
    let user_id = auth_ctx.user_id;
    
    let cache_key = format!("notification_preferences:{}", user_id);
    
    // Try to get from cache first
    let (email_enabled, push_enabled, in_app_enabled, categories, quiet_hours) = 
        match app_state.cache.get::<String>(&cache_key).await {
            Ok(Some(cached_data)) => {
                if let Ok(data) = serde_json::from_str::<serde_json::Value>(&cached_data) {
                    (
                        data.get("email_enabled").and_then(|v| v.as_bool()).unwrap_or(true),
                        data.get("push_enabled").and_then(|v| v.as_bool()).unwrap_or(true),
                        data.get("in_app_enabled").and_then(|v| v.as_bool()).unwrap_or(true),
                        data.get("categories").and_then(|v| serde_json::from_value(v.clone()).ok()).unwrap_or_default(),
                        data.get("quiet_hours").and_then(|v| serde_json::from_value(v.clone()).ok())
                    )
                } else {
                    (true, true, true, vec![], None) // defaults
                }
            },
            _ => (true, true, true, vec![], None) // defaults
        };

    let response = NotificationPreferencesResponse {
        user_id: user_id.to_string(),
        email_enabled,
        push_enabled,
        in_app_enabled,
        categories,
        quiet_hours,
        updated_at: Utc::now(),
    };

    Ok(Json(response))
}

/// POST /api/v1/admin/notifications - Create admin notification (broadcast or targeted)
pub async fn create_admin_notification_handler(
    auth_ctx: AuthCtx,
    State(app_state): State<AppState>,
    Json(req): Json<CreateNotificationRequest>,
) -> Result<Json<CreateNotificationResponse>, StatusCode> {
    let admin_user_id = auth_ctx.user_id;
    
    info!("Admin {} creating notification: {}", admin_user_id, req.title);
    
    // Get database connection
    let mut conn = match app_state.db_pool.get().await {
        Ok(conn) => conn,
        Err(e) => {
            error!("Failed to get database connection: {}", e);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };
    
    let notification_id = Uuid::new_v4();
    
    match &req.target_users {
        Some(user_ids) => {
            // Create targeted notifications
            let mut created_count = 0u64;
            
            for user_id_str in user_ids {
                if let Ok(user_uuid) = Uuid::parse_str(user_id_str) {
                    let notification = NewDieselNotification::new(
                        user_uuid,
                        &req.title,
                        &req.message,
                        parse_notification_type(&req.notification_type),
                        parse_notification_priority(&req.priority),
                        req.expires_at,
                        req.metadata.clone(),
                    );
                    
                    if let Ok(created) = create_notification(&mut conn, notification).await {
                        created_count += 1;
                        
                        // Send via notification service if FCM push is requested
                        if req.channels.contains(&"push".to_string()) {
                            let domain_notification = DomainNotification {
                                id: Some(created.id.to_string()),
                                recipient: NotificationRecipient::User(UserId(user_uuid)),
                                notification_type: convert_to_domain_type(&req.notification_type),
                                priority: convert_to_domain_priority(&req.priority),
                                title: req.title.clone(),
                                message: req.message.clone(),
                                data: req.metadata.clone(),
                                scheduled_for: req.scheduled_for,
                                expires_at: req.expires_at,
                            };
                            
                            if let Err(e) = app_state.notification_service.send_notification(domain_notification).await {
                                error!("Failed to send FCM notification: {}", e);
                            }
                        }
                    }
                }
            }
            
            let response = CreateNotificationResponse {
                notification_id: notification_id.to_string(),
                target_count: created_count,
                created_at: Utc::now(),
                scheduled_for: req.scheduled_for,
                status: "created".to_string(),
            };
            
            info!("Created {} targeted notifications", created_count);
            Ok(Json(response))
        },
        None => {
            // Broadcast notification - get all users
            use crate::infra::db::diesel::schema::users;
            let all_users = match users::table
                .select(users::id)
                .load::<Uuid>(&mut conn)
                .await
            {
                Ok(users) => users,
                Err(e) => {
                    error!("Failed to get all users for broadcast: {}", e);
                    return Err(StatusCode::INTERNAL_SERVER_ERROR);
                }
            };
            
            let mut created_count = 0u64;
            
            for user_uuid in all_users {
                let notification = NewDieselNotification::new(
                    user_uuid,
                    &req.title,
                    &req.message,
                    parse_notification_type(&req.notification_type),
                    parse_notification_priority(&req.priority),
                    req.expires_at,
                    req.metadata.clone(),
                );
                
                if let Ok(_) = create_notification(&mut conn, notification).await {
                    created_count += 1;
                }
            }
            
            // Send broadcast FCM if requested
            if req.channels.contains(&"push".to_string()) {
                let domain_notification = DomainNotification {
                    id: Some(notification_id.to_string()),
                    recipient: NotificationRecipient::Broadcast,
                    notification_type: convert_to_domain_type(&req.notification_type),
                    priority: convert_to_domain_priority(&req.priority),
                    title: req.title.clone(),
                    message: req.message.clone(),
                    data: req.metadata.clone(),
                    scheduled_for: req.scheduled_for,
                    expires_at: req.expires_at,
                };
                
                if let Err(e) = app_state.notification_service.send_notification(domain_notification).await {
                    error!("Failed to send broadcast FCM notification: {}", e);
                }
            }
            
            let response = CreateNotificationResponse {
                notification_id: notification_id.to_string(),
                target_count: created_count,
                created_at: Utc::now(),
                scheduled_for: req.scheduled_for,
                status: "created".to_string(),
            };
            
            info!("Created broadcast notification to {} users", created_count);
            Ok(Json(response))
        }
    }
}

// ============================================================================
// HELPER FUNCTIONS
// ============================================================================

/// Convert DieselNotification to NotificationResponse DTO
fn convert_to_notification_response(notification: DieselNotification) -> NotificationResponse {
    NotificationResponse {
        id: notification.id.to_string(),
        user_id: notification.user_id.to_string(),
        title: notification.title,
        message: notification.message,
        notification_type: format!("{:?}", notification.notification_type).to_lowercase(),
        category: format!("{:?}", notification.notification_type).to_lowercase(),
        priority: format!("{:?}", notification.priority).to_lowercase(),
        status: if notification.is_read { "read".to_string() } else { "unread".to_string() },
        channel: "in_app".to_string(), // Default channel
        metadata: notification.metadata,
        created_at: notification.created_at,
        updated_at: None,
        read_at: None,
        expires_at: notification.expires_at,
        scheduled_for: None,
        sent_at: notification.delivered_at,
        delivery_status: notification.delivery_status,
        error_message: notification.fcm_failed_reason,
        retry_count: notification.delivery_attempts.unwrap_or(0),
    }
}

/// Parse string to NotificationType
fn parse_notification_type(type_str: &str) -> NotificationType {
    match type_str.to_lowercase().as_str() {
        "info" => NotificationType::Info,
        "warning" => NotificationType::Warning,
        "error" => NotificationType::Error,
        "success" => NotificationType::Success,
        "role_change" => NotificationType::RoleChange,
        _ => NotificationType::Info, // Default
    }
}

/// Parse string to NotificationPriority
fn parse_notification_priority(priority_str: &str) -> NotificationPriority {
    match priority_str.to_lowercase().as_str() {
        "low" => NotificationPriority::Low,
        "medium" | "normal" => NotificationPriority::Medium,
        "high" => NotificationPriority::High,
        "critical" => NotificationPriority::Critical,
        _ => NotificationPriority::Medium, // Default
    }
}

/// Convert string to DomainNotificationType
fn convert_to_domain_type(type_str: &str) -> DomainNotificationType {
    match type_str.to_lowercase().as_str() {
        "system" | "system_maintenance" => DomainNotificationType::SystemMaintenance,
        "security" | "security_alert" => DomainNotificationType::SecurityAlert,
        "account" | "account_update" => DomainNotificationType::AccountUpdate,
        "payment" | "payment_notification" => DomainNotificationType::PaymentNotification,
        "feature" | "feature_expiration" => DomainNotificationType::FeatureExpiration,
        "module" | "module_access_changed" => DomainNotificationType::ModuleAccessChanged,
        "quota" | "quota_warning" => DomainNotificationType::QuotaWarning,
        _ => DomainNotificationType::AccountUpdate, // Default
    }
}

/// Convert string to DomainNotificationPriority
fn convert_to_domain_priority(priority_str: &str) -> DomainNotificationPriority {
    match priority_str.to_lowercase().as_str() {
        "low" => DomainNotificationPriority::Low,
        "medium" | "normal" => DomainNotificationPriority::Normal,
        "high" => DomainNotificationPriority::High,
        "critical" => DomainNotificationPriority::Critical,
        _ => DomainNotificationPriority::Normal, // Default
    }
}