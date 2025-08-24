// Package tier-based permission system

use async_trait::async_trait;
use chrono::{DateTime, Utc, Datelike, Timelike};
use serde::{Serialize, Deserialize};
use std::collections::{HashMap, HashSet};

use crate::dom::values::UserId;
use crate::dom::entities::iam::PackageTier;
use super::core::{Permission, PermissionContext, PermissionDecision, PermissionValidator, EffectivePermissions};
use super::errors::{PermissionError, ValidationError};

/// Tier feature enumeration
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TierFeature {
    BasicTrading,
    AdvancedAnalytics,
    ApiAccess,
    PrioritySupport,
    AdvancedOrders,
    PortfolioTools,
    ResearchReports,
    InstitutionalFeatures,
    Custom(String),
}

/// Tier-specific limits
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TierLimit {
    pub feature: TierFeature,
    pub limit_type: TierLimitType,
    pub max_value: i64,
    pub current_value: i64,
    pub reset_period: Option<TierResetPeriod>,
    pub burst_allowance: Option<i64>,
}

/// Tier limit types
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TierLimitType {
    RequestsPerMinute,
    RequestsPerHour,
    RequestsPerDay,
    ConcurrentConnections,
    DataExportMB,
    StorageMB,
    ApiCalls,
    AdvancedFeatures,
    SupportTickets,
    Custom(String),
}

/// Tier reset periods
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TierResetPeriod {
    Minute,
    Hour,
    Day,
    Week,
    Month,
    Year,
    Never,
}

/// Tier access configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TierAccess {
    pub user_id: UserId,
    pub package_tier: PackageTier,
    pub enabled_features: HashSet<TierFeature>,
    pub limits: Vec<TierLimit>,
    pub upgraded_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
    pub auto_renew: bool,
    pub trial_mode: bool,
}

/// Package tier validator
pub struct PackageTierValidator {
    /// User tier access configurations
    tier_access: HashMap<UserId, TierAccess>,
    /// Tier feature mappings
    tier_features: HashMap<PackageTier, HashSet<TierFeature>>,
    /// Default limits per tier
    default_limits: HashMap<PackageTier, Vec<TierLimit>>,
    /// Tier hierarchy for upgrades
    tier_hierarchy: Vec<PackageTier>,
    /// Configuration
    config: PackageTierConfig,
}

/// Package tier validation configuration
#[derive(Debug, Clone)]
pub struct PackageTierConfig {
    /// Enable feature inheritance from lower tiers
    pub enable_inheritance: bool,
    /// Enable trial mode features
    pub enable_trials: bool,
    /// Grace period after tier expiration (seconds)
    pub grace_period_seconds: u64,
    /// Default trial period (days)
    pub trial_period_days: u32,
    /// Enable burst limits
    pub enable_burst_limits: bool,
}

// Tier-specific feature definitions

/// Free tier features
pub mod free_tier {
    use super::*;
    
    pub const BASIC_DASHBOARD: &str = "free:dashboard";
    pub const LIMITED_DATA: &str = "free:limited-data";
    pub const COMMUNITY_SUPPORT: &str = "free:community-support";
    
    pub fn get_default_features() -> HashSet<TierFeature> {
        [TierFeature::BasicTrading].iter().cloned().collect()
    }
    
    pub fn get_default_limits() -> Vec<TierLimit> {
        vec![
            TierLimit {
                feature: TierFeature::ApiAccess,
                limit_type: TierLimitType::RequestsPerHour,
                max_value: 100,
                current_value: 0,
                reset_period: Some(TierResetPeriod::Hour),
                burst_allowance: Some(10),
            },
            TierLimit {
                feature: TierFeature::BasicTrading,
                limit_type: TierLimitType::ConcurrentConnections,
                max_value: 1,
                current_value: 0,
                reset_period: None,
                burst_allowance: None,
            },
        ]
    }
}

/// Bronze tier features
pub mod bronze_tier {
    use super::*;
    
    pub const BASIC_ANALYTICS: &str = "bronze:basic-analytics";
    pub const EMAIL_SUPPORT: &str = "bronze:email-support";
    pub const EXTENDED_DATA: &str = "bronze:extended-data";
    
    pub fn get_default_features() -> HashSet<TierFeature> {
        [
            TierFeature::BasicTrading,
            TierFeature::ApiAccess,
        ].iter().cloned().collect()
    }
    
