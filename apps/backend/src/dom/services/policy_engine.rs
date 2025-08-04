// Policy evaluation engine - AWS-style IAM policy evaluation

use std::collections::{HashMap, HashSet};
use chrono::{DateTime, Utc, Timelike, Duration};

use crate::dom::entities::iam::{
    IamPolicy, IamRole, UserPermissionOverride, PolicyStatement,
    Effect, ActionSet, ResourceSet, Condition, Permission, PackageTier, IamError
};
use crate::dom::values::UserId;

/// Context for policy evaluation
#[derive(Debug, Clone)]
pub struct EvaluationContext {
    /// User being evaluated
    pub user_id: UserId,
    /// Action being attempted
    pub action: String,
    /// Resource being accessed
    pub resource: String,
    /// Request time
    pub request_time: DateTime<Utc>,
    /// Additional context variables
    pub context_vars: HashMap<String, String>,
    /// Source IP address
    pub source_ip: Option<String>,
    /// User agent
    pub user_agent: Option<String>,
}

/// Result of policy evaluation
#[derive(Debug, Clone, PartialEq)]
pub enum EvaluationResult {
    /// Explicitly allowed
    Allow,
    /// Explicitly denied
    Deny,
    /// No matching policy (default deny)
    NoMatch,
}

/// Detailed evaluation decision with reasoning
#[derive(Debug, Clone)]
pub struct EvaluationDecision {
    pub result: EvaluationResult,
    pub reasons: Vec<String>,
    pub matching_policies: Vec<String>,
    pub package_tier_access: bool,
    pub explicit_permissions: Vec<String>,
}

/// Permission check result for caching
#[derive(Debug, Clone)]
struct PermissionCheckResult {
    result: EvaluationResult,
    cached_at: DateTime<Utc>,
}

/// Main policy evaluation engine
pub struct PolicyEngine {
    /// Cache for compiled policy evaluators
    policy_cache: HashMap<String, CompiledPolicy>,
    /// Cache for permission check results
    permission_check_cache: HashMap<String, PermissionCheckResult>,
    /// Cache for wildcard match results
    wildcard_match_cache: HashMap<String, (bool, DateTime<Utc>)>,
    /// Cache configuration
    cache_config: CacheConfig,
}

/// Cache configuration for PolicyEngine
#[derive(Debug, Clone)]
pub struct CacheConfig {
    /// TTL for compiled policies (30 minutes)
    policy_ttl_minutes: i64,
    /// TTL for permission check results (2 minutes)
    permission_check_ttl_minutes: i64,
    /// TTL for wildcard match results (10 minutes)
    wildcard_match_ttl_minutes: i64,
    /// Maximum cache entries before cleanup
    max_cache_entries: usize,
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            policy_ttl_minutes: 30,
            permission_check_ttl_minutes: 2,
            wildcard_match_ttl_minutes: 10,
            max_cache_entries: 10000,
        }
    }
}

/// Cache statistics for monitoring
#[derive(Debug, Clone)]
pub struct CacheStats {
    pub policy_cache_entries: usize,
    pub permission_check_cache_entries: usize,
    pub wildcard_match_cache_entries: usize,
    pub total_cache_entries: usize,
    pub max_cache_entries: usize,
}

/// Compiled policy for faster evaluation
#[derive(Debug, Clone)]
#[allow(dead_code)]
struct CompiledPolicy {
    policy_id: String,
    allow_patterns: Vec<CompiledStatement>,
    deny_patterns: Vec<CompiledStatement>,
    last_compiled: DateTime<Utc>,
}

/// Compiled policy statement for pattern matching
#[derive(Debug, Clone)]
struct CompiledStatement {
    actions: ActionMatcher,
    resources: ResourceMatcher,
    conditions: Option<ConditionMatcher>,
    effect: Effect,
}

/// Optimized action matcher
#[derive(Debug, Clone)]
enum ActionMatcher {
    All,
    Exact(HashSet<String>),
    Patterns(Vec<regex::Regex>),
}

/// Optimized resource matcher
#[derive(Debug, Clone)]
enum ResourceMatcher {
    All,
    Exact(HashSet<String>),
    Patterns(Vec<regex::Regex>),
}

/// Condition evaluation matcher
#[derive(Debug, Clone)]
struct ConditionMatcher {
    conditions: HashMap<String, ConditionOperator>,
}

/// Supported condition operators
#[derive(Debug, Clone)]
#[allow(dead_code)]
enum ConditionOperator {
    StringEquals(HashSet<String>),
    StringLike(Vec<regex::Regex>),
    IpAddress(Vec<ipnetwork::IpNetwork>),
    DateLessThan(DateTime<Utc>),
    DateGreaterThan(DateTime<Utc>),
    NumericLessThan(f64),
    NumericGreaterThan(f64),
    Bool(bool),
}

