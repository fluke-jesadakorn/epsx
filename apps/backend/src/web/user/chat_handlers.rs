use axum::{
    extract::{Path, Query, State},
    response::sse::{Event, KeepAlive, Sse},
    response::IntoResponse,
    Extension, Json,
};
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
pub struct ChatSSEQuery {
    pub token: Option<String>,
}

// ============================================================================
// USER CHAT HANDLERS
// ============================================================================

/// List active topics
pub async fn list_topics(
    State(app_state): State<AppState>,
) -> Result<Json<UnifiedApiResponse<Vec<ChatTopicDb>>>, Json<UnifiedApiResponse<()>>> {
    match ChatRepository::list_topics(&app_state.db_pool).await {
        Ok(topics) => Ok(Json(UnifiedApiResponse::success(topics))),
        Err(e) => {
            error!("Failed to list topics: {}", e);
            Err(Json(UnifiedApiResponse::error(
                500,
                "Failed to load topics",
                &e,
            )))
        }
    }
}

/// Create new conversation with first message
pub async fn create_conversation(
    State(app_state): State<AppState>,
    Extension(ctx): Extension<OpenIDUserContext>,
    Json(body): Json<CreateConversationRequest>,
) -> Result<Json<UnifiedApiResponse<ChatConversationDb>>, Json<UnifiedApiResponse<()>>> {
    if body.subject.trim().is_empty() || body.message.trim().is_empty() {
        return Err(Json(UnifiedApiResponse::error(
            400,
            "Invalid request",
            "Subject and message required",
        )));
    }

    let sanitized_subject =
        crate::infrastructure::security::sanitize_chat_content(body.subject.trim());
    let sanitized_message =
        crate::infrastructure::security::sanitize_chat_content(body.message.trim());

    match ChatRepository::create_conversation(
        &app_state.db_pool,
        body.topic_id,
        &ctx.wallet_address,
        &sanitized_subject,
        &sanitized_message,
    )
    .await
    {
        Ok(conv) => {
            // Publish to Redis for admin notification
            if let Some(pubsub) = &app_state.pubsub {
                let event = serde_json::json!({
                    "type": "new_conversation",
                    "conversation_id": conv.id,
                    "wallet_address": ctx.wallet_address,
                    "subject": conv.subject,
                });
                let payload = serde_json::to_vec(&event).unwrap_or_default();
                let _ = pubsub.publish("chat:new", &payload).await;
            }

            info!(
                "Created conversation {} for {}",
                conv.id, ctx.wallet_address
            );
            Ok(Json(UnifiedApiResponse::success(conv)))
        }
        Err(e) => {
            error!("Failed to create conversation: {}", e);
            Err(Json(UnifiedApiResponse::error(
                500,
                "Failed to create conversation",
                &e,
            )))
        }
    }
}

/// List user's conversations
pub async fn list_conversations(
    State(app_state): State<AppState>,
    Extension(ctx): Extension<OpenIDUserContext>,
) -> Result<Json<UnifiedApiResponse<Vec<ChatConversationDb>>>, Json<UnifiedApiResponse<()>>> {
    match ChatRepository::list_user_conversations(&app_state.db_pool, &ctx.wallet_address).await {
        Ok(convs) => Ok(Json(UnifiedApiResponse::success(convs))),
        Err(e) => {
            error!("Failed to list conversations: {}", e);
            Err(Json(UnifiedApiResponse::error(
                500,
                "Failed to load conversations",
                &e,
            )))
        }
    }
}

/// Get conversation detail
pub async fn get_conversation(
    State(app_state): State<AppState>,
    Extension(ctx): Extension<OpenIDUserContext>,
    Path(id): Path<Uuid>,
) -> Result<Json<UnifiedApiResponse<ChatConversationDb>>, Json<UnifiedApiResponse<()>>> {
    match ChatRepository::get_conversation(&app_state.db_pool, id).await {
        Ok(Some(conv)) if conv.wallet_address == ctx.wallet_address => {
            Ok(Json(UnifiedApiResponse::success(conv)))
        }
        Ok(Some(_)) => Err(Json(UnifiedApiResponse::error(
            403,
            "Forbidden",
            "Not your conversation",
        ))),
        Ok(None) => Err(Json(UnifiedApiResponse::error(
            404,
            "Not found",
            "Conversation not found",
        ))),
        Err(e) => {
            error!("Failed to get conversation: {}", e);
            Err(Json(UnifiedApiResponse::error(500, "Database error", &e)))
        }
    }
}

