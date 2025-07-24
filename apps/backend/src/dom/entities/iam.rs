// IAM domain entities - roles, policies, permissions, and groups

use chrono::{DateTime, Utc};
use serde::{Serialize, Deserialize};
use std::collections::{HashMap, HashSet};

use crate::dom::values::{UserId, Role as UserRole};

/// Unique identifier for IAM roles
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct RoleId(uuid::Uuid);

/// Unique identifier for IAM policies  
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PolicyId(uuid::Uuid);

/// Unique identifier for IAM groups
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct GroupId(uuid::Uuid);

/// IAM Role entity - represents a collection of permissions and policies
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IamRole {
    id: RoleId,
    name: String,
    description: Option<String>,
    /// Package tier this role corresponds to (Bronze, Silver, Gold, Platinum, Admin)
    package_tier: PackageTier,
    /// AWS-style policies attached to this role
    policies: Vec<PolicyId>,
    /// Direct permissions granted to this role
    inline_permissions: Vec<Permission>,
    /// Whether this role can be assigned to users
    assignable: bool,
    /// Tags for organization and filtering
    tags: HashMap<String, String>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
    created_by: UserId,
}

/// IAM Policy entity - AWS-style policy document
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IamPolicy {
    id: PolicyId,
    name: String,
    description: Option<String>,
    /// AWS-style policy document
    policy_document: PolicyDocument,
    /// Policy type - managed or inline
    policy_type: PolicyType,
    /// Version for policy updates
    version: String,
    /// Whether this policy is active
    active: bool,
    /// Tags for organization
    tags: HashMap<String, String>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
    created_by: UserId,
}

/// IAM Group entity - collection of users with shared permissions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IamGroup {
    id: GroupId,
    name: String,
    description: Option<String>,
    /// Users in this group
    members: HashSet<UserId>,
    /// Policies attached to this group
    policies: Vec<PolicyId>,
    /// Direct permissions for this group
    inline_permissions: Vec<Permission>,
    /// Tags for organization
    tags: HashMap<String, String>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
    created_by: UserId,
}

/// User Permission Override - individual permissions granted to specific users
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserPermissionOverride {
    pub user_id: UserId,
    /// Package tier the user has purchased
    pub package_tier: PackageTier,
    /// Additional roles granted beyond package tier
    pub additional_roles: Vec<RoleId>,
    /// Direct permission grants
    pub permission_grants: Vec<Permission>,
    /// Permission denials (explicit deny)
    pub permission_denials: Vec<Permission>,
    /// Expiration for temporary permissions
    pub expires_at: Option<DateTime<Utc>>,
    /// Reason for the override
    pub reason: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub created_by: UserId,
}

impl UserPermissionOverride {
    pub fn new(user_id: UserId, granted_permissions: Vec<String>, denied_permissions: Vec<String>) -> Self {
        let now = Utc::now();
        let created_by = user_id.clone();
        Self {
            user_id,
            package_tier: PackageTier::Free,
            additional_roles: Vec::new(),
            permission_grants: granted_permissions.into_iter().map(|p| Permission::new(p, "*".to_string())).collect(),
            permission_denials: denied_permissions.into_iter().map(|p| Permission::new(p, "*".to_string())).collect(),
            expires_at: None,
            reason: None,
            created_at: now,
            updated_at: now,
            created_by,
        }
    }

    pub fn user_id(&self) -> &UserId {
        &self.user_id
    }

    pub fn granted_permissions(&self) -> Vec<String> {
        self.permission_grants.iter().map(|p| p.action().to_string()).collect()
    }

    pub fn denied_permissions(&self) -> Vec<String> {
        self.permission_denials.iter().map(|p| p.action().to_string()).collect()
    }
}

/// AWS-style policy document structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyDocument {
    /// Policy language version
    version: String,
    /// Policy statements
    statements: Vec<PolicyStatement>,
}

/// Individual policy statement
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyStatement {
    /// Statement ID (optional)
    sid: Option<String>,
    /// Allow or Deny
    effect: Effect,
    /// Actions this statement applies to
    actions: ActionSet,
    /// Resources this statement applies to
    resources: ResourceSet,
    /// Conditions for when this statement applies
    pub conditions: Option<Condition>,
    /// Principals this applies to (for resource-based policies)
    principals: Option<PrincipalSet>,
}

/// Permission effect - Allow or Deny
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Effect {
    Allow,
    Deny,
}

/// Set of actions in a policy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ActionSet {
    /// All actions (*)
    All,
    /// Specific actions
    Actions(Vec<String>),
    /// Actions with wildcards
    Patterns(Vec<String>),
}

