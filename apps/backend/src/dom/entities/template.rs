// IAM Role Template domain entities for quick role setup and management

use chrono::{DateTime, Utc};
use serde::{Serialize, Deserialize};
use std::collections::HashMap;

use crate::dom::entities::iam::{PackageTier, Permission, PolicyId};
use crate::dom::values::UserId;

/// Unique identifier for IAM role templates
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TemplateId(String);

/// IAM Role Template - predefined role configurations for common use cases
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoleTemplate {
    /// Unique identifier
    id: TemplateId,
    /// Template name (e.g., "Bronze User", "Content Moderator", "Admin Assistant")
    name: String,
    /// Description of what this template provides
    description: String,
    /// Package tier this template is designed for
    target_tier: PackageTier,
    /// Category for organization (e.g., "user", "moderator", "admin")
    category: TemplateCategory,
    /// Whether this template is available for use
    active: bool,
    /// Predefined permissions included in this template
    default_permissions: Vec<Permission>,
    /// AWS-style policies to attach
    policy_attachments: Vec<PolicyId>,
    /// Tags for searchability and organization
    tags: Vec<String>,
    /// Configuration metadata
    metadata: TemplateMetadata,
    /// Template creation info
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
    created_by: UserId,
    version: String,
}

/// Template categories for organization
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TemplateCategory {
    /// Standard user roles
    User,
    /// Moderator and content management roles
    Moderator,
    /// Administrative roles
    Admin,
    /// Custom business-specific roles
    Custom,
    /// System and service roles
    System,
    /// Business-specific roles
    Business,
    /// Technical roles
    Technical,
    /// Administrative roles (alias)
    Administrative,
    /// Compliance-related roles
    Compliance,
}

/// Template metadata for additional configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateMetadata {
    /// Prerequisites (required tiers, conditions)
    pub prerequisites: Vec<String>,
    /// Warnings or notices for role assignment
    pub warnings: Vec<String>,
    /// Recommended use cases
    pub use_cases: Vec<String>,
    /// Maximum number of users that should have this role
    pub max_assignments: Option<u32>,
    /// Whether this role requires approval for assignment
    pub requires_approval: bool,
    /// Auto-expiration settings
    pub auto_expire_days: Option<u32>,
    /// Additional custom metadata
    pub custom_fields: HashMap<String, String>,
}

/// Template search query
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateQuery {
    /// Filter by name (partial match)
    pub name: Option<String>,
    /// Filter by category
    pub category: Option<TemplateCategory>,
    /// Filter by target tier
    pub target_tier: Option<PackageTier>,
    /// Filter by tags (any of these tags)
    pub tags: Vec<String>,
    /// Only active templates
    pub active_only: bool,
    /// Pagination
    pub limit: Option<u32>,
    pub offset: Option<u32>,
}

/// Template application request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApplyTemplateRequest {
    /// Template to apply
    pub template_id: TemplateId,
    /// Target users
    pub user_ids: Vec<UserId>,
    /// Override default permissions if needed
    pub permission_overrides: Option<Vec<Permission>>,
    /// Reason for applying template
    pub reason: Option<String>,
    /// Whether to merge with existing permissions or replace
    pub merge_permissions: bool,
    /// Auto-expiration for this application
    pub expires_at: Option<DateTime<Utc>>,
    /// Who is applying the template
    pub applied_by: UserId,
}

impl ApplyTemplateRequest {
    pub fn template_id(&self) -> &TemplateId {
        &self.template_id
    }
    
    pub fn user_ids(&self) -> &[UserId] {
        &self.user_ids
    }
    
    pub fn applied_by(&self) -> &UserId {
        &self.applied_by
    }
}

/// Result of template application
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApplyTemplateResult {
    /// Request that was processed
    pub request: ApplyTemplateRequest,
    /// Successfully processed users
    pub successful_users: Vec<UserId>,
    /// Failed users with reasons
    pub failed_users: Vec<(UserId, String)>,
    /// Summary of changes made
    pub changes_summary: Vec<String>,
    /// Timestamp of application
    pub applied_at: DateTime<Utc>,
    /// Who applied the template
    pub applied_by: UserId,
}

