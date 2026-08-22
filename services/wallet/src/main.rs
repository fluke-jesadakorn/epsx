use alloy::signers::{local::PrivateKeySigner, Signer};
use alloy_primitives::{Address, U256};
use axum::{
    extract::{Extension, Path as AxPath, State},
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use clap::{Parser, ValueEnum};
use epsx_kernel::{ChainId, Token};
use epsx_service_auth::VerifiedPrincipal;
use epsx_wallet::{
    build_auth_verifier, canonical_owner, protect_router, verify_schema_compatibility,
};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use std::net::SocketAddr;
use std::str::FromStr;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::info;

mod commerce;

#[derive(Parser)]
#[command(name = "epsx-wallet", about = "EPSX Wallet Service")]
struct Args {
    #[arg(long, default_value = "8102")]
    port: u16,
    #[arg(long, default_value = "0.0.0.0")]
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
    chain_id: Arc<RwLock<u64>>,
    provider: Arc<RwLock<Option<Arc<dyn alloy::providers::Provider + Send + Sync>>>>,
}

#[derive(Serialize, Deserialize)]
struct CreateAccountRequest {
    chain_id: u64,
    label: Option<String>,
    role: Option<String>,
    private_key: Option<String>,
    address: Option<String>,
}

#[derive(Serialize, Deserialize, FromRow)]
struct AccountResponse {
    address: String,
    chain_id: String,
    label: Option<String>,
    role: Option<String>,
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

    let chain_id = Arc::new(RwLock::new(56u64));
    let provider: Arc<RwLock<Option<Arc<dyn alloy::providers::Provider + Send + Sync>>>> =
        Arc::new(RwLock::new(None));

    if let Ok(p) = epsx_web3::provider_for_chain(ChainId(56)) {
        *provider.write().await = Some(Arc::from(p));
    }

    let app = Router::new()
        .route("/health", get(health))
        .route(
            "/api/v1/wallet/accounts",
            post(create_account).get(list_accounts),
        )
        .route("/api/v1/wallet/accounts/{address}", get(get_account))
        .route("/api/v1/wallet/balance/{chain}/{address}", get(get_balance))
        .route("/api/v1/wallet/send", post(send_transaction))
        .route("/api/v1/wallet/sign-message", post(sign_message))
        .route("/api/v1/wallet/verify-message", post(verify_message))
        .route("/api/v1/wallet/estimate-gas", post(estimate_gas))
        .route("/api/v1/admin/wallets", get(commerce::list_admin_wallets))
        .route(
            "/api/v1/admin/wallets/stats",
            get(commerce::admin_wallet_stats),
        )
        .route(
            "/api/v1/admin/wallets/{address}",
            get(commerce::get_admin_wallet),
        )
        .route(
            "/api/v1/admin/wallets/{address}/disable",
            post(commerce::disable_admin_wallet),
        )
        .route(
            "/api/v1/admin/wallets/{address}/enable",
            post(commerce::enable_admin_wallet),
        )
        .route(
            "/api/v1/admin/wallets/{address}/metadata",
            axum::routing::patch(commerce::update_admin_wallet_metadata),
        )
        .route("/api/v1/admin/credits", get(commerce::admin_credit_stats))
        .route(
            "/api/v1/admin/credits/{address}",
            get(commerce::get_admin_credits),
        )
        .route(
            "/api/v1/admin/credits/{address}/grant",
            post(commerce::grant_admin_credits),
        )
        .route(
            "/api/v1/admin/credits/{address}/revoke",
            post(commerce::revoke_admin_credits),
        )
        .with_state(AppState {
            db,
            chain_id,
            provider,
        });
    let app = protect_router(app, verifier);

    let addr: SocketAddr = format!("{}:{}", args.host, args.port).parse().unwrap();
    info!("Wallet service listening on {}", addr);
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn health() -> StatusCode {
    StatusCode::OK
}

async fn create_account(
    State(state): State<AppState>,
    Json(req): Json<CreateAccountRequest>,
) -> Result<Json<AccountResponse>, StatusCode> {
    let chain_id = database_chain_id(req.chain_id)?;
    let role = req.role.unwrap_or_else(|| "user".to_string());
    if role.chars().count() > 50 {
        return Err(StatusCode::BAD_REQUEST);
    }

    let address = if let Some(provided) = req.address.as_ref() {
        canonical_evm_address(provided)?.1
    } else {
        let signer: PrivateKeySigner = if let Some(pk) = req.private_key.as_ref() {
            PrivateKeySigner::from_str(pk).map_err(|_| StatusCode::BAD_REQUEST)?
        } else {
            PrivateKeySigner::random()
        };
        canonical_address(signer.address())
    };

    sqlx::query(
        "INSERT INTO public.accounts (address, chain_id, label, role) VALUES ($1, $2, $3, $4)
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
        role: Some(role),
    }))
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
        epsx_web3::fetch_balance(p.as_ref(), addr)
            .await
            .unwrap_or(U256::ZERO)
    } else {
        U256::ZERO
    };

    let tokens: Vec<TokenBalance> = Token::for_chain(chain_id_n)
        .iter()
        .map(|t| TokenBalance {
            symbol: t.symbol().to_string(),
            address: t
                .address(ChainId(chain_id_n))
                .map(|a| a.0)
                .unwrap_or_default(),
            decimals: t.decimals(),
        })
        .collect();

    Ok(Json(BalanceInfo {
        native: native_balance.to_string(),
        tokens,
    }))
}

