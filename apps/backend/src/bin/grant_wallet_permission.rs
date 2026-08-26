use clap::{Arg, Command};
use epsx::infrastructure::models::permission::NewWalletDirectPermissionDb;
use epsx::infrastructure::models::wallet_user::NewWalletUserDb;
use sqlx::PgPool;
use std::env;
use uuid::Uuid;

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
                .required(true),
        )
        .arg(
            Arg::new("permission")
                .long("permission")
                .short('p')
                .value_name("PERMISSION")
                .help("Permission to grant (e.g., 'epsx:analytics:view')")
                .required(false),
        )
        .arg(
            Arg::new("analytics")
                .long("analytics")
                .help("Grant all analytics permissions")
                .action(clap::ArgAction::SetTrue),
        )
        .get_matches();

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL environment variable not set");
    let pool = sqlx::postgres::PgPoolOptions::new()
        .max_connections(2)
        .connect(&database_url)
        .await?;

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
    wallet_address: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let permissions = vec![
        "epsx:analytics:view",
        "epsx:analytics:basic",
        "epsx:analytics:premium",
        "epsx:analytics:professional",
    ];

    println!(
        "Granting analytics permissions to wallet: {}",
        wallet_address
    );

    ensure_wallet_user_exists(pool, wallet_address).await?;

    for permission in permissions {
        grant_single_permission_direct(pool, wallet_address, permission).await?;
        println!("Granted: {}", permission);
    }

    println!("All analytics permissions granted successfully!");
    Ok(())
}

async fn grant_single_permission(
    pool: &PgPool,
    wallet_address: &str,
    permission: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    println!(
        "Granting permission '{}' to wallet: {}",
        permission, wallet_address
    );

    ensure_wallet_user_exists(pool, wallet_address).await?;
    grant_single_permission_direct(pool, wallet_address, permission).await?;

    println!("Permission granted successfully!");
    Ok(())
}

async fn ensure_wallet_user_exists(
    pool: &PgPool,
    wallet_addr_str: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let existing: Option<(String,)> =
        sqlx::query_as("SELECT wallet_address FROM wallet_users WHERE wallet_address = $1")
            .bind(wallet_addr_str)
            .fetch_optional(pool)
            .await?;

    if existing.is_some() {
        println!("Found existing wallet user: {}", wallet_addr_str);
        return Ok(());
    }

    let new_wallet_user = NewWalletUserDb {
        wallet_address: wallet_addr_str.to_string(),
        is_active: true,
        tier_level: "Bronze".to_string(),
        wallet_metadata: serde_json::json!({}),
    };

    sqlx::query(
        "INSERT INTO wallet_users (wallet_address, is_active, tier_level, wallet_metadata, created_at, updated_at) \
         VALUES ($1, $2, 'Bronze', $3, NOW(), NOW())",
    )
    .bind(&new_wallet_user.wallet_address)
    .bind(new_wallet_user.is_active)
    .bind(&new_wallet_user.wallet_metadata)
    .execute(pool)
    .await?;

    println!("Created new wallet user: {}", wallet_addr_str);
    Ok(())
}

async fn grant_single_permission_direct(
    pool: &PgPool,
    wallet_addr_str: &str,
    permission_str: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    #[derive(sqlx::FromRow)]
    struct PermRow {
        id: Uuid,
    }

    let perm_row: Option<PermRow> =
        sqlx::query_as("SELECT id FROM permissions WHERE permission_string = $1")
            .bind(permission_str)
            .fetch_optional(pool)
            .await?;

    let perm = match perm_row {
        Some(p) => p,
        None => {
            eprintln!(
                "Error: Permission definition '{}' not found in permissions catalog.",
                permission_str
            );
            eprintln!("Please add it to the permissions table first.");
            std::process::exit(1);
        }
    };

    let existing_count: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM wallet_direct_permissions \
         WHERE wallet_address = $1 AND permission_id = $2",
    )
    .bind(wallet_addr_str)
    .bind(perm.id)
    .fetch_one(pool)
    .await?;

    if existing_count.0 > 0 {
        println!(
            "Permission '{}' already granted to wallet '{}'",
            permission_str, wallet_addr_str
        );
        return Ok(());
    }

    let new_grant = NewWalletDirectPermissionDb {
        wallet_address: wallet_addr_str.to_string(),
        permission_id: perm.id,
        granted_by: Some("Manual (CLI)".to_string()),
        grant_reason: Some("CLI tool grant".to_string()),
        expires_at: None,
        is_active: true,
    };

    sqlx::query(
        "INSERT INTO wallet_direct_permissions (wallet_address, permission_id, granted_by, grant_reason, expires_at, is_active, granted_at) \
         VALUES ($1, $2, $3, $4, $5, $6, NOW())",
    )
    .bind(&new_grant.wallet_address)
    .bind(new_grant.permission_id)
    .bind(&new_grant.granted_by)
    .bind(&new_grant.grant_reason)
    .bind(new_grant.expires_at)
    .bind(new_grant.is_active)
    .execute(pool)
    .await?;

    println!(
        "Permission '{}' granted to wallet '{}'",
        permission_str, wallet_addr_str
    );
    Ok(())
}
