use async_trait::async_trait;
use axum::{
    extract::{Extension, Path, RawQuery, Request, State},
    http::{header, HeaderMap, HeaderName, Method, StatusCode},
    middleware::{self, Next},
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};
use epsx_service_auth::{
    authenticate_headers, AccessTokenVerifier, JwksVerifier, JwksVerifierConfig, VerifiedPrincipal,
    ADMIN_AUDIENCE, FRONTEND_AUDIENCE,
};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use std::{
    sync::Arc,
    time::{Duration, Instant},
};
use thiserror::Error;

const ANALYTICS_VIEW_PERMISSION: &str = "admin:analytics:view";
const AUDIT_READ_PERMISSION: &str = "admin:audit:read";
const DEFAULT_AUDIT_LIMIT: i64 = 20;
const MAX_AUDIT_CURSOR_CHARS: usize = 256;
const AUDIT_LIST_SQL: &str = "SELECT id::text AS id, category, action, resource_type, effect, \
     created_at AS occurred_at FROM infra_logs.unified_audit_log \
     WHERE ($1::text IS NULL OR category = $1) \
       AND ($2::timestamptz IS NULL OR (created_at, id) < ($2, $3)) \
     ORDER BY created_at DESC, id DESC LIMIT $4";

#[derive(Debug, Error)]
pub enum AnalyticsConfigError {
    #[error("HTTP client configuration failed")]
    Http(#[from] reqwest::Error),
    #[error("OIDC verifier configuration failed")]
    Auth(#[from] epsx_service_auth::VerifyError),
}

pub fn build_auth_verifier(
    issuer: &str,
    jwks_url: &str,
    production: bool,
) -> Result<Arc<JwksVerifier>, AnalyticsConfigError> {
    let client = reqwest::Client::builder()
        .connect_timeout(Duration::from_secs(5))
        .timeout(Duration::from_secs(15))
        .pool_idle_timeout(Duration::from_secs(60))
        .redirect(reqwest::redirect::Policy::none())
        .user_agent("epsx-analytics/1")
        .build()?;
    let config =
        JwksVerifierConfig::new(issuer, jwks_url, Duration::from_secs(5 * 60), production)?;
    Ok(Arc::new(JwksVerifier::new(config, client)))
}

#[derive(Debug, Clone, Deserialize)]
pub struct TrackEventRequest {
    pub user_id: Option<String>,
    pub event_name: String,
    pub properties: Option<serde_json::Value>,
    pub chain_id: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuditListQuery {
    pub limit: i64,
    pub category: Option<String>,
    cursor: Option<AuditCursor>,
}

impl Default for AuditListQuery {
    fn default() -> Self {
        Self {
            limit: DEFAULT_AUDIT_LIMIT,
            category: None,
            cursor: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct AuditCursor {
    occurred_at: chrono::DateTime<chrono::Utc>,
    id: uuid::Uuid,
    category: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, FromRow)]
#[serde(deny_unknown_fields)]
pub struct AuditSummary {
    pub id: String,
    pub category: String,
    pub action: String,
    pub resource_type: String,
    pub effect: String,
    pub occurred_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct AuditList {
    pub items: Vec<AuditSummary>,
    pub next_cursor: Option<String>,
    pub has_more: bool,
}

#[derive(Serialize, FromRow)]
struct Event {
    id: uuid::Uuid,
    user_id: Option<uuid::Uuid>,
    subject: Option<String>,
    event_name: String,
    properties_json: Option<serde_json::Value>,
    chain_id: Option<String>,
    created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Error)]
#[error("analytics store operation failed")]
pub struct AnalyticsStoreError;

#[async_trait]
pub trait AnalyticsStore: Send + Sync {
    async fn track_event(
        &self,
        request: &TrackEventRequest,
    ) -> Result<serde_json::Value, AnalyticsStoreError>;
    async fn list_events(&self) -> Result<serde_json::Value, AnalyticsStoreError>;
    async fn metric(&self, metric: &str) -> Result<serde_json::Value, AnalyticsStoreError>;
    async fn revenue(&self) -> Result<serde_json::Value, AnalyticsStoreError>;
    async fn list_audit(&self, query: &AuditListQuery) -> Result<AuditList, AnalyticsStoreError>;
}

pub struct SqlAnalyticsStore {
    db: sqlx::PgPool,
}

impl SqlAnalyticsStore {
    pub fn new(db: sqlx::PgPool) -> Self {
        Self { db }
    }
}

const ANALYTICS_SCHEMA_COMPATIBILITY_QUERY: &str = r#"
WITH expected_columns (
    column_name,
    data_type,
    udt_name,
    is_nullable,
    character_maximum_length,
    default_kind
) AS (
    VALUES
        ('id', 'uuid', 'uuid', 'NO', NULL::bigint, 'uuid'),
        ('user_id', 'uuid', 'uuid', 'YES', NULL::bigint, 'none'),
        ('event_name', 'character varying', 'varchar', 'NO', 100::bigint, 'none'),
        ('properties_json', 'jsonb', 'jsonb', 'YES', NULL::bigint, 'empty_json'),
        ('chain_id', 'character varying', 'varchar', 'YES', 10::bigint, 'none'),
        ('subject', 'character varying', 'varchar', 'YES', 128::bigint, 'none'),
        ('created_at', 'timestamp with time zone', 'timestamptz', 'YES', NULL::bigint, 'now')
),
column_compatibility AS (
    SELECT bool_and(
        actual.column_name IS NOT NULL
        AND actual.data_type = expected.data_type
        AND actual.udt_name = expected.udt_name
        AND actual.is_nullable = expected.is_nullable
        AND actual.character_maximum_length IS NOT DISTINCT FROM expected.character_maximum_length
        AND COALESCE(
            CASE expected.default_kind
                WHEN 'uuid' THEN actual.column_default = 'gen_random_uuid()'
                WHEN 'empty_json' THEN actual.column_default = '''{}''::jsonb'
                WHEN 'now' THEN actual.column_default IN ('now()', 'CURRENT_TIMESTAMP')
                ELSE actual.column_default IS NULL
            END,
            false
        )
    ) AS compatible
    FROM expected_columns AS expected
    LEFT JOIN information_schema.columns AS actual
      ON actual.table_schema = 'public'
     AND actual.table_name = 'events'
     AND actual.column_name = expected.column_name
),
primary_key_compatibility AS (
    SELECT EXISTS (
        SELECT 1
        FROM pg_catalog.pg_constraint AS constraint_record
        JOIN pg_catalog.pg_class AS table_record
          ON table_record.oid = constraint_record.conrelid
        JOIN pg_catalog.pg_namespace AS namespace_record
          ON namespace_record.oid = table_record.relnamespace
        JOIN pg_catalog.pg_attribute AS attribute_record
          ON attribute_record.attrelid = table_record.oid
         AND attribute_record.attnum = constraint_record.conkey[1]
        WHERE namespace_record.nspname = 'public'
          AND table_record.relname = 'events'
          AND constraint_record.contype = 'p'
          AND cardinality(constraint_record.conkey) = 1
          AND attribute_record.attname = 'id'
    ) AS compatible
)
SELECT
    to_regclass('public.events') IS NOT NULL
    AND (
        SELECT COUNT(*) = 7
        FROM information_schema.columns
        WHERE table_schema = 'public'
          AND table_name = 'events'
    )
    AND COALESCE((SELECT compatible FROM column_compatibility), false)
    AND (SELECT compatible FROM primary_key_compatibility)
"#;

const AUDIT_SCHEMA_COMPATIBILITY_QUERY: &str = r#"
WITH expected_columns (column_name, data_type, udt_name, is_nullable, character_maximum_length) AS (
    VALUES
        ('id', 'uuid', 'uuid', 'NO', NULL::bigint),
        ('actor', 'character varying', 'varchar', 'YES', 42::bigint),
        ('actor_type', 'character varying', 'varchar', 'NO', 20::bigint),
        ('created_at', 'timestamp with time zone', 'timestamptz', 'NO', NULL::bigint),
        ('resource_type', 'character varying', 'varchar', 'NO', 50::bigint),
        ('resource_id', 'character varying', 'varchar', 'YES', 255::bigint),
        ('action', 'character varying', 'varchar', 'NO', 50::bigint),
        ('effect', 'character varying', 'varchar', 'NO', 20::bigint),
        ('before_state', 'jsonb', 'jsonb', 'YES', NULL::bigint),
        ('after_state', 'jsonb', 'jsonb', 'YES', NULL::bigint),
        ('ip_address', 'character varying', 'varchar', 'YES', 45::bigint),
        ('user_agent', 'text', 'text', 'YES', NULL::bigint),
        ('metadata', 'jsonb', 'jsonb', 'YES', NULL::bigint),
        ('category', 'character varying', 'varchar', 'NO', 30::bigint)
),
column_compatibility AS (
    SELECT bool_and(
        actual.column_name IS NOT NULL
        AND actual.data_type = expected.data_type
        AND actual.udt_name = expected.udt_name
        AND actual.is_nullable = expected.is_nullable
        AND actual.character_maximum_length IS NOT DISTINCT FROM expected.character_maximum_length
    ) AS compatible
    FROM expected_columns AS expected
    LEFT JOIN information_schema.columns AS actual
      ON actual.table_schema = 'infra_logs'
     AND actual.table_name = 'unified_audit_log'
     AND actual.column_name = expected.column_name
),
primary_key_compatibility AS (
    SELECT EXISTS (
        SELECT 1
        FROM pg_catalog.pg_constraint AS constraint_record
        JOIN pg_catalog.pg_class AS table_record
          ON table_record.oid = constraint_record.conrelid
        JOIN pg_catalog.pg_namespace AS namespace_record
          ON namespace_record.oid = table_record.relnamespace
        JOIN pg_catalog.pg_attribute AS attribute_record
          ON attribute_record.attrelid = table_record.oid
         AND attribute_record.attnum = constraint_record.conkey[1]
        WHERE namespace_record.nspname = 'infra_logs'
          AND table_record.relname = 'unified_audit_log'
          AND constraint_record.contype = 'p'
          AND cardinality(constraint_record.conkey) = 1
          AND attribute_record.attname = 'id'
    ) AS compatible
)
SELECT
    to_regclass('infra_logs.unified_audit_log') IS NOT NULL
    AND (
        SELECT COUNT(*) = 14
        FROM information_schema.columns
        WHERE table_schema = 'infra_logs'
          AND table_name = 'unified_audit_log'
    )
    AND COALESCE((SELECT compatible FROM column_compatibility), false)
    AND (SELECT compatible FROM primary_key_compatibility)
"#;

#[derive(Debug, Error)]
pub enum AnalyticsSchemaError {
    #[error("analytics schema compatibility query failed")]
    Query(#[source] sqlx::Error),
    #[error(
        "analytics schema is incompatible; run the reviewed analytics migration before startup"
    )]
    Incompatible,
}

pub async fn verify_schema_compatibility(db: &sqlx::PgPool) -> Result<(), AnalyticsSchemaError> {
    let analytics_compatible = sqlx::query_scalar::<_, bool>(ANALYTICS_SCHEMA_COMPATIBILITY_QUERY)
        .fetch_one(db)
        .await
        .map_err(AnalyticsSchemaError::Query)?;
    let audit_compatible = sqlx::query_scalar::<_, bool>(AUDIT_SCHEMA_COMPATIBILITY_QUERY)
        .fetch_one(db)
        .await
        .map_err(AnalyticsSchemaError::Query)?;
    if !analytics_compatible || !audit_compatible {
        return Err(AnalyticsSchemaError::Incompatible);
    }
    Ok(())
}

#[async_trait]
impl AnalyticsStore for SqlAnalyticsStore {
    async fn track_event(
        &self,
        request: &TrackEventRequest,
    ) -> Result<serde_json::Value, AnalyticsStoreError> {
        let id = uuid::Uuid::new_v4();
        let user_uuid = request
            .user_id
            .as_deref()
            .and_then(|subject| uuid::Uuid::parse_str(subject).ok());
        let properties = request
            .properties
            .clone()
            .unwrap_or_else(|| serde_json::json!({}));
        sqlx::query(
            "INSERT INTO public.events (id, user_id, event_name, properties_json, chain_id, subject) \
             VALUES ($1, $2, $3, $4, $5, $6)",
        )
        .bind(id)
        .bind(user_uuid)
        .bind(&request.event_name)
        .bind(&properties)
        .bind(&request.chain_id)
        .bind(&request.user_id)
        .execute(&self.db)
        .await
        .map_err(|_| AnalyticsStoreError)?;

        Ok(serde_json::json!({
            "event_id": id.to_string(),
            "accepted": true
        }))
    }

    async fn list_events(&self) -> Result<serde_json::Value, AnalyticsStoreError> {
        let events: Vec<Event> = sqlx::query_as::<_, Event>(
            "SELECT * FROM public.events ORDER BY created_at DESC LIMIT 100",
        )
        .fetch_all(&self.db)
        .await
        .map_err(|_| AnalyticsStoreError)?;
        serde_json::to_value(events).map_err(|_| AnalyticsStoreError)
    }

    async fn metric(&self, metric: &str) -> Result<serde_json::Value, AnalyticsStoreError> {
        let count: (i64,) =
            sqlx::query_as("SELECT COUNT(*) FROM public.events WHERE event_name = $1")
                .bind(metric)
                .fetch_one(&self.db)
                .await
                .map_err(|_| AnalyticsStoreError)?;
        let last_24h: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM public.events WHERE event_name = $1 \
             AND created_at >= NOW() - INTERVAL '24 hours'",
        )
        .bind(metric)
        .fetch_one(&self.db)
        .await
        .map_err(|_| AnalyticsStoreError)?;
        let unique_users: (Option<i64>,) = sqlx::query_as(
            "SELECT COUNT(DISTINCT user_id) FROM public.events \
             WHERE event_name = $1 AND user_id IS NOT NULL",
        )
        .bind(metric)
        .fetch_one(&self.db)
        .await
        .map_err(|_| AnalyticsStoreError)?;

        Ok(serde_json::json!({
            "metric": metric,
            "total": count.0,
            "last_24h": last_24h.0,
            "unique_users": unique_users.0.unwrap_or(0)
        }))
    }

    async fn revenue(&self) -> Result<serde_json::Value, AnalyticsStoreError> {
        // Revenue and subscription counts are financial decisions owned by
        // payment/subscription authorities, not inferred from analytics
        // events. Keep the route fail-closed until that projection exists.
        Err(AnalyticsStoreError)
    }

    async fn list_audit(&self, query: &AuditListQuery) -> Result<AuditList, AnalyticsStoreError> {
        let mut transaction = self.db.begin().await.map_err(|_| AnalyticsStoreError)?;
        // This must be the first statement after BEGIN so the page is read
        // from one immutable snapshot.
        sqlx::query("SET TRANSACTION ISOLATION LEVEL REPEATABLE READ, READ ONLY")
            .execute(&mut *transaction)
            .await
            .map_err(|_| AnalyticsStoreError)?;

        // Deliberately omit actor_type, before/after state, network/device
        // fields, actor/target identity, and metadata from both the SELECT and
        // the public DTO. Fetch one extra row to derive a stable keyset cursor.
        let cursor_at = query.cursor.as_ref().map(|cursor| cursor.occurred_at);
        let cursor_id = query.cursor.as_ref().map(|cursor| cursor.id);
        let mut items = sqlx::query_as::<_, AuditSummary>(AUDIT_LIST_SQL)
            .bind(&query.category)
            .bind(cursor_at)
            .bind(cursor_id)
            .bind(query.limit + 1)
            .fetch_all(&mut *transaction)
            .await
            .map_err(|_| AnalyticsStoreError)?;

        transaction
            .commit()
            .await
            .map_err(|_| AnalyticsStoreError)?;

        let has_more = items.len() > query.limit as usize;
        if has_more {
            items.pop();
        }
        if items.iter().any(|item| !audit_summary_is_valid(item)) {
            return Err(AnalyticsStoreError);
        }
        let next_cursor = if has_more {
            let last = items.last().ok_or(AnalyticsStoreError)?;
            let id = uuid::Uuid::parse_str(&last.id).map_err(|_| AnalyticsStoreError)?;
            Some(encode_audit_cursor(&AuditCursor {
                occurred_at: last.occurred_at,
                id,
                category: query.category.clone(),
            })?)
        } else {
            None
        };

        Ok(AuditList {
            items,
            next_cursor,
            has_more,
        })
    }
}

#[derive(Clone)]
struct AppState {
    store: Arc<dyn AnalyticsStore>,
    verifier: Arc<dyn AccessTokenVerifier>,
}

pub fn build_router(
    store: Arc<dyn AnalyticsStore>,
    verifier: Arc<dyn AccessTokenVerifier>,
) -> Router {
    let state = AppState { store, verifier };
    let app = Router::new()
        .route("/health", get(health))
        .route("/api/v1/analytics/track", post(track_event))
        .route("/api/v1/analytics/events", get(list_events))
        .route("/api/v1/analytics/metrics/{metric}", get(get_metrics))
        .route("/api/v1/analytics/revenue", get(get_revenue))
        .route("/api/v1/analytics/admin/audit-log", get(list_audit))
        .route("/api/v1/analytics/metrics/prometheus", get(not_found))
        .route("/api/v1/analytics/prometheus/metrics", get(not_found))
        .fallback(not_found)
        .layer(middleware::from_fn(track_http_metrics))
        .layer(middleware::from_fn_with_state(
            state.clone(),
            authorize_request,
        ))
        .with_state(state);
    app
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AccessPolicy {
    Public,
    Authenticated,
    Permission(&'static str),
    InternalOnly,
    Blocked,
}

fn classify(method: &Method, path: &str) -> AccessPolicy {
    match (method, path) {
        (&Method::GET | &Method::HEAD, "/health") => AccessPolicy::Public,
        (&Method::POST, "/api/v1/analytics/track") => AccessPolicy::Authenticated,
        (&Method::GET, "/api/v1/analytics/metrics/prometheus")
        | (&Method::GET, "/api/v1/analytics/prometheus/metrics") => AccessPolicy::InternalOnly,
        (&Method::GET, "/api/v1/analytics/events")
        | (&Method::GET, "/api/v1/analytics/revenue") => {
            AccessPolicy::Permission(ANALYTICS_VIEW_PERMISSION)
        }
        (&Method::GET, "/api/v1/analytics/admin/audit-log") => {
            AccessPolicy::Permission(AUDIT_READ_PERMISSION)
        }
        (&Method::GET, path)
            if path
                .strip_prefix("/api/v1/analytics/metrics/")
                .is_some_and(|metric| {
                    !metric.is_empty()
                        && !metric.contains('/')
                        && !metric.contains('%')
                        && !matches!(metric, "." | "..")
                }) =>
        {
            AccessPolicy::Permission(ANALYTICS_VIEW_PERMISSION)
        }
        _ => AccessPolicy::Blocked,
    }
}

async fn authorize_request(
    State(state): State<AppState>,
    mut request: Request,
    next: Next,
) -> Response {
    strip_spoofable_identity_headers(request.headers_mut());
    let policy = classify(request.method(), request.uri().path());
    let principal = match policy {
        AccessPolicy::Public => {
            request.headers_mut().remove(header::AUTHORIZATION);
            None
        }
        AccessPolicy::Authenticated => {
            let principal =
                match authenticate_headers(state.verifier.as_ref(), request.headers()).await {
                    Ok(principal) => principal,
                    Err(_) => return auth_error(StatusCode::UNAUTHORIZED),
                };
            if principal.audience != FRONTEND_AUDIENCE && principal.audience != ADMIN_AUDIENCE {
                return auth_error(StatusCode::FORBIDDEN);
            }
            Some(principal)
        }
        AccessPolicy::Permission(permission) => {
            let principal =
                match authenticate_headers(state.verifier.as_ref(), request.headers()).await {
                    Ok(principal) => principal,
                    Err(_) => return auth_error(StatusCode::UNAUTHORIZED),
                };
            if principal.audience != ADMIN_AUDIENCE || !principal.has_permission(permission) {
                return auth_error(StatusCode::FORBIDDEN);
            }
            Some(principal)
        }
        AccessPolicy::InternalOnly | AccessPolicy::Blocked => {
            return StatusCode::NOT_FOUND.into_response();
        }
    };
    if let Some(principal) = principal {
        request.extensions_mut().insert(principal);
    }
    next.run(request).await
}

fn auth_error(status: StatusCode) -> Response {
    let code = if status == StatusCode::FORBIDDEN {
        "forbidden"
    } else {
        "unauthorized"
    };
    (status, Json(serde_json::json!({ "error": code }))).into_response()
}

fn strip_spoofable_identity_headers(headers: &mut HeaderMap) {
    let names: Vec<HeaderName> = headers
        .keys()
        .filter(|name| {
            let name = name.as_str();
            name.starts_with("x-user-")
                || name.starts_with("x-wallet-")
                || name.starts_with("x-auth-")
                || name.starts_with("x-epsx-")
                || matches!(
                    name,
                    "x-user"
                        | "x-subject"
                        | "x-principal"
                        | "x-wallet"
                        | "x-address"
                        | "x-chain-id"
                        | "x-client-id"
                        | "x-permissions"
                        | "x-role"
                        | "x-roles"
                        | "x-scope"
                        | "x-forwarded-user"
                )
        })
        .cloned()
        .collect();
    for name in names {
        headers.remove(name);
    }
}

async fn health() -> StatusCode {
    StatusCode::OK
}

async fn track_event(
    State(state): State<AppState>,
    Extension(principal): Extension<VerifiedPrincipal>,
    Json(mut request): Json<TrackEventRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    if !event_name_is_valid(&request.event_name) {
        return Err(StatusCode::UNPROCESSABLE_ENTITY);
    }
    if request
        .properties
        .as_ref()
        .is_some_and(|properties| properties.to_string().len() > 64 * 1024)
    {
        return Err(StatusCode::PAYLOAD_TOO_LARGE);
    }
    // The verified wallet is the only subject allowed to reach persistence;
    // caller-supplied identity fields are ignored.
    request.user_id = Some(principal.wallet_address);
    let response = state
        .store
        .track_event(&request)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let chain = request
        .chain_id
        .clone()
        .unwrap_or_else(|| "unknown".to_string());
    metrics::counter!(
        "epsx_events_tracked_total",
        "event" => request.event_name,
        "chain" => chain
    )
    .increment(1);
    Ok(Json(response))
}

fn event_name_is_valid(value: &str) -> bool {
    let mut chars = value.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    value.len() <= 100
        && first.is_ascii_alphabetic()
        && chars.all(|character| {
            character.is_ascii_alphanumeric() || matches!(character, '.' | '_' | '-')
        })
}

async fn list_events(State(state): State<AppState>) -> Result<Json<serde_json::Value>, StatusCode> {
    state
        .store
        .list_events()
        .await
        .map(Json)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

async fn get_metrics(
    State(state): State<AppState>,
    Path(metric): Path<String>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    state
        .store
        .metric(&metric)
        .await
        .map(Json)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

async fn get_revenue(State(state): State<AppState>) -> Result<Json<serde_json::Value>, StatusCode> {
    state
        .store
        .revenue()
        .await
        .map(Json)
        .map_err(|_| StatusCode::NOT_IMPLEMENTED)
}

fn parse_audit_query(raw_query: Option<&str>) -> Result<AuditListQuery, ()> {
    let mut parsed = AuditListQuery::default();
    let mut category_seen = false;
    let mut cursor_seen = false;
    let mut url = reqwest::Url::parse("http://analytics.invalid/")
        .expect("the fixed audit query base URL is valid");
    url.set_query(raw_query.filter(|query| !query.is_empty()));

    for (key, value) in url.query_pairs() {
        match key.as_ref() {
            "category" => {
                if category_seen {
                    return Err(());
                }
                category_seen = true;
                if !audit_category_is_valid(&value) {
                    return Err(());
                }
                parsed.category = Some(value.into_owned());
            }
            "cursor" => {
                if cursor_seen {
                    return Err(());
                }
                cursor_seen = true;
                if value.is_empty() || value.len() > MAX_AUDIT_CURSOR_CHARS {
                    return Err(());
                }
                parsed.cursor = Some(decode_audit_cursor(&value)?);
            }
            _ => return Err(()),
        }
    }
    if parsed
        .cursor
        .as_ref()
        .is_some_and(|cursor| cursor.category != parsed.category)
    {
        return Err(());
    }
    Ok(parsed)
}

fn audit_category_is_valid(category: &str) -> bool {
    matches!(
        category,
        "auth"
            | "developer"
            | "notification"
            | "payment"
            | "permission"
            | "plan"
            | "system"
            | "wallet"
    )
}

fn bounded_control_free(value: &str, max_chars: usize) -> bool {
    !value.trim().is_empty()
        && value.chars().count() <= max_chars
        && !value.chars().any(char::is_control)
}

fn audit_summary_is_valid(item: &AuditSummary) -> bool {
    uuid::Uuid::parse_str(&item.id).is_ok()
        && audit_category_is_valid(&item.category)
        && bounded_control_free(&item.action, 50)
        && bounded_control_free(&item.resource_type, 50)
        && matches!(item.effect.as_str(), "success" | "failure" | "denied")
}

fn encode_audit_cursor(cursor: &AuditCursor) -> Result<String, AnalyticsStoreError> {
    let bytes = serde_json::to_vec(cursor).map_err(|_| AnalyticsStoreError)?;
    Ok(URL_SAFE_NO_PAD.encode(bytes))
}

fn decode_audit_cursor(value: &str) -> Result<AuditCursor, ()> {
    let bytes = URL_SAFE_NO_PAD.decode(value).map_err(|_| ())?;
    let cursor: AuditCursor = serde_json::from_slice(&bytes).map_err(|_| ())?;
    if cursor
        .category
        .as_deref()
        .is_some_and(|value| !audit_category_is_valid(value))
    {
        return Err(());
    }
    if encode_audit_cursor(&cursor).map_err(|_| ())? != value {
        return Err(());
    }
    Ok(cursor)
}

async fn list_audit(State(state): State<AppState>, RawQuery(raw_query): RawQuery) -> Response {
    let query = match parse_audit_query(raw_query.as_deref()) {
        Ok(query) => query,
        Err(()) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({ "error": "invalid_audit_query" })),
            )
                .into_response();
        }
    };
    match state.store.list_audit(&query).await {
        Ok(list) => Json(list).into_response(),
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    }
}

async fn not_found() -> StatusCode {
    StatusCode::NOT_FOUND
}

async fn track_http_metrics(request: Request, next: Next) -> Response {
    let method = request.method().clone();
    let path_template = request
        .extensions()
        .get::<axum::extract::MatchedPath>()
        .map(|path| path.as_str().to_string())
        .unwrap_or_else(|| request.uri().path().to_string());
    let started = Instant::now();
    let response = next.run(request).await;
    let elapsed = started.elapsed().as_secs_f64();
    let status = response.status().as_u16().to_string();
    metrics::counter!(
        "epsx_http_requests_total",
        "method" => method.to_string(),
        "path" => path_template.clone(),
        "status" => status.clone()
    )
    .increment(1);
    metrics::histogram!(
        "epsx_http_request_duration_seconds",
        "method" => method.to_string(),
        "path" => path_template,
        "status" => status
    )
    .record(elapsed);
    response
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use epsx_service_auth::{VerifiedPrincipal, VerifyError};
    use std::sync::atomic::{AtomicUsize, Ordering};
    use tower::ServiceExt;

    struct FakeVerifier;

    #[async_trait]
    impl AccessTokenVerifier for FakeVerifier {
        async fn verify(&self, token: &str) -> Result<VerifiedPrincipal, VerifyError> {
            let (audience, permissions) = match token {
                "frontend" => (FRONTEND_AUDIENCE, vec![]),
                "frontend-with-permission" => {
                    (FRONTEND_AUDIENCE, vec![ANALYTICS_VIEW_PERMISSION.into()])
                }
                "admin" => (ADMIN_AUDIENCE, vec![]),
                "admin-view" => (ADMIN_AUDIENCE, vec![ANALYTICS_VIEW_PERMISSION.into()]),
                "admin-audit" => (ADMIN_AUDIENCE, vec![AUDIT_READ_PERMISSION.into()]),
                "admin-analytics-wildcard" => (ADMIN_AUDIENCE, vec!["admin:analytics:*".into()]),
                "admin-audit-wildcard" => (ADMIN_AUDIENCE, vec!["admin:audit:*".into()]),
                "admin-global" => (ADMIN_AUDIENCE, vec!["admin:*:*".into()]),
                "frontend-audit" => (FRONTEND_AUDIENCE, vec![AUDIT_READ_PERMISSION.into()]),
                "other-audience" => ("epsx-other", vec![]),
                "wrong-audience" => return Err(VerifyError::Validation),
                _ => return Err(VerifyError::Validation),
            };
            Ok(VerifiedPrincipal {
                subject: "0xabc".into(),
                wallet_address: "0xabc".into(),
                audience: audience.into(),
                permissions,
            })
        }
    }

    #[derive(Default)]
    struct FakeStore {
        hits: AtomicUsize,
        caller_identity_seen: AtomicUsize,
    }

    #[async_trait]
    impl AnalyticsStore for FakeStore {
        async fn track_event(
            &self,
            request: &TrackEventRequest,
        ) -> Result<serde_json::Value, AnalyticsStoreError> {
            self.hits.fetch_add(1, Ordering::SeqCst);
            if request.user_id.is_some() {
                self.caller_identity_seen.fetch_add(1, Ordering::SeqCst);
            }
            Ok(serde_json::json!({ "accepted": true, "event_id": "test" }))
        }

        async fn list_events(&self) -> Result<serde_json::Value, AnalyticsStoreError> {
            self.hits.fetch_add(1, Ordering::SeqCst);
            Ok(serde_json::json!([]))
        }

        async fn metric(&self, metric: &str) -> Result<serde_json::Value, AnalyticsStoreError> {
            self.hits.fetch_add(1, Ordering::SeqCst);
            Ok(serde_json::json!({ "metric": metric }))
        }

        async fn revenue(&self) -> Result<serde_json::Value, AnalyticsStoreError> {
            self.hits.fetch_add(1, Ordering::SeqCst);
            Ok(serde_json::json!({ "active_subscriptions": 0 }))
        }

        async fn list_audit(
            &self,
            _query: &AuditListQuery,
        ) -> Result<AuditList, AnalyticsStoreError> {
            self.hits.fetch_add(1, Ordering::SeqCst);
            Ok(AuditList {
                items: Vec::new(),
                next_cursor: None,
                has_more: false,
            })
        }
    }

    fn app() -> (Router, Arc<FakeStore>) {
        let store = Arc::new(FakeStore::default());
        (build_router(store.clone(), Arc::new(FakeVerifier)), store)
    }

    fn request(method: Method, path: &str, bearer: Option<&str>) -> axum::http::Request<Body> {
        let mut builder = axum::http::Request::builder().method(method).uri(path);
        if let Some(bearer) = bearer {
            builder = builder.header(header::AUTHORIZATION, format!("Bearer {bearer}"));
        }
        builder.body(Body::empty()).unwrap()
    }

    fn track_request(bearer: Option<&str>) -> axum::http::Request<Body> {
        let mut builder = axum::http::Request::builder()
            .method(Method::POST)
            .uri("/api/v1/analytics/track")
            .header(header::CONTENT_TYPE, "application/json");
        if let Some(bearer) = bearer {
            builder = builder.header(header::AUTHORIZATION, format!("Bearer {bearer}"));
        }
        builder
            .body(Body::from(
                r#"{"user_id":"00000000-0000-0000-0000-000000000001","event_name":"page.view"}"#,
            ))
            .unwrap()
    }

    async fn status(app: &Router, request: axum::http::Request<Body>) -> StatusCode {
        app.clone().oneshot(request).await.unwrap().status()
    }

    #[tokio::test]
    async fn health_is_the_only_anonymous_allowlist() {
        let (app, store) = app();
        assert_eq!(
            status(&app, request(Method::GET, "/health", None)).await,
            StatusCode::OK
        );
        assert_eq!(
            status(&app, request(Method::HEAD, "/health", None)).await,
            StatusCode::OK
        );
        assert_ne!(
            status(&app, request(Method::POST, "/health", None)).await,
            StatusCode::OK
        );
        assert_eq!(store.hits.load(Ordering::SeqCst), 0);
    }

    #[tokio::test]
    async fn track_requires_verified_frontend_or_admin_before_store() {
        let (app, store) = app();
        for bearer in [None, Some("wrong-audience")] {
            assert_eq!(
                status(&app, track_request(bearer)).await,
                StatusCode::UNAUTHORIZED
            );
            assert_eq!(store.hits.load(Ordering::SeqCst), 0);
        }
        assert_eq!(
            status(&app, track_request(Some("other-audience"))).await,
            StatusCode::FORBIDDEN
        );
        assert_eq!(store.hits.load(Ordering::SeqCst), 0);
        assert_eq!(
            status(&app, track_request(Some("frontend"))).await,
            StatusCode::OK
        );
        assert_eq!(
            status(&app, track_request(Some("admin"))).await,
            StatusCode::OK
        );
        assert_eq!(store.hits.load(Ordering::SeqCst), 2);
        assert_eq!(store.caller_identity_seen.load(Ordering::SeqCst), 2);
    }

    #[tokio::test]
    async fn spoofed_identity_headers_never_establish_a_principal() {
        let (app, store) = app();
        let mut anonymous = track_request(None);
        anonymous
            .headers_mut()
            .insert("x-user-id", "attacker".parse().unwrap());
        anonymous
            .headers_mut()
            .insert("x-permissions", ANALYTICS_VIEW_PERMISSION.parse().unwrap());
        assert_eq!(status(&app, anonymous).await, StatusCode::UNAUTHORIZED);

        let mut unprivileged_admin =
            request(Method::GET, "/api/v1/analytics/events", Some("admin"));
        unprivileged_admin
            .headers_mut()
            .insert("x-permissions", ANALYTICS_VIEW_PERMISSION.parse().unwrap());
        assert_eq!(
            status(&app, unprivileged_admin).await,
            StatusCode::FORBIDDEN
        );
        assert_eq!(store.hits.load(Ordering::SeqCst), 0);
    }

    #[tokio::test]
    async fn admin_reads_require_admin_audience_and_canonical_permission() {
        let (app, store) = app();
        for bearer in ["frontend-with-permission", "admin"] {
            assert_eq!(
                status(
                    &app,
                    request(Method::GET, "/api/v1/analytics/events", Some(bearer)),
                )
                .await,
                StatusCode::FORBIDDEN
            );
        }
        assert_eq!(store.hits.load(Ordering::SeqCst), 0);

        for path in [
            "/api/v1/analytics/events",
            "/api/v1/analytics/metrics/page.view",
            "/api/v1/analytics/revenue",
        ] {
            for bearer in ["admin-view", "admin-analytics-wildcard", "admin-global"] {
                assert_eq!(
                    status(&app, request(Method::GET, path, Some(bearer))).await,
                    StatusCode::OK
                );
            }
        }
        assert_eq!(store.hits.load(Ordering::SeqCst), 9);
    }

    #[tokio::test]
    async fn audit_read_requires_admin_audience_and_canonical_audit_permission() {
        let (app, store) = app();
        let path = "/api/v1/analytics/admin/audit-log";
        for bearer in [
            "frontend-audit",
            "admin",
            "admin-view",
            "admin-analytics-wildcard",
        ] {
            assert_eq!(
                status(&app, request(Method::GET, path, Some(bearer))).await,
                StatusCode::FORBIDDEN
            );
        }
        assert_eq!(store.hits.load(Ordering::SeqCst), 0);
        for bearer in ["admin-audit", "admin-audit-wildcard", "admin-global"] {
            assert_eq!(
                status(&app, request(Method::GET, path, Some(bearer))).await,
                StatusCode::OK
            );
        }
        assert_eq!(store.hits.load(Ordering::SeqCst), 3);
    }

    #[tokio::test]
    async fn audit_query_rejects_unknown_duplicate_or_malformed_values_before_store() {
        let (app, store) = app();
        for query in [
            "?limit=20",
            "?category=unknown",
            "?category=auth&category=system",
            "?cursor=not-base64url",
            "?unknown=value",
        ] {
            assert_eq!(
                status(
                    &app,
                    request(
                        Method::GET,
                        &format!("/api/v1/analytics/admin/audit-log{query}"),
                        Some("admin-audit"),
                    ),
                )
                .await,
                StatusCode::BAD_REQUEST,
                "query {query}"
            );
        }
        assert_eq!(store.hits.load(Ordering::SeqCst), 0);
    }

    #[test]
    fn audit_cursor_is_canonical_and_bound_to_its_category() {
        let cursor = encode_audit_cursor(&AuditCursor {
            occurred_at: chrono::DateTime::parse_from_rfc3339("2026-07-22T12:00:00Z")
                .unwrap()
                .with_timezone(&chrono::Utc),
            id: uuid::Uuid::nil(),
            category: Some("system".to_string()),
        })
        .unwrap();
        assert!(parse_audit_query(Some(&format!("category=system&cursor={cursor}"))).is_ok());
        assert!(parse_audit_query(Some(&format!("category=auth&cursor={cursor}"))).is_err());
        assert!(parse_audit_query(Some(&format!("cursor={cursor}"))).is_err());
    }

    #[test]
    fn audit_projection_sql_never_selects_sensitive_identity_or_detail_fields() {
        assert!(AUDIT_LIST_SQL.contains("ORDER BY created_at DESC, id DESC"));
        for forbidden in [
            "actor",
            "resource_id",
            "ip_address",
            "user_agent",
            "before_state",
            "after_state",
            "metadata",
        ] {
            assert!(
                !AUDIT_LIST_SQL.contains(forbidden),
                "audit SELECT leaked {forbidden}"
            );
        }
    }

    #[tokio::test]
    async fn internal_and_unknown_routes_fail_before_store() {
        let (app, store) = app();
        for (method, path) in [
            (Method::GET, "/api/v1/analytics/metrics/prometheus"),
            (Method::GET, "/api/v1/analytics/prometheus/metrics"),
            (Method::GET, "/api/v1/analytics/unknown"),
            (Method::POST, "/api/v1/analytics/events"),
        ] {
            assert_eq!(
                status(&app, request(method, path, Some("admin-view"))).await,
                StatusCode::NOT_FOUND
            );
        }
        assert_eq!(store.hits.load(Ordering::SeqCst), 0);
    }

    #[test]
    fn production_auth_requires_https_non_local_identity_urls() {
        assert!(build_auth_verifier(
            "http://issuer.example",
            "https://issuer.example/.well-known/jwks.json",
            true,
        )
        .is_err());
        assert!(build_auth_verifier(
            "https://issuer.localhost",
            "https://issuer.example/.well-known/jwks.json",
            true,
        )
        .is_err());
        assert!(build_auth_verifier(
            "https://issuer.example",
            "https://127.0.0.1/.well-known/jwks.json",
            true,
        )
        .is_err());
    }
}
