use axum::{
    extract::{Path as AxPath, State},
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use clap::Parser;
use epsx_kernel::{ChainId, Money, PaymentStatus, Token};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use std::net::SocketAddr;
use std::str::FromStr;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::info;
use alloy::primitives::Address;

#[derive(Parser)]
#[command(name = "epsx-payment", about = "EPSX Payment Service")]
struct Args {
    #[arg(long, default_value = "8103")]
    port: u16,
    #[arg(long, default_value = "0.0.0.0")]
    host: String,
    #[arg(long, default_value = "postgres://epsx:epsx@localhost:5432/epsx_payment")]
    database_url: String,
    #[arg(long, default_value = "56")]
    chain_id: u64,
    #[arg(long, default_value = "0")]
    escrow_contract: String,
}

#[derive(Clone)]
struct AppState {
    db: sqlx::PgPool,
    chain_id: u64,
    provider: Arc<RwLock<Option<Arc<dyn alloy::providers::Provider + Send + Sync>>>>,
    escrow_contract: String,
}

#[derive(Serialize, Deserialize, FromRow, Clone)]
struct PaymentIntent {
    id: String,
    chain_id: String,
    payer: String,
    payee: String,
    amount: String,
    token_address: String,
    status: String,
    escrow_id: Option<String>,
    tx_hash: Option<String>,
    description: Option<String>,
    expires_at: Option<chrono::NaiveDateTime>,
    created_at: chrono::NaiveDateTime,
    updated_at: chrono::NaiveDateTime,
}

#[derive(Serialize, Deserialize, FromRow, Clone)]
struct EscrowRecord {
    id: String,
    chain_id: String,
    payer: String,
    payee: String,
    amount: String,
    token_address: String,
    fee_amount: String,
    status: String,
    on_chain_id: Option<String>,
    tx_hash: Option<String>,
    dispute_reason: Option<String>,
    created_at: chrono::NaiveDateTime,
    updated_at: chrono::NaiveDateTime,
}

#[derive(Serialize, Deserialize)]
struct CreateIntentRequest {
    payer: String,
    payee: String,
    amount: String,
    token: String,
    description: Option<String>,
    expires_in: Option<i64>,
}

#[derive(Serialize, Deserialize)]
struct IntentResponse {
    intent: PaymentIntent,
    payment_url: String,
    expires_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Serialize, Deserialize)]
struct ReleaseEscrowRequest {
    escrow_id: String,
    signature: Option<String>,
}

#[derive(Serialize, Deserialize)]
struct RefundEscrowRequest {
    escrow_id: String,
    reason: Option<String>,
}

#[derive(Serialize, Deserialize)]
struct DisputeEscrowRequest {
    escrow_id: String,
    reason: String,
}

#[derive(Serialize, Deserialize)]
struct ResolveDisputeRequest {
    escrow_id: String,
    to_payee: bool,
    signature: Option<String>,
}

#[derive(Serialize, Deserialize)]
struct IntentListResponse {
    items: Vec<PaymentIntent>,
    total: i64,
}

#[derive(Serialize, Deserialize)]
struct EscrowListResponse {
    items: Vec<EscrowRecord>,
    total: i64,
}

#[tokio::main]
async fn main() {
    epsx_observability::Observability::init("payment");
    let args = Args::parse();

    let db = sqlx::PgPool::connect(&args.database_url).await.expect("Failed to connect to database");

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS payment_intents (
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
    ).execute(&db).await.expect("Failed to create payment_intents table");

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
    ).execute(&db).await.expect("Failed to create escrows table");

    sqlx::query("CREATE INDEX IF NOT EXISTS idx_intents_payer ON payment_intents (payer, status)").execute(&db).await.expect("Failed to create intent index");
    sqlx::query("CREATE INDEX IF NOT EXISTS idx_intents_payee ON payment_intents (payee, status)").execute(&db).await.expect("Failed to create intent index");
    sqlx::query("CREATE INDEX IF NOT EXISTS idx_escrows_status ON escrows (status)").execute(&db).await.expect("Failed to create escrow index");

    let provider: Arc<RwLock<Option<Arc<dyn alloy::providers::Provider + Send + Sync>>>> =
        Arc::new(RwLock::new(None));

    if let Ok(p) = epsx_web3::provider_for_chain(ChainId(args.chain_id)) {
        *provider.write().await = Some(Arc::from(p));
    }

    let app = Router::new()
        .route("/health", get(health))
        .route("/api/v1/payment/intents", post(create_intent).get(list_intents))
        .route("/api/v1/payment/intents/{id}", get(get_intent))
        .route("/api/v1/payment/intents/{id}/confirm", post(confirm_intent))
        .route("/api/v1/payment/intents/{id}/cancel", post(cancel_intent))
        .route("/api/v1/payment/escrows", get(list_escrows))
        .route("/api/v1/payment/escrows/{id}", get(get_escrow))
        .route("/api/v1/payment/escrows/{id}/release", post(release_escrow))
        .route("/api/v1/payment/escrows/{id}/refund", post(refund_escrow))
        .route("/api/v1/payment/escrows/{id}/dispute", post(dispute_escrow))
        .route("/api/v1/payment/escrows/{id}/resolve", post(resolve_dispute))
        .route("/api/v1/payment/escrows/{id}/confirm-deposit", post(confirm_escrow_deposit))
        .with_state(AppState {
            db,
            chain_id: args.chain_id,
            provider,
            escrow_contract: args.escrow_contract,
        });

    let addr: SocketAddr = format!("{}:{}", args.host, args.port).parse().unwrap();
    info!("Payment service listening on {}", addr);
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn health() -> StatusCode { StatusCode::OK }

