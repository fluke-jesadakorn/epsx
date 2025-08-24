// Admin module-based permission system

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Serialize, Deserialize};
use std::collections::{HashMap, HashSet};

use crate::dom::values::UserId;
use super::core::{Permission, PermissionContext, PermissionDecision, PermissionValidator, PermissionLevel, PermissionDenial, EffectivePermissions, DenialReason};
use super::errors::{PermissionError, ValidationError};

/// Admin module enumeration
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AdminModule {
    UserManagement,
    AnalyticsAccess,
    SystemConfiguration,
    AuditLogs,
    FinancialOversight,
    ContentManagement,
    SupportAccess,
    SecurityManagement,
    Custom(String),
}

/// Admin module permission levels
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AdminModulePermission {
    /// Read-only access to module
    View,
    /// Create new items in module
    Create,
    /// Modify existing items in module
    Update,
    /// Delete items in module
    Delete,
    /// Administrative control over module
    Admin,
    /// Full ownership and control
    Owner,
}

/// Admin module access configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdminModuleAccess {
    pub user_id: UserId,
    pub module: AdminModule,
    pub permissions: HashSet<AdminModulePermission>,
    pub granted_at: DateTime<Utc>,
    pub granted_by: UserId,
    pub expires_at: Option<DateTime<Utc>>,
    pub conditions: Option<HashMap<String, String>>,
    pub reason: Option<String>,
}

/// Admin module permission context
#[derive(Debug, Clone)]
pub struct AdminModuleContext {
    pub user_id: UserId,
    pub module: AdminModule,
    pub action: AdminModulePermission,
    pub resource: String,
    pub context_data: HashMap<String, String>,
    pub timestamp: DateTime<Utc>,
}

/// Admin module validator
pub struct AdminModuleValidator {
    /// Module access configurations
    module_access: HashMap<UserId, Vec<AdminModuleAccess>>,
    /// Module hierarchy for inheritance
    module_hierarchy: HashMap<AdminModule, Vec<AdminModule>>,
    /// Default permissions per module
    default_permissions: HashMap<AdminModule, HashSet<AdminModulePermission>>,
    /// Module configuration
    config: AdminModuleConfig,
}

/// Admin module validation configuration
#[derive(Debug, Clone)]
pub struct AdminModuleConfig {
    /// Enable inheritance from parent modules
    pub enable_inheritance: bool,
    /// Enable temporary permissions
    pub enable_temporary: bool,
    /// Default cache TTL for module permissions
    pub cache_ttl_seconds: u64,
    /// Maximum concurrent admin sessions
    pub max_concurrent_sessions: u32,
    /// Require re-authentication for sensitive modules
    pub require_reauth: HashMap<AdminModule, u32>, // minutes
}

// Module-specific permission definitions

/// User management module permissions
pub mod user_management {
    
    
    pub const VIEW_USERS: &str = "user-management:view";
    pub const CREATE_USERS: &str = "user-management:create";
    pub const UPDATE_USERS: &str = "user-management:update";
    pub const DELETE_USERS: &str = "user-management:delete";
    pub const MANAGE_ROLES: &str = "user-management:roles";
    pub const RESET_PASSWORDS: &str = "user-management:reset-password";
    pub const VIEW_SENSITIVE_INFO: &str = "user-management:sensitive-info";
    
    pub fn get_all_permissions() -> Vec<&'static str> {
        vec![
            VIEW_USERS,
            CREATE_USERS,
            UPDATE_USERS,
            DELETE_USERS,
            MANAGE_ROLES,
            RESET_PASSWORDS,
            VIEW_SENSITIVE_INFO,
        ]
    }
}

/// Analytics access module permissions
pub mod analytics_access {
    
    
    pub const VIEW_ANALYTICS: &str = "analytics-access:view";
    pub const EXPORT_DATA: &str = "analytics-access:export";
    pub const CREATE_REPORTS: &str = "analytics-access:reports";
    pub const MANAGE_DASHBOARDS: &str = "analytics-access:dashboards";
    pub const ACCESS_RAW_DATA: &str = "analytics-access:raw-data";
    pub const CONFIGURE_METRICS: &str = "analytics-access:metrics";
    
    pub fn get_all_permissions() -> Vec<&'static str> {
        vec![
            VIEW_ANALYTICS,
            EXPORT_DATA,
            CREATE_REPORTS,
            MANAGE_DASHBOARDS,
            ACCESS_RAW_DATA,
            CONFIGURE_METRICS,
        ]
    }
}

