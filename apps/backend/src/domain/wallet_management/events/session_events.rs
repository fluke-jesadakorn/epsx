use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::domain::shared_kernel::{DomainEvent, domain_event::EventMetadata};
use crate::domain::shared_kernel::value_objects::SessionId;
use crate::domain::wallet_management::value_objects::WalletAddress;

/// Event raised when a new session is created
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionCreatedEvent {
    pub metadata: EventMetadata,
    pub session_id: SessionId,
    pub wallet_address: WalletAddress,
    pub expires_at: DateTime<Utc>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
}

impl SessionCreatedEvent {
    pub fn new(
        session_id: SessionId,
        wallet_address: WalletAddress,
        expires_at: DateTime<Utc>,
        ip_address: Option<String>,
        user_agent: Option<String>,
        aggregate_version: u64
    ) -> Self {
        Self {
            metadata: EventMetadata::new(session_id.to_string(), aggregate_version),
            session_id,
            wallet_address,
            expires_at,
            ip_address,
            user_agent,
        }
    }
}

impl DomainEvent for SessionCreatedEvent {
    fn event_id(&self) -> Uuid {
        self.metadata.event_id
    }

    fn event_type(&self) -> &'static str {
        "SessionCreated"
    }

    fn aggregate_type(&self) -> &'static str {
        "Session"
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

/// Event raised when a session is invalidated
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionInvalidatedEvent {
    pub metadata: EventMetadata,
    pub session_id: SessionId,
    pub wallet_address: WalletAddress,
    pub reason: SessionInvalidationReason,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SessionInvalidationReason {
    UserLogout,
    Expired,
    SecurityViolation,
    AdminRevocation,
    SystemMaintenance,
}

impl SessionInvalidatedEvent {
    pub fn new(
        session_id: SessionId,
        wallet_address: WalletAddress,
        reason: SessionInvalidationReason,
        aggregate_version: u64
    ) -> Self {
        Self {
            metadata: EventMetadata::new(session_id.to_string(), aggregate_version),
            session_id,
            wallet_address,
            reason,
        }
    }
}

impl DomainEvent for SessionInvalidatedEvent {
    fn event_id(&self) -> Uuid {
        self.metadata.event_id
    }

    fn event_type(&self) -> &'static str {
        "SessionInvalidated"
    }

    fn aggregate_type(&self) -> &'static str {
        "Session"
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

/// Event raised when a session is extended
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionExtendedEvent {
    pub metadata: EventMetadata,
    pub session_id: SessionId,
    pub wallet_address: WalletAddress,
    pub old_expires_at: DateTime<Utc>,
    pub new_expires_at: DateTime<Utc>,
}

impl SessionExtendedEvent {
    pub fn new(
        session_id: SessionId,
        wallet_address: WalletAddress,
        old_expires_at: DateTime<Utc>,
        new_expires_at: DateTime<Utc>,
        aggregate_version: u64
    ) -> Self {
        Self {
            metadata: EventMetadata::new(session_id.to_string(), aggregate_version),
            session_id,
            wallet_address,
            old_expires_at,
            new_expires_at,
        }
    }
}

impl DomainEvent for SessionExtendedEvent {
    fn event_id(&self) -> Uuid {
        self.metadata.event_id
    }

    fn event_type(&self) -> &'static str {
        "SessionExtended"
    }

    fn aggregate_type(&self) -> &'static str {
        "Session"
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