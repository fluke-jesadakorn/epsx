use axum::{extract::State, http::StatusCode, routing::{get, post}, Json, Router};
use clap::Parser;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use std::net::SocketAddr;

#[derive(Parser)]
#[command(name = "epsx-subscription", about = "EPSX Subscription Service")]
struct Args {
    #[arg(long, default_value = "8104")]
    port: u16,
    #[arg(long, default_value = "0.0.0.0")]
    host: String,
    #[arg(long, default_value = "postgres://epsx:epsx@localhost:5432/epsx_subscription")]
    database_url: String,
}

#[derive(Clone)]
struct AppState {
    db: sqlx::PgPool,
}

#[derive(Serialize, FromRow)]
struct SubscriptionPlan {
    id: String,
    merchant_id: String,
    name: String,
    description: Option<String>,
    amount: String,
    currency: String,
    chain_id: String,
    interval: i32,
    active: bool,
    created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Serialize, FromRow)]
struct Subscription {
    id: String,
    user_id: String,
    plan_id: String,
    status: String,
    account_id: Option<String>,
    payment_token: Option<String>,
    vault_position_id: Option<String>,
    current_period_start: Option<chrono::DateTime<chrono::Utc>>,
    current_period_end: Option<chrono::DateTime<chrono::Utc>>,
    created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Deserialize)]
struct CreatePlanRequest {
    merchant_id: String,
    name: String,
    description: Option<String>,
    amount: String,
    currency: String,
    chain_id: String,
    interval: i32,
}

#[derive(Deserialize)]
struct CreateSubscriptionRequest {
    user_id: String,
    plan_id: String,
    account_id: String,
    payment_token: String,
}

#[tokio::main]
async fn main() {
    epsx_observability::Observability::init("subscription");
    let args = Args::parse();

    let db = sqlx::PgPool::connect(&args.database_url).await.expect("Failed to connect to database");

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS subscription_plans (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            merchant_id UUID NOT NULL,
            name VARCHAR(100) NOT NULL,
            description TEXT,
            amount VARCHAR(78) NOT NULL,
            currency VARCHAR(10) NOT NULL,
            chain_id VARCHAR(10) NOT NULL,
            interval INTEGER NOT NULL,
            active BOOLEAN DEFAULT true,
            created_at TIMESTAMPTZ DEFAULT NOW()
        )"
    ).execute(&db).await.expect("Failed to create plans table");

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS subscriptions (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            user_id UUID NOT NULL,
            plan_id UUID REFERENCES subscription_plans(id),
            status VARCHAR(20) DEFAULT 'active',
            account_id VARCHAR(42),
            payment_token VARCHAR(42),
            vault_position_id VARCHAR(100),
            current_period_start TIMESTAMPTZ,
            current_period_end TIMESTAMPTZ,
            created_at TIMESTAMPTZ DEFAULT NOW()
        )"
    ).execute(&db).await.expect("Failed to create subscriptions table");

    let state = AppState { db };

    let app = Router::new()
        .route("/health", get(health))
        .route("/api/v1/subscription/plans", post(create_plan).get(list_plans))
        .route("/api/v1/subscription/plans/{id}", get(get_plan))
        .route("/api/v1/subscription/subscriptions", post(create_subscription).get(list_subscriptions))
        .route("/api/v1/subscription/subscriptions/{id}", get(get_subscription))
        .route("/api/v1/subscription/subscriptions/{id}/cancel", post(cancel_subscription))
        .route("/api/v1/subscription/vault/{chain_id}", get(get_vault_config))
        .with_state(state);

    let addr: SocketAddr = format!("{}:{}", args.host, args.port).parse().unwrap();
    tracing::info!("Subscription service listening on {}", addr);
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn health() -> StatusCode { StatusCode::OK }

async fn create_plan(
    State(state): State<AppState>,
    Json(req): Json<CreatePlanRequest>,
) -> Result<Json<SubscriptionPlan>, StatusCode> {
    let plan: SubscriptionPlan = sqlx::query_as::<_, SubscriptionPlan>(
        "INSERT INTO subscription_plans (merchant_id, name, description, amount, currency, chain_id, interval) VALUES ($1, $2, $3, $4, $5, $6, $7) RETURNING *"
    ).bind(&req.merchant_id).bind(&req.name).bind(&req.description).bind(&req.amount).bind(&req.currency).bind(&req.chain_id).bind(&req.interval)
    .fetch_one(&state.db).await.map_err(|e| { tracing::error!("create_plan: {}", e); StatusCode::INTERNAL_SERVER_ERROR })?;
    Ok(Json(plan))
}

async fn list_plans(State(state): State<AppState>) -> Result<Json<Vec<SubscriptionPlan>>, StatusCode> {
    let plans: Vec<SubscriptionPlan> = sqlx::query_as::<_, SubscriptionPlan>("SELECT * FROM subscription_plans WHERE active = true ORDER BY created_at DESC")
        .fetch_all(&state.db).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(plans))
}

async fn get_plan(
    State(state): State<AppState>,
    axum::extract::Path(id): axum::extract::Path<String>,
) -> Result<Json<SubscriptionPlan>, StatusCode> {
    let plan: SubscriptionPlan = sqlx::query_as::<_, SubscriptionPlan>("SELECT * FROM subscription_plans WHERE id = $1")
        .bind(&id).fetch_one(&state.db).await.map_err(|_| StatusCode::NOT_FOUND)?;
    Ok(Json(plan))
}

async fn create_subscription(
    State(state): State<AppState>,
    Json(req): Json<CreateSubscriptionRequest>,
) -> Result<Json<Subscription>, StatusCode> {
    let sub: Subscription = sqlx::query_as::<_, Subscription>(
        "INSERT INTO subscriptions (user_id, plan_id, account_id, payment_token) VALUES ($1, $2, $3, $4) RETURNING *"
    ).bind(&req.user_id).bind(&req.plan_id).bind(&req.account_id).bind(&req.payment_token)
    .fetch_one(&state.db).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(sub))
}

async fn list_subscriptions(State(state): State<AppState>) -> Result<Json<Vec<Subscription>>, StatusCode> {
    let subs: Vec<Subscription> = sqlx::query_as::<_, Subscription>("SELECT * FROM subscriptions ORDER BY created_at DESC")
        .fetch_all(&state.db).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(subs))
}

async fn get_subscription(
    State(state): State<AppState>,
    axum::extract::Path(id): axum::extract::Path<String>,
) -> Result<Json<Subscription>, StatusCode> {
    let sub: Subscription = sqlx::query_as::<_, Subscription>("SELECT * FROM subscriptions WHERE id = $1")
        .bind(&id).fetch_one(&state.db).await.map_err(|_| StatusCode::NOT_FOUND)?;
    Ok(Json(sub))
}

async fn cancel_subscription(
    State(state): State<AppState>,
    axum::extract::Path(id): axum::extract::Path<String>,
) -> Result<Json<Subscription>, StatusCode> {
    let sub: Subscription = sqlx::query_as::<_, Subscription>(
        "UPDATE subscriptions SET status = 'cancelled' WHERE id = $1 RETURNING *"
    ).bind(&id).fetch_one(&state.db).await.map_err(|_| StatusCode::NOT_FOUND)?;
    Ok(Json(sub))
}

async fn get_vault_config(
    axum::extract::Path(chain_id): axum::extract::Path<String>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    Ok(Json(serde_json::json!({
        "chain_id": chain_id,
        "vault_address": "0x0000000000000000000000000000000000000000",
        "token_address": "0x55d398326f99059fF775485246999027B3197955",
        "stream_rate": "0.000000000000000001"
    })))
}