/// System configuration module permissions
pub mod system_configuration {
    
    
    pub const VIEW_CONFIG: &str = "system-configuration:view";
    pub const UPDATE_CONFIG: &str = "system-configuration:update";
    pub const MANAGE_FEATURES: &str = "system-configuration:features";
    pub const SYSTEM_MAINTENANCE: &str = "system-configuration:maintenance";
    pub const DATABASE_ADMIN: &str = "system-configuration:database";
    pub const CACHE_MANAGEMENT: &str = "system-configuration:cache";
    pub const API_CONFIGURATION: &str = "system-configuration:api";
    
    pub fn get_all_permissions() -> Vec<&'static str> {
        vec![
            VIEW_CONFIG,
            UPDATE_CONFIG,
            MANAGE_FEATURES,
            SYSTEM_MAINTENANCE,
            DATABASE_ADMIN,
            CACHE_MANAGEMENT,
            API_CONFIGURATION,
        ]
    }
}

/// Audit logs module permissions
pub mod audit_logs {
    
    
    pub const VIEW_AUDIT_LOGS: &str = "audit-logs:view";
    pub const EXPORT_AUDIT_LOGS: &str = "audit-logs:export";
    pub const MANAGE_RETENTION: &str = "audit-logs:retention";
    pub const SECURITY_ANALYSIS: &str = "audit-logs:security-analysis";
    pub const COMPLIANCE_REPORTS: &str = "audit-logs:compliance";
    pub const ALERT_CONFIGURATION: &str = "audit-logs:alerts";
    
    pub fn get_all_permissions() -> Vec<&'static str> {
        vec![
            VIEW_AUDIT_LOGS,
            EXPORT_AUDIT_LOGS,
            MANAGE_RETENTION,
            SECURITY_ANALYSIS,
            COMPLIANCE_REPORTS,
            ALERT_CONFIGURATION,
        ]
    }
}

/// Security management module permissions
pub mod security_management {
    
    
    pub const VIEW_SECURITY_EVENTS: &str = "security-management:events";
    pub const MANAGE_POLICIES: &str = "security-management:policies";
    pub const THREAT_RESPONSE: &str = "security-management:threat-response";
    pub const ACCESS_CONTROL: &str = "security-management:access-control";
    pub const VULNERABILITY_MANAGEMENT: &str = "security-management:vulnerabilities";
    pub const INCIDENT_MANAGEMENT: &str = "security-management:incidents";
    
    pub fn get_all_permissions() -> Vec<&'static str> {
        vec![
            VIEW_SECURITY_EVENTS,
            MANAGE_POLICIES,
            THREAT_RESPONSE,
            ACCESS_CONTROL,
            VULNERABILITY_MANAGEMENT,
            INCIDENT_MANAGEMENT,
        ]
    }
}

// Implementations

impl AdminModule {
    /// Get the string representation of the module
    pub fn as_str(&self) -> &str {
        match self {
            AdminModule::UserManagement => "user-management",
            AdminModule::AnalyticsAccess => "analytics-access",
            AdminModule::SystemConfiguration => "system-configuration",
            AdminModule::AuditLogs => "audit-logs",
            AdminModule::FinancialOversight => "financial-oversight",
            AdminModule::ContentManagement => "content-management",
            AdminModule::SupportAccess => "support-access",
            AdminModule::SecurityManagement => "security-management",
            AdminModule::Custom(name) => name,
        }
    }
    
    /// Get all available permissions for this module
    pub fn get_permissions(&self) -> Vec<String> {
        match self {
            AdminModule::UserManagement => user_management::get_all_permissions()
                .into_iter()
                .map(|s| s.to_string())
                .collect(),
            AdminModule::AnalyticsAccess => analytics_access::get_all_permissions()
                .into_iter()
                .map(|s| s.to_string())
                .collect(),
            AdminModule::SystemConfiguration => system_configuration::get_all_permissions()
                .into_iter()
                .map(|s| s.to_string())
                .collect(),
            AdminModule::AuditLogs => audit_logs::get_all_permissions()
                .into_iter()
                .map(|s| s.to_string())
                .collect(),
            AdminModule::SecurityManagement => security_management::get_all_permissions()
                .into_iter()
                .map(|s| s.to_string())
                .collect(),
            _ => vec![], // Other modules would need their own implementations
        }
    }
    
