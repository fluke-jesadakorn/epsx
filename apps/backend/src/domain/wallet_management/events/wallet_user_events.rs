use crate::prelude::*;
use crate::domain::shared_kernel::{DomainEvent, EventMetadata};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use crate::domain::wallet_management::value_objects::{Permission, WalletAddress};
use std::collections::HashSet;

/// Event raised when a new wallet user is created
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalletUserCreatedEvent {
    pub metadata: EventMetadata,
    pub wallet_address: WalletAddress,
    pub plans: HashSet<String>,
}

impl WalletUserCreatedEvent {
    pub fn new(wallet_address: WalletAddress, plans: HashSet<String>, aggregate_version: u64) -> Self {
        Self {
            metadata: EventMetadata::new(wallet_address.to_string(), aggregate_version),
            wallet_address,
            plans,
        }
    }
}

impl DomainEvent for WalletUserCreatedEvent {
    fn event_id(&self) -> Uuid {
        self.metadata.event_id
    }

    fn event_type(&self) -> &'static str {
        "WalletUserCreated"
    }

    fn aggregate_type(&self) -> &'static str {
        "WalletUser"
    }

    fn aggregate_id(&self) -> String {
        self.metadata.aggregate_id.clone()
    }

    fn aggregate_version(&self) -> u64 {
        self.metadata.aggregate_version
    }

    fn occurred_at(&self) -> DateTime<Utc> {
        self.metadata.occurred_at
    }

    fn to_json(&self) -> Result<String, Box<dyn std::error::Error>> {
        Ok(serde_json::to_string(self)?)
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

/// Event raised when a wallet user is activated
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalletUserActivatedEvent {
    pub metadata: EventMetadata,
    pub wallet_address: WalletAddress,
}

impl WalletUserActivatedEvent {
    pub fn new(wallet_address: WalletAddress, aggregate_version: u64) -> Self {
        Self {
            metadata: EventMetadata::new(wallet_address.to_string(), aggregate_version),
            wallet_address,
        }
    }
}

impl DomainEvent for WalletUserActivatedEvent {
    fn event_id(&self) -> Uuid {
        self.metadata.event_id
    }

    fn event_type(&self) -> &'static str {
        "WalletUserActivated"
    }

    fn aggregate_type(&self) -> &'static str {
        "WalletUser"
    }

    fn aggregate_id(&self) -> String {
        self.metadata.aggregate_id.clone()
    }

    fn aggregate_version(&self) -> u64 {
        self.metadata.aggregate_version
    }

    fn occurred_at(&self) -> DateTime<Utc> {
        self.metadata.occurred_at
    }

    fn to_json(&self) -> Result<String, Box<dyn std::error::Error>> {
        Ok(serde_json::to_string(self)?)
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

/// Event raised when a wallet user is deactivated
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalletUserDeactivatedEvent {
    pub metadata: EventMetadata,
    pub wallet_address: WalletAddress,
    pub reason: String,
    pub revoked_permissions: Vec<Permission>,
}

impl WalletUserDeactivatedEvent {
    pub fn new(
        wallet_address: WalletAddress,
        reason: String,
        revoked_permissions: Vec<Permission>,
        aggregate_version: u64,
    ) -> Self {
        Self {
            metadata: EventMetadata::new(wallet_address.to_string(), aggregate_version),
            wallet_address,
            reason,
            revoked_permissions,
        }
    }
}

impl DomainEvent for WalletUserDeactivatedEvent {
    fn event_id(&self) -> Uuid {
        self.metadata.event_id
    }

    fn event_type(&self) -> &'static str {
        "WalletUserDeactivated"
    }

    fn aggregate_type(&self) -> &'static str {
        "WalletUser"
    }

    fn aggregate_id(&self) -> String {
        self.metadata.aggregate_id.clone()
    }

    fn aggregate_version(&self) -> u64 {
        self.metadata.aggregate_version
    }

    fn occurred_at(&self) -> DateTime<Utc> {
        self.metadata.occurred_at
    }

    fn to_json(&self) -> Result<String, Box<dyn std::error::Error>> {
        Ok(serde_json::to_string(self)?)
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

/// Event raised when permissions are updated in batch
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalletPermissionsUpdatedEvent {
    pub metadata: EventMetadata,
    pub wallet_address: WalletAddress,
    pub old_permissions: Vec<Permission>,
    pub new_permissions: Vec<Permission>,
}

impl WalletPermissionsUpdatedEvent {
    pub fn new(
        wallet_address: WalletAddress,
        old_permissions: Vec<Permission>,
        new_permissions: Vec<Permission>,
        aggregate_version: u64,
    ) -> Self {
        Self {
            metadata: EventMetadata::new(wallet_address.to_string(), aggregate_version),
            wallet_address,
            old_permissions,
            new_permissions,
        }
    }
}

impl DomainEvent for WalletPermissionsUpdatedEvent {
    fn event_id(&self) -> Uuid {
        self.metadata.event_id
    }

    fn event_type(&self) -> &'static str {
        "WalletPermissionsUpdated"
    }

    fn aggregate_type(&self) -> &'static str {
        "WalletUser"
    }

    fn aggregate_id(&self) -> String {
        self.metadata.aggregate_id.clone()
    }

    fn aggregate_version(&self) -> u64 {
        self.metadata.aggregate_version
    }

    fn occurred_at(&self) -> DateTime<Utc> {
        self.metadata.occurred_at
    }

    fn to_json(&self) -> Result<String, Box<dyn std::error::Error>> {
        Ok(serde_json::to_string(self)?)
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

// TierLevelChangedEvent removed - using permission plans instead
