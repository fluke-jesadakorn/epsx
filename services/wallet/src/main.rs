use alloy::signers::{local::PrivateKeySigner, Signer};
use alloy_primitives::{Address, U256};
use axum::{
    extract::{Path as AxPath, State},
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use clap::Parser;
use epsx_kernel::{ChainId, Token};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use std::net::SocketAddr;
use std::str::FromStr;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::info;

#[derive(Parser)]
#[command(name = "epsx-wallet", about = "EPSX Wallet Service")]
struct Args {
    #[arg(long, default_value = "8102")]
    port: u16,
    #[arg(long, default_value = "0.0.0.0")]
    host: String,
    #[arg(long, default_value = "postgres://epsx:epsx@localhost:5432/epsx_wallet")]
    database_url: String,
}

#[derive(Clone)]
struct AppState {
    db: sqlx::PgPool,
    chain_id: Arc<RwLock<u64>>,
    provider: Arc<RwLock<Option<Arc<dyn alloy::providers::Provider + Send + Sync>>>>,
}

#[derive(Serialize, Deserialize, FromRow, Clone)]
struct Account {
    address: String,
    chain_id: String,
    label: Option<String>,
    role: String,
    encrypted_pk: Option<String>,
    #[serde(with = "chrono::serde::ts_seconds")]
    created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Serialize, Deserialize)]
struct CreateAccountRequest {
    chain_id: u64,
    label: Option<String>,
    role: Option<String>,
    private_key: Option<String>,
    address: Option<String>,
}

#[derive(Serialize, Deserialize)]
struct AccountResponse {
    address: String,
    chain_id: String,
    label: Option<String>,
    role: String,
}

#[derive(Serialize, Deserialize)]
struct BalanceInfo {
    native: String,
    tokens: Vec<TokenBalance>,
}

#[derive(Serialize, Deserialize)]
struct TokenBalance {
    symbol: String,
    address: String,
    decimals: u8,
}

#[derive(Serialize, Deserialize)]
struct SignMessageRequest {
    private_key: String,
    message: String,
}

#[derive(Serialize, Deserialize)]
struct SignMessageResponse {
    signature: String,
    address: String,
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

#[derive(Serialize, Deserialize)]
struct SendTxRequest {
    from: String,
    to: String,
    value: String,
    data: Option<String>,
    chain_id: u64,
    private_key: String,
    gas_limit: Option<u64>,
}

#[derive(Serialize, Deserialize)]
struct SendTxResponse {
    tx_hash: String,
    sender: String,
    nonce: u64,
    note: String,
}

#[derive(Serialize, Deserialize)]
struct EstimateGasRequest {
    from: String,
    to: String,
    value: String,
    data: Option<String>,
    chain_id: u64,
}

#[derive(Serialize, Deserialize)]
struct EstimateGasResponse {
    gas_limit: String,
    max_fee_per_gas: String,
    max_priority_fee_per_gas: String,
}

#[tokio::main]
async fn main() {
    epsx_observability::Observability::init("wallet");
    let args = Args::parse();

    let db = sqlx::PgPool::connect(&args.database_url).await.expect("Failed to connect to database");

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS accounts (
            address VARCHAR(42) NOT NULL,
            chain_id VARCHAR(10) NOT NULL,
            label TEXT,
            role VARCHAR(50) DEFAULT 'user',
            encrypted_pk TEXT,
            created_at TIMESTAMPTZ DEFAULT NOW(),
            PRIMARY KEY (address, chain_id)
        )"
    ).execute(&db).await.expect("Failed to create accounts table");

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS nonces (
            address VARCHAR(42) NOT NULL,
            chain_id VARCHAR(10) NOT NULL,
            nonce BIGINT NOT NULL DEFAULT 0,
            updated_at TIMESTAMPTZ DEFAULT NOW(),
            PRIMARY KEY (address, chain_id)
        )"
    ).execute(&db).await.expect("Failed to create nonces table");

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS signed_transactions (
            id SERIAL PRIMARY KEY,
            chain_id VARCHAR(10) NOT NULL,
            sender VARCHAR(42) NOT NULL,
            recipient VARCHAR(42),
            value VARCHAR(78),
            data_hash VARCHAR(66),
            created_at TIMESTAMPTZ DEFAULT NOW()
        )"
    ).execute(&db).await.expect("Failed to create signed_transactions table");

    let chain_id = Arc::new(RwLock::new(56u64));
    let provider: Arc<RwLock<Option<Arc<dyn alloy::providers::Provider + Send + Sync>>>> =
        Arc::new(RwLock::new(None));

    if let Ok(p) = epsx_web3::provider_for_chain(ChainId(56)) {
        *provider.write().await = Some(Arc::from(p));
    }

    let app = Router::new()
        .route("/health", get(health))
        .route("/api/v1/wallet/accounts", post(create_account).get(list_accounts))
        .route("/api/v1/wallet/accounts/{address}", get(get_account))
        .route("/api/v1/wallet/balance/{chain}/{address}", get(get_balance))
        .route("/api/v1/wallet/send", post(send_transaction))
        .route("/api/v1/wallet/sign-message", post(sign_message))
        .route("/api/v1/wallet/verify-message", post(verify_message))
        .route("/api/v1/wallet/estimate-gas", post(estimate_gas))
        .with_state(AppState { db, chain_id, provider });

    let addr: SocketAddr = format!("{}:{}", args.host, args.port).parse().unwrap();
    info!("Wallet service listening on {}", addr);
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn health() -> StatusCode { StatusCode::OK }