impl PolicyEngine {
    pub fn new() -> Self {
        Self {
            policy_cache: HashMap::new(),
            permission_check_cache: HashMap::new(),
            wildcard_match_cache: HashMap::new(),
            cache_config: CacheConfig::default(),
        }
    }

    pub fn with_cache_config(cache_config: CacheConfig) -> Self {
        Self {
            policy_cache: HashMap::new(),
            permission_check_cache: HashMap::new(),
            wildcard_match_cache: HashMap::new(),
            cache_config,
        }
    }

    /// Main evaluation entry point (async for compatibility with permission resolver)
    pub async fn evaluate(&self, _context: &EvaluationContext) -> Result<EvaluationDecision, IamError> {
        // TODO: Implement async evaluation using context to fetch user data
        // For now, return a simple evaluation based on context
        Ok(EvaluationDecision {
            result: EvaluationResult::Allow,
            reasons: vec!["Simple evaluation - allow all".to_string()],
            matching_policies: vec![],
            package_tier_access: true,
            explicit_permissions: vec![],
        })
    }
    
    /// Main evaluation entry point
    pub fn evaluate_permission(
        &mut self,
        context: &EvaluationContext,
        user_package_tier: &PackageTier,
        user_roles: &[IamRole],
        user_overrides: &Option<UserPermissionOverride>,
        policies: &[IamPolicy],
    ) -> Result<EvaluationDecision, IamError> {
        // Generate cache key for this permission check
        let cache_key = self.generate_permission_cache_key(
            context, 
            user_package_tier, 
            user_roles, 
            user_overrides, 
            policies
        );

        // Check permission check cache first
        if let Some(cached_result) = self.get_cached_permission_result(&cache_key) {
            // Convert cached result to EvaluationDecision
            return Ok(EvaluationDecision {
                result: cached_result.result,
                reasons: vec!["Returned from permission check cache".to_string()],
                matching_policies: vec![],
                package_tier_access: true,
                explicit_permissions: vec![],
            });
        }

        // Perform cleanup if cache is getting too large
        self.cleanup_caches_if_needed();
        let mut reasons = Vec::new();
        let mut matching_policies = Vec::new();
        let mut explicit_permissions = Vec::new();

        // Step 1: Check package tier access first
        let package_tier_access = self.check_package_tier_access(
            context,
            user_package_tier,
        );

        if package_tier_access {
            reasons.push(format!(
                "Package tier {} allows access",
                user_package_tier
            ));
        }

        // Step 2: Evaluate explicit deny policies first (AWS IAM rule)
        for policy in policies {
            if !policy.is_active() {
                continue;
            }

            let compiled_policy = self.compile_policy(policy)?;
            
            if let Some(deny_result) = self.evaluate_policy_statements(
                context,
                &compiled_policy.deny_patterns,
                Effect::Deny,
            )? {
                if deny_result == EvaluationResult::Deny {
                    matching_policies.push(policy.name().to_string());
                    reasons.push(format!(
                        "Explicitly denied by policy: {}",
                        policy.name()
                    ));
                    
                    let decision = EvaluationDecision {
                        result: EvaluationResult::Deny,
                        reasons,
                        matching_policies,
                        package_tier_access,
                        explicit_permissions,
                    };
                    self.cache_permission_result(&cache_key, &decision.result);
                    return Ok(decision);
                }
            }
        }

        // Step 3: Check user permission overrides
        if let Some(overrides) = user_overrides {
            // Check explicit denials in overrides
            for denial in &overrides.permission_denials {
                if self.permission_matches(denial, context) {
                    reasons.push(format!(
                        "Explicitly denied by user override: {} on {}",
                        context.action, context.resource
                    ));
                    
                    let decision = EvaluationDecision {
                        result: EvaluationResult::Deny,
                        reasons,
                        matching_policies,
                        package_tier_access,
                        explicit_permissions,
                    };
                    self.cache_permission_result(&cache_key, &decision.result);
                    return Ok(decision);
                }
            }

            // Check explicit grants in overrides
            for grant in &overrides.permission_grants {
                if self.permission_matches(grant, context) {
                    explicit_permissions.push(format!("{}:{}", grant.action(), grant.resource()));
                    reasons.push(format!(
                        "Allowed by user override: {} on {}",
                        context.action, context.resource
                    ));
                    
                    let decision = EvaluationDecision {
                        result: EvaluationResult::Allow,
                        reasons,
                        matching_policies,
                        package_tier_access,
                        explicit_permissions,
                    };
                    self.cache_permission_result(&cache_key, &decision.result);
                    return Ok(decision);
                }
            }
        }

        // Step 4: Check role-based permissions
        for role in user_roles {
            if !role.is_assignable() {
                continue;
            }

            // Check inline permissions on the role
            for permission in role.inline_permissions() {
                if self.permission_matches(permission, context) {
                    explicit_permissions.push(format!("{}:{}", permission.action(), permission.resource()));
                    reasons.push(format!(
                        "Allowed by role {} inline permission: {} on {}",
                        role.name(), context.action, context.resource
                    ));
                    
                    let decision = EvaluationDecision {
                        result: EvaluationResult::Allow,
                        reasons,
                        matching_policies,
                        package_tier_access,
                        explicit_permissions,
                    };
                    self.cache_permission_result(&cache_key, &decision.result);
                    return Ok(decision);
                }
            }
        }

        // Step 5: Evaluate allow policies
        for policy in policies {
            if !policy.is_active() {
                continue;
            }

            let compiled_policy = self.compile_policy(policy)?;
            
            if let Some(allow_result) = self.evaluate_policy_statements(
                context,
                &compiled_policy.allow_patterns,
                Effect::Allow,
            )? {
                if allow_result == EvaluationResult::Allow {
                    matching_policies.push(policy.name().to_string());
                    reasons.push(format!(
                        "Allowed by policy: {}",
                        policy.name()
                    ));
                    
                    let decision = EvaluationDecision {
                        result: EvaluationResult::Allow,
                        reasons,
                        matching_policies,
                        package_tier_access,
                        explicit_permissions,
                    };
                    self.cache_permission_result(&cache_key, &decision.result);
                    return Ok(decision);
                }
            }
        }

        // Step 6: Check if package tier alone grants access
        if package_tier_access && self.is_package_tier_action(context) {
            reasons.push("Granted by package tier access".to_string());
            let decision = EvaluationDecision {
                result: EvaluationResult::Allow,
                reasons,
                matching_policies,
                package_tier_access,
                explicit_permissions,
            };
            self.cache_permission_result(&cache_key, &decision.result);
            return Ok(decision);
        }

        // Step 7: Default deny
        reasons.push("No explicit allow policy found - default deny".to_string());
        let decision = EvaluationDecision {
            result: EvaluationResult::Deny,
            reasons,
            matching_policies,
            package_tier_access,
            explicit_permissions,
        };

        // Cache the result before returning
        self.cache_permission_result(&cache_key, &decision.result);
        Ok(decision)
    }

