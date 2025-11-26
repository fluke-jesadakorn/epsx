use clap::{Arg, Command};
use diesel::prelude::*;
use epsx::infrastructure::models::wallet_user::WalletUserDb;
use epsx::infrastructure::models::wallet_user::NewWalletUserDb;
use epsx::infrastructure::models::permission::NewPermissionDb;
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
    use epsx::schema::wallet_users::dsl::*;

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
    permission: &str
) -> Result<(), Box<dyn std::error::Error>> {
    use epsx::schema::permissions::dsl::*;

    // Check if permission already exists in the unified permissions table
    let existing_permission_count: i64 = permissions
        .count()
        .filter(wallet_address.eq(wallet_addr_str))
        .filter(permission_string.eq(permission))
        .get_result(conn)?;

    if existing_permission_count > 0 {
        println!("Permission '{}' already exists for wallet '{}'", permission, wallet_addr_str);
        return Ok(());
    }

    // Create new permission using unified permission structure
    let permission_parts: Vec<&str> = permission.split(':').collect();
    if permission_parts.len() != 3 {
        eprintln!("Error: Permission must be in format 'platform:resource:action'");
        std::process::exit(1);
    }

    let new_permission = NewPermissionDb {
        permission_string: permission.to_string(),
        platform: permission_parts[0].to_string(),
        resource: permission_parts[1].to_string(),
        action: permission_parts[2].to_string(),
        description: None,
        permission_type: "manual".to_string(),
        wallet_address: Some(wallet_addr_str.to_string()),
        source_type: Some("direct".to_string()),
        source_id: None,
        granted_by: Some("Manual".to_string()),
        grant_reason: Some("CLI tool".to_string()),
    };

    diesel::insert_into(permissions)
        .values(&new_permission)
        .execute(conn)?;

    println!("Permission '{}' granted to wallet '{}'", permission, wallet_addr_str);
    Ok(())
}