    pub fn get_default_limits() -> Vec<TierLimit> {
        vec![
            TierLimit {
                feature: TierFeature::ApiAccess,
                limit_type: TierLimitType::RequestsPerHour,
                max_value: 1000,
                current_value: 0,
                reset_period: Some(TierResetPeriod::Hour),
                burst_allowance: Some(100),
            },
            TierLimit {
                feature: TierFeature::BasicTrading,
                limit_type: TierLimitType::ConcurrentConnections,
                max_value: 3,
                current_value: 0,
                reset_period: None,
                burst_allowance: None,
            },
        ]
    }
}

/// Silver tier features
pub mod silver_tier {
    use super::*;
    
    pub const ADVANCED_CHARTS: &str = "silver:advanced-charts";
    pub const PRIORITY_EMAIL: &str = "silver:priority-email";
    pub const REAL_TIME_DATA: &str = "silver:real-time-data";
    
    pub fn get_default_features() -> HashSet<TierFeature> {
        [
            TierFeature::BasicTrading,
            TierFeature::AdvancedAnalytics,
            TierFeature::ApiAccess,
            TierFeature::PortfolioTools,
        ].iter().cloned().collect()
    }
    
    pub fn get_default_limits() -> Vec<TierLimit> {
        vec![
            TierLimit {
                feature: TierFeature::ApiAccess,
                limit_type: TierLimitType::RequestsPerHour,
                max_value: 5000,
                current_value: 0,
                reset_period: Some(TierResetPeriod::Hour),
                burst_allowance: Some(500),
            },
            TierLimit {
                feature: TierFeature::AdvancedAnalytics,
                limit_type: TierLimitType::RequestsPerDay,
                max_value: 1000,
                current_value: 0,
                reset_period: Some(TierResetPeriod::Day),
                burst_allowance: Some(100),
            },
        ]
    }
}

/// Gold tier features
pub mod gold_tier {
    use super::*;
    
    pub const PREMIUM_RESEARCH: &str = "gold:premium-research";
    pub const PHONE_SUPPORT: &str = "gold:phone-support";
    pub const CUSTOM_ALERTS: &str = "gold:custom-alerts";
    pub const PORTFOLIO_ANALYSIS: &str = "gold:portfolio-analysis";
    
    pub fn get_default_features() -> HashSet<TierFeature> {
        [
            TierFeature::BasicTrading,
            TierFeature::AdvancedAnalytics,
            TierFeature::ApiAccess,
            TierFeature::PrioritySupport,
            TierFeature::AdvancedOrders,
            TierFeature::PortfolioTools,
            TierFeature::ResearchReports,
        ].iter().cloned().collect()
    }
    
    pub fn get_default_limits() -> Vec<TierLimit> {
        vec![
            TierLimit {
                feature: TierFeature::ApiAccess,
                limit_type: TierLimitType::RequestsPerHour,
                max_value: 20000,
                current_value: 0,
                reset_period: Some(TierResetPeriod::Hour),
                burst_allowance: Some(2000),
            },
            TierLimit {
                feature: TierFeature::AdvancedAnalytics,
                limit_type: TierLimitType::RequestsPerDay,
                max_value: 10000,
                current_value: 0,
                reset_period: Some(TierResetPeriod::Day),
                burst_allowance: Some(1000),
            },
        ]
    }
}

/// Platinum tier features
pub mod platinum_tier {
    use super::*;
    
    pub const DEDICATED_SUPPORT: &str = "platinum:dedicated-support";
    pub const INSTITUTIONAL_TOOLS: &str = "platinum:institutional-tools";
    pub const UNLIMITED_API: &str = "platinum:unlimited-api";
    pub const CUSTOM_INTEGRATIONS: &str = "platinum:custom-integrations";
    
    pub fn get_default_features() -> HashSet<TierFeature> {
        [
            TierFeature::BasicTrading,
            TierFeature::AdvancedAnalytics,
            TierFeature::ApiAccess,
            TierFeature::PrioritySupport,
            TierFeature::AdvancedOrders,
            TierFeature::PortfolioTools,
            TierFeature::ResearchReports,
            TierFeature::InstitutionalFeatures,
        ].iter().cloned().collect()
    }
    