/// Set of resources in a policy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ResourceSet {
    /// All resources (*)
    All,
    /// Specific resources
    Resources(Vec<String>),
    /// Resource patterns with wildcards
    Patterns(Vec<String>),
}

/// Condition block for policies
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Condition {
    /// Condition operators and values
    pub conditions: HashMap<String, HashMap<String, Vec<String>>>,
}

/// Principal set for resource-based policies
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PrincipalSet {
    /// All principals (*)
    All,
    /// Specific users
    Users(Vec<UserId>),
    /// Roles
    Roles(Vec<RoleId>),
    /// Groups  
    Groups(Vec<GroupId>),
}

/// Package tier enumeration
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PackageTier {
    Free,
    Bronze,
    Silver,
    Gold,
    Platinum,
    Admin,
    SuperAdmin,
}

/// Policy type enumeration
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PolicyType {
    /// AWS managed policy equivalent
    Managed,
    /// Inline policy attached directly
    Inline,
    /// System-defined policy
    System,
}

/// Individual permission
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Permission {
    /// Action (e.g., "users:read", "payments:write")
    action: String,
    /// Resource (e.g., "users/*", "payments/own")
    resource: String,
    /// Optional conditions
    conditions: Option<HashMap<String, String>>,
}

// Implementations




impl IamGroup {
    pub fn new(name: String, created_by: UserId) -> Self {
        let now = chrono::Utc::now();
        Self {
            id: GroupId::generate(),
            name,
            description: None,
            members: std::collections::HashSet::new(),
            policies: Vec::new(),
            inline_permissions: Vec::new(),
            tags: std::collections::HashMap::new(),
            created_at: now,
            updated_at: now,
            created_by,
        }
    }
    
    pub fn id(&self) -> &GroupId {
        &self.id
    }
    
    pub fn name(&self) -> &str {
        &self.name
    }
}

impl IamRole {
    pub fn new(
        name: String,
        package_tier: PackageTier,
        created_by: UserId,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: RoleId::generate(),
            name,
            description: None,
            package_tier,
            policies: Vec::new(),
            inline_permissions: Vec::new(),
            assignable: true,
            tags: HashMap::new(),
            created_at: now,
            updated_at: now,
            created_by,
        }
    }
    
    pub fn id(&self) -> &RoleId {
        &self.id
    }
    
    pub fn name(&self) -> &str {
        &self.name
    }
    
    pub fn package_tier(&self) -> &PackageTier {
        &self.package_tier
    }
    
    pub fn policies(&self) -> &[PolicyId] {
        &self.policies
    }
    
    pub fn inline_permissions(&self) -> &[Permission] {
        &self.inline_permissions
    }
    
    pub fn is_assignable(&self) -> bool {
        self.assignable
    }
    
    pub fn add_policy(&mut self, policy_id: PolicyId) {
        if !self.policies.contains(&policy_id) {
            self.policies.push(policy_id);
            self.updated_at = Utc::now();
        }
    }
    
    pub fn remove_policy(&mut self, policy_id: &PolicyId) {
        if let Some(pos) = self.policies.iter().position(|p| p == policy_id) {
            self.policies.remove(pos);
            self.updated_at = Utc::now();
        }
    }
    
    pub fn add_permission(&mut self, permission: Permission) {
        if !self.inline_permissions.contains(&permission) {
            self.inline_permissions.push(permission);
            self.updated_at = Utc::now();
        }
    }
    
    pub fn set_assignable(&mut self, assignable: bool) {
        if self.assignable != assignable {
            self.assignable = assignable;
            self.updated_at = Utc::now();
        }
    }
}

impl IamPolicy {
    pub fn new(
        name: String,
        policy_document: PolicyDocument,
        policy_type: PolicyType,
        created_by: UserId,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: PolicyId::generate(),
            name,
            description: None,
            policy_document,
            policy_type,
            version: "1.0".to_string(),
            active: true,
            tags: HashMap::new(),
            created_at: now,
            updated_at: now,
            created_by,
        }
    }
    
    pub fn id(&self) -> &PolicyId {
        &self.id
    }
    
    pub fn name(&self) -> &str {
        &self.name
    }
    
    pub fn policy_document(&self) -> &PolicyDocument {
        &self.policy_document
    }
    
    pub fn policy_type(&self) -> &PolicyType {
        &self.policy_type
    }
    
    pub fn version(&self) -> &str {
        &self.version
    }
    
    pub fn is_active(&self) -> bool {
        self.active
    }
    
    pub fn update_document(&mut self, document: PolicyDocument, version: String) {
        self.policy_document = document;
        self.version = version;
        self.updated_at = Utc::now();
    }
    
    pub fn activate(&mut self) {
        if !self.active {
            self.active = true;
            self.updated_at = Utc::now();
        }
    }
    
    pub fn deactivate(&mut self) {
        if self.active {
            self.active = false;
            self.updated_at = Utc::now();
        }
    }
}

