// ============================================================================
// EPSX DYNAMIC RULES ENGINE - SOPHISTICATED GROUP RULE EVALUATION
// ============================================================================
// This module implements a powerful rules engine for dynamic group assignment
// that supports:
// - Complex conditional logic with AND/OR/NOT operations
// - Behavioral pattern matching and analysis
// - Temporal rule evaluation with scheduling
// - ML-powered rule confidence scoring  
// - Real-time event-driven rule triggers
// - Performance optimization and caching
// ============================================================================

use chrono::{DateTime, Utc, Timelike, Weekday, Datelike};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;
use tracing::{info, warn, error, debug, instrument};
use anyhow::Result;
use regex::Regex;

// ============================================================================
// RULE CONDITION TYPES & STRUCTURES
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LogicOperator {
    And,
    Or,
    Not,
    Xor,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConditionOperator {
    // Comparison operators
    Equals,
    NotEquals,
    GreaterThan,
    GreaterThanOrEqual,
    LessThan,
    LessThanOrEqual,
    
    // String operators
    Contains,
    NotContains,
    StartsWith,
    EndsWith,
    Matches, // Regex
    
    // Array operators
    In,
    NotIn,
    ContainsAny,
    ContainsAll,
    
    // Time operators
    Within,
    Before,
    After,
    Between,
    WithinWindow,
    
    // Behavioral operators
    HasPattern,
    ExceedsThreshold,
    TrendingUp,
    TrendingDown,
    
    // Custom operators
    Custom(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuleCondition {
    pub field: String,
    pub operator: ConditionOperator,
    pub value: serde_json::Value,
    pub weight: Option<f64>,
    pub aggregation_period: Option<String>,
    pub contract: Option<String>, // For Web3 conditions
    pub custom_config: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DynamicRule {
    pub id: Uuid,
    pub group_id: Uuid,
    pub rule_name: String,
    pub rule_type: RuleType,
    pub is_active: bool,
    pub priority: i32,
    pub logic_operator: LogicOperator,
    pub conditions: Vec<RuleCondition>,
    pub actions: RuleActions,
    pub behavioral_triggers: Option<serde_json::Value>,
    pub behavioral_patterns: Option<serde_json::Value>,
    pub temporal_schedule: Option<serde_json::Value>,
    pub timezone: String,
    pub ml_model_config: Option<serde_json::Value>,
    pub confidence_threshold: f64,
    pub evaluation_count: i64,
    pub success_rate: f64,
    pub last_evaluated_at: Option<DateTime<Utc>>,
    pub avg_evaluation_time_ms: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RuleType {
    Conditional,
    Behavioral, 
    Temporal,
    EventDriven,
    MlPowered,
    Composite,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuleActions {
    pub assign: bool,
    pub notify: bool,
    pub trigger_events: Vec<String>,
    pub custom_actions: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserContext {
    pub user_id: Uuid,
    pub email: Option<String>,
    pub subscription_tier: Option<String>,
    pub last_login: Option<DateTime<Utc>>,
    pub registration_date: Option<DateTime<Utc>>,
    pub permissions_used: Vec<String>,
    pub behavioral_data: Option<UserBehavioralData>,
    pub web3_data: Option<Web3UserData>,
    pub custom_fields: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserBehavioralData {
    pub login_frequency: i32,
    pub session_duration_minutes: i32,
    pub feature_usage_count: HashMap<String, i32>,
    pub api_calls_count: i32,
    pub active_days: i32,
    pub bounce_rate: f64,
    pub permissions_used: Vec<String>,
    pub permission_success_rate: f64,
    pub peak_usage_hours: Vec<i32>,
    pub preferred_features: Vec<String>,
    pub usage_patterns: Option<serde_json::Value>,
    pub user_cluster: Option<String>,
    pub behavior_score: f64,
    pub engagement_level: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Web3UserData {
    pub wallet_addresses: Vec<String>,
    pub token_balances: HashMap<String, String>, // contract_address -> balance
    pub nft_holdings: Vec<NftHolding>,
    pub dao_memberships: Vec<String>,
    pub transaction_count: i32,
    pub last_transaction_date: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NftHolding {
    pub contract_address: String,
    pub token_id: String,
    pub quantity: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuleEvaluationResult {
    pub rule_id: Uuid,
    pub user_id: Uuid,
    pub group_id: Uuid,
    pub evaluation_result: bool,
    pub confidence_score: f64,
    pub evaluation_reasons: serde_json::Value,
    pub evaluation_time_ms: u128,
    pub conditions_evaluated: i32,
    pub conditions_matched: i32,
    pub actions_executed: serde_json::Value,
    pub group_assignment_changed: bool,
    pub evaluation_context: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvaluationContext {
    pub current_time: DateTime<Utc>,
    pub user_timezone: Option<String>,
    pub device_type: Option<String>,
    pub location: Option<String>,
    pub environment: String, // development, staging, production
    pub feature_flags: Vec<String>,
    pub session_data: Option<serde_json::Value>,
}

// ============================================================================
// DYNAMIC RULES ENGINE IMPLEMENTATION
// ============================================================================

pub struct DynamicGroupRulesEngine {
    #[allow(dead_code)]
    timezone: String,
    cache: std::sync::Arc<tokio::sync::RwLock<HashMap<String, CachedEvaluation>>>,
    performance_metrics: std::sync::Arc<tokio::sync::RwLock<PerformanceMetrics>>,
}

#[derive(Debug, Clone)]
struct CachedEvaluation {
    result: bool,
    confidence: f64,
    cached_at: DateTime<Utc>,
    ttl_seconds: u64,
}

#[derive(Debug, Clone)]
pub struct PerformanceMetrics {
    total_evaluations: u64,
    total_time_ms: u128,
    cache_hits: u64,
    cache_misses: u64,
}

impl DynamicGroupRulesEngine {
    pub fn new() -> Self {
        Self {
            timezone: "UTC".to_string(),
            cache: std::sync::Arc::new(tokio::sync::RwLock::new(HashMap::new())),
            performance_metrics: std::sync::Arc::new(tokio::sync::RwLock::new(PerformanceMetrics {
                total_evaluations: 0,
                total_time_ms: 0,
                cache_hits: 0,
                cache_misses: 0,
            })),
        }
    }

    /// Main entry point for evaluating rules for a user
    #[instrument(skip(self, user_context, evaluation_context))]
    pub async fn evaluate_rules_for_user(
        &self,
        rules: Vec<DynamicRule>,
        user_context: &UserContext,
        evaluation_context: &EvaluationContext,
    ) -> Result<Vec<RuleEvaluationResult>> {
        let start_time = std::time::Instant::now();
        let mut results = Vec::new();

        info!("Evaluating {} rules for user {}", rules.len(), user_context.user_id);

        // Sort rules by priority (higher priority first)
        let mut sorted_rules = rules;
        sorted_rules.sort_by(|a, b| b.priority.cmp(&a.priority));

        for rule in sorted_rules {
            if !rule.is_active {
                debug!("Skipping inactive rule: {}", rule.id);
                continue;
            }

            // Check cache first
            let cache_key = self.generate_cache_key(&rule, user_context, evaluation_context);
            if let Some(cached) = self.get_cached_evaluation(&cache_key).await {
                debug!("Using cached evaluation for rule {}", rule.id);
                let mut metrics = self.performance_metrics.write().await;
                metrics.cache_hits += 1;
                
                // Create result from cache
                let result = RuleEvaluationResult {
                    rule_id: rule.id,
                    user_id: user_context.user_id,
                    group_id: rule.group_id,
                    evaluation_result: cached.result,
                    confidence_score: cached.confidence,
                    evaluation_reasons: serde_json::json!({"source": "cache", "cached_at": cached.cached_at}),
                    evaluation_time_ms: 0,
                    conditions_evaluated: rule.conditions.len() as i32,
                    conditions_matched: if cached.result { rule.conditions.len() as i32 } else { 0 },
                    actions_executed: serde_json::json!({}),
                    group_assignment_changed: false,
                    evaluation_context: serde_json::to_value(evaluation_context)?,
                };
                results.push(result);
                continue;
            }

            // Evaluate rule
            let rule_start = std::time::Instant::now();
            let result = self.evaluate_single_rule(&rule, user_context, evaluation_context).await?;
            let rule_duration = rule_start.elapsed();

            // Cache the result
            self.cache_evaluation(&cache_key, result.evaluation_result, result.confidence_score, 300).await; // 5 min TTL

            // Update metrics
            let mut metrics = self.performance_metrics.write().await;
            metrics.cache_misses += 1;
            metrics.total_evaluations += 1;
            metrics.total_time_ms += rule_duration.as_millis();

            results.push(result);
        }

        let total_duration = start_time.elapsed();
        info!("Evaluated {} rules in {:?} for user {}", results.len(), total_duration, user_context.user_id);

        Ok(results)
    }

    /// Evaluate a single rule against user context
    #[instrument(skip(self, rule, user_context, evaluation_context))]
    async fn evaluate_single_rule(
        &self,
        rule: &DynamicRule,
        user_context: &UserContext,
        evaluation_context: &EvaluationContext,
    ) -> Result<RuleEvaluationResult> {
        let start_time = std::time::Instant::now();
        let mut conditions_evaluated = 0;
        let mut conditions_matched = 0;
        let mut evaluation_reasons = Vec::new();

        debug!("Evaluating rule: {} ({})", rule.rule_name, rule.id);

        // Pre-check: Handle rule type specific logic
        match rule.rule_type {
            RuleType::Temporal => {
                if !self.evaluate_temporal_conditions(rule, evaluation_context).await? {
                    debug!("Temporal conditions not met for rule {}", rule.id);
                    return self.create_negative_result(rule, user_context, evaluation_context, 
                        "Temporal conditions not met", start_time).await;
                }
            },
            RuleType::EventDriven => {
                if !self.check_event_triggers(rule, evaluation_context).await? {
                    debug!("Event triggers not met for rule {}", rule.id);
                    return self.create_negative_result(rule, user_context, evaluation_context,
                        "Event triggers not met", start_time).await;
                }
            },
            _ => {} // Other types proceed to condition evaluation
        }

        // Evaluate conditions based on logic operator
        let conditions_result = match rule.logic_operator {
            LogicOperator::And => {
                self.evaluate_and_conditions(&rule.conditions, user_context, evaluation_context,
                    &mut conditions_evaluated, &mut conditions_matched, &mut evaluation_reasons).await?
            },
            LogicOperator::Or => {
                self.evaluate_or_conditions(&rule.conditions, user_context, evaluation_context,
                    &mut conditions_evaluated, &mut conditions_matched, &mut evaluation_reasons).await?
            },
            LogicOperator::Not => {
                !self.evaluate_and_conditions(&rule.conditions, user_context, evaluation_context,
                    &mut conditions_evaluated, &mut conditions_matched, &mut evaluation_reasons).await?
            },
            LogicOperator::Xor => {
                self.evaluate_xor_conditions(&rule.conditions, user_context, evaluation_context,
                    &mut conditions_evaluated, &mut conditions_matched, &mut evaluation_reasons).await?
            },
        };

        // Calculate confidence score
        let confidence_score = self.calculate_confidence_score(
            rule, conditions_matched, conditions_evaluated, user_context
        ).await?;

        // Check if confidence meets threshold
        let final_result = conditions_result && confidence_score >= rule.confidence_threshold;

        let evaluation_time = start_time.elapsed().as_millis();

        Ok(RuleEvaluationResult {
            rule_id: rule.id,
            user_id: user_context.user_id,
            group_id: rule.group_id,
            evaluation_result: final_result,
            confidence_score,
            evaluation_reasons: serde_json::json!({
                "conditions_result": conditions_result,
                "confidence_score": confidence_score,
                "confidence_threshold": rule.confidence_threshold,
                "details": evaluation_reasons
            }),
            evaluation_time_ms: evaluation_time,
            conditions_evaluated,
            conditions_matched,
            actions_executed: serde_json::json!({}), // Will be filled by action executor
            group_assignment_changed: false, // Will be updated by assignment service
            evaluation_context: serde_json::to_value(evaluation_context)?,
        })
    }

    /// Evaluate AND logic (all conditions must match)
    async fn evaluate_and_conditions(
        &self,
        conditions: &[RuleCondition],
        user_context: &UserContext,
        evaluation_context: &EvaluationContext,
        conditions_evaluated: &mut i32,
        conditions_matched: &mut i32,
        evaluation_reasons: &mut Vec<serde_json::Value>,
    ) -> Result<bool> {
        for condition in conditions {
            *conditions_evaluated += 1;
            let condition_result = self.evaluate_condition(condition, user_context, evaluation_context).await?;
            
            evaluation_reasons.push(serde_json::json!({
                "condition": condition,
                "result": condition_result.0,
                "confidence": condition_result.1,
                "reason": condition_result.2
            }));

            if condition_result.0 {
                *conditions_matched += 1;
            } else {
                // Early termination for AND - if any condition fails, return false
                return Ok(false);
            }
        }
        Ok(true)
    }

    /// Evaluate OR logic (at least one condition must match)
    async fn evaluate_or_conditions(
        &self,
        conditions: &[RuleCondition],
        user_context: &UserContext,
        evaluation_context: &EvaluationContext,
        conditions_evaluated: &mut i32,
        conditions_matched: &mut i32,
        evaluation_reasons: &mut Vec<serde_json::Value>,
    ) -> Result<bool> {
        for condition in conditions {
            *conditions_evaluated += 1;
            let condition_result = self.evaluate_condition(condition, user_context, evaluation_context).await?;
            
            evaluation_reasons.push(serde_json::json!({
                "condition": condition,
                "result": condition_result.0,
                "confidence": condition_result.1,
                "reason": condition_result.2
            }));

            if condition_result.0 {
                *conditions_matched += 1;
                // Early termination for OR - if any condition succeeds, return true
                return Ok(true);
            }
        }
        Ok(false)
    }

    /// Evaluate XOR logic (exactly one condition must match)
    async fn evaluate_xor_conditions(
        &self,
        conditions: &[RuleCondition],
        user_context: &UserContext,
        evaluation_context: &EvaluationContext,
        conditions_evaluated: &mut i32,
        conditions_matched: &mut i32,
        evaluation_reasons: &mut Vec<serde_json::Value>,
    ) -> Result<bool> {
        let mut matches = 0;

        for condition in conditions {
            *conditions_evaluated += 1;
            let condition_result = self.evaluate_condition(condition, user_context, evaluation_context).await?;
            
            evaluation_reasons.push(serde_json::json!({
                "condition": condition,
                "result": condition_result.0,
                "confidence": condition_result.1,
                "reason": condition_result.2
            }));

            if condition_result.0 {
                *conditions_matched += 1;
                matches += 1;
            }
        }

        Ok(matches == 1) // Exactly one match for XOR
    }

    /// Evaluate a single condition
    #[instrument(skip(self, condition, user_context, evaluation_context))]
    async fn evaluate_condition(
        &self,
        condition: &RuleCondition,
        user_context: &UserContext,
        evaluation_context: &EvaluationContext,
    ) -> Result<(bool, f64, String)> {
        debug!("Evaluating condition: {} {} {:?}", condition.field, 
            serde_json::to_string(&condition.operator)?, condition.value);

        // Extract value from user context based on field path
        let actual_value = self.extract_field_value(&condition.field, user_context, evaluation_context).await?;

        let (result, confidence, reason) = match &condition.operator {
            ConditionOperator::Equals => {
                let matches = actual_value == condition.value;
                (matches, 1.0, format!("Value {} equals {}", actual_value, condition.value))
            },
            ConditionOperator::NotEquals => {
                let matches = actual_value != condition.value;
                (matches, 1.0, format!("Value {} not equals {}", actual_value, condition.value))
            },
            ConditionOperator::GreaterThan => {
                self.evaluate_numeric_comparison(&actual_value, &condition.value, |a, b| a > b)
                    .map(|r| (r, 1.0, format!("Value {} > {}", actual_value, condition.value)))?
            },
            ConditionOperator::GreaterThanOrEqual => {
                self.evaluate_numeric_comparison(&actual_value, &condition.value, |a, b| a >= b)
                    .map(|r| (r, 1.0, format!("Value {} >= {}", actual_value, condition.value)))?
            },
            ConditionOperator::LessThan => {
                self.evaluate_numeric_comparison(&actual_value, &condition.value, |a, b| a < b)
                    .map(|r| (r, 1.0, format!("Value {} < {}", actual_value, condition.value)))?
            },
            ConditionOperator::LessThanOrEqual => {
                self.evaluate_numeric_comparison(&actual_value, &condition.value, |a, b| a <= b)
                    .map(|r| (r, 1.0, format!("Value {} <= {}", actual_value, condition.value)))?
            },
            ConditionOperator::Contains => {
                self.evaluate_string_contains(&actual_value, &condition.value)
            },
            ConditionOperator::StartsWith => {
                self.evaluate_string_starts_with(&actual_value, &condition.value)
            },
            ConditionOperator::Within => {
                self.evaluate_time_within(&actual_value, &condition.value, evaluation_context).await?
            },
            ConditionOperator::WithinWindow => {
                self.evaluate_time_window(&condition.value, evaluation_context).await?
            },
            ConditionOperator::HasPattern => {
                self.evaluate_behavioral_pattern(condition, user_context).await?
            },
            ConditionOperator::ExceedsThreshold => {
                self.evaluate_threshold_condition(condition, user_context).await?
            },
            ConditionOperator::In => {
                self.evaluate_in_array(&actual_value, &condition.value)
            },
            ConditionOperator::Matches => {
                self.evaluate_regex_match(&actual_value, &condition.value)?
            },
            _ => {
                warn!("Unimplemented condition operator: {:?}", condition.operator);
                (false, 0.0, "Operator not implemented".to_string())
            },
        };

        debug!("Condition result: {} (confidence: {}, reason: {})", result, confidence, reason);
        Ok((result, confidence, reason))
    }

    // ============================================================================
    // FIELD VALUE EXTRACTION
    // ============================================================================

    /// Extract value from user context using dot notation field paths
    async fn extract_field_value(
        &self,
        field_path: &str,
        user_context: &UserContext,
        evaluation_context: &EvaluationContext,
    ) -> Result<serde_json::Value> {
        let parts: Vec<&str> = field_path.split('.').collect();
        
        match parts.as_slice() {
            ["user", "id"] => Ok(serde_json::Value::String(user_context.user_id.to_string())),
            ["user", "email"] => Ok(user_context.email.as_ref()
                .map(|e| serde_json::Value::String(e.clone()))
                .unwrap_or(serde_json::Value::Null)),
            ["user", "subscription_tier"] => Ok(user_context.subscription_tier.as_ref()
                .map(|t| serde_json::Value::String(t.clone()))
                .unwrap_or(serde_json::Value::String("free".to_string()))),
            ["user", "last_login"] => Ok(user_context.last_login
                .map(|dt| serde_json::Value::String(dt.to_rfc3339()))
                .unwrap_or(serde_json::Value::Null)),
            ["user", "registration_date"] => Ok(user_context.registration_date
                .map(|dt| serde_json::Value::String(dt.to_rfc3339()))
                .unwrap_or(serde_json::Value::Null)),
            ["user", "behavior_score"] => {
                if let Some(behavioral_data) = &user_context.behavioral_data {
                    Ok(serde_json::Value::Number(serde_json::Number::from_f64(behavioral_data.behavior_score)
                        .unwrap_or_else(|| serde_json::Number::from(0))))
                } else {
                    Ok(serde_json::Value::Number(serde_json::Number::from(0)))
                }
            },
            ["user", "active_days"] => {
                if let Some(behavioral_data) = &user_context.behavioral_data {
                    Ok(serde_json::Value::Number(serde_json::Number::from(behavioral_data.active_days)))
                } else {
                    Ok(serde_json::Value::Number(serde_json::Number::from(0)))
                }
            },
            ["user", "analytics_usage"] => {
                if let Some(behavioral_data) = &user_context.behavioral_data {
                    let usage = behavioral_data.feature_usage_count.get("analytics").unwrap_or(&0);
                    Ok(serde_json::Value::Number(serde_json::Number::from(*usage)))
                } else {
                    Ok(serde_json::Value::Number(serde_json::Number::from(0)))
                }
            },
            ["user", "engagement_level"] => {
                if let Some(behavioral_data) = &user_context.behavioral_data {
                    Ok(behavioral_data.engagement_level.as_ref()
                        .map(|e| serde_json::Value::String(e.clone()))
                        .unwrap_or(serde_json::Value::String("low".to_string())))
                } else {
                    Ok(serde_json::Value::String("low".to_string()))
                }
            },
            ["web3", "token_balance"] => {
                if let Some(web3_data) = &user_context.web3_data {
                    if parts.len() > 2 {
                        let contract = parts[2];
                        let default_balance = "0".to_string();
                        let balance = web3_data.token_balances.get(contract).unwrap_or(&default_balance);
                        Ok(serde_json::Value::String(balance.clone()))
                    } else {
                        Ok(serde_json::Value::String("0".to_string()))
                    }
                } else {
                    Ok(serde_json::Value::String("0".to_string()))
                }
            },
            ["current_time"] => Ok(serde_json::Value::String(evaluation_context.current_time.to_rfc3339())),
            ["environment"] => Ok(serde_json::Value::String(evaluation_context.environment.clone())),
            _ => {
                // Check custom fields
                if parts.len() >= 2 && parts[0] == "custom" {
                    let custom_field = parts[1..].join(".");
                    Ok(user_context.custom_fields.get(&custom_field)
                        .cloned()
                        .unwrap_or(serde_json::Value::Null))
                } else {
                    warn!("Unknown field path: {}", field_path);
                    Ok(serde_json::Value::Null)
                }
            }
        }
    }

    // ============================================================================
    // CONDITION EVALUATION HELPERS
    // ============================================================================

    fn evaluate_numeric_comparison<F>(
        &self,
        actual: &serde_json::Value,
        expected: &serde_json::Value,
        compare: F,
    ) -> Result<bool>
    where
        F: Fn(f64, f64) -> bool,
    {
        let actual_num = match actual {
            serde_json::Value::Number(n) => n.as_f64().unwrap_or(0.0),
            serde_json::Value::String(s) => s.parse::<f64>().unwrap_or(0.0),
            _ => 0.0,
        };

        let expected_num = match expected {
            serde_json::Value::Number(n) => n.as_f64().unwrap_or(0.0),
            serde_json::Value::String(s) => s.parse::<f64>().unwrap_or(0.0),
            _ => 0.0,
        };

        Ok(compare(actual_num, expected_num))
    }

    fn evaluate_string_contains(
        &self,
        actual: &serde_json::Value,
        expected: &serde_json::Value,
    ) -> (bool, f64, String) {
        let actual_str = actual.as_str().unwrap_or("");
        let expected_str = expected.as_str().unwrap_or("");
        
        let contains = actual_str.contains(expected_str);
        (contains, 1.0, format!("String '{}' contains '{}'", actual_str, expected_str))
    }

    fn evaluate_string_starts_with(
        &self,
        actual: &serde_json::Value,
        expected: &serde_json::Value,
    ) -> (bool, f64, String) {
        let actual_str = actual.as_str().unwrap_or("");
        let expected_str = expected.as_str().unwrap_or("");
        
        let starts_with = actual_str.starts_with(expected_str);
        (starts_with, 1.0, format!("String '{}' starts with '{}'", actual_str, expected_str))
    }

    fn evaluate_in_array(
        &self,
        actual: &serde_json::Value,
        expected_array: &serde_json::Value,
    ) -> (bool, f64, String) {
        if let serde_json::Value::Array(arr) = expected_array {
            let in_array = arr.contains(actual);
            (in_array, 1.0, format!("Value {:?} in array {:?}", actual, arr))
        } else {
            (false, 0.0, "Expected array for 'in' operator".to_string())
        }
    }

    fn evaluate_regex_match(
        &self,
        actual: &serde_json::Value,
        pattern: &serde_json::Value,
    ) -> Result<(bool, f64, String)> {
        let actual_str = actual.as_str().unwrap_or("");
        let pattern_str = pattern.as_str().unwrap_or("");
        
        match Regex::new(pattern_str) {
            Ok(regex) => {
                let matches = regex.is_match(actual_str);
                Ok((matches, 1.0, format!("String '{}' matches pattern '{}'", actual_str, pattern_str)))
            },
            Err(e) => {
                error!("Invalid regex pattern '{}': {}", pattern_str, e);
                Ok((false, 0.0, format!("Invalid regex pattern: {}", e)))
            }
        }
    }

    async fn evaluate_time_within(
        &self,
        actual: &serde_json::Value,
        expected: &serde_json::Value,
        evaluation_context: &EvaluationContext,
    ) -> Result<(bool, f64, String)> {
        let expected_str = expected.as_str().unwrap_or("");
        
        if let Some(actual_time_str) = actual.as_str() {
            if let Ok(actual_time) = DateTime::parse_from_rfc3339(actual_time_str) {
                let current_time = evaluation_context.current_time;
                let duration = current_time.signed_duration_since(actual_time.with_timezone(&Utc));
                
                let within_duration = match expected_str {
                    time_str if time_str.ends_with("_days") => {
                        let days: i64 = time_str.replace("_days", "").parse().unwrap_or(0);
                        duration <= chrono::Duration::days(days)
                    },
                    time_str if time_str.ends_with("_hours") => {
                        let hours: i64 = time_str.replace("_hours", "").parse().unwrap_or(0);
                        duration <= chrono::Duration::hours(hours)
                    },
                    time_str if time_str.ends_with("_minutes") => {
                        let minutes: i64 = time_str.replace("_minutes", "").parse().unwrap_or(0);
                        duration <= chrono::Duration::minutes(minutes)
                    },
                    _ => false,
                };
                
                return Ok((within_duration, 1.0, format!("Time {} within {}", actual_time_str, expected_str)));
            }
        }
        
        Ok((false, 0.0, "Invalid time format".to_string()))
    }

    async fn evaluate_time_window(
        &self,
        time_window_config: &serde_json::Value,
        evaluation_context: &EvaluationContext,
    ) -> Result<(bool, f64, String)> {
        if let Some(obj) = time_window_config.as_object() {
            let start_time = obj.get("start").and_then(|v| v.as_str()).unwrap_or("00:00");
            let end_time = obj.get("end").and_then(|v| v.as_str()).unwrap_or("23:59");
            let allowed_days = obj.get("days")
                .and_then(|v| v.as_array())
                .map(|arr| arr.iter().filter_map(|v| v.as_str()).collect::<Vec<_>>())
                .unwrap_or_else(|| vec!["mon", "tue", "wed", "thu", "fri", "sat", "sun"]);

            let current_time = evaluation_context.current_time;
            let current_hour_minute = format!("{:02}:{:02}", current_time.hour(), current_time.minute());
            let current_weekday = current_time.weekday();

            // Check if current day is allowed
            let day_abbr = match current_weekday {
                Weekday::Mon => "mon",
                Weekday::Tue => "tue", 
                Weekday::Wed => "wed",
                Weekday::Thu => "thu",
                Weekday::Fri => "fri",
                Weekday::Sat => "sat",
                Weekday::Sun => "sun",
            };

            let day_allowed = allowed_days.contains(&day_abbr);
            let time_in_range = current_hour_minute.as_str() >= start_time && current_hour_minute.as_str() <= end_time;
            
            let within_window = day_allowed && time_in_range;
            
            return Ok((within_window, 1.0, format!("Current time {} on {} within window {}-{} on {:?}", 
                current_hour_minute, day_abbr, start_time, end_time, allowed_days)));
        }
        
        Ok((false, 0.0, "Invalid time window configuration".to_string()))
    }

    // ============================================================================
    // BEHAVIORAL & ADVANCED CONDITION EVALUATION
    // ============================================================================

    async fn evaluate_behavioral_pattern(
        &self,
        condition: &RuleCondition,
        user_context: &UserContext,
    ) -> Result<(bool, f64, String)> {
        if let Some(behavioral_data) = &user_context.behavioral_data {
            let pattern_name = condition.value.as_str().unwrap_or("");
            
            match pattern_name {
                "power_user" => {
                    let is_power_user = behavioral_data.behavior_score > 0.8 
                        && behavioral_data.login_frequency > 20
                        && behavioral_data.session_duration_minutes > 60;
                    Ok((is_power_user, 0.9, "Power user pattern detected".to_string()))
                },
                "regular_user" => {
                    let is_regular = behavioral_data.behavior_score > 0.5 
                        && behavioral_data.active_days > 7;
                    Ok((is_regular, 0.8, "Regular user pattern detected".to_string()))
                },
                "churning_user" => {
                    let is_churning = behavioral_data.behavior_score < 0.3 
                        && behavioral_data.active_days < 3;
                    Ok((is_churning, 0.7, "Churning user pattern detected".to_string()))
                },
                _ => Ok((false, 0.0, format!("Unknown pattern: {}", pattern_name)))
            }
        } else {
            Ok((false, 0.0, "No behavioral data available".to_string()))
        }
    }

    async fn evaluate_threshold_condition(
        &self,
        condition: &RuleCondition,
        user_context: &UserContext,
    ) -> Result<(bool, f64, String)> {
        if let Some(behavioral_data) = &user_context.behavioral_data {
            let threshold = condition.value.as_f64().unwrap_or(0.0);
            let actual_score = behavioral_data.behavior_score;
            
            let exceeds = actual_score > threshold;
            let confidence = if exceeds { (actual_score - threshold).min(1.0) } else { 0.0 };
            
            Ok((exceeds, confidence, format!("Behavior score {} vs threshold {}", actual_score, threshold)))
        } else {
            Ok((false, 0.0, "No behavioral data for threshold evaluation".to_string()))
        }
    }

    // ============================================================================
    // TEMPORAL RULE EVALUATION
    // ============================================================================

    async fn evaluate_temporal_conditions(
        &self,
        rule: &DynamicRule,
        evaluation_context: &EvaluationContext,
    ) -> Result<bool> {
        if let Some(schedule_config) = &rule.temporal_schedule {
            // For now, simple implementation - can be extended with cron parsing
            if let Some(obj) = schedule_config.as_object() {
                if let Some(active_hours) = obj.get("active_hours") {
                    if let Some(hours_array) = active_hours.as_array() {
                        let current_hour = evaluation_context.current_time.hour();
                        let hour_allowed = hours_array.iter()
                            .any(|h| h.as_u64().map(|hour| hour as u32) == Some(current_hour));
                        
                        return Ok(hour_allowed);
                    }
                }
            }
        }
        
        // If no temporal schedule, always allow
        Ok(true)
    }

    async fn check_event_triggers(
        &self,
        rule: &DynamicRule,
        _evaluation_context: &EvaluationContext,
    ) -> Result<bool> {
        // Event-driven rules would check for specific events
        // This is a placeholder implementation
        if rule.behavioral_triggers.is_some() {
            // Could check for login events, purchase events, etc.
            Ok(true) // Placeholder
        } else {
            Ok(true)
        }
    }

    // ============================================================================
    // CONFIDENCE SCORING
    // ============================================================================

    async fn calculate_confidence_score(
        &self,
        rule: &DynamicRule,
        conditions_matched: i32,
        conditions_evaluated: i32,
        user_context: &UserContext,
    ) -> Result<f64> {
        if conditions_evaluated == 0 {
            return Ok(0.0);
        }

        // Base confidence from condition match ratio
        let base_confidence = conditions_matched as f64 / conditions_evaluated as f64;

        // Adjust based on rule type and user data quality
        let type_multiplier = match rule.rule_type {
            RuleType::MlPowered => 0.9, // ML has inherent uncertainty
            RuleType::Behavioral => {
                if user_context.behavioral_data.is_some() { 0.95 } else { 0.3 }
            },
            RuleType::Temporal => 1.0, // Time-based rules are very certain
            RuleType::Conditional => 1.0, // Direct conditions are certain
            RuleType::EventDriven => 0.8, // Events can have noise
            RuleType::Composite => 0.85, // Composite rules have multiple factors
        };

        // Factor in rule's historical success rate
        let historical_factor = if rule.evaluation_count > 10 {
            (rule.success_rate * 0.3) + 0.7 // Weight historical success but not too heavily
        } else {
            1.0 // No historical data yet
        };

        let final_confidence = (base_confidence * type_multiplier * historical_factor).min(1.0);

        Ok(final_confidence)
    }

    // ============================================================================
    // CACHING UTILITIES
    // ============================================================================

    fn generate_cache_key(
        &self,
        rule: &DynamicRule,
        user_context: &UserContext,
        evaluation_context: &EvaluationContext,
    ) -> String {
        // Generate a deterministic cache key based on rule, user, and relevant context
        format!("rule_{}:user_{}:env_{}:time_{}", 
            rule.id, 
            user_context.user_id,
            evaluation_context.environment,
            evaluation_context.current_time.timestamp() / 300 // 5-minute buckets
        )
    }

    async fn get_cached_evaluation(&self, cache_key: &str) -> Option<CachedEvaluation> {
        let cache = self.cache.read().await;
        if let Some(cached) = cache.get(cache_key) {
            if cached.cached_at + chrono::Duration::seconds(cached.ttl_seconds as i64) > Utc::now() {
                return Some(cached.clone());
            }
        }
        None
    }

    async fn cache_evaluation(&self, cache_key: &str, result: bool, confidence: f64, ttl_seconds: u64) {
        let mut cache = self.cache.write().await;
        cache.insert(cache_key.to_string(), CachedEvaluation {
            result,
            confidence,
            cached_at: Utc::now(),
            ttl_seconds,
        });

        // Simple cache cleanup - remove expired entries if cache gets large
        if cache.len() > 10000 {
            let now = Utc::now();
            cache.retain(|_, cached| {
                cached.cached_at + chrono::Duration::seconds(cached.ttl_seconds as i64) > now
            });
        }
    }

    // ============================================================================
    // UTILITY METHODS
    // ============================================================================

    async fn create_negative_result(
        &self,
        rule: &DynamicRule,
        user_context: &UserContext,
        evaluation_context: &EvaluationContext,
        reason: &str,
        start_time: std::time::Instant,
    ) -> Result<RuleEvaluationResult> {
        Ok(RuleEvaluationResult {
            rule_id: rule.id,
            user_id: user_context.user_id,
            group_id: rule.group_id,
            evaluation_result: false,
            confidence_score: 1.0, // We're confident it doesn't match
            evaluation_reasons: serde_json::json!({"reason": reason}),
            evaluation_time_ms: start_time.elapsed().as_millis(),
            conditions_evaluated: 0,
            conditions_matched: 0,
            actions_executed: serde_json::json!({}),
            group_assignment_changed: false,
            evaluation_context: serde_json::to_value(evaluation_context)?,
        })
    }

    /// Get performance metrics for monitoring
    pub async fn get_performance_metrics(&self) -> PerformanceMetrics {
        self.performance_metrics.read().await.clone()
    }

    /// Clear cache (useful for testing or manual cache management)
    pub async fn clear_cache(&self) {
        self.cache.write().await.clear();
    }
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_user_context() -> UserContext {
        UserContext {
            user_id: Uuid::new_v4(),
            email: Some("test@example.com".to_string()),
            subscription_tier: Some("premium".to_string()),
            last_login: Some(Utc::now() - chrono::Duration::hours(2)),
            registration_date: Some(Utc::now() - chrono::Duration::days(30)),
            permissions_used: vec!["epsx:analytics:view".to_string()],
            behavioral_data: Some(UserBehavioralData {
                login_frequency: 25,
                session_duration_minutes: 45,
                feature_usage_count: {
                    let mut map = HashMap::new();
                    map.insert("analytics".to_string(), 150);
                    map
                },
                api_calls_count: 1000,
                active_days: 20,
                bounce_rate: 0.1,
                permissions_used: vec!["epsx:analytics:view".to_string()],
                permission_success_rate: 0.98,
                peak_usage_hours: vec![9, 10, 14, 15, 16],
                preferred_features: vec!["analytics".to_string()],
                usage_patterns: None,
                user_cluster: Some("power_users".to_string()),
                behavior_score: 0.85,
                engagement_level: Some("high".to_string()),
            }),
            web3_data: None,
            custom_fields: HashMap::new(),
        }
    }

    fn create_test_evaluation_context() -> EvaluationContext {
        EvaluationContext {
            current_time: Utc::now(),
            user_timezone: Some("UTC".to_string()),
            device_type: Some("desktop".to_string()),
            location: Some("US".to_string()),
            environment: "production".to_string(),
            feature_flags: vec![],
            session_data: None,
        }
    }

    #[tokio::test]
    async fn test_simple_condition_evaluation() {
        let engine = DynamicGroupRulesEngine::new();
        let user_context = create_test_user_context();
        let evaluation_context = create_test_evaluation_context();

        let condition = RuleCondition {
            field: "user.subscription_tier".to_string(),
            operator: ConditionOperator::Equals,
            value: serde_json::Value::String("premium".to_string()),
            weight: None,
            aggregation_period: None,
            contract: None,
            custom_config: None,
        };

        let result = engine.evaluate_condition(&condition, &user_context, &evaluation_context).await.unwrap();
        assert!(result.0); // Should match
        assert_eq!(result.1, 1.0); // Full confidence
    }

    #[tokio::test]
    async fn test_behavioral_pattern_evaluation() {
        let engine = DynamicGroupRulesEngine::new();
        let user_context = create_test_user_context();
        let evaluation_context = create_test_evaluation_context();

        let condition = RuleCondition {
            field: "user.behavior".to_string(),
            operator: ConditionOperator::HasPattern,
            value: serde_json::Value::String("power_user".to_string()),
            weight: None,
            aggregation_period: None,
            contract: None,
            custom_config: None,
        };

        let result = engine.evaluate_behavioral_pattern(&condition, &user_context).await.unwrap();
        assert!(result.0); // Should match power user pattern
        assert!(result.1 > 0.8); // High confidence
    }

    #[tokio::test]
    async fn test_numeric_comparison() {
        let engine = DynamicGroupRulesEngine::new();
        let user_context = create_test_user_context();
        let evaluation_context = create_test_evaluation_context();

        let condition = RuleCondition {
            field: "user.analytics_usage".to_string(),
            operator: ConditionOperator::GreaterThan,
            value: serde_json::Value::Number(serde_json::Number::from(100)),
            weight: None,
            aggregation_period: None,
            contract: None,
            custom_config: None,
        };

        let result = engine.evaluate_condition(&condition, &user_context, &evaluation_context).await.unwrap();
        assert!(result.0); // 150 > 100
    }

    #[tokio::test]
    async fn test_time_window_evaluation() {
        let engine = DynamicGroupRulesEngine::new();
        let _user_context = create_test_user_context();
        let evaluation_context = create_test_evaluation_context();

        let time_window = serde_json::json!({
            "start": "00:00",
            "end": "23:59",
            "days": ["mon", "tue", "wed", "thu", "fri", "sat", "sun"]
        });

        let result = engine.evaluate_time_window(&time_window, &evaluation_context).await.unwrap();
        assert!(result.0); // Should always be within 24/7 window
    }
}