async fn create_intent(
    State(state): State<AppState>,
    Json(req): Json<CreateIntentRequest>,
) -> Result<Json<IntentResponse>, StatusCode> {
    // Validate addresses
    let _payer = Address::from_str(&req.payer).map_err(|_| StatusCode::BAD_REQUEST)?;
    let _payee = Address::from_str(&req.payee).map_err(|_| StatusCode::BAD_REQUEST)?;

    // Resolve token address
    let chain_id_enum = ChainId(state.chain_id);
    let token_symbol = req.token.to_uppercase();
    let token = match token_symbol.as_str() {
        "USDT" => Token::USDT,
        "USDC" => Token::USDC,
        "BNB" => Token::BNB,
        _ => return Err(StatusCode::BAD_REQUEST),
    };
    let token_address = token.address(chain_id_enum)
        .map(|a| a.0)
        .unwrap_or_else(|| "0x0000000000000000000000000000000000000000".to_string());

    let id = format!("0x{}", uuid::Uuid::new_v4().simple());
    let now = chrono::Utc::now().naive_utc();
    let expires = req.expires_in.unwrap_or(3600);
    let expires_at = chrono::Utc::now()
        .checked_add_signed(chrono::Duration::seconds(expires))
        .unwrap()
        .naive_utc();

    sqlx::query(
        "INSERT INTO payment_intents (id, chain_id, payer, payee, amount, token_address, status, description, expires_at, created_at, updated_at)
         VALUES ($1, $2, $3, $4, $5, $6, 'pending', $7, $8, $9, $9)"
    )
    .bind(&id)
    .bind(state.chain_id.to_string())
    .bind(&req.payer.to_lowercase())
    .bind(&req.payee.to_lowercase())
    .bind(&req.amount)
    .bind(&token_address)
    .bind(&req.description)
    .bind(expires_at)
    .bind(now)
    .execute(&state.db)
    .await
    .map_err(|e| { tracing::error!("intent insert: {}", e); StatusCode::INTERNAL_SERVER_ERROR })?;

    let intent = PaymentIntent {
        id: id.clone(),
        chain_id: state.chain_id.to_string(),
        payer: req.payer.to_lowercase(),
        payee: req.payee.to_lowercase(),
        amount: req.amount.clone(),
        token_address,
        status: "pending".to_string(),
        escrow_id: None,
        tx_hash: None,
        description: req.description,
        expires_at: Some(expires_at),
        created_at: now,
        updated_at: now,
    };

    let payment_url = format!("/pay?intent={}", id);

    Ok(Json(IntentResponse {
        intent,
        payment_url,
        expires_at: chrono::DateTime::<chrono::Utc>::from_naive_utc_and_offset(expires_at, chrono::Utc),
    }))
}

