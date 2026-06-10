use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fmt;
use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Error)]
pub enum EventError {
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    #[error("Redis error: {0}")]
    Redis(#[from] redis::RedisError),
}

pub type Result<T> = std::result::Result<T, EventError>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DomainEvent {
    pub id: String,
    pub event_type: String,
    pub service: String,
    pub payload: serde_json::Value,
    pub timestamp: DateTime<Utc>,
    pub version: u32,
}

impl DomainEvent {
    pub fn new(service: &str, event_type: &str, payload: serde_json::Value) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            event_type: event_type.to_string(),
            service: service.to_string(),
            payload,
            timestamp: Utc::now(),
            version: 1,
        }
    }
}

impl fmt::Display for DomainEvent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{}] {} from {}", self.event_type, self.id, self.service)
    }
}

pub struct EventStore {
    redis: redis::aio::ConnectionManager,
}

impl EventStore {
    pub async fn new(redis_url: &str) -> Result<Self> {
        let client = redis::Client::open(redis_url)?;
        let conn = client.get_tokio_connection_manager().await?;
        Ok(Self { redis: conn })
    }

    pub async fn publish(&mut self, stream: &str, event: &DomainEvent) -> Result<String> {
        let payload = serde_json::to_vec(event)?;
        let id: String = redis::cmd("XADD")
            .arg(stream)
            .arg("*")
            .arg("event")
            .arg(payload)
            .query_async(&mut self.redis)
            .await?;
        Ok(id)
    }

    pub async fn subscribe(
        &mut self,
        stream: &str,
        group: &str,
        consumer: &str,
    ) -> Result<Vec<DomainEvent>> {
        let result: Vec<(String, Vec<(String, Vec<(String, String)>)>)> = redis::cmd("XREADGROUP")
            .arg("GROUP")
            .arg(group)
            .arg(consumer)
            .arg("COUNT")
            .arg("10")
            .arg("BLOCK")
            .arg("1000")
            .arg("STREAMS")
            .arg(stream)
            .arg(">")
            .query_async(&mut self.redis)
            .await?;

        let mut events = Vec::new();
        for (_key, messages) in result {
            for (_id, fields) in messages {
                for (k, v) in fields {
                    if k == "event" {
                        if let Ok(event) = serde_json::from_str::<DomainEvent>(&v) {
                            events.push(event);
                        }
                    }
                }
            }
        }
        Ok(events)
    }

    pub async fn ack(&mut self, stream: &str, group: &str, id: &str) -> Result<()> {
        let _: () = redis::cmd("XACK")
            .arg(stream)
            .arg(group)
            .arg(id)
            .query_async(&mut self.redis)
            .await?;
        Ok(())
    }
}
