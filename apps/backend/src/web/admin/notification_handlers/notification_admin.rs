// Admin notification handlers
use axum::{
    extract::{State, Path, Query},
    http::HeaderMap,
    response::IntoResponse,
    Json,
};
use chrono::Utc;
use diesel::prelude::*;
use diesel_async::RunQueryDsl;

use crate::{
    infrastructure::services::audit_service::{AuditCtx, AuditEntry},
    web::auth::AppState,
    web::notifications::SSENotification,
};
use epsx_contracts::errors::{AppError, ErrorKind};
use super::notification_types::*;
use super::super::notification_query_helper::NotificationQueryFilter;
use super::super::wallet_notification_repository::WalletNotificationRepository;

// ============================================================================
// ADMIN HANDLERS
// ============================================================================

/// Send notification to specific user, plan, or broadcast
#[utoipa::path(
    post,
    path = "/api/admin/notifications/send",
    tag = "admin-notifications",
    request_body = SendNotificationRequest,
    responses(
        (status = 200, description = "Notification sent successfully"),
        (status = 400, description = "Invalid request"),
        (status = 401, description = "Unauthorized"),
        (status = 500, description = "Internal server error")
    ),
    security(("bearerAuth" = []))
)]
pub async fn send_notification_handler(
    State(app_state): State<AppState>,
    axum::Extension(user_ctx): axum::Extension<crate::web::middleware::bearer_middleware::OpenIDUserContext>,
    headers: HeaderMap,
    Json(request): Json<SendNotificationRequest>,
) -> Result<impl IntoResponse, AppError> {
    // Validate request
    if request.title.trim().is_empty() {
        return Err(AppError::new(
            ErrorKind::ValidationError,
            "Notification title cannot be empty".to_string(),
        ));
    }

    if request.message.trim().is_empty() {
        return Err(AppError::new(
            ErrorKind::ValidationError,
            "Notification message cannot be empty".to_string(),
        ));
    }

    // Determine target wallet address (convert to lowercase for consistency)
    let wallet_addresses = if request.broadcast.unwrap_or(false) {
        vec!["all".to_string()]
    } else if let Some(ref addr) = request.recipient_wallet_address {
        vec![addr.to_lowercase()]
    } else if let Some(ref plan) = request.recipient_plan {
        // Fetch wallet addresses for plan from database
        #[derive(QueryableByName)]
        struct PlanMemberRow {
            #[diesel(sql_type = diesel::sql_types::Text)]
            wallet_address: String,
        }

        let mut conn = app_state.db_pool.get().await
            .map_err(|e| AppError::new(ErrorKind::DatabaseError, format!("Failed to get database connection: {}", e)))?;

        let plan_members = diesel::sql_query(
            r#"
            SELECT wallet_address
            FROM wallet_plan_assignments wga
            INNER JOIN plans pg ON wga.plan_id = pg.id
            WHERE pg.slug = $1 AND wga.is_active = true
            "#
        )
        .bind::<diesel::sql_types::Text, _>(&plan)
        .load::<PlanMemberRow>(&mut conn)
        .await
        .map_err(|e| AppError::new(ErrorKind::DatabaseError, format!("Failed to fetch plan members: {}", e)))?
        .into_iter()
        .map(|r| r.wallet_address)
        .collect::<Vec<String>>();

        if plan_members.is_empty() {
            return Err(AppError::new(
                ErrorKind::ValidationError,
                format!("No active members found in plan: {}", plan),
            ));
        }

        plan_members
    } else {
        return Err(AppError::new(
            ErrorKind::ValidationError,
            "Must specify recipient_wallet_address, recipient_plan, or broadcast=true".to_string(),
        ));
    };

    // Convert notification type and priority to string for database
    let notif_type = format!("{:?}", request.notification_type).to_lowercase();
    let notif_priority = format!("{:?}", request.priority).to_lowercase();

    // Use repository for database operations
    // Use repository for database operations - notifications table is in separate DB
    let notifications_pool = if let Ok(p) = crate::infrastructure::database::get_notifications_pool().await {
        std::sync::Arc::new(p)
    } else {
        app_state.db_pool.clone()
    };
    let repo = WalletNotificationRepository::new(notifications_pool);

    // Track total subscribers across all recipients
    let mut total_subscriber_count = 0;
    let mut notification_ids = Vec::new();

    // Handle broadcast case separately
    let is_broadcast = wallet_addresses.len() == 1 && wallet_addresses[0] == "all";

    if is_broadcast {
        // Create single notification for broadcast
        let notification_id = uuid::Uuid::new_v4();
        notification_ids.push(notification_id.to_string());

        // Create SSE notification
        let sse_notification = SSENotification {
            id: notification_id.to_string(),
            wallet_address: "all".to_string(),
            notification_type: request.notification_type.clone(),
            title: request.title.clone(),
            message: request.message.clone(),
            data: request.data.clone(),
            priority: request.priority.clone(),
            timestamp: Utc::now(),
            expires_at: request.expires_at,
        };

        // Persist to database
        repo.create(
            notification_id,
            "all",
            &notif_type,
            &request.title,
            &request.message,
            request.data.clone(),
            &notif_priority,
            request.expires_at,
            request.action_url.clone(),
            request.image_url.clone(),
        ).await?;

        // Publish via Redis pub/sub (if available)
        if let Some(redis_broadcaster) = &app_state.redis_broadcaster {
            total_subscriber_count = redis_broadcaster.publish_to_all(&sse_notification).await?;
        } else {
            tracing::warn!("Redis not available - notification saved to database but not broadcast in real-time");
        }

        // Update delivery attempt
        repo.update_delivery_attempt(notification_id).await?;
    } else {
        // Send individual notifications to each wallet address
        for wallet_address in &wallet_addresses {
            let notification_id = uuid::Uuid::new_v4();
            notification_ids.push(notification_id.to_string());

            // Create SSE notification
            let sse_notification = SSENotification {
                id: notification_id.to_string(),
                wallet_address: wallet_address.clone(),
                notification_type: request.notification_type.clone(),
                title: request.title.clone(),
                message: request.message.clone(),
                data: request.data.clone(),
                priority: request.priority.clone(),
                timestamp: Utc::now(),
                expires_at: request.expires_at,
            };

            // Persist to database
            repo.create(
                notification_id,
                wallet_address,
                &notif_type,
                &request.title,
                &request.message,
                request.data.clone(),
                &notif_priority,
                request.expires_at,
                request.action_url.clone(),
                request.image_url.clone(),
            ).await?;

            // Publish via Redis pub/sub (if available)
            if let Some(redis_broadcaster) = &app_state.redis_broadcaster {
                let count = redis_broadcaster.publish_to_wallet(wallet_address, &sse_notification).await?;
                total_subscriber_count += count;
            } else {
                tracing::warn!("Redis not available for {} - notification saved to database but not broadcast in real-time", wallet_address);
            }

            // Update delivery attempt
            repo.update_delivery_attempt(notification_id).await?;
        }
    }

    // Audit logging
    let ctx = AuditCtx::from_wallet(&user_ctx.wallet_address, &headers);
    app_state.audit.log(ctx, AuditEntry::new("notification", "create", "notification")
        .id(&notification_ids.join(","))
        .after(serde_json::json!({
            "title": request.title,
            "message": request.message,
            "notification_type": request.notification_type,
            "priority": request.priority,
            "broadcast": request.broadcast,
            "recipient_wallet_address": request.recipient_wallet_address,
            "recipient_plan": request.recipient_plan,
            "recipients_count": total_subscriber_count,
        })));

    // Build response
    let delivery_message = if app_state.redis_broadcaster.is_some() {
        if is_broadcast {
            "Broadcast notification sent successfully via Redis".to_string()
        } else {
            format!("Notifications sent to {} recipients via Redis", wallet_addresses.len())
        }
    } else {
        "Notification(s) saved to database (Redis unavailable - no real-time broadcast)".to_string()
    };

    let response = SendNotificationResponse {
        success: true,
        data: SendNotificationData {
            notification_id: notification_ids.join(","), // Return comma-separated IDs for multiple notifications
            recipients_count: total_subscriber_count,
            scheduled: request.schedule_at.is_some(),
            delivery_status: if request.schedule_at.is_some() {
                "scheduled".to_string()
            } else if total_subscriber_count > 0 {
                "sent".to_string()
            } else {
                "queued".to_string() // No active subscribers, will deliver when they connect
            },
        },
        message: delivery_message,
        api_version: "v1".to_string(),
    };

    Ok(Json(response))
}

