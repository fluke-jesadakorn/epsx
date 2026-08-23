use axum::{
    extract::{Path, Query, State},
    http::HeaderMap,
    response::sse::{Event, KeepAlive, Sse},
    response::IntoResponse,
    Extension, Json,
};
use diesel::dsl::count_star;
use diesel::{
    prelude::*,
    sql_query,
    sql_types::{Jsonb, Text, Uuid as DieselUuid},
};
use diesel_async::RunQueryDsl;
use serde::Deserialize;
use serde::Serialize;

use std::time::Duration;
use tracing::{error, info};
use uuid::Uuid;

use crate::infrastructure::models::chat::*;
use crate::infrastructure::repositories::ChatRepository;
use crate::infrastructure::services::audit_service::{AuditCtx, AuditEntry};
use crate::schemas::primary::chat_conversations;
use crate::web::{
    auth::AppState,
    middleware::{OpenIDUserContext, RequestId},
    responses::UnifiedApiResponse,
};

const ADMIN_AUDIENCE: &str = "epsx-admin";
const CHAT_READ_PERMISSION: &str = "admin:chat:read";
const CHAT_MANAGE_PERMISSION: &str = "admin:chat:manage";
const DEFAULT_PAGE: u32 = 1;
const DEFAULT_LIMIT: u32 = 20;
const MAX_LIMIT: u32 = 50;
const MAX_OFFSET: i64 = 1_000_000;
const MAX_FILTER_CHARS: usize = 128;
const MAX_SUBJECT_CHARS: usize = 255;
const MAX_MESSAGE_CHARS: usize = 16_384;
const MAX_IDEMPOTENCY_KEY_CHARS: usize = 128;

#[derive(Debug)]
enum ChatMutationClaimError {
    Invalid,
    Conflict,
    Database,
}

#[derive(Debug, QueryableByName)]
struct ExistingChatOperation {
    #[diesel(sql_type = Text)]
    action: String,
    #[diesel(sql_type = DieselUuid)]
    conversation_id: Uuid,
    #[diesel(sql_type = Text)]
    actor: String,
}

fn chat_idempotency_key(headers: &HeaderMap) -> Result<String, ChatMutationClaimError> {
    let value = headers
        .get("idempotency-key")
        .ok_or(ChatMutationClaimError::Invalid)?
        .to_str()
        .map_err(|_| ChatMutationClaimError::Invalid)?
        .trim();
    if value.is_empty()
        || value.chars().count() > MAX_IDEMPOTENCY_KEY_CHARS
        || value.chars().any(char::is_control)
    {
        return Err(ChatMutationClaimError::Invalid);
    }
    Ok(value.to_string())
}

async fn claim_chat_operation(
    app_state: &AppState,
    headers: &HeaderMap,
    context: &OpenIDUserContext,
    conversation_id: Uuid,
    action: &'static str,
) -> Result<(Uuid, String), ChatMutationClaimError> {
    let key = chat_idempotency_key(headers)?;
    let operation_id = Uuid::new_v4();
    let mut conn = app_state
        .db_pool
        .acquire().await
        .await
        .map_err(|_| ChatMutationClaimError::Database)?;
    let inserted = sql_query(
        "INSERT INTO admin_chat_operations
         (operation_id, idempotency_key, conversation_id, action, actor)
         VALUES ($1, $2, $3, $4, $5)
         ON CONFLICT (idempotency_key) DO NOTHING",
    )
    .bind::<DieselUuid, _>(operation_id)
    .bind::<Text, _>(&key)
    .bind::<DieselUuid, _>(conversation_id)
    .bind::<Text, _>(action)
    .bind::<Text, _>(&context.wallet_address)
    .execute(&mut *conn)
    .await
    .map_err(|_| ChatMutationClaimError::Database)?;
    if inserted == 1 {
        return Ok((operation_id, key));
    }
    let existing = sql_query(
        "SELECT action, conversation_id, actor
         FROM admin_chat_operations WHERE idempotency_key = $1",
    )
    .bind::<Text, _>(&key)
    .get_result::<ExistingChatOperation>(&mut conn)
    .await
    .map_err(|_| ChatMutationClaimError::Database)?;
    let _same_operation = existing.action == action
        && existing.conversation_id == conversation_id
        && existing.actor == context.wallet_address;
    Err(ChatMutationClaimError::Conflict)
}

