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
    
    // Read configuration from environment variables with defaults
    let max_connections = std::env::var("DATABASE_MAX_CONNECTIONS")
        .unwrap_or_else(|_| "4".to_string())
        .parse::<u32>()
        .unwrap_or(4);
    
    let acquire_timeout = std::env::var("DATABASE_ACQUIRE_TIMEOUT")
        .unwrap_or_else(|_| "60".to_string())
        .parse::<u64>()
        .unwrap_or(60);
    
    let idle_timeout = std::env::var("DATABASE_IDLE_TIMEOUT")
        .unwrap_or_else(|_| "300".to_string())
        .parse::<u64>()
        .unwrap_or(300);

    tracing::info!(
        "Database pool config: max_connections={}, acquire_timeout={}s, idle_timeout={}s", 
        max_connections, acquire_timeout, idle_timeout
    );

    let config = AsyncDieselConnectionManager::<AsyncPgConnection>::new(database_url);
    let pool = Pool::builder()
        .max_size(max_connections)
        .connection_timeout(Duration::from_secs(acquire_timeout))
        .idle_timeout(Some(Duration::from_secs(idle_timeout)))
        .max_lifetime(Some(Duration::from_secs(1800))) // 30 minute max connection lifetime
        .build(config)
        .await
        .map_err(|e| {
            tracing::error!("Failed to build database connection pool: {}", e);
            e
        })?;
    
    // Test the connection pool immediately (optional in production to allow faster startup)
    let skip_test = std::env::var("SKIP_DB_TEST").unwrap_or_else(|_| "false".to_string()) == "true";
    
    if skip_test {
        tracing::info!("Skipping database connection test (SKIP_DB_TEST=true)");
    } else {
        tracing::info!("Testing database connection pool...");
        match pool.get().await {
            Ok(mut conn) => {
                // Test a simple query
                use diesel::sql_query;
                use diesel_async::RunQueryDsl;
                
                match sql_query("SELECT 1").execute(&mut conn).await {
                    Ok(_) => tracing::info!("✅ Database connection test successful"),
                    Err(e) => {
                        tracing::error!("Failed to execute test query: {}", e);
                        return Err(Box::new(e));
                    }
                }
            },
            Err(e) => {
                tracing::error!("Failed to get test connection from pool: {}", e);
                return Err(Box::new(e));
            }
        }
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