/// Get all notifications (admin view with filters)
#[utoipa::path(
    get,
    path = "/api/admin/notifications",
    tag = "admin-notifications",
    params(
        ("page" = Option<u32>, Query, description = "Page number"),
        ("limit" = Option<u32>, Query, description = "Items per page"),
        ("type" = Option<String>, Query, description = "Filter by notification type"),
        ("priority" = Option<String>, Query, description = "Filter by priority"),
        ("status" = Option<String>, Query, description = "Filter by status"),
        ("wallet_address" = Option<String>, Query, description = "Filter by wallet address")
    ),
    responses(
        (status = 200, description = "Notifications retrieved successfully"),
        (status = 401, description = "Unauthorized"),
        (status = 500, description = "Internal server error")
    ),
    security(("bearerAuth" = []))
)]
pub async fn get_all_notifications_handler(
    State(app_state): State<AppState>,
    Query(filters): Query<NotificationFilters>,
) -> Result<impl IntoResponse, AppError> {
    let pg = crate::web::pagination::Pagination::standard(filters.page, filters.limit);

    // Build filter helper
    let mut filter = NotificationQueryFilter::new();
    if let Some(wallet) = filters.wallet_address {
        filter = filter.wallet(wallet);
    }
    if let Some(notif_type) = filters.notification_type {
        filter = filter.notification_type(notif_type);
    }
    if let Some(priority) = filters.priority {
        filter = filter.priority(priority);
    }
    if let Some(status) = filters.status {
        filter = filter.status(status);
    }

    // Use repository for database operations - notifications table is in separate DB
    let notifications_pool = if let Ok(p) = crate::infrastructure::database::get_notifications_pool().await {
        std::sync::Arc::new(p)
    } else {
        app_state.db_pool.clone()
    };
    let repo = WalletNotificationRepository::new(notifications_pool);

    // Fetch notifications
    let records = repo.find_with_filters(&filter, pg.limit as i64, pg.offset).await?;

    let notifications: Vec<NotificationDto> = records.into_iter().map(|r| NotificationDto {
        id: r.id.to_string(),
        wallet_address: r.wallet_address,
        notification_type: r.notification_type,
        title: r.title,
        message: r.message,
        data: r.data,
        priority: r.priority,
        timestamp: r.timestamp,
        expires_at: r.expires_at,
        read_at: r.read_at,
        clicked_at: r.clicked_at,
        delivered_at: r.delivered_at,
        action_url: r.action_url,
        image_url: r.image_url,
    }).collect();

    // Get total count
    let total_count = repo.count_with_filters(&filter).await?;

    // Get unread count
    let unread_count = repo.count_unread_with_filters(&filter).await?;

    let total_pages = pg.total_pages(total_count as u64);

    let response = NotificationsResponse {
        success: true,
        data: NotificationsData {
            notifications,
            total_count: total_count as usize,
            unread_count: unread_count as usize,
            page: pg.page,
            limit: pg.limit,
            total_pages,
        },
        api_version: "v1".to_string(),
        access_level: "admin".to_string(),
    };

    Ok(Json(response))
}

