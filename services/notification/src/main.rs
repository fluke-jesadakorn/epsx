use axum::{
    extract::{Path as AxPath, State},
    http::StatusCode,
    routing::{delete, get, post},
    Json, Router,
};
use clap::Parser;
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
    #[arg(long, default_value = "postgres://epsx:epsx@localhost:5432/epsx_notification")]
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

#[tokio::main]
async fn main() {
    epsx_observability::Observability::init("notification");
    let args = Args::parse();

    let db = sqlx::PgPool::connect(&args.database_url).await.expect("Failed to connect to database");

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS templates (
            id VARCHAR(66) PRIMARY KEY,
            name VARCHAR(100) UNIQUE NOT NULL,
            channel VARCHAR(20) NOT NULL,
            subject TEXT,
            body TEXT NOT NULL,
            variables JSONB DEFAULT '{}',
            active BOOLEAN DEFAULT true,
            created_at TIMESTAMPTZ DEFAULT NOW(),
            updated_at TIMESTAMPTZ DEFAULT NOW()
        )"
    ).execute(&db).await.expect("Failed to create templates table");

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS notifications (
            id VARCHAR(66) PRIMARY KEY,
            user_id VARCHAR(66),
            channel VARCHAR(20) NOT NULL,
            recipient VARCHAR(255) NOT NULL,
            template_id VARCHAR(66),
            subject TEXT,
            body TEXT NOT NULL,
            data JSONB,
            status VARCHAR(20) DEFAULT 'pending',
            error TEXT,
            sent_at TIMESTAMPTZ,
            created_at TIMESTAMPTZ DEFAULT NOW()
        )"
    ).execute(&db).await.expect("Failed to create notifications table");

    sqlx::query("CREATE INDEX IF NOT EXISTS idx_notif_user ON notifications (user_id, created_at DESC)").execute(&db).await.expect("Failed to create index");
    sqlx::query("CREATE INDEX IF NOT EXISTS idx_notif_status ON notifications (status)").execute(&db).await.expect("Failed to create index");

    // Seed default templates
    seed_default_templates(&db).await;
    seed_sample_notifications(&db).await;

    let mut hb = Handlebars::new();
    hb.set_strict_mode(false);
    load_templates_to_hb(&db, &mut hb).await;

    // Init SMTP
    let smtp: Option<SmtpTransport> = if !args.smtp_host.is_empty() {
        let creds = Credentials::new(args.smtp_user.clone(), args.smtp_password.clone());
        match SmtpTransport::relay(&args.smtp_host) {
            Ok(builder) => Some(builder.credentials(creds).build()),
            Err(_) => Some(SmtpTransport::builder_dangerous(&args.smtp_host).port(args.smtp_port).credentials(creds).build()),
        }
    } else {
        None
    };

    let app = Router::new()
        .route("/health", get(health))
        .route("/api/v1/notification/templates", get(list_templates).post(create_template))
        .route("/api/v1/notification/templates/{id}", get(get_template).delete(delete_template))
        .route("/api/v1/notification/send", post(send_notification))
        .route("/api/v1/notification/list", get(list_notifications))
        .route("/api/v1/notification/unread-count", get(unread_count))
        .route("/api/v1/notification/mark-all-read", post(mark_all_read))
        .route("/api/v1/notification/clear-all", post(clear_all))
        .route("/api/v1/notification/{id}/read", post(mark_read))
        .route("/api/v1/notification/{id}/unread", post(mark_unread))
        .route("/api/v1/notification/{id}", get(get_notification).delete(delete_notification))
        .with_state(AppState {
            db,
            templates: Arc::new(RwLock::new(hb)),
            smtp: Arc::new(RwLock::new(smtp)),
            from: args.from_address,
            from_name: args.from_name,
        });

    let addr: SocketAddr = format!("{}:{}", args.host, args.port).parse().unwrap();
    info!("Notification service listening on {}", addr);
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn health() -> StatusCode { StatusCode::OK }

