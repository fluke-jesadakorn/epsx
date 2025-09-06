use chrono::{DateTime, Utc};// Database Schema Management Tool for EPSX
// Modern consolidated database schema management with Diesel migrations
// Optimized for production deployment and development workflows

use clap::{Parser, Subcommand};
use std::env;
use tracing::{info, warn};

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
    /// Initialize database with modern consolidated schema
    Init,
    /// Check database connection and schema status
    Status,
    /// Drop all tables and recreate with fresh schema (WARNING: destructive)
    Reset,
    /// Run pending migrations only
    Up,
    /// Rollback last migration
    Down,
    /// Show migration history
    History,
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
        Commands::Up => {
            run_migrations(&database_url)?;
        }
        Commands::Down => {
            rollback_migration(&database_url)?;
        }
        Commands::History => {
            show_migration_history(&database_url)?;
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

    info!("📊 Modern consolidated database includes:");
    info!("   - Streamlined user management (Firebase + JWT)");
    info!("   - Simplified admin permission system (8 core modules)");
    info!("   - Optimized security infrastructure");
    info!("   - EPS analytics with sample data");
    info!("   - 25 focused performance indexes");
    info!("   - JWT helper functions & optimized views");
    info!("   - Production-ready seed data");

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
        "users", "sessions", "firebase_sessions",
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

    // Check data counts
    #[derive(diesel::QueryableByName)]
    struct CountResult {
        #[diesel(sql_type = BigInt)]
        count: i64,
    }

    // Admin modules table removed - using structured permissions now

    let user_count = sql_query("SELECT COUNT(*) as count FROM users")
        .get_result::<CountResult>(&mut connection)
        .map(|r| r.count)
        .unwrap_or(0);

    let eps_count = sql_query("SELECT COUNT(*) as count FROM eps_growth_analytics")
        .get_result::<CountResult>(&mut connection)
        .map(|r| r.count)
        .unwrap_or(0);

    info!("📊 Data Summary:");
    info!("   - Users: {}", user_count);
    info!("   - EPS analytics samples: {}", eps_count);

    Ok(())
}

#[cfg(not(feature = "database"))]
fn check_status(_database_url: &str) -> Result<(), Box<dyn std::error::Error>> {
    error!("Database feature not enabled. Compile with --features database");
    Err("Database feature not enabled".into())
}

#[cfg(feature = "database")]
fn run_migrations(database_url: &str) -> Result<(), Box<dyn std::error::Error>> {
    info!("Running pending migrations...");
    
    let mut connection = PgConnection::establish(database_url)
        .map_err(|e| format!("Failed to connect to database: {}", e))?;

    let migration_versions = connection.run_pending_migrations(MIGRATIONS)
        .map_err(|e| format!("Failed to run migrations: {}", e))?;

    if migration_versions.is_empty() {
        info!("✅ No pending migrations - database is up to date!");
    } else {
        info!("✅ Applied {} migrations successfully!", migration_versions.len());
        for version in &migration_versions {
            info!("   - Applied: {}", version);
        }
    }

    Ok(())
}

#[cfg(feature = "database")]
fn rollback_migration(database_url: &str) -> Result<(), Box<dyn std::error::Error>> {
    warn!("⚠️  Rolling back last migration...");
    
    let mut connection = PgConnection::establish(database_url)
        .map_err(|e| format!("Failed to connect to database: {}", e))?;

    connection.revert_last_migration(MIGRATIONS)
        .map_err(|e| format!("Failed to rollback migration: {}", e))?;

    info!("✅ Migration rollback completed");
    Ok(())
}

#[cfg(feature = "database")]
fn show_migration_history(database_url: &str) -> Result<(), Box<dyn std::error::Error>> {
    use diesel::sql_query;
    use diesel::sql_types::*;

    info!("📜 Migration History:");
    
    let mut connection = PgConnection::establish(database_url)
        .map_err(|e| format!("Failed to connect to database: {}", e))?;

    #[derive(diesel::QueryableByName)]
    struct MigrationRecord {
        #[diesel(sql_type = Text)]
        version: String,
        #[diesel(sql_type = Timestamp)]
        run_on: chrono::NaiveDateTime,
    }

    #[derive(diesel::QueryableByName)]
    struct TableExists {
        #[diesel(sql_type = Bool)]
        exists: bool,
    }

    // Check if __diesel_schema_migrations table exists
    let table_exists = sql_query("SELECT EXISTS (SELECT FROM information_schema.tables WHERE table_name = '__diesel_schema_migrations') as exists")
        .get_result::<TableExists>(&mut connection)
        .map(|result| result.exists)
        .unwrap_or(false);

    if !table_exists {
        warn!("   No migration history found - database not initialized");
        return Ok(());
    }

    let migrations: Vec<MigrationRecord> = sql_query(
        "SELECT version, run_on FROM __diesel_schema_migrations ORDER BY run_on DESC"
    )
    .load(&mut connection)
    .unwrap_or_default();

    if migrations.is_empty() {
        info!("   No migrations have been run");
    } else {
        for migration in migrations {
            info!("   ✅ {} (applied: {})", migration.version, migration.run_on.format("%Y-%m-%d %H:%M:%S"));
        }
    }

    Ok(())
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

    // Automatically reinitialize with fresh schema
    info!("🔄 Reinitializing with fresh schema...");
    init_database(database_url)?;

    Ok(())
}

#[cfg(not(feature = "database"))]
fn run_migrations(_database_url: &str) -> Result<(), Box<dyn std::error::Error>> {
    error!("Database feature not enabled. Compile with --features database");
    Err("Database feature not enabled".into())
}

#[cfg(not(feature = "database"))]
fn rollback_migration(_database_url: &str) -> Result<(), Box<dyn std::error::Error>> {
    error!("Database feature not enabled. Compile with --features database");
    Err("Database feature not enabled".into())
}

#[cfg(not(feature = "database"))]
fn show_migration_history(_database_url: &str) -> Result<(), Box<dyn std::error::Error>> {
    error!("Database feature not enabled. Compile with --features database");
    Err("Database feature not enabled".into())
}

#[cfg(not(feature = "database"))]
fn reset_database(_database_url: &str) -> Result<(), Box<dyn std::error::Error>> {
    error!("Database feature not enabled. Compile with --features database");
    Err("Database feature not enabled".into())
}