impl ApplyTemplateResult {
    pub fn new(
        request: ApplyTemplateRequest,
        successful_users: Vec<UserId>,
        failed_users: Vec<(UserId, String)>,
        changes_summary: Vec<String>,
        applied_by: UserId,
    ) -> Self {
        Self {
            request,
            successful_users,
            failed_users,
            changes_summary,
            applied_at: Utc::now(),
            applied_by,
        }
    }
}

/// Default role templates for common scenarios
pub struct DefaultTemplates;

// Implementations

impl TemplateId {
    pub fn new(id: String) -> Self {
        Self(id)
    }
    
    pub fn value(&self) -> &str {
        &self.0
    }
    
    pub fn generate() -> Self {
        Self(uuid::Uuid::new_v4().to_string())
    }
}

impl From<uuid::Uuid> for TemplateId {
    fn from(uuid: uuid::Uuid) -> Self {
        Self(uuid.to_string())
    }
}

impl From<String> for TemplateId {
    fn from(s: String) -> Self {
        Self(s)
    }
}

impl RoleTemplate {
    pub fn new(
        name: String,
        description: String,
        target_tier: PackageTier,
        category: TemplateCategory,
        created_by: UserId,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: TemplateId::generate(),
            name,
            description,
            target_tier,
            category,
            active: true,
            default_permissions: Vec::new(),
            policy_attachments: Vec::new(),
            tags: Vec::new(),
            metadata: TemplateMetadata::default(),
            created_at: now,
            updated_at: now,
            created_by,
            version: "1.0".to_string(),
        }
    }
    
    // Constructor for database loading with all fields
    pub fn from_db(
        id: TemplateId,
        name: String,
        description: String,
        target_tier: PackageTier,
        category: TemplateCategory,
        active: bool,
        permissions: Vec<Permission>,
        policy_attachments: Vec<PolicyId>,
        created_by: UserId,
        created_at: DateTime<Utc>,
        updated_at: DateTime<Utc>,
    ) -> Self {
        Self {
            id,
            name,
            description,
            target_tier,
            category,
            active,
            default_permissions: permissions,
            policy_attachments,
            tags: Vec::new(),
            metadata: TemplateMetadata::default(),
            created_at,
            updated_at,
            created_by,
            version: "1.0".to_string(),
        }
    }
    
    // Getters
    pub fn id(&self) -> &TemplateId {
        &self.id
    }
    
    pub fn name(&self) -> &str {
        &self.name
    }
    
    pub fn description(&self) -> &str {
        &self.description
    }
    
    pub fn target_tier(&self) -> &PackageTier {
        &self.target_tier
    }
    
    pub fn category(&self) -> &TemplateCategory {
        &self.category
    }
    
    pub fn is_active(&self) -> bool {
        self.active
    }
    
    pub fn default_permissions(&self) -> &[Permission] {
        &self.default_permissions
    }
    
    pub fn policy_attachments(&self) -> &[PolicyId] {
        &self.policy_attachments
    }
    
    pub fn tags(&self) -> &[String] {
        &self.tags
    }
    
    pub fn metadata(&self) -> &TemplateMetadata {
        &self.metadata
    }
    
    pub fn version(&self) -> &str {
        &self.version
    }
    
    pub fn created_at(&self) -> &DateTime<Utc> {
        &self.created_at
    }
    
    pub fn updated_at(&self) -> &DateTime<Utc> {
        &self.updated_at
    }
    
    pub fn created_by(&self) -> &UserId {
        &self.created_by
    }
    
    // Setters
    pub fn set_name(&mut self, name: String) {
        self.name = name;
        self.updated_at = Utc::now();
    }
    
    pub fn set_description(&mut self, description: String) {
        self.description = description;
        self.updated_at = Utc::now();
    }
    
    pub fn set_active(&mut self, active: bool) {
        self.active = active;
        self.updated_at = Utc::now();
    }
    
    pub fn add_permission(&mut self, permission: Permission) {
        if !self.default_permissions.contains(&permission) {
            self.default_permissions.push(permission);
            self.updated_at = Utc::now();
        }
    }
    
    pub fn remove_permission(&mut self, permission: &Permission) {
        if let Some(pos) = self.default_permissions.iter().position(|p| p == permission) {
            self.default_permissions.remove(pos);
            self.updated_at = Utc::now();
        }
    }
    
    pub fn add_policy(&mut self, policy_id: PolicyId) {
        if !self.policy_attachments.contains(&policy_id) {
            self.policy_attachments.push(policy_id);
            self.updated_at = Utc::now();
        }
    }
    
    pub fn remove_policy(&mut self, policy_id: &PolicyId) {
        if let Some(pos) = self.policy_attachments.iter().position(|p| p == policy_id) {
            self.policy_attachments.remove(pos);
            self.updated_at = Utc::now();
        }
    }
    
    pub fn add_tag(&mut self, tag: String) {
        if !self.tags.contains(&tag) {
            self.tags.push(tag);
            self.updated_at = Utc::now();
        }
    }
    
    pub fn remove_tag(&mut self, tag: &str) {
        if let Some(pos) = self.tags.iter().position(|t| t == tag) {
            self.tags.remove(pos);
            self.updated_at = Utc::now();
        }
    }
    
    pub fn update_metadata(&mut self, metadata: TemplateMetadata) {
        self.metadata = metadata;
        self.updated_at = Utc::now();
    }
    
    pub fn increment_version(&mut self) {
        // Simple version increment (1.0 -> 1.1 -> 1.2, etc.)
        if let Some(dot_pos) = self.version.rfind('.') {
            let major = &self.version[..dot_pos];
            if let Ok(minor) = self.version[dot_pos + 1..].parse::<u32>() {
                self.version = format!("{}.{}", major, minor + 1);
            }
        }
        self.updated_at = Utc::now();
    }
}

