// Core permission validation engine and types

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Serialize, Deserialize};
use std::collections::{HashMap, HashSet};
use std::time::{Duration, Instant};
use uuid::Uuid;

use crate::dom::values::UserId;
use crate::dom::entities::iam::PackageTier;
use super::errors::PermissionError;
use super::admin_modules::AdminModule;
use super::package_tiers::TierFeature;

/// Core permission entity representing a single permission
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Permission {
    /// Permission identifier (e.g., "user:read", "admin:delete")
    pub id: String,
    /// Resource this permission applies to
    pub resource: String,
    /// Optional conditions for permission evaluation
    pub conditions: Option<HashMap<String, String>>,
    /// Permission scope (global, module, user-specific)
    pub scope: PermissionScope,
    /// Permission level (read, write, admin)
    pub level: PermissionLevel,
    /// Expiration time for temporary permissions
    pub expires_at: Option<DateTime<Utc>>,
    /// Permission metadata
    pub metadata: PermissionMetadata,
}

/// Permission scope enumeration
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PermissionScope {
    Global,
    Module(String),
    Resource(String),
    User(UserId),
}

/// Permission level enumeration
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PermissionLevel {
    Read,
    Write,
    Admin,
    Owner,
}

/// Permission metadata
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PermissionMetadata {
    pub granted_at: DateTime<Utc>,
    pub granted_by: Option<UserId>,
    pub reason: Option<String>,
    pub tags: Vec<String>,
    pub custom_fields: HashMap<String, String>,
}

/// Permission request context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PermissionContext {
    /// User requesting permission
    pub user_id: UserId,
    /// Permission being requested
    pub permission: String,
    /// Resource being accessed
    pub resource: String,
    /// Additional context data
    pub context_data: HashMap<String, String>,
    /// Request timestamp
    pub timestamp: DateTime<Utc>,
    /// IP address of request
    pub ip_address: Option<String>,
    /// User agent
    pub user_agent: Option<String>,
    /// Session ID
    pub session_id: Option<String>,
}

/// Permission request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PermissionRequest {
    pub id: Uuid,
    pub user_id: UserId,
    pub permissions: Vec<String>,
    pub resource: String,
    pub context: HashMap<String, String>,
    pub requested_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
    pub priority: RequestPriority,
}

/// Permission grant result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PermissionGrant {
    pub request_id: Uuid,
    pub user_id: UserId,
    pub granted_permissions: Vec<Permission>,
    pub granted_at: DateTime<Utc>,
    pub granted_by: UserId,
    pub expires_at: Option<DateTime<Utc>>,
    pub conditions: Option<HashMap<String, String>>,
}

/// Permission denial result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PermissionDenial {
    pub request_id: Uuid,
    pub user_id: UserId,
    pub denied_permissions: Vec<String>,
    pub reason: DenialReason,
    pub denied_at: DateTime<Utc>,
    pub retry_after: Option<DateTime<Utc>>,
}

/// Permission decision from validation
#[derive(Debug, Clone)]
pub enum PermissionDecision {
    Granted(PermissionGrant),
    Denied(PermissionDenial),
    Partial(PermissionGrant, PermissionDenial),
}

/// Validation result struct expected by handlers
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PermissionResult {
    pub allowed: bool,
    pub permission: Permission,
    pub context: PermissionContext,
    pub validation_time_ms: f64,
    pub cached: bool,
    pub source: Option<String>,
    pub expires_at: Option<DateTime<Utc>>,
    pub audit_id: Option<Uuid>,
}

/// Request priority levels
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum RequestPriority {
    Low,
    Normal,
    High,
    Critical,
    Emergency,
}

/// Permission denial reasons
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DenialReason {
    InsufficientPermissions,
    ExpiredPermission,
    ResourceNotFound,
    PolicyViolation,
    RateLimited,
    SecurityThreat,
    MaintenanceMode,
    InsufficientTier,
    ModuleDisabled,
    TemporaryBlock,
    Custom(String),
}

/// User permission profile combining all permission sources
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserPermissionProfile {
    pub user_id: UserId,
    pub package_tier: PackageTier,
    pub admin_modules: HashSet<AdminModule>,
    pub tier_features: HashSet<TierFeature>,
    pub direct_permissions: Vec<Permission>,
    pub inherited_permissions: Vec<Permission>,
    pub temporary_permissions: Vec<Permission>,
    pub denied_permissions: Vec<Permission>,
    pub last_updated: DateTime<Utc>,
    pub cache_version: u64,
}

