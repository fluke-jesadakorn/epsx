//! epsx-pay-svc — Axum backend service for pay.epsx.io.
//!
//! wave49(slice-3): Modularized from the original 534-LoC
//! `main.rs`. The router setup is now thin; all handlers
//! live in `crate::handlers::*`.
//!
//! Module layout:
//! - `crate::db`     — schema init + alloy provider builder
//! - `crate::types`  — request/response structs (PayIntent,
//!   EscrowRecord, PayLink, …)
//! - `crate::handlers::intents`      — pay_intents CRUD
//! - `crate::handlers::escrows`      — escrow lifecycle
//! - `crate::handlers::pay_links`    — shareable URLs (slice-3)
//! - `crate::handlers::pay_history`  — per-address history (slice-3)
//! - `crate::handlers::pay_admin`    — admin ops (slice-3)
//! - `crate::handlers::pay_webhooks` — on-chain events (slice-3)
//!
//! Endpoints summary (all under `/api/v1/pay/*` except admin
//! which lives under `/api/v1/admin/pay/*` for symmetry with
//! the monolith's existing admin router):
//!
//! Public (pay service, no auth):
//! - POST   /api/v1/pay/intents
//! - GET    /api/v1/pay/intents
//! - GET    /api/v1/pay/intents/{id}
//! - POST   /api/v1/pay/intents/{id}/confirm
//! - POST   /api/v1/pay/intents/{id}/cancel
//! - GET    /api/v1/pay/escrows
//! - GET    /api/v1/pay/escrows/{id}
//! - POST   /api/v1/pay/escrows/{id}/release
//! - POST   /api/v1/pay/escrows/{id}/refund
//! - POST   /api/v1/pay/escrows/{id}/dispute
//! - POST   /api/v1/pay/escrows/{id}/resolve
//! - POST   /api/v1/pay/escrows/{id}/confirm-deposit
//! - POST   /api/v1/pay/links                   (slice-3)
//! - GET    /api/v1/pay/links/{slug}            (slice-3)
//! - POST   /api/v1/pay/links/{slug}/redeem     (slice-3)
//! - GET    /api/v1/pay/history/{address}       (slice-3)
//! - POST   /api/v1/pay/webhooks/on-chain       (slice-3)
//!
//! Admin (gated by monolith or future identity service):
//! - GET    /api/v1/admin/pay/intents                       (slice-3)
//! - POST   /api/v1/admin/pay/intents/{id}/force-cancel     (slice-3)
//! - POST   /api/v1/admin/pay/escrows/{id}/force-release    (slice-3)
//! - POST   /api/v1/admin/pay/escrows/{id}/force-refund     (slice-3)

use axum::{
    http::StatusCode,
    routing::{get, post},
    Router,
};
use clap::Parser;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::info;

pub mod db;
pub mod handlers;
pub mod types;

pub use db::{build_provider, init_schema};

#[derive(Parser)]
#[command(name = "epsx-pay-svc", about = "EPSX Pay Service")]
struct Args {
    #[arg(long, default_value = "8103")]
    port: u16,
    #[arg(long, default_value = "0.0.0.0")]
    host: String,
    #[arg(long, default_value = "postgres://epsx:epsx@localhost:5432/epsx_pay")]
    database_url: String,
    #[arg(long, default_value = "56")]
    chain_id: u64,
    #[arg(long, default_value = "0")]
    escrow_contract: String,
}

#[derive(Clone)]
pub struct AppState {
    pub db: sqlx::PgPool,
    pub chain_id: u64,
    pub provider: Arc<RwLock<Option<Arc<dyn alloy::providers::Provider + Send + Sync>>>>,
    pub escrow_contract: String,
}

async fn health() -> StatusCode { StatusCode::OK }

#[tokio::main]
async fn main() {
    epsx_observability::Observability::init("pay-svc");
    let args = Args::parse();

    let db = sqlx::PgPool::connect(&args.database_url)
        .await
        .expect("Failed to connect to database");

    init_schema(&db).await.expect("Failed to initialize schema");
    let provider = build_provider(args.chain_id);

    let app_state = AppState {
        db: db.clone(),
        chain_id: args.chain_id,
        provider,
        escrow_contract: args.escrow_contract,
    };

    let app = Router::new()
        .route("/health", get(health))
        // === Pay intents (5) ===
        .route("/api/v1/pay/intents",
            post(handlers::intents::create_pay_intent).get(handlers::intents::list_pay_intents))
        .route("/api/v1/pay/intents/{id}", get(handlers::intents::get_pay_intent))
        .route("/api/v1/pay/intents/{id}/confirm", post(handlers::intents::confirm_pay_intent))
        .route("/api/v1/pay/intents/{id}/cancel", post(handlers::intents::cancel_pay_intent))
        // === Pay escrows (7) ===
        .route("/api/v1/pay/escrows", get(handlers::escrows::list_escrows))
        .route("/api/v1/pay/escrows/{id}", get(handlers::escrows::get_escrow))
        .route("/api/v1/pay/escrows/{id}/release", post(handlers::escrows::release_escrow))
        .route("/api/v1/pay/escrows/{id}/refund", post(handlers::escrows::refund_escrow))
        .route("/api/v1/pay/escrows/{id}/dispute", post(handlers::escrows::dispute_escrow))
        .route("/api/v1/pay/escrows/{id}/resolve", post(handlers::escrows::resolve_dispute))
        .route("/api/v1/pay/escrows/{id}/confirm-deposit", post(handlers::escrows::confirm_escrow_deposit))
        // === Slice-3: pay links (3) ===
        .route("/api/v1/pay/links", post(handlers::pay_links::create_pay_link))
        .route("/api/v1/pay/links/{slug}", get(handlers::pay_links::get_pay_link))
        .route("/api/v1/pay/links/{slug}/redeem", post(handlers::pay_links::redeem_pay_link))
        // === Slice-3: history (1) ===
        .route("/api/v1/pay/history/{address}", get(handlers::pay_history::get_pay_history))
        // === Slice-3: webhooks (1) ===
        .route("/api/v1/pay/webhooks/on-chain", post(handlers::pay_webhooks::on_chain_webhook))
        // === Slice-3: admin (4) ===
        .route("/api/v1/admin/pay/intents", get(handlers::pay_admin::admin_list_pay_intents))
        .route("/api/v1/admin/pay/intents/{id}/force-cancel", post(handlers::pay_admin::admin_force_cancel_pay_intent))
        .route("/api/v1/admin/pay/escrows/{id}/force-release", post(handlers::pay_admin::admin_force_release_escrow))
        .route("/api/v1/admin/pay/escrows/{id}/force-refund", post(handlers::pay_admin::admin_force_refund_escrow))
        .with_state(app_state);

    let addr: SocketAddr = format!("{}:{}", args.host, args.port).parse().unwrap();
    info!("Pay service listening on {}", addr);
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}