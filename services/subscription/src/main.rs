use axum::{
    extract::State,
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use clap::{Parser, ValueEnum};
use epsx_subscription::{build_auth_verifier, protect_router, verify_schema_compatibility};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use std::net::SocketAddr;
use uuid::Uuid;

mod admin;

#[derive(Parser)]
#[command(name = "epsx-subscription", about = "EPSX Subscription Service")]
struct Args {
    #[arg(long, default_value = "8104")]
    port: u16,
    #[arg(long, default_value = "0.0.0.0")]
    host: String,
    #[arg(
        long,
        env = "DATABASE_URL",
        default_value = "postgres://epsx:epsx@localhost:5432/epsx_subscription"
    )]
    database_url: String,
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
}

#[derive(Serialize, FromRow)]
struct SubscriptionPlan {
    id: Uuid,
    merchant_id: Uuid,
    name: String,
    description: Option<String>,
    amount: String,
    currency: String,
    chain_id: String,
    interval: i32,
    active: Option<bool>,
    created_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Serialize, FromRow)]
struct Subscription {
    id: Uuid,
    user_id: Uuid,
    plan_id: Option<Uuid>,
    status: Option<String>,
    account_id: Option<String>,
    payment_token: Option<String>,
    vault_position_id: Option<String>,
    current_period_start: Option<chrono::DateTime<chrono::Utc>>,
    current_period_end: Option<chrono::DateTime<chrono::Utc>>,
    created_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Deserialize)]
struct CreatePlanRequest {
    merchant_id: Uuid,
    name: String,
    description: Option<String>,
    amount: String,
    currency: String,
    chain_id: String,
    interval: i32,
}

#[derive(Deserialize)]
struct CreateSubscriptionRequest {
    user_id: Uuid,
    // New subscriptions require an explicit target even though legacy rows
    // may retain a NULL plan_id and are decoded as Option<Uuid> above.
    plan_id: Uuid,
    account_id: Option<String>,
    payment_token: Option<String>,
}

#[tokio::main]
async fn main() {
    epsx_observability::Observability::init("subscription");
    let args = Args::parse();

    let production = matches!(args.environment, Environment::Production);
    let jwks_url = args.jwks_url.unwrap_or_else(|| {
        format!(
            "{}/.well-known/jwks.json",
            args.oidc_issuer.trim_end_matches('/')
        )
    });
    let verifier = build_auth_verifier(&args.oidc_issuer, &jwks_url, production)
        .expect("subscription OIDC configuration must be valid");

    let db = sqlx::PgPool::connect(&args.database_url)
        .await
        .expect("Failed to connect to database");
    verify_schema_compatibility(&db)
        .await
        .expect("subscription schema must be compatible before serving");

    let state = AppState { db };

    #[rustfmt::skip]
    let app = Router::new()
        .route("/health", get(health))
        .route("/api/v1/subscription/plans", post(create_plan).get(list_plans))
        .route("/api/v1/subscription/plans/{id}", get(get_plan))
        .route("/api/v1/subscription/subscriptions", post(create_subscription).get(list_subscriptions))
        .route("/api/v1/subscription/subscriptions/{id}", get(get_subscription))
        .route("/api/v1/subscription/subscriptions/{id}/cancel", post(cancel_subscription))
        .route("/api/v1/subscription/vault/{chain_id}", get(get_vault_config))
        .route(
            "/api/v1/admin/subscription/plans",
            get(admin::list_plans).post(admin::create_plan),
        )
        .route(
            "/api/v1/admin/subscription/plans/{id}",
            get(admin::get_plan).patch(admin::update_plan),
        )
        .route(
            "/api/v1/admin/subscription/access",
            get(admin::get_access),
        )
        .route(
            "/api/v1/admin/subscription/access/assign",
            post(admin::assign_access),
        )
        .route(
            "/api/v1/admin/subscription/access/revoke",
            post(admin::revoke_access),
        )
        .with_state(state);
    let app = protect_router(app, verifier);

    let addr: SocketAddr = format!("{}:{}", args.host, args.port).parse().unwrap();
    tracing::info!("Subscription service listening on {}", addr);
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn health() -> StatusCode {
    StatusCode::OK
}

async fn create_plan(
    State(state): State<AppState>,
    Json(req): Json<CreatePlanRequest>,
) -> Result<Json<SubscriptionPlan>, StatusCode> {
    let plan: SubscriptionPlan = sqlx::query_as::<_, SubscriptionPlan>(
        "INSERT INTO public.subscription_plans (merchant_id, name, description, amount, currency, chain_id, interval) VALUES ($1, $2, $3, $4, $5, $6, $7) RETURNING *"
    ).bind(req.merchant_id).bind(&req.name).bind(&req.description).bind(&req.amount).bind(&req.currency).bind(&req.chain_id).bind(req.interval)
    .fetch_one(&state.db).await.map_err(|e| { tracing::error!("create_plan: {}", e); StatusCode::INTERNAL_SERVER_ERROR })?;
    Ok(Json(plan))
}

async fn list_plans(
    State(state): State<AppState>,
) -> Result<Json<Vec<SubscriptionPlan>>, StatusCode> {
    let plans: Vec<SubscriptionPlan> = sqlx::query_as::<_, SubscriptionPlan>(
        "SELECT * FROM public.subscription_plans WHERE active = true ORDER BY created_at DESC",
    )
    .fetch_all(&state.db)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(plans))
}