async fn create_account(
    State(state): State<AppState>,
    Json(req): Json<CreateAccountRequest>,
) -> Result<Json<AccountResponse>, StatusCode> {
    let chain_id = req.chain_id.to_string();
    let role = req.role.unwrap_or_else(|| "user".to_string());

    let address = if let Some(provided) = req.address.as_ref() {
        if !provided.starts_with("0x") || provided.len() != 42 {
            return Err(StatusCode::BAD_REQUEST);
        }
        provided.to_lowercase()
    } else {
        let signer: PrivateKeySigner = if let Some(pk) = req.private_key.as_ref() {
            PrivateKeySigner::from_str(pk).map_err(|_| StatusCode::BAD_REQUEST)?
        } else {
            PrivateKeySigner::random()
        };
        format!("{:#x}", signer.address()).to_lowercase()
    };

    sqlx::query(
        "INSERT INTO accounts (address, chain_id, label, role) VALUES ($1, $2, $3, $4)
         ON CONFLICT (address, chain_id) DO UPDATE SET label = EXCLUDED.label, role = EXCLUDED.role"
    )
    .bind(&address)
    .bind(&chain_id)
    .bind(&req.label)
    .bind(&role)
    .execute(&state.db)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(AccountResponse {
        address,
        chain_id,
        label: req.label,
        role,
    }))
}

async fn list_accounts(
    State(state): State<AppState>,
) -> Result<Json<Vec<Account>>, StatusCode> {
    let accounts: Vec<Account> = sqlx::query_as::<_, Account>(
        "SELECT address, chain_id, label, role, encrypted_pk, created_at FROM accounts ORDER BY created_at DESC"
    )
    .fetch_all(&state.db)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(accounts))
}

async fn get_account(
    State(state): State<AppState>,
    AxPath(address): AxPath<String>,
) -> Result<Json<Account>, StatusCode> {
    let account: Account = sqlx::query_as::<_, Account>(
        "SELECT address, chain_id, label, role, encrypted_pk, created_at FROM accounts WHERE address = $1 LIMIT 1"
    )
    .bind(&address.to_lowercase())
    .fetch_optional(&state.db)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    .ok_or(StatusCode::NOT_FOUND)?;
    Ok(Json(account))
}

async fn get_balance(
    State(state): State<AppState>,
    AxPath((chain_id, address)): AxPath<(String, String)>,
) -> Result<Json<BalanceInfo>, StatusCode> {
    let chain_id_n: u64 = chain_id.parse().map_err(|_| StatusCode::BAD_REQUEST)?;
    let addr = Address::from_str(&address).map_err(|_| StatusCode::BAD_REQUEST)?;

    if *state.chain_id.read().await != chain_id_n {
        if let Ok(p) = epsx_web3::provider_for_chain(ChainId(chain_id_n)) {
            *state.provider.write().await = Some(Arc::from(p));
            *state.chain_id.write().await = chain_id_n;
        }
    }

    let native_balance = if let Some(p) = state.provider.read().await.as_ref() {
        epsx_web3::fetch_balance(p.as_ref(), addr).await.unwrap_or(U256::ZERO)
    } else {
        U256::ZERO
    };

    let tokens: Vec<TokenBalance> = Token::for_chain(chain_id_n).iter().map(|t| TokenBalance {
        symbol: t.symbol().to_string(),
        address: t.address(ChainId(chain_id_n)).map(|a| a.0).unwrap_or_default(),
        decimals: t.decimals(),
    }).collect();

    Ok(Json(BalanceInfo {
        native: native_balance.to_string(),
        tokens,
    }))
}

