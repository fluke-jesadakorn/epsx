use axum::{extract::{Request, State}, http::StatusCode, middleware::{self, Next}, response::Response, routing::{get, post}, Json, Router};
use clap::Parser;
use metrics_exporter_prometheus::{PrometheusBuilder, PrometheusHandle};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use std::net::SocketAddr;
use std::time::Instant;

#[derive(Parser)]
#[command(name = "epsx-analytics", about = "EPSX Analytics Service")]
struct Args {
    #[arg(long, default_value = "8107")]
    port: u16,
    #[arg(long, default_value = "0.0.0.0")]
    host: String,
    #[arg(long, default_value = "postgres://epsx:epsx@localhost:5432/epsx_analytics")]
    database_url: String,
}

#[derive(Clone)]
struct AppState {
    db: sqlx::PgPool,
    prom: PrometheusHandle,
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

#[derive(Deserialize)]
struct TrackEventRequest {
    user_id: Option<String>,
    event_name: String,
    properties: Option<serde_json::Value>,
    chain_id: Option<String>,
}

#[derive(Serialize)]
#[allow(dead_code)]
struct MetricPoint {
    timestamp: String,
    value: f64,
}

#[tokio::main]
async fn main() {
    epsx_observability::Observability::init("analytics");
    let args = Args::parse();

    let db = sqlx::PgPool::connect(&args.database_url).await.expect("Failed to connect to database");

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS events (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            user_id UUID,
            event_name VARCHAR(100) NOT NULL,
            properties_json JSONB DEFAULT '{}',
            chain_id VARCHAR(10),
            created_at TIMESTAMPTZ DEFAULT NOW()
        )"
    ).execute(&db).await.expect("Failed to create events table");

    let prom = PrometheusBuilder::new()
        .install_recorder()
        .expect("Failed to install Prometheus recorder");

    let state = AppState { db, prom };

    let app = Router::new()
        .route("/health", get(health))
        .route("/api/v1/analytics/track", post(track_event))
        .route("/api/v1/analytics/events", get(list_events))
        .route("/api/v1/analytics/metrics/{metric}", get(get_metrics))
        .route("/api/v1/analytics/revenue", get(get_revenue))
        .route("/api/v1/analytics/metrics/prometheus", get(prometheus_metrics))
        .route("/api/v1/analytics/prometheus/metrics", get(prometheus_metrics))
        .layer(middleware::from_fn(track_http_metrics))
        .with_state(state);

    let addr: SocketAddr = format!("{}:{}", args.host, args.port).parse().unwrap();
    tracing::info!("Analytics service listening on {}", addr);
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn health() -> StatusCode { StatusCode::OK }

async fn track_event(
    State(state): State<AppState>,
    Json(req): Json<TrackEventRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let id = uuid::Uuid::new_v4();
    let user_uuid = req.user_id.as_deref().and_then(|s| uuid::Uuid::parse_str(s).ok());
    let props = req.properties.unwrap_or(serde_json::json!({}));
    sqlx::query("INSERT INTO events (id, user_id, event_name, properties_json, chain_id) VALUES ($1, $2, $3, $4, $5)")
        .bind(id).bind(user_uuid).bind(&req.event_name).bind(&props).bind(&req.chain_id)
        .execute(&state.db).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let chain_label = req.chain_id.clone().unwrap_or_else(|| "unknown".to_string());
    metrics::counter!("epsx_events_tracked_total", "event" => req.event_name.clone(), "chain" => chain_label).increment(1);

    Ok(Json(serde_json::json!({ "event_id": id.to_string(), "accepted": true })))
}

async fn list_events(State(state): State<AppState>) -> Result<Json<Vec<Event>>, StatusCode> {
    let events: Vec<Event> = sqlx::query_as::<_, Event>("SELECT * FROM events ORDER BY created_at DESC LIMIT 100")
        .fetch_all(&state.db).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(events))
}

async fn get_metrics(
    State(state): State<AppState>,
    axum::extract::Path(metric): axum::extract::Path<String>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM events WHERE event_name = $1")
        .bind(&metric).fetch_one(&state.db).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let last_24h: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM events WHERE event_name = $1 AND created_at >= NOW() - INTERVAL '24 hours'"
    )
    .bind(&metric).fetch_one(&state.db).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let unique_users: (Option<i64>,) = sqlx::query_as(
        "SELECT COUNT(DISTINCT user_id) FROM events WHERE event_name = $1 AND user_id IS NOT NULL"
    )
    .bind(&metric).fetch_one(&state.db).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(serde_json::json!({
        "metric": metric,
        "total": count.0,
        "last_24h": last_24h.0,
        "unique_users": unique_users.0.unwrap_or(0),
    })))
}

async fn get_revenue(State(state): State<AppState>) -> Result<Json<serde_json::Value>, StatusCode> {
    // Sum subscription_plans prices for active subscriptions.
    // For now we just count plans; real revenue needs the payment service.
    let active_plans: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM events WHERE event_name = 'subscription.created'")
        .fetch_one(&state.db).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let last_30d: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM events WHERE event_name = 'subscription.created' AND created_at >= NOW() - INTERVAL '30 days'"
    )
    .fetch_one(&state.db).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(serde_json::json!({
        "active_subscriptions": active_plans.0,
        "new_subscriptions_30d": last_30d.0,
        "currency": "USDT",
        "period": "30d",
        "note": "Aggregated from event log; integrate payment service for exact USD totals"
    })))
}

async fn prometheus_metrics(State(state): State<AppState>) -> Result<String, StatusCode> {
    Ok(state.prom.render())
}

async fn track_http_metrics(req: Request, next: Next) -> Response {
    let method = req.method().clone();
    let path_template = req
        .extensions()
        .get::<axum::extract::MatchedPath>()
        .map(|m| m.as_str().to_string())
        .unwrap_or_else(|| req.uri().path().to_string());
    let started = Instant::now();
    let response = next.run(req).await;
    let elapsed = started.elapsed().as_secs_f64();
    let status = response.status().as_u16().to_string();

    metrics::counter!(
        "epsx_http_requests_total",
        "method" => method.to_string(),
        "path" => path_template.clone(),
        "status" => status.clone()
    ).increment(1);
    metrics::histogram!(
        "epsx_http_request_duration_seconds",
        "method" => method.to_string(),
        "path" => path_template,
        "status" => status
    ).record(elapsed);

    response
}
