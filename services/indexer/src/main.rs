use alloy::primitives::{Address, B256};
use axum::{
    extract::{Path as AxPath, State},
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use clap::{Parser, ValueEnum};
use epsx_indexer::{build_auth_verifier, protect_router, verify_schema_compatibility};
use serde::Serialize;
use sqlx::FromRow;
use std::{net::SocketAddr, str::FromStr};
use tracing::info;

#[derive(Parser)]
#[command(name = "epsx-indexer", about = "EPSX Blockchain Indexer Service")]
struct Args {
    #[arg(long, default_value = "8108")]
    port: u16,
    #[arg(long, default_value = "0.0.0.0")]
    host: String,
    #[arg(
        long,
        default_value = "postgres://epsx:epsx@localhost:5432/epsx_indexer"
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
struct BlockRecord {
    chain_id: String,
    number: i64,
    hash: String,
    parent_hash: String,
    timestamp: chrono::DateTime<chrono::Utc>,
    miner: Option<String>,
    gas_used: i64,
    gas_limit: i64,
    tx_count: i32,
}

#[derive(Serialize, FromRow)]
struct TxRecord {
    chain_id: String,
    hash: String,
    from_address: String,
    to_address: Option<String>,
    value: String,
    block_number: i64,
    status: Option<i32>,
    timestamp: chrono::DateTime<chrono::Utc>,
}

#[derive(Serialize)]
struct ChainStatus {
    chain_id: String,
    name: String,
    indexer_block: u64,
    healthy: bool,
    degraded_reason: String,
}

#[derive(Serialize, FromRow)]
struct TokenTransfer {
    chain_id: String,
    tx_hash: String,
    log_index: i32,
    token_address: String,
    from_address: String,
    to_address: String,
    value: String,
    block_number: i64,
    timestamp: chrono::DateTime<chrono::Utc>,
}

#[tokio::main]
async fn main() {
    epsx_observability::Observability::init("indexer");
    let args = Args::parse();

    let production = matches!(args.environment, Environment::Production);
    let jwks_url = args.jwks_url.unwrap_or_else(|| {
        format!(
            "{}/.well-known/jwks.json",
            args.oidc_issuer.trim_end_matches('/')
        )
    });
    let verifier = build_auth_verifier(&args.oidc_issuer, &jwks_url, production)
        .expect("indexer OIDC configuration must be valid");

    let db = sqlx::PgPool::connect(&args.database_url)
        .await
        .expect("Failed to connect to database");
    verify_schema_compatibility(&db)
        .await
        .expect("indexer schema must be compatible; run the reviewed indexer migration first");

    // A12 has not supplied canonical ingestion, a durable checkpoint lease,
    // finality, or replay rules. Startup deliberately creates no provider and
    // launches no background sync task. Only the fail-closed HTTP shell is
    // served after the exact read-only schema probe succeeds.
    let state = AppState { db };
    let app = Router::new()
        .route("/health", get(health))
        .route("/api/v1/indexer/status/{chain}", get(get_chain_status))
        .route("/api/v1/indexer/block/{chain}/{number}", get(get_block))
        .route("/api/v1/indexer/tx/{chain}/{hash}", get(get_transaction))
        .route(
            "/api/v1/indexer/transfers/{chain}/{address}",
            get(get_address_transfers),
        )
        .route("/api/v1/indexer/sync", post(sync_unavailable))
        .with_state(state);
    let app = protect_router(app, verifier);

    let addr: SocketAddr = format!("{}:{}", args.host, args.port).parse().unwrap();
    info!("Indexer service listening on {}", addr);
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn health() -> StatusCode {
    StatusCode::OK
}

fn canonical_chain_id(value: &str) -> Result<String, StatusCode> {
    let parsed = value.parse::<u64>().map_err(|_| StatusCode::BAD_REQUEST)?;
    let canonical = parsed.to_string();
    if parsed == 0 || canonical != value || canonical.len() > 10 {
        return Err(StatusCode::BAD_REQUEST);
    }
    Ok(canonical)
}

fn canonical_hash(value: &str) -> Result<String, StatusCode> {
    B256::from_str(value)
        .map(|hash| format!("{hash:#x}"))
        .map_err(|_| StatusCode::BAD_REQUEST)
}

fn canonical_address(value: &str) -> Result<String, StatusCode> {
    Address::from_str(value)
        .map(|address| format!("{address:#x}"))
        .map_err(|_| StatusCode::BAD_REQUEST)
}

fn checked_block_number(value: i64) -> Result<i64, StatusCode> {
    if value < 0 {
        return Err(StatusCode::BAD_REQUEST);
    }
    Ok(value)
}

async fn get_chain_status(
    State(state): State<AppState>,
    AxPath(chain_id): AxPath<String>,
) -> Result<Json<ChainStatus>, StatusCode> {
    let chain_id = canonical_chain_id(&chain_id)?;
    let indexed: Option<i64> =
        sqlx::query_scalar("SELECT MAX(number) FROM public.blocks WHERE chain_id = $1")
            .bind(&chain_id)
            .fetch_one(&state.db)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let indexer_block = indexed
        .map(u64::try_from)
        .transpose()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .unwrap_or(0);
    let chain_num = chain_id
        .parse::<u64>()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(ChainStatus {
        chain_id,
        name: epsx_kernel::ChainId(chain_num).name().to_string(),
        indexer_block,
        healthy: false,
        degraded_reason: "canonical ingestion and finality are unavailable".to_string(),
    }))
}

async fn get_block(
    State(state): State<AppState>,
    AxPath((chain_id, number)): AxPath<(String, i64)>,
) -> Result<Json<BlockRecord>, StatusCode> {
    let chain_id = canonical_chain_id(&chain_id)?;
    let number = checked_block_number(number)?;
    let block: BlockRecord = sqlx::query_as::<_, BlockRecord>(
        "SELECT chain_id, number, hash, parent_hash, timestamp, miner, gas_used, gas_limit, tx_count
         FROM public.blocks
         WHERE chain_id = $1 AND number = $2",
    )
    .bind(&chain_id)
    .bind(number)
    .fetch_optional(&state.db)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    .ok_or(StatusCode::NOT_FOUND)?;
    Ok(Json(block))
}

async fn get_transaction(
    State(state): State<AppState>,
    AxPath((chain_id, hash)): AxPath<(String, String)>,
) -> Result<Json<TxRecord>, StatusCode> {
    let chain_id = canonical_chain_id(&chain_id)?;
    let hash = canonical_hash(&hash)?;
    let tx: TxRecord = sqlx::query_as::<_, TxRecord>(
        "SELECT chain_id, hash, from_address, to_address, value, block_number, status, timestamp
         FROM public.transactions
         WHERE chain_id = $1 AND hash = $2",
    )
    .bind(&chain_id)
    .bind(&hash)
    .fetch_optional(&state.db)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    .ok_or(StatusCode::NOT_FOUND)?;
    Ok(Json(tx))
}

async fn get_address_transfers(
    State(state): State<AppState>,
    AxPath((chain_id, address)): AxPath<(String, String)>,
) -> Result<Json<Vec<TokenTransfer>>, StatusCode> {
    let chain_id = canonical_chain_id(&chain_id)?;
    let address = canonical_address(&address)?;
    let transfers: Vec<TokenTransfer> = sqlx::query_as::<_, TokenTransfer>(
        "SELECT chain_id, tx_hash, log_index, token_address, from_address, to_address, value, block_number, timestamp
         FROM public.token_transfers
         WHERE chain_id = $1 AND (from_address = $2 OR to_address = $2)
         ORDER BY block_number DESC, tx_hash DESC, log_index DESC
         LIMIT 100",
    )
    .bind(&chain_id)
    .bind(&address)
    .fetch_all(&state.db)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(transfers))
}

async fn sync_unavailable() -> StatusCode {
    StatusCode::NOT_FOUND
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn chain_ids_are_canonical_and_fit_the_schema() {
        assert_eq!(canonical_chain_id("56"), Ok("56".to_string()));
        assert_eq!(canonical_chain_id("42161"), Ok("42161".to_string()));
        for invalid in ["", "0", "056", "+56", "-1", "10000000000", "bsc"] {
            assert_eq!(canonical_chain_id(invalid), Err(StatusCode::BAD_REQUEST));
        }
    }

    #[test]
    fn hashes_and_addresses_are_parsed_not_shape_only_lowercased() {
        let hash = format!("0x{}", "ab".repeat(32));
        assert_eq!(canonical_hash(&hash), Ok(hash));
        assert_eq!(canonical_hash("0xnot-a-hash"), Err(StatusCode::BAD_REQUEST));

        let mixed = "0x8AC76a51cc950d9822D68b83fE1Ad97B32Cd580d";
        assert_eq!(
            canonical_address(mixed),
            Ok("0x8ac76a51cc950d9822d68b83fe1ad97b32cd580d".to_string())
        );
        assert_eq!(canonical_address("0xinvalid"), Err(StatusCode::BAD_REQUEST));
    }

    #[test]
    fn negative_database_block_numbers_never_wrap_unsigned() {
        assert_eq!(checked_block_number(0), Ok(0));
        assert_eq!(checked_block_number(i64::MAX), Ok(i64::MAX));
        assert_eq!(checked_block_number(-1), Err(StatusCode::BAD_REQUEST));
        assert!(u64::try_from(-1_i64).is_err());
    }

    #[test]
    fn timestamptz_and_nullable_models_keep_the_wire_shape() {
        let timestamp = chrono::DateTime::parse_from_rfc3339("2026-07-22T12:34:56Z")
            .unwrap()
            .with_timezone(&chrono::Utc);
        let block = BlockRecord {
            chain_id: "56".to_string(),
            number: 1,
            hash: format!("0x{}", "11".repeat(32)),
            parent_hash: format!("0x{}", "00".repeat(32)),
            timestamp,
            miner: None,
            gas_used: 0,
            gas_limit: 30_000_000,
            tx_count: 0,
        };
        let value = serde_json::to_value(block).unwrap();
        assert_eq!(value["timestamp"], "2026-07-22T12:34:56Z");
        assert!(value["miner"].is_null());
    }
}
