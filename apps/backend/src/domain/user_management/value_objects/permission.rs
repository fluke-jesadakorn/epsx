use chrono::{DateTime, Utc};
use std::fmt;
use std::collections::BTreeMap;
use serde::{Deserialize, Serialize};

use crate::domain::shared_kernel::ValueObject;
use crate::domain::shared_kernel::value_object::ValueObjectError;

/// Web3 Permission Type - determines how permission is validated
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PermissionType {
    /// Manual - Traditional admin-assigned permissions
    Manual,
    /// NFT-gated - Permissions based on NFT ownership
    NftGated {
        /// NFT contract address
        contract_address: String,
        /// Required NFT token IDs (empty = any NFT from contract)
        token_ids: Vec<u64>,
        /// Blockchain chain ID (1 = Ethereum, 56 = BSC, 137 = Polygon)
        chain_id: u64,
    },
    /// Token-gated - Permissions based on ERC-20 token holdings
    TokenGated {
        /// Token contract address
        contract_address: String,
        /// Minimum token balance required (in wei/smallest unit)
        min_balance: String,
        /// Blockchain chain ID
        chain_id: u64,
    },
    /// DAO governance - Permissions based on DAO membership/voting power
    DaoGovernance {
        /// DAO contract address (governance token or DAO contract)
        dao_contract: String,
        /// Minimum voting power required
        min_voting_power: String,
        /// Blockchain chain ID
        chain_id: u64,
        /// Additional DAO-specific metadata
        governance_metadata: std::collections::BTreeMap<String, String>,
    },
}

/// Permission value object
/// Represents a structured permission in the format "platform:resource:action"
/// with Web3-based validation and optional temporal constraints
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Permission {
    /// The full permission string (e.g., "admin:users:manage")
    permission: String,
    /// Optional expiration timestamp for time-limited permissions
    expires_at: Option<DateTime<Utc>>,
    /// Platform scope (e.g., "admin", "epsx", "epsx-pay")
    platform: String,
    /// Resource type (e.g., "users", "analytics", "payments")
    resource: String,
    /// Action allowed (e.g., "view", "manage", "create", "delete")
    action: String,
    /// Web3 permission type and validation rules
    permission_type: PermissionType,
    /// Optional metadata for additional context
    metadata: std::collections::BTreeMap<String, String>,
}

impl Permission {
    /// Create a new manual permission from a permission string
    pub fn new(permission: impl Into<String>) -> Result<Self, ValueObjectError> {
        let permission = permission.into();
        let parts = Self::parse_permission(&permission)?;
        
        let instance = Self {
            permission: permission.clone(),
            expires_at: None,
            platform: parts.0,
            resource: parts.1,
            action: parts.2,
            permission_type: PermissionType::Manual,
            metadata: std::collections::BTreeMap::new(),
        };
        
        instance.validate()?;
        Ok(instance)
    }
    
    /// Create a new NFT-gated permission
    pub fn new_nft_gated(
        permission: impl Into<String>,
        contract_address: String,
        token_ids: Vec<u64>,
        chain_id: u64,
    ) -> Result<Self, ValueObjectError> {
        let permission = permission.into();
        let parts = Self::parse_permission(&permission)?;
        
        let instance = Self {
            permission: permission.clone(),
            expires_at: None,
            platform: parts.0,
            resource: parts.1,
            action: parts.2,
            permission_type: PermissionType::NftGated {
                contract_address,
                token_ids,
                chain_id,
            },
            metadata: std::collections::BTreeMap::new(),
        };
        
        instance.validate()?;
        Ok(instance)
    }
    
    /// Create a new token-gated permission
    pub fn new_token_gated(
        permission: impl Into<String>,
        contract_address: String,
        min_balance: String,
        chain_id: u64,
    ) -> Result<Self, ValueObjectError> {
        let permission = permission.into();
        let parts = Self::parse_permission(&permission)?;
        
        let instance = Self {
            permission: permission.clone(),
            expires_at: None,
            platform: parts.0,
            resource: parts.1,
            action: parts.2,
            permission_type: PermissionType::TokenGated {
                contract_address,
                min_balance,
                chain_id,
            },
            metadata: std::collections::BTreeMap::new(),
        };
        
        instance.validate()?;
        Ok(instance)
    }
    
