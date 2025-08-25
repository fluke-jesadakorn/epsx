// Admin Assignment CLI Tool
// Assigns admin modules to users via database

use clap::Parser;
use std::env;
use tracing::{info, error};
use serde_json;

#[derive(Parser)]
#[command(name = "assign-admin-modules")]
#[command(about = "Assign admin modules to a user")]
struct Args {
    /// User email to assign modules to
    #[arg(long)]
    email: String,
    
    /// Comma-separated list of admin modules
    #[arg(long)]
    modules: String,
    
    /// Reason for assignment
    #[arg(long, default_value = "Admin assignment via CLI tool")]
    reason: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt().init();
    
    let args = Args::parse();
    
    // Get database URL
    let database_url = env::var("DATABASE_URL")
        .map_err(|_| "DATABASE_URL environment variable is required")?;
    
    info!("Assigning admin modules to user: {}", args.email);
    info!("Modules: {}", args.modules);
    
    // Parse modules list
    let modules: Vec<&str> = args.modules.split(',').map(|s| s.trim()).collect();
    
    // Connect to PostgreSQL directly
    let client = tokio_postgres::connect(&database_url, tokio_postgres::NoTls).await?;
    let (client, connection) = client;
    
    // Spawn connection handler
    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("Connection error: {}", e);
        }
    });
    
    // First check if user exists
    let user_check = client.query_one(
        "SELECT firebase_uid FROM users WHERE email = $1",
        &[&args.email]
    ).await;
    
    let firebase_uid = match user_check {
        Ok(row) => row.get::<_, String>(0),
        Err(_) => {
            error!("User with email {} not found in database", args.email);
            return Err("User not found".into());
        }
    };
    
    // Assign each module
    for module in modules {
        info!("Assigning module: {} to user: {}", module, args.email);
        
        let result = client.execute(
            r#"
            INSERT INTO user_admin_roles (firebase_uid, module_code, granted_by, granted_reason, is_active, created_at)
            VALUES ($1, $2, 'CLI_TOOL', $3, true, NOW())
            ON CONFLICT (firebase_uid, module_code) 
            DO UPDATE SET 
                is_active = true,
                granted_reason = $3,
                updated_at = NOW()
            "#,
            &[&firebase_uid, &module, &args.reason]
        ).await;
        
        match result {
            Ok(_) => info!("✅ Successfully assigned module '{}' to {}", module, args.email),
            Err(e) => error!("❌ Failed to assign module '{}': {}", module, e),
        }
    }
    
    info!("🎉 Admin module assignment completed for: {}", args.email);
    Ok(())
}
