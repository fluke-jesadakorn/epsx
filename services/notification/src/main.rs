use axum::{
    extract::{Extension, Path as AxPath, State},
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use clap::{Parser, ValueEnum};
use epsx_notification::{
    build_auth_verifier, canonical_owner, protect_router, verify_schema_compatibility,
};
use epsx_service_auth::VerifiedPrincipal;
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
}

#[derive(Serialize, Deserialize)]
struct NotificationListResponse {
    items: Vec<Notification>,
    total: i64,
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
    let id = format!("0x{}", Uuid::new_v4().simple());
    sqlx::query(
        "INSERT INTO public.templates (id, name, channel, subject, body, variables, active) VALUES ($1, $2, $3, $4, $5, $6, true)
         ON CONFLICT (name) DO UPDATE SET body = EXCLUDED.body, subject = EXCLUDED.subject, variables = EXCLUDED.variables, updated_at = NOW()"
    )
    .bind(&id).bind(&req.name).bind(&req.channel).bind(&req.subject).bind(&req.body).bind(req.variables.clone())
    .execute(&state.db).await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let _ = state
        .templates
        .write()
        .await
        .register_template_string(&req.name, req.body.clone());

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
    Json(req): Json<SendNotificationRequest>,
) -> Result<Json<SendNotificationResponse>, StatusCode> {
    let id = format!("0x{}", Uuid::new_v4().simple());
    let (subject, body) = if let Some(template_id) = &req.template_id {
        let template: Option<Template> = sqlx::query_as::<_, Template>(
            "SELECT id, name, channel, subject, body, variables, active, created_at, updated_at FROM public.templates WHERE id = $1 AND active = true"
        )
        .bind(template_id)
        .fetch_optional(&state.db).await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

        if let Some(t) = template {
            let data_map: HashMap<String, serde_json::Value> = req
                .data
                .clone()
                .and_then(|d| serde_json::from_value(d).ok())
                .unwrap_or_default();
            let rendered = state
                .templates
                .read()
                .await
                .render(&t.name, &data_map)
                .unwrap_or_else(|_| t.body.clone());
            (t.subject, rendered)
        } else {
            (req.subject.clone(), req.body.clone().unwrap_or_default())
        }
    } else {
        (req.subject.clone(), req.body.clone().unwrap_or_default())
    };

    let subject_str = subject.clone().unwrap_or_default();
    let body_str = body.clone();

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
        "INSERT INTO public.notifications (id, user_id, channel, recipient, template_id, subject, body, data, status, error, sent_at)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)"
    )
    .bind(&id)
    .bind(&req.user_id)
    .bind(&req.channel)
    .bind(&req.recipient)
    .bind(&req.template_id)
    .bind(&subject_str)
    .bind(&body_str)
    .bind(req.data.clone())
    .bind(&status)
    .bind(&error)
    .bind(if status == "sent" {
        Some(chrono::Utc::now())
    } else {
        None
    })
    .execute(&state.db).await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(SendNotificationResponse {
        id,
        status,
        delivered,
        error,
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
    // For now, just return sent status
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
            // No SMTP configured, log only
            tracing::info!(
                "[EMAIL MOCK] To: {} Subject: {} Body: {}",
                to,
                subject,
                body
            );
            return ("sent".to_string(), None, true);
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
                        .body(format!("<html><body>{}</body></html>", body)),
                ),
        );

    match email {
        Ok(msg) => match smtp.send(&msg) {
            Ok(_) => ("sent".to_string(), None, true),
            Err(e) => ("failed".to_string(), Some(e.to_string()), false),
        },
        Err(e) => ("failed".to_string(), Some(e.to_string()), false),
    }
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
