use async_trait::async_trait;
use std::collections::HashMap;
use chrono::{DateTime, Utc, NaiveDate};

use crate::core::errors::AppResult;
use crate::domain::user_management::{
    aggregates::{WalletUser},
    value_objects::{WalletAddress, Permission, PermissionType}
};

/// Repository port for WalletUser aggregate persistence
/// This interface defines the contract for Web3 wallet-based user data access
#[async_trait]
pub trait WalletUserRepositoryPort: Send + Sync {
    /// Find a user by their wallet address (primary key)
    async fn find_by_wallet(&self, wallet_address: &WalletAddress) -> AppResult<Option<WalletUser>>;
    
    /// Find users by multiple wallet addresses (batch lookup)
    async fn find_by_wallets(&self, wallet_addresses: &[WalletAddress]) -> AppResult<Vec<WalletUser>>;
    
    /// Save a wallet user (create or update)
    async fn save(&self, user: &WalletUser) -> AppResult<()>;
    
    /// Delete a wallet user
    async fn delete(&self, wallet_address: &WalletAddress) -> AppResult<()>;
    
    /// Find users with specific permissions
    async fn find_by_permission(&self, permission: &Permission) -> AppResult<Vec<WalletUser>>;
    
    /// Find users by permission type (manual, NFT, token, DAO)
    async fn find_by_permission_type(&self, permission_type: &PermissionType) -> AppResult<Vec<WalletUser>>;
    
    /// Find users by permission group  
    async fn find_by_permission_group(&self, permission_group: &str) -> AppResult<Vec<WalletUser>>;
    
    /// Find users by multiple criteria with pagination
    async fn find_by_criteria(
        &self,
        criteria: &WalletUserSearchCriteria,
        limit: u32,
        offset: u32
    ) -> AppResult<WalletUserSearchResult>;
    
    /// Count users matching criteria
    async fn count_by_criteria(&self, criteria: &WalletUserSearchCriteria) -> AppResult<u64>;
    
    /// Find users eligible for automatic Web3 permission assignment
    async fn find_eligible_for_web3_permissions(&self, chain_id: u64) -> AppResult<Vec<WalletUser>>;
    
    /// Batch operations for efficiency
    async fn save_batch(&self, users: &[WalletUser]) -> AppResult<()>;
    
    /// Health check for the repository
    async fn health_check(&self) -> AppResult<()>;
    
    /// Clean up expired permissions across all users
    async fn cleanup_expired_permissions(&self) -> AppResult<u32>;
    
    /// Web3-specific methods
    
    /// Find users who own specific NFTs
    async fn find_by_nft_ownership(
        &self,
        contract_address: &str,
        token_ids: Option<&[u64]>,
        chain_id: u64
    ) -> AppResult<Vec<WalletUser>>;
    
    /// Find users who hold minimum token balance
    async fn find_by_token_balance(
        &self,
        contract_address: &str,
        min_balance: &str,
        chain_id: u64
    ) -> AppResult<Vec<WalletUser>>;
    
    /// Find users by DAO membership
    async fn find_by_dao_membership(
        &self,
        dao_contract: &str,
        min_voting_power: &str,
        chain_id: u64
    ) -> AppResult<Vec<WalletUser>>;
    
    /// Validate Web3 permissions against blockchain state
    async fn validate_web3_permissions(
        &self,
        wallet_address: &WalletAddress,
        permissions: &[Permission]
    ) -> AppResult<Vec<bool>>;
    
    /// Cache blockchain validation results
    async fn cache_web3_validation(
        &self,
        wallet_address: &WalletAddress,
        permission: &Permission,
        is_valid: bool,
        cache_duration_seconds: u64
    ) -> AppResult<()>;
}

/// Search criteria for finding wallet users
#[derive(Debug, Clone, Default)]
pub struct WalletUserSearchCriteria {
    /// Search by wallet address pattern (supports partial matches)
    pub wallet_pattern: Option<String>,
    
    /// Filter by active status
    pub is_active: Option<bool>,
    
    /// Filter by permission group
    pub permission_group: Option<String>,
    
    /// Filter by users who have specific permissions
    pub has_permissions: Vec<Permission>,
    
    /// Filter by permission type (manual, NFT, token, DAO)
    pub permission_type: Option<PermissionType>,
    
    /// Filter by blockchain chain ID
    pub chain_id: Option<u64>,
    
    /// Filter by users created after this date
    pub created_after: Option<DateTime<Utc>>,
    
    /// Filter by users created before this date
    pub created_before: Option<DateTime<Utc>>,
    
    /// Filter by users who authenticated after this date
    pub last_auth_after: Option<DateTime<Utc>>,
    
    /// Filter by users who authenticated before this date
    pub last_auth_before: Option<DateTime<Utc>>,
    
    /// Filter by contract interaction (for NFT/token-gated users)
    pub interacted_with_contract: Option<String>,
    