async fn complete_chat_operation(
    app_state: &AppState,
    operation_id: Uuid,
    result: serde_json::Value,
) -> Result<(), ChatMutationClaimError> {
    let mut conn = app_state
        .db_pool
        .acquire().await
        .await
        .map_err(|_| ChatMutationClaimError::Database)?;
    diesel::sql_query(
        "UPDATE admin_chat_operations
         SET result = $1, completed_at = NOW() WHERE operation_id = $2",
    )
    .bind::<Jsonb, _>(result)
    .bind::<DieselUuid, _>(operation_id)
    .execute(&mut *conn)
    .await
    .map_err(|_| ChatMutationClaimError::Database)?;
    Ok(())
}

async fn audit_chat_mutation(
    app_state: &AppState,
    context: &OpenIDUserContext,
    headers: &HeaderMap,
    request_id: &RequestId,
    conversation_id: Uuid,
    action: &'static str,
    idempotency_key: &str,
) -> bool {
    app_state
        .audit
        .log_sync(
            &AuditCtx::from_wallet(&context.wallet_address, headers),
            &AuditEntry::new("chat", action, "support")
                .id(&conversation_id.to_string())
                .meta(serde_json::json!({
                    "request_id": request_id.0,
                    "idempotency_key": idempotency_key,
                })),
        )
        .await
        .is_ok()
}

fn chat_claim_failure(error: ChatMutationClaimError) -> Json<UnifiedApiResponse<()>> {
    match error {
        ChatMutationClaimError::Invalid => Json(UnifiedApiResponse::error(
            400,
            "Invalid idempotency key",
            "A bounded idempotency-key header is required",
        )),
        ChatMutationClaimError::Conflict => Json(UnifiedApiResponse::error(
            409,
            "Chat mutation already claimed",
            "Use the original request result or a new idempotency key",
        )),
        ChatMutationClaimError::Database => Json(UnifiedApiResponse::error(
            503,
            "Chat mutation unavailable",
            "The durable chat operation ledger is unavailable",
        )),
    }
}

fn default_page() -> u32 {
    DEFAULT_PAGE
}
fn default_limit() -> u32 {
    DEFAULT_LIMIT
}

// ============================================================================
// QUERY PARAMS
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct AdminConversationQuery {
    pub status: Option<String>,
    pub topic_id: Option<Uuid>,
    pub agent: Option<String>,
    #[serde(default = "default_page")]
    pub page: u32,
    #[serde(default = "default_limit")]
    pub limit: u32,
}

#[derive(Debug, Serialize)]
#[serde(deny_unknown_fields)]
pub struct AdminConversationSummary {
    pub id: Uuid,
    pub topic_id: Uuid,
    pub wallet_address: String,
    pub subject: String,
    pub status: String,
    pub assigned_agent: Option<String>,
    pub last_message_at: chrono::DateTime<chrono::Utc>,
    pub unread_user: i32,
    pub unread_agent: i32,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Serialize)]
#[serde(deny_unknown_fields)]
pub struct AdminMessageSummary {
    pub id: Uuid,
    pub conversation_id: Uuid,
    pub sender_type: String,
    pub sender_address: Option<String>,
    pub content: String,
    pub is_read: bool,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Serialize)]
#[serde(deny_unknown_fields)]
pub struct AdminConversationPage {
    pub items: Vec<AdminConversationSummary>,
    pub total: i64,
    pub page: u32,
    pub limit: u32,
    pub has_next: bool,
}

#[derive(Debug, Serialize)]
#[serde(deny_unknown_fields)]
pub struct AdminTopicSummary {
    pub id: Uuid,
    pub name: String,
    pub label: String,
    pub is_active: bool,
}

#[derive(Debug, Serialize)]
#[serde(deny_unknown_fields)]
pub struct AdminChatOverview {
    pub stats: ChatStatsResponse,
    pub conversations: Vec<AdminConversationSummary>,
    pub topics: Vec<AdminTopicSummary>,
}

fn authorize(
    context: &OpenIDUserContext,
    request_id: &RequestId,
    permission: &'static str,
    operation: &'static str,
) -> Result<(), epsx_contracts::errors::AppError> {
    let request_id = Some(request_id.0.clone());
    if !matches!(
        context.token_audiences.as_deref(),
        Some([audience]) if audience == ADMIN_AUDIENCE
    ) {
        return Err(epsx_contracts::errors::AppError::with_full_context(
            epsx_contracts::errors::ErrorKind::AuthenticationError,
            "A valid admin audience is required",
            Some(context.wallet_address.clone()),
            request_id,
            operation,
            "admin-chat",
        ));
    }
    if !epsx_contracts::permissions::has_permission(&context.permissions, permission) {
        return Err(epsx_contracts::errors::AppError::with_full_context(
            epsx_contracts::errors::ErrorKind::AuthorizationError,
            "The required chat permission is missing",
            Some(context.wallet_address.clone()),
            request_id,
            operation,
            "admin-chat",
        ));
    }
    Ok(())
}

