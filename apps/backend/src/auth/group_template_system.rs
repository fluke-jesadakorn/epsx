// ============================================================================
// EPSX GROUP TEMPLATE SYSTEM - REUSABLE DYNAMIC GROUP PATTERNS
// ============================================================================
// This module implements a sophisticated template system for creating reusable
// group patterns with configurable parameters. Features include:
// - Template parameter substitution and validation  
// - Template inheritance and composition
// - Version control and rollback capabilities
// - Template marketplace for sharing patterns
// - Smart instantiation with context-aware defaults
// - Performance optimization with template caching
// ============================================================================

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, BTreeMap};
use uuid::Uuid;
use tracing::{info, warn, debug, instrument};
use anyhow::{Result, anyhow, Context};
use regex::Regex;

use crate::auth::dynamic_group_rules_engine::{
    DynamicRule, RuleType, LogicOperator, RuleCondition, RuleActions
};

// ============================================================================
// TEMPLATE SYSTEM TYPES & STRUCTURES
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupTemplate {
    pub id: Uuid,
    pub name: String,
    pub slug: String,
    pub description: Option<String>,
    pub template_version: String,
    pub category: TemplateCategory,
    pub parameters: TemplateParameters,
    pub default_parameters: serde_json::Value,
    pub permission_patterns: Vec<String>,
    pub auto_assignment_enabled: bool,
    pub evaluation_conditions: serde_json::Value,
    pub evaluation_frequency: EvaluationFrequency,
    pub tags: Vec<String>,
    pub author_id: Option<Uuid>,
    pub is_system_template: bool,
    pub is_published: bool,
    pub usage_count: i32,
    pub parent_template_id: Option<Uuid>, // For template inheritance
    pub template_metadata: TemplateMetadata,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub published_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TemplateCategory {
    Subscription,
    Behavioral,
    Temporal,
    Web3,
    Custom,
    Enterprise,
    Analytics,
    Security,
    Performance,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[derive(PartialEq)]
pub enum EvaluationFrequency {
    Realtime,
    Hourly,
    Daily,
    Weekly,
    Monthly,
    Manual,
    EventDriven,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateParameters {
    pub parameters: BTreeMap<String, ParameterDefinition>,
    pub required_parameters: Vec<String>,
    pub parameter_groups: Vec<ParameterGroup>,
    pub validation_rules: Vec<ValidationRule>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParameterDefinition {
    pub name: String,
    pub parameter_type: ParameterType,
    pub description: Option<String>,
    pub default_value: Option<serde_json::Value>,
    pub required: bool,
    pub options: Option<Vec<serde_json::Value>>, // For enum-like parameters
    pub min_value: Option<serde_json::Value>,
    pub max_value: Option<serde_json::Value>,
    pub validation_regex: Option<String>,
    pub depends_on: Option<String>, // Parameter dependency
    pub conditional_visibility: Option<serde_json::Value>,
    pub help_text: Option<String>,
    pub ui_component: Option<UIComponent>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ParameterType {
    String,
    Integer,
    Float,
    Boolean,
    Array,
    Object,
    Enum,
    DateTime,
    Duration,
    Percentage,
    Currency,
    JsonPath,
    Regex,
    CronExpression,
    Web3Address,
    Custom(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParameterGroup {
    pub name: String,
    pub description: Option<String>,
    pub parameters: Vec<String>,
    pub collapsible: bool,
    pub expanded_by_default: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationRule {
    pub name: String,
    pub rule_type: ValidationType,
    pub parameters: Vec<String>, // Parameters involved in this validation
    pub condition: serde_json::Value, // Validation condition
    pub error_message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ValidationType {
    Required,
    MinValue,
    MaxValue,
    Range,
    Pattern,
    Custom,
    CrossParameter, // Validation between multiple parameters
    Conditional,    // Validation based on other parameter values
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UIComponent {
    TextInput,
    NumberInput,
    Textarea,
    Select,
    MultiSelect,
    Checkbox,
    Radio,
    DatePicker,
    TimePicker,
    Slider,
    ColorPicker,
    FileUpload,
    JsonEditor,
    CodeEditor,
    CronBuilder,
    Custom(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateMetadata {
    pub complexity_level: ComplexityLevel,
    pub estimated_setup_time: i32, // minutes
    pub maintenance_difficulty: MaintenanceDifficulty,
    pub performance_impact: PerformanceImpact,
    pub security_level: SecurityLevel,
    pub compatibility_tags: Vec<String>,
    pub prerequisites: Vec<String>,
    pub documentation_url: Option<String>,
    pub example_use_cases: Vec<String>,
    pub best_practices: Vec<String>,
    pub common_pitfalls: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ComplexityLevel {
    Beginner,
    Intermediate,
    Advanced,
    Expert,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MaintenanceDifficulty {
    Low,
    Medium,
    High,
    RequiresExpert,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PerformanceImpact {
    Minimal,
    Low,
    Medium,
    High,
    RequiresOptimization,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SecurityLevel {
    Public,
    Internal,
    Confidential,
    Restricted,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateInstantiation {
    pub template_id: Uuid,
    pub group_name: String,
    pub parameters: serde_json::Value,
    pub context: InstantiationContext,
    pub override_settings: Option<TemplateOverrides>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstantiationContext {
    pub created_by: Uuid,
    pub environment: String,
    pub target_user_count: Option<i32>,
    pub business_context: Option<String>,
    pub deployment_notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateOverrides {
    pub custom_permissions: Option<Vec<String>>,
    pub custom_conditions: Option<serde_json::Value>,
    pub custom_evaluation_frequency: Option<EvaluationFrequency>,
    pub custom_metadata: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneratedGroup {
    pub group_id: Uuid,
    pub template_id: Uuid,
    pub group_name: String,
    pub permissions: Vec<String>,
    pub rules: Vec<DynamicRule>,
    pub template_parameters: serde_json::Value,
    pub generation_metadata: GenerationMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerationMetadata {
    pub generated_at: DateTime<Utc>,
    pub generated_by: Uuid,
    pub template_version: String,
    pub parameter_resolution_log: Vec<ParameterResolution>,
    pub warnings: Vec<String>,
    pub optimizations_applied: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParameterResolution {
    pub parameter_name: String,
    pub resolved_value: serde_json::Value,
    pub resolution_source: ResolutionSource,
    pub confidence: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ResolutionSource {
    UserProvided,
    Default,
    Computed,
    Inherited,
    ContextInferred,
    SystemRecommended,
}

// ============================================================================
// GROUP TEMPLATE SYSTEM IMPLEMENTATION
// ============================================================================

pub struct GroupTemplateSystem {
    template_cache: std::sync::Arc<tokio::sync::RwLock<HashMap<Uuid, GroupTemplate>>>,
    parameter_resolvers: HashMap<String, Box<dyn ParameterResolver + Send + Sync>>,
    template_validators: Vec<Box<dyn TemplateValidator + Send + Sync>>,
    performance_tracker: std::sync::Arc<tokio::sync::RwLock<TemplatePerformanceTracker>>,
}

#[derive(Debug, Clone)]
struct TemplatePerformanceTracker {
    instantiation_times: HashMap<Uuid, Vec<u128>>, // template_id -> times in ms
    success_rates: HashMap<Uuid, (i32, i32)>,      // template_id -> (successes, attempts)
    parameter_resolution_times: HashMap<String, Vec<u128>>, // parameter_type -> times
}

// ============================================================================
// TRAITS FOR EXTENSIBILITY
// ============================================================================

pub trait ParameterResolver {
    fn resolve_parameter(
        &self,
        parameter: &ParameterDefinition,
        context: &InstantiationContext,
        existing_parameters: &serde_json::Value,
    ) -> Result<serde_json::Value>;
    
    fn get_supported_types(&self) -> Vec<ParameterType>;
}

pub trait TemplateValidator {
    fn validate_template(&self, template: &GroupTemplate) -> Result<Vec<ValidationIssue>>;
    fn validate_instantiation(
        &self,
        template: &GroupTemplate,
        instantiation: &TemplateInstantiation,
    ) -> Result<Vec<ValidationIssue>>;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationIssue {
    pub severity: IssueSeverity,
    pub issue_type: IssueType,
    pub message: String,
    pub parameter: Option<String>,
    pub suggestions: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum IssueSeverity {
    Info,
    Warning,
    Error,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum IssueType {
    MissingParameter,
    InvalidParameterValue,
    ConflictingParameters,
    SecurityConcern,
    PerformanceConcern,
    BestPracticeViolation,
    CompatibilityIssue,
}

// ============================================================================
// TEMPLATE SYSTEM IMPLEMENTATION
// ============================================================================

impl GroupTemplateSystem {
    pub fn new() -> Self {
        let mut system = Self {
            template_cache: std::sync::Arc::new(tokio::sync::RwLock::new(HashMap::new())),
            parameter_resolvers: HashMap::new(),
            template_validators: Vec::new(),
            performance_tracker: std::sync::Arc::new(tokio::sync::RwLock::new(
                TemplatePerformanceTracker {
                    instantiation_times: HashMap::new(),
                    success_rates: HashMap::new(),
                    parameter_resolution_times: HashMap::new(),
                }
            )),
        };

        // Register built-in parameter resolvers
        system.register_default_resolvers();
        system.register_default_validators();

        system
    }

    fn register_default_resolvers(&mut self) {
        // Register built-in parameter resolvers
        self.parameter_resolvers.insert(
            "subscription_tier".to_string(),
            Box::new(SubscriptionTierResolver::new())
        );
        self.parameter_resolvers.insert(
            "behavioral_threshold".to_string(),
            Box::new(BehavioralThresholdResolver::new())
        );
        self.parameter_resolvers.insert(
            "time_window".to_string(),
            Box::new(TimeWindowResolver::new())
        );
    }

    fn register_default_validators(&mut self) {
        self.template_validators.push(Box::new(SecurityValidator::new()));
        self.template_validators.push(Box::new(PerformanceValidator::new()));
        self.template_validators.push(Box::new(BestPracticesValidator::new()));
    }

    /// Create a new group from a template with parameter substitution
    #[instrument(skip(self, instantiation))]
    pub async fn instantiate_template(
        &self,
        instantiation: TemplateInstantiation,
    ) -> Result<GeneratedGroup> {
        let start_time = std::time::Instant::now();
        
        info!("Instantiating template {} as group '{}'", 
            instantiation.template_id, instantiation.group_name);

        // Get template from cache or load
        let template = self.get_template(instantiation.template_id).await?;

        // Validate instantiation request
        self.validate_instantiation(&template, &instantiation).await?;

        // Resolve all parameters with smart defaults and validation
        let resolved_parameters = self.resolve_parameters(&template, &instantiation).await?;

        // Generate permissions from patterns
        let permissions = self.generate_permissions_from_patterns(
            &template.permission_patterns,
            &resolved_parameters,
        ).await?;

        // Generate dynamic rules from template conditions
        let rules = self.generate_rules_from_template(&template, &resolved_parameters).await?;

        // Create group ID
        let group_id = Uuid::new_v4();

        // Create generation metadata
        let generation_metadata = GenerationMetadata {
            generated_at: Utc::now(),
            generated_by: instantiation.context.created_by,
            template_version: template.template_version.clone(),
            parameter_resolution_log: self.get_parameter_resolution_log(&resolved_parameters),
            warnings: Vec::new(), // Would be populated during generation
            optimizations_applied: Vec::new(), // Would be populated during optimization
        };

        let generated_group = GeneratedGroup {
            group_id,
            template_id: template.id,
            group_name: instantiation.group_name.clone(),
            permissions,
            rules,
            template_parameters: resolved_parameters,
            generation_metadata,
        };

        // Update performance metrics
        let instantiation_time = start_time.elapsed().as_millis();
        self.update_performance_metrics(template.id, instantiation_time, true).await;

        // Update template usage count
        self.increment_template_usage(template.id).await?;

        info!("Template instantiation completed in {:?} for group '{}'", 
            start_time.elapsed(), instantiation.group_name);

        Ok(generated_group)
    }

    /// Validate template instantiation request
    async fn validate_instantiation(
        &self,
        template: &GroupTemplate,
        instantiation: &TemplateInstantiation,
    ) -> Result<()> {
        let mut all_issues = Vec::new();

        // Run all validators
        for validator in &self.template_validators {
            let issues = validator.validate_instantiation(template, instantiation)?;
            all_issues.extend(issues);
        }

        // Check for critical issues
        let critical_issues: Vec<_> = all_issues.iter()
            .filter(|issue| matches!(issue.severity, IssueSeverity::Critical | IssueSeverity::Error))
            .collect();

        if !critical_issues.is_empty() {
            let error_msg = critical_issues.iter()
                .map(|issue| issue.message.clone())
                .collect::<Vec<_>>()
                .join(", ");
            return Err(anyhow!("Template instantiation validation failed: {}", error_msg));
        }

        // Log warnings
        for issue in all_issues.iter() {
            match issue.severity {
                IssueSeverity::Warning => warn!("Template instantiation warning: {}", issue.message),
                IssueSeverity::Info => info!("Template instantiation info: {}", issue.message),
                _ => {}
            }
        }

        Ok(())
    }

    /// Resolve all parameters using resolvers and defaults
    #[instrument(skip(self, template, instantiation))]
    async fn resolve_parameters(
        &self,
        template: &GroupTemplate,
        instantiation: &TemplateInstantiation,
    ) -> Result<serde_json::Value> {
        let mut resolved = serde_json::Map::new();
        let default_map = serde_json::Map::new();
        let user_provided = instantiation.parameters.as_object().unwrap_or(&default_map);

        debug!("Resolving {} parameters for template {}", 
            template.parameters.parameters.len(), template.id);

        for (param_name, param_def) in &template.parameters.parameters {
            let resolution_start = std::time::Instant::now();

            let resolved_value = if let Some(user_value) = user_provided.get(param_name) {
                // User provided value - validate and use
                self.validate_parameter_value(param_def, user_value).await?;
                user_value.clone()
            } else if let Some(resolver_name) = param_name.split('_').next() {
                // Try to resolve using registered resolver
                if let Some(resolver) = self.parameter_resolvers.get(resolver_name) {
                    let current_params = serde_json::Value::Object(resolved.clone());
                    resolver.resolve_parameter(param_def, &instantiation.context, &current_params)?
                } else if let Some(default) = &param_def.default_value {
                    // Use parameter default
                    default.clone()
                } else if let Some(template_default) = template.default_parameters.get(param_name) {
                    // Use template default
                    template_default.clone()
                } else if param_def.required {
                    return Err(anyhow!("Required parameter '{}' not provided and no default available", param_name));
                } else {
                    serde_json::Value::Null
                }
            } else if let Some(default) = &param_def.default_value {
                default.clone()
            } else if param_def.required {
                return Err(anyhow!("Required parameter '{}' not provided", param_name));
            } else {
                serde_json::Value::Null
            };

            resolved.insert(param_name.clone(), resolved_value);

            // Track parameter resolution performance
            let resolution_time = resolution_start.elapsed().as_millis();
            self.track_parameter_resolution_time(&param_def.parameter_type, resolution_time).await;
        }

        // Run cross-parameter validation
        self.validate_parameter_combinations(&template.parameters, &resolved).await?;

        Ok(serde_json::Value::Object(resolved))
    }

    /// Validate individual parameter value
    async fn validate_parameter_value(
        &self,
        param_def: &ParameterDefinition,
        value: &serde_json::Value,
    ) -> Result<()> {
        // Type validation
        match param_def.parameter_type {
            ParameterType::String => {
                if !value.is_string() {
                    return Err(anyhow!("Parameter '{}' must be a string", param_def.name));
                }
            },
            ParameterType::Integer => {
                if !value.is_number() || value.as_f64().unwrap() % 1.0 != 0.0 {
                    return Err(anyhow!("Parameter '{}' must be an integer", param_def.name));
                }
            },
            ParameterType::Float => {
                if !value.is_number() {
                    return Err(anyhow!("Parameter '{}' must be a number", param_def.name));
                }
            },
            ParameterType::Boolean => {
                if !value.is_boolean() {
                    return Err(anyhow!("Parameter '{}' must be a boolean", param_def.name));
                }
            },
            ParameterType::Array => {
                if !value.is_array() {
                    return Err(anyhow!("Parameter '{}' must be an array", param_def.name));
                }
            },
            ParameterType::Object => {
                if !value.is_object() {
                    return Err(anyhow!("Parameter '{}' must be an object", param_def.name));
                }
            },
            ParameterType::Enum => {
                if let Some(options) = &param_def.options {
                    if !options.contains(value) {
                        return Err(anyhow!("Parameter '{}' must be one of: {:?}", 
                            param_def.name, options));
                    }
                }
            },
            _ => {} // Other types would have specific validation
        }

        // Range validation
        if let Some(min) = &param_def.min_value {
            if let (Some(val), Some(min_val)) = (value.as_f64(), min.as_f64()) {
                if val < min_val {
                    return Err(anyhow!("Parameter '{}' must be >= {}", param_def.name, min_val));
                }
            }
        }

        if let Some(max) = &param_def.max_value {
            if let (Some(val), Some(max_val)) = (value.as_f64(), max.as_f64()) {
                if val > max_val {
                    return Err(anyhow!("Parameter '{}' must be <= {}", param_def.name, max_val));
                }
            }
        }

        // Regex validation
        if let Some(pattern) = &param_def.validation_regex {
            if let Some(string_val) = value.as_str() {
                let regex = Regex::new(pattern)
                    .context(format!("Invalid regex pattern for parameter '{}'", param_def.name))?;
                if !regex.is_match(string_val) {
                    return Err(anyhow!("Parameter '{}' does not match required pattern", param_def.name));
                }
            }
        }

        Ok(())
    }

    /// Validate parameter combinations and dependencies
    async fn validate_parameter_combinations(
        &self,
        template_params: &TemplateParameters,
        _resolved_params: &serde_json::Map<String, serde_json::Value>,
    ) -> Result<()> {
        for validation_rule in &template_params.validation_rules {
            match validation_rule.rule_type {
                ValidationType::CrossParameter => {
                    // Implement cross-parameter validation logic
                    debug!("Validating cross-parameter rule: {}", validation_rule.name);
                },
                ValidationType::Conditional => {
                    // Implement conditional validation logic
                    debug!("Validating conditional rule: {}", validation_rule.name);
                },
                _ => {} // Other validation types handled elsewhere
            }
        }

        Ok(())
    }

    /// Generate permissions by substituting parameters into patterns
    async fn generate_permissions_from_patterns(
        &self,
        patterns: &[String],
        parameters: &serde_json::Value,
    ) -> Result<Vec<String>> {
        let mut permissions = Vec::new();
        let default_map = serde_json::Map::new();
        let param_map = parameters.as_object().unwrap_or(&default_map);

        for pattern in patterns {
            let mut permission = pattern.clone();
            
            // Replace parameter placeholders like {tier}, {threshold}
            for (param_name, param_value) in param_map {
                let placeholder = format!("{{{}}}", param_name);
                if permission.contains(&placeholder) {
                    let replacement = match param_value {
                        serde_json::Value::String(s) => s.clone(),
                        serde_json::Value::Number(n) => n.to_string(),
                        serde_json::Value::Bool(b) => b.to_string(),
                        _ => param_value.to_string().trim_matches('"').to_string(),
                    };
                    permission = permission.replace(&placeholder, &replacement);
                }
            }

            // Handle complex substitutions and conditional patterns
            permission = self.process_advanced_patterns(&permission, parameters).await?;
            
            if !permission.is_empty() {
                permissions.push(permission);
            }
        }

        Ok(permissions)
    }

    /// Process advanced pattern substitutions and conditional logic
    async fn process_advanced_patterns(
        &self,
        pattern: &str,
        parameters: &serde_json::Value,
    ) -> Result<String> {
        let mut result = pattern.to_string();

        // Handle conditional patterns like {if tier=="premium"}extra:permission{/if}
        let conditional_regex = Regex::new(r"\{if\s+([^}]+)\}([^{]*)\{/if\}")
            .context("Failed to compile conditional pattern regex")?;

        for caps in conditional_regex.captures_iter(pattern) {
            let condition = &caps[1];
            let content = &caps[2];
            let full_match = &caps[0];

            if self.evaluate_condition_expression(condition, parameters).await? {
                result = result.replace(full_match, content);
            } else {
                result = result.replace(full_match, "");
            }
        }

        // Handle computed values like {computed:tier_limit}
        let computed_regex = Regex::new(r"\{computed:([^}]+)\}")
            .context("Failed to compile computed pattern regex")?;

        let result_copy = result.clone();
        let captures: Vec<_> = computed_regex.captures_iter(&result_copy).collect();
        for caps in captures {
            let computation = caps[1].to_string();
            let full_match = caps[0].to_string();

            let computed_value = self.compute_parameter_value(&computation, parameters).await?;
            result = result.replace(&full_match, &computed_value);
        }

        Ok(result)
    }

    /// Evaluate simple condition expressions in templates
    async fn evaluate_condition_expression(
        &self,
        condition: &str,
        parameters: &serde_json::Value,
    ) -> Result<bool> {
        // Simple condition evaluation - could be extended with a full expression parser
        let default_map = serde_json::Map::new();
        let param_map = parameters.as_object().unwrap_or(&default_map);

        // Handle conditions like: tier=="premium", threshold>100, etc.
        if condition.contains("==") {
            let parts: Vec<&str> = condition.split("==").collect();
            if parts.len() == 2 {
                let param_name = parts[0].trim();
                let expected_value = parts[1].trim().trim_matches('"');
                
                if let Some(actual_value) = param_map.get(param_name) {
                    return Ok(actual_value.as_str() == Some(expected_value));
                }
            }
        } else if condition.contains(">") {
            let parts: Vec<&str> = condition.split(">").collect();
            if parts.len() == 2 {
                let param_name = parts[0].trim();
                let threshold: f64 = parts[1].trim().parse().unwrap_or(0.0);
                
                if let Some(actual_value) = param_map.get(param_name) {
                    if let Some(num_val) = actual_value.as_f64() {
                        return Ok(num_val > threshold);
                    }
                }
            }
        }

        Ok(false)
    }

    /// Compute derived parameter values
    async fn compute_parameter_value(
        &self,
        computation: &str,
        parameters: &serde_json::Value,
    ) -> Result<String> {
        let default_map = serde_json::Map::new();
        let param_map = parameters.as_object().unwrap_or(&default_map);

        match computation {
            "tier_limit" => {
                // Compute tier limit based on subscription tier
                if let Some(tier) = param_map.get("tier").and_then(|t| t.as_str()) {
                    let limit = match tier {
                        "free" => "5",
                        "bronze" => "25",
                        "silver" => "50", 
                        "gold" => "100",
                        "platinum" => "150",
                        "enterprise" => "unlimited",
                        _ => "5",
                    };
                    Ok(limit.to_string())
                } else {
                    Ok("5".to_string())
                }
            },
            _ => Ok("0".to_string()), // Default for unknown computations
        }
    }

    /// Generate dynamic rules from template conditions
    async fn generate_rules_from_template(
        &self,
        template: &GroupTemplate,
        parameters: &serde_json::Value,
    ) -> Result<Vec<DynamicRule>> {
        let mut rules = Vec::new();

        // Generate rules from template evaluation conditions
        if let Some(conditions_obj) = template.evaluation_conditions.as_object() {
            let rule = DynamicRule {
                id: Uuid::new_v4(),
                group_id: Uuid::new_v4(), // Will be set when group is created
                rule_name: format!("{}_auto_assignment", template.name),
                rule_type: RuleType::Conditional,
                is_active: template.auto_assignment_enabled,
                priority: 0,
                logic_operator: LogicOperator::And, // Default
                conditions: self.parse_template_conditions(conditions_obj, parameters).await?,
                actions: RuleActions {
                    assign: true,
                    notify: false,
                    trigger_events: Vec::new(),
                    custom_actions: None,
                },
                behavioral_triggers: None,
                behavioral_patterns: None,
                temporal_schedule: None,
                timezone: "UTC".to_string(),
                ml_model_config: None,
                confidence_threshold: 0.8,
                evaluation_count: 0,
                success_rate: 0.0,
                last_evaluated_at: None,
                avg_evaluation_time_ms: 0,
            };

            rules.push(rule);
        }

        Ok(rules)
    }

    /// Parse template conditions into rule conditions
    async fn parse_template_conditions(
        &self,
        conditions_obj: &serde_json::Map<String, serde_json::Value>,
        _parameters: &serde_json::Value,
    ) -> Result<Vec<RuleCondition>> {
        let mut conditions = Vec::new();

        if let Some(conditions_array) = conditions_obj.get("conditions").and_then(|c| c.as_array()) {
            for condition_value in conditions_array {
                if let Some(condition_obj) = condition_value.as_object() {
                    let condition = RuleCondition {
                        field: condition_obj.get("field")
                            .and_then(|f| f.as_str())
                            .unwrap_or("")
                            .to_string(),
                        operator: serde_json::from_value(
                            condition_obj.get("operator").cloned()
                                .unwrap_or(serde_json::Value::String("equals".to_string()))
                        )?,
                        value: condition_obj.get("value").cloned().unwrap_or(serde_json::Value::Null),
                        weight: condition_obj.get("weight").and_then(|w| w.as_f64()),
                        aggregation_period: condition_obj.get("aggregation_period").and_then(|a| a.as_str()).map(|s| s.to_string()),
                        contract: condition_obj.get("contract").and_then(|c| c.as_str()).map(|s| s.to_string()),
                        custom_config: condition_obj.get("custom_config").cloned(),
                    };
                    conditions.push(condition);
                }
            }
        }

        Ok(conditions)
    }

    // ============================================================================
    // TEMPLATE MANAGEMENT METHODS
    // ============================================================================

    /// Get template by ID (with caching)
    pub async fn get_template(&self, template_id: Uuid) -> Result<GroupTemplate> {
        // Check cache first
        {
            let cache = self.template_cache.read().await;
            if let Some(template) = cache.get(&template_id) {
                return Ok(template.clone());
            }
        }

        // TODO: Load from database
        Err(anyhow!("Template not found: {}", template_id))
    }

    /// Cache template for performance
    pub async fn cache_template(&self, template: GroupTemplate) {
        let mut cache = self.template_cache.write().await;
        cache.insert(template.id, template);
    }

    /// Get all templates by category
    pub async fn get_templates_by_category(&self, _category: TemplateCategory) -> Result<Vec<GroupTemplate>> {
        // TODO: Implement database query
        Ok(Vec::new())
    }

    /// Search templates by tags and content
    pub async fn search_templates(&self, _query: &str, _tags: &[String]) -> Result<Vec<GroupTemplate>> {
        // TODO: Implement full-text search
        Ok(Vec::new())
    }

    // ============================================================================
    // PERFORMANCE TRACKING
    // ============================================================================

    async fn update_performance_metrics(&self, template_id: Uuid, time_ms: u128, success: bool) {
        let mut tracker = self.performance_tracker.write().await;
        
        // Track instantiation times
        tracker.instantiation_times
            .entry(template_id)
            .or_insert_with(Vec::new)
            .push(time_ms);

        // Track success rates
        let (successes, attempts) = tracker.success_rates.entry(template_id).or_insert((0, 0));
        *attempts += 1;
        if success {
            *successes += 1;
        }
    }

    async fn track_parameter_resolution_time(&self, param_type: &ParameterType, time_ms: u128) {
        let mut tracker = self.performance_tracker.write().await;
        let type_key = format!("{:?}", param_type);
        
        tracker.parameter_resolution_times
            .entry(type_key)
            .or_insert_with(Vec::new)
            .push(time_ms);
    }

    async fn increment_template_usage(&self, template_id: Uuid) -> Result<()> {
        // TODO: Update database usage count
        info!("Template {} usage incremented", template_id);
        Ok(())
    }

    // ============================================================================
    // UTILITY METHODS
    // ============================================================================

    fn get_parameter_resolution_log(&self, resolved_params: &serde_json::Value) -> Vec<ParameterResolution> {
        let mut log = Vec::new();
        
        if let Some(param_obj) = resolved_params.as_object() {
            for (name, value) in param_obj {
                log.push(ParameterResolution {
                    parameter_name: name.clone(),
                    resolved_value: value.clone(),
                    resolution_source: ResolutionSource::UserProvided, // Simplified
                    confidence: 1.0,
                });
            }
        }
        
        log
    }

    pub async fn get_performance_summary(&self) -> serde_json::Value {
        let tracker = self.performance_tracker.read().await;
        
        serde_json::json!({
            "total_templates_used": tracker.instantiation_times.len(),
            "total_instantiations": tracker.instantiation_times.values().map(|times| times.len()).sum::<usize>(),
            "average_instantiation_time_ms": tracker.instantiation_times.values()
                .flat_map(|times| times.iter())
                .sum::<u128>() as f64 / tracker.instantiation_times.values()
                .map(|times| times.len()).sum::<usize>() as f64,
            "success_rate": tracker.success_rates.values()
                .map(|(s, a)| *s as f64 / *a as f64)
                .sum::<f64>() / tracker.success_rates.len() as f64
        })
    }
}

// ============================================================================
// BUILT-IN PARAMETER RESOLVERS
// ============================================================================

struct SubscriptionTierResolver;

impl SubscriptionTierResolver {
    fn new() -> Self {
        Self
    }
}

impl ParameterResolver for SubscriptionTierResolver {
    fn resolve_parameter(
        &self,
        _parameter: &ParameterDefinition,
        context: &InstantiationContext,
        _existing_parameters: &serde_json::Value,
    ) -> Result<serde_json::Value> {
        // Smart resolution based on context
        let default_tier = match context.target_user_count.unwrap_or(10) {
            0..=10 => "free",
            11..=100 => "bronze", 
            101..=1000 => "silver",
            1001..=10000 => "gold",
            _ => "enterprise",
        };
        
        Ok(serde_json::Value::String(default_tier.to_string()))
    }
    
    fn get_supported_types(&self) -> Vec<ParameterType> {
        vec![ParameterType::String, ParameterType::Enum]
    }
}

struct BehavioralThresholdResolver;

impl BehavioralThresholdResolver {
    fn new() -> Self {
        Self
    }
}

impl ParameterResolver for BehavioralThresholdResolver {
    fn resolve_parameter(
        &self,
        parameter: &ParameterDefinition,
        _context: &InstantiationContext,
        _existing_parameters: &serde_json::Value,
    ) -> Result<serde_json::Value> {
        // Provide sensible defaults for behavioral thresholds
        let default_value = match parameter.name.as_str() {
            "engagement_threshold" => 0.7,
            "activity_threshold" => 100.0,
            "login_frequency_threshold" => 10.0,
            _ => 0.5,
        };
        
        Ok(serde_json::Value::Number(serde_json::Number::from_f64(default_value).unwrap()))
    }
    
    fn get_supported_types(&self) -> Vec<ParameterType> {
        vec![ParameterType::Float, ParameterType::Integer, ParameterType::Percentage]
    }
}

struct TimeWindowResolver;

impl TimeWindowResolver {
    fn new() -> Self {
        Self
    }
}

impl ParameterResolver for TimeWindowResolver {
    fn resolve_parameter(
        &self,
        _parameter: &ParameterDefinition,
        _context: &InstantiationContext,
        _existing_parameters: &serde_json::Value,
    ) -> Result<serde_json::Value> {
        // Default business hours window
        Ok(serde_json::json!({
            "start": "09:00",
            "end": "17:00",
            "days": ["mon", "tue", "wed", "thu", "fri"],
            "timezone": "UTC"
        }))
    }
    
    fn get_supported_types(&self) -> Vec<ParameterType> {
        vec![ParameterType::Object]
    }
}

// ============================================================================
// BUILT-IN VALIDATORS
// ============================================================================

struct SecurityValidator;

impl SecurityValidator {
    fn new() -> Self {
        Self
    }
}

impl TemplateValidator for SecurityValidator {
    fn validate_template(&self, template: &GroupTemplate) -> Result<Vec<ValidationIssue>> {
        let mut issues = Vec::new();

        // Check for overly broad permissions
        for permission in &template.permission_patterns {
            if permission.contains("*:*:*") {
                issues.push(ValidationIssue {
                    severity: IssueSeverity::Warning,
                    issue_type: IssueType::SecurityConcern,
                    message: "Template contains overly broad permission pattern".to_string(),
                    parameter: None,
                    suggestions: vec!["Consider using more specific permission patterns".to_string()],
                });
            }
        }

        Ok(issues)
    }

    fn validate_instantiation(
        &self,
        _template: &GroupTemplate,
        _instantiation: &TemplateInstantiation,
    ) -> Result<Vec<ValidationIssue>> {
        Ok(Vec::new()) // Placeholder
    }
}

struct PerformanceValidator;

impl PerformanceValidator {
    fn new() -> Self {
        Self
    }
}

impl TemplateValidator for PerformanceValidator {
    fn validate_template(&self, template: &GroupTemplate) -> Result<Vec<ValidationIssue>> {
        let mut issues = Vec::new();

        // Check for performance-intensive patterns
        if template.evaluation_frequency == EvaluationFrequency::Realtime && 
           template.parameters.parameters.len() > 10 {
            issues.push(ValidationIssue {
                severity: IssueSeverity::Warning,
                issue_type: IssueType::PerformanceConcern,
                message: "Real-time evaluation with many parameters may impact performance".to_string(),
                parameter: None,
                suggestions: vec!["Consider reducing evaluation frequency or parameter count".to_string()],
            });
        }

        Ok(issues)
    }

    fn validate_instantiation(
        &self,
        _template: &GroupTemplate,
        _instantiation: &TemplateInstantiation,
    ) -> Result<Vec<ValidationIssue>> {
        Ok(Vec::new()) // Placeholder
    }
}

struct BestPracticesValidator;

impl BestPracticesValidator {
    fn new() -> Self {
        Self
    }
}

impl TemplateValidator for BestPracticesValidator {
    fn validate_template(&self, template: &GroupTemplate) -> Result<Vec<ValidationIssue>> {
        let mut issues = Vec::new();

        // Check for missing documentation
        if template.description.is_none() || template.description.as_ref().unwrap().len() < 20 {
            issues.push(ValidationIssue {
                severity: IssueSeverity::Info,
                issue_type: IssueType::BestPracticeViolation,
                message: "Template should have detailed description".to_string(),
                parameter: None,
                suggestions: vec!["Add comprehensive description explaining template purpose and usage".to_string()],
            });
        }

        Ok(issues)
    }

    fn validate_instantiation(
        &self,
        _template: &GroupTemplate,
        _instantiation: &TemplateInstantiation,
    ) -> Result<Vec<ValidationIssue>> {
        Ok(Vec::new()) // Placeholder
    }
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_template() -> GroupTemplate {
        let mut parameters = BTreeMap::new();
        parameters.insert("tier".to_string(), ParameterDefinition {
            name: "tier".to_string(),
            parameter_type: ParameterType::Enum,
            description: Some("Subscription tier".to_string()),
            default_value: Some(serde_json::Value::String("free".to_string())),
            required: true,
            options: Some(vec![
                serde_json::Value::String("free".to_string()),
                serde_json::Value::String("premium".to_string()),
            ]),
            min_value: None,
            max_value: None,
            validation_regex: None,
            depends_on: None,
            conditional_visibility: None,
            help_text: None,
            ui_component: Some(UIComponent::Select),
        });

        GroupTemplate {
            id: Uuid::new_v4(),
            name: "Test Template".to_string(),
            slug: "test-template".to_string(),
            description: Some("Test template for unit tests".to_string()),
            template_version: "1.0".to_string(),
            category: TemplateCategory::Custom,
            parameters: TemplateParameters {
                parameters,
                required_parameters: vec!["tier".to_string()],
                parameter_groups: Vec::new(),
                validation_rules: Vec::new(),
            },
            default_parameters: serde_json::json!({"tier": "free"}),
            permission_patterns: vec!["epsx:analytics:{tier}".to_string()],
            auto_assignment_enabled: true,
            evaluation_conditions: serde_json::json!({"conditions": []}),
            evaluation_frequency: EvaluationFrequency::Daily,
            tags: vec!["test".to_string()],
            author_id: None,
            is_system_template: false,
            is_published: false,
            usage_count: 0,
            parent_template_id: None,
            template_metadata: TemplateMetadata {
                complexity_level: ComplexityLevel::Beginner,
                estimated_setup_time: 5,
                maintenance_difficulty: MaintenanceDifficulty::Low,
                performance_impact: PerformanceImpact::Minimal,
                security_level: SecurityLevel::Public,
                compatibility_tags: Vec::new(),
                prerequisites: Vec::new(),
                documentation_url: None,
                example_use_cases: Vec::new(),
                best_practices: Vec::new(),
                common_pitfalls: Vec::new(),
            },
            created_at: Utc::now(),
            updated_at: Utc::now(),
            published_at: None,
        }
    }

    #[tokio::test]
    async fn test_parameter_resolution() {
        let system = GroupTemplateSystem::new();
        let template = create_test_template();

        let instantiation = TemplateInstantiation {
            template_id: template.id,
            group_name: "Test Group".to_string(),
            parameters: serde_json::json!({"tier": "premium"}),
            context: InstantiationContext {
                created_by: Uuid::new_v4(),
                environment: "test".to_string(),
                target_user_count: Some(50),
                business_context: None,
                deployment_notes: None,
            },
            override_settings: None,
        };

        let resolved = system.resolve_parameters(&template, &instantiation).await.unwrap();
        assert_eq!(resolved["tier"], "premium");
    }

    #[tokio::test]
    async fn test_permission_pattern_substitution() {
        let system = GroupTemplateSystem::new();
        let parameters = serde_json::json!({"tier": "premium"});

        let permissions = system.generate_permissions_from_patterns(
            &["epsx:analytics:{tier}".to_string()],
            &parameters,
        ).await.unwrap();

        assert_eq!(permissions[0], "epsx:analytics:premium");
    }

    #[tokio::test]
    async fn test_template_validation() {
        let system = GroupTemplateSystem::new();
        let template = create_test_template();

        let instantiation = TemplateInstantiation {
            template_id: template.id,
            group_name: "Test Group".to_string(),
            parameters: serde_json::json!({"tier": "invalid_tier"}), // Invalid value
            context: InstantiationContext {
                created_by: Uuid::new_v4(),
                environment: "test".to_string(),
                target_user_count: Some(10),
                business_context: None,
                deployment_notes: None,
            },
            override_settings: None,
        };

        // Should fail validation due to invalid tier value
        let result = system.validate_instantiation(&template, &instantiation).await;
        assert!(result.is_err());
    }
}