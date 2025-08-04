// Repository traits for the module system

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use std::collections::HashMap;
use uuid::Uuid;

use crate::dom::{
    entities::module::{SubModule, UserSubModuleAssignment, ApiKey, ModuleUsageLog},
    error::DomainError,
    values::UserId,
};
use crate::web::middleware::module_auth_middleware::{UserModuleAccess, ApiKeyAccess};

// ========================================
// MODULE REPOSITORY TRAIT
// ========================================

#[async_trait]
pub trait ModuleRepository: Send + Sync {
    // Sub-module management
    async fn create_sub_module(&self, module: &SubModule) -> Result<(), DomainError>;
    async fn update_sub_module(&self, module: &SubModule) -> Result<(), DomainError>;
    async fn delete_sub_module(&self, module_id: &Uuid) -> Result<(), DomainError>;
    async fn get_sub_module(&self, module_id: &Uuid) -> Result<Option<SubModule>, DomainError>;
    async fn get_sub_module_by_name(&self, name: &str) -> Result<Option<SubModule>, DomainError>;
    async fn list_sub_modules(&self, filters: &ModuleFilters) -> Result<Vec<SubModule>, DomainError>;
    async fn list_active_modules(&self) -> Result<Vec<SubModule>, DomainError>;

    // User module assignments
    async fn create_assignment(&self, assignment: &UserSubModuleAssignment) -> Result<(), DomainError>;
    async fn update_assignment(&self, assignment: &UserSubModuleAssignment) -> Result<(), DomainError>;
    async fn delete_assignment(&self, assignment_id: &Uuid) -> Result<(), DomainError>;
    async fn get_assignment(&self, assignment_id: &Uuid) -> Result<Option<UserSubModuleAssignment>, DomainError>;
    async fn get_user_assignments(&self, user_id: &UserId) -> Result<Vec<UserSubModuleAssignment>, DomainError>;
    async fn get_module_assignments(&self, module_id: &Uuid) -> Result<Vec<UserSubModuleAssignment>, DomainError>;
    
    // Enhanced user module access (with module details)
    async fn get_user_module_assignments(&self, user_id: &UserId) -> Result<Vec<UserModuleAccess>, DomainError>;
    async fn has_user_module_access(&self, user_id: &UserId, module_name: &str) -> Result<bool, DomainError>;
    async fn get_user_access_level(&self, user_id: &UserId, module_name: &str) -> Result<Option<String>, DomainError>;

    // Bulk operations
    async fn bulk_assign_modules(&self, assignments: &[UserSubModuleAssignment]) -> Result<Vec<Uuid>, DomainError>;
    async fn bulk_revoke_assignments(&self, assignment_ids: &[Uuid], reason: &str) -> Result<(), DomainError>;
    async fn bulk_update_access_levels(&self, updates: &[(Uuid, String)]) -> Result<(), DomainError>;

    // API key management
    async fn create_api_key(&self, api_key: &ApiKey) -> Result<(), DomainError>;
    async fn update_api_key(&self, api_key: &ApiKey) -> Result<(), DomainError>;
    async fn delete_api_key(&self, key_id: &Uuid) -> Result<(), DomainError>;
    async fn get_api_key(&self, key_id: &Uuid) -> Result<Option<ApiKey>, DomainError>;
    async fn get_api_key_by_hash(&self, key_hash: &str) -> Result<Option<ApiKey>, DomainError>;
    async fn get_api_key_access(&self, key_hash: &str) -> Result<Option<ApiKeyAccess>, DomainError>;
    async fn list_api_keys(&self, filters: &ApiKeyFilters) -> Result<Vec<ApiKey>, DomainError>;

    // Usage logging
    async fn log_usage(&self, usage_log: &ModuleUsageLog) -> Result<(), DomainError>;
    async fn get_usage_logs(&self, filters: &UsageLogFilters) -> Result<Vec<ModuleUsageLog>, DomainError>;
    async fn get_usage_stats(&self, stats_request: &UsageStatsRequest) -> Result<UsageStats, DomainError>;

    // Analytics and reporting
    async fn get_module_usage_summary(&self, module_id: &Uuid, period: &DateRange) -> Result<ModuleUsageSummary, DomainError>;
    async fn get_user_usage_summary(&self, user_id: &UserId, period: &DateRange) -> Result<UserUsageSummary, DomainError>;
    async fn get_api_key_usage_summary(&self, key_id: &Uuid, period: &DateRange) -> Result<ApiKeyUsageSummary, DomainError>;

    // Quota management
    async fn get_current_usage(&self, user_id: &UserId, module_name: &str, quota_type: &str) -> Result<i32, DomainError>;
    async fn get_quota_limits(&self, user_id: &UserId, module_name: &str) -> Result<HashMap<String, i32>, DomainError>;
    async fn check_quota_availability(&self, user_id: &UserId, module_name: &str, quota_type: &str, amount: i32) -> Result<bool, DomainError>;