/// Get notification statistics
#[utoipa::path(
    get,
    path = "/api/admin/notifications/stats",
    tag = "admin-notifications",
    responses(
        (status = 200, description = "Statistics retrieved successfully"),
        (status = 401, description = "Unauthorized"),
        (status = 500, description = "Internal server error")
    ),
    security(("bearerAuth" = []))
)]
pub async fn get_notification_stats_handler(
    State(app_state): State<AppState>,
) -> Result<impl IntoResponse, AppError> {
    // Get notifications database connection
    let notifications_pool = if let Ok(p) = crate::infrastructure::database::get_notifications_pool().await {
        std::sync::Arc::new(p)
    } else {
        app_state.db_pool.clone()
    };
    let mut conn = notifications_pool.get().await
        .map_err(|e| AppError::new(ErrorKind::DatabaseError, format!("Failed to get database connection: {}", e)))?;

    #[derive(QueryableByName)]
    struct CountRow {
        #[diesel(sql_type = diesel::sql_types::BigInt)]
        count: i64,
    }

    // Get total notifications count (exclude soft-deleted)
    let total_count: i64 = diesel::sql_query("SELECT COUNT(*) as count FROM wallet_notifications WHERE status != 'deleted'")
        .get_result::<CountRow>(&mut conn)
        .await
        .map_err(|e| AppError::new(ErrorKind::DatabaseError, format!("Failed to count notifications: {}", e)))?
        .count;

    // Get notifications sent today (exclude soft-deleted)
    let today_count: i64 = diesel::sql_query(
        "SELECT COUNT(*) as count FROM wallet_notifications WHERE created_at >= CURRENT_DATE AND status != 'deleted'"
    )
        .get_result::<CountRow>(&mut conn)
        .await
        .map_err(|e| AppError::new(ErrorKind::DatabaseError, format!("Failed to count today's notifications: {}", e)))?
        .count;

    // Get notifications sent this week (exclude soft-deleted)
    let week_count: i64 = diesel::sql_query(
        "SELECT COUNT(*) as count FROM wallet_notifications WHERE created_at >= CURRENT_DATE - INTERVAL '7 days' AND status != 'deleted'"
    )
        .get_result::<CountRow>(&mut conn)
        .await
        .map_err(|e| AppError::new(ErrorKind::DatabaseError, format!("Failed to count week's notifications: {}", e)))?
        .count;

    // Get notifications sent this month (exclude soft-deleted)
    let month_count: i64 = diesel::sql_query(
        "SELECT COUNT(*) as count FROM wallet_notifications WHERE created_at >= CURRENT_DATE - INTERVAL '30 days' AND status != 'deleted'"
    )
        .get_result::<CountRow>(&mut conn)
        .await
        .map_err(|e| AppError::new(ErrorKind::DatabaseError, format!("Failed to count month's notifications: {}", e)))?
        .count;

    #[derive(QueryableByName)]
    struct TypeCountRow {
        #[diesel(sql_type = diesel::sql_types::Text)]
        notification_type: String,
        #[diesel(sql_type = diesel::sql_types::BigInt)]
        count: i64,
    }

    // Get count by type (exclude soft-deleted)
    let type_counts = diesel::sql_query(
        "SELECT notification_type, COUNT(*) as count FROM wallet_notifications WHERE status != 'deleted' GROUP BY notification_type"
    )
        .load::<TypeCountRow>(&mut conn)
        .await
        .map_err(|e| AppError::new(ErrorKind::DatabaseError, format!("Failed to get type counts: {}", e)))?;

    let mut by_type = serde_json::Map::new();
    for row in type_counts {
        by_type.insert(row.notification_type, serde_json::json!(row.count));
    }

    #[derive(QueryableByName)]
    struct PriorityCountRow {
        #[diesel(sql_type = diesel::sql_types::Text)]
        priority: String,
        #[diesel(sql_type = diesel::sql_types::BigInt)]
        count: i64,
    }

    // Get count by priority (exclude soft-deleted)
    let priority_counts = diesel::sql_query(
        "SELECT priority, COUNT(*) as count FROM wallet_notifications WHERE status != 'deleted' GROUP BY priority"
    )
        .load::<PriorityCountRow>(&mut conn)
        .await
        .map_err(|e| AppError::new(ErrorKind::DatabaseError, format!("Failed to get priority counts: {}", e)))?;

    let mut by_priority = serde_json::Map::new();
    for row in priority_counts {
        by_priority.insert(row.priority, serde_json::json!(row.count));
    }

    // Calculate delivery rate (all active notifications are delivered)
    let delivery_rate = if total_count > 0 { 1.0 } else { 0.0 };

    // Calculate read rate (exclude soft-deleted)
    let read_count: i64 = diesel::sql_query(
        "SELECT COUNT(*) as count FROM wallet_notifications WHERE status = 'read'"
    )
        .get_result::<CountRow>(&mut conn)
        .await
        .map_err(|e| AppError::new(ErrorKind::DatabaseError, format!("Failed to count read notifications: {}", e)))?
        .count;

    let read_rate = if total_count > 0 {
        (read_count as f64) / (total_count as f64)
    } else {
        0.0
    };

    // Calculate click rate (Not tracked in new schema, defaulting to 0)
    let clicked_count: i64 = 0;
/*
    let clicked_count: i64 = diesel::sql_query(
        "SELECT COUNT(*) as count FROM wallet_notifications WHERE clicked_at IS NOT NULL AND deleted_at IS NULL"
    )
        .get_result::<CountRow>(&mut conn)
        .await
        .map_err(|e| AppError::new(ErrorKind::DatabaseError, format!("Failed to count clicked notifications: {}", e)))?
        .count;
*/

    let click_rate = if total_count > 0 {
        (clicked_count as f64) / (total_count as f64)
    } else {
        0.0
    };

    #[derive(QueryableByName)]
    struct RecentActivityRow {
        #[diesel(sql_type = diesel::sql_types::Timestamptz)]
        hour: chrono::DateTime<Utc>,
        #[diesel(sql_type = diesel::sql_types::BigInt)]
        count: i64,
    }

    // Get recent activity (last 24 hours, planed by hour, exclude soft-deleted)
    let recent_activity_records = diesel::sql_query(
        r#"
        SELECT
            DATE_TRUNC('hour', created_at) as hour,
            COUNT(*) as count
        FROM wallet_notifications
        WHERE created_at >= NOW() - INTERVAL '24 hours'
          AND status != 'deleted'
        GROUP BY hour
        ORDER BY hour DESC
        LIMIT 10
        "#
    )
        .load::<RecentActivityRow>(&mut conn)
        .await
        .map_err(|e| AppError::new(ErrorKind::DatabaseError, format!("Failed to get recent activity: {}", e)))?;

    let recent_activity: Vec<RecentActivity> = recent_activity_records
        .into_iter()
        .map(|row| RecentActivity {
            timestamp: row.hour,
            action: "notification_sent".to_string(),
            count: row.count as usize,
        })
        .collect();

    let stats = NotificationStats {
        total_notifications: total_count as usize,
        sent_today: today_count as usize,
        sent_this_week: week_count as usize,
        sent_this_month: month_count as usize,
        delivery_rate,
        read_rate,
        click_rate,
        by_type: serde_json::Value::Object(by_type),
        by_priority: serde_json::Value::Object(by_priority),
        recent_activity,
    };

    let response = NotificationStatsResponse {
        success: true,
        data: stats,
        api_version: "v1".to_string(),
        access_level: "admin".to_string(),
    };

    Ok(Json(response))
}