    /// Create a new DAO governance permission
    pub fn new_dao_governance(
        permission: impl Into<String>,
        dao_contract: String,
        min_voting_power: String,
        chain_id: u64,
        governance_metadata: BTreeMap<String, String>,
    ) -> Result<Self, ValueObjectError> {
        let permission = permission.into();
        let parts = Self::parse_permission(&permission)?;
        
        let instance = Self {
            permission: permission.clone(),
            expires_at: None,
            platform: parts.0,
            resource: parts.1,
            action: parts.2,
            permission_type: PermissionType::DaoGovernance {
                dao_contract,
                min_voting_power,
                chain_id,
                governance_metadata,
            },
            metadata: std::collections::BTreeMap::new(),
        };
        
        instance.validate()?;
        Ok(instance)
    }
    
    /// Create a new manual permission with expiration
    pub fn new_with_expiration(
        permission: impl Into<String>,
        expires_at: DateTime<Utc>
    ) -> Result<Self, ValueObjectError> {
        let mut perm = Self::new(permission)?;
        perm.expires_at = Some(expires_at);
        Ok(perm)
    }
    
    /// Set expiration for any permission type
    pub fn with_expiration(mut self, expires_at: DateTime<Utc>) -> Self {
        self.expires_at = Some(expires_at);
        self
    }
    
    /// Add metadata to permission
    pub fn with_metadata(mut self, key: String, value: String) -> Self {
        self.metadata.insert(key, value);
        self
    }
    
    /// Create a manual permission from embedded timestamp format
    /// Format: "platform:resource:action:unix_timestamp"
    pub fn from_embedded_timestamp(permission: impl Into<String>) -> Result<Self, ValueObjectError> {
        let permission = permission.into();
        let parts: Vec<&str> = permission.split(':').collect();
        
        if parts.len() == 4 {
            // Has timestamp
            let timestamp = parts[3].parse::<i64>()
                .map_err(|_| ValueObjectError::InvalidFormat("Invalid timestamp format".to_string()))?;
            
            let expires_at = DateTime::from_timestamp(timestamp, 0)
                .ok_or_else(|| ValueObjectError::InvalidFormat("Invalid timestamp value".to_string()))?;
            
            let base_permission = format!("{}:{}:{}", parts[0], parts[1], parts[2]);
            let mut perm = Self::new(base_permission)?;
            perm.expires_at = Some(expires_at);
            Ok(perm)
        } else {
            // No timestamp, regular permission
            Self::new(permission)
        }
    }
    
    /// Get the full permission string
    pub fn as_str(&self) -> &str {
        &self.permission
    }
    
    /// Get the platform scope
    pub fn platform(&self) -> &str {
        &self.platform
    }
    
    /// Get the resource
    pub fn resource(&self) -> &str {
        &self.resource
    }
    
    /// Get the action
    pub fn action(&self) -> &str {
        &self.action
    }
    
    /// Get the expiration timestamp
    pub fn expires_at(&self) -> Option<DateTime<Utc>> {
        self.expires_at
    }
    
    /// Get the permission type
    pub fn permission_type(&self) -> &PermissionType {
        &self.permission_type
    }
    
    /// Get metadata
    pub fn metadata(&self) -> &BTreeMap<String, String> {
        &self.metadata
    }
    
    /// Check if permission is manual type
    pub fn is_manual(&self) -> bool {
        matches!(self.permission_type, PermissionType::Manual)
    }
    
    /// Check if permission is NFT-gated
    pub fn is_nft_gated(&self) -> bool {
        matches!(self.permission_type, PermissionType::NftGated { .. })
    }
    
    /// Check if permission is token-gated
    pub fn is_token_gated(&self) -> bool {
        matches!(self.permission_type, PermissionType::TokenGated { .. })
    }
    
    /// Check if permission is DAO governance
    pub fn is_dao_governance(&self) -> bool {
        matches!(self.permission_type, PermissionType::DaoGovernance { .. })
    }
    
    /// Check if permission requires Web3 validation
    pub fn requires_web3_validation(&self) -> bool {
        !self.is_manual()
    }
    
    /// Get chain ID for Web3 permissions
    pub fn chain_id(&self) -> Option<u64> {
        match &self.permission_type {
            PermissionType::Manual => None,
            PermissionType::NftGated { chain_id, .. } => Some(*chain_id),
            PermissionType::TokenGated { chain_id, .. } => Some(*chain_id),
            PermissionType::DaoGovernance { chain_id, .. } => Some(*chain_id),
        }
    }
    