    pub fn get_default_limits() -> Vec<TierLimit> {
        vec![
            TierLimit {
                feature: TierFeature::ApiAccess,
                limit_type: TierLimitType::RequestsPerHour,
                max_value: 100000,
                current_value: 0,
                reset_period: Some(TierResetPeriod::Hour),
                burst_allowance: Some(10000),
            },
            TierLimit {
                feature: TierFeature::AdvancedAnalytics,
                limit_type: TierLimitType::RequestsPerDay,
                max_value: -1, // Unlimited
                current_value: 0,
                reset_period: Some(TierResetPeriod::Day),
                burst_allowance: None,
            },
        ]
    }
}

// Implementations

impl TierFeature {
    /// Get the string representation of the feature
    pub fn as_str(&self) -> &str {
        match self {
            TierFeature::BasicTrading => "basic-trading",
            TierFeature::AdvancedAnalytics => "advanced-analytics",
            TierFeature::ApiAccess => "api-access",
            TierFeature::PrioritySupport => "priority-support",
            TierFeature::AdvancedOrders => "advanced-orders",
            TierFeature::PortfolioTools => "portfolio-tools",
            TierFeature::ResearchReports => "research-reports",
            TierFeature::InstitutionalFeatures => "institutional-features",
            TierFeature::Custom(name) => name,
        }
    }
    
    /// Get the required minimum tier for this feature
    pub fn required_tier(&self) -> PackageTier {
        match self {
            TierFeature::BasicTrading => PackageTier::Free,
            TierFeature::ApiAccess => PackageTier::Bronze,
            TierFeature::AdvancedAnalytics => PackageTier::Silver,
            TierFeature::PortfolioTools => PackageTier::Silver,
            TierFeature::PrioritySupport => PackageTier::Gold,
            TierFeature::AdvancedOrders => PackageTier::Gold,
            TierFeature::ResearchReports => PackageTier::Gold,
            TierFeature::InstitutionalFeatures => PackageTier::Platinum,
            TierFeature::Custom(_) => PackageTier::Free,
        }
    }
    
    /// Check if feature is premium (requires payment)
    pub fn is_premium(&self) -> bool {
        self.required_tier() != PackageTier::Free
    }
}

impl std::str::FromStr for TierFeature {
    type Err = ValidationError;
    
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "basic-trading" => Ok(TierFeature::BasicTrading),
            "advanced-analytics" => Ok(TierFeature::AdvancedAnalytics),
            "api-access" => Ok(TierFeature::ApiAccess),
            "priority-support" => Ok(TierFeature::PrioritySupport),
            "advanced-orders" => Ok(TierFeature::AdvancedOrders),
            "portfolio-tools" => Ok(TierFeature::PortfolioTools),
            "research-reports" => Ok(TierFeature::ResearchReports),
            "institutional-features" => Ok(TierFeature::InstitutionalFeatures),
            _ => Ok(TierFeature::Custom(s.to_string())),
        }
    }
}

impl std::fmt::Display for TierFeature {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl TierLimit {
    pub fn new(feature: TierFeature, limit_type: TierLimitType, max_value: i64) -> Self {
        Self {
            feature,
            limit_type,
            max_value,
            current_value: 0,
            reset_period: None,
            burst_allowance: None,
        }
    }
    
    pub fn with_reset_period(mut self, period: TierResetPeriod) -> Self {
        self.reset_period = Some(period);
        self
    }
    
    pub fn with_burst_allowance(mut self, allowance: i64) -> Self {
        self.burst_allowance = Some(allowance);
        self
    }
    
    pub fn is_exceeded(&self) -> bool {
        if self.max_value < 0 {
            return false; // Unlimited
        }
        self.current_value >= self.max_value
    }
    
    pub fn can_burst(&self) -> bool {
        if let Some(burst) = self.burst_allowance {
            self.current_value < self.max_value + burst
        } else {
            !self.is_exceeded()
        }
    }
    
    pub fn remaining(&self) -> i64 {
        if self.max_value < 0 {
            i64::MAX // Unlimited
        } else {
            (self.max_value - self.current_value).max(0)
        }
    }
    
    pub fn increment(&mut self, amount: i64) {
        self.current_value += amount;
    }
    
    pub fn reset(&mut self) {
        self.current_value = 0;
    }
    
