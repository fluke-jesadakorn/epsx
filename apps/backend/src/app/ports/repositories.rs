// Repository port interfaces for data persistence

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;

use crate::dom::entities::{User, Session, Payment, Stock};
use crate::dom::entities::iam::{IamRole, IamPolicy, IamGroup, UserPermissionOverride, RoleId, PolicyId, GroupId, IamError};
use crate::dom::entities::audit::{AuditLogEntry, AuditLogId, AuditQuery, AuditStatistics, AuditError};
use crate::dom::entities::permission_profile::{PermissionProfile, PermissionProfileId, PermissionProfileQuery, ApplyPermissionProfileRequest, ApplyPermissionProfileResult, PermissionProfileError};
use crate::dom::entities::module::{SubModule, UserSubModuleAssignment, ApiKey, ModuleUsageLog};
use crate::dom::values::{UserId, SessId, PayId, Symbol, Email, Role, PayStatus, Market};
use crate::dom::error::DomainError;
use crate::app::dtos::LevelChangeRecord;
use crate::web::middleware::module_auth_middleware::{UserModuleAccess, ApiKeyAccess};
use std::collections::HashMap;
use uuid::Uuid;

#[cfg(test)]
use mockall::{automock, predicate::*};

#[async_trait]
#[cfg_attr(test, automock)]
pub trait UserRepo: Send + Sync {
    async fn get(&self, id: &UserId) -> Result<Option<User>, RepoError>;
    async fn save(&self, user: &User) -> Result<(), RepoError>;
    async fn delete(&self, id: &UserId) -> Result<(), RepoError>;
    async fn find_by_email(&self, email: &Email) -> Result<Option<User>, RepoError>;
    async fn find_by_firebase_uid(&self, firebase_uid: &str) -> Result<Option<User>, RepoError>;
    async fn find_by_role(&self, role: &Role) -> Result<Vec<User>, RepoError>;
    async fn list(&self, offset: u32, limit: u32) -> Result<Vec<User>, RepoError>;
    async fn count(&self) -> Result<u64, RepoError>;
    
    // Batch operations for migration
    async fn save_batch(&self, users: &[User]) -> Result<(), RepoError>;
    async fn find_all(&self) -> Result<Vec<User>, RepoError>;
    
    // For compatibility with existing code
    async fn find_by_id(&self, id: &UserId) -> Result<User, RepoError>;
    
    // Job system requirements
    async fn find_users_for_auto_assignment(&self) -> Result<Vec<User>, RepoError>;
    async fn count_total_users(&self) -> Result<i64, RepoError>;
    async fn is_user_active_since(&self, user_id: &UserId, since: DateTime<Utc>) -> Result<bool, RepoError>;
    async fn has_good_payment_history(&self, user_id: &UserId, days: i64) -> Result<bool, RepoError>;
    async fn health_check(&self) -> Result<(), RepoError>;
}

#[async_trait]
#[cfg_attr(test, automock)]
pub trait IamRepo: Send + Sync {
    // Role operations
    async fn create_role(&self, role: IamRole) -> Result<IamRole, IamError>;
    async fn get_role(&self, id: &RoleId) -> Result<IamRole, IamError>;
    async fn update_role(&self, role: IamRole) -> Result<IamRole, IamError>;
    async fn delete_role(&self, id: &RoleId) -> Result<(), IamError>;
    async fn list_roles(&self) -> Result<Vec<IamRole>, IamError>;
    
    // Policy operations
    async fn create_policy(&self, policy: IamPolicy) -> Result<IamPolicy, IamError>;
    async fn get_policy(&self, id: &PolicyId) -> Result<IamPolicy, IamError>;
    async fn update_policy(&self, policy: IamPolicy) -> Result<IamPolicy, IamError>;
    async fn delete_policy(&self, id: &PolicyId) -> Result<(), IamError>;
    async fn list_policies(&self) -> Result<Vec<IamPolicy>, IamError>;
    