impl PolicyDocument {
    pub fn new(statements: Vec<PolicyStatement>) -> Self {
        Self {
            version: "2012-10-17".to_string(),
            statements,
        }
    }
    
    pub fn statements(&self) -> &[PolicyStatement] {
        &self.statements
    }
    
    pub fn version(&self) -> &str {
        &self.version
    }
}

impl PolicyStatement {
    pub fn allow(actions: ActionSet, resources: ResourceSet) -> Self {
        Self {
            sid: None,
            effect: Effect::Allow,
            actions,
            resources,
            conditions: None,
            principals: None,
        }
    }
    
    pub fn deny(actions: ActionSet, resources: ResourceSet) -> Self {
        Self {
            sid: None,
            effect: Effect::Deny,
            actions,
            resources,
            conditions: None,
            principals: None,
        }
    }
    
    pub fn effect(&self) -> &Effect {
        &self.effect
    }
    
    pub fn actions(&self) -> &ActionSet {
        &self.actions
    }
    
    pub fn resources(&self) -> &ResourceSet {
        &self.resources
    }
    
    pub fn conditions(&self) -> Option<&Condition> {
        self.conditions.as_ref()
    }
}

impl PackageTier {
    pub fn hierarchy_level(&self) -> u8 {
        match self {
            PackageTier::Free => 0,
            PackageTier::Bronze => 1,
            PackageTier::Silver => 2,
            PackageTier::Gold => 3,
            PackageTier::Platinum => 4,
            PackageTier::Admin => 5,
            PackageTier::SuperAdmin => 6,
        }
    }
    
    pub fn can_access_tier(&self, required_tier: &PackageTier) -> bool {
        self.hierarchy_level() >= required_tier.hierarchy_level()
    }
    
    pub fn from_user_role(role: &UserRole) -> Self {
        match role {
            UserRole::Free => PackageTier::Free,
            UserRole::User => PackageTier::Free,
            UserRole::Premium => PackageTier::Bronze,
            UserRole::Moderator => PackageTier::Silver,
            UserRole::Admin => PackageTier::Admin,
            UserRole::SuperAdmin => PackageTier::SuperAdmin,
        }
    }
}

impl std::fmt::Display for PackageTier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PackageTier::Free => write!(f, "free"),
            PackageTier::Bronze => write!(f, "bronze"),
            PackageTier::Silver => write!(f, "silver"),
            PackageTier::Gold => write!(f, "gold"),
            PackageTier::Platinum => write!(f, "platinum"),
            PackageTier::Admin => write!(f, "admin"),
            PackageTier::SuperAdmin => write!(f, "super_admin"),
        }
    }
}

impl Permission {
    pub fn new(action: String, resource: String) -> Self {
        Self {
            action,
            resource,
            conditions: None,
        }
    }
    
    pub fn with_conditions(action: String, resource: String, conditions: HashMap<String, String>) -> Self {
        Self {
            action,
            resource,
            conditions: Some(conditions),
        }
    }
    
    pub fn action(&self) -> &str {
        &self.action
    }
    
    pub fn resource(&self) -> &str {
        &self.resource
    }
    
    pub fn conditions(&self) -> Option<&HashMap<String, String>> {
        self.conditions.as_ref()
    }
    
    pub fn matches_action(&self, action: &str) -> bool {
        if self.action == "*" {
            return true;
        }
        
        if self.action.contains('*') {
            // Simple wildcard matching
            let pattern = self.action.replace('*', ".*");
            regex::Regex::new(&pattern)
                .map(|re| re.is_match(action))
                .unwrap_or(false)
        } else {
            self.action == action
        }
    }
    
    pub fn matches_resource(&self, resource: &str) -> bool {
        if self.resource == "*" {
            return true;
        }
        
        if self.resource.contains('*') {
            // Simple wildcard matching
            let pattern = self.resource.replace('*', ".*");
            regex::Regex::new(&pattern)
                .map(|re| re.is_match(resource))
                .unwrap_or(false)
        } else {
            self.resource == resource
        }
    }
}

