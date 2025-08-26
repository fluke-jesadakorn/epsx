// Repository port interfaces for data persistence

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;

use crate::dom::entities::{User, Session, Stock};
use crate::dom::entities::iam::{IamRole, IamPolicy, IamGroup, UserPermissionOverride, RoleId, PolicyId, GroupId, IamError};
use crate::dom::entities::audit::{AuditLogEntry, AuditLogId, AuditQuery, AuditStatistics, AuditError};
use crate::dom::entities::permission_profile::{PermissionProfile, PermissionProfileId, PermissionProfileQuery, ApplyPermissionProfileRequest, ApplyPermissionProfileResult, PermissionProfileError};
use crate::dom::entities::temporary_permission::{TemporaryPermission, TemporaryPermissionStatus};
use crate::dom::ports::notification::{DomainNotification, NotificationError};
use crate::dom::entities::module::{SubModule, UserSubModuleAssignment, ApiKey, ModuleUsageLog};
use crate::dom::values::{UserId, SessId, Symbol, Email, Market};
use crate::dom::error::DomainError;
use crate::app::dtos::LevelChangeRecord;
// use crate::web::middleware::module_auth_middleware::{UserModuleAccess, ApiKeyAccess};
use std::collections::HashMap;
use uuid::Uuid;

// Simple stubs for removed module auth
#[derive(Debug, Clone)]
pub struct UserModuleAccess {
    pub user_id: UserId,
    pub module_name: String,
    pub access_level: String,
}

#[derive(Debug, Clone)]
pub struct ApiKeyAccess {
    pub key_id: Uuid,
    pub module_name: String,
    pub access_level: String,
}

// Search filter structures for advanced user search
#[derive(Debug, Clone)]
pub struct UserSearchFilters {
    pub search: Option<String>,
    pub email: Option<String>,
    pub package_tier: Option<String>,
    pub status: Option<String>,
    pub tier: Option<String>,
    pub last_login_after: Option<DateTime<Utc>>,
    pub last_login_before: Option<DateTime<Utc>>,
    pub created_after: Option<DateTime<Utc>>,
    pub created_before: Option<DateTime<Utc>>,
    pub has_module: Option<String>,
    pub permission_profile: Option<String>,
    pub email_verified: Option<bool>,
    pub two_factor_enabled: Option<bool>,
    pub has_api_keys: Option<bool>,
}

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
    async fn find_by_admin_module(&self, admin_module: &str) -> Result<Vec<User>, RepoError>;
    async fn find_by_package_tier(&self, package_tier: &str) -> Result<Vec<User>, RepoError>;
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
    async fn is_user_active_since(&self, _user_id: &UserId, since: DateTime<Utc>) -> Result<bool, RepoError>;
    async fn has_good_payment_history(&self, _user_id: &UserId, days: i64) -> Result<bool, RepoError>;
    async fn health_check(&self) -> Result<(), RepoError>;
    
    // Advanced search functionality
    async fn search_users(
        &self, 
        filters: &UserSearchFilters, 
        offset: u32, 
        limit: u32, 
        sort_by: &str, 
        sort_order: &str
    ) -> Result<Vec<User>, RepoError>;
    async fn count_search_users(&self, filters: &UserSearchFilters) -> Result<u64, RepoError>;
}

#[async_trait]
#[cfg_attr(test, automock)]
pub trait IamRepo: Send + Sync {
    // Role operations
    async fn create_role(&self, role: IamRole) -> Result<IamRole, IamError>;
    async fn get_role(&self, _id: &RoleId) -> Result<IamRole, IamError>;
    async fn update_role(&self, role: IamRole) -> Result<IamRole, IamError>;
    async fn delete_role(&self, _id: &RoleId) -> Result<(), IamError>;
    async fn list_roles(&self) -> Result<Vec<IamRole>, IamError>;
    
    // Policy operations
    async fn create_policy(&self, policy: IamPolicy) -> Result<IamPolicy, IamError>;
    async fn get_policy(&self, _id: &PolicyId) -> Result<IamPolicy, IamError>;
    async fn update_policy(&self, policy: IamPolicy) -> Result<IamPolicy, IamError>;
    async fn delete_policy(&self, _id: &PolicyId) -> Result<(), IamError>;
    async fn list_policies(&self) -> Result<Vec<IamPolicy>, IamError>;
    
