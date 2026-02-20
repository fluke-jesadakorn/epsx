use crate::prelude::*;

use std::collections::HashSet;

use crate::domain::shared_kernel::aggregate_root::AggregateBase;

use crate::domain::wallet_management::value_objects::{
    Permission, WalletAddress
};

use crate::domain::wallet_management::events::{
    WalletUserCreatedEvent,
    WalletUserActivatedEvent,
    WalletUserDeactivatedEvent,
    WalletPermissionsUpdatedEvent
};

/// Parameters for loading a WalletUser
pub struct WalletUserLoadParams {
    pub wallet_address: WalletAddress,
    pub is_active: bool,
    pub permissions: HashSet<Permission>,
    pub plans: HashSet<String>,
    pub wallet_metadata: WalletMetadata,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub last_auth_at: Option<DateTime<Utc>>,
    pub version: u64,
}

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
    
    // Permission plans the user belongs to
    plans: HashSet<String>,
    
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

// TierLevel enum removed - using permission plans instead

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
        initial_plans: HashSet<String>,
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
            plans: initial_plans.clone(),
            wallet_metadata: WalletMetadata::default(),
            created_at: now,
            updated_at: now,
            last_auth_at: None,
            base,
        };
        
        // Raise domain event
        user.base.add_event(Box::new(WalletUserCreatedEvent::new(
            wallet_address,
            initial_plans.clone(),
            user.base.version
        )));
        
        user.plans = initial_plans;
        
        Ok(user)
    }
    
    /// Load existing wallet user (for repository reconstruction)
    pub fn load(params: WalletUserLoadParams) -> Self {
        let mut base = AggregateBase::new();
        base.version = params.version;
        base.created_at = params.created_at;
        base.updated_at = params.updated_at;

        Self {
            wallet_address: params.wallet_address,
            is_active: params.is_active,
            permissions: params.permissions,
            plans: params.plans,
            wallet_metadata: params.wallet_metadata,
            created_at: params.created_at,
            updated_at: params.updated_at,
            last_auth_at: params.last_auth_at,
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
    
    /// Get user permission plans
    pub fn plans(&self) -> &HashSet<String> {
        &self.plans
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
    
    /// Update permission plans
    pub fn update_plans(&mut self, new_plans: HashSet<String>) -> AppResult<()> {
        if self.plans == new_plans {
            return Ok(()); // No change needed
        }
        
        let _old_plans = self.plans.clone();
        self.plans = new_plans.clone();
        self.updated_at = Utc::now();
        
        // Permission plans changed - could raise an event here if needed
        // self.base.add_event(Box::new(PermissionPlansChangedEvent::new(...)));
        
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
    
    /// Check if user has admin privileges (requires explicit admin wildcard or dashboard perm)
    pub fn is_admin(&self) -> bool {
        let strs: Vec<String> = self.permissions.iter().map(|p| p.as_str().to_string()).collect();
        crate::core::permissions::is_admin(&strs)
    }
    
    /// Check if user has premium access
    pub fn is_premium(&self) -> bool {
        self.plans.contains("Premium Access Plan") ||
        self.plans.contains("Premium Access Group") ||
        self.plans.contains("Professional Access Plan") ||
        self.plans.contains("Professional Access Group") ||
        self.plans.contains("Enterprise Access Plan") ||
        self.plans.contains("Enterprise Access Group")
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

    /// Take ownership of uncommitted events (for CQRS TransactionalOutbox)
    /// This moves events out of the aggregate, clearing the internal list
    pub fn take_events(&mut self) -> Vec<Box<dyn DomainEvent>> {
        self.base.take_events()
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
    use crate::domain::wallet_management::value_objects::WalletAddress;
    
    #[test]
    fn test_create_wallet_user() {
        let wallet_address = WalletAddress::new("0x742d35Cc67C9c24d4D3A6A5c9B1c4D6F8F8c8B8d").unwrap();
        let initial_plans = HashSet::from(["Basic Access Plan".to_string()]);
        let wallet = WalletUser::create(wallet_address.clone(), initial_plans.clone()).unwrap();

        assert_eq!(wallet.wallet_address(), &wallet_address);
        assert!(wallet.is_active());
        assert_eq!(wallet.plans(), &initial_plans);
        assert!(wallet.permissions().is_empty());
    }
    
    #[test]
    fn test_grant_revoke_permission() {
        let wallet_address = WalletAddress::new("0x742d35Cc67C9c24d4D3A6A5c9B1c4D6F8F8c8B8d").unwrap();
        let initial_plans = HashSet::from(["Basic Access Plan".to_string()]);
        let mut user = WalletUser::create(wallet_address, initial_plans).unwrap();
        
        let permission = Permission::new("epsx:read").unwrap();
        user.grant_permission(permission.clone()).unwrap();
        
        assert!(user.has_permission(&permission));
        
        user.revoke_permission(&permission).unwrap();
        assert!(!user.has_permission(&permission));
    }
    
    #[test]
    fn test_permission_plan_update() {
        let wallet_address = WalletAddress::new("0x742d35Cc67C9c24d4D3A6A5c9B1c4D6F8F8c8B8d").unwrap();
        let initial_plans = HashSet::from(["Basic Access Plan".to_string()]);
        let mut user = WalletUser::create(wallet_address, initial_plans).unwrap();
        
        assert!(!user.is_premium());
        
        let premium_plans = HashSet::from(["Premium Access Plan".to_string()]);
        user.update_plans(premium_plans.clone()).unwrap();
        assert!(user.is_premium());
        assert_eq!(user.plans(), &premium_plans);
    }
    
    #[test]
    fn test_deactivate_user_clears_permissions() {
        let wallet_address = WalletAddress::new("0x742d35Cc67C9c24d4D3A6A5c9B1c4D6F8F8c8B8d").unwrap();
        let initial_plans = HashSet::from(["Basic Access Plan".to_string()]);
        let mut user = WalletUser::create(wallet_address, initial_plans).unwrap();
        
        let permission = Permission::new("epsx:read").unwrap();
        user.grant_permission(permission.clone()).unwrap();
        assert!(user.has_permission(&permission));
        
        user.deactivate("Test deactivation".to_string()).unwrap();
        assert!(!user.is_active());
        assert!(user.permissions().is_empty());
    }
}