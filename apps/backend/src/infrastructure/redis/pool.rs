use redis::{Client, aio::ConnectionManager};

#[derive(Clone)]
pub struct RedisPool {
    client: Client,
    manager: ConnectionManager,
}

impl RedisPool {
    pub async fn new(redis_url: &str) -> Result<Self, redis::RedisError> {
        let client = Client::open(redis_url)?;
        let manager = client.get_connection_manager().await?;

        tracing::info!("Redis pool created: url={}", redis_url);

        Ok(Self { client, manager })
    }

    pub fn get_connection(&self) -> ConnectionManager {
        self.manager.clone()
    }

    pub async fn get_pubsub(&self) -> Result<redis::aio::PubSub, redis::RedisError> {
        self.client.get_async_pubsub().await
    }

    pub async fn health_check(&self) -> bool {
        let mut conn = self.manager.clone();
        redis::cmd("PING").query_async::<String>(&mut conn).await.is_ok()
    }
}
