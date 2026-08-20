use ammonia::{Builder as AmmoniaBuilder, UrlRelative};
use axum::{
    body::Bytes,
    extract::{Extension, Path as AxPath, RawQuery, State},
    http::{HeaderMap, StatusCode},
    response::{
        sse::{Event, KeepAlive, Sse},
        IntoResponse, Response,
    },
    routing::{delete, get, post, put},
    Json, Router,
};
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};
use clap::{Parser, ValueEnum};
use epsx_notification::{
    build_auth_verifier, canonical_owner, delivery::DeliveryWorker, protect_router,
    verify_lifecycle_schema_compatibility, verify_schema_compatibility,
    NOTIFICATIONS_MANAGE_PERMISSION,
};
use epsx_service_auth::{VerifiedPrincipal, ADMIN_AUDIENCE};
use futures::StreamExt;
use handlebars::Handlebars;
use hmac::{Hmac, Mac};
use lettre::{transport::smtp::authentication::Credentials, Message, SmtpTransport, Transport};
use redis::AsyncCommands;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use sqlx::{postgres::PgPoolOptions, FromRow, Postgres, Transaction};
use std::collections::{BTreeMap, HashMap, HashSet};
use std::convert::Infallible;
use std::net::SocketAddr;
use std::sync::{
    atomic::{AtomicU64, Ordering},
    Arc,
};
use thiserror::Error;
use tokio::sync::{Notify, RwLock};
use tracing::info;
use uuid::Uuid;
use web_push::{
    ContentEncoding, SubscriptionInfo, Urgency, VapidSignatureBuilder, WebPushError,
    WebPushMessageBuilder,
};

const MAX_REALTIME_CONNECTIONS: usize = 256;
const SMTP_TRANSPORT_TIMEOUT_SECONDS: u64 = 30;
const MAX_PLAN_FANOUT: usize = 10_000;
const PLAN_DB_READ_ONLY_SESSION_SQL: &str = "SET default_transaction_read_only = on";

#[derive(Parser)]
#[command(name = "epsx-notification", about = "EPSX Notification Service")]
struct Args {
    #[arg(long, env = "PORT", default_value = "8106")]
    port: u16,
    #[arg(long, env = "HOST", default_value = "0.0.0.0")]
    host: String,
    #[arg(
        long,
        env = "DATABASE_URL",
        default_value = "postgres://epsx:epsx@localhost:5432/epsx_notification"
    )]
    database_url: String,
    #[arg(long, env = "SMTP_HOST", default_value = "")]
    smtp_host: String,
    #[arg(long, env = "SMTP_PORT", default_value = "587")]
    smtp_port: u16,
    #[arg(long, env = "SMTP_USER", default_value = "")]
    smtp_user: String,
    #[arg(long, env = "SMTP_PASSWORD", default_value = "")]
    smtp_password: String,
    #[arg(long, env = "FROM_ADDRESS", default_value = "noreply@epsx.io")]
    from_address: String,
    #[arg(long, env = "FROM_NAME", default_value = "EPSX")]
    from_name: String,
    #[arg(long, env = "REDIS_URL")]
    redis_url: Option<String>,
    /// Read-only core database used only for server-side plan audience
    /// expansion. It is intentionally separate from the notification store.
    #[arg(long, env = "NOTIFICATION_PLAN_DATABASE_URL")]
    plan_database_url: Option<String>,
    #[arg(long, env = "OIDC_ISSUER")]
    oidc_issuer: String,
    #[arg(long, env = "OIDC_JWKS_URL")]
    jwks_url: Option<String>,
    #[arg(long, env = "EPSX_ENV", value_enum, default_value = "development")]
    environment: Environment,
}

#[derive(Clone, Copy, Debug, ValueEnum)]
enum Environment {
    Development,
    Staging,
    Production,
}

/// Connect the optional plan resolver with a database-session guard rather
/// than relying only on the deployment role's grants. Every pooled connection
/// defaults its transactions to read-only before it can be checked out by the
/// resolver, so a future query regression cannot mutate core tables through
/// this boundary.
async fn connect_read_only_plan_pool(url: &str) -> Result<sqlx::PgPool, sqlx::Error> {
    PgPoolOptions::new()
        .after_connect(|connection, _meta| {
            Box::pin(async move {
                sqlx::query(PLAN_DB_READ_ONLY_SESSION_SQL)
                    .execute(&mut *connection)
                    .await?;
                Ok(())
            })
        })
        .connect(url)
        .await
}

#[derive(Clone)]
struct AppState {
    db: sqlx::PgPool,
    plan_db: Option<sqlx::PgPool>,
    templates: Arc<RwLock<Handlebars<'static>>>,
    smtp: Arc<RwLock<Option<SmtpTransport>>>,
    provider_signing_secrets: Vec<Arc<Vec<u8>>>,
    realtime_slots: Arc<tokio::sync::Semaphore>,
    stream_metrics: Arc<StreamMetrics>,
    vapid_key_id: String,
    vapid_public_key: Option<String>,
    vapid_private_key: Option<Arc<Vec<u8>>>,
    vapid_previous_key_id: Option<String>,
    vapid_previous_private_key: Option<Arc<Vec<u8>>>,
    from: String,
    from_name: String,
    redis: Option<redis::Client>,
    realtime_notify: Arc<Notify>,
}

#[derive(Default)]
struct StreamMetrics {
    connections_total: AtomicU64,
    reconnects_total: AtomicU64,
    replayed_events_total: AtomicU64,
    lag_seconds_total: AtomicU64,
    lag_samples_total: AtomicU64,
    query_failures_total: AtomicU64,
}

#[derive(Serialize, Deserialize, FromRow, Clone)]
struct Template {
    id: String,
    name: String,
    channel: String,
    subject: Option<String>,
    body: String,
    variables: serde_json::Value,
    active: bool,
    created_at: chrono::DateTime<chrono::Utc>,
    updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Serialize, Deserialize, FromRow, Clone)]
struct Notification {
    id: String,
    user_id: Option<String>,
    channel: String,
    recipient: String,
    template_id: Option<String>,
    subject: Option<String>,
    body: String,
    data: Option<serde_json::Value>,
    status: String,
    error: Option<String>,
    sent_at: Option<chrono::DateTime<chrono::Utc>>,
    created_at: chrono::DateTime<chrono::Utc>,
    read_at: Option<chrono::DateTime<chrono::Utc>>,
    clicked_at: Option<chrono::DateTime<chrono::Utc>>,
    title: Option<String>,
    notification_type: Option<String>,
    priority: Option<String>,
    action_url: Option<String>,
    expires_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(deny_unknown_fields)]
