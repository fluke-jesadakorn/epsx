use async_trait::async_trait;
use axum::{
    extract::{Path, Request, State},
    http::{header, HeaderMap, HeaderName, Method, StatusCode},
    middleware::{self, Next},
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use epsx_service_auth::{
    authenticate_headers, AccessTokenVerifier, JwksVerifier, JwksVerifierConfig, ADMIN_AUDIENCE,
    FRONTEND_AUDIENCE,
};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use std::{
    sync::Arc,
    time::{Duration, Instant},
};
use thiserror::Error;

const ANALYTICS_VIEW_PERMISSION: &str = "admin:analytics:view";

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

#[derive(Serialize, FromRow)]
struct Event {
    id: uuid::Uuid,
    user_id: Option<uuid::Uuid>,
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
}

pub struct SqlAnalyticsStore {
    db: sqlx::PgPool,
}

impl SqlAnalyticsStore {
    pub fn new(db: sqlx::PgPool) -> Self {
        Self { db }
    }
}

pub async fn init_schema(db: &sqlx::PgPool) -> Result<(), sqlx::Error> {
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS events (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            user_id UUID,
            event_name VARCHAR(100) NOT NULL,
            properties_json JSONB DEFAULT '{}',
            chain_id VARCHAR(10),
            created_at TIMESTAMPTZ DEFAULT NOW()
        )",
    )
    .execute(db)
    .await?;
    Ok(())
}

#[async_trait]
impl AnalyticsStore for SqlAnalyticsStore {
    async fn track_event(
        &self,
        request: &TrackEventRequest,
    ) -> Result<serde_json::Value, AnalyticsStoreError> {
        let id = uuid::Uuid::new_v4();
        // Canonical subjects are wallet strings while this legacy column is
        // UUID. Persist NULL until a schema/domain slice defines attribution;
        // never turn the compatibility request field into identity.
        let user_uuid: Option<uuid::Uuid> = None;
        let properties = request
            .properties
            .clone()
            .unwrap_or_else(|| serde_json::json!({}));
        sqlx::query(
            "INSERT INTO events (id, user_id, event_name, properties_json, chain_id) \
             VALUES ($1, $2, $3, $4, $5)",
        )
        .bind(id)
        .bind(user_uuid)
        .bind(&request.event_name)
        .bind(&properties)
        .bind(&request.chain_id)
        .execute(&self.db)
        .await
        .map_err(|_| AnalyticsStoreError)?;

        Ok(serde_json::json!({
            "event_id": id.to_string(),
            "accepted": true
        }))
    }

    async fn list_events(&self) -> Result<serde_json::Value, AnalyticsStoreError> {
        let events: Vec<Event> =
            sqlx::query_as::<_, Event>("SELECT * FROM events ORDER BY created_at DESC LIMIT 100")
                .fetch_all(&self.db)
                .await
                .map_err(|_| AnalyticsStoreError)?;
        serde_json::to_value(events).map_err(|_| AnalyticsStoreError)
    }

    async fn metric(&self, metric: &str) -> Result<serde_json::Value, AnalyticsStoreError> {
        let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM events WHERE event_name = $1")
            .bind(metric)
            .fetch_one(&self.db)
            .await
            .map_err(|_| AnalyticsStoreError)?;
        let last_24h: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM events WHERE event_name = $1 \
             AND created_at >= NOW() - INTERVAL '24 hours'",
        )
        .bind(metric)
        .fetch_one(&self.db)
        .await
        .map_err(|_| AnalyticsStoreError)?;
        let unique_users: (Option<i64>,) = sqlx::query_as(
            "SELECT COUNT(DISTINCT user_id) FROM events \
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
        let active_plans: (i64,) =
            sqlx::query_as("SELECT COUNT(*) FROM events WHERE event_name = 'subscription.created'")
                .fetch_one(&self.db)
                .await
                .map_err(|_| AnalyticsStoreError)?;
        let last_30d: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM events WHERE event_name = 'subscription.created' \
             AND created_at >= NOW() - INTERVAL '30 days'",
        )
        .fetch_one(&self.db)
        .await
        .map_err(|_| AnalyticsStoreError)?;
        Ok(serde_json::json!({
            "active_subscriptions": active_plans.0,
            "new_subscriptions_30d": last_30d.0,
            "currency": "USDT",
            "period": "30d",
            "note": "Aggregated from event log; integrate payment service for exact USD totals"
        }))
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
            if principal.audience != ADMIN_AUDIENCE
                || !principal.permissions.iter().any(|held| held == permission)
            {
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
    Json(mut request): Json<TrackEventRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    // Preserve request compatibility but prevent caller-selected attribution.
    request.user_id = None;
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
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
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
                "admin-wildcard" => (ADMIN_AUDIENCE, vec!["admin:analytics:*".into()]),
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
        assert_eq!(store.caller_identity_seen.load(Ordering::SeqCst), 0);
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
    async fn admin_reads_require_admin_audience_and_literal_permission() {
        let (app, store) = app();
        for bearer in ["frontend-with-permission", "admin", "admin-wildcard"] {
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
            assert_eq!(
                status(&app, request(Method::GET, path, Some("admin-view"))).await,
                StatusCode::OK
            );
        }
        assert_eq!(store.hits.load(Ordering::SeqCst), 3);
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