    /// Check if this module is sensitive and requires enhanced security
    pub fn is_sensitive(&self) -> bool {
        matches!(
            self,
            AdminModule::SecurityManagement
                | AdminModule::SystemConfiguration
                | AdminModule::AuditLogs
                | AdminModule::FinancialOversight
        )
    }
    
    /// Get the required minimum permission level for this module
    pub fn required_minimum_level(&self) -> PermissionLevel {
        match self {
            AdminModule::UserManagement => PermissionLevel::Admin,
            AdminModule::SecurityManagement => PermissionLevel::Owner,
            AdminModule::SystemConfiguration => PermissionLevel::Owner,
            AdminModule::AuditLogs => PermissionLevel::Admin,
            AdminModule::FinancialOversight => PermissionLevel::Admin,
            _ => PermissionLevel::Write,
        }
    }
}

impl std::str::FromStr for AdminModule {
    type Err = ValidationError;
    
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "user-management" => Ok(AdminModule::UserManagement),
            "analytics-access" => Ok(AdminModule::AnalyticsAccess),
            "system-configuration" => Ok(AdminModule::SystemConfiguration),
            "audit-logs" => Ok(AdminModule::AuditLogs),
            "financial-oversight" => Ok(AdminModule::FinancialOversight),
            "content-management" => Ok(AdminModule::ContentManagement),
            "support-access" => Ok(AdminModule::SupportAccess),
            "security-management" => Ok(AdminModule::SecurityManagement),
            _ => Ok(AdminModule::Custom(s.to_string())),
        }
    }
}

impl std::fmt::Display for AdminModule {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl AdminModulePermission {
    /// Convert to permission level
    pub fn to_permission_level(&self) -> PermissionLevel {
        match self {
            AdminModulePermission::View => PermissionLevel::Read,
            AdminModulePermission::Create => PermissionLevel::Write,
            AdminModulePermission::Update => PermissionLevel::Write,
            AdminModulePermission::Delete => PermissionLevel::Admin,
            AdminModulePermission::Admin => PermissionLevel::Admin,
            AdminModulePermission::Owner => PermissionLevel::Owner,
        }
    }
    
    /// Check if this permission includes another permission
    pub fn includes(&self, other: &AdminModulePermission) -> bool {
        use AdminModulePermission::*;
        
        match (self, other) {
            (Owner, _) => true,
            (Admin, Admin) | (Admin, Delete) | (Admin, Update) | (Admin, Create) | (Admin, View) => true,
            (Delete, Delete) | (Delete, Update) | (Delete, Create) | (Delete, View) => true,
            (Update, Update) | (Update, Create) | (Update, View) => true,
            (Create, Create) | (Create, View) => true,
            (View, View) => true,
            _ => false,
        }
    }
}

impl std::str::FromStr for AdminModulePermission {
    type Err = ValidationError;
    
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "view" => Ok(AdminModulePermission::View),
            "create" => Ok(AdminModulePermission::Create),
            "update" => Ok(AdminModulePermission::Update),
            "delete" => Ok(AdminModulePermission::Delete),
            "admin" => Ok(AdminModulePermission::Admin),
            "owner" => Ok(AdminModulePermission::Owner),
            _ => Err(ValidationError::InvalidLevel {
                level: s.to_string(),
            }),
        }
    }
}

impl std::fmt::Display for AdminModulePermission {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            AdminModulePermission::View => "view",
            AdminModulePermission::Create => "create",
            AdminModulePermission::Update => "update",
            AdminModulePermission::Delete => "delete",
            AdminModulePermission::Admin => "admin",
            AdminModulePermission::Owner => "owner",
        };
        write!(f, "{}", s)
    }
}

impl AdminModuleAccess {
    pub fn new(
        user_id: UserId,
        module: AdminModule,
        permissions: HashSet<AdminModulePermission>,
        granted_by: UserId,
    ) -> Self {
        Self {
            user_id,
            module,
            permissions,
            granted_at: Utc::now(),
            granted_by,
            expires_at: None,
            conditions: None,
            reason: None,
        }
    }
    
    pub fn with_expiration(mut self, expires_at: DateTime<Utc>) -> Self {
        self.expires_at = Some(expires_at);
        self
    }
    
    pub fn with_conditions(mut self, conditions: HashMap<String, String>) -> Self {
        self.conditions = Some(conditions);
        self
    }
    
    pub fn with_reason(mut self, reason: String) -> Self {
        self.reason = Some(reason);
        self
    }
    
