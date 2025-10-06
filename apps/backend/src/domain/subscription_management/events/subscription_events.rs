use crate::prelude::*;
use crate::domain::shared_kernel::{DomainEvent, EventMetadata};
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use uuid::Uuid;

/// Event emitted when a subscription starts
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubscriptionStartedEvent {
    pub metadata: EventMetadata,
    pub subscription_id: String,
    pub wallet_address: String,
    pub plan_id: i32,
    pub started_at: DateTime<Utc>,
}

impl SubscriptionStartedEvent {
    pub fn new(
        subscription_id: String,
        wallet_address: String,
        plan_id: i32,
        started_at: DateTime<Utc>,
        aggregate_version: u64,
    ) -> Self {
        Self {
            metadata: EventMetadata::new(subscription_id.clone(), aggregate_version),
            subscription_id,
            wallet_address,
            plan_id,
            started_at,
        }
    }
}

impl DomainEvent for SubscriptionStartedEvent {
    fn event_id(&self) -> Uuid {
        self.metadata.event_id
    }

    fn event_type(&self) -> &'static str {
        "SubscriptionStarted"
    }

    fn aggregate_type(&self) -> &'static str {
        "Subscription"
    }

    fn occurred_at(&self) -> DateTime<Utc> {
        self.metadata.occurred_at
    }

    fn aggregate_version(&self) -> u64 {
        self.metadata.aggregate_version
    }

    fn aggregate_id(&self) -> String {
        self.metadata.aggregate_id.clone()
    }

    fn to_json(&self) -> Result<String, Box<dyn std::error::Error>> {
        Ok(serde_json::to_string(self)?)
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

/// Event emitted when a subscription is renewed
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubscriptionRenewedEvent {
    pub metadata: EventMetadata,
    pub subscription_id: String,
    pub new_expires_at: DateTime<Utc>,
    pub renewed_at: DateTime<Utc>,
}

impl SubscriptionRenewedEvent {
    pub fn new(
        subscription_id: String,
        new_expires_at: DateTime<Utc>,
        renewed_at: DateTime<Utc>,
        aggregate_version: u64,
    ) -> Self {
        Self {
            metadata: EventMetadata::new(subscription_id.clone(), aggregate_version),
            subscription_id,
            new_expires_at,
            renewed_at,
        }
    }
}

impl DomainEvent for SubscriptionRenewedEvent {
    fn event_id(&self) -> Uuid {
        self.metadata.event_id
    }

    fn event_type(&self) -> &'static str {
        "SubscriptionRenewed"
    }

    fn aggregate_type(&self) -> &'static str {
        "Subscription"
    }

    fn occurred_at(&self) -> DateTime<Utc> {
        self.metadata.occurred_at
    }

    fn aggregate_version(&self) -> u64 {
        self.metadata.aggregate_version
    }

    fn aggregate_id(&self) -> String {
        self.metadata.aggregate_id.clone()
    }

    fn to_json(&self) -> Result<String, Box<dyn std::error::Error>> {
        Ok(serde_json::to_string(self)?)
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

/// Event emitted when a subscription is cancelled
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubscriptionCancelledEvent {
    pub metadata: EventMetadata,
    pub subscription_id: String,
    pub cancelled_at: DateTime<Utc>,
}

impl SubscriptionCancelledEvent {
    pub fn new(
        subscription_id: String,
        cancelled_at: DateTime<Utc>,
        aggregate_version: u64,
    ) -> Self {
        Self {
            metadata: EventMetadata::new(subscription_id.clone(), aggregate_version),
            subscription_id,
            cancelled_at,
        }
    }
}

impl DomainEvent for SubscriptionCancelledEvent {
    fn event_id(&self) -> Uuid {
        self.metadata.event_id
    }

    fn event_type(&self) -> &'static str {
        "SubscriptionCancelled"
    }

    fn aggregate_type(&self) -> &'static str {
        "Subscription"
    }

    fn occurred_at(&self) -> DateTime<Utc> {
        self.metadata.occurred_at
    }

    fn aggregate_version(&self) -> u64 {
        self.metadata.aggregate_version
    }

    fn aggregate_id(&self) -> String {
        self.metadata.aggregate_id.clone()
    }

    fn to_json(&self) -> Result<String, Box<dyn std::error::Error>> {
        Ok(serde_json::to_string(self)?)
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

/// Event emitted when a subscription expires
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubscriptionExpiredEvent {
    pub metadata: EventMetadata,
    pub subscription_id: String,
    pub expired_at: DateTime<Utc>,
}

impl SubscriptionExpiredEvent {
    pub fn new(
        subscription_id: String,
        expired_at: DateTime<Utc>,
        aggregate_version: u64,
    ) -> Self {
        Self {
            metadata: EventMetadata::new(subscription_id.clone(), aggregate_version),
            subscription_id,
            expired_at,
        }
    }
}

impl DomainEvent for SubscriptionExpiredEvent {
    fn event_id(&self) -> Uuid {
        self.metadata.event_id
    }

    fn event_type(&self) -> &'static str {
        "SubscriptionExpired"
    }

    fn aggregate_type(&self) -> &'static str {
        "Subscription"
    }

    fn occurred_at(&self) -> DateTime<Utc> {
        self.metadata.occurred_at
    }

    fn aggregate_version(&self) -> u64 {
        self.metadata.aggregate_version
    }

    fn aggregate_id(&self) -> String {
        self.metadata.aggregate_id.clone()
    }

    fn to_json(&self) -> Result<String, Box<dyn std::error::Error>> {
        Ok(serde_json::to_string(self)?)
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}
