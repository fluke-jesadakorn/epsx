// Repository port interfaces for data persistence

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Serialize, Deserialize};

use crate::dom::entities::{User, Session, Stock};
use crate::dom::entities::audit::{AuditLogEntry, AuditLogId, AuditQuery, AuditStatistics, AuditError};
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
pub trait UserRepository: Send + Sync {
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
pub trait SessionRepository: Send + Sync {
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
pub trait StockRepository: Send + Sync {
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
pub trait LevelHistoryRepository: Send + Sync {
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
pub trait AuditRepository: Send + Sync {
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
pub trait ModuleRepository: Send + Sync {
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
pub trait UsageRepository: Send + Sync {
    async fn log_usage(&self, usage_log: ModuleUsageLog) -> Result<(), DomainError>;
    async fn get_usage_stats(&self, _user_id: &UserId, module_name: &str) -> Result<HashMap<String, i32>, DomainError>;
    async fn get_current_usage(&self, _user_id: &UserId, module_name: &str, quota_type: &str) -> Result<i32, DomainError>;
}


#[async_trait]
#[cfg_attr(test, automock)]
pub trait NotificationRepository: Send + Sync {
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

    // NEW: Missing methods identified from notification service analysis
    
    /// Get a notification by ID (for specific user)
    async fn get_by_id(&self, notification_id: &str, user_id: &UserId) -> Result<Option<DomainNotification>, NotificationError>;
    
    /// Mark all notifications as read for a user
    async fn mark_all_as_read(&self, user_id: &UserId) -> Result<u64, NotificationError>;
    
    /// Update delivery status of a notification
    async fn update_delivery_status(&self, notification_id: &str, status: &str) -> Result<(), NotificationError>;
    
    /// Delete a notification
    async fn delete(&self, notification_id: &str, user_id: &UserId) -> Result<bool, NotificationError>;
    
    /// Get user notification preferences
    async fn get_user_preferences(&self, user_id: &UserId) -> Result<Option<NotificationPreferences>, NotificationError>;
    
    /// Update user notification preferences
    async fn upsert_user_preferences(&self, user_id: &UserId, preferences: &NotificationPreferences) -> Result<(), NotificationError>;
    
    /// Get pending notifications for processing
    async fn get_pending_notifications(&self, limit: u32) -> Result<Vec<DomainNotification>, NotificationError>;
    
    /// Cleanup expired notifications
    async fn cleanup_expired(&self, before: DateTime<Utc>) -> Result<u64, NotificationError>;
    
    /// Get critical notifications count for user
    async fn count_critical_notifications(&self, user_id: &UserId) -> Result<u64, NotificationError>;
    
    /// Get today's notifications count for user  
    async fn count_today_notifications(&self, user_id: &UserId) -> Result<u64, NotificationError>;
    
    /// Get last notification timestamp for user
    async fn get_last_notification_time(&self, user_id: &UserId) -> Result<Option<DateTime<Utc>>, NotificationError>;
}

/// User notification preferences
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationPreferences {
    pub email_enabled: bool,
    pub push_enabled: bool,
    pub feature_expiration: bool,
    pub security_alerts: bool,
    pub account_updates: bool,
    pub marketing: bool,
}


// Supporting types


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