    pub fn is_expired(&self) -> bool {
        if let Some(expires_at) = self.expires_at {
            Utc::now() > expires_at
        } else {
            false
        }
    }
    
    pub fn has_permission(&self, permission: &AdminModulePermission) -> bool {
        if self.is_expired() {
            return false;
        }
        
        self.permissions.iter().any(|p| p.includes(permission))
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
}

impl AdminModuleValidator {
    pub fn new(config: AdminModuleConfig) -> Self {
        let mut validator = Self {
            module_access: HashMap::new(),
            module_hierarchy: HashMap::new(),
            default_permissions: HashMap::new(),
            config,
        };
        
        validator.initialize_hierarchy();
        validator.initialize_default_permissions();
        validator
    }
    
    fn initialize_hierarchy(&mut self) {
        // Define module hierarchy for inheritance
        self.module_hierarchy.insert(
            AdminModule::UserManagement,
            vec![AdminModule::SupportAccess],
        );
        self.module_hierarchy.insert(
            AdminModule::SecurityManagement,
            vec![AdminModule::AuditLogs, AdminModule::SystemConfiguration],
        );
    }
    
    fn initialize_default_permissions(&mut self) {
        // Define default permissions per module
        self.default_permissions.insert(
            AdminModule::SupportAccess,
            [AdminModulePermission::View, AdminModulePermission::Create]
                .iter()
                .cloned()
                .collect(),
        );
        
        self.default_permissions.insert(
            AdminModule::ContentManagement,
            [
                AdminModulePermission::View,
                AdminModulePermission::Create,
                AdminModulePermission::Update,
            ]
            .iter()
            .cloned()
            .collect(),
        );
    }
    
    pub fn grant_module_access(&mut self, access: AdminModuleAccess) {
        self.module_access
            .entry(access.user_id.clone())
            .or_insert_with(Vec::new)
            .push(access);
    }
    
    pub fn revoke_module_access(&mut self, user_id: &UserId, module: &AdminModule) {
        if let Some(access_list) = self.module_access.get_mut(user_id) {
            access_list.retain(|access| &access.module != module);
        }
    }
    
    pub fn get_user_module_access(&self, user_id: &UserId) -> Vec<&AdminModuleAccess> {
        self.module_access
            .get(user_id)
            .map(|access_list| access_list.iter().filter(|a| !a.is_expired()).collect())
            .unwrap_or_default()
    }
    
    pub fn validate_module_permission(
        &self,
        user_id: &UserId,
        module: &AdminModule,
        permission: &AdminModulePermission,
        context: &HashMap<String, String>,
    ) -> Result<bool, PermissionError> {
        // Check direct module access
        if let Some(access_list) = self.module_access.get(user_id) {
            for access in access_list.iter().filter(|a| !a.is_expired()) {
                if &access.module == module {
                    if access.has_permission(permission) && access.evaluate_conditions(context) {
                        return Ok(true);
                    }
                }
            }
        }
        
        // Check inherited permissions if enabled
        if self.config.enable_inheritance {
            if let Some(parent_modules) = self.module_hierarchy.get(module) {
                for parent in parent_modules {
                    if self.validate_module_permission(user_id, parent, permission, context)? {
                        return Ok(true);
                    }
                }
            }
        }
        
        // Check default permissions
        if let Some(default_perms) = self.default_permissions.get(module) {
            if default_perms.iter().any(|p| p.includes(permission)) {
                return Ok(true);
            }
        }
        
        Ok(false)
    }
}

#[async_trait]
impl PermissionValidator for AdminModuleValidator {
    async fn validate(&self, context: &PermissionContext) -> Result<PermissionDecision, PermissionError> {
        // Parse permission to extract module and action
        let permission_parts: Vec<&str> = context.permission.split(':').collect();
        if permission_parts.len() != 2 {
            return Ok(PermissionDecision::Denied(PermissionDenial {
                request_id: uuid::Uuid::new_v4(),
                user_id: context.user_id.clone(),
                denied_permissions: vec![context.permission.clone()],
                reason: DenialReason::PolicyViolation,
                denied_at: chrono::Utc::now(),
                retry_after: None,
            }));
        }
        
        let module_str = permission_parts[0];
        let action_str = permission_parts[1];
        
        let module: AdminModule = match module_str.parse() {
            Ok(m) => m,
            Err(_) => return Ok(PermissionDecision::Denied(PermissionDenial {
                request_id: uuid::Uuid::new_v4(),
                user_id: context.user_id.clone(),
                denied_permissions: vec![context.permission.clone()],
                reason: DenialReason::PolicyViolation,
                denied_at: chrono::Utc::now(),
                retry_after: None,
            })),
        };
        
        let action: AdminModulePermission = match action_str.parse() {
            Ok(a) => a,
            Err(_) => return Ok(PermissionDecision::Denied(PermissionDenial {
                request_id: uuid::Uuid::new_v4(),
                user_id: context.user_id.clone(),
                denied_permissions: vec![context.permission.clone()],
                reason: DenialReason::PolicyViolation,
                denied_at: chrono::Utc::now(),
                retry_after: None,
            })),
        };
        
        // Validate permission
        let has_permission = match self.validate_module_permission(
            &context.user_id,
            &module,
            &action,
            &context.context_data,
        ) {
            Ok(result) => result,
            Err(_) => return Ok(PermissionDecision::Denied(PermissionDenial {
                request_id: uuid::Uuid::new_v4(),
                user_id: context.user_id.clone(),
                denied_permissions: vec![context.permission.clone()],
                reason: DenialReason::InsufficientPermissions,
                denied_at: chrono::Utc::now(),
                retry_after: None,
            })),
        };
        
        if has_permission {
            let grant = super::core::PermissionGrant {
                request_id: uuid::Uuid::new_v4(),
                user_id: context.user_id.clone(),
                granted_permissions: vec![Permission::new(
                    context.permission.clone(),
                    context.resource.clone(),
                )],
                granted_at: context.timestamp,
                granted_by: UserId::new("system".to_string()),
                expires_at: None,
                conditions: None,
            };
            
            Ok(PermissionDecision::Granted(grant))
        } else {
            let denial = super::core::PermissionDenial {
                request_id: uuid::Uuid::new_v4(),
                user_id: context.user_id.clone(),
                denied_permissions: vec![context.permission.clone()],
                reason: super::core::DenialReason::InsufficientPermissions,
                denied_at: context.timestamp,
                retry_after: None,
            };
            
            Ok(PermissionDecision::Denied(denial))
        }
    }
    
