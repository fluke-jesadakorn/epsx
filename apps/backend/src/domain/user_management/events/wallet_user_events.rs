use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use crate::domain::shared_kernel::DomainEvent;
use crate::domain::user_management::value_objects::{Permission, WalletAddress};
use std::collections::HashSet;

/// Event raised when a new wallet user is created
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalletUserCreatedEvent {
    pub wallet_address: WalletAddress,
    pub permission_groups: HashSet<String>,
    pub aggregate_version: u64,
    pub occurred_at: DateTime<Utc>,
}

impl WalletUserCreatedEvent {
    pub fn new(wallet_address: WalletAddress, permission_groups: HashSet<String>, aggregate_version: u64) -> Self {
        Self {
            wallet_address,
            permission_groups,
            aggregate_version,
            occurred_at: Utc::now(),
        }
    }
}

impl DomainEvent for WalletUserCreatedEvent {
    fn event_id(&self) -> Uuid {
        Uuid::new_v4()
    }
    
    fn event_type(&self) -> &'static str {
        "WalletUserCreated"
    }

    fn aggregate_id(&self) -> String {
        self.wallet_address.to_string()
    }

    fn aggregate_version(&self) -> u64 {
        self.aggregate_version
    }

    fn occurred_at(&self) -> DateTime<Utc> {
        self.occurred_at
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
    pub wallet_address: WalletAddress,
    pub aggregate_version: u64,
    pub occurred_at: DateTime<Utc>,
}

impl WalletUserActivatedEvent {
    pub fn new(wallet_address: WalletAddress, aggregate_version: u64) -> Self {
        Self {
            wallet_address,
            aggregate_version,
            occurred_at: Utc::now(),
        }
    }
}

impl DomainEvent for WalletUserActivatedEvent {
    fn event_id(&self) -> Uuid {
        Uuid::new_v4()
    }
    
    fn event_type(&self) -> &'static str {
        "WalletUserActivated"
    }

    fn aggregate_id(&self) -> String {
        self.wallet_address.to_string()
    }

    fn aggregate_version(&self) -> u64 {
        self.aggregate_version
    }

    fn occurred_at(&self) -> DateTime<Utc> {
        self.occurred_at
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
    pub wallet_address: WalletAddress,
    pub reason: String,
    pub revoked_permissions: Vec<Permission>,
    pub aggregate_version: u64,
    pub occurred_at: DateTime<Utc>,
}

impl WalletUserDeactivatedEvent {
    pub fn new(
        wallet_address: WalletAddress,
        reason: String,
        revoked_permissions: Vec<Permission>,
        aggregate_version: u64,
    ) -> Self {
        Self {
            wallet_address,
            reason,
            revoked_permissions,
            aggregate_version,
            occurred_at: Utc::now(),
        }
    }
}

impl DomainEvent for WalletUserDeactivatedEvent {
    fn event_id(&self) -> Uuid {
        Uuid::new_v4()
    }
    
    fn event_type(&self) -> &'static str {
        "WalletUserDeactivated"
    }

    fn aggregate_id(&self) -> String {
        self.wallet_address.to_string()
    }

    fn aggregate_version(&self) -> u64 {
        self.aggregate_version
    }

    fn occurred_at(&self) -> DateTime<Utc> {
        self.occurred_at
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
    pub wallet_address: WalletAddress,
    pub old_permissions: Vec<Permission>,
    pub new_permissions: Vec<Permission>,
    pub aggregate_version: u64,
    pub occurred_at: DateTime<Utc>,
}

impl WalletPermissionsUpdatedEvent {
    pub fn new(
        wallet_address: WalletAddress,
        old_permissions: Vec<Permission>,
        new_permissions: Vec<Permission>,
        aggregate_version: u64,
    ) -> Self {
        Self {
            wallet_address,
            old_permissions,
            new_permissions,
            aggregate_version,
            occurred_at: Utc::now(),
        }
    }
}

impl DomainEvent for WalletPermissionsUpdatedEvent {
    fn event_id(&self) -> Uuid {
        Uuid::new_v4()
    }
    
    fn event_type(&self) -> &'static str {
        "WalletPermissionsUpdated"
    }

    fn aggregate_id(&self) -> String {
        self.wallet_address.to_string()
    }

    fn aggregate_version(&self) -> u64 {
        self.aggregate_version
    }

    fn occurred_at(&self) -> DateTime<Utc> {
        self.occurred_at
    }
    
    fn to_json(&self) -> Result<String, Box<dyn std::error::Error>> {
        Ok(serde_json::to_string(self)?)
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

// TierLevelChangedEvent removed - using permission groups instead