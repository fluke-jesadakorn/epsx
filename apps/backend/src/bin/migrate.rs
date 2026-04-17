use clap::{Parser, Subcommand};
use std::env;
use diesel::prelude::*;
use diesel::pg::PgConnection;
use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};

// Embed migrations for each domain
pub const MIGRATIONS_CORE: EmbeddedMigrations = embed_migrations!("migrations/core");
pub const MIGRATIONS_ANALYTICS: EmbeddedMigrations = embed_migrations!("migrations/analytics");
pub const MIGRATIONS_PAYMENTS: EmbeddedMigrations = embed_migrations!("migrations/payments");
pub const MIGRATIONS_NOTIFICATIONS: EmbeddedMigrations = embed_migrations!("migrations/notifications");

#[derive(Parser)]
#[command(name = "migrate")]
#[command(about = "EPSX Database Migration Tool")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Run pending migrations for all configured databases
    Up,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();
    
    epsx::config::env::load_env();
    
    // Define database configurations: (Env Var, Migration Source, "Label")
    let databases = vec![
        ("DATABASE_URL", MIGRATIONS_CORE, "Default/Core"),
        ("ANALYTICS_DATABASE_URL", MIGRATIONS_ANALYTICS, "Analytics"),
        ("PAYMENTS_DATABASE_URL", MIGRATIONS_PAYMENTS, "Payments"),
        ("NOTIFICATIONS_DATABASE_URL", MIGRATIONS_NOTIFICATIONS, "Notifications"),
    ];

    match &cli.command {
        Commands::Up => {
             for (env_var, migrations, label) in databases {
                 if let Ok(db_url) = env::var(env_var) {
                     println!("\n� Processing {} Database...", label);
                     // Create DB if not exists (using postgres maintenance DB)
                     ensure_database_exists(&db_url)?;
                     // Run migrations
                     run_migrations(&db_url, migrations)?;
                 } else {
                     println!("Skipping {} Database ({} not set)", label, env_var);
                 }
             }
        }
    }

    Ok(())
}

fn ensure_database_exists(url: &str) -> Result<(), Box<dyn std::error::Error>> {
    // Parse URL to check DB name and connect to 'postgres' DB
    let (base_url, db_name) = split_url_db(url)?;
    
    let mut conn = PgConnection::establish(&base_url)
        .map_err(|e| format!("Failed to connect to base postgres database: {}", e))?;

    // Check if database exists
    let exists: bool = diesel::dsl::sql::<diesel::sql_types::Bool>(
        &format!("SELECT EXISTS(SELECT 1 FROM pg_database WHERE datname = '{}')", db_name)
    ).get_result(&mut conn)?;

    if !exists {
        println!("Creating database '{}'...", db_name);
        diesel::dsl::sql::<diesel::sql_types::Integer>(
            &format!("CREATE DATABASE \"{}\"", db_name)
        ).execute(&mut conn)?;
        println!("Database created successfully.");
    } else {
        println!("Database '{}' already exists.", db_name);
    }

    Ok(())
}

// Helper to strip DB name and return (base_url_pointing_to_postgres, db_name)
fn split_url_db(url: &str) -> Result<(String, String), Box<dyn std::error::Error>> {
    let parts: Vec<&str> = url.rsplitn(2, '/').collect();
    if parts.len() != 2 {
        return Err("Invalid database URL format".into());
    }
    let db_name = parts[0].split('?').next().unwrap_or(parts[0]); // Handle ?sslmode...
    let base = parts[1];
    Ok((format!("{}/postgres", base), db_name.to_string()))
}

fn run_migrations(database_url: &str, migrations: EmbeddedMigrations) -> Result<(), Box<dyn std::error::Error>> {
    let mut conn = PgConnection::establish(database_url)
        .map_err(|e| format!("Failed to connect to database: {}", e))?;
    
    match conn.run_pending_migrations(migrations) {
        Ok(applied) => {
            if applied.is_empty() {
                println!("Schema is up to date.");
            } else {
                println!("Applied {} migrations:", applied.len());
                for m in applied {
                    println!("  - {}", m);
                }
            }
        }
        Err(e) => {
            eprintln!("Migration failed: {}", e);
            return Err(e);
        }
    }
    
    Ok(())
}