    /// Get contract address for Web3 permissions
    pub fn contract_address(&self) -> Option<&String> {
        match &self.permission_type {
            PermissionType::Manual => None,
            PermissionType::NftGated { contract_address, .. } => Some(contract_address),
            PermissionType::TokenGated { contract_address, .. } => Some(contract_address),
            PermissionType::DaoGovernance { dao_contract, .. } => Some(dao_contract),
        }
    }
    
    /// Check if this permission has expired
    pub fn is_expired(&self) -> bool {
        if let Some(expires_at) = self.expires_at {
            Utc::now() > expires_at
        } else {
            false
        }
    }
    
    /// Check if this permission is active (not expired)
    pub fn is_active(&self) -> bool {
        !self.is_expired()
    }
    
    /// Convert to embedded timestamp format if expires_at is set
    pub fn to_embedded_format(&self) -> String {
        if let Some(expires_at) = self.expires_at {
            format!("{}:{}", self.permission, expires_at.timestamp())
        } else {
            self.permission.clone()
        }
    }
    
    /// Check if this permission starts with the given prefix
    pub fn starts_with(&self, prefix: &str) -> bool {
        self.permission.starts_with(prefix)
    }
    
    /// Check if this permission matches another permission pattern
    /// Supports wildcards with "*"
    pub fn matches(&self, pattern: &str) -> bool {
        // Handle wildcard matching
        if pattern.ends_with("*") {
            let prefix = &pattern[..pattern.len() - 1];
            self.permission.starts_with(prefix)
        } else {
            self.permission == pattern
        }
    }
    
    /// Check if this permission grants access to a specific action on a resource
    /// For Web3 permissions, this only checks the permission pattern - Web3 validation must be done separately
    pub fn grants_access(&self, platform: &str, resource: &str, action: &str) -> bool {
        if self.is_expired() {
            return false;
        }
        
        // Check permission pattern match
        if !self.matches_permission_pattern(platform, resource, action) {
            return false;
        }
        
        // For manual permissions, pattern match is sufficient
        // For Web3 permissions, additional validation is required (done externally)
        true
    }
    
    /// Check if permission pattern matches (without Web3 validation)
    pub fn matches_permission_pattern(&self, platform: &str, resource: &str, action: &str) -> bool {
        // Exact match
        if self.platform == platform && self.resource == resource && self.action == action {
            return true;
        }
        
        // Wildcard matching
        if self.action == "*" && self.platform == platform && self.resource == resource {
            return true;
        }
        
        if self.resource == "*" && self.platform == platform {
            return true;
        }
        
        false
    }
    
    fn parse_permission(permission: &str) -> Result<(String, String, String), ValueObjectError> {
        let parts: Vec<&str> = permission.split(':').collect();
        
        if parts.len() < 3 {
            return Err(ValueObjectError::InvalidFormat(
                "Permission must be in format 'platform:resource:action'".to_string()
            ));
        }
        
        Ok((
            parts[0].to_string(),
            parts[1].to_string(),
            parts[2].to_string(),
        ))
    }
    
    /// Validate Ethereum address format (basic validation)
    fn is_valid_ethereum_address(address: &str) -> bool {
        // Basic validation: starts with 0x and is 42 characters long with valid hex
        if !address.starts_with("0x") || address.len() != 42 {
            return false;
        }
        
        let hex_part = &address[2..];
        hex_part.chars().all(|c| c.is_ascii_hexdigit())
    }
}

impl ValueObject for Permission {
    type Error = ValueObjectError;
    
