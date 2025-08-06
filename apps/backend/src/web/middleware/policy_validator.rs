// Policy validation and conflict detection for Casbin

use std::collections::{HashMap, HashSet};
use serde::{Serialize, Deserialize};

/// Policy validation errors
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PolicyValidationError {
    InvalidSubject(String),
    InvalidResource(String),
    InvalidAction(String),
    ConflictingPolicies {
        policy1: PolicyRule,
        policy2: PolicyRule,
        conflict_type: ConflictType,
    },
    CircularRoleInheritance {
        role_chain: Vec<String>,
    },
    DuplicatePolicy(PolicyRule),
    InvalidRoleHierarchy {
        parent_role: String,
        child_role: String,
        reason: String,
    },
}

/// Types of policy conflicts
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConflictType {
    AllowDenyConflict,
    RedundantPolicy,
    OverprivilegedRole,
}

/// Represents a policy rule for validation
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PolicyRule {
    pub subject: String,
    pub resource: String,
    pub action: String,
}

impl PolicyRule {
    pub fn new(subject: String, resource: String, action: String) -> Self {
        Self { subject, resource, action }
    }
    
    /// Check if this policy rule is more specific than another
    pub fn is_more_specific_than(&self, other: &PolicyRule) -> bool {
        // A policy is more specific if it has the same or more specific subject/resource/action
        let subject_match = self.subject == other.subject || other.subject == "*";
        let resource_match = self.resource == other.resource || other.resource == "*";
        let action_match = self.action == other.action || other.action == "*";
        
        subject_match && resource_match && action_match && 
        (self.subject != "*" || self.resource != "*" || self.action != "*")
    }
}

/// Role inheritance rule
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct RoleRule {
    pub user: String,
    pub role: String,
}

/// Policy validation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    pub is_valid: bool,
    pub errors: Vec<PolicyValidationError>,
    pub warnings: Vec<String>,
    pub suggestions: Vec<String>,
}

impl ValidationResult {
    pub fn new() -> Self {
        Self {
            is_valid: true,
            errors: Vec::new(),
            warnings: Vec::new(),
            suggestions: Vec::new(),
        }
    }
    
    pub fn add_error(&mut self, error: PolicyValidationError) {
        self.is_valid = false;
        self.errors.push(error);
    }
    
    pub fn add_warning(&mut self, warning: String) {
        self.warnings.push(warning);
    }
    
    pub fn add_suggestion(&mut self, suggestion: String) {
        self.suggestions.push(suggestion);
    }
}

/// Policy validator for Casbin rules
pub struct PolicyValidator {
    // EPSX-specific validation rules
    valid_resources: HashSet<String>,
    valid_actions: HashSet<String>,
    valid_roles: HashSet<String>,
}

impl PolicyValidator {
    pub fn new() -> Self {
        // Define valid resources and actions for EPSX trading platform
        let valid_resources = vec![
            "/api/v1/users".to_string(),
            "/api/v1/admin".to_string(),
            "/api/v1/iam".to_string(),
            "/api/v1/trading".to_string(),
            "/api/v1/analytics".to_string(),
            "/api/v1/premium".to_string(),
            "/api/v1/system".to_string(),
            "/api/v1/auth/profile".to_string(),
            "/api/v1/auth/logout".to_string(),
            "/api/v1/auth/refresh".to_string(),
            "modules".to_string(),
            "*".to_string(), // Wildcard allowed
        ].into_iter().collect();
        
        let valid_actions = vec![
            "GET".to_string(),
            "POST".to_string(),
            "PUT".to_string(),
            "DELETE".to_string(),
            "*".to_string(), // Wildcard allowed
        ].into_iter().collect();
        
        let valid_roles = vec![
            "basic_user".to_string(),
            "premium_user".to_string(),
            "moderator".to_string(),
            "admin".to_string(),
        ].into_iter().collect();
        
        Self {
            valid_resources,
            valid_actions,
            valid_roles,
        }
    }
    
    /// Validate a single policy rule
    pub fn validate_policy(&self, policy: &PolicyRule) -> ValidationResult {
        let mut result = ValidationResult::new();
        
        // Validate subject (can be user ID or role)
        if policy.subject.is_empty() {
            result.add_error(PolicyValidationError::InvalidSubject(
                "Subject cannot be empty".to_string()
            ));
        }
        
        // Validate resource
        if !self.valid_resources.contains(&policy.resource) && !self.is_valid_resource_pattern(&policy.resource) {
            result.add_error(PolicyValidationError::InvalidResource(
                format!("Resource '{}' is not recognized", policy.resource)
            ));
        }
        
        // Validate action
        if !self.valid_actions.contains(&policy.action) {
            result.add_error(PolicyValidationError::InvalidAction(
                format!("Action '{}' is not recognized", policy.action)
            ));
        }
        
        result
    }
    