    // Group operations
    async fn create_group(&self, group: IamGroup) -> Result<IamGroup, IamError>;
    async fn get_group(&self, id: &GroupId) -> Result<IamGroup, IamError>;
    async fn update_group(&self, group: IamGroup) -> Result<IamGroup, IamError>;
    async fn delete_group(&self, id: &GroupId) -> Result<(), IamError>;
    async fn list_groups(&self) -> Result<Vec<IamGroup>, IamError>;
    
    // User-role relationships
    async fn get_user_roles(&self, user_id: &UserId) -> Result<Vec<IamRole>, IamError>;
    async fn assign_role_to_user(&self, user_id: &UserId, role_id: &RoleId) -> Result<(), IamError>;
    async fn remove_role_from_user(&self, user_id: &UserId, role_id: &RoleId) -> Result<(), IamError>;
    
    // User permission overrides
    async fn get_user_overrides(&self, user_id: &UserId) -> Result<UserPermissionOverride, IamError>;
    async fn set_user_overrides(&self, overrides: UserPermissionOverride) -> Result<(), IamError>;
    async fn delete_user_overrides(&self, user_id: &UserId) -> Result<(), IamError>;
}

#[async_trait]
#[cfg_attr(test, automock)]
pub trait SessRepo: Send + Sync {
    async fn get(&self, id: &SessId) -> Result<Option<Session>, RepoError>;
    async fn save(&self, session: &Session) -> Result<(), RepoError>;
    async fn delete(&self, id: &SessId) -> Result<(), RepoError>;
    async fn find_by_user(&self, uid: &UserId) -> Result<Vec<Session>, RepoError>;
    async fn cleanup_expired(&self) -> Result<u64, RepoError>; // Returns count of cleaned up sessions
    async fn deactivate_user_sessions(&self, uid: &UserId) -> Result<(), RepoError>;
    
    // For compatibility with existing code
    async fn find_by_id(&self, id: &SessId) -> Result<Session, RepoError>;
}

#[async_trait]
#[cfg_attr(test, automock)]
pub trait PayRepo: Send + Sync {
    async fn get(&self, id: &PayId) -> Result<Option<Payment>, RepoError>;
    async fn save(&self, payment: &Payment) -> Result<(), RepoError>;
    async fn find_by_user(&self, uid: &UserId) -> Result<Vec<Payment>, RepoError>;
    async fn find_by_status(&self, status: &PayStatus) -> Result<Vec<Payment>, RepoError>;
    async fn find_by_date_range(&self, start: DateTime<Utc>, end: DateTime<Utc>) -> Result<Vec<Payment>, RepoError>;
    
    // Analytics queries
    async fn total_revenue(&self, start: DateTime<Utc>, end: DateTime<Utc>) -> Result<Decimal, RepoError>;
    async fn payment_stats(&self) -> Result<PaymentStats, RepoError>;
}

#[async_trait]
#[cfg_attr(test, automock)]
pub trait StockRepo: Send + Sync {
    async fn get(&self, symbol: &Symbol) -> Result<Option<Stock>, RepoError>;
    async fn save(&self, stock: &Stock) -> Result<(), RepoError>;
    async fn list_by_market(&self, market: &Market) -> Result<Vec<Stock>, RepoError>;
    async fn find_top_movers(&self, limit: u32) -> Result<Vec<Stock>, RepoError>;
    async fn find_by_symbols(&self, symbols: &[Symbol]) -> Result<Vec<Stock>, RepoError>;
    
    // Real-time data
    async fn save_price_history(&self, symbol: &Symbol, prices: &[PricePoint]) -> Result<(), RepoError>;
    async fn get_price_history(&self, symbol: &Symbol, duration: chrono::Duration) -> Result<Vec<PricePoint>, RepoError>;
    
    // Batch operations
    async fn save_batch(&self, stocks: &[Stock]) -> Result<(), RepoError>;
}

