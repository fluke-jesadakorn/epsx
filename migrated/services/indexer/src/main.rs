use alloy::providers::Provider;
use alloy::rpc::types::Filter;
use alloy::sol_types::SolEvent;
use axum::{
    extract::{Path as AxPath, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{any, get},
    Json, Router,
};
use clap::Parser;
use epsx_web3::IERC20Events::Transfer;
use serde::Serialize;
use sqlx::FromRow;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use tracing::{error, info, warn};

#[derive(Parser)]
#[command(name = "epsx-indexer", about = "EPSX Blockchain Indexer Service")]
struct Args {
    #[arg(long, default_value = "8108")]
    port: u16,
    #[arg(long, default_value = "0.0.0.0")]
    host: String,
    #[arg(long, default_value = "postgres://epsx:epsx@localhost:5432/epsx_indexer")]
    database_url: String,
    #[arg(long, default_value = "56", env = "EPSX_INDEXER_CHAIN_ID")]
    chain_id: u64,
    #[arg(long, default_value = "3", env = "EPSX_INDEXER_POLL")]
    poll_interval: u64,
    #[arg(long, default_value = "true", env = "EPSX_INDEXER_SYNC_ON_START")]
    sync_on_start: bool,
    #[arg(long, env = "BSC_RPC_URL")]
    bsc_rpc_url: Option<String>,
    #[arg(long, env = "BSC_TESTNET_RPC_URL")]
    bsc_testnet_rpc_url: Option<String>,
    #[arg(long, env = "EPSX_INDEXER_FROM_BLOCK", default_value = "0")]
    indexer_from_block: i64,
    #[arg(long, env = "EPSX_INDEXER_BATCH", default_value = "10")]
    batch_size: u64,
}

#[derive(Clone)]
struct AppState {
    db: sqlx::PgPool,
    chain_id: u64,
    provider: Arc<RwLock<Option<Arc<dyn Provider + Send + Sync>>>>,
    last_block: Arc<RwLock<u64>>,
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
    from_address: Option<String>,
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
    latest_block: u64,
    indexer_block: u64,
    lag: u64,
    healthy: bool,
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

    let db = sqlx::PgPool::connect(&args.database_url).await.expect("Failed to connect to database");

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS blocks (
            chain_id VARCHAR(10) NOT NULL,
            number BIGINT NOT NULL,
            hash VARCHAR(66) NOT NULL,
            parent_hash VARCHAR(66),
            timestamp TIMESTAMPTZ NOT NULL,
            miner VARCHAR(42),
            gas_used BIGINT,
            gas_limit BIGINT,
            tx_count INTEGER DEFAULT 0,
            PRIMARY KEY (chain_id, number)
        )"
    ).execute(&db).await.expect("Failed to create blocks table");

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS transactions (
            chain_id VARCHAR(10) NOT NULL,
            hash VARCHAR(66) PRIMARY KEY,
            from_address VARCHAR(42),
            to_address VARCHAR(42),
            value VARCHAR(78),
            block_number BIGINT,
            status INTEGER,
            timestamp TIMESTAMPTZ,
            input_data BYTEA
        )"
    ).execute(&db).await.expect("Failed to create transactions table");

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS token_transfers (
            chain_id VARCHAR(10) NOT NULL,
            tx_hash VARCHAR(66) NOT NULL,
            log_index INTEGER NOT NULL,
            token_address VARCHAR(42) NOT NULL,
            from_address VARCHAR(42) NOT NULL,
            to_address VARCHAR(42) NOT NULL,
            value VARCHAR(78) NOT NULL,
            block_number BIGINT NOT NULL,
            timestamp TIMESTAMPTZ NOT NULL,
            PRIMARY KEY (chain_id, tx_hash, log_index)
        )"
    ).execute(&db).await.expect("Failed to create token_transfers table");

    sqlx::query(
        "CREATE INDEX IF NOT EXISTS idx_blocks_timestamp ON blocks (chain_id, timestamp DESC)"
    ).execute(&db).await.expect("Failed to create blocks index");
    sqlx::query(
        "CREATE INDEX IF NOT EXISTS idx_transfers_token ON token_transfers (chain_id, token_address, timestamp DESC)"
    ).execute(&db).await.expect("Failed to create transfers index");

    let provider = Arc::new(RwLock::new(None));
    let last_block = Arc::new(RwLock::new(0u64));

    if let Some(url) = args.bsc_rpc_url.clone().or(args.bsc_testnet_rpc_url.clone()) {
        match epsx_web3::provider_for_url(&url) {
            Ok(p) => {
                *provider.write().await = Some(Arc::from(p));
                info!("Connected to RPC: {}", url);
            }
            Err(e) => warn!("Failed to create RPC provider: {} (will retry on sync)", e),
        }
    } else {
        let chain_id_enum = epsx_kernel::ChainId(args.chain_id);
        match epsx_web3::provider_for_chain(chain_id_enum) {
            Ok(p) => {
                *provider.write().await = Some(Arc::from(p));
                info!("Connected to default BSC provider for chain {}", args.chain_id);
            }
            Err(e) => warn!("Failed to create BSC provider: {} (will retry on sync)", e),
        }
    }

    if args.indexer_from_block > 0 {
        let from = args.indexer_from_block as u64;
        let db_max: Option<i64> = sqlx::query_scalar("SELECT MAX(number) FROM blocks WHERE chain_id = $1")
            .bind(args.chain_id.to_string())
            .fetch_optional(&db)
            .await
            .ok()
            .flatten();
        let resume = match db_max {
            Some(max) if max > 0 => std::cmp::max(max as u64, from.saturating_sub(1)),
            _ => from.saturating_sub(1),
        };
        *last_block.write().await = resume;
        info!("Resuming from block {} (env: {}, db max: {:?})", resume, from, db_max);
    }

    let sync_state = AppState {
        db: db.clone(),
        chain_id: args.chain_id,
        provider: provider.clone(),
        last_block: last_block.clone(),
    };

    if args.sync_on_start {
        let sync_state = sync_state.clone();
        let poll = args.poll_interval;
        let batch = args.batch_size;
        tokio::spawn(async move {
            sync_chain(sync_state, poll, batch).await;
        });
    }

    let app = Router::new()
        .route("/health", get(health))
        .route("/api/v1/indexer/status/{chain}", get(get_chain_status))
        .route("/api/v1/indexer/block/{chain}/{number}", get(get_block))
        .route("/api/v1/indexer/tx/{chain}/{hash}", get(get_transaction))
        .route("/api/v1/indexer/transfers/{chain}/{address}", get(get_address_transfers))
        .route("/api/v1/indexer/sync", any(trigger_sync))
        .with_state(sync_state);

    let addr: SocketAddr = format!("{}:{}", args.host, args.port).parse().unwrap();
    info!("Indexer service listening on {}", addr);
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn health() -> StatusCode { StatusCode::OK }

