use chrono::{DateTime, Utc};
use std::collections::HashSet;
use serde::{Deserialize, Serialize};

use crate::domain::shared_kernel::{
    AggregateRoot, 
    DomainEvent, 
    ValueObject,
    aggregate_root::AggregateBase
};
use crate::core::errors::{AppError, AppResult};

use crate::domain::user_management::value_objects::{
    Permission, WalletAddress
};

use crate::domain::user_management::events::{
    WalletUserCreatedEvent,
    WalletUserActivatedEvent,
    WalletUserDeactivatedEvent,
    WalletPermissionsUpdatedEvent
};

/// WalletUser aggregate root - Pure Web3 user model
/// Represents a user in the system identified by their wallet address
/// This is the consistency boundary for wallet-based user operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalletUser {
    // Primary identity - wallet address is the key
    wallet_address: WalletAddress,
    
    // User status
    is_active: bool,
    
    // Web3 permissions system
    permissions: HashSet<Permission>,
    
    // Permission groups the user belongs to
    permission_groups: HashSet<String>,
    
    // Web3-specific metadata
    wallet_metadata: WalletMetadata,
    
    // Audit fields
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
    last_auth_at: Option<DateTime<Utc>>,
    
    // Aggregate infrastructure
    #[serde(flatten)]
    base: AggregateBase,
}

// TierLevel enum removed - using permission groups instead

/// Web3-specific metadata for wallet users
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalletMetadata {
    /// Blockchain networks the wallet has been verified on
    pub verified_networks: Vec<String>,
    /// Last permission verification timestamp
    pub last_permission_check: Option<DateTime<Utc>>,
    /// NFT-based permissions cache
    pub nft_permissions_cache: Option<serde_json::Value>,
    /// Token-based permissions cache
    pub token_permissions_cache: Option<serde_json::Value>,
    /// DAO governance data
    pub dao_memberships: Vec<String>,
    /// Custom metadata for Web3 features
    pub custom_data: serde_json::Value,
}

impl Default for WalletMetadata {
    fn default() -> Self {
        Self {
            verified_networks: vec!["ethereum".to_string()],
            last_permission_check: None,
            nft_permissions_cache: None,
            token_permissions_cache: None,
            dao_memberships: Vec::new(),
            custom_data: serde_json::Value::Object(serde_json::Map::new()),
        }
    }
}

impl WalletMetadata {
    /// Convert WalletMetadata to JSON for database storage
    pub fn to_json(&self) -> Result<serde_json::Value, serde_json::Error> {
        serde_json::to_value(self)
    }
    
    /// Create WalletMetadata from JSON for database reconstruction
    pub fn from_json(json: serde_json::Value) -> Result<Self, serde_json::Error> {
        serde_json::from_value(json)
    }
}

impl WalletUser {
    /// Create a new wallet user
    pub fn create(
        wallet_address: WalletAddress,
        initial_groups: HashSet<String>,
    ) -> AppResult<Self> {
        // Business rule: Wallet address must be valid  
        wallet_address.validate()
            .map_err(|e| AppError::validation_error(format!("Invalid wallet address: {}", e))
                .with_component("wallet_user"))?;
        
        let now = Utc::now();
        let base = AggregateBase::new();
        
        let mut user = Self {
            wallet_address: wallet_address.clone(),
            is_active: true, // New wallet users are active by default
            permissions: HashSet::new(),
            permission_groups: initial_groups.clone(),
            wallet_metadata: WalletMetadata::default(),
            created_at: now,
            updated_at: now,
            last_auth_at: None,
            base,
        };
        
        // Raise domain event
        user.base.add_event(Box::new(WalletUserCreatedEvent::new(
            wallet_address,
            initial_groups,
            user.base.version
        )));
        
        Ok(user)
    }
    
    /// Load existing wallet user (for repository reconstruction)
    pub fn load(
        wallet_address: WalletAddress,
        is_active: bool,
        permissions: HashSet<Permission>,
        permission_groups: HashSet<String>,
        wallet_metadata: WalletMetadata,
        created_at: DateTime<Utc>,
        updated_at: DateTime<Utc>,
        last_auth_at: Option<DateTime<Utc>>,
        version: u64,
    ) -> Self {
        let mut base = AggregateBase::new();
        base.version = version;
        base.created_at = created_at;
        base.updated_at = updated_at;
        
        Self {
            wallet_address,
            is_active,
            permissions,
            permission_groups,
            wallet_metadata,
            created_at,
            updated_at,
            last_auth_at,
            base,
        }
    }
    
    /// Get wallet address (primary key)
    pub fn wallet_address(&self) -> &WalletAddress {
        &self.wallet_address
    }
    