async fn seed_default_templates(db: &sqlx::PgPool) {
    let defaults: Vec<(&str, &str, &str, &str, &str)> = vec![
        ("welcome", "email", "Welcome to EPSX", "Hi {{user_name}}, welcome to EPSX! Get started at {{app_url}}", "{}"),
        ("payment_received", "email", "Payment Received", "Your payment of {{amount}} {{token}} has been received.", "{}"),
        ("subscription_renewed", "email", "Subscription Renewed", "Your {{plan_name}} subscription has been renewed for {{period}}.", "{}"),
        ("escrow_released", "email", "Escrow Released", "Escrow {{escrow_id}} has been released. Amount: {{amount}} {{token}}", "{}"),
        ("kyc_approved", "email", "KYC Approved", "Hi {{user_name}}, your KYC verification has been approved.", "{}"),
        ("login_code", "email", "Your Login Code", "Your EPSX login code: {{code}} (valid for 5 minutes)", "{}"),
    ];

    for (name, channel, subject, body, vars) in defaults {
        let id = format!("0x{}", Uuid::new_v4().simple());
        sqlx::query(
            "INSERT INTO templates (id, name, channel, subject, body, variables, active) VALUES ($1, $2, $3, $4, $5, $6, true)
             ON CONFLICT (name) DO UPDATE SET body = EXCLUDED.body, subject = EXCLUDED.subject, updated_at = NOW()"
        )
        .bind(id).bind(name).bind(channel).bind(subject).bind(body).bind(vars)
        .execute(db).await.ok();
    }
}

async fn load_templates_to_hb(db: &sqlx::PgPool, hb: &mut Handlebars<'static>) {
    if let Ok(rows) = sqlx::query_as::<_, Template>("SELECT id, name, channel, subject, body, variables, active, created_at, updated_at FROM templates WHERE active = true")
        .fetch_all(db).await
    {
        for t in rows {
            hb.register_template_string(&t.name, t.body.clone());
        }
    }
}

async fn list_templates(State(state): State<AppState>) -> Result<Json<TemplateListResponse>, StatusCode> {
    let items: Vec<Template> = sqlx::query_as::<_, Template>(
        "SELECT id, name, channel, subject, body, variables, active, created_at, updated_at FROM templates ORDER BY name"
    )
    .fetch_all(&state.db)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let total: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM templates").fetch_one(&state.db).await.unwrap_or(0);
    Ok(Json(TemplateListResponse { items, total }))
}

async fn create_template(
    State(state): State<AppState>,
    Json(req): Json<CreateTemplateRequest>,
) -> Result<Json<Template>, StatusCode> {
    let id = format!("0x{}", Uuid::new_v4().simple());
    sqlx::query(
        "INSERT INTO templates (id, name, channel, subject, body, variables, active) VALUES ($1, $2, $3, $4, $5, $6, true)
         ON CONFLICT (name) DO UPDATE SET body = EXCLUDED.body, subject = EXCLUDED.subject, variables = EXCLUDED.variables, updated_at = NOW()"
    )
    .bind(&id).bind(&req.name).bind(&req.channel).bind(&req.subject).bind(&req.body).bind(req.variables.clone())
    .execute(&state.db).await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    state.templates.write().await.register_template_string(&req.name, req.body.clone());

    let template: Template = sqlx::query_as::<_, Template>(
        "SELECT id, name, channel, subject, body, variables, active, created_at, updated_at FROM templates WHERE id = $1"
    )
    .bind(&id)
    .fetch_one(&state.db).await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(template))
}

async fn get_template(State(state): State<AppState>, AxPath(id): AxPath<String>) -> Result<Json<Template>, StatusCode> {
    let template: Template = sqlx::query_as::<_, Template>(
        "SELECT id, name, channel, subject, body, variables, active, created_at, updated_at FROM templates WHERE id = $1"
    )
    .bind(&id)
    .fetch_optional(&state.db).await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    .ok_or(StatusCode::NOT_FOUND)?;
    Ok(Json(template))
}

