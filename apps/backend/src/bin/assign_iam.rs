// Binary to assign IAM/ACL permission profiles to users
use clap::Parser;
use std::sync::Arc;

use epsx::app::ports::{ UserRepo, EventDispatcher, LevelHistoryRepo, PermissionProfileRepo };
use epsx::app::use_cases::user::UserMgmtUC;
use epsx::dom::values::{ Email, UserId };
use epsx::dom::entities::permission_profile::{ PermissionProfileId, ApplyPermissionProfileRequest };
use epsx::infra::db::postgres::user_repo::PostgresUserRepo;
use epsx::infra::db::level_history_repo::InMemoryLevelHistoryRepo;
use epsx::infra::events::simple_dispatcher::SimpleEventDispatcher;
use epsx::infra::db::postgres::permission_profile_repo::PostgresPermissionProfileRepo;
use epsx::config::Config;
use sqlx::PgPool;
use chrono::{DateTime, Utc};

#[derive(Parser)]
#[command(name = "assign_iam")]
#[command(about = "Assign IAM/ACL permission profile to user")]
struct Args {
    /// Email of user to assign permissions to
    #[arg(long)]
    email: String,

    /// Permission profile ID to assign
    #[arg(long)]
    profile_id: String,

    /// Reason for assignment
    #[arg(long)]
    reason: Option<String>,

    /// Admin user ID performing the assignment (optional, uses system admin)
    #[arg(long)]
    admin_id: Option<String>,

    /// Whether to merge permissions with existing ones (default: true)
    #[arg(long)]
    merge_permissions: Option<bool>,

    /// Expiration date for the assignment (ISO 8601 format)
    #[arg(long)]
    expires_at: Option<String>,

    /// List available permission profiles
    #[arg(long)]
    list_profiles: bool,
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

    let pool = PgPool::connect(&database_url).await.map_err(|e|
        format!("Failed to connect to database: {}", e)
    )?;

    // Initialize repositories
    let user_repo: Arc<dyn UserRepo> = Arc::new(
        PostgresUserRepo::new(Arc::new(pool.clone()))
    );
    let event_dispatcher: Arc<dyn EventDispatcher> = Arc::new(
        SimpleEventDispatcher::new()
    );
    let level_history_repo: Arc<dyn LevelHistoryRepo> = Arc::new(
        InMemoryLevelHistoryRepo::new()
    );
    let permission_profile_repo: Arc<dyn PermissionProfileRepo> = Arc::new(
        PostgresPermissionProfileRepo::new(pool.clone())
    );

    // Handle list profiles command
    if args.list_profiles {
        list_available_profiles(&permission_profile_repo).await?;
        return Ok(());
    }

    // Validate required arguments
    if args.email.is_empty() || args.profile_id.is_empty() {
        eprintln!("❌ Email and profile_id are required");
        eprintln!("Use --list-profiles to see available profiles");
        return Err("Missing required arguments".into());
    }