    /// Validate multiple policies for conflicts
    pub fn validate_policies(&self, policies: &[PolicyRule]) -> ValidationResult {
        let mut result = ValidationResult::new();
        
        // Check each policy individually
        for policy in policies {
            let policy_result = self.validate_policy(policy);
            result.errors.extend(policy_result.errors);
            result.warnings.extend(policy_result.warnings);
            result.is_valid = result.is_valid && policy_result.is_valid;
        }
        
        // Check for duplicates
        let mut seen_policies = HashSet::new();
        for policy in policies {
            if !seen_policies.insert(policy.clone()) {
                result.add_error(PolicyValidationError::DuplicatePolicy(policy.clone()));
            }
        }
        
        // Check for conflicting policies
        self.check_policy_conflicts(policies, &mut result);
        
        result
    }
    
    /// Validate role inheritance rules
    pub fn validate_role_inheritance(&self, role_rules: &[RoleRule]) -> ValidationResult {
        let mut result = ValidationResult::new();
        
        // Build role inheritance graph
        let mut role_graph: HashMap<String, Vec<String>> = HashMap::new();
        
        for rule in role_rules {
            // Validate role exists
            if !self.valid_roles.contains(&rule.role) {
                result.add_warning(format!(
                    "Role '{}' is not in the predefined role list", 
                    rule.role
                ));
            }
            
            role_graph.entry(rule.user.clone())
                .or_insert_with(Vec::new)
                .push(rule.role.clone());
        }
        
        // Check for circular dependencies
        for (user, roles) in &role_graph {
            if let Some(cycle) = self.detect_role_cycle(user, roles, &role_graph) {
                result.add_error(PolicyValidationError::CircularRoleInheritance {
                    role_chain: cycle,
                });
            }
        }
        
        result
    }
    
    /// Check if a resource pattern is valid (e.g., /api/v1/users/*)
    fn is_valid_resource_pattern(&self, resource: &str) -> bool {
        // Allow wildcard patterns like /api/v1/users/*
        if resource.ends_with("/*") {
            let base = &resource[..resource.len() - 2];
            return self.valid_resources.contains(base);
        }
        
        // Allow specific resource IDs like /api/v1/users/123
        for valid_resource in &self.valid_resources {
            if valid_resource.ends_with("/*") {
                continue;
            }
            if resource.starts_with(valid_resource) && resource != *valid_resource {
                // Check if it's a valid ID pattern
                let suffix = &resource[valid_resource.len()..];
                if suffix.starts_with('/') && !suffix[1..].contains('/') {
                    return true;
                }
            }
        }
        
        false
    }
    
    /// Check for policy conflicts
    fn check_policy_conflicts(&self, policies: &[PolicyRule], result: &mut ValidationResult) {
        for (i, policy1) in policies.iter().enumerate() {
            for policy2 in policies.iter().skip(i + 1) {
                // Check if policies overlap in a conflicting way
                if self.policies_overlap(policy1, policy2) {
                    if self.is_redundant_policy(policy1, policy2) {
                        result.add_error(PolicyValidationError::ConflictingPolicies {
                            policy1: policy1.clone(),
                            policy2: policy2.clone(),
                            conflict_type: ConflictType::RedundantPolicy,
                        });
                    }
                }
            }
        }
    }
    
    /// Check if two policies overlap (same or similar scope)
    fn policies_overlap(&self, policy1: &PolicyRule, policy2: &PolicyRule) -> bool {
        let subject_overlap = policy1.subject == policy2.subject || 
                             policy1.subject == "*" || policy2.subject == "*";
        let resource_overlap = policy1.resource == policy2.resource ||
                              policy1.resource == "*" || policy2.resource == "*";
        let action_overlap = policy1.action == policy2.action ||
                            policy1.action == "*" || policy2.action == "*";
        
        subject_overlap && resource_overlap && action_overlap
    }
    
    /// Check if one policy is redundant given another
    fn is_redundant_policy(&self, policy1: &PolicyRule, policy2: &PolicyRule) -> bool {
        // A policy is redundant if another policy already covers it
        policy1.is_more_specific_than(policy2) || policy2.is_more_specific_than(policy1)
    }
    