struct CreateTemplateRequest {
    name: String,
    channel: String,
    subject: Option<String>,
    body: String,
    variables: serde_json::Value,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct TemplateRollbackRequest {
    version: i32,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct TemplatePreviewRequest {
    #[serde(default)]
    data: serde_json::Value,
}

#[derive(Serialize)]
struct TemplatePreviewResponse {
    template_id: String,
    version: i32,
    subject: Option<String>,
    body: String,
}

#[derive(Serialize)]
struct TemplateAuditResponse {
    items: Vec<TemplateAuditEntry>,
}

#[derive(Serialize, FromRow)]
struct TemplateAuditEntry {
    id: String,
    template_id: String,
    action: String,
    from_version: Option<i32>,
    to_version: Option<i32>,
    actor_subject: String,
    metadata: serde_json::Value,
    created_at: chrono::DateTime<chrono::Utc>,
}

const TEMPLATE_AUDIT_MAX_ITEMS: usize = 100;

fn valid_template_audit_text(value: &str, max_chars: usize) -> bool {
    !value.is_empty() && value.chars().count() <= max_chars && !value.chars().any(char::is_control)
}

fn valid_template_audit_metadata(action: &str, metadata: &serde_json::Value) -> bool {
    let Some(object) = metadata.as_object() else {
        return false;
    };
    match action {
        "created" | "updated" | "deleted" => {
            object.len() == 1
                && object
                    .get("template_name")
                    .and_then(serde_json::Value::as_str)
                    .is_some_and(|name| valid_template_audit_text(name, 100))
        }
        "rollback" => {
            object.len() == 2
                && object
                    .get("restored_version")
                    .and_then(serde_json::Value::as_i64)
                    .is_some_and(|version| version > 0)
                && object
                    .get("new_version")
                    .and_then(serde_json::Value::as_i64)
                    .is_some_and(|version| version > 0)
        }
        _ => false,
    }
}

fn valid_template_audit_entry(entry: &TemplateAuditEntry) -> bool {
    valid_template_audit_text(&entry.id, 128)
        && valid_template_audit_text(&entry.template_id, 66)
        && matches!(
            entry.action.as_str(),
            "created" | "updated" | "deleted" | "rollback"
        )
        && entry.from_version.is_none_or(|version| version > 0)
        && entry.to_version.is_none_or(|version| version > 0)
        && valid_template_audit_text(&entry.actor_subject, 255)
        && serde_json::to_vec(&entry.metadata).is_ok_and(|bytes| bytes.len() <= 16 * 1024)
        && valid_template_audit_metadata(&entry.action, &entry.metadata)
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct SendNotificationRequest {
    user_id: Option<String>,
    channel: String,
    recipient: String,
    template_id: Option<String>,
    subject: Option<String>,
    body: Option<String>,
    data: Option<serde_json::Value>,
    #[serde(default)]
    expires_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(deny_unknown_fields)]
struct PublishNotificationRequest {
    event_id: String,
    event_type: String,
    aggregate_id: String,
    idempotency_key: String,
    recipient_wallet_address: String,
    notification_type: String,
    priority: String,
    title: String,
    message: String,
    #[serde(default)]
    data: Option<serde_json::Value>,
    #[serde(default)]
    action_url: Option<String>,
    #[serde(default)]
    expires_at: Option<chrono::DateTime<chrono::Utc>>,
    /// Optional server-resolved plan audience. When present, the caller must
    /// set `recipient_wallet_address` to `all`; the service expands active
    /// memberships to concrete wallet rows inside the publish transaction.
    #[serde(default)]
    plan_id: Option<Uuid>,
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct ProviderEventRequest {
    provider: String,
    provider_event_id: String,
    #[serde(default)]
    provider_message_id: Option<String>,
    #[serde(default)]
    job_id: Option<String>,
    event_type: String,
    payload: serde_json::Value,
    #[serde(default)]
    occurred_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Serialize)]
struct ProviderEventResponse {
    provider: String,
    provider_event_id: String,
    status: &'static str,
}

#[derive(FromRow)]
struct IdempotencyRecord {
    request_hash: String,
    response_status: i16,
    response_body: serde_json::Value,
}

#[derive(FromRow)]
struct InboxRecord {
    request_hash: String,
    state: String,
}

#[derive(Serialize, Deserialize)]
struct SendNotificationResponse {
    id: String,
    status: String,
    delivered: bool,
    error: Option<String>,
    request_id: String,
}

const MAX_IDEMPOTENCY_KEY_CHARS: usize = 56;
const MAX_RECIPIENT_CHARS: usize = 255;
const MAX_SUBJECT_CHARS: usize = 255;
const MAX_BODY_CHARS: usize = 16_384;
const MAX_DATA_BYTES: usize = 32 * 1024;

fn bounded_control_free(value: &str, max_chars: usize) -> bool {
    !value.trim().is_empty()
        && value.chars().count() <= max_chars
        && !value.chars().any(char::is_control)
}

fn valid_idempotency_key(value: &str) -> bool {
    (1..=MAX_IDEMPOTENCY_KEY_CHARS).contains(&value.chars().count())
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'_' | b'-'))
}

fn request_id(headers: &HeaderMap) -> String {
    headers
        .get("x-request-id")
        .and_then(|value| value.to_str().ok())
        .filter(|value| bounded_control_free(value, 128))
        .map(str::to_string)
        .unwrap_or_else(|| Uuid::new_v4().to_string())
}

fn idempotent_notification_id(key: &str) -> String {
    format!("idem_{key}")
}

#[derive(FromRow)]
struct SendNotificationRecord {
    id: String,
    user_id: Option<String>,
    channel: String,
    recipient: String,
    template_id: Option<String>,
    subject: Option<String>,
    body: String,
    data: Option<serde_json::Value>,
    status: String,
    error: Option<String>,
    expires_at: Option<chrono::DateTime<chrono::Utc>>,
}

fn send_response_status(status: &str) -> (&'static str, bool) {
    match status {
        "sent" => ("sent", true),
        "pending" | "queued" => ("pending", false),
        _ => ("failed", false),
    }
}

fn response_from_existing(
    notification: &SendNotificationRecord,
    request_id: String,
) -> SendNotificationResponse {
    let (status, delivered) = send_response_status(&notification.status);
    SendNotificationResponse {
        id: notification.id.clone(),
        status: status.to_string(),
        delivered,
        error: notification.error.clone(),
        request_id,
    }
}

fn same_send_request(
    notification: &SendNotificationRecord,
    request: &SendNotificationRequest,
    subject: &str,
    body: &str,
) -> bool {
    notification.user_id == request.user_id
        && notification.channel == request.channel
        && notification.recipient == request.recipient
        && notification.template_id == request.template_id
        && notification.subject.as_deref().unwrap_or_default() == subject
        && notification.body == body
        && notification.data == request.data
        && notification.expires_at == request.expires_at
}

fn require_admin_notifications(principal: &VerifiedPrincipal) -> Result<(), StatusCode> {
    if principal.audience != ADMIN_AUDIENCE
        || !principal.has_permission(NOTIFICATIONS_MANAGE_PERMISSION)
    {
        return Err(StatusCode::FORBIDDEN);
    }
    Ok(())
}

#[derive(Serialize, Deserialize)]
struct NotificationListResponse {
    items: Vec<Notification>,
    total: i64,
}

#[derive(Serialize)]
struct NotificationMetricsResponse {
    queue_depth: i64,
    queue_age_seconds: Option<i64>,
    suppressed: i64,
    retry_wait: i64,
    terminal_failed: i64,
    dead_lettered: i64,
    provider_accepted: i64,
    attempting: i64,
    channel_outcomes: BTreeMap<String, i64>,
    provider_events: i64,
    delivery_attempts: i64,
    replay_cursors: i64,
    replay_cursor_age_seconds: Option<i64>,
    active_streams: usize,
    stream_connections_total: u64,
    stream_reconnects_total: u64,
    stream_replayed_events_total: u64,
    stream_lag_seconds: Option<u64>,
    stream_query_failures_total: u64,
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(deny_unknown_fields)]
struct NotificationPreferencesRequest {
    channels: serde_json::Value,
    #[serde(default)]
    quiet_hours: Option<serde_json::Value>,
    #[serde(default)]
    timezone: Option<String>,
}

#[derive(Serialize, FromRow)]
struct NotificationPreferencesResponse {
    channels: serde_json::Value,
    quiet_hours: Option<serde_json::Value>,
    timezone: Option<String>,
    updated_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(FromRow)]
struct DeliveryPreferencePolicy {
    channels: serde_json::Value,
    quiet_until: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct AdminNotificationQuery {
    limit: i64,
    offset: i64,
    status: Option<String>,
    notification_type: Option<String>,
    priority: Option<String>,
    wallet_address: Option<String>,
}

impl Default for AdminNotificationQuery {
    fn default() -> Self {
        Self {
            limit: 20,
            offset: 0,
            status: None,
            notification_type: None,
            priority: None,
            wallet_address: None,
        }
    }
}

impl AdminNotificationQuery {
    fn parse(raw: Option<&str>) -> Result<Self, StatusCode> {
        let Some(raw) = raw.filter(|raw| !raw.is_empty()) else {
            return Ok(Self::default());
        };

        let mut query = Self::default();
        let mut limit_seen = false;
        let mut offset_seen = false;
        let mut status_seen = false;
        let mut type_seen = false;
        let mut notification_type_seen = false;
        let mut priority_seen = false;
        let mut wallet_seen = false;

        for pair in raw.split('&') {
            let (key, value) = pair.split_once('=').ok_or(StatusCode::BAD_REQUEST)?;
            match key {
                "limit" if !limit_seen => {
                    let value = parse_admin_decimal(value, 1..=50)?;
                    query.limit = value;
                    limit_seen = true;
                }
                "offset" if !offset_seen => {
                    let value = parse_admin_decimal(value, 0..=1_000_000)?;
                    query.offset = value;
                    offset_seen = true;
                }
                "status" if !status_seen => {
                    if !matches!(
                        value,
                        "all"
                            | "read"
                            | "unread"
                            | "pending"
                            | "sent"
                            | "failed"
                            | "suppressed"
                            | "expired"
                    ) {
                        return Err(StatusCode::BAD_REQUEST);
                    }
                    query.status = Some(value.to_string());
                    status_seen = true;
                }
                "type" if !type_seen && !notification_type_seen => {
                    if !safe_lower_ascii_token(value, 50) {
                        return Err(StatusCode::BAD_REQUEST);
                    }
                    query.notification_type = Some(value.to_string());
                    type_seen = true;
                }
                "notification_type" if !type_seen && !notification_type_seen => {
                    if !safe_lower_ascii_token(value, 50) {
                        return Err(StatusCode::BAD_REQUEST);
                    }
                    query.notification_type = Some(value.to_string());
                    notification_type_seen = true;
                }
                "priority" if !priority_seen => {
                    if !matches!(value, "low" | "normal" | "high" | "critical" | "urgent") {
                        return Err(StatusCode::BAD_REQUEST);
                    }
                    query.priority = Some(value.to_string());
                    priority_seen = true;
                }
                "wallet_address" if !wallet_seen => {
                    let normalized = value.to_ascii_lowercase();
                    if !valid_wallet_address(&normalized) {
                        return Err(StatusCode::BAD_REQUEST);
                    }
                    query.wallet_address = Some(normalized);
                    wallet_seen = true;
                }
                _ => return Err(StatusCode::BAD_REQUEST),
            }
        }

        Ok(query)
    }
}

fn parse_admin_decimal(
    value: &str,
    range: std::ops::RangeInclusive<i64>,
) -> Result<i64, StatusCode> {
    if value.is_empty() || !value.bytes().all(|byte| byte.is_ascii_digit()) {
        return Err(StatusCode::BAD_REQUEST);
    }
    let value = value.parse::<i64>().map_err(|_| StatusCode::BAD_REQUEST)?;
    range
        .contains(&value)
        .then_some(value)
        .ok_or(StatusCode::BAD_REQUEST)
}

#[derive(Debug, FromRow)]
struct AdminNotificationRow {
    id: String,
    title: Option<String>,
    subject: Option<String>,
    channel: String,
    status: String,
    notification_type: Option<String>,
    priority: Option<String>,
    sent_at: Option<chrono::DateTime<chrono::Utc>>,
    created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
struct AdminNotificationItem {
    id: String,
    title: Option<String>,
    subject: Option<String>,
    channel: String,
    status: String,
    notification_type: Option<String>,
    priority: Option<String>,
    sent_at: Option<chrono::DateTime<chrono::Utc>>,
    created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
struct AdminNotificationListResponse {
    items: Vec<AdminNotificationItem>,
    total: i64,
    limit: i64,
    offset: i64,
}

const ADMIN_NOTIFICATION_LIST_SQL: &str = concat!(
    "SELECT id, title, subject, channel, CASE WHEN n.read_",
    "at IS NOT NULL OR EXISTS (SELECT 1 FROM public.notification_engagement projection_engagement WHERE projection_engagement.notification_id = n.id AND projection_engagement.read_",
    "at IS NOT NULL) THEN 'read' ELSE status END AS status, notification_type, priority, sent_at, created_at FROM public.notifications n WHERE ($3::text IS NULL OR $3 = 'all' OR ($3 = 'read' AND (n.read_",
    "at IS NOT NULL OR EXISTS (SELECT 1 FROM public.notification_engagement filter_engagement WHERE filter_engagement.notification_id = n.id AND filter_engagement.",
    "read_",
    "at IS NOT NULL))) OR ($3 = 'unread' AND n.read_",
    "at IS NULL AND NOT EXISTS (SELECT 1 FROM public.notification_engagement filter_engagement WHERE filter_engagement.notification_id = n.id AND filter_engagement.",
    "read_",
    "at IS NOT NULL)) OR ($3 NOT IN ('all', 'read', 'unread') AND status = $3)) AND ($4::text IS NULL OR notification_type = $4) AND ($5::text IS NULL OR priority = $5) AND ($6::text IS NULL OR lower(user_",
    "id) = $6 OR (user_",
    "id IS NULL AND lower(recip",
    "ient) = $6)) ORDER BY created_at DESC, id DESC LIMIT $1 OFFSET $2"
);
const ADMIN_NOTIFICATION_COUNT_SQL: &str = concat!(
    "SELECT COUNT(*) FROM public.notifications n WHERE ($1::text IS NULL OR $1 = 'all' OR ($1 = 'read' AND (n.read_",
    "at IS NOT NULL OR EXISTS (SELECT 1 FROM public.notification_engagement filter_engagement WHERE filter_engagement.notification_id = n.id AND filter_engagement.",
    "read_",
    "at IS NOT NULL))) OR ($1 = 'unread' AND n.read_",
    "at IS NULL AND NOT EXISTS (SELECT 1 FROM public.notification_engagement filter_engagement WHERE filter_engagement.notification_id = n.id AND filter_engagement.",
    "read_",
    "at IS NOT NULL)) OR ($1 NOT IN ('all', 'read', 'unread') AND status = $1)) AND ($2::text IS NULL OR notification_type = $2) AND ($3::text IS NULL OR priority = $3) AND ($4::text IS NULL OR lower(user_",
    "id) = $4 OR (user_",
    "id IS NULL AND lower(recip",
    "ient) = $4))"
);
const BROADCAST_NOTIFICATION_INSERT_SQL: &str = "INSERT INTO public.notifications (id, user_id, channel, recipient, subject, body, data, status, error, title, notification_type, priority, action_url) VALUES ($1, NULL, 'in_app', 'all', NULL, $2, $3, 'pending', NULL, $4, $5, $6, $7)";

const OWNER_NOTIFICATION_SCOPE_SQL: &str =
    "(n.user_id = $1 OR (n.user_id IS NULL AND lower(n.recipient) = 'all')) AND (x.expires_at IS NULL OR x.expires_at > NOW())";
const OWNER_NOTIFICATION_SELECT_FIELDS: &str =
    "n.id, n.user_id, n.channel, n.recipient, n.template_id, n.subject, n.body, n.data, n.status, n.error, n.sent_at, n.created_at, COALESCE(e.read_at, n.read_at) AS read_at, e.clicked_at, n.title, n.notification_type, n.priority, n.action_url, x.expires_at";
const OWNER_NOTIFICATION_JOIN: &str =
    "FROM public.notifications n LEFT JOIN public.notification_engagement e ON e.notification_id = n.id AND e.owner_id = $1 LEFT JOIN public.notification_expirations x ON x.notification_id = n.id";
const OWNER_NOTIFICATION_FILTER_SQL: &str = "AND ($2::text IS NULL OR (($2 IN ('read', 'unread') AND (($2 = 'read' AND COALESCE(e.read_at, n.read_at) IS NOT NULL) OR ($2 = 'unread' AND COALESCE(e.read_at, n.read_at) IS NULL))) OR ($2 NOT IN ('read', 'unread') AND n.status = $2))) AND ($3::text IS NULL OR n.notification_type = $3) AND ($4::text IS NULL OR n.priority = $4) AND ($5::timestamptz IS NULL OR n.created_at >= $5) AND ($6::timestamptz IS NULL OR n.created_at <= $6)";

fn control_free_with_max(value: &str, max_chars: usize) -> bool {
    value.chars().count() <= max_chars && !value.chars().any(char::is_control)
}

fn safe_lower_ascii_token(value: &str, max_bytes: usize) -> bool {
    !value.is_empty()
        && value.len() <= max_bytes
        && value.bytes().all(|byte| {
            byte.is_ascii_lowercase() || byte.is_ascii_digit() || matches!(byte, b'-' | b'_')
        })
}

fn template_uses_only_escaped_output(body: &str) -> bool {
    !body.contains("{{{") && !body.contains("{{&")
}

fn valid_template_image_url(value: &str) -> bool {
    if value.is_empty()
        || value.len() > 2048
        || value
            .chars()
            .any(|character| character.is_control() || character.is_whitespace())
        || value.contains('\\')
        || value.starts_with("//")
    {
        return false;
    }
    if value.starts_with('/') {
        return true;
    }
    reqwest::Url::parse(value).is_ok_and(|url| {
        url.scheme() == "https"
            && url.host_str().is_some()
            && url.username().is_empty()
            && url.password().is_none()
    })
}

fn template_image_urls_are_safe(body: &str) -> bool {
    let lower = body.to_ascii_lowercase();
    let mut search_from = 0;
    while let Some(relative) = lower[search_from..].find("src") {
        let start = search_from + relative + 3;
        let mut equals = start;
        while lower
            .as_bytes()
            .get(equals)
            .is_some_and(u8::is_ascii_whitespace)
        {
            equals += 1;
        }
        if lower.as_bytes().get(equals) != Some(&b'=') {
            search_from = start;
            continue;
        }
        let mut value_start = equals + 1;
        while lower
            .as_bytes()
            .get(value_start)
            .is_some_and(u8::is_ascii_whitespace)
        {
            value_start += 1;
        }
        let Some(quote) = lower.as_bytes().get(value_start).copied() else {
            return false;
        };
        if quote != b'\'' && quote != b'"' {
            return false;
        }
        let content_start = value_start + 1;
        let Some(relative_end) = lower
            .as_bytes()
            .get(content_start..)
            .expect("content start was derived from the same string")
            .iter()
            .position(|candidate| *candidate == quote)
        else {
            return false;
        };
        let content_end = content_start + relative_end;
        if !valid_template_image_url(&body[content_start..content_end]) {
            return false;
        }
        search_from = content_end + 1;
    }
    true
}

fn valid_template_link_url(value: &str) -> bool {
    if value.is_empty()
        || value.len() > 2048
        || value
            .chars()
            .any(|character| character.is_control() || character.is_whitespace())
        || value.contains('\\')
        || value.starts_with("//")
    {
        return false;
    }
    if value.starts_with('/') || value.starts_with('#') {
        return true;
    }
    reqwest::Url::parse(value).is_ok_and(|url| {
        url.scheme() == "https"
            && url.host_str().is_some()
            && url.username().is_empty()
            && url.password().is_none()
    })
}

fn template_link_urls_are_safe(body: &str) -> bool {
    let lower = body.to_ascii_lowercase();
    let mut search_from = 0;
    while let Some(relative) = lower[search_from..].find("href") {
        let start = search_from + relative + 4;
        let mut equals = start;
        while lower
            .as_bytes()
            .get(equals)
            .is_some_and(u8::is_ascii_whitespace)
        {
            equals += 1;
        }
        if lower.as_bytes().get(equals) != Some(&b'=') {
            search_from = start;
            continue;
        }
        let mut value_start = equals + 1;
        while lower
            .as_bytes()
            .get(value_start)
            .is_some_and(u8::is_ascii_whitespace)
        {
            value_start += 1;
        }
        let Some(quote) = lower.as_bytes().get(value_start).copied() else {
            return false;
        };
        if quote != b'\'' && quote != b'"' {
            return false;
        }
        let content_start = value_start + 1;
        let Some(relative_end) = lower
            .as_bytes()
            .get(content_start..)
            .expect("content start was derived from the same string")
            .iter()
            .position(|candidate| *candidate == quote)
        else {
            return false;
        };
        let content_end = content_start + relative_end;
        if !valid_template_link_url(&body[content_start..content_end]) {
            return false;
        }
        search_from = content_end + 1;
    }
    true
}

/// Parse template HTML with html5ever through Ammonia and require the
/// canonical sanitized fragment to be byte-identical to the submitted body.
/// This keeps storage deterministic while rejecting malformed or unsafe markup
/// through a parser rather than relying only on substring checks. URL policy is
/// still enforced below because relative paths and HTTPS hosts are application
/// constraints, not generic sanitizer defaults.
fn parser_sanitized_template_body(body: &str) -> Option<String> {
    const TAGS: &[&str] = &[
        "a",
        "b",
        "blockquote",
        "br",
        "code",
        "div",
        "em",
        "h1",
        "h2",
        "h3",
        "h4",
        "h5",
        "h6",
        "hr",
        "i",
        "img",
        "li",
        "ol",
        "p",
        "pre",
        "span",
        "strong",
        "u",
        "ul",
    ];
    let tags = TAGS.iter().copied().collect::<HashSet<_>>();
    let mut tag_attributes = HashMap::new();
    tag_attributes.insert("a", ["href"].into_iter().collect::<HashSet<_>>());
    tag_attributes.insert(
        "img",
        ["src", "alt", "width", "height"]
            .into_iter()
            .collect::<HashSet<_>>(),
    );
    let cleaned = AmmoniaBuilder::default()
        .tags(tags)
        .tag_attributes(tag_attributes)
        .generic_attributes(HashSet::new())
        .url_schemes(["https"].into_iter().collect())
        .url_relative(UrlRelative::PassThrough)
        .link_rel(None)
        .clean(body)
        .to_string();
    (cleaned == body).then_some(cleaned)
}

fn template_markup_is_safe(body: &str) -> bool {
    let lower = body.to_ascii_lowercase();
    [
        "<script",
        "</script",
        "<iframe",
        "</iframe",
        "<object",
        "<embed",
        "<base",
        "<form",
        "<link",
        "<meta",
        "<style",
        "javascript:",
        "vbscript:",
        "data:text/html",
        "srcdoc=",
        "onerror=",
        "onload=",
        "onclick=",
        "onmouseover=",
    ]
    .iter()
    .all(|marker| !lower.contains(marker))
        && parser_sanitized_template_body(body).is_some()
        && !template_contains_event_handler(&lower)
        && template_tags_are_allowlisted(body)
        && template_image_urls_are_safe(body)
        && template_link_urls_are_safe(body)
}

fn template_tags_are_allowlisted(body: &str) -> bool {
    const ALLOWED_TAGS: &[&str] = &[
        "a",
        "b",
        "blockquote",
        "br",
        "code",
        "div",
        "em",
        "h1",
        "h2",
        "h3",
        "h4",
        "h5",
        "h6",
        "hr",
        "i",
        "img",
        "li",
        "ol",
        "p",
        "pre",
        "span",
        "strong",
        "u",
        "ul",
    ];
    const VOID_TAGS: &[&str] = &["br", "hr", "img"];
    let bytes = body.as_bytes();
    let mut cursor = 0;
    let mut open_tags: Vec<String> = Vec::new();
    while let Some(relative_start) = body[cursor..].find('<') {
        let start = cursor + relative_start;
        let mut quote = None;
        let mut end = None;
        for (offset, byte) in bytes[start + 1..].iter().enumerate() {
            match (quote, *byte) {
                (None, b'\'' | b'"') => quote = Some(*byte),
                (Some(active), byte) if active == byte => quote = None,
                (None, b'>') => {
                    end = Some(start + 1 + offset);
                    break;
                }
                _ => {}
            }
        }
        let Some(end) = end else {
            return false;
        };
        let mut tag = body[start + 1..end].trim();
        let closing = tag.starts_with('/');
        if closing {
            tag = tag[1..].trim_start();
        }
        if tag.is_empty() || matches!(tag.as_bytes().first(), Some(b'!' | b'?')) {
            return false;
        }
        let name_end = tag
            .bytes()
            .position(|byte| byte.is_ascii_whitespace() || byte == b'/')
            .unwrap_or(tag.len());
        let name = tag[..name_end].to_ascii_lowercase();
        if !ALLOWED_TAGS.contains(&name.as_str()) {
            return false;
        }
        let remainder = &tag[name_end..];
        if closing {
            if !remainder.trim().is_empty() {
                return false;
            }
            if open_tags.pop().as_deref() != Some(name.as_str()) {
                return false;
            }
        } else {
            let self_closing = remainder.trim_end().ends_with('/');
            if !template_attributes_are_safe(&name, remainder)
                || (self_closing && !VOID_TAGS.contains(&name.as_str()))
            {
                return false;
            }
            if !self_closing && !VOID_TAGS.contains(&name.as_str()) {
                open_tags.push(name);
            }
        }
        cursor = end + 1;
    }
    open_tags.is_empty()
}

fn template_attributes_are_safe(tag: &str, attributes: &str) -> bool {
    let mut input = attributes.trim();
    if input.ends_with('/') {
        input = input[..input.len() - 1].trim_end();
    }
    let bytes = input.as_bytes();
    let mut cursor = 0;
    while cursor < bytes.len() {
        while bytes.get(cursor).is_some_and(u8::is_ascii_whitespace) {
            cursor += 1;
        }
        if cursor == bytes.len() {
            break;
        }
        let name_start = cursor;
        while bytes
            .get(cursor)
            .is_some_and(|byte| byte.is_ascii_alphanumeric() || *byte == b'-')
        {
            cursor += 1;
        }
        if cursor == name_start {
            return false;
        }
        let name = input[name_start..cursor].to_ascii_lowercase();
        let allowed = match tag {
            "a" => name == "href",
            "img" => matches!(name.as_str(), "src" | "alt" | "width" | "height"),
            _ => false,
        };
        if !allowed {
            return false;
        }
        while bytes.get(cursor).is_some_and(u8::is_ascii_whitespace) {
            cursor += 1;
        }
        if bytes.get(cursor) != Some(&b'=') {
            return false;
        }
        cursor += 1;
        while bytes.get(cursor).is_some_and(u8::is_ascii_whitespace) {
            cursor += 1;
        }
        let Some(quote) = bytes.get(cursor).copied() else {
            return false;
        };
        if quote != b'\'' && quote != b'"' {
            return false;
        }
        cursor += 1;
        let value_start = cursor;
        while bytes.get(cursor).is_some_and(|byte| *byte != quote) {
            cursor += 1;
        }
        let Some(_) = bytes.get(cursor) else {
            return false;
        };
        let value = &input[value_start..cursor];
        let valid = match name.as_str() {
            "href" => valid_template_link_url(value),
            "src" => valid_template_image_url(value),
            "alt" => control_free_with_max(value, 255),
            "width" | "height" => value
                .parse::<u16>()
                .is_ok_and(|value| (1..=4096).contains(&value)),
            _ => false,
        };
        if !valid {
            return false;
        }
        cursor += 1;
    }
    true
}

fn template_contains_event_handler(lower: &str) -> bool {
    let bytes = lower.as_bytes();
    let mut index = 0;
    while index + 2 < bytes.len() {
        if bytes[index] != b'o' || bytes[index + 1] != b'n' {
            index += 1;
            continue;
        }
        let tag_start = lower[..index].rfind('<');
        let tag_end = lower[..index].rfind('>');
        let inside_tag = tag_start.is_some_and(|start| tag_end.is_none_or(|end| start > end));
        let preceded_by_boundary =
            index == 0 || bytes[index - 1].is_ascii_whitespace() || bytes[index - 1] == b'<';
        if !inside_tag || !preceded_by_boundary {
            index += 2;
            continue;
        }
        let mut cursor = index + 2;
        while bytes.get(cursor).is_some_and(u8::is_ascii_whitespace) {
            cursor += 1;
        }
        let event_start = cursor;
        while bytes.get(cursor).is_some_and(u8::is_ascii_alphabetic) {
            cursor += 1;
        }
        if cursor > event_start {
            while bytes.get(cursor).is_some_and(u8::is_ascii_whitespace) {
                cursor += 1;
            }
            if bytes.get(cursor) == Some(&b'=') {
                return true;
            }
        }
        index += 2;
    }
    false
}

fn validate_template_content(
    subject: Option<&str>,
    body: &str,
    variables: &serde_json::Value,
) -> Result<(), StatusCode> {
    if subject.is_some_and(|subject| !control_free_with_max(subject, 255))
        || body.is_empty()
        || body.len() > 64 * 1024
        || body.chars().any(char::is_control)
        || !template_uses_only_escaped_output(body)
        || !template_markup_is_safe(body)
        || !valid_template_variables(variables)
    {
        return Err(StatusCode::BAD_REQUEST);
    }
    Handlebars::new()
        .register_template_string("candidate", body)
        .map_err(|_| StatusCode::BAD_REQUEST)?;
    Ok(())
}

fn validate_template_request(request: &CreateTemplateRequest) -> Result<(), StatusCode> {
    if !control_free_with_max(&request.name, 100)
        || request.name.trim().is_empty()
        || !matches!(request.channel.as_str(), "email" | "in_app" | "push")
    {
        return Err(StatusCode::BAD_REQUEST);
    }
    validate_template_content(
        request.subject.as_deref(),
        &request.body,
        &request.variables,
    )
}

fn validate_template_rollback_request(request: &TemplateRollbackRequest) -> Result<(), StatusCode> {
    (request.version > 0)
        .then_some(())
        .ok_or(StatusCode::BAD_REQUEST)
}

fn valid_template_variables(value: &serde_json::Value) -> bool {
    let Some(object) = value.as_object() else {
        return false;
    };
    object.len() <= 128
        && object.iter().all(|(name, definition)| {
            !name.is_empty()
                && name.len() <= 64
                && name
                    .bytes()
                    .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-' | b'.'))
                && definition.as_object().is_some_and(|definition| {
                    definition.len() <= 3
                        && definition
                            .get("type")
                            .and_then(serde_json::Value::as_str)
                            .is_some_and(|kind| {
                                matches!(
                                    kind,
                                    "string"
                                        | "number"
                                        | "integer"
                                        | "boolean"
                                        | "object"
                                        | "array"
                                )
                            })
                        && definition
                            .get("required")
                            .is_none_or(serde_json::Value::is_boolean)
                        && definition
                            .get("description")
                            .and_then(serde_json::Value::as_str)
                            .is_none_or(|description| control_free_with_max(description, 512))
                        && definition
                            .keys()
                            .all(|key| matches!(key.as_str(), "type" | "required" | "description"))
                })
        })
}

fn template_value_matches(value: &serde_json::Value, kind: &str) -> bool {
    match kind {
        "string" => value.is_string(),
        "number" => value.is_number(),
        "integer" => value.as_i64().is_some() || value.as_u64().is_some(),
        "boolean" => value.is_boolean(),
        "object" => value.is_object(),
        "array" => value.is_array(),
        _ => false,
    }
}

fn validate_template_data(
    schema: &serde_json::Value,
    data: &HashMap<String, serde_json::Value>,
) -> Result<(), StatusCode> {
    let object = schema.as_object().ok_or(StatusCode::BAD_REQUEST)?;
    if data.keys().any(|name| !object.contains_key(name)) {
        return Err(StatusCode::BAD_REQUEST);
    }
    for (name, definition) in object {
        let definition = definition.as_object().ok_or(StatusCode::BAD_REQUEST)?;
        let kind = definition
            .get("type")
            .and_then(serde_json::Value::as_str)
            .ok_or(StatusCode::BAD_REQUEST)?;
        let Some(value) = data.get(name) else {
            if definition
                .get("required")
                .and_then(serde_json::Value::as_bool)
                .unwrap_or(false)
            {
                return Err(StatusCode::BAD_REQUEST);
            }
            continue;
        };
        if !template_value_matches(value, kind) {
            return Err(StatusCode::BAD_REQUEST);
        }
    }
    Ok(())
}

fn validate_template_preview_data(
    value: serde_json::Value,
) -> Result<HashMap<String, serde_json::Value>, StatusCode> {
    let encoded = serde_json::to_vec(&value).map_err(|_| StatusCode::BAD_REQUEST)?;
    if !value.is_object() || encoded.len() > 64 * 1024 {
        return Err(StatusCode::BAD_REQUEST);
    }
    serde_json::from_value(value).map_err(|_| StatusCode::BAD_REQUEST)
}

fn project_admin_notification(row: AdminNotificationRow) -> Option<AdminNotificationItem> {
    if row.id.trim().is_empty()
        || !control_free_with_max(&row.id, 66)
        || row
            .title
            .as_deref()
            .is_some_and(|value| !control_free_with_max(value, 255))
        || row
            .subject
            .as_deref()
            .is_some_and(|value| !control_free_with_max(value, 255))
        || !safe_lower_ascii_token(&row.channel, 20)
        || !matches!(
            row.status.as_str(),
            "pending" | "sent" | "failed" | "suppressed" | "expired" | "read"
        )
        || row
            .notification_type
            .as_deref()
            .is_some_and(|value| !control_free_with_max(value, 50))
        || row.priority.as_deref().is_some_and(|value| {
            !matches!(value, "low" | "normal" | "high" | "critical" | "urgent")
        })
    {
        return None;
    }

    Some(AdminNotificationItem {
        id: row.id,
        title: row.title,
        subject: row.subject,
        channel: row.channel,
        status: row.status,
        notification_type: row.notification_type,
        priority: row.priority,
        sent_at: row.sent_at,
        created_at: row.created_at,
    })
}

fn admin_notification_cardinality_is_valid(
    total: i64,
    limit: i64,
    offset: i64,
    item_count: usize,
) -> bool {
    let Ok(item_count) = i64::try_from(item_count) else {
        return false;
    };
    if total < 0 || limit <= 0 || offset < 0 || item_count > limit {
        return false;
    }
    if item_count == 0 {
        return total == 0 || offset >= total;
    }
    offset < total
        && offset
            .checked_add(item_count)
            .is_some_and(|page_end| page_end <= total)
}

#[derive(Serialize, Deserialize)]
struct TemplateListResponse {
    items: Vec<Template>,
    total: i64,
}

#[derive(Debug, Error)]
enum TemplateLoadError {
    #[error("notification template query failed")]
    Query(#[from] sqlx::Error),
    #[error("notification template registration failed")]
    Registration(#[from] handlebars::TemplateError),
    #[error("notification template contains raw output syntax")]
    RawOutput,
    #[error("notification template content is invalid or unsafe")]
    InvalidContent,
}

#[tokio::main]
async fn main() {
    epsx_observability::Observability::init("notification");
    let args = Args::parse();

    let production = matches!(
        args.environment,
        Environment::Staging | Environment::Production
    );
    let jwks_url = args.jwks_url.unwrap_or_else(|| {
        format!(
            "{}/.well-known/jwks.json",
            args.oidc_issuer.trim_end_matches('/')
        )
    });
    let verifier = build_auth_verifier(&args.oidc_issuer, &jwks_url, production)
        .expect("notification OIDC configuration must be valid");

    let db = sqlx::PgPool::connect(&args.database_url)
        .await
        .expect("Failed to connect to database");
    verify_schema_compatibility(&db)
        .await
        .expect("notification schema must be compatible before startup");
    verify_lifecycle_schema_compatibility(&db)
        .await
        .expect("notification lifecycle schema must be compatible before startup");

    // Plan membership is owned by the core backend database, not the
    // notification store. Keep this resolver pool optional and read-only by
    // contract: without an explicit URL, plan-targeted publisher requests
    // fail closed instead of accidentally querying the isolated notification
    // database for a table it does not own.
    let plan_db = match args
        .plan_database_url
        .as_deref()
        .map(str::trim)
        .filter(|url| !url.is_empty())
    {
        Some(url) => Some(
            connect_read_only_plan_pool(url)
                .await
                .expect("notification plan resolver database must be reachable when configured"),
        ),
        None => None,
    };

    let mut hb = Handlebars::new();
    hb.set_strict_mode(true);
    load_templates_to_hb(&db, &mut hb)
        .await
        .expect("active notification templates must load before startup");

    // Init SMTP. Both supported modes require TLS; an invalid host or TLS
    // configuration leaves the provider unavailable so queued jobs fail
    // closed instead of silently sending credentials over plaintext SMTP.
    let smtp = build_smtp_transport(
        &args.smtp_host,
        args.smtp_port,
        &args.smtp_user,
        &args.smtp_password,
    );

    let redis = args
        .redis_url
        .as_deref()
        .filter(|url| !url.trim().is_empty())
        .and_then(|url| match redis::Client::open(url) {
            Ok(client) => Some(client),
            Err(_error) => {
                tracing::warn!("notification realtime Redis configuration rejected");
                None
            }
        });
    let realtime_notify = Arc::new(Notify::new());

    let vapid_key_id = std::env::var("NOTIFICATION_VAPID_KEY_ID")
        .ok()
        .filter(|key_id| valid_vapid_key_id(key_id))
        .unwrap_or_else(|| "active".to_string());
    let vapid_public_key = std::env::var("NOTIFICATION_VAPID_PUBLIC_KEY")
        .ok()
        .filter(|key| valid_vapid_public_key(key));
    let vapid_private_key = vapid_private_key_from_env().filter(|private_key| {
        vapid_public_key
            .as_deref()
            .is_some_and(|public_key| vapid_private_key_matches_public(private_key, public_key))
    });
    let vapid_previous_key_id = std::env::var("NOTIFICATION_VAPID_PREVIOUS_KEY_ID")
        .ok()
        .filter(|key_id| valid_vapid_key_id(key_id))
        .filter(|key_id| key_id != &vapid_key_id);
    let vapid_previous_public_key = std::env::var("NOTIFICATION_VAPID_PREVIOUS_PUBLIC_KEY")
        .ok()
        .filter(|key| valid_vapid_public_key(key));
    let vapid_previous_private_key =
        vapid_private_key_from_env_name("NOTIFICATION_VAPID_PREVIOUS_PRIVATE_KEY")
            .filter(|private_key| {
                vapid_previous_public_key
                    .as_deref()
                    .is_some_and(|public_key| {
                        vapid_private_key_matches_public(private_key, public_key)
                    })
            })
            .filter(|_| vapid_previous_key_id.is_some());

    let state = AppState {
        db,
        plan_db,
        templates: Arc::new(RwLock::new(hb)),
        smtp: Arc::new(RwLock::new(smtp)),
        provider_signing_secrets: provider_signing_secrets_from_env(),
        realtime_slots: Arc::new(tokio::sync::Semaphore::new(MAX_REALTIME_CONNECTIONS)),
        stream_metrics: Arc::new(StreamMetrics::default()),
        vapid_key_id,
        vapid_public_key,
        vapid_private_key,
        vapid_previous_key_id,
        vapid_previous_private_key,
        from: args.from_address,
        from_name: args.from_name,
        redis,
        realtime_notify,
    };
    if let Some(client) = state.redis.clone() {
        tokio::spawn(run_redis_listener(
            client,
            Arc::clone(&state.realtime_notify),
        ));
    }
    tokio::spawn(run_delivery_worker(state.clone()));

    let app = Router::new()
        .route("/health", get(health))
        .route("/ready", get(readiness))
        .route(
            "/api/v1/notification/templates",
            get(list_templates).post(create_template),
        )
        .route(
            "/api/v1/notification/templates/{id}",
            get(get_template).delete(delete_template),
        )
        .route(
            "/api/v1/notification/templates/{id}/preview",
            post(preview_template),
        )
        .route(
            "/api/v1/notification/templates/{id}/rollback",
            post(rollback_template),
        )
        .route(
            "/api/v1/notification/templates/{id}/audit",
            get(list_template_audit),
        )
        .route("/api/v1/notification/send", post(send_notification))
        .route("/api/v1/notification/publish", post(publish_notification))
        .route(
            "/api/v1/notification/provider-events",
            post(record_provider_event),
        )
        .route(
            "/api/v1/notification/admin/list",
            get(list_admin_notifications),
        )
        .route(
            "/api/v1/notification/admin/{id}/read",
            post(admin_mark_read),
        )
        .route(
            "/api/v1/notification/admin/{id}",
            delete(admin_delete_notification),
        )
        .route("/api/v1/notification/admin/metrics", get(admin_metrics))
        .route(
            "/api/v1/notification/admin/dead-letters/{id}/redrive",
            post(redrive_dead_letter),
        )
        .route("/api/v1/notification/list", get(list_notifications))
        .route("/api/v1/notification/unread-count", get(unread_count))
        .route("/api/v1/notification/stream", get(notification_stream))
        .route("/api/v1/notification/stream/ack", post(acknowledge_stream))
        .route(
            "/api/v1/notification/push",
            get(push_status)
                .put(push_subscribe)
                .delete(push_unsubscribe),
        )
        // Source compatibility: the development client revokes every active
        // subscription for the verified owner and sends no endpoint body.
        .route(
            "/api/v1/notification/push/unsubscribe",
            axum::routing::delete(push_unsubscribe_all),
        )
        .route(
            "/api/v1/notification/preferences",
            get(get_preferences).put(update_preferences),
        )
        .route("/api/v1/notification/mark-all-read", post(mark_all_read))
        .route("/api/v1/notification/clear-all", post(clear_all))
        .route("/api/v1/notification/{id}/read", post(mark_read))
        .route("/api/v1/notification/{id}/unread", post(mark_unread))
        .route("/api/v1/notification/{id}/click", post(mark_clicked))
        .route("/api/v1/notification/{id}/dismiss", post(mark_dismissed))
        .route(
            "/api/v1/notification/{id}/acknowledge",
            put(acknowledge_notification),
        )
        .route(
            "/api/v1/notification/{id}",
            get(get_notification).delete(delete_notification),
        )
        .with_state(state);
    let app = protect_router(app, verifier);

    let addr: SocketAddr = format!("{}:{}", args.host, args.port).parse().unwrap();
    info!("Notification service listening on {}", addr);
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn health() -> StatusCode {
    StatusCode::OK
}

fn build_smtp_transport(
    host: &str,
    port: u16,
    username: &str,
    password: &str,
) -> Option<SmtpTransport> {
    if host.trim().is_empty()
        || host.len() > 253
        || host
            .chars()
            .any(|character| character.is_whitespace() || character.is_control())
        || port == 0
    {
        return None;
    }
    let credentials = Credentials::new(username.to_owned(), password.to_owned());
    // SMTPS on 465 wraps the connection immediately. Other submission ports
    // use STARTTLS and refuse to continue if the server cannot upgrade.
    let builder = if port == 465 {
        SmtpTransport::relay(host)
    } else {
        SmtpTransport::starttls_relay(host)
    };
    builder.ok().map(|builder| {
        builder
            .port(port)
            .timeout(Some(std::time::Duration::from_secs(
                SMTP_TRANSPORT_TIMEOUT_SECONDS,
            )))
            .credentials(credentials)
            .build()
    })
}

/// Drain durable channel jobs outside request admission. Provider calls are
/// isolated from the request task; unsupported channels are terminally failed
/// and retained in the dead-letter table for an operator decision.
async fn run_delivery_worker(state: AppState) {
    let db = state.db.clone();
    let worker = DeliveryWorker::new(db.clone());
    let worker_id = format!("notification-{}", Uuid::new_v4().simple());
    loop {
        match worker.expire_due_jobs().await {
            Ok(_) => {}
            Err(error) => {
                tracing::error!("notification expiry sweep failed: {error}");
                tokio::time::sleep(std::time::Duration::from_secs(1)).await;
                continue;
            }
        }
        match worker.claim_next(&worker_id, 30).await {
            Ok(Some(job)) => {
                match worker.expire_if_due(&job.id).await {
                    Ok(true) => continue,
                    Ok(false) => {}
                    Err(error) => {
                        tracing::error!(job_id = %job.id, "notification expiry check failed: {error}");
                        continue;
                    }
                }
                if worker.begin_attempt(&job.id).await.is_err() {
                    continue;
                }
                match worker.expire_if_due(&job.id).await {
                    Ok(true) => continue,
                    Ok(false) => {}
                    Err(error) => {
                        tracing::error!(job_id = %job.id, "notification expiry check failed: {error}");
                        continue;
                    }
                }
                let mut provider_message_id = None;
                let (outcome, error_code) = match job.channel.as_str() {
                    "in_app" => (epsx_notification::delivery::DeliveryResult::Accepted, None),
                    "email" => {
                        let delivery = sqlx::query_as::<_, (String, String, String)>(
                            "SELECT recipient, COALESCE(subject, ''), body FROM public.notifications WHERE id = $1",
                        )
                        .bind(&job.notification_id)
                        .fetch_optional(&db)
                        .await;
                        match delivery {
                            Ok(Some((recipient, subject, body))) => {
                                let (status, error, _, message_id) =
                                    send_email(&state, &recipient, &subject, &body, &job.id).await;
                                if status == "sent" {
                                    provider_message_id = message_id;
                                    (epsx_notification::delivery::DeliveryResult::Accepted, None)
                                } else if error.as_deref() == Some("provider_not_configured") {
                                    (
                                        epsx_notification::delivery::DeliveryResult::TerminalFailure,
                                        Some("provider_not_configured"),
                                    )
                                } else {
                                    (
                                        epsx_notification::delivery::DeliveryResult::Retry,
                                        Some("provider_send_failed"),
                                    )
                                }
                            }
                            Ok(None) => (
                                epsx_notification::delivery::DeliveryResult::TerminalFailure,
                                Some("notification_missing"),
                            ),
                            Err(_) => (
                                epsx_notification::delivery::DeliveryResult::Retry,
                                Some("notification_lookup_failed"),
                            ),
                        }
                    }
                    "push" => {
                        let delivery = sqlx::query_as::<_, (
                            Option<String>,
                            String,
                            Option<serde_json::Value>,
                            Option<String>,
                            String,
                            String,
                            String,
                            String,
                        )>(
                            "SELECT n.title, n.body, n.data, n.action_url, s.vapid_key_id, s.endpoint, s.p256dh, s.auth FROM public.notifications n JOIN public.notification_push_subscriptions s ON s.endpoint = $1 AND s.revoked_at IS NULL WHERE n.id = $2",
                        )
                        .bind(&job.recipient)
                        .bind(&job.notification_id)
                        .fetch_optional(&db)
                        .await;
                        match delivery {
                            Ok(Some((
                                title,
                                body,
                                data,
                                action_url,
                                vapid_key_id,
                                endpoint,
                                p256dh,
                                auth,
                            ))) => {
                                let (status, error, message_id) = send_push(
                                    &state,
                                    PushDelivery {
                                        job_id: &job.id,
                                        vapid_key_id: &vapid_key_id,
                                        endpoint: &endpoint,
                                        p256dh: &p256dh,
                                        auth: &auth,
                                        title: title.as_deref().unwrap_or("EPSX notification"),
                                        body: &body,
                                        data: data.as_ref(),
                                        action_url: action_url.as_deref(),
                                    },
                                )
                                .await;
                                if status == "sent" {
                                    provider_message_id = message_id;
                                    (epsx_notification::delivery::DeliveryResult::Accepted, None)
                                } else if matches!(
                                    error.as_deref(),
                                    Some(
                                        "provider_not_configured"
                                            | "push_endpoint_revoked"
                                            | "push_payload_too_large"
                                            | "push_payload_invalid"
                                            | "push_invalid_vapid_key"
                                            | "push_invalid_subscription"
                                            | "push_provider_rejected"
                                    )
                                ) {
                                    let reason = match error.as_deref() {
                                        Some("push_endpoint_revoked") => "push_endpoint_revoked",
                                        Some("push_payload_too_large") => "push_payload_too_large",
                                        Some("push_payload_invalid") => "push_payload_invalid",
                                        Some("push_invalid_vapid_key") => "push_invalid_vapid_key",
                                        Some("push_invalid_subscription") => {
                                            "push_invalid_subscription"
                                        }
                                        Some("push_provider_rejected") => "push_provider_rejected",
                                        _ => "provider_not_configured",
                                    };
                                    (
                                        epsx_notification::delivery::DeliveryResult::TerminalFailure,
                                        Some(reason),
                                    )
                                } else {
                                    (
                                        epsx_notification::delivery::DeliveryResult::Retry,
                                        Some("provider_send_failed"),
                                    )
                                }
                            }
                            Ok(None) => (
                                epsx_notification::delivery::DeliveryResult::TerminalFailure,
                                Some("push_subscription_not_found"),
                            ),
                            Err(_) => (
                                epsx_notification::delivery::DeliveryResult::Retry,
                                Some("notification_lookup_failed"),
                            ),
                        }
                    }
                    _ => (
                        epsx_notification::delivery::DeliveryResult::TerminalFailure,
                        Some("provider_not_configured"),
                    ),
                };
                let outcome =
                    if matches!(outcome, epsx_notification::delivery::DeliveryResult::Retry)
                        && epsx_notification::delivery::retry_will_exhaust_attempts(
                            job.attempt_count,
                            epsx_notification::delivery::DEFAULT_MAX_ATTEMPTS,
                        )
                    {
                        epsx_notification::delivery::DeliveryResult::TerminalFailure
                    } else {
                        outcome
                    };
                if worker
                    .record_result(&job.id, outcome, provider_message_id.as_deref(), error_code)
                    .await
                    .is_err()
                {
                    continue;
                }
                match outcome {
                    epsx_notification::delivery::DeliveryResult::Accepted => {
                        if let Err(error) = sqlx::query(
                            "UPDATE public.notifications SET status = 'sent', sent_at = COALESCE(sent_at, NOW()), error = NULL WHERE id = $1",
                        )
                        .bind(&job.notification_id)
                        .execute(&db)
                        .await
                        {
                            tracing::error!(job_id = %job.id, "in-app notification status update failed: {error}");
                        }
                    }
                    epsx_notification::delivery::DeliveryResult::TerminalFailure => {
                        let payload = serde_json::json!({
                            "job_id": job.id,
                            "notification_id": job.notification_id,
                            "channel": job.channel,
                        });
                        let reason = error_code.unwrap_or("delivery_failed");
                        if let Err(error) = sqlx::query(
                            "UPDATE public.notifications SET status = 'failed', error = $2 WHERE id = $1",
                        )
                        .bind(&job.notification_id)
                        .bind(reason)
                        .execute(&db)
                        .await
                        {
                            tracing::error!(job_id = %job.id, "notification failure status update failed: {error}");
                        }
                        if let Err(error) = worker
                            .dead_letter(&job.id, reason, payload)
                            .await
                        {
                            tracing::error!(job_id = %job.id, "notification dead-letter transition failed: {error}");
                        }
                    }
                    epsx_notification::delivery::DeliveryResult::Retry => {}
                }
            }
            Ok(None) => tokio::time::sleep(std::time::Duration::from_millis(250)).await,
            Err(error) => {
                tracing::error!("notification delivery worker claim failed: {error}");
                tokio::time::sleep(std::time::Duration::from_secs(1)).await;
            }
        }
    }
}

async fn readiness(State(state): State<AppState>) -> Response {
    let redis_fanout_configured = state.redis.is_some();
    let redis_reachable = redis_reachable(state.redis.as_ref()).await;
    let plan_targeting_configured = state.plan_db.is_some();
    let smtp_configured = state.smtp.read().await.is_some();
    let vapid_configured = vapid_is_configured(&state);
    let vapid_rotation_configured =
        state.vapid_previous_key_id.is_some() && state.vapid_previous_private_key.is_some();
    let provider_callbacks_configured = !state.provider_signing_secrets.is_empty();
    let database_ok = sqlx::query_scalar::<_, i32>("SELECT 1")
        .fetch_one(&state.db)
        .await
        .is_ok();
    let lifecycle_ok = database_ok
        && verify_schema_compatibility(&state.db).await.is_ok()
        && verify_lifecycle_schema_compatibility(&state.db)
            .await
            .is_ok();
    let queue = if lifecycle_ok {
        sqlx::query_as::<_, (i64, Option<i64>)>(
            "SELECT COUNT(*)::bigint, GREATEST(0, EXTRACT(EPOCH FROM (NOW() - MIN(available_at))))::bigint FROM public.notification_channel_jobs WHERE state IN ('queued', 'leased', 'attempting', 'retry_wait')",
        )
        .fetch_one(&state.db)
        .await
        .ok()
    } else {
        None
    };
    let ready = lifecycle_ok && queue.is_some();
    if ready {
        let (queue_depth, queue_age_seconds) = queue.expect("checked above");
        (
            StatusCode::OK,
            Json(serde_json::json!({
                "status": "ready",
                "database": true,
                "lifecycle": true,
                "redis_fanout_configured": redis_fanout_configured,
                "redis_reachable": redis_reachable,
                "plan_targeting_configured": plan_targeting_configured,
                "smtp_configured": smtp_configured,
                "vapid_configured": vapid_configured,
                "vapid_rotation_configured": vapid_rotation_configured,
                "provider_callbacks_configured": provider_callbacks_configured,
                "queue_depth": queue_depth,
                "queue_age_seconds": queue_age_seconds
            })),
        )
            .into_response()
    } else {
        (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(serde_json::json!({
                "status": "not_ready",
                "database": database_ok,
                "lifecycle": lifecycle_ok,
                "redis_fanout_configured": redis_fanout_configured,
                "redis_reachable": redis_reachable,
                "plan_targeting_configured": plan_targeting_configured,
                "smtp_configured": smtp_configured,
                "vapid_configured": vapid_configured,
                "vapid_rotation_configured": vapid_rotation_configured,
                "provider_callbacks_configured": provider_callbacks_configured,
                "queue_depth": serde_json::Value::Null,
                "queue_age_seconds": serde_json::Value::Null
            })),
        )
            .into_response()
    }
}

/// Probe optional Redis wake-up fanout with a bounded PING. Redis remains a
/// best-effort hint channel, so this signal is reported separately and never
/// turns a healthy PostgreSQL/lifecycle decision into a delivery claim.
async fn redis_reachable(client: Option<&redis::Client>) -> bool {
    let Some(client) = client else {
        return false;
    };
    let result = tokio::time::timeout(std::time::Duration::from_secs(1), async {
        let mut connection = client.get_multiplexed_async_connection().await?;
        redis::cmd("PING")
            .query_async::<String>(&mut connection)
            .await
    })
    .await;
    matches!(result, Ok(Ok(response)) if response == "PONG")
}

fn validate_preferences_request(
    request: &NotificationPreferencesRequest,
) -> Result<(), StatusCode> {
    if !valid_channel_preferences(&request.channels)
        || serde_json::to_vec(&request.channels)
            .map(|bytes| bytes.len() > 64 * 1024)
            .unwrap_or(true)
    {
        return Err(StatusCode::BAD_REQUEST);
    }
    if request
        .quiet_hours
        .as_ref()
        .is_some_and(|value| !valid_quiet_hours(value))
    {
        return Err(StatusCode::BAD_REQUEST);
    }
    if request.timezone.as_deref().is_some_and(|timezone| {
        timezone.trim().is_empty()
            || timezone.len() > 64
            || timezone.chars().any(|character| character.is_control())
    }) {
        return Err(StatusCode::BAD_REQUEST);
    }
    Ok(())
}

fn valid_channel_preferences(value: &serde_json::Value) -> bool {
    let Some(object) = value.as_object() else {
        return false;
    };
    object.iter().all(|(key, value)| match key.as_str() {
        "email" | "in_app" | "push" => value.is_boolean(),
        "types" => valid_type_preferences(value),
        "priority_filter" => value.as_str().is_some_and(valid_notification_priority),
        _ => false,
    })
}

const NOTIFICATION_TYPES: &[&str] = &[
    "system",
    "security",
    "permission",
    "wallet_management",
    "wallet",
    "payment",
    "general",
    "announcement",
    "advertisement",
    "chat",
];

fn valid_notification_type(value: &str) -> bool {
    NOTIFICATION_TYPES.contains(&value)
}

fn valid_notification_priority(value: &str) -> bool {
    matches!(value, "low" | "normal" | "high" | "critical" | "urgent")
}

fn valid_type_preferences(value: &serde_json::Value) -> bool {
    let Some(object) = value.as_object() else {
        return false;
    };
    object
        .iter()
        .all(|(key, value)| valid_notification_type(key) && value.is_boolean())
}

fn valid_clock(value: &str) -> bool {
    let bytes = value.as_bytes();
    if bytes.len() != 5
        || bytes[2] != b':'
        || !bytes[0].is_ascii_digit()
        || !bytes[1].is_ascii_digit()
        || !bytes[3].is_ascii_digit()
        || !bytes[4].is_ascii_digit()
    {
        return false;
    }
    let hour = (bytes[0] - b'0') * 10 + bytes[1] - b'0';
    let minute = (bytes[3] - b'0') * 10 + bytes[4] - b'0';
    hour < 24 && minute < 60
}

fn valid_quiet_hours(value: &serde_json::Value) -> bool {
    let Some(object) = value.as_object() else {
        return false;
    };
    let Some(start) = object.get("start").and_then(serde_json::Value::as_str) else {
        return false;
    };
    let Some(end) = object.get("end").and_then(serde_json::Value::as_str) else {
        return false;
    };
    valid_clock(start)
        && valid_clock(end)
        && object.iter().all(|(key, value)| {
            matches!(key.as_str(), "start" | "end" | "enabled")
                && (matches!(key.as_str(), "start" | "end") && value.is_string()
                    || key == "enabled" && value.is_boolean())
        })
}

fn channel_preference_enabled(channels: &serde_json::Value, channel: &str) -> bool {
    match channels.get(channel) {
        None => true,
        Some(value) => value.as_bool().unwrap_or(false),
    }
}

fn notification_priority_rank(value: &str) -> Option<u8> {
    Some(match value {
        "low" => 0,
        "normal" => 1,
        "high" => 2,
        "critical" => 3,
        "urgent" => 4,
        _ => return None,
    })
}

/// Evaluate all persisted owner policy dimensions before creating a delivery
/// job. Missing legacy metadata is deliberately permissive so existing target
/// rows retain their channel-only behavior; malformed metadata never grants
/// delivery.
fn notification_policy_allows(
    channels: &serde_json::Value,
    channel: &str,
    notification_type: &str,
    priority: &str,
) -> bool {
    if !channel_preference_enabled(channels, channel)
        || !valid_notification_type(notification_type)
        || !valid_notification_priority(priority)
    {
        return false;
    }
    if let Some(types) = channels.get("types") {
        if !valid_type_preferences(types)
            || !types
                .get(notification_type)
                .and_then(serde_json::Value::as_bool)
                .unwrap_or(true)
        {
            return false;
        }
    }
    if let Some(filter) = channels.get("priority_filter") {
        let Some(filter) = filter
            .as_str()
            .filter(|value| valid_notification_priority(value))
        else {
            return false;
        };
        if priority_rank(priority) < priority_rank(filter) {
            return false;
        }
    }
    true
}

fn priority_rank(value: &str) -> u8 {
    notification_priority_rank(value).unwrap_or(0)
}

async fn load_delivery_preference_policy(
    db: &sqlx::PgPool,
    user_id: Option<&str>,
) -> Result<DeliveryPreferencePolicy, StatusCode> {
    let Some(user_id) = user_id else {
        return Ok(DeliveryPreferencePolicy {
            channels: serde_json::json!({}),
            quiet_until: None,
        });
    };
    sqlx::query_as::<_, DeliveryPreferencePolicy>(
        r#"
WITH policy AS (
    SELECT
        channels,
        quiet_hours,
        COALESCE(NULLIF(timezone, ''), 'UTC') AS timezone_name,
        timezone(COALESCE(NULLIF(timezone, ''), 'UTC'), NOW()) AS local_now
    FROM public.notification_preferences
    WHERE user_id = $1
)
SELECT
    channels,
    CASE
        WHEN quiet_hours IS NULL
          OR COALESCE((quiet_hours->>'enabled')::boolean, TRUE) IS NOT TRUE
          OR NOT (
              ((quiet_hours->>'start')::time < (quiet_hours->>'end')::time
               AND local_now::time >= (quiet_hours->>'start')::time
               AND local_now::time < (quiet_hours->>'end')::time)
              OR
              ((quiet_hours->>'start')::time >= (quiet_hours->>'end')::time
               AND (local_now::time >= (quiet_hours->>'start')::time
                    OR local_now::time < (quiet_hours->>'end')::time))
          )
        THEN NULL::timestamptz
        ELSE
            make_timestamptz(
                EXTRACT(YEAR FROM local_now)::int,
                EXTRACT(MONTH FROM local_now)::int,
                EXTRACT(DAY FROM local_now)::int,
                0,
                0,
                0,
                timezone_name
            )
            + (EXTRACT(HOUR FROM (quiet_hours->>'end')::time) * INTERVAL '1 hour')
            + (EXTRACT(MINUTE FROM (quiet_hours->>'end')::time) * INTERVAL '1 minute')
            + CASE
                WHEN local_now::time >= (quiet_hours->>'start')::time
                 AND (quiet_hours->>'start')::time >= (quiet_hours->>'end')::time
                THEN INTERVAL '1 day'
                ELSE INTERVAL '0 day'
              END
    END AS quiet_until
FROM policy
"#,
    )
    .bind(user_id)
    .fetch_optional(db)
    .await
    .map_err(|error| {
        tracing::error!("notification delivery preference query failed: {error}");
        StatusCode::SERVICE_UNAVAILABLE
    })
    .map(|policy| {
        policy.unwrap_or(DeliveryPreferencePolicy {
            channels: serde_json::json!({}),
            quiet_until: None,
        })
    })
}

async fn get_preferences(
    State(state): State<AppState>,
    Extension(principal): Extension<VerifiedPrincipal>,
) -> Result<Json<NotificationPreferencesResponse>, StatusCode> {
    let owner = canonical_owner(&principal, None)?;
    let preferences = sqlx::query_as::<_, NotificationPreferencesResponse>(
        "SELECT channels, quiet_hours, timezone, updated_at FROM public.notification_preferences WHERE user_id = $1",
    )
    .bind(owner.to_ascii_lowercase())
    .fetch_optional(&state.db)
    .await
    .map_err(|error| {
        tracing::error!("notification preferences read failed: {error}");
        StatusCode::SERVICE_UNAVAILABLE
    })?
    .unwrap_or(NotificationPreferencesResponse {
        channels: serde_json::json!({}),
        quiet_hours: None,
        timezone: None,
        updated_at: None,
    });
    Ok(Json(preferences))
}

async fn update_preferences(
    State(state): State<AppState>,
    Extension(principal): Extension<VerifiedPrincipal>,
    Json(request): Json<NotificationPreferencesRequest>,
) -> Result<Json<NotificationPreferencesResponse>, StatusCode> {
    validate_preferences_request(&request)?;
    if let Some(timezone) = request.timezone.as_deref() {
        let known_timezone = sqlx::query_scalar::<_, bool>(
            "SELECT EXISTS (SELECT 1 FROM pg_timezone_names WHERE name = $1)",
        )
        .bind(timezone)
        .fetch_one(&state.db)
        .await
        .map_err(|error| {
            tracing::error!("notification timezone validation failed: {error}");
            StatusCode::SERVICE_UNAVAILABLE
        })?;
        if !known_timezone {
            return Err(StatusCode::BAD_REQUEST);
        }
    }
    let owner = canonical_owner(&principal, None)?.to_ascii_lowercase();
    let preferences = sqlx::query_as::<_, NotificationPreferencesResponse>(
        "INSERT INTO public.notification_preferences (user_id, channels, quiet_hours, timezone) VALUES ($1, $2, $3, $4) ON CONFLICT (user_id) DO UPDATE SET channels = EXCLUDED.channels, quiet_hours = EXCLUDED.quiet_hours, timezone = EXCLUDED.timezone, updated_at = NOW() RETURNING channels, quiet_hours, timezone, updated_at",
    )
    .bind(owner)
    .bind(request.channels)
    .bind(request.quiet_hours)
    .bind(request.timezone)
    .fetch_one(&state.db)
    .await
    .map_err(|error| {
        tracing::error!("notification preferences write failed: {error}");
        StatusCode::SERVICE_UNAVAILABLE
    })?;
    Ok(Json(preferences))
}

const STREAM_BATCH_LIMIT: i64 = 100;
const STREAM_POLL_INTERVAL: std::time::Duration = std::time::Duration::from_secs(1);
const REDIS_RECONNECT_INTERVAL: std::time::Duration = std::time::Duration::from_secs(5);
const REDIS_WAKEUP_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(1);

fn realtime_wallet_channel(owner: &str) -> String {
    format!("notifications:wallet:{owner}")
}

/// Publish only a wake-up hint. PostgreSQL remains the source of truth for
/// payload, owner scope, ordering, and replay; Redis cannot inject an event
/// into an SSE stream by itself.
async fn publish_realtime_wakeup(state: &AppState, channel: &str, notification_id: &str) {
    publish_realtime_wakeup_parts(
        state.redis.as_ref(),
        &state.realtime_notify,
        channel,
        notification_id,
    )
    .await;
}

async fn publish_realtime_wakeup_parts(
    client: Option<&redis::Client>,
    notify: &Notify,
    channel: &str,
    notification_id: &str,
) {
    notify.notify_waiters();
    let Some(client) = client else {
        return;
    };
    let payload = serde_json::json!({ "id": notification_id }).to_string();
    let mut connection = match tokio::time::timeout(
        REDIS_WAKEUP_TIMEOUT,
        client.get_multiplexed_async_connection(),
    )
    .await
    {
        Ok(Ok(connection)) => connection,
        Ok(Err(_)) | Err(_) => {
            tracing::warn!("notification realtime Redis connection unavailable");
            return;
        }
    };
    let result: redis::RedisResult<i64> =
        match tokio::time::timeout(REDIS_WAKEUP_TIMEOUT, connection.publish(channel, &payload))
            .await
        {
            Ok(result) => result,
            Err(_) => {
                tracing::warn!("notification realtime Redis publish timed out");
                return;
            }
        };
    if result.is_err() {
        tracing::warn!("notification realtime Redis publish failed");
    }
}

/// Keep one bounded Redis subscription per service instance. A message only
/// wakes polling streams; every stream still re-queries PostgreSQL with its
/// own authenticated cursor, so cross-owner payload leakage is impossible.
async fn run_redis_listener(client: redis::Client, notify: Arc<Notify>) {
    loop {
        let mut pubsub = match client.get_async_pubsub().await {
            Ok(pubsub) => pubsub,
            Err(_error) => {
                tracing::warn!("notification realtime Redis subscriber unavailable");
                tokio::time::sleep(REDIS_RECONNECT_INTERVAL).await;
                continue;
            }
        };
        if pubsub.psubscribe("notifications:wallet:*").await.is_err() {
            tracing::warn!("notification realtime Redis wallet subscription failed");
            tokio::time::sleep(REDIS_RECONNECT_INTERVAL).await;
            continue;
        }
        if pubsub.subscribe("notifications:all").await.is_err() {
            tracing::warn!("notification realtime Redis broadcast subscription failed");
            tokio::time::sleep(REDIS_RECONNECT_INTERVAL).await;
            continue;
        }
        let mut messages = pubsub.on_message();
        while messages.next().await.is_some() {
            notify.notify_waiters();
        }
        tokio::time::sleep(REDIS_RECONNECT_INTERVAL).await;
    }
}

#[derive(Debug, FromRow)]
struct StreamNotificationRow {
    id: String,
    title: Option<String>,
    body: String,
    notification_type: Option<String>,
    priority: Option<String>,
    data: Option<serde_json::Value>,
    action_url: Option<String>,
    read_at: Option<chrono::DateTime<chrono::Utc>>,
    created_at: chrono::DateTime<chrono::Utc>,
    expires_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct StreamAcknowledgementRequest {
    event_id: String,
}

fn valid_stream_cursor(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 128
        && !value
            .chars()
            .any(|character| character.is_control() || character.is_whitespace())
}

async fn persisted_stream_cursor(
    db: &sqlx::PgPool,
    owner: &str,
) -> Result<Option<String>, StatusCode> {
    sqlx::query_scalar::<_, Option<String>>(
        "SELECT last_event_id FROM public.notification_replay_cursors WHERE owner_id = $1 AND stream = 'owner'",
    )
    .bind(owner)
    .fetch_optional(db)
    .await
    .map(|row| row.flatten())
    .map_err(|error| {
        tracing::error!("notification stream cursor read failed: {error}");
        StatusCode::SERVICE_UNAVAILABLE
    })
}

async fn stream_cursor_belongs_to_owner(
    db: &sqlx::PgPool,
    owner: &str,
    cursor: &str,
) -> Result<bool, StatusCode> {
    sqlx::query_scalar::<_, bool>(
        "SELECT EXISTS (SELECT 1 FROM public.notifications n LEFT JOIN public.notification_expirations x ON x.notification_id = n.id WHERE n.id = $1 AND (n.user_id = $2 OR (n.user_id IS NULL AND lower(n.recipient) = 'all')) AND (x.expires_at IS NULL OR x.expires_at > NOW()))"
    )
    .bind(cursor)
    .bind(owner)
    .fetch_one(db)
    .await
    .map_err(|error| {
        tracing::error!("notification stream cursor validation failed: {error}");
        StatusCode::SERVICE_UNAVAILABLE
    })
}

async fn notification_stream(
    State(state): State<AppState>,
    Extension(principal): Extension<VerifiedPrincipal>,
    headers: axum::http::HeaderMap,
) -> Result<Sse<impl tokio_stream::Stream<Item = Result<Event, Infallible>>>, Response> {
    let owner = canonical_owner(&principal, None)
        .map_err(|status| {
            (status, Json(serde_json::json!({"error": "forbidden"}))).into_response()
        })?
        .to_ascii_lowercase();
    let requested_cursor = headers
        .get("last-event-id")
        .map(|value| value.to_str().map(str::to_owned))
        .transpose()
        .map_err(|_| {
            (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({"error": "invalid_last_event_id"})),
            )
                .into_response()
        })?;
    state
        .stream_metrics
        .connections_total
        .fetch_add(1, Ordering::Relaxed);
    if requested_cursor.is_some() {
        state
            .stream_metrics
            .reconnects_total
            .fetch_add(1, Ordering::Relaxed);
    }
    if requested_cursor
        .as_deref()
        .is_some_and(|cursor| !valid_stream_cursor(cursor))
    {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({ "error": "invalid_last_event_id" })),
        )
            .into_response());
    }
    let cursor = match requested_cursor {
        Some(cursor) => Some(cursor),
        None => persisted_stream_cursor(&state.db, &owner)
            .await
            .map_err(|status| {
                (
                    status,
                    Json(serde_json::json!({"error": "stream_unavailable"})),
                )
                    .into_response()
            })?,
    };
    if let Some(cursor) = cursor.as_deref() {
        if !stream_cursor_belongs_to_owner(&state.db, &owner, cursor)
            .await
            .map_err(|status| {
                (
                    status,
                    Json(serde_json::json!({"error": "stream_unavailable"})),
                )
                    .into_response()
            })?
        {
            return Err((
                StatusCode::GONE,
                Json(serde_json::json!({ "error": "stream_cursor_expired" })),
            )
                .into_response());
        }
    }
    let permit = state
        .realtime_slots
        .clone()
        .try_acquire_owned()
        .map_err(|_| {
            (
                StatusCode::TOO_MANY_REQUESTS,
                Json(serde_json::json!({ "error": "stream_capacity_exhausted" })),
            )
                .into_response()
        })?;
    let db = state.db.clone();
    let stream_metrics = state.stream_metrics.clone();
    let realtime_notify = Arc::clone(&state.realtime_notify);
    let stream = async_stream::stream! {
        let _permit = permit;
        let mut cursor = cursor;
        loop {
            let rows = sqlx::query_as::<_, StreamNotificationRow>(
                "WITH cursor AS (SELECT n.created_at FROM public.notifications n LEFT JOIN public.notification_expirations x ON x.notification_id = n.id WHERE n.id = $2 AND (n.user_id = $1 OR (n.user_id IS NULL AND lower(n.recipient) = 'all')) AND (x.expires_at IS NULL OR x.expires_at > NOW())) SELECT n.id, n.title, n.body, n.notification_type, n.priority, n.data, n.action_url, COALESCE(e.read_at, n.read_at) AS read_at, n.created_at, x.expires_at FROM public.notifications n LEFT JOIN public.notification_engagement e ON e.notification_id = n.id AND e.owner_id = $1 LEFT JOIN public.notification_expirations x ON x.notification_id = n.id WHERE (n.user_id = $1 OR (n.user_id IS NULL AND lower(n.recipient) = 'all')) AND (x.expires_at IS NULL OR x.expires_at > NOW()) AND ($2::varchar IS NULL OR n.created_at > (SELECT created_at FROM cursor) OR (n.created_at = (SELECT created_at FROM cursor) AND n.id > $2::varchar)) ORDER BY n.created_at ASC, n.id ASC LIMIT $3",
            )
            .bind(&owner)
            .bind(cursor.as_deref())
            .bind(STREAM_BATCH_LIMIT)
            .fetch_all(&db)
            .await;
            let Ok(rows) = rows else {
                stream_metrics
                    .query_failures_total
                    .fetch_add(1, Ordering::Relaxed);
                tracing::error!("notification stream query failed");
                break;
            };
            if rows.is_empty() {
                tokio::select! {
                    _ = realtime_notify.notified() => {}
                    _ = tokio::time::sleep(STREAM_POLL_INTERVAL) => {}
                }
                continue;
            }
            for row in rows {
                let lag_seconds = chrono::Utc::now()
                    .signed_duration_since(row.created_at)
                    .num_seconds()
                    .max(0) as u64;
                stream_metrics
                    .lag_seconds_total
                    .fetch_add(lag_seconds, Ordering::Relaxed);
                stream_metrics
                    .lag_samples_total
                    .fetch_add(1, Ordering::Relaxed);
                let payload = serde_json::json!({
                    "id": row.id,
                    "title": row.title,
                    "body": row.body,
                    "notification_type": row.notification_type,
                    "priority": row.priority,
                    "data": row.data,
                    "action_url": row.action_url,
                    "read_at": row.read_at,
                    "created_at": row.created_at,
                    "expires_at": row.expires_at,
                });
                let Ok(data) = serde_json::to_string(&payload) else {
                    continue;
                };
                cursor = Some(payload["id"].as_str().unwrap_or_default().to_string());
                stream_metrics
                    .replayed_events_total
                    .fetch_add(1, Ordering::Relaxed);
                yield Ok(Event::default().id(cursor.clone().unwrap_or_default()).event("notification").data(data));
            }
        }
    };
    Ok(Sse::new(stream).keep_alive(
        KeepAlive::new()
            .interval(std::time::Duration::from_secs(20))
            .event(Event::default().data("ping")),
    ))
}

async fn acknowledge_stream(
    State(state): State<AppState>,
    Extension(principal): Extension<VerifiedPrincipal>,
    Json(request): Json<StreamAcknowledgementRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    if !valid_stream_cursor(&request.event_id) {
        return Err(StatusCode::BAD_REQUEST);
    }
    let owner = canonical_owner(&principal, None)?.to_ascii_lowercase();
    if !stream_cursor_belongs_to_owner(&state.db, &owner, &request.event_id).await? {
        return Err(StatusCode::NOT_FOUND);
    }
    sqlx::query(
        "INSERT INTO public.notification_replay_cursors (owner_id, stream, last_event_id) VALUES ($1, 'owner', $2) ON CONFLICT (owner_id, stream) DO UPDATE SET last_event_id = EXCLUDED.last_event_id, updated_at = NOW()",
    )
    .bind(owner)
    .bind(&request.event_id)
    .execute(&state.db)
    .await
    .map_err(|error| {
        tracing::error!("notification stream cursor write failed: {error}");
        StatusCode::SERVICE_UNAVAILABLE
    })?;
    Ok(Json(
        serde_json::json!({ "ok": true, "event_id": request.event_id }),
    ))
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct PushSubscriptionRequest {
    endpoint: String,
    p256dh: String,
    auth: String,
    #[serde(default)]
    user_agent: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct PushUnsubscribeRequest {
    endpoint: String,
}

#[derive(Debug, Serialize)]
struct PushStatusResponse {
    enabled: bool,
    subscribed: bool,
    public_key: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    subscription_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    created_at: Option<chrono::DateTime<chrono::Utc>>,
}

fn valid_push_token(value: &str, max_len: usize) -> bool {
    !value.is_empty()
        && value.len() <= max_len
        && value.bytes().all(|byte| {
            byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'+' | b'/' | b'=')
        })
}

fn valid_vapid_public_key(value: &str) -> bool {
    valid_push_token(value, 256)
}

fn valid_vapid_key_id(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 128
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-'))
}

/// Read VAPID signing material without logging or exposing it in readiness
/// responses. Both PEM and raw URL-safe base64 private keys are supported by
/// `web-push`; malformed keys fail closed and leave push delivery unavailable.
fn vapid_private_key_from_env() -> Option<Arc<Vec<u8>>> {
    vapid_private_key_from_env_name("NOTIFICATION_VAPID_PRIVATE_KEY")
}

fn vapid_private_key_from_env_name(name: &str) -> Option<Arc<Vec<u8>>> {
    let raw = std::env::var(name).ok()?;
    if raw.is_empty() || raw.len() > 8192 || raw.chars().any(|character| character.is_control()) {
        return None;
    }
    let valid = if raw.contains("BEGIN") {
        VapidSignatureBuilder::from_pem_no_sub(raw.as_bytes()).is_ok()
    } else {
        valid_push_token(&raw, 256) && VapidSignatureBuilder::from_base64_no_sub(&raw).is_ok()
    };
    valid.then(|| Arc::new(raw.into_bytes()))
}

fn vapid_private_key_matches_public(private_key: &[u8], public_key: &str) -> bool {
    let derived = if private_key.starts_with(b"-----BEGIN") {
        VapidSignatureBuilder::from_pem_no_sub(private_key)
            .ok()
            .map(|builder| builder.get_public_key())
    } else {
        std::str::from_utf8(private_key)
            .ok()
            .and_then(|encoded| VapidSignatureBuilder::from_base64_no_sub(encoded).ok())
            .map(|builder| builder.get_public_key())
    };
    derived.is_some_and(|key| URL_SAFE_NO_PAD.encode(key) == public_key)
}

fn vapid_is_configured(state: &AppState) -> bool {
    state.vapid_public_key.is_some() && state.vapid_private_key.is_some()
}

fn vapid_private_key_for_id<'a>(state: &'a AppState, key_id: &str) -> Option<&'a [u8]> {
    if key_id == state.vapid_key_id {
        return state.vapid_private_key.as_deref().map(Vec::as_slice);
    }
    if state.vapid_previous_key_id.as_deref() == Some(key_id) {
        return state
            .vapid_previous_private_key
            .as_deref()
            .map(Vec::as_slice);
    }
    None
}

fn valid_push_endpoint(value: &str) -> bool {
    value.len() <= 2048
        && reqwest::Url::parse(value).is_ok_and(|url| {
            url.scheme() == "https"
                && url.username().is_empty()
                && url.password().is_none()
                && url.query().is_none()
                && url.fragment().is_none()
        })
}

fn validate_push_subscription(request: &PushSubscriptionRequest) -> Result<(), StatusCode> {
    if !valid_push_endpoint(&request.endpoint)
        || !valid_push_token(&request.p256dh, 256)
        || !valid_push_token(&request.auth, 256)
        || request.user_agent.as_deref().is_some_and(|user_agent| {
            user_agent.is_empty()
                || user_agent.len() > 512
                || user_agent.chars().any(char::is_control)
        })
    {
        return Err(StatusCode::BAD_REQUEST);
    }
    Ok(())
}

fn push_subscription_id(endpoint: &str) -> String {
    format!("push_{:x}", Sha256::digest(endpoint.as_bytes()))
}

async fn active_push_subscription_identity(
    db: &sqlx::PgPool,
    owner: &str,
) -> Result<Option<(String, chrono::DateTime<chrono::Utc>)>, StatusCode> {
    sqlx::query_as::<_, (String, chrono::DateTime<chrono::Utc>)>(
        "SELECT endpoint, created_at FROM public.notification_push_subscriptions WHERE user_id = $1 AND revoked_at IS NULL ORDER BY created_at DESC, endpoint DESC LIMIT 1",
    )
    .bind(owner)
    .fetch_optional(db)
    .await
    .map_err(|error| {
        tracing::error!("notification push identity read failed: {error}");
        StatusCode::SERVICE_UNAVAILABLE
    })
}

fn push_status_identity(
    identity: Option<(String, chrono::DateTime<chrono::Utc>)>,
) -> (Option<String>, Option<chrono::DateTime<chrono::Utc>>) {
    identity
        .map(|(endpoint, created_at)| (Some(push_subscription_id(&endpoint)), Some(created_at)))
        .unwrap_or((None, None))
}

async fn push_status(
    State(state): State<AppState>,
    Extension(principal): Extension<VerifiedPrincipal>,
) -> Result<Json<PushStatusResponse>, StatusCode> {
    let owner = canonical_owner(&principal, None)?.to_ascii_lowercase();
    let identity = active_push_subscription_identity(&state.db, &owner).await?;
    let subscribed = identity.is_some();
    let (subscription_id, created_at) = push_status_identity(identity);
    Ok(Json(PushStatusResponse {
        enabled: vapid_is_configured(&state),
        subscribed,
        public_key: state.vapid_public_key.clone(),
        subscription_id,
        created_at,
    }))
}

async fn push_subscribe(
    State(state): State<AppState>,
    Extension(principal): Extension<VerifiedPrincipal>,
    Json(request): Json<PushSubscriptionRequest>,
) -> Result<Json<PushStatusResponse>, StatusCode> {
    validate_push_subscription(&request)?;
    let Some(public_key) = state.vapid_public_key.clone() else {
        return Err(StatusCode::SERVICE_UNAVAILABLE);
    };
    let owner = canonical_owner(&principal, None)?.to_ascii_lowercase();
    let mut transaction = state.db.begin().await.map_err(|error| {
        tracing::error!("notification push transaction failed: {error}");
        StatusCode::SERVICE_UNAVAILABLE
    })?;
    let existing_owner = sqlx::query_scalar::<_, Option<String>>(
        "SELECT user_id FROM public.notification_push_subscriptions WHERE endpoint = $1 FOR UPDATE",
    )
    .bind(&request.endpoint)
    .fetch_optional(&mut *transaction)
    .await
    .map_err(|error| {
        tracing::error!("notification push ownership check failed: {error}");
        StatusCode::SERVICE_UNAVAILABLE
    })?
    .flatten();
    if existing_owner
        .as_deref()
        .is_some_and(|existing| !existing.eq_ignore_ascii_case(&owner))
    {
        return Err(StatusCode::FORBIDDEN);
    }
    sqlx::query(
        "INSERT INTO public.notification_push_subscriptions (endpoint, user_id, p256dh, auth, user_agent, vapid_key_id, revoked_at) VALUES ($1, $2, $3, $4, $5, $6, NULL) ON CONFLICT (endpoint) DO UPDATE SET user_id = EXCLUDED.user_id, p256dh = EXCLUDED.p256dh, auth = EXCLUDED.auth, user_agent = EXCLUDED.user_agent, vapid_key_id = EXCLUDED.vapid_key_id, revoked_at = NULL",
    )
    .bind(&request.endpoint)
    .bind(&owner)
    .bind(&request.p256dh)
    .bind(&request.auth)
    .bind(&request.user_agent)
    .bind(&state.vapid_key_id)
    .execute(&mut *transaction)
    .await
    .map_err(|error| {
        tracing::error!("notification push subscription write failed: {error}");
        StatusCode::SERVICE_UNAVAILABLE
    })?;
    transaction.commit().await.map_err(|error| {
        tracing::error!("notification push subscription commit failed: {error}");
        StatusCode::SERVICE_UNAVAILABLE
    })?;
    let identity = active_push_subscription_identity(&state.db, &owner).await?;
    let (subscription_id, created_at) = push_status_identity(identity);
    Ok(Json(PushStatusResponse {
        enabled: true,
        subscribed: true,
        public_key: Some(public_key),
        subscription_id,
        created_at,
    }))
}

async fn push_unsubscribe(
    State(state): State<AppState>,
    Extension(principal): Extension<VerifiedPrincipal>,
    Json(request): Json<PushUnsubscribeRequest>,
) -> Result<Json<PushStatusResponse>, StatusCode> {
    if !valid_push_endpoint(&request.endpoint) {
        return Err(StatusCode::BAD_REQUEST);
    }
    let owner = canonical_owner(&principal, None)?.to_ascii_lowercase();
    sqlx::query(
        "UPDATE public.notification_push_subscriptions SET revoked_at = NOW() WHERE endpoint = $1 AND user_id = $2 AND revoked_at IS NULL",
    )
    .bind(&request.endpoint)
    .bind(owner.clone())
    .execute(&state.db)
    .await
    .map_err(|error| {
        tracing::error!("notification push subscription revoke failed: {error}");
        StatusCode::SERVICE_UNAVAILABLE
    })?;
    let identity = active_push_subscription_identity(&state.db, &owner).await?;
    let subscribed = identity.is_some();
    let (subscription_id, created_at) = push_status_identity(identity);
    Ok(Json(PushStatusResponse {
        enabled: vapid_is_configured(&state),
        subscribed,
        public_key: state.vapid_public_key.clone(),
        subscription_id,
        created_at,
    }))
}

async fn push_unsubscribe_all(
    State(state): State<AppState>,
    Extension(principal): Extension<VerifiedPrincipal>,
) -> Result<Json<PushStatusResponse>, StatusCode> {
    let owner = canonical_owner(&principal, None)?.to_ascii_lowercase();
    sqlx::query(
        "UPDATE public.notification_push_subscriptions SET revoked_at = NOW() WHERE user_id = $1 AND revoked_at IS NULL",
    )
    .bind(&owner)
    .execute(&state.db)
    .await
    .map_err(|error| {
        tracing::error!("notification push bulk revoke failed: {error}");
        StatusCode::SERVICE_UNAVAILABLE
    })?;
    Ok(Json(PushStatusResponse {
        enabled: vapid_is_configured(&state),
        subscribed: false,
        public_key: state.vapid_public_key.clone(),
        subscription_id: None,
        created_at: None,
    }))
}

fn validate_publish_request(request: &PublishNotificationRequest) -> Result<(), StatusCode> {
    let bounded = [
        (&request.event_id, 128),
        (&request.event_type, 100),
        (&request.aggregate_id, 128),
        (&request.idempotency_key, 255),
        (&request.notification_type, 50),
        (&request.priority, 20),
        (&request.title, 512),
        (&request.message, 16 * 1024),
    ];
    if bounded
        .iter()
        .any(|(value, max)| value.trim().is_empty() || value.len() > *max)
        || request
            .event_id
            .chars()
            .any(|character| character.is_control() || character.is_whitespace())
        || request
            .idempotency_key
            .chars()
            .any(|character| character.is_control() || character.is_whitespace())
        || !allowed_publish_event_type(&request.event_type)
        || !matches!(
            request.priority.as_str(),
            "low" | "normal" | "high" | "critical" | "urgent"
        )
        || request
            .notification_type
            .chars()
            .any(|character| character.is_control() || character.is_whitespace())
    {
        return Err(StatusCode::BAD_REQUEST);
    }
    validate_expiration(request.expires_at)?;
    let wallet = request.recipient_wallet_address.trim();
    if wallet != "all" && !valid_publish_wallet(wallet) {
        return Err(StatusCode::BAD_REQUEST);
    }
    if request
        .action_url
        .as_deref()
        .is_some_and(|url| !valid_action_url(url))
    {
        return Err(StatusCode::BAD_REQUEST);
    }
    // The two generic publisher event types have mutually exclusive target
    // semantics. Keeping this invariant at the service boundary prevents a
    // caller from silently turning a point notification into a broadcast (or
    // vice versa) by reusing an otherwise valid payload.
    if (request.event_type == "notification.send" && request.recipient_wallet_address == "all")
        || (request.event_type == "notification.broadcast"
            && request.recipient_wallet_address != "all")
        || (request.plan_id.is_none()
            && request.recipient_wallet_address == "all"
            && request.event_type != "notification.broadcast")
    {
        return Err(StatusCode::BAD_REQUEST);
    }
    if request.plan_id.is_some_and(|plan_id| plan_id.is_nil()) {
        return Err(StatusCode::BAD_REQUEST);
    }
    if request.plan_id.is_some()
        && (request.recipient_wallet_address != "all"
            || matches!(
                request.event_type.as_str(),
                "notification.send" | "notification.broadcast"
            ))
    {
        return Err(StatusCode::BAD_REQUEST);
    }
    Ok(())
}

fn valid_publish_wallet(wallet: &str) -> bool {
    wallet.len() == 42
        && wallet.starts_with("0x")
        && wallet[2..]
            .chars()
            .all(|character| character.is_ascii_hexdigit())
}

fn validate_expiration(
    expires_at: Option<chrono::DateTime<chrono::Utc>>,
) -> Result<(), StatusCode> {
    let Some(expires_at) = expires_at else {
        return Ok(());
    };
    let now = chrono::Utc::now();
    if expires_at <= now || expires_at > now + chrono::Duration::days(365) {
        return Err(StatusCode::BAD_REQUEST);
    }
    Ok(())
}

fn valid_action_url(url: &str) -> bool {
    !url.is_empty()
        && url.len() <= 2048
        && url.starts_with('/')
        && !url.starts_with("//")
        && !url.contains('\\')
        && !url.contains("://")
        && url
            .chars()
            .all(|character| !character.is_control() && !character.is_whitespace())
}

/// Map provider vocabulary to the durable job state used by reconciliation.
/// Keeping this mapping centralized ensures an accepted/delivered callback can
/// never be mistaken for a terminal failure (or vice versa) when callbacks are
/// reordered or replayed.
fn provider_event_target_state(event_type: &str) -> Option<&'static str> {
    match event_type {
        "accepted" | "delivered" => Some("provider_accepted"),
        "bounced" | "complained" | "failed" => Some("terminal_failed"),
        _ => None,
    }
}

fn validate_provider_event_request(request: &ProviderEventRequest) -> Result<(), StatusCode> {
    if request.provider.trim().is_empty()
        || request.provider.len() > 64
        || request.provider.chars().any(char::is_control)
        || request.provider_event_id.trim().is_empty()
        || request.provider_event_id.len() > 255
        || request.provider_event_id.chars().any(char::is_control)
        || request
            .provider_message_id
            .as_deref()
            .is_some_and(|message_id| {
                message_id.trim().is_empty()
                    || message_id.len() > 255
                    || message_id.chars().any(char::is_control)
            })
        || request.event_type.len() > 64
        || request
            .event_type
            .chars()
            .any(|character| character.is_control() || character.is_whitespace())
        || provider_event_target_state(&request.event_type).is_none()
        || request.job_id.as_deref().is_some_and(|job_id| {
            job_id.trim().is_empty() || job_id.len() > 128 || job_id.chars().any(char::is_control)
        })
        || !request.payload.is_object()
        || serde_json::to_vec(&request.payload)
            .map(|encoded| encoded.len() > 64 * 1024)
            .unwrap_or(true)
    {
        return Err(StatusCode::BAD_REQUEST);
    }
    Ok(())
}

fn publish_request_hash(request: &PublishNotificationRequest) -> Result<String, StatusCode> {
    let payload = serde_json::to_vec(request).map_err(|_| StatusCode::BAD_REQUEST)?;
    Ok(format!("{:x}", Sha256::digest(payload)))
}

fn allowed_publish_event_type(value: &str) -> bool {
    matches!(value, "notification.send" | "notification.broadcast")
        || [
            "payment.",
            "subscription.",
            "permission.",
            "chat.",
            "expiry.",
        ]
        .iter()
        .any(|prefix| value.starts_with(prefix) && value.len() > prefix.len())
}

#[derive(Debug, FromRow)]
struct PlanTargetMembership {
    wallet: String,
}

#[derive(Debug, FromRow)]
struct PlanTargetPolicy {
    wallet: String,
    channels: serde_json::Value,
    quiet_until: Option<chrono::DateTime<chrono::Utc>>,
}

/// Resolve only active plan membership from the core database. The query is
/// intentionally limited to the table owned by core; notification preferences
/// are resolved separately from the notification database below.
const PLAN_TARGET_MEMBERSHIPS_SQL: &str = r#"
SELECT DISTINCT lower(wpa.wallet_address) AS wallet
FROM public.wallet_plan_assignments wpa
WHERE wpa.plan_id = $1
  AND wpa.is_active = TRUE
  AND (wpa.expires_at IS NULL OR wpa.expires_at > NOW())
ORDER BY wallet
LIMIT $2
"#;

/// Resolve persisted notification policy from the notification database for a
/// bounded membership snapshot. Keeping this query on the notification pool
/// avoids assuming the isolated core database also owns preferences.
const PLAN_TARGET_POLICIES_SQL: &str = r#"
WITH policy AS (
    SELECT
        lower(user_id) AS wallet,
        COALESCE(channels, '{}'::jsonb) AS channels,
        quiet_hours,
        COALESCE(NULLIF(timezone, ''), 'UTC') AS timezone_name,
        timezone(COALESCE(NULLIF(timezone, ''), 'UTC'), NOW()) AS local_now
    FROM public.notification_preferences
    WHERE lower(user_id) = ANY($1::text[])
)
SELECT
    wallet,
    channels,
    CASE
        WHEN quiet_hours IS NULL
          OR COALESCE((quiet_hours->>'enabled')::boolean, TRUE) IS NOT TRUE
          OR NOT (
              ((quiet_hours->>'start')::time < (quiet_hours->>'end')::time
               AND local_now::time >= (quiet_hours->>'start')::time
               AND local_now::time < (quiet_hours->>'end')::time)
              OR
              ((quiet_hours->>'start')::time >= (quiet_hours->>'end')::time
               AND (local_now::time >= (quiet_hours->>'start')::time
                    OR local_now::time < (quiet_hours->>'end')::time))
          )
        THEN NULL::timestamptz
        ELSE
            make_timestamptz(
                EXTRACT(YEAR FROM local_now)::int,
                EXTRACT(MONTH FROM local_now)::int,
                EXTRACT(DAY FROM local_now)::int,
                0,
                0,
                0,
                timezone_name
            )
            + (EXTRACT(HOUR FROM (quiet_hours->>'end')::time) * INTERVAL '1 hour')
            + (EXTRACT(MINUTE FROM (quiet_hours->>'end')::time) * INTERVAL '1 minute')
            + CASE
                WHEN local_now::time >= (quiet_hours->>'start')::time
                 AND (quiet_hours->>'start')::time >= (quiet_hours->>'end')::time
                THEN INTERVAL '1 day'
                ELSE INTERVAL '0 day'
              END
    END AS quiet_until
FROM policy
ORDER BY wallet
"#;

fn plan_notification_id(request_hash: &str, wallet: &str) -> String {
    let digest = Sha256::digest(format!("{}:{}", request_hash, wallet).as_bytes());
    format!("0x{:x}", digest)
}

fn validate_plan_fanout_count(count: usize) -> Result<(), StatusCode> {
    if count > MAX_PLAN_FANOUT {
        Err(StatusCode::PAYLOAD_TOO_LARGE)
    } else {
        Ok(())
    }
}

async fn publish_notification(
    State(state): State<AppState>,
    Extension(principal): Extension<VerifiedPrincipal>,
    Json(request): Json<PublishNotificationRequest>,
) -> Result<Response, StatusCode> {
    let mut request = request;
    if request.recipient_wallet_address != "all" {
        request.recipient_wallet_address = request.recipient_wallet_address.to_ascii_lowercase();
    }
    validate_publish_request(&request)?;
    let delivery_policy = if request.plan_id.is_none() && request.recipient_wallet_address != "all"
    {
        Some(
            load_delivery_preference_policy(&state.db, Some(&request.recipient_wallet_address))
                .await?,
        )
    } else {
        None
    };
    let request_hash = publish_request_hash(&request)?;
    let payload = serde_json::to_value(&request).map_err(|_| StatusCode::BAD_REQUEST)?;
    let response_body = serde_json::json!({
        "event_id": request.event_id,
        "status": "accepted"
    });
    let mut realtime_owner = None;
    let mut realtime_notification_id = None;
    let mut transaction = state.db.begin().await.map_err(|error| {
        tracing::error!("notification publish transaction failed: {error}");
        StatusCode::SERVICE_UNAVAILABLE
    })?;

    if let Some(existing) = sqlx::query_as::<_, IdempotencyRecord>(
        "SELECT request_hash, response_status, response_body FROM public.notification_request_idempotency WHERE principal_subject = $1 AND event_type = $2 AND idempotency_key = $3",
    )
    .bind(&principal.subject)
    .bind(&request.event_type)
    .bind(&request.idempotency_key)
    .fetch_optional(&mut *transaction)
    .await
    .map_err(|error| {
        tracing::error!("notification idempotency lookup failed: {error}");
        StatusCode::SERVICE_UNAVAILABLE
    })? {
        if existing.request_hash != request_hash {
            return Err(StatusCode::CONFLICT);
        }
        let status = StatusCode::from_u16(existing.response_status as u16)
            .unwrap_or(StatusCode::ACCEPTED);
        return Ok((status, Json(existing.response_body)).into_response());
    }

    if let Some(existing) = sqlx::query_as::<_, InboxRecord>(
        "SELECT request_hash, state FROM public.notification_inbox WHERE principal_subject = $1 AND event_id = $2",
    )
    .bind(&principal.subject)
    .bind(&request.event_id)
    .fetch_optional(&mut *transaction)
    .await
    .map_err(|error| {
        tracing::error!("notification inbox lookup failed: {error}");
        StatusCode::SERVICE_UNAVAILABLE
    })? {
        if existing.request_hash != request_hash || existing.state == "rejected" {
            return Err(StatusCode::CONFLICT);
        }
        return Ok((StatusCode::ACCEPTED, Json(response_body)).into_response());
    }

    sqlx::query(
        "INSERT INTO public.notification_request_idempotency (principal_subject, event_type, idempotency_key, request_hash, response_status, response_body) VALUES ($1, $2, $3, $4, $5, $6)",
    )
    .bind(&principal.subject)
    .bind(&request.event_type)
    .bind(&request.idempotency_key)
    .bind(&request_hash)
    .bind(StatusCode::ACCEPTED.as_u16() as i16)
    .bind(&response_body)
    .execute(&mut *transaction)
    .await
    .map_err(|error| {
        tracing::error!("notification idempotency insert failed: {error}");
        StatusCode::CONFLICT
    })?;

    sqlx::query(
        "INSERT INTO public.notification_inbox (principal_subject, event_id, request_hash, payload) VALUES ($1, $2, $3, $4)",
    )
    .bind(&principal.subject)
    .bind(&request.event_id)
    .bind(&request_hash)
    .bind(&payload)
    .execute(&mut *transaction)
    .await
    .map_err(|error| {
        tracing::error!("notification inbox insert failed: {error}");
        StatusCode::CONFLICT
    })?;

    let outbox_inserted = sqlx::query(
        "INSERT INTO public.notification_outbox (event_id, event_type, aggregate_id, payload) VALUES ($1, $2, $3, $4) ON CONFLICT (event_id) DO NOTHING",
    )
    .bind(&request.event_id)
    .bind(&request.event_type)
    .bind(&request.aggregate_id)
    .bind(&payload)
    .execute(&mut *transaction)
    .await
    .map_err(|error| {
        tracing::error!("notification outbox insert failed: {error}");
        StatusCode::SERVICE_UNAVAILABLE
    })?;
    if outbox_inserted.rows_affected() != 1 {
        return Err(StatusCode::CONFLICT);
    }

    if let Some(plan_id) = request.plan_id {
        let plan_db = state
            .plan_db
            .as_ref()
            .ok_or(StatusCode::SERVICE_UNAVAILABLE)?;
        let memberships = sqlx::query_as::<_, PlanTargetMembership>(PLAN_TARGET_MEMBERSHIPS_SQL)
            .bind(plan_id)
            .bind((MAX_PLAN_FANOUT + 1) as i64)
            .fetch_all(plan_db)
            .await
            .map_err(|error| {
                tracing::error!("notification plan target lookup failed: {error}");
                StatusCode::SERVICE_UNAVAILABLE
            })?;
        validate_plan_fanout_count(memberships.len())?;
        let wallets = memberships
            .iter()
            .map(|membership| membership.wallet.clone())
            .collect::<Vec<_>>();
        let policies = if wallets.is_empty() {
            Vec::new()
        } else {
            sqlx::query_as::<_, PlanTargetPolicy>(PLAN_TARGET_POLICIES_SQL)
                .bind(&wallets)
                .fetch_all(&state.db)
                .await
                .map_err(|error| {
                    tracing::error!("notification plan policy lookup failed: {error}");
                    StatusCode::SERVICE_UNAVAILABLE
                })?
        };
        let policies_by_wallet = policies
            .into_iter()
            .map(|policy| (policy.wallet.clone(), policy))
            .collect::<HashMap<_, _>>();
        let has_recipients = !memberships.is_empty();
        for target in memberships {
            if !valid_publish_wallet(&target.wallet) {
                tracing::error!("notification plan target contains malformed wallet");
                return Err(StatusCode::SERVICE_UNAVAILABLE);
            }
            let policy = policies_by_wallet
                .get(&target.wallet)
                .map(|policy| (policy.channels.clone(), policy.quiet_until))
                .unwrap_or_else(|| (serde_json::json!({}), None));
            let notification_id = plan_notification_id(&request_hash, &target.wallet);
            let channel_enabled = notification_policy_allows(
                &policy.0,
                "in_app",
                &request.notification_type,
                &request.priority,
            );
            sqlx::query(
                "INSERT INTO public.notifications (id, user_id, channel, recipient, subject, body, data, status, error, title, notification_type, priority, action_url) VALUES ($1, $2, 'in_app', $2, NULL, $3, $4, $5, $6, $7, $8, $9, $10)",
            )
            .bind(&notification_id)
            .bind(&target.wallet)
            .bind(&request.message)
            .bind(request.data.clone())
            .bind(if channel_enabled { "pending" } else { "suppressed" })
            .bind((!channel_enabled).then_some("channel_disabled_by_preference"))
            .bind(&request.title)
            .bind(&request.notification_type)
            .bind(&request.priority)
            .bind(&request.action_url)
            .execute(&mut *transaction)
            .await
            .map_err(|error| {
                tracing::error!("notification plan materialization failed: {error}");
                StatusCode::SERVICE_UNAVAILABLE
            })?;

            if let Some(expires_at) = request.expires_at {
                sqlx::query(
                    "INSERT INTO public.notification_expirations (notification_id, expires_at) VALUES ($1, $2)",
                )
                .bind(&notification_id)
                .bind(expires_at)
                .execute(&mut *transaction)
                .await
                .map_err(|error| {
                    tracing::error!("notification plan expiry projection failed: {error}");
                    StatusCode::SERVICE_UNAVAILABLE
                })?;
            }

            if channel_enabled {
                let job_id = format!("{}:in_app", notification_id);
                let idempotency_key = format!("{}:{}:in_app", request_hash, target.wallet);
                sqlx::query(
                    "INSERT INTO public.notification_channel_jobs (id, source_event_id, notification_id, channel, recipient, idempotency_key, available_at) VALUES ($1, $2, $3, 'in_app', $4, $5, COALESCE($6, NOW()))",
                )
                .bind(&job_id)
                .bind(&request.event_id)
                .bind(&notification_id)
                .bind(&target.wallet)
                .bind(&idempotency_key)
                .bind(policy.1)
                .execute(&mut *transaction)
                .await
                .map_err(|error| {
                    tracing::error!("notification plan channel job enqueue failed: {error}");
                    StatusCode::SERVICE_UNAVAILABLE
                })?;
            }
        }
        if has_recipients {
            realtime_owner = Some("all".to_string());
            realtime_notification_id = Some(request_hash.clone());
        }
    }

    // A concrete wallet is safe to materialize at admission time because the
    // publisher supplied only a normalized owner address. Plan targets are
    // expanded above from the authoritative membership table; broadcasts use
    // one durable recipient='all' row below and never trust a caller list.
    if request.plan_id.is_none() && request.recipient_wallet_address != "all" {
        let notification_id = format!("0x{}", Uuid::new_v4().simple());
        realtime_owner = Some(request.recipient_wallet_address.clone());
        realtime_notification_id = Some(notification_id.clone());
        let channel_enabled = delivery_policy.as_ref().is_some_and(|policy| {
            notification_policy_allows(
                &policy.channels,
                "in_app",
                &request.notification_type,
                &request.priority,
            )
        });
        sqlx::query(
            "INSERT INTO public.notifications (id, user_id, channel, recipient, subject, body, data, status, error, title, notification_type, priority, action_url) VALUES ($1, $2, 'in_app', $2, NULL, $3, $4, $5, $6, $7, $8, $9, $10)",
        )
        .bind(&notification_id)
        .bind(&request.recipient_wallet_address)
        .bind(&request.message)
        .bind(request.data.clone())
        .bind(if channel_enabled { "pending" } else { "suppressed" })
        .bind((!channel_enabled).then_some("channel_disabled_by_preference"))
        .bind(&request.title)
        .bind(&request.notification_type)
        .bind(&request.priority)
        .bind(&request.action_url)
        .execute(&mut *transaction)
        .await
        .map_err(|error| {
            tracing::error!("notification materialization failed: {error}");
            StatusCode::SERVICE_UNAVAILABLE
        })?;

        if let Some(expires_at) = request.expires_at {
            sqlx::query(
                "INSERT INTO public.notification_expirations (notification_id, expires_at) VALUES ($1, $2)",
            )
            .bind(&notification_id)
            .bind(expires_at)
            .execute(&mut *transaction)
            .await
            .map_err(|error| {
                tracing::error!("notification expiry projection failed: {error}");
                StatusCode::SERVICE_UNAVAILABLE
            })?;
        }

        if channel_enabled {
            let job_id = format!("{}:in_app", &request_hash[..64]);
            sqlx::query(
                "INSERT INTO public.notification_channel_jobs (id, source_event_id, notification_id, channel, recipient, idempotency_key, available_at) VALUES ($1, $2, $3, 'in_app', $4, $5, COALESCE($6, NOW()))",
            )
            .bind(&job_id)
            .bind(&request.event_id)
            .bind(&notification_id)
            .bind(&request.recipient_wallet_address)
            .bind(format!("{}:in_app", &request_hash[..64]))
            .bind(delivery_policy.as_ref().and_then(|policy| policy.quiet_until))
            .execute(&mut *transaction)
            .await
            .map_err(|error| {
                tracing::error!("notification channel job enqueue failed: {error}");
                StatusCode::SERVICE_UNAVAILABLE
            })?;
        }
    }

    if request.plan_id.is_none() && request.recipient_wallet_address == "all" {
        // A single durable broadcast row gives owner streams a PostgreSQL
        // source of truth. Provider fanout remains a separate channel
        // resolver; this row never trusts a caller-provided recipient list
        // and never creates one provider job per owner.
        let notification_id = format!("0x{}", &request_hash[..64]);
        sqlx::query(BROADCAST_NOTIFICATION_INSERT_SQL)
            .bind(&notification_id)
            .bind(&request.message)
            .bind(request.data.clone())
            .bind(&request.title)
            .bind(&request.notification_type)
            .bind(&request.priority)
            .bind(&request.action_url)
            .execute(&mut *transaction)
            .await
            .map_err(|error| {
                tracing::error!("broadcast notification materialization failed: {error}");
                StatusCode::SERVICE_UNAVAILABLE
            })?;
        if let Some(expires_at) = request.expires_at {
            sqlx::query(
                "INSERT INTO public.notification_expirations (notification_id, expires_at) VALUES ($1, $2)",
            )
            .bind(&notification_id)
            .bind(expires_at)
            .execute(&mut *transaction)
            .await
            .map_err(|error| {
                tracing::error!("broadcast expiry projection failed: {error}");
                StatusCode::SERVICE_UNAVAILABLE
            })?;
        }
        realtime_owner = Some("all".to_string());
        realtime_notification_id = Some(notification_id);
    }

    transaction.commit().await.map_err(|error| {
        tracing::error!("notification publish commit failed: {error}");
        StatusCode::SERVICE_UNAVAILABLE
    })?;
    if let (Some(owner), Some(notification_id)) = (
        realtime_owner.as_deref(),
        realtime_notification_id.as_deref(),
    ) {
        let channel = if owner == "all" {
            "notifications:all".to_string()
        } else {
            realtime_wallet_channel(owner)
        };
        publish_realtime_wakeup(&state, &channel, notification_id).await;
    }
    Ok((StatusCode::ACCEPTED, Json(response_body)).into_response())
}

async fn record_provider_event(
    State(state): State<AppState>,
    Extension(_principal): Extension<VerifiedPrincipal>,
    headers: HeaderMap,
    body: Bytes,
) -> Result<Response, StatusCode> {
    if state.provider_signing_secrets.is_empty() {
        return Err(StatusCode::SERVICE_UNAVAILABLE);
    }
    let timestamp = headers
        .get("x-epsx-provider-timestamp")
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.parse::<i64>().ok())
        .ok_or(StatusCode::UNAUTHORIZED)?;
    let signature = headers
        .get("x-epsx-provider-signature")
        .and_then(|value| value.to_str().ok())
        .ok_or(StatusCode::UNAUTHORIZED)?;
    let now = chrono::Utc::now().timestamp();
    if !state
        .provider_signing_secrets
        .iter()
        .any(|secret| verify_provider_signature(secret, timestamp, &body, signature, now))
    {
        return Err(StatusCode::UNAUTHORIZED);
    }
    let request: ProviderEventRequest =
        serde_json::from_slice(&body).map_err(|_| StatusCode::BAD_REQUEST)?;
    validate_provider_event_request(&request)?;
    let target_state =
        provider_event_target_state(&request.event_type).ok_or(StatusCode::BAD_REQUEST)?;
    let mut transaction = state.db.begin().await.map_err(|error| {
        tracing::error!("notification provider-event transaction failed: {error}");
        StatusCode::SERVICE_UNAVAILABLE
    })?;

