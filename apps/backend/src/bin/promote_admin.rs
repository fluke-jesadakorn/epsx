// Promote User to Full Admin CLI Tool
// Assigns all available admin modules to a user using Diesel ORM

use clap::Parser;
use std::env;
use tracing::{info, error};
use uuid::Uuid;
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use chrono::Utc;

use epsx::infra::db::diesel::{create_pool, schema::*};
use epsx::infra::db::diesel::types::AdminModule;

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

// All available admin modules (matching the Diesel enum)
const ALL_ADMIN_MODULES: &[AdminModule] = &[
    AdminModule::UserManagement,
    AdminModule::Analytics, 
    AdminModule::Security,
    AdminModule::Billing,
    AdminModule::Settings,
    AdminModule::ContentManagement,
    AdminModule::SupportAccess,
    AdminModule::ApiManagement,
];

#[derive(Insertable)]
#[diesel(table_name = user_admin_roles)]
struct NewUserAdminRole {
    id: Uuid,
    firebase_uid: String,
    module_code: AdminModule,
    granted_by: Option<String>,
    expires_at: Option<chrono::DateTime<Utc>>,
    is_active: Option<bool>,
    created_at: Option<chrono::DateTime<Utc>>,
    updated_at: Option<chrono::DateTime<Utc>>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    tracing_subscriber::fmt().init();
    
    let args = Args::parse();
    
    // Get database URL
    let database_url = env::var("DATABASE_URL")
        .map_err(|_| "DATABASE_URL environment variable is required")?;
    
    info!("Promoting user to full admin: {}", args.email);
    
    // Create Diesel connection pool
    let pool = create_pool(&database_url).await.map_err(|e| {
        error!("Failed to create database pool: {}", e);
        e
    })?;
    let mut conn = pool.get().await.map_err(|e| {
        error!("Failed to get database connection: {}", e);
        e
    })?;
    
    // First check if user exists and get firebase_uid
    let user_result = users::table
        .select(users::firebase_uid)
        .filter(users::email.eq(&args.email))
        .first::<String>(&mut conn)
        .await;
    
    let firebase_uid = match user_result {
        Ok(uid) => uid,
        Err(diesel::result::Error::NotFound) => {
            error!("User with email {} not found in database", args.email);
            return Err("User not found".into());
        }
        Err(e) => {
            error!("Database error while looking up user {}: {}", args.email, e);
            return Err(e.into());
        }
    };
    
    info!("Found user with firebase_uid: {}", firebase_uid);
    info!("Assigning {} admin modules to user", ALL_ADMIN_MODULES.len());
    
    // Assign all admin modules
    let mut success_count = 0;
    let mut error_count = 0;
    let now = Utc::now();
    
    for module in ALL_ADMIN_MODULES {
        info!("Assigning module: {:?}", module);
        
        let new_role = NewUserAdminRole {
            id: Uuid::new_v4(),
            firebase_uid: firebase_uid.clone(),
            module_code: module.clone(),
            granted_by: Some("CLI_PROMOTE_ADMIN".to_string()),
            expires_at: None, // No expiration
            is_active: Some(true),
            created_at: Some(now),
            updated_at: Some(now),
        };
        
        // Use INSERT ... ON CONFLICT to handle duplicates
        let result = diesel::insert_into(user_admin_roles::table)
            .values(&new_role)
            .on_conflict((user_admin_roles::firebase_uid, user_admin_roles::module_code))
            .do_update()
            .set((
                user_admin_roles::is_active.eq(true),
                user_admin_roles::granted_by.eq("CLI_PROMOTE_ADMIN"),
                user_admin_roles::updated_at.eq(now),
            ))
            .execute(&mut conn)
            .await;
        
        match result {
            Ok(_) => {
                info!("✅ Successfully assigned module '{:?}'", module);
                success_count += 1;
            },
            Err(e) => {
                error!("❌ Failed to assign module '{:?}': {}", module, e);
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
