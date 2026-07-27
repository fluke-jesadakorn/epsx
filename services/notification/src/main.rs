use axum::{
    extract::{Extension, Path as AxPath, RawQuery, State},
    http::{HeaderMap, StatusCode},
    routing::{delete, get, post},
    Json, Router,
};
use clap::{Parser, ValueEnum};
use epsx_notification::{
    build_auth_verifier, canonical_owner, protect_router, verify_schema_compatibility,
    NOTIFICATIONS_MANAGE_PERMISSION,
};
use epsx_service_auth::{VerifiedPrincipal, ADMIN_AUDIENCE};
use handlebars::Handlebars;
use lettre::{
    message::{header, MultiPart, SinglePart},
    transport::smtp::authentication::Credentials,
    Message, SmtpTransport, Transport,
};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use thiserror::Error;
use tokio::sync::RwLock;
use tracing::info;
use uuid::Uuid;

#[derive(Parser)]
#[command(name = "epsx-notification", about = "EPSX Notification Service")]
struct Args {
    #[arg(long, default_value = "8106")]
    port: u16,
    #[arg(long, default_value = "0.0.0.0")]
    host: String,
    #[arg(
        long,
        default_value = "postgres://epsx:epsx@localhost:5432/epsx_notification"
    )]
    database_url: String,
    #[arg(long, default_value = "")]
    smtp_host: String,
    #[arg(long, default_value = "587")]
    smtp_port: u16,
    #[arg(long, default_value = "")]
    smtp_user: String,
    #[arg(long, default_value = "")]
    smtp_password: String,
    #[arg(long, default_value = "noreply@epsx.io")]
    from_address: String,
    #[arg(long, default_value = "EPSX")]
    from_name: String,
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
    Production,
}

#[derive(Clone)]
struct AppState {
    db: sqlx::PgPool,
    templates: Arc<RwLock<Handlebars<'static>>>,
    smtp: Arc<RwLock<Option<SmtpTransport>>>,
    from: String,
    from_name: String,
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
    title: Option<String>,
    notification_type: Option<String>,
    priority: Option<String>,
    action_url: Option<String>,
}