/// Effective permissions result after resolution
#[derive(Debug, Clone)]
pub struct EffectivePermissions {
    pub user_id: UserId,
    pub permissions: Vec<Permission>,
    pub admin_modules: HashMap<AdminModule, PermissionLevel>,
    pub tier_features: HashMap<TierFeature, bool>,
    pub limitations: Vec<PermissionLimitation>,
    pub expires_at: Option<DateTime<Utc>>,
    pub computed_at: DateTime<Utc>,
    pub computation_time: Duration,
}

/// Permission limitations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PermissionLimitation {
    pub permission: String,
    pub limit_type: LimitationType,
    pub current_value: i64,
    pub max_value: i64,
    pub reset_time: Option<DateTime<Utc>>,
}

/// Limitation types
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum LimitationType {
    RateLimit,
    QuotaLimit,
    ConcurrentLimit,
    DailyLimit,
    MonthlyLimit,
    Custom(String),
}

/// Core permission validation engine
pub struct PermissionEngine {
    /// Performance metrics
    pub metrics: PermissionMetrics,
    /// Configuration
    pub config: PermissionConfig,
}

/// Permission validation metrics
#[derive(Debug, Default)]
pub struct PermissionMetrics {
    pub total_validations: u64,
    pub cache_hits: u64,
    pub cache_misses: u64,
    pub average_validation_time: Duration,
    pub denied_validations: u64,
    pub security_violations: u64,
    pub last_reset: DateTime<Utc>,
}

/// Permission system configuration
#[derive(Debug, Clone)]
pub struct PermissionConfig {
    pub cache_ttl: Duration,
    pub max_cache_size: u64,
    pub max_validation_time: Duration,
    pub enable_audit_logging: bool,
    pub enable_security_monitoring: bool,
    pub deny_by_default: bool,
    pub enable_inheritance: bool,
    pub enable_temporary_permissions: bool,
}

/// Main unified permission system
pub struct UnifiedPermissionSystem {
    pub engine: PermissionEngine,
    pub validators: Vec<Box<dyn PermissionValidator>>,
    pub cache: Option<Box<dyn super::cache::PermissionCache>>,
    pub audit: Option<Box<dyn super::audit::PermissionAuditTrait>>,
}

// Core trait implementations

#[async_trait]
pub trait PermissionValidator: Send + Sync {
    /// Validate a permission request
    async fn validate(&self, context: &PermissionContext) -> Result<PermissionDecision, PermissionError>;
    
    /// Check if user has specific permission
    async fn has_permission(&self, _user_id: &UserId, permission: &str, resource: &str) -> bool;
    
    /// Get all permissions for user
    async fn get_permissions(&self, _user_id: &UserId) -> Result<Vec<Permission>, PermissionError>;
    
    /// Get effective permissions with inheritance
    async fn get_effective_permissions(&self, _user_id: &UserId) -> Result<EffectivePermissions, PermissionError>;
    
    /// Validate multiple permissions at once
    async fn validate_batch(&self, contexts: &[PermissionContext]) -> Result<Vec<PermissionDecision>, PermissionError>;
}

// Implementations

impl Permission {
    pub fn new(id: String, resource: String) -> Self {
        Self {
            id,
            resource,
            conditions: None,
            scope: PermissionScope::Global,
            level: PermissionLevel::Read,
            expires_at: None,
            metadata: PermissionMetadata::default(),
        }
    }
    
    pub fn with_scope(mut self, scope: PermissionScope) -> Self {
        self.scope = scope;
        self
    }
    
    pub fn with_level(mut self, level: PermissionLevel) -> Self {
        self.level = level;
        self
    }
    
    pub fn with_conditions(mut self, conditions: HashMap<String, String>) -> Self {
        self.conditions = Some(conditions);
        self
    }
    
    pub fn with_expiration(mut self, expires_at: DateTime<Utc>) -> Self {
        self.expires_at = Some(expires_at);
        self
    }
    
    pub fn is_expired(&self) -> bool {
        if let Some(expires_at) = self.expires_at {
            Utc::now() > expires_at
        } else {
            false
        }
    }
    
