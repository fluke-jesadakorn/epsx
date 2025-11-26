use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::domain::shared_kernel::DomainEvent;
use crate::domain::shared_kernel::domain_event::EventMetadata;
use crate::domain::wallet_management::value_objects::{WalletAddress, Email};

/// Event raised when a wallet is successfully authenticated via Web3
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalletAuthenticatedEvent {
    pub wallet_address: WalletAddress,
    pub nonce_used: String,
    pub signature: String,
    pub authentication_method: String, // "siwe"
    pub metadata: EventMetadata,
}

impl WalletAuthenticatedEvent {
    pub fn new(
        wallet_address: WalletAddress,
        nonce_used: String,
        signature: String,
        version: u64,
    ) -> Self {
        Self {
            wallet_address: wallet_address.clone(),
            nonce_used,
            signature,
            authentication_method: "siwe".to_string(),
            metadata: EventMetadata::new(wallet_address.to_string(), version),
        }
    }
}

impl DomainEvent for WalletAuthenticatedEvent {
    fn event_type(&self) -> &'static str {
        "WalletAuthenticated"
    }

    fn aggregate_type(&self) -> &'static str {
        "WalletUser"
    }

    fn aggregate_id(&self) -> String {
        self.wallet_address.to_string()
    }

    fn event_id(&self) -> Uuid {
        self.metadata.event_id
    }

    fn occurred_at(&self) -> DateTime<Utc> {
        self.metadata.occurred_at
    }

    fn aggregate_version(&self) -> u64 {
        self.metadata.aggregate_version
    }

    fn to_json(&self) -> Result<String, Box<dyn std::error::Error>> {
        Ok(serde_json::to_string(self)?)
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

/// Event raised when a wallet is linked to an existing user account
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalletLinkedToUserEvent {
    pub wallet_address: WalletAddress,
    pub previous_email: Option<Email>,
    pub linked_via: String, // "migration", "manual_link", etc.
    pub metadata: EventMetadata,
}

impl WalletLinkedToUserEvent {
    pub fn new(
        wallet_address: WalletAddress,
        previous_email: Option<Email>,
        linked_via: String,
        version: u64,
    ) -> Self {
        Self {
            wallet_address: wallet_address.clone(),
            previous_email,
            linked_via,
            metadata: EventMetadata::new(wallet_address.to_string(), version),
        }
    }
}

impl DomainEvent for WalletLinkedToUserEvent {
    fn event_type(&self) -> &'static str {
        "WalletLinkedToUser"
    }

    fn aggregate_type(&self) -> &'static str {
        "WalletUser"
    }

    fn aggregate_id(&self) -> String {
        self.wallet_address.to_string()
    }

    fn event_id(&self) -> Uuid {
        self.metadata.event_id
    }

    fn occurred_at(&self) -> DateTime<Utc> {
        self.metadata.occurred_at
    }

    fn aggregate_version(&self) -> u64 {
        self.metadata.aggregate_version
    }

    fn to_json(&self) -> Result<String, Box<dyn std::error::Error>> {
        Ok(serde_json::to_string(self)?)
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

/// Event raised when a user is created via wallet authentication
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserCreatedViaWalletEvent {
    pub wallet_address: WalletAddress,
    pub temporary_email: Email,
    pub authentication_method: String,
    pub metadata: EventMetadata,
}

impl UserCreatedViaWalletEvent {
    pub fn new(
        wallet_address: WalletAddress,
        temporary_email: Email,
        version: u64,
    ) -> Self {
        Self {
            wallet_address: wallet_address.clone(),
            temporary_email,
            authentication_method: "web3_wallet".to_string(),
            metadata: EventMetadata::new(wallet_address.to_string(), version),
        }
    }
}

impl DomainEvent for UserCreatedViaWalletEvent {
    fn event_type(&self) -> &'static str {
        "UserCreatedViaWallet"
    }

    fn aggregate_type(&self) -> &'static str {
        "WalletUser"
    }

    fn aggregate_id(&self) -> String {
        self.wallet_address.to_string()
    }

    fn event_id(&self) -> Uuid {
        self.metadata.event_id
    }

    fn occurred_at(&self) -> DateTime<Utc> {
        self.metadata.occurred_at
    }

    fn aggregate_version(&self) -> u64 {
        self.metadata.aggregate_version
    }

    fn to_json(&self) -> Result<String, Box<dyn std::error::Error>> {
        Ok(serde_json::to_string(self)?)
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

/// Event raised when Web3 permission is automatically granted
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Web3PermissionGrantedEvent {
    pub wallet_address: WalletAddress,
    pub permission: String,
    pub permission_type: String, // nft_gated, token_gated, dao_granted
    pub contract_address: String,
    pub network: String,
    pub verification_data: serde_json::Value,
    pub metadata: EventMetadata,
}

impl Web3PermissionGrantedEvent {
    pub fn new(
        wallet_address: WalletAddress,
        permission: String,
        permission_type: String,
        contract_address: String,
        network: String,
        verification_data: serde_json::Value,
        version: u64,
    ) -> Self {
        Self {
            wallet_address: wallet_address.clone(),
            permission,
            permission_type,
            contract_address,
            network,
            verification_data,
            metadata: EventMetadata::new(wallet_address.to_string(), version),
        }
    }
}

impl DomainEvent for Web3PermissionGrantedEvent {
    fn event_type(&self) -> &'static str {
        "Web3PermissionGranted"
    }

    fn aggregate_type(&self) -> &'static str {
        "WalletUser"
    }

    fn aggregate_id(&self) -> String {
        self.wallet_address.to_string()
    }

    fn event_id(&self) -> Uuid {
        self.metadata.event_id
    }

    fn occurred_at(&self) -> DateTime<Utc> {
        self.metadata.occurred_at
    }

    fn aggregate_version(&self) -> u64 {
        self.metadata.aggregate_version
    }

    fn to_json(&self) -> Result<String, Box<dyn std::error::Error>> {
        Ok(serde_json::to_string(self)?)
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

/// Event raised when DAO proposal for permission is created
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DAOPermissionProposalCreatedEvent {
    pub proposal_id: Uuid,
    pub dao_contract_address: String,
    pub network: String,
    pub on_chain_proposal_id: String,
    pub target_wallet_address: WalletAddress,
    pub permission: String,
    pub title: String,
    pub proposer_address: String,
    pub metadata: EventMetadata,
}

impl DAOPermissionProposalCreatedEvent {
    pub fn new(
        proposal_id: Uuid,
        dao_contract_address: String,
        network: String,
        on_chain_proposal_id: String,
        target_wallet_address: WalletAddress,
        permission: String,
        title: String,
        proposer_address: String,
        version: u64,
    ) -> Self {
        Self {
            proposal_id,
            dao_contract_address,
            network,
            on_chain_proposal_id,
            target_wallet_address,
            permission,
            title,
            proposer_address,
            metadata: EventMetadata::new(proposal_id.to_string(), version),
        }
    }
}

impl DomainEvent for DAOPermissionProposalCreatedEvent {
    fn event_type(&self) -> &'static str {
        "DAOPermissionProposalCreated"
    }

    fn aggregate_type(&self) -> &'static str {
        "DAOProposal"
    }

    fn aggregate_id(&self) -> String {
        self.proposal_id.to_string()
    }

    fn event_id(&self) -> Uuid {
        self.metadata.event_id
    }

    fn occurred_at(&self) -> DateTime<Utc> {
        self.metadata.occurred_at
    }

    fn aggregate_version(&self) -> u64 {
        self.metadata.aggregate_version
    }

    fn to_json(&self) -> Result<String, Box<dyn std::error::Error>> {
        Ok(serde_json::to_string(self)?)
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}