async fn delete_template(State(state): State<AppState>, AxPath(id): AxPath<String>) -> Result<StatusCode, StatusCode> {
    sqlx::query("DELETE FROM templates WHERE id = $1")
        .bind(&id)
        .execute(&state.db).await
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
            "SELECT id, name, channel, subject, body, variables, active, created_at, updated_at FROM templates WHERE id = $1 AND active = true"
        )
        .bind(template_id)
        .fetch_optional(&state.db).await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

        if let Some(t) = template {
            let data_map: HashMap<String, serde_json::Value> = req.data.clone()
                .and_then(|d| serde_json::from_value(d).ok())
                .unwrap_or_default();
            let rendered = state.templates.read().await.render(&t.name, &data_map).unwrap_or_else(|_| t.body.clone());
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
        "in_app" => send_in_app(&state, &id, &req.user_id, &req.recipient, &subject_str, &body_str, req.data.as_ref()).await,
        _ => ("failed".to_string(), Some(format!("unknown channel: {}", req.channel)), false),
    };

    sqlx::query(
        "INSERT INTO notifications (id, user_id, channel, recipient, template_id, subject, body, data, status, error, sent_at)
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
    .bind(if status == "sent" { Some(chrono::Utc::now().naive_utc()) } else { None })
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

async fn send_email(state: &AppState, to: &str, subject: &str, body: &str) -> (String, Option<String>, bool) {
    let smtp_opt = state.smtp.read().await.clone();
    let smtp = match smtp_opt {
        Some(s) => s,
        None => {
            // No SMTP configured, log only
            tracing::info!("[EMAIL MOCK] To: {} Subject: {} Body: {}", to, subject, body);
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
        return ("failed".to_string(), Some("Invalid email address".to_string()), false);
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
                        .body(body.to_string())
                )
                .singlepart(
                    SinglePart::builder()
                        .header(header::ContentType::TEXT_HTML)
                        .body(format!("<html><body>{}</body></html>", body))
                )
        );

    match email {
        Ok(msg) => {
            match smtp.send(&msg) {
                Ok(_) => ("sent".to_string(), None, true),
                Err(e) => ("failed".to_string(), Some(e.to_string()), false),
            }
        }
        Err(e) => ("failed".to_string(), Some(e.to_string()), false),
    }
}

async fn list_notifications(
    State(state): State<AppState>,
    axum::extract::Query(params): axum::extract::Query<std::collections::HashMap<String, String>>,
) -> Result<Json<NotificationListResponse>, StatusCode> {
    let user_id = params.get("user_id").cloned();
    let status = params.get("status").cloned();
    let limit: i64 = params.get("limit").and_then(|s| s.parse().ok()).unwrap_or(50);
    let offset: i64 = params.get("offset").and_then(|s| s.parse().ok()).unwrap_or(0);

    let items: Vec<Notification> = match (user_id, status) {
        (Some(u), Some(s)) => {
            sqlx::query_as::<_, Notification>(
                "SELECT id, user_id, channel, recipient, template_id, subject, body, data, status, error, sent_at, created_at, read_at, title, notification_type, priority, action_url FROM notifications WHERE user_id = $1 AND status = $2 ORDER BY created_at DESC LIMIT $3 OFFSET $4"
            )
            .bind(u).bind(s).bind(limit).bind(offset)
            .fetch_all(&state.db).await
        }
        (Some(u), None) => {
            sqlx::query_as::<_, Notification>(
                "SELECT id, user_id, channel, recipient, template_id, subject, body, data, status, error, sent_at, created_at, read_at, title, notification_type, priority, action_url FROM notifications WHERE user_id = $1 ORDER BY created_at DESC LIMIT $2 OFFSET $3"
            )
            .bind(u).bind(limit).bind(offset)
            .fetch_all(&state.db).await
        }
        (None, Some(s)) => {
            sqlx::query_as::<_, Notification>(
                "SELECT id, user_id, channel, recipient, template_id, subject, body, data, status, error, sent_at, created_at, read_at, title, notification_type, priority, action_url FROM notifications WHERE status = $1 ORDER BY created_at DESC LIMIT $2 OFFSET $3"
            )
            .bind(s).bind(limit).bind(offset)
            .fetch_all(&state.db).await
        }
        (None, None) => {
            sqlx::query_as::<_, Notification>(
                "SELECT id, user_id, channel, recipient, template_id, subject, body, data, status, error, sent_at, created_at, read_at, title, notification_type, priority, action_url FROM notifications ORDER BY created_at DESC LIMIT $1 OFFSET $2"
            )
            .bind(limit).bind(offset)
            .fetch_all(&state.db).await
        }
    }
    .map_err(|e| { tracing::error!("list query failed: {e}"); StatusCode::INTERNAL_SERVER_ERROR })?;

    let total: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM notifications").fetch_one(&state.db).await.unwrap_or(0);
    Ok(Json(NotificationListResponse { items, total }))
}

async fn get_notification(State(state): State<AppState>, AxPath(id): AxPath<String>) -> Result<Json<Notification>, StatusCode> {
    let n: Notification = sqlx::query_as::<_, Notification>(
        "SELECT id, user_id, channel, recipient, template_id, subject, body, data, status, error, sent_at, created_at, read_at, title, notification_type, priority, action_url FROM notifications WHERE id = $1"
    )
    .bind(&id)
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
    AxPath(id): AxPath<String>,
) -> Result<Json<MarkReadResponse>, StatusCode> {
    let read_at = chrono::Utc::now();
    let updated: Option<(String,)> = sqlx::query_as(
        "UPDATE notifications SET read_at = $2, status = CASE WHEN status = 'pending' THEN 'sent' ELSE status END WHERE id = $1 RETURNING id"
    )
    .bind(&id)
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
    AxPath(id): AxPath<String>,
) -> Result<StatusCode, StatusCode> {
    let result = sqlx::query("UPDATE notifications SET read_at = NULL WHERE id = $1")
        .bind(&id)
        .execute(&state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    if result.rows_affected() == 0 { Err(StatusCode::NOT_FOUND) } else { Ok(StatusCode::NO_CONTENT) }
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
    axum::extract::Query(q): axum::extract::Query<MarkAllReadQuery>,
) -> Result<Json<MarkAllReadResponse>, StatusCode> {
    let result = match q.user_id {
        Some(uid) => {
            sqlx::query("UPDATE notifications SET read_at = NOW() WHERE user_id = $1 AND read_at IS NULL")
                .bind(uid)
                .execute(&state.db)
                .await
        }
        None => {
            sqlx::query("UPDATE notifications SET read_at = NOW() WHERE read_at IS NULL")
                .execute(&state.db)
                .await
        }
    }
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(MarkAllReadResponse { marked: result.rows_affected() }))
}

async fn delete_notification(
    State(state): State<AppState>,
    AxPath(id): AxPath<String>,
) -> Result<StatusCode, StatusCode> {
    let result = sqlx::query("DELETE FROM notifications WHERE id = $1")
        .bind(&id)
        .execute(&state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    if result.rows_affected() == 0 { Err(StatusCode::NOT_FOUND) } else { Ok(StatusCode::NO_CONTENT) }
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
    axum::extract::Query(q): axum::extract::Query<ClearAllQuery>,
) -> Result<Json<ClearAllResponse>, StatusCode> {
    let result = match q.user_id {
        Some(uid) => {
            sqlx::query("DELETE FROM notifications WHERE user_id = $1")
                .bind(uid)
                .execute(&state.db)
                .await
        }
        None => sqlx::query("DELETE FROM notifications").execute(&state.db).await,
    }
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(ClearAllResponse { deleted: result.rows_affected() }))
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
    axum::extract::Query(q): axum::extract::Query<UnreadCountQuery>,
) -> Result<Json<UnreadCountResponse>, StatusCode> {
    let count: i64 = match q.user_id {
        Some(uid) => {
            sqlx::query_scalar("SELECT COUNT(*) FROM notifications WHERE user_id = $1 AND read_at IS NULL")
                .bind(uid)
                .fetch_one(&state.db)
                .await
        }
        None => {
            sqlx::query_scalar("SELECT COUNT(*) FROM notifications WHERE read_at IS NULL")
                .fetch_one(&state.db)
                .await
        }
    }
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(UnreadCountResponse { count }))
}