    pub fn should_reset(&self, now: DateTime<Utc>, last_reset: DateTime<Utc>) -> bool {
        if let Some(period) = &self.reset_period {
            match period {
                TierResetPeriod::Minute => now.minute() != last_reset.minute(),
                TierResetPeriod::Hour => now.hour() != last_reset.hour(),
                TierResetPeriod::Day => now.day() != last_reset.day(),
                TierResetPeriod::Week => now.iso_week() != last_reset.iso_week(),
                TierResetPeriod::Month => now.month() != last_reset.month(),
                TierResetPeriod::Year => now.year() != last_reset.year(),
                TierResetPeriod::Never => false,
            }
        } else {
            false
        }
    }
}

impl TierAccess {
    pub fn new(user_id: UserId, package_tier: PackageTier) -> Self {
        Self {
            user_id,
            package_tier,
            enabled_features: HashSet::new(),
            limits: Vec::new(),
            upgraded_at: Utc::now(),
            expires_at: None,
            auto_renew: false,
            trial_mode: false,
        }
    }
    
    pub fn with_features(mut self, features: HashSet<TierFeature>) -> Self {
        self.enabled_features = features;
        self
    }
    
    pub fn with_limits(mut self, limits: Vec<TierLimit>) -> Self {
        self.limits = limits;
        self
    }
    
    pub fn with_expiration(mut self, expires_at: DateTime<Utc>) -> Self {
        self.expires_at = Some(expires_at);
        self
    }
    
    pub fn with_trial_mode(mut self, trial: bool) -> Self {
        self.trial_mode = trial;
        self
    }
    
    pub fn is_expired(&self) -> bool {
        if let Some(expires_at) = self.expires_at {
            Utc::now() > expires_at
        } else {
            false
        }
    }
    
    pub fn is_in_grace_period(&self, grace_seconds: u64) -> bool {
        if let Some(expires_at) = self.expires_at {
            let grace_end = expires_at + chrono::Duration::seconds(grace_seconds as i64);
            Utc::now() <= grace_end
        } else {
            false
        }
    }
    
    pub fn has_feature(&self, feature: &TierFeature) -> bool {
        if self.is_expired() {
            return false;
        }
        
        self.enabled_features.contains(feature)
    }
    
    pub fn get_limit(&self, feature: &TierFeature, limit_type: &TierLimitType) -> Option<&TierLimit> {
        self.limits.iter().find(|limit| {
            &limit.feature == feature && &limit.limit_type == limit_type
        })
    }
    
    pub fn get_limit_mut(&mut self, feature: &TierFeature, limit_type: &TierLimitType) -> Option<&mut TierLimit> {
        self.limits.iter_mut().find(|limit| {
            &limit.feature == feature && &limit.limit_type == limit_type
        })
    }
    
    pub fn check_limit(&self, feature: &TierFeature, limit_type: &TierLimitType, amount: i64) -> bool {
        if let Some(limit) = self.get_limit(feature, limit_type) {
            if limit.max_value < 0 {
                return true; // Unlimited
            }
            limit.current_value + amount <= limit.max_value
        } else {
            true // No limit defined
        }
    }
    
    pub fn consume_limit(&mut self, feature: &TierFeature, limit_type: &TierLimitType, amount: i64) -> bool {
        if let Some(limit) = self.get_limit_mut(feature, limit_type) {
            if limit.max_value < 0 {
                return true; // Unlimited
            }
            if limit.current_value + amount <= limit.max_value {
                limit.increment(amount);
                true
            } else if limit.can_burst() {
                limit.increment(amount);
                true
            } else {
                false
            }
        } else {
            true // No limit defined
        }
    }
}

impl PackageTierValidator {
    pub fn new(config: PackageTierConfig) -> Self {
        let mut validator = Self {
            tier_access: HashMap::new(),
            tier_features: HashMap::new(),
            default_limits: HashMap::new(),
            tier_hierarchy: vec![
                PackageTier::Free,
                PackageTier::Bronze,
                PackageTier::Silver,
                PackageTier::Gold,
                PackageTier::Platinum,
                PackageTier::Admin,
                PackageTier::SuperAdmin,
            ],
            config,
        };
        
        validator.initialize_tier_features();
        validator.initialize_default_limits();
        validator
    }
    
