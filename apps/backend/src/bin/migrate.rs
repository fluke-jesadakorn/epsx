// Migration CLI tool for database management
use clap::{Parser, Subcommand};
use epsx::infra::db::MigrationRunner;
use sqlx::postgres::PgPoolOptions;
use epsx::config::env::get_env_var;
use tracing::{info, error};
use tracing_subscriber;

#[derive(Parser)]
#[command(name = "migrate")]
#[command(about = "Database migration tool for EPSX")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Run all pending migrations
    Up,
    /// Check migration status
    Status,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter("migrate=info,epsx_backend=info")
        .init();

    let cli = Cli::parse();

    // Get database URL from environment
    let database_url = get_env_var("DATABASE_URL")
        .or_else(|_| get_env_var("POSTGRES_URL"))
        .unwrap_or_else(|_| {
            error!("DATABASE_URL or POSTGRES_URL environment variable is required");
            std::process::exit(1);
        });

    info!("Connecting to database...");
    
    // Create database connection pool
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await?;

    info!("Database connection established");

    // Get migrations directory
    let migrations_dir = get_env_var("MIGRATIONS_DIR")
        .unwrap_or_else(|_| "migrations".to_string());

    let runner = MigrationRunner::new(pool, migrations_dir);

    // Execute command
    match cli.command {
        Commands::Up => {
            info!("Running migrations...");
            match runner.migrate().await {
                Ok(count) => {
                    if count > 0 {
                        info!("✅ Applied {} migrations successfully", count);
                    } else {
                        info!("✅ Database is up to date");
                    }
                }
                Err(e) => {
                    error!("❌ Migration failed: {}", e);
                    std::process::exit(1);
                }
            }
        }
        Commands::Status => {
            info!("Checking migration status...");
            match runner.status().await {
                Ok(()) => {
                    info!("✅ Migration status check completed");
                }
                Err(e) => {
                    error!("❌ Failed to check migration status: {}", e);
                    std::process::exit(1);
                }
            }
        }
    }

    Ok(())
}

