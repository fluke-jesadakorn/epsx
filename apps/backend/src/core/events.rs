// Enhanced event system for microservices-ready patterns

use crate::prelude::*;

use std::collections::HashMap;
use crate::core::errors::ErrorKind;

/// Enhanced domain event trait
pub trait DomainEvent: Send + Sync + Debug {
    /// Event type identifier
    fn event_type(&self) -> &'static str;
    
    /// Aggregate ID that generated this event
    fn aggregate_id(&self) -> String;
    
    /// Event version for schema evolution
    fn version(&self) -> u32;
    
    /// When the event occurred
    fn occurred_at(&self) -> DateTime<Utc>;
    
    /// Serialize event to JSON
    fn to_json(&self) -> Result<String, serde_json::Error>;
    
    /// Event metadata
    fn metadata(&self) -> &HashMap<String, String>;
    
    /// Downcast to Any for type introspection
    fn as_any(&self) -> &dyn std::any::Any;
}

/// Event envelope for transport
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventEnvelope {
    /// Unique event identifier
    pub event_id: String,
    /// Event type
    pub event_type: String,
    /// Aggregate identifier
    pub aggregate_id: String,
    /// Aggregate type
    pub aggregate_type: String,
    /// Event version
    pub version: u32,
    /// Event data (JSON)
    pub data: serde_json::Value,
    /// Event metadata
    pub metadata: HashMap<String, String>,
    /// When event occurred
    pub occurred_at: DateTime<Utc>,
    /// Correlation ID for tracing
    pub correlation_id: String,
    /// Causation ID (event that caused this)
    pub causation_id: Option<String>,
}

/// Stored event with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoredEvent {
    pub envelope: EventEnvelope,
    pub stream_version: u64,
    pub global_position: u64,
    pub stored_at: DateTime<Utc>,
}

/// Event stream
#[derive(Debug)]
pub struct EventStream {
    pub stream_id: String,
    pub events: Vec<StoredEvent>,
    pub version: u64,
    pub is_deleted: bool,
}

/// Aggregate snapshot for performance
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Snapshot {
    pub aggregate_id: String,
    pub aggregate_type: String,
    pub version: u64,
    pub data: serde_json::Value,
    pub created_at: DateTime<Utc>,
}

/// Event handler trait
#[async_trait]
pub trait EventHandler: Send + Sync {
    /// Handle a single event
    async fn handle(&self, event: &StoredEvent) -> AppResult<()>;
    
    /// Get supported event types
    fn event_types(&self) -> Vec<String>;
    
    /// Handler name for identification
    fn name(&self) -> &str;
}

/// Event subscription
pub struct Subscription {
    pub id: String,
    pub event_types: Vec<String>,
    pub created_at: DateTime<Utc>,
    pub is_active: bool,
}

/// Circuit breaker for resilient event processing
pub struct CircuitBreaker {
    state: CircuitState,
    failure_count: std::sync::atomic::AtomicU32,
    last_failure: std::sync::atomic::AtomicU64,
    config: CircuitBreakerConfig,
}

#[derive(Debug, Clone, PartialEq)]
pub enum CircuitState {
    Closed,
    Open,
    HalfOpen,
}

#[derive(Debug, Clone)]
pub struct CircuitBreakerConfig {
    pub failure_threshold: u32,
    pub recovery_timeout: std::time::Duration,
    pub success_threshold: u32,
}

impl CircuitBreaker {
    pub fn new(config: CircuitBreakerConfig) -> Self {
        Self {
            state: CircuitState::Closed,
            failure_count: std::sync::atomic::AtomicU32::new(0),
            last_failure: std::sync::atomic::AtomicU64::new(0),
            config,
        }
    }
    
    pub async fn call<F, T>(&self, f: F) -> AppResult<T>
    where
        F: std::future::Future<Output = AppResult<T>>,
    {
        match self.state {
            CircuitState::Open => {
                let now = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs();
                let last_failure = self.last_failure.load(std::sync::atomic::Ordering::Relaxed);
                
                if now - last_failure > self.config.recovery_timeout.as_secs() {
                    // Try to recover
                    match f.await {
                        Ok(result) => {
                            // Reset circuit breaker
                            self.failure_count.store(0, std::sync::atomic::Ordering::Relaxed);
                            Ok(result)
                        }
                        Err(e) => {
                            self.record_failure();
                            Err(e)
                        }
                    }
                } else {
                    Err(AppError::new(
                        ErrorKind::ServiceUnavailable,
                        "Circuit breaker is open"
                    ))
                }
            }
            _ => {
                match f.await {
                    Ok(result) => Ok(result),
                    Err(e) => {
                        self.record_failure();
                        Err(e)
                    }
                }
            }
        }
    }
    
    fn record_failure(&self) {
        let failures = self.failure_count.fetch_add(1, std::sync::atomic::Ordering::Relaxed) + 1;
        
        if failures >= self.config.failure_threshold {
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs();
            self.last_failure.store(now, std::sync::atomic::Ordering::Relaxed);
        }
    }
}