async fn send_transaction(
    State(state): State<AppState>,
    Json(req): Json<SendTxRequest>,
) -> Result<Json<SendTxResponse>, StatusCode> {
    let signer =
        PrivateKeySigner::from_str(&req.private_key).map_err(|_| StatusCode::BAD_REQUEST)?;
    let from_addr = signer.address();
    let (expected_from, sender) = canonical_evm_address(&req.from)?;
    if from_addr != expected_from {
        return Err(StatusCode::BAD_REQUEST);
    }
    let (_, recipient) = canonical_evm_address(&req.to)?;

    let chain_id = database_chain_id(req.chain_id)?;
    let value = canonical_transaction_value(&req.value)?;
    let data_hash = normalize_data_hash(req.data.as_deref())?;
    let mut transaction = state
        .db
        .begin()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let nonce: i64 = sqlx::query_scalar(
        "INSERT INTO public.nonces AS wallet_nonces (address, chain_id, nonce) VALUES ($1, $2, 0)
         ON CONFLICT (address, chain_id) DO UPDATE SET nonce = wallet_nonces.nonce + 1, updated_at = NOW()
         RETURNING nonce",
    )
    .bind(&sender)
    .bind(&chain_id)
    .fetch_one(&mut *transaction)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let response_nonce = u64::try_from(nonce).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let tx_hash = format!("0x{:064x}", response_nonce);

    sqlx::query(
        "INSERT INTO public.signed_transactions (chain_id, sender, recipient, value, data_hash) VALUES ($1, $2, $3, $4, $5)"
    )
    .bind(&chain_id)
    .bind(&sender)
    .bind(&recipient)
    .bind(&value)
    .bind(data_hash)
    .execute(&mut *transaction)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    transaction
        .commit()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(SendTxResponse {
        tx_hash,
        sender,
        nonce: response_nonce,
        note: "Transaction prepared. Use frontend wallet to broadcast (signing delegated to user wallet for security)".to_string(),
    }))
}

fn canonical_evm_address(value: &str) -> Result<(Address, String), StatusCode> {
    if value.len() != 42 || !(value.starts_with("0x") || value.starts_with("0X")) {
        return Err(StatusCode::BAD_REQUEST);
    }
    let address = Address::from_str(value).map_err(|_| StatusCode::BAD_REQUEST)?;
    let canonical = canonical_address(address);
    if canonical.len() != 42
        || !canonical.starts_with("0x")
        || !canonical[2..]
            .bytes()
            .all(|byte| byte.is_ascii_digit() || matches!(byte, b'a'..=b'f'))
    {
        return Err(StatusCode::BAD_REQUEST);
    }
    Ok((address, canonical))
}

fn canonical_address(address: Address) -> String {
    format!("{address:#x}").to_ascii_lowercase()
}

fn canonical_transaction_value(value: &str) -> Result<String, StatusCode> {
    if value.is_empty() {
        return Err(StatusCode::BAD_REQUEST);
    }
    let parsed = if let Some(hex) = value
        .strip_prefix("0x")
        .or_else(|| value.strip_prefix("0X"))
    {
        if hex.is_empty() || !hex.bytes().all(|byte| byte.is_ascii_hexdigit()) {
            return Err(StatusCode::BAD_REQUEST);
        }
        U256::from_str_radix(hex, 16).map_err(|_| StatusCode::BAD_REQUEST)?
    } else {
        if !value.bytes().all(|byte| byte.is_ascii_digit()) {
            return Err(StatusCode::BAD_REQUEST);
        }
        U256::from_str_radix(value, 10).map_err(|_| StatusCode::BAD_REQUEST)?
    };
    Ok(parsed.to_string())
}