    /// Detect circular role inheritance
    fn detect_role_cycle(
        &self, 
        _user: &str, 
        user_roles: &[String], 
        role_graph: &HashMap<String, Vec<String>>
    ) -> Option<Vec<String>> {
        // Simple cycle detection for role inheritance
        // In a more complex system, you'd check role-to-role inheritance
        
        let mut visited = HashSet::new();
        let mut path = Vec::new();
        
        for role in user_roles {
            if self.dfs_cycle_check(role, &mut visited, &mut path, role_graph) {
                return Some(path);
            }
        }
        
        None
    }
    
    /// Depth-first search for cycle detection
    fn dfs_cycle_check(
        &self,
        current: &str,
        visited: &mut HashSet<String>,
        path: &mut Vec<String>,
        role_graph: &HashMap<String, Vec<String>>,
    ) -> bool {
        if path.contains(&current.to_string()) {
            path.push(current.to_string());
            return true;
        }
        
        if visited.contains(current) {
            return false;
        }
        
        visited.insert(current.to_string());
        path.push(current.to_string());
        
        if let Some(children) = role_graph.get(current) {
            for child in children {
                if self.dfs_cycle_check(child, visited, path, role_graph) {
                    return true;
                }
            }
        }
        
        path.pop();
        false
    }
    
    /// Generate optimization suggestions for policies
    pub fn suggest_optimizations(&self, policies: &[PolicyRule]) -> Vec<String> {
        let mut suggestions = Vec::new();
        
        // Group policies by subject
        let mut subject_policies: HashMap<String, Vec<&PolicyRule>> = HashMap::new();
        for policy in policies {
            subject_policies.entry(policy.subject.clone())
                .or_insert_with(Vec::new)
                .push(policy);
        }
        
        // Suggest wildcard consolidation
        for (subject, subject_policy_list) in subject_policies {
            if subject_policy_list.len() > 3 {
                let resources: HashSet<String> = subject_policy_list.iter()
                    .map(|p| p.resource.clone())
                    .collect();
                
                if resources.len() > 2 {
                    suggestions.push(format!(
                        "Consider using wildcard resource '*' for subject '{}' which has {} different resource permissions",
                        subject, resources.len()
                    ));
                }
            }
        }
        
        suggestions
    }
}

impl Default for PolicyValidator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_policy_validation() {
        let validator = PolicyValidator::new();
        
        // Valid policy
        let valid_policy = PolicyRule::new(
            "user1".to_string(),
            "/api/v1/users".to_string(),
            "GET".to_string(),
        );
        let result = validator.validate_policy(&valid_policy);
        assert!(result.is_valid);
        
        // Invalid resource
        let invalid_policy = PolicyRule::new(
            "user1".to_string(),
            "/invalid/resource".to_string(),
            "GET".to_string(),
        );
        let result = validator.validate_policy(&invalid_policy);
        assert!(!result.is_valid);
        assert!(!result.errors.is_empty());
    }
    
    #[test]
    fn test_duplicate_policy_detection() {
        let validator = PolicyValidator::new();
        
        let policy1 = PolicyRule::new(
            "user1".to_string(),
            "/api/v1/users".to_string(),
            "GET".to_string(),
        );
        let policy2 = policy1.clone();
        
        let result = validator.validate_policies(&[policy1, policy2]);
        assert!(!result.is_valid);
        
        // Should have a duplicate policy error
        assert!(result.errors.iter().any(|e| matches!(e, PolicyValidationError::DuplicatePolicy(_))));
    }
    
    #[test]
    fn test_role_inheritance_validation() {
        let validator = PolicyValidator::new();
        
        let roles = vec![
            RoleRule { user: "user1".to_string(), role: "basic_user".to_string() },
            RoleRule { user: "user1".to_string(), role: "premium_user".to_string() },
        ];
        
        let result = validator.validate_role_inheritance(&roles);
        assert!(result.is_valid);
    }
    
    #[test]
    fn test_optimization_suggestions() {
        let validator = PolicyValidator::new();
        
        let policies = vec![
            PolicyRule::new("user1".to_string(), "/api/v1/users".to_string(), "GET".to_string()),
            PolicyRule::new("user1".to_string(), "/api/v1/admin".to_string(), "GET".to_string()),
            PolicyRule::new("user1".to_string(), "/api/v1/trading".to_string(), "GET".to_string()),
            PolicyRule::new("user1".to_string(), "/api/v1/analytics".to_string(), "GET".to_string()),
        ];
        
        let suggestions = validator.suggest_optimizations(&policies);
        assert!(!suggestions.is_empty());
    }
}