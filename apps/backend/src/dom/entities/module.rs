// Domain entities for the module system

use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};

use crate::dom::values::UserId;

// ========================================
// SUB-MODULE ENTITY  
// ========================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubModule {
    id: Uuid,
    name: String,
    display_name: String,
    description: Option<String>,
    category: String,
    icon: Option<String>,
    
    // Configuration
    api_endpoints: serde_json::Value,
    ui_components: serde_json::Value,
    feature_flags: serde_json::Value,
    access_levels: serde_json::Value,
    default_quotas: serde_json::Value,
    pricing_tiers: serde_json::Value,
    
    // Dependencies
    dependencies: Vec<String>,
    conflicts: Vec<String>,
    
    // Status
    status: String,
    version: String,
    min_package_tier: Option<String>,
    
    // Audit
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
    created_by: UserId,
}

impl SubModule {
    pub fn new(
        name: String,
        display_name: String,
        description: Option<String>,
        category: String,
        created_by: UserId,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            name,
            display_name,
            description,
            category,
            icon: None,
            api_endpoints: serde_json::json!({}),
            ui_components: serde_json::json!({}),
            feature_flags: serde_json::json!({}),
            access_levels: serde_json::json!({}),
            default_quotas: serde_json::json!({}),
            pricing_tiers: serde_json::json!({}),
            dependencies: vec![],
            conflicts: vec![],
            status: "active".to_string(),
            version: "1.0".to_string(),
            min_package_tier: None,
            created_at: now,
            updated_at: now,
            created_by,
        }
    }

    // Getters
    pub fn id(&self) -> &Uuid { &self.id }
    pub fn name(&self) -> &str { &self.name }
    pub fn display_name(&self) -> &str { &self.display_name }
    pub fn description(&self) -> Option<&str> { self.description.as_deref() }
    pub fn category(&self) -> &str { &self.category }
    pub fn icon(&self) -> Option<&str> { self.icon.as_deref() }
    pub fn status(&self) -> &str { &self.status }
    pub fn version(&self) -> &str { &self.version }
    pub fn created_at(&self) -> &DateTime<Utc> { &self.created_at }
    pub fn updated_at(&self) -> &DateTime<Utc> { &self.updated_at }
    pub fn created_by(&self) -> &UserId { &self.created_by }

    // Configuration getters
    pub fn api_endpoints(&self) -> &serde_json::Value { &self.api_endpoints }
    pub fn ui_components(&self) -> &serde_json::Value { &self.ui_components }
    pub fn feature_flags(&self) -> &serde_json::Value { &self.feature_flags }
    pub fn access_levels(&self) -> &serde_json::Value { &self.access_levels }
    pub fn default_quotas(&self) -> &serde_json::Value { &self.default_quotas }
    pub fn pricing_tiers(&self) -> &serde_json::Value { &self.pricing_tiers }
    pub fn dependencies(&self) -> &[String] { &self.dependencies }
    pub fn conflicts(&self) -> &[String] { &self.conflicts }

    // Setters
    pub fn set_description(&mut self, description: Option<String>) {
        self.description = description;
        self.updated_at = Utc::now();
    }

    pub fn set_icon(&mut self, icon: Option<String>) {
        self.icon = icon;
        self.updated_at = Utc::now();
    }

    pub fn set_status(&mut self, status: String) {
        self.status = status;
        self.updated_at = Utc::now();
    }

    pub fn set_api_endpoints(&mut self, endpoints: serde_json::Value) {
        self.api_endpoints = endpoints;
        self.updated_at = Utc::now();
    }

    pub fn set_ui_components(&mut self, components: serde_json::Value) {
        self.ui_components = components;
        self.updated_at = Utc::now();
    }

    pub fn set_feature_flags(&mut self, flags: serde_json::Value) {
        self.feature_flags = flags;
        self.updated_at = Utc::now();
    }

    pub fn set_access_levels(&mut self, levels: serde_json::Value) {
        self.access_levels = levels;
        self.updated_at = Utc::now();
    }

    pub fn set_default_quotas(&mut self, quotas: serde_json::Value) {
        self.default_quotas = quotas;
        self.updated_at = Utc::now();
    }

    pub fn set_dependencies(&mut self, dependencies: Vec<String>) {
        self.dependencies = dependencies;
        self.updated_at = Utc::now();
    }

    // Business logic
    pub fn is_active(&self) -> bool {
        self.status == "active"
    }

    pub fn has_dependency(&self, module_name: &str) -> bool {
        self.dependencies.contains(&module_name.to_string())
    }

    pub fn conflicts_with(&self, module_name: &str) -> bool {
        self.conflicts.contains(&module_name.to_string())
    }

    pub fn supports_access_level(&self, level: &str) -> bool {
        self.access_levels.as_object()
            .map(|levels| levels.contains_key(level))
            .unwrap_or(false)
    }
}