fn database_chain_id(chain_id: u64) -> Result<String, StatusCode> {
    let chain_id = chain_id.to_string();
    if chain_id.len() > 10 {
        return Err(StatusCode::BAD_REQUEST);
    }
    Ok(chain_id)
}

fn normalize_data_hash(data: Option<&str>) -> Result<Option<String>, StatusCode> {
    let Some(data) = data else {
        return Ok(None);
    };
    let bytes =
        alloy::hex::decode(data.trim_start_matches("0x")).map_err(|_| StatusCode::BAD_REQUEST)?;
    if bytes.len() > 32 {
        return Err(StatusCode::BAD_REQUEST);
    }
    Ok(Some(format!("0x{}", alloy::hex::encode(bytes))))
}

async fn sign_message(
    Json(req): Json<SignMessageRequest>,
) -> Result<Json<SignMessageResponse>, StatusCode> {
    let signer =
        PrivateKeySigner::from_str(&req.private_key).map_err(|_| StatusCode::BAD_REQUEST)?;
    let sig = signer
        .sign_message(req.message.as_bytes())
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
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

async fn estimate_gas(
    State(state): State<AppState>,
    Json(req): Json<EstimateGasRequest>,
) -> Result<Json<EstimateGasResponse>, StatusCode> {
    let _to = Address::from_str(&req.to).map_err(|_| StatusCode::BAD_REQUEST)?;
    let _value = U256::from_str_radix(
        req.value.trim_start_matches("0x").trim_start_matches("0X"),
        10,
    )
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

#[cfg(test)]
mod schema_bind_tests {
    use super::*;

    #[test]
    fn database_chain_ids_fit_the_legacy_varchar_boundary() {
        assert_eq!(database_chain_id(56).unwrap(), "56");
        assert_eq!(database_chain_id(9_999_999_999).unwrap(), "9999999999");
        assert_eq!(
            database_chain_id(10_000_000_000),
            Err(StatusCode::BAD_REQUEST)
        );
    }

    #[test]
    fn transaction_data_fits_the_legacy_hash_boundary_and_rejects_bad_hex() {
        assert_eq!(normalize_data_hash(None).unwrap(), None);
        assert_eq!(
            normalize_data_hash(Some("0xAa00")).unwrap(),
            Some("0xaa00".to_string())
        );
        assert_eq!(
            normalize_data_hash(Some("xyz")),
            Err(StatusCode::BAD_REQUEST)
        );
        assert_eq!(
            normalize_data_hash(Some(&format!("0x{}", "aa".repeat(33)))),
            Err(StatusCode::BAD_REQUEST)
        );
    }

    #[test]
    fn alloy_addresses_are_hex_parsed_and_canonicalized_before_database_use() {
        let mixed = "0x111111111111111111111111111111111111AaAa";
        let (_, canonical) = canonical_evm_address(mixed).unwrap();
        assert_eq!(canonical, "0x111111111111111111111111111111111111aaaa");
        assert_eq!(canonical.len(), 42);
        for invalid in [
            "",
            "0xabc",
            "1111111111111111111111111111111111111111",
            "0xgggggggggggggggggggggggggggggggggggggggg",
        ] {
            assert_eq!(canonical_evm_address(invalid), Err(StatusCode::BAD_REQUEST));
        }
    }

    #[test]
    fn transaction_values_are_u256_parsed_and_stored_as_canonical_decimal() {
        assert_eq!(canonical_transaction_value("0").unwrap(), "0");
        assert_eq!(canonical_transaction_value("00042").unwrap(), "42");
        assert_eq!(canonical_transaction_value("0x2a").unwrap(), "42");
        let max = "115792089237316195423570985008687907853269984665640564039457584007913129639935";
        assert_eq!(canonical_transaction_value(max).unwrap(), max);
        for invalid in [
            "",
            " ",
            "-1",
            "+1",
            "1.0",
            "0x",
            "0xzz",
            "115792089237316195423570985008687907853269984665640564039457584007913129639936",
            "0x10000000000000000000000000000000000000000000000000000000000000000",
        ] {
            assert_eq!(
                canonical_transaction_value(invalid),
                Err(StatusCode::BAD_REQUEST),
                "{invalid:?}"
            );
        }
    }
}
