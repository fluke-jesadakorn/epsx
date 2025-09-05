// Real-time Events Domain Events

use chrono::{DateTime, Utc};
use uuid::Uuid;
use serde::{Serialize, Deserialize};

use crate::domain::shared_kernel::DomainEvent;
use super::value_objects::{EventId, EventType};

/// Event was created and ready for processing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventCreated {
    pub event_id: EventId,
    pub event_type: EventType,
    pub channel: String,
    pub target_user_count: u32,
    pub created_at: DateTime<Utc>,
    pub domain_event_id: Uuid,
    pub aggregate_version: u64,
}

impl DomainEvent for EventCreated {
    fn event_type(&self) -> &'static str {
        "EventCreated"
    }
    
    fn aggregate_id(&self) -> String {
        self.event_id.to_string()
    }
    
    fn occurred_at(&self) -> DateTime<Utc> {
        self.created_at
    }
    
    fn event_id(&self) -> Uuid {
        self.domain_event_id
    }
    
    fn aggregate_version(&self) -> u64 {
        self.aggregate_version
    }
    
    fn to_json(&self) -> Result<String, Box<dyn std::error::Error>> {
        Ok(serde_json::to_string(self)?)
    }
}

/// Event was successfully delivered
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventDelivered {
    pub event_id: EventId,
    pub delivered_at: DateTime<Utc>,
    pub delivery_attempts: u32,
    pub domain_event_id: Uuid,
    pub aggregate_version: u64,
}

impl DomainEvent for EventDelivered {
    fn event_type(&self) -> &'static str {
        "EventDelivered"
    }
    
    fn aggregate_id(&self) -> String {
        self.event_id.to_string()
    }
    
    fn occurred_at(&self) -> DateTime<Utc> {
        self.delivered_at
    }
    
    fn event_id(&self) -> Uuid {
        self.domain_event_id
    }
    
    fn aggregate_version(&self) -> u64 {
        self.aggregate_version
    }
    
    fn to_json(&self) -> Result<String, Box<dyn std::error::Error>> {
        Ok(serde_json::to_string(self)?)
    }
}

/// Event delivery failed after all retries
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventFailed {
    pub event_id: EventId,
    pub failed_at: DateTime<Utc>,
    pub reason: String,
    pub total_attempts: u32,
    pub domain_event_id: Uuid,
    pub aggregate_version: u64,
}

impl DomainEvent for EventFailed {
    fn event_type(&self) -> &'static str {
        "EventFailed"
    }
    
    fn aggregate_id(&self) -> String {
        self.event_id.to_string()
    }
    
    fn occurred_at(&self) -> DateTime<Utc> {
        self.failed_at
    }
    
    fn event_id(&self) -> Uuid {
        self.domain_event_id
    }
    
    fn aggregate_version(&self) -> u64 {
        self.aggregate_version
    }
    
    fn to_json(&self) -> Result<String, Box<dyn std::error::Error>> {
        Ok(serde_json::to_string(self)?)
    }
}

/// Event retry was scheduled
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventRetryScheduled {
    pub event_id: EventId,
    pub attempt: u32,
    pub retry_at: DateTime<Utc>,
    pub reason: String,
    pub domain_event_id: Uuid,
    pub aggregate_version: u64,
}

impl DomainEvent for EventRetryScheduled {
    fn event_type(&self) -> &'static str {
        "EventRetryScheduled"
    }
    
    fn aggregate_id(&self) -> String {
        self.event_id.to_string()
    }
    
    fn occurred_at(&self) -> DateTime<Utc> {
        self.retry_at
    }
    
    fn event_id(&self) -> Uuid {
        self.domain_event_id
    }
    
    fn aggregate_version(&self) -> u64 {
        self.aggregate_version
    }
    
    fn to_json(&self) -> Result<String, Box<dyn std::error::Error>> {
        Ok(serde_json::to_string(self)?)
    }
}

/// Connection was established
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionEstablished {
    pub connection_id: super::value_objects::ConnectionId,
    pub user_id: super::value_objects::UserId,
    pub established_at: DateTime<Utc>,
    pub domain_event_id: Uuid,
    pub aggregate_version: u64,
}

impl DomainEvent for ConnectionEstablished {
    fn event_type(&self) -> &'static str {
        "ConnectionEstablished"
    }
    
    fn aggregate_id(&self) -> String {
        self.connection_id.to_string()
    }
    
    fn occurred_at(&self) -> DateTime<Utc> {
        self.established_at
    }
    
    fn event_id(&self) -> Uuid {
        self.domain_event_id
    }
    
    fn aggregate_version(&self) -> u64 {
        self.aggregate_version
    }
    
    fn to_json(&self) -> Result<String, Box<dyn std::error::Error>> {
        Ok(serde_json::to_string(self)?)
    }
}

/// Connection was closed
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionClosed {
    pub connection_id: super::value_objects::ConnectionId,
    pub closed_at: DateTime<Utc>,
    pub domain_event_id: Uuid,
    pub aggregate_version: u64,
}

impl DomainEvent for ConnectionClosed {
    fn event_type(&self) -> &'static str {
        "ConnectionClosed"
    }
    
    fn aggregate_id(&self) -> String {
        self.connection_id.to_string()
    }
    
    fn occurred_at(&self) -> DateTime<Utc> {
        self.closed_at
    }
    
    fn event_id(&self) -> Uuid {
        self.domain_event_id
    }
    
    fn aggregate_version(&self) -> u64 {
        self.aggregate_version
    }
    
    fn to_json(&self) -> Result<String, Box<dyn std::error::Error>> {
        Ok(serde_json::to_string(self)?)
    }
}

/// Subscription was created
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubscriptionCreated {
    pub subscription_id: Uuid,
    pub user_id: super::value_objects::UserId,
    pub topic: String,
    pub created_at: DateTime<Utc>,
    pub domain_event_id: Uuid,
    pub aggregate_version: u64,
}

impl DomainEvent for SubscriptionCreated {
    fn event_type(&self) -> &'static str {
        "SubscriptionCreated"
    }
    
    fn aggregate_id(&self) -> String {
        self.subscription_id.to_string()
    }
    
    fn occurred_at(&self) -> DateTime<Utc> {
        self.created_at
    }
    
    fn event_id(&self) -> Uuid {
        self.domain_event_id
    }
    
    fn aggregate_version(&self) -> u64 {
        self.aggregate_version
    }
    
    fn to_json(&self) -> Result<String, Box<dyn std::error::Error>> {
        Ok(serde_json::to_string(self)?)
    }
}

// Missing events that are exported but not defined
pub use EventDelivered as EventPublished;