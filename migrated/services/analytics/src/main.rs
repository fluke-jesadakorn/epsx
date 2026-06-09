use axum::{extract::State, http::StatusCode, routing::{get, post}, Json, Router};
use clap::Parser;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use std::net::SocketAddr;

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
}

#[derive(Serialize, FromRow)]
struct Event {
    id: String,
    user_id: Option<String>,
    event_name: String,
    properties_json: Option<String>,
    chain_id: Option<String>,
    created_at: chrono::NaiveDateTime,
}

#[derive(Deserialize)]
struct TrackEventRequest {
    user_id: Option<String>,
    event_name: String,
    properties: Option<serde_json::Value>,
    chain_id: Option<String>,
}

#[derive(Serialize)]
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

    let state = AppState { db };

    let app = Router::new()
        .route("/health", get(health))
        .route("/api/v1/analytics/track", post(track_event))
        .route("/api/v1/analytics/events", get(list_events))
        .route("/api/v1/analytics/metrics/{metric}", get(get_metrics))
        .route("/api/v1/analytics/revenue", get(get_revenue))
        .route("/api/v1/analytics/prometheus/metrics", get(prometheus_metrics))
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
    let id = uuid::Uuid::new_v4().to_string();
    let props = req.properties.unwrap_or(serde_json::json!({}));
    sqlx::query("INSERT INTO events (id, user_id, event_name, properties_json, chain_id) VALUES ($1, $2, $3, $4, $5)")
        .bind(&id).bind(&req.user_id).bind(&req.event_name).bind(&props.to_string()).bind(&req.chain_id)
        .execute(&state.db).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    metrics::counter!("events_tracked").increment(1);
    Ok(Json(serde_json::json!({ "event_id": id, "accepted": true })))
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
    Ok(Json(serde_json::json!({
        "metric": metric,
        "count": count.0
    })))
}

async fn get_revenue(State(state): State<AppState>) -> Result<Json<serde_json::Value>, StatusCode> {
    Ok(Json(serde_json::json!({
        "total_revenue": "0",
        "currency": "USDT",
        "period": "all"
    })))
}

async fn prometheus_metrics() -> Result<String, StatusCode> {
    Ok("# Prometheus metrics placeholder\n".to_string())
}