    pub fn matches(&self, permission: &str, resource: &str) -> bool {
        self.matches_permission(permission) && self.matches_resource(resource)
    }
    
    pub fn matches_permission(&self, permission: &str) -> bool {
        super::utils::matches_permission_pattern(permission, &self.id)
    }
    
    pub fn matches_resource(&self, resource: &str) -> bool {
        super::utils::matches_permission_pattern(resource, &self.resource)
    }
    
    pub fn evaluate_conditions(&self, context: &HashMap<String, String>) -> bool {
        if let Some(conditions) = &self.conditions {
            conditions.iter().all(|(key, value)| {
                context.get(key).map_or(false, |v| v == value)
            })
        } else {
            true
        }
    }
    
    /// Get permission name (alias for id for compatibility)
    pub fn name(&self) -> &str {
        &self.id
    }
    
    /// Get granted_at timestamp (from metadata for compatibility)
    pub fn granted_at(&self) -> DateTime<Utc> {
        self.metadata.granted_at
    }
    
    /// Get source information (from metadata for compatibility)
    pub fn source(&self) -> Option<String> {
        self.metadata.reason.clone().or_else(|| Some("direct".to_string()))
    }
}

impl PermissionMetadata {
    pub fn new(granted_by: Option<UserId>) -> Self {
        Self {
            granted_at: Utc::now(),
            granted_by,
            reason: None,
            tags: Vec::new(),
            custom_fields: HashMap::new(),
        }
    }
    
    pub fn with_reason(mut self, reason: String) -> Self {
        self.reason = Some(reason);
        self
    }
    
    pub fn with_tags(mut self, tags: Vec<String>) -> Self {
        self.tags = tags;
        self
    }
}

impl Default for PermissionMetadata {
    fn default() -> Self {
        Self::new(None)
    }
}

impl PermissionContext {
    pub fn new(user_id: UserId, permission: String, resource: String) -> Self {
        Self {
            user_id,
            permission,
            resource,
            context_data: HashMap::new(),
            timestamp: Utc::now(),
            ip_address: None,
            user_agent: None,
            session_id: None,
        }
    }
    
    pub fn with_context_data(mut self, data: HashMap<String, String>) -> Self {
        self.context_data = data;
        self
    }
    
    pub fn with_ip_address(mut self, ip: String) -> Self {
        self.ip_address = Some(ip);
        self
    }
    
    pub fn with_session(mut self, session_id: String) -> Self {
        self.session_id = Some(session_id);
        self
    }
}

impl UserPermissionProfile {
    pub fn new(user_id: UserId, package_tier: PackageTier) -> Self {
        Self {
            user_id,
            package_tier,
            admin_modules: HashSet::new(),
            tier_features: HashSet::new(),
            direct_permissions: Vec::new(),
            inherited_permissions: Vec::new(),
            temporary_permissions: Vec::new(),
            denied_permissions: Vec::new(),
            last_updated: Utc::now(),
            cache_version: 1,
        }
    }
    
    pub fn add_admin_module(&mut self, module: AdminModule) {
        self.admin_modules.insert(module);
        self.touch();
    }
    
    pub fn add_tier_feature(&mut self, feature: TierFeature) {
        self.tier_features.insert(feature);
        self.touch();
    }
    
    pub fn add_permission(&mut self, permission: Permission) {
        self.direct_permissions.push(permission);
        self.touch();
    }
    
    pub fn has_admin_module(&self, module: &AdminModule) -> bool {
        self.admin_modules.contains(module)
    }
    
    pub fn has_tier_feature(&self, feature: &TierFeature) -> bool {
        self.tier_features.contains(feature)
    }
    
    pub fn get_all_permissions(&self) -> Vec<Permission> {
        let mut permissions = Vec::new();
        permissions.extend(self.direct_permissions.clone());
        permissions.extend(self.inherited_permissions.clone());
        permissions.extend(
            self.temporary_permissions
                .iter()
                .filter(|p| !p.is_expired())
                .cloned()
        );
        permissions
    }
    
    fn touch(&mut self) {
        self.last_updated = Utc::now();
        self.cache_version += 1;
    }
}

impl PermissionEngine {
    pub fn new(config: PermissionConfig) -> Self {
        Self {
            metrics: PermissionMetrics::default(),
            config,
        }
    }
    
