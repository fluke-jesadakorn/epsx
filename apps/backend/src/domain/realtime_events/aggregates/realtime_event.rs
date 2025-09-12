// RealtimeEvent Aggregate Root
// Manages event lifecycle, delivery, and retry logic

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::domain::shared_kernel::aggregate_root::{AggregateRoot, AggregateBase};
use crate::domain::shared_kernel::domain_event::DomainEvent;
use crate::domain::realtime_events::value_objects::{EventId, UserId, EventPayload};
use super::super::RealtimeEventsBoundedContext;

/// Real-time Event Aggregate Root
/// Manages the complete lifecycle of a real-time event from creation to delivery
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RealtimeEvent {
    id: EventId,
    payload: EventPayload,
    target_users: Vec<UserId>,
    channel: String,
    priority: EventPriority,
    status: EventStatus,
    delivery_attempts: u32,
    max_retry_attempts: u32,
    created_at: DateTime<Utc>,
    scheduled_for: Option<DateTime<Utc>>,
    delivered_at: Option<DateTime<Utc>>,
    failed_at: Option<DateTime<Utc>>,
    failure_reason: Option<String>,
    delivery_metadata: HashMap<String, String>,
    base: AggregateBase,
}

impl RealtimeEvent {
    /// Create new real-time event
    pub fn create(
        payload: EventPayload,
        target_users: Vec<UserId>,
        channel: String,
    ) -> Result<Self, RealtimeEventError> {
        // Validate payload size
        payload.validate_size()
            .map_err(|e| RealtimeEventError::InvalidPayload(e.to_string()))?;
        
        // Validate channel name
        if channel.is_empty() {
            return Err(RealtimeEventError::InvalidChannel("Channel cannot be empty".to_string()));
        }
        
        // Validate target users for events that require targeting
        if payload.event_type().requires_user_targeting() && target_users.is_empty() {
            return Err(RealtimeEventError::MissingTargetUsers);
        }
        
        let now = Utc::now();
        let mut event = Self {
            id: EventId::new(),
            payload,
            target_users,
            channel,
            priority: EventPriority::Normal,
            status: EventStatus::Pending,
            delivery_attempts: 0,
            max_retry_attempts: RealtimeEventsBoundedContext::MAX_RETRY_ATTEMPTS,
            created_at: now,
            scheduled_for: None,
            delivered_at: None,
            failed_at: None,
            failure_reason: None,
            delivery_metadata: HashMap::new(),
            base: AggregateBase::new(),
        };
        
        // Publish event created domain event
        event.base.add_event(Box::new(super::super::events::EventCreated {
            event_id: event.id.clone(),
            event_type: event.payload.event_type().clone(),
            channel: event.channel.clone(),
            target_user_count: event.target_users.len() as u32,
            created_at: now,
            domain_event_id: uuid::Uuid::new_v4(),
            aggregate_version: 1,
        }));
        
        Ok(event)
    }
    
    /// Create broadcast event (targets all connected users)
    pub fn create_broadcast(
        payload: EventPayload,
        channel: String,
    ) -> Result<Self, RealtimeEventError> {
        Self::create(payload, vec![], channel)
    }
    
    /// Set event priority
    pub fn set_priority(&mut self, priority: EventPriority) -> Result<(), RealtimeEventError> {
        if matches!(self.status, EventStatus::Delivered | EventStatus::Failed) {
            return Err(RealtimeEventError::EventAlreadyProcessed);
        }
        
        self.priority = priority;
        Ok(())
    }
    
    /// Schedule event for future delivery
    pub fn schedule_for(&mut self, delivery_time: DateTime<Utc>) -> Result<(), RealtimeEventError> {
        if delivery_time <= Utc::now() {
            return Err(RealtimeEventError::InvalidScheduleTime);
        }
        
        if matches!(self.status, EventStatus::Delivered | EventStatus::Failed) {
            return Err(RealtimeEventError::EventAlreadyProcessed);
        }
        
        self.scheduled_for = Some(delivery_time);
        self.status = EventStatus::Scheduled;
        Ok(())
    }
    