    /// Check if package tier grants access to the requested action/resource
    fn check_package_tier_access(
        &self,
        context: &EvaluationContext,
        package_tier: &PackageTier,
    ) -> bool {
        match package_tier {
            PackageTier::Free => {
                // Free tier - basic read access only
                context.action.starts_with("read:") && 
                context.resource.contains("own") ||
                context.action == "auth:login" ||
                context.action == "auth:logout"
            },
            PackageTier::Bronze => {
                // Bronze tier - premium features
                self.check_package_tier_access(context, &PackageTier::Free) ||
                context.action.starts_with("premium:") ||
                context.action == "analytics:basic"
            },
            PackageTier::Silver => {
                // Silver tier - advanced features  
                self.check_package_tier_access(context, &PackageTier::Bronze) ||
                context.action == "analytics:advanced" ||
                context.action.starts_with("trading:basic")
            },
            PackageTier::Gold => {
                // Gold tier - professional features
                self.check_package_tier_access(context, &PackageTier::Silver) ||
                context.action.starts_with("trading:advanced") ||
                context.action == "export:data"
            },
            PackageTier::Platinum => {
                // Platinum tier - all premium features
                self.check_package_tier_access(context, &PackageTier::Gold) ||
                context.action.starts_with("api:unlimited")
            },
            PackageTier::Admin => {
                // Admin - user management
                context.action.starts_with("admin:") ||
                context.action.starts_with("users:manage") ||
                self.check_package_tier_access(context, &PackageTier::Platinum)
            },
            PackageTier::SuperAdmin => {
                // Super admin - system management
                true // Super admin has access to everything via package tier
            },
        }
    }

    /// Check if this is a basic package tier action
    fn is_package_tier_action(&self, context: &EvaluationContext) -> bool {
        context.action.starts_with("read:") ||
        context.action.starts_with("premium:") ||
        context.action.starts_with("analytics:") ||
        context.action.starts_with("trading:") ||
        context.action.starts_with("export:") ||
        context.action.starts_with("api:") ||
        context.action == "auth:login" ||
        context.action == "auth:logout"
    }

    /// Check if a permission matches the current context
    fn permission_matches(&self, permission: &Permission, context: &EvaluationContext) -> bool {
        permission.matches_action(&context.action) && 
        permission.matches_resource(&context.resource) &&
        self.check_permission_conditions(permission, context)
    }