    // Providers commonly omit our internal job identifier from callbacks and
    // return only their message identifier. Resolve that identifier inside the
    // same transaction as event insertion so callback replay cannot race a
    // delivery transition and provider events remain useful for recovery.
    let resolved_job_id = if request.job_id.is_some() {
        request.job_id.clone()
    } else if let Some(provider_message_id) = request.provider_message_id.as_deref() {
        sqlx::query_scalar::<_, String>(
            "SELECT id FROM public.notification_channel_jobs WHERE provider_message_id = $1 ORDER BY updated_at DESC, id DESC LIMIT 1",
        )
        .bind(provider_message_id)
        .fetch_optional(&mut *transaction)
        .await
        .map_err(|error| {
            tracing::error!("notification provider job lookup failed: {error}");
            StatusCode::SERVICE_UNAVAILABLE
        })?
    } else {
        None
    };
    let inserted = sqlx::query(
        "INSERT INTO public.notification_provider_events (provider, provider_event_id, job_id, event_type, payload, occurred_at) VALUES ($1, $2, $3, $4, $5, $6) ON CONFLICT (provider, provider_event_id) DO NOTHING",
    )
    .bind(&request.provider)
    .bind(&request.provider_event_id)
    .bind(&resolved_job_id)
    .bind(&request.event_type)
    .bind(&request.payload)
    .bind(request.occurred_at)
    .execute(&mut *transaction)
    .await
    .map_err(|error| {
        tracing::error!("notification provider-event insert failed: {error}");
        StatusCode::SERVICE_UNAVAILABLE
    })?
    .rows_affected()
        == 1;