// String conversion traits
impl std::str::FromStr for PackageTier {
    type Err = IamError;
    
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "free" => Ok(PackageTier::Free),
            "bronze" => Ok(PackageTier::Bronze),
            "silver" => Ok(PackageTier::Silver),
            "gold" => Ok(PackageTier::Gold),
            "platinum" => Ok(PackageTier::Platinum),
            "admin" => Ok(PackageTier::Admin),
            "super_admin" | "superadmin" => Ok(PackageTier::SuperAdmin),
            _ => Err(IamError::InvalidPackageTier(s.to_string())),
        }
    }
}

// Error types
#[derive(Debug, thiserror::Error)]
pub enum IamError {
    #[error("Invalid package tier: {0}")]
    InvalidPackageTier(String),
    
    #[error("Policy evaluation failed: {0}")]
    PolicyEvaluationFailed(String),
    
    #[error("Permission denied for action {action} on resource {resource}")]
    PermissionDenied { action: String, resource: String },
    
    #[error("Role not found: {0}")]
    RoleNotFound(String),
    
    #[error("Policy not found: {0}")]
    PolicyNotFound(String),
    
    #[error("Group not found: {0}")]
    GroupNotFound(String),
    
    #[error("Invalid policy document: {0}")]
    InvalidPolicyDocument(String),
    
    #[error("Database error: {0}")]
    DatabaseError(String),
    
    #[error("Circular dependency detected in role hierarchy")]
    CircularDependency,
    
    #[error("Invalid data: {0}")]
    InvalidData(String),
    
    #[error("Serialization error: {0}")]
    SerializationError(String),
    
    #[error("Entity not found")]
    NotFound,
}

// ID type implementations
impl RoleId {
    pub fn new(id: String) -> Self {
        Self(uuid::Uuid::parse_str(&id).unwrap_or_else(|_| uuid::Uuid::new_v4()))
    }
    
    pub fn generate() -> Self {
        Self(uuid::Uuid::new_v4())
    }
    
    pub fn from(id: uuid::Uuid) -> Self {
        Self(id)
    }
    
    pub fn value(&self) -> &uuid::Uuid {
        &self.0
    }
}

impl std::fmt::Display for RoleId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl PolicyId {
    pub fn new(id: String) -> Self {
        Self(uuid::Uuid::parse_str(&id).unwrap_or_else(|_| uuid::Uuid::new_v4()))
    }
    
    pub fn generate() -> Self {
        Self(uuid::Uuid::new_v4())
    }
    
    pub fn from(id: uuid::Uuid) -> Self {
        Self(id)
    }
    
    pub fn value(&self) -> &uuid::Uuid {
        &self.0
    }
}

impl std::fmt::Display for PolicyId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl GroupId {
    pub fn new(id: String) -> Self {
        Self(uuid::Uuid::parse_str(&id).unwrap_or_else(|_| uuid::Uuid::new_v4()))
    }
    
    pub fn generate() -> Self {
        Self(uuid::Uuid::new_v4())
    }
    
    pub fn from(id: uuid::Uuid) -> Self {
        Self(id)
    }
    
    pub fn value(&self) -> &uuid::Uuid {
        &self.0
    }
}

impl std::fmt::Display for GroupId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn should_create_iam_role() {
        let creator_id = UserId::new("creator123".to_string());
        let role = IamRole::new(
            "TestRole".to_string(),
            PackageTier::Bronze,
            creator_id
        );
        
        assert_eq!(role.name(), "TestRole");
        assert_eq!(role.package_tier(), &PackageTier::Bronze);
        assert!(role.is_assignable());
        assert!(role.policies().is_empty());
    }
    
    #[test]
    fn should_create_policy_document() {
        let statement = PolicyStatement::allow(
            ActionSet::Actions(vec!["users:read".to_string()]),
            ResourceSet::All
        );
        
        let doc = PolicyDocument::new(vec![statement]);
        assert_eq!(doc.statements().len(), 1);
        assert_eq!(doc.version(), "2012-10-17");
    }
    
    #[test]
    fn should_match_permission_wildcards() {
        let perm = Permission::new("users:*".to_string(), "users/*".to_string());
        
        assert!(perm.matches_action("users:read"));
        assert!(perm.matches_action("users:write"));
        assert!(!perm.matches_action("payments:read"));
        
        assert!(perm.matches_resource("users/123"));
        assert!(perm.matches_resource("users/456"));
        assert!(!perm.matches_resource("payments/123"));
    }
    
    #[test]
    fn should_compare_package_tiers() {
        assert!(PackageTier::Gold.can_access_tier(&PackageTier::Bronze));
        assert!(PackageTier::Admin.can_access_tier(&PackageTier::Platinum));
        assert!(!PackageTier::Bronze.can_access_tier(&PackageTier::Gold));
    }
}