async fn get_chain_status(
    State(state): State<AppState>,
    AxPath(chain_id): AxPath<String>,
) -> Result<Json<ChainStatus>, StatusCode> {
    let result: Option<(Option<i64>,)> = sqlx::query_as("SELECT MAX(number) FROM blocks WHERE chain_id = $1")
        .bind(&chain_id).fetch_optional(&state.db).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let indexer_block = result.and_then(|r| r.0).unwrap_or(0) as u64;

    let latest_block = if let Some(p) = state.provider.read().await.as_ref() {
        epsx_web3::fetch_block_number(p.as_ref()).await.unwrap_or(indexer_block)
    } else {
        indexer_block
    };

    let lag = latest_block.saturating_sub(indexer_block);
    let chain_enum = epsx_kernel::ChainId(chain_id.parse().unwrap_or(56));

    Ok(Json(ChainStatus {
        chain_id,
        name: chain_enum.name().to_string(),
        latest_block,
        indexer_block,
        lag,
        healthy: lag < 100,
    }))
}

async fn get_block(
    State(state): State<AppState>,
    AxPath((chain_id, number)): AxPath<(String, i64)>,
) -> Result<Json<BlockRecord>, StatusCode> {
    let block: BlockRecord = sqlx::query_as::<_, BlockRecord>(
        "SELECT chain_id, number, hash, parent_hash, timestamp, miner, gas_used, gas_limit, tx_count FROM blocks WHERE chain_id = $1 AND number = $2"
    )
    .bind(&chain_id)
    .bind(&number)
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
    let tx: TxRecord = sqlx::query_as::<_, TxRecord>(
        "SELECT chain_id, hash, from_address, to_address, value, block_number, status, timestamp FROM transactions WHERE chain_id = $1 AND hash = $2"
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
    let address = address.to_lowercase();
    let transfers: Vec<TokenTransfer> = sqlx::query_as::<_, TokenTransfer>(
        "SELECT chain_id, tx_hash, log_index, token_address, from_address, to_address, value, block_number, timestamp FROM token_transfers WHERE chain_id = $1 AND (from_address = $2 OR to_address = $2) ORDER BY block_number DESC LIMIT 100"
    )
    .bind(&chain_id)
    .bind(&address)
    .fetch_all(&state.db)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(transfers))
}