async fn seed_sample_notifications(db: &sqlx::PgPool) {
    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM notifications")
        .fetch_one(db)
        .await
        .unwrap_or(0);
    if count > 0 {
        return;
    }
    let samples: Vec<(&str, &str, &str, &str, &str, &str)> = vec![
        ("demo", "device_signin",  "New device signed in",
         "A new device just signed in to your EPSX account from Chrome on macOS.",
         "security", "high"),
        ("demo", "subscription_renewed", "Subscription renewed",
         "Your Pro plan was renewed for $29 USDT. Next billing: June 30, 2026.",
         "payment", "normal"),
        ("demo", "watchlist_alert", "Watchlist alert: AAPL +5%",
         "Apple Inc. (AAPL) is up 5% in the last hour. View your watchlist for details.",
         "analytics", "normal"),
        ("demo", "rank_milestone", "You unlocked rank #1",
         "Your portfolio hit a new all-time high. 28 stocks in EPS growth >15%.",
         "analytics", "high"),
        ("demo", "api_quota", "API quota at 80%",
         "You've used 80% of your daily API quota. Upgrade to Pro for 10x capacity.",
         "system", "warning"),
    ];
    for (uid, ntype, title, body, ntype_v, prio) in samples {
        let id = Uuid::new_v4().to_string();
        let _ = sqlx::query(
            "INSERT INTO notifications (id, user_id, channel, recipient, title, body, notification_type, priority, status, data)
             VALUES ($1, $2, 'in_app', $2, $3, $4, $5, $6, 'sent', '{}'::jsonb)"
        )
        .bind(&id)
        .bind(uid)
        .bind(title)
        .bind(body)
        .bind(ntype_v)
        .bind(if prio == "warning" { "high" } else { prio })
        .execute(db)
        .await;
    }
    let _ = sqlx::query(
        "UPDATE notifications SET read_at = NOW() WHERE user_id = 'demo' AND title IN ('Subscription renewed', 'Watchlist alert: AAPL +5%', 'You unlocked rank #1', 'API quota at 80%')"
    )
    .execute(db)
    .await;
}
