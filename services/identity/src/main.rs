use axum::{
    extract::State,
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use clap::Parser;
use deadpool_redis::redis::{aio::ConnectionLike, AsyncCommands};
use deadpool_redis::{Config as RedisConfig, Pool as RedisPool, Runtime};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use std::net::SocketAddr;
use std::str::FromStr;
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
    #[arg(long, default_value = "redis://:epsx@localhost:6379")]
    redis_url: String,
    #[arg(long, default_value = "super-secret-jwt-key")]
    jwt_secret: String,
}

#[derive(Clone)]
struct AppState {
    db: sqlx::PgPool,
    jwt_service: Arc<epsx_crypto::JwtService>,
    redis: RedisPool,
    domain: String,
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

    let redis_cfg = RedisConfig::from_url(&args.redis_url);
    let redis = redis_cfg
        .create_pool(Some(Runtime::Tokio1))
        .expect("Failed to create Redis pool");

    let domain = std::env::var("EPSX_AUTH_DOMAIN").unwrap_or_else(|_| "epsx.io".to_string());

    if let Ok(addr) = std::env::var("EPSX_BOOTSTRAP_ADMIN") {
        let addr = addr.to_lowercase();
        let _ = sqlx::query(
            "UPDATE users SET roles = ARRAY['admin','user'] WHERE address = $1"
        )
        .bind(&addr)
        .execute(&db)
        .await
        .map_err(|e| tracing::warn!("bootstrap admin: {e}"));
        tracing::info!("bootstrap admin role applied to {addr}");
    }

    let state = AppState { db, jwt_service, redis, domain };

    let app = Router::new()
        .route("/health", get(health))
        .route("/api/v1/identity/auth/challenge", post(auth_challenge))
        .route("/api/v1/identity/auth/siwe", post(siwe_auth))
        .route("/api/v1/identity/auth/refresh", post(refresh_token))
        .route("/api/v1/identity/auth/me", get(current_user))
        .route("/api/v1/identity/auth/demo", post(demo_login))
        .route("/api/v1/identity/users", get(list_users).post(create_user))
        .route("/api/v1/identity/users/{id}", get(get_user).put(update_user).delete(delete_user))
        .with_state(state);

