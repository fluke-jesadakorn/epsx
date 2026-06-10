// Notification domain events
// Events representing state changes in the notification lifecycle

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::domain::shared_kernel::domain_event::{DomainEvent, EventMetadata};
use crate::domain::notification::value_objects::user_preferences::NotificationType;
use super::super::aggregates::notification::{NotificationPriority, NotificationStatus};

/// Event: Notification was created
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationCreated {
    pub metadata: EventMetadata,
    pub recipient_wallet_address: String,
    pub topic_name: Option<String>,
    pub notification_type: NotificationType,
    pub priority: NotificationPriority,
    pub status: NotificationStatus,
}

impl NotificationCreated {
    pub fn new(
        aggregate_id: impl Into<String>,
        aggregate_version: u64,
        recipient_wallet_address: String,
        topic_name: Option<String>,
        notification_type: NotificationType,
        priority: NotificationPriority,
        status: NotificationStatus,
    ) -> Self {
        Self {
            metadata: EventMetadata::new(aggregate_id.into(), aggregate_version),
            recipient_wallet_address,
            topic_name,
            notification_type,
            priority,
            status,
        }
    }
}

impl DomainEvent for NotificationCreated {
    fn event_id(&self) -> Uuid { self.metadata.event_id }
    fn event_type(&self) -> &'static str { "NotificationCreated" }
    fn aggregate_type(&self) -> &'static str { "Notification" }
    fn occurred_at(&self) -> DateTime<Utc> { self.metadata.occurred_at }
    fn aggregate_version(&self) -> u64 { self.metadata.aggregate_version }
    fn aggregate_id(&self) -> String { self.metadata.aggregate_id.clone() }
    fn to_json(&self) -> Result<String, Box<dyn std::error::Error>> {
        Ok(serde_json::to_string(self)?)
    }
    fn as_any(&self) -> &dyn std::any::Any { self }
}

/// Event: Notification was scheduled for delivery
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationScheduled {
    pub metadata: EventMetadata,
    pub scheduled_at: Option<DateTime<Utc>>,
    pub status: NotificationStatus,
}

impl NotificationScheduled {
    pub fn new(
        aggregate_id: impl Into<String>,
        aggregate_version: u64,
        scheduled_at: Option<DateTime<Utc>>,
        status: NotificationStatus,
    ) -> Self {
        Self {
            metadata: EventMetadata::new(aggregate_id.into(), aggregate_version),
            scheduled_at,
            status,
        }
    }
}

impl DomainEvent for NotificationScheduled {
    fn event_id(&self) -> Uuid { self.metadata.event_id }
    fn event_type(&self) -> &'static str { "NotificationScheduled" }
    fn aggregate_type(&self) -> &'static str { "Notification" }
    fn occurred_at(&self) -> DateTime<Utc> { self.metadata.occurred_at }
    fn aggregate_version(&self) -> u64 { self.metadata.aggregate_version }
    fn aggregate_id(&self) -> String { self.metadata.aggregate_id.clone() }
    fn to_json(&self) -> Result<String, Box<dyn std::error::Error>> {
        Ok(serde_json::to_string(self)?)
    }
    fn as_any(&self) -> &dyn std::any::Any { self }
}

/// Event: Notification is being sent
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationSending {
    pub metadata: EventMetadata,
    pub channel_count: u32,
}

impl NotificationSending {
    pub fn new(aggregate_id: impl Into<String>, aggregate_version: u64, channel_count: u32) -> Self {
        Self {
            metadata: EventMetadata::new(aggregate_id.into(), aggregate_version),
            channel_count,
        }
    }
}

impl DomainEvent for NotificationSending {
    fn event_id(&self) -> Uuid { self.metadata.event_id }
    fn event_type(&self) -> &'static str { "NotificationSending" }
    fn aggregate_type(&self) -> &'static str { "Notification" }
    fn occurred_at(&self) -> DateTime<Utc> { self.metadata.occurred_at }
    fn aggregate_version(&self) -> u64 { self.metadata.aggregate_version }
    fn aggregate_id(&self) -> String { self.metadata.aggregate_id.clone() }
    fn to_json(&self) -> Result<String, Box<dyn std::error::Error>> {
        Ok(serde_json::to_string(self)?)
    }
    fn as_any(&self) -> &dyn std::any::Any { self }
}

/// Event: Notification delivery completed (success or partial)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationDeliveryCompleted {
    pub metadata: EventMetadata,
    pub status: NotificationStatus,
    pub successful_channels: Vec<String>,
    pub failed_channels: Vec<String>,
}

