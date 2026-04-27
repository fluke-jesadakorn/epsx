use axum::{
    extract::{Query, State},
    response::sse::{Event, KeepAlive, Sse},
    response::IntoResponse,
};
use chrono::{DateTime, Utc};
use futures::StreamExt;
use serde::{Deserialize, Serialize};
use std::time::Duration;

use crate::{core::errors::AppError, web::auth::AppState};

// ============================================================================
// SSE NOTIFICATION TYPES
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
pub struct SSENotification {
    pub id: String,
    pub wallet_address: String,
    pub notification_type: NotificationType,
    pub title: String,
    pub message: String,
    pub data: Option<serde_json::Value>,
    pub priority: NotificationPriority,
    pub timestamp: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, utoipa::ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum NotificationType {
    System,
    Security,
    Permission,
    WalletManagement,
    Wallet,
    Payment,
    General,
    Announcement,
    Advertisement,
    Chat,
}

#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "lowercase")]
pub enum NotificationPriority {
    Low,
    Normal,
    High,
    Critical,
}

#[derive(Debug, Deserialize)]
pub struct SSEQuery {
    pub types: Option<String>, // comma-separated notification types
    pub timeout: Option<u64>,  // seconds
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct ScalarQuery {
    pub types: Option<String>,     // comma-separated notification types
    pub limit: Option<u32>,        // maximum number to return
    pub unread_only: Option<bool>, // filter for unread only
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct ScalarListQuery {
    pub limit: Option<u32>,  // maximum number to return
    pub offset: Option<u32>, // number to skip
}

// ============================================================================
// SSE HANDLERS
// ============================================================================

/// SSE endpoint for real-time notifications via Redis pub/sub
/// Supports wallet-specific notifications + broadcast notifications
#[utoipa::path(
    get,
    path = "/api/notifications/stream",
    tag = "notifications",
    responses(
        (status = 200, description = "Successfully established SSE connection"),
        (status = 401, description = "Unauthorized"),
        (status = 500, description = "Internal server error")
    ),
    params(
        ("types" = Option<String>, Query, description = "Comma-separated notification types to filter"),
        ("timeout" = Option<u64>, Query, description = "Connection timeout in seconds")
    ),
    security(("bearerAuth" = []))
)]
pub async fn sse_notifications_handler(
    State(app_state): State<AppState>,
    Query(query): Query<SSEQuery>,
    request: axum::extract::Request,
) -> Result<impl IntoResponse, AppError> {
    let token = crate::web::middleware::bearer_middleware::extract_bearer_token_from_headers(
        request.headers(),
    )
    .ok_or_else(|| AppError::unauthorized("Authentication token is required"))?;

    let token_service = app_state
        .domain_container
        .get_token_service()
        .ok_or_else(|| AppError::internal_server_error("Authentication service unavailable"))?;

    let claims = token_service
        .validate_access_token(&token)
        .await
        .map_err(|e| {
            tracing::warn!(
                "SSE Auth: Token validation failed, rejecting SSE connection: {}",
                e
            );
            AppError::unauthorized("Invalid or expired authentication token")
        })?;
    let wallet_address = claims.wallet_address.to_lowercase();
    tracing::debug!("SSE Auth: Validated wallet from token: {}", wallet_address);

    tracing::info!(
        "SSE connection request: wallet={}, types={:?}",
        wallet_address,
        query.types
    );

    // Fetch queued (offline) notifications first from NOTIFICATIONS database
    use crate::infrastructure::database::get_notifications_pool;
    let queued_notifications = if wallet_address != "all" {
        match get_notifications_pool().await {
            Ok(notifications_pool) => crate::web::notifications::fetch_queued_notifications(
                notifications_pool,
                &wallet_address,
            )
            .await
            .unwrap_or_default(),
            Err(e) => {
                tracing::error!("Failed to get notifications database pool: {}", e);
                vec![]
            }
        }
    } else {
        vec![]
    };

    tracing::info!(
        "Found {} queued notifications for wallet: {}",
        queued_notifications.len(),
        wallet_address
    );

    // Subscribe to Redis pub/sub (if available)
    let redis_broadcaster = app_state.redis_broadcaster.clone();

    if redis_broadcaster.is_none() {
        tracing::warn!("Redis not available - SSE will only send queued notifications");
    }

    let mut pubsub = match &redis_broadcaster {
        Some(broadcaster) => Some(broadcaster.subscribe_to_wallet(&wallet_address).await?),
        None => None,
    };

    // Parse notification type filters
    let allowed_types = parse_notification_types(query.types);

    // Clone for async stream - need to get notifications pool inside stream
    let wallet_for_stream = wallet_address.clone();

    // Create stream from Redis pub/sub messages
    let redis_stream = async_stream::stream! {
        // First, send queued notifications
        for notification in queued_notifications {
            if should_send_notification(&notification, &allowed_types) {
                match serde_json::to_string(&notification) {
                    Ok(data) => {
                        yield Ok::<Event, axum::Error>(
                            Event::default()
                                .event("notification")
                                .id(notification.id.clone())
                                .data(data)
                        );

                        // Mark as delivered in background with notifications pool
                        let notif_id = notification.id.clone();
                        let notif_title = notification.title.clone();
                        tokio::spawn(async move {
                            match crate::infrastructure::database::get_notifications_pool().await {
                                Ok(pool) => {
                                    match crate::web::notifications::mark_as_delivered(pool, &notif_id).await {
                                        Ok(_) => {
                                            tracing::debug!("Background task: Marked notification as delivered: id={}", notif_id);
                                        }
                                        Err(e) => {
                                            tracing::error!(
                                                "Background task failed: Could not mark notification as delivered: id={}, title='{}', error={}",
                                                notif_id,
                                                notif_title,
                                                e
                                            );
                                        }
                                    }
                                }
                                Err(e) => {
                                    tracing::error!("Failed to get notifications pool in background task: {}", e);
                                }
                            }
                        });
                    }
                    Err(e) => {
                        tracing::error!("Failed to serialize queued notification: {}", e);
                    }
                }
            }
        }

        // Then stream real-time notifications from Redis (if available)
        if let Some(ref mut ps) = pubsub {
            let mut message_stream = ps.on_message();
            while let Some(msg) = message_stream.next().await {
            let payload: String = match msg.get_payload() {
                Ok(p) => p,
                Err(e) => {
                    tracing::error!("Failed to get Redis message payload: {}", e);
                    continue;
                }
            };

            let notification: SSENotification = match serde_json::from_str(&payload) {
                Ok(n) => n,
                Err(e) => {
                    tracing::error!("Failed to deserialize notification from Redis: {}", e);
                    continue;
                }
            };

            tracing::info!(
                "Received notification from Redis: wallet={}, id={}, title={}",
                wallet_for_stream,
                notification.id,
                notification.title
            );

            if should_send_notification(&notification, &allowed_types) {
                match serde_json::to_string(&notification) {
                    Ok(data) => {
                        yield Ok::<Event, axum::Error>(
                            Event::default()
                                .event("notification")
                                .id(notification.id.clone())
                                .data(data)
                        );
                    }
                    Err(e) => {
                        tracing::error!("Failed to serialize notification: {}", e);
                    }
                }
            }
            }

            tracing::info!("Redis pub/sub stream ended for wallet: {}", wallet_for_stream);
        } else {
            tracing::info!("Redis not available - SSE connection will only show queued notifications");
        }
    };

    // Send initial ping event to establish connection
    let initial_ping = tokio_stream::once(Ok::<Event, axum::Error>(
        Event::default().event("ping").data("connected"),
    ));

    // Combine streams
    let combined_stream = initial_ping.chain(redis_stream);

    // Keep-alive every 15 seconds
    let keep_alive_duration = Duration::from_secs(15);

    Ok(Sse::new(combined_stream).keep_alive(KeepAlive::new().interval(keep_alive_duration)))
}

// ============================================================================
// HEALTH CHECK
// ============================================================================

/// SSE health check endpoint
pub async fn sse_health_handler(
    State(app_state): State<AppState>,
) -> Result<impl IntoResponse, AppError> {
    // Check Redis health (if available)
    let redis_healthy = if let Some(redis_pool) = &app_state.redis_pool {
        redis_pool.health_check().await
    } else {
        false
    };

    // Get notification stats from NOTIFICATIONS database
    let stats = match crate::infrastructure::database::get_notifications_pool().await {
        Ok(pool) => crate::web::notifications::get_notification_stats(pool)
            .await
            .ok(),
        Err(_) => None,
    };

    Ok(axum::response::Json(serde_json::json!({
        "status": if redis_healthy { "healthy" } else { "degraded" },
        "redis_healthy": redis_healthy,
        "timestamp": Utc::now(),
        "stats": stats,
    })))
}

// ============================================================================
// HELPER FUNCTIONS
// ============================================================================

/// Helper function to filter notifications
fn should_send_notification(
    notification: &SSENotification,
    allowed_types: &Option<Vec<NotificationType>>,
) -> bool {
    // Check type filter
    if let Some(types) = allowed_types {
        if !types.contains(&notification.notification_type) {
            return false;
        }
    }

    // Check expiry
    if let Some(expires_at) = notification.expires_at {
        if Utc::now() > expires_at {
            return false;
        }
    }

    true
}

/// Parse notification types from query string
fn parse_notification_types(types_str: Option<String>) -> Option<Vec<NotificationType>> {
    types_str.map(|s| {
        s.split(',')
            .filter_map(|t| match t.trim() {
                "system" => Some(NotificationType::System),
                "security" => Some(NotificationType::Security),
                "permission" => Some(NotificationType::Permission),
                "wallet_management" => Some(NotificationType::WalletManagement),
                "wallet" => Some(NotificationType::Wallet),
                "payment" => Some(NotificationType::Payment),
                "general" => Some(NotificationType::General),
                "announcement" => Some(NotificationType::Announcement),
                "advertisement" => Some(NotificationType::Advertisement),
                "chat" => Some(NotificationType::Chat),
                _ => None,
            })
            .collect()
    })
}