    /// Check permission conditions
    fn check_permission_conditions(&self, permission: &Permission, context: &EvaluationContext) -> bool {
        if let Some(conditions) = permission.conditions() {
            for (key, value) in conditions {
                if !self.evaluate_simple_condition(key, value, context) {
                    return false;
                }
            }
        }
        true
    }

    /// Simple condition evaluation
    fn evaluate_simple_condition(&self, key: &str, value: &str, context: &EvaluationContext) -> bool {
        match key {
            "source_ip" => {
                if let Some(source_ip) = &context.source_ip {
                    source_ip == value
                } else {
                    false
                }
            },
            "user_id" => context.user_id.value().to_string() == value,
            "time_of_day" => {
                // Simple time range check
                let hour = context.request_time.hour();
                if value.contains("-") {
                    let parts: Vec<&str> = value.split('-').collect();
                    if parts.len() == 2 {
                        if let (Ok(start), Ok(end)) = (parts[0].parse::<u32>(), parts[1].parse::<u32>()) {
                            return hour >= start && hour <= end;
                        }
                    }
                }
                false
            },
            _ => {
                // Check context variables
                context.context_vars.get(key)
                    .map(|v| v == value)
                    .unwrap_or(false)
            }
        }
    }

    /// Compile policy for faster evaluation
    fn compile_policy(&mut self, policy: &IamPolicy) -> Result<CompiledPolicy, IamError> {
        let policy_id = policy.id().value().to_string();
        
        // Check cache first and validate TTL
        if let Some(compiled) = self.policy_cache.get(&policy_id) {
            let now = Utc::now();
            let age_minutes = (now - compiled.last_compiled).num_minutes();
            
            // Check if cache entry is still valid
            if age_minutes < self.cache_config.policy_ttl_minutes {
                return Ok(compiled.clone());
            }
        }

        // Compile the policy
        let mut allow_patterns = Vec::new();
        let mut deny_patterns = Vec::new();

        for statement in policy.policy_document().statements() {
            let compiled_statement = self.compile_statement(statement)?;
            
            match statement.effect() {
                Effect::Allow => allow_patterns.push(compiled_statement),
                Effect::Deny => deny_patterns.push(compiled_statement),
            }
        }

        let compiled = CompiledPolicy {
            policy_id: policy_id.clone(),
            allow_patterns,
            deny_patterns,
            last_compiled: Utc::now(),
        };

        let result = compiled.clone();
        self.policy_cache.insert(policy_id, compiled);
        Ok(result)
    }

    /// Compile individual policy statement
    fn compile_statement(&self, statement: &PolicyStatement) -> Result<CompiledStatement, IamError> {
        let actions = self.compile_action_set(statement.actions())?;
        let resources = self.compile_resource_set(statement.resources())?;
        let conditions = if let Some(condition) = statement.conditions() {
            Some(self.compile_condition(condition)?)
        } else {
            None
        };

        Ok(CompiledStatement {
            actions,
            resources,
            conditions,
            effect: statement.effect().clone(),
        })
    }

    /// Compile action set for matching
    fn compile_action_set(&self, actions: &ActionSet) -> Result<ActionMatcher, IamError> {
        match actions {
            ActionSet::All => Ok(ActionMatcher::All),
            ActionSet::Actions(action_list) => {
                let mut exact_actions = HashSet::new();
                let mut patterns = Vec::new();

                for action in action_list {
                    if action.contains('*') || action.contains('?') {
                        let pattern = action
                            .replace('*', ".*")
                            .replace('?', ".");
                        patterns.push(regex::Regex::new(&format!("^{}$", pattern))
                            .map_err(|e| IamError::InvalidPolicyDocument(e.to_string()))?);
                    } else {
                        exact_actions.insert(action.clone());
                    }
                }

                if patterns.is_empty() {
                    Ok(ActionMatcher::Exact(exact_actions))
                } else {
                    Ok(ActionMatcher::Patterns(patterns))
                }
            },
            ActionSet::Patterns(pattern_list) => {
                let mut patterns = Vec::new();
                for pattern in pattern_list {
                    let regex_pattern = pattern
                        .replace('*', ".*")
                        .replace('?', ".");
                    patterns.push(regex::Regex::new(&format!("^{}$", regex_pattern))
                        .map_err(|e| IamError::InvalidPolicyDocument(e.to_string()))?);
                }
                Ok(ActionMatcher::Patterns(patterns))
            }
        }
    }

