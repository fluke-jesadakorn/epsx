//! EPSX Pay: native transaction preparation and contract-scoped reconciliation.
//! `native_pay` writes intents/operations; `native_reconcile` alone finalizes
//! new escrow records from confirmed chain evidence. Legacy package contracts
//! and legacy Pay history stay separate. SQL-only force transfers stay blocked.

use axum::{
    extract::State,
    http::StatusCode,
    routing::{get, post},
    Router,
};
use clap::{Parser, ValueEnum};
use epsx_pay_svc::{build_auth_verifier, protect_native_router};
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::info;

pub mod db;
pub mod handlers;
mod merchant;
mod native_chain;
mod native_pay;
mod native_reconcile;
mod native_webhook;
pub mod types;

pub use db::{build_provider, verify_schema_compatibility};

#[derive(Parser)]
#[command(name = "epsx-pay-svc", about = "EPSX Pay Service")]
struct Args {
    #[arg(long, env = "PORT", default_value = "8103")]
    port: u16,
    #[arg(long, env = "HOST", default_value = "127.0.0.1")]
    host: String,
    // Read from DATABASE_URL env so the K8s manifest can override
    // the localhost default. Without `env = "DATABASE_URL"` the
    // env var would be set in the pod but clap would still pick
    // up the default and the service would try to connect to
    // localhost:5432 (which fails inside a pod).
    #[arg(
        long,
        env = "DATABASE_URL",
        default_value = "postgres://epsx:epsx@localhost:5432/epsx_payments_dev"
    )]
    database_url: String,
    // CHAIN_ID + ESCROW_CONTRACT defaults match the kustomize
    // env vars (56 + "0" placeholder), but reading from env
    // keeps the two sources of truth in sync.
    #[arg(long, env = "CHAIN_ID", default_value = "56")]
    chain_id: u64,
    #[arg(long, env = "ESCROW_CONTRACT", default_value = "0")]
    escrow_contract: String,
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
    Staging,
    Production,
}

#[derive(Clone)]
pub struct AppState {
    pub native_chain: Option<Arc<native_chain::Chain>>,
    pub db: sqlx::PgPool,
    pub chain_id: u64,
    pub provider: Arc<RwLock<Option<Arc<dyn alloy::providers::Provider + Send + Sync>>>>,
    pub escrow_contract: String,
}

async fn health() -> StatusCode {
    StatusCode::OK
}

async fn ready(State(state): State<AppState>) -> StatusCode {
    if let Some(c) = &state.native_chain {
        let verified = sqlx::query_scalar::<_, bool>("SELECT healthy AND last_checked_at > now()-interval '60 seconds' FROM pay_v1_chain_checkpoints WHERE chain_id=$1 AND contract_address=$2").bind(c.chain_id as i64).bind(c.contract.to_string().to_ascii_lowercase()).fetch_optional(&state.db).await.ok().flatten().unwrap_or(false);
        if !verified {
            return StatusCode::SERVICE_UNAVAILABLE;
        }
    }

    match sqlx::query_scalar::<_, i32>("SELECT 1")
        .fetch_one(&state.db)
        .await
    {
        Ok(1) => StatusCode::OK,
        Ok(_) | Err(_) => StatusCode::SERVICE_UNAVAILABLE,
    }
}

