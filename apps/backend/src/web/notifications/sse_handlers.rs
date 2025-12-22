use axum::{
    extract::{Query, State, Path},
    response::sse::{Event, KeepAlive, Sse},
    response::IntoResponse,
};
use serde::{Deserialize, Serialize};
use std::time::Duration;
use chrono::{DateTime, Utc};
use futures::StreamExt;

use crate::{
    web::auth::AppState,
    core::errors::AppError,
};

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
    pub token: Option<String>, // Bearer token (for EventSource compatibility)
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct ScalarQuery {
    pub types: Option<String>,    // comma-separated notification types
    pub limit: Option<u32>,       // maximum number to return
    pub unread_only: Option<bool>, // filter for unread only
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct ScalarListQuery {
    pub limit: Option<u32>, // maximum number to return
    pub offset: Option<u32>, // number to skip
}

// ============================================================================
// SSE HANDLERS
// ============================================================================

/// SSE endpoint for real-time notifications via Redis pub/sub
/// Supports wallet-specific notifications + broadcast notifications
#[utoipa::path(
    get,
    path = "/api/v1/notifications/stream",
    tag = "notifications",
    responses(
        (status = 200, description = "Successfully established SSE connection"),
        (status = 401, description = "Unauthorized"),
        (status = 500, description = "Internal server error")
    ),
    params(
        ("types" = Option<String>, Query, description = "Comma-separated notification types to filter"),
        ("timeout" = Option<u64>, Query, description = "Connection timeout in seconds"),
        ("token" = Option<String>, Query, description = "Bearer token for authentication (for EventSource compatibility)")
    ),
    security(("bearerAuth" = []))
)]
pub async fn sse_notifications_handler(
    State(app_state): State<AppState>,
    Query(query): Query<SSEQuery>,
    request: axum::extract::Request,
) -> Result<impl IntoResponse, AppError> {
    // Extract wallet address from authentication (Header or Query)
    let mut wallet_address = "all".to_string();
    let mut token_to_validate = None;

    // 1. Check Authorization header
    if let Some(auth_header) = request.headers().get("authorization").and_then(|h| h.to_str().ok()) {
        if auth_header.starts_with("Bearer ") {
            token_to_validate = Some(auth_header[7..].to_string());
        }
    }

    // 2. Fallback to query param
    if token_to_validate.is_none() {
        if let Some(token) = query.token {
            token_to_validate = Some(token);
        }
    }

    // 3. Validate token if present
    if let Some(token) = token_to_validate {
        if let Some(token_service) = app_state.domain_container.get_token_service() {
            match token_service.validate_access_token(&token).await {
                Ok(claims) => {
                    wallet_address = claims.wallet_address.to_lowercase();
                    tracing::debug!("✅ SSE Auth: Validated wallet from token: {}", wallet_address);
                }
                Err(e) => {
                    tracing::warn!("❌ SSE Auth: Token validation failed: {}", e);
                    // Fallback to legacy extraction (only if needed/safe)
                    if let Some(legacy_wallet) = extract_wallet_from_token(Some(&token)) {
                        wallet_address = legacy_wallet;
                         tracing::warn!("⚠️ SSE Auth: Validated using legacy method (deprecated)");
                    }
                }
            }
        } else {
             tracing::error!("❌ SSE Auth: Token service not available");
             // Fallback
             if let Some(legacy_wallet) = extract_wallet_from_token(Some(&token)) {
                wallet_address = legacy_wallet;
            }
        }
    }

    tracing::info!(
        "🔌 SSE connection request: wallet={}, types={:?}",
        wallet_address,
        query.types
    );

    // Fetch queued (offline) notifications first
    let queued_notifications = if wallet_address != "all" {
        crate::web::notifications::fetch_queued_notifications(
            &app_state.db_pool,
            &wallet_address
        ).await.unwrap_or_default()
    } else {
        vec![]
    };

    tracing::info!(
        "📦 Found {} queued notifications for wallet: {}",
        queued_notifications.len(),
        wallet_address
    );

    // Subscribe to Redis pub/sub (if available)
    let redis_broadcaster = app_state.redis_broadcaster.clone();

    if redis_broadcaster.is_none() {
        tracing::warn!("⚠️ Redis not available - SSE will only send queued notifications");
    }

    let mut pubsub = match &redis_broadcaster {
        Some(broadcaster) => Some(broadcaster.subscribe_to_wallet(&wallet_address).await?),
        None => None,
    };

    // Parse notification type filters
    let allowed_types = parse_notification_types(query.types);

    // Clone for async stream
    let db_pool = app_state.db_pool.clone();
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

                        // Mark as delivered in background with error logging
                        let db = db_pool.clone();
                        let notif_id = notification.id.clone();
                        let notif_title = notification.title.clone();
                        tokio::spawn(async move {
                            match crate::web::notifications::mark_as_delivered(&db, &notif_id).await {
                                Ok(_) => {
                                    tracing::debug!("✅ Background task: Marked notification as delivered: id={}", notif_id);
                                }
                                Err(e) => {
                                    tracing::error!(
                                        "❌ Background task failed: Could not mark notification as delivered: id={}, title='{}', error={}",
                                        notif_id,
                                        notif_title,
                                        e
                                    );
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
                "📨 Received notification from Redis: wallet={}, id={}, title={}",
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

            tracing::info!("🔴 Redis pub/sub stream ended for wallet: {}", wallet_for_stream);
        } else {
            tracing::info!("⚠️ Redis not available - SSE connection will only show queued notifications");
        }
    };

    // Send initial ping event to establish connection
    let initial_ping = tokio_stream::once(Ok::<Event, axum::Error>(
        Event::default()
            .event("ping")
            .data("connected")
    ));

    // Combine streams
    let combined_stream = initial_ping.chain(redis_stream);

    // Keep-alive every 15 seconds
    let keep_alive_duration = Duration::from_secs(15);

    Ok(Sse::new(combined_stream)
        .keep_alive(KeepAlive::new().interval(keep_alive_duration)))
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

    // Get notification stats
    let stats = crate::web::notifications::get_notification_stats(&app_state.db_pool)
        .await
        .ok();

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
                _ => None,
            })
            .collect()
    })
}