    if inserted {
        if let Some(job_id) = resolved_job_id.as_deref() {
            sqlx::query(
                "UPDATE public.notification_channel_jobs SET provider_message_id = COALESCE(provider_message_id, $2), state = CASE WHEN $3 = 'provider_accepted' AND state NOT IN ('dead_lettered', 'terminal_failed') THEN $3 WHEN $3 = 'terminal_failed' AND state NOT IN ('dead_lettered', 'terminal_failed', 'provider_accepted') THEN $3 ELSE state END, updated_at = NOW() WHERE id = $1",
            )
            .bind(job_id)
            .bind(&request.provider_message_id)
            .bind(target_state)
            .execute(&mut *transaction)
            .await
            .map_err(|error| {
                tracing::error!("notification provider job update failed: {error}");
                StatusCode::SERVICE_UNAVAILABLE
            })?;
            sqlx::query(
                "UPDATE public.notifications AS n SET status = CASE WHEN $2 = 'provider_accepted' THEN 'sent' WHEN $2 = 'terminal_failed' THEN 'failed' ELSE n.status END, sent_at = CASE WHEN $2 = 'provider_accepted' THEN COALESCE(n.sent_at, $4) ELSE n.sent_at END, error = CASE WHEN $2 = 'terminal_failed' THEN $3 ELSE NULL END WHERE n.id = (SELECT notification_id FROM public.notification_channel_jobs WHERE id = $1) AND n.status = 'pending' AND EXISTS (SELECT 1 FROM public.notification_channel_jobs WHERE id = $1 AND (($2 = 'provider_accepted' AND state = 'provider_accepted') OR ($2 = 'terminal_failed' AND state = 'terminal_failed'))) ",
            )
            .bind(job_id)
            .bind(target_state)
            .bind(request
                .payload
                .get("error")
                .and_then(serde_json::Value::as_str)
                .unwrap_or("provider_delivery_failed"))
            .bind(request.occurred_at.or(Some(chrono::Utc::now())))
            .execute(&mut *transaction)
            .await
            .map_err(|error| {
                tracing::error!("notification provider notification update failed: {error}");
                StatusCode::SERVICE_UNAVAILABLE
            })?;
        }
    }

    transaction.commit().await.map_err(|error| {
        tracing::error!("notification provider-event commit failed: {error}");
        StatusCode::SERVICE_UNAVAILABLE
    })?;
    let status = if inserted { "recorded" } else { "duplicate" };
    Ok((
        StatusCode::ACCEPTED,
        Json(ProviderEventResponse {
            provider: request.provider,
            provider_event_id: request.provider_event_id,
            status,
        }),
    )
        .into_response())
}

fn valid_provider_signing_secret(secret: &str) -> bool {
    (32..=256).contains(&secret.len())
        && secret
            .chars()
            .all(|character| !character.is_control() && !character.is_whitespace())
}

/// Load an active signing key plus optional rotation keys without ever
/// logging their values. The comma-separated form allows a previous key to
/// remain accepted during a bounded provider rotation window; duplicates and
/// malformed values are discarded, and an empty result fails closed at the
/// callback boundary.
fn provider_signing_secrets_from_env() -> Vec<Arc<Vec<u8>>> {
    let mut candidates = Vec::new();
    for variable in [
        "NOTIFICATION_PROVIDER_SIGNING_SECRET",
        "NOTIFICATION_PROVIDER_SIGNING_SECRET_PREVIOUS",
        "NOTIFICATION_PROVIDER_SIGNING_SECRETS",
    ] {
        let Ok(raw) = std::env::var(variable) else {
            continue;
        };
        candidates.extend(raw.split(',').map(str::trim).map(str::to_owned));
    }
    provider_signing_secrets_from_values(candidates)
}

fn provider_signing_secrets_from_values<I, S>(values: I) -> Vec<Arc<Vec<u8>>>
where
    I: IntoIterator<Item = S>,
    S: AsRef<str>,
{
    let mut secrets = Vec::new();
    for candidate in values
        .into_iter()
        .map(|value| value.as_ref().trim().to_owned())
    {
        if valid_provider_signing_secret(&candidate)
            && !secrets
                .iter()
                .any(|existing: &Arc<Vec<u8>>| existing.as_slice() == candidate.as_bytes())
        {
            secrets.push(Arc::new(candidate.as_bytes().to_vec()));
        }
    }
    secrets
}

fn verify_provider_signature(
    secret: &[u8],
    timestamp: i64,
    body: &[u8],
    signature: &str,
    now: i64,
) -> bool {
    const MAX_SKEW_SECONDS: i64 = 5 * 60;
    if secret.len() < 32
        || timestamp <= 0
        || now
            .checked_sub(timestamp)
            .and_then(|delta| delta.checked_abs())
            .is_none_or(|age| age > MAX_SKEW_SECONDS)
        || !signature.starts_with("v1=")
    {
        return false;
    }
    let Ok(expected) = hex::decode(&signature[3..]) else {
        return false;
    };
    if expected.len() != 32 {
        return false;
    }
    let mut input = Vec::with_capacity(32 + body.len());
    input.extend_from_slice(b"epsx.notification.provider.v1.");
    input.extend_from_slice(timestamp.to_string().as_bytes());
    input.push(b'.');
    input.extend_from_slice(body);
    let Ok(mut mac) = Hmac::<Sha256>::new_from_slice(secret) else {
        return false;
    };
    mac.update(&input);
    mac.verify_slice(&expected).is_ok()
}

async fn load_templates_to_hb(
    db: &sqlx::PgPool,
    hb: &mut Handlebars<'static>,
) -> Result<(), TemplateLoadError> {
    let rows = sqlx::query_as::<_, Template>(
        "SELECT id, name, channel, subject, body, variables, active, created_at, updated_at FROM public.templates WHERE active = true",
    )
    .fetch_all(db)
    .await?;
    for template in rows {
        if !template_uses_only_escaped_output(&template.body) {
            return Err(TemplateLoadError::RawOutput);
        }
        validate_template_content(
            template.subject.as_deref(),
            &template.body,
            &template.variables,
        )
        .map_err(|_| TemplateLoadError::InvalidContent)?;
        hb.register_template_string(&template.name, template.body)?;
    }
    Ok(())
}

async fn list_templates(
    State(state): State<AppState>,
) -> Result<Json<TemplateListResponse>, StatusCode> {
    let items: Vec<Template> = sqlx::query_as::<_, Template>(
        "SELECT id, name, channel, subject, body, variables, active, created_at, updated_at FROM public.templates ORDER BY name"
    )
    .fetch_all(&state.db)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let total: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM public.templates")
        .fetch_one(&state.db)
        .await
        .unwrap_or(0);
    Ok(Json(TemplateListResponse { items, total }))
}

async fn create_template(
    State(state): State<AppState>,
    Extension(principal): Extension<VerifiedPrincipal>,
    Json(req): Json<CreateTemplateRequest>,
) -> Result<Json<Template>, StatusCode> {
    validate_template_request(&req)?;
    let mut transaction = state
        .db
        .begin()
        .await
        .map_err(|_| StatusCode::SERVICE_UNAVAILABLE)?;
    let existing_id = sqlx::query_scalar::<_, Option<String>>(
        "SELECT id FROM public.templates WHERE name = $1 FOR UPDATE",
    )
    .bind(&req.name)
    .fetch_optional(&mut *transaction)
    .await
    .map_err(|_| StatusCode::SERVICE_UNAVAILABLE)?
    .flatten();
    let candidate_id = existing_id
        .clone()
        .unwrap_or_else(|| format!("0x{}", Uuid::new_v4().simple()));
    let id = sqlx::query_scalar::<_, String>(
        "INSERT INTO public.templates (id, name, channel, subject, body, variables, active) VALUES ($1, $2, $3, $4, $5, $6, true) ON CONFLICT (name) DO UPDATE SET channel = EXCLUDED.channel, body = EXCLUDED.body, subject = EXCLUDED.subject, variables = EXCLUDED.variables, active = true, updated_at = NOW() RETURNING id",
    )
    .bind(&candidate_id)
    .bind(&req.name)
    .bind(&req.channel)
    .bind(&req.subject)
    .bind(&req.body)
    .bind(req.variables.clone())
    .fetch_one(&mut *transaction)
    .await
    .map_err(|_| StatusCode::SERVICE_UNAVAILABLE)?;
    let previous_version: i32 = sqlx::query_scalar(
        "SELECT COALESCE(MAX(version), 0) FROM public.notification_template_versions WHERE template_id = $1",
    )
    .bind(&id)
    .fetch_one(&mut *transaction)
    .await
    .map_err(|_| StatusCode::SERVICE_UNAVAILABLE)?;
    let version = previous_version
        .checked_add(1)
        .ok_or(StatusCode::SERVICE_UNAVAILABLE)?;
    sqlx::query(
        "INSERT INTO public.notification_template_versions (id, template_id, version, subject, body, variables) VALUES ($1, $2, $3, $4, $5, $6)",
    )
    .bind(format!("{id}:v{version}"))
    .bind(&id)
    .bind(version)
    .bind(&req.subject)
    .bind(&req.body)
    .bind(req.variables.clone())
    .execute(&mut *transaction)
    .await
    .map_err(|_| StatusCode::SERVICE_UNAVAILABLE)?;
    let action = if previous_version > 0 {
        "updated"
    } else {
        "created"
    };
    sqlx::query(
        "INSERT INTO public.notification_template_audit (id, template_id, action, from_version, to_version, actor_subject, metadata) VALUES ($1, $2, $3, $4, $5, $6, $7)",
    )
    .bind(format!("{}:audit:v{}", id, version))
    .bind(&id)
    .bind(action)
    .bind((previous_version > 0).then_some(previous_version))
    .bind(version)
    .bind(&principal.subject)
    .bind(serde_json::json!({"template_name": req.name}))
    .execute(&mut *transaction)
    .await
    .map_err(|_| StatusCode::SERVICE_UNAVAILABLE)?;
    transaction
        .commit()
        .await
        .map_err(|_| StatusCode::SERVICE_UNAVAILABLE)?;
    state
        .templates
        .write()
        .await
        .register_template_string(&req.name, req.body.clone())
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let template: Template = sqlx::query_as::<_, Template>(
        "SELECT id, name, channel, subject, body, variables, active, created_at, updated_at FROM public.templates WHERE id = $1"
    )
    .bind(&id)
    .fetch_one(&state.db).await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(template))
}

async fn get_template(
    State(state): State<AppState>,
    AxPath(id): AxPath<String>,
) -> Result<Json<Template>, StatusCode> {
    let template: Template = sqlx::query_as::<_, Template>(
        "SELECT id, name, channel, subject, body, variables, active, created_at, updated_at FROM public.templates WHERE id = $1"
    )
    .bind(&id)
    .fetch_optional(&state.db).await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    .ok_or(StatusCode::NOT_FOUND)?;
    Ok(Json(template))
}

async fn preview_template(
    State(state): State<AppState>,
    AxPath(id): AxPath<String>,
    Json(request): Json<TemplatePreviewRequest>,
) -> Result<Json<TemplatePreviewResponse>, StatusCode> {
    let data_map = validate_template_preview_data(request.data)?;
    let template: Template = sqlx::query_as::<_, Template>(
        "SELECT id, name, channel, subject, body, variables, active, created_at, updated_at FROM public.templates WHERE id = $1 AND active = true",
    )
    .bind(&id)
    .fetch_optional(&state.db)
    .await
    .map_err(|_| StatusCode::SERVICE_UNAVAILABLE)?
    .ok_or(StatusCode::NOT_FOUND)?;
    validate_template_data(&template.variables, &data_map)?;
    let body = state
        .templates
        .read()
        .await
        .render(&template.name, &data_map)
        .map_err(|_| StatusCode::BAD_REQUEST)?;
    let version = sqlx::query_scalar::<_, i32>(
        "SELECT COALESCE(MAX(version), 0) FROM public.notification_template_versions WHERE template_id = $1",
    )
    .bind(&template.id)
    .fetch_one(&state.db)
    .await
    .map_err(|_| StatusCode::SERVICE_UNAVAILABLE)?;
    Ok(Json(TemplatePreviewResponse {
        template_id: template.id,
        version,
        subject: template.subject,
        body,
    }))
}

async fn rollback_template(
    State(state): State<AppState>,
    AxPath(id): AxPath<String>,
    Extension(principal): Extension<VerifiedPrincipal>,
    Json(request): Json<TemplateRollbackRequest>,
) -> Result<Json<Template>, StatusCode> {
    validate_template_rollback_request(&request)?;
    let mut transaction = state
        .db
        .begin()
        .await
        .map_err(|_| StatusCode::SERVICE_UNAVAILABLE)?;
    let template: Template = sqlx::query_as::<_, Template>(
        "SELECT id, name, channel, subject, body, variables, active, created_at, updated_at FROM public.templates WHERE id = $1 FOR UPDATE",
    )
    .bind(&id)
    .fetch_optional(&mut *transaction)
    .await
    .map_err(|_| StatusCode::SERVICE_UNAVAILABLE)?
    .ok_or(StatusCode::NOT_FOUND)?;
    let current_version: i32 = sqlx::query_scalar(
        "SELECT COALESCE(MAX(version), 0) FROM public.notification_template_versions WHERE template_id = $1",
    )
    .bind(&id)
    .fetch_one(&mut *transaction)
    .await
    .map_err(|_| StatusCode::SERVICE_UNAVAILABLE)?;
    let version: Option<(Option<String>, String, serde_json::Value)> = sqlx::query_as(
        "SELECT subject, body, variables FROM public.notification_template_versions WHERE template_id = $1 AND version = $2",
    )
    .bind(&id)
    .bind(request.version)
    .fetch_optional(&mut *transaction)
    .await
    .map_err(|_| StatusCode::SERVICE_UNAVAILABLE)?;
    let (subject, body, variables) = version.ok_or(StatusCode::NOT_FOUND)?;
    validate_template_content(subject.as_deref(), &body, &variables)?;
    sqlx::query(
        "UPDATE public.templates SET subject = $2, body = $3, variables = $4, active = true, updated_at = NOW() WHERE id = $1",
    )
    .bind(&id)
    .bind(&subject)
    .bind(&body)
    .bind(&variables)
    .execute(&mut *transaction)
    .await
    .map_err(|_| StatusCode::SERVICE_UNAVAILABLE)?;
    let next_version: i32 = sqlx::query_scalar(
        "SELECT COALESCE(MAX(version), 0) + 1 FROM public.notification_template_versions WHERE template_id = $1",
    )
    .bind(&id)
    .fetch_one(&mut *transaction)
    .await
    .map_err(|_| StatusCode::SERVICE_UNAVAILABLE)?;
    sqlx::query(
        "INSERT INTO public.notification_template_versions (id, template_id, version, subject, body, variables) VALUES ($1, $2, $3, $4, $5, $6)",
    )
    .bind(format!("{}:v{}", id, next_version))
    .bind(&id)
    .bind(next_version)
    .bind(&subject)
    .bind(&body)
    .bind(&variables)
    .execute(&mut *transaction)
    .await
    .map_err(|_| StatusCode::SERVICE_UNAVAILABLE)?;
    sqlx::query(
        "INSERT INTO public.notification_template_audit (id, template_id, action, from_version, to_version, actor_subject, metadata) VALUES ($1, $2, 'rollback', $3, $4, $5, $6)",
    )
    .bind(format!("{}:rollback:v{}", id, next_version))
    .bind(&id)
    .bind((current_version > 0).then_some(current_version))
    .bind(next_version)
    .bind(&principal.subject)
    .bind(serde_json::json!({
        "restored_version": request.version,
        "new_version": next_version,
    }))
    .execute(&mut *transaction)
    .await
    .map_err(|_| StatusCode::SERVICE_UNAVAILABLE)?;
    transaction
        .commit()
        .await
        .map_err(|_| StatusCode::SERVICE_UNAVAILABLE)?;
    state
        .templates
        .write()
        .await
        .register_template_string(&template.name, &body)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let current: Template = sqlx::query_as::<_, Template>(
        "SELECT id, name, channel, subject, body, variables, active, created_at, updated_at FROM public.templates WHERE id = $1",
    )
    .bind(&id)
    .fetch_one(&state.db)
    .await
    .map_err(|_| StatusCode::SERVICE_UNAVAILABLE)?;
    Ok(Json(current))
}

async fn list_template_audit(
    State(state): State<AppState>,
    AxPath(id): AxPath<String>,
    Extension(_principal): Extension<VerifiedPrincipal>,
) -> Result<Json<TemplateAuditResponse>, StatusCode> {
    let items = sqlx::query_as::<_, TemplateAuditEntry>(
        "SELECT id, template_id, action, from_version, to_version, actor_subject, metadata, created_at FROM public.notification_template_audit WHERE template_id = $1 ORDER BY created_at DESC, id DESC LIMIT 100",
    )
    .bind(&id)
    .fetch_all(&state.db)
    .await
    .map_err(|_| StatusCode::SERVICE_UNAVAILABLE)?;
    if items.len() > TEMPLATE_AUDIT_MAX_ITEMS || !items.iter().all(valid_template_audit_entry) {
        return Err(StatusCode::SERVICE_UNAVAILABLE);
    }
    Ok(Json(TemplateAuditResponse { items }))
}

async fn delete_template(
    State(state): State<AppState>,
    AxPath(id): AxPath<String>,
    Extension(principal): Extension<VerifiedPrincipal>,
) -> Result<StatusCode, StatusCode> {
    let mut transaction = state
        .db
        .begin()
        .await
        .map_err(|_| StatusCode::SERVICE_UNAVAILABLE)?;
    let name = sqlx::query_scalar::<_, Option<String>>(
        "SELECT name FROM public.templates WHERE id = $1 FOR UPDATE",
    )
    .bind(&id)
    .fetch_optional(&mut *transaction)
    .await
    .map_err(|_| StatusCode::SERVICE_UNAVAILABLE)?
    .flatten()
    .ok_or(StatusCode::NOT_FOUND)?;
    let current_version: i32 = sqlx::query_scalar(
        "SELECT COALESCE(MAX(version), 0) FROM public.notification_template_versions WHERE template_id = $1",
    )
    .bind(&id)
    .fetch_one(&mut *transaction)
    .await
    .map_err(|_| StatusCode::SERVICE_UNAVAILABLE)?;
    sqlx::query("UPDATE public.templates SET active = false, updated_at = NOW() WHERE id = $1")
        .bind(&id)
        .execute(&mut *transaction)
        .await
        .map_err(|_| StatusCode::SERVICE_UNAVAILABLE)?;
    sqlx::query(
        "INSERT INTO public.notification_template_audit (id, template_id, action, from_version, actor_subject, metadata) VALUES ($1, $2, 'deleted', $3, $4, $5)",
    )
    .bind(format!("{}:delete:{}", id, Uuid::new_v4().simple()))
    .bind(&id)
    .bind((current_version > 0).then_some(current_version))
    .bind(&principal.subject)
    .bind(serde_json::json!({"template_name": name}))
    .execute(&mut *transaction)
    .await
    .map_err(|_| StatusCode::SERVICE_UNAVAILABLE)?;
    transaction
        .commit()
        .await
        .map_err(|_| StatusCode::SERVICE_UNAVAILABLE)?;
    state.templates.write().await.unregister_template(&name);
    Ok(StatusCode::NO_CONTENT)
}

async fn send_notification(
    State(state): State<AppState>,
    Extension(principal): Extension<VerifiedPrincipal>,
    headers: HeaderMap,
    Json(mut req): Json<SendNotificationRequest>,
) -> Result<Response, StatusCode> {
    require_admin_notifications(&principal)?;
    if let Some(user_id) = req.user_id.as_mut() {
        *user_id = user_id.trim().to_ascii_lowercase();
    }
    req.recipient = req.recipient.trim().to_string();
    validate_send_request(&req)?;
    let request_id = request_id(&headers);
    let idempotency_key = headers
        .get("idempotency-key")
        .and_then(|value| value.to_str().ok())
        .filter(|value| valid_idempotency_key(value))
        .ok_or(StatusCode::BAD_REQUEST)?;
    let id = idempotent_notification_id(idempotency_key);
    let owner_id = req.user_id.as_deref().map(str::to_ascii_lowercase);
    let (subject, body) = if let Some(template_id) = &req.template_id {
        let template: Option<Template> = sqlx::query_as::<_, Template>(
            "SELECT id, name, channel, subject, body, variables, active, created_at, updated_at FROM public.templates WHERE id = $1 AND active = true"
        )
        .bind(template_id)
        .fetch_optional(&state.db).await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

        let t = template.ok_or(StatusCode::NOT_FOUND)?;
        if t.channel != req.channel {
            return Err(StatusCode::BAD_REQUEST);
        }
        let data_map: HashMap<String, serde_json::Value> =
            serde_json::from_value(req.data.clone().unwrap_or_else(|| serde_json::json!({})))
                .map_err(|_| StatusCode::BAD_REQUEST)?;
        validate_template_data(&t.variables, &data_map)?;
        let rendered = state
            .templates
            .read()
            .await
            .render(&t.name, &data_map)
            .map_err(|_| StatusCode::BAD_REQUEST)?;
        (t.subject, rendered)
    } else {
        (
            req.subject.clone(),
            req.body.clone().ok_or(StatusCode::BAD_REQUEST)?,
        )
    };

    let subject_str = subject.clone().unwrap_or_default();
    let body_str = body.clone();
    let delivery_policy = load_delivery_preference_policy(&state.db, owner_id.as_deref()).await?;
    let suppressed = !channel_preference_enabled(&delivery_policy.channels, &req.channel);
    let stored_status = if suppressed { "suppressed" } else { "pending" };
    let stored_error = suppressed.then_some("channel_disabled_by_preference");
    let payload = serde_json::json!({
        "notification_id": &id,
        "channel": &req.channel,
        "recipient": &req.recipient,
        "template_id": &req.template_id,
        "subject": &subject_str,
        "body": &body_str,
        "data": &req.data,
        "expires_at": &req.expires_at,
    });
    let mut transaction = state
        .db
        .begin()
        .await
        .map_err(|_| StatusCode::SERVICE_UNAVAILABLE)?;

    let existing = sqlx::query_as::<_, SendNotificationRecord>(
        "SELECT n.id, n.user_id, n.channel, n.recipient, n.template_id, n.subject, n.body, n.data, n.status, n.error, x.expires_at FROM public.notifications n LEFT JOIN public.notification_expirations x ON x.notification_id = n.id WHERE n.id = $1",
    )
    .bind(&id)
    .fetch_optional(&mut *transaction)
    .await
    .map_err(|_| StatusCode::SERVICE_UNAVAILABLE)?;
    if let Some(existing) = existing {
        if !same_send_request(&existing, &req, &subject_str, &body_str) {
            return Err(StatusCode::CONFLICT);
        }
        return Ok((
            StatusCode::ACCEPTED,
            Json(response_from_existing(&existing, request_id)),
        )
            .into_response());
    }

    let claimed = sqlx::query(
        "INSERT INTO public.notifications (id, user_id, channel, recipient, template_id, subject, body, data, status, error) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10) ON CONFLICT (id) DO NOTHING",
    )
    .bind(&id)
    .bind(&owner_id)
    .bind(&req.channel)
    .bind(&req.recipient)
    .bind(&req.template_id)
    .bind(&subject_str)
    .bind(&body_str)
    .bind(req.data.clone())
    .bind(stored_status)
    .bind(stored_error)
    .execute(&mut *transaction)
    .await
    .map_err(|_| StatusCode::SERVICE_UNAVAILABLE)?
    .rows_affected();
    if claimed == 0 {
        let existing = sqlx::query_as::<_, SendNotificationRecord>(
            "SELECT n.id, n.user_id, n.channel, n.recipient, n.template_id, n.subject, n.body, n.data, n.status, n.error, x.expires_at FROM public.notifications n LEFT JOIN public.notification_expirations x ON x.notification_id = n.id WHERE n.id = $1",
        )
        .bind(&id)
        .fetch_optional(&mut *transaction)
        .await
        .map_err(|_| StatusCode::SERVICE_UNAVAILABLE)?
        .ok_or(StatusCode::CONFLICT)?;
        if !same_send_request(&existing, &req, &subject_str, &body_str) {
            return Err(StatusCode::CONFLICT);
        }
        return Ok((
            StatusCode::ACCEPTED,
            Json(response_from_existing(&existing, request_id)),
        )
            .into_response());
    }
    if let Some(expires_at) = req.expires_at {
        sqlx::query(
            "INSERT INTO public.notification_expirations (notification_id, expires_at) VALUES ($1, $2)",
        )
        .bind(&id)
        .bind(expires_at)
        .execute(&mut *transaction)
        .await
        .map_err(|_| StatusCode::SERVICE_UNAVAILABLE)?;
    }
    sqlx::query(
        "INSERT INTO public.notification_outbox (event_id, event_type, aggregate_id, payload) VALUES ($1, 'notification.send', $2, $3)",
    )
    .bind(&id)
    .bind(owner_id.as_deref().unwrap_or(&req.recipient))
    .bind(&payload)
    .execute(&mut *transaction)
    .await
    .map_err(|_| StatusCode::CONFLICT)?;
    if !suppressed {
        sqlx::query(
            "INSERT INTO public.notification_channel_jobs (id, source_event_id, notification_id, channel, recipient, idempotency_key, available_at) VALUES ($1, $2, $3, $4, $5, $6, COALESCE($7, NOW()))",
        )
        .bind(format!("{}:{}", id, req.channel))
        .bind(&id)
        .bind(&id)
        .bind(&req.channel)
        .bind(&req.recipient)
        .bind(&id)
        .bind(delivery_policy.quiet_until)
        .execute(&mut *transaction)
        .await
        .map_err(|_| StatusCode::CONFLICT)?;
    }
    transaction
        .commit()
        .await
        .map_err(|_| StatusCode::SERVICE_UNAVAILABLE)?;
    if let Some(owner) = owner_id.as_deref() {
        let channel = realtime_wallet_channel(owner);
        publish_realtime_wakeup(&state, &channel, &id).await;
    }

    let (response_status, delivered) = send_response_status(stored_status);
    Ok((
        StatusCode::ACCEPTED,
        Json(SendNotificationResponse {
            id,
            status: response_status.to_string(),
            delivered,
            error: stored_error.map(str::to_string),
            request_id,
        }),
    )
        .into_response())
}

async fn send_email(
    state: &AppState,
    to: &str,
    subject: &str,
    body: &str,
    job_id: &str,
) -> (String, Option<String>, bool, Option<String>) {
    let smtp_opt = state.smtp.read().await.clone();
    let smtp = match smtp_opt {
        Some(s) => s,
        None => {
            return email_provider_unavailable();
        }
    };

    let provider_message_id = smtp_message_id(job_id);

    let from_addr = if state.from_name.is_empty() {
        state.from.clone()
    } else {
        format!("{} <{}>", state.from_name, state.from)
    };

    let to_parsed: Result<lettre::message::Mailbox, _> = to.parse();
    let from_parsed: Result<lettre::message::Mailbox, _> = from_addr.parse();

    if to_parsed.is_err() || from_parsed.is_err() {
        return (
            "failed".to_string(),
            Some("Invalid email address".to_string()),
            false,
            None,
        );
    }

    let email = Message::builder()
        .from(from_parsed.unwrap())
        .to(to_parsed.unwrap())
        .subject(subject)
        .message_id(Some(provider_message_id.clone()))
        .body(body.to_string());

    match email {
        Ok(msg) => match tokio::task::spawn_blocking(move || smtp.send(&msg)).await {
            Ok(Ok(_)) => ("sent".to_string(), None, true, Some(provider_message_id)),
            Ok(Err(_)) => (
                "failed".to_string(),
                Some("provider_send_failed".to_string()),
                false,
                None,
            ),
            Err(_) => (
                "failed".to_string(),
                Some("provider_worker_failed".to_string()),
                false,
                None,
            ),
        },
        Err(_) => (
            "failed".to_string(),
            Some("invalid_message".to_string()),
            false,
            None,
        ),
    }
}

fn smtp_message_id(job_id: &str) -> String {
    format!(
        "<epsx-{:x}@epsx.invalid>",
        Sha256::digest(job_id.as_bytes())
    )
}

fn email_provider_unavailable() -> (String, Option<String>, bool, Option<String>) {
    (
        "failed".to_string(),
        Some("provider_not_configured".to_string()),
        false,
        None,
    )
}

/// Deliver one standards-compliant Web Push message. The durable worker owns
/// retries and state transitions; this function only performs encryption,
/// VAPID signing, and one bounded provider call. Endpoint identifiers and
/// payload contents are deliberately absent from logs and error text.
struct PushDelivery<'a> {
    job_id: &'a str,
    vapid_key_id: &'a str,
    endpoint: &'a str,
    p256dh: &'a str,
    auth: &'a str,
    title: &'a str,
    body: &'a str,
    data: Option<&'a serde_json::Value>,
    action_url: Option<&'a str>,
}

async fn send_push(
    state: &AppState,
    delivery: PushDelivery<'_>,
) -> (String, Option<String>, Option<String>) {
    let Some(private_key) = vapid_private_key_for_id(state, delivery.vapid_key_id) else {
        return (
            "failed".to_string(),
            Some("provider_not_configured".to_string()),
            None,
        );
    };
    let payload = serde_json::json!({
        "title": delivery.title,
        "body": delivery.body,
        "data": delivery.data,
        "action_url": delivery.action_url.filter(|url| valid_action_url(url)),
    });
    let content = match serde_json::to_vec(&payload) {
        Ok(content) if content.len() <= 3800 => content,
        Ok(_) => {
            return (
                "failed".to_string(),
                Some("push_payload_too_large".to_string()),
                None,
            )
        }
        Err(_) => {
            return (
                "failed".to_string(),
                Some("push_payload_invalid".to_string()),
                None,
            )
        }
    };

    let subscription = SubscriptionInfo::new(delivery.endpoint, delivery.p256dh, delivery.auth);
    let mut signature = if private_key.starts_with(b"-----BEGIN") {
        match VapidSignatureBuilder::from_pem(private_key, &subscription) {
            Ok(builder) => builder,
            Err(error) => return push_build_failure(error),
        }
    } else {
        let Ok(encoded) = std::str::from_utf8(private_key) else {
            return (
                "failed".to_string(),
                Some("push_invalid_vapid_key".to_string()),
                None,
            );
        };
        match VapidSignatureBuilder::from_base64(encoded, &subscription) {
            Ok(builder) => builder,
            Err(error) => return push_build_failure(error),
        }
    };
    signature.add_claim("sub", format!("mailto:{}", state.from));
    let mut builder = WebPushMessageBuilder::new(&subscription);
    builder.set_ttl(86_400);
    builder.set_urgency(Urgency::Normal);
    builder.set_payload(ContentEncoding::Aes128Gcm, &content);
    let signature = match signature.build() {
        Ok(signature) => signature,
        Err(error) => return push_build_failure(error),
    };
    builder.set_vapid_signature(signature);
    let message = match builder.build() {
        Ok(message) => message,
        Err(error) => return push_build_failure(error),
    };

    let provider_message_id = push_message_id(delivery.job_id);
    let client = match reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(
            SMTP_TRANSPORT_TIMEOUT_SECONDS,
        ))
        .redirect(reqwest::redirect::Policy::none())
        .build()
    {
        Ok(client) => client,
        Err(_) => {
            return (
                "failed".to_string(),
                Some("push_client_unavailable".to_string()),
                None,
            )
        }
    };
    let mut request = client
        .post(message.endpoint.to_string())
        .header("TTL", message.ttl.to_string());
    if let Some(urgency) = message.urgency {
        request = request.header("Urgency", urgency.to_string());
    }
    if let Some(topic) = message.topic {
        request = request.header("Topic", topic);
    }
    if let Some(payload) = message.payload {
        request = request
            .header("Content-Encoding", payload.content_encoding.to_str())
            .header("Content-Length", payload.content.len().to_string())
            .header("Content-Type", "application/octet-stream");
        for (key, value) in payload.crypto_headers {
            request = request.header(key, value);
        }
        request = request.body(payload.content);
    } else {
        request = request.body(Vec::new());
    }
    let response = match request.send().await {
        Ok(response) => response,
        Err(_) => {
            return (
                "failed".to_string(),
                Some("push_provider_failed".to_string()),
                None,
            )
        }
    };
    let status = response.status();
    if status.is_success() {
        return ("sent".to_string(), None, Some(provider_message_id));
    }
    if status == reqwest::StatusCode::GONE || status == reqwest::StatusCode::NOT_FOUND {
        if let Err(error) = sqlx::query(
            "UPDATE public.notification_push_subscriptions SET revoked_at = NOW() WHERE endpoint = $1 AND revoked_at IS NULL",
        )
        .bind(delivery.endpoint)
        .execute(&state.db)
        .await
        {
            tracing::error!("notification push endpoint revocation failed: {error}");
        }
        return (
            "failed".to_string(),
            Some("push_endpoint_revoked".to_string()),
            None,
        );
    }
    let error = if status == reqwest::StatusCode::PAYLOAD_TOO_LARGE {
        "push_payload_too_large"
    } else if status.is_server_error() || status == reqwest::StatusCode::TOO_MANY_REQUESTS {
        "push_provider_failed"
    } else {
        "push_provider_rejected"
    };
    ("failed".to_string(), Some(error.to_string()), None)
}

fn push_build_failure(error: WebPushError) -> (String, Option<String>, Option<String>) {
    (
        "failed".to_string(),
        Some(push_error_code(&error).to_string()),
        None,
    )
}

