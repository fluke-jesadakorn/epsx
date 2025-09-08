use diesel_async::{AsyncPgConnection, pooled_connection::{bb8::Pool, AsyncDieselConnectionManager}};
use std::time::Duration;

/// Database connection pool type
pub type DbPool = Pool<AsyncPgConnection>;

/// Create a new database connection pool
pub async fn create_pool(database_url: &str) -> Result<DbPool, Box<dyn std::error::Error + Send + Sync>> {
    // Log database URL with password masked
    let masked_url = if let Some(at_pos) = database_url.find('@') {
        let (before_at, after_at) = database_url.split_at(at_pos);
        if let Some(colon_pos) = before_at.rfind(':') {
            format!("{}:***{}", &before_at[..colon_pos], after_at)
        } else {
            database_url.to_string()
        }
    } else {
        database_url.to_string()
    };
    tracing::info!("Creating database connection pool with URL: {}", masked_url);
    
    let config = AsyncDieselConnectionManager::<AsyncPgConnection>::new(database_url);
    let pool = Pool::builder()
        .max_size(10)  // Reduced pool size for stability
        .connection_timeout(Duration::from_secs(10))  // Shorter timeout for faster failure detection
        .idle_timeout(Some(Duration::from_secs(600))) // 10 minute idle timeout
        .max_lifetime(Some(Duration::from_secs(1800))) // 30 minute max connection lifetime
        .build(config)
        .await
        .map_err(|e| {
            tracing::error!("Failed to build database connection pool: {}", e);
            e
        })?;
    
    // Test the connection pool immediately
    tracing::info!("Testing database connection pool...");
    {
        let mut conn = pool.get().await
            .map_err(|e| {
                tracing::error!("Failed to get test connection from pool: {}", e);
                e
            })?;
        
        // Test a simple query
        use diesel::sql_query;
        use diesel_async::RunQueryDsl;
        sql_query("SELECT 1").execute(&mut conn).await
            .map_err(|e| {
                tracing::error!("Failed to execute test query: {}", e);
                e
            })?;
    }
    
    tracing::info!("✅ Database connection pool created and tested successfully");
    Ok(pool)
}

/// Create a test database pool with smaller limits
pub async fn create_test_pool() -> Result<DbPool, Box<dyn std::error::Error + Send + Sync>> {
    let database_url = std::env::var("TEST_DATABASE_URL")
        .unwrap_or_else(|_| "postgresql://localhost/epsx_test".to_string());
    
    let config = AsyncDieselConnectionManager::<AsyncPgConnection>::new(&database_url);
    let pool = Pool::builder()
        .max_size(5)  // Smaller pool for tests
        .connection_timeout(Duration::from_secs(10))
        .build(config)
        .await?;
    Ok(pool)
}