// Event Bus Port - Hexagonal Architecture Outbound Port
use async_trait::async_trait;
// Defines interface for publishing domain events to external event infrastructure

use std::sync::Arc;
use uuid::Uuid;

use crate::domain::shared_kernel::DomainEvent;

/// Result type for event publishing operations
pub type EventPublishResult = Result<EventReceipt, EventBusError>;

/// Receipt returned after successful event publishing
#[derive(Debug, Clone)]
pub struct EventReceipt {
    pub event_id: Uuid,
    pub published_at: chrono::DateTime<chrono::Utc>,
    pub topic: String,
    pub sequence_number: Option<u64>,
}

/// Errors that can occur during event publishing
#[derive(Debug, thiserror::Error)]
pub enum EventBusError {
    #[error("Event serialization failed: {0}")]
    SerializationError(String),
    
    #[error("Event publishing failed: {0}")]
    PublishingError(String),
    
    #[error("Event bus unavailable: {0}")]
    UnavailableError(String),
    
    #[error("Event validation failed: {0}")]
    ValidationError(String),
    
    #[error("Topic not found: {0}")]
    TopicNotFound(String),
    
    #[error("Rate limit exceeded")]
    RateLimitExceeded,
    
    #[error("Event too large: {size} bytes")]
    EventTooLarge { size: usize },
}

/// Port interface for event bus operations (Hexagonal Architecture)
/// This is the boundary between application and infrastructure layers
#[async_trait]
pub trait EventBusPort: Send + Sync {
    /// Publish a single domain event
    async fn publish(&self, event: Arc<dyn DomainEvent>) -> EventPublishResult;
    
    /// Publish multiple events as a batch (transactional)
    async fn publish_batch(&self, events: Vec<Arc<dyn DomainEvent>>) -> Result<Vec<EventReceipt>, EventBusError>;
    
    /// Publish event to specific topic/channel
    async fn publish_to_topic(
        &self, 
        event: Arc<dyn DomainEvent>, 
        topic: &str
    ) -> EventPublishResult;
    
    /// Check if event bus is healthy and available
    async fn health_check(&self) -> Result<EventBusHealthStatus, EventBusError>;
    
    /// Get event bus metrics
    async fn metrics(&self) -> Result<EventBusMetrics, EventBusError>;
}

/// Health status of event bus
#[derive(Debug, Clone)]
pub struct EventBusHealthStatus {
    pub is_healthy: bool,
    pub latency_ms: Option<u64>,
    pub connection_count: Option<u32>,
    pub queue_depth: Option<u64>,
    pub last_error: Option<String>,
}

/// Event bus operational metrics
#[derive(Debug, Clone)]
pub struct EventBusMetrics {
    pub events_published: u64,
    pub events_failed: u64,
    pub average_latency_ms: f64,
    pub current_queue_depth: u64,
    pub uptime_seconds: u64,
}

/// Event routing strategy
#[derive(Debug, Clone)]
pub enum EventRoutingStrategy {
    /// Route by event type
    ByEventType,
    /// Route by aggregate type
    ByAggregateType,  
    /// Route by bounded context
    ByBoundedContext,
    /// Custom routing based on event content
    Custom(fn(&dyn DomainEvent) -> String),
}

/// Event publishing options
#[derive(Debug, Clone)]
pub struct PublishOptions {
    /// Routing strategy to use
    pub routing_strategy: EventRoutingStrategy,
    /// Maximum retry attempts
    pub max_retries: u32,
    /// Timeout for publishing operation
    pub timeout_ms: u64,
    /// Whether to wait for acknowledgment
    pub wait_for_ack: bool,
    /// Event priority level
    pub priority: EventPriority,
}

/// Event priority levels
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EventPriority {
    Low,
    Normal, 
    High,
    Critical,
}

impl Default for PublishOptions {
    fn default() -> Self {
        Self {
            routing_strategy: EventRoutingStrategy::ByEventType,
            max_retries: 3,
            timeout_ms: 5000,
            wait_for_ack: true,
            priority: EventPriority::Normal,
        }
    }
}

/// Advanced event bus port with additional capabilities
#[async_trait]
pub trait AdvancedEventBusPort: EventBusPort {
    /// Publish with custom options
    async fn publish_with_options(
        &self,
        event: Arc<dyn DomainEvent>,
        options: PublishOptions,
    ) -> EventPublishResult;
    
    /// Schedule event for future publishing
    async fn schedule_event(
        &self,
        event: Arc<dyn DomainEvent>,
        schedule_at: chrono::DateTime<chrono::Utc>,
    ) -> EventPublishResult;
    
    /// Cancel scheduled event
    async fn cancel_scheduled_event(&self, receipt: &EventReceipt) -> Result<(), EventBusError>;
}

/// Event subscriber port for receiving events (if needed)
#[async_trait]
pub trait EventSubscriberPort: Send + Sync {
    /// Subscribe to events of specific type
    async fn subscribe_to_event_type(
        &self,
        event_type: &str,
        handler: Box<dyn EventHandler>,
    ) -> Result<SubscriptionId, EventBusError>;
    
    /// Unsubscribe from events
    async fn unsubscribe(&self, subscription_id: SubscriptionId) -> Result<(), EventBusError>;
}

/// Event handler interface
#[async_trait]
pub trait EventHandler: Send + Sync {
    async fn handle(&self, event: Arc<dyn DomainEvent>) -> Result<(), Box<dyn std::error::Error>>;
}

/// Subscription identifier
pub type SubscriptionId = Uuid;

// Re-export commonly used types
pub use EventBusPort as DomainEventBusPort;
pub use EventBusError as DomainEventBusError;