impl TemplateMetadata {
    pub fn empty() -> Self {
        Self {
            prerequisites: Vec::new(),
            warnings: Vec::new(),
            use_cases: Vec::new(),
            max_assignments: None,
            requires_approval: false,
            auto_expire_days: None,
            custom_fields: HashMap::new(),
        }
    }
    
    pub fn with_prerequisites(mut self, prerequisites: Vec<String>) -> Self {
        self.prerequisites = prerequisites;
        self
    }
    
    pub fn with_warnings(mut self, warnings: Vec<String>) -> Self {
        self.warnings = warnings;
        self
    }
    
    pub fn with_use_cases(mut self, use_cases: Vec<String>) -> Self {
        self.use_cases = use_cases;
        self
    }
    
    pub fn with_max_assignments(mut self, max: u32) -> Self {
        self.max_assignments = Some(max);
        self
    }
    
    pub fn requires_approval(mut self) -> Self {
        self.requires_approval = true;
        self
    }
    
    pub fn with_auto_expire(mut self, days: u32) -> Self {
        self.auto_expire_days = Some(days);
        self
    }
    
    pub fn add_custom_field(mut self, key: String, value: String) -> Self {
        self.custom_fields.insert(key, value);
        self
    }
}

impl Default for TemplateMetadata {
    fn default() -> Self {
        Self::empty()
    }
}

impl TemplateQuery {
    pub fn new() -> Self {
        Self {
            name: None,
            category: None,
            target_tier: None,
            tags: Vec::new(),
            active_only: true,
            limit: Some(50),
            offset: Some(0),
        }
    }
    
    pub fn by_name(mut self, name: String) -> Self {
        self.name = Some(name);
        self
    }
    
    pub fn by_category(mut self, category: TemplateCategory) -> Self {
        self.category = Some(category);
        self
    }
    
    pub fn by_tier(mut self, tier: PackageTier) -> Self {
        self.target_tier = Some(tier);
        self
    }
    
    pub fn with_tags(mut self, tags: Vec<String>) -> Self {
        self.tags = tags;
        self
    }
    
    pub fn include_inactive(mut self) -> Self {
        self.active_only = false;
        self
    }
    