    /// Check if user is active
    pub fn is_active(&self) -> bool {
        self.is_active
    }
    
    /// Get user permissions
    pub fn permissions(&self) -> &HashSet<Permission> {
        &self.permissions
    }
    
    /// Get user permission groups
    pub fn permission_groups(&self) -> &HashSet<String> {
        &self.permission_groups
    }
    
    /// Get wallet metadata
    pub fn wallet_metadata(&self) -> &WalletMetadata {
        &self.wallet_metadata
    }
    
    /// Get last authentication timestamp
    pub fn last_auth_at(&self) -> Option<DateTime<Utc>> {
        self.last_auth_at
    }
    
    /// Activate wallet user
    pub fn activate(&mut self) -> AppResult<()> {
        if self.is_active {
            return Err(AppError::business_rule_violation("User is already active")
                .with_component("wallet_user"));
        }
        
        self.is_active = true;
        self.updated_at = Utc::now();
        
        // Raise domain event
        self.base.add_event(Box::new(WalletUserActivatedEvent::new(
            self.wallet_address.clone(),
            self.base.version
        )));
        
        Ok(())
    }
    
    /// Deactivate wallet user
    pub fn deactivate(&mut self, reason: String) -> AppResult<()> {
        if !self.is_active {
            return Err(AppError::business_rule_violation("User is already inactive")
                .with_component("wallet_user"));
        }
        
        self.is_active = false;
        self.updated_at = Utc::now();
        
        // Clear permissions when deactivating
        let old_permissions = self.permissions.clone();
        self.permissions.clear();
        
        // Raise domain event
        self.base.add_event(Box::new(WalletUserDeactivatedEvent::new(
            self.wallet_address.clone(),
            reason,
            old_permissions.into_iter().collect(),
            self.base.version
        )));
        
        Ok(())
    }
    
    /// Grant permission to wallet user
    pub fn grant_permission(&mut self, permission: Permission) -> AppResult<()> {
        // Business rule: User must be active to receive permissions
        if !self.is_active {
            return Err(AppError::business_rule_violation("Cannot grant permissions to inactive user")
                .with_component("wallet_user"));
        }
        
        // Business rule: Don't grant duplicate permissions
        if self.permissions.contains(&permission) {
            return Ok(()); // Idempotent operation
        }
        
        let old_permissions: Vec<Permission> = self.permissions.iter().cloned().collect();
        self.permissions.insert(permission.clone());
        self.updated_at = Utc::now();
        let new_permissions: Vec<Permission> = self.permissions.iter().cloned().collect();
        
        // Raise domain event
        self.base.add_event(Box::new(WalletPermissionsUpdatedEvent::new(
            self.wallet_address.clone(),
            old_permissions,
            new_permissions,
            self.base.version
        )));
        
        Ok(())
    }
    
    /// Revoke permission from wallet user
    pub fn revoke_permission(&mut self, permission: &Permission) -> AppResult<()> {
        if !self.permissions.contains(permission) {
            return Ok(()); // Idempotent operation
        }
        
        let old_permissions: Vec<Permission> = self.permissions.iter().cloned().collect();
        self.permissions.remove(permission);
        self.updated_at = Utc::now();
        let new_permissions: Vec<Permission> = self.permissions.iter().cloned().collect();
        
        // Raise domain event
        self.base.add_event(Box::new(WalletPermissionsUpdatedEvent::new(
            self.wallet_address.clone(),
            old_permissions,
            new_permissions,
            self.base.version
        )));
        
        Ok(())
    }
    
    /// Update permission groups
    pub fn update_permission_groups(&mut self, new_groups: HashSet<String>) -> AppResult<()> {
        if self.permission_groups == new_groups {
            return Ok(()); // No change needed
        }
        
        let _old_groups = self.permission_groups.clone();
        self.permission_groups = new_groups.clone();
        self.updated_at = Utc::now();
        
        // Permission groups changed - could raise an event here if needed
        // self.base.add_event(Box::new(PermissionGroupsChangedEvent::new(...)));
        
        Ok(())
    }
    
    /// Record authentication
    pub fn record_authentication(&mut self) -> AppResult<()> {
        self.last_auth_at = Some(Utc::now());
        self.updated_at = Utc::now();
        
        Ok(())
    }
    
    /// Update wallet metadata
    pub fn update_metadata(&mut self, metadata: WalletMetadata) -> AppResult<()> {
        self.wallet_metadata = metadata;
        self.updated_at = Utc::now();
        
        Ok(())
    }
    
    /// Check if user has specific permission
    pub fn has_permission(&self, permission: &Permission) -> bool {
        self.permissions.contains(permission)
    }
    