    async fn has_permission(&self, user_id: &UserId, permission: &str, _resource: &str) -> bool {
        let context = HashMap::new();
        
        // Parse permission
        if let Ok(module) = permission.split(':').next().unwrap_or("").parse::<AdminModule>() {
            if let Ok(action) = permission.split(':').nth(1).unwrap_or("").parse::<AdminModulePermission>() {
                return self.validate_module_permission(user_id, &module, &action, &context)
                    .unwrap_or(false);
            }
        }
        
        false
    }
    
    async fn get_permissions(&self, user_id: &UserId) -> Result<Vec<Permission>, PermissionError> {
        let mut permissions = Vec::new();
        
        if let Some(access_list) = self.module_access.get(user_id) {
            for access in access_list.iter().filter(|a| !a.is_expired()) {
                for perm in &access.permissions {
                    let permission_str = format!("{}:{}", access.module, perm);
                    permissions.push(Permission::new(permission_str, "*".to_string()));
                }
            }
        }
        
        Ok(permissions)
    }

    async fn get_effective_permissions(&self, user_id: &UserId) -> Result<EffectivePermissions, PermissionError> {
        let permissions = self.get_permissions(user_id).await?;
        
        let mut admin_modules = HashMap::new();
        if let Some(access_list) = self.module_access.get(user_id) {
            for access in access_list.iter().filter(|a| !a.is_expired()) {
                // Determine permission level based on permissions granted
                let level = if access.permissions.contains(&AdminModulePermission::Owner) {
                    PermissionLevel::Owner
                } else if access.permissions.contains(&AdminModulePermission::Admin) {
                    PermissionLevel::Admin
                } else if access.permissions.contains(&AdminModulePermission::Update) || 
                         access.permissions.contains(&AdminModulePermission::Delete) || 
                         access.permissions.contains(&AdminModulePermission::Create) {
                    PermissionLevel::Write
                } else {
                    PermissionLevel::Read
                };
                admin_modules.insert(access.module.clone(), level);
            }
        }
        
        Ok(EffectivePermissions {
            user_id: user_id.clone(),
            permissions,
            admin_modules,
            tier_features: HashMap::new(), // Not used in admin module validator
            limitations: Vec::new(),
            expires_at: None,
            computed_at: chrono::Utc::now(),
            computation_time: std::time::Duration::from_millis(0),
        })
    }

