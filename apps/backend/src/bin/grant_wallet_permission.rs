use clap::{Arg, Command};
use diesel::prelude::*;
use epsx::infrastructure::models::wallet_user::NewWalletUserDb;
use epsx::infrastructure::models::permission::{PermissionDb, NewWalletDirectPermissionDb};
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

    let mut conn = diesel::PgConnection::establish(&database_url)?;

    let wallet_address = matches.get_one::<String>("wallet").unwrap();

    if matches.get_flag("analytics") {
        grant_analytics_permissions(&mut conn, wallet_address)?;
    } else if let Some(permission) = matches.get_one::<String>("permission") {
        grant_single_permission(&mut conn, wallet_address, permission)?;
    } else {
        eprintln!("Error: Either --analytics or --permission must be specified");
        std::process::exit(1);
    }

    Ok(())
}

fn grant_analytics_permissions(
    conn: &mut diesel::PgConnection,
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
    let wallet_addr = ensure_wallet_user_exists(conn, wallet_address)?;

    for permission in permissions {
        grant_single_permission_direct(conn, &wallet_addr, permission)?;
        println!("✅ Granted: {}", permission);
    }

    println!("🎉 All analytics permissions granted successfully!");
    Ok(())
}

fn grant_single_permission(
    conn: &mut diesel::PgConnection,
    wallet_address: &str,
    permission: &str
) -> Result<(), Box<dyn std::error::Error>> {
    println!("Granting permission '{}' to wallet: {}", permission, wallet_address);

    // Ensure the wallet user exists
    let wallet_addr = ensure_wallet_user_exists(conn, wallet_address)?;

    // Grant the permission
    grant_single_permission_direct(conn, &wallet_addr, permission)?;

    println!("✅ Permission granted successfully!");
    Ok(())
}

fn ensure_wallet_user_exists(
    conn: &mut diesel::PgConnection,
    wallet_addr_str: &str
) -> Result<String, Box<dyn std::error::Error>> {
    use epsx::schemas::primary::wallet_users::dsl::*;

    // Check if user exists
    let existing_user: Option<String> = wallet_users
        .select(wallet_address)
        .filter(wallet_address.eq(wallet_addr_str))
        .first(conn)
        .optional()?;

    if let Some(user_addr) = existing_user {
        println!("Found existing wallet user: {}", user_addr);
        return Ok(user_addr);
    }

    // Create new wallet user
    let new_wallet_user = NewWalletUserDb {
        wallet_address: wallet_addr_str.to_string(),
        is_active: true,
        wallet_metadata: serde_json::json!({}),
    };

    diesel::insert_into(wallet_users)
        .values(&new_wallet_user)
        .execute(conn)?;

    println!("Created new wallet user: {}", wallet_addr_str);
    Ok(wallet_addr_str.to_string())
}

fn grant_single_permission_direct(
    conn: &mut diesel::PgConnection,
    wallet_addr_str: &str,
    permission_str: &str
) -> Result<(), Box<dyn std::error::Error>> {
    use epsx::schemas::primary::permissions::dsl::*;
    use epsx::schemas::primary::wallet_direct_permissions::dsl as wdp;

    // 1. Get permission definition
    let perm_def: Option<PermissionDb> = permissions
        .filter(permission_string.eq(permission_str))
        .first(conn)
        .optional()?;

    let perm = match perm_def {
        Some(p) => p,
        None => {
            eprintln!("Error: Permission definition '{}' not found in permissions catalog.", permission_str);
            eprintln!("Please add it to the permissions table first.");
            std::process::exit(1);
        }
    };

    // 2. Check if already granted
    let existing_count: i64 = wdp::wallet_direct_permissions
        .count()
        .filter(wdp::wallet_address.eq(wallet_addr_str))
        .filter(wdp::permission_id.eq(perm.id))
        .get_result(conn)?;

    if existing_count > 0 {
        println!("Permission '{}' already granted to wallet '{}'", permission_str, wallet_addr_str);
        return Ok(());
    }

    // 3. Grant permission
    let new_grant = NewWalletDirectPermissionDb {
        wallet_address: wallet_addr_str.to_string(),
        permission_id: perm.id,
        granted_by: Some("Manual (CLI)".to_string()),
        grant_reason: Some("CLI tool grant".to_string()),
        expires_at: None,
        is_active: true,
    };

    diesel::insert_into(wdp::wallet_direct_permissions)
        .values(&new_grant)
        .execute(conn)?;

    println!("Permission '{}' granted to wallet '{}'", permission_str, wallet_addr_str);
    Ok(())
}