fn push_error_code(error: &WebPushError) -> &'static str {
    match error {
        WebPushError::EndpointNotFound(_) | WebPushError::EndpointNotValid(_) => {
            "push_endpoint_revoked"
        }
        WebPushError::PayloadTooLarge => "push_payload_too_large",
        WebPushError::InvalidUri
        | WebPushError::InvalidCryptoKeys
        | WebPushError::MissingCryptoKeys
        | WebPushError::InvalidClaims
        | WebPushError::InvalidTtl
        | WebPushError::InvalidTopic => "push_invalid_subscription",
        WebPushError::Unauthorized(_)
        | WebPushError::BadRequest(_)
        | WebPushError::NotImplemented(_)
        | WebPushError::Other(_) => "push_provider_rejected",
        WebPushError::ServerError { .. }
        | WebPushError::Unspecified
        | WebPushError::Io(_)
        | WebPushError::InvalidResponse
        | WebPushError::ResponseTooLarge => "push_provider_failed",
        WebPushError::InvalidPackageName => "push_invalid_subscription",
    }
}

fn push_message_id(job_id: &str) -> String {
    format!(
        "<epsx-push-{:x}@epsx.invalid>",
        Sha256::digest(job_id.as_bytes())
    )
}

fn validate_send_request(request: &SendNotificationRequest) -> Result<(), StatusCode> {
    let owner = request
        .user_id
        .as_deref()
        .filter(|value| valid_wallet_address(value))
        .ok_or(StatusCode::BAD_REQUEST)?;

    if !matches!(request.channel.as_str(), "email" | "in_app" | "push")
        || !bounded_control_free(&request.recipient, MAX_RECIPIENT_CHARS)
        || request.recipient == "all"
        || (request.channel == "email" && !valid_email_recipient(&request.recipient))
        || (request.channel == "in_app" && !request.recipient.eq_ignore_ascii_case(owner))
        || (request.channel == "push" && !valid_push_recipient(&request.recipient))
        || request
            .template_id
            .as_deref()
            .is_some_and(|value| value.len() > 66 || value.chars().any(char::is_control))
        || request
            .subject
            .as_deref()
            .is_some_and(|value| !bounded_control_free(value, MAX_SUBJECT_CHARS))
        || request
            .body
            .as_deref()
            .is_some_and(|value| !bounded_control_free(value, MAX_BODY_CHARS))
        || request.data.as_ref().is_some_and(|value| {
            !value.is_object()
                || serde_json::to_vec(value)
                    .map(|encoded| encoded.len() > MAX_DATA_BYTES)
                    .unwrap_or(true)
        })
        || (request.template_id.is_some() && (request.subject.is_some() || request.body.is_some()))
        || (request.template_id.is_none()
            && request
                .body
                .as_deref()
                .is_none_or(|body| body.trim().is_empty()))
    {
        return Err(StatusCode::BAD_REQUEST);
    }
    validate_expiration(request.expires_at)?;
    Ok(())
}

fn valid_wallet_address(value: &str) -> bool {
    value.len() == 42
        && value.starts_with("0x")
        && value[2..].bytes().all(|byte| byte.is_ascii_hexdigit())
}

fn valid_email_recipient(value: &str) -> bool {
    let mut parts = value.split('@');
    let Some(local) = parts.next() else {
        return false;
    };
    let Some(domain) = parts.next() else {
        return false;
    };
    !local.is_empty()
        && !domain.is_empty()
        && parts.next().is_none()
        && !local.starts_with('.')
        && !local.ends_with('.')
        && !domain.starts_with('.')
        && !domain.ends_with('.')
        && !domain.contains("..")
}