    // Assignment audit
    async fn log_assignment_change(&self, audit_log: &AssignmentAuditLog) -> Result<(), DomainError>;
    async fn get_assignment_audit_logs(&self, filters: &AuditLogFilters) -> Result<Vec<AssignmentAuditLog>, DomainError>;
}

// ========================================
// FILTER AND REQUEST TYPES
// ========================================

#[derive(Debug, Clone, Default)]
pub struct ModuleFilters {
    pub category: Option<String>,
    pub status: Option<String>,
    pub created_by: Option<UserId>,
    pub search: Option<String>,
    pub limit: Option<i32>,
    pub offset: Option<i32>,
}

#[derive(Debug, Clone, Default)]
pub struct ApiKeyFilters {
    pub client_name: Option<String>,
    pub status: Option<String>,
    pub created_by: Option<UserId>,
    pub expires_before: Option<DateTime<Utc>>,
    pub expires_after: Option<DateTime<Utc>>,
    pub limit: Option<i32>,
    pub offset: Option<i32>,
}

#[derive(Debug, Clone, Default)]
pub struct UsageLogFilters {
    pub user_id: Option<UserId>,
    pub api_key_id: Option<Uuid>,
    pub module_id: Option<Uuid>,
    pub endpoint: Option<String>,
    pub start_date: Option<DateTime<Utc>>,
    pub end_date: Option<DateTime<Utc>>,
    pub billable_only: Option<bool>,
    pub limit: Option<i32>,
    pub offset: Option<i32>,
}

#[derive(Debug, Clone, Default)]
pub struct AuditLogFilters {
    pub user_id: Option<UserId>,
    pub module_id: Option<Uuid>,
    pub action: Option<String>,
    pub performed_by: Option<UserId>,
    pub start_date: Option<DateTime<Utc>>,
    pub end_date: Option<DateTime<Utc>>,
    pub limit: Option<i32>,
    pub offset: Option<i32>,
}

#[derive(Debug, Clone)]
pub struct DateRange {
    pub start: DateTime<Utc>,
    pub end: DateTime<Utc>,
}

impl DateRange {
    pub fn last_30_days() -> Self {
        let end = Utc::now();
        let start = end - chrono::Duration::days(30);
        Self { start, end }
    }

    pub fn last_7_days() -> Self {
        let end = Utc::now();
        let start = end - chrono::Duration::days(7);
        Self { start, end }
    }

    pub fn current_month() -> Self {
        let now = Utc::now();
        let start = now.with_day(1).unwrap().with_hour(0).unwrap().with_minute(0).unwrap().with_second(0).unwrap();
        let end = now;
        Self { start, end }
    }
}

#[derive(Debug, Clone)]
pub struct UsageStatsRequest {
    pub user_id: Option<UserId>,
    pub api_key_id: Option<Uuid>,
    pub module_id: Option<Uuid>,
    pub period: DateRange,
    pub group_by: String, // "day", "hour", "module", "user"
}