    fn validate(&self) -> Result<(), Self::Error> {
        if self.permission.is_empty() {
            return Err(ValueObjectError::Required("Permission cannot be empty".to_string()));
        }
        
        // Validate format
        Self::parse_permission(&self.permission)?;
        
        // Validate individual parts
        if self.platform.is_empty() {
            return Err(ValueObjectError::Required("Platform cannot be empty".to_string()));
        }
        
        if self.resource.is_empty() {
            return Err(ValueObjectError::Required("Resource cannot be empty".to_string()));
        }
        
        if self.action.is_empty() {
            return Err(ValueObjectError::Required("Action cannot be empty".to_string()));
        }
        
        // Validate allowed characters
        let valid_chars = |s: &str| s.chars().all(|c| c.is_alphanumeric() || c == '_' || c == '-' || c == '*');
        
        if !valid_chars(&self.platform) || !valid_chars(&self.resource) || !valid_chars(&self.action) {
            return Err(ValueObjectError::InvalidFormat(
                "Permission parts can only contain alphanumeric characters, underscores, hyphens, and asterisks".to_string()
            ));
        }
        
        // Validate Web3 permission type specific rules
        match &self.permission_type {
            PermissionType::Manual => {}, // No additional validation needed
            PermissionType::NftGated { contract_address, chain_id, .. } => {
                if contract_address.is_empty() {
                    return Err(ValueObjectError::Required("NFT contract address cannot be empty".to_string()));
                }
                if !Self::is_valid_ethereum_address(contract_address) {
                    return Err(ValueObjectError::InvalidFormat("Invalid NFT contract address format".to_string()));
                }
                if *chain_id == 0 {
                    return Err(ValueObjectError::InvalidFormat("Chain ID must be greater than 0".to_string()));
                }
            },
            PermissionType::TokenGated { contract_address, min_balance, chain_id } => {
                if contract_address.is_empty() {
                    return Err(ValueObjectError::Required("Token contract address cannot be empty".to_string()));
                }
                if !Self::is_valid_ethereum_address(contract_address) {
                    return Err(ValueObjectError::InvalidFormat("Invalid token contract address format".to_string()));
                }
                if min_balance.is_empty() || min_balance.parse::<u128>().is_err() {
                    return Err(ValueObjectError::InvalidFormat("Minimum balance must be a valid number".to_string()));
                }
                if *chain_id == 0 {
                    return Err(ValueObjectError::InvalidFormat("Chain ID must be greater than 0".to_string()));
                }
            },
            PermissionType::DaoGovernance { dao_contract, min_voting_power, chain_id, .. } => {
                if dao_contract.is_empty() {
                    return Err(ValueObjectError::Required("DAO contract address cannot be empty".to_string()));
                }
                if !Self::is_valid_ethereum_address(dao_contract) {
                    return Err(ValueObjectError::InvalidFormat("Invalid DAO contract address format".to_string()));
                }
                if min_voting_power.is_empty() || min_voting_power.parse::<u128>().is_err() {
                    return Err(ValueObjectError::InvalidFormat("Minimum voting power must be a valid number".to_string()));
                }
                if *chain_id == 0 {
                    return Err(ValueObjectError::InvalidFormat("Chain ID must be greater than 0".to_string()));
                }
            },
        }
        
        Ok(())
    }
}

impl fmt::Display for Permission {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.permission_type {
            PermissionType::Manual => {
                write!(f, "{}[manual]", self.permission)
            },
            PermissionType::NftGated { contract_address, chain_id, .. } => {
                write!(f, "{}[nft:{}:{}]", self.permission, contract_address, chain_id)
            },
            PermissionType::TokenGated { contract_address, min_balance, chain_id } => {
                write!(f, "{}[token:{}:{}:{}]", self.permission, contract_address, min_balance, chain_id)
            },
            PermissionType::DaoGovernance { dao_contract, min_voting_power, chain_id, .. } => {
                write!(f, "{}[dao:{}:{}:{}]", self.permission, dao_contract, min_voting_power, chain_id)
            },
        }
    }
}

