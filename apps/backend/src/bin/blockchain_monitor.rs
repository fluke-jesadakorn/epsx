// Blockchain Monitor Binary
// Standalone service for monitoring BSC blockchain events and processing payments

use epsx::infrastructure::BlockchainMonitor;
use diesel_async::AsyncPgConnection;
use diesel_async::pooled_connection::deadpool::Pool;
use diesel_async::pooled_connection::AsyncDieselConnectionManager;
use diesel_async::RunQueryDsl;
use std::sync::Arc;
use tracing::{error, info, warn};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing subscriber for logging
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("epsx=info".parse()?)
                .add_directive("blockchain_monitor=debug".parse()?)
        )
        .init();

    info!("🚀 Starting EPSX Blockchain Payment Monitor");

    // Load environment variables
    dotenv::dotenv().ok();

    // Load configuration from environment
    let database_url = std::env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set");

    let network = std::env::var("BLOCKCHAIN_NETWORK")
        .unwrap_or_else(|_| "testnet".to_string());

    let (rpc_url, contract_address_var) = if network == "mainnet" {
        (
            std::env::var("BSC_MAINNET_RPC_URL")
                .expect("BSC_MAINNET_RPC_URL must be set for mainnet"),
            "PAYMENT_ESCROW_CONTRACT_MAINNET"
        )
    } else {
        (
            std::env::var("BSC_TESTNET_RPC_URL")
                .expect("BSC_TESTNET_RPC_URL must be set for testnet"),
            "PAYMENT_ESCROW_CONTRACT_TESTNET"
        )
    };

    let contract_address = std::env::var(contract_address_var)
        .unwrap_or_else(|_| {
            warn!("⚠️  {} not set, using placeholder", contract_address_var);
            "0x0000000000000000000000000000000000000000".to_string()
        });

    let start_block: u64 = std::env::var("BLOCKCHAIN_START_BLOCK")
        .unwrap_or_else(|_| "0".to_string())
        .parse()
        .expect("BLOCKCHAIN_START_BLOCK must be a valid number");

    let poll_interval: u64 = std::env::var("BLOCKCHAIN_POLL_INTERVAL_SECONDS")
        .unwrap_or_else(|_| "3".to_string())
        .parse()
        .expect("BLOCKCHAIN_POLL_INTERVAL_SECONDS must be a valid number");

    info!("📊 Configuration:");
    info!("   Network: {}", network);
    info!("   RPC: {}", rpc_url);
    info!("   Contract: {}", contract_address);
    info!("   Start Block: {}", start_block);
    info!("   Poll Interval: {}s", poll_interval);

    // Validate contract address
    if contract_address == "0x0000000000000000000000000000000000000000" {
        error!("❌ Invalid contract address. Please set {} in .env", contract_address_var);
        error!("   Deploy the smart contract first and update the environment variable.");
        std::process::exit(1);
    }

    // Connect to database
    info!("🔌 Connecting to PostgreSQL database...");
    let config = AsyncDieselConnectionManager::<AsyncPgConnection>::new(&database_url);
    let pool = Pool::builder(config)
        .max_size(10)
        .build()
        .expect("Failed to create database pool");

    // Test connection
    pool.get().await.expect("Failed to connect to PostgreSQL database");
    info!("✅ Database connection established");

    // Verify database migration
    info!("🔍 Verifying database schema...");

    #[derive(diesel::QueryableByName)]
    struct TableExistsResult {
        #[diesel(sql_type = diesel::sql_types::Bool)]
        exists: bool,
    }

    let mut conn = pool.get().await.expect("Failed to get database connection");
    let migration_check = diesel::sql_query(
        "SELECT EXISTS (
            SELECT FROM information_schema.tables
            WHERE table_name = 'processed_blockchain_events'
        ) as exists"
    )
    .get_result::<TableExistsResult>(&mut conn)
    .await;

    match migration_check {
        Ok(row) => {
            let exists: bool = row.exists;
            if !exists {
                error!("❌ Database migration not applied!");
                error!("   Run: diesel migration run");
                error!("   Or: psql -d {} -f migrations/008_blockchain_payments.sql", database_url);
                std::process::exit(1);
            }
            info!("✅ Database schema verified");
        }
        Err(e) => {
            error!("❌ Failed to verify database schema: {}", e);
            std::process::exit(1);
        }
    }

    // Leak the pool to make it 'static (required for BlockchainMonitor)
    let pool_static: &'static Pool<AsyncPgConnection> = Box::leak(Box::new(pool));
    let pool_arc = Arc::new(pool_static);

    // Create blockchain monitor
    info!("🔧 Initializing blockchain monitor...");
    let monitor = BlockchainMonitor::new(
        rpc_url.clone(),
        contract_address.clone(),
        start_block,
        poll_interval,
        pool_arc,
    )
    .expect("Failed to create blockchain monitor");

    info!("✅ Monitor initialized successfully");

    // Start monitoring in background
    info!("👂 Starting event listener...");
    monitor.start_monitoring()
        .await
        .expect("Failed to start blockchain monitoring");

    info!("🎯 Blockchain monitor is running!");
    info!("   Listening for PaymentReceived events on contract: {}", contract_address);
    info!("   Press Ctrl+C to stop");

    // Wait for shutdown signal
    match tokio::signal::ctrl_c().await {
        Ok(()) => {
            info!("🛑 Shutdown signal received");
        }
        Err(err) => {
            error!("Unable to listen for shutdown signal: {}", err);
        }
    }

    // Graceful shutdown
    info!("🔄 Stopping blockchain monitor...");
    monitor.stop_monitoring().await;
    info!("✅ Blockchain monitor stopped successfully");
    info!("👋 Goodbye!");

    Ok(())
}
