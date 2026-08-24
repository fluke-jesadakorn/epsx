//! EPSX Database Migration Tool
//!
//! BIG-BANG: migrated from diesel_migrations to sqlx::migrate!
//! Per DB: `DATABASE_URL`, `ANALYTICS_DATABASE_URL`, `PAYMENTS_DATABASE_URL`,
//! `NOTIFICATIONS_DATABASE_URL`. Migrations live in:
//! `migrations/{core,analytics,payments,notifications}` (raw SQL files).

use clap::{Parser, Subcommand};
use sqlx::{Connection, Executor, PgConnection, migrate::MigrateDatabase};
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

fn main() -> Result<(), Box<dyn std::error::Error>> {
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
                    ensure_database_exists(&db_url)?;
                    run_migrations(&db_url, migrations_dir)?;
                } else {
                    println!("Skipping {} Database ({} not set)", label, env_var);
                }
            }
        }
    }

    Ok(())
}

fn ensure_database_exists(url: &str) -> Result<(), Box<dyn std::error::Error>> {
    let (base_url, db_name) = split_url_db(url)?;
    let mut conn = PgConnection::connect(&base_url)
        .await
        .map_err(|e| format!("Failed to connect to base postgres database: {}", e))?;

    let row: (bool,) = sqlx::query_as("SELECT EXISTS(SELECT 1 FROM pg_database WHERE datname = $1)")
        .bind(&db_name)
        .fetch_one(&mut conn)
        .await
        .map_err(|e| format!("Failed to query pg_database: {}", e))?;

    if !row.0 {
        println!("Creating database '{}'...", db_name);
        let create_stmt = format!("CREATE DATABASE \"{}\"", db_name);
        conn.execute(create_stmt.as_str())
            .await
            .map_err(|e| format!("Failed to create database: {}", e))?;
        println!("Database created successfully.");
    } else {
        println!("Database '{}' already exists.", db_name);
    }

    Ok(())
}

fn split_url_db(url: &str) -> Result<(String, String), Box<dyn std::error::Error>> {
    let parts: Vec<&str> = url.rsplitn(2, '/').collect();
    if parts.len() != 2 {
        return Err("Invalid database URL format".into());
    }
    let db_name = parts[0].split('?').next().unwrap_or(parts[0]);
    let base = parts[1];
    Ok((format!("{}/postgres", base), db_name.to_string()))
}

async fn run_migrations(
    database_url: &str,
    migrations_dir: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut migrator = sqlx::migrate::Migrator::new(std::path::Path::new(migrations_dir))
        .await
        .map_err(|e| format!("Failed to load migrations from {}: {}", migrations_dir, e))?;
    let mut conn = PgConnection::connect(database_url)
        .await
        .map_err(|e| format!("Failed to connect to database: {}", e))?;

    match migrator.run(&mut conn).await {
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
            return Err(e.into());
        }
    }

    Ok(())
}

// Re-export db creation helper
use sqlx::migrate::MigrateDatabase as _;