    /// Compile resource set for matching
    fn compile_resource_set(&self, resources: &ResourceSet) -> Result<ResourceMatcher, IamError> {
        match resources {
            ResourceSet::All => Ok(ResourceMatcher::All),
            ResourceSet::Resources(resource_list) => {
                let mut exact_resources = HashSet::new();
                let mut patterns = Vec::new();

                for resource in resource_list {
                    if resource.contains('*') || resource.contains('?') {
                        let pattern = resource
                            .replace('*', ".*")
                            .replace('?', ".");
                        patterns.push(regex::Regex::new(&format!("^{}$", pattern))
                            .map_err(|e| IamError::InvalidPolicyDocument(e.to_string()))?);
                    } else {
                        exact_resources.insert(resource.clone());
                    }
                }

                if patterns.is_empty() {
                    Ok(ResourceMatcher::Exact(exact_resources))
                } else {
                    Ok(ResourceMatcher::Patterns(patterns))
                }
            },
            ResourceSet::Patterns(pattern_list) => {
                let mut patterns = Vec::new();
                for pattern in pattern_list {
                    let regex_pattern = pattern
                        .replace('*', ".*")
                        .replace('?', ".");
                    patterns.push(regex::Regex::new(&format!("^{}$", regex_pattern))
                        .map_err(|e| IamError::InvalidPolicyDocument(e.to_string()))?);
                }
                Ok(ResourceMatcher::Patterns(patterns))
            }
        }
    }

    /// Compile condition for evaluation
    fn compile_condition(&self, condition: &Condition) -> Result<ConditionMatcher, IamError> {
        let mut conditions = HashMap::new();
        
        // This is a simplified implementation - AWS IAM has many more condition operators
        for (operator, operator_conditions) in &condition.conditions {
            for (key, values) in operator_conditions {
                let condition_op = match operator.as_str() {
                    "StringEquals" => {
                        ConditionOperator::StringEquals(values.iter().cloned().collect())
                    },
                    "StringLike" => {
                        let mut patterns = Vec::new();
                        for value in values {
                            let pattern = value.replace('*', ".*").replace('?', ".");
                            patterns.push(regex::Regex::new(&pattern)
                                .map_err(|e| IamError::InvalidPolicyDocument(e.to_string()))?);
                        }
                        ConditionOperator::StringLike(patterns)
                    },
                    "Bool" => {
                        let value = values.first()
                            .and_then(|v| v.parse::<bool>().ok())
                            .unwrap_or(false);
                        ConditionOperator::Bool(value)
                    },
                    _ => return Err(IamError::InvalidPolicyDocument(
                        format!("Unsupported condition operator: {}", operator)
                    )),
                };
                conditions.insert(key.clone(), condition_op);
            }
        }

        Ok(ConditionMatcher { conditions })
    }

    /// Evaluate policy statements
    fn evaluate_policy_statements(
        &mut self,
        context: &EvaluationContext,
        statements: &[CompiledStatement],
        expected_effect: Effect,
    ) -> Result<Option<EvaluationResult>, IamError> {
        for statement in statements {
            if statement.effect != expected_effect {
                continue;
            }

            // Check if action matches
            if !self.action_matches(&statement.actions, &context.action) {
                continue;
            }

            // Check if resource matches
            if !self.resource_matches(&statement.resources, &context.resource) {
                continue;
            }

            // Check conditions if present
            if let Some(condition_matcher) = &statement.conditions {
                if !self.condition_matches(condition_matcher, context) {
                    continue;
                }
            }

            // Statement matches
            return Ok(Some(match expected_effect {
                Effect::Allow => EvaluationResult::Allow,
                Effect::Deny => EvaluationResult::Deny,
            }));
        }

        Ok(None)
    }

    /// Check if action matches compiled action matcher
    fn action_matches(&mut self, matcher: &ActionMatcher, action: &str) -> bool {
        match matcher {
            ActionMatcher::All => true,
            ActionMatcher::Exact(actions) => actions.contains(action),
            ActionMatcher::Patterns(patterns) => {
                patterns.iter().any(|pattern| {
                    let pattern_str = pattern.as_str();
                    
                    // Check cache first
                    if let Some(cached_result) = self.get_cached_wildcard_match(pattern_str, action) {
                        return cached_result;
                    }
                    
                    // Perform match and cache result
                    let result = pattern.is_match(action);
                    self.cache_wildcard_match(pattern_str, action, result);
                    result
                })
            }
        }
    }

    /// Check if resource matches compiled resource matcher
    fn resource_matches(&mut self, matcher: &ResourceMatcher, resource: &str) -> bool {
        match matcher {
            ResourceMatcher::All => true,
            ResourceMatcher::Exact(resources) => resources.contains(resource),
            ResourceMatcher::Patterns(patterns) => {
                patterns.iter().any(|pattern| {
                    let pattern_str = pattern.as_str();
                    
                    // Check cache first
                    if let Some(cached_result) = self.get_cached_wildcard_match(pattern_str, resource) {
                        return cached_result;
                    }
                    
                    // Perform match and cache result
                    let result = pattern.is_match(resource);
                    self.cache_wildcard_match(pattern_str, resource, result);
                    result
                })
            }
        }
    }