/// List messages in a conversation
pub async fn list_messages(
    State(app_state): State<AppState>,
    Extension(ctx): Extension<OpenIDUserContext>,
    Path(id): Path<Uuid>,
) -> Result<Json<UnifiedApiResponse<Vec<ChatMessageDb>>>, Json<UnifiedApiResponse<()>>> {
    // Verify ownership
    match ChatRepository::get_conversation(&app_state.db_pool, id).await {
        Ok(Some(conv)) if conv.wallet_address == ctx.wallet_address => {}
        Ok(Some(_)) => {
            return Err(Json(UnifiedApiResponse::error(
                403,
                "Forbidden",
                "Not your conversation",
            )))
        }
        Ok(None) => {
            return Err(Json(UnifiedApiResponse::error(
                404,
                "Not found",
                "Conversation not found",
            )))
        }
        Err(e) => return Err(Json(UnifiedApiResponse::error(500, "Database error", &e))),
    }

    match ChatRepository::list_messages(&app_state.db_pool, id).await {
        Ok(msgs) => Ok(Json(UnifiedApiResponse::success(msgs))),
        Err(e) => {
            error!("Failed to list messages: {}", e);
            Err(Json(UnifiedApiResponse::error(
                500,
                "Failed to load messages",
                &e,
            )))
        }
    }
}

/// Send a message
pub async fn send_message(
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

    // Verify ownership
    let conv = match ChatRepository::get_conversation(&app_state.db_pool, id).await {
        Ok(Some(conv)) if conv.wallet_address == ctx.wallet_address => conv,
        Ok(Some(_)) => {
            return Err(Json(UnifiedApiResponse::error(
                403,
                "Forbidden",
                "Not your conversation",
            )))
        }
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
        "user",
        Some(&ctx.wallet_address),
        &sanitized_content,
    )
    .await
    {
        Ok(msg) => {
            // Publish to Redis
            if let Some(pubsub) = &app_state.pubsub {
                let event = serde_json::json!({
                    "type": "new_message",
                    "conversation_id": id,
                    "message": msg,
                });
                let payload = serde_json::to_vec(&event).unwrap_or_default();
                // Notify assigned agent
                if let Some(agent) = &conv.assigned_agent {
                    let channel = format!("chat:agent:{}", agent);
                    let _ = pubsub.publish(&channel, &payload).await;
                }
                // Notify admin channel
                let _ = pubsub.publish("chat:new", &payload).await;
            }

            // Send notification to assigned agent (or broadcast if unassigned)
            let notif_state = app_state.clone();
            let notif_content = body.content.chars().take(100).collect::<String>();
            let notif_conv_id = id;
            let notif_subject = conv.subject.clone();
            let notif_agent = conv.assigned_agent.clone();
            tokio::spawn(async move {
                // Wave 10 / R3: route through the NotificationPort.
                use epsx_contracts::notification_port::{
                    BroadcastNotificationRequest, SendNotificationRequest,
                };
                if let Some(port) = notif_state.notification_port.as_ref() {
                    if let Some(agent) = notif_agent {
                        let _ = port
                            .send(SendNotificationRequest {
                                recipient_wallet_address: agent,
                                notification_type: "chat".to_string(),
                                priority: "normal".to_string(),
                                title: format!("New message: {}", notif_subject),
                                message: notif_content.clone(),
                                data: Some(serde_json::json!({ "conversation_id": notif_conv_id })),
                                action_url: Some("/chat".to_string()),
                            })
                            .await;
                    } else {
                        let _ = port
                            .broadcast(BroadcastNotificationRequest {
                                notification_type: "chat".to_string(),
                                priority: "normal".to_string(),
                                title: format!("New message: {}", notif_subject),
                                message: notif_content.clone(),
                                data: Some(serde_json::json!({ "conversation_id": notif_conv_id })),
                            })
                            .await;
                    }
                } else {
                    tracing::warn!(
                        "notification_port not wired in AppState; chat notification \
                         for conversation={} dropped",
                        notif_conv_id
                    );
                }
            });

            Ok(Json(UnifiedApiResponse::success(msg)))
        }
        Err(e) => {
            error!("Failed to send message: {}", e);
            Err(Json(UnifiedApiResponse::error(
                500,
                "Failed to send message",
                &e,
            )))
        }
    }
}