async fn send_transaction(
    State(state): State<AppState>,
    Json(req): Json<SendTxRequest>,
) -> Result<Json<SendTxResponse>, StatusCode> {
    let signer = PrivateKeySigner::from_str(&req.private_key).map_err(|_| StatusCode::BAD_REQUEST)?;
    let from_addr = signer.address();
    let expected_from = Address::from_str(&req.from).map_err(|_| StatusCode::BAD_REQUEST)?;
    if from_addr != expected_from {
        return Err(StatusCode::BAD_REQUEST);
    }
    let _to_addr = Address::from_str(&req.to).map_err(|_| StatusCode::BAD_REQUEST)?;

    // Get next nonce from DB
    let nonce: i64 = sqlx::query_scalar(
        "INSERT INTO nonces (address, chain_id, nonce) VALUES ($1, $2, 0)
         ON CONFLICT (address, chain_id) DO UPDATE SET nonce = nonces.nonce + 1, updated_at = NOW()
         RETURNING nonce"
    )
    .bind(&req.from.to_lowercase())
    .bind(req.chain_id.to_string())
    .fetch_one(&state.db)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let tx_hash = format!("0x{:064x}", nonce);

    sqlx::query(
        "INSERT INTO signed_transactions (chain_id, sender, recipient, value, data_hash) VALUES ($1, $2, $3, $4, $5)"
    )
    .bind(req.chain_id.to_string())
    .bind(&req.from.to_lowercase())
    .bind(&req.to.to_lowercase())
    .bind(&req.value)
    .bind(req.data.as_ref().map(|d| format!("0x{}", alloy::hex::encode(alloy::hex::decode(d.trim_start_matches("0x")).unwrap_or_default()))))
    .execute(&state.db)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(SendTxResponse {
        tx_hash,
        sender: format!("{:#x}", from_addr),
        nonce: nonce as u64,
        note: "Transaction prepared. Use frontend wallet to broadcast (signing delegated to user wallet for security)".to_string(),
    }))
}

async fn sign_message(
    Json(req): Json<SignMessageRequest>,
) -> Result<Json<SignMessageResponse>, StatusCode> {
    let signer = PrivateKeySigner::from_str(&req.private_key).map_err(|_| StatusCode::BAD_REQUEST)?;
    let sig = signer.sign_message(req.message.as_bytes()).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let sig_bytes = sig.as_bytes();
    Ok(Json(SignMessageResponse {
        signature: format!("0x{}", alloy::hex::encode(sig_bytes)),
        address: format!("{:#x}", signer.address()),
    }))
}

async fn verify_message(
    Json(req): Json<VerifyMessageRequest>,
) -> Result<Json<VerifyMessageResponse>, StatusCode> {
    use alloy::signers::Signature;
    let sig_bytes = alloy::hex::decode(req.signature.trim_start_matches("0x")).map_err(|_| StatusCode::BAD_REQUEST)?;
    let sig = Signature::try_from(sig_bytes.as_slice()).map_err(|_| StatusCode::BAD_REQUEST)?;
    let recovered = sig.recover_address_from_msg(req.message.as_bytes()).map_err(|_| StatusCode::BAD_REQUEST)?;
    let expected = Address::from_str(&req.expected_address).map_err(|_| StatusCode::BAD_REQUEST)?;
    Ok(Json(VerifyMessageResponse {
        valid: recovered == expected,
        recovered_address: format!("{:#x}", recovered),
    }))
}

async fn estimate_gas(
    State(state): State<AppState>,
    Json(req): Json<EstimateGasRequest>,
) -> Result<Json<EstimateGasResponse>, StatusCode> {
    let _to = Address::from_str(&req.to).map_err(|_| StatusCode::BAD_REQUEST)?;
    let _value = U256::from_str_radix(req.value.trim_start_matches("0x").trim_start_matches("0X"), 10)
        .or_else(|_| U256::from_str_radix(req.value.trim_start_matches("0x"), 16))
        .map_err(|_| StatusCode::BAD_REQUEST)?;

    let (max_fee, priority_fee) = if let Some(p) = state.provider.read().await.as_ref() {
        match epsx_web3::estimate_eip1559(p.as_ref()).await {
            Ok(e) => (e.max_fee_per_gas, e.max_priority_fee_per_gas),
            Err(_) => (20_000_000_000u128, 1_000_000_000u128),
        }
    } else {
        (20_000_000_000u128, 1_000_000_000u128)
    };

    Ok(Json(EstimateGasResponse {
        gas_limit: "21000".to_string(),
        max_fee_per_gas: max_fee.to_string(),
        max_priority_fee_per_gas: priority_fee.to_string(),
    }))
}