// ========================================
// USER MODULE ASSIGNMENT ENTITY
// ========================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserSubModuleAssignment {
    id: Uuid,
    user_id: UserId,
    sub_module_id: Uuid,
    
    // Access configuration
    access_level: String,
    custom_quotas: serde_json::Value,
    restrictions: serde_json::Value,
    
    // Assignment metadata
    assigned_by: UserId,
    assignment_reason: String,
    assignment_type: String,
    
    // Lifecycle
    starts_at: DateTime<Utc>,
    expires_at: Option<DateTime<Utc>>,
    status: String,
    
    // Usage tracking
    first_used_at: Option<DateTime<Utc>>,
    last_used_at: Option<DateTime<Utc>>,
    usage_count: i32,
    
    // Audit
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl UserSubModuleAssignment {
    pub fn new(
        user_id: UserId,
        sub_module_id: Uuid,
        access_level: String,
        assigned_by: UserId,
        assignment_reason: String,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            user_id,
            sub_module_id,
            access_level,
            custom_quotas: serde_json::json!({}),
            restrictions: serde_json::json!({}),
            assigned_by,
            assignment_reason,
            assignment_type: "manual".to_string(),
            starts_at: now,
            expires_at: None,
            status: "active".to_string(),
            first_used_at: None,
            last_used_at: None,
            usage_count: 0,
            created_at: now,
            updated_at: now,
        }
    }

    // Getters
    pub fn id(&self) -> &Uuid { &self.id }
    pub fn user_id(&self) -> &UserId { &self.user_id }
    pub fn sub_module_id(&self) -> &Uuid { &self.sub_module_id }
    pub fn access_level(&self) -> &str { &self.access_level }
    pub fn custom_quotas(&self) -> &serde_json::Value { &self.custom_quotas }
    pub fn restrictions(&self) -> &serde_json::Value { &self.restrictions }
    pub fn assigned_by(&self) -> &UserId { &self.assigned_by }
    pub fn assignment_reason(&self) -> &str { &self.assignment_reason }
    pub fn assignment_type(&self) -> &str { &self.assignment_type }
    pub fn starts_at(&self) -> &DateTime<Utc> { &self.starts_at }
    pub fn expires_at(&self) -> Option<&DateTime<Utc>> { self.expires_at.as_ref() }
    pub fn status(&self) -> &str { &self.status }
    pub fn usage_count(&self) -> i32 { self.usage_count }
    pub fn created_at(&self) -> &DateTime<Utc> { &self.created_at }
    pub fn updated_at(&self) -> &DateTime<Utc> { &self.updated_at }

    // Setters
    pub fn set_access_level(&mut self, level: String) {
        self.access_level = level;
        self.updated_at = Utc::now();
    }

    pub fn set_custom_quotas(&mut self, quotas: serde_json::Value) {
        self.custom_quotas = quotas;
        self.updated_at = Utc::now();
    }

    pub fn set_restrictions(&mut self, restrictions: serde_json::Value) {
        self.restrictions = restrictions;
        self.updated_at = Utc::now();
    }

    pub fn set_expires_at(&mut self, expires_at: Option<DateTime<Utc>>) {
        self.expires_at = expires_at;
        self.updated_at = Utc::now();
    }

    pub fn set_status(&mut self, status: String) {
        self.status = status;
        self.updated_at = Utc::now();
    }

    pub fn record_usage(&mut self) {
        let now = Utc::now();
        if self.first_used_at.is_none() {
            self.first_used_at = Some(now);
        }
        self.last_used_at = Some(now);
        self.usage_count += 1;
        self.updated_at = now;
    }

    // Business logic
    pub fn is_active(&self) -> bool {
        self.status == "active" && 
        self.starts_at <= Utc::now() &&
        self.expires_at.map_or(true, |exp| exp > Utc::now())
    }

    pub fn is_expired(&self) -> bool {
        self.expires_at.map_or(false, |exp| exp <= Utc::now())
    }

    pub fn suspend(&mut self, reason: &str) {
        self.status = "suspended".to_string();
        self.assignment_reason = format!("{} | Suspended: {}", self.assignment_reason, reason);
        self.updated_at = Utc::now();
    }

    pub fn revoke(&mut self, reason: &str) {
        self.status = "revoked".to_string();
        self.assignment_reason = format!("{} | Revoked: {}", self.assignment_reason, reason);
        self.updated_at = Utc::now();
    }
}