/// Update conversation status (resolve/close)
pub async fn update_status(
    State(app_state): State<AppState>,
    Extension(ctx): Extension<OpenIDUserContext>,
    Path(id): Path<Uuid>,
    Json(body): Json<UpdateStatusRequest>,
) -> Result<Json<UnifiedApiResponse<ChatConversationDb>>, Json<UnifiedApiResponse<()>>> {
    // User can only resolve or close
    if body.status != "resolved" && body.status != "closed" {
        return Err(Json(UnifiedApiResponse::error(
            400,
            "Invalid status",
            "Users can only resolve or close",
        )));
    }

    // Verify ownership
    match ChatRepository::get_conversation(&app_state.db_pool, id).await {
        Ok(Some(conv)) if conv.wallet_address == ctx.wallet_address => {}
        Ok(Some(_)) => {
            return Err(Json(UnifiedApiResponse::error(
                403,
                "Forbidden",
                "Not your conversation",
            )))
        }
        Ok(None) => {
            return Err(Json(UnifiedApiResponse::error(
                404,
                "Not found",
                "Conversation not found",
            )))
        }
        Err(e) => return Err(Json(UnifiedApiResponse::error(500, "Database error", &e))),
    }

    match ChatRepository::update_status(&app_state.db_pool, id, &body.status).await {
        Ok(conv) => {
            // Publish status_changed to admin channels
            if let Some(pubsub) = &app_state.pubsub {
                let event = serde_json::json!({
                    "type": "status_changed",
                    "conversation_id": id,
                    "status": body.status,
                    "conversation": conv,
                });
                let payload = serde_json::to_vec(&event).unwrap_or_default();
                let _ = pubsub.publish("chat:new", &payload).await;
                if let Some(agent) = &conv.assigned_agent {
                    let channel = format!("chat:agent:{}", agent);
                    let _ = pubsub.publish(&channel, &payload).await;
                }
            }
            Ok(Json(UnifiedApiResponse::success(conv)))
        }
        Err(e) => {
            error!("Failed to update status: {}", e);
            Err(Json(UnifiedApiResponse::error(
                500,
                "Failed to update status",
                &e,
            )))
        }
    }
}

/// Mark messages as read
pub async fn mark_read(
    State(app_state): State<AppState>,
    Extension(ctx): Extension<OpenIDUserContext>,
    Path(id): Path<Uuid>,
) -> Result<Json<UnifiedApiResponse<()>>, Json<UnifiedApiResponse<()>>> {
    // Verify ownership
    let conv = match ChatRepository::get_conversation(&app_state.db_pool, id).await {
        Ok(Some(conv)) if conv.wallet_address == ctx.wallet_address => conv,
        Ok(Some(_)) => {
            return Err(Json(UnifiedApiResponse::error(
                403,
                "Forbidden",
                "Not your conversation",
            )))
        }
        Ok(None) => {
            return Err(Json(UnifiedApiResponse::error(
                404,
                "Not found",
                "Conversation not found",
            )))
        }
        Err(e) => return Err(Json(UnifiedApiResponse::error(500, "Database error", &e))),
    };

    match ChatRepository::mark_read_by_user(&app_state.db_pool, id).await {
        Ok(()) => {
            // Notify assigned agent that user has read messages
            if let Some(pubsub) = &app_state.pubsub {
                let event = serde_json::json!({
                    "type": "messages_read",
                    "conversation_id": id,
                    "reader": "user",
                });
                let payload = serde_json::to_vec(&event).unwrap_or_default();
                let _ = pubsub.publish("chat:new", &payload).await;
                if let Some(agent) = &conv.assigned_agent {
                    let channel = format!("chat:agent:{}", agent);
                    let _ = pubsub.publish(&channel, &payload).await;
                }
            }
            Ok(Json(UnifiedApiResponse::success(())))
        }
        Err(e) => {
            error!("Failed to mark read: {}", e);
            Err(Json(UnifiedApiResponse::error(
                500,
                "Failed to mark read",
                &e,
            )))
        }
    }
}

/// Get unread count
pub async fn get_unread(
    State(app_state): State<AppState>,
    Extension(ctx): Extension<OpenIDUserContext>,
) -> Result<Json<UnifiedApiResponse<serde_json::Value>>, Json<UnifiedApiResponse<()>>> {
    match ChatRepository::get_unread_count(&app_state.db_pool, &ctx.wallet_address).await {
        Ok(count) => Ok(Json(UnifiedApiResponse::success(
            serde_json::json!({ "count": count }),
        ))),
        Err(e) => {
            error!("Failed to get unread count: {}", e);
            Ok(Json(UnifiedApiResponse::success(
                serde_json::json!({ "count": 0 }),
            )))
        }
    }
}