fn auth_failure(error: epsx_contracts::errors::AppError) -> Json<UnifiedApiResponse<()>> {
    Json(UnifiedApiResponse::error(
        error.http_status(),
        "Chat request rejected",
        &format!("correlation_id={}", error.correlation_id),
    ))
}

fn admin_chat_stream_is_authorized(audiences: &[String], permissions: &[String]) -> bool {
    matches!(audiences, [audience] if audience == ADMIN_AUDIENCE)
        && epsx_contracts::permissions::has_permission(permissions, CHAT_READ_PERMISSION)
}

fn stream_auth_error(
    request: &axum::extract::Request,
    kind: epsx_contracts::errors::ErrorKind,
    message: &'static str,
) -> epsx_contracts::errors::AppError {
    epsx_contracts::errors::AppError::with_full_context(
        kind,
        message,
        None,
        request
            .extensions()
            .get::<RequestId>()
            .map(|request_id| request_id.0.clone()),
        "admin.chat.stream",
        "admin-chat",
    )
}

fn validate_query(query: &AdminConversationQuery) -> Result<(), epsx_contracts::errors::AppError> {
    if !(1..=MAX_LIMIT).contains(&query.limit) || query.page == 0 {
        return Err(epsx_contracts::errors::AppError::bad_request(
            "chat page must be positive and limit must be between 1 and 50",
        ));
    }
    if query
        .status
        .as_deref()
        .is_some_and(|status| !matches!(status, "open" | "in_progress" | "resolved" | "closed"))
        || query
            .agent
            .as_deref()
            .is_some_and(|agent| !bounded_text(agent, MAX_FILTER_CHARS))
    {
        return Err(epsx_contracts::errors::AppError::bad_request(
            "invalid chat filter",
        ));
    }
    let _ = query
        .page
        .checked_sub(1)
        .and_then(|page| page.checked_mul(query.limit))
        .map(i64::from)
        .filter(|offset| *offset <= MAX_OFFSET)
        .ok_or_else(|| {
            epsx_contracts::errors::AppError::bad_request("chat page is out of bounds")
        })?;
    Ok(())
}

fn offset(query: &AdminConversationQuery) -> i64 {
    i64::from(query.page - 1) * i64::from(query.limit)
}

fn bounded_text(value: &str, max_chars: usize) -> bool {
    !value.trim().is_empty()
        && value.chars().count() <= max_chars
        && !value.chars().any(char::is_control)
}

fn project_conversation(
    conversation: ChatConversationDb,
) -> Result<AdminConversationSummary, epsx_contracts::errors::AppError> {
    if !bounded_text(&conversation.wallet_address, 128)
        || !bounded_text(&conversation.subject, MAX_SUBJECT_CHARS)
        || !matches!(
            conversation.status.as_str(),
            "open" | "in_progress" | "resolved" | "closed"
        )
        || conversation.unread_user < 0
        || conversation.unread_agent < 0
        || conversation
            .assigned_agent
            .as_deref()
            .is_some_and(|agent| !bounded_text(agent, 128))
    {
        return Err(epsx_contracts::errors::AppError::internal_error(
            "chat conversation data failed projection validation",
        ));
    }
    Ok(AdminConversationSummary {
        id: conversation.id,
        topic_id: conversation.topic_id,
        wallet_address: conversation.wallet_address,
        subject: conversation.subject,
        status: conversation.status,
        assigned_agent: conversation.assigned_agent,
        last_message_at: conversation.last_message_at,
        unread_user: conversation.unread_user,
        unread_agent: conversation.unread_agent,
        created_at: conversation.created_at,
        updated_at: conversation.updated_at,
    })
}

