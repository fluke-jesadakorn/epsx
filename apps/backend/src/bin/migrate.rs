//! EPSX Database Migration Tool
//!
//! Migrated from diesel_migrations to sqlx::migrate!
//! Per DB: `DATABASE_URL`, `ANALYTICS_DATABASE_URL`, `PAYMENTS_DATABASE_URL`,
//! `NOTIFICATIONS_DATABASE_URL`. Migrations live in:
//! `migrations/{core,analytics,payments,notifications}` (raw SQL files).

use clap::{Parser, Subcommand};
use sqlx::migrate::MigrateDatabase;
use sqlx::{Connection, PgConnection, Postgres};
use std::env;

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

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    epsx::config::env::load_env();

    let databases = vec![
        ("DATABASE_URL", "migrations/core", "Default/Core"),
        ("ANALYTICS_DATABASE_URL", "migrations/analytics", "Analytics"),
        ("PAYMENTS_DATABASE_URL", "migrations/payments", "Payments"),
        (
            "NOTIFICATIONS_DATABASE_URL",
            "migrations/notifications",
            "Notifications",
        ),
    ];

    match &cli.command {
        Commands::Up => {
            for (env_var, migrations_dir, label) in databases {
                if let Ok(db_url) = env::var(env_var) {
                    println!("\nProcessing {} Database...", label);
                    ensure_database_exists(&db_url).await?;
                    run_migrations(&db_url, migrations_dir).await?;
                } else {
                    println!("Skipping {} Database ({} not set)", label, env_var);
                }
            }
        }
    }

    Ok(())
}

async fn ensure_database_exists(url: &str) -> Result<(), Box<dyn std::error::Error>> {
    if !Postgres::database_exists(url).await.unwrap_or(false) {
        println!("Creating database for '{}'...", url);
        Postgres::create_database(url)
            .await
            .map_err(|e| format!("Failed to create database: {}", e))?;
        println!("Database created successfully.");
    } else {
        println!("Database already exists.");
    }

    Ok(())
}

async fn run_migrations(
    database_url: &str,
    migrations_dir: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let migrator = sqlx::migrate::Migrator::new(std::path::Path::new(migrations_dir))
        .await
        .map_err(|e| format!("Failed to load migrations from {}: {}", migrations_dir, e))?;
    let mut conn = PgConnection::connect(database_url)
        .await
        .map_err(|e| format!("Failed to connect to database: {}", e))?;

    match migrator.run(&mut conn).await {
        Ok(()) => {
            println!("Migrations applied successfully for {}.", migrations_dir);
        }
        Err(e) => {
            eprintln!("Migration failed for {}: {}", migrations_dir, e);
            return Err(e.into());
        }
    }

    Ok(())
}