/// Chat inbox: returns topics + conversations in one call
/// GET /chat/inbox
pub async fn chat_inbox_handler(
    State(app_state): State<AppState>,
    Extension(ctx): Extension<OpenIDUserContext>,
) -> Result<Json<UnifiedApiResponse<serde_json::Value>>, Json<UnifiedApiResponse<()>>> {
    let wallet = ctx.wallet_address.to_lowercase();
    info!("User: Getting chat inbox for {}", wallet);

    let (topics, conversations) = tokio::join!(
        ChatRepository::list_topics(&app_state.db_pool),
        ChatRepository::list_user_conversations(&app_state.db_pool, &wallet),
    );

    let response = serde_json::json!({
        "topics": topics.unwrap_or_default(),
        "conversations": conversations.unwrap_or_default(),
    });

    Ok(Json(UnifiedApiResponse::success(response)))
}

/// Full conversation with messages
/// GET /chat/conversations/{id}/full
pub async fn get_full_conversation_handler(
    State(app_state): State<AppState>,
    Extension(ctx): Extension<OpenIDUserContext>,
    Path(id): Path<Uuid>,
) -> Result<Json<UnifiedApiResponse<serde_json::Value>>, Json<UnifiedApiResponse<()>>> {
    let wallet = ctx.wallet_address.to_lowercase();
    info!("User: Getting full conversation {} for {}", id, wallet);

    let (conversation, messages) = tokio::join!(
        ChatRepository::get_conversation(&app_state.db_pool, id),
        ChatRepository::list_messages(&app_state.db_pool, id),
    );

    let conv = match conversation {
        Ok(Some(c)) if c.wallet_address == ctx.wallet_address => c,
        Ok(Some(_)) => {
            return Err(Json(UnifiedApiResponse::error(
                403,
                "Forbidden",
                "Not your conversation",
            )))
        }
        Ok(None) => {
            return Err(Json(UnifiedApiResponse::error(
                404,
                "Not found",
                "Conversation not found",
            )))
        }
        Err(e) => {
            error!("Failed to get conversation: {}", e);
            return Err(Json(UnifiedApiResponse::error(500, "Database error", &e)));
        }
    };

    let msgs = messages.unwrap_or_default();

    let response = serde_json::json!({
        "conversation": conv,
        "messages": msgs,
    });

    Ok(Json(UnifiedApiResponse::success(response)))
}

/// SSE stream for real-time chat messages
pub async fn chat_stream(
    State(app_state): State<AppState>,
    Query(_query): Query<ChatSSEQuery>,
    request: axum::extract::Request,
) -> Result<impl IntoResponse, epsx_contracts::errors::AppError> {
    let token = crate::web::middleware::bearer_middleware::extract_bearer_token_from_headers(
        request.headers(),
    )
    .ok_or_else(|| {
        epsx_contracts::errors::AppError::unauthorized("Authentication required for chat stream")
    })?;

    let token_service = app_state
        .domain_container
        .get_token_service()
        .ok_or_else(|| {
            epsx_contracts::errors::AppError::internal_server_error(
                "Authentication service unavailable",
            )
        })?;

    let claims = token_service
        .validate_access_token(&token)
        .await
        .map_err(|_| {
            epsx_contracts::errors::AppError::unauthorized("Invalid or expired authentication token")
        })?;
    let wallet_address = claims.wallet_address.to_lowercase();

    info!("Chat SSE connection: wallet={}", wallet_address);

    let pubsub = app_state.pubsub.clone();
    let channel = format!("chat:wallet:{}", wallet_address);

    let mut message_stream = match &pubsub {
        Some(port) => Some(port.subscribe(&[channel.as_str()])?),
        None => None,
    };

    let stream = async_stream::stream! {
        yield Ok::<Event, axum::Error>(Event::default().event("ping").data("connected"));

        if let Some(ref mut stream) = message_stream {
            while let Some(payload) = stream.next_message().await {
                match String::from_utf8(payload) {
                    Ok(s) => {
                        yield Ok::<Event, axum::Error>(
                            Event::default().event("chat_message").data(s)
                        );
                    }
                    Err(_) => continue,
                }
            }
        }
    };

    Ok(Sse::new(stream).keep_alive(KeepAlive::new().interval(Duration::from_secs(15))))
}
