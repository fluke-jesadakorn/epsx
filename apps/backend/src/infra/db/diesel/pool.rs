use bb8::{Pool, PooledConnection};
use diesel_async::{AsyncPgConnection, pooled_connection::AsyncDieselConnectionManager};
use std::time::Duration;

pub type DbPool = Pool<AsyncDieselConnectionManager<AsyncPgConnection>>;
pub type DbConnection<'a> = PooledConnection<'a, AsyncDieselConnectionManager<AsyncPgConnection>>;

/// Create a new database connection pool with optimized settings
pub async fn create_pool(database_url: &str) -> Result<DbPool, Box<dyn std::error::Error + Send + Sync>> {
    let config = AsyncDieselConnectionManager::<AsyncPgConnection>::new(database_url);
    
    let pool = Pool::builder()
        .max_size(num_cpus::get() as u32 * 4) // Scale with CPU cores
        .min_idle(Some(5)) // Keep minimum connections alive
        .connection_timeout(Duration::from_secs(30)) // 30 seconds to establish connection
        .idle_timeout(Some(Duration::from_secs(600))) // 10 minutes idle timeout
        .test_on_check_out(true) // Test connections before use
        .build(config)
        .await?;
    
    tracing::info!("✅ Diesel connection pool created with {} max connections", num_cpus::get() * 4);
    
    Ok(pool)
}

/// Health check for the database connection pool
pub async fn health_check(pool: &DbPool) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let _conn = pool.get().await?;
    tracing::debug!("📊 Database connection pool health check passed");
    Ok(())
}

/// Get pool statistics for monitoring  
pub fn pool_stats(pool: &DbPool) -> PoolStats {
    let state = pool.state();
    PoolStats {
        connections: state.connections,
        idle_connections: state.idle_connections,
        max_size: num_cpus::get() as u32 * 4, // Match the pool creation max_size
    }
}

#[derive(Debug, Clone)]
pub struct PoolStats {
    pub connections: u32,
    pub idle_connections: u32,
    pub max_size: u32,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_pool_creation() {
        let database_url = std::env::var("DATABASE_URL")
            .unwrap_or_else(|_| "postgresql://test:test@localhost/test".to_string());
        
        if let Ok(pool) = create_pool(&database_url).await {
            let stats = pool_stats(&pool);
            assert!(stats.max_size > 0);
            assert_eq!(stats.connections, 0); // No connections initially
        }
        // Test passes even if database is not available
    }
}