    fn initialize_tier_features(&mut self) {
        self.tier_features.insert(PackageTier::Free, free_tier::get_default_features());
        self.tier_features.insert(PackageTier::Bronze, bronze_tier::get_default_features());
        self.tier_features.insert(PackageTier::Silver, silver_tier::get_default_features());
        self.tier_features.insert(PackageTier::Gold, gold_tier::get_default_features());
        self.tier_features.insert(PackageTier::Platinum, platinum_tier::get_default_features());
        
        // Admin tiers get all features
        let all_features: HashSet<TierFeature> = [
            TierFeature::BasicTrading,
            TierFeature::AdvancedAnalytics,
            TierFeature::ApiAccess,
            TierFeature::PrioritySupport,
            TierFeature::AdvancedOrders,
            TierFeature::PortfolioTools,
            TierFeature::ResearchReports,
            TierFeature::InstitutionalFeatures,
        ].iter().cloned().collect();
        
        self.tier_features.insert(PackageTier::Admin, all_features.clone());
        self.tier_features.insert(PackageTier::SuperAdmin, all_features);
    }
    
    fn initialize_default_limits(&mut self) {
        self.default_limits.insert(PackageTier::Free, free_tier::get_default_limits());
        self.default_limits.insert(PackageTier::Bronze, bronze_tier::get_default_limits());
        self.default_limits.insert(PackageTier::Silver, silver_tier::get_default_limits());
        self.default_limits.insert(PackageTier::Gold, gold_tier::get_default_limits());
        self.default_limits.insert(PackageTier::Platinum, platinum_tier::get_default_limits());
        
        // Admin tiers have unlimited access
        self.default_limits.insert(PackageTier::Admin, vec![]);
        self.default_limits.insert(PackageTier::SuperAdmin, vec![]);
    }
    
    pub fn grant_tier_access(&mut self, access: TierAccess) {
        self.tier_access.insert(access.user_id.clone(), access);
    }
    
    pub fn upgrade_user_tier(&mut self, user_id: &UserId, new_tier: PackageTier) -> Result<(), PermissionError> {
        // First, get the current tier and validate upgrade path
        let current_tier = if let Some(access) = self.tier_access.get(user_id) {
            access.package_tier.clone()
        } else {
            return Err(PermissionError::ValidationFailed(
                ValidationError::InvalidFormat("User not found".to_string())
            ));
        };
        
        // Validate upgrade path before mutable borrow
        if !self.can_upgrade_to_tier(&current_tier, &new_tier) {
            return Err(PermissionError::ValidationFailed(
                ValidationError::InvalidFormat("Invalid tier upgrade path".to_string())
            ));
        }
        
        // Now get mutable access and update
        if let Some(access) = self.tier_access.get_mut(user_id) {
            
            access.package_tier = new_tier.clone();
            access.upgraded_at = Utc::now();
            
            // Update features and limits
            if let Some(features) = self.tier_features.get(&new_tier) {
                access.enabled_features = features.clone();
            }
            
            if let Some(limits) = self.default_limits.get(&new_tier) {
                access.limits = limits.clone();
            }
        } else {
            return Err(PermissionError::UserNotFound {
                user_id: user_id.clone(),
            });
        }
        
        Ok(())
    }
    
    fn can_upgrade_to_tier(&self, current: &PackageTier, target: &PackageTier) -> bool {
        if let (Some(current_pos), Some(target_pos)) = (
            self.tier_hierarchy.iter().position(|t| t == current),
            self.tier_hierarchy.iter().position(|t| t == target),
        ) {
            target_pos >= current_pos
        } else {
            false
        }
    }
    
    pub fn validate_tier_feature(
        &self,
        user_id: &UserId,
        feature: &TierFeature,
        _amount: i64,
    ) -> Result<bool, PermissionError> {
        if let Some(access) = self.tier_access.get(user_id) {
            // Check if tier has expired (with grace period)
            if access.is_expired() && !access.is_in_grace_period(self.config.grace_period_seconds) {
                return Ok(false);
            }
            
            // Check if feature is enabled
            if !access.has_feature(feature) {
                // Check inheritance if enabled
                if self.config.enable_inheritance {
                    if let Some(tier_features) = self.tier_features.get(&access.package_tier) {
                        if !tier_features.contains(feature) {
                            return Ok(false);
                        }
                    }
                } else {
                    return Ok(false);
                }
            }
            
            // Check limits if applicable
            // This would be extended to check various limit types
            Ok(true)
        } else {
            Ok(false)
        }
    }
    
