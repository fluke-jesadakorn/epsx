use clap::{Parser, Subcommand};
use sqlx::migrate::MigrateDatabase;
use sqlx::{PgPool, Postgres};
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
    /// Run pending migrations
    Up,
    /// Show migration status
    Status,
    /// Create a new migration file
    Add { name: String },
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();
    
    let database_url = env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set");

    println!("Using database: {}", &database_url);

    match &cli.command {
        Commands::Up => {
            run_migrations(&database_url).await?;
        }
        Commands::Status => {
            show_status(&database_url).await?;
        }
        Commands::Add { name } => {
            create_migration(name).await?;
        }
    }

    Ok(())
}

async fn run_migrations(database_url: &str) -> Result<(), Box<dyn std::error::Error>> {
    println!("🔄 Running database migrations...");
    
    // Ensure database exists
    if !Postgres::database_exists(database_url).await? {
        println!("📦 Creating database...");
        Postgres::create_database(database_url).await?;
    }

    // Connect to database
    let pool = PgPool::connect(database_url).await?;
    
    // Run migrations
    let migrator = sqlx::migrate!("./migrations");
    migrator.run(&pool).await?;
    
    println!("✅ Migrations completed successfully!");
    
    pool.close().await;
    Ok(())
}

async fn show_status(database_url: &str) -> Result<(), Box<dyn std::error::Error>> {
    println!("📊 Checking migration status...");
    
    if !Postgres::database_exists(database_url).await? {
        println!("❌ Database does not exist");
        return Ok(());
    }

    let pool = PgPool::connect(database_url).await?;
    
    // Get applied migrations
    let rows = sqlx::query!(
        "SELECT version, description, installed_on FROM _sqlx_migrations ORDER BY version"
    )
    .fetch_all(&pool)
    .await?;

    if rows.is_empty() {
        println!("📭 No migrations have been applied");
    } else {
        println!("📝 Applied migrations:");
        for row in rows {
            println!("  {} - {} (applied: {})", 
                row.version, 
                row.description,
                row.installed_on.format("%Y-%m-%d %H:%M:%S")
            );
        }
    }
    
    pool.close().await;
    Ok(())
}

async fn create_migration(name: &str) -> Result<(), Box<dyn std::error::Error>> {
    use std::fs;
    use chrono::Utc;
    
    let timestamp = Utc::now().format("%Y%m%d_%H%M%S");
    let filename = format!("migrations/{}_{}.sql", timestamp, name.replace(' ', "_"));
    
    let content = format!(
        "-- Migration: {}\n-- Created: {}\n\n-- Add your SQL here\n",
        name,
        Utc::now().format("%Y-%m-%d %H:%M:%S UTC")
    );
    
    fs::write(&filename, content)?;
    println!("📝 Created migration file: {}", filename);
    
    Ok(())
}