// ========================================
// API KEY ENTITY
// ========================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiKey {
    id: Uuid,
    key_hash: String,
    key_prefix: String,
    
    // Client information
    client_name: String,
    client_description: Option<String>,
    client_contact_email: Option<String>,
    client_website: Option<String>,
    
    // Access configuration
    allowed_modules: serde_json::Value,
    rate_limits: serde_json::Value,
    permissions: serde_json::Value,
    
    // Security configuration
    ip_restrictions: Vec<String>,
    allowed_domains: Vec<String>,
    allowed_user_agents: Vec<String>,
    require_https: bool,
    
    // Lifecycle
    starts_at: DateTime<Utc>,
    expires_at: Option<DateTime<Utc>>,
    status: String,
    
    // Usage tracking
    first_used_at: Option<DateTime<Utc>>,
    last_used_at: Option<DateTime<Utc>>,
    total_requests: i32,
    usage_stats: serde_json::Value,
    
    // Management
    created_by: UserId,
    managed_by: Option<UserId>,
    notes: Option<String>,
    
    // Audit
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl ApiKey {
    pub fn new(
        key_hash: String,
        key_prefix: String,
        client_name: String,
        created_by: UserId,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            key_hash,
            key_prefix,
            client_name,
            client_description: None,
            client_contact_email: None,
            client_website: None,
            allowed_modules: serde_json::json!([]),
            rate_limits: serde_json::json!({}),
            permissions: serde_json::json!({}),
            ip_restrictions: vec![],
            allowed_domains: vec![],
            allowed_user_agents: vec![],
            require_https: true,
            starts_at: now,
            expires_at: None,
            status: "active".to_string(),
            first_used_at: None,
            last_used_at: None,
            total_requests: 0,
            usage_stats: serde_json::json!({}),
            created_by,
            managed_by: None,
            notes: None,
            created_at: now,
            updated_at: now,
        }
    }

    // Getters
    pub fn id(&self) -> &Uuid { &self.id }
    pub fn key_hash(&self) -> &str { &self.key_hash }
    pub fn key_prefix(&self) -> &str { &self.key_prefix }
    pub fn client_name(&self) -> &str { &self.client_name }
    pub fn client_description(&self) -> Option<&str> { self.client_description.as_deref() }
    pub fn client_contact_email(&self) -> Option<&str> { self.client_contact_email.as_deref() }
    pub fn allowed_modules(&self) -> &serde_json::Value { &self.allowed_modules }
    pub fn rate_limits(&self) -> &serde_json::Value { &self.rate_limits }
    pub fn permissions(&self) -> &serde_json::Value { &self.permissions }
    pub fn status(&self) -> &str { &self.status }
    pub fn total_requests(&self) -> i32 { self.total_requests }
    pub fn created_by(&self) -> &UserId { &self.created_by }
    pub fn created_at(&self) -> &DateTime<Utc> { &self.created_at }

    // Business logic
    pub fn is_active(&self) -> bool {
        self.status == "active" && 
        self.starts_at <= Utc::now() &&
        self.expires_at.map_or(true, |exp| exp > Utc::now())
    }

    pub fn record_usage(&mut self) {
        let now = Utc::now();
        if self.first_used_at.is_none() {
            self.first_used_at = Some(now);
        }
        self.last_used_at = Some(now);
        self.total_requests += 1;
        self.updated_at = now;
    }

    pub fn has_module_access(&self, module_id: &Uuid) -> bool {
        self.allowed_modules.as_array()
            .map(|modules| {
                modules.iter().any(|m| {
                    m.get("module_id")
                        .and_then(|id| id.as_str())
                        .map(|id_str| id_str == module_id.to_string())
                        .unwrap_or(false)
                })
            })
            .unwrap_or(false)
    }

    pub fn set_status(&mut self, status: String) {
        self.status = status;
        self.updated_at = Utc::now();
    }

    pub fn add_module_access(&mut self, module_id: Uuid, access_level: String, quotas: serde_json::Value) {
        if let Some(modules) = self.allowed_modules.as_array_mut() {
            modules.push(serde_json::json!({
                "module_id": module_id,
                "access_level": access_level,
                "quotas": quotas
            }));
        }
        self.updated_at = Utc::now();
    }

    pub fn remove_module_access(&mut self, module_id: &Uuid) {
        if let Some(modules) = self.allowed_modules.as_array_mut() {
            modules.retain(|m| {
                m.get("module_id")
                    .and_then(|id| id.as_str())
                    .map(|id_str| id_str != module_id.to_string())
                    .unwrap_or(true)
            });
        }
        self.updated_at = Utc::now();
    }
}