    let addr: SocketAddr = format!("{}:{}", args.host, args.port).parse().unwrap();
    tracing::info!("Identity service listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn health() -> StatusCode {
    StatusCode::OK
}

#[derive(Deserialize)]
struct ChallengeRequest {
    address: String,
    chain_id: String,
}

#[derive(Serialize)]
struct ChallengeResponse {
    message: String,
    nonce: String,
    /// When the nonce expires (RFC3339).
    expires_at: String,
}

/// Issue a SIWE challenge. The wallet signs `message` (which includes the
/// nonce) and posts the signature to `/api/v1/identity/auth/siwe`. The
/// nonce is stored in Redis with a 5-minute TTL so it can be verified
/// exactly once.
async fn auth_challenge(
    State(state): State<AppState>,
    Json(req): Json<ChallengeRequest>,
) -> Result<Json<ChallengeResponse>, StatusCode> {
    let address = req.address.trim().to_lowercase();
    if !address.starts_with("0x") || address.len() != 42 {
        return Err(StatusCode::BAD_REQUEST);
    }
    let chain_id_num: u64 = req.chain_id.parse().map_err(|_| StatusCode::BAD_REQUEST)?;
    let uri = format!("https://{}", state.domain);

    let (message, nonce) = epsx_crypto::SiweVerifier::build_message(
        &state.domain,
        &address,
        chain_id_num,
        &uri,
        Some("Sign in to EPSX"),
    ).map_err(|e| { tracing::error!("siwe build: {e}"); StatusCode::INTERNAL_SERVER_ERROR })?;

    let mut conn = state.redis.get().await.map_err(|e| { tracing::error!("redis: {e}"); StatusCode::SERVICE_UNAVAILABLE })?;
    let key = format!("siwe:challenge:{}:{}", address, nonce);
    let _: () = conn.set_ex(&key, &address, 300).await.map_err(|e| { tracing::error!("redis set: {e}"); StatusCode::SERVICE_UNAVAILABLE })?;

    let expires_at = (chrono::Utc::now() + chrono::Duration::minutes(5))
        .to_rfc3339_opts(chrono::SecondsFormat::Secs, true);

    Ok(Json(ChallengeResponse { message, nonce, expires_at }))
}

async fn siwe_auth(
    State(state): State<AppState>,
    Json(req): Json<SiweAuthRequest>,
) -> Result<Json<AuthResponse>, StatusCode> {
    tracing::info!("siwe_auth called: message len={}, sig len={}, chain_id={}", req.message.len(), req.signature.len(), req.chain_id);
    // Parse the message once to extract the (nonce, address) pair so we can
    // look up + consume the challenge that was issued by `auth_challenge`.
    let parsed = siwe::Message::from_str(&req.message).map_err(|e| { tracing::warn!("siwe parse: {e}"); StatusCode::BAD_REQUEST })?;
    let parsed_nonce = parsed.nonce.clone();
    let parsed_address_lower = siwe::eip55(&parsed.address).to_lowercase();

    // Consume the challenge. Read + delete in two ops; the TTL means even
    // if delete fails, the nonce will expire in 5 min.
    let mut conn = state.redis.get().await.map_err(|e| { tracing::error!("redis: {e}"); StatusCode::SERVICE_UNAVAILABLE })?;
    let key = format!("siwe:challenge:{}:{}", parsed_address_lower, parsed_nonce);
    let stored: Option<String> = conn.get(&key).await
        .map_err(|e| { tracing::error!("redis get: {e}"); StatusCode::SERVICE_UNAVAILABLE })?;
    if stored.is_none() {
        return Err(StatusCode::UNAUTHORIZED);
    }
    let _: i64 = conn.del(&key).await
        .map_err(|e| { tracing::error!("redis del: {e}"); StatusCode::SERVICE_UNAVAILABLE })?;

    let address = epsx_crypto::SiweVerifier::verify(&req.message, &req.signature, &state.domain)
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

#[derive(Serialize)]
struct UserList {
    users: Vec<User>,
    total: i64,
}

#[derive(Deserialize)]
struct ListUsersQuery {
    limit: Option<i64>,
    offset: Option<i64>,
    role: Option<String>,
}

async fn list_users(
    State(state): State<AppState>,
    headers: HeaderMap,
    axum::extract::Query(q): axum::extract::Query<ListUsersQuery>,
) -> Result<Json<UserList>, StatusCode> {
    let _ = require_admin(&state, &headers).await?;

    let limit = q.limit.unwrap_or(50).clamp(1, 200);
    let offset = q.offset.unwrap_or(0).max(0);

    let (users, total) = if let Some(role) = q.role.as_ref() {
        let users: Vec<User> = sqlx::query_as::<_, User>(
            "SELECT id, address, chain_id, roles FROM users WHERE $1 = ANY(roles) ORDER BY created_at DESC LIMIT $2 OFFSET $3"
        )
        .bind(role)
        .bind(limit)
        .bind(offset)
        .fetch_all(&state.db)
        .await
        .map_err(|e| { tracing::error!("list_users db error: {e}"); StatusCode::INTERNAL_SERVER_ERROR })?;
        let total: (i64,) = sqlx::query_as("SELECT COUNT(*)::BIGINT FROM users WHERE $1 = ANY(roles)")
            .bind(role)
            .fetch_one(&state.db)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        (users, total.0)
    } else {
        let users: Vec<User> = sqlx::query_as::<_, User>(
            "SELECT id, address, chain_id, roles FROM users ORDER BY created_at DESC LIMIT $1 OFFSET $2"
        )
        .bind(limit)
        .bind(offset)
        .fetch_all(&state.db)
        .await
        .map_err(|e| { tracing::error!("list_users db error: {e}"); StatusCode::INTERNAL_SERVER_ERROR })?;
        let total: (i64,) = sqlx::query_as("SELECT COUNT(*)::BIGINT FROM users")
            .fetch_one(&state.db)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        (users, total.0)
    };

    Ok(Json(UserList { users, total }))
}

#[derive(Deserialize)]
struct CreateUserRequest {
    address: String,
    chain_id: String,
    roles: Option<Vec<String>>,
}

async fn create_user(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(req): Json<CreateUserRequest>,
) -> Result<Json<User>, StatusCode> {
    let _ = require_admin(&state, &headers).await?;

    let address = req.address.trim().to_lowercase();
    if !address.starts_with("0x") || address.len() != 42 {
        return Err(StatusCode::BAD_REQUEST);
    }
    let roles = req.roles.unwrap_or_else(|| vec!["user".to_string()]);

    let user: User = sqlx::query_as::<_, User>(
        "INSERT INTO users (address, chain_id, roles) VALUES ($1, $2, $3)
         ON CONFLICT (address) DO UPDATE SET roles = $3, updated_at = NOW()
         RETURNING id, address, chain_id, roles"
    )
    .bind(&address)
    .bind(&req.chain_id)
    .bind(&roles)
    .fetch_one(&state.db)
    .await
    .map_err(|e| { tracing::error!("create_user db error: {e}"); StatusCode::INTERNAL_SERVER_ERROR })?;

    Ok(Json(user))
}

#[derive(Deserialize)]
struct UpdateUserRequest {
    roles: Option<Vec<String>>,
    chain_id: Option<String>,
}

async fn update_user(
    State(state): State<AppState>,
    headers: HeaderMap,
    axum::extract::Path(id): axum::extract::Path<String>,
    Json(req): Json<UpdateUserRequest>,
) -> Result<Json<User>, StatusCode> {
    let _ = require_admin(&state, &headers).await?;

    let id_uuid = Uuid::parse_str(&id).map_err(|_| StatusCode::BAD_REQUEST)?;
    let user: User = sqlx::query_as::<_, User>(
        "UPDATE users SET
            roles = COALESCE($2, roles),
            chain_id = COALESCE($3, chain_id),
            updated_at = NOW()
         WHERE id = $1
         RETURNING id, address, chain_id, roles"
    )
    .bind(id_uuid)
    .bind(&req.roles)
    .bind(&req.chain_id)
    .fetch_one(&state.db)
    .await
    .map_err(|e| { tracing::error!("update_user db error: {e}"); StatusCode::INTERNAL_SERVER_ERROR })?;

    Ok(Json(user))
}

async fn delete_user(
    State(state): State<AppState>,
    headers: HeaderMap,
    axum::extract::Path(id): axum::extract::Path<String>,
) -> Result<StatusCode, StatusCode> {
    let _ = require_admin(&state, &headers).await?;
    let id_uuid = Uuid::parse_str(&id).map_err(|_| StatusCode::BAD_REQUEST)?;
    let res = sqlx::query("DELETE FROM users WHERE id = $1")
        .bind(id_uuid)
        .execute(&state.db)
        .await
        .map_err(|e| { tracing::error!("delete_user db error: {e}"); StatusCode::INTERNAL_SERVER_ERROR })?;
    if res.rows_affected() == 0 { Err(StatusCode::NOT_FOUND) } else { Ok(StatusCode::NO_CONTENT) }
}

async fn require_admin(state: &AppState, headers: &HeaderMap) -> Result<Uuid, StatusCode> {
    let token = extract_bearer(headers).ok_or(StatusCode::UNAUTHORIZED)?;
    let claims = state.jwt_service.verify_token(&token)
        .map_err(|_| StatusCode::UNAUTHORIZED)?;
    let sub_uuid = Uuid::parse_str(&claims.sub).map_err(|_| StatusCode::UNAUTHORIZED)?;
    let row: (Vec<String>,) = sqlx::query_as("SELECT roles FROM users WHERE id = $1")
        .bind(sub_uuid)
        .fetch_one(&state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    if row.0.iter().any(|r| r == "admin") {
        Ok(sub_uuid)
    } else {
        Err(StatusCode::FORBIDDEN)
    }
}