fn valid_push_recipient(value: &str) -> bool {
    value.len() <= 2048
        && value.starts_with("https://")
        && !value.contains('\\')
        && !value.chars().any(|character| character.is_whitespace())
        && reqwest::Url::parse(value).is_ok_and(|url| {
            url.scheme() == "https"
                && url.host_str().is_some()
                && url.username().is_empty()
                && url.password().is_none()
        })
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct OwnerNotificationQuery {
    status: Option<String>,
    notification_type: Option<String>,
    priority: Option<String>,
    start_date: Option<chrono::DateTime<chrono::Utc>>,
    end_date: Option<chrono::DateTime<chrono::Utc>>,
    limit: i64,
    offset: i64,
}

fn parse_owner_filter_text(
    params: &std::collections::HashMap<String, String>,
    key: &str,
    max_bytes: usize,
) -> Result<Option<String>, StatusCode> {
    let Some(value) = params.get(key) else {
        return Ok(None);
    };
    if value.is_empty()
        || value.len() > max_bytes
        || value
            .chars()
            .any(|character| character.is_control() || character.is_whitespace())
    {
        return Err(StatusCode::BAD_REQUEST);
    }
    Ok(Some(value.clone()))
}

fn parse_owner_filter_date(
    params: &std::collections::HashMap<String, String>,
    key: &str,
) -> Result<Option<chrono::DateTime<chrono::Utc>>, StatusCode> {
    params
        .get(key)
        .map(|value| {
            chrono::DateTime::parse_from_rfc3339(value)
                .map(|date| date.with_timezone(&chrono::Utc))
                .map_err(|_| StatusCode::BAD_REQUEST)
        })
        .transpose()
}

fn parse_owner_notification_query(
    params: &std::collections::HashMap<String, String>,
) -> Result<OwnerNotificationQuery, StatusCode> {
    if params.keys().any(|key| {
        !matches!(
            key.as_str(),
            "user_id"
                | "status"
                | "limit"
                | "offset"
                | "type"
                | "notification_type"
                | "priority"
                | "start_date"
                | "end_date"
        )
    }) {
        return Err(StatusCode::BAD_REQUEST);
    }
    if params.contains_key("type") && params.contains_key("notification_type") {
        return Err(StatusCode::BAD_REQUEST);
    }
    let status = params.get("status").cloned();
    if status.as_deref().is_some_and(|status| {
        !matches!(
            status,
            "pending" | "sent" | "failed" | "suppressed" | "read" | "unread" | "all"
        )
    }) {
        return Err(StatusCode::BAD_REQUEST);
    }
    let status = status.filter(|status| status != "all");
    let notification_type = parse_owner_filter_text(params, "type", 64)?
        .or(parse_owner_filter_text(params, "notification_type", 64)?);
    let priority = parse_owner_filter_text(params, "priority", 32)?;
    let start_date = parse_owner_filter_date(params, "start_date")?;
    let end_date = parse_owner_filter_date(params, "end_date")?;
    if start_date
        .zip(end_date)
        .is_some_and(|(start, end)| start > end)
    {
        return Err(StatusCode::BAD_REQUEST);
    }
    let limit = match params.get("limit") {
        None => 50,
        Some(value) => value
            .parse::<i64>()
            .ok()
            .filter(|value| (1..=100).contains(value))
            .ok_or(StatusCode::BAD_REQUEST)?,
    };
    let offset = match params.get("offset") {
        None => 0,
        Some(value) => value
            .parse::<i64>()
            .ok()
            .filter(|value| (0..=1_000_000).contains(value))
            .ok_or(StatusCode::BAD_REQUEST)?,
    };
    Ok(OwnerNotificationQuery {
        status,
        notification_type,
        priority,
        start_date,
        end_date,
        limit,
        offset,
    })
}

async fn list_notifications(
    State(state): State<AppState>,
    Extension(principal): Extension<VerifiedPrincipal>,
    axum::extract::Query(params): axum::extract::Query<std::collections::HashMap<String, String>>,
) -> Result<Json<NotificationListResponse>, StatusCode> {
    let user_id = canonical_owner(&principal, params.get("user_id").map(String::as_str))?;
    let query = parse_owner_notification_query(&params)?;
    let items: Vec<Notification> = sqlx::query_as(&format!(
        "SELECT {OWNER_NOTIFICATION_SELECT_FIELDS} {OWNER_NOTIFICATION_JOIN} WHERE {OWNER_NOTIFICATION_SCOPE_SQL} {OWNER_NOTIFICATION_FILTER_SQL} ORDER BY n.created_at DESC, n.id DESC LIMIT $7 OFFSET $8"
    ))
    .bind(&user_id)
    .bind(query.status.as_deref())
    .bind(query.notification_type.as_deref())
    .bind(query.priority.as_deref())
    .bind(query.start_date)
    .bind(query.end_date)
    .bind(query.limit)
    .bind(query.offset)
    .fetch_all(&state.db)
    .await
    .map_err(|e| {
        tracing::error!("list query failed: {e}");
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    let total: i64 = require_owner_notification_total(
        sqlx::query_scalar(&format!(
            "SELECT COUNT(*) FROM public.notifications n LEFT JOIN public.notification_engagement e ON e.notification_id = n.id AND e.owner_id = $1 LEFT JOIN public.notification_expirations x ON x.notification_id = n.id WHERE {OWNER_NOTIFICATION_SCOPE_SQL} {OWNER_NOTIFICATION_FILTER_SQL}"
        ))
        .bind(&user_id)
        .bind(query.status.as_deref())
        .bind(query.notification_type.as_deref())
        .bind(query.priority.as_deref())
        .bind(query.start_date)
        .bind(query.end_date)
        .fetch_one(&state.db)
        .await,
    )?;
    Ok(Json(NotificationListResponse { items, total }))
}

fn require_owner_notification_total(result: Result<i64, sqlx::Error>) -> Result<i64, StatusCode> {
    result.map_err(|error| {
        tracing::error!("owner notification count query failed: {error}");
        StatusCode::INTERNAL_SERVER_ERROR
    })
}

async fn list_admin_notifications(
    State(state): State<AppState>,
    Extension(principal): Extension<VerifiedPrincipal>,
    RawQuery(raw_query): RawQuery,
) -> Result<Json<AdminNotificationListResponse>, StatusCode> {
    require_admin_notifications(&principal)?;
    let query = AdminNotificationQuery::parse(raw_query.as_deref())?;
    let mut transaction = state
        .db
        .begin()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    sqlx::query("SET TRANSACTION ISOLATION LEVEL REPEATABLE READ, READ ONLY")
        .execute(&mut *transaction)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let total: i64 = sqlx::query_scalar(ADMIN_NOTIFICATION_COUNT_SQL)
        .bind(query.status.as_deref())
        .bind(query.notification_type.as_deref())
        .bind(query.priority.as_deref())
        .bind(query.wallet_address.as_deref())
        .fetch_one(&mut *transaction)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let rows: Vec<AdminNotificationRow> = sqlx::query_as(ADMIN_NOTIFICATION_LIST_SQL)
        .bind(query.limit)
        .bind(query.offset)
        .bind(query.status.as_deref())
        .bind(query.notification_type.as_deref())
        .bind(query.priority.as_deref())
        .bind(query.wallet_address.as_deref())
        .fetch_all(&mut *transaction)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    transaction
        .commit()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    if !admin_notification_cardinality_is_valid(total, query.limit, query.offset, rows.len()) {
        tracing::error!(
            total,
            limit = query.limit,
            offset = query.offset,
            item_count = rows.len(),
            "admin notification inventory returned an impossible page"
        );
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }

    let items = rows
        .into_iter()
        .map(project_admin_notification)
        .collect::<Option<Vec<_>>>()
        .ok_or_else(|| {
            tracing::error!("admin notification inventory contains invalid stored fields");
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    Ok(Json(AdminNotificationListResponse {
        items,
        total,
        limit: query.limit,
        offset: query.offset,
    }))
}

async fn admin_metrics(
    State(state): State<AppState>,
    Extension(_principal): Extension<VerifiedPrincipal>,
) -> Result<Json<NotificationMetricsResponse>, StatusCode> {
    type NotificationMetricsRow = (
        i64,
        Option<i64>,
        i64,
        i64,
        i64,
        i64,
        i64,
        i64,
        i64,
        i64,
        i64,
        Option<i64>,
    );

    let (
        queue_depth,
        queue_age_seconds,
        suppressed,
        retry_wait,
        terminal_failed,
        dead_lettered,
        provider_accepted,
        attempting,
        provider_events,
        delivery_attempts,
        replay_cursors,
        replay_cursor_age_seconds,
    ): NotificationMetricsRow = sqlx::query_as(
        "SELECT COUNT(*) FILTER (WHERE state IN ('queued', 'leased', 'attempting', 'retry_wait'))::bigint, GREATEST(0, EXTRACT(EPOCH FROM (NOW() - MIN(available_at) FILTER (WHERE state IN ('queued', 'leased', 'attempting', 'retry_wait')))))::bigint, (SELECT COUNT(*) FROM public.notifications WHERE status = 'suppressed')::bigint, COUNT(*) FILTER (WHERE state = 'retry_wait')::bigint, COUNT(*) FILTER (WHERE state = 'terminal_failed')::bigint, COUNT(*) FILTER (WHERE state = 'dead_lettered')::bigint, COUNT(*) FILTER (WHERE state = 'provider_accepted')::bigint, COUNT(*) FILTER (WHERE state = 'attempting')::bigint, (SELECT COUNT(*) FROM public.notification_provider_events)::bigint, (SELECT COUNT(*) FROM public.notification_delivery_attempts)::bigint, (SELECT COUNT(*) FROM public.notification_replay_cursors)::bigint, (SELECT GREATEST(0, EXTRACT(EPOCH FROM (NOW() - MIN(updated_at))))::bigint FROM public.notification_replay_cursors) FROM public.notification_channel_jobs",
    )
        .fetch_one(&state.db)
        .await
        .map_err(|error| {
            tracing::error!("notification metrics query failed: {error}");
            StatusCode::SERVICE_UNAVAILABLE
        })?;
    let channel_rows = sqlx::query_as::<_, (String, i64)>(
        "SELECT channel, COUNT(*)::bigint FROM public.notification_channel_jobs GROUP BY channel ORDER BY channel",
    )
    .fetch_all(&state.db)
    .await
    .map_err(|error| {
        tracing::error!("notification channel metrics query failed: {error}");
        StatusCode::SERVICE_UNAVAILABLE
    })?;
    let channel_outcomes = project_channel_outcomes(channel_rows)?;
    let stream_lag_samples = state
        .stream_metrics
        .lag_samples_total
        .load(Ordering::Relaxed);
    let stream_lag_seconds = (stream_lag_samples > 0).then(|| {
        state
            .stream_metrics
            .lag_seconds_total
            .load(Ordering::Relaxed)
            / stream_lag_samples
    });
    Ok(Json(NotificationMetricsResponse {
        queue_depth,
        queue_age_seconds,
        suppressed,
        retry_wait,
        terminal_failed,
        dead_lettered,
        provider_accepted,
        attempting,
        channel_outcomes,
        provider_events,
        delivery_attempts,
        replay_cursors,
        replay_cursor_age_seconds,
        active_streams: MAX_REALTIME_CONNECTIONS
            .saturating_sub(state.realtime_slots.available_permits()),
        stream_connections_total: state
            .stream_metrics
            .connections_total
            .load(Ordering::Relaxed),
        stream_reconnects_total: state
            .stream_metrics
            .reconnects_total
            .load(Ordering::Relaxed),
        stream_replayed_events_total: state
            .stream_metrics
            .replayed_events_total
            .load(Ordering::Relaxed),
        stream_lag_seconds,
        stream_query_failures_total: state
            .stream_metrics
            .query_failures_total
            .load(Ordering::Relaxed),
    }))
}

fn project_channel_outcomes(rows: Vec<(String, i64)>) -> Result<BTreeMap<String, i64>, StatusCode> {
    let mut outcomes = BTreeMap::new();
    for (channel, count) in rows {
        if !matches!(channel.as_str(), "email" | "in_app" | "push") || count < 0 {
            return Err(StatusCode::SERVICE_UNAVAILABLE);
        }
        if outcomes.insert(channel, count).is_some() {
            return Err(StatusCode::SERVICE_UNAVAILABLE);
        }
    }
    Ok(outcomes)
}

async fn redrive_dead_letter(
    State(state): State<AppState>,
    Extension(_principal): Extension<VerifiedPrincipal>,
    AxPath(id): AxPath<String>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match DeliveryWorker::new(state.db.clone()).redrive(&id).await {
        Ok(()) => Ok(Json(
            serde_json::json!({ "job_id": id, "status": "queued" }),
        )),
        Err(epsx_notification::delivery::DeliveryWorkerError::Transition) => {
            Err(StatusCode::NOT_FOUND)
        }
        Err(epsx_notification::delivery::DeliveryWorkerError::Database(error)) => {
            tracing::error!("notification dead-letter redrive failed: {error}");
            Err(StatusCode::SERVICE_UNAVAILABLE)
        }
    }
}

fn valid_admin_notification_id(value: &str) -> bool {
    (1..=66).contains(&value.len())
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_'))
}

async fn admin_mark_read(
    State(state): State<AppState>,
    Extension(principal): Extension<VerifiedPrincipal>,
    AxPath(id): AxPath<String>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    require_admin_notifications(&principal)?;
    if !valid_admin_notification_id(&id) {
        return Err(StatusCode::BAD_REQUEST);
    }
    let result = sqlx::query(
        "UPDATE public.notifications SET read_at = NOW(), status = CASE WHEN status = 'pending' THEN 'sent' ELSE status END WHERE id = $1",
    )
    .bind(&id)
    .execute(&state.db)
    .await
    .map_err(|_| StatusCode::SERVICE_UNAVAILABLE)?;
    if result.rows_affected() == 0 {
        return Err(StatusCode::NOT_FOUND);
    }
    Ok(Json(serde_json::json!({ "id": id, "read": true })))
}

async fn admin_delete_notification(
    State(state): State<AppState>,
    Extension(principal): Extension<VerifiedPrincipal>,
    AxPath(id): AxPath<String>,
) -> Result<StatusCode, StatusCode> {
    require_admin_notifications(&principal)?;
    if !valid_admin_notification_id(&id) {
        return Err(StatusCode::BAD_REQUEST);
    }
    let result = sqlx::query("DELETE FROM public.notifications WHERE id = $1")
        .bind(&id)
        .execute(&state.db)
        .await
        .map_err(|_| StatusCode::SERVICE_UNAVAILABLE)?;
    if result.rows_affected() == 0 {
        return Err(StatusCode::NOT_FOUND);
    }
    Ok(StatusCode::NO_CONTENT)
}

async fn get_notification(
    State(state): State<AppState>,
    Extension(principal): Extension<VerifiedPrincipal>,
    AxPath(id): AxPath<String>,
) -> Result<Json<Notification>, StatusCode> {
    let owner = canonical_owner(&principal, None)?;
    let n: Notification = sqlx::query_as::<_, Notification>(
        "SELECT n.id, n.user_id, n.channel, n.recipient, n.template_id, n.subject, n.body, n.data, n.status, n.error, n.sent_at, n.created_at, COALESCE(e.read_at, n.read_at) AS read_at, e.clicked_at, n.title, n.notification_type, n.priority, n.action_url, x.expires_at FROM public.notifications n LEFT JOIN public.notification_engagement e ON e.notification_id = n.id AND e.owner_id = $2 LEFT JOIN public.notification_expirations x ON x.notification_id = n.id WHERE n.id = $1 AND (n.user_id = $2 OR (n.user_id IS NULL AND lower(n.recipient) = 'all')) AND (x.expires_at IS NULL OR x.expires_at > NOW())"
    )
    .bind(&id)
    .bind(&owner)
    .fetch_optional(&state.db).await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    .ok_or(StatusCode::NOT_FOUND)?;
    Ok(Json(n))
}

#[derive(Serialize, Deserialize)]
struct MarkReadResponse {
    id: String,
    read_at: chrono::DateTime<chrono::Utc>,
}

async fn mark_read(
    State(state): State<AppState>,
    Extension(principal): Extension<VerifiedPrincipal>,
    AxPath(id): AxPath<String>,
) -> Result<Json<MarkReadResponse>, StatusCode> {
    let owner = canonical_owner(&principal, None)?;
    let read_at = chrono::Utc::now();
    let mut transaction = state
        .db
        .begin()
        .await
        .map_err(|_| StatusCode::SERVICE_UNAVAILABLE)?;
    let updated: Option<(String,)> = sqlx::query_as(
        "WITH target AS (SELECT n.id, n.user_id FROM public.notifications n LEFT JOIN public.notification_expirations x ON x.notification_id = n.id WHERE n.id = $1 AND (n.user_id = $2 OR (n.user_id IS NULL AND lower(n.recipient) = 'all')) AND (x.expires_at IS NULL OR x.expires_at > NOW()) FOR UPDATE OF n), updated AS (UPDATE public.notifications n SET read_at = $3 FROM target t WHERE n.id = t.id AND t.user_id IS NOT NULL RETURNING n.id) INSERT INTO public.notification_engagement (notification_id, owner_id, read_at, updated_at) SELECT t.id, $2, $3, NOW() FROM target t ON CONFLICT (notification_id, owner_id) DO UPDATE SET read_at = EXCLUDED.read_at, updated_at = NOW() RETURNING notification_id",
    )
    .bind(&id)
    .bind(&owner)
    .bind(read_at)
    .fetch_optional(&mut *transaction)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let Some((_id,)) = updated else {
        return Err(StatusCode::NOT_FOUND);
    };
    transaction
        .commit()
        .await
        .map_err(|_| StatusCode::SERVICE_UNAVAILABLE)?;
    Ok(Json(MarkReadResponse { id, read_at }))
}

async fn mark_unread(
    State(state): State<AppState>,
    Extension(principal): Extension<VerifiedPrincipal>,
    AxPath(id): AxPath<String>,
) -> Result<StatusCode, StatusCode> {
    let owner = canonical_owner(&principal, None)?;
    let mut transaction = state
        .db
        .begin()
        .await
        .map_err(|_| StatusCode::SERVICE_UNAVAILABLE)?;
    let updated: Option<(String,)> = sqlx::query_as(
        "WITH target AS (SELECT n.id, n.user_id FROM public.notifications n LEFT JOIN public.notification_expirations x ON x.notification_id = n.id WHERE n.id = $1 AND (n.user_id = $2 OR (n.user_id IS NULL AND lower(n.recipient) = 'all')) AND (x.expires_at IS NULL OR x.expires_at > NOW()) FOR UPDATE OF n), updated AS (UPDATE public.notifications n SET read_at = NULL FROM target t WHERE n.id = t.id AND t.user_id IS NOT NULL RETURNING n.id) INSERT INTO public.notification_engagement (notification_id, owner_id, read_at, updated_at) SELECT t.id, $2, NULL, NOW() FROM target t ON CONFLICT (notification_id, owner_id) DO UPDATE SET read_at = EXCLUDED.read_at, updated_at = NOW() RETURNING notification_id",
    )
    .bind(&id)
    .bind(&owner)
    .fetch_optional(&mut *transaction)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let Some((_id,)) = updated else {
        return Err(StatusCode::NOT_FOUND);
    };
    transaction
        .commit()
        .await
        .map_err(|_| StatusCode::SERVICE_UNAVAILABLE)?;
    Ok(StatusCode::NO_CONTENT)
}

#[derive(Serialize)]
struct EngagementEventResponse {
    id: String,
    event: &'static str,
    occurred_at: chrono::DateTime<chrono::Utc>,
}

async fn mark_clicked(
    State(state): State<AppState>,
    Extension(principal): Extension<VerifiedPrincipal>,
    AxPath(id): AxPath<String>,
) -> Result<Json<EngagementEventResponse>, StatusCode> {
    let owner = canonical_owner(&principal, None)?;
    let clicked_at = chrono::Utc::now();
    let clicked: Option<(String, chrono::DateTime<chrono::Utc>)> = sqlx::query_as(
        "INSERT INTO public.notification_engagement (notification_id, owner_id, clicked_at, updated_at) SELECT $1, $2, $3, NOW() WHERE EXISTS (SELECT 1 FROM public.notifications n LEFT JOIN public.notification_expirations x ON x.notification_id = n.id WHERE n.id = $1 AND (n.user_id = $2 OR (n.user_id IS NULL AND lower(n.recipient) = 'all')) AND (x.expires_at IS NULL OR x.expires_at > NOW())) ON CONFLICT (notification_id, owner_id) DO UPDATE SET clicked_at = COALESCE(public.notification_engagement.clicked_at, EXCLUDED.clicked_at), updated_at = NOW() RETURNING notification_id, clicked_at",
    )
    .bind(&id)
    .bind(&owner)
    .bind(clicked_at)
    .fetch_optional(&state.db)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    match clicked {
        Some((id, occurred_at)) => Ok(Json(EngagementEventResponse {
            id,
            event: "clicked",
            occurred_at,
        })),
        None => Err(StatusCode::NOT_FOUND),
    }
}

async fn mark_dismissed(
    State(state): State<AppState>,
    Extension(principal): Extension<VerifiedPrincipal>,
    AxPath(id): AxPath<String>,
) -> Result<Json<EngagementEventResponse>, StatusCode> {
    let owner = canonical_owner(&principal, None)?;
    let dismissed_at = chrono::Utc::now();
    let dismissed: Option<(String, chrono::DateTime<chrono::Utc>)> = sqlx::query_as(
        "INSERT INTO public.notification_engagement (notification_id, owner_id, dismissed_at, updated_at) SELECT $1, $2, $3, NOW() WHERE EXISTS (SELECT 1 FROM public.notifications n LEFT JOIN public.notification_expirations x ON x.notification_id = n.id WHERE n.id = $1 AND (n.user_id = $2 OR (n.user_id IS NULL AND lower(n.recipient) = 'all')) AND (x.expires_at IS NULL OR x.expires_at > NOW())) ON CONFLICT (notification_id, owner_id) DO UPDATE SET dismissed_at = COALESCE(public.notification_engagement.dismissed_at, EXCLUDED.dismissed_at), updated_at = NOW() RETURNING notification_id, dismissed_at",
    )
    .bind(&id)
    .bind(&owner)
    .bind(dismissed_at)
    .fetch_optional(&state.db)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    match dismissed {
        Some((id, occurred_at)) => Ok(Json(EngagementEventResponse {
            id,
            event: "dismissed",
            occurred_at,
        })),
        None => Err(StatusCode::NOT_FOUND),
    }
}

#[derive(Serialize)]
struct AcknowledgeNotificationResponse {
    id: String,
    acknowledged_at: chrono::DateTime<chrono::Utc>,
}

async fn acknowledge_notification(
    State(state): State<AppState>,
    Extension(principal): Extension<VerifiedPrincipal>,
    AxPath(id): AxPath<String>,
) -> Result<Json<AcknowledgeNotificationResponse>, StatusCode> {
    let owner = canonical_owner(&principal, None)?;
    let acknowledged_at = chrono::Utc::now();
    let acknowledged: Option<(String, chrono::DateTime<chrono::Utc>)> = sqlx::query_as(
        "INSERT INTO public.notification_engagement (notification_id, owner_id, acknowledged_at, updated_at) SELECT $1, $2, $3, NOW() WHERE EXISTS (SELECT 1 FROM public.notifications n LEFT JOIN public.notification_expirations x ON x.notification_id = n.id WHERE n.id = $1 AND (n.user_id = $2 OR (n.user_id IS NULL AND lower(n.recipient) = 'all')) AND (x.expires_at IS NULL OR x.expires_at > NOW())) ON CONFLICT (notification_id, owner_id) DO UPDATE SET acknowledged_at = COALESCE(public.notification_engagement.acknowledged_at, EXCLUDED.acknowledged_at), updated_at = NOW() RETURNING notification_id, acknowledged_at",
    )
    .bind(&id)
    .bind(&owner)
    .bind(acknowledged_at)
    .fetch_optional(&state.db)
    .await
    .map_err(|error| {
        tracing::error!("notification acknowledgement persistence failed: {error}");
        StatusCode::INTERNAL_SERVER_ERROR
    })?;
    match acknowledged {
        Some((id, acknowledged_at)) => Ok(Json(AcknowledgeNotificationResponse {
            id,
            acknowledged_at,
        })),
        None => Err(StatusCode::NOT_FOUND),
    }
}

/// Remove the dependent lifecycle rows for owner-owned notifications before
/// deleting the notification itself. Foreign keys are intentionally
/// restrictive so erasure must be explicit and ordered. An outbox event is
/// removed only when no remaining channel job references it; plan fanout can
/// legitimately share one source event across several owners.
async fn erase_notification_dependencies(
    transaction: &mut Transaction<'_, Postgres>,
    notification_ids: &[String],
) -> Result<(), sqlx::Error> {
    if notification_ids.is_empty() {
        return Ok(());
    }
    let job_rows: Vec<(String, String)> = sqlx::query_as(
        "SELECT id, source_event_id FROM public.notification_channel_jobs WHERE notification_id = ANY($1::varchar[]) FOR UPDATE",
    )
    .bind(notification_ids)
    .fetch_all(&mut **transaction)
    .await?;
    let job_ids: Vec<String> = job_rows.iter().map(|(id, _)| id.clone()).collect();
    let event_ids: Vec<String> = job_rows
        .iter()
        .map(|(_, event_id)| event_id.clone())
        .collect();

    if !job_ids.is_empty() {
        sqlx::query(
            "DELETE FROM public.notification_provider_events WHERE job_id = ANY($1::varchar[])",
        )
        .bind(&job_ids)
        .execute(&mut **transaction)
        .await?;
        sqlx::query(
            "DELETE FROM public.notification_dead_letters WHERE job_id = ANY($1::varchar[])",
        )
        .bind(&job_ids)
        .execute(&mut **transaction)
        .await?;
        sqlx::query(
            "DELETE FROM public.notification_delivery_attempts WHERE job_id = ANY($1::varchar[])",
        )
        .bind(&job_ids)
        .execute(&mut **transaction)
        .await?;
        sqlx::query("DELETE FROM public.notification_channel_jobs WHERE id = ANY($1::varchar[])")
            .bind(&job_ids)
            .execute(&mut **transaction)
            .await?;
    }
    sqlx::query(
        "DELETE FROM public.notification_engagement WHERE notification_id = ANY($1::varchar[])",
    )
    .bind(notification_ids)
    .execute(&mut **transaction)
    .await?;
    sqlx::query(
        "DELETE FROM public.notification_expirations WHERE notification_id = ANY($1::varchar[])",
    )
    .bind(notification_ids)
    .execute(&mut **transaction)
    .await?;
    if !event_ids.is_empty() {
        sqlx::query(
            "DELETE FROM public.notification_outbox o WHERE o.event_id = ANY($1::varchar[]) AND NOT EXISTS (SELECT 1 FROM public.notification_channel_jobs j WHERE j.source_event_id = o.event_id)",
        )
        .bind(&event_ids)
        .execute(&mut **transaction)
        .await?;
    }
    Ok(())
}

#[derive(Deserialize)]
struct MarkAllReadQuery {
    user_id: Option<String>,
}

#[derive(Serialize)]
struct MarkAllReadResponse {
    marked: u64,
}

async fn mark_all_read(
    State(state): State<AppState>,
    Extension(principal): Extension<VerifiedPrincipal>,
    axum::extract::Query(q): axum::extract::Query<MarkAllReadQuery>,
) -> Result<Json<MarkAllReadResponse>, StatusCode> {
    let owner = canonical_owner(&principal, q.user_id.as_deref())?;
    let mut transaction = state
        .db
        .begin()
        .await
        .map_err(|_| StatusCode::SERVICE_UNAVAILABLE)?;
    let updated: (i64,) = sqlx::query_as(
        "WITH owner_updated AS (UPDATE public.notifications n SET read_at = NOW() WHERE n.user_id = $1 AND n.read_at IS NULL AND NOT EXISTS (SELECT 1 FROM public.notification_expirations x WHERE x.notification_id = n.id AND x.expires_at <= NOW()) RETURNING n.id), broadcast_targets AS (SELECT n.id FROM public.notifications n LEFT JOIN public.notification_engagement e ON e.notification_id = n.id AND e.owner_id = $1 LEFT JOIN public.notification_expirations x ON x.notification_id = n.id WHERE n.user_id IS NULL AND lower(n.recipient) = 'all' AND (x.expires_at IS NULL OR x.expires_at > NOW()) AND e.read_at IS NULL), targets AS (SELECT id FROM owner_updated UNION ALL SELECT id FROM broadcast_targets), inserted AS (INSERT INTO public.notification_engagement (notification_id, owner_id, read_at, updated_at) SELECT id, $1, NOW(), NOW() FROM targets ON CONFLICT (notification_id, owner_id) DO UPDATE SET read_at = EXCLUDED.read_at, updated_at = NOW() RETURNING notification_id) SELECT COUNT(*)::bigint FROM inserted",
    )
    .bind(&owner)
    .fetch_one(&mut *transaction)
    .await
    .map_err(|_| StatusCode::SERVICE_UNAVAILABLE)?;
    transaction
        .commit()
        .await
        .map_err(|_| StatusCode::SERVICE_UNAVAILABLE)?;
    Ok(Json(MarkAllReadResponse {
        marked: updated.0 as u64,
    }))
}

async fn delete_notification(
    State(state): State<AppState>,
    Extension(principal): Extension<VerifiedPrincipal>,
    AxPath(id): AxPath<String>,
) -> Result<StatusCode, StatusCode> {
    let owner = canonical_owner(&principal, None)?;
    let mut transaction = state
        .db
        .begin()
        .await
        .map_err(|_| StatusCode::SERVICE_UNAVAILABLE)?;
    let target: Option<(Option<String>,)> = sqlx::query_as(
        "SELECT n.user_id FROM public.notifications n LEFT JOIN public.notification_expirations x ON x.notification_id = n.id WHERE n.id = $1 AND (n.user_id = $2 OR (n.user_id IS NULL AND lower(n.recipient) = 'all')) AND (x.expires_at IS NULL OR x.expires_at > NOW()) FOR UPDATE OF n",
    )
    .bind(&id)
    .bind(&owner)
    .fetch_optional(&mut *transaction)
    .await
    .map_err(|_| StatusCode::SERVICE_UNAVAILABLE)?;
    let Some((target_owner,)) = target else {
        return Err(StatusCode::NOT_FOUND);
    };
    sqlx::query(
        "DELETE FROM public.notification_engagement WHERE notification_id = $1 AND owner_id = $2",
    )
    .bind(&id)
    .bind(&owner)
    .execute(&mut *transaction)
    .await
    .map_err(|_| StatusCode::SERVICE_UNAVAILABLE)?;
    if target_owner.is_none() {
        transaction
            .commit()
            .await
            .map_err(|_| StatusCode::SERVICE_UNAVAILABLE)?;
        return Ok(StatusCode::NO_CONTENT);
    }
    erase_notification_dependencies(&mut transaction, std::slice::from_ref(&id))
        .await
        .map_err(|error| {
            tracing::error!("owner notification dependency cleanup failed: {error}");
            StatusCode::SERVICE_UNAVAILABLE
        })?;
    let deleted: Option<(String,)> = sqlx::query_as(
        "DELETE FROM public.notifications WHERE id = $1 AND user_id = $2 RETURNING id",
    )
    .bind(&id)
    .bind(&owner)
    .fetch_optional(&mut *transaction)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let Some((_id,)) = deleted else {
        return Err(StatusCode::NOT_FOUND);
    };
    transaction
        .commit()
        .await
        .map_err(|_| StatusCode::SERVICE_UNAVAILABLE)?;
    Ok(StatusCode::NO_CONTENT)
}

#[derive(Deserialize)]
struct ClearAllQuery {
    user_id: Option<String>,
}

#[derive(Serialize)]
struct ClearAllResponse {
    deleted: u64,
}

async fn clear_all(
    State(state): State<AppState>,
    Extension(principal): Extension<VerifiedPrincipal>,
    axum::extract::Query(q): axum::extract::Query<ClearAllQuery>,
) -> Result<Json<ClearAllResponse>, StatusCode> {
    let owner = canonical_owner(&principal, q.user_id.as_deref())?;
    let mut transaction = state
        .db
        .begin()
        .await
        .map_err(|_| StatusCode::SERVICE_UNAVAILABLE)?;
    sqlx::query("DELETE FROM public.notification_engagement WHERE owner_id = $1")
        .bind(&owner)
        .execute(&mut *transaction)
        .await
        .map_err(|_| StatusCode::SERVICE_UNAVAILABLE)?;
    let owner_notification_ids: Vec<String> =
        sqlx::query_scalar("SELECT id FROM public.notifications WHERE user_id = $1 FOR UPDATE")
            .bind(&owner)
            .fetch_all(&mut *transaction)
            .await
            .map_err(|_| StatusCode::SERVICE_UNAVAILABLE)?;
    erase_notification_dependencies(&mut transaction, &owner_notification_ids)
        .await
        .map_err(|error| {
            tracing::error!("owner notification dependency cleanup failed: {error}");
            StatusCode::SERVICE_UNAVAILABLE
        })?;
    let deleted: Vec<(String,)> =
        sqlx::query_as("DELETE FROM public.notifications WHERE user_id = $1 RETURNING id")
            .bind(&owner)
            .fetch_all(&mut *transaction)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    transaction
        .commit()
        .await
        .map_err(|_| StatusCode::SERVICE_UNAVAILABLE)?;
    Ok(Json(ClearAllResponse {
        deleted: deleted.len() as u64,
    }))
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct UnreadCountQuery {
    user_id: Option<String>,
    status: Option<String>,
    #[serde(rename = "type")]
    notification_type: Option<String>,
    priority: Option<String>,
    start_date: Option<chrono::DateTime<chrono::Utc>>,
    end_date: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Serialize)]
struct UnreadCountResponse {
    count: i64,
}

fn validate_unread_count_query(query: &UnreadCountQuery) -> Result<(), StatusCode> {
    if query.status.as_deref().is_some_and(|status| {
        !matches!(
            status,
            "all" | "read" | "unread" | "pending" | "sent" | "failed" | "suppressed" | "expired"
        )
    }) || query.notification_type.as_deref().is_some_and(|value| {
        value.is_empty()
            || value.len() > 64
            || value
                .chars()
                .any(|character| character.is_control() || character.is_whitespace())
    }) || query
        .priority
        .as_deref()
        .is_some_and(|value| !matches!(value, "low" | "normal" | "high" | "critical" | "urgent"))
        || query
            .start_date
            .zip(query.end_date)
            .is_some_and(|(start, end)| start > end)
    {
        return Err(StatusCode::BAD_REQUEST);
    }
    Ok(())
}

const OWNER_UNREAD_COUNT_SQL: &str = "SELECT COUNT(*) FROM public.notifications n LEFT JOIN public.notification_engagement e ON e.notification_id = n.id AND e.owner_id = $1 LEFT JOIN public.notification_expirations x ON x.notification_id = n.id WHERE (n.user_id = $1 OR (n.user_id IS NULL AND lower(n.recipient) = 'all')) AND (x.expires_at IS NULL OR x.expires_at > NOW()) AND COALESCE(e.read_at, n.read_at) IS NULL AND ($2::text IS NULL OR (($2 IN ('read', 'unread') AND (($2 = 'read' AND COALESCE(e.read_at, n.read_at) IS NOT NULL) OR ($2 = 'unread' AND COALESCE(e.read_at, n.read_at) IS NULL))) OR ($2 NOT IN ('read', 'unread') AND n.status = $2))) AND ($3::text IS NULL OR n.notification_type = $3) AND ($4::text IS NULL OR n.priority = $4) AND ($5::timestamptz IS NULL OR n.created_at >= $5) AND ($6::timestamptz IS NULL OR n.created_at <= $6)";

async fn unread_count(
    State(state): State<AppState>,
    Extension(principal): Extension<VerifiedPrincipal>,
    axum::extract::Query(q): axum::extract::Query<UnreadCountQuery>,
) -> Result<Json<UnreadCountResponse>, StatusCode> {
    validate_unread_count_query(&q)?;
    let owner = canonical_owner(&principal, q.user_id.as_deref())?;
    let status = q.status.as_deref().filter(|status| *status != "all");
    let count: i64 = sqlx::query_scalar(OWNER_UNREAD_COUNT_SQL)
        .bind(&owner)
        .bind(status)
        .bind(q.notification_type.as_deref())
        .bind(q.priority.as_deref())
        .bind(q.start_date)
        .bind(q.end_date)
        .fetch_one(&state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(UnreadCountResponse { count }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{TimeZone, Utc};
    use epsx_notification::{NOTIFICATION_PUBLISHER_AUDIENCE, NOTIFICATION_PUBLISH_PERMISSION};
    use epsx_service_auth::FRONTEND_AUDIENCE;

    fn valid_send_request() -> SendNotificationRequest {
        let owner = "0x1111111111111111111111111111111111111111";
        SendNotificationRequest {
            user_id: Some(owner.into()),
            channel: "in_app".into(),
            recipient: owner.into(),
            template_id: None,
            subject: Some("Migration update".into()),
            body: Some("The migration is ready for review.".into()),
            data: Some(serde_json::json!({"source": "admin"})),
            expires_at: None,
        }
    }

    #[test]
    fn send_request_validation_is_bounded_and_requires_content() {
        let valid = valid_send_request();
        assert_eq!(validate_send_request(&valid), Ok(()));

        let mut unknown_channel = valid_send_request();
        unknown_channel.channel = "sms".into();
        assert_eq!(
            validate_send_request(&unknown_channel),
            Err(StatusCode::BAD_REQUEST)
        );

        let mut no_content = valid_send_request();
        no_content.body = None;
        no_content.subject = None;
        assert_eq!(
            validate_send_request(&no_content),
            Err(StatusCode::BAD_REQUEST)
        );

        let mut non_object_data = valid_send_request();
        non_object_data.data = Some(serde_json::json!(["private"]));
        assert_eq!(
            validate_send_request(&non_object_data),
            Err(StatusCode::BAD_REQUEST)
        );

        let mut oversized_body = valid_send_request();
        oversized_body.body = Some("x".repeat(MAX_BODY_CHARS + 1));
        assert_eq!(
            validate_send_request(&oversized_body),
            Err(StatusCode::BAD_REQUEST)
        );
    }

    #[test]
    fn email_validation_rejects_invalid_recipients_and_unknown_request_fields() {
        let mut invalid_email = valid_send_request();
        invalid_email.channel = "email".into();
        invalid_email.recipient = "not-an-email".into();
        assert_eq!(
            validate_send_request(&invalid_email),
            Err(StatusCode::BAD_REQUEST)
        );

        let unknown = serde_json::json!({
            "channel": "in_app",
            "recipient": "0xrecipient",
            "body": "body",
            "unexpected": true
        });
        assert!(serde_json::from_value::<SendNotificationRequest>(unknown).is_err());
    }

    #[test]
    fn idempotency_key_is_ascii_bounded_and_maps_to_a_canonical_record_id() {
        assert!(valid_idempotency_key("admin.send.2026-07-22_01"));
        assert!(!valid_idempotency_key(""));
        assert!(!valid_idempotency_key("contains space"));
        assert!(!valid_idempotency_key(
            &"x".repeat(MAX_IDEMPOTENCY_KEY_CHARS + 1)
        ));
        assert_eq!(
            idempotent_notification_id("admin.send.2026-07-22_01"),
            "idem_admin.send.2026-07-22_01"
        );
    }

    fn valid_admin_row() -> AdminNotificationRow {
        AdminNotificationRow {
            id: "notification-001".into(),
            title: Some("Portfolio alert".into()),
            subject: Some("EPS changed".into()),
            channel: "in_app".into(),
            status: "sent".into(),
            notification_type: Some("portfolio-alert".into()),
            priority: Some("high".into()),
            sent_at: Some(Utc.with_ymd_and_hms(2026, 7, 22, 3, 4, 5).unwrap()),
            created_at: Utc.with_ymd_and_hms(2026, 7, 22, 3, 0, 0).unwrap(),
        }
    }

    #[test]
    fn owner_notification_count_failure_never_becomes_authoritative_zero() {
        assert_eq!(require_owner_notification_total(Ok(7)), Ok(7));
        assert_eq!(
            require_owner_notification_total(Err(sqlx::Error::RowNotFound)),
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        );
    }

    #[test]
    fn admin_projection_accepts_preference_suppressed_notifications() {
        let mut row = valid_admin_row();
        row.status = "suppressed".into();
        assert_eq!(
            project_admin_notification(row).unwrap().status,
            "suppressed"
        );
    }

    #[test]
    fn owner_notification_query_rejects_unknown_status_and_unbounded_values() {
        let mut valid = std::collections::HashMap::new();
        valid.insert("status".to_string(), "sent".to_string());
        valid.insert("limit".to_string(), "20".to_string());
        valid.insert("offset".to_string(), "40".to_string());
        assert_eq!(
            parse_owner_notification_query(&valid),
            Ok(OwnerNotificationQuery {
                status: Some("sent".to_string()),
                notification_type: None,
                priority: None,
                start_date: None,
                end_date: None,
                limit: 20,
                offset: 40,
            })
        );

        valid.insert("status".to_string(), "read".to_string());
        assert_eq!(
            parse_owner_notification_query(&valid)
                .unwrap()
                .status
                .as_deref(),
            Some("read")
        );
        valid.insert("status".to_string(), "all".to_string());
        assert_eq!(parse_owner_notification_query(&valid).unwrap().status, None);
        valid.insert("type".to_string(), "payment".to_string());
        valid.insert("priority".to_string(), "high".to_string());
        valid.insert("start_date".to_string(), "2026-07-01T00:00:00Z".to_string());
        valid.insert("end_date".to_string(), "2026-07-31T23:59:59Z".to_string());
        let filtered = parse_owner_notification_query(&valid).unwrap();
        assert_eq!(filtered.notification_type.as_deref(), Some("payment"));
        assert_eq!(filtered.priority.as_deref(), Some("high"));
        assert!(filtered.start_date.is_some());
        assert!(filtered.end_date.is_some());
        valid.remove("type");
        valid.insert("notification_type".to_string(), "payment".to_string());
        assert_eq!(
            parse_owner_notification_query(&valid)
                .unwrap()
                .notification_type
                .as_deref(),
            Some("payment")
        );
        valid.insert("type".to_string(), "security".to_string());
        assert_eq!(
            parse_owner_notification_query(&valid),
            Err(StatusCode::BAD_REQUEST)
        );
        valid.remove("type");
        valid.remove("notification_type");
        valid.insert("start_date".to_string(), "2026-08-01T00:00:00Z".to_string());
        valid.insert("end_date".to_string(), "2026-07-01T00:00:00Z".to_string());
        assert_eq!(
            parse_owner_notification_query(&valid),
            Err(StatusCode::BAD_REQUEST)
        );
        valid.remove("start_date");
        valid.remove("end_date");
        valid.remove("status");
        valid.insert("limit".to_string(), "0".to_string());
        assert_eq!(
            parse_owner_notification_query(&valid),
            Err(StatusCode::BAD_REQUEST)
        );
        valid.insert("limit".to_string(), "20".to_string());
        valid.insert("offset".to_string(), "1000001".to_string());
        assert_eq!(
            parse_owner_notification_query(&valid),
            Err(StatusCode::BAD_REQUEST)
        );
        valid.remove("offset");
        valid.insert("unknown".to_string(), "value".to_string());
        assert_eq!(
            parse_owner_notification_query(&valid),
            Err(StatusCode::BAD_REQUEST)
        );
    }

    #[test]
    fn unread_count_query_preserves_source_filters_and_rejects_drift() {
        let valid = UnreadCountQuery {
            user_id: None,
            status: Some("unread".into()),
            notification_type: Some("payment".into()),
            priority: Some("high".into()),
            start_date: Some(Utc.with_ymd_and_hms(2026, 7, 1, 0, 0, 0).unwrap()),
            end_date: Some(Utc.with_ymd_and_hms(2026, 7, 31, 23, 59, 59).unwrap()),
        };
        assert_eq!(validate_unread_count_query(&valid), Ok(()));

        let mut invalid = valid.clone();
        invalid.status = Some("unknown".into());
        assert_eq!(
            validate_unread_count_query(&invalid),
            Err(StatusCode::BAD_REQUEST)
        );
        invalid = valid.clone();
        invalid.priority = Some("medium".into());
        assert_eq!(
            validate_unread_count_query(&invalid),
            Err(StatusCode::BAD_REQUEST)
        );
        invalid = valid;
        invalid.start_date = invalid.end_date;
        invalid.end_date = Some(Utc.with_ymd_and_hms(2026, 6, 30, 0, 0, 0).unwrap());
        assert_eq!(
            validate_unread_count_query(&invalid),
            Err(StatusCode::BAD_REQUEST)
        );
    }

    #[test]
    fn owner_reads_include_only_explicit_broadcasts_and_scope_engagement_per_owner() {
        assert!(OWNER_NOTIFICATION_SCOPE_SQL.contains("n.user_id = $1"));
        assert!(OWNER_NOTIFICATION_SCOPE_SQL.contains("lower(n.recipient) = 'all'"));
        assert!(
            OWNER_NOTIFICATION_SCOPE_SQL.contains("x.expires_at IS NULL OR x.expires_at > NOW()")
        );
        assert!(OWNER_NOTIFICATION_JOIN.contains("e.owner_id = $1"));
        assert!(OWNER_NOTIFICATION_JOIN.contains("notification_expirations"));
        assert!(OWNER_NOTIFICATION_SELECT_FIELDS.contains("COALESCE(e.read_at, n.read_at)"));
        assert!(OWNER_NOTIFICATION_SELECT_FIELDS.contains("e.clicked_at"));
        assert!(OWNER_NOTIFICATION_SELECT_FIELDS.contains("x.expires_at"));

        let status_sql = format!(
            "SELECT {OWNER_NOTIFICATION_SELECT_FIELDS} {OWNER_NOTIFICATION_JOIN} WHERE {OWNER_NOTIFICATION_SCOPE_SQL} {OWNER_NOTIFICATION_FILTER_SQL} ORDER BY n.created_at DESC, n.id DESC LIMIT $7 OFFSET $8"
        );
        assert!(status_sql.contains("n.status = $2"));
        assert!(status_sql.contains("n.notification_type = $3"));
        assert!(status_sql.contains("n.priority = $4"));
        assert!(status_sql.contains("n.created_at >= $5"));
        assert!(status_sql.contains("n.created_at <= $6"));
        assert!(status_sql.contains("ORDER BY n.created_at DESC, n.id DESC"));
    }

    fn valid_publish_request() -> PublishNotificationRequest {
        PublishNotificationRequest {
            event_id: "payment-event-1".into(),
            event_type: "payment.completed".into(),
            aggregate_id: "payment-1".into(),
            idempotency_key: "payment-event-1".into(),
            recipient_wallet_address: "0x1111111111111111111111111111111111111111".into(),
            notification_type: "payment".into(),
            priority: "normal".into(),
            title: "Payment complete".into(),
            message: "Your payment completed".into(),
            data: Some(serde_json::json!({"amount": 10})),
            action_url: Some("/payments/payment-1".into()),
            expires_at: None,
            plan_id: None,
        }
    }

    #[test]
    fn publisher_payload_validation_is_bounded_and_hash_is_stable() {
        let request = valid_publish_request();
        assert_eq!(validate_publish_request(&request), Ok(()));
        assert_eq!(
            publish_request_hash(&request),
            publish_request_hash(&request)
        );

        let mut invalid = request.clone();
        invalid.recipient_wallet_address = "0xattacker".into();
        assert_eq!(
            validate_publish_request(&invalid),
            Err(StatusCode::BAD_REQUEST)
        );
        let mut broadcast = valid_publish_request();
        broadcast.recipient_wallet_address = "all".into();
        broadcast.event_type = "notification.broadcast".into();
        assert_eq!(validate_publish_request(&broadcast), Ok(()));
        let mut send_broadcast = valid_publish_request();
        send_broadcast.event_type = "notification.send".into();
        send_broadcast.recipient_wallet_address = "all".into();
        assert_eq!(
            validate_publish_request(&send_broadcast),
            Err(StatusCode::BAD_REQUEST)
        );
        let mut broadcast_wallet = valid_publish_request();
        broadcast_wallet.event_type = "notification.broadcast".into();
        assert_eq!(
            validate_publish_request(&broadcast_wallet),
            Err(StatusCode::BAD_REQUEST)
        );
        let mut domain_broadcast = valid_publish_request();
        domain_broadcast.recipient_wallet_address = "all".into();
        assert_eq!(
            validate_publish_request(&domain_broadcast),
            Err(StatusCode::BAD_REQUEST)
        );
        let mut plan_target = valid_publish_request();
        plan_target.recipient_wallet_address = "all".into();
        plan_target.plan_id = Some(Uuid::from_u128(1));
        assert_eq!(validate_publish_request(&plan_target), Ok(()));
        let mut nil_plan = plan_target.clone();
        nil_plan.plan_id = Some(Uuid::nil());
        assert_eq!(
            validate_publish_request(&nil_plan),
            Err(StatusCode::BAD_REQUEST)
        );
        let mut plan_broadcast = plan_target.clone();
        plan_broadcast.event_type = "notification.broadcast".into();
        assert_eq!(
            validate_publish_request(&plan_broadcast),
            Err(StatusCode::BAD_REQUEST)
        );
        let mut plan_wallet = plan_target.clone();
        plan_wallet.recipient_wallet_address = "0x1111111111111111111111111111111111111111".into();
        assert_eq!(
            validate_publish_request(&plan_wallet),
            Err(StatusCode::BAD_REQUEST)
        );
        let first_id = plan_notification_id(
            "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
            "0x1111111111111111111111111111111111111111",
        );
        assert_eq!(first_id.len(), 66);
        assert_eq!(
            first_id,
            plan_notification_id(
                "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
                "0x1111111111111111111111111111111111111111",
            )
        );
        assert_ne!(
            first_id,
            plan_notification_id(
                "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
                "0x2222222222222222222222222222222222222222",
            )
        );
        assert!(PLAN_TARGET_MEMBERSHIPS_SQL.contains("wallet_plan_assignments"));
        assert!(PLAN_TARGET_MEMBERSHIPS_SQL.contains("is_active = TRUE"));
        assert!(PLAN_TARGET_MEMBERSHIPS_SQL.contains("LIMIT $2"));
        assert!(!PLAN_TARGET_MEMBERSHIPS_SQL.contains("notification_preferences"));
        assert!(PLAN_TARGET_POLICIES_SQL.contains("notification_preferences"));
        assert!(PLAN_TARGET_POLICIES_SQL.contains("quiet_until"));
        assert!(!PLAN_TARGET_POLICIES_SQL.contains("INSERT"));
        assert!(!PLAN_TARGET_POLICIES_SQL.contains("UPDATE"));
        assert!(!PLAN_TARGET_POLICIES_SQL.contains("DELETE"));
        assert_eq!(
            PLAN_DB_READ_ONLY_SESSION_SQL,
            "SET default_transaction_read_only = on"
        );
        assert_eq!(validate_plan_fanout_count(MAX_PLAN_FANOUT), Ok(()));
        assert_eq!(
            validate_plan_fanout_count(MAX_PLAN_FANOUT + 1),
            Err(StatusCode::PAYLOAD_TOO_LARGE)
        );
        let mut external = request;
        external.action_url = Some("https://evil.example".into());
        assert_eq!(
            validate_publish_request(&external),
            Err(StatusCode::BAD_REQUEST)
        );
        assert!(valid_action_url("/notifications/evt-1"));
        assert!(!valid_action_url("//evil.example"));
        assert!(!valid_action_url("/notifications\\evil"));
        let mut unknown_event = valid_publish_request();
        unknown_event.event_type = "untrusted.event".into();
        assert_eq!(
            validate_publish_request(&unknown_event),
            Err(StatusCode::BAD_REQUEST)
        );
        let mut unsafe_identity = valid_publish_request();
        unsafe_identity.event_id = "payment event 1".into();
        assert_eq!(
            validate_publish_request(&unsafe_identity),
            Err(StatusCode::BAD_REQUEST)
        );
    }

    #[test]
    fn expiration_validation_requires_a_bounded_future_window() {
        assert_eq!(validate_expiration(None), Ok(()));
        assert_eq!(
            validate_expiration(Some(Utc::now() + chrono::Duration::minutes(5))),
            Ok(())
        );
        assert_eq!(
            validate_expiration(Some(Utc::now() - chrono::Duration::seconds(1))),
            Err(StatusCode::BAD_REQUEST)
        );
        assert_eq!(
            validate_expiration(Some(Utc::now() + chrono::Duration::days(366))),
            Err(StatusCode::BAD_REQUEST)
        );
    }

    #[test]
    fn provider_event_validation_is_bounded_and_replay_safe() {
        let valid = ProviderEventRequest {
            provider: "smtp-relay".into(),
            provider_event_id: "evt-001".into(),
            provider_message_id: Some("msg-001".into()),
            job_id: Some("job-001".into()),
            event_type: "delivered".into(),
            payload: serde_json::json!({"provider": "smtp-relay"}),
            occurred_at: Some(Utc.with_ymd_and_hms(2026, 7, 23, 4, 5, 6).unwrap()),
        };
        assert_eq!(validate_provider_event_request(&valid), Ok(()));

        let mut invalid = valid;
        invalid.event_type = "opened".into();
        assert_eq!(
            validate_provider_event_request(&invalid),
            Err(StatusCode::BAD_REQUEST)
        );

        let mut invalid = ProviderEventRequest {
            provider: "smtp\nrelay".into(),
            provider_event_id: "evt-001".into(),
            provider_message_id: None,
            job_id: None,
            event_type: "accepted".into(),
            payload: serde_json::json!({}),
            occurred_at: None,
        };
        assert_eq!(
            validate_provider_event_request(&invalid),
            Err(StatusCode::BAD_REQUEST)
        );

        invalid.provider = "smtp-relay".into();
        invalid.payload = serde_json::json!(["not-an-object"]);
        assert_eq!(
            validate_provider_event_request(&invalid),
            Err(StatusCode::BAD_REQUEST)
        );

        assert_eq!(
            provider_event_target_state("accepted"),
            Some("provider_accepted")
        );
        assert_eq!(
            provider_event_target_state("delivered"),
            Some("provider_accepted")
        );
        assert_eq!(
            provider_event_target_state("bounced"),
            Some("terminal_failed")
        );
        assert_eq!(
            provider_event_target_state("complained"),
            Some("terminal_failed")
        );
        assert_eq!(
            provider_event_target_state("failed"),
            Some("terminal_failed")
        );
        assert_eq!(provider_event_target_state("opened"), None);
    }

    #[test]
    fn push_delivery_classifies_permanent_endpoints_and_stable_provider_ids() {
        assert_eq!(
            push_error_code(&WebPushError::PayloadTooLarge),
            "push_payload_too_large"
        );
        assert_eq!(
            push_error_code(&WebPushError::InvalidCryptoKeys),
            "push_invalid_subscription"
        );
        assert_eq!(push_message_id("job-1"), push_message_id("job-1"));
        assert_ne!(push_message_id("job-1"), push_message_id("job-2"));
        assert!(valid_vapid_key_id("active-2026"));
        assert!(valid_vapid_key_id("rotation_01"));
        assert!(!valid_vapid_key_id("previous key"));
        assert!(!valid_vapid_key_id("active/key"));
        assert!(!valid_vapid_key_id(""));
    }

    #[tokio::test]
    async fn push_provider_acceptance_encrypts_payload_and_uses_stable_message_id() {
        let capture = Arc::new(tokio::sync::Mutex::new(None::<(HeaderMap, Vec<u8>)>));
        let capture_state = Arc::clone(&capture);
        let app = Router::new().route(
            "/push",
            post(move |headers: HeaderMap, body: Bytes| {
                let capture = Arc::clone(&capture_state);
                async move {
                    *capture.lock().await = Some((headers, body.to_vec()));
                    StatusCode::CREATED
                }
            }),
        );
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
            .await
            .expect("bind local push provider");
        let address = listener.local_addr().expect("local provider address");
        let server = tokio::spawn(async move {
            axum::serve(listener, app)
                .await
                .expect("local provider server");
        });

        let state = AppState {
            db: PgPoolOptions::new()
                .connect_lazy("postgres://invalid.invalid/notification")
                .expect("lazy database pool"),
            plan_db: None,
            templates: Arc::new(RwLock::new(Handlebars::new())),
            smtp: Arc::new(RwLock::new(None)),
            provider_signing_secrets: Vec::new(),
            realtime_slots: Arc::new(tokio::sync::Semaphore::new(MAX_REALTIME_CONNECTIONS)),
            stream_metrics: Arc::new(StreamMetrics::default()),
            vapid_key_id: "active".into(),
            vapid_public_key: None,
            vapid_private_key: Some(Arc::new(
                b"IQ9Ur0ykXoHS9gzfYX0aBjy9lvdrjx_PFUXmie9YRcY".to_vec(),
            )),
            vapid_previous_key_id: None,
            vapid_previous_private_key: None,
            from: "runtime@example.test".into(),
            from_name: "Runtime audit".into(),
            redis: None,
            realtime_notify: Arc::new(Notify::new()),
        };
        let job_id = "push-provider-runtime-job";
        let (status, error, provider_message_id) = send_push(
            &state,
            PushDelivery {
                job_id,
                vapid_key_id: "active",
                endpoint: &format!("http://{address}/push"),
                p256dh: "BH1HTeKM7-NwaLGHEqxeu2IamQaVVLkcsFHPIHmsCnqxcBHPQBprF41bEMOr3O1hUQ2jU1opNEm1F_lZV_sxMP8",
                auth: "sBXU5_tIYz-5w7G2B25BEw",
                title: "Runtime push",
                body: "Provider acceptance audit",
                data: Some(&serde_json::json!({"source": "runtime"})),
                action_url: Some("/notifications/runtime"),
            },
        )
        .await;

        assert_eq!(status, "sent");
        assert_eq!(error, None);
        assert_eq!(provider_message_id, Some(push_message_id(job_id)));
        let (headers, body) = capture
            .lock()
            .await
            .take()
            .expect("provider received one request");
        assert_eq!(
            headers
                .get("content-encoding")
                .and_then(|value| value.to_str().ok()),
            Some("aes128gcm")
        );
        assert!(headers.contains_key("authorization"));
        assert!(!body.is_empty(), "encrypted payload must not be empty");

        server.abort();
    }

    #[test]
    fn provider_signatures_are_domain_separated_bounded_and_constant_time_verified() {
        let secret = vec![7_u8; 32];
        let timestamp = 1_750_000_000_i64;
        let body = br#"{"provider":"smtp-relay","event_type":"delivered"}"#;
        let mut input = b"epsx.notification.provider.v1.".to_vec();
        input.extend_from_slice(timestamp.to_string().as_bytes());
        input.push(b'.');
        input.extend_from_slice(body);
        let mut mac = Hmac::<Sha256>::new_from_slice(&secret).unwrap();
        mac.update(&input);
        let signature = format!("v1={}", hex::encode(mac.finalize().into_bytes()));

        assert!(valid_provider_signing_secret(&"s".repeat(32)));
        assert!(!valid_provider_signing_secret("short"));
        assert!(verify_provider_signature(
            &secret,
            timestamp,
            body,
            &signature,
            timestamp + 60
        ));
        assert!(!verify_provider_signature(
            &secret,
            timestamp,
            br#"{"provider":"tampered"}"#,
            &signature,
            timestamp + 60
        ));
        assert!(!verify_provider_signature(
            &[8_u8; 32],
            timestamp,
            body,
            &signature,
            timestamp + 60
        ));
        assert!(!verify_provider_signature(
            &secret,
            timestamp,
            body,
            &signature,
            timestamp + 301
        ));
        assert!(!verify_provider_signature(
            &secret,
            timestamp,
            body,
            "sha256=bad",
            timestamp + 60
        ));
    }

    #[test]
    fn provider_signing_key_rotation_accepts_distinct_valid_keys_only() {
        let active = "a".repeat(32);
        let previous = "b".repeat(48);
        let secrets = provider_signing_secrets_from_values([
            format!("  {active}  "),
            previous.clone(),
            active.clone(),
            "too-short".to_owned(),
            "invalid key with whitespace".to_owned(),
        ]);

        assert_eq!(secrets.len(), 2);
        assert_eq!(secrets[0].as_slice(), active.as_bytes());
        assert_eq!(secrets[1].as_slice(), previous.as_bytes());
    }

    #[test]
    fn missing_email_provider_never_reports_delivery_success() {
        let (status, error, delivered, provider_message_id) = email_provider_unavailable();
        assert_eq!(status, "failed");
        assert_eq!(error.as_deref(), Some("provider_not_configured"));
        assert!(!delivered);
        assert!(provider_message_id.is_none());
    }

    #[test]
    fn smtp_message_id_is_stable_and_does_not_expose_job_identity() {
        let first = smtp_message_id("payment:job-001");
        assert_eq!(first, smtp_message_id("payment:job-001"));
        assert_ne!(first, smtp_message_id("payment:job-002"));
        assert!(first.starts_with("<epsx-"));
        assert!(first.ends_with("@epsx.invalid>"));
        assert!(!first.contains("job-001"));
    }

    #[test]
    fn smtp_configuration_requires_a_tls_transport() {
        assert!(build_smtp_transport("smtp.example.test", 465, "user", "secret").is_some());
        assert!(build_smtp_transport("smtp.example.test", 587, "user", "secret").is_some());
        assert!(build_smtp_transport("smtp example.test", 587, "user", "secret").is_none());
        assert!(build_smtp_transport("smtp.example.test", 0, "user", "secret").is_none());
    }

    #[test]
    fn send_requests_are_bounded_before_durable_enqueue() {
        let valid = SendNotificationRequest {
            user_id: Some("0x1111111111111111111111111111111111111111".into()),
            channel: "email".into(),
            recipient: "user@example.test".into(),
            template_id: None,
            subject: Some("Subject".into()),
            body: Some("Body".into()),
            data: Some(serde_json::json!({"key": "value"})),
            expires_at: None,
        };
        assert_eq!(validate_send_request(&valid), Ok(()));
        let mut invalid = valid;
        invalid.recipient = "\nuser@example.test".into();
        assert_eq!(
            validate_send_request(&invalid),
            Err(StatusCode::BAD_REQUEST)
        );
    }

    #[test]
    fn admin_send_binds_channel_recipients_to_a_canonical_wallet_target() {
        let owner = "0x1111111111111111111111111111111111111111";
        let mut in_app = SendNotificationRequest {
            user_id: Some(owner.into()),
            channel: "in_app".into(),
            recipient: owner.into(),
            template_id: None,
            subject: None,
            body: Some("Body".into()),
            data: None,
            expires_at: None,
        };
        assert_eq!(validate_send_request(&in_app), Ok(()));

        in_app.recipient = "0x2222222222222222222222222222222222222222".into();
        assert_eq!(validate_send_request(&in_app), Err(StatusCode::BAD_REQUEST));

        let mut push = in_app;
        push.channel = "push".into();
        push.recipient = "https://push.example.test/subscription".into();
        assert_eq!(validate_send_request(&push), Ok(()));

        let mut no_owner = push;
        no_owner.user_id = None;
        assert_eq!(
            validate_send_request(&no_owner),
            Err(StatusCode::BAD_REQUEST)
        );
    }

    #[test]
    fn admin_send_rejects_conflicting_template_and_inline_content() {
        let request = SendNotificationRequest {
            user_id: Some("0x1111111111111111111111111111111111111111".into()),
            channel: "email".into(),
            recipient: "user@example.test".into(),
            template_id: Some("template-1".into()),
            subject: Some("inline subject".into()),
            body: None,
            data: None,
            expires_at: None,
        };
        assert_eq!(
            validate_send_request(&request),
            Err(StatusCode::BAD_REQUEST)
        );
    }

    #[test]
    fn preference_payload_validation_requires_bounded_objects() {
        let valid = NotificationPreferencesRequest {
            channels: serde_json::json!({"email": true}),
            quiet_hours: Some(serde_json::json!({"start": "22:00", "end": "07:00"})),
            timezone: Some("UTC".into()),
        };
        assert_eq!(validate_preferences_request(&valid), Ok(()));
        let invalid = NotificationPreferencesRequest {
            channels: serde_json::json!(["email"]),
            quiet_hours: None,
            timezone: None,
        };
        assert_eq!(
            validate_preferences_request(&invalid),
            Err(StatusCode::BAD_REQUEST)
        );
        let invalid_channel = NotificationPreferencesRequest {
            channels: serde_json::json!({"email": "yes"}),
            quiet_hours: None,
            timezone: None,
        };
        assert_eq!(
            validate_preferences_request(&invalid_channel),
            Err(StatusCode::BAD_REQUEST)
        );
        let invalid_quiet_hours = NotificationPreferencesRequest {
            channels: serde_json::json!({"in_app": true}),
            quiet_hours: Some(serde_json::json!({"start": "25:00", "end": "07:00"})),
            timezone: Some("UTC".into()),
        };
        assert_eq!(
            validate_preferences_request(&invalid_quiet_hours),
            Err(StatusCode::BAD_REQUEST)
        );
        assert!(valid_clock("00:00"));
        assert!(valid_clock("23:59"));
        assert!(!valid_clock("24:00"));
    }

    #[test]
    fn source_preference_type_and_priority_metadata_is_validated() {
        let valid = NotificationPreferencesRequest {
            channels: serde_json::json!({
                "email": true,
                "in_app": true,
                "push": false,
                "types": {"payment": false, "system": true},
                "priority_filter": "high"
            }),
            quiet_hours: None,
            timezone: None,
        };
        assert_eq!(validate_preferences_request(&valid), Ok(()));
        assert!(notification_policy_allows(
            &valid.channels,
            "in_app",
            "system",
            "high"
        ));
        assert!(!notification_policy_allows(
            &valid.channels,
            "in_app",
            "payment",
            "urgent"
        ));
        assert!(!notification_policy_allows(
            &valid.channels,
            "in_app",
            "system",
            "normal"
        ));

        let invalid_type = NotificationPreferencesRequest {
            channels: serde_json::json!({"types": {"unknown": true}}),
            quiet_hours: None,
            timezone: None,
        };
        assert_eq!(
            validate_preferences_request(&invalid_type),
            Err(StatusCode::BAD_REQUEST)
        );
        let invalid_priority = NotificationPreferencesRequest {
            channels: serde_json::json!({"priority_filter": "medium"}),
            quiet_hours: None,
            timezone: None,
        };
        assert_eq!(
            validate_preferences_request(&invalid_priority),
            Err(StatusCode::BAD_REQUEST)
        );
    }

    #[test]
    fn channel_preferences_fail_closed_for_malformed_enabled_values() {
        assert!(channel_preference_enabled(&serde_json::json!({}), "email"));
        assert!(!channel_preference_enabled(
            &serde_json::json!({"email": false}),
            "email"
        ));
        assert!(!channel_preference_enabled(
            &serde_json::json!({"email": "yes"}),
            "email"
        ));
        assert!(channel_preference_enabled(
            &serde_json::json!({"push": true}),
            "email"
        ));
    }

    #[test]
    fn stream_and_push_boundaries_reject_invalid_cursors_and_endpoints() {
        assert!(valid_stream_cursor("0xevent-1"));
        assert!(!valid_stream_cursor("event id"));
        assert!(!valid_stream_cursor("event\n"));
        assert!(valid_push_endpoint(
            "https://push.example.test/subscription"
        ));
        assert!(!valid_push_endpoint(
            "http://push.example.test/subscription"
        ));
        assert!(!valid_push_endpoint(
            "https://push.example.test/subscription?token=secret"
        ));
        assert!(valid_vapid_public_key("B".repeat(65).as_str()));
        assert!(!valid_vapid_public_key("private key with spaces"));
        assert!(validate_push_subscription(&PushSubscriptionRequest {
            endpoint: "https://push.example.test/subscription".into(),
            p256dh: "key_123".into(),
            auth: "auth_123".into(),
            user_agent: Some("browser".into()),
        })
        .is_ok());
    }

    #[test]
    fn template_validation_is_strict_and_handlebars_compilable() {
        let valid = CreateTemplateRequest {
            name: "portfolio-alert".into(),
            channel: "in_app".into(),
            subject: Some("Alert".into()),
            body: "Hello {{wallet}}".into(),
            variables: serde_json::json!({"wallet": {"type": "string"}}),
        };
        assert!(validate_template_request(&valid).is_ok());
        let mut invalid = valid.clone();
        invalid.channel = "sms".into();
        assert_eq!(
            validate_template_request(&invalid),
            Err(StatusCode::BAD_REQUEST)
        );
        let mut invalid = valid.clone();
        invalid.body = "{{#if}}".into();
        assert_eq!(
            validate_template_request(&invalid),
            Err(StatusCode::BAD_REQUEST)
        );

        let mut markup = valid.clone();
        markup.body =
            r#"<p>Hello {{wallet}}</p><img src="https://cdn.example.test/icon.png">"#.into();
        assert!(validate_template_request(&markup).is_ok());
        assert_eq!(
            parser_sanitized_template_body(&markup.body),
            Some(markup.body.clone())
        );
        markup.body = "online = {{wallet}}".into();
        assert!(validate_template_request(&markup).is_ok());
        markup.body = "<script>alert(1)</script>".into();
        assert!(parser_sanitized_template_body(&markup.body).is_none());
        assert_eq!(
            validate_template_request(&markup),
            Err(StatusCode::BAD_REQUEST)
        );
        markup.body = r#"<button onfocus = "alert(1)">Focus</button>"#.into();
        assert_eq!(
            validate_template_request(&markup),
            Err(StatusCode::BAD_REQUEST)
        );
        markup.body = r#"<meta http-equiv="refresh" content="0;url=https://evil.example">"#.into();
        assert_eq!(
            validate_template_request(&markup),
            Err(StatusCode::BAD_REQUEST)
        );
        markup.body = r#"<img src="https://user:pass@cdn.example.test/icon.png">"#.into();
        assert_eq!(
            validate_template_request(&markup),
            Err(StatusCode::BAD_REQUEST)
        );
        markup.body = r#"<a href="https://cdn.example.test/help">Help</a>"#.into();
        assert!(validate_template_request(&markup).is_ok());
        markup.body = r#"<a href="//cdn.example.test/help">Help</a>"#.into();
        assert_eq!(
            validate_template_request(&markup),
            Err(StatusCode::BAD_REQUEST)
        );
        markup.body = r#"<a href="https://user:pass@cdn.example.test/help">Help</a>"#.into();
        assert_eq!(
            validate_template_request(&markup),
            Err(StatusCode::BAD_REQUEST)
        );
        markup.body = r#"<svg><a href="https://cdn.example.test/help">Help</a></svg>"#.into();
        assert_eq!(
            validate_template_request(&markup),
            Err(StatusCode::BAD_REQUEST)
        );
        markup.body = r#"<p class="unsafe">Hello</p>"#.into();
        assert_eq!(
            validate_template_request(&markup),
            Err(StatusCode::BAD_REQUEST)
        );
        markup.body =
            r#"<img src="https://cdn.example.test/icon.png" width="64" height="64" alt="Icon">"#
                .into();
        assert!(validate_template_request(&markup).is_ok());
        markup.body = r#"<img src=https://cdn.example.test/icon.png>"#.into();
        assert_eq!(
            validate_template_request(&markup),
            Err(StatusCode::BAD_REQUEST)
        );
        markup.body = "<p>Unclosed".into();
        assert_eq!(
            validate_template_request(&markup),
            Err(StatusCode::BAD_REQUEST)
        );
        markup.body = "<p><strong>Mismatched</p></strong>".into();
        assert_eq!(
            validate_template_request(&markup),
            Err(StatusCode::BAD_REQUEST)
        );

        let mut raw = CreateTemplateRequest {
            name: "raw".into(),
            channel: "in_app".into(),
            subject: None,
            body: "{{{wallet}}}".into(),
            variables: serde_json::json!({"wallet": {"type": "string"}}),
        };
        assert_eq!(
            validate_template_request(&raw),
            Err(StatusCode::BAD_REQUEST)
        );
        raw.body = "{{&wallet}}".into();
        assert_eq!(
            validate_template_request(&raw),
            Err(StatusCode::BAD_REQUEST)
        );
    }

    #[test]
    fn template_variables_are_typed_and_data_is_validated_before_render() {
        let schema = serde_json::json!({
            "wallet": {"type": "string", "required": true},
            "amount": {"type": "number"}
        });
        assert!(valid_template_variables(&schema));
        assert!(validate_template_data(
            &schema,
            &HashMap::from([
                ("wallet".into(), serde_json::json!("0xabc")),
                ("amount".into(), serde_json::json!(12.5)),
            ])
        )
        .is_ok());
        assert_eq!(
            validate_template_data(
                &schema,
                &HashMap::from([
                    ("wallet".into(), serde_json::json!("0xabc")),
                    ("unexpected".into(), serde_json::json!(true)),
                ])
            ),
            Err(StatusCode::BAD_REQUEST)
        );
        assert_eq!(
            validate_template_data(
                &schema,
                &HashMap::from([("wallet".into(), serde_json::json!(12))])
            ),
            Err(StatusCode::BAD_REQUEST)
        );
        assert!(!valid_template_variables(&serde_json::json!({
            "wallet": {"type": "string", "unknown": true}
        })));
        assert!(!valid_template_variables(&serde_json::json!({
            "wallet": "string"
        })));
    }

    #[test]
    fn template_preview_data_is_bounded_and_object_only() {
        assert!(validate_template_preview_data(serde_json::json!({
            "wallet": "0xabc"
        }))
        .is_ok());
        assert_eq!(
            validate_template_preview_data(serde_json::json!(["not-an-object"])),
            Err(StatusCode::BAD_REQUEST)
        );
        assert_eq!(
            validate_template_preview_data(serde_json::json!({
                "body": "a".repeat(64 * 1024)
            })),
            Err(StatusCode::BAD_REQUEST)
        );
    }

    #[test]
    fn template_audit_projection_is_bounded_and_metadata_allowlisted() {
        let mut entry = TemplateAuditEntry {
            id: "template-audit-1".into(),
            template_id: "0xtemplate".into(),
            action: "rollback".into(),
            from_version: Some(2),
            to_version: Some(3),
            actor_subject: "admin-subject".into(),
            metadata: serde_json::json!({"restored_version": 2, "new_version": 3}),
            created_at: Utc::now(),
        };
        assert!(valid_template_audit_entry(&entry));

        entry.metadata = serde_json::json!({
            "restored_version": 2,
            "new_version": 3,
            "secret": "must not cross the audit boundary"
        });
        assert!(!valid_template_audit_entry(&entry));

        entry.action = "updated".into();
        entry.metadata = serde_json::json!({"template_name": "portfolio-alert"});
        assert!(valid_template_audit_entry(&entry));

        entry.metadata = serde_json::json!({"template_name": "\u{0000}"});
        assert!(!valid_template_audit_entry(&entry));
    }

    #[test]
    fn channel_metrics_are_allowlisted_and_fail_closed() {
        let projected = project_channel_outcomes(vec![("email".into(), 2), ("in_app".into(), 3)])
            .expect("known channels should project");
        assert_eq!(projected.get("email"), Some(&2));
        assert_eq!(projected.get("in_app"), Some(&3));
        assert_eq!(
            project_channel_outcomes(vec![("sms".into(), 1)]),
            Err(StatusCode::SERVICE_UNAVAILABLE)
        );
        assert_eq!(
            project_channel_outcomes(vec![("push".into(), -1)]),
            Err(StatusCode::SERVICE_UNAVAILABLE)
        );
    }

    #[test]
    fn template_rollback_version_is_positive_and_bounded() {
        assert_eq!(
            validate_template_rollback_request(&TemplateRollbackRequest { version: 1 }),
            Ok(())
        );
        assert_eq!(
            validate_template_rollback_request(&TemplateRollbackRequest { version: 0 }),
            Err(StatusCode::BAD_REQUEST)
        );
    }

    #[tokio::test]
    #[ignore = "requires a disposable migrated local notification database"]
    async fn runtime_template_rollback_replays_body_and_audit(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let database_url = std::env::var("NOTIFICATION_RUNTIME_DATABASE_URL")?;
        let db = sqlx::PgPool::connect(&database_url).await?;
        let mut handlebars = Handlebars::new();
        handlebars.set_strict_mode(true);
        let state = AppState {
            db: db.clone(),
            plan_db: None,
            templates: Arc::new(RwLock::new(handlebars)),
            smtp: Arc::new(RwLock::new(None)),
            provider_signing_secrets: Vec::new(),
            realtime_slots: Arc::new(tokio::sync::Semaphore::new(MAX_REALTIME_CONNECTIONS)),
            stream_metrics: Arc::new(StreamMetrics::default()),
            vapid_key_id: "active".into(),
            vapid_public_key: None,
            vapid_private_key: None,
            vapid_previous_key_id: None,
            vapid_previous_private_key: None,
            from: "runtime@example.test".into(),
            from_name: "Runtime audit".into(),
            redis: None,
            realtime_notify: Arc::new(Notify::new()),
        };
        let principal = VerifiedPrincipal {
            subject: "svc:template-runtime-audit".into(),
            wallet_address: "svc:template-runtime-audit".into(),
            audience: ADMIN_AUDIENCE.into(),
            permissions: vec![NOTIFICATIONS_MANAGE_PERMISSION.into()],
        };
        let name = format!("runtime-template-{}", Uuid::new_v4().simple());
        let created = create_template(
            State(state.clone()),
            Extension(principal.clone()),
            Json(CreateTemplateRequest {
                name: name.clone(),
                channel: "in_app".into(),
                subject: None,
                body: "Hello {{name}}".into(),
                variables: serde_json::json!({
                    "name": {"type": "string", "required": true}
                }),
            }),
        )
        .await
        .map_err(|status| format!("template create failed: {status}"))?
        .0;
        let updated = create_template(
            State(state.clone()),
            Extension(principal.clone()),
            Json(CreateTemplateRequest {
                name,
                channel: "in_app".into(),
                subject: None,
                body: "Changed {{name}}".into(),
                variables: serde_json::json!({
                    "name": {"type": "string", "required": true}
                }),
            }),
        )
        .await
        .map_err(|status| format!("template update failed: {status}"))?
        .0;
        assert_eq!(updated.body, "Changed {{name}}");

        let restored = rollback_template(
            State(state),
            AxPath(created.id.clone()),
            Extension(principal),
            Json(TemplateRollbackRequest { version: 1 }),
        )
        .await
        .map_err(|status| format!("template rollback failed: {status}"))?
        .0;
        assert_eq!(restored.body, "Hello {{name}}");
        let (version_count, rollback_count): (i64, i64) = sqlx::query_as(
            "SELECT (SELECT COUNT(*) FROM public.notification_template_versions WHERE template_id = $1), (SELECT COUNT(*) FROM public.notification_template_audit WHERE template_id = $1 AND action = 'rollback')",
        )
        .bind(&created.id)
        .fetch_one(&db)
        .await?;
        assert_eq!(version_count, 3);
        assert_eq!(rollback_count, 1);
        let restored_version: i32 = sqlx::query_scalar(
            "SELECT (metadata->>'restored_version')::int FROM public.notification_template_audit WHERE template_id = $1 AND action = 'rollback' ORDER BY created_at DESC LIMIT 1",
        )
        .bind(&created.id)
        .fetch_one(&db)
        .await?;
        assert_eq!(restored_version, 1);

        sqlx::query("DELETE FROM public.notification_template_audit WHERE template_id = $1")
            .bind(&created.id)
            .execute(&db)
            .await?;
        sqlx::query("DELETE FROM public.notification_template_versions WHERE template_id = $1")
            .bind(&created.id)
            .execute(&db)
            .await?;
        sqlx::query("DELETE FROM public.templates WHERE id = $1")
            .bind(&created.id)
            .execute(&db)
            .await?;
        db.close().await;
        Ok(())
    }

    #[tokio::test]
    #[ignore = "requires a disposable migrated local notification database"]
    async fn runtime_publisher_replay_is_idempotent_and_broadcast_is_single_row(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let database_url = std::env::var("NOTIFICATION_RUNTIME_DATABASE_URL")?;
        let db = sqlx::PgPool::connect(&database_url).await?;
        let plan_database_url = std::env::var("NOTIFICATION_RUNTIME_PLAN_DATABASE_URL")?;
        let plan_setup = sqlx::PgPool::connect(&plan_database_url).await?;
        let plan_db = connect_read_only_plan_pool(&plan_database_url).await?;
        let mut handlebars = Handlebars::new();
        handlebars.set_strict_mode(true);
        let state = AppState {
            db: db.clone(),
            plan_db: Some(plan_db.clone()),
            templates: Arc::new(RwLock::new(handlebars)),
            smtp: Arc::new(RwLock::new(None)),
            provider_signing_secrets: Vec::new(),
            realtime_slots: Arc::new(tokio::sync::Semaphore::new(MAX_REALTIME_CONNECTIONS)),
            stream_metrics: Arc::new(StreamMetrics::default()),
            vapid_key_id: "active".into(),
            vapid_public_key: None,
            vapid_private_key: None,
            vapid_previous_key_id: None,
            vapid_previous_private_key: None,
            from: "runtime@example.test".into(),
            from_name: "Runtime audit".into(),
            redis: None,
            realtime_notify: Arc::new(Notify::new()),
        };
        let principal = VerifiedPrincipal {
            subject: "svc:publisher-runtime-audit".into(),
            wallet_address: "svc:publisher-runtime-audit".into(),
            audience: NOTIFICATION_PUBLISHER_AUDIENCE.into(),
            permissions: vec![NOTIFICATION_PUBLISH_PERMISSION.into()],
        };
        let wallet = "0x1111111111111111111111111111111111111111";

        // Simulate a producer-side dependency failure after admission has
        // inserted the idempotency, inbox, and outbox records. A closed
        // read-only plan pool forces the handler to fail before commit; all
        // three records must disappear together so a retry can safely replay
        // the same source event instead of seeing a phantom accepted request.
        let rollback_event_id = format!("runtime-publisher-rollback-{}", Uuid::new_v4().simple());
        let rollback_request = PublishNotificationRequest {
            event_id: rollback_event_id.clone(),
            event_type: "payment.completed".into(),
            aggregate_id: rollback_event_id.clone(),
            idempotency_key: rollback_event_id.clone(),
            recipient_wallet_address: "all".into(),
            notification_type: "payment".into(),
            priority: "normal".into(),
            title: "Runtime publisher rollback".into(),
            message: "Producer rollback audit".into(),
            data: None,
            action_url: None,
            expires_at: None,
            plan_id: Some(Uuid::new_v4()),
        };
        let rollback_plan_db = connect_read_only_plan_pool(&plan_database_url).await?;
        rollback_plan_db.close().await;
        let mut rollback_state = state.clone();
        rollback_state.plan_db = Some(rollback_plan_db);
        assert_eq!(
            publish_notification(
                State(rollback_state),
                Extension(principal.clone()),
                Json(rollback_request),
            )
            .await
            .expect_err("producer dependency failure must roll back admission"),
            StatusCode::SERVICE_UNAVAILABLE
        );
        let rollback_idempotency: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM public.notification_request_idempotency WHERE idempotency_key = $1",
        )
        .bind(&rollback_event_id)
        .fetch_one(&db)
        .await?;
        let rollback_inbox: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM public.notification_inbox WHERE event_id = $1",
        )
        .bind(&rollback_event_id)
        .fetch_one(&db)
        .await?;
        let rollback_outbox: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM public.notification_outbox WHERE event_id = $1",
        )
        .bind(&rollback_event_id)
        .fetch_one(&db)
        .await?;
        assert_eq!(rollback_idempotency, 0);
        assert_eq!(rollback_inbox, 0);
        assert_eq!(rollback_outbox, 0);

        let event_id = format!("runtime-publisher-{}", Uuid::new_v4().simple());
        let request = PublishNotificationRequest {
            event_id: event_id.clone(),
            event_type: "payment.completed".into(),
            aggregate_id: event_id.clone(),
            idempotency_key: event_id.clone(),
            recipient_wallet_address: wallet.into(),
            notification_type: "payment".into(),
            priority: "normal".into(),
            title: "Runtime publisher".into(),
            message: "Publisher replay audit".into(),
            data: Some(serde_json::json!({"runtime_event": event_id})),
            action_url: Some("/notifications/runtime-publisher".into()),
            expires_at: None,
            plan_id: None,
        };
        let first = publish_notification(
            State(state.clone()),
            Extension(principal.clone()),
            Json(request.clone()),
        )
        .await
        .map_err(|status| format!("publisher first admission failed: {status}"))?;
        assert_eq!(first.status(), StatusCode::ACCEPTED);
        let replay = publish_notification(
            State(state.clone()),
            Extension(principal.clone()),
            Json(request.clone()),
        )
        .await
        .map_err(|status| format!("publisher idempotent replay failed: {status}"))?;
        assert_eq!(replay.status(), StatusCode::ACCEPTED);

        let mut alternate_key = request.clone();
        alternate_key.idempotency_key = format!("{event_id}-alternate");
        alternate_key.message = "mismatched replay".into();
        assert_eq!(
            publish_notification(
                State(state.clone()),
                Extension(principal.clone()),
                Json(alternate_key),
            )
            .await
            .expect_err("payload mismatch must conflict"),
            StatusCode::CONFLICT
        );

        let concrete_count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM public.notifications WHERE user_id = $1 AND body = 'Publisher replay audit'",
        )
        .bind(wallet)
        .fetch_one(&db)
        .await?;
        let concrete_jobs: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM public.notification_channel_jobs WHERE source_event_id = $1",
        )
        .bind(&event_id)
        .fetch_one(&db)
        .await?;
        let outbox_count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM public.notification_outbox WHERE event_id = $1",
        )
        .bind(&event_id)
        .fetch_one(&db)
        .await?;
        assert_eq!(concrete_count, 1);
        assert_eq!(concrete_jobs, 1);
        assert_eq!(outbox_count, 1);

        let plan_id = Uuid::new_v4();
        sqlx::query(
            "INSERT INTO public.wallet_plan_assignments (wallet_address, plan_id, is_active, expires_at) VALUES ($1, $2, TRUE, NULL)",
        )
        .bind(wallet)
        .bind(plan_id)
        .execute(&plan_setup)
        .await?;
        let plan_event_id = format!("runtime-plan-{}", Uuid::new_v4().simple());
        let plan_request = PublishNotificationRequest {
            event_id: plan_event_id.clone(),
            event_type: "payment.completed".into(),
            aggregate_id: plan_event_id.clone(),
            idempotency_key: plan_event_id.clone(),
            recipient_wallet_address: "all".into(),
            notification_type: "payment".into(),
            priority: "normal".into(),
            title: "Runtime plan".into(),
            message: "Plan fanout audit".into(),
            data: None,
            action_url: Some("/notifications/runtime-plan".into()),
            expires_at: None,
            plan_id: Some(plan_id),
        };
        let plan_response = publish_notification(
            State(state.clone()),
            Extension(principal.clone()),
            Json(plan_request),
        )
        .await
        .map_err(|status| format!("plan admission failed: {status}"))?;
        assert_eq!(plan_response.status(), StatusCode::ACCEPTED);
        let plan_notifications: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM public.notifications WHERE user_id = $1 AND body = 'Plan fanout audit'",
        )
        .bind(wallet)
        .fetch_one(&db)
        .await?;
        let plan_jobs: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM public.notification_channel_jobs WHERE source_event_id = $1 AND recipient = $2",
        )
        .bind(&plan_event_id)
        .bind(wallet)
        .fetch_one(&db)
        .await?;
        assert_eq!(plan_notifications, 1);
        assert_eq!(plan_jobs, 1);

        let broadcast_event_id = format!("runtime-broadcast-{}", Uuid::new_v4().simple());
        let broadcast = PublishNotificationRequest {
            event_id: broadcast_event_id.clone(),
            event_type: "notification.broadcast".into(),
            aggregate_id: "all".into(),
            idempotency_key: broadcast_event_id.clone(),
            recipient_wallet_address: "all".into(),
            notification_type: "announcement".into(),
            priority: "normal".into(),
            title: "Runtime broadcast".into(),
            message: "Broadcast replay audit".into(),
            data: None,
            action_url: None,
            expires_at: None,
            plan_id: None,
        };
        let broadcast_hash = publish_request_hash(&broadcast)
            .map_err(|status| format!("broadcast hash failed: {status}"))?;
        let first_broadcast = publish_notification(
            State(state.clone()),
            Extension(principal.clone()),
            Json(broadcast.clone()),
        )
        .await
        .map_err(|status| format!("broadcast admission failed: {status}"))?;
        assert_eq!(first_broadcast.status(), StatusCode::ACCEPTED);
        let second_broadcast = publish_notification(
            State(state.clone()),
            Extension(principal.clone()),
            Json(broadcast),
        )
        .await
        .map_err(|status| format!("broadcast replay failed: {status}"))?;
        assert_eq!(second_broadcast.status(), StatusCode::ACCEPTED);
        let broadcast_id = format!("0x{}", &broadcast_hash[..64]);
        let broadcast_count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM public.notifications WHERE id = $1 AND user_id IS NULL AND lower(recipient) = 'all'",
        )
        .bind(&broadcast_id)
        .fetch_one(&db)
        .await?;
        assert_eq!(broadcast_count, 1);

        let concrete_id: String = sqlx::query_scalar(
            "SELECT id FROM public.notifications WHERE user_id = $1 AND body = 'Publisher replay audit' LIMIT 1",
        )
        .bind(wallet)
        .fetch_one(&db)
        .await?;
        sqlx::query("DELETE FROM public.notification_delivery_attempts WHERE job_id IN (SELECT id FROM public.notification_channel_jobs WHERE source_event_id = $1)")
            .bind(&event_id)
            .execute(&db)
            .await?;
        sqlx::query("DELETE FROM public.notification_delivery_attempts WHERE job_id IN (SELECT id FROM public.notification_channel_jobs WHERE source_event_id = $1)")
            .bind(&plan_event_id)
            .execute(&db)
            .await?;
        sqlx::query("DELETE FROM public.notification_channel_jobs WHERE source_event_id = $1")
            .bind(&event_id)
            .execute(&db)
            .await?;
        sqlx::query("DELETE FROM public.notification_channel_jobs WHERE source_event_id = $1")
            .bind(&plan_event_id)
            .execute(&db)
            .await?;
        sqlx::query("DELETE FROM public.notifications WHERE id IN ($1, $2)")
            .bind(&concrete_id)
            .bind(&broadcast_id)
            .execute(&db)
            .await?;
        sqlx::query(
            "DELETE FROM public.notifications WHERE user_id = $1 AND body = 'Plan fanout audit'",
        )
        .bind(wallet)
        .execute(&db)
        .await?;
        sqlx::query("DELETE FROM public.notification_inbox WHERE event_id IN ($1, $2)")
            .bind(&event_id)
            .bind(&broadcast_event_id)
            .execute(&db)
            .await?;
        sqlx::query("DELETE FROM public.notification_inbox WHERE event_id = $1")
            .bind(&plan_event_id)
            .execute(&db)
            .await?;
        sqlx::query("DELETE FROM public.notification_request_idempotency WHERE principal_subject = $1 AND idempotency_key LIKE 'runtime-%'")
            .bind(&principal.subject)
            .execute(&db)
            .await?;
        sqlx::query("DELETE FROM public.notification_outbox WHERE event_id IN ($1, $2)")
            .bind(&event_id)
            .bind(&broadcast_event_id)
            .execute(&db)
            .await?;
        sqlx::query("DELETE FROM public.notification_outbox WHERE event_id = $1")
            .bind(&plan_event_id)
            .execute(&db)
            .await?;
        sqlx::query("DELETE FROM public.wallet_plan_assignments WHERE plan_id = $1")
            .bind(plan_id)
            .execute(&plan_setup)
            .await?;
        db.close().await;
        plan_db.close().await;
        plan_setup.close().await;
        Ok(())
    }

    #[tokio::test]
    #[ignore = "requires a disposable migrated local notification database"]
    async fn runtime_owner_list_filters_match_source_semantics(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let database_url = std::env::var("NOTIFICATION_RUNTIME_DATABASE_URL")?;
        let db = sqlx::PgPool::connect(&database_url).await?;
        let mut handlebars = Handlebars::new();
        handlebars.set_strict_mode(true);
        let state = AppState {
            db: db.clone(),
            plan_db: None,
            templates: Arc::new(RwLock::new(handlebars)),
            smtp: Arc::new(RwLock::new(None)),
            provider_signing_secrets: Vec::new(),
            realtime_slots: Arc::new(tokio::sync::Semaphore::new(MAX_REALTIME_CONNECTIONS)),
            stream_metrics: Arc::new(StreamMetrics::default()),
            vapid_key_id: "active".into(),
            vapid_public_key: None,
            vapid_private_key: None,
            vapid_previous_key_id: None,
            vapid_previous_private_key: None,
            from: "runtime@example.test".into(),
            from_name: "Runtime audit".into(),
            redis: None,
            realtime_notify: Arc::new(Notify::new()),
        };
        let owner = "0x4444444444444444444444444444444444444444";
        let principal = VerifiedPrincipal {
            subject: owner.into(),
            wallet_address: owner.into(),
            audience: FRONTEND_AUDIENCE.into(),
            permissions: Vec::new(),
        };
        let suffix = Uuid::new_v4().simple().to_string();
        let selected_id = format!("runtime-owner-selected-{suffix}");
        let read_id = format!("runtime-owner-read-{suffix}");
        let other_id = format!("runtime-owner-other-{suffix}");
        let broadcast_id = format!("runtime-owner-broadcast-{suffix}");
        let now = Utc::now();
        macro_rules! insert_notification {
            ($id:expr, $user_id:expr, $recipient:expr, $body:expr, $status:expr, $created_at:expr, $notification_type:expr, $priority:expr $(,)?) => {
                sqlx::query(
                    "INSERT INTO public.notifications (id, user_id, channel, recipient, body, status, created_at, notification_type, priority) VALUES ($1, $2, 'in_app', $3, $4, $5, $6, $7, $8)",
                )
                .bind($id)
                .bind($user_id)
                .bind($recipient)
                .bind($body)
                .bind($status)
                .bind($created_at)
                .bind($notification_type)
                .bind($priority)
                .execute(&db)
                .await?;
            };
        }
        insert_notification!(
            &selected_id,
            Some(owner),
            owner,
            "owner selected",
            "sent",
            now - chrono::Duration::hours(1),
            "payment",
            "high",
        );
        insert_notification!(
            &read_id,
            Some(owner),
            owner,
            "owner read",
            "sent",
            now - chrono::Duration::hours(2),
            "payment",
            "high",
        );
        insert_notification!(
            &other_id,
            Some(owner),
            owner,
            "owner other",
            "failed",
            now - chrono::Duration::hours(3),
            "security",
            "low",
        );
        insert_notification!(
            &broadcast_id,
            None::<&str>,
            "all",
            "broadcast selected",
            "sent",
            now - chrono::Duration::hours(4),
            "payment",
            "high",
        );
        sqlx::query(
            "INSERT INTO public.notification_engagement (notification_id, owner_id, read_at, updated_at) VALUES ($1, $2, $3, NOW())",
        )
        .bind(&read_id)
        .bind(owner)
        .bind(now - chrono::Duration::hours(1))
        .execute(&db)
        .await?;

        let start = (now - chrono::Duration::hours(5)).to_rfc3339();
        let end = (now + chrono::Duration::minutes(1)).to_rfc3339();
        let mut unread = std::collections::HashMap::new();
        unread.insert("status".into(), "unread".into());
        unread.insert("type".into(), "payment".into());
        unread.insert("priority".into(), "high".into());
        unread.insert("start_date".into(), start.clone());
        unread.insert("end_date".into(), end.clone());
        let Json(unread_response) = list_notifications(
            State(state.clone()),
            Extension(principal.clone()),
            axum::extract::Query(unread),
        )
        .await
        .map_err(|status| format!("unread owner list failed: {status}"))?;
        assert_eq!(unread_response.total, 2);
        assert_eq!(unread_response.items.len(), 2);
        assert!(unread_response
            .items
            .iter()
            .all(|item| item.notification_type.as_deref() == Some("payment")));
        assert!(unread_response
            .items
            .iter()
            .any(|item| item.id == selected_id));
        assert!(unread_response
            .items
            .iter()
            .any(|item| item.id == broadcast_id));

        let mut read = std::collections::HashMap::new();
        read.insert("status".into(), "read".into());
        read.insert("type".into(), "payment".into());
        read.insert("priority".into(), "high".into());
        let Json(read_response) = list_notifications(
            State(state.clone()),
            Extension(principal.clone()),
            axum::extract::Query(read),
        )
        .await
        .map_err(|status| format!("read owner list failed: {status}"))?;
        assert_eq!(read_response.total, 1);
        assert_eq!(read_response.items[0].id, read_id);
        assert!(read_response.items[0].read_at.is_some());

        let mut paged = std::collections::HashMap::new();
        paged.insert("status".into(), "all".into());
        paged.insert("type".into(), "payment".into());
        paged.insert("priority".into(), "high".into());
        paged.insert("start_date".into(), start);
        paged.insert("end_date".into(), end);
        paged.insert("limit".into(), "1".into());
        paged.insert("offset".into(), "1".into());
        let Json(paged_response) = list_notifications(
            State(state.clone()),
            Extension(principal),
            axum::extract::Query(paged),
        )
        .await
        .map_err(|status| format!("paged owner list failed: {status}"))?;
        assert_eq!(paged_response.total, 3);
        assert_eq!(paged_response.items.len(), 1);
        assert_ne!(paged_response.items[0].id, other_id);

        sqlx::query("DELETE FROM public.notification_engagement WHERE notification_id = $1")
            .bind(&read_id)
            .execute(&db)
            .await?;
        sqlx::query("DELETE FROM public.notifications WHERE id = ANY($1)")
            .bind(vec![selected_id, read_id, other_id, broadcast_id])
            .execute(&db)
            .await?;
        db.close().await;
        Ok(())
    }

    #[tokio::test]
    #[ignore = "requires a disposable migrated local notification database"]
    async fn runtime_metrics_snapshot_is_redacted_and_bounded(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let database_url = std::env::var("NOTIFICATION_RUNTIME_DATABASE_URL")?;
        let db = sqlx::PgPool::connect(&database_url).await?;
        let mut handlebars = Handlebars::new();
        handlebars.set_strict_mode(true);
        let state = AppState {
            db: db.clone(),
            plan_db: None,
            templates: Arc::new(RwLock::new(handlebars)),
            smtp: Arc::new(RwLock::new(None)),
            provider_signing_secrets: Vec::new(),
            realtime_slots: Arc::new(tokio::sync::Semaphore::new(MAX_REALTIME_CONNECTIONS)),
            stream_metrics: Arc::new(StreamMetrics::default()),
            vapid_key_id: "active".into(),
            vapid_public_key: None,
            vapid_private_key: None,
            vapid_previous_key_id: None,
            vapid_previous_private_key: None,
            from: "runtime@example.test".into(),
            from_name: "Runtime audit".into(),
            redis: None,
            realtime_notify: Arc::new(Notify::new()),
        };
        let principal = VerifiedPrincipal {
            subject: "svc:metrics-runtime-audit".into(),
            wallet_address: "svc:metrics-runtime-audit".into(),
            audience: ADMIN_AUDIENCE.into(),
            permissions: vec![NOTIFICATIONS_MANAGE_PERMISSION.into()],
        };
        let suffix = Uuid::new_v4().simple().to_string();
        let notification_id = format!("runtime-metrics-notification-{suffix}");
        let event_id = format!("runtime-metrics-event-{suffix}");
        let job_id = format!("runtime-metrics-job-{suffix}");
        let owner = "0x5555555555555555555555555555555555555555";

        sqlx::query(
            "INSERT INTO public.notifications (id, user_id, channel, recipient, body, status) VALUES ($1, $2, 'email', 'metrics@example.test', 'metrics body must never be returned', 'suppressed')",
        )
        .bind(&notification_id)
        .bind(owner)
        .execute(&db)
        .await?;
        sqlx::query(
            "INSERT INTO public.notification_outbox (event_id, event_type, aggregate_id, payload) VALUES ($1, 'metrics.runtime', $2, '{}'::jsonb)",
        )
        .bind(&event_id)
        .bind(&notification_id)
        .execute(&db)
        .await?;
        sqlx::query(
            "INSERT INTO public.notification_channel_jobs (id, source_event_id, notification_id, channel, recipient, idempotency_key, state) VALUES ($1, $2, $3, 'email', $4, $5, 'queued')",
        )
        .bind(&job_id)
        .bind(&event_id)
        .bind(&notification_id)
        .bind(owner)
        .bind(format!("{job_id}:key"))
        .execute(&db)
        .await?;
        sqlx::query(
            "INSERT INTO public.notification_provider_events (provider, provider_event_id, job_id, event_type, payload) VALUES ('metrics', $1, $2, 'accepted', '{}'::jsonb)",
        )
        .bind(format!("event-{suffix}"))
        .bind(&job_id)
        .execute(&db)
        .await?;
        sqlx::query(
            "INSERT INTO public.notification_replay_cursors (owner_id, stream, last_event_id) VALUES ($1, 'owner', $2)",
        )
        .bind(owner)
        .bind(&notification_id)
        .execute(&db)
        .await?;

        let Json(metrics) = admin_metrics(State(state.clone()), Extension(principal))
            .await
            .map_err(|status| format!("metrics endpoint failed: {status}"))?;
        let value = serde_json::to_value(&metrics)?;
        let object = value
            .as_object()
            .ok_or("metrics response was not an object")?;
        let mut keys = object.keys().cloned().collect::<Vec<_>>();
        keys.sort();
        assert_eq!(
            keys,
            [
                "active_streams",
                "attempting",
                "channel_outcomes",
                "dead_lettered",
                "delivery_attempts",
                "provider_accepted",
                "provider_events",
                "queue_age_seconds",
                "queue_depth",
                "replay_cursor_age_seconds",
                "replay_cursors",
                "retry_wait",
                "stream_connections_total",
                "stream_lag_seconds",
                "stream_query_failures_total",
                "stream_reconnects_total",
                "stream_replayed_events_total",
                "suppressed",
                "terminal_failed",
            ]
            .into_iter()
            .map(str::to_owned)
            .collect::<Vec<_>>()
        );
        assert_eq!(object["queue_depth"], 1);
        assert_eq!(object["suppressed"], 1);
        assert_eq!(object["provider_events"], 1);
        assert_eq!(object["replay_cursors"], 1);
        assert_eq!(object["channel_outcomes"]["email"], 1);
        let serialized = value.to_string();
        for forbidden in ["0x5555", "metrics body", "recipient", "payload", "token"] {
            assert!(
                !serialized.contains(forbidden),
                "metrics leaked forbidden material: {forbidden}"
            );
        }

        sqlx::query("DELETE FROM public.notification_provider_events WHERE job_id = $1")
            .bind(&job_id)
            .execute(&db)
            .await?;
        sqlx::query("DELETE FROM public.notification_replay_cursors WHERE owner_id = $1")
            .bind(owner)
            .execute(&db)
            .await?;
        sqlx::query("DELETE FROM public.notification_channel_jobs WHERE id = $1")
            .bind(&job_id)
            .execute(&db)
            .await?;
        sqlx::query("DELETE FROM public.notification_outbox WHERE event_id = $1")
            .bind(&event_id)
            .execute(&db)
            .await?;
        sqlx::query("DELETE FROM public.notifications WHERE id = $1")
            .bind(&notification_id)
            .execute(&db)
            .await?;
        db.close().await;
        Ok(())
    }

    #[tokio::test]
    #[ignore = "requires a disposable migrated local notification database"]
    async fn runtime_preferences_enforce_quiet_hours_and_suppression(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let database_url = std::env::var("NOTIFICATION_RUNTIME_DATABASE_URL")?;
        let db = sqlx::PgPool::connect(&database_url).await?;
        let mut handlebars = Handlebars::new();
        handlebars.set_strict_mode(true);
        let state = AppState {
            db: db.clone(),
            plan_db: None,
            templates: Arc::new(RwLock::new(handlebars)),
            smtp: Arc::new(RwLock::new(None)),
            provider_signing_secrets: Vec::new(),
            realtime_slots: Arc::new(tokio::sync::Semaphore::new(MAX_REALTIME_CONNECTIONS)),
            stream_metrics: Arc::new(StreamMetrics::default()),
            vapid_key_id: "active".into(),
            vapid_public_key: Some("B".repeat(65)),
            vapid_private_key: Some(Arc::new(b"runtime-private-key".to_vec())),
            vapid_previous_key_id: None,
            vapid_previous_private_key: None,
            from: "runtime@example.test".into(),
            from_name: "Runtime audit".into(),
            redis: None,
            realtime_notify: Arc::new(Notify::new()),
        };
        let owner = "0x2222222222222222222222222222222222222222";
        let principal = VerifiedPrincipal {
            subject: owner.into(),
            wallet_address: owner.into(),
            audience: FRONTEND_AUDIENCE.into(),
            permissions: Vec::new(),
        };
        let now = Utc::now();
        let preferences = update_preferences(
            State(state.clone()),
            Extension(principal.clone()),
            Json(NotificationPreferencesRequest {
                channels: serde_json::json!({"in_app": false}),
                quiet_hours: Some(serde_json::json!({
                    "enabled": true,
                    "start": (now - chrono::Duration::hours(1)).format("%H:%M").to_string(),
                    "end": (now + chrono::Duration::hours(1)).format("%H:%M").to_string()
                })),
                timezone: Some("UTC".into()),
            }),
        )
        .await
        .map_err(|status| format!("preference update failed: {status}"))?;
        assert_eq!(preferences.0.channels["in_app"], false);
        assert_eq!(preferences.0.timezone.as_deref(), Some("UTC"));
        let policy = load_delivery_preference_policy(&db, Some(owner))
            .await
            .map_err(|status| format!("preference policy load failed: {status}"))?;
        assert!(!channel_preference_enabled(&policy.channels, "in_app"));
        assert!(policy.quiet_until.is_some());

        assert_eq!(
            update_preferences(
                State(state.clone()),
                Extension(principal.clone()),
                Json(NotificationPreferencesRequest {
                    channels: serde_json::json!({}),
                    quiet_hours: None,
                    timezone: Some("Not/A/Timezone".into()),
                }),
            )
            .await
            .map(|_| ())
            .expect_err("unknown timezone must fail closed"),
            StatusCode::BAD_REQUEST
        );

        let endpoint = "https://push.example.test/runtime-preferences";
        let subscribed = push_subscribe(
            State(state.clone()),
            Extension(principal.clone()),
            Json(PushSubscriptionRequest {
                endpoint: endpoint.into(),
                p256dh: "B".repeat(32),
                auth: "A".repeat(16),
                user_agent: Some("runtime-audit".into()),
            }),
        )
        .await
        .map_err(|status| format!("push subscribe failed: {status}"))?;
        assert!(subscribed.0.subscribed);
        assert!(subscribed.0.enabled);
        let push_key_id: String = sqlx::query_scalar(
            "SELECT vapid_key_id FROM public.notification_push_subscriptions WHERE endpoint = $1",
        )
        .bind(endpoint)
        .fetch_one(&db)
        .await?;
        assert_eq!(push_key_id, "active");
        let other_owner = VerifiedPrincipal {
            subject: "0x3333333333333333333333333333333333333333".into(),
            wallet_address: "0x3333333333333333333333333333333333333333".into(),
            audience: FRONTEND_AUDIENCE.into(),
            permissions: Vec::new(),
        };
        assert_eq!(
            push_subscribe(
                State(state.clone()),
                Extension(other_owner),
                Json(PushSubscriptionRequest {
                    endpoint: endpoint.into(),
                    p256dh: "B".repeat(32),
                    auth: "A".repeat(16),
                    user_agent: None,
                }),
            )
            .await
            .map(|_| ())
            .expect_err("push endpoint ownership must be enforced"),
            StatusCode::FORBIDDEN
        );
        let unsubscribed = push_unsubscribe(
            State(state.clone()),
            Extension(principal.clone()),
            Json(PushUnsubscribeRequest {
                endpoint: endpoint.into(),
            }),
        )
        .await
        .map_err(|status| format!("push unsubscribe failed: {status}"))?;
        assert!(!unsubscribed.0.subscribed);

        let admin_principal = VerifiedPrincipal {
            subject: "runtime-notification-admin".into(),
            wallet_address: owner.into(),
            audience: ADMIN_AUDIENCE.into(),
            permissions: vec![NOTIFICATIONS_MANAGE_PERMISSION.into()],
        };
        let mut send_headers = HeaderMap::new();
        send_headers.insert(
            "idempotency-key",
            "runtime-preferences-suppression".parse()?,
        );
        let send_response = send_notification(
            State(state),
            Extension(admin_principal),
            send_headers,
            Json(SendNotificationRequest {
                user_id: Some(owner.into()),
                channel: "in_app".into(),
                recipient: owner.into(),
                template_id: None,
                subject: None,
                body: Some("Preference runtime audit".into()),
                data: None,
                expires_at: None,
            }),
        )
        .await
        .map_err(|status| format!("suppressed send failed: {status}"))?;
        assert_eq!(send_response.status(), StatusCode::ACCEPTED);
        let (notification_id, status, error): (String, String, String) = sqlx::query_as(
            "SELECT id, status, error FROM public.notifications WHERE user_id = $1 AND body = 'Preference runtime audit' ORDER BY created_at DESC LIMIT 1",
        )
        .bind(owner)
        .fetch_one(&db)
        .await?;
        assert_eq!(status, "suppressed");
        assert_eq!(error, "channel_disabled_by_preference");
        let job_count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM public.notification_channel_jobs WHERE notification_id = $1",
        )
        .bind(&notification_id)
        .fetch_one(&db)
        .await?;
        assert_eq!(job_count, 0);

        sqlx::query("DELETE FROM public.notification_outbox WHERE event_id = $1")
            .bind(&notification_id)
            .execute(&db)
            .await?;
        sqlx::query("DELETE FROM public.notifications WHERE id = $1")
            .bind(&notification_id)
            .execute(&db)
            .await?;
        sqlx::query("DELETE FROM public.notification_preferences WHERE user_id = $1")
            .bind(owner)
            .execute(&db)
            .await?;
        sqlx::query("DELETE FROM public.notification_push_subscriptions WHERE endpoint = $1")
            .bind(endpoint)
            .execute(&db)
            .await?;
        db.close().await;
        Ok(())
    }

    #[tokio::test]
    #[ignore = "requires a disposable migrated local notification database"]
    async fn runtime_stream_cursor_acknowledgement_is_owner_bound(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let database_url = std::env::var("NOTIFICATION_RUNTIME_DATABASE_URL")?;
        let db = sqlx::PgPool::connect(&database_url).await?;
        let owner = "0x4444444444444444444444444444444444444444";
        let notification_id = "runtime-stream-notification";
        sqlx::query(
            "INSERT INTO public.notifications (id, user_id, channel, recipient, body, status) VALUES ($1, $2, 'in_app', $2, 'stream runtime audit', 'pending')",
        )
        .bind(notification_id)
        .bind(owner)
        .execute(&db)
        .await?;

        let mut handlebars = Handlebars::new();
        handlebars.set_strict_mode(true);
        let state = AppState {
            db: db.clone(),
            plan_db: None,
            templates: Arc::new(RwLock::new(handlebars)),
            smtp: Arc::new(RwLock::new(None)),
            provider_signing_secrets: Vec::new(),
            realtime_slots: Arc::new(tokio::sync::Semaphore::new(MAX_REALTIME_CONNECTIONS)),
            stream_metrics: Arc::new(StreamMetrics::default()),
            vapid_key_id: "active".into(),
            vapid_public_key: None,
            vapid_private_key: None,
            vapid_previous_key_id: None,
            vapid_previous_private_key: None,
            from: "runtime@example.test".into(),
            from_name: "Runtime audit".into(),
            redis: None,
            realtime_notify: Arc::new(Notify::new()),
        };
        let principal = VerifiedPrincipal {
            subject: owner.into(),
            wallet_address: owner.into(),
            audience: FRONTEND_AUDIENCE.into(),
            permissions: Vec::new(),
        };

        let initial_response = notification_stream(
            State(state.clone()),
            Extension(principal.clone()),
            axum::http::HeaderMap::new(),
        )
        .await
        .expect("an owner may open the durable event stream")
        .into_response();
        assert!(
            initial_response
                .headers()
                .get(axum::http::header::CONTENT_TYPE)
                .and_then(|value| value.to_str().ok())
                .is_some_and(|value| value.starts_with("text/event-stream")),
            "stream must expose an SSE content type"
        );
        let mut initial_stream = initial_response.into_body().into_data_stream();
        let first_chunk =
            tokio::time::timeout(std::time::Duration::from_secs(2), initial_stream.next())
                .await
                .expect("first SSE notification should arrive")
                .expect("SSE body should yield a first chunk")
                .expect("SSE body chunk should be readable");
        let first_wire = String::from_utf8(first_chunk.to_vec()).expect("SSE is UTF-8");
        assert!(first_wire.contains("id: runtime-stream-notification"));
        drop(initial_stream);

        let acknowledged = acknowledge_stream(
            State(state.clone()),
            Extension(principal.clone()),
            Json(StreamAcknowledgementRequest {
                event_id: notification_id.into(),
            }),
        )
        .await
        .map_err(|status| format!("stream acknowledgement failed: {status}"))?;
        assert_eq!(acknowledged.0["ok"], true);
        assert_eq!(
            persisted_stream_cursor(&db, owner)
                .await
                .map_err(|status| format!("stream cursor read failed: {status}"))?,
            Some(notification_id.into())
        );

        let next_notification_id = "runtime-stream-notification-next";
        sqlx::query(
            "INSERT INTO public.notifications (id, user_id, channel, recipient, body, status) VALUES ($1, $2, 'in_app', $2, 'stream reconnect audit', 'pending')",
        )
        .bind(next_notification_id)
        .bind(owner)
        .execute(&db)
        .await?;
        let mut reconnect_headers = axum::http::HeaderMap::new();
        reconnect_headers.insert(
            axum::http::HeaderName::from_static("last-event-id"),
            axum::http::HeaderValue::from_static(notification_id),
        );
        let reconnect_response = notification_stream(
            State(state.clone()),
            Extension(principal.clone()),
            reconnect_headers,
        )
        .await
        .expect("an owner may reconnect from its durable event cursor")
        .into_response();
        let mut reconnect_stream = reconnect_response.into_body().into_data_stream();
        let reconnect_chunk =
            tokio::time::timeout(std::time::Duration::from_secs(2), reconnect_stream.next())
                .await
                .expect("reconnected SSE notification should arrive")
                .expect("reconnected SSE body should yield a chunk")
                .expect("reconnected SSE body chunk should be readable");
        let reconnect_wire = String::from_utf8(reconnect_chunk.to_vec()).expect("SSE is UTF-8");
        assert!(reconnect_wire.contains("id: runtime-stream-notification-next"));
        drop(reconnect_stream);

        let other_owner = VerifiedPrincipal {
            subject: "0x5555555555555555555555555555555555555555".into(),
            wallet_address: "0x5555555555555555555555555555555555555555".into(),
            audience: FRONTEND_AUDIENCE.into(),
            permissions: Vec::new(),
        };
        assert_eq!(
            acknowledge_stream(
                State(state),
                Extension(other_owner),
                Json(StreamAcknowledgementRequest {
                    event_id: notification_id.into(),
                }),
            )
            .await
            .expect_err("a different owner must not acknowledge this cursor"),
            StatusCode::NOT_FOUND
        );

        sqlx::query("DELETE FROM public.notification_replay_cursors WHERE owner_id = $1")
            .bind(owner)
            .execute(&db)
            .await?;
        sqlx::query("DELETE FROM public.notifications WHERE id = $1")
            .bind(notification_id)
            .execute(&db)
            .await?;
        sqlx::query("DELETE FROM public.notifications WHERE id = $1")
            .bind(next_notification_id)
            .execute(&db)
            .await?;
        db.close().await;
        Ok(())
    }

    #[tokio::test]
    #[ignore = "requires a disposable migrated local notification database"]
    async fn runtime_owner_delete_cleans_lifecycle_dependencies(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let database_url = std::env::var("NOTIFICATION_RUNTIME_DATABASE_URL")?;
        let db = sqlx::PgPool::connect(&database_url).await?;
        let mut handlebars = Handlebars::new();
        handlebars.set_strict_mode(true);
        let state = AppState {
            db: db.clone(),
            plan_db: None,
            templates: Arc::new(RwLock::new(handlebars)),
            smtp: Arc::new(RwLock::new(None)),
            provider_signing_secrets: Vec::new(),
            realtime_slots: Arc::new(tokio::sync::Semaphore::new(MAX_REALTIME_CONNECTIONS)),
            stream_metrics: Arc::new(StreamMetrics::default()),
            vapid_key_id: "active".into(),
            vapid_public_key: None,
            vapid_private_key: None,
            vapid_previous_key_id: None,
            vapid_previous_private_key: None,
            from: "runtime@example.test".into(),
            from_name: "Runtime audit".into(),
            redis: None,
            realtime_notify: Arc::new(Notify::new()),
        };
        let owner = "0x3333333333333333333333333333333333333333";
        let principal = VerifiedPrincipal {
            subject: owner.into(),
            wallet_address: owner.into(),
            audience: FRONTEND_AUDIENCE.into(),
            permissions: Vec::new(),
        };
        let suffix = Uuid::new_v4().simple().to_string();
        let notification_id = format!("runtime-erase-{suffix}");
        let event_id = format!("runtime-erase-event-{suffix}");
        let job_id = format!("runtime-erase-job-{suffix}");
        let broadcast_id = format!("runtime-erase-broadcast-{suffix}");
        sqlx::query(
            "INSERT INTO public.notifications (id, user_id, channel, recipient, body, status) VALUES ($1, $2, 'email', $2, 'Owner erase audit', 'failed')",
        )
        .bind(&notification_id)
        .bind(owner)
        .execute(&db)
        .await?;
        sqlx::query(
            "INSERT INTO public.notification_outbox (event_id, event_type, aggregate_id, payload) VALUES ($1, 'runtime.erase', $2, '{}'::jsonb)",
        )
        .bind(&event_id)
        .bind(&notification_id)
        .execute(&db)
        .await?;
        sqlx::query(
            "INSERT INTO public.notification_channel_jobs (id, source_event_id, notification_id, channel, recipient, state, attempt_count, idempotency_key) VALUES ($1, $2, $3, 'email', $4, 'terminal_failed', 1, $5)",
        )
        .bind(&job_id)
        .bind(&event_id)
        .bind(&notification_id)
        .bind(owner)
        .bind(format!("{job_id}:key"))
        .execute(&db)
        .await?;
        sqlx::query(
            "INSERT INTO public.notification_delivery_attempts (job_id, attempt_no, outcome, error_code) VALUES ($1, 1, 'permanent_failure', 'runtime_erase')",
        )
        .bind(&job_id)
        .execute(&db)
        .await?;
        sqlx::query(
            "INSERT INTO public.notification_dead_letters (job_id, reason, payload) VALUES ($1, 'runtime_erase', '{}'::jsonb)",
        )
        .bind(&job_id)
        .execute(&db)
        .await?;
        sqlx::query(
            "INSERT INTO public.notification_provider_events (provider, provider_event_id, job_id, event_type, payload) VALUES ('runtime', $1, $2, 'failed', '{}'::jsonb)",
        )
        .bind(format!("provider-{suffix}"))
        .bind(&job_id)
        .execute(&db)
        .await?;
        sqlx::query(
            "INSERT INTO public.notification_engagement (notification_id, owner_id, read_at) VALUES ($1, $2, NOW())",
        )
        .bind(&notification_id)
        .bind(owner)
        .execute(&db)
        .await?;
        sqlx::query(
            "INSERT INTO public.notifications (id, user_id, channel, recipient, body, status) VALUES ($1, NULL, 'in_app', 'all', 'Shared erase broadcast', 'sent')",
        )
        .bind(&broadcast_id)
        .execute(&db)
        .await?;
        sqlx::query(
            "INSERT INTO public.notification_engagement (notification_id, owner_id, read_at) VALUES ($1, $2, NOW())",
        )
        .bind(&broadcast_id)
        .bind(owner)
        .execute(&db)
        .await?;
        sqlx::query(
            "INSERT INTO public.notification_expirations (notification_id, expires_at) VALUES ($1, NOW() + INTERVAL '1 hour')",
        )
        .bind(&notification_id)
        .execute(&db)
        .await?;

        let read = mark_read(
            State(state.clone()),
            Extension(principal.clone()),
            AxPath(notification_id.clone()),
        )
        .await
        .map_err(|status| format!("owner mark-read failed: {status}"))?;
        assert_eq!(read.0.id, notification_id);
        let unread = mark_unread(
            State(state.clone()),
            Extension(principal.clone()),
            AxPath(notification_id.clone()),
        )
        .await
        .map_err(|status| format!("owner mark-unread failed: {status}"))?;
        assert_eq!(unread, StatusCode::NO_CONTENT);

        let broadcast_response = delete_notification(
            State(state.clone()),
            Extension(principal.clone()),
            AxPath(broadcast_id.clone()),
        )
        .await
        .map_err(|status| format!("broadcast owner delete failed: {status}"))?;
        assert_eq!(broadcast_response, StatusCode::NO_CONTENT);
        let broadcast_rows: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM public.notifications WHERE id = $1 AND user_id IS NULL AND lower(recipient) = 'all'",
        )
        .bind(&broadcast_id)
        .fetch_one(&db)
        .await?;
        let broadcast_engagement: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM public.notification_engagement WHERE notification_id = $1 AND owner_id = $2",
        )
        .bind(&broadcast_id)
        .bind(owner)
        .fetch_one(&db)
        .await?;
        assert_eq!(broadcast_rows, 1);
        assert_eq!(broadcast_engagement, 0);

        let response = delete_notification(
            State(state),
            Extension(principal),
            AxPath(notification_id.clone()),
        )
        .await
        .map_err(|status| format!("owner erase failed: {status}"))?;
        assert_eq!(response, StatusCode::NO_CONTENT);
        for (table, column, value) in [
            ("notifications", "id", notification_id.as_str()),
            (
                "notification_channel_jobs",
                "notification_id",
                notification_id.as_str(),
            ),
            ("notification_outbox", "event_id", event_id.as_str()),
            ("notification_delivery_attempts", "job_id", job_id.as_str()),
            ("notification_dead_letters", "job_id", job_id.as_str()),
            ("notification_provider_events", "job_id", job_id.as_str()),
            (
                "notification_engagement",
                "notification_id",
                notification_id.as_str(),
            ),
            (
                "notification_expirations",
                "notification_id",
                notification_id.as_str(),
            ),
        ] {
            let query = format!("SELECT COUNT(*) FROM public.{table} WHERE {column} = $1");
            let count: i64 = sqlx::query_scalar(&query)
                .bind(value)
                .fetch_one(&db)
                .await?;
            assert_eq!(count, 0, "{table}.{column} still contains erased data");
        }
        sqlx::query("DELETE FROM public.notifications WHERE id = $1")
            .bind(&broadcast_id)
            .execute(&db)
            .await?;
        db.close().await;
        Ok(())
    }

    #[tokio::test]
    #[ignore = "requires a disposable migrated local notification database"]
    async fn runtime_provider_callback_reconciles_and_deduplicates(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let database_url = std::env::var("NOTIFICATION_RUNTIME_DATABASE_URL")?;
        let db = sqlx::PgPool::connect(&database_url).await?;
        let mut handlebars = Handlebars::new();
        handlebars.set_strict_mode(true);
        let secret = b"runtime-provider-secret-012345678901234567890123".to_vec();
        let previous_secret = b"runtime-provider-previous-012345678901234567890".to_vec();
        let state = AppState {
            db: db.clone(),
            plan_db: None,
            templates: Arc::new(RwLock::new(handlebars)),
            smtp: Arc::new(RwLock::new(None)),
            provider_signing_secrets: vec![
                Arc::new(secret.clone()),
                Arc::new(previous_secret.clone()),
            ],
            realtime_slots: Arc::new(tokio::sync::Semaphore::new(MAX_REALTIME_CONNECTIONS)),
            stream_metrics: Arc::new(StreamMetrics::default()),
            vapid_key_id: "active".into(),
            vapid_public_key: None,
            vapid_private_key: None,
            vapid_previous_key_id: None,
            vapid_previous_private_key: None,
            from: "runtime@example.test".into(),
            from_name: "Runtime audit".into(),
            redis: None,
            realtime_notify: Arc::new(Notify::new()),
        };
        let principal = VerifiedPrincipal {
            subject: "provider-runtime-audit".into(),
            wallet_address: "provider-runtime-audit".into(),
            audience: "epsx-notification-provider".into(),
            permissions: vec!["internal:notifications:provider-events".into()],
        };
        let suffix = Uuid::new_v4().simple().to_string();
        let notification_id = format!("runtime-provider-notification-{suffix}");
        let event_id = format!("runtime-provider-event-{suffix}");
        let job_id = format!("runtime-provider-job-{suffix}");
        sqlx::query(
            "INSERT INTO public.notifications (id, user_id, channel, recipient, body, status) VALUES ($1, '0x4444444444444444444444444444444444444444', 'email', 'runtime@example.test', 'Provider callback audit', 'pending')",
        )
        .bind(&notification_id)
        .execute(&db)
        .await?;
        sqlx::query(
            "INSERT INTO public.notification_outbox (event_id, event_type, aggregate_id, payload) VALUES ($1, 'runtime.provider', $2, '{}'::jsonb)",
        )
        .bind(&event_id)
        .bind(&notification_id)
        .execute(&db)
        .await?;
        sqlx::query(
            "INSERT INTO public.notification_channel_jobs (id, source_event_id, notification_id, channel, recipient, state, attempt_count, idempotency_key) VALUES ($1, $2, $3, 'email', 'runtime@example.test', 'attempting', 1, $4)",
        )
        .bind(&job_id)
        .bind(&event_id)
        .bind(&notification_id)
        .bind(format!("{job_id}:key"))
        .execute(&db)
        .await?;

        let timestamp = Utc::now().timestamp();
        let accepted_body = serde_json::json!({
            "provider": "runtime",
            "provider_event_id": format!("accepted-{suffix}"),
            "provider_message_id": format!("message-{suffix}"),
            "job_id": job_id,
            "event_type": "delivered",
            "payload": {"provider": "runtime"},
            "occurred_at": Utc::now(),
        });
        let accepted_bytes = serde_json::to_vec(&accepted_body)?;
        // A callback signed by the previous key must remain accepted during
        // the bounded rotation window. This exercises the real handler and
        // durable reconciliation path, not only the environment parser.
        let accepted_signature =
            provider_test_signature(&previous_secret, timestamp, &accepted_bytes);
        let accepted_headers = provider_test_headers(timestamp, &accepted_signature);
        let first = record_provider_event(
            State(state.clone()),
            Extension(principal.clone()),
            accepted_headers.clone(),
            Bytes::from(accepted_bytes.clone()),
        )
        .await
        .map_err(|status| format!("provider callback failed: {status}"))?;
        assert_eq!(first.status(), StatusCode::ACCEPTED);
        let replay = record_provider_event(
            State(state.clone()),
            Extension(principal.clone()),
            accepted_headers,
            Bytes::from(accepted_bytes),
        )
        .await
        .map_err(|status| format!("provider callback replay failed: {status}"))?;
        assert_eq!(replay.status(), StatusCode::ACCEPTED);

        let (job_state, notification_status, event_count): (String, String, i64) = sqlx::query_as(
            "SELECT j.state, n.status, (SELECT COUNT(*) FROM public.notification_provider_events WHERE job_id = j.id) FROM public.notification_channel_jobs j JOIN public.notifications n ON n.id = j.notification_id WHERE j.id = $1",
        )
        .bind(&job_id)
        .fetch_one(&db)
        .await?;
        assert_eq!(job_state, "provider_accepted");
        assert_eq!(notification_status, "sent");
        assert_eq!(event_count, 1);

        let failed_body = serde_json::json!({
            "provider": "runtime",
            "provider_event_id": format!("failed-{suffix}"),
            "provider_message_id": format!("message-{suffix}"),
            "job_id": job_id,
            "event_type": "failed",
            "payload": {"error": "late failure"},
            "occurred_at": Utc::now(),
        });
        let failed_bytes = serde_json::to_vec(&failed_body)?;
        // The active key remains accepted while the previous key is still in
        // the overlap window.
        let failed_signature = provider_test_signature(&secret, timestamp, &failed_bytes);
        let failed = record_provider_event(
            State(state.clone()),
            Extension(principal.clone()),
            provider_test_headers(timestamp, &failed_signature),
            Bytes::from(failed_bytes),
        )
        .await
        .map_err(|status| format!("reordered provider callback failed: {status}"))?;
        assert_eq!(failed.status(), StatusCode::ACCEPTED);
        let state_after_reorder: String =
            sqlx::query_scalar("SELECT state FROM public.notification_channel_jobs WHERE id = $1")
                .bind(&job_id)
                .fetch_one(&db)
                .await?;
        assert_eq!(state_after_reorder, "provider_accepted");

        let mut invalid_headers = provider_test_headers(timestamp, "v1=00");
        invalid_headers.insert(
            "x-epsx-provider-signature",
            axum::http::HeaderValue::from_static("v1=00"),
        );
        let invalid = record_provider_event(
            State(state.clone()),
            Extension(principal),
            invalid_headers,
            Bytes::from_static(br#"{"provider":"runtime","provider_event_id":"bad","event_type":"delivered","payload":{}}"#),
        )
        .await
        .expect_err("invalid provider signature must fail closed");
        assert_eq!(invalid, StatusCode::UNAUTHORIZED);

        sqlx::query("DELETE FROM public.notification_provider_events WHERE job_id = $1")
            .bind(&job_id)
            .execute(&db)
            .await?;
        sqlx::query("DELETE FROM public.notification_channel_jobs WHERE id = $1")
            .bind(&job_id)
            .execute(&db)
            .await?;
        sqlx::query("DELETE FROM public.notification_outbox WHERE event_id = $1")
            .bind(&event_id)
            .execute(&db)
            .await?;
        sqlx::query("DELETE FROM public.notifications WHERE id = $1")
            .bind(&notification_id)
            .execute(&db)
            .await?;
        db.close().await;
        Ok(())
    }

    fn provider_test_signature(secret: &[u8], timestamp: i64, body: &[u8]) -> String {
        let mut input = b"epsx.notification.provider.v1.".to_vec();
        input.extend_from_slice(timestamp.to_string().as_bytes());
        input.push(b'.');
        input.extend_from_slice(body);
        let mut mac = Hmac::<Sha256>::new_from_slice(secret).unwrap();
        mac.update(&input);
        format!("v1={}", hex::encode(mac.finalize().into_bytes()))
    }

    fn provider_test_headers(timestamp: i64, signature: &str) -> HeaderMap {
        let mut headers = HeaderMap::new();
        headers.insert(
            "x-epsx-provider-timestamp",
            axum::http::HeaderValue::from_str(&timestamp.to_string()).unwrap(),
        );
        headers.insert(
            "x-epsx-provider-signature",
            axum::http::HeaderValue::from_str(signature).unwrap(),
        );
        headers
    }

    #[test]
    fn strict_template_registry_rejects_missing_variables() {
        let mut handlebars = Handlebars::new();
        handlebars.set_strict_mode(true);
        handlebars
            .register_template_string("strict", "Hello {{name}}")
            .unwrap();
        let values = HashMap::<String, serde_json::Value>::new();
        assert!(handlebars.render("strict", &values).is_err());
    }

    #[test]
    fn admin_query_defaults_and_accepts_only_bounded_limit_and_offset() {
        assert_eq!(
            AdminNotificationQuery::parse(None).unwrap(),
            AdminNotificationQuery {
                limit: 20,
                offset: 0,
                status: None,
                notification_type: None,
                priority: None,
                wallet_address: None,
            }
        );
        assert_eq!(
            AdminNotificationQuery::parse(Some("")).unwrap(),
            AdminNotificationQuery {
                limit: 20,
                offset: 0,
                status: None,
                notification_type: None,
                priority: None,
                wallet_address: None,
            }
        );
        assert_eq!(
            AdminNotificationQuery::parse(Some("offset=1000000&limit=50")).unwrap(),
            AdminNotificationQuery {
                limit: 50,
                offset: 1_000_000,
                status: None,
                notification_type: None,
                priority: None,
                wallet_address: None,
            }
        );
        assert_eq!(
            AdminNotificationQuery::parse(Some("limit=1&offset=0")).unwrap(),
            AdminNotificationQuery {
                limit: 1,
                offset: 0,
                status: None,
                notification_type: None,
                priority: None,
                wallet_address: None,
            }
        );
    }

    #[test]
    fn admin_query_accepts_bounded_inventory_filters_and_normalizes_wallets() {
        assert_eq!(
            AdminNotificationQuery::parse(Some(
                "status=suppressed&type=portfolio-alert&priority=urgent&wallet_address=0X1111111111111111111111111111111111111111"
            ))
            .unwrap(),
            AdminNotificationQuery {
                limit: 20,
                offset: 0,
                status: Some("suppressed".into()),
                notification_type: Some("portfolio-alert".into()),
                priority: Some("urgent".into()),
                wallet_address: Some("0x1111111111111111111111111111111111111111".into()),
            }
        );
        assert_eq!(
            AdminNotificationQuery::parse(Some("notification_type=system"))
                .unwrap()
                .notification_type,
            Some("system".into())
        );
        assert_eq!(
            AdminNotificationQuery::parse(Some("status=read"))
                .unwrap()
                .status,
            Some("read".into())
        );
    }

    #[test]
    fn admin_query_rejects_unknown_duplicate_malformed_and_out_of_bounds_values() {
        for raw in [
            "foo=sent",
            "status=sent&status=failed",
            "type=system&notification_type=system",
            "priority=medium",
            "wallet_address=0xattacker",
            "type=bad%20type",
            "limit=20&limit=10",
            "offset=0&offset=1",
            "limit",
            "limit=",
            "=20",
            "limit=+1",
            "limit=-1",
            "limit=1.0",
            "limit=%32%30",
            "limit=0",
            "limit=51",
            "offset=-1",
            "offset=1000001",
            "limit=20&",
            "&limit=20",
        ] {
            assert_eq!(
                AdminNotificationQuery::parse(Some(raw)),
                Err(StatusCode::BAD_REQUEST),
                "{raw}"
            );
        }
    }

    #[test]
    fn admin_projection_serializes_only_the_redacted_exact_contract() {
        let item = project_admin_notification(valid_admin_row()).unwrap();
        let response = AdminNotificationListResponse {
            items: vec![item],
            total: 1,
            limit: 20,
            offset: 0,
        };
        assert_eq!(
            serde_json::to_value(&response).unwrap(),
            serde_json::json!({
                "items": [{
                    "id": "notification-001",
                    "title": "Portfolio alert",
                    "subject": "EPS changed",
                    "channel": "in_app",
                    "status": "sent",
                    "notification_type": "portfolio-alert",
                    "priority": "high",
                    "sent_at": "2026-07-22T03:04:05Z",
                    "created_at": "2026-07-22T03:00:00Z"
                }],
                "total": 1,
                "limit": 20,
                "offset": 0
            })
        );

        let encoded = serde_json::to_string(&response).unwrap();
        for sensitive in [
            "user_id",
            "recipient",
            "template_id",
            "body",
            "data",
            "error",
            "read_at",
            "action_url",
        ] {
            assert!(
                !encoded.contains(sensitive),
                "unexpected field: {sensitive}"
            );
        }
    }

    #[test]
    fn admin_projection_fails_closed_on_stored_field_drift() {
        let mut cases: Vec<AdminNotificationRow> = Vec::new();

        let mut row = valid_admin_row();
        row.id.clear();
        cases.push(row);
        let mut row = valid_admin_row();
        row.id = "   ".into();
        cases.push(row);
        let mut row = valid_admin_row();
        row.id = "x".repeat(67);
        cases.push(row);
        let mut row = valid_admin_row();
        row.title = Some("unsafe\nvalue".into());
        cases.push(row);
        let mut row = valid_admin_row();
        row.subject = Some("s".repeat(256));
        cases.push(row);
        let mut row = valid_admin_row();
        row.channel = "email/html".into();
        cases.push(row);
        let mut row = valid_admin_row();
        row.channel = "Email".into();
        cases.push(row);
        let mut row = valid_admin_row();
        row.status = "unknown".into();
        cases.push(row);
        let mut row = valid_admin_row();
        row.notification_type = Some("unsafe\ntype".into());
        cases.push(row);
        let mut row = valid_admin_row();
        row.priority = Some("medium".into());
        cases.push(row);

        for row in cases {
            assert!(project_admin_notification(row).is_none());
        }
    }

    #[test]
    fn admin_cardinality_accepts_only_snapshot_consistent_pages() {
        for (total, limit, offset, item_count) in [
            (0, 20, 0, 0),
            (1, 20, 0, 1),
            (21, 20, 20, 1),
            (21, 20, 40, 0),
        ] {
            assert!(admin_notification_cardinality_is_valid(
                total, limit, offset, item_count
            ));
        }

        for (total, limit, offset, item_count) in [
            (-1, 20, 0, 0),
            (1, 0, 0, 0),
            (1, 20, -1, 0),
            (1, 20, 0, 0),
            (0, 20, 0, 1),
            (20, 20, 20, 1),
            (20, 20, 0, 21),
        ] {
            assert!(!admin_notification_cardinality_is_valid(
                total, limit, offset, item_count
            ));
        }
    }

    #[test]
    fn admin_contract_denies_unknown_fields_and_sql_order_is_stable() {
        let unknown = serde_json::json!({
            "items": [],
            "total": 0,
            "limit": 20,
            "offset": 0,
            "recipient": "should-never-appear"
        });
        assert!(serde_json::from_value::<AdminNotificationListResponse>(unknown).is_err());
        assert!(ADMIN_NOTIFICATION_LIST_SQL
            .contains("ORDER BY created_at DESC, id DESC LIMIT $1 OFFSET $2"));
        assert!(ADMIN_NOTIFICATION_LIST_SQL.contains("status = $3"));
        assert!(ADMIN_NOTIFICATION_LIST_SQL.contains("notification_type = $4"));
        assert!(ADMIN_NOTIFICATION_LIST_SQL.contains("priority = $5"));
        assert!(ADMIN_NOTIFICATION_LIST_SQL.contains("lower(user_id) = $6"));
        assert_eq!(
            ADMIN_NOTIFICATION_COUNT_SQL,
            "SELECT COUNT(*) FROM public.notifications n WHERE ($1::text IS NULL OR $1 = 'all' OR ($1 = 'read' AND (n.read_at IS NOT NULL OR EXISTS (SELECT 1 FROM public.notification_engagement filter_engagement WHERE filter_engagement.notification_id = n.id AND filter_engagement.read_at IS NOT NULL))) OR ($1 = 'unread' AND n.read_at IS NULL AND NOT EXISTS (SELECT 1 FROM public.notification_engagement filter_engagement WHERE filter_engagement.notification_id = n.id AND filter_engagement.read_at IS NOT NULL)) OR ($1 NOT IN ('all', 'read', 'unread') AND status = $1)) AND ($2::text IS NULL OR notification_type = $2) AND ($3::text IS NULL OR priority = $3) AND ($4::text IS NULL OR lower(user_id) = $4 OR (user_id IS NULL AND lower(recipient) = $4))"
        );
        for sensitive in [
            "user_id",
            "recipient",
            "template_id",
            "body",
            "data",
            "error",
            "read_at",
            "action_url",
        ] {
            assert!(
                !ADMIN_NOTIFICATION_LIST_SQL
                    .split(" FROM ")
                    .next()
                    .unwrap()
                    .split(',')
                    .any(|field| field.trim() == sensitive),
                "sensitive select field: {sensitive}"
            );
        }
    }

    #[test]
    fn realtime_wakeup_channels_are_owner_scoped_and_broadcast_explicit() {
        assert_eq!(
            realtime_wallet_channel("0xabc"),
            "notifications:wallet:0xabc"
        );
        assert_ne!(
            realtime_wallet_channel("0xabc"),
            realtime_wallet_channel("0xdef")
        );
        let wake = serde_json::json!({ "id": "notification-001" });
        assert_eq!(wake.as_object().unwrap().len(), 1);
        assert!(wake["id"].is_string());
    }

    #[tokio::test]
    async fn redis_wakeup_failure_preserves_local_replay_wakeup() {
        let notify = Arc::new(Notify::new());
        let notified = notify.notified();
        let unavailable = redis::Client::open("redis://127.0.0.1:1").expect("valid Redis URL");
        tokio::time::timeout(
            std::time::Duration::from_secs(2),
            publish_realtime_wakeup_parts(
                Some(&unavailable),
                notify.as_ref(),
                "notifications:wallet:0xabc",
                "notification-001",
            ),
        )
        .await
        .expect("Redis wake-up fallback must be bounded");
        tokio::time::timeout(std::time::Duration::from_millis(100), notified)
            .await
            .expect("local PostgreSQL-backed stream wake-up must survive Redis loss");
    }

    #[tokio::test]
    #[ignore = "requires a disposable local Redis instance"]
    async fn redis_multi_instance_fanout_and_loss_fallback_are_bounded() {
        let redis_url = std::env::var("NOTIFICATION_RUNTIME_REDIS_URL")
            .unwrap_or_else(|_| "redis://:epsx@127.0.0.1:6379".to_string());
        let publisher = redis::Client::open(redis_url.clone()).expect("valid Redis URL");
        let listener_a = redis::Client::open(redis_url.clone()).expect("valid Redis URL");
        let listener_b = redis::Client::open(redis_url).expect("valid Redis URL");
        let notify_a = Arc::new(Notify::new());
        let notify_b = Arc::new(Notify::new());
        let task_a = tokio::spawn(run_redis_listener(listener_a, Arc::clone(&notify_a)));
        let task_b = tokio::spawn(run_redis_listener(listener_b, Arc::clone(&notify_b)));

        // Give both pubsub connections time to establish their wallet and
        // broadcast subscriptions. Redis is only a wake-up hint; no payload
        // is trusted from the channel.
        tokio::time::sleep(std::time::Duration::from_millis(250)).await;
        let wake_a = notify_a.notified();
        let wake_b = notify_b.notified();
        publish_realtime_wakeup_parts(
            Some(&publisher),
            notify_a.as_ref(),
            "notifications:wallet:0xabc",
            "multi-instance-notification",
        )
        .await;
        tokio::time::timeout(std::time::Duration::from_secs(2), wake_a)
            .await
            .expect("publishing instance must wake its local stream");
        tokio::time::timeout(std::time::Duration::from_secs(2), wake_b)
            .await
            .expect("second instance must receive the Redis wake-up");

        // A failed Redis connection cannot suppress the local notify path;
        // the stream will still poll PostgreSQL and preserve replay semantics.
        let fallback_notify = Arc::new(Notify::new());
        let fallback_wait = fallback_notify.notified();
        let unavailable = redis::Client::open("redis://127.0.0.1:1").expect("valid Redis URL");
        tokio::time::timeout(
            std::time::Duration::from_secs(2),
            publish_realtime_wakeup_parts(
                Some(&unavailable),
                fallback_notify.as_ref(),
                "notifications:wallet:0xabc",
                "redis-loss-notification",
            ),
        )
        .await
        .expect("Redis loss must remain bounded");
        tokio::time::timeout(std::time::Duration::from_millis(100), fallback_wait)
            .await
            .expect("local wake-up must survive Redis loss");

        task_a.abort();
        task_b.abort();
    }

    #[tokio::test]
    #[ignore = "requires the redis-server binary and a disposable local port"]
    async fn redis_broker_restart_recovers_multi_instance_listeners() {
        use std::process::{Child, Command, Stdio};

        struct RedisChild(Option<Child>);

        impl RedisChild {
            fn start(port: u16) -> Self {
                let child = Command::new("redis-server")
                    .args([
                        "--bind",
                        "127.0.0.1",
                        "--port",
                        &port.to_string(),
                        "--save",
                        "",
                        "--appendonly",
                        "no",
                        "--protected-mode",
                        "no",
                    ])
                    .stdout(Stdio::null())
                    .stderr(Stdio::null())
                    .spawn()
                    .expect("start disposable redis-server");
                Self(Some(child))
            }

            fn stop(&mut self) {
                if let Some(mut child) = self.0.take() {
                    let _ = child.kill();
                    let _ = child.wait();
                }
            }
        }

        impl Drop for RedisChild {
            fn drop(&mut self) {
                self.stop();
            }
        }

        let probe =
            std::net::TcpListener::bind("127.0.0.1:0").expect("reserve an ephemeral Redis port");
        let port = probe.local_addr().expect("Redis port").port();
        drop(probe);
        let redis_url = format!("redis://127.0.0.1:{port}");
        let mut redis_process = RedisChild::start(port);
        let publisher = redis::Client::open(redis_url.clone()).expect("valid Redis URL");

        let mut redis_ready = false;
        for _ in 0..40 {
            let mut connection = publisher.get_multiplexed_async_connection().await.ok();
            if let Some(connection) = connection.as_mut() {
                if redis::cmd("PING")
                    .query_async::<String>(connection)
                    .await
                    .is_ok()
                {
                    redis_ready = true;
                    break;
                }
            }
            tokio::time::sleep(std::time::Duration::from_millis(50)).await;
        }
        assert!(redis_ready, "disposable Redis did not become ready");

        let listener_a = redis::Client::open(redis_url.clone()).expect("valid Redis URL");
        let listener_b = redis::Client::open(redis_url.clone()).expect("valid Redis URL");
        let notify_a = Arc::new(Notify::new());
        let notify_b = Arc::new(Notify::new());
        let task_a = tokio::spawn(run_redis_listener(listener_a, Arc::clone(&notify_a)));
        let task_b = tokio::spawn(run_redis_listener(listener_b, Arc::clone(&notify_b)));
        tokio::time::sleep(std::time::Duration::from_millis(400)).await;

        let first_a = notify_a.notified();
        let first_b = notify_b.notified();
        publish_realtime_wakeup_parts(
            Some(&publisher),
            notify_a.as_ref(),
            "notifications:wallet:0xabc",
            "before-restart",
        )
        .await;
        tokio::time::timeout(std::time::Duration::from_secs(2), first_a)
            .await
            .expect("first instance must receive the pre-restart wake-up");
        tokio::time::timeout(std::time::Duration::from_secs(2), first_b)
            .await
            .expect("second instance must receive the pre-restart wake-up");

        redis_process.stop();
        tokio::time::sleep(std::time::Duration::from_millis(300)).await;
        let local_during_loss = notify_a.notified();
        tokio::time::timeout(
            std::time::Duration::from_secs(2),
            publish_realtime_wakeup_parts(
                Some(&publisher),
                notify_a.as_ref(),
                "notifications:wallet:0xabc",
                "during-restart",
            ),
        )
        .await
        .expect("Redis outage must remain bounded");
        tokio::time::timeout(std::time::Duration::from_millis(100), local_during_loss)
            .await
            .expect("local replay wake-up must survive the broker outage");

        redis_process = RedisChild::start(port);
        let mut restored = false;
        for _ in 0..40 {
            let mut connection = publisher.get_multiplexed_async_connection().await.ok();
            if let Some(connection) = connection.as_mut() {
                if redis::cmd("PING")
                    .query_async::<String>(connection)
                    .await
                    .is_ok()
                {
                    restored = true;
                    break;
                }
            }
            tokio::time::sleep(std::time::Duration::from_millis(50)).await;
        }
        assert!(restored, "disposable Redis did not recover");
        // The listener intentionally uses a bounded five-second reconnect
        // interval. Wait beyond it before publishing the recovery marker.
        tokio::time::sleep(REDIS_RECONNECT_INTERVAL + std::time::Duration::from_millis(250)).await;
        let recovered_b = notify_b.notified();
        publish_realtime_wakeup_parts(
            Some(&publisher),
            notify_a.as_ref(),
            "notifications:wallet:0xabc",
            "after-restart",
        )
        .await;
        tokio::time::timeout(std::time::Duration::from_secs(3), recovered_b)
            .await
            .expect("second instance must receive a wake-up after broker recovery");

        task_a.abort();
        task_b.abort();
        redis_process.stop();
    }

    #[test]
    fn broadcast_materialization_is_one_durable_all_recipient_row() {
        assert!(BROADCAST_NOTIFICATION_INSERT_SQL.contains("user_id, channel, recipient"));
        assert!(BROADCAST_NOTIFICATION_INSERT_SQL.contains("NULL, 'in_app', 'all'"));
        assert!(BROADCAST_NOTIFICATION_INSERT_SQL.contains("VALUES ($1, NULL"));
        assert!(!BROADCAST_NOTIFICATION_INSERT_SQL.contains("notification_channel_jobs"));
    }
}
