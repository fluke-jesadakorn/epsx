// Binary to promote user to SuperAdmin role
use clap::Parser;
use std::sync::Arc;

use epsx::app::ports::{ UserRepo, EventDispatcher, LevelHistoryRepo };
use epsx::app::use_cases::user::UserMgmtUC;
use epsx::dom::values::{ Email, Role };
use epsx::dom::entities::user::User;
use epsx::dom::events::DomainEvent;
use epsx::infra::db::postgres::user_repo::PostgresUserRepo;
use epsx::infra::db::level_history_repo::InMemoryLevelHistoryRepo;
use epsx::infra::events::simple_dispatcher::SimpleEventDispatcher;
use epsx::config::Config;
use sqlx::PgPool;

#[derive(Parser)]
#[command(name = "promote_admin")]
#[command(about = "Promote user to SuperAdmin role with full access")]
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

  /// Promote to SuperAdmin with ALL permissions (full system access)
  #[arg(long)]
  super_admin: bool,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
  let args = Args::parse();

  // Initialize config
  let config = Config::from_env();

  // Initialize database connection
  let database_url = std::env
    ::var("DATABASE_URL")
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
    PostgresUserRepo::new(Arc::new(pool))
  );
  let event_dispatcher: Arc<dyn EventDispatcher> = Arc::new(
    SimpleEventDispatcher::new()
  );
  let level_history_repo: Arc<dyn LevelHistoryRepo> = Arc::new(
    InMemoryLevelHistoryRepo::new()
  );

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

  // Check promotion mode
  if args.super_admin {
    println!("🚀 Super Admin mode: Promoting with ALL ACCESS");
  } else {
    println!("🔧 Standard Admin mode: Promoting to SuperAdmin");
  }

  // Check if already SuperAdmin
  if matches!(user.role(), Role::SuperAdmin) {
    if args.super_admin {
      println!("User is already a SuperAdmin, ensuring full permissions...");
    } else {
      println!("User is already a SuperAdmin");
      return Ok(());
    }
  }

  // Admin script mode: bypass permissions and directly upgrade role
  println!("🔧 Admin script mode: bypassing permissions for role upgrade");

  let mut target_user = user.clone();
  let _old_role = target_user.role().clone();

  // Force upgrade role directly (bypass validation for admin script)
  use epsx::dom::events::UserRoleChangedEvent;
  use epsx::dom::values::PermSet;
  use chrono::Utc;

  let old_role = target_user.role().clone();

  // Determine permissions based on super-admin flag
  let permissions = if args.super_admin {
    // Grant ALL permissions for super admin mode
    use epsx::dom::values::Permissions;
    let mut all_perms = PermSet::new();
    
    // Add all available permissions for full system access
    all_perms.add_permission(Permissions::READ_ALL.to_string());
    all_perms.add_permission(Permissions::WRITE_ALL.to_string());
    all_perms.add_permission(Permissions::DELETE_ALL.to_string());
    all_perms.add_permission(Permissions::MANAGE_USERS.to_string());
    all_perms.add_permission(Permissions::DELETE_USERS.to_string());
    all_perms.add_permission(Permissions::MANAGE_SYSTEM.to_string());
    all_perms.add_permission(Permissions::MANAGE_ADMIN.to_string());
    all_perms.add_permission(Permissions::MODERATE_CONTENT.to_string());
    all_perms.add_permission(Permissions::MODERATE_USERS.to_string());
    all_perms.add_permission(Permissions::WRITE_CONTENT.to_string());
    all_perms.add_permission(Permissions::ACCESS_PREMIUM.to_string());
    all_perms.add_permission(Permissions::ACCESS_PREMIUM_FEATURES.to_string());
    all_perms.add_permission(Permissions::READ_PREMIUM.to_string());
    all_perms.add_permission(Permissions::READ_ADVANCED_ANALYTICS.to_string());
    all_perms.add_permission(Permissions::READ_ALL_DATA.to_string());
    all_perms.add_permission(Permissions::WRITE_USER_DATA.to_string());
    all_perms.add_permission(Permissions::READ_USER_REPORTS.to_string());
    
    println!("🔑 Granting ALL SYSTEM PERMISSIONS for super admin access");
    all_perms
  } else {
    // Standard SuperAdmin permissions
    PermSet::for_role(&Role::SuperAdmin)
  };

  // Use reconstruct to create user with SuperAdmin role, preserving all other fields
  let promoted_user = User::reconstruct(
    target_user.id().clone(),
    target_user.firebase_uid().to_string(),
    target_user.email().clone(),
    Role::SuperAdmin,
    permissions,
    target_user.sub().clone(),
    target_user.created_at(),
    Utc::now(), // updated_at
    target_user.deleted_at()
  );

  // Create event manually
  let event = UserRoleChangedEvent::new(
    target_user.id().clone(),
    old_role,
    Role::SuperAdmin
  );

  // Use the promoted user as target
  target_user = promoted_user;

  // Save the user with new role
  if let Err(e) = user_repo.save(&target_user).await {
    eprintln!("❌ Failed to save user: {}", e);
    return Err(format!("Failed to save user: {}", e).into());
  }

  // Dispatch event
  if let Err(e) = event_dispatcher.dispatch(Box::new(event.clone())).await {
    eprintln!("⚠️  Failed to dispatch event: {}", e);
  }

  println!("✅ Successfully promoted user to SuperAdmin!");
  println!("Event ID: {}", event.event_id());
  if let Some(reason) = args.reason {
    println!("Reason: {}", reason);
  }
  return Ok(());
}
