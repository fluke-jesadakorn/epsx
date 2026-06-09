use axum::{
    extract::State,
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use clap::Parser;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use std::net::SocketAddr;
use std::sync::Arc;
use uuid::Uuid;

#[derive(Parser)]
#[command(name = "epsx-identity", about = "EPSX Identity Service")]
struct Args {
    #[arg(long, default_value = "8101")]
    port: u16,
    #[arg(long, default_value = "0.0.0.0")]
    host: String,
    #[arg(long, default_value = "postgres://epsx:epsx@localhost:5432/epsx_identity")]
    database_url: String,
    #[arg(long, default_value = "redis://localhost:6379")]
    redis_url: String,
    #[arg(long, default_value = "super-secret-jwt-key")]
    jwt_secret: String,
}

#[derive(Clone)]
struct AppState {
    db: sqlx::PgPool,
    jwt_service: Arc<epsx_crypto::JwtService>,
}

#[derive(Deserialize)]
struct SiweAuthRequest {
    message: String,
    signature: String,
    chain_id: String,
}

#[derive(Serialize)]
struct AuthResponse {
    access_token: String,
    refresh_token: String,
    expires_in: u64,
    user: User,
}

#[derive(Serialize, FromRow)]
struct User {
    id: Uuid,
    address: String,
    chain_id: String,
    roles: Vec<String>,
}

#[derive(Deserialize)]
struct RefreshRequest {
    refresh_token: String,
}

#[tokio::main]
async fn main() {
    epsx_observability::Observability::init("identity");
    let args = Args::parse();

    let db = sqlx::PgPool::connect(&args.database_url)
        .await
        .expect("Failed to connect to database");

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS users (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            address VARCHAR(42) UNIQUE NOT NULL,
            chain_id VARCHAR(10) NOT NULL,
            roles TEXT[] DEFAULT '{user}',
            created_at TIMESTAMPTZ DEFAULT NOW(),
            updated_at TIMESTAMPTZ DEFAULT NOW()
        )"
    )
    .execute(&db)
    .await
    .expect("Failed to create users table");

    let jwt_service = Arc::new(epsx_crypto::JwtService::new(&args.jwt_secret, 3600, 604800));

    let state = AppState { db, jwt_service };

    let app = Router::new()
        .route("/health", get(health))
        .route("/api/v1/identity/auth/siwe", post(siwe_auth))
        .route("/api/v1/identity/auth/refresh", post(refresh_token))
        .route("/api/v1/identity/auth/me", get(current_user))
        .route("/api/v1/identity/auth/demo", post(demo_login))
        .route("/api/v1/identity/users/{id}", get(get_user))
        .with_state(state);

    let addr: SocketAddr = format!("{}:{}", args.host, args.port).parse().unwrap();
    tracing::info!("Identity service listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn health() -> StatusCode {
    StatusCode::OK
}

async fn siwe_auth(
    State(state): State<AppState>,
    Json(req): Json<SiweAuthRequest>,
) -> Result<Json<AuthResponse>, StatusCode> {
    let address = epsx_crypto::SiweVerifier::verify(&req.message, &req.signature, "epsx.io")
        .await
        .map_err(|_| StatusCode::UNAUTHORIZED)?;

    let address_lower = address.to_lowercase();

    let user: User = sqlx::query_as::<_, User>(
        "INSERT INTO users (address, chain_id) VALUES ($1, $2) ON CONFLICT (address) DO UPDATE SET chain_id = $2, updated_at = NOW() RETURNING id, address, chain_id, roles"
    )
    .bind(&address_lower)
    .bind(&req.chain_id)
    .fetch_one(&state.db)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let tokens = state.jwt_service
        .generate_tokens(&user.id.to_string(), &user.address, &user.chain_id, user.roles.clone())
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(AuthResponse {
        access_token: tokens.access_token,
        refresh_token: tokens.refresh_token,
        expires_in: tokens.expires_in,
        user,
    }))
}