#[async_trait]
#[cfg_attr(test, automock)]
pub trait LevelHistoryRepo: Send + Sync {
    async fn save_level_change(&self, record: &LevelChangeRecord) -> Result<(), RepoError>;
    async fn get_user_level_history(
        &self, 
        user_id: &UserId, 
        limit: u32, 
        offset: u32
    ) -> Result<Vec<LevelChangeRecord>, RepoError>;
    async fn count_user_level_changes(&self, user_id: &UserId) -> Result<u64, RepoError>;
    async fn get_recent_level_changes(&self, limit: u32) -> Result<Vec<LevelChangeRecord>, RepoError>;
}

#[async_trait]
#[cfg_attr(test, automock)]
pub trait AuditRepo: Send + Sync {
    /// Store a new audit log entry
    async fn store(&self, entry: &AuditLogEntry) -> Result<(), AuditError>;
    
    /// Retrieve a specific audit log entry by ID
    async fn get(&self, id: &AuditLogId) -> Result<Option<AuditLogEntry>, AuditError>;
    
    /// Search audit logs with filters and pagination
    async fn search(&self, query: &AuditQuery) -> Result<Vec<AuditLogEntry>, AuditError>;
    
    /// Count audit logs matching query filters
    async fn count(&self, query: &AuditQuery) -> Result<u64, AuditError>;
    
    /// Get audit statistics for a time range
    async fn statistics(&self, from: DateTime<Utc>, to: DateTime<Utc>) -> Result<AuditStatistics, AuditError>;
    
    /// Delete audit logs older than specified date (for retention policy)
    async fn cleanup_old_entries(&self, older_than: DateTime<Utc>) -> Result<u64, AuditError>;
    
    /// Batch store multiple audit entries (for performance)
    async fn store_batch(&self, entries: &[AuditLogEntry]) -> Result<(), AuditError>;
    
    /// Export audit logs to external format (compliance)
    async fn export(&self, query: &AuditQuery, format: ExportFormat) -> Result<Vec<u8>, AuditError>;
    
    // Job system requirements
    async fn cleanup_old_logs(&self, days: i64) -> Result<i64, AuditError>;
    async fn log_permission_assignment(&self, user_id: &UserId, profile_id: &PermissionProfileId, assigned_by: &str, reason: &str) -> Result<(), AuditError>;
    async fn log_permission_revocation(&self, user_id: &UserId, profile_id: &PermissionProfileId, revoked_by: &str, reason: &str) -> Result<(), AuditError>;
    async fn log_system_event(&self, event_type: &str, details: &str) -> Result<(), AuditError>;
    async fn log_notification_sent(&self, recipient: &str, subject: &str, notification_type: &str, message_id: Option<&str>) -> Result<(), AuditError>;
    async fn log_notification_failed(&self, recipient: &str, subject: &str, notification_type: &str, error: &str) -> Result<(), AuditError>;
    async fn health_check(&self) -> Result<(), AuditError>;
}

/// Export formats for audit logs
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExportFormat {
    Json,
    Csv,
    Xml,
}

#[async_trait]
#[cfg_attr(test, automock)]
pub trait PermissionProfileRepo: Send + Sync {
    /// Create a new permission profile
    async fn create(&self, profile: PermissionProfile) -> Result<PermissionProfile, PermissionProfileError>;
    
    /// Get a permission profile by ID
    async fn get(&self, id: &PermissionProfileId) -> Result<Option<PermissionProfile>, PermissionProfileError>;
    
    /// Update an existing permission profile
    async fn update(&self, profile: PermissionProfile) -> Result<PermissionProfile, PermissionProfileError>;
    
    /// Delete a permission profile (soft delete - mark as inactive)
    async fn delete(&self, id: &PermissionProfileId) -> Result<(), PermissionProfileError>;
    
    /// Search permission profiles with filters and pagination
    async fn search(&self, query: &PermissionProfileQuery) -> Result<Vec<PermissionProfile>, PermissionProfileError>;
    
