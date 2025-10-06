use crate::prelude::*;
use crate::domain::shared_kernel::{DomainEvent, EventMetadata};
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use uuid::Uuid;
use std::collections::HashMap;
use crate::domain::wallet_management::value_objects::{Permission, WalletAddress};

/// Parameters for TokenPermissionGrantedEvent
pub struct TokenPermissionParams {
    pub wallet_address: WalletAddress,
    pub permission: Permission,
    pub contract_address: String,
    pub token_balance: String,
    pub min_balance_required: String,
    pub chain_id: u64,
    pub token_metadata: HashMap<String, String>,
    pub aggregate_version: u64,
}

/// Parameters for DaoPermissionGrantedEvent
pub struct DaoPermissionParams {
    pub wallet_address: WalletAddress,
    pub permission: Permission,
    pub dao_contract: String,
    pub voting_power: String,
    pub min_voting_power_required: String,
    pub chain_id: u64,
    pub dao_metadata: HashMap<String, String>,
    pub proposal_participation: Option<HashMap<String, String>>,
    pub aggregate_version: u64,
}

/// Parameters for CrossChainPermissionValidatedEvent
pub struct CrossChainPermissionParams {
    pub wallet_address: WalletAddress,
    pub permission: Permission,
    pub source_chain_id: u64,
    pub target_chain_id: u64,
    pub validation_result: bool,
    pub cross_chain_metadata: HashMap<String, String>,
    pub bridge_used: Option<String>,
    pub aggregate_version: u64,
}

/// Event raised when a Web3 permission is validated against blockchain state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Web3PermissionValidatedEvent {
    pub metadata: EventMetadata,
    pub wallet_address: WalletAddress,
    pub permission: Permission,
    pub validation_result: bool,
    pub chain_id: u64,
    pub validation_metadata: HashMap<String, String>,
}

impl Web3PermissionValidatedEvent {
    pub fn new(
        wallet_address: WalletAddress,
        permission: Permission,
        validation_result: bool,
        chain_id: u64,
        validation_metadata: HashMap<String, String>,
        aggregate_version: u64,
    ) -> Self {
        Self {
            metadata: EventMetadata::new(wallet_address.to_string(), aggregate_version),
            wallet_address,
            permission,
            validation_result,
            chain_id,
            validation_metadata,
        }
    }
}

impl DomainEvent for Web3PermissionValidatedEvent {
    fn event_id(&self) -> Uuid {
        self.metadata.event_id
    }

    fn event_type(&self) -> &'static str {
        "Web3PermissionValidated"
    }

    fn aggregate_type(&self) -> &'static str {
        "Web3Permission"
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

/// Event raised when an NFT-gated permission is granted
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NftPermissionGrantedEvent {
    pub metadata: EventMetadata,
    pub wallet_address: WalletAddress,
    pub permission: Permission,
    pub contract_address: String,
    pub token_ids: Vec<u64>,
    pub chain_id: u64,
    pub nft_metadata: HashMap<String, String>,
}

impl NftPermissionGrantedEvent {
    pub fn new(
        wallet_address: WalletAddress,
        permission: Permission,
        contract_address: String,
        token_ids: Vec<u64>,
        chain_id: u64,
        nft_metadata: HashMap<String, String>,
        aggregate_version: u64,
    ) -> Self {
        Self {
            metadata: EventMetadata::new(wallet_address.to_string(), aggregate_version),
            wallet_address,
            permission,
            contract_address,
            token_ids,
            chain_id,
            nft_metadata,
        }
    }
}

impl DomainEvent for NftPermissionGrantedEvent {
    fn event_id(&self) -> Uuid {
        self.metadata.event_id
    }

    fn event_type(&self) -> &'static str {
        "NftPermissionGranted"
    }

    fn aggregate_type(&self) -> &'static str {
        "Web3Permission"
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

/// Event raised when a token-gated permission is granted
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenPermissionGrantedEvent {
    pub metadata: EventMetadata,
    pub wallet_address: WalletAddress,
    pub permission: Permission,
    pub contract_address: String,
    pub token_balance: String,
    pub min_balance_required: String,
    pub chain_id: u64,
    pub token_metadata: HashMap<String, String>,
}

impl TokenPermissionGrantedEvent {
    pub fn new(params: TokenPermissionParams) -> Self {
        Self {
            metadata: EventMetadata::new(params.wallet_address.to_string(), params.aggregate_version),
            wallet_address: params.wallet_address,
            permission: params.permission,
            contract_address: params.contract_address,
            token_balance: params.token_balance,
            min_balance_required: params.min_balance_required,
            chain_id: params.chain_id,
            token_metadata: params.token_metadata,
        }
    }
}

impl DomainEvent for TokenPermissionGrantedEvent {
    fn event_id(&self) -> Uuid {
        self.metadata.event_id
    }

    fn event_type(&self) -> &'static str {
        "TokenPermissionGranted"
    }

    fn aggregate_type(&self) -> &'static str {
        "Web3Permission"
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