    /// Check if user has any admin permissions
    pub fn is_admin(&self) -> bool {
        self.permission_groups.contains("Enterprise Access Group") ||
        self.permissions.iter().any(|p| p.as_str().starts_with("admin:"))
    }
    
    /// Check if user has premium access
    pub fn is_premium(&self) -> bool {
        self.permission_groups.contains("Premium Access Group") ||
        self.permission_groups.contains("Professional Access Group") ||
        self.permission_groups.contains("Enterprise Access Group")
    }
    
    /// Update permissions in batch
    pub fn update_permissions(&mut self, new_permissions: HashSet<Permission>) -> AppResult<()> {
        let old_permissions = self.permissions.clone();
        self.permissions = new_permissions.clone();
        self.updated_at = Utc::now();
        
        // Raise domain event
        self.base.add_event(Box::new(WalletPermissionsUpdatedEvent::new(
            self.wallet_address.clone(),
            old_permissions.into_iter().collect(),
            new_permissions.into_iter().collect(),
            self.base.version
        )));
        
        Ok(())
    }
    
    /// Get created timestamp
    pub fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }
    
    /// Get updated timestamp
    pub fn updated_at(&self) -> DateTime<Utc> {
        self.updated_at
    }
}

impl AggregateRoot for WalletUser {
    type Id = WalletAddress;
    
    fn id(&self) -> &Self::Id {
        &self.wallet_address
    }
    
    fn version(&self) -> u64 {
        self.base.version
    }
    
    fn increment_version(&mut self) {
        self.base.increment_version();
    }
    
    fn uncommitted_events(&self) -> &[Box<dyn DomainEvent>] {
        &self.base.events
    }
    
    fn mark_events_as_committed(&mut self) {
        self.base.events.clear();
    }
    
    fn created_at(&self) -> DateTime<Utc> {
        self.base.created_at
    }
    
    fn updated_at(&self) -> DateTime<Utc> {
        self.base.updated_at
    }
    
    fn touch(&mut self) {
        self.base.touch();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::user_management::value_objects::WalletAddress;
    
    #[test]
    fn test_create_wallet_user() {
        let wallet_address = WalletAddress::new("0x742d35Cc67C9c24d4D3A6A5c9B1c4D6F8F8c8B8d").unwrap();
        let initial_groups = HashSet::from(["Basic Access Group".to_string()]);
        let user = WalletUser::create(wallet_address.clone(), initial_groups.clone()).unwrap();
        
        assert_eq!(user.wallet_address(), &wallet_address);
        assert!(user.is_active());
        assert_eq!(user.permission_groups(), &initial_groups);
        assert!(user.permissions().is_empty());
    }
    
    #[test]
    fn test_grant_revoke_permission() {
        let wallet_address = WalletAddress::new("0x742d35Cc67C9c24d4D3A6A5c9B1c4D6F8F8c8B8d").unwrap();
        let initial_groups = HashSet::from(["Basic Access Group".to_string()]);
        let mut user = WalletUser::create(wallet_address, initial_groups).unwrap();
        
        let permission = Permission::new("epsx:read").unwrap();
        user.grant_permission(permission.clone()).unwrap();
        
        assert!(user.has_permission(&permission));
        
        user.revoke_permission(&permission).unwrap();
        assert!(!user.has_permission(&permission));
    }
    
    #[test]
    fn test_permission_group_update() {
        let wallet_address = WalletAddress::new("0x742d35Cc67C9c24d4D3A6A5c9B1c4D6F8F8c8B8d").unwrap();
        let initial_groups = HashSet::from(["Basic Access Group".to_string()]);
        let mut user = WalletUser::create(wallet_address, initial_groups).unwrap();
        
        assert!(!user.is_premium());
        
        let premium_groups = HashSet::from(["Premium Access Group".to_string()]);
        user.update_permission_groups(premium_groups.clone()).unwrap();
        assert!(user.is_premium());
        assert_eq!(user.permission_groups(), &premium_groups);
    }
    
    #[test]
    fn test_deactivate_user_clears_permissions() {
        let wallet_address = WalletAddress::new("0x742d35Cc67C9c24d4D3A6A5c9B1c4D6F8F8c8B8d").unwrap();
        let initial_groups = HashSet::from(["Basic Access Group".to_string()]);
        let mut user = WalletUser::create(wallet_address, initial_groups).unwrap();
        
        let permission = Permission::new("epsx:read").unwrap();
        user.grant_permission(permission.clone()).unwrap();
        assert!(user.has_permission(&permission));
        
        user.deactivate("Test deactivation".to_string()).unwrap();
        assert!(!user.is_active());
        assert!(user.permissions().is_empty());
    }
}