    // Group operations
    async fn create_group(&self, group: IamGroup) -> Result<IamGroup, IamError>;
    async fn get_group(&self, id: &GroupId) -> Result<IamGroup, IamError>;
    async fn update_group(&self, group: IamGroup) -> Result<IamGroup, IamError>;
    async fn delete_group(&self, id: &GroupId) -> Result<(), IamError>;
    async fn list_groups(&self) -> Result<Vec<IamGroup>, IamError>;
    
    // User-role relationships
    async fn get_user_roles(&self, _user_id: &UserId) -> Result<Vec<IamRole>, IamError>;
    async fn assign_role_to_user(&self, _user_id: &UserId, role_id: &RoleId) -> Result<(), IamError>;
    async fn remove_role_from_user(&self, _user_id: &UserId, role_id: &RoleId) -> Result<(), IamError>;
    
    // User permission overrides
    async fn get_user_overrides(&self, _user_id: &UserId) -> Result<UserPermissionOverride, IamError>;
    async fn set_user_overrides(&self, overrides: UserPermissionOverride) -> Result<(), IamError>;
    async fn delete_user_overrides(&self, _user_id: &UserId) -> Result<(), IamError>;
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
pub trait StockRepo: Send + Sync {
    async fn get(&self, symbol: &Symbol) -> Result<Option<Stock>, RepoError>;
    async fn save(&self, stock: &Stock) -> Result<(), RepoError>;
    async fn list_by_market(&self, market: &Market) -> Result<Vec<Stock>, RepoError>;
    async fn find_top_movers(&self, limit: u32) -> Result<Vec<Stock>, RepoError>;
    async fn find_by_symbols(&self, symbols: &[Symbol]) -> Result<Vec<Stock>, RepoError>;
    
    // Real-time data
    async fn save_price_history(&self, symbol: &Symbol, prices: &[PricePoint]) -> Result<(), RepoError>;
    async fn get_price_history(&self, symbol: &Symbol, _duration: chrono::Duration) -> Result<Vec<PricePoint>, RepoError>;
    
    // Batch operations
    async fn save_batch(&self, _stocks: &[Stock]) -> Result<(), RepoError>;
}

#[async_trait]
#[cfg_attr(test, automock)]
pub trait LevelHistoryRepo: Send + Sync {
    async fn save_level_change(&self, record: &LevelChangeRecord) -> Result<(), RepoError>;
    async fn get_user_level_history(
        &self, 
        _user_id: &UserId, 
        limit: u32, 
        offset: u32
    ) -> Result<Vec<LevelChangeRecord>, RepoError>;
    async fn count_user_level_changes(&self, _user_id: &UserId) -> Result<u64, RepoError>;
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
    async fn log_permission_assignment(&self, _user_id: &UserId, _profile_id: &PermissionProfileId, assigned_by: &str, reason: &str) -> Result<(), AuditError>;
    async fn log_permission_revocation(&self, _user_id: &UserId, _profile_id: &PermissionProfileId, revoked_by: &str, reason: &str) -> Result<(), AuditError>;
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
    async fn get(&self, _id: &PermissionProfileId) -> Result<Option<PermissionProfile>, PermissionProfileError>;
    
    /// Update an existing permission profile
    async fn update(&self, profile: PermissionProfile) -> Result<PermissionProfile, PermissionProfileError>;
    
    /// Delete a permission profile (soft delete - mark as inactive)
    async fn delete(&self, _id: &PermissionProfileId) -> Result<(), PermissionProfileError>;
    
    /// Search permission profiles with filters and pagination
    async fn search(&self, query: &PermissionProfileQuery) -> Result<Vec<PermissionProfile>, PermissionProfileError>;
    
    /// Count permission profiles matching query
    async fn count(&self, query: &PermissionProfileQuery) -> Result<u64, PermissionProfileError>;
    
    /// Get all permission profiles for a specific category
    async fn get_by_category(&self, category: &crate::dom::entities::permission_profile::PermissionProfileCategory) -> Result<Vec<PermissionProfile>, PermissionProfileError>;
    
    /// Apply permission profile to users (returns application results)
    async fn apply_permission_profile(&self, request: &ApplyPermissionProfileRequest) -> Result<ApplyPermissionProfileResult, PermissionProfileError>;
    