    pub fn with_pagination(mut self, limit: u32, offset: u32) -> Self {
        self.limit = Some(limit);
        self.offset = Some(offset);
        self
    }
    
    pub fn is_active(&self) -> Option<bool> {
        Some(self.active_only)
    }
    
    // Getter methods for repository access
    pub fn name(&self) -> Option<&String> {
        self.name.as_ref()
    }
    
    pub fn category(&self) -> Option<&TemplateCategory> {
        self.category.as_ref()
    }
    
    pub fn limit(&self) -> Option<u32> {
        self.limit
    }
    
    pub fn offset(&self) -> Option<u32> {
        self.offset
    }
}

impl Default for TemplateQuery {
    fn default() -> Self {
        Self::new()
    }
}

impl DefaultTemplates {
    /// Create default Bronze user template
    pub fn bronze_user(created_by: UserId) -> RoleTemplate {
        let mut template = RoleTemplate::new(
            "Bronze User".to_string(),
            "Basic user with bronze tier access to platform features".to_string(),
            PackageTier::Bronze,
            TemplateCategory::User,
            created_by,
        );
        
        // Add bronze user permissions
        template.add_permission(Permission::new("dashboard:read".to_string(), "*".to_string()));
        template.add_permission(Permission::new("profile:read".to_string(), "own".to_string()));
        template.add_permission(Permission::new("profile:write".to_string(), "own".to_string()));
        template.add_permission(Permission::new("stocks:read".to_string(), "basic".to_string()));
        
        template.add_tag("bronze".to_string());
        template.add_tag("user".to_string());
        template.add_tag("standard".to_string());
        
        template.update_metadata(
            TemplateMetadata::empty()
                .with_use_cases(vec![
                    "Basic platform access".to_string(),
                    "Bronze tier subscribers".to_string(),
                    "Standard user accounts".to_string(),
                ])
                .with_max_assignments(10000)
        );
        
        template
    }
    
    /// Create default Silver user template
    pub fn silver_user(created_by: UserId) -> RoleTemplate {
        let mut template = RoleTemplate::new(
            "Silver User".to_string(),
            "Enhanced user with silver tier access and additional features".to_string(),
            PackageTier::Silver,
            TemplateCategory::User,
            created_by,
        );
        
        // Add silver user permissions (inherits bronze + additional)
        template.add_permission(Permission::new("dashboard:read".to_string(), "*".to_string()));
        template.add_permission(Permission::new("profile:read".to_string(), "own".to_string()));
        template.add_permission(Permission::new("profile:write".to_string(), "own".to_string()));
        template.add_permission(Permission::new("stocks:read".to_string(), "detailed".to_string()));
        template.add_permission(Permission::new("analytics:read".to_string(), "basic".to_string()));
        template.add_permission(Permission::new("notifications:manage".to_string(), "own".to_string()));
        
        template.add_tag("silver".to_string());
        template.add_tag("user".to_string());
        template.add_tag("enhanced".to_string());
        
        template.update_metadata(
            TemplateMetadata::empty()
                .with_use_cases(vec![
                    "Enhanced platform access".to_string(),
                    "Silver tier subscribers".to_string(),
                    "Users with analytics needs".to_string(),
                ])
                .with_max_assignments(5000)
        );
        
        template
    }
    
    /// Create default Gold user template
    pub fn gold_user(created_by: UserId) -> RoleTemplate {
        let mut template = RoleTemplate::new(
            "Gold User".to_string(),
            "Premium user with gold tier access to advanced features".to_string(),
            PackageTier::Gold,
            TemplateCategory::User,
            created_by,
        );
        
        // Add gold user permissions
        template.add_permission(Permission::new("dashboard:read".to_string(), "*".to_string()));
        template.add_permission(Permission::new("profile:read".to_string(), "own".to_string()));
        template.add_permission(Permission::new("profile:write".to_string(), "own".to_string()));
        template.add_permission(Permission::new("stocks:read".to_string(), "premium".to_string()));
        template.add_permission(Permission::new("analytics:read".to_string(), "advanced".to_string()));
        template.add_permission(Permission::new("notifications:manage".to_string(), "own".to_string()));
        template.add_permission(Permission::new("trading:execute".to_string(), "own".to_string()));
        template.add_permission(Permission::new("reports:generate".to_string(), "own".to_string()));
        
        template.add_tag("gold".to_string());
        template.add_tag("user".to_string());
        template.add_tag("premium".to_string());
        template.add_tag("trading".to_string());
        
        template.update_metadata(
            TemplateMetadata::empty()
                .with_use_cases(vec![
                    "Premium platform access".to_string(),
                    "Gold tier subscribers".to_string(),
                    "Active traders".to_string(),
                    "Users requiring advanced analytics".to_string(),
                ])
                .with_max_assignments(1000)
        );
        
        template
    }
    
