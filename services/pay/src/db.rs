//! Database init + schema migrations for epsx-pay-svc.
//!
//! wave49(slice-3): extracted from main.rs. Owns all the
//! `CREATE TABLE IF NOT EXISTS …` + `CREATE INDEX IF NOT EXISTS …`
//! statements. Called once at startup, before the axum router
//! binds. Tables:
//!
//! - `pay_intents` — one row per `POST /api/v1/pay/intents`
//! - `escrows` — created when an intent is confirmed
//! - `pay_links` — shareable URL slugs that resolve to intents
//!   (added in slice-3)
//! - `pay_webhook_events` — idempotency log for on-chain
//!   webhook deliveries (added in slice-3)

use sqlx::PgPool;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Initialize all tables + indexes. Idempotent (every CREATE
/// uses `IF NOT EXISTS`). Returns the pool wrapped in the
/// `AppState` shape — `db: PgPool`, plus the alloy provider
/// state for any handler that needs to query on-chain.
///
/// We keep this in a function (not a `lazy_static!` / `OnceCell`)
/// because the pool is per-process and we want a fresh schema
/// check on every startup. The cost is ~6 cheap `CREATE` calls
/// + 6 cheap `CREATE INDEX` calls — well under 100ms on a warm
/// Postgres.
pub async fn init_schema(db: &PgPool) -> Result<(), sqlx::Error> {
    // pay_intents
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS pay_intents (
            id VARCHAR(66) PRIMARY KEY,
            chain_id VARCHAR(10) NOT NULL,
            payer VARCHAR(42) NOT NULL,
            payee VARCHAR(42) NOT NULL,
            amount VARCHAR(78) NOT NULL,
            token_address VARCHAR(42) NOT NULL,
            status VARCHAR(30) DEFAULT 'pending',
            escrow_id VARCHAR(66),
            tx_hash VARCHAR(66),
            description TEXT,
            expires_at TIMESTAMPTZ,
            created_at TIMESTAMPTZ DEFAULT NOW(),
            updated_at TIMESTAMPTZ DEFAULT NOW()
        )"
    ).execute(db).await?;

    // escrows
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS escrows (
            id VARCHAR(66) PRIMARY KEY,
            chain_id VARCHAR(10) NOT NULL,
            payer VARCHAR(42) NOT NULL,
            payee VARCHAR(42) NOT NULL,
            amount VARCHAR(78) NOT NULL,
            token_address VARCHAR(42) NOT NULL,
            fee_amount VARCHAR(78) DEFAULT '0',
            status VARCHAR(30) DEFAULT 'active',
            on_chain_id VARCHAR(78),
            tx_hash VARCHAR(66),
            dispute_reason TEXT,
            created_at TIMESTAMPTZ DEFAULT NOW(),
            updated_at TIMESTAMPTZ DEFAULT NOW()
        )"
    ).execute(db).await?;

    // pay_links (slice-3)
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS pay_links (
            id VARCHAR(66) PRIMARY KEY,
            slug VARCHAR(32) UNIQUE NOT NULL,
            intent_id VARCHAR(66) NOT NULL,
            max_uses INTEGER DEFAULT 1,
            current_uses INTEGER DEFAULT 0,
            expires_at TIMESTAMPTZ,
            created_at TIMESTAMPTZ DEFAULT NOW()
        )"
    ).execute(db).await?;

    // pay_webhook_events (slice-3)
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS pay_webhook_events (
            event_id VARCHAR(128) PRIMARY KEY,
            intent_id VARCHAR(66),
            escrow_id VARCHAR(66),
            event_type VARCHAR(64) NOT NULL,
            payload JSONB NOT NULL,
            received_at TIMESTAMPTZ DEFAULT NOW()
        )"
    ).execute(db).await?;

    // Indexes
    sqlx::query("CREATE INDEX IF NOT EXISTS idx_pay_intents_payer ON pay_intents (payer, status)").execute(db).await?;
    sqlx::query("CREATE INDEX IF NOT EXISTS idx_pay_intents_payee ON pay_intents (payee, status)").execute(db).await?;
    sqlx::query("CREATE INDEX IF NOT EXISTS idx_escrows_status ON escrows (status)").execute(db).await?;
    sqlx::query("CREATE INDEX IF NOT EXISTS idx_pay_links_slug ON pay_links (slug)").execute(db).await?;
    sqlx::query("CREATE INDEX IF NOT EXISTS idx_pay_links_intent ON pay_links (intent_id)").execute(db).await?;
    sqlx::query("CREATE INDEX IF NOT EXISTS idx_pay_webhook_intent ON pay_webhook_events (intent_id)").execute(db).await?;

    Ok(())
}

/// Compute the 0.3% escrow fee from a U256-formatted amount string.
/// Pulled out of main.rs as part of the modularization so it can
/// be unit-tested independently and reused by any future handler
/// that needs fee math.
pub fn compute_fee(amount: &str) -> String {
    use std::str::FromStr;
    if let Ok(amt) = alloy::primitives::U256::from_str(amount) {
        // 0.3% fee
        let fee = amt / alloy::primitives::U256::from(333u64);
        fee.to_string()
    } else {
        "0".to_string()
    }
}

/// Build the alloy provider state — `Arc<RwLock<Option<…>>>` so the
/// provider can be lazily initialized per-chain (the alloy crate's
/// `Provider` impl is not `Send` in every flavor; we wrap with
/// `RwLock` so handlers can re-acquire).
pub fn build_provider(chain_id: u64) -> Arc<RwLock<Option<Arc<dyn alloy::providers::Provider + Send + Sync>>>> {
    let provider: Arc<RwLock<Option<Arc<dyn alloy::providers::Provider + Send + Sync>>>> =
        Arc::new(RwLock::new(None));
    if let Ok(p) = epsx_web3::provider_for_chain(epsx_kernel::ChainId(chain_id)) {
        // Best-effort: try_lock avoids the poison-future footgun.
        // If the lock is held elsewhere (it shouldn't be at startup),
        // we silently skip — the handler will return 503 if it
        // needs the provider.
        if let Ok(mut guard) = provider.try_write() {
            *guard = Some(Arc::from(p));
        }
    }
    provider
}

/// Re-export the response/request types that handlers need
/// without importing `crate::types::*` everywhere. Keeps the
/// module-public API surface small.
pub mod prelude {
    pub use crate::types::{
        CreatePayIntentRequest, CreatePayLinkRequest, DisputeEscrowRequest,
        EscrowListResponse, EscrowRecord, PayHistoryResponse, PayIntent,
        PayIntentListResponse, PayIntentResponse, PayLink, PayLinkResponse,
        RedeemPayLinkRequest, RedeemPayLinkResponse, RefundEscrowRequest,
        ReleaseEscrowRequest, ResolveDisputeRequest, WebhookAck,
    };
}