    /// Count permission profiles matching query
    async fn count(&self, query: &PermissionProfileQuery) -> Result<u64, PermissionProfileError>;
    
    /// Get all permission profiles for a specific category
    async fn get_by_category(&self, category: &crate::dom::entities::permission_profile::PermissionProfileCategory) -> Result<Vec<PermissionProfile>, PermissionProfileError>;
    
    /// Apply permission profile to users (returns application results)
    async fn apply_permission_profile(&self, request: &ApplyPermissionProfileRequest) -> Result<ApplyPermissionProfileResult, PermissionProfileError>;
    
    /// Get permission profile application history
    async fn get_application_history(&self, profile_id: &PermissionProfileId, limit: u32) -> Result<Vec<ApplyPermissionProfileResult>, PermissionProfileError>;
    
    /// Check if permission profile can be applied to user (validates prerequisites)
    async fn can_apply_to_user(&self, profile_id: &PermissionProfileId, user_id: &UserId) -> Result<bool, PermissionProfileError>;
    
    /// Get active assignment count for a permission profile
    async fn get_assignment_count(&self, profile_id: &PermissionProfileId) -> Result<u32, PermissionProfileError>;
    
    /// Initialize default permission profiles (call once on startup)
    async fn initialize_defaults(&self, admin_user_id: &UserId) -> Result<Vec<PermissionProfile>, PermissionProfileError>;
    
    // Job system requirements
    async fn find_assignments_expiring_before(&self, cutoff_date: DateTime<Utc>) -> Result<Vec<PermissionAssignment>, PermissionProfileError>;
    async fn revoke_assignment(&self, user_id: &UserId, profile_id: &PermissionProfileId) -> Result<(), PermissionProfileError>;
    async fn cleanup_expired_assignments(&self) -> Result<i64, PermissionProfileError>;
    async fn count_active_profiles(&self) -> Result<i64, PermissionProfileError>;
    async fn count_total_assignments(&self) -> Result<i64, PermissionProfileError>;
    async fn find_user_assignments_with_expiration(&self, user_id: &UserId) -> Result<Vec<PermissionAssignment>, PermissionProfileError>;
    async fn extend_assignment_expiration(&self, user_id: &UserId, profile_id: &PermissionProfileId, new_expiration: DateTime<Utc>) -> Result<(), PermissionProfileError>;
    async fn find_by_id(&self, id: &PermissionProfileId) -> Result<Option<PermissionProfile>, PermissionProfileError>;
    async fn health_check(&self) -> Result<(), PermissionProfileError>;
}

#[async_trait]
#[cfg_attr(test, automock)]
pub trait ModuleRepo: Send + Sync {
    // Sub-module management
    async fn create_sub_module(&self, module: &SubModule) -> Result<(), DomainError>;
    async fn update_sub_module(&self, module: &SubModule) -> Result<(), DomainError>;
    async fn delete_sub_module(&self, module_id: &Uuid) -> Result<(), DomainError>;
    async fn get_sub_module(&self, module_id: &Uuid) -> Result<Option<SubModule>, DomainError>;
    async fn get_sub_module_by_name(&self, name: &str) -> Result<Option<SubModule>, DomainError>;
    async fn list_active_modules(&self) -> Result<Vec<SubModule>, DomainError>;

    // User module assignments
    async fn create_assignment(&self, assignment: &UserSubModuleAssignment) -> Result<(), DomainError>;
    async fn update_assignment(&self, assignment: &UserSubModuleAssignment) -> Result<(), DomainError>;
    async fn delete_assignment(&self, assignment_id: &Uuid) -> Result<(), DomainError>;
    async fn get_assignment(&self, assignment_id: &Uuid) -> Result<Option<UserSubModuleAssignment>, DomainError>;
    async fn get_user_module_assignments(&self, user_id: &UserId) -> Result<Vec<UserModuleAccess>, DomainError>;
    async fn has_user_module_access(&self, user_id: &UserId, module_name: &str) -> Result<bool, DomainError>;
    async fn get_user_access_level(&self, user_id: &UserId, module_name: &str) -> Result<Option<String>, DomainError>;