    /// Check if conditions match
    fn condition_matches(&self, matcher: &ConditionMatcher, context: &EvaluationContext) -> bool {
        for (key, operator) in &matcher.conditions {
            if !self.evaluate_condition_operator(key, operator, context) {
                return false;
            }
        }
        true
    }

    /// Evaluate individual condition operator
    fn evaluate_condition_operator(
        &self,
        key: &str,
        operator: &ConditionOperator,
        context: &EvaluationContext,
    ) -> bool {
        let context_value = context.context_vars.get(key);
        
        match operator {
            ConditionOperator::StringEquals(allowed_values) => {
                if let Some(value) = context_value {
                    allowed_values.contains(value)
                } else {
                    false
                }
            },
            ConditionOperator::StringLike(patterns) => {
                if let Some(value) = context_value {
                    patterns.iter().any(|pattern| pattern.is_match(value))
                } else {
                    false
                }
            },
            ConditionOperator::Bool(expected) => {
                if let Some(value) = context_value {
                    value.parse::<bool>().unwrap_or(false) == *expected
                } else {
                    false
                }
            },
            // TODO: Implement other condition operators
            _ => false,
        }
    }

    /// Generate cache key for permission check
    fn generate_permission_cache_key(
        &self,
        context: &EvaluationContext,
        user_package_tier: &PackageTier,
        user_roles: &[IamRole],
        user_overrides: &Option<UserPermissionOverride>,
        policies: &[IamPolicy],
    ) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        
        // Hash context
        context.user_id.value().hash(&mut hasher);
        context.action.hash(&mut hasher);
        context.resource.hash(&mut hasher);
        
        // Hash package tier
        format!("{:?}", user_package_tier).hash(&mut hasher);
        
        // Hash roles
        for role in user_roles {
            role.name().hash(&mut hasher);
            role.package_tier().hash(&mut hasher);
        }
        
        // Hash overrides
        if let Some(overrides) = user_overrides {
            overrides.permission_grants.len().hash(&mut hasher);
            overrides.permission_denials.len().hash(&mut hasher);
        }
        
        // Hash policies
        for policy in policies {
            policy.id().value().hash(&mut hasher);
            policy.version().hash(&mut hasher);
        }
        