// ========================================
// RESPONSE TYPES
// ========================================

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct UsageStats {
    pub total_requests: i64,
    pub unique_users: i64,
    pub unique_api_keys: i64,
    pub billable_requests: i64,
    pub total_cost_units: f64,
    pub avg_response_time_ms: Option<f64>,
    pub success_rate: f64,
    pub data_points: Vec<UsageDataPoint>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct UsageDataPoint {
    pub timestamp: DateTime<Utc>,
    pub requests: i64,
    pub users: i64,
    pub avg_response_time: Option<f64>,
    pub error_rate: f64,
    pub metadata: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ModuleUsageSummary {
    pub module_id: Uuid,
    pub module_name: String,
    pub total_requests: i64,
    pub unique_users: i64,
    pub unique_api_keys: i64,
    pub avg_requests_per_user: f64,
    pub peak_hour_requests: i64,
    pub most_used_endpoints: Vec<EndpointUsage>,
    pub access_level_breakdown: HashMap<String, i64>,
    pub error_rate: f64,
    pub period: DateRange,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct UserUsageSummary {
    pub user_id: UserId,
    pub total_requests: i64,
    pub modules_used: i64,
    pub favorite_module: Option<String>,
    pub quota_utilization: HashMap<String, f64>, // module_name -> utilization percentage
    pub daily_averages: HashMap<String, f64>, // metric -> average
    pub period: DateRange,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ApiKeyUsageSummary {
    pub key_id: Uuid,
    pub client_name: String,
    pub total_requests: i64,
    pub modules_used: i64,
    pub rate_limit_hits: i64,
    pub quota_utilization: HashMap<String, f64>,
    pub geographic_distribution: HashMap<String, i64>, // IP country -> request count
    pub period: DateRange,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct EndpointUsage {
    pub endpoint: String,
    pub requests: i64,
    pub unique_users: i64,
    pub avg_response_time: Option<f64>,
    pub error_rate: f64,
}

// ========================================
// AUDIT LOG TYPES
// ========================================

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AssignmentAuditLog {
    pub id: Uuid,
    pub assignment_id: Option<Uuid>,
    pub user_id: UserId,
    pub sub_module_id: Uuid,
    
    // Change details
    pub action: String, // created, updated, suspended, revoked, expired
    pub old_values: Option<serde_json::Value>,
    pub new_values: Option<serde_json::Value>,
    pub changes: Option<serde_json::Value>,
    
    // Context
    pub performed_by: UserId,
    pub reason: String,
    pub session_id: Option<String>,
    pub client_ip: Option<String>,
    pub user_agent: Option<String>,
    
    // Timestamp
    pub timestamp: DateTime<Utc>,
}

impl AssignmentAuditLog {
    pub fn new(
        user_id: UserId,
        sub_module_id: Uuid,
        action: String,
        performed_by: UserId,
        reason: String,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            assignment_id: None,
            user_id,
            sub_module_id,
            action,
            old_values: None,
            new_values: None,
            changes: None,
            performed_by,
            reason,
            session_id: None,
            client_ip: None,
            user_agent: None,
            timestamp: Utc::now(),
        }
    }

    pub fn for_assignment(mut self, assignment_id: Uuid) -> Self {
        self.assignment_id = Some(assignment_id);
        self
    }

    pub fn with_changes(mut self, old_values: serde_json::Value, new_values: serde_json::Value) -> Self {
        self.old_values = Some(old_values);
        self.new_values = Some(new_values);
        self
    }

    pub fn with_context(mut self, session_id: Option<String>, client_ip: Option<String>, user_agent: Option<String>) -> Self {
        self.session_id = session_id;
        self.client_ip = client_ip;
        self.user_agent = user_agent;
        self
    }
}

// ========================================
// UTILITY TRAITS
// ========================================

pub trait ModuleAccessChecker {
    fn check_access(&self, user_id: &UserId, module_name: &str, action: &str) -> impl std::future::Future<Output = Result<bool, DomainError>> + Send;
    fn check_quota(&self, user_id: &UserId, module_name: &str, quota_type: &str, amount: i32) -> impl std::future::Future<Output = Result<bool, DomainError>> + Send;
}

pub trait UsageTracker {
    fn track_usage(&self, usage_log: &ModuleUsageLog) -> impl std::future::Future<Output = Result<(), DomainError>> + Send;
    fn update_quotas(&self, user_id: &UserId, module_name: &str, quota_type: &str, consumed: i32) -> impl std::future::Future<Output = Result<(), DomainError>> + Send;
}

// ========================================
// BUILDER PATTERNS
// ========================================

pub struct ModuleAssignmentBuilder {
    user_id: UserId,
    sub_module_id: Uuid,
    access_level: String,
    assigned_by: UserId,
    assignment_reason: String,
    custom_quotas: Option<serde_json::Value>,
    restrictions: Option<serde_json::Value>,
    expires_at: Option<DateTime<Utc>>,
}

impl ModuleAssignmentBuilder {
    pub fn new(
        user_id: UserId,
        sub_module_id: Uuid,
        access_level: String,
        assigned_by: UserId,
        assignment_reason: String,
    ) -> Self {
        Self {
            user_id,
            sub_module_id,
            access_level,
            assigned_by,
            assignment_reason,
            custom_quotas: None,
            restrictions: None,
            expires_at: None,
        }
    }

    pub fn with_custom_quotas(mut self, quotas: serde_json::Value) -> Self {
        self.custom_quotas = Some(quotas);
        self
    }

    pub fn with_restrictions(mut self, restrictions: serde_json::Value) -> Self {
        self.restrictions = Some(restrictions);
        self
    }

    pub fn with_expiration(mut self, expires_at: DateTime<Utc>) -> Self {
        self.expires_at = Some(expires_at);
        self
    }

    pub fn build(self) -> UserSubModuleAssignment {
        let mut assignment = UserSubModuleAssignment::new(
            self.user_id,
            self.sub_module_id,
            self.access_level,
            self.assigned_by,
            self.assignment_reason,
        );

        if let Some(quotas) = self.custom_quotas {
            assignment.set_custom_quotas(quotas);
        }

        if let Some(restrictions) = self.restrictions {
            assignment.set_restrictions(restrictions);
        }

        if let Some(expires_at) = self.expires_at {
            assignment.set_expires_at(Some(expires_at));
        }

        assignment
    }
}