impl From<Permission> for String {
    fn from(permission: Permission) -> Self {
        permission.permission
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn valid_permission_should_pass() {
        let perm = Permission::new("admin:users:manage");
        assert!(perm.is_ok());
        
        let perm = perm.unwrap();
        assert_eq!(perm.platform(), "admin");
        assert_eq!(perm.resource(), "users");
        assert_eq!(perm.action(), "manage");
    }
    
    #[test]
    fn invalid_permission_format_should_fail() {
        let perm = Permission::new("invalid");
        assert!(perm.is_err());
    }
    
    #[test]
    fn permission_with_expiration() {
        let expires_at = Utc::now() + chrono::Duration::hours(1);
        let perm = Permission::new_with_expiration("admin:users:view", expires_at);
        assert!(perm.is_ok());
        
        let perm = perm.unwrap();
        assert!(!perm.is_expired());
        assert!(perm.is_active());
    }
    
    #[test]
    fn embedded_timestamp_format() {
        let timestamp = (Utc::now() + chrono::Duration::hours(1)).timestamp();
        let embedded = format!("admin:users:view:{}", timestamp);
        
        let perm = Permission::from_embedded_timestamp(&embedded);
        assert!(perm.is_ok());
        
        let perm = perm.unwrap();
        assert!(perm.expires_at().is_some());
        assert!(!perm.is_expired());
    }
    
    #[test]
    fn permission_matching() {
        let perm = Permission::new("admin:users:manage").unwrap();
        
        assert!(perm.matches("admin:users:manage"));
        assert!(perm.matches("admin:users:*"));
        assert!(perm.matches("admin:*"));
        assert!(!perm.matches("epsx:users:manage"));
    }
    
    #[test]
    fn permission_with_metadata_and_expiration() {
        let expires_at = Utc::now() + chrono::Duration::hours(1);
        let perm = Permission::new("admin:users:view")
            .unwrap()
            .with_expiration(expires_at)
            .with_metadata("granted_by".to_string(), "admin123".to_string())
            .with_metadata("reason".to_string(), "temporary access".to_string());
        
        assert!(perm.expires_at().is_some());
        assert!(!perm.is_expired());
        assert_eq!(perm.metadata().get("granted_by"), Some(&"admin123".to_string()));
        assert_eq!(perm.metadata().get("reason"), Some(&"temporary access".to_string()));
    }
    
    #[test]
    fn grants_access_logic() {
        let perm = Permission::new("admin:users:manage").unwrap();
        
        assert!(perm.grants_access("admin", "users", "manage"));
        assert!(!perm.grants_access("admin", "users", "delete"));
        assert!(!perm.grants_access("epsx", "users", "manage"));
        
        let wildcard_perm = Permission::new("admin:users:*").unwrap();
        assert!(wildcard_perm.grants_access("admin", "users", "manage"));
        assert!(wildcard_perm.grants_access("admin", "users", "delete"));
    }
    
    #[test]
    fn web3_permission_types() {
        // NFT-gated permission
        let nft_perm = Permission::new_nft_gated(
            "epsx:premium:access",
            "0x742d35Cc6634C0532925a3b8D369D7763F3c45c6".to_string(),
            vec![1, 2, 3],
            1, // Ethereum mainnet
        ).unwrap();
        
        assert!(nft_perm.is_nft_gated());
        assert!(!nft_perm.is_manual());
        assert!(nft_perm.requires_web3_validation());
        assert_eq!(nft_perm.chain_id(), Some(1));
        
        // Token-gated permission
        let token_perm = Permission::new_token_gated(
            "epsx:vip:access",
            "0x742d35Cc6634C0532925a3b8D369D7763F3c45c6".to_string(),
            "1000000000000000000".to_string(), // 1 token with 18 decimals
            56, // BSC
        ).unwrap();
        
        assert!(token_perm.is_token_gated());
        assert!(token_perm.requires_web3_validation());
        assert_eq!(token_perm.chain_id(), Some(56));
        
        // DAO governance permission
        let mut governance_metadata = std::collections::BTreeMap::new();
        governance_metadata.insert("proposal_threshold".to_string(), "100".to_string());
        
        let dao_perm = Permission::new_dao_governance(
            "epsx:governance:vote",
            "0x742d35Cc6634C0532925a3b8D369D7763F3c45c6".to_string(),
            "1000000000000000000".to_string(),
            137, // Polygon
            governance_metadata,
        ).unwrap();
        
        assert!(dao_perm.is_dao_governance());
        assert!(dao_perm.requires_web3_validation());
        assert_eq!(dao_perm.chain_id(), Some(137));
    }
    
    #[test]
    fn ethereum_address_validation() {
        assert!(Permission::is_valid_ethereum_address("0x742d35Cc6634C0532925a3b8D369D7763F3c45c6"));
        assert!(!Permission::is_valid_ethereum_address("742d35Cc6634C0532925a3b8D369D7763F3c45c6")); // No 0x prefix
        assert!(!Permission::is_valid_ethereum_address("0x742d35Cc")); // Too short
        assert!(!Permission::is_valid_ethereum_address("0xGGGd35Cc6634C0532925a3b8D369D7763F3c45c6")); // Invalid hex
    }
}