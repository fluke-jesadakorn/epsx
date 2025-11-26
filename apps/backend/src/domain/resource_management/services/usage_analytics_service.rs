// Usage Analytics Service
// Domain service for analyzing resource usage patterns and generating insights

use crate::domain::resource_management::aggregates::UserResourceUsage;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageAnalytics {
    pub wallet_address: String,
    pub plan_id: Option<i32>,
    pub period_start: DateTime<Utc>,
    pub period_end: DateTime<Utc>,
    pub total_requests: i64,
    pub peak_usage_time: Option<DateTime<Utc>>,
    pub resource_breakdown: HashMap<String, ResourceAnalytics>,
    pub efficiency_score: f64,
    pub predicted_overage: HashMap<String, f64>,
    pub recommendations: Vec<UsageRecommendation>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceAnalytics {
    pub resource_type: String,
    pub total_usage: i64,
    pub quota_limit: i64,
    pub usage_percentage: f64,
    pub peak_usage: i64,
    pub average_daily_usage: f64,
    pub trend: UsageTrend,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum UsageTrend {
    Increasing,
    Decreasing,
    Stable,
    Volatile,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageRecommendation {
    pub recommendation_type: RecommendationType,
    pub priority: Priority,
    pub title: String,
    pub description: String,
    pub potential_savings: Option<f64>,
    pub suggested_action: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RecommendationType {
    PlanUpgrade,
    PlanDowngrade,
    UsageOptimization,
    RateLimitAdjustment,
    CostSaving,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Priority {
    High,
    Medium,
    Low,
}

pub struct UsageAnalyticsService;

impl UsageAnalyticsService {
    pub fn new() -> Self {
        Self
    }

    /// Analyze user's resource usage patterns
    pub fn analyze_usage(
        &self,
        usage: &UserResourceUsage,
        historical_data: &[UserResourceUsage],
    ) -> UsageAnalytics {
        let mut resource_breakdown = HashMap::new();
        let mut total_requests = 0;
        let mut recommendations = Vec::new();

        // Analyze each resource type
        for (resource_type, current_usage) in &usage.current_usage {
            let quota_limit = usage.quota_limits.get(resource_type).unwrap_or(&0);
            let usage_percentage = if *quota_limit > 0 {
                (*current_usage as f64 / *quota_limit as f64) * 100.0
            } else {
                0.0
            };

            let trend = self.calculate_usage_trend(resource_type, historical_data);
            let average_daily = self.calculate_average_daily_usage(resource_type, historical_data);

            resource_breakdown.insert(resource_type.clone(), ResourceAnalytics {
                resource_type: resource_type.clone(),
                total_usage: *current_usage,
                quota_limit: *quota_limit,
                usage_percentage,
                peak_usage: *current_usage, // Simplified - in real implementation, track peak
                average_daily_usage: average_daily,
                trend,
            });

            total_requests += current_usage;

            // Generate recommendations based on usage patterns
            if usage_percentage > 80.0 {
                recommendations.push(UsageRecommendation {
                    recommendation_type: RecommendationType::PlanUpgrade,
                    priority: if usage_percentage > 95.0 { Priority::High } else { Priority::Medium },
                    title: format!("Consider upgrading for {}", resource_type),
                    description: format!(
                        "You're using {}% of your {} quota. Consider upgrading to avoid service interruption.",
                        usage_percentage.round(),
                        resource_type
                    ),
                    potential_savings: None,
                    suggested_action: "Upgrade to a higher-tier plan".to_string(),
                });
            } else if usage_percentage < 20.0 && *quota_limit > 1000 {
                recommendations.push(UsageRecommendation {
                    recommendation_type: RecommendationType::PlanDowngrade,
                    priority: Priority::Low,
                    title: format!("Potential savings on {}", resource_type),
                    description: format!(
                        "You're only using {}% of your {} quota. You might save money with a lower-tier plan.",
                        usage_percentage.round(),
                        resource_type
                    ),
                    potential_savings: Some(10.0), // Simplified calculation
                    suggested_action: "Consider downgrading to a more suitable plan".to_string(),
                });
            }
        }

        let efficiency_score = self.calculate_efficiency_score(&resource_breakdown);
        let predicted_overage = self.predict_overage(usage, historical_data);

        UsageAnalytics {
            wallet_address: usage.wallet_address.clone(),
            plan_id: usage.plan_id,
            period_start: usage.billing_period_start,
            period_end: usage.billing_period_end,
            total_requests,
            peak_usage_time: None, // Would track from historical data
            resource_breakdown,
            efficiency_score,
            predicted_overage,
            recommendations,
        }
    }

    fn calculate_usage_trend(&self, resource_type: &str, historical_data: &[UserResourceUsage]) -> UsageTrend {
        if historical_data.len() < 2 {
            return UsageTrend::Stable;
        }

        let recent_usage: Vec<i64> = historical_data
            .iter()
            .rev()
            .take(5)
            .filter_map(|usage| usage.current_usage.get(resource_type))
            .copied()
            .collect();

        if recent_usage.len() < 2 {
            return UsageTrend::Stable;
        }

        let first_half_avg = recent_usage[..recent_usage.len()/2].iter().sum::<i64>() as f64 / (recent_usage.len()/2) as f64;
        let second_half_avg = recent_usage[recent_usage.len()/2..].iter().sum::<i64>() as f64 / (recent_usage.len() - recent_usage.len()/2) as f64;

        let change_percentage = ((second_half_avg - first_half_avg) / first_half_avg) * 100.0;

        match change_percentage {
            x if x > 20.0 => UsageTrend::Increasing,
            x if x < -20.0 => UsageTrend::Decreasing,
            _ => UsageTrend::Stable,
        }
    }

    fn calculate_average_daily_usage(&self, resource_type: &str, historical_data: &[UserResourceUsage]) -> f64 {
        if historical_data.is_empty() {
            return 0.0;
        }

        let total_usage: i64 = historical_data
            .iter()
            .filter_map(|usage| usage.current_usage.get(resource_type))
            .sum();

        total_usage as f64 / historical_data.len() as f64
    }

    fn calculate_efficiency_score(&self, resource_breakdown: &HashMap<String, ResourceAnalytics>) -> f64 {
        if resource_breakdown.is_empty() {
            return 0.0;
        }

        let total_efficiency: f64 = resource_breakdown
            .values()
            .map(|analytics| {
                // Efficiency is high when usage is between 60-80% of quota
                let usage_pct = analytics.usage_percentage;
                if (60.0..=80.0).contains(&usage_pct) {
                    100.0
                } else if usage_pct < 60.0 {
                    usage_pct / 60.0 * 100.0
                } else {
                    // Penalize over-usage
                    100.0 - ((usage_pct - 80.0) * 2.0)
                }.max(0.0)
            })
            .sum();

        total_efficiency / resource_breakdown.len() as f64
    }

    fn predict_overage(&self, usage: &UserResourceUsage, _historical_data: &[UserResourceUsage]) -> HashMap<String, f64> {
        let mut predictions = HashMap::new();

        for (resource_type, current_usage) in &usage.current_usage {
            if let Some(limit) = usage.quota_limits.get(resource_type) {
                if *limit > 0 {
                    let usage_percentage = (*current_usage as f64 / *limit as f64) * 100.0;
                    if usage_percentage > 70.0 {
                        // Simple prediction: if trending high, predict overage
                        let predicted_overage = (*current_usage as f64 * 1.2) - *limit as f64;
                        if predicted_overage > 0.0 {
                            predictions.insert(resource_type.clone(), predicted_overage);
                        }
                    }
                }
            }
        }

        predictions
    }
}

impl Default for UsageAnalyticsService {
    fn default() -> Self {
        Self::new()
    }
}