    // API key management
    async fn create_api_key(&self, api_key: &ApiKey) -> Result<(), DomainError>;
    async fn update_api_key(&self, api_key: &ApiKey) -> Result<(), DomainError>;
    async fn delete_api_key(&self, key_id: &Uuid) -> Result<(), DomainError>;
    async fn get_api_key(&self, key_id: &Uuid) -> Result<Option<ApiKey>, DomainError>;
    async fn get_api_key_by_hash(&self, key_hash: &str) -> Result<Option<ApiKey>, DomainError>;
    async fn get_api_key_access(&self, key_hash: &str) -> Result<Option<ApiKeyAccess>, DomainError>;

    // Usage logging
    async fn log_usage(&self, usage_log: &ModuleUsageLog) -> Result<(), DomainError>;
    async fn get_current_usage(&self, user_id: &UserId, module_name: &str, quota_type: &str) -> Result<i32, DomainError>;
    async fn get_quota_limits(&self, user_id: &UserId, module_name: &str) -> Result<HashMap<String, i32>, DomainError>;
    async fn check_quota_availability(&self, user_id: &UserId, module_name: &str, quota_type: &str, amount: i32) -> Result<bool, DomainError>;
}

#[async_trait]
#[cfg_attr(test, automock)]
pub trait UsageRepo: Send + Sync {
    async fn log_usage(&self, usage_log: ModuleUsageLog) -> Result<(), DomainError>;
    async fn get_usage_stats(&self, user_id: &UserId, module_name: &str) -> Result<HashMap<String, i32>, DomainError>;
    async fn get_current_usage(&self, user_id: &UserId, module_name: &str, quota_type: &str) -> Result<i32, DomainError>;
}

// Supporting types
#[derive(Debug, Clone)]
pub struct PermissionAssignment {
    pub user_id: UserId,
    pub permission_profile_id: PermissionProfileId,
    pub assigned_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
    pub assigned_by: String,
    pub reason: String,
}

#[derive(Debug, Clone)]
pub struct PaymentStats {
    pub total_payments: u64,
    pub total_revenue: Decimal,
    pub pending_payments: u64,
    pub completed_payments: u64,
    pub failed_payments: u64,
}

#[derive(Debug, Clone)]
pub struct PricePoint {
    pub price: Decimal,
    pub volume: u64,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, thiserror::Error)]
pub enum RepoError {
    #[error("Database connection error: {0}")]
    ConnectionError(String),
    
    #[error("Query execution error: {0}")]
    QueryError(String),
    
    #[error("Serialization error: {0}")]
    SerializationError(String),
    
    #[error("Data not found")]
    NotFound,
    
    #[error("Data already exists")]
    AlreadyExists,
    
    #[error("Invalid data: {0}")]
    InvalidData(String),
    
    #[error("Internal error: {0}")]
    Internal(String),
}


impl From<serde_json::Error> for RepoError {
    fn from(err: serde_json::Error) -> Self {
        RepoError::SerializationError(err.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn should_create_payment_stats() {
        let stats = PaymentStats {
            total_payments: 100,
            total_revenue: rust_decimal_macros::dec!(5000.0),
            pending_payments: 10,
            completed_payments: 85,
            failed_payments: 5,
        };
        
        assert_eq!(stats.total_payments, 100);
        assert_eq!(stats.total_revenue, rust_decimal_macros::dec!(5000.0));
    }
    
    #[test]
    fn should_create_price_point() {
        let point = PricePoint {
            price: rust_decimal_macros::dec!(150.50),
            volume: 1000000,
            timestamp: Utc::now(),
        };
        
        assert_eq!(point.price, rust_decimal_macros::dec!(150.50));
        assert_eq!(point.volume, 1000000);
    }
}