    /// Custom filters for extensibility
    pub custom_filters: HashMap<String, String>,
}

/// Result of a wallet user search operation
#[derive(Debug, Clone)]
pub struct WalletUserSearchResult {
    /// The wallet users that matched the search criteria
    pub users: Vec<WalletUser>,
    
    /// Total count of users that match (for pagination)
    pub total_count: u64,
    
    /// The offset used in this search
    pub offset: u32,
    
    /// The limit used in this search
    pub limit: u32,
    
    /// Whether there are more results available
    pub has_more: bool,
    
    /// Additional Web3 metadata from the search
    pub web3_metadata: HashMap<String, String>,
}

impl WalletUserSearchResult {
    pub fn new(users: Vec<WalletUser>, total_count: u64, offset: u32, limit: u32) -> Self {
        let has_more = (offset + limit) < total_count as u32;
        
        Self {
            users,
            total_count,
            offset,
            limit,
            has_more,
            web3_metadata: HashMap::new(),
        }
    }
    
    pub fn with_metadata(mut self, key: String, value: String) -> Self {
        self.web3_metadata.insert(key, value);
        self
    }
}

/// Aggregated statistics about wallet users
#[derive(Debug, Clone)]
pub struct WalletUserStatistics {
    pub total_users: u64,
    pub active_users: u64,
    pub users_by_permission_group: HashMap<String, u64>,
    pub users_by_chain: HashMap<u64, u64>,
    pub manual_permissions: u64,
    pub nft_gated_permissions: u64,
    pub token_gated_permissions: u64,
    pub dao_governance_permissions: u64,
    pub recent_authentications_24h: u64,
    pub new_wallets_24h: u64,
}

/// Web3-specific analytics for wallet users
#[derive(Debug, Clone)]
pub struct Web3Analytics {
    /// Top NFT contracts by user count
    pub top_nft_contracts: Vec<(String, u64)>,
    
    /// Top token contracts by user count  
    pub top_token_contracts: Vec<(String, u64)>,
    
    /// Top DAO contracts by user count
    pub top_dao_contracts: Vec<(String, u64)>,
    
    /// Chain distribution
    pub chain_distribution: HashMap<u64, u64>,
    
    /// Permission type distribution
    pub permission_type_distribution: HashMap<String, u64>,
}

/// Extended port with Web3 analytics capabilities
#[async_trait]
pub trait WalletUserAnalyticsPort: Send + Sync {
    /// Get wallet user statistics
    async fn get_statistics(&self) -> AppResult<WalletUserStatistics>;
    
    /// Get Web3-specific analytics
    async fn get_web3_analytics(&self) -> AppResult<Web3Analytics>;
    
    /// Get permission distribution across users
    async fn get_permission_distribution(&self) -> AppResult<HashMap<String, u64>>;
    
    /// Get user activity patterns by chain
    async fn get_activity_patterns_by_chain(
        &self,
        chain_id: u64,
        days: u32
    ) -> AppResult<Vec<(NaiveDate, u64)>>;
    
    /// Find inactive users (haven't authenticated for specified days)
    async fn find_inactive_users(&self, days: u32) -> AppResult<Vec<WalletUser>>;
    
    /// Get permission group progression analytics
    async fn get_group_progression(&self) -> AppResult<HashMap<String, Vec<String>>>;
    
    /// Get Web3 permission validation success rates
    async fn get_validation_success_rates(&self) -> AppResult<HashMap<String, f64>>;
    
    /// Analyze cross-chain user behavior
    async fn get_cross_chain_analysis(&self) -> AppResult<HashMap<String, u64>>;
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn wallet_user_search_result_has_more_calculation() {
        let users = vec![];
        let result = WalletUserSearchResult::new(users, 100, 0, 10);
        assert!(result.has_more);
        
        let result = WalletUserSearchResult::new(vec![], 5, 0, 10);
        assert!(!result.has_more);
        
        let result = WalletUserSearchResult::new(vec![], 100, 95, 10);
        assert!(!result.has_more);
    }
    
    #[test]
    fn wallet_user_search_result_with_metadata() {
        let result = WalletUserSearchResult::new(vec![], 10, 0, 10)
            .with_metadata("chain_id".to_string(), "1".to_string())
            .with_metadata("contract_type".to_string(), "ERC721".to_string());
        
        assert_eq!(result.web3_metadata.get("chain_id"), Some(&"1".to_string()));
        assert_eq!(result.web3_metadata.get("contract_type"), Some(&"ERC721".to_string()));
    }
    
    #[test]
    fn wallet_user_search_criteria_default() {
        let criteria = WalletUserSearchCriteria::default();
        assert!(criteria.wallet_pattern.is_none());
        assert!(criteria.is_active.is_none());
        assert!(criteria.permission_group.is_none());
        assert!(criteria.has_permissions.is_empty());
        assert!(criteria.custom_filters.is_empty());
    }
}