/// Extract wallet address from Bearer token in request
/// Returns None if no auth present or token invalid
fn extract_wallet_from_request(request: &axum::extract::Request) -> Option<String> {
    // Get Authorization header
    let auth_header = request.headers()
        .get("authorization")
        .and_then(|h| h.to_str().ok())?;

    // Check Bearer format
    if !auth_header.starts_with("Bearer ") {
        return None;
    }

    let token = &auth_header[7..];
    extract_wallet_from_token(Some(token))
}

/// Extract wallet address from Bearer token string
/// Returns None if token is invalid or missing
fn extract_wallet_from_token(token: Option<&str>) -> Option<String> {
    use jsonwebtoken::{decode, DecodingKey, Validation, Algorithm};
    use serde::Deserialize;

    #[derive(Debug, Deserialize)]
    struct TokenClaims {
        #[serde(default)]
        wallet_address: String,
        #[serde(default)]
        sub: String,
    }

    let token = token?;

    // First, try legacy format: "web3_token_{wallet_address}"
    if token.starts_with("web3_token_") {
        let wallet = token.strip_prefix("web3_token_").unwrap_or("").to_string();
        if !wallet.is_empty() && wallet.len() >= 20 {
            tracing::debug!("✅ Extracted wallet from legacy token format: {}", wallet);
            return Some(wallet.to_lowercase());
        }
    }

    // Fall back to JWT decoding using environment variable
    let secret = std::env::var("JWT_SECRET")
        .unwrap_or_else(|_| "epsx-web3-bearer-token-secret-key".to_string());
    let decoding_key = DecodingKey::from_secret(secret.as_bytes());

    let mut validation = Validation::new(Algorithm::HS256);
    validation.validate_exp = true;

    // Try to decode token
    match decode::<TokenClaims>(token, &decoding_key, &validation) {
        Ok(token_data) => {
            let wallet = if !token_data.claims.wallet_address.is_empty() {
                token_data.claims.wallet_address
            } else {
                token_data.claims.sub
            };

            if !wallet.is_empty() && wallet != "anonymous" {
                tracing::debug!("✅ Extracted wallet from JWT token: {}", wallet);
                Some(wallet.to_lowercase())
            } else {
                None
            }
        }
        Err(e) => {
            tracing::warn!("❌ Failed to decode token as JWT: {:?}", e);
            None
        }
    }
}