    pub fn get_user_tier_access(&self, user_id: &UserId) -> Option<&TierAccess> {
        self.tier_access.get(user_id)
    }
    
    pub fn get_available_features(&self, tier: &PackageTier) -> HashSet<TierFeature> {
        if self.config.enable_inheritance {
            let mut features = HashSet::new();
            
            // Add features from current tier and all lower tiers
            for hierarchy_tier in &self.tier_hierarchy {
                if let Some(tier_features) = self.tier_features.get(hierarchy_tier) {
                    features.extend(tier_features.clone());
                }
                
                if hierarchy_tier == tier {
                    break;
                }
            }
            
            features
        } else {
            self.tier_features.get(tier).cloned().unwrap_or_default()
        }
    }
}

#[async_trait]
impl PermissionValidator for PackageTierValidator {
    async fn validate(&self, context: &PermissionContext) -> Result<PermissionDecision, PermissionError> {
        // Parse permission to extract feature
        if let Ok(feature) = context.permission.parse::<TierFeature>() {
            let has_access = self.validate_tier_feature(&context.user_id, &feature, 1)?;
            
            if has_access {
                let grant = super::core::PermissionGrant {
                    request_id: uuid::Uuid::new_v4(),
                    user_id: context.user_id.clone(),
                    granted_permissions: vec![Permission::new(
                        context.permission.clone(),
                        context.resource.clone(),
                    )],
                    granted_at: context.timestamp,
                    granted_by: UserId::new("system".to_string()),
                    expires_at: self.tier_access
                        .get(&context.user_id)
                        .and_then(|access| access.expires_at),
                    conditions: None,
                };
                
                Ok(PermissionDecision::Granted(grant))
            } else {
                let denial = super::core::PermissionDenial {
                    request_id: uuid::Uuid::new_v4(),
                    user_id: context.user_id.clone(),
                    denied_permissions: vec![context.permission.clone()],
                    reason: super::core::DenialReason::InsufficientTier,
                    denied_at: context.timestamp,
                    retry_after: None,
                };
                
                Ok(PermissionDecision::Denied(denial))
            }
        } else {
            Err(PermissionError::InvalidPermissionFormat {
                permission: context.permission.clone(),
            })
        }
    }
    
    async fn has_permission(&self, user_id: &UserId, permission: &str, _resource: &str) -> bool {
        if let Ok(feature) = permission.parse::<TierFeature>() {
            self.validate_tier_feature(user_id, &feature, 1).unwrap_or(false)
        } else {
            false
        }
    }
    
    async fn get_permissions(&self, user_id: &UserId) -> Result<Vec<Permission>, PermissionError> {
        if let Some(access) = self.tier_access.get(user_id) {
            let permissions = access
                .enabled_features
                .iter()
                .map(|feature| Permission::new(feature.to_string(), "*".to_string()))
                .collect();
            
            Ok(permissions)
        } else {
            Ok(vec![])
        }
    }

