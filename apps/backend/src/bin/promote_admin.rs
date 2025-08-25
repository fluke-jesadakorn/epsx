// Promote User to Full Admin CLI Tool
// Assigns all available admin modules to a user

use clap::Parser;
use std::env;
use tracing::{info, error};

#[derive(Parser)]
#[command(name = "promote-admin")]
#[command(about = "Promote user to full admin with all modules")]
struct Args {
    /// User email to promote to admin
    #[arg(long)]
    email: String,
    
    /// Reason for promotion
    #[arg(long, default_value = "Full admin promotion via CLI tool")]
    reason: String,
}

// All available admin modules
const ALL_ADMIN_MODULES: &[&str] = &[
    "system_admin",
    "user_management", 
    "analytics_access",
    "security_management",
    "audit_logs",
    "financial_oversight",
    "content_management",
    "support_access",
    "database_management",
    "developer_portal",
    "module_management"
];

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt().init();
    
    let args = Args::parse();
    
    // Get database URL
    let database_url = env::var("DATABASE_URL")
        .map_err(|_| "DATABASE_URL environment variable is required")?;
    
    info!("Promoting user to full admin: {}", args.email);
    
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
    
    info!("Assigning {} admin modules to user", ALL_ADMIN_MODULES.len());
    
    // Assign all admin modules
    let mut success_count = 0;
    let mut error_count = 0;
    
    for module in ALL_ADMIN_MODULES {
        info!("Assigning module: {}", module);
        
        let result = client.execute(
            r#"
            INSERT INTO user_admin_roles (firebase_uid, module_code, granted_by, granted_reason, is_active, created_at)
            VALUES ($1, $2, 'CLI_PROMOTE_ADMIN', $3, true, NOW())
            ON CONFLICT (firebase_uid, module_code) 
            DO UPDATE SET 
                is_active = true,
                granted_reason = $3,
                updated_at = NOW()
            "#,
            &[&firebase_uid, module, &args.reason]
        ).await;
        
        match result {
            Ok(_) => {
                info!("✅ Successfully assigned module '{}'", module);
                success_count += 1;
            },
            Err(e) => {
                error!("❌ Failed to assign module '{}': {}", module, e);
                error_count += 1;
            }
        }
    }
    
    info!("🎉 Admin promotion completed for: {}", args.email);
    info!("✅ Success: {} modules assigned", success_count);
    if error_count > 0 {
        info!("❌ Errors: {} modules failed", error_count);
    }
    
    Ok(())
}