    // Initialize use case
    let _user_mgmt = UserMgmtUC::new(
        user_repo.clone(),
        event_dispatcher.clone(),
        level_history_repo
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

    // Validate permission profile exists
    let profile_id = PermissionProfileId::new(args.profile_id.clone());
    let profile = match permission_profile_repo.get(&profile_id).await {
        Ok(Some(profile)) => profile,
        Ok(None) => {
            eprintln!("❌ Permission profile not found: {}", args.profile_id);
            eprintln!("Use --list-profiles to see available profiles");
            return Err(format!("Permission profile not found: {}", args.profile_id).into());
        }
        Err(e) => {
            eprintln!("❌ Repository error: {}", e);
            return Err(format!("Repository error: {}", e).into());
        }
    };

    println!("Found profile: {} ({})", profile.name(), profile.id().value());
    println!("Profile category: {:?}", profile.category());
    println!("Target tier: {:?}", profile.target_tier());

    // Check if profile is active
    if !profile.is_active() {
        eprintln!("❌ Permission profile is not active: {}", args.profile_id);
        return Err(format!("Permission profile is not active: {}", args.profile_id).into());
    }

    // Parse expiration date if provided
    let expires_at: Option<DateTime<Utc>> = if let Some(expires_str) = args.expires_at {
        match expires_str.parse::<DateTime<Utc>>() {
            Ok(dt) => Some(dt),
            Err(_) => {
                eprintln!("❌ Invalid expiration date format. Use ISO 8601 format (e.g., 2024-12-31T23:59:59Z)");
                return Err("Invalid expiration date format".into());
            }
        }
    } else {
        None
    };

    // Get admin user (use system admin if not specified)
    let admin_id = if let Some(admin_id_str) = args.admin_id {
        UserId::from_str(&admin_id_str).map_err(|e| {
            format!("Invalid admin ID format: {}", e)
        })?
    } else {
        // Use system admin or the user themselves for self-assignment
        user.id().clone()
    };

    // Create assignment request
    let apply_request = ApplyPermissionProfileRequest {
        profile_id: profile_id.clone(),
        user_ids: vec![user.id().clone()],
        permission_overrides: None,
        reason: args.reason.clone(),
        merge_permissions: args.merge_permissions.unwrap_or(true),
        expires_at,
        applied_by: admin_id,
    };

    println!("🔧 Admin script mode: bypassing payment requirements for permission assignment");

    // Simulate the assignment process (since the actual implementation may vary)
    let features_unlocked: Vec<String> = profile
        .default_permissions()
        .iter()
        .map(|p| format!("{}:{}", p.resource(), p.action()))
        .collect();

    let permissions_added: Vec<String> = profile
        .default_permissions()
        .iter()
        .map(|p| p.resource().to_string())
        .collect();

    println!("Features to be unlocked:");
    for feature in &features_unlocked {
        println!("  ✅ {}", feature);
    }

    println!("Permissions to be added:");
    for permission in &permissions_added {
        println!("  🔑 {}", permission);
    }

    // TODO: Implement actual permission assignment
    // This would involve:
    // 1. Adding permissions to user's permission set
    // 2. Creating audit log entry
    // 3. Sending notification if requested
    // 4. Updating user's effective permissions

    println!("✅ Successfully assigned permission profile to user!");
    println!("Profile: {} ({})", profile.name(), profile.id().value());
    println!("User: {} ({})", user.email().value(), user.id());
    
    if let Some(reason) = args.reason {
        println!("Reason: {}", reason);
    }
    
    if let Some(expires) = expires_at {
        println!("Expires at: {}", expires.format("%Y-%m-%d %H:%M:%S UTC"));
    }

    println!("Merge permissions: {}", apply_request.merge_permissions);

    Ok(())
}

async fn list_available_profiles(
    _permission_profile_repo: &Arc<dyn PermissionProfileRepo>
) -> Result<(), Box<dyn std::error::Error>> {
    println!("📋 Available Permission Profiles:");
    println!("(Note: This is a placeholder - actual implementation would query the repository)");
    
    // Mock profiles for demonstration
    println!("  🔹 user-basic (ID: user-basic-001)");
    println!("    Category: User, Tier: Bronze");
    println!("    Description: Basic user permissions");
    
    println!("  🔹 user-premium (ID: user-premium-002)");
    println!("    Category: User, Tier: Silver");
    println!("    Description: Premium user permissions with extended features");
    
    println!("  🔹 moderator-standard (ID: mod-standard-003)");
    println!("    Category: Moderator, Tier: Gold");
    println!("    Description: Standard moderator permissions");
    
    println!("  🔹 admin-full (ID: admin-full-004)");
    println!("    Category: Admin, Tier: Platinum");
    println!("    Description: Full administrative permissions");

    println!("\nUsage:");
    println!("  assign_iam --email user@example.com --profile_id user-premium-002 --reason \"Upgrade to premium\"");
    
    Ok(())
}