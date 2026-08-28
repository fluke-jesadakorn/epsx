//! Same-origin BFF adapter for the backend-owned support chat.

use axum::{
    body::Body,
    extract::{Multipart, Path, Request, State},
    http::{header, HeaderValue, StatusCode},
    response::{IntoResponse, Redirect, Response},
    Json,
};
use epsx_bff::session::SessionUser;
use epsx_client::ServiceClient;
use futures::StreamExt;
use serde::{de::DeserializeOwned, Deserialize, Serialize};

use epsx_dioxus_ui::pages::chat::{
    decode_chat_detail, decode_chat_inbox, ChatAttachment, ChatConversation, ChatDetailData,
    ChatInboxData, ChatMessage, ChatTopic,
};

use crate::AppState;

const CHAT_INBOX_PATH: &str = "/api/chat/inbox";
const CHAT_CONVERSATIONS_PATH: &str = "/api/chat/conversations";
const MAX_REQUEST_BYTES: usize = 24 * 1024;
const MAX_RESPONSE_BYTES: usize = 512 * 1024;
const MAX_MESSAGE_CHARS: usize = 16_384;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ChatLoadError {
    Forbidden,
    Unavailable,
    Malformed,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct BackendEnvelope<T> {
    success: bool,
    data: Option<T>,
    error: Option<serde_json::Value>,
    meta: Option<serde_json::Value>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct UpstreamTopic {
    id: String,
    name: String,
    label: String,
    description: Option<String>,
    icon: Option<String>,
    sort_order: i32,
    is_active: bool,
    created_at: String,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct UpstreamConversation {
    id: String,
    topic_id: String,
    wallet_address: String,
    subject: String,
    status: String,
    assigned_agent: Option<String>,
    last_message_at: String,
    unread_user: i32,
    unread_agent: i32,
    metadata: serde_json::Value,
    created_at: String,
    updated_at: String,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct UpstreamMessage {
    id: String,
    conversation_id: String,
    sender_type: String,
    sender_address: Option<String>,
    content: String,
    is_read: bool,
    metadata: serde_json::Value,
    created_at: String,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct UpstreamInbox {
    topics: Vec<UpstreamTopic>,
    conversations: Vec<UpstreamConversation>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct UpstreamDetail {
    conversation: UpstreamConversation,
    messages: Vec<UpstreamMessage>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct CreateConversationRequest {
    topic_id: String,
    subject: String,
    message: String,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct SendMessageRequest {
    content: String,
}

#[derive(Debug, Serialize)]
struct UpdateStatusRequest<'a> {
    status: &'a str,
}

pub(crate) async fn load_chat_inbox_for_ssr(
    client: &ServiceClient,
    bearer: &str,
    expected_owner: &str,
) -> Result<ChatInboxData, ChatLoadError> {
    let upstream = authenticated_get::<UpstreamInbox>(client, CHAT_INBOX_PATH, bearer).await?;
    project_inbox(upstream, expected_owner).ok_or(ChatLoadError::Malformed)
}

pub(crate) async fn load_chat_detail_for_ssr(
    client: &ServiceClient,
    bearer: &str,
    expected_owner: &str,
    id: uuid::Uuid,
) -> Result<ChatDetailData, ChatLoadError> {
    let path = format!("{CHAT_CONVERSATIONS_PATH}/{id}/full");
    let upstream = authenticated_get::<UpstreamDetail>(client, &path, bearer).await?;
    project_detail(upstream, expected_owner, id).ok_or(ChatLoadError::Malformed)
}

async fn authenticated_get<T: DeserializeOwned>(
    client: &ServiceClient,
    path: &str,
    bearer: &str,
) -> Result<T, ChatLoadError> {
    let response = client
        .auth_client()
        .get(format!(
            "{}{}",
            client.base_url().trim_end_matches('/'),
            path
        ))
        .bearer_auth(bearer)
        .header(header::ACCEPT, "application/json")
        .send()
        .await
        .map_err(|_| ChatLoadError::Unavailable)?;
    decode_response(response).await
}

async fn decode_response<T: DeserializeOwned>(
    response: reqwest::Response,
) -> Result<T, ChatLoadError> {
    if matches!(
        response.status(),
        reqwest::StatusCode::UNAUTHORIZED | reqwest::StatusCode::FORBIDDEN
    ) {
        return Err(ChatLoadError::Forbidden);
    }
    if !response.status().is_success() {
        return Err(ChatLoadError::Unavailable);
    }
    let is_json = response
        .headers()
        .get(reqwest::header::CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.split(';').next())
        .is_some_and(|value| value.trim().eq_ignore_ascii_case("application/json"));
    if !is_json {
        return Err(ChatLoadError::Malformed);
    }
    if response
        .content_length()
        .is_some_and(|length| length > MAX_RESPONSE_BYTES as u64)
    {
        return Err(ChatLoadError::Malformed);
    }
    let mut body = Vec::new();
    let mut stream = response.bytes_stream();
    while let Some(chunk) = stream.next().await {
        let chunk = chunk.map_err(|_| ChatLoadError::Unavailable)?;
        if body
            .len()
            .checked_add(chunk.len())
            .is_none_or(|length| length > MAX_RESPONSE_BYTES)
        {
            return Err(ChatLoadError::Malformed);
        }
        body.extend_from_slice(&chunk);
    }
    let envelope = serde_json::from_slice::<BackendEnvelope<T>>(&body)
        .map_err(|_| ChatLoadError::Malformed)?;
    if !envelope.success || envelope.error.is_some() || !valid_meta(envelope.meta.as_ref()) {
        return Err(ChatLoadError::Malformed);
    }
    envelope.data.ok_or(ChatLoadError::Malformed)
}

fn valid_meta(meta: Option<&serde_json::Value>) -> bool {
    let Some(meta) = meta else {
        return true;
    };
    let Some(object) = meta.as_object() else {
        return false;
    };
    object
        .get("timestamp")
        .and_then(serde_json::Value::as_str)
        .is_some_and(|value| {
            value.len() <= 64 && chrono::DateTime::parse_from_rfc3339(value).is_ok()
        })
        && serde_json::to_vec(meta).is_ok_and(|encoded| encoded.len() <= 8 * 1024)
}

fn project_inbox(upstream: UpstreamInbox, expected_owner: &str) -> Option<ChatInboxData> {
    if upstream.topics.len() > 100 || upstream.conversations.len() > 200 {
        return None;
    }
    let topics = upstream
        .topics
        .into_iter()
        .map(project_topic)
        .collect::<Option<Vec<_>>>()?;
    let conversations = upstream
        .conversations
        .into_iter()
        .map(|conversation| project_conversation(conversation, expected_owner))
        .collect::<Option<Vec<_>>>()?;
    let projection = ChatInboxData {
        topics,
        conversations,
    };
    serde_json::to_value(&projection)
        .ok()
        .and_then(decode_chat_inbox)
}

fn project_detail(
    upstream: UpstreamDetail,
    expected_owner: &str,
    expected_id: uuid::Uuid,
) -> Option<ChatDetailData> {
    if upstream.conversation.id != expected_id.to_string() || upstream.messages.len() > 500 {
        return None;
    }
    let conversation = project_conversation(upstream.conversation, expected_owner)?;
    let messages = upstream
        .messages
        .into_iter()
        .map(project_message)
        .collect::<Option<Vec<_>>>()?;
    let projection = ChatDetailData {
        conversation,
        messages,
    };
    serde_json::to_value(&projection)
        .ok()
        .and_then(decode_chat_detail)
}

fn project_topic(topic: UpstreamTopic) -> Option<ChatTopic> {
    if !topic.is_active
        || topic.sort_order < 0
        || topic.created_at.len() > 64
        || chrono::DateTime::parse_from_rfc3339(&topic.created_at).is_err()
    {
        return None;
    }
    Some(ChatTopic {
        id: topic.id,
        name: topic.name,
        label: topic.label,
        description: topic.description,
        icon: topic.icon,
    })
}

fn project_conversation(
    conversation: UpstreamConversation,
    expected_owner: &str,
) -> Option<ChatConversation> {
    if !conversation
        .wallet_address
        .eq_ignore_ascii_case(expected_owner)
        || conversation.unread_agent < 0
        || serde_json::to_vec(&conversation.metadata)
            .ok()
            .is_none_or(|encoded| encoded.len() > 32 * 1024)
    {
        return None;
    }
    Some(ChatConversation {
        id: conversation.id,
        topic_id: conversation.topic_id,
        subject: conversation.subject,
        status: conversation.status,
        assigned_agent: conversation.assigned_agent,
        last_message_at: conversation.last_message_at,
        unread_user: conversation.unread_user,
        created_at: conversation.created_at,
        updated_at: conversation.updated_at,
    })
}

fn project_message(message: UpstreamMessage) -> Option<ChatMessage> {
    if message
        .sender_address
        .as_deref()
        .is_some_and(|value| !bounded_text(value, 128))
        || serde_json::to_vec(&message.metadata)
            .ok()
            .is_none_or(|encoded| encoded.len() > 32 * 1024)
    {
        return None;
    }
    let attachment = message
        .metadata
        .get("attachments")
        .and_then(|value| value.as_array())
        .and_then(|array| array.first())
        .and_then(|value| {
            let filename = value.get("filename")?.as_str()?.trim().to_string();
            let url = value.get("url")?.as_str()?.trim().to_string();
            let file_type = value
                .get("file_type")
                .and_then(|v| v.as_str())
                .unwrap_or("application/octet-stream")
                .to_string();
            let size = value.get("size").and_then(|v| v.as_u64()).unwrap_or(0);
            if filename.is_empty()
                || filename.chars().count() > 255
                || filename.chars().any(char::is_control)
                || url.is_empty()
                || url.len() > 2048
                || file_type.len() > 128
                || size > 10 * 1024 * 1024
            {
                return None;
            }
            Some(ChatAttachment {
                filename,
                url,
                file_type,
                size,
            })
        });
    Some(ChatMessage {
        id: message.id,
        conversation_id: message.conversation_id,
        sender_type: message.sender_type,
        content: message.content,
        is_read: message.is_read,
        created_at: message.created_at,
        attachment,
    })
}

fn bounded_text(value: &str, max_chars: usize) -> bool {
    !value.trim().is_empty()
        && value.chars().count() <= max_chars
        && !value.chars().any(char::is_control)
}

fn bounded_message(value: &str) -> bool {
    !value.trim().is_empty()
        && value.chars().count() <= MAX_MESSAGE_CHARS
        && !value
            .chars()
            .any(|character| character.is_control() && !matches!(character, '\n' | '\r' | '\t'))
}

fn valid_create(request: &CreateConversationRequest) -> bool {
    uuid::Uuid::parse_str(&request.topic_id).is_ok_and(|id| id.to_string() == request.topic_id)
        && bounded_text(&request.subject, 255)
        && bounded_message(&request.message)
}

async fn verified_identity(
    state: &AppState,
    headers: &axum::http::HeaderMap,
) -> Result<(String, SessionUser), Response> {
    crate::auth::verified_access_token(headers, state.verifier.as_ref(), state.cookie_environment)
        .await
        .ok_or_else(|| chat_error(StatusCode::UNAUTHORIZED, "invalid_access_token"))
}

fn same_origin(headers: &axum::http::HeaderMap) -> bool {
    let Some(origin) = headers
        .get(header::ORIGIN)
        .and_then(|value| value.to_str().ok())
    else {
        return false;
    };
    let Some(host) = headers
        .get(header::HOST)
        .and_then(|value| value.to_str().ok())
    else {
        return false;
    };
    let origin_host = origin
        .strip_prefix("https://")
        .or_else(|| origin.strip_prefix("http://"))
        .and_then(|value| value.split('/').next())
        .unwrap_or("");
    origin_host == host
        && !origin_host.is_empty()
        && headers
            .get("sec-fetch-site")
            .and_then(|value| value.to_str().ok())
            .is_none_or(|value| matches!(value, "same-origin" | "same-site"))
}

fn chat_private(mut response: Response) -> Response {
    response.headers_mut().insert(
        header::CACHE_CONTROL,
        HeaderValue::from_static("private, no-store"),
    );
    response.headers_mut().insert(
        header::VARY,
        HeaderValue::from_static("Cookie, Authorization"),
    );
    response
}

fn chat_error(status: StatusCode, code: &'static str) -> Response {
    chat_private(
        (
            status,
            Json(serde_json::json!({"success": false, "error": code})),
        )
            .into_response(),
    )
}

fn chat_success<T: Serialize>(value: T) -> Response {
    chat_private(
        Json(serde_json::json!({
            "success": true,
            "data": value,
            "error": null
        }))
        .into_response(),
    )
}

pub(crate) async fn chat_inbox_api(
    State(state): State<AppState>,
    headers: axum::http::HeaderMap,
) -> Response {
    let (token, user) = match verified_identity(&state, &headers).await {
        Ok(identity) => identity,
        Err(response) => return response,
    };
    match load_chat_inbox_for_ssr(state.wallet.as_ref(), &token, &user.wallet_address).await {
        Ok(inbox) => chat_success(inbox),
        Err(error) => load_error_response(error),
    }
}

pub(crate) async fn chat_detail_api(
    State(state): State<AppState>,
    Path(id): Path<uuid::Uuid>,
    headers: axum::http::HeaderMap,
) -> Response {
    let (token, user) = match verified_identity(&state, &headers).await {
        Ok(identity) => identity,
        Err(response) => return response,
    };
    match load_chat_detail_for_ssr(state.wallet.as_ref(), &token, &user.wallet_address, id).await {
        Ok(detail) => chat_success(detail),
        Err(error) => load_error_response(error),
    }
}

fn load_error_response(error: ChatLoadError) -> Response {
    match error {
        ChatLoadError::Forbidden => chat_error(StatusCode::FORBIDDEN, "chat_forbidden"),
        ChatLoadError::Unavailable => {
            chat_error(StatusCode::SERVICE_UNAVAILABLE, "chat_upstream_unavailable")
        }
        ChatLoadError::Malformed => chat_error(StatusCode::BAD_GATEWAY, "chat_upstream_malformed"),
    }
}

async fn parse_json<T: DeserializeOwned>(body: Body) -> Result<T, Response> {
    let body = axum::body::to_bytes(body, MAX_REQUEST_BYTES)
        .await
        .map_err(|_| chat_error(StatusCode::PAYLOAD_TOO_LARGE, "chat_request_too_large"))?;
    serde_json::from_slice(&body)
        .map_err(|_| chat_error(StatusCode::BAD_REQUEST, "invalid_chat_request"))
}

async fn create_upstream(
    state: &AppState,
    token: &str,
    owner: &str,
    request: &CreateConversationRequest,
) -> Result<ChatConversation, ChatLoadError> {
    let response = state
        .wallet
        .auth_client()
        .post(format!(
            "{}{}",
            state.wallet.base_url().trim_end_matches('/'),
            CHAT_CONVERSATIONS_PATH
        ))
        .bearer_auth(token)
        .json(request)
        .send()
        .await
        .map_err(|_| ChatLoadError::Unavailable)?;
    let conversation = decode_response::<UpstreamConversation>(response).await?;
    let projection = project_conversation(conversation, owner).ok_or(ChatLoadError::Malformed)?;
    // Validate the conversation through the detail projection because an inbox
    // requires topic rows that are intentionally not part of this mutation.
    let detail = ChatDetailData {
        conversation: projection.clone(),
        messages: Vec::new(),
    };
    serde_json::to_value(detail)
        .ok()
        .and_then(decode_chat_detail)
        .map(|detail| detail.conversation)
        .ok_or(ChatLoadError::Malformed)
}

async fn send_upstream(
    state: &AppState,
    token: &str,
    id: uuid::Uuid,
    request: &SendMessageRequest,
) -> Result<ChatMessage, ChatLoadError> {
    let response = state
        .wallet
        .auth_client()
        .post(format!(
            "{}{CHAT_CONVERSATIONS_PATH}/{id}/messages",
            state.wallet.base_url().trim_end_matches('/')
        ))
        .bearer_auth(token)
        .json(request)
        .send()
        .await
        .map_err(|_| ChatLoadError::Unavailable)?;
    let message = project_message(decode_response::<UpstreamMessage>(response).await?)
        .ok_or(ChatLoadError::Malformed)?;
    (message.conversation_id == id.to_string())
        .then_some(message)
        .ok_or(ChatLoadError::Malformed)
}

async fn resolve_upstream(
    state: &AppState,
    token: &str,
    owner: &str,
    id: uuid::Uuid,
) -> Result<ChatConversation, ChatLoadError> {
    let response = state
        .wallet
        .auth_client()
        .put(format!(
            "{}{CHAT_CONVERSATIONS_PATH}/{id}/status",
            state.wallet.base_url().trim_end_matches('/')
        ))
        .bearer_auth(token)
        .json(&UpdateStatusRequest { status: "resolved" })
        .send()
        .await
        .map_err(|_| ChatLoadError::Unavailable)?;
    let conversation = project_conversation(
        decode_response::<UpstreamConversation>(response).await?,
        owner,
    )
    .ok_or(ChatLoadError::Malformed)?;
    (conversation.id == id.to_string() && conversation.status == "resolved")
        .then_some(conversation)
        .ok_or(ChatLoadError::Malformed)
}

pub(crate) async fn chat_create_api(State(state): State<AppState>, request: Request) -> Response {
    let (parts, body) = request.into_parts();
    if !same_origin(&parts.headers) {
        return chat_error(StatusCode::FORBIDDEN, "chat_mutation_origin_rejected");
    }
    let (token, user) = match verified_identity(&state, &parts.headers).await {
        Ok(identity) => identity,
        Err(response) => return response,
    };
    let body = match parse_json::<CreateConversationRequest>(body).await {
        Ok(body) if valid_create(&body) => body,
        Ok(_) => return chat_error(StatusCode::BAD_REQUEST, "invalid_chat_request"),
        Err(response) => return response,
    };
    match create_upstream(&state, &token, &user.wallet_address, &body).await {
        Ok(conversation) => chat_success(conversation),
        Err(error) => load_error_response(error),
    }
}

pub(crate) async fn chat_send_api(
    State(state): State<AppState>,
    Path(id): Path<uuid::Uuid>,
    request: Request,
) -> Response {
    let (parts, body) = request.into_parts();
    if !same_origin(&parts.headers) {
        return chat_error(StatusCode::FORBIDDEN, "chat_mutation_origin_rejected");
    }
    let (token, _) = match verified_identity(&state, &parts.headers).await {
        Ok(identity) => identity,
        Err(response) => return response,
    };
    let body = match parse_json::<SendMessageRequest>(body).await {
        Ok(body) if bounded_message(&body.content) => body,
        Ok(_) => return chat_error(StatusCode::BAD_REQUEST, "invalid_chat_request"),
        Err(response) => return response,
    };
    match send_upstream(&state, &token, id, &body).await {
        Ok(message) => chat_success(message),
        Err(error) => load_error_response(error),
    }
}

pub(crate) async fn chat_upload_api(
    State(state): State<AppState>,
    Path(id): Path<uuid::Uuid>,
    headers: axum::http::HeaderMap,
    mut multipart: Multipart,
) -> Response {
    if !same_origin(&headers) {
        return chat_error(StatusCode::FORBIDDEN, "chat_mutation_origin_rejected");
    }
    let (token, _user) = match verified_identity(&state, &headers).await {
        Ok(identity) => identity,
        Err(response) => return response,
    };
    let field = match multipart.next_field().await {
        Ok(Some(field)) => field,
        Ok(None) => return chat_error(StatusCode::BAD_REQUEST, "invalid_chat_request"),
        Err(_) => return chat_error(StatusCode::BAD_REQUEST, "invalid_chat_request"),
    };
    let filename = field
        .file_name()
        .map(str::to_string)
        .unwrap_or_else(|| "file".to_string());
    let content_type = field.content_type().map(|v| v.to_string());
    let bytes = match field.bytes().await {
        Ok(bytes) => bytes,
        Err(_) => return chat_error(StatusCode::BAD_REQUEST, "invalid_chat_request"),
    };
    if bytes.is_empty() || bytes.len() > 5 * 1024 * 1024 {
        return chat_error(StatusCode::PAYLOAD_TOO_LARGE, "chat_request_too_large");
    }
    let lower = filename.to_ascii_lowercase();
    let ext = lower.rsplit('.').next().unwrap_or_default();
    if !matches!(ext, "jpg" | "jpeg" | "png" | "gif" | "webp" | "pdf") {
        return chat_error(StatusCode::BAD_REQUEST, "invalid_chat_request");
    }
    let mime = content_type.unwrap_or_else(|| match ext {
        "jpg" | "jpeg" => "image/jpeg".to_string(),
        "png" => "image/png".to_string(),
        "gif" => "image/gif".to_string(),
        "webp" => "image/webp".to_string(),
        "pdf" => "application/pdf".to_string(),
        _ => "application/octet-stream".to_string(),
    });
    let part = match reqwest::multipart::Part::bytes(bytes.to_vec())
        .file_name(filename.clone())
        .mime_str(&mime)
    {
        Ok(part) => part,
        Err(_) => {
            return chat_error(StatusCode::BAD_REQUEST, "invalid_chat_request");
        }
    };
    let form = reqwest::multipart::Form::new().part("file", part);
    let url = format!(
        "{}/api/chat/conversations/{}/upload",
        state.wallet.base_url().trim_end_matches('/'),
        id
    );
    let response = match state
        .wallet
        .auth_client()
        .post(&url)
        .bearer_auth(token)
        .multipart(form)
        .send()
        .await
    {
        Ok(resp) => resp,
        Err(_) => {
            return chat_error(StatusCode::SERVICE_UNAVAILABLE, "chat_upstream_unavailable");
        }
    };
    match decode_response::<serde_json::Value>(response).await {
        Ok(value) => chat_success(value),
        Err(error) => load_error_response(error),
    }
}

fn parse_form(body: &[u8]) -> Result<std::collections::BTreeMap<String, String>, ()> {
    let mut fields = std::collections::BTreeMap::new();
    for (key, value) in url::form_urlencoded::parse(body) {
        if fields
            .insert(key.into_owned(), value.into_owned())
            .is_some()
        {
            return Err(());
        }
    }
    Ok(fields)
}

async fn read_form(body: Body) -> Result<std::collections::BTreeMap<String, String>, Response> {
    let body = axum::body::to_bytes(body, MAX_REQUEST_BYTES)
        .await
        .map_err(|_| chat_error(StatusCode::PAYLOAD_TOO_LARGE, "chat_request_too_large"))?;
    parse_form(&body).map_err(|_| chat_error(StatusCode::BAD_REQUEST, "invalid_chat_request"))
}

fn redirect(location: &str) -> Response {
    chat_private(Redirect::to(location).into_response())
}

pub(crate) async fn chat_create_form(State(state): State<AppState>, request: Request) -> Response {
    let (parts, body) = request.into_parts();
    if !same_origin(&parts.headers) {
        return chat_error(StatusCode::FORBIDDEN, "chat_mutation_origin_rejected");
    }
    let (token, user) = match verified_identity(&state, &parts.headers).await {
        Ok(identity) => identity,
        Err(response) => return response,
    };
    let mut fields = match read_form(body).await {
        Ok(fields) => fields,
        Err(response) => return response,
    };
    let request = CreateConversationRequest {
        topic_id: fields.remove("topic_id").unwrap_or_default(),
        subject: fields.remove("subject").unwrap_or_default(),
        message: fields.remove("message").unwrap_or_default(),
    };
    if !fields.is_empty() || !valid_create(&request) {
        return redirect("/chat?new=1&chat=error");
    }
    match create_upstream(&state, &token, &user.wallet_address, &request).await {
        Ok(conversation) => redirect(&format!("/chat/{}?chat=created", conversation.id)),
        Err(_) => redirect("/chat?new=1&chat=error"),
    }
}

pub(crate) async fn chat_conversation_form(
    State(state): State<AppState>,
    Path(id): Path<uuid::Uuid>,
    request: Request,
) -> Response {
    let (parts, body) = request.into_parts();
    if !same_origin(&parts.headers) {
        return chat_error(StatusCode::FORBIDDEN, "chat_mutation_origin_rejected");
    }
    let (token, user) = match verified_identity(&state, &parts.headers).await {
        Ok(identity) => identity,
        Err(response) => return response,
    };
    let mut fields = match read_form(body).await {
        Ok(fields) => fields,
        Err(response) => return response,
    };
    let operation = fields.remove("operation").unwrap_or_default();
    let location = format!("/chat/{id}");
    match operation.as_str() {
        "send" => {
            let content = fields.remove("content").unwrap_or_default();
            if !fields.is_empty() || !bounded_message(&content) {
                return redirect(&format!("{location}?chat=error"));
            }
            let request = SendMessageRequest { content };
            match send_upstream(&state, &token, id, &request).await {
                Ok(_) => redirect(&format!("{location}?chat=sent")),
                Err(_) => redirect(&format!("{location}?chat=error")),
            }
        }
        "resolve" if fields.is_empty() => {
            match resolve_upstream(&state, &token, &user.wallet_address, id).await {
                Ok(_) => redirect(&format!("{location}?chat=resolved")),
                Err(_) => redirect(&format!("{location}?chat=error")),
            }
        }
        _ => redirect(&format!("{location}?chat=error")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_contract_rejects_owner_and_status_fields() {
        let mut value = serde_json::json!({
            "topic_id": "17ea1b05-5ec1-4b6e-9e5a-13751ec2ed6d",
            "subject": "Local test",
            "message": "Hello"
        });
        value["wallet_address"] = serde_json::json!("attacker");
        assert!(serde_json::from_value::<CreateConversationRequest>(value).is_err());
    }

    #[test]
    fn forms_reject_duplicate_and_unknown_authority_fields() {
        assert!(parse_form(b"operation=send&operation=resolve").is_err());
        let fields = parse_form(b"operation=send&owner=other").unwrap();
        assert!(fields.contains_key("owner"));
    }

    #[test]
    fn multiline_messages_are_bounded() {
        assert!(bounded_message("hello\nworld"));
        assert!(!bounded_message("hello\u{0000}world"));
        assert!(!bounded_message(""));
    }
}