    pub fn record_validation(&mut self, duration: Duration, granted: bool) {
        self.metrics.total_validations += 1;
        
        if !granted {
            self.metrics.denied_validations += 1;
        }
        
        // Update average validation time using exponential moving average
        let alpha = 0.1;
        let current_avg = self.metrics.average_validation_time.as_nanos() as f64;
        let new_time = duration.as_nanos() as f64;
        let new_avg = alpha * new_time + (1.0 - alpha) * current_avg;
        self.metrics.average_validation_time = Duration::from_nanos(new_avg as u64);
    }
    
    pub fn is_performance_degraded(&self) -> bool {
        self.metrics.average_validation_time > self.config.max_validation_time
    }
    
    pub fn reset_metrics(&mut self) {
        self.metrics = PermissionMetrics::default();
        self.metrics.last_reset = Utc::now();
    }
}

impl Default for PermissionConfig {
    fn default() -> Self {
        Self {
            cache_ttl: Duration::from_secs(super::constants::PERMISSION_CACHE_TTL_SECONDS),
            max_cache_size: super::constants::PERMISSION_CACHE_MAX_SIZE,
            max_validation_time: Duration::from_millis(super::constants::MAX_PERMISSION_CHECK_TIME_MS),
            enable_audit_logging: true,
            enable_security_monitoring: true,
            deny_by_default: true,
            enable_inheritance: true,
            enable_temporary_permissions: true,
        }
    }
}

impl UnifiedPermissionSystem {
    pub fn new(config: PermissionConfig) -> Self {
        Self {
            engine: PermissionEngine::new(config),
            validators: Vec::new(),
            cache: None,
            audit: None,
        }
    }
    