// ========================================
// MODULE USAGE LOG ENTITY
// ========================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModuleUsageLog {
    pub id: Uuid,
    pub user_id: Option<UserId>,
    pub api_key_id: Option<Uuid>,
    pub sub_module_id: Option<Uuid>,
    
    // Request details
    pub endpoint: String,
    pub request_method: String,
    pub response_status: i32,
    pub response_time_ms: Option<i32>,
    
    // Usage tracking
    pub quota_consumed: i32,
    pub quota_type: Option<String>,
    
    // Request metadata
    pub client_ip: Option<String>,
    pub user_agent: Option<String>,
    pub request_id: Option<String>,
    pub session_id: Option<String>,
    
    // Data context
    pub request_size_bytes: Option<i32>,
    pub response_size_bytes: Option<i32>,
    pub cache_hit: bool,
    
    // Billing
    pub billable: bool,
    pub cost_units: Option<f64>,
    
    // Timestamp
    pub timestamp: DateTime<Utc>,
    
    // Additional context
    pub request_metadata: serde_json::Value,
}

impl ModuleUsageLog {
    pub fn new(
        endpoint: String,
        request_method: String,
        sub_module_id: Option<Uuid>,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            user_id: None,
            api_key_id: None,
            sub_module_id,
            endpoint,
            request_method,
            response_status: 200,
            response_time_ms: None,
            quota_consumed: 1,
            quota_type: Some("api_calls".to_string()),
            client_ip: None,
            user_agent: None,
            request_id: None,
            session_id: None,
            request_size_bytes: None,
            response_size_bytes: None,
            cache_hit: false,
            billable: true,
            cost_units: None,
            timestamp: Utc::now(),
            request_metadata: serde_json::json!({}),
        }
    }

    pub fn for_user(mut self, user_id: UserId) -> Self {
        self.user_id = Some(user_id);
        self
    }

    pub fn for_api_key(mut self, api_key_id: Uuid) -> Self {
        self.api_key_id = Some(api_key_id);
        self
    }

    pub fn with_response(mut self, status: i32, time_ms: Option<i32>) -> Self {
        self.response_status = status;
        self.response_time_ms = time_ms;
        self
    }

    pub fn with_quota(mut self, consumed: i32, quota_type: String) -> Self {
        self.quota_consumed = consumed;
        self.quota_type = Some(quota_type);
        self
    }

    pub fn with_client_info(mut self, ip: Option<String>, user_agent: Option<String>) -> Self {
        self.client_ip = ip;
        self.user_agent = user_agent;
        self
    }
}