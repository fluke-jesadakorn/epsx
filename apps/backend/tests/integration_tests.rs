// Integration tests for EPSX backend
use epsx::infra::db::MigrationRunner;
use sqlx::postgres::PgPoolOptions;
use std::env;
use tokio;

#[tokio::test]
async fn test_migration_system() {
    // Skip if no test database URL is provided
    let database_url = match env::var("TEST_DATABASE_URL") {
        Ok(url) => url,
        Err(_) => {
            println!("Skipping migration test - no TEST_DATABASE_URL set");
            return;
        }
    };

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .expect("Failed to connect to test database");

    let runner = MigrationRunner::new(pool, "migrations".to_string());

    // Test initialization
    let result = runner.init().await;
    assert!(result.is_ok(), "Migration initialization should succeed");

    // Test status check
    let result = runner.status().await;
    assert!(result.is_ok(), "Status check should succeed");
}

#[tokio::test]
async fn test_database_connection() {
    let database_url = match env::var("TEST_DATABASE_URL") {
        Ok(url) => url,
        Err(_) => {
            println!("Skipping database connection test - no TEST_DATABASE_URL set");
            return;
        }
    };

    let pool = PgPoolOptions::new()
        .max_connections(1)
        .connect(&database_url)
        .await;

    assert!(pool.is_ok(), "Database connection should succeed");
}