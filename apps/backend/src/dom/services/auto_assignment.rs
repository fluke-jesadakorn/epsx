// Auto-assignment engine for permission_profile assignment during user registration
use crate::dom::values::UserId;
use crate::dom::entities::permission_profile::PermissionProfileId;
use crate::app::ports::repositories::{PermissionProfileRepo, UserRepo, PermissionAssignmentRepo};
use std::sync::Arc;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistrationContext {
    pub email: String,
    pub package_tier: PackageTier,
    pub referral_code: Option<String>,
    pub source: String, // 'web_registration', 'api', 'admin_create'
    pub region: Option<String>,
    pub email_domain: String,
    pub user_agent: Option<String>,
    pub utm_source: Option<String>,
    pub utm_campaign: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PackageTier {
    Bronze,
    Silver,
    Gold,
    Platinum,
}

impl From<&str> for PackageTier {
    fn from(tier: &str) -> Self {
        match tier.to_lowercase().as_str() {
            "silver" => PackageTier::Silver,
            "gold" => PackageTier::Gold,
            "platinum" => PackageTier::Platinum,
            _ => PackageTier::Bronze, // Default
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutoAssignmentRule {
    pub permission_profile_id: PermissionProfileId,
    pub package_tiers: Vec<PackageTier>,
    pub triggers: Vec<AssignmentTrigger>,
    pub priority: i32,
    pub expires_after_days: Option<i32>,
    pub requires_payment: bool,
    pub variables: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AssignmentTrigger {
    // Tier-based triggers
    TierMatch(PackageTier),
    TierMinimum(PackageTier),
    
    // Email-based triggers
    EmailDomain(String),
    EmailPattern(String), // Regex pattern
    
    // Referral triggers
    ReferralCode(String),
    ReferralProgram(String),
    
    // UTM/Source triggers
    UtmSource(String),
    UtmCampaign(String),
    SourceMatch(String),
    
    // Geographic triggers
    Region(String),
    Country(String),
    
    // Always applies (default assignment)
    Always,
}

#[derive(Debug, Clone)]
pub struct AssignmentResult {
    pub permission_profile_id: PermissionProfileId,
    pub feature_id: String,
    pub success: bool,
    pub reason: String,
    pub variables_applied: serde_json::Value,
    pub expires_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Debug)]
pub struct AssignmentResults {
    pub assignments: Vec<AssignmentResult>,
    pub total_assigned: u32,
    pub total_failed: u32,
}

#[derive(Debug, thiserror::Error)]
pub enum AutoAssignmentError {
    #[error("Repository error: {0}")]
    Repository(String),
    
    #[error("Permission profile not found: {0}")]
    PermissionProfileNotFound(PermissionProfileId),
    
    #[error("User not found: {0}")]
    UserNotFound(UserId),
    
    #[error("Assignment rule validation failed: {0}")]
    RuleValidation(String),
    
    #[error("Permission profile variable substitution failed: {0}")]
    VariableSubstitution(String),
    
    #[error("Assignment already exists")]
    AlreadyExists,
}

pub struct AutoAssignmentEngine {
    permission_profile_repo: Arc<dyn PermissionProfileRepo>,
    assignment_repo: Arc<dyn PermissionAssignmentRepo>,
    user_repo: Arc<dyn UserRepo>,
}

impl AutoAssignmentEngine {
    pub fn new(
        permission_profile_repo: Arc<dyn PermissionProfileRepo>,
        assignment_repo: Arc<dyn PermissionAssignmentRepo>,
        user_repo: Arc<dyn UserRepo>,
    ) -> Self {
        Self {
            permission_profile_repo,
            assignment_repo,
            user_repo,
        }
    }

    /// Process user registration and apply auto-assignment rules
    pub async fn process_registration(
        &self,
        user_id: &UserId,
        context: &RegistrationContext,
    ) -> Result<AssignmentResults, AutoAssignmentError> {
        // 1. Get user details from repository to enrich context
        let user = self.user_repo
            .find_by_id(user_id)
            .await
            .map_err(|e| AutoAssignmentError::Repository(e.to_string()))?;
        
        tracing::info!(
            "Processing auto-assignment for user {} with package tier {:?}, email domain: {}",
            user_id, context.package_tier, context.email_domain
        );
        
        // 2. Create enriched context with user data
        let enriched_context = self.enrich_registration_context(context, &user).await?;
        
        // 3. Get auto-assignment rules for package tier
        let rules = self.get_auto_assignment_rules(&enriched_context.package_tier).await?;
        
        tracing::info!("Found {} potential auto-assignment rules for evaluation", rules.len());
        
        // 4. Evaluate registration triggers 
        let triggered_permission_profiles = self.evaluate_registration_triggers(&enriched_context, &rules).await?;
        
        tracing::info!("Triggered {} permission profiles for assignment", triggered_permission_profiles.len());
        
        // 5. Apply permission profiles with variable substitution
        let mut results = Vec::new();
        for assignment_rule in triggered_permission_profiles {
            match self.assign_permission_profile_to_user(user_id, &assignment_rule, &enriched_context).await {
                Ok(result) => {
                    tracing::info!(
                        "Successfully assigned permission profile {} to user {}",
                        assignment_rule.permission_profile_id.value(), user_id
                    );
                    results.push(result);
                },
                Err(AutoAssignmentError::AlreadyExists) => {
                    tracing::info!(
                        "Permission profile {} already assigned to user {} - skipping",
                        assignment_rule.permission_profile_id.value(), user_id
                    );
                    // Skip duplicates without error
                    continue;
                },
                Err(e) => {
                    // Log error but continue with other assignments
                    tracing::warn!("Failed to assign permission profile {}: {}", assignment_rule.permission_profile_id.value(), e);
                    results.push(AssignmentResult {
                        permission_profile_id: assignment_rule.permission_profile_id.clone(),
                        feature_id: "unknown".to_string(),
                        success: false,
                        reason: e.to_string(),
                        variables_applied: serde_json::Value::Null,
                        expires_at: None,
                    });
                }
            }
        }
        
        let total_assigned = results.iter().filter(|r| r.success).count() as u32;
        let total_failed = results.len() as u32 - total_assigned;
        
        tracing::info!(
            "Auto-assignment completed for user {}: {} successful, {} failed",
            user_id, total_assigned, total_failed
        );
        
        Ok(AssignmentResults {
            assignments: results,
            total_assigned,
            total_failed,
        })
    }
    
    /// Enrich registration context with user data from repository
    async fn enrich_registration_context(
        &self,
        context: &RegistrationContext,
        user: &crate::dom::entities::User,
    ) -> Result<RegistrationContext, AutoAssignmentError> {
        // Extract additional context from user entity
        let _registration_time = chrono::Utc::now(); // Could be extracted from user if stored
        let actual_email_domain = user.email().value()
            .split('@')
            .nth(1)
            .unwrap_or(&context.email_domain)
            .to_string();
        
        // Create enriched context
        let mut enriched_context = context.clone();
        enriched_context.email = user.email().value().to_string();
        enriched_context.email_domain = actual_email_domain;
        
        // Add user-specific data for condition evaluation
        // This could include signup location, device info, etc. from user metadata
        tracing::debug!(
            "Enriched registration context for user {}: email_domain = {}, package_tier = {:?}",
            user.id(), enriched_context.email_domain, enriched_context.package_tier
        );
        
        Ok(enriched_context)
    }

    /// Get auto-assignment rules for a specific package tier
    async fn get_auto_assignment_rules(
        &self,
        package_tier: &PackageTier,
    ) -> Result<Vec<AutoAssignmentRule>, AutoAssignmentError> {
        // Get all active permission profiles
        let permission_profiles = self.permission_profile_repo
            .get_by_category(&crate::dom::entities::permission_profile::PermissionProfileCategory::System)
            .await
            .map_err(|e| AutoAssignmentError::Repository(e.to_string()))?;

        let mut rules = Vec::new();
        
        for permission_profile in permission_profiles {
            // For now, create simple auto-assignment rules based on the profile's target tier
            if let Ok(rule) = self.create_simple_assignment_rule(&permission_profile, package_tier) {
                rules.push(rule);
            }
        }
        
        // Sort by priority (higher priority first)
        rules.sort_by(|a, b| b.priority.cmp(&a.priority));
        
        Ok(rules)
    }

    /// Evaluate registration triggers to determine which permission profiles should be assigned
    async fn evaluate_registration_triggers(
        &self,
        context: &RegistrationContext,
        rules: &[AutoAssignmentRule],
    ) -> Result<Vec<AutoAssignmentRule>, AutoAssignmentError> {
        let mut triggered_rules = Vec::new();
        
        for rule in rules {
            let mut should_trigger = false;
            
            // Check if any trigger matches
            for trigger in &rule.triggers {
                if self.evaluate_trigger(trigger, context) {
                    should_trigger = true;
                    break;
                }
            }
            
            if should_trigger {
                triggered_rules.push(rule.clone());
            }
        }
        
        Ok(triggered_rules)
    }

    /// Apply a permission profile assignment to a user
    async fn assign_permission_profile_to_user(
        &self,
        user_id: &UserId,
        rule: &AutoAssignmentRule,
        context: &RegistrationContext,
    ) -> Result<AssignmentResult, AutoAssignmentError> {
        // Get the permission profile
        let permission_profile = self.permission_profile_repo
            .get(&rule.permission_profile_id)
            .await
            .map_err(|e| AutoAssignmentError::Repository(e.to_string()))?
            .ok_or_else(|| AutoAssignmentError::PermissionProfileNotFound(rule.permission_profile_id.clone()))?;

        // Check if user already has this permission profile assigned using the new repository
        let existing_assignments = self.assignment_repo
            .get_user_assignments(user_id)
            .await
            .map_err(|e| AutoAssignmentError::Repository(e.to_string()))?;
            
        let already_assigned = existing_assignments.iter()
            .any(|assignment| assignment.permission_profile_id == rule.permission_profile_id && assignment.is_active);
            
        if already_assigned {
            return Err(AutoAssignmentError::AlreadyExists);
        }

        // Calculate expiration
        let expires_at = rule.expires_after_days.map(|days| {
            chrono::Utc::now() + chrono::Duration::days(days as i64)
        });

        // Apply variable substitution
        let variables = self.substitute_variables(&rule.variables, context)?;

        // Extract feature ID from permission profile (use name as feature ID)
        let feature_id = permission_profile.name().to_string();

        // Create assignment record using the permission assignment repository
        let reason = Some(format!("Auto-assigned during registration: {}", context.source));

        self.assignment_repo
            .assign_permission_profile(
                user_id,
                &rule.permission_profile_id,
                user_id, // assigned_by - using user_id as this is auto-assignment
                expires_at,
                reason
            )
            .await
            .map_err(|e| AutoAssignmentError::Repository(e.to_string()))?;

        tracing::info!(
            "Auto-assigned permission profile {} (feature: {}) to user {} via registration",
            permission_profile.name(), feature_id, user_id
        );

        Ok(AssignmentResult {
            permission_profile_id: rule.permission_profile_id.clone(),
            feature_id,
            success: true,
            reason: format!("Auto-assigned during registration from {}", context.source),
            variables_applied: variables,
            expires_at,
        })
    }


    /// Create a simple assignment rule from permission profile
    fn create_simple_assignment_rule(
        &self,
        permission_profile: &crate::dom::entities::permission_profile::PermissionProfile,
        package_tier: &PackageTier,
    ) -> Result<AutoAssignmentRule, AutoAssignmentError> {
        // Convert our PackageTier to the entity's PackageTier
        let entity_tier = match package_tier {
            PackageTier::Bronze => crate::dom::entities::iam::PackageTier::Bronze,
            PackageTier::Silver => crate::dom::entities::iam::PackageTier::Silver,
            PackageTier::Gold => crate::dom::entities::iam::PackageTier::Gold,
            PackageTier::Platinum => crate::dom::entities::iam::PackageTier::Admin,
        };
        
        // Only create rule if tiers match
        if permission_profile.target_tier() != &entity_tier {
            return Err(AutoAssignmentError::RuleValidation("Tier mismatch".to_string()));
        }
        
        Ok(AutoAssignmentRule {
            permission_profile_id: permission_profile.id().clone(),
            package_tiers: vec![package_tier.clone()],
            triggers: vec![AssignmentTrigger::TierMatch(package_tier.clone())],
            priority: 0,
            expires_after_days: None,
            requires_payment: false,
            variables: serde_json::Value::Object(serde_json::Map::new()),
        })
    }



    /// Evaluate if a trigger condition matches the registration context
    fn evaluate_trigger(&self, trigger: &AssignmentTrigger, context: &RegistrationContext) -> bool {
        match trigger {
            AssignmentTrigger::TierMatch(tier) => {
                std::mem::discriminant(&context.package_tier) == std::mem::discriminant(tier)
            }
            AssignmentTrigger::EmailDomain(domain) => {
                context.email_domain.eq_ignore_ascii_case(domain)
            }
            AssignmentTrigger::ReferralCode(code) => {
                context.referral_code.as_ref().map_or(false, |c| c == code)
            }
            AssignmentTrigger::UtmSource(source) => {
                context.utm_source.as_ref().map_or(false, |s| s == source)
            }
            AssignmentTrigger::UtmCampaign(campaign) => {
                context.utm_campaign.as_ref().map_or(false, |c| c == campaign)
            }
            AssignmentTrigger::SourceMatch(source) => {
                context.source == *source
            }
            AssignmentTrigger::Region(region) => {
                context.region.as_ref().map_or(false, |r| r == region)
            }
            AssignmentTrigger::Always => true,
            _ => false, // TODO: Implement other triggers
        }
    }

    /// Substitute variables in configuration
    fn substitute_variables(
        &self,
        variables: &serde_json::Value,
        context: &RegistrationContext,
    ) -> Result<serde_json::Value, AutoAssignmentError> {
        // Simple variable substitution - can be enhanced with a more advanced engine
        let mut substituted = variables.clone();
        
        if let Some(obj) = substituted.as_object_mut() {
            for (_key, value) in obj.iter_mut() {
                if let Some(permission_profile_str) = value.as_str() {
                    let substituted_str = permission_profile_str
                        .replace("{{package_tier}}", &format!("{:?}", context.package_tier))
                        .replace("{{email}}", &context.email)
                        .replace("{{source}}", &context.source)
                        .replace("{{email_domain}}", &context.email_domain);
                    
                    *value = serde_json::Value::String(substituted_str);
                }
            }
        }
        
        Ok(substituted)
    }
}

// Auto-assignment integration completed - using PermissionAssignmentRepo port trait for clean architecture

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn should_parse_package_tier_from_string() {
        assert!(matches!(PackageTier::from("bronze"), PackageTier::Bronze));
        assert!(matches!(PackageTier::from("SILVER"), PackageTier::Silver));
        assert!(matches!(PackageTier::from("gold"), PackageTier::Gold));
        assert!(matches!(PackageTier::from("platinum"), PackageTier::Platinum));
        assert!(matches!(PackageTier::from("unknown"), PackageTier::Bronze));
    }

    #[test]
    fn should_evaluate_email_domain_trigger() {
        let engine = create_test_engine();
        let context = create_test_context();
        
        let trigger = AssignmentTrigger::EmailDomain("example.com".to_string());
        assert!(engine.evaluate_trigger(&trigger, &context));
        
        let trigger = AssignmentTrigger::EmailDomain("other.com".to_string());
        assert!(!engine.evaluate_trigger(&trigger, &context));
    }

    #[test]
    fn should_evaluate_tier_match_trigger() {
        let engine = create_test_engine();
        let context = create_test_context();
        
        let trigger = AssignmentTrigger::TierMatch(PackageTier::Silver);
        assert!(engine.evaluate_trigger(&trigger, &context));
        
        let trigger = AssignmentTrigger::TierMatch(PackageTier::Gold);
        assert!(!engine.evaluate_trigger(&trigger, &context));
    }

    fn create_test_engine() -> AutoAssignmentEngine {
        // Mock implementation for testing
        unimplemented!("Test implementation needed")
    }

    fn create_test_context() -> RegistrationContext {
        RegistrationContext {
            email: "test@example.com".to_string(),
            package_tier: PackageTier::Silver,
            referral_code: None,
            source: "web_registration".to_string(),
            region: Some("US".to_string()),
            email_domain: "example.com".to_string(),
            user_agent: None,
            utm_source: None,
            utm_campaign: None,
        }
    }
}