    async fn validate_batch(&self, contexts: &[PermissionContext]) -> Result<Vec<PermissionDecision>, PermissionError> {
        let mut results = Vec::new();
        for context in contexts {
            let result = self.validate(context).await?;
            results.push(result);
        }
        Ok(results)
    }
}

impl Default for AdminModuleConfig {
    fn default() -> Self {
        let mut require_reauth = HashMap::new();
        require_reauth.insert(AdminModule::SecurityManagement, 15);
        require_reauth.insert(AdminModule::SystemConfiguration, 30);
        require_reauth.insert(AdminModule::FinancialOversight, 20);
        
        Self {
            enable_inheritance: true,
            enable_temporary: true,
            cache_ttl_seconds: 300, // 5 minutes
            max_concurrent_sessions: 10,
            require_reauth,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_admin_module_parsing() {
        let module: AdminModule = "user-management".parse().unwrap();
        assert_eq!(module, AdminModule::UserManagement);
        assert_eq!(module.as_str(), "user-management");
    }
    
    #[test]
    fn test_admin_module_permission_hierarchy() {
        assert!(AdminModulePermission::Owner.includes(&AdminModulePermission::View));
        assert!(AdminModulePermission::Admin.includes(&AdminModulePermission::Delete));
        assert!(AdminModulePermission::Update.includes(&AdminModulePermission::View));
        assert!(!AdminModulePermission::View.includes(&AdminModulePermission::Update));
    }
    
    #[test]
    fn test_admin_module_access_creation() {
        let user_id = UserId::new("user123".to_string());
        let granted_by = UserId::new("admin123".to_string());
        
        let mut permissions = HashSet::new();
        permissions.insert(AdminModulePermission::View);
        permissions.insert(AdminModulePermission::Create);
        
        let access = AdminModuleAccess::new(
            user_id.clone(),
            AdminModule::UserManagement,
            permissions.clone(),
            granted_by,
        );
        
        assert_eq!(access.user_id, user_id);
        assert_eq!(access.module, AdminModule::UserManagement);
        assert_eq!(access.permissions, permissions);
        assert!(!access.is_expired());
    }
    
    #[test]
    fn test_admin_module_access_expiration() {
        let user_id = UserId::new("user123".to_string());
        let granted_by = UserId::new("admin123".to_string());
        
        let permissions = [AdminModulePermission::View].iter().cloned().collect();
        let past_time = Utc::now() - chrono::Duration::hours(1);
        
        let access = AdminModuleAccess::new(
            user_id,
            AdminModule::UserManagement,
            permissions,
            granted_by,
        ).with_expiration(past_time);
        
        assert!(access.is_expired());
        assert!(!access.has_permission(&AdminModulePermission::View));
    }
    
    #[test]
    fn test_admin_module_validator_creation() {
        let config = AdminModuleConfig::default();
        let validator = AdminModuleValidator::new(config);
        
        assert!(validator.config.enable_inheritance);
        assert!(validator.config.enable_temporary);
    }
    
    #[tokio::test]
    async fn test_admin_module_permission_validation() {
        let config = AdminModuleConfig::default();
        let mut validator = AdminModuleValidator::new(config);
        
        let user_id = UserId::new("user123".to_string());
        let admin_id = UserId::new("admin123".to_string());
        
        // Grant user management access
        let permissions = [AdminModulePermission::View, AdminModulePermission::Create]
            .iter()
            .cloned()
            .collect();
        
        let access = AdminModuleAccess::new(
            user_id.clone(),
            AdminModule::UserManagement,
            permissions,
            admin_id,
        );
        
        validator.grant_module_access(access);
        
        // Test permission validation
        assert!(validator.has_permission(&user_id, "user-management:view", "*").await);
        assert!(validator.has_permission(&user_id, "user-management:create", "*").await);
        assert!(!validator.has_permission(&user_id, "user-management:delete", "*").await);
    }
    
    #[test]
    fn test_module_sensitivity() {
        assert!(AdminModule::SecurityManagement.is_sensitive());
        assert!(AdminModule::SystemConfiguration.is_sensitive());
        assert!(!AdminModule::SupportAccess.is_sensitive());
    }
    
    #[test]
    fn test_module_required_levels() {
        assert_eq!(
            AdminModule::SecurityManagement.required_minimum_level(),
            PermissionLevel::Owner
        );
        assert_eq!(
            AdminModule::UserManagement.required_minimum_level(),
            PermissionLevel::Admin
        );
    }
}