    /// Mark event as being delivered
    pub fn start_delivery(&mut self) -> Result<(), RealtimeEventError> {
        if !matches!(self.status, EventStatus::Pending | EventStatus::Scheduled | EventStatus::Retrying) {
            return Err(RealtimeEventError::InvalidStatusTransition {
                from: self.status.clone(),
                to: EventStatus::Delivering,
            });
        }
        
        self.status = EventStatus::Delivering;
        self.delivery_attempts += 1;
        
        // Add delivery metadata
        self.delivery_metadata.insert("delivery_started_at".to_string(), Utc::now().to_rfc3339());
        self.delivery_metadata.insert("attempt".to_string(), self.delivery_attempts.to_string());
        
        Ok(())
    }
    
    /// Mark event as successfully delivered
    pub fn mark_delivered(&mut self) -> Result<(), RealtimeEventError> {
        if !matches!(self.status, EventStatus::Delivering) {
            return Err(RealtimeEventError::InvalidStatusTransition {
                from: self.status.clone(),
                to: EventStatus::Delivered,
            });
        }
        
        self.status = EventStatus::Delivered;
        self.delivered_at = Some(Utc::now());
        
        // Publish domain event
        self.base.add_event(Box::new(super::super::events::EventDelivered {
            event_id: self.id.clone(),
            delivered_at: self.delivered_at.unwrap(),
            delivery_attempts: self.delivery_attempts,
            domain_event_id: uuid::Uuid::new_v4(),
            aggregate_version: self.base.version(),
        }));
        
        Ok(())
    }
    
    /// Mark event delivery as failed and potentially retry
    pub fn mark_failed(&mut self, reason: String) -> Result<(), RealtimeEventError> {
        if !matches!(self.status, EventStatus::Delivering) {
            return Err(RealtimeEventError::InvalidStatusTransition {
                from: self.status.clone(),
                to: EventStatus::Failed,
            });
        }
        
        // Check if we should retry
        if self.delivery_attempts < self.max_retry_attempts {
            self.status = EventStatus::Retrying;
            self.failure_reason = Some(reason.clone());
            
            // Schedule retry with exponential backoff
            let retry_delay_seconds = 2_i64.pow(self.delivery_attempts - 1);
            let retry_at = Utc::now() + chrono::Duration::seconds(retry_delay_seconds);
            self.scheduled_for = Some(retry_at);
            
            // Publish retry event
            self.base.add_event(Box::new(super::super::events::EventRetryScheduled {
                event_id: self.id.clone(),
                attempt: self.delivery_attempts,
                retry_at,
                reason: reason.clone(),
                domain_event_id: uuid::Uuid::new_v4(),
                aggregate_version: self.base.version(),
            }));
        } else {
            // Final failure
            self.status = EventStatus::Failed;
            self.failed_at = Some(Utc::now());
            self.failure_reason = Some(reason.clone());
            
            // Publish failure event
            self.base.add_event(Box::new(super::super::events::EventFailed {
                event_id: self.id.clone(),
                failed_at: self.failed_at.unwrap(),
                reason,
                total_attempts: self.delivery_attempts,
                domain_event_id: uuid::Uuid::new_v4(),
                aggregate_version: self.base.version(),
            }));
        }
        
        Ok(())
    }
    
    /// Check if event is ready for delivery
    pub fn is_ready_for_delivery(&self) -> bool {
        match self.status {
            EventStatus::Pending => true,
            EventStatus::Scheduled | EventStatus::Retrying => {
                self.scheduled_for.map_or(true, |time| time <= Utc::now())
            },
            _ => false,
        }
    }
    
    /// Check if event has expired
    pub fn is_expired(&self) -> bool {
        let ttl_duration = chrono::Duration::minutes(
            RealtimeEventsBoundedContext::BROADCAST_EVENT_TTL_MINUTES as i64
        );
        Utc::now() > self.created_at + ttl_duration
    }
    
    /// Add delivery metadata
    pub fn add_delivery_metadata(&mut self, key: String, value: String) {
        self.delivery_metadata.insert(key, value);
    }
    
    /// Get event ID
    pub fn id(&self) -> &EventId {
        &self.id
    }
    
    /// Get event payload
    pub fn payload(&self) -> &EventPayload {
        &self.payload
    }
    
    /// Get target users
    pub fn target_users(&self) -> &[UserId] {
        &self.target_users
    }
    
    /// Get channel
    pub fn channel(&self) -> &str {
        &self.channel
    }
    
    /// Get priority
    pub fn priority(&self) -> EventPriority {
        self.priority
    }
    
