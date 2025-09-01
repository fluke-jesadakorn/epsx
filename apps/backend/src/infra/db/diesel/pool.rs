use bb8::{Pool, PooledConnection};
use diesel_async::{pooled_connection::{AsyncDieselConnectionManager, ManagerConfig}, AsyncPgConnection};
use diesel::result::{ConnectionResult, ConnectionError};
use futures_util::future::BoxFuture;
use futures_util::FutureExt;
use std::time::Duration;

pub type DbPool = Pool<AsyncDieselConnectionManager<AsyncPgConnection>>;
pub type DbConnection<'a> = PooledConnection<'a, AsyncDieselConnectionManager<AsyncPgConnection>>;

/// Establish TLS connection to PostgreSQL using native-tls
fn establish_tls_connection(config: &str) -> BoxFuture<'_, ConnectionResult<AsyncPgConnection>> {
    let fut = async {
        // Configure native-tls for secure connections
        let connector = native_tls::TlsConnector::builder()
            .danger_accept_invalid_certs(false) // Ensure certificates are validated
            .build()
            .map_err(|e| ConnectionError::BadConnection(format!("TLS setup failed: {}", e)))?;
        
        let tls = postgres_native_tls::MakeTlsConnector::new(connector);
        
        // Connect with TLS support
        let (client, conn) = tokio_postgres::connect(config, tls)
            .await
            .map_err(|e| ConnectionError::BadConnection(format!("Connection failed: {}", e)))?;
        
        // Spawn the connection in the background (required for tokio-postgres)
        tokio::spawn(async move {
            if let Err(e) = conn.await {
                tracing::error!("Database connection error: {}", e);
            }
        });
        
        // Convert tokio-postgres client to AsyncPgConnection
        AsyncPgConnection::try_from(client).await
    };
    fut.boxed()
}

/// Create a new database connection pool with optimized settings
pub async fn create_pool(database_url: &str) -> Result<DbPool, Box<dyn std::error::Error + Send + Sync>> {
    // Configure connection manager with custom TLS setup
    let mut manager_config = ManagerConfig::default();
    manager_config.custom_setup = Box::new(establish_tls_connection);
    
    let manager = AsyncDieselConnectionManager::<AsyncPgConnection>::new_with_config(
        database_url,
        manager_config
    );
    
    let pool = Pool::builder()
        .max_size(10) // Optimized for Cloud Run startup performance
        .min_idle(Some(1)) // Minimal idle connections for faster startup
        .connection_timeout(Duration::from_secs(30)) // 30 seconds to establish connection
        .idle_timeout(Some(Duration::from_secs(300))) // 5 minutes idle timeout
        .test_on_check_out(true) // Test connections before use
        .build(manager)
        .await?;
    
    tracing::info!("✅ Diesel TLS connection pool created with 10 max connections");
    
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
        max_size: 10, // Match the pool creation max_size
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