async fn list_intents(
    State(state): State<AppState>,
    axum::extract::Query(params): axum::extract::Query<std::collections::HashMap<String, String>>,
) -> Result<Json<IntentListResponse>, StatusCode> {
    let payer = params.get("payer").cloned();
    let status = params.get("status").cloned();
    let limit: i64 = params.get("limit").and_then(|s| s.parse().ok()).unwrap_or(50);
    let offset: i64 = params.get("offset").and_then(|s| s.parse().ok()).unwrap_or(0);

    let mut q = "SELECT id, chain_id, payer, payee, amount, token_address, status, escrow_id, tx_hash, description, expires_at, created_at, updated_at FROM payment_intents WHERE 1=1".to_string();
    let mut args: Vec<String> = vec![];
    if let Some(p) = &payer {
        args.push(p.clone());
        q.push_str(&format!(" AND payer = ${}", args.len()));
    }
    if let Some(s) = &status {
        args.push(s.clone());
        q.push_str(&format!(" AND status = ${}", args.len()));
    }
    q.push_str(&format!(" ORDER BY created_at DESC LIMIT {} OFFSET {}", limit, offset));

    let mut query = sqlx::query_as::<_, PaymentIntent>(&q);
    for a in &args { query = query.bind(a); }
    let items: Vec<PaymentIntent> = query.fetch_all(&state.db).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let total: i64 = if args.is_empty() {
        sqlx::query_scalar("SELECT COUNT(*) FROM payment_intents").fetch_one(&state.db).await.unwrap_or(0)
    } else {
        let mut q2 = "SELECT COUNT(*) FROM payment_intents WHERE 1=1".to_string();
        if let Some(_p) = &payer { q2.push_str(" AND payer = $1"); }
        if let Some(_s) = &status {
            let idx = if payer.is_some() { 2 } else { 1 };
            q2.push_str(&format!(" AND status = ${}", idx));
        }
        let mut query2 = sqlx::query_scalar(&q2);
        for a in &args { query2 = query2.bind(a); }
        query2.fetch_one(&state.db).await.unwrap_or(0)
    };

    Ok(Json(IntentListResponse { items, total }))
}

async fn get_intent(
    State(state): State<AppState>,
    AxPath(id): AxPath<String>,
) -> Result<Json<PaymentIntent>, StatusCode> {
    let intent: PaymentIntent = sqlx::query_as::<_, PaymentIntent>(
        "SELECT id, chain_id, payer, payee, amount, token_address, status, escrow_id, tx_hash, description, expires_at, created_at, updated_at FROM payment_intents WHERE id = $1"
    )
    .bind(&id)
    .fetch_optional(&state.db)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    .ok_or(StatusCode::NOT_FOUND)?;
    Ok(Json(intent))
}

