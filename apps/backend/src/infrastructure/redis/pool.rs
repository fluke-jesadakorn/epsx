use redis::{Client, aio::ConnectionManager};
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Clone)]
pub struct RedisPool {
    client: Client,
    manager: Arc<RwLock<Option<ConnectionManager>>>,
}

impl RedisPool {
    pub async fn new(redis_url: &str) -> Result<Self, redis::RedisError> {
        let client = Client::open(redis_url)?;
        let manager = client.get_connection_manager().await?;

        tracing::info!("✅ Redis pool created: url={}", redis_url);

        Ok(Self {
            client,
            manager: Arc::new(RwLock::new(Some(manager))),
        })
    }

    pub async fn get_connection(&self) -> Result<ConnectionManager, redis::RedisError> {
        let manager_guard = self.manager.read().await;
        manager_guard.as_ref()
            .ok_or_else(|| redis::RedisError::from((
                redis::ErrorKind::IoError,
                "No connection available"
            )))
            .cloned()
    }

    pub async fn get_pubsub(&self) -> Result<redis::aio::PubSub, redis::RedisError> {
        let conn = self.client.get_async_connection().await?;
        Ok(conn.into_pubsub())
    }

    pub async fn health_check(&self) -> bool {
        match self.get_connection().await {
            Ok(mut conn) => {
                redis::cmd("PING").query_async::<_, String>(&mut conn).await.is_ok()
            }
            Err(_) => false,
        }
    }
}