    async fn get_effective_permissions(&self, user_id: &UserId) -> Result<EffectivePermissions, PermissionError> {
        let permissions = self.get_permissions(user_id).await?;
        
        let mut tier_features = HashMap::new();
        if let Some(access) = self.tier_access.get(user_id) {
            if !access.is_expired() {
                for feature in &access.enabled_features {
                    tier_features.insert(feature.clone(), true);
                }
            }
        }
        
        Ok(EffectivePermissions {
            user_id: user_id.clone(),
            permissions,
            admin_modules: HashMap::new(), // Not used in tier validator
            tier_features,
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

impl Default for PackageTierConfig {
    fn default() -> Self {
        Self {
            enable_inheritance: true,
            enable_trials: true,
            grace_period_seconds: 86400, // 1 day
            trial_period_days: 14,
            enable_burst_limits: true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_tier_feature_parsing() {
        let feature: TierFeature = "advanced-analytics".parse().unwrap();
        assert_eq!(feature, TierFeature::AdvancedAnalytics);
        assert_eq!(feature.as_str(), "advanced-analytics");
    }
    
    #[test]
    fn test_tier_feature_requirements() {
        assert_eq!(TierFeature::BasicTrading.required_tier(), PackageTier::Free);
        assert_eq!(TierFeature::AdvancedAnalytics.required_tier(), PackageTier::Silver);
        assert_eq!(TierFeature::InstitutionalFeatures.required_tier(), PackageTier::Platinum);
        
        assert!(!TierFeature::BasicTrading.is_premium());
        assert!(TierFeature::AdvancedAnalytics.is_premium());
    }
    
    #[test]
    fn test_tier_limit_functionality() {
        let mut limit = TierLimit::new(
            TierFeature::ApiAccess,
            TierLimitType::RequestsPerHour,
            1000,
        );
        
        assert!(!limit.is_exceeded());
        assert_eq!(limit.remaining(), 1000);
        
        limit.increment(500);
        assert_eq!(limit.remaining(), 500);
        assert!(!limit.is_exceeded());
        
        limit.increment(500);
        assert_eq!(limit.remaining(), 0);
        assert!(limit.is_exceeded());
        
        limit.reset();
        assert_eq!(limit.remaining(), 1000);
        assert!(!limit.is_exceeded());
    }
    
    #[test]
    fn test_tier_access_creation() {
        let user_id = UserId::new("user123".to_string());
        let features = [TierFeature::BasicTrading, TierFeature::ApiAccess]
            .iter()
            .cloned()
            .collect();
        
        let access = TierAccess::new(user_id.clone(), PackageTier::Bronze)
            .with_features(features.clone());
        
        assert_eq!(access.user_id, user_id);
        assert_eq!(access.package_tier, PackageTier::Bronze);
        assert_eq!(access.enabled_features, features);
        assert!(!access.is_expired());
    }
    
    #[test]
    fn test_tier_access_expiration() {
        let user_id = UserId::new("user123".to_string());
        let past_time = Utc::now() - chrono::Duration::hours(1);
        
        let access = TierAccess::new(user_id, PackageTier::Bronze)
            .with_expiration(past_time);
        
        assert!(access.is_expired());
        assert!(!access.has_feature(&TierFeature::BasicTrading));
        
        // Test grace period
        assert!(access.is_in_grace_period(7200)); // 2 hours
        assert!(!access.is_in_grace_period(1800)); // 30 minutes
    }
    
    #[tokio::test]
    async fn test_package_tier_validator() {
        let config = PackageTierConfig::default();
        let mut validator = PackageTierValidator::new(config);
        
        let user_id = UserId::new("user123".to_string());
        let features = [TierFeature::BasicTrading, TierFeature::ApiAccess]
            .iter()
            .cloned()
            .collect();
        
        let access = TierAccess::new(user_id.clone(), PackageTier::Bronze)
            .with_features(features);
        
        validator.grant_tier_access(access);
        
        // Test permission validation
        assert!(validator.has_permission(&user_id, "basic-trading", "*").await);
        assert!(validator.has_permission(&user_id, "api-access", "*").await);
        assert!(!validator.has_permission(&user_id, "advanced-analytics", "*").await);
    }
    
    #[test]
    fn test_tier_upgrade() {
        let config = PackageTierConfig::default();
        let mut validator = PackageTierValidator::new(config);
        
        let user_id = UserId::new("user123".to_string());
        let access = TierAccess::new(user_id.clone(), PackageTier::Bronze);
        
        validator.grant_tier_access(access);
        
        // Test valid upgrade
        assert!(validator.upgrade_user_tier(&user_id, PackageTier::Gold).is_ok());
        
        // Test invalid downgrade (should still work as it's >= current)
        assert!(validator.upgrade_user_tier(&user_id, PackageTier::Silver).is_ok());
    }
    
    #[test]
    fn test_tier_feature_inheritance() {
        let config = PackageTierConfig::default();
        let validator = PackageTierValidator::new(config);
        
        let gold_features = validator.get_available_features(&PackageTier::Gold);
        
        // Gold should have all features from lower tiers
        assert!(gold_features.contains(&TierFeature::BasicTrading)); // Free
        assert!(gold_features.contains(&TierFeature::ApiAccess)); // Bronze
        assert!(gold_features.contains(&TierFeature::AdvancedAnalytics)); // Silver
        assert!(gold_features.contains(&TierFeature::PrioritySupport)); // Gold
    }
}