async fn refresh_token(
    State(state): State<AppState>,
    Json(req): Json<RefreshRequest>,
) -> Result<Json<AuthResponse>, StatusCode> {
    let claims = state.jwt_service.verify_token(&req.refresh_token)
        .map_err(|_| StatusCode::UNAUTHORIZED)?;
    let sub_uuid = Uuid::parse_str(&claims.sub).map_err(|_| StatusCode::UNAUTHORIZED)?;

    let user: User = sqlx::query_as::<_, User>(
        "SELECT id, address, chain_id, roles FROM users WHERE id = $1"
    )
    .bind(sub_uuid)
    .fetch_one(&state.db)
    .await
    .map_err(|_| StatusCode::NOT_FOUND)?;

    let tokens = state.jwt_service
        .generate_tokens(&user.id.to_string(), &user.address, &user.chain_id, user.roles.clone())
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(AuthResponse {
        access_token: tokens.access_token,
        refresh_token: tokens.refresh_token,
        expires_in: tokens.expires_in,
        user,
    }))
}

async fn get_user(
    State(state): State<AppState>,
    axum::extract::Path(id): axum::extract::Path<String>,
) -> Result<Json<User>, StatusCode> {
    let user: User = sqlx::query_as::<_, User>(
        "SELECT id, address, chain_id, roles FROM users WHERE id = $1"
    )
    .bind(&id)
    .fetch_one(&state.db)
    .await
    .map_err(|_| StatusCode::NOT_FOUND)?;

    Ok(Json(user))
}

fn extract_bearer(headers: &HeaderMap) -> Option<String> {
    headers.get("authorization")?.to_str().ok()
        .and_then(|s| s.strip_prefix("Bearer "))
        .map(|s| s.to_string())
}

async fn current_user(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<User>, StatusCode> {
    let token = extract_bearer(&headers).ok_or(StatusCode::UNAUTHORIZED)?;
    let claims = state.jwt_service.verify_token(&token)
        .map_err(|e| { tracing::warn!("token verify failed: {e}"); StatusCode::UNAUTHORIZED })?;
    let sub_uuid = Uuid::parse_str(&claims.sub).map_err(|_| StatusCode::UNAUTHORIZED)?;
    let user: User = sqlx::query_as::<_, User>(
        "SELECT id, address, chain_id, roles FROM users WHERE id = $1"
    )
    .bind(sub_uuid)
    .fetch_optional(&state.db)
    .await
    .map_err(|e| { tracing::error!("me db error: {e}"); StatusCode::INTERNAL_SERVER_ERROR })?
    .ok_or(StatusCode::NOT_FOUND)?;
    Ok(Json(user))
}

#[derive(Deserialize)]
struct DemoLoginRequest {
    address: Option<String>,
    chain_id: Option<String>,
}

#[derive(Serialize)]
struct DemoLoginResponse {
    access_token: String,
    refresh_token: String,
    expires_in: u64,
    user: User,
    demo: bool,
}

/// Demo login: creates or fetches a user for the given wallet address (or a
/// hard-coded demo address if none is supplied). Issues a real JWT. Disabled
/// in production builds via the `EPSX_ENABLE_DEMO_LOGIN` env var.
async fn demo_login(
    State(state): State<AppState>,
    Json(req): Json<DemoLoginRequest>,
) -> Result<Json<DemoLoginResponse>, StatusCode> {
    if std::env::var("EPSX_ENABLE_DEMO_LOGIN").ok().as_deref() != Some("1") {
        tracing::warn!("demo login disabled");
        return Err(StatusCode::NOT_FOUND);
    }
    let address = req.address.unwrap_or_else(|| "0xDEMO0000000000000000000000000000000000".to_string());
    let chain_id = req.chain_id.unwrap_or_else(|| "56".to_string());

    let user: User = sqlx::query_as::<_, User>(
        "INSERT INTO users (address, chain_id) VALUES ($1, $2) ON CONFLICT (address) DO UPDATE SET chain_id = $2, updated_at = NOW() RETURNING id, address, chain_id, roles"
    )
    .bind(address.to_lowercase())
    .bind(&chain_id)
    .fetch_one(&state.db)
    .await
    .map_err(|e| { tracing::error!("demo login db error: {e}"); StatusCode::INTERNAL_SERVER_ERROR })?;

    let tokens = state.jwt_service
        .generate_tokens(&user.id.to_string(), &user.address, &user.chain_id, user.roles.clone())
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(DemoLoginResponse {
        access_token: tokens.access_token,
        refresh_token: tokens.refresh_token,
        expires_in: tokens.expires_in,
        user,
        demo: true,
    }))
}
