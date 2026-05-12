use axum::{
    extract::{Path, Query, State},
    response::sse::{Event, KeepAlive, Sse},
    response::IntoResponse,
    Extension, Json,
};
use futures::StreamExt;
use serde::Deserialize;
use std::time::Duration;
use tracing::{error, info};
use uuid::Uuid;

use crate::infrastructure::models::chat::*;
use crate::infrastructure::repositories::ChatRepository;
use crate::web::{auth::AppState, middleware::OpenIDUserContext, responses::UnifiedApiResponse};

// ============================================================================
// QUERY PARAMS
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct AdminConversationQuery {
    pub status: Option<String>,
    pub topic_id: Option<Uuid>,
    pub agent: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct AdminChatSSEQuery {
    pub token: Option<String>,
}

// ============================================================================
// ADMIN CHAT HANDLERS
// ============================================================================

/// List all conversations with filters
pub async fn admin_list_conversations(
    State(app_state): State<AppState>,
    Query(query): Query<AdminConversationQuery>,
) -> Result<Json<UnifiedApiResponse<Vec<ChatConversationDb>>>, Json<UnifiedApiResponse<()>>> {
    match ChatRepository::list_all_conversations(
        &app_state.db_pool,
        query.status.as_deref(),
        query.topic_id,
        query.agent.as_deref(),
    )
    .await
    {
        Ok(convs) => Ok(Json(UnifiedApiResponse::success(convs))),
        Err(e) => {
            error!("Admin: Failed to list conversations: {}", e);
            Err(Json(UnifiedApiResponse::error(
                500,
                "Failed to load conversations",
                &e,
            )))
        }
    }
}

/// Get conversation detail (admin can see any)
pub async fn admin_get_conversation(
    State(app_state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<UnifiedApiResponse<ChatConversationDb>>, Json<UnifiedApiResponse<()>>> {
    match ChatRepository::get_conversation(&app_state.db_pool, id).await {
        Ok(Some(conv)) => Ok(Json(UnifiedApiResponse::success(conv))),
        Ok(None) => Err(Json(UnifiedApiResponse::error(
            404,
            "Not found",
            "Conversation not found",
        ))),
        Err(e) => {
            error!("Admin: Failed to get conversation: {}", e);
            Err(Json(UnifiedApiResponse::error(500, "Database error", &e)))
        }
    }
}

/// List messages (admin can see any conversation)
pub async fn admin_list_messages(
    State(app_state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<UnifiedApiResponse<Vec<ChatMessageDb>>>, Json<UnifiedApiResponse<()>>> {
    match ChatRepository::list_messages(&app_state.db_pool, id).await {
        Ok(msgs) => Ok(Json(UnifiedApiResponse::success(msgs))),
        Err(e) => {
            error!("Admin: Failed to list messages: {}", e);
            Err(Json(UnifiedApiResponse::error(
                500,
                "Failed to load messages",
                &e,
            )))
        }
    }
}

/// Agent sends reply
pub async fn admin_send_reply(
    State(app_state): State<AppState>,
    Extension(ctx): Extension<OpenIDUserContext>,
    Path(id): Path<Uuid>,
    Json(body): Json<SendMessageRequest>,
) -> Result<Json<UnifiedApiResponse<ChatMessageDb>>, Json<UnifiedApiResponse<()>>> {
    if body.content.trim().is_empty() {
        return Err(Json(UnifiedApiResponse::error(
            400,
            "Invalid request",
            "Message content required",
        )));
    }

    let conv = match ChatRepository::get_conversation(&app_state.db_pool, id).await {
        Ok(Some(conv)) => conv,
        Ok(None) => {
            return Err(Json(UnifiedApiResponse::error(
                404,
                "Not found",
                "Conversation not found",
            )))
        }
        Err(e) => return Err(Json(UnifiedApiResponse::error(500, "Database error", &e))),
    };

    let sanitized_content =
        crate::infrastructure::security::sanitize_chat_content(body.content.trim());

    match ChatRepository::send_message(
        &app_state.db_pool,
        id,
        "agent",
        Some(&ctx.wallet_address),
        &sanitized_content,
    )
    .await
    {
        Ok(msg) => {
            // Publish to user's channel
            if let Some(broadcaster) = &app_state.redis_broadcaster {
                let event = serde_json::json!({
                    "type": "new_message",
                    "conversation_id": id,
                    "message": msg,
                });
                let payload = serde_json::to_string(&event).unwrap_or_default();
                let _ = broadcaster
                    .publish_to_channel(&format!("chat:wallet:{}", conv.wallet_address), &payload)
                    .await;
            }

            // Notify user about new support message
            let notif_wallet = conv.wallet_address.clone();
            let notif_conv_id = id;
            let notif_content = body.content.chars().take(100).collect::<String>();
            let notif_state = app_state.clone();
            tokio::spawn(async move {
                use crate::infrastructure::services::NotificationService;
                use crate::web::notifications::{NotificationPriority, NotificationType};
                let _ = NotificationService::send(
                    &notif_state,
                    &notif_wallet,
                    NotificationType::Chat,
                    NotificationPriority::Normal,
                    "New Support Message",
                    &notif_content,
                    Some(serde_json::json!({ "conversation_id": notif_conv_id })),
                    Some(format!("/chat/{}", notif_conv_id)),
                )
                .await;
            });

            info!(
                "Agent {} replied to conversation {}",
                ctx.wallet_address, id
            );
            Ok(Json(UnifiedApiResponse::success(msg)))
        }
        Err(e) => {
            error!("Admin: Failed to send reply: {}", e);
            Err(Json(UnifiedApiResponse::error(
                500,
                "Failed to send reply",
                &e,
            )))
        }
    }
}

/// Assign agent to conversation
pub async fn admin_assign_agent(
    State(app_state): State<AppState>,
    Extension(ctx): Extension<OpenIDUserContext>,
    Path(id): Path<Uuid>,
    Json(body): Json<AssignAgentRequest>,
) -> Result<Json<UnifiedApiResponse<ChatConversationDb>>, Json<UnifiedApiResponse<()>>> {
    let agent = body.agent_address.as_deref().unwrap_or(&ctx.wallet_address);

    match ChatRepository::assign_agent(&app_state.db_pool, id, Some(agent)).await {
        Ok(conv) => {
            // Publish agent_assigned to relevant channels
            if let Some(broadcaster) = &app_state.redis_broadcaster {
                let event = serde_json::json!({
                    "type": "agent_assigned",
                    "conversation_id": id,
                    "assigned_agent": agent,
                    "conversation": conv,
                });
                let payload = serde_json::to_string(&event).unwrap_or_default();
                let _ = broadcaster.publish_to_channel("chat:new", &payload).await;
                let _ = broadcaster
                    .publish_to_channel(&format!("chat:agent:{}", agent), &payload)
                    .await;
            }
            info!("Agent {} assigned to conversation {}", agent, id);
            Ok(Json(UnifiedApiResponse::success(conv)))
        }
        Err(e) => {
            error!("Admin: Failed to assign agent: {}", e);
            Err(Json(UnifiedApiResponse::error(
                500,
                "Failed to assign agent",
                &e,
            )))
        }
    }
}

/// Update conversation status
pub async fn admin_update_status(
    State(app_state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(body): Json<UpdateStatusRequest>,
) -> Result<Json<UnifiedApiResponse<ChatConversationDb>>, Json<UnifiedApiResponse<()>>> {
    let valid = ["open", "in_progress", "resolved", "closed"];
    if !valid.contains(&body.status.as_str()) {
        return Err(Json(UnifiedApiResponse::error(
            400,
            "Invalid status",
            "Must be open, in_progress, resolved, or closed",
        )));
    }

    // Fetch conversation first to get wallet_address for user notification
    let existing = match ChatRepository::get_conversation(&app_state.db_pool, id).await {
        Ok(Some(c)) => c,
        Ok(None) => {
            return Err(Json(UnifiedApiResponse::error(
                404,
                "Not found",
                "Conversation not found",
            )))
        }
        Err(e) => return Err(Json(UnifiedApiResponse::error(500, "Database error", &e))),
    };

    match ChatRepository::update_status(&app_state.db_pool, id, &body.status).await {
        Ok(conv) => {
            // Publish status_changed to user + admin channels
            if let Some(broadcaster) = &app_state.redis_broadcaster {
                let event = serde_json::json!({
                    "type": "status_changed",
                    "conversation_id": id,
                    "status": body.status,
                    "conversation": conv,
                });
                let payload = serde_json::to_string(&event).unwrap_or_default();
                let _ = broadcaster
                    .publish_to_channel(
                        &format!("chat:wallet:{}", existing.wallet_address),
                        &payload,
                    )
                    .await;
                let _ = broadcaster.publish_to_channel("chat:new", &payload).await;
            }
            Ok(Json(UnifiedApiResponse::success(conv)))
        }
        Err(e) => {
            error!("Admin: Failed to update status: {}", e);
            Err(Json(UnifiedApiResponse::error(
                500,
                "Failed to update status",
                &e,
            )))
        }
    }
}

/// Mark messages as read by agent
pub async fn admin_mark_read(
    State(app_state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<UnifiedApiResponse<()>>, Json<UnifiedApiResponse<()>>> {
    let conv = match ChatRepository::get_conversation(&app_state.db_pool, id).await {
        Ok(Some(c)) => c,
        Ok(None) => {
            return Err(Json(UnifiedApiResponse::error(
                404,
                "Not found",
                "Conversation not found",
            )))
        }
        Err(e) => return Err(Json(UnifiedApiResponse::error(500, "Database error", &e))),
    };

    match ChatRepository::mark_read_by_agent(&app_state.db_pool, id).await {
        Ok(()) => {
            // Notify user that agent has read their messages
            if let Some(broadcaster) = &app_state.redis_broadcaster {
                let event = serde_json::json!({
                    "type": "messages_read",
                    "conversation_id": id,
                    "reader": "agent",
                });
                let payload = serde_json::to_string(&event).unwrap_or_default();
                let _ = broadcaster
                    .publish_to_channel(&format!("chat:wallet:{}", conv.wallet_address), &payload)
                    .await;
            }
            Ok(Json(UnifiedApiResponse::success(())))
        }
        Err(e) => {
            error!("Admin: Failed to mark read: {}", e);
            Err(Json(UnifiedApiResponse::error(
                500,
                "Failed to mark read",
                &e,
            )))
        }
    }
}

/// Get chat stats
pub async fn admin_get_stats(
    State(app_state): State<AppState>,
) -> Result<Json<UnifiedApiResponse<ChatStatsResponse>>, Json<UnifiedApiResponse<()>>> {
    match ChatRepository::get_stats(&app_state.db_pool).await {
        Ok(stats) => Ok(Json(UnifiedApiResponse::success(stats))),
        Err(e) => {
            error!("Admin: Failed to get stats: {}", e);
            Err(Json(UnifiedApiResponse::error(
                500,
                "Failed to get stats",
                &e,
            )))
        }
    }
}

/// List topics (admin)
pub async fn admin_list_topics(
    State(app_state): State<AppState>,
) -> Result<Json<UnifiedApiResponse<Vec<ChatTopicDb>>>, Json<UnifiedApiResponse<()>>> {
    match ChatRepository::list_topics(&app_state.db_pool).await {
        Ok(topics) => Ok(Json(UnifiedApiResponse::success(topics))),
        Err(e) => {
            error!("Admin: Failed to list topics: {}", e);
            Err(Json(UnifiedApiResponse::error(
                500,
                "Failed to load topics",
                &e,
            )))
        }
    }
}

/// Admin chat overview: stats + conversations + topics in one call
/// GET /admin/chat/overview
pub async fn admin_chat_overview_handler(
    State(app_state): State<AppState>,
) -> Result<Json<UnifiedApiResponse<serde_json::Value>>, Json<UnifiedApiResponse<()>>> {
    info!("Admin: Getting chat overview");

    let (stats, conversations, topics) = tokio::join!(
        ChatRepository::get_stats(&app_state.db_pool),
        ChatRepository::list_all_conversations(&app_state.db_pool, None, None, None),
        ChatRepository::list_topics(&app_state.db_pool),
    );

    let response = serde_json::json!({
        "stats": stats.map(|s| serde_json::to_value(s).unwrap_or_else(|_| serde_json::json!({}))).unwrap_or_else(|_| serde_json::json!({})),
        "conversations": conversations.unwrap_or_default(),
        "topics": topics.unwrap_or_default(),
    });

    Ok(Json(UnifiedApiResponse::success(response)))
}

/// SSE stream for admin - listens to new conversations + assigned conversations
pub async fn admin_chat_stream(
    State(app_state): State<AppState>,
    Query(_query): Query<AdminChatSSEQuery>,
    request: axum::extract::Request,
) -> Result<impl IntoResponse, crate::core::errors::AppError> {
    let token = crate::web::middleware::bearer_middleware::extract_bearer_token_from_headers(
        request.headers(),
    )
    .ok_or_else(|| {
        crate::core::errors::AppError::new(
            crate::core::errors::ErrorKind::AuthenticationError,
            "Authentication required for admin chat stream",
        )
    })?;

    let token_service = app_state
        .domain_container
        .get_token_service()
        .ok_or_else(|| {
            crate::core::errors::AppError::internal_server_error(
                "Authentication service unavailable",
            )
        })?;

    let claims = token_service
        .validate_access_token(&token)
        .await
        .map_err(|_| {
            crate::core::errors::AppError::new(
                crate::core::errors::ErrorKind::AuthenticationError,
                "Invalid or expired authentication token",
            )
        })?;

    let permissions: Vec<String> = claims
        .scope
        .split_whitespace()
        .filter(|s| *s != "openid" && *s != "profile")
        .map(|s| s.to_string())
        .collect();
    if !crate::core::permissions::is_admin(&permissions) {
        return Err(crate::core::errors::AppError::new(
            crate::core::errors::ErrorKind::AuthorizationError,
            "Admin access required",
        ));
    }
    let wallet_address = claims.wallet_address.to_lowercase();

    info!("Admin Chat SSE connection: wallet={}", wallet_address);

    let redis_broadcaster = app_state.redis_broadcaster.clone();

    // Subscribe to new conversation channel + agent-specific channel
    let mut pubsub = match &redis_broadcaster {
        Some(broadcaster) => {
            let mut ps = broadcaster.subscribe_to_channel("chat:new").await?;
            if wallet_address != "all" {
                let agent_channel = format!("chat:agent:{}", wallet_address);
                ps.subscribe(&agent_channel).await.map_err(|e| {
                    crate::core::errors::AppError::new(
                        crate::core::errors::ErrorKind::InternalError,
                        format!("Redis subscribe failed: {}", e),
                    )
                })?;
            }
            Some(ps)
        }
        None => None,
    };

    let stream = async_stream::stream! {
        yield Ok::<Event, axum::Error>(Event::default().event("ping").data("connected"));

        if let Some(ref mut ps) = pubsub {
            let mut msg_stream = ps.on_message();
            while let Some(msg) = msg_stream.next().await {
                let payload: String = match msg.get_payload() {
                    Ok(p) => p,
                    Err(_) => continue,
                };
                yield Ok::<Event, axum::Error>(
                    Event::default().event("chat_event").data(payload)
                );
            }
        }
    };

    Ok(Sse::new(stream).keep_alive(KeepAlive::new().interval(Duration::from_secs(15))))
}