    /// Get status
    pub fn status(&self) -> &EventStatus {
        &self.status
    }
    
    /// Get delivery attempts
    pub fn delivery_attempts(&self) -> u32 {
        self.delivery_attempts
    }
    
    /// Get created timestamp
    pub fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }
    
    /// Get delivery metadata
    pub fn delivery_metadata(&self) -> &HashMap<String, String> {
        &self.delivery_metadata
    }
}

impl AggregateRoot for RealtimeEvent {
    type Id = EventId;
    
    fn id(&self) -> &Self::Id {
        &self.id
    }
    
    fn version(&self) -> u64 {
        self.base.version()
    }
    
    fn increment_version(&mut self) {
        self.base.increment_version()
    }
    
    fn uncommitted_events(&self) -> &[Box<dyn DomainEvent>] {
        self.base.uncommitted_events()
    }
    
    fn mark_events_as_committed(&mut self) {
        self.base.mark_events_as_committed()
    }
    
    fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }
    
    fn updated_at(&self) -> DateTime<Utc> {
        self.base.updated_at
    }
    
    fn touch(&mut self) {
        self.base.touch()
    }
}

/// Event priority levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EventPriority {
    Low,
    Normal,
    High,
    Critical,
}

impl EventPriority {
    /// Get numeric priority for sorting (higher number = higher priority)
    pub fn as_numeric(&self) -> u8 {
        match self {
            Self::Low => 1,
            Self::Normal => 2,
            Self::High => 3,
            Self::Critical => 4,
        }
    }
}

/// Event delivery status
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum EventStatus {
    Pending,
    Scheduled,
    Delivering,
    Delivered,
    Retrying,
    Failed,
    Expired,
}

/// Errors that can occur with real-time events
#[derive(Debug, thiserror::Error)]
pub enum RealtimeEventError {
    #[error("Invalid payload: {0}")]
    InvalidPayload(String),
    
    #[error("Invalid channel: {0}")]
    InvalidChannel(String),
    
    #[error("Target users required for this event type")]
    MissingTargetUsers,
    
    #[error("Event already processed")]
    EventAlreadyProcessed,
    
    #[error("Invalid schedule time (must be in the future)")]
    InvalidScheduleTime,
    
    #[error("Invalid status transition from {from:?} to {to:?}")]
    InvalidStatusTransition { from: EventStatus, to: EventStatus },
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::realtime_events::value_objects::{EventPayload, NotificationLevel};
    
    #[test]
    fn test_create_broadcast_event() {
        let payload = EventPayload::system_notification(
            "Test".to_string(),
            "Test message".to_string(),
            NotificationLevel::Info,
            None,
        );
        
        let event = RealtimeEvent::create_broadcast(payload, "notifications".to_string()).unwrap();
        
        assert_eq!(event.status(), &EventStatus::Pending);
        assert_eq!(event.channel(), "notifications");
        assert!(event.target_users().is_empty());
        assert_eq!(event.delivery_attempts(), 0);
    }
    
    #[test]
    fn test_event_delivery_lifecycle() {
        let payload = EventPayload::system_notification(
            "Test".to_string(),
            "Test message".to_string(),
            NotificationLevel::Info,
            None,
        );
        
        let event = RealtimeEvent::create_broadcast(payload, "notifications".to_string()).unwrap();
        
        // Start delivery
        event.start_delivery().unwrap();
        assert_eq!(event.status(), &EventStatus::Delivering);
        assert_eq!(event.delivery_attempts(), 1);
        
        // Mark as delivered
        event.mark_delivered().unwrap();
        assert_eq!(event.status(), &EventStatus::Delivered);
        assert!(event.delivered_at.is_some());
    }
    
    #[test]
    fn test_event_retry_logic() {
        let payload = EventPayload::system_notification(
            "Test".to_string(),
            "Test message".to_string(),
            NotificationLevel::Info,
            None,
        );
        
        let event = RealtimeEvent::create_broadcast(payload, "notifications".to_string()).unwrap();
        
        // Start and fail delivery
        event.start_delivery().unwrap();
        event.mark_failed("Network error".to_string()).unwrap();
        
        assert_eq!(event.status(), &EventStatus::Retrying);
        assert!(event.scheduled_for.is_some());
        assert_eq!(event.failure_reason, Some("Network error".to_string()));
    }
}