    pub fn add_validator<V: PermissionValidator + 'static>(&mut self, validator: V) {
        self.validators.push(Box::new(validator));
    }
    
    pub fn set_cache<C: super::cache::PermissionCache + 'static>(&mut self, cache: C) {
        self.cache = Some(Box::new(cache));
    }
    
    pub fn set_audit<A: super::audit::PermissionAuditTrait + 'static>(&mut self, audit: A) {
        self.audit = Some(Box::new(audit));
    }
    
    pub async fn validate_permission(&self, context: &PermissionContext) -> Result<PermissionDecision, PermissionError> {
        let start_time = Instant::now();
        
        // TODO: Cache temporarily disabled due to type mismatch between PermissionResult and PermissionDecision
        // This needs to be fixed by either:
        // 1. Converting between the types
        // 2. Updating cache to use PermissionDecision
        /*
        if let Some(cache) = &self.cache {
            let cache_key = super::utils::generate_permission_key(
                &context.user_id.value().to_string(),
                &context.permission,
                &context.resource,
            );
            
            if let Ok(Some(cached_result)) = cache.get(&cache_key).await {
                self.engine.metrics.cache_hits += 1;
                return Ok(cached_result);
            }
            
            self.engine.metrics.cache_misses += 1;
        }
        */
        
        // Run through all validators
        let mut results = Vec::new();
        for validator in &self.validators {
            match validator.validate(context).await {
                Ok(result) => results.push(result),
                Err(e) => {
                    if let Some(audit) = &self.audit {
                        let _ = audit.log_error(&context.user_id, &context.permission, &e).await;
                    }
                    return Err(e);
                }
            }
        }
        
        // Combine results using policy engine
        let final_result = self.combine_results(results).await?;
        
        // TODO: Cache set temporarily disabled due to type mismatch
        /*
        if let Some(cache) = &self.cache {
            let cache_key = super::utils::generate_permission_key(
                &context.user_id.value().to_string(),
                &context.permission,
                &context.resource,
            );
            let _ = cache.set(&cache_key, &final_result, self.engine.config.cache_ttl).await;
        }
        */
        
        // Record metrics
        let _duration = start_time.elapsed();
        let _granted = matches!(final_result, PermissionDecision::Granted(_));
        // TODO: Implement metrics recording
        // self.engine.record_validation(_duration, _granted);
        
        // Audit log
        if let Some(audit) = &self.audit {
            let _ = audit.log_validation(context, &final_result).await;
        }
        
        Ok(final_result)
    }
    
    /// Get all permissions for a user (stub implementation)
    pub async fn get_permissions(&self, _user_id: &UserId) -> Result<Vec<Permission>, PermissionError> {
        // TODO: Implement actual permission retrieval logic
        Ok(vec![])
    }
    
    async fn combine_results(&self, results: Vec<PermissionDecision>) -> Result<PermissionDecision, PermissionError> {
        if results.is_empty() {
            return Ok(PermissionDecision::Denied(PermissionDenial {
                request_id: Uuid::new_v4(),
                user_id: UserId::new("unknown".to_string()),
                denied_permissions: vec!["*".to_string()],
                reason: DenialReason::InsufficientPermissions,
                denied_at: Utc::now(),
                retry_after: None,
            }));
        }
        
        // Simple policy: any grant wins, but explicit denials take precedence
        let mut granted_permissions = Vec::new();
        let mut denied_permissions = Vec::new();
        let mut has_explicit_denial = false;
        
        for result in results {
            match result {
                PermissionDecision::Granted(grant) => {
                    granted_permissions.extend(grant.granted_permissions);
                }
                PermissionDecision::Denied(denial) => {
                    denied_permissions.extend(denial.denied_permissions);
                    if matches!(denial.reason, DenialReason::PolicyViolation | DenialReason::SecurityThreat) {
                        has_explicit_denial = true;
                    }
                }
                PermissionDecision::Partial(grant, denial) => {
                    granted_permissions.extend(grant.granted_permissions);
                    denied_permissions.extend(denial.denied_permissions);
                }
            }
        }
        
        if has_explicit_denial || (granted_permissions.is_empty() && !denied_permissions.is_empty()) {
            Ok(PermissionDecision::Denied(PermissionDenial {
                request_id: Uuid::new_v4(),
                user_id: UserId::new("combined".to_string()),
                denied_permissions,
                reason: if has_explicit_denial { DenialReason::PolicyViolation } else { DenialReason::InsufficientPermissions },
                denied_at: Utc::now(),
                retry_after: None,
            }))
        } else if !granted_permissions.is_empty() {
            Ok(PermissionDecision::Granted(PermissionGrant {
                request_id: Uuid::new_v4(),
                user_id: UserId::new("combined".to_string()),
                granted_permissions,
                granted_at: Utc::now(),
                granted_by: UserId::new("system".to_string()),
                expires_at: None,
                conditions: None,
            }))
        } else {
            Ok(PermissionDecision::Denied(PermissionDenial {
                request_id: Uuid::new_v4(),
                user_id: UserId::new("combined".to_string()),
                denied_permissions: vec!["*".to_string()],
                reason: DenialReason::InsufficientPermissions,
                denied_at: Utc::now(),
                retry_after: None,
            }))
        }
    }
}

impl PermissionDecision {
    /// Check if permission was granted
    pub fn allowed(&self) -> bool {
        matches!(self, PermissionDecision::Granted(_) | PermissionDecision::Partial(_, _))
    }
    
    /// Check if result was cached (for compatibility)
    pub fn cached(&self) -> bool {
        false // Default for now
    }
    
    /// Get source information (for compatibility) 
    pub fn source(&self) -> Option<String> {
        Some("permission_engine".to_string())
    }
    
    /// Convert PermissionDecision to PermissionResult for compatibility
    pub fn to_result(&self, context: &PermissionContext, validation_time_ms: f64) -> PermissionResult {
        match self {
            PermissionDecision::Granted(grant) => {
                let permission = grant.granted_permissions.first()
                    .cloned()
                    .unwrap_or_else(|| Permission::new(
                        context.permission.clone(),
                        context.resource.clone()
                    ));
                
                PermissionResult {
                    allowed: true,
                    permission,
                    context: context.clone(),
                    validation_time_ms,
                    cached: false,
                    source: Some("permission_engine".to_string()),
                    expires_at: grant.expires_at,
                    audit_id: Some(grant.request_id),
                }
            }
            PermissionDecision::Denied(_) => {
                let permission = Permission::new(
                    context.permission.clone(),
                    context.resource.clone()
                );
                
                PermissionResult {
                    allowed: false,
                    permission,
                    context: context.clone(),
                    validation_time_ms,
                    cached: false,
                    source: Some("permission_engine".to_string()),
                    expires_at: None,
                    audit_id: Some(uuid::Uuid::new_v4()),
                }
            }
            PermissionDecision::Partial(grant, _) => {
                let permission = grant.granted_permissions.first()
                    .cloned()
                    .unwrap_or_else(|| Permission::new(
                        context.permission.clone(),
                        context.resource.clone()
                    ));
                
                PermissionResult {
                    allowed: true,
                    permission,
                    context: context.clone(),
                    validation_time_ms,
                    cached: false,
                    source: Some("permission_engine".to_string()),
                    expires_at: grant.expires_at,
                    audit_id: Some(grant.request_id),
                }
            }
        }
    }
}