    /// Create default Content Moderator template
    pub fn content_moderator(created_by: UserId) -> RoleTemplate {
        let mut template = RoleTemplate::new(
            "Content Moderator".to_string(),
            "Moderator role with permissions to manage user content and basic moderation".to_string(),
            PackageTier::Silver,
            TemplateCategory::Moderator,
            created_by,
        );
        
        // Add moderator permissions
        template.add_permission(Permission::new("content:read".to_string(), "*".to_string()));
        template.add_permission(Permission::new("content:moderate".to_string(), "*".to_string()));
        template.add_permission(Permission::new("users:read".to_string(), "basic".to_string()));
        template.add_permission(Permission::new("reports:read".to_string(), "content".to_string()));
        template.add_permission(Permission::new("moderation:actions".to_string(), "content".to_string()));
        
        template.add_tag("moderator".to_string());
        template.add_tag("content".to_string());
        template.add_tag("moderation".to_string());
        
        template.update_metadata(
            TemplateMetadata::empty()
                .with_use_cases(vec![
                    "Content moderation tasks".to_string(),
                    "Community management".to_string(),
                    "Basic user oversight".to_string(),
                ])
                .with_max_assignments(10)
                .requires_approval()
        );
        
        template
    }
    
    /// Create default Admin Assistant template
    pub fn admin_assistant(created_by: UserId) -> RoleTemplate {
        let mut template = RoleTemplate::new(
            "Admin Assistant".to_string(),
            "Administrative assistant with limited admin permissions for routine tasks".to_string(),
            PackageTier::Admin,
            TemplateCategory::Admin,
            created_by,
        );
        
        // Add admin assistant permissions
        template.add_permission(Permission::new("users:read".to_string(), "*".to_string()));
        template.add_permission(Permission::new("users:write".to_string(), "basic".to_string()));
        template.add_permission(Permission::new("reports:read".to_string(), "*".to_string()));
        template.add_permission(Permission::new("reports:generate".to_string(), "standard".to_string()));
        template.add_permission(Permission::new("support:manage".to_string(), "*".to_string()));
        template.add_permission(Permission::new("notifications:send".to_string(), "users".to_string()));
        
        template.add_tag("admin".to_string());
        template.add_tag("assistant".to_string());
        template.add_tag("support".to_string());
        template.add_tag("limited-admin".to_string());
        
        template.update_metadata(
            TemplateMetadata::empty()
                .with_use_cases(vec![
                    "Administrative support tasks".to_string(),
                    "User support management".to_string(),
                    "Report generation".to_string(),
                    "Limited user management".to_string(),
                ])
                .with_warnings(vec![
                    "This role has administrative privileges".to_string(),
                    "Requires careful user selection".to_string(),
                ])
                .with_max_assignments(5)
                .requires_approval()
        );
        
        template
    }
    
    /// Get all default templates
    pub fn all_default_templates(created_by: UserId) -> Vec<RoleTemplate> {
        vec![
            Self::bronze_user(created_by.clone()),
            Self::silver_user(created_by.clone()),
            Self::gold_user(created_by.clone()),
            Self::content_moderator(created_by.clone()),
            Self::admin_assistant(created_by),
        ]
    }
}