fn project_message(
    message: ChatMessageDb,
) -> Result<AdminMessageSummary, epsx_contracts::errors::AppError> {
    if !bounded_text(&message.sender_type, 32)
        || message
            .sender_address
            .as_deref()
            .is_some_and(|sender| !bounded_text(sender, 128))
        || !bounded_text(&message.content, MAX_MESSAGE_CHARS)
    {
        return Err(epsx_contracts::errors::AppError::internal_error(
            "chat message data failed projection validation",
        ));
    }
    Ok(AdminMessageSummary {
        id: message.id,
        conversation_id: message.conversation_id,
        sender_type: message.sender_type,
        sender_address: message.sender_address,
        content: message.content,
        is_read: message.is_read,
        created_at: message.created_at,
    })
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
    Extension(context): Extension<OpenIDUserContext>,
    Extension(request_id): Extension<RequestId>,
) -> Result<Json<UnifiedApiResponse<AdminConversationPage>>, epsx_contracts::errors::AppError> {
    authorize(
        &context,
        &request_id,
        CHAT_READ_PERMISSION,
        "list_conversations",
    )?;
    validate_query(&query)?;
    let mut conn = app_state.db_pool.acquire().await.map_err(|_| {
        epsx_contracts::errors::AppError::database_error("chat database unavailable")
    })?;

    let mut count_query = chat_conversations::table.into_boxed();
    if let Some(status) = query.status.as_deref() {
        count_query = count_query.filter(chat_conversations::status.eq(status));
    }
    if let Some(topic_id) = query.topic_id {
        count_query = count_query.filter(chat_conversations::topic_id.eq(topic_id));
    }
    if let Some(agent) = query.agent.as_deref() {
        count_query = count_query.filter(chat_conversations::assigned_agent.eq(agent));
    }

    let total = count_query
        .select(count_star())
        .first::<i64>(&mut conn)
        .await
        .map_err(|_| epsx_contracts::errors::AppError::database_error("chat count unavailable"))?;
    let mut rows_query = chat_conversations::table.into_boxed();
    if let Some(status) = query.status.as_deref() {
        rows_query = rows_query.filter(chat_conversations::status.eq(status));
    }
    if let Some(topic_id) = query.topic_id {
        rows_query = rows_query.filter(chat_conversations::topic_id.eq(topic_id));
    }
    if let Some(agent) = query.agent.as_deref() {
        rows_query = rows_query.filter(chat_conversations::assigned_agent.eq(agent));
    }
    let rows = rows_query
        .order(chat_conversations::last_message_at.desc())
        .limit(i64::from(query.limit))
        .offset(offset(&query))
        .load::<ChatConversationDb>(&mut conn)
        .await
        .map_err(|_| {
            epsx_contracts::errors::AppError::database_error("chat conversations unavailable")
        })?;
    let items = rows
        .into_iter()
        .map(project_conversation)
        .collect::<Result<Vec<_>, _>>()?;
    let has_next = offset(&query)
        .checked_add(i64::try_from(items.len()).unwrap_or(i64::MAX))
        .is_some_and(|end| end < total);
    Ok(Json(UnifiedApiResponse::success(AdminConversationPage {
        items,
        total,
        page: query.page,
        limit: query.limit,
        has_next,
    })))
}

/// Get conversation detail (admin can see any)
pub async fn admin_get_conversation(
    State(app_state): State<AppState>,
    Path(id): Path<Uuid>,
    Extension(context): Extension<OpenIDUserContext>,
    Extension(request_id): Extension<RequestId>,
) -> Result<Json<UnifiedApiResponse<AdminConversationSummary>>, epsx_contracts::errors::AppError> {
    authorize(
        &context,
        &request_id,
        CHAT_READ_PERMISSION,
        "get_conversation",
    )?;
    match ChatRepository::get_conversation(&app_state.db_pool, id).await {
        Ok(Some(conv)) => Ok(Json(UnifiedApiResponse::success(project_conversation(
            conv,
        )?))),
        Ok(None) => Err(epsx_contracts::errors::AppError::not_found(
            "conversation not found",
        )),
        Err(_) => Err(epsx_contracts::errors::AppError::database_error(
            "conversation unavailable",
        )),
    }
}

/// List messages (admin can see any conversation)
pub async fn admin_list_messages(
    State(app_state): State<AppState>,
    Path(id): Path<Uuid>,
    Extension(context): Extension<OpenIDUserContext>,
    Extension(request_id): Extension<RequestId>,
) -> Result<Json<UnifiedApiResponse<Vec<AdminMessageSummary>>>, epsx_contracts::errors::AppError> {
    authorize(&context, &request_id, CHAT_READ_PERMISSION, "list_messages")?;
    match ChatRepository::list_messages(&app_state.db_pool, id).await {
        Ok(msgs) => Ok(Json(UnifiedApiResponse::success(
            msgs.into_iter()
                .map(project_message)
                .collect::<Result<Vec<_>, _>>()?,
        ))),
        Err(_) => Err(epsx_contracts::errors::AppError::database_error(
            "chat messages unavailable",
        )),
    }
}

