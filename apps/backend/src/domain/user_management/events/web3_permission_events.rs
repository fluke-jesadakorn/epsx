use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;
use crate::domain::shared_kernel::DomainEvent;
use crate::domain::user_management::value_objects::{Permission, WalletAddress};

/// Event raised when a Web3 permission is validated against blockchain state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Web3PermissionValidatedEvent {
    pub wallet_address: WalletAddress,
    pub permission: Permission,
    pub validation_result: bool,
    pub chain_id: u64,
    pub validation_metadata: HashMap<String, String>,
    pub aggregate_version: u64,
    pub occurred_at: DateTime<Utc>,
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
            wallet_address,
            permission,
            validation_result,
            chain_id,
            validation_metadata,
            aggregate_version,
            occurred_at: Utc::now(),
        }
    }
}

impl DomainEvent for Web3PermissionValidatedEvent {
    fn event_id(&self) -> Uuid {
        Uuid::new_v4()
    }
    
    fn event_type(&self) -> &'static str {
        "Web3PermissionValidated"
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

/// Event raised when an NFT-gated permission is granted
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NftPermissionGrantedEvent {
    pub wallet_address: WalletAddress,
    pub permission: Permission,
    pub contract_address: String,
    pub token_ids: Vec<u64>,
    pub chain_id: u64,
    pub nft_metadata: HashMap<String, String>,
    pub aggregate_version: u64,
    pub occurred_at: DateTime<Utc>,
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
            wallet_address,
            permission,
            contract_address,
            token_ids,
            chain_id,
            nft_metadata,
            aggregate_version,
            occurred_at: Utc::now(),
        }
    }
}

impl DomainEvent for NftPermissionGrantedEvent {
    fn event_id(&self) -> Uuid {
        Uuid::new_v4()
    }
    
    fn event_type(&self) -> &'static str {
        "NftPermissionGranted"
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

/// Event raised when a token-gated permission is granted
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenPermissionGrantedEvent {
    pub wallet_address: WalletAddress,
    pub permission: Permission,
    pub contract_address: String,
    pub token_balance: String,
    pub min_balance_required: String,
    pub chain_id: u64,
    pub token_metadata: HashMap<String, String>,
    pub aggregate_version: u64,
    pub occurred_at: DateTime<Utc>,
}

impl TokenPermissionGrantedEvent {
    pub fn new(
        wallet_address: WalletAddress,
        permission: Permission,
        contract_address: String,
        token_balance: String,
        min_balance_required: String,
        chain_id: u64,
        token_metadata: HashMap<String, String>,
        aggregate_version: u64,
    ) -> Self {
        Self {
            wallet_address,
            permission,
            contract_address,
            token_balance,
            min_balance_required,
            chain_id,
            token_metadata,
            aggregate_version,
            occurred_at: Utc::now(),
        }
    }
}

impl DomainEvent for TokenPermissionGrantedEvent {
    fn event_id(&self) -> Uuid {
        Uuid::new_v4()
    }
    
    fn event_type(&self) -> &'static str {
        "TokenPermissionGranted"
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

/// Event raised when a DAO governance permission is granted
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DaoPermissionGrantedEvent {
    pub wallet_address: WalletAddress,
    pub permission: Permission,
    pub dao_contract: String,
    pub voting_power: String,
    pub min_voting_power_required: String,
    pub chain_id: u64,
    pub dao_metadata: HashMap<String, String>,
    pub proposal_participation: Option<HashMap<String, String>>,
    pub aggregate_version: u64,
    pub occurred_at: DateTime<Utc>,
}

impl DaoPermissionGrantedEvent {
    pub fn new(
        wallet_address: WalletAddress,
        permission: Permission,
        dao_contract: String,
        voting_power: String,
        min_voting_power_required: String,
        chain_id: u64,
        dao_metadata: HashMap<String, String>,
        proposal_participation: Option<HashMap<String, String>>,
        aggregate_version: u64,
    ) -> Self {
        Self {
            wallet_address,
            permission,
            dao_contract,
            voting_power,
            min_voting_power_required,
            chain_id,
            dao_metadata,
            proposal_participation,
            aggregate_version,
            occurred_at: Utc::now(),
        }
    }
}

impl DomainEvent for DaoPermissionGrantedEvent {
    fn event_id(&self) -> Uuid {
        Uuid::new_v4()
    }
    
    fn event_type(&self) -> &'static str {
        "DaoPermissionGranted"
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

/// Event raised when Web3 permission validation fails
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Web3PermissionValidationFailedEvent {
    pub wallet_address: WalletAddress,
    pub permission: Permission,
    pub failure_reason: String,
    pub chain_id: u64,
    pub error_details: HashMap<String, String>,
    pub retry_count: u32,
    pub aggregate_version: u64,
    pub occurred_at: DateTime<Utc>,
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
            wallet_address,
            permission,
            failure_reason,
            chain_id,
            error_details,
            retry_count,
            aggregate_version,
            occurred_at: Utc::now(),
        }
    }
}

impl DomainEvent for Web3PermissionValidationFailedEvent {
    fn event_id(&self) -> Uuid {
        Uuid::new_v4()
    }
    
    fn event_type(&self) -> &'static str {
        "Web3PermissionValidationFailed"
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

/// Event raised when Web3 permissions are synchronized with blockchain state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Web3PermissionsSyncedEvent {
    pub wallet_address: WalletAddress,
    pub synced_permissions: Vec<Permission>,
    pub chain_id: u64,
    pub sync_duration_ms: u64,
    pub blockchain_block_number: u64,
    pub sync_metadata: HashMap<String, String>,
    pub aggregate_version: u64,
    pub occurred_at: DateTime<Utc>,
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
            wallet_address,
            synced_permissions,
            chain_id,
            sync_duration_ms,
            blockchain_block_number,
            sync_metadata,
            aggregate_version,
            occurred_at: Utc::now(),
        }
    }
}

impl DomainEvent for Web3PermissionsSyncedEvent {
    fn event_id(&self) -> Uuid {
        Uuid::new_v4()
    }
    
    fn event_type(&self) -> &'static str {
        "Web3PermissionsSynced"
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

/// Event raised when cross-chain permission validation occurs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrossChainPermissionValidatedEvent {
    pub wallet_address: WalletAddress,
    pub permission: Permission,
    pub source_chain_id: u64,
    pub target_chain_id: u64,
    pub validation_result: bool,
    pub cross_chain_metadata: HashMap<String, String>,
    pub bridge_used: Option<String>,
    pub aggregate_version: u64,
    pub occurred_at: DateTime<Utc>,
}

impl CrossChainPermissionValidatedEvent {
    pub fn new(
        wallet_address: WalletAddress,
        permission: Permission,
        source_chain_id: u64,
        target_chain_id: u64,
        validation_result: bool,
        cross_chain_metadata: HashMap<String, String>,
        bridge_used: Option<String>,
        aggregate_version: u64,
    ) -> Self {
        Self {
            wallet_address,
            permission,
            source_chain_id,
            target_chain_id,
            validation_result,
            cross_chain_metadata,
            bridge_used,
            aggregate_version,
            occurred_at: Utc::now(),
        }
    }
}

impl DomainEvent for CrossChainPermissionValidatedEvent {
    fn event_id(&self) -> Uuid {
        Uuid::new_v4()
    }
    
    fn event_type(&self) -> &'static str {
        "CrossChainPermissionValidated"
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