// Display implementations
impl std::fmt::Display for PermissionLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PermissionLevel::Read => write!(f, "read"),
            PermissionLevel::Write => write!(f, "write"),
            PermissionLevel::Admin => write!(f, "admin"),
            PermissionLevel::Owner => write!(f, "owner"),
        }
    }
}

impl std::fmt::Display for DenialReason {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DenialReason::InsufficientPermissions => write!(f, "Insufficient permissions"),
            DenialReason::ExpiredPermission => write!(f, "Permission expired"),
            DenialReason::ResourceNotFound => write!(f, "Resource not found"),
            DenialReason::PolicyViolation => write!(f, "Policy violation"),
            DenialReason::RateLimited => write!(f, "Rate limited"),
            DenialReason::SecurityThreat => write!(f, "Security threat detected"),
            DenialReason::MaintenanceMode => write!(f, "System in maintenance mode"),
            DenialReason::InsufficientTier => write!(f, "Insufficient package tier"),
            DenialReason::ModuleDisabled => write!(f, "Module disabled"),
            DenialReason::TemporaryBlock => write!(f, "Temporarily blocked"),
            DenialReason::Custom(reason) => write!(f, "{}", reason),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_permission_creation() {
        let perm = Permission::new("user:read".to_string(), "users/*".to_string())
            .with_level(PermissionLevel::Read)
            .with_scope(PermissionScope::Module("users".to_string()));
        
        assert_eq!(perm.id, "user:read");
        assert_eq!(perm.resource, "users/*");
        assert_eq!(perm.level, PermissionLevel::Read);
        assert!(!perm.is_expired());
    }
    
    #[test]
    fn test_permission_matching() {
        let perm = Permission::new("user:*".to_string(), "users/*".to_string());
        
        assert!(perm.matches("user:read", "users/123"));
        assert!(perm.matches("user:write", "users/456"));
        assert!(!perm.matches("admin:read", "users/123"));
        assert!(!perm.matches("user:read", "payments/123"));
    }
    
    #[test]
    fn test_permission_expiration() {
        let past_time = Utc::now() - chrono::Duration::hours(1);
        let future_time = Utc::now() + chrono::Duration::hours(1);
        
        let expired_perm = Permission::new("test:read".to_string(), "*".to_string())
            .with_expiration(past_time);
        let valid_perm = Permission::new("test:read".to_string(), "*".to_string())
            .with_expiration(future_time);
        let no_expiry_perm = Permission::new("test:read".to_string(), "*".to_string());
        
        assert!(expired_perm.is_expired());
        assert!(!valid_perm.is_expired());
        assert!(!no_expiry_perm.is_expired());
    }
    
    #[test]
    fn test_context_creation() {
        let user_id = UserId::new("user123".to_string());
        let context = PermissionContext::new(
            user_id.clone(),
            "user:read".to_string(),
            "profile".to_string(),
        ).with_ip_address("127.0.0.1".to_string());
        
        assert_eq!(context.user_id, user_id);
        assert_eq!(context.permission, "user:read");
        assert_eq!(context.resource, "profile");
        assert_eq!(context.ip_address, Some("127.0.0.1".to_string()));
    }
    
    #[test]
    fn test_user_permission_profile() {
        let user_id = UserId::new("user123".to_string());
        let mut profile = UserPermissionProfile::new(user_id.clone(), PackageTier::Gold);
        
        profile.add_permission(Permission::new("user:read".to_string(), "*".to_string()));
        
        assert_eq!(profile.user_id, user_id);
        assert_eq!(profile.package_tier, PackageTier::Gold);
        assert_eq!(profile.direct_permissions.len(), 1);
        assert_eq!(profile.cache_version, 2); // Incremented after adding permission
    }
}