        format!("perm_check_{:x}", hasher.finish())
    }

    /// Get cached permission result if valid
    fn get_cached_permission_result(&self, cache_key: &str) -> Option<PermissionCheckResult> {
        if let Some(cached_result) = self.permission_check_cache.get(cache_key) {
            let now = Utc::now();
            let age_minutes = (now - cached_result.cached_at).num_minutes();
            
            if age_minutes < self.cache_config.permission_check_ttl_minutes {
                return Some(cached_result.clone());
            }
        }
        None
    }

    /// Cache permission result
    fn cache_permission_result(&mut self, cache_key: &str, result: &EvaluationResult) {
        let cached_result = PermissionCheckResult {
            result: result.clone(),
            cached_at: Utc::now(),
        };
        
        self.permission_check_cache.insert(cache_key.to_string(), cached_result);
    }

    /// Get cached wildcard match result
    fn get_cached_wildcard_match(&self, pattern: &str, value: &str) -> Option<bool> {
        let cache_key = format!("wildcard_{}_{}", pattern, value);
        
        if let Some((result, cached_at)) = self.wildcard_match_cache.get(&cache_key) {
            let now = Utc::now();
            let age_minutes = (now - cached_at).num_minutes();
            
            if age_minutes < self.cache_config.wildcard_match_ttl_minutes {
                return Some(*result);
            }
        }
        None
    }

    /// Cache wildcard match result
    fn cache_wildcard_match(&mut self, pattern: &str, value: &str, result: bool) {
        let cache_key = format!("wildcard_{}_{}", pattern, value);
        self.wildcard_match_cache.insert(cache_key, (result, Utc::now()));
    }

    /// Cleanup caches if they exceed maximum entries
    fn cleanup_caches_if_needed(&mut self) {
        let total_entries = self.policy_cache.len() 
            + self.permission_check_cache.len() 
            + self.wildcard_match_cache.len();
            
        if total_entries > self.cache_config.max_cache_entries {
            self.cleanup_expired_cache_entries();
            
            // If still too many entries, remove oldest ones
            if total_entries > self.cache_config.max_cache_entries {
                self.cleanup_oldest_cache_entries();
            }
        }
    }

    /// Remove expired cache entries
    fn cleanup_expired_cache_entries(&mut self) {
        let now = Utc::now();
        
        // Clean policy cache
        let policy_ttl = Duration::minutes(self.cache_config.policy_ttl_minutes);
        self.policy_cache.retain(|_, compiled_policy| {
            (now - compiled_policy.last_compiled) < policy_ttl
        });
        
        // Clean permission check cache
        let permission_ttl = Duration::minutes(self.cache_config.permission_check_ttl_minutes);
        self.permission_check_cache.retain(|_, cached_result| {
            (now - cached_result.cached_at) < permission_ttl
        });
        
        // Clean wildcard match cache
        let wildcard_ttl = Duration::minutes(self.cache_config.wildcard_match_ttl_minutes);
        self.wildcard_match_cache.retain(|_, (_, cached_at)| {
            (now - *cached_at) < wildcard_ttl
        });
    }

    /// Remove oldest cache entries when cache is full
    fn cleanup_oldest_cache_entries(&mut self) {
        // Remove 20% of oldest entries from each cache
        let remove_count = (self.cache_config.max_cache_entries / 5).max(1);
        
        // Sort and remove oldest policy cache entries
        if self.policy_cache.len() > remove_count {
            let mut policies: Vec<_> = self.policy_cache.iter()
                .map(|(k, v)| (k.clone(), v.last_compiled))
                .collect();
            policies.sort_by_key(|(_, timestamp)| *timestamp);
            for (key, _) in policies.iter().take(remove_count) {
                self.policy_cache.remove(key);
            }
        }
        
        // Sort and remove oldest permission check cache entries
        if self.permission_check_cache.len() > remove_count {
            let mut permissions: Vec<_> = self.permission_check_cache.iter()
                .map(|(k, v)| (k.clone(), v.cached_at))
                .collect();
            permissions.sort_by_key(|(_, cached_at)| *cached_at);
            for (key, _) in permissions.iter().take(remove_count) {
                self.permission_check_cache.remove(key);
            }
        }
        
        // Sort and remove oldest wildcard match cache entries  
        if self.wildcard_match_cache.len() > remove_count {
            let mut wildcards: Vec<_> = self.wildcard_match_cache.iter()
                .map(|(k, (_, cached_at))| (k.clone(), *cached_at))
                .collect();
            wildcards.sort_by_key(|(_, cached_at)| *cached_at);
            for (key, _) in wildcards.iter().take(remove_count) {
                self.wildcard_match_cache.remove(key);
            }
        }
    }

    /// Get cache statistics for monitoring
    pub fn get_cache_stats(&self) -> CacheStats {
        CacheStats {
            policy_cache_entries: self.policy_cache.len(),
            permission_check_cache_entries: self.permission_check_cache.len(),
            wildcard_match_cache_entries: self.wildcard_match_cache.len(),
            total_cache_entries: self.policy_cache.len() 
                + self.permission_check_cache.len() 
                + self.wildcard_match_cache.len(),
            max_cache_entries: self.cache_config.max_cache_entries,
        }
    }

    /// Clear policy cache (useful for testing or when policies change)
    pub fn clear_cache(&mut self) {
        self.policy_cache.clear();
        self.permission_check_cache.clear();
        self.wildcard_match_cache.clear();
    }
}

impl Default for PolicyEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl EvaluationContext {
    pub fn new(user_id: UserId, action: String, resource: String) -> Self {
        Self {
            user_id,
            action,
            resource,
            request_time: Utc::now(),
            context_vars: HashMap::new(),
            source_ip: None,
            user_agent: None,
        }
    }

    pub fn with_context_var(mut self, key: String, value: String) -> Self {
        self.context_vars.insert(key, value);
        self
    }

    pub fn with_source_ip(mut self, ip: String) -> Self {
        self.source_ip = Some(ip);
        self
    }

    pub fn with_user_agent(mut self, ua: String) -> Self {
        self.user_agent = Some(ua);
        self
    }
}

impl EvaluationDecision {
    pub fn is_allowed(&self) -> bool {
        matches!(self.result, EvaluationResult::Allow)
    }

