// Binary to promote user to SuperAdmin role
use clap::Parser;
use std::sync::Arc;

use epsx::app::ports::{UserRepo, EventDispatcher, LevelHistoryRepo};
use epsx::app::use_cases::user::UserMgmtUC;
use epsx::app::dtos::user::UpdateRoleReq;
use epsx::dom::values::{UserId, Email, Role};
use epsx::dom::events::DomainEvent;
use epsx::infra::db::postgres::user_repo::PostgresUserRepo;
use epsx::infra::db::level_history_repo::InMemoryLevelHistoryRepo;
use epsx::infra::events::simple_dispatcher::SimpleEventDispatcher;
use epsx::config::Config;
use sqlx::PgPool;

#[derive(Parser)]
#[command(name = "promote_admin")]
#[command(about = "Promote user to SuperAdmin role")]
struct Args {
    /// Email of user to promote
    #[arg(long)]
    email: String,
    
    /// Reason for promotion
    #[arg(long)]
    reason: Option<String>,
    
    /// Admin user ID performing the promotion (optional, uses system admin)
    #[arg(long)]
    admin_id: Option<String>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    
    // Initialize config
    let config = Config::from_env();
    
    // Initialize database connection
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| {
            format!(
                "postgresql://{}:{}@{}:{}/{}",
                config.database.username,
                config.database.password,
                config.database.host,
                config.database.port,
                config.database.database
            )
        });
    
    let pool = PgPool::connect(&database_url).await
        .map_err(|e| format!("Failed to connect to database: {}", e))?;
    
    // Initialize repositories
    let user_repo: Arc<dyn UserRepo> = Arc::new(PostgresUserRepo::new(Arc::new(pool)));
    let event_dispatcher: Arc<dyn EventDispatcher> = Arc::new(SimpleEventDispatcher::new());
    let level_history_repo: Arc<dyn LevelHistoryRepo> = Arc::new(InMemoryLevelHistoryRepo::new());
    
    // Initialize use case
    let user_mgmt = UserMgmtUC::new(
        user_repo.clone(),
        event_dispatcher.clone(),
        level_history_repo,
    );
    
    // Parse and validate email
    let email = match Email::new(args.email.clone()) {
        Ok(email) => email,
        Err(_) => {
            eprintln!("❌ Invalid email format: {}", args.email);
            return Err(format!("Invalid email format: {}", args.email).into());
        }
    };
    
    // Find user by email
    let user = match user_repo.find_by_email(&email).await {
        Ok(Some(user)) => user,
        Ok(None) => {
            eprintln!("❌ User not found with email: {}", args.email);
            return Err(format!("User not found with email: {}", args.email).into());
        }
        Err(e) => {
            eprintln!("❌ Repository error: {}", e);
            return Err(format!("Repository error: {}", e).into());
        }
    };
    
    println!("Found user: {} ({})", user.email().value(), user.id());
    println!("Current role: {:?}", user.role());
    
    // Check if already SuperAdmin
    if matches!(user.role(), Role::SuperAdmin) {
        println!("User is already a SuperAdmin");
        return Ok(());
    }
    
    // Determine admin user ID
    let admin_id = if let Some(admin_id_str) = args.admin_id {
        UserId::from_string(admin_id_str)
    } else {
        // Use system admin - find an existing SuperAdmin
        match user_repo.find_by_role(&Role::SuperAdmin).await {
            Ok(admins) if !admins.is_empty() => {
                println!("Found existing SuperAdmin, using for promotion");
                admins[0].id().clone()
            },
            _ => {
                // No SuperAdmin exists - bypass permissions for initial setup
                println!("⚠️  No SuperAdmin found. Using direct role upgrade for initial setup.");
                
                // Direct role upgrade bypassing permission checks
                let mut target_user = user.clone();
                let _old_role = target_user.role().clone();
                
                // Force upgrade role directly
                if let Ok(event) = target_user.upgrade_role(Role::SuperAdmin) {
                    // Save the user with new role
                    if let Err(e) = user_repo.save(&target_user).await {
                        eprintln!("❌ Failed to save user: {}", e);
                        return Err(format!("Failed to save user: {}", e).into());
                    }
                    
                    // Dispatch event
                    if let Err(e) = event_dispatcher.dispatch(Box::new(event.clone())).await {
                        eprintln!("⚠️  Failed to dispatch event: {}", e);
                    }
                    
                    println!("✅ Successfully promoted user to SuperAdmin! (Initial setup)");
                    println!("Event ID: {}", event.event_id());
                    if let Some(reason) = args.reason {
                        println!("Reason: {}", reason);
                    }
                    return Ok(());
                } else {
                    eprintln!("❌ Failed to upgrade role directly");
                    return Err("Failed to upgrade role directly".into());
                }
            }
        }
    };
    
    println!("Performing promotion with admin ID: {}", admin_id);
    
    // Create update request
    let update_req = UpdateRoleReq {
        usr_id: user.id().clone(),
        admin_id,
        new_role: "super_admin".to_string(),
    };
    
    // Perform role update
    match user_mgmt.update_role(update_req).await {
        Ok(response) => {
            println!("✅ Successfully promoted user to SuperAdmin!");
            println!("Event ID: {}", response.event_id);
            if let Some(reason) = args.reason {
                println!("Reason: {}", reason);
            }
            Ok(())
        }
        Err(e) => {
            eprintln!("❌ Failed to promote user: {}", e);
            Err(format!("Failed to promote user: {}", e).into())
        }
    }
}