impl NotificationDeliveryCompleted {
    pub fn new(
        aggregate_id: impl Into<String>,
        aggregate_version: u64,
        status: NotificationStatus,
        successful_channels: Vec<String>,
        failed_channels: Vec<String>,
    ) -> Self {
        Self {
            metadata: EventMetadata::new(aggregate_id.into(), aggregate_version),
            status,
            successful_channels,
            failed_channels,
        }
    }
}

impl DomainEvent for NotificationDeliveryCompleted {
    fn event_id(&self) -> Uuid { self.metadata.event_id }
    fn event_type(&self) -> &'static str { "NotificationDeliveryCompleted" }
    fn aggregate_type(&self) -> &'static str { "Notification" }
    fn occurred_at(&self) -> DateTime<Utc> { self.metadata.occurred_at }
    fn aggregate_version(&self) -> u64 { self.metadata.aggregate_version }
    fn aggregate_id(&self) -> String { self.metadata.aggregate_id.clone() }
    fn to_json(&self) -> Result<String, Box<dyn std::error::Error>> {
        Ok(serde_json::to_string(self)?)
    }
    fn as_any(&self) -> &dyn std::any::Any { self }
}

/// Event: Notification expired before delivery
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationExpired {
    pub metadata: EventMetadata,
    pub expired_at: Option<DateTime<Utc>>,
}

impl NotificationExpired {
    pub fn new(
        aggregate_id: impl Into<String>,
        aggregate_version: u64,
        expired_at: Option<DateTime<Utc>>,
    ) -> Self {
        Self {
            metadata: EventMetadata::new(aggregate_id.into(), aggregate_version),
            expired_at,
        }
    }
}

impl DomainEvent for NotificationExpired {
    fn event_id(&self) -> Uuid { self.metadata.event_id }
    fn event_type(&self) -> &'static str { "NotificationExpired" }
    fn aggregate_type(&self) -> &'static str { "Notification" }
    fn occurred_at(&self) -> DateTime<Utc> { self.metadata.occurred_at }
    fn aggregate_version(&self) -> u64 { self.metadata.aggregate_version }
    fn aggregate_id(&self) -> String { self.metadata.aggregate_id.clone() }
    fn to_json(&self) -> Result<String, Box<dyn std::error::Error>> {
        Ok(serde_json::to_string(self)?)
    }
    fn as_any(&self) -> &dyn std::any::Any { self }
}

/// Event: Notification priority was updated
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationPriorityUpdated {
    pub metadata: EventMetadata,
    pub old_priority: NotificationPriority,
    pub new_priority: NotificationPriority,
}

impl NotificationPriorityUpdated {
    pub fn new(
        aggregate_id: impl Into<String>,
        aggregate_version: u64,
        old_priority: NotificationPriority,
        new_priority: NotificationPriority,
    ) -> Self {
        Self {
            metadata: EventMetadata::new(aggregate_id.into(), aggregate_version),
            old_priority,
            new_priority,
        }
    }
}

impl DomainEvent for NotificationPriorityUpdated {
    fn event_id(&self) -> Uuid { self.metadata.event_id }
    fn event_type(&self) -> &'static str { "NotificationPriorityUpdated" }
    fn aggregate_type(&self) -> &'static str { "Notification" }
    fn occurred_at(&self) -> DateTime<Utc> { self.metadata.occurred_at }
    fn aggregate_version(&self) -> u64 { self.metadata.aggregate_version }
    fn aggregate_id(&self) -> String { self.metadata.aggregate_id.clone() }
    fn to_json(&self) -> Result<String, Box<dyn std::error::Error>> {
        Ok(serde_json::to_string(self)?)
    }
    fn as_any(&self) -> &dyn std::any::Any { self }
}

/// Event: Notification was cancelled
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationCancelled {
    pub metadata: EventMetadata,
    pub reason: String,
}

impl NotificationCancelled {
    pub fn new(aggregate_id: impl Into<String>, aggregate_version: u64, reason: String) -> Self {
        Self {
            metadata: EventMetadata::new(aggregate_id.into(), aggregate_version),
            reason,
        }
    }
}

impl DomainEvent for NotificationCancelled {
    fn event_id(&self) -> Uuid { self.metadata.event_id }
    fn event_type(&self) -> &'static str { "NotificationCancelled" }
    fn aggregate_type(&self) -> &'static str { "Notification" }
    fn occurred_at(&self) -> DateTime<Utc> { self.metadata.occurred_at }
    fn aggregate_version(&self) -> u64 { self.metadata.aggregate_version }
    fn aggregate_id(&self) -> String { self.metadata.aggregate_id.clone() }
    fn to_json(&self) -> Result<String, Box<dyn std::error::Error>> {
        Ok(serde_json::to_string(self)?)
    }
    fn as_any(&self) -> &dyn std::any::Any { self }
}