async fn confirm_intent(
    State(state): State<AppState>,
    AxPath(id): AxPath<String>,
    Json(req): Json<serde_json::Value>,
) -> Result<Json<PaymentIntent>, StatusCode> {
    let tx_hash = req.get("tx_hash").and_then(|v| v.as_str()).unwrap_or_default().to_string();
    if tx_hash.is_empty() {
        return Err(StatusCode::BAD_REQUEST);
    }

    // Create escrow record for this payment
    let intent: PaymentIntent = sqlx::query_as::<_, PaymentIntent>(
        "SELECT id, chain_id, payer, payee, amount, token_address, status, escrow_id, tx_hash, description, expires_at, created_at, updated_at FROM payment_intents WHERE id = $1"
    )
    .bind(&id)
    .fetch_optional(&state.db)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    .ok_or(StatusCode::NOT_FOUND)?;

    if intent.status != "pending" {
        return Err(StatusCode::CONFLICT);
    }

    let escrow_id = format!("0x{}", uuid::Uuid::new_v4().simple());
    let fee = compute_fee(&intent.amount);

    sqlx::query(
        "INSERT INTO escrows (id, chain_id, payer, payee, amount, token_address, fee_amount, status, tx_hash) VALUES ($1, $2, $3, $4, $5, $6, $7, 'active', $8)"
    )
    .bind(&escrow_id)
    .bind(&intent.chain_id)
    .bind(&intent.payer)
    .bind(&intent.payee)
    .bind(&intent.amount)
    .bind(&intent.token_address)
    .bind(&fee)
    .bind(&tx_hash)
    .execute(&state.db)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    sqlx::query("UPDATE payment_intents SET status = 'escrowed', escrow_id = $1, tx_hash = $2, updated_at = NOW() WHERE id = $3")
        .bind(&escrow_id)
        .bind(&tx_hash)
        .bind(&id)
        .execute(&state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    get_intent(State(state), AxPath(id)).await
}

async fn cancel_intent(
    State(state): State<AppState>,
    AxPath(id): AxPath<String>,
) -> Result<Json<PaymentIntent>, StatusCode> {
    sqlx::query("UPDATE payment_intents SET status = 'cancelled', updated_at = NOW() WHERE id = $1 AND status = 'pending'")
        .bind(&id)
        .execute(&state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    get_intent(State(state), AxPath(id)).await
}

async fn list_escrows(
    State(state): State<AppState>,
    axum::extract::Query(params): axum::extract::Query<std::collections::HashMap<String, String>>,
) -> Result<Json<EscrowListResponse>, StatusCode> {
    let status = params.get("status").cloned();
    let limit: i64 = params.get("limit").and_then(|s| s.parse().ok()).unwrap_or(50);
    let offset: i64 = params.get("offset").and_then(|s| s.parse().ok()).unwrap_or(0);

    let items: Vec<EscrowRecord> = if let Some(s) = &status {
        sqlx::query_as::<_, EscrowRecord>(
            "SELECT id, chain_id, payer, payee, amount, token_address, fee_amount, status, on_chain_id, tx_hash, dispute_reason, created_at, updated_at FROM escrows WHERE status = $1 ORDER BY created_at DESC LIMIT $2 OFFSET $3"
        )
        .bind(s)
        .bind(limit)
        .bind(offset)
        .fetch_all(&state.db)
        .await
    } else {
        sqlx::query_as::<_, EscrowRecord>(
            "SELECT id, chain_id, payer, payee, amount, token_address, fee_amount, status, on_chain_id, tx_hash, dispute_reason, created_at, updated_at FROM escrows ORDER BY created_at DESC LIMIT $1 OFFSET $2"
        )
        .bind(limit)
        .bind(offset)
        .fetch_all(&state.db)
        .await
    }
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let total: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM escrows").fetch_one(&state.db).await.unwrap_or(0);

    Ok(Json(EscrowListResponse { items, total }))
}

async fn get_escrow(
    State(state): State<AppState>,
    AxPath(id): AxPath<String>,
) -> Result<Json<EscrowRecord>, StatusCode> {
    let escrow: EscrowRecord = sqlx::query_as::<_, EscrowRecord>(
        "SELECT id, chain_id, payer, payee, amount, token_address, fee_amount, status, on_chain_id, tx_hash, dispute_reason, created_at, updated_at FROM escrows WHERE id = $1"
    )
    .bind(&id)
    .fetch_optional(&state.db)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    .ok_or(StatusCode::NOT_FOUND)?;
    Ok(Json(escrow))
}

async fn release_escrow(
    State(state): State<AppState>,
    AxPath(id): AxPath<String>,
    Json(_req): Json<ReleaseEscrowRequest>,
) -> Result<Json<EscrowRecord>, StatusCode> {
    sqlx::query("UPDATE escrows SET status = 'released', updated_at = NOW() WHERE id = $1 AND status = 'active'")
        .bind(&id)
        .execute(&state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    get_escrow(State(state), AxPath(id)).await
}

async fn refund_escrow(
    State(state): State<AppState>,
    AxPath(id): AxPath<String>,
    Json(req): Json<RefundEscrowRequest>,
) -> Result<Json<EscrowRecord>, StatusCode> {
    sqlx::query("UPDATE escrows SET status = 'refunded', dispute_reason = $1, updated_at = NOW() WHERE id = $2 AND status IN ('active', 'disputed')")
        .bind(&req.reason)
        .bind(&id)
        .execute(&state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    get_escrow(State(state), AxPath(id)).await
}

async fn dispute_escrow(
    State(state): State<AppState>,
    AxPath(id): AxPath<String>,
    Json(req): Json<DisputeEscrowRequest>,
) -> Result<Json<EscrowRecord>, StatusCode> {
    sqlx::query("UPDATE escrows SET status = 'disputed', dispute_reason = $1, updated_at = NOW() WHERE id = $2 AND status = 'active'")
        .bind(&req.reason)
        .bind(&id)
        .execute(&state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    get_escrow(State(state), AxPath(id)).await
}

async fn resolve_dispute(
    State(state): State<AppState>,
    AxPath(id): AxPath<String>,
    Json(req): Json<ResolveDisputeRequest>,
) -> Result<Json<EscrowRecord>, StatusCode> {
    let new_status = if req.to_payee { "released" } else { "refunded" };
    sqlx::query("UPDATE escrows SET status = $1, updated_at = NOW() WHERE id = $2 AND status = 'disputed'")
        .bind(new_status)
        .bind(&id)
        .execute(&state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    get_escrow(State(state), AxPath(id)).await
}

async fn confirm_escrow_deposit(
    State(state): State<AppState>,
    AxPath(id): AxPath<String>,
    Json(req): Json<serde_json::Value>,
) -> Result<Json<EscrowRecord>, StatusCode> {
    let on_chain_id = req.get("on_chain_id").and_then(|v| v.as_str()).unwrap_or_default();
    let tx_hash = req.get("tx_hash").and_then(|v| v.as_str()).unwrap_or_default();

    sqlx::query("UPDATE escrows SET on_chain_id = $1, tx_hash = $2, updated_at = NOW() WHERE id = $3")
        .bind(on_chain_id)
        .bind(tx_hash)
        .bind(&id)
        .execute(&state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    get_escrow(State(state), AxPath(id)).await
}

fn compute_fee(amount: &str) -> String {
    use std::str::FromStr;
    if let Ok(amt) = alloy::primitives::U256::from_str(amount) {
        // 0.3% fee
        let fee = amt / alloy::primitives::U256::from(333u64);
        fee.to_string()
    } else {
        "0".to_string()
    }
}
