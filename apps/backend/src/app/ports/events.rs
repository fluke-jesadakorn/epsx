// Event handling port interfaces
use uuid::Uuid;

use async_trait::async_trait;


use crate::dom::events::DomainEvent;


#[cfg(test)]
use mockall::{automock, predicate::*};


#[async_trait]
#[cfg_attr(test, automock)]
pub trait EventDispatcher: Send + Sync {
    async fn dispatch(&self, event: Box<dyn DomainEvent>) -> Result<(), EventError>;
    async fn dispatch_batch(&self, events: Vec<Box<dyn DomainEvent>>) -> Result<(), EventError>;
}

#[async_trait] 
pub trait EventHandler<T>: Send + Sync
where
    T: DomainEvent,
{
    async fn handle(&self, event: &T) -> Result<(), EventHandlerError>;
}

#[async_trait]
pub trait EventStore: Send + Sync {
    async fn save_event(&self, event: &dyn DomainEvent) -> Result<(), EventStoreError>;
    async fn get_events(&self, aggregate_id: &Uuid) -> Result<Vec<StoredEvent>, EventStoreError>;
    async fn get_events_by_type(&self, event_type: &str) -> Result<Vec<StoredEvent>, EventStoreError>;
}

// Supporting types
#[derive(Debug, Clone)]
pub struct StoredEvent {
    pub event_id: Uuid,
    pub aggregate_id: Uuid,
    pub event_type: String,
    pub event_data: serde_json::Value,
    pub occurred_at: chrono::DateTime<chrono::Utc>,
    pub version: u32,
}

// Error types
#[derive(Debug, thiserror::Error)]
pub enum EventError {
    #[error("Event serialization failed: {0}")]
    SerializationError(String),
    
    #[error("Event dispatch failed: {0}")]
    DispatchError(String),
    
    #[error("No handlers found for event type: {0}")]
    NoHandlersFound(String),
    
    #[error("Handler execution failed: {0}")]
    HandlerError(String),
    
    #[error("Internal error: {0}")]
    InternalError(String),
}

#[derive(Debug, thiserror::Error)]
pub enum EventHandlerError {
    #[error("Handler processing failed: {0}")]
    ProcessingFailed(String),
    
    #[error("External dependency failed: {0}")]
    ExternalDependencyFailed(String),
    
    #[error("Retry limit exceeded")]
    RetryLimitExceeded,
    
    #[error("Invalid event data: {0}")]
    InvalidEventData(String),
}

#[derive(Debug, thiserror::Error)]
pub enum EventStoreError {
    #[error("Event storage failed: {0}")]
    StorageFailed(String),
    
    #[error("Event retrieval failed: {0}")]
    RetrievalFailed(String),
    
    #[error("Serialization error: {0}")]
    SerializationError(String),
    
    #[error("Version conflict")]
    VersionConflict,
    
    #[error("Event not found: {0}")]
    EventNotFound(String),
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    
    #[test]
    fn should_create_stored_event() {
        let event = StoredEvent {
            event_id: Uuid::new_v4(),
            aggregate_id: Uuid::new_v4(),
            event_type: "UserRegistered".to_string(),
            event_data: serde_json::json!({"email": "test@example.com"}),
            occurred_at: Utc::now(),
            version: 1,
        };
        
        assert_eq!(event.event_type, "UserRegistered");
        assert_eq!(event.version, 1);
    }
}