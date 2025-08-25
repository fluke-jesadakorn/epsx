// Database Schema Initialization Tool for EPSX
// Manages the consolidated database schema with Diesel

use clap::{Parser, Subcommand};
use std::env;
use tracing::{info, error, warn};

#[cfg(feature = "database")]
use diesel::{Connection, PgConnection, RunQueryDsl};

#[cfg(feature = "database")]
use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};

#[derive(Parser)]
#[command(name = "migrate")]
#[command(about = "Database schema initialization tool for EPSX")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize database with consolidated schema
    Init,
    /// Check database connection and schema status
    Status,
    /// Drop all tables (WARNING: destructive)
    Reset,
}

const MIGRATIONS: EmbeddedMigrations = embed_migrations!("diesel_migrations");

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter("migrate=info,epsx_backend=info")
        .init();

    let cli = Cli::parse();

    // Get database URL from environment
    let database_url = env::var("DATABASE_URL")
        .map_err(|_| "DATABASE_URL environment variable not set")?;

    match cli.command {
        Commands::Init => {
            init_database(&database_url)?;
        }
        Commands::Status => {
            check_status(&database_url)?;
        }
        Commands::Reset => {
            reset_database(&database_url)?;
        }
    }

    Ok(())
}

#[cfg(feature = "database")]
fn init_database(database_url: &str) -> Result<(), Box<dyn std::error::Error>> {
    info!("Initializing database with Diesel migrations...");

    // Connect to database
    let mut connection = PgConnection::establish(database_url)
        .map_err(|e| format!("Failed to connect to database: {}", e))?;

    info!("Connected to database, running migrations...");

    // Run all pending migrations
    let migration_versions = connection.run_pending_migrations(MIGRATIONS)
        .map_err(|e| format!("Failed to run migrations: {}", e))?;

    if migration_versions.is_empty() {
        info!("✅ No pending migrations - database is up to date!");
    } else {
        info!("✅ Applied {} migrations successfully!", migration_versions.len());
        for version in &migration_versions {
            info!("   - Migration: {}", version);
        }
    }

    info!("📊 Database includes:");
    info!("   - Core user management tables");
    info!("   - Admin module permission system");
    info!("   - Security infrastructure");
    info!("   - Analytics tables");
    info!("   - 50+ performance indexes");
    info!("   - JWT helper functions");
    info!("   - Initial seed data");

    Ok(())
}

#[cfg(not(feature = "database"))]
fn init_database(_database_url: &str) -> Result<(), Box<dyn std::error::Error>> {
    error!("Database feature not enabled. Compile with --features database");
    Err("Database feature not enabled".into())
}

#[cfg(feature = "database")]
fn check_status(database_url: &str) -> Result<(), Box<dyn std::error::Error>> {
    use diesel::sql_query;
    use diesel::sql_types::*;

    info!("Checking database status...");

    let mut connection = PgConnection::establish(database_url)
        .map_err(|e| format!("Failed to connect to database: {}", e))?;

    // Check if key tables exist
    let tables = vec![
        "users", "admin_modules", "sessions", "firebase_sessions",
        "security_events", "audit_logs", "notifications", "eps_growth_analytics"
    ];

    info!("🔍 Checking core tables:");
    for table in tables {
        #[derive(diesel::QueryableByName)]
        struct TableExists {
            #[diesel(sql_type = Bool)]
            exists: bool,
        }

        let query = format!(
            "SELECT EXISTS (SELECT FROM information_schema.tables WHERE table_name = '{}') as exists",
            table
        );

        let result: Result<TableExists, _> = sql_query(&query).get_result(&mut connection);

        match result {
            Ok(result) if result.exists => info!("   ✅ {} - exists", table),
            Ok(_) => warn!("   ❌ {} - missing", table),
            Err(e) => warn!("   ❓ {} - error checking: {}", table, e),
        }
    }

    // Check admin modules data
    #[derive(diesel::QueryableByName)]
    struct CountResult {
        #[diesel(sql_type = BigInt)]
        count: i64,
    }

    let admin_count = sql_query("SELECT COUNT(*) as count FROM admin_modules WHERE is_active = true")
        .get_result::<CountResult>(&mut connection)
        .map(|r| r.count)
        .unwrap_or(0);

    info!("📊 Admin modules configured: {}", admin_count);

    // Check EPS analytics sample data
    let eps_count = sql_query("SELECT COUNT(*) as count FROM eps_growth_analytics")
        .get_result::<CountResult>(&mut connection)
        .map(|r| r.count)
        .unwrap_or(0);

    info!("📈 EPS analytics samples: {}", eps_count);

    Ok(())
}

#[cfg(not(feature = "database"))]
fn check_status(_database_url: &str) -> Result<(), Box<dyn std::error::Error>> {
    error!("Database feature not enabled. Compile with --features database");
    Err("Database feature not enabled".into())
}

#[cfg(feature = "database")]
fn reset_database(database_url: &str) -> Result<(), Box<dyn std::error::Error>> {
    use diesel::sql_query;

    warn!("⚠️  DESTRUCTIVE OPERATION: Resetting database...");
    warn!("This will drop ALL tables and data!");

    let mut connection = PgConnection::establish(database_url)
        .map_err(|e| format!("Failed to connect to database: {}", e))?;

    // Drop all tables (cascade will handle dependencies)
    let drop_sql = r#"
        DROP SCHEMA public CASCADE;
        CREATE SCHEMA public;
        GRANT ALL ON SCHEMA public TO postgres;
        GRANT ALL ON SCHEMA public TO public;
    "#;

    sql_query(drop_sql)
        .execute(&mut connection)
        .map_err(|e| format!("Failed to reset database: {}", e))?;

    info!("✅ Database reset completed");
    info!("💡 Run 'migrate init' to recreate schema");

    Ok(())
}

#[cfg(not(feature = "database"))]
fn reset_database(_database_url: &str) -> Result<(), Box<dyn std::error::Error>> {
    error!("Database feature not enabled. Compile with --features database");
    Err("Database feature not enabled".into())
}