async fn get_plan(
    State(state): State<AppState>,
    axum::extract::Path(id): axum::extract::Path<Uuid>,
) -> Result<Json<SubscriptionPlan>, StatusCode> {
    let plan: SubscriptionPlan = sqlx::query_as::<_, SubscriptionPlan>(
        "SELECT * FROM public.subscription_plans WHERE id = $1",
    )
    .bind(id)
    .fetch_one(&state.db)
    .await
    .map_err(|_| StatusCode::NOT_FOUND)?;
    Ok(Json(plan))
}

async fn create_subscription(
    State(state): State<AppState>,
    Json(req): Json<CreateSubscriptionRequest>,
) -> Result<Json<Subscription>, StatusCode> {
    let sub: Subscription = sqlx::query_as::<_, Subscription>(
        "INSERT INTO public.subscriptions (user_id, plan_id, account_id, payment_token) VALUES ($1, $2, $3, $4) RETURNING *"
    ).bind(req.user_id).bind(req.plan_id).bind(&req.account_id).bind(&req.payment_token)
    .fetch_one(&state.db).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(sub))
}

async fn list_subscriptions(
    State(state): State<AppState>,
) -> Result<Json<Vec<Subscription>>, StatusCode> {
    let subs: Vec<Subscription> = sqlx::query_as::<_, Subscription>(
        "SELECT * FROM public.subscriptions ORDER BY created_at DESC",
    )
    .fetch_all(&state.db)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(subs))
}

async fn get_subscription(
    State(state): State<AppState>,
    axum::extract::Path(id): axum::extract::Path<Uuid>,
) -> Result<Json<Subscription>, StatusCode> {
    let sub: Subscription =
        sqlx::query_as::<_, Subscription>("SELECT * FROM public.subscriptions WHERE id = $1")
            .bind(id)
            .fetch_one(&state.db)
            .await
            .map_err(|_| StatusCode::NOT_FOUND)?;
    Ok(Json(sub))
}

async fn cancel_subscription(
    State(state): State<AppState>,
    axum::extract::Path(id): axum::extract::Path<Uuid>,
) -> Result<Json<Subscription>, StatusCode> {
    let sub: Subscription = sqlx::query_as::<_, Subscription>(
        "UPDATE public.subscriptions SET status = 'cancelled' WHERE id = $1 RETURNING *",
    )
    .bind(id)
    .fetch_one(&state.db)
    .await
    .map_err(|_| StatusCode::NOT_FOUND)?;
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

#[cfg(test)]
mod tests {
    use super::*;

    const USER_ID: &str = "018f6b72-7e42-7db2-9fd0-6e7df9ae59b7";
    const PLAN_ID: &str = "018f6b72-8001-7be1-9252-39fc6687e7f6";

    #[test]
    fn request_models_decode_uuid_columns_and_nullable_transport_fields() {
        let plan: CreatePlanRequest = serde_json::from_value(serde_json::json!({
            "merchant_id": USER_ID,
            "name": "Core",
            "amount": "10.00",
            "currency": "USD",
            "chain_id": "56",
            "interval": 30
        }))
        .expect("valid plan request must decode");
        let _: Uuid = plan.merchant_id;
        assert!(plan.description.is_none());

        let subscription: CreateSubscriptionRequest = serde_json::from_value(serde_json::json!({
            "user_id": USER_ID,
            "plan_id": PLAN_ID
        }))
        .expect("valid subscription request must decode");
        let _: Uuid = subscription.user_id;
        let _: Uuid = subscription.plan_id;
        assert!(subscription.account_id.is_none());
        assert!(subscription.payment_token.is_none());
    }

    #[test]
    fn malformed_uuid_request_fields_are_rejected_before_database_binding() {
        let invalid_merchant = serde_json::json!({
            "merchant_id": "not-a-uuid",
            "name": "Core",
            "amount": "10.00",
            "currency": "USD",
            "chain_id": "56",
            "interval": 30
        });
        assert!(serde_json::from_value::<CreatePlanRequest>(invalid_merchant).is_err());

        let invalid_user = serde_json::json!({
            "user_id": "not-a-uuid",
            "plan_id": PLAN_ID
        });
        assert!(serde_json::from_value::<CreateSubscriptionRequest>(invalid_user).is_err());

        let invalid_plan = serde_json::json!({
            "user_id": USER_ID,
            "plan_id": "not-a-uuid"
        });
        assert!(serde_json::from_value::<CreateSubscriptionRequest>(invalid_plan).is_err());
    }

    #[test]
    fn response_models_serialize_every_nullable_legacy_column_as_null() {
        let id = Uuid::parse_str(PLAN_ID).expect("fixture UUID must parse");
        let user_id = Uuid::parse_str(USER_ID).expect("fixture UUID must parse");
        let plan = SubscriptionPlan {
            id,
            merchant_id: user_id,
            name: "Core".into(),
            description: None,
            amount: "10.00".into(),
            currency: "USD".into(),
            chain_id: "56".into(),
            interval: 30,
            active: None,
            created_at: None,
        };
        let plan_json = serde_json::to_value(plan).expect("plan response must serialize");
        for field in ["description", "active", "created_at"] {
            assert!(plan_json[field].is_null(), "{field} must serialize as null");
        }

        let subscription = Subscription {
            id,
            user_id,
            plan_id: None,
            status: None,
            account_id: None,
            payment_token: None,
            vault_position_id: None,
            current_period_start: None,
            current_period_end: None,
            created_at: None,
        };
        let subscription_json =
            serde_json::to_value(subscription).expect("subscription response must serialize");
        for field in [
            "plan_id",
            "status",
            "account_id",
            "payment_token",
            "vault_position_id",
            "current_period_start",
            "current_period_end",
            "created_at",
        ] {
            assert!(
                subscription_json[field].is_null(),
                "{field} must serialize as null"
            );
        }
    }
}