/// Agent sends reply
pub async fn admin_send_reply(
    State(app_state): State<AppState>,
    Extension(ctx): Extension<OpenIDUserContext>,
    Extension(request_id): Extension<RequestId>,
    headers: HeaderMap,
    Path(id): Path<Uuid>,
    Json(body): Json<SendMessageRequest>,
) -> Result<Json<UnifiedApiResponse<AdminMessageSummary>>, Json<UnifiedApiResponse<()>>> {
    if let Err(error) = authorize(&ctx, &request_id, CHAT_MANAGE_PERMISSION, "send_reply") {
        return Err(auth_failure(error));
    }
    if !bounded_text(&body.content, MAX_MESSAGE_CHARS) {
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
    let (operation_id, idempotency_key) =
        match claim_chat_operation(&app_state, &headers, &ctx, id, "send_reply").await {
            Ok(value) => value,
            Err(error) => return Err(chat_claim_failure(error)),
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
            let notif_msg_id = msg.id;
            let projected = match project_message(msg) {
                Ok(value) => value,
                Err(error) => return Err(auth_failure(error)),
            };
            // Publish to user's channel
            if let Some(pubsub) = &app_state.pubsub {
                let event = serde_json::json!({
                    "type": "new_message",
                    "conversation_id": id,
                    "message": projected,
                });
                let payload = serde_json::to_vec(&event).unwrap_or_default();
                let channel = format!("chat:wallet:{}", conv.wallet_address);
                let _ = pubsub.publish(&channel, &payload).await;
            }

            // Notify user about new support message
            let notif_wallet = conv.wallet_address.clone();
            let notif_conv_id = id;
            let notif_content = body.content.chars().take(100).collect::<String>();
            let notif_state = app_state.clone();
            tokio::spawn(async move {
                // Wave 10 / R3: route through the NotificationPort.
                use epsx_contracts::notification_port::SendNotificationRequest;
                if let Some(port) = notif_state.notification_port.as_ref() {
                    let _ = port
                        .send_with_event_id_retry(
                            &format!("chat.message:{notif_msg_id}"),
                            SendNotificationRequest {
                                recipient_wallet_address: notif_wallet.clone(),
                                notification_type: "chat".to_string(),
                                priority: "normal".to_string(),
                                title: "New Support Message".to_string(),
                                message: notif_content.clone(),
                                data: Some(serde_json::json!({ "conversation_id": notif_conv_id })),
                                action_url: Some(format!("/chat/{}", notif_conv_id)),
                                expires_at: None,
                            },
                        )
                        .await;
                } else {
                    tracing::warn!(
                        "notification_port not wired in AppState; admin chat-reply \
                         notification for conversation={} dropped",
                        notif_conv_id
                    );
                }
            });

            info!(
                "Agent {} replied to conversation {}",
                ctx.wallet_address, id
            );
            if !audit_chat_mutation(
                &app_state,
                &ctx,
                &headers,
                &request_id,
                id,
                "send_reply",
                &idempotency_key,
            )
            .await
            {
                return Err(Json(UnifiedApiResponse::error(
                    503,
                    "Chat mutation pending",
                    "The chat audit record could not be durably written",
                )));
            }
            if complete_chat_operation(
                &app_state,
                operation_id,
                serde_json::to_value(&projected).unwrap_or_default(),
            )
            .await
            .is_err()
            {
                return Err(Json(UnifiedApiResponse::error(
                    503,
                    "Chat mutation pending",
                    "The chat operation result could not be durably recorded",
                )));
            }
            Ok(Json(UnifiedApiResponse::success(projected)))
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
    Extension(request_id): Extension<RequestId>,
    headers: HeaderMap,
    Path(id): Path<Uuid>,
    Json(body): Json<AssignAgentRequest>,
) -> Result<Json<UnifiedApiResponse<AdminConversationSummary>>, Json<UnifiedApiResponse<()>>> {
    if let Err(error) = authorize(&ctx, &request_id, CHAT_MANAGE_PERMISSION, "assign_agent") {
        return Err(auth_failure(error));
    }
    let agent = body.agent_address.as_deref().unwrap_or(&ctx.wallet_address);
    if !bounded_text(agent, 128) {
        return Err(Json(UnifiedApiResponse::error(
            400,
            "Invalid agent",
            "agent address is bounded and required",
        )));
    }

    let (operation_id, idempotency_key) =
        match claim_chat_operation(&app_state, &headers, &ctx, id, "assign_agent").await {
            Ok(value) => value,
            Err(error) => return Err(chat_claim_failure(error)),
        };

    match ChatRepository::assign_agent(&app_state.db_pool, id, Some(agent)).await {
        Ok(conv) => {
            let projected = match project_conversation(conv) {
                Ok(value) => value,
                Err(error) => return Err(auth_failure(error)),
            };
            // Publish agent_assigned to relevant channels
            if let Some(pubsub) = &app_state.pubsub {
                let event = serde_json::json!({
                    "type": "agent_assigned",
                    "conversation_id": id,
                    "assigned_agent": agent,
                    "conversation": projected,
                });
                let payload = serde_json::to_vec(&event).unwrap_or_default();
                let _ = pubsub.publish("chat:new", &payload).await;
                let channel = format!("chat:agent:{}", agent);
                let _ = pubsub.publish(&channel, &payload).await;
            }
            info!("Agent {} assigned to conversation {}", agent, id);
            if !audit_chat_mutation(
                &app_state,
                &ctx,
                &headers,
                &request_id,
                id,
                "assign_agent",
                &idempotency_key,
            )
            .await
            {
                return Err(Json(UnifiedApiResponse::error(
                    503,
                    "Chat mutation pending",
                    "The chat audit record could not be durably written",
                )));
            }
            if complete_chat_operation(
                &app_state,
                operation_id,
                serde_json::to_value(&projected).unwrap_or_default(),
            )
            .await
            .is_err()
            {
                return Err(Json(UnifiedApiResponse::error(
                    503,
                    "Chat mutation pending",
                    "The chat operation result could not be durably recorded",
                )));
            }
            Ok(Json(UnifiedApiResponse::success(projected)))
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
    Extension(ctx): Extension<OpenIDUserContext>,
    Extension(request_id): Extension<RequestId>,
    headers: HeaderMap,
    Path(id): Path<Uuid>,
    Json(body): Json<UpdateStatusRequest>,
) -> Result<Json<UnifiedApiResponse<AdminConversationSummary>>, Json<UnifiedApiResponse<()>>> {
    if let Err(error) = authorize(&ctx, &request_id, CHAT_MANAGE_PERMISSION, "update_status") {
        return Err(auth_failure(error));
    }
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
    let (operation_id, idempotency_key) =
        match claim_chat_operation(&app_state, &headers, &ctx, id, "update_status").await {
            Ok(value) => value,
            Err(error) => return Err(chat_claim_failure(error)),
        };

    match ChatRepository::update_status(&app_state.db_pool, id, &body.status).await {
        Ok(conv) => {
            let projected = match project_conversation(conv) {
                Ok(value) => value,
                Err(error) => return Err(auth_failure(error)),
            };
            // Publish status_changed to user + admin channels
            if let Some(pubsub) = &app_state.pubsub {
                let event = serde_json::json!({
                    "type": "status_changed",
                    "conversation_id": id,
                    "status": body.status,
                    "conversation": projected,
                });
                let payload = serde_json::to_vec(&event).unwrap_or_default();
                let channel = format!("chat:wallet:{}", existing.wallet_address);
                let _ = pubsub.publish(&channel, &payload).await;
                let _ = pubsub.publish("chat:new", &payload).await;
            }
            if !audit_chat_mutation(
                &app_state,
                &ctx,
                &headers,
                &request_id,
                id,
                "update_status",
                &idempotency_key,
            )
            .await
            {
                return Err(Json(UnifiedApiResponse::error(
                    503,
                    "Chat mutation pending",
                    "The chat audit record could not be durably written",
                )));
            }
            if complete_chat_operation(
                &app_state,
                operation_id,
                serde_json::to_value(&projected).unwrap_or_default(),
            )
            .await
            .is_err()
            {
                return Err(Json(UnifiedApiResponse::error(
                    503,
                    "Chat mutation pending",
                    "The chat operation result could not be durably recorded",
                )));
            }
            Ok(Json(UnifiedApiResponse::success(projected)))
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
    Extension(ctx): Extension<OpenIDUserContext>,
    Extension(request_id): Extension<RequestId>,
    headers: HeaderMap,
    Path(id): Path<Uuid>,
) -> Result<Json<UnifiedApiResponse<()>>, Json<UnifiedApiResponse<()>>> {
    if let Err(error) = authorize(&ctx, &request_id, CHAT_MANAGE_PERMISSION, "mark_read") {
        return Err(auth_failure(error));
    }
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
    let (operation_id, idempotency_key) =
        match claim_chat_operation(&app_state, &headers, &ctx, id, "mark_read").await {
            Ok(value) => value,
            Err(error) => return Err(chat_claim_failure(error)),
        };

    match ChatRepository::mark_read_by_agent(&app_state.db_pool, id).await {
        Ok(()) => {
            // Notify user that agent has read their messages
            if let Some(pubsub) = &app_state.pubsub {
                let event = serde_json::json!({
                    "type": "messages_read",
                    "conversation_id": id,
                    "reader": "agent",
                });
                let payload = serde_json::to_vec(&event).unwrap_or_default();
                let channel = format!("chat:wallet:{}", conv.wallet_address);
                let _ = pubsub.publish(&channel, &payload).await;
            }
            if !audit_chat_mutation(
                &app_state,
                &ctx,
                &headers,
                &request_id,
                id,
                "mark_read",
                &idempotency_key,
            )
            .await
            {
                return Err(Json(UnifiedApiResponse::error(
                    503,
                    "Chat mutation pending",
                    "The chat audit record could not be durably written",
                )));
            }
            if complete_chat_operation(&app_state, operation_id, serde_json::json!({}))
                .await
                .is_err()
            {
                return Err(Json(UnifiedApiResponse::error(
                    503,
                    "Chat mutation pending",
                    "The chat operation result could not be durably recorded",
                )));
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
    Extension(context): Extension<OpenIDUserContext>,
    Extension(request_id): Extension<RequestId>,
) -> Result<Json<UnifiedApiResponse<ChatStatsResponse>>, epsx_contracts::errors::AppError> {
    authorize(
        &context,
        &request_id,
        CHAT_READ_PERMISSION,
        "get_chat_stats",
    )?;
    match ChatRepository::get_stats(&app_state.db_pool).await {
        Ok(stats) => Ok(Json(UnifiedApiResponse::success(stats))),
        Err(_) => Err(epsx_contracts::errors::AppError::database_error(
            "chat stats unavailable",
        )),
    }
}

/// List topics (admin)
pub async fn admin_list_topics(
    State(app_state): State<AppState>,
    Extension(context): Extension<OpenIDUserContext>,
    Extension(request_id): Extension<RequestId>,
) -> Result<Json<UnifiedApiResponse<Vec<AdminTopicSummary>>>, epsx_contracts::errors::AppError> {
    authorize(
        &context,
        &request_id,
        CHAT_READ_PERMISSION,
        "list_chat_topics",
    )?;
    match ChatRepository::list_topics(&app_state.db_pool).await {
        Ok(topics) => {
            let topics = topics
                .into_iter()
                .map(|topic| {
                    if !bounded_text(&topic.name, 128) || !bounded_text(&topic.label, 128) {
                        return Err(epsx_contracts::errors::AppError::internal_error(
                            "chat topic data failed projection validation",
                        ));
                    }
                    Ok(AdminTopicSummary {
                        id: topic.id,
                        name: topic.name,
                        label: topic.label,
                        is_active: topic.is_active,
                    })
                })
                .collect::<Result<Vec<_>, _>>()?;
            Ok(Json(UnifiedApiResponse::success(topics)))
        }
        Err(_) => Err(epsx_contracts::errors::AppError::database_error(
            "chat topics unavailable",
        )),
    }
}

/// Admin chat overview: stats + conversations + topics in one call
/// GET /admin/chat/overview
pub async fn admin_chat_overview_handler(
    State(app_state): State<AppState>,
    Extension(context): Extension<OpenIDUserContext>,
    Extension(request_id): Extension<RequestId>,
) -> Result<Json<UnifiedApiResponse<AdminChatOverview>>, epsx_contracts::errors::AppError> {
    authorize(
        &context,
        &request_id,
        CHAT_READ_PERMISSION,
        "get_chat_overview",
    )?;
    info!("Admin: Getting chat overview");

    let (stats, conversations, topics) = tokio::join!(
        ChatRepository::get_stats(&app_state.db_pool),
        ChatRepository::list_all_conversations(&app_state.db_pool, None, None, None),
        ChatRepository::list_topics(&app_state.db_pool),
    );

    let stats = stats
        .map_err(|_| epsx_contracts::errors::AppError::database_error("chat stats unavailable"))?;
    let conversations = conversations
        .map_err(|_| {
            epsx_contracts::errors::AppError::database_error("chat conversations unavailable")
        })?
        .into_iter()
        .map(project_conversation)
        .collect::<Result<Vec<_>, _>>()?;
    let topics = topics
        .map_err(|_| epsx_contracts::errors::AppError::database_error("chat topics unavailable"))?
        .into_iter()
        .map(|topic| {
            if !bounded_text(&topic.name, 128) || !bounded_text(&topic.label, 128) {
                return Err(epsx_contracts::errors::AppError::internal_error(
                    "chat topic data failed projection validation",
                ));
            }
            Ok(AdminTopicSummary {
                id: topic.id,
                name: topic.name,
                label: topic.label,
                is_active: topic.is_active,
            })
        })
        .collect::<Result<Vec<_>, _>>()?;

    Ok(Json(UnifiedApiResponse::success(AdminChatOverview {
        stats,
        conversations,
        topics,
    })))
}

/// SSE stream for admin - listens to new conversations + assigned conversations
pub async fn admin_chat_stream(
    State(app_state): State<AppState>,
    Query(_query): Query<AdminChatSSEQuery>,
    request: axum::extract::Request,
) -> Result<impl IntoResponse, epsx_contracts::errors::AppError> {
    let token = crate::web::middleware::bearer_middleware::extract_bearer_token_from_headers(
        request.headers(),
    )
    .ok_or_else(|| {
        stream_auth_error(
            &request,
            epsx_contracts::errors::ErrorKind::AuthenticationError,
            "Authentication required for admin chat stream",
        )
    })?;

    let token_service = app_state
        .domain_container
        .get_token_service()
        .ok_or_else(|| {
            stream_auth_error(
                &request,
                epsx_contracts::errors::ErrorKind::InternalServerError,
                "Authentication service unavailable",
            )
        })?;

    let claims = token_service
        .validate_access_token(&token)
        .await
        .map_err(|_| {
            stream_auth_error(
                &request,
                epsx_contracts::errors::ErrorKind::AuthenticationError,
                "Invalid or expired authentication token",
            )
        })?;

    let permissions: Vec<String> = claims
        .scope
        .split_whitespace()
        .filter(|s| *s != "openid" && *s != "profile")
        .map(|s| s.to_string())
        .collect();
    if !admin_chat_stream_is_authorized(&claims.aud, &permissions) {
        return Err(stream_auth_error(
            &request,
            epsx_contracts::errors::ErrorKind::AuthorizationError,
            "The required chat read permission is missing",
        ));
    }
    let wallet_address = claims.wallet_address.to_lowercase();

    info!("Admin Chat SSE connection: wallet={}", wallet_address);

    let pubsub = app_state.pubsub.clone();

    // Subscribe to new conversation channel + agent-specific channel
    let mut message_stream = match &pubsub {
        Some(port) => {
            // Collect all channels we need to subscribe to. The
            // admin chat stream is multi-channel: every admin gets
            // `chat:new`, and a specific agent also gets
            // `chat:agent:<wallet>`.
            let mut channels: Vec<String> = vec!["chat:new".to_string()];
            if wallet_address != "all" {
                channels.push(format!("chat:agent:{}", wallet_address));
            }
            let channel_refs: Vec<&str> = channels.iter().map(|s| s.as_str()).collect();
            Some(port.subscribe(&channel_refs)?)
        }
        None => None,
    };

    let stream = async_stream::stream! {
        yield Ok::<Event, axum::Error>(Event::default().event("ping").data("connected"));

        if let Some(ref mut stream) = message_stream {
            while let Some(payload) = stream.next_message().await {
                match String::from_utf8(payload) {
                    Ok(s) => {
                        yield Ok::<Event, axum::Error>(
                            Event::default().event("chat_event").data(s)
                        );
                    }
                    Err(_) => continue,
                }
            }
        }
    };

    Ok(Sse::new(stream).keep_alive(KeepAlive::new().interval(Duration::from_secs(15))))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn values(values: &[&str]) -> Vec<String> {
        values.iter().map(|value| (*value).to_string()).collect()
    }

    #[test]
    fn chat_stream_requires_exact_admin_audience_and_read_permission() {
        assert!(admin_chat_stream_is_authorized(
            &values(&[ADMIN_AUDIENCE]),
            &values(&[CHAT_READ_PERMISSION]),
        ));
        assert!(!admin_chat_stream_is_authorized(
            &values(&[ADMIN_AUDIENCE]),
            &values(&[CHAT_MANAGE_PERMISSION]),
        ));
        assert!(!admin_chat_stream_is_authorized(
            &values(&["epsx-admin", "epsx-frontend"]),
            &values(&[CHAT_READ_PERMISSION]),
        ));
        assert!(!admin_chat_stream_is_authorized(
            &values(&["epsx-frontend"]),
            &values(&["admin:*:*"]),
        ));
    }
}