/// Event raised when a DAO governance permission is granted
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DaoPermissionGrantedEvent {
    pub metadata: EventMetadata,
    pub wallet_address: WalletAddress,
    pub permission: Permission,
    pub dao_contract: String,
    pub voting_power: String,
    pub min_voting_power_required: String,
    pub chain_id: u64,
    pub dao_metadata: HashMap<String, String>,
    pub proposal_participation: Option<HashMap<String, String>>,
}

impl DaoPermissionGrantedEvent {
    pub fn new(params: DaoPermissionParams) -> Self {
        Self {
            metadata: EventMetadata::new(params.wallet_address.to_string(), params.aggregate_version),
            wallet_address: params.wallet_address,
            permission: params.permission,
            dao_contract: params.dao_contract,
            voting_power: params.voting_power,
            min_voting_power_required: params.min_voting_power_required,
            chain_id: params.chain_id,
            dao_metadata: params.dao_metadata,
            proposal_participation: params.proposal_participation,
        }
    }
}

impl DomainEvent for DaoPermissionGrantedEvent {
    fn event_id(&self) -> Uuid {
        self.metadata.event_id
    }

    fn event_type(&self) -> &'static str {
        "DaoPermissionGranted"
    }

    fn aggregate_type(&self) -> &'static str {
        "Web3Permission"
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

/// Event raised when Web3 permission validation fails
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Web3PermissionValidationFailedEvent {
    pub metadata: EventMetadata,
    pub wallet_address: WalletAddress,
    pub permission: Permission,
    pub failure_reason: String,
    pub chain_id: u64,
    pub error_details: HashMap<String, String>,
    pub retry_count: u32,
}

impl Web3PermissionValidationFailedEvent {
    pub fn new(
        wallet_address: WalletAddress,
        permission: Permission,
        failure_reason: String,
        chain_id: u64,
        error_details: HashMap<String, String>,
        retry_count: u32,
        aggregate_version: u64,
    ) -> Self {
        Self {
            metadata: EventMetadata::new(wallet_address.to_string(), aggregate_version),
            wallet_address,
            permission,
            failure_reason,
            chain_id,
            error_details,
            retry_count,
        }
    }
}

impl DomainEvent for Web3PermissionValidationFailedEvent {
    fn event_id(&self) -> Uuid {
        self.metadata.event_id
    }

    fn event_type(&self) -> &'static str {
        "Web3PermissionValidationFailed"
    }

    fn aggregate_type(&self) -> &'static str {
        "Web3Permission"
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

/// Event raised when Web3 permissions are synchronized with blockchain state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Web3PermissionsSyncedEvent {
    pub metadata: EventMetadata,
    pub wallet_address: WalletAddress,
    pub synced_permissions: Vec<Permission>,
    pub chain_id: u64,
    pub sync_duration_ms: u64,
    pub blockchain_block_number: u64,
    pub sync_metadata: HashMap<String, String>,
}

impl Web3PermissionsSyncedEvent {
    pub fn new(
        wallet_address: WalletAddress,
        synced_permissions: Vec<Permission>,
        chain_id: u64,
        sync_duration_ms: u64,
        blockchain_block_number: u64,
        sync_metadata: HashMap<String, String>,
        aggregate_version: u64,
    ) -> Self {
        Self {
            metadata: EventMetadata::new(wallet_address.to_string(), aggregate_version),
            wallet_address,
            synced_permissions,
            chain_id,
            sync_duration_ms,
            blockchain_block_number,
            sync_metadata,
        }
    }
}

impl DomainEvent for Web3PermissionsSyncedEvent {
    fn event_id(&self) -> Uuid {
        self.metadata.event_id
    }

    fn event_type(&self) -> &'static str {
        "Web3PermissionsSynced"
    }

    fn aggregate_type(&self) -> &'static str {
        "Web3Permission"
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

/// Event raised when cross-chain permission validation occurs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrossChainPermissionValidatedEvent {
    pub metadata: EventMetadata,
    pub wallet_address: WalletAddress,
    pub permission: Permission,
    pub source_chain_id: u64,
    pub target_chain_id: u64,
    pub validation_result: bool,
    pub cross_chain_metadata: HashMap<String, String>,
    pub bridge_used: Option<String>,
}

impl CrossChainPermissionValidatedEvent {
    pub fn new(params: CrossChainPermissionParams) -> Self {
        Self {
            metadata: EventMetadata::new(params.wallet_address.to_string(), params.aggregate_version),
            wallet_address: params.wallet_address,
            permission: params.permission,
            source_chain_id: params.source_chain_id,
            target_chain_id: params.target_chain_id,
            validation_result: params.validation_result,
            cross_chain_metadata: params.cross_chain_metadata,
            bridge_used: params.bridge_used,
        }
    }
}

impl DomainEvent for CrossChainPermissionValidatedEvent {
    fn event_id(&self) -> Uuid {
        self.metadata.event_id
    }

    fn event_type(&self) -> &'static str {
        "CrossChainPermissionValidated"
    }

    fn aggregate_type(&self) -> &'static str {
        "Web3Permission"
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