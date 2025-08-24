// Permission API Data Transfer Objects
//
// Defines request/response structures for the permission validation API,
// providing type-safe data exchange with comprehensive validation and
// serialization support for all permission operations.

use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use std::collections::HashMap;
use uuid::Uuid;
use crate::{
    // permissions::*,
    dom::values::UserId,
};

// ============================================================================
// Request DTOs
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidatePermissionRequest {
    pub user_id: UserId,
    pub permission: String,
    pub resource: Option<String>,
    pub context: Option<PermissionContextDto>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidateBatchPermissionsRequest {
    pub user_id: UserId,
    pub permissions: Vec<PermissionRequestDto>,
    pub context: Option<PermissionContextDto>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PermissionRequestDto {
    pub permission: String,
    pub resource: Option<String>,
    pub required: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PermissionContextDto {
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub session_id: Option<String>,
    pub additional_claims: Option<HashMap<String, serde_json::Value>>,
    pub security_level: Option<String>,
    pub time_constraints: Option<TimeConstraintsDto>,
    pub location_constraints: Option<LocationConstraintsDto>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeConstraintsDto {
    pub valid_from: Option<DateTime<Utc>>,
    pub valid_until: Option<DateTime<Utc>>,
    pub allowed_hours: Option<Vec<u8>>, // 0-23
    pub allowed_days: Option<Vec<u8>>,  // 0-6 (Sunday-Saturday)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocationConstraintsDto {
    pub allowed_countries: Option<Vec<String>>,
    pub allowed_ip_ranges: Option<Vec<String>>,
    pub blocked_countries: Option<Vec<String>>,
    pub blocked_ip_ranges: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GrantPermissionRequest {
    pub user_id: UserId,
    pub permission: String,
    pub resource: Option<String>,
    pub expires_at: Option<DateTime<Utc>>,
    pub granted_by: UserId,
    pub reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RevokePermissionRequest {
    pub user_id: UserId,
    pub permission: String,
    pub resource: Option<String>,
    pub revoked_by: UserId,
    pub reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ElevatePermissionsRequest {
    pub user_id: UserId,
    pub permissions: Vec<String>,
    pub duration_minutes: i64,
    pub justification: String,
    pub approved_by: Option<UserId>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssignModulePermissionRequest {
    pub user_id: UserId,
    pub module: String,
    pub permissions: Vec<String>,
    pub assigned_by: UserId,
    pub expires_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GrantTemporaryAdminRequest {
    pub user_id: UserId,
    pub admin_level: String,
    pub duration_minutes: i64,
    pub justification: String,
    pub modules: Vec<String>,
    pub approved_by: UserId,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpgradeTierRequest {
    pub user_id: UserId,
    pub target_tier: String,
    pub effective_date: Option<DateTime<Utc>>,
    pub billing_cycle: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreatePermissionTemplateRequest {
    pub name: String,
    pub description: String,
    pub permissions: Vec<TemplatePermissionDto>,
    pub applicable_roles: Vec<String>,
    pub created_by: UserId,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplatePermissionDto {
    pub permission: String,
    pub resource_pattern: Option<String>,
    pub conditions: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApplyTemplateRequest {
    pub template_id: Uuid,
    pub user_ids: Vec<UserId>,
    pub applied_by: UserId,
    pub override_existing: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BulkPermissionRequest {
    pub operations: Vec<BulkPermissionOperationDto>,
    pub executed_by: UserId,
    pub dry_run: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BulkPermissionOperationDto {
    pub operation_type: String, // "grant", "revoke", "validate"
    pub user_id: UserId,
    pub permission: String,
    pub resource: Option<String>,
    pub expires_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreatePermissionPolicyRequest {
    pub name: String,
    pub description: String,
    pub policy_type: String,
    pub conditions: serde_json::Value,
    pub actions: Vec<String>,
    pub priority: i32,
    pub enabled: bool,
    pub created_by: UserId,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestPermissionPolicyRequest {
    pub policy_id: Uuid,
    pub test_cases: Vec<PolicyTestCaseDto>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyTestCaseDto {
    pub user_id: UserId,
    pub permission: String,
    pub resource: Option<String>,
    pub context: Option<PermissionContextDto>,
    pub expected_result: bool,
}

// ============================================================================
// Response DTOs
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidatePermissionResponse {
    pub allowed: bool,
    pub permission: String,
    pub resource: Option<String>,
    pub user_id: UserId,
    pub validation_time_ms: f64,
    pub cached: bool,
    pub source: String, // "admin_module", "package_tier", "explicit", "inherited"
    pub expires_at: Option<DateTime<Utc>>,
    pub constraints: Option<PermissionConstraintsDto>,
    pub audit_id: Uuid,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PermissionConstraintsDto {
    pub time_limited: bool,
    pub ip_restricted: bool,
    pub location_restricted: bool,
    pub session_bound: bool,
    pub elevation_required: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidateBatchPermissionsResponse {
    pub user_id: UserId,
    pub results: Vec<BatchPermissionResultDto>,
    pub total_validation_time_ms: f64,
    pub cache_hit_rate: f64,
    pub audit_id: Uuid,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchPermissionResultDto {
    pub permission: String,
    pub resource: Option<String>,
    pub allowed: bool,
    pub source: String,
    pub validation_time_ms: f64,
    pub cached: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserPermissionsResponse {
    pub user_id: UserId,
    pub effective_permissions: Vec<EffectivePermissionDto>,
    pub admin_modules: Vec<AdminModulePermissionDto>,
    pub package_tier: PackageTierInfoDto,
    pub temporary_elevations: Vec<TemporaryElevationDto>,
    pub inherited_permissions: Vec<InheritedPermissionDto>,
    pub last_updated: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EffectivePermissionDto {
    pub permission: String,
    pub resource: Option<String>,
    pub source: String,
    pub granted_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
    pub granted_by: Option<UserId>,
    pub constraints: Option<PermissionConstraintsDto>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdminModulePermissionDto {
    pub module: String,
    pub permissions: Vec<String>,
    pub assigned_at: DateTime<Utc>,
    pub assigned_by: UserId,
    pub expires_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageTierInfoDto {
    pub tier: String,
    pub tier_name: String,
    pub features: Vec<TierFeatureDto>,
    pub limits: HashMap<String, serde_json::Value>,
    pub upgraded_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TierFeatureDto {
    pub feature: String,
    pub enabled: bool,
    pub usage_limit: Option<i64>,
    pub current_usage: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemporaryElevationDto {
    pub permissions: Vec<String>,
    pub elevated_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub elevated_by: UserId,
    pub justification: String,
    pub remaining_minutes: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InheritedPermissionDto {
    pub permission: String,
    pub inherited_from: String, // role, group, etc.
    pub inheritance_path: Vec<String>,
    pub depth: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdminModulesResponse {
    pub modules: Vec<AdminModuleDto>,
    pub user_assignments: HashMap<String, Vec<UserId>>,
    pub total_users: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdminModuleDto {
    pub module: String,
    pub display_name: String,
    pub description: String,
    pub permissions: Vec<String>,
    pub required_tier: Option<String>,
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageTiersResponse {
    pub tiers: Vec<PackageTierDto>,
    pub current_promotions: Vec<PromotionDto>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageTierDto {
    pub tier: String,
    pub display_name: String,
    pub description: String,
    pub features: Vec<String>,
    pub limits: HashMap<String, serde_json::Value>,
    pub pricing: Option<PricingInfoDto>,
    pub popular: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PricingInfoDto {
    pub monthly_price: f64,
    pub annual_price: f64,
    pub currency: String,
    pub billing_cycles: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromotionDto {
    pub id: Uuid,
    pub title: String,
    pub description: String,
    pub discount_percent: f64,
    pub valid_until: DateTime<Utc>,
    pub applicable_tiers: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PermissionTemplatesResponse {
    pub templates: Vec<PermissionTemplateDto>,
    pub total: i64,
    pub page: i32,
    pub per_page: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PermissionTemplateDto {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub permissions: Vec<TemplatePermissionDto>,
    pub applicable_roles: Vec<String>,
    pub usage_count: i64,
    pub created_by: UserId,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEventsResponse {
    pub events: Vec<AuditEventDto>,
    pub total: i64,
    pub page: i32,
    pub per_page: i32,
    pub has_more: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEventDto {
    pub id: Uuid,
    pub event_type: String,
    pub user_id: Option<UserId>,
    pub permission: Option<String>,
    pub resource: Option<String>,
    pub action: String,
    pub result: String,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub timestamp: DateTime<Utc>,
    pub session_id: Option<String>,
    pub additional_data: Option<serde_json::Value>,
    pub security_score: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PermissionHealthResponse {
    pub status: String,
    pub cache_health: CacheHealthDto,
    pub database_health: DatabaseHealthDto,
    pub validation_performance: PerformanceMetricsDto,
    pub error_rates: ErrorRatesDto,
    pub system_load: SystemLoadDto,
    pub last_updated: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheHealthDto {
    pub status: String,
    pub hit_rate: f64,
    pub memory_usage: f64,
    pub eviction_rate: f64,
    pub connection_pool_status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseHealthDto {
    pub status: String,
    pub connection_pool_active: i32,
    pub connection_pool_idle: i32,
    pub query_performance_ms: f64,
    pub migration_status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetricsDto {
    pub avg_validation_time_ms: f64,
    pub p95_validation_time_ms: f64,
    pub p99_validation_time_ms: f64,
    pub throughput_per_second: f64,
    pub cache_hit_rate: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorRatesDto {
    pub validation_errors: f64,
    pub cache_errors: f64,
    pub database_errors: f64,
    pub network_errors: f64,
    pub timeout_errors: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemLoadDto {
    pub cpu_usage: f64,
    pub memory_usage: f64,
    pub active_connections: i32,
    pub queue_depth: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PermissionMetricsResponse {
    pub metrics: PermissionSystemMetricsDto,
    pub time_range: TimeRangeDto,
    pub collected_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PermissionSystemMetricsDto {
    pub total_validations: i64,
    pub successful_validations: i64,
    pub failed_validations: i64,
    pub cache_hits: i64,
    pub cache_misses: i64,
    pub avg_response_time_ms: f64,
    pub error_distribution: HashMap<String, i64>,
    pub top_permissions: Vec<PermissionUsageDto>,
    pub active_users: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PermissionUsageDto {
    pub permission: String,
    pub usage_count: i64,
    pub success_rate: f64,
    pub avg_response_time_ms: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeRangeDto {
    pub start: DateTime<Utc>,
    pub end: DateTime<Utc>,
    pub duration_hours: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BulkOperationResponse {
    pub operation_id: Uuid,
    pub total_operations: i32,
    pub successful_operations: i32,
    pub failed_operations: i32,
    pub results: Vec<BulkOperationResultDto>,
    pub execution_time_ms: f64,
    pub dry_run: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BulkOperationResultDto {
    pub operation_type: String,
    pub user_id: UserId,
    pub permission: String,
    pub success: bool,
    pub error: Option<String>,
    pub execution_time_ms: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PermissionPolicyResponse {
    pub policies: Vec<PermissionPolicyDto>,
    pub total: i64,
    pub active_policies: i64,
    pub inactive_policies: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PermissionPolicyDto {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub policy_type: String,
    pub conditions: serde_json::Value,
    pub actions: Vec<String>,
    pub priority: i32,
    pub enabled: bool,
    pub created_by: UserId,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub last_evaluated: Option<DateTime<Utc>>,
    pub evaluation_count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyTestResponse {
    pub policy_id: Uuid,
    pub test_results: Vec<PolicyTestResultDto>,
    pub overall_success: bool,
    pub execution_time_ms: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyTestResultDto {
    pub test_case: PolicyTestCaseDto,
    pub actual_result: bool,
    pub expected_result: bool,
    pub success: bool,
    pub execution_time_ms: f64,
    pub error: Option<String>,
}

// ============================================================================
// Common Error Response
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiErrorResponse {
    pub error: String,
    pub error_code: String,
    pub message: String,
    pub details: Option<serde_json::Value>,
    pub timestamp: DateTime<Utc>,
    pub request_id: Uuid,
}

// ============================================================================
// Validation and Conversion Traits
// ============================================================================

impl ValidatePermissionRequest {
    pub fn validate(&self) -> Result<(), String> {
        if self.permission.is_empty() {
            return Err("Permission cannot be empty".to_string());
        }
        
        if !self.permission.contains(':') {
            return Err("Permission must have format 'module:action'".to_string());
        }
        
        Ok(())
    }
}

impl ValidateBatchPermissionsRequest {
    pub fn validate(&self) -> Result<(), String> {
        if self.permissions.is_empty() {
            return Err("Must specify at least one permission to validate".to_string());
        }
        
        if self.permissions.len() > 100 {
            return Err("Cannot validate more than 100 permissions at once".to_string());
        }
        
        for perm in &self.permissions {
            if perm.permission.is_empty() {
                return Err("Permission cannot be empty".to_string());
            }
            
            if !perm.permission.contains(':') {
                return Err("Permission must have format 'module:action'".to_string());
            }
        }
        
        Ok(())
    }
}

// Conversion from domain types
impl From<crate::permissions::PermissionResult> for ValidatePermissionResponse {
    fn from(result: crate::permissions::PermissionResult) -> Self {
        Self {
            allowed: result.allowed,
            permission: result.permission.name().to_string(),
            resource: Some(result.permission.resource),
            user_id: result.context.user_id,
            validation_time_ms: result.validation_time_ms,
            cached: result.cached,
            source: result.source.unwrap_or_else(|| "unknown".to_string()),
            expires_at: result.expires_at,
            constraints: None, // TODO: Map from domain constraints
            audit_id: result.audit_id.unwrap_or_else(|| Uuid::new_v4()),
        }
    }
}