async fn trigger_sync(
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    if let Err(e) = sync_chain_once(state.clone(), 5).await {
        return Ok(Json(serde_json::json!({ "success": false, "error": e })));
    }
    Ok(Json(serde_json::json!({ "success": true, "indexer_block": *state.last_block.read().await })))
}

async fn sync_chain(state: AppState, poll_interval: u64, batch: u64) {
    let mut backoff = Duration::from_secs(poll_interval);
    let max_backoff = Duration::from_secs(30);
    let mut ticker = tokio::time::interval(backoff);
    loop {
        ticker.tick().await;
        match sync_chain_once(state.clone(), batch).await {
            Ok(()) => {
                if backoff > Duration::from_secs(poll_interval) {
                    info!("Sync recovered, backoff reset to {}s", poll_interval);
                }
                backoff = Duration::from_secs(poll_interval);
            }
            Err(e) => {
                error!("Sync error: {} (retrying in {:?})", e, backoff);
                tokio::time::sleep(backoff).await;
                backoff = (backoff * 2).min(max_backoff);
            }
        }
    }
}

async fn sync_chain_once(state: AppState, batch: u64) -> Result<(), String> {
    if state.provider.read().await.is_none() {
        let chain_id_enum = epsx_kernel::ChainId(state.chain_id);
        match epsx_web3::provider_for_chain(chain_id_enum) {
            Ok(p) => { *state.provider.write().await = Some(Arc::from(p)); }
            Err(e) => return Err(format!("provider init: {}", e)),
        }
    }

    let provider_opt = state.provider.read().await.clone();
    let provider = provider_opt.ok_or_else(|| "no provider".to_string())?;

    let latest = epsx_web3::fetch_block_number(&*provider).await.map_err(|e| e.to_string())?;
    let current = *state.last_block.read().await;
    let from = if current == 0 { latest.saturating_sub(batch) } else { current + 1 };

    if from > latest {
        return Ok(());
    }

    let end = latest.min(from + batch - 1);
    info!("Syncing blocks {} -> {} (latest: {})", from, end, latest);
    for n in from..=end {
        if let Err(e) = index_block(&state, state.chain_id, n).await {
            warn!("Failed to index block {}: {}", n, e);
        } else {
            *state.last_block.write().await = n;
        }
    }

    Ok(())
}