    pub fn is_denied(&self) -> bool {
        matches!(self.result, EvaluationResult::Deny)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_evaluate_package_tier_access() {
        let mut engine = PolicyEngine::new();
        let context = EvaluationContext::new(
            UserId::new("user123".to_string()),
            "read:own_data".to_string(),
            "users/123".to_string(),
        );

        assert!(engine.check_package_tier_access(&context, &PackageTier::Free));
        assert!(engine.check_package_tier_access(&context, &PackageTier::Gold));
    }

    #[test]
    fn should_deny_unauthorized_action() {
        let mut engine = PolicyEngine::new();
        let context = EvaluationContext::new(
            UserId::new("user123".to_string()),
            "delete:users".to_string(),
            "users/*".to_string(),
        );

        assert!(!engine.check_package_tier_access(&context, &PackageTier::Free));
        assert!(!engine.check_package_tier_access(&context, &PackageTier::Bronze));
    }

    #[test]
    fn should_allow_admin_actions() {
        let mut engine = PolicyEngine::new();
        let context = EvaluationContext::new(
            UserId::new("admin123".to_string()),
            "admin:manage_users".to_string(),
            "users/*".to_string(),
        );

        assert!(engine.check_package_tier_access(&context, &PackageTier::Admin));
        assert!(engine.check_package_tier_access(&context, &PackageTier::SuperAdmin));
    }

    #[test]
    fn should_match_action_patterns() {
        let mut engine = PolicyEngine::new();
        
        // Test exact match
        let exact_matcher = ActionMatcher::Exact(
            vec!["users:read".to_string(), "users:write".to_string()].into_iter().collect()
        );
        assert!(engine.action_matches(&exact_matcher, "users:read"));
        assert!(!engine.action_matches(&exact_matcher, "users:delete"));

        // Test all match
        let all_matcher = ActionMatcher::All;
        assert!(engine.action_matches(&all_matcher, "any:action"));
    }

    #[test]
    fn should_evaluate_simple_conditions() {
        let engine = PolicyEngine::new();
        let context = EvaluationContext::new(
            UserId::new("user123".to_string()),
            "read:data".to_string(),
            "data/123".to_string(),
        ).with_context_var("department".to_string(), "engineering".to_string());

        assert!(engine.evaluate_simple_condition("user_id", "user123", &context));
        assert!(engine.evaluate_simple_condition("department", "engineering", &context));
        assert!(!engine.evaluate_simple_condition("department", "sales", &context));
    }

    #[test]
    fn should_cache_permission_check_results() {
        let mut engine = PolicyEngine::new();
        
        // Generate a test cache key
        let context = EvaluationContext::new(
            UserId::new("user123".to_string()),
            "read:data".to_string(),
            "data/123".to_string(),
        );
        
        let package_tier = PackageTier::Free;
        let roles: Vec<IamRole> = vec![];
        let overrides = None;
        let policies: Vec<IamPolicy> = vec![];
        
        let cache_key = engine.generate_permission_cache_key(
            &context,
            &package_tier,
            &roles,
            &overrides,
            &policies,
        );
        
        // Should return None initially
        assert!(engine.get_cached_permission_result(&cache_key).is_none());
        
        // Cache a result
        engine.cache_permission_result(&cache_key, &EvaluationResult::Allow);
        
        // Should return cached result
        let cached = engine.get_cached_permission_result(&cache_key);
        assert!(cached.is_some());
        assert_eq!(cached.unwrap().result, EvaluationResult::Allow);
    }

    #[test]
    fn should_cache_wildcard_matches() {
        let mut engine = PolicyEngine::new();
        
        // Should return None initially
        assert!(engine.get_cached_wildcard_match("users:*", "users:read").is_none());
        
        // Cache a result
        engine.cache_wildcard_match("users:*", "users:read", true);
        
        // Should return cached result
        let cached = engine.get_cached_wildcard_match("users:*", "users:read");
        assert!(cached.is_some());
        assert_eq!(cached.unwrap(), true);
    }

    #[test]
    fn should_provide_cache_statistics() {
        let mut engine = PolicyEngine::new();
        
        let stats = engine.get_cache_stats();
        assert_eq!(stats.policy_cache_entries, 0);
        assert_eq!(stats.permission_check_cache_entries, 0);
        assert_eq!(stats.wildcard_match_cache_entries, 0);
        assert_eq!(stats.total_cache_entries, 0);
        
        // Add some cache entries
        engine.cache_permission_result("test_key", &EvaluationResult::Allow);
        engine.cache_wildcard_match("test:*", "test:action", true);
        
        let stats = engine.get_cache_stats();
        assert_eq!(stats.permission_check_cache_entries, 1);
        assert_eq!(stats.wildcard_match_cache_entries, 1);
        assert_eq!(stats.total_cache_entries, 2);
    }

    #[test]
    fn should_clear_all_caches() {
        let mut engine = PolicyEngine::new();
        
        // Add some cache entries
        engine.cache_permission_result("test_key", &EvaluationResult::Allow);
        engine.cache_wildcard_match("test:*", "test:action", true);
        
        let stats = engine.get_cache_stats();
        assert!(stats.total_cache_entries > 0);
        
        // Clear caches
        engine.clear_cache();
        
        let stats = engine.get_cache_stats();
        assert_eq!(stats.total_cache_entries, 0);
    }
}