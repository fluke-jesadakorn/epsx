use chrono::{DateTime, Utc};
use serde::{Serialize, Deserialize};

use crate::application::shared::{Query, ApplicationResult, ValidationUtils};
use crate::domain::shared_kernel::value_objects::UserId;
// use crate::domain::wallet_management::value_objects::Email; // REMOVED - Web3-first uses wallet addresses

/// Query to get a user by ID
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetWalletQuery {
    /// User ID to retrieve
    pub wallet_address: String,
    
    /// Whether to include permissions in the response
    pub include_permissions: bool,
    
    /// Whether to include session information
    pub include_sessions: bool,
    
    /// Query metadata
    pub requested_by: Option<String>,
    pub correlation_id: Option<String>,
}

/// User information response (Web3-first: wallet-based)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetWalletResponse {
    /// Wallet address (user identifier)
    pub wallet_address: UserId,

    /// Whether the user is active
    pub is_active: bool,

    /// When the user was created
    pub created_at: DateTime<Utc>,

    /// When the user was last updated
    pub updated_at: DateTime<Utc>,

    /// When the user last logged in
    pub last_login_at: Option<DateTime<Utc>>,

    /// User permissions (if requested)
    pub permissions: Option<Vec<String>>,

    /// Active session count (if requested)
    pub active_session_count: Option<u32>,

    /// Wallet statistics
    pub stats: WalletStats,
}

/// Wallet statistics for the response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalletStats {
    /// Total number of permissions
    pub total_permissions: u32,
    
    /// Number of active permissions (non-expired)
    pub active_permissions: u32,
    
    /// Number of expired permissions
    pub expired_permissions: u32,
    
    /// Account age in days
    pub account_age_days: i64,
}

impl Query for GetWalletQuery {
    type Response = GetWalletResponse;
    
    fn validate(&self) -> ApplicationResult<()> {
        // Validate wallet address is not empty
        if let Some(error) = ValidationUtils::required("wallet_address", &self.wallet_address) {
            return Err(crate::application::ApplicationError::validation(
                &error.field,
                &error.message
            ));
        }
        
        Ok(())
    }
}

impl GetWalletQuery {
    /// Create a new GetWalletQuery
    pub fn new(wallet_address: String) -> Self {
        Self {
            wallet_address,
            include_permissions: false,
            include_sessions: false,
            requested_by: None,
            correlation_id: None,
        }
    }
    
    /// Include permissions in the response
    pub fn with_permissions(mut self) -> Self {
        self.include_permissions = true;
        self
    }
    
    /// Include session information in the response
    pub fn with_sessions(mut self) -> Self {
        self.include_sessions = true;
        self
    }
    
    /// Set who requested this query (for audit)
    pub fn requested_by(mut self, wallet_address: String) -> Self {
        self.requested_by = Some(wallet_address);
        self
    }
    
    /// Set correlation ID for tracing
    pub fn with_correlation_id(mut self, correlation_id: String) -> Self {
        self.correlation_id = Some(correlation_id);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn get_user_query_validation_success() {
        let query = GetWalletQuery::new("user_123".to_string());
        assert!(query.validate().is_ok());
    }
    
    #[test]
    fn get_user_query_validation_emptyuser_id() {
        let query = GetWalletQuery::new("".to_string());
        assert!(query.validate().is_err());
    }
    
    #[test]
    fn get_user_query_builder_pattern() {
        let query = GetWalletQuery::new("user_123".to_string())
            .with_permissions()
            .with_sessions()
            .requested_by("requester_id".to_string())
            .with_correlation_id("corr_123".to_string());
        
        assert!(query.include_permissions);
        assert!(query.include_sessions);
        assert_eq!(query.requested_by, Some("requester_id".to_string()));
        assert_eq!(query.correlation_id, Some("corr_123".to_string()));
    }
}