#[tokio::main]
async fn main() {
    epsx_observability::Observability::init("pay-svc");
    let args = Args::parse();

    let production = matches!(
        args.environment,
        Environment::Staging | Environment::Production
    );
    let jwks_url = args.jwks_url.unwrap_or_else(|| {
        format!(
            "{}/.well-known/jwks.json",
            args.oidc_issuer.trim_end_matches('/')
        )
    });
    let verifier = build_auth_verifier(&args.oidc_issuer, &jwks_url, production)
        .expect("pay OIDC configuration must be valid");

    let db = sqlx::PgPool::connect(&args.database_url)
        .await
        .expect("Failed to connect to database");

    verify_schema_compatibility(&db)
        .await
        .expect("Pay schema must be migrated and exactly compatible before startup");
    let provider = build_provider(args.chain_id);

    let native_chain = native_chain::Chain::from_env()
        .expect("invalid Pay v1 chain configuration")
        .map(Arc::new);
    if native_chain.is_some() {
        sqlx::query("SELECT d.contract_version,o.transaction_parameters,c.next_block FROM pay_v1_deals d CROSS JOIN pay_v1_operations o CROSS JOIN pay_v1_chain_checkpoints c LIMIT 0").execute(&db).await.expect("Pay v1 migrations must be explicitly applied before startup");
    }
    let app_state = AppState {
        native_chain,
        db: db.clone(),
        chain_id: args.chain_id,
        provider,
        escrow_contract: args.escrow_contract,
    };

    native_reconcile::spawn(app_state.clone());
    let app = application(app_state, verifier);

    let addr: SocketAddr = format!("{}:{}", args.host, args.port).parse().unwrap();
    info!("Pay service listening on {}", addr);
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

fn application(
    app_state: AppState,
    verifier: Arc<dyn epsx_service_auth::AccessTokenVerifier>,
) -> Router {
    let merchant_db = app_state.db.clone();
    let app = Router::new()
        .layer(axum::extract::DefaultBodyLimit::max(64 * 1024))
        .route(
            "/api/v1/admin/pay/contract/pause",
            post(native_pay::prepare_pause),
        )
        .route(
            "/api/v1/admin/pay/contract/operations/{id}",
            get(native_pay::contract_operation),
        )
        .route(
            "/api/v1/admin/pay/contract/operations/{id}/confirm",
            post(native_pay::confirm_contract_operation),
        )
        .route("/health", get(health))
        .route("/ready", get(ready))
        .route("/api/v1/pay/config", get(native_pay::config))
        .route("/api/v1/pay/webhooks/on-chain", post(native_webhook::hint))
        .route(
            "/api/v1/pay/intents",
            post(native_pay::create_intent).get(native_pay::list_intents),
        )
        .route("/api/v1/pay/intents/{id}", get(native_pay::get_intent))
        .route(
            "/api/v1/pay/intents/{id}/deposit",
            post(native_pay::prepare_deposit),
        )
        .route("/api/v1/pay/escrows", get(native_pay::list_intents))
        .route("/api/v1/pay/escrows/{id}", get(native_pay::get_intent))
        .route(
            "/api/v1/pay/escrows/{id}/release",
            post(native_pay::release),
        )
        .route("/api/v1/pay/escrows/{id}/refund", post(native_pay::refund))
        .route(
            "/api/v1/pay/escrows/{id}/dispute",
            post(native_pay::dispute),
        )
        .route(
            "/api/v1/pay/escrows/{id}/resolve",
            post(native_pay::resolve),
        )
        .route("/api/v1/pay/links", post(native_pay::create_link))
        .route("/api/v1/pay/links/{slug}", get(native_pay::get_link))
        .route(
            "/api/v1/pay/links/{slug}/redeem",
            post(native_pay::redeem_link),
        )
        .route("/api/v1/pay/operations/{id}", get(native_pay::operation))
        .route(
            "/api/v1/pay/operations/{id}/confirm",
            post(native_pay::confirm_operation),
        )
        .route(
            "/api/v1/pay/history/{address}",
            get(handlers::pay_history::get_pay_history),
        )
        .route("/api/v1/admin/pay/escrows", get(native_pay::admin_list))
        .route("/api/v1/admin/pay/escrows/{id}", get(native_pay::admin_get))
        .route(
            "/api/v1/admin/pay/escrows/{id}/resolve",
            post(native_pay::resolve),
        )
        .route(
            "/api/v1/admin/pay/operations/{id}",
            get(native_pay::operation),
        )
        .route(
            "/api/v1/admin/pay/operations/{id}/confirm",
            post(native_pay::confirm_operation),
        )
        // === Slice-3: admin (4) ===
        .route(
            "/api/v1/admin/pay/intents",
            get(handlers::pay_admin::admin_list_pay_intents),
        )
        .route(
            "/api/v1/admin/pay/intents/{id}/force-cancel",
            post(handlers::pay_admin::admin_force_cancel_pay_intent),
        )
        .route(
            "/api/v1/admin/pay/escrows/{id}/force-release",
            post(handlers::pay_admin::admin_force_release_escrow),
        )
        .route(
            "/api/v1/admin/pay/escrows/{id}/force-refund",
            post(handlers::pay_admin::admin_force_refund_escrow),
        )
        .route(
            "/api/v1/admin/pay/links",
            post(handlers::admin_commerce::create_admin_pay_link)
                .get(handlers::admin_commerce::list_admin_pay_links),
        )
        .route(
            "/api/v1/admin/pay/links/{id}/disable",
            post(handlers::admin_commerce::disable_admin_pay_link),
        )
        .route(
            "/api/v1/admin/pay/intents/{id}/cancel",
            post(handlers::admin_commerce::cancel_admin_pay_intent),
        )
        .with_state(app_state);
    let app = protect_native_router(app, verifier.clone());
    merchant::wrap(app, merchant_db, verifier)
}

#[cfg(test)]
mod native_integration;