async fn index_block(state: &AppState, chain_id: u64, number: u64) -> Result<(), String> {
    let provider_opt = state.provider.read().await.clone();
    let provider = provider_opt.ok_or_else(|| "no provider".to_string())?;

    let block = epsx_web3::fetch_block_full(&*provider, number).await
        .map_err(|e| e.to_string())?
        .ok_or_else(|| format!("block {} not found", number))?;

    let hash = format!("{:#x}", block.header.hash);
    let parent_hash = format!("{:#x}", block.header.parent_hash);
    let miner = format!("{:#x}", block.header.beneficiary);
    let gas_used = block.header.gas_used as i64;
    let gas_limit = block.header.gas_limit as i64;
    let ts_secs = block.header.timestamp as i64;

    if miner.len() > 42 {
        return Err(format!("miner address too long: {} (len={})", miner, miner.len()));
    }

    let tx_count = match &block.transactions {
        alloy::rpc::types::BlockTransactions::Full(txs) => txs.len() as i32,
        alloy::rpc::types::BlockTransactions::Hashes(h) => h.len() as i32,
        alloy::rpc::types::BlockTransactions::Uncle => 0,
    };

    let tx_kind = match &block.transactions {
        alloy::rpc::types::BlockTransactions::Full(_) => "Full",
        alloy::rpc::types::BlockTransactions::Hashes(_) => "Hashes",
        alloy::rpc::types::BlockTransactions::Uncle => "Uncle",
    };

    if number % 100 == 0 {
        info!("Block {}: {} txs ({}), miner={}", number, tx_count, tx_kind, miner);
    }

    sqlx::query(
        "INSERT INTO blocks (chain_id, number, hash, parent_hash, timestamp, miner, gas_used, gas_limit, tx_count)
         VALUES ($1, $2, $3, $4, to_timestamp($5), $6, $7, $8, $9)
         ON CONFLICT (chain_id, number) DO UPDATE SET
            hash = EXCLUDED.hash,
            parent_hash = EXCLUDED.parent_hash,
            gas_used = EXCLUDED.gas_used,
            gas_limit = EXCLUDED.gas_limit,
            tx_count = EXCLUDED.tx_count,
            timestamp = EXCLUDED.timestamp"
    )
    .bind(chain_id.to_string())
    .bind(number as i64)
    .bind(&hash)
    .bind(&parent_hash)
    .bind(ts_secs)
    .bind(&miner)
    .bind(gas_used)
    .bind(gas_limit)
    .bind(tx_count)
    .execute(&state.db)
    .await
    .map_err(|e| e.to_string())?;

    match &block.transactions {
        alloy::rpc::types::BlockTransactions::Full(txs) => {
            for tx in txs {
                let tx_hash = format!("{:#x}", tx.inner.tx_hash());
                let from = format!("{:#x}", tx.inner.signer());
                match sqlx::query(
                    "INSERT INTO transactions (chain_id, hash, from_address, to_address, value, block_number, status, timestamp)
                     VALUES ($1, $2, $3, NULL, '0', $4, NULL, to_timestamp($5))
                     ON CONFLICT (hash) DO NOTHING"
                )
                .bind(chain_id.to_string())
                .bind(&tx_hash)
                .bind(&from)
                .bind(number as i64)
                .bind(ts_secs)
                .execute(&state.db)
                .await
                {
                    Ok(_) => {}
                    Err(e) => warn!("Block {} tx {} insert failed: {e}", number, tx_hash),
                }
            }
        }
        alloy::rpc::types::BlockTransactions::Hashes(hashes) => {
            for h in hashes {
                let tx_hash = format!("{:#x}", h);
                if let Err(e) = sqlx::query(
                    "INSERT INTO transactions (chain_id, hash, from_address, to_address, value, block_number, status, timestamp)
                     VALUES ($1, $2, NULL, NULL, '0', $3, NULL, to_timestamp($4))
                     ON CONFLICT (hash) DO NOTHING"
                )
                .bind(chain_id.to_string())
                .bind(&tx_hash)
                .bind(number as i64)
                .bind(ts_secs)
                .execute(&state.db)
                .await
                {
                    warn!("Failed to insert tx {} in block {}: {e}", tx_hash, number);
                }
            }
        }
        alloy::rpc::types::BlockTransactions::Uncle => {}
    }

    let filter = Filter::new()
        .from_block(number)
        .to_block(number)
        .event_signature(Transfer::SIGNATURE_HASH);

    match epsx_web3::fetch_logs(&*provider, filter).await {
        Ok(logs) => {
            let mut count = 0;
            for log in logs {
                let token = format!("{:#x}", log.address());
                if token.len() > 42 {
                    warn!("Block {}: skipping transfer with long token addr (len={})", number, token.len());
                    continue;
                }
                if let Some(tx_hash) = log.transaction_hash {
                    if let Ok(decoded) = log.log_decode::<Transfer>() {
                        let from = format!("{:#x}", decoded.inner.from);
                        let to = format!("{:#x}", decoded.inner.to);
                        let value = decoded.inner.value.to_string();
                        let log_idx = log.log_index.unwrap_or_default() as i32;
                        let _ = sqlx::query(
                            "INSERT INTO token_transfers (chain_id, tx_hash, log_index, token_address, from_address, to_address, value, block_number, timestamp)
                             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, to_timestamp($9))
                             ON CONFLICT (chain_id, tx_hash, log_index) DO NOTHING"
                        )
                        .bind(chain_id.to_string())
                        .bind(format!("{:#x}", tx_hash))
                        .bind(log_idx)
                        .bind(&token)
                        .bind(&from)
                        .bind(&to)
                        .bind(&value)
                        .bind(number as i64)
                        .bind(ts_secs)
                        .execute(&state.db)
                        .await;
                        count += 1;
                    }
                }
            }
            if count > 0 {
                info!("Block {}: {} ERC-20 Transfer events decoded", number, count);
            }
        }
        Err(e) => {
            warn!("Block {}: eth_getLogs failed: {}", number, e);
        }
    }

    Ok(())
}
