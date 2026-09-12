use alloy_primitives::Address;
use axum::{
    extract::{Extension, Path as AxPath, State},
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use clap::{Parser, ValueEnum};
use epsx_service_auth::VerifiedPrincipal;
use epsx_wallet::{
    build_auth_verifier, canonical_owner, protect_router, verify_schema_compatibility,
};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use std::net::SocketAddr;
use std::str::FromStr;
use tracing::info;

mod commerce;

#[derive(Parser)]
#[command(name = "epsx-wallet", about = "EPSX Wallet Service")]
struct Args {
    #[arg(long, env = "PORT", default_value = "8102")]
    port: u16,
    #[arg(long, env = "HOST", default_value = "127.0.0.1")]
    host: String,
    #[arg(
        long,
        env = "DATABASE_URL",
        default_value = "postgres://epsx:epsx@localhost:5432/epsx_wallet"
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

#[derive(Serialize, Deserialize, FromRow)]
struct AccountResponse {
    address: String,
    chain_id: String,
    label: Option<String>,
    role: Option<String>,
}

#[derive(Serialize, Deserialize)]
struct VerifyMessageRequest {
    message: String,
    signature: String,
    expected_address: String,
}

#[derive(Serialize, Deserialize)]
struct VerifyMessageResponse {
    valid: bool,
    recovered_address: String,
}

#[tokio::main]
async fn main() {
    epsx_observability::Observability::init("wallet");
    let args = Args::parse();

    let production = matches!(args.environment, Environment::Production);
    let jwks_url = args.jwks_url.unwrap_or_else(|| {
        format!(
            "{}/.well-known/jwks.json",
            args.oidc_issuer.trim_end_matches('/')
        )
    });
    let verifier = build_auth_verifier(&args.oidc_issuer, &jwks_url, production)
        .expect("wallet OIDC configuration must be valid");

    let db = sqlx::PgPool::connect(&args.database_url)
        .await
        .expect("Failed to connect to database");
    verify_schema_compatibility(&db)
        .await
        .expect("wallet schema must be compatible before serving");

    let app = Router::new()
        .route("/health", get(health))
        .route("/api/v1/wallet/accounts", get(list_accounts))
        .route("/api/v1/wallet/accounts/{address}", get(get_account))
        .route("/api/v1/wallet/verify-message", post(verify_message))
        .route("/api/v1/admin/wallets", get(commerce::list_admin_wallets))
        .route("/api/admin/wallets", get(commerce::list_admin_wallets))
        .route(
            "/api/v1/admin/wallets/stats",
            get(commerce::admin_wallet_stats),
        )
        .route(
            "/api/admin/wallets/stats",
            get(commerce::admin_wallet_stats),
        )
        .route(
            "/api/v1/admin/wallets/{address}",
            get(commerce::get_admin_wallet),
        )
        .route(
            "/api/admin/wallets/{address}",
            get(commerce::get_admin_wallet),
        )
        .route(
            "/api/v1/admin/wallets/{address}/disable",
            post(commerce::disable_admin_wallet),
        )
        .route(
            "/api/admin/wallets/{address}/disable",
            post(commerce::disable_admin_wallet),
        )
        .route(
            "/api/v1/admin/wallets/{address}/enable",
            post(commerce::enable_admin_wallet),
        )
        .route(
            "/api/admin/wallets/{address}/enable",
            post(commerce::enable_admin_wallet),
        )
        .route(
            "/api/v1/admin/wallets/{address}/metadata",
            axum::routing::patch(commerce::update_admin_wallet_metadata),
        )
        .route(
            "/api/admin/wallets/{address}/metadata",
            axum::routing::patch(commerce::update_admin_wallet_metadata),
        )
        .route("/api/v1/admin/credits", get(commerce::admin_credit_stats))
        .route("/api/admin/credits", get(commerce::admin_credit_stats))
        .route(
            "/api/v1/admin/credits/{address}",
            get(commerce::get_admin_credits),
        )
        .route(
            "/api/admin/credits/{address}",
            get(commerce::get_admin_credits),
        )
        .route(
            "/api/v1/admin/credits/{address}/grant",
            post(commerce::grant_admin_credits),
        )
        .route(
            "/api/admin/credits/{address}/grant",
            post(commerce::grant_admin_credits),
        )
        .route(
            "/api/v1/admin/credits/{address}/revoke",
            post(commerce::revoke_admin_credits),
        )
        .route(
            "/api/admin/credits/{address}/revoke",
            post(commerce::revoke_admin_credits),
        )
        .with_state(AppState { db });
    let app = protect_router(app, verifier);

    let addr: SocketAddr = format!("{}:{}", args.host, args.port).parse().unwrap();
    info!("Wallet service listening on {}", addr);
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn health() -> StatusCode {
    StatusCode::OK
}

async fn list_accounts(
    State(state): State<AppState>,
    Extension(principal): Extension<VerifiedPrincipal>,
) -> Result<Json<Vec<AccountResponse>>, StatusCode> {
    let owner = canonical_owner(&principal, None)?;
    let accounts: Vec<AccountResponse> = sqlx::query_as::<_, AccountResponse>(
        "SELECT address, chain_id, label, role
         FROM public.accounts
         WHERE lower(address) = $1
         ORDER BY created_at DESC",
    )
    .bind(owner)
    .fetch_all(&state.db)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(accounts))
}

async fn get_account(
    State(state): State<AppState>,
    Extension(principal): Extension<VerifiedPrincipal>,
    AxPath(address): AxPath<String>,
) -> Result<Json<AccountResponse>, StatusCode> {
    let owner = canonical_owner(&principal, Some(&address))?;
    let account: AccountResponse = sqlx::query_as::<_, AccountResponse>(
        "SELECT address, chain_id, label, role
         FROM public.accounts
         WHERE lower(address) = $1
         LIMIT 1",
    )
    .bind(owner)
    .fetch_optional(&state.db)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    .ok_or(StatusCode::NOT_FOUND)?;
    Ok(Json(account))
}

async fn verify_message(
    Json(req): Json<VerifyMessageRequest>,
) -> Result<Json<VerifyMessageResponse>, StatusCode> {
    use alloy::signers::Signature;
    let sig_bytes = alloy::hex::decode(req.signature.trim_start_matches("0x"))
        .map_err(|_| StatusCode::BAD_REQUEST)?;
    let sig = Signature::try_from(sig_bytes.as_slice()).map_err(|_| StatusCode::BAD_REQUEST)?;
    let recovered = sig
        .recover_address_from_msg(req.message.as_bytes())
        .map_err(|_| StatusCode::BAD_REQUEST)?;
    let expected = Address::from_str(&req.expected_address).map_err(|_| StatusCode::BAD_REQUEST)?;
    Ok(Json(VerifyMessageResponse {
        valid: recovered == expected,
        recovered_address: format!("{:#x}", recovered),
    }))
}