    /// Get permission profile application history
    async fn get_application_history(&self, _profile_id: &PermissionProfileId, limit: u32) -> Result<Vec<ApplyPermissionProfileResult>, PermissionProfileError>;
    
    /// Check if permission profile can be applied to user (validates prerequisites)
    async fn can_apply_to_user(&self, _profile_id: &PermissionProfileId, _user_id: &UserId) -> Result<bool, PermissionProfileError>;
    
    /// Get active assignment count for a permission profile
    async fn get_assignment_count(&self, _profile_id: &PermissionProfileId) -> Result<u32, PermissionProfileError>;
    
    /// Initialize default permission profiles (call once on startup)
    async fn initialize_defaults(&self, admin_user_id: &UserId) -> Result<Vec<PermissionProfile>, PermissionProfileError>;
    
    // Job system requirements
    async fn find_assignments_expiring_before(&self, _cutoff_date: DateTime<Utc>) -> Result<Vec<PermissionAssignment>, PermissionProfileError>;
    async fn revoke_assignment(&self, _user_id: &UserId, _profile_id: &PermissionProfileId) -> Result<(), PermissionProfileError>;
    async fn cleanup_expired_assignments(&self) -> Result<i64, PermissionProfileError>;
    async fn count_active_profiles(&self) -> Result<i64, PermissionProfileError>;
    async fn count_total_assignments(&self) -> Result<i64, PermissionProfileError>;
    async fn find_user_assignments_with_expiration(&self, _user_id: &UserId) -> Result<Vec<PermissionAssignment>, PermissionProfileError>;
    async fn extend_assignment_expiration(&self, _user_id: &UserId, _profile_id: &PermissionProfileId, _new_expiration: DateTime<Utc>) -> Result<(), PermissionProfileError>;
    async fn find_by_id(&self, _id: &PermissionProfileId) -> Result<Option<PermissionProfile>, PermissionProfileError>;
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
    async fn get_user_module_assignments(&self, _user_id: &UserId) -> Result<Vec<UserModuleAccess>, DomainError>;
    async fn has_user_module_access(&self, _user_id: &UserId, module_name: &str) -> Result<bool, DomainError>;
    async fn get_user_access_level(&self, _user_id: &UserId, module_name: &str) -> Result<Option<String>, DomainError>;

    // API key management
    async fn create_api_key(&self, api_key: &ApiKey) -> Result<(), DomainError>;
    async fn update_api_key(&self, api_key: &ApiKey) -> Result<(), DomainError>;
    async fn delete_api_key(&self, key_id: &Uuid) -> Result<(), DomainError>;
    async fn get_api_key(&self, key_id: &Uuid) -> Result<Option<ApiKey>, DomainError>;
    async fn get_api_key_by_hash(&self, key_hash: &str) -> Result<Option<ApiKey>, DomainError>;
    async fn get_api_key_access(&self, key_hash: &str) -> Result<Option<ApiKeyAccess>, DomainError>;

    // Usage logging
    async fn log_usage(&self, usage_log: &ModuleUsageLog) -> Result<(), DomainError>;
    async fn get_current_usage(&self, _user_id: &UserId, module_name: &str, quota_type: &str) -> Result<i32, DomainError>;
    async fn get_quota_limits(&self, _user_id: &UserId, module_name: &str) -> Result<HashMap<String, i32>, DomainError>;
    async fn check_quota_availability(&self, _user_id: &UserId, module_name: &str, quota_type: &str, amount: i32) -> Result<bool, DomainError>;
}

#[async_trait]
#[cfg_attr(test, automock)]
pub trait UsageRepo: Send + Sync {
    async fn log_usage(&self, usage_log: ModuleUsageLog) -> Result<(), DomainError>;
    async fn get_usage_stats(&self, _user_id: &UserId, module_name: &str) -> Result<HashMap<String, i32>, DomainError>;
    async fn get_current_usage(&self, _user_id: &UserId, module_name: &str, quota_type: &str) -> Result<i32, DomainError>;
}

/// Query parameters for searching temporary permissions
#[derive(Debug, Clone)]
pub struct TemporaryPermissionQuery {
    pub user_id: Option<UserId>,
    pub permission: Option<String>,
    pub resource: Option<String>,
    pub action: Option<String>,
    pub status: Option<TemporaryPermissionStatus>,
    pub active_only: Option<bool>,
    pub expires_before: Option<DateTime<Utc>>,
    pub expires_after: Option<DateTime<Utc>>,
    pub granted_by: Option<UserId>,
    pub limit: Option<i32>,
    pub offset: Option<i32>,
}

impl Default for TemporaryPermissionQuery {
    fn default() -> Self {
        Self {
            user_id: None,
            permission: None,
            resource: None,
            action: None,
            status: None,
            active_only: None,
            expires_before: None,
            expires_after: None,
            granted_by: None,
            limit: Some(100),
            offset: Some(0),
        }
    }
}

#[async_trait]
#[cfg_attr(test, automock)]
pub trait NotificationRepo: Send + Sync {
    /// Send a notification to a user
    async fn send_notification(&self, user_id: &UserId, notification: &DomainNotification) -> Result<(), NotificationError>;
    