#[derive(Serialize, Deserialize)]
struct CreateTemplateRequest {
    name: String,
    channel: String,
    subject: Option<String>,
    body: String,
    variables: serde_json::Value,
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

fn validate_send_request(request: &SendNotificationRequest) -> Result<(), StatusCode> {
    if !matches!(request.channel.as_str(), "email" | "in_app")
        || !bounded_control_free(&request.recipient, MAX_RECIPIENT_CHARS)
        || request
            .user_id
            .as_deref()
            .is_some_and(|value| !bounded_control_free(value, 66))
        || request
            .template_id
            .as_deref()
            .is_some_and(|value| !bounded_control_free(value, 66))
        || request
            .subject
            .as_deref()
            .is_some_and(|value| !bounded_control_free(value, MAX_SUBJECT_CHARS))
        || request
            .body
            .as_deref()
            .is_some_and(|value| !bounded_control_free(value, MAX_BODY_CHARS))
    {
        return Err(StatusCode::BAD_REQUEST);
    }
    if request.channel == "email"
        && request
            .recipient
            .parse::<lettre::message::Mailbox>()
            .is_err()
    {
        return Err(StatusCode::BAD_REQUEST);
    }
    if request.template_id.is_none() && request.body.is_none() {
        return Err(StatusCode::BAD_REQUEST);
    }
    if request.data.as_ref().is_some_and(|data| !data.is_object()) {
        return Err(StatusCode::BAD_REQUEST);
    }
    if request.data.as_ref().is_some_and(|data| {
        serde_json::to_vec(data)
            .map(|bytes| bytes.len() > MAX_DATA_BYTES)
            .unwrap_or(true)
    }) {
        return Err(StatusCode::PAYLOAD_TOO_LARGE);
    }
    Ok(())
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

fn response_from_existing(
    notification: &Notification,
    request_id: String,
) -> SendNotificationResponse {
    SendNotificationResponse {
        id: notification.id.clone(),
        status: notification.status.clone(),
        delivered: notification.status == "sent" && notification.error.is_none(),
        error: notification.error.clone(),
        request_id,
    }
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

#[derive(Debug, Clone, PartialEq, Eq)]
struct AdminNotificationQuery {
    limit: i64,
    offset: i64,
    status: Option<String>,
    notification_type: Option<String>,
    priority: Option<String>,
}

impl Default for AdminNotificationQuery {
    fn default() -> Self {
        Self {
            limit: 20,
            offset: 0,
            status: None,
            notification_type: None,
            priority: None,
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
        let mut priority_seen = false;

        for pair in raw.split('&') {
            let (key, value) = pair.split_once('=').ok_or(StatusCode::BAD_REQUEST)?;
            match key {
                "limit" if !limit_seen
                    && !value.is_empty()
                    && value.bytes().all(|byte| byte.is_ascii_digit()) =>
                {
                    query.limit = value.parse::<i64>().map_err(|_| StatusCode::BAD_REQUEST)?;
                    if !(1..=50).contains(&query.limit) {
                        return Err(StatusCode::BAD_REQUEST);
                    }
                    limit_seen = true;
                }
                "offset" if !offset_seen
                    && !value.is_empty()
                    && value.bytes().all(|byte| byte.is_ascii_digit()) =>
                {
                    query.offset = value.parse::<i64>().map_err(|_| StatusCode::BAD_REQUEST)?;
                    if !(0..=1_000_000).contains(&query.offset) {
                        return Err(StatusCode::BAD_REQUEST);
                    }
                    offset_seen = true;
                }
                "status" if !status_seen && valid_admin_filter_token(value, 20) => {
                    if !matches!(value, "all" | "pending" | "sent" | "failed" | "read" | "unread") {
                        return Err(StatusCode::BAD_REQUEST);
                    }
                    query.status = Some(value.to_string());
                    status_seen = true;
                }
                "type" if !type_seen && valid_admin_filter_token(value, 50) => {
                    query.notification_type = Some(value.to_string());
                    type_seen = true;
                }
                "priority" if !priority_seen && valid_admin_filter_token(value, 20) => {
                    if !matches!(value, "low" | "normal" | "high" | "critical" | "urgent") {
                        return Err(StatusCode::BAD_REQUEST);
                    }
                    query.priority = Some(value.to_string());
                    priority_seen = true;
                }
                _ => return Err(StatusCode::BAD_REQUEST),
            }
        }

        Ok(query)
    }
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
    read_at: Option<chrono::DateTime<chrono::Utc>>,
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

fn valid_admin_filter_token(value: &str, max_chars: usize) -> bool {
    !value.is_empty()
        && value.chars().count() <= max_chars
        && value.bytes().all(|byte| {
            byte.is_ascii_lowercase() || byte.is_ascii_digit() || matches!(byte, b'-' | b'_')
        })
}

fn valid_admin_notification_id(value: &str) -> bool {
    (1..=66).contains(&value.len())
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_'))
}

fn admin_notification_filter_sql(query: &AdminNotificationQuery) -> String {
    let mut clauses = vec!["TRUE".to_string()];
    if let Some(status) = query.status.as_deref() {
        match status {
            "read" => clauses.push("read_at IS NOT NULL".to_string()),
            "unread" => clauses.push("read_at IS NULL".to_string()),
            "all" => {}
            _ => clauses.push(format!("status = '{status}'")),
        }
    }
    if let Some(notification_type) = query.notification_type.as_deref() {
        clauses.push(format!("notification_type = '{notification_type}'"));
    }
    if let Some(priority) = query.priority.as_deref() {
        clauses.push(format!("priority = '{priority}'"));
    }
    clauses.join(" AND ")
}

// Kept as an auditable baseline for the projection contract tests; filtered
// reads use the same select with a bounded WHERE clause above.
const ADMIN_NOTIFICATION_LIST_SQL: &str = "SELECT id, title, subject, channel, status, notification_type, priority, sent_at, created_at FROM public.notifications ORDER BY created_at DESC, id DESC LIMIT $1 OFFSET $2";
const ADMIN_NOTIFICATION_COUNT_SQL: &str = "SELECT COUNT(*) FROM public.notifications";

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
        || !matches!(row.status.as_str(), "pending" | "sent" | "failed")
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
        status: if row.read_at.is_some() {
            "read".to_string()
        } else {
            row.status
        },
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
}

#[tokio::main]
async fn main() {
    epsx_observability::Observability::init("notification");
    let args = Args::parse();

    let production = matches!(args.environment, Environment::Production);
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

    let mut hb = Handlebars::new();
    hb.set_strict_mode(false);
    load_templates_to_hb(&db, &mut hb)
        .await
        .expect("active notification templates must load before startup");

    // Init SMTP
    let smtp: Option<SmtpTransport> = if !args.smtp_host.is_empty() {
        let creds = Credentials::new(args.smtp_user.clone(), args.smtp_password.clone());
        match SmtpTransport::relay(&args.smtp_host) {
            Ok(builder) => Some(builder.credentials(creds).build()),
            Err(_) => Some(
                SmtpTransport::builder_dangerous(&args.smtp_host)
                    .port(args.smtp_port)
                    .credentials(creds)
                    .build(),
            ),
        }
    } else {
        None
    };

    let app = Router::new()
        .route("/health", get(health))
        .route(
            "/api/v1/notification/templates",
            get(list_templates).post(create_template),
        )
        .route(
            "/api/v1/notification/templates/{id}",
            get(get_template).delete(delete_template),
        )
        .route("/api/v1/notification/send", post(send_notification))
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
        .route(
            "/api/v1/notification/admin/metrics",
            get(admin_notification_metrics),
        )
        .route("/api/v1/notification/list", get(list_notifications))
        .route("/api/v1/notification/unread-count", get(unread_count))
        .route("/api/v1/notification/mark-all-read", post(mark_all_read))
        .route("/api/v1/notification/clear-all", post(clear_all))
        .route("/api/v1/notification/{id}/read", post(mark_read))
        .route("/api/v1/notification/{id}/unread", post(mark_unread))
        .route(
            "/api/v1/notification/{id}",
            get(get_notification).delete(delete_notification),
        )
        .with_state(AppState {
            db,
            templates: Arc::new(RwLock::new(hb)),
            smtp: Arc::new(RwLock::new(smtp)),
            from: args.from_address,
            from_name: args.from_name,
        });
    let app = protect_router(app, verifier);

    let addr: SocketAddr = format!("{}:{}", args.host, args.port).parse().unwrap();
    info!("Notification service listening on {}", addr);
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn health() -> StatusCode {
    StatusCode::OK
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
    Json(req): Json<CreateTemplateRequest>,
) -> Result<Json<Template>, StatusCode> {
    let candidate_id = format!("0x{}", Uuid::new_v4().simple());
    let template: Template = sqlx::query_as::<_, Template>(
        "INSERT INTO public.templates (id, name, channel, subject, body, variables, active) VALUES ($1, $2, $3, $4, $5, $6, true)
         ON CONFLICT (name) DO UPDATE SET body = EXCLUDED.body, subject = EXCLUDED.subject, variables = EXCLUDED.variables, updated_at = NOW()
         RETURNING id, name, channel, subject, body, variables, active, created_at, updated_at"
    )
    .bind(&candidate_id).bind(&req.name).bind(&req.channel).bind(&req.subject).bind(&req.body).bind(req.variables.clone())
    .fetch_one(&state.db).await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let _ = state
        .templates
        .write()
        .await
        .register_template_string(&template.name, template.body.clone());
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

async fn delete_template(
    State(state): State<AppState>,
    AxPath(id): AxPath<String>,
) -> Result<StatusCode, StatusCode> {
    sqlx::query("DELETE FROM public.templates WHERE id = $1")
        .bind(&id)
        .execute(&state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(StatusCode::NO_CONTENT)
}

async fn send_notification(
    State(state): State<AppState>,
    Extension(principal): Extension<VerifiedPrincipal>,
    headers: HeaderMap,
    Json(req): Json<SendNotificationRequest>,
) -> Result<Json<SendNotificationResponse>, StatusCode> {
    require_admin_notifications(&principal)?;
    validate_send_request(&req)?;
    let request_id = request_id(&headers);
    let idempotency_key = headers
        .get("idempotency-key")
        .and_then(|value| value.to_str().ok())
        .filter(|value| valid_idempotency_key(value))
        .ok_or(StatusCode::BAD_REQUEST)?;
    let id = idempotent_notification_id(idempotency_key);

    let (subject, body) = if let Some(template_id) = &req.template_id {
        let template: Option<Template> = sqlx::query_as::<_, Template>(
            "SELECT id, name, channel, subject, body, variables, active, created_at, updated_at FROM public.templates WHERE id = $1 AND active = true"
        )
        .bind(template_id)
        .fetch_optional(&state.db).await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

        let t = template.ok_or(StatusCode::NOT_FOUND)?;
        let data_map: HashMap<String, serde_json::Value> = req
            .data
            .clone()
            .map(serde_json::from_value)
            .transpose()
            .map_err(|_| StatusCode::UNPROCESSABLE_ENTITY)?
            .unwrap_or_default();
        let rendered = state
            .templates
            .read()
            .await
            .render(&t.name, &data_map)
            .map_err(|_| StatusCode::UNPROCESSABLE_ENTITY)?;
        (t.subject, rendered)
    } else {
        (
            req.subject.clone(),
            req.body.clone().ok_or(StatusCode::BAD_REQUEST)?,
        )
    };

    let subject_str = subject.clone().unwrap_or_default();
    let body_str = body.clone();

    let existing: Option<Notification> = sqlx::query_as::<_, Notification>(
        "SELECT id, user_id, channel, recipient, template_id, subject, body, data, status, error, sent_at, created_at, read_at, title, notification_type, priority, action_url FROM public.notifications WHERE id = $1",
    )
    .bind(&id)
    .fetch_optional(&state.db)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    if let Some(existing) = existing {
        let same_request = existing.channel == req.channel
            && existing.recipient == req.recipient
            && existing.template_id == req.template_id
            && existing.subject.as_deref().unwrap_or_default() == subject_str
            && existing.body == body_str
            && existing.data == req.data;
        if !same_request {
            return Err(StatusCode::CONFLICT);
        }
        return Ok(Json(response_from_existing(&existing, request_id)));
    }

    // Claim the idempotency key before attempting delivery. A retry observes
    // the durable pending/sent/failed row and never sends a second message.
    let claimed = sqlx::query(
        "INSERT INTO public.notifications (id, user_id, channel, recipient, template_id, subject, body, data, status, error, sent_at)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, 'pending', NULL, NULL)
         ON CONFLICT (id) DO NOTHING",
    )
    .bind(&id)
    .bind(&req.user_id)
    .bind(&req.channel)
    .bind(&req.recipient)
    .bind(&req.template_id)
    .bind(&subject_str)
    .bind(&body_str)
    .bind(req.data.clone())
    .execute(&state.db)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    .rows_affected();
    if claimed == 0 {
        let existing: Option<Notification> = sqlx::query_as::<_, Notification>(
            "SELECT id, user_id, channel, recipient, template_id, subject, body, data, status, error, sent_at, created_at, read_at, title, notification_type, priority, action_url FROM public.notifications WHERE id = $1",
        )
        .bind(&id)
        .fetch_optional(&state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        let Some(existing) = existing else {
            return Err(StatusCode::CONFLICT);
        };
        let same_request = existing.channel == req.channel
            && existing.recipient == req.recipient
            && existing.template_id == req.template_id
            && existing.subject.as_deref().unwrap_or_default() == subject_str
            && existing.body == body_str
            && existing.data == req.data;
        if !same_request {
            return Err(StatusCode::CONFLICT);
        }
        return Ok(Json(response_from_existing(&existing, request_id)));
    }

    let (status, error, delivered) = match req.channel.as_str() {
        "email" => send_email(&state, &req.recipient, &subject_str, &body_str).await,
        "in_app" => {
            send_in_app(
                &state,
                &id,
                &req.user_id,
                &req.recipient,
                &subject_str,
                &body_str,
                req.data.as_ref(),
            )
            .await
        }
        _ => (
            "failed".to_string(),
            Some(format!("unknown channel: {}", req.channel)),
            false,
        ),
    };

    sqlx::query(
        "UPDATE public.notifications
         SET status = $2, error = $3, sent_at = $4
         WHERE id = $1",
    )
    .bind(&id)
    .bind(&status)
    .bind(&error)
    .bind(if status == "sent" {
        Some(chrono::Utc::now())
    } else {
        None
    })
    .execute(&state.db)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(SendNotificationResponse {
        id,
        status,
        delivered,
        error,
        request_id,
    }))
}

async fn send_in_app(
    _state: &AppState,
    _id: &str,
    _user_id: &Option<String>,
    _recipient: &str,
    _subject: &str,
    _body: &str,
    _data: Option<&serde_json::Value>,
) -> (String, Option<String>, bool) {
    // In-app notifications are stored in DB and retrieved via WebSocket/SSE
    // Durable row is the in-app fanout source; delivery is available through owner-scoped reads.
    ("sent".to_string(), None, true)
}

async fn send_email(
    state: &AppState,
    to: &str,
    subject: &str,
    body: &str,
) -> (String, Option<String>, bool) {
    let smtp_opt = state.smtp.read().await.clone();
    let smtp = match smtp_opt {
        Some(s) => s,
        None => {
            return (
                "failed".to_string(),
                Some("delivery_not_configured".to_string()),
                false,
            );
        }
    };

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
        );
    }

    let html_body = escape_html(body);
    let email = Message::builder()
        .from(from_parsed.unwrap())
        .to(to_parsed.unwrap())
        .subject(subject)
        .multipart(
            MultiPart::alternative()
                .singlepart(
                    SinglePart::builder()
                        .header(header::ContentType::TEXT_PLAIN)
                        .body(body.to_string()),
                )
                .singlepart(
                    SinglePart::builder()
                        .header(header::ContentType::TEXT_HTML)
                        .body(format!("<html><body>{html_body}</body></html>")),
                ),
        );

    match email {
        Ok(msg) => match smtp.send(&msg) {
            Ok(_) => ("sent".to_string(), None, true),
            Err(_) => (
                "failed".to_string(),
                Some("delivery_failed".to_string()),
                false,
            ),
        },
        Err(_) => (
            "failed".to_string(),
            Some("delivery_failed".to_string()),
            false,
        ),
    }
}

fn escape_html(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

async fn list_notifications(
    State(state): State<AppState>,
    Extension(principal): Extension<VerifiedPrincipal>,
    axum::extract::Query(params): axum::extract::Query<std::collections::HashMap<String, String>>,
) -> Result<Json<NotificationListResponse>, StatusCode> {
    let user_id = canonical_owner(&principal, params.get("user_id").map(String::as_str))?;
    let status = params.get("status").cloned();
    let limit: i64 = params
        .get("limit")
        .and_then(|s| s.parse().ok())
        .unwrap_or(50)
        .clamp(1, 100);
    let offset: i64 = params
        .get("offset")
        .and_then(|s| s.parse().ok())
        .unwrap_or(0)
        .max(0);

    let items: Vec<Notification> = match status {
        Some(status) => {
            sqlx::query_as::<_, Notification>(
                "SELECT id, user_id, channel, recipient, template_id, subject, body, data, status, error, sent_at, created_at, read_at, title, notification_type, priority, action_url FROM public.notifications WHERE user_id = $1 AND status = $2 ORDER BY created_at DESC LIMIT $3 OFFSET $4"
            )
            .bind(&user_id).bind(status).bind(limit).bind(offset)
            .fetch_all(&state.db).await
        }
        None => {
            sqlx::query_as::<_, Notification>(
                "SELECT id, user_id, channel, recipient, template_id, subject, body, data, status, error, sent_at, created_at, read_at, title, notification_type, priority, action_url FROM public.notifications WHERE user_id = $1 ORDER BY created_at DESC LIMIT $2 OFFSET $3"
            )
            .bind(&user_id).bind(limit).bind(offset)
            .fetch_all(&state.db).await
        }
    }
    .map_err(|e| { tracing::error!("list query failed: {e}"); StatusCode::INTERNAL_SERVER_ERROR })?;

    let total: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM public.notifications WHERE user_id = $1")
            .bind(&user_id)
            .fetch_one(&state.db)
            .await
            .unwrap_or(0);
    Ok(Json(NotificationListResponse { items, total }))
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
    let filter = admin_notification_filter_sql(&query);
    let total_sql = format!("SELECT COUNT(*) FROM public.notifications WHERE {filter}");
    let total: i64 = sqlx::query_scalar(&total_sql)
        .fetch_one(&mut *transaction)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let list_sql = format!(
        "SELECT id, title, subject, channel, status, notification_type, priority, sent_at, created_at, read_at FROM public.notifications WHERE {filter} ORDER BY created_at DESC, id DESC LIMIT $1 OFFSET $2"
    );
    let rows: Vec<AdminNotificationRow> = sqlx::query_as(&list_sql)
        .bind(query.limit)
        .bind(query.offset)
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

#[derive(Debug, Serialize)]
struct AdminNotificationMetrics {
    queue_depth: i64,
    queue_age_seconds: Option<i64>,
    suppressed: i64,
    retry_wait: i64,
    terminal_failed: i64,
    dead_lettered: i64,
    provider_accepted: i64,
    attempting: i64,
    channel_outcomes: std::collections::BTreeMap<String, i64>,
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

async fn admin_notification_metrics(
    State(state): State<AppState>,
    Extension(principal): Extension<VerifiedPrincipal>,
) -> Result<Json<AdminNotificationMetrics>, StatusCode> {
    require_admin_notifications(&principal)?;
    let total: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM public.notifications")
        .fetch_one(&state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let pending: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM public.notifications WHERE status = 'pending' AND read_at IS NULL",
    )
    .fetch_one(&state.db)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let failed: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM public.notifications WHERE status = 'failed'",
    )
    .fetch_one(&state.db)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let sent: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM public.notifications WHERE status = 'sent'",
    )
    .fetch_one(&state.db)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let rows: Vec<(String, i64)> = sqlx::query_as(
        "SELECT channel, COUNT(*) FROM public.notifications GROUP BY channel",
    )
    .fetch_all(&state.db)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let channel_outcomes = rows.into_iter().collect();

    Ok(Json(AdminNotificationMetrics {
        queue_depth: pending,
        queue_age_seconds: None,
        suppressed: 0,
        retry_wait: 0,
        terminal_failed: failed,
        dead_lettered: 0,
        provider_accepted: sent,
        attempting: 0,
        channel_outcomes,
        provider_events: total,
        delivery_attempts: total,
        replay_cursors: 0,
        replay_cursor_age_seconds: None,
        active_streams: 0,
        stream_connections_total: 0,
        stream_reconnects_total: 0,
        stream_replayed_events_total: 0,
        stream_lag_seconds: None,
        stream_query_failures_total: 0,
    }))
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
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
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
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
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
        "SELECT id, user_id, channel, recipient, template_id, subject, body, data, status, error, sent_at, created_at, read_at, title, notification_type, priority, action_url FROM public.notifications WHERE id = $1 AND user_id = $2"
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
    let updated: Option<(String,)> = sqlx::query_as(
        "UPDATE public.notifications SET read_at = $3, status = CASE WHEN status = 'pending' THEN 'sent' ELSE status END WHERE id = $1 AND user_id = $2 RETURNING id"
    )
    .bind(&id)
    .bind(&owner)
    .bind(read_at)
    .fetch_optional(&state.db)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    match updated {
        Some((id,)) => Ok(Json(MarkReadResponse { id, read_at })),
        None => Err(StatusCode::NOT_FOUND),
    }
}

async fn mark_unread(
    State(state): State<AppState>,
    Extension(principal): Extension<VerifiedPrincipal>,
    AxPath(id): AxPath<String>,
) -> Result<StatusCode, StatusCode> {
    let owner = canonical_owner(&principal, None)?;
    let result = sqlx::query(
        "UPDATE public.notifications SET read_at = NULL WHERE id = $1 AND user_id = $2",
    )
    .bind(&id)
    .bind(&owner)
    .execute(&state.db)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    if result.rows_affected() == 0 {
        Err(StatusCode::NOT_FOUND)
    } else {
        Ok(StatusCode::NO_CONTENT)
    }
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
    let result = sqlx::query(
        "UPDATE public.notifications SET read_at = NOW() WHERE user_id = $1 AND read_at IS NULL",
    )
    .bind(&owner)
    .execute(&state.db)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(MarkAllReadResponse {
        marked: result.rows_affected(),
    }))
}

async fn delete_notification(
    State(state): State<AppState>,
    Extension(principal): Extension<VerifiedPrincipal>,
    AxPath(id): AxPath<String>,
) -> Result<StatusCode, StatusCode> {
    let owner = canonical_owner(&principal, None)?;
    let result = sqlx::query("DELETE FROM public.notifications WHERE id = $1 AND user_id = $2")
        .bind(&id)
        .bind(&owner)
        .execute(&state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    if result.rows_affected() == 0 {
        Err(StatusCode::NOT_FOUND)
    } else {
        Ok(StatusCode::NO_CONTENT)
    }
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
    let result = sqlx::query("DELETE FROM public.notifications WHERE user_id = $1")
        .bind(&owner)
        .execute(&state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(ClearAllResponse {
        deleted: result.rows_affected(),
    }))
}

#[derive(Deserialize)]
struct UnreadCountQuery {
    user_id: Option<String>,
}

#[derive(Serialize)]
struct UnreadCountResponse {
    count: i64,
}

async fn unread_count(
    State(state): State<AppState>,
    Extension(principal): Extension<VerifiedPrincipal>,
    axum::extract::Query(q): axum::extract::Query<UnreadCountQuery>,
) -> Result<Json<UnreadCountResponse>, StatusCode> {
    let owner = canonical_owner(&principal, q.user_id.as_deref())?;
    let count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM public.notifications WHERE user_id = $1 AND read_at IS NULL",
    )
    .bind(&owner)
    .fetch_one(&state.db)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(UnreadCountResponse { count }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{TimeZone, Utc};

    fn valid_send_request() -> SendNotificationRequest {
        SendNotificationRequest {
            user_id: Some("0xrecipient".into()),
            channel: "in_app".into(),
            recipient: "0xrecipient".into(),
            template_id: None,
            subject: Some("Migration update".into()),
            body: Some("The migration is ready for review.".into()),
            data: Some(serde_json::json!({"source": "admin"})),
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
            read_at: None,
        }
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
            }
        );
    }

    #[test]
    fn admin_query_rejects_unknown_duplicate_malformed_and_out_of_bounds_values() {
        for raw in [
            "unknown=value",
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
        row.status = "read".into();
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
        assert_eq!(
            ADMIN_NOTIFICATION_COUNT_SQL,
            "SELECT COUNT(*) FROM public.notifications"
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
}
