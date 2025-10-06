use clap::{Arg, Command};
use sqlx::PgPool;
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let matches = Command::new("grant_wallet_permission")
        .about("Grant permissions to a Web3 wallet address")
        .arg(
            Arg::new("wallet")
                .long("wallet")
                .short('w')
                .value_name("WALLET_ADDRESS")
                .help("Wallet address to grant permissions to")
                .required(true)
        )
        .arg(
            Arg::new("permission")
                .long("permission")
                .short('p')
                .value_name("PERMISSION")
                .help("Permission to grant (e.g., 'epsx:analytics:view')")
                .required(false)
        )
        .arg(
            Arg::new("analytics")
                .long("analytics")
                .help("Grant all analytics permissions")
                .action(clap::ArgAction::SetTrue)
        )
        .get_matches();

    // Get database connection
    let database_url = env::var("DATABASE_URL")
        .expect("DATABASE_URL environment variable not set");
    
    let pool = PgPool::connect(&database_url).await?;

    let wallet_address = matches.get_one::<String>("wallet").unwrap();

    if matches.get_flag("analytics") {
        grant_analytics_permissions(&pool, wallet_address).await?;
    } else if let Some(permission) = matches.get_one::<String>("permission") {
        grant_single_permission(&pool, wallet_address, permission).await?;
    } else {
        eprintln!("Error: Either --analytics or --permission must be specified");
        std::process::exit(1);
    }

    Ok(())
}

async fn grant_analytics_permissions(
    pool: &PgPool,
    wallet_address: &str
) -> Result<(), Box<dyn std::error::Error>> {
    let permissions = vec![
        "epsx:analytics:view",
        "epsx:analytics:basic", 
        "epsx:analytics:premium",
        "epsx:analytics:professional"
    ];

    println!("Granting analytics permissions to wallet: {}", wallet_address);

    // First, ensure the wallet user exists
    let wallet_addr = ensure_wallet_user_exists(pool, wallet_address).await?;

    for permission in permissions {
        grant_single_permission_direct(pool, &wallet_addr, permission).await?;
        println!("✅ Granted: {}", permission);
    }

    println!("🎉 All analytics permissions granted successfully!");
    Ok(())
}

async fn grant_single_permission(
    pool: &PgPool,
    wallet_address: &str,
    permission: &str
) -> Result<(), Box<dyn std::error::Error>> {
    println!("Granting permission '{}' to wallet: {}", permission, wallet_address);

    // Ensure the wallet user exists
    let wallet_addr = ensure_wallet_user_exists(pool, wallet_address).await?;
    
    // Grant the permission
    grant_single_permission_direct(pool, &wallet_addr, permission).await?;

    println!("✅ Permission granted successfully!");
    Ok(())
}

async fn ensure_wallet_user_exists(
    pool: &PgPool, 
    wallet_address: &str
) -> Result<String, Box<dyn std::error::Error>> {
    // Check if user exists
    let existing_user = sqlx::query!(
        "SELECT wallet_address FROM wallet_users WHERE wallet_address = $1",
        wallet_address
    )
    .fetch_optional(pool)
    .await?;

    if let Some(user) = existing_user {
        println!("Found existing wallet user: {}", user.wallet_address);
        return Ok(user.wallet_address);
    }

    // Create new wallet user
    sqlx::query!(
        "INSERT INTO wallet_users (wallet_address, is_active, permissions, wallet_metadata) VALUES ($1, $2, $3, $4)",
        wallet_address,
        true,
        serde_json::json!([]),
        serde_json::json!({})
    )
    .execute(pool)
    .await?;

    println!("Created new wallet user: {}", wallet_address);
    Ok(wallet_address.to_string())
}

async fn grant_single_permission_direct(
    pool: &PgPool,
    wallet_address: &str,
    permission: &str
) -> Result<(), Box<dyn std::error::Error>> {
    // Use the database function to add permission
    let result = sqlx::query!(
        "SELECT add_wallet_user_permission($1, $2, $3, $4, $5) as success",
        wallet_address,
        permission,
        "Manual",
        Option::<chrono::DateTime<chrono::Utc>>::None,
        serde_json::json!({})
    )
    .fetch_one(pool)
    .await?;

    if result.success.unwrap_or(false) {
        println!("Permission '{}' granted to wallet '{}'", permission, wallet_address);
    } else {
        println!("Permission '{}' already exists for wallet '{}'", permission, wallet_address);
    }

    Ok(())
}