/// Delete notification (admin only - hard delete)
#[utoipa::path(
    delete,
    path = "/api/admin/notifications/{id}",
    tag = "admin-notifications",
    params(
        ("id" = String, Path, description = "Notification ID")
    ),
    responses(
        (status = 200, description = "Notification deleted successfully"),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Notification not found"),
        (status = 500, description = "Internal server error")
    ),
    security(("bearerAuth" = []))
)]
pub async fn delete_admin_notification_handler(
    State(app_state): State<AppState>,
    axum::Extension(user_ctx): axum::Extension<crate::web::middleware::bearer_middleware::OpenIDUserContext>,
    headers: HeaderMap,
    Path(notification_id): Path<String>,
) -> Result<impl IntoResponse, AppError> {
    let notif_uuid = uuid::Uuid::parse_str(&notification_id)
        .map_err(|e| AppError::new(ErrorKind::ValidationError, format!("Invalid notification ID: {}", e)))?;

    // Get notifications database connection
    let notifications_pool = if let Ok(p) = crate::infrastructure::database::get_notifications_pool().await {
        std::sync::Arc::new(p)
    } else {
        app_state.db_pool.clone()
    };
    let mut conn = notifications_pool.get().await
        .map_err(|e| AppError::new(ErrorKind::DatabaseError, format!("Failed to get database connection: {}", e)))?;

    // Hard delete for admin
    let rows_affected = diesel::sql_query(
        "DELETE FROM wallet_notifications WHERE id = $1"
    )
    .bind::<diesel::sql_types::Uuid, _>(notif_uuid)
    .execute(&mut conn)
    .await
    .map_err(|e| AppError::new(ErrorKind::DatabaseError, format!("Failed to delete notification: {}", e)))?;

    if rows_affected == 0 {
        return Err(AppError::new(
            ErrorKind::AggregateNotFound,
            "Notification not found".to_string(),
        ));
    }

    // Audit logging
    let ctx = AuditCtx::from_wallet(&user_ctx.wallet_address, &headers);
    app_state.audit.log(ctx, AuditEntry::new("notification", "delete", "notification")
        .id(&notification_id)
        .after(serde_json::json!({
            "deleted": true,
        })));

    Ok(Json(serde_json::json!({
        "success": true,
        "message": "Notification deleted successfully",
        "notification_id": notification_id
    })))
}