    /// Get notifications for a user
    async fn get_user_notifications(&self, user_id: &UserId, offset: u32, limit: u32) -> Result<Vec<DomainNotification>, NotificationError>;
    
    /// Mark notification as read
    async fn mark_as_read(&self, notification_id: &str) -> Result<(), NotificationError>;
    
    /// Get notification count for user
    async fn count_user_notifications(&self, user_id: &UserId) -> Result<u64, NotificationError>;
    
    /// Get unread notification count for user
    async fn count_unread_notifications(&self, user_id: &UserId) -> Result<u64, NotificationError>;
}

#[async_trait]
#[cfg_attr(test, automock)]
pub trait TemporaryPermissionRepo: Send + Sync {
    /// Create a new temporary permission
    async fn create(&self, permission: &TemporaryPermission) -> Result<TemporaryPermission, RepoError>;
    
    /// Get a temporary permission by ID
    async fn find_by_id(&self, id: &Uuid) -> Result<Option<TemporaryPermission>, RepoError>;
    
    /// Search temporary permissions with filters and pagination
    async fn find_by_query(&self, _query: &TemporaryPermissionQuery) -> Result<Vec<TemporaryPermission>, RepoError>;
    
    /// Get active temporary permissions for a user
    async fn find_active_for_user(&self, _user_id: &UserId) -> Result<Vec<TemporaryPermission>, RepoError>;
    
    /// Update an existing temporary permission
    async fn update(&self, permission: &TemporaryPermission) -> Result<TemporaryPermission, RepoError>;
    
    /// Delete a temporary permission
    async fn delete(&self, _id: &Uuid) -> Result<bool, RepoError>;
    
    /// Expire permissions that have passed their expiry time
    async fn expire_permissions(&self, _before: DateTime<Utc>) -> Result<u64, RepoError>;
    
    /// Clean up expired permissions (convenience method)
    async fn cleanup_expired(&self) -> Result<u64, RepoError>;
    
    /// Count temporary permissions matching query
    async fn count_by_query(&self, _query: &TemporaryPermissionQuery) -> Result<i64, RepoError>;
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
    pub is_active: bool,
}

/// Repository trait for permission assignments
#[async_trait]
#[cfg_attr(test, automock)]
pub trait PermissionAssignmentRepo: Send + Sync {
    /// Get all assignments for a user
    async fn get_user_assignments(&self, _user_id: &UserId) -> Result<Vec<PermissionAssignment>, RepoError>;
    
    /// Assign permission profile to user
    async fn assign_permission_profile(
        &self,
        _user_id: &UserId,
        permission_profile_id: &PermissionProfileId,
        assigned_by: &UserId,
        expires_at: Option<DateTime<Utc>>,
        reason: Option<String>,
    ) -> Result<(), RepoError>;
    
    /// Revoke permission assignment
    async fn revoke_assignment(&self, _user_id: &UserId, permission_profile_id: &PermissionProfileId) -> Result<(), RepoError>;
    
    /// Check if user has active assignment for permission profile
    async fn has_active_assignment(&self, _user_id: &UserId, permission_profile_id: &PermissionProfileId) -> Result<bool, RepoError>;
    
    /// Get assignments expiring before a date
    async fn get_assignments_expiring_before(&self, _cutoff_date: DateTime<Utc>) -> Result<Vec<PermissionAssignment>, RepoError>;
    
    /// Clean up expired assignments
    async fn cleanup_expired_assignments(&self) -> Result<i64, RepoError>;
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