// Display implementations
impl std::fmt::Display for TemplateCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TemplateCategory::User => write!(f, "user"),
            TemplateCategory::Moderator => write!(f, "moderator"),
            TemplateCategory::Admin => write!(f, "admin"),
            TemplateCategory::Custom => write!(f, "custom"),
            TemplateCategory::System => write!(f, "system"),
            TemplateCategory::Business => write!(f, "business"),
            TemplateCategory::Technical => write!(f, "technical"),
            TemplateCategory::Administrative => write!(f, "administrative"),
            TemplateCategory::Compliance => write!(f, "compliance"),
        }
    }
}

// Error types
#[derive(Debug, thiserror::Error)]
pub enum TemplateError {
    #[error("Template not found: {0}")]
    NotFoundWithMessage(String),
    
    #[error("Template not found")]
    NotFound,
    
    #[error("Template is not active")]
    Inactive,
    
    #[error("Prerequisites not met: {0}")]
    PrerequisitesNotMet(String),
    
    #[error("Maximum assignments exceeded: {current}/{max}")]
    MaxAssignmentsExceeded { current: u32, max: u32 },
    
    #[error("Template requires approval")]
    RequiresApproval,
    
    #[error("Invalid template configuration: {0}")]
    InvalidConfiguration(String),
    
    #[error("Permission denied for template operation")]
    PermissionDenied,
    
    #[error("Database error: {0}")]
    DatabaseError(String),
    
    #[error("Invalid data: {0}")]
    InvalidData(String),
    
    #[error("Serialization error: {0}")]
    SerializationError(String),
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn should_create_role_template() {
        let creator_id = UserId::new("admin123".to_string());
        let template = RoleTemplate::new(
            "Test Template".to_string(),
            "Test description".to_string(),
            PackageTier::Bronze,
            TemplateCategory::User,
            creator_id.clone(),
        );
        
        assert_eq!(template.name(), "Test Template");
        assert_eq!(template.target_tier(), &PackageTier::Bronze);
        assert_eq!(template.category(), &TemplateCategory::User);
        assert_eq!(template.created_by(), &creator_id);
        assert!(template.is_active());
    }
    
    #[test]
    fn should_manage_template_permissions() {
        let creator_id = UserId::new("admin123".to_string());
        let mut template = RoleTemplate::new(
            "Test Template".to_string(),
            "Test description".to_string(),
            PackageTier::Bronze,
            TemplateCategory::User,
            creator_id,
        );
        
        let permission = Permission::new("test:read".to_string(), "*".to_string());
        template.add_permission(permission.clone());
        
        assert_eq!(template.default_permissions().len(), 1);
        assert!(template.default_permissions().contains(&permission));
        
        template.remove_permission(&permission);
        assert_eq!(template.default_permissions().len(), 0);
    }
    
    #[test]
    fn should_create_bronze_user_template() {
        let creator_id = UserId::new("admin123".to_string());
        let template = DefaultTemplates::bronze_user(creator_id);
        
        assert_eq!(template.name(), "Bronze User");
        assert_eq!(template.target_tier(), &PackageTier::Bronze);
        assert_eq!(template.category(), &TemplateCategory::User);
        assert!(template.tags().contains(&"bronze".to_string()));
        assert!(template.default_permissions().len() > 0);
    }
    
    #[test]
    fn should_create_template_query() {
        let query = TemplateQuery::new()
            .by_category(TemplateCategory::User)
            .by_tier(PackageTier::Bronze)
            .with_pagination(10, 0);
        
        assert_eq!(query.category, Some(TemplateCategory::User));
        assert_eq!(query.target_tier, Some(PackageTier::Bronze));
        assert_eq!(query.limit, Some(10));
        assert_eq!(query.offset, Some(0));
        assert!(query.active_only);
    }
    
    #[test]
    fn should_increment_template_version() {
        let creator_id = UserId::new("admin123".to_string());
        let mut template = RoleTemplate::new(
            "Test Template".to_string(),
            "Test description".to_string(),
            PackageTier::Bronze,
            TemplateCategory::User,
            creator_id,
        );
        
        assert_eq!(template.version(), "1.0");
        
        template.increment_version();
        assert_eq!(template.version(), "1.1");
        
        template.increment_version();
        assert_eq!(template.version(), "1.2");
    }
}