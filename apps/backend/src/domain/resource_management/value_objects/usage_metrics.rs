// Usage metrics value object
// Tracks actual resource consumption with time-based aggregations

use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc, Datelike, Timelike};
use rust_decimal::Decimal;
use rust_decimal::prelude::*;
use std::collections::HashMap;

use super::resource_type::{ResourceType, ResourceCategory};

/// Usage metrics for a specific time period
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageMetrics {
    pub wallet_address: Option<String>,
    pub api_key: Option<String>,
    pub session_id: Option<String>,
    pub time_period: TimePeriod,
    pub resource_usage: HashMap<ResourceType, ResourceUsageStats>,
    pub category_totals: HashMap<ResourceCategory, CategoryUsageStats>,
    pub total_cost: Decimal,
    pub total_requests: u64,
    pub total_data_transfer_bytes: u64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Time period for usage aggregation
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TimePeriod {
    Minute { timestamp: DateTime<Utc> },
    Hour { timestamp: DateTime<Utc> },
    Day { date: chrono::NaiveDate },
    Month { year: i32, month: u32 },
    Year { year: i32 },
}

/// Usage statistics for a specific resource type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceUsageStats {
    pub resource_type: ResourceType,
    pub count: u64,
    pub total_cost: Decimal,
    pub average_cost_per_unit: Decimal,
    pub peak_usage_per_minute: u64,
    pub total_bytes: Option<u64>,
    pub total_duration_ms: Option<u64>,
    pub first_used_at: DateTime<Utc>,
    pub last_used_at: DateTime<Utc>,
}

/// Aggregated usage statistics by category
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CategoryUsageStats {
    pub category: ResourceCategory,
    pub total_count: u64,
    pub total_cost: Decimal,
    pub cost_percentage: f64,
    pub resource_type_breakdown: HashMap<ResourceType, u64>,
    pub peak_usage_timestamp: DateTime<Utc>,
    pub efficiency_score: f64, // 0.0 to 1.0 - cost effectiveness
}

/// Real-time usage tracking for rate limiting and quota management
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RealTimeUsageTracker {
    pub identifier: String, // user_id, api_key, or session_id
    pub current_minute: UsageWindow,
    pub current_hour: UsageWindow,
    pub current_day: UsageWindow,
    pub current_month: UsageWindow,
    pub last_updated: DateTime<Utc>,
}

/// Usage window for real-time tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageWindow {
    pub window_start: DateTime<Utc>,
    pub total_requests: u64,
    pub total_cost: Decimal,
    pub resource_counts: HashMap<ResourceType, u64>,
    pub category_counts: HashMap<ResourceCategory, u64>,
}

/// Usage analytics for business intelligence
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageAnalytics {
    pub time_period: TimePeriod,
    pub total_users: u64,
    pub total_api_keys: u64,
    pub usage_trends: UsageTrends,
    pub cost_analysis: CostAnalysis,
    pub performance_metrics: PerformanceMetrics,
    pub top_consumers: Vec<TopConsumer>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageTrends {
    pub growth_rate_percentage: f64,
    pub seasonal_patterns: Vec<SeasonalPattern>,
    pub peak_usage_hours: Vec<u8>,
    pub prediction_next_period: UsagePrediction,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CostAnalysis {
    pub total_infrastructure_cost: Decimal,
    pub total_billable_revenue: Decimal,
    pub profit_margin_percentage: f64,
    pub cost_per_user: Decimal,
    pub revenue_per_api_key: Decimal,
    pub most_expensive_resources: Vec<(ResourceType, Decimal)>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    pub average_response_time_ms: f64,
    pub p95_response_time_ms: f64,
    pub error_rate_percentage: f64,
    pub cache_hit_rate_percentage: f64,
    pub resource_efficiency_score: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TopConsumer {
    pub identifier: String,
    pub identifier_type: ConsumerType,
    pub total_usage: u64,
    pub total_cost: Decimal,
    pub primary_resource_type: ResourceType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConsumerType {
    User,
    ApiKey,
    AdminSession,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SeasonalPattern {
    pub pattern_type: PatternType,
    pub multiplier: f64,
    pub confidence_score: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PatternType {
    DailyPeak,
    WeeklyTrend,
    MonthlySpike,
    BusinessHours,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct UsagePrediction {
    pub predicted_requests: u64,
    pub predicted_cost: Decimal,
    pub confidence_interval_low: u64,
    pub confidence_interval_high: u64,
    pub prediction_basis: PredictionBasis,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub enum PredictionBasis {
    #[default]
    HistoricalTrend,
    SeasonalAnalysis,
    MachinelearningModel,
    LinearRegression,
}

impl UsageMetrics {
    pub fn new(
        wallet_address: Option<String>,
        api_key: Option<String>,
        session_id: Option<String>,
        time_period: TimePeriod,
    ) -> Self {
        let now = Utc::now();
        Self {
            wallet_address,
            api_key,
            session_id,
            time_period,
            resource_usage: HashMap::new(),
            category_totals: HashMap::new(),
            total_cost: Decimal::ZERO,
            total_requests: 0,
            total_data_transfer_bytes: 0,
            created_at: now,
            updated_at: now,
        }
    }

    /// Add a resource usage event
    pub fn add_resource_usage(
        &mut self,
        resource_type: ResourceType,
        cost: Decimal,
        bytes: Option<u64>,
        duration_ms: Option<u64>,
    ) {
        let now = Utc::now();
        
        // Update resource-specific stats
        let resource_stats = self.resource_usage
            .entry(resource_type.clone())
            .or_insert_with(|| ResourceUsageStats {
                resource_type: resource_type.clone(),
                count: 0,
                total_cost: Decimal::ZERO,
                average_cost_per_unit: Decimal::ZERO,
                peak_usage_per_minute: 0,
                total_bytes: None,
                total_duration_ms: None,
                first_used_at: now,
                last_used_at: now,
            });

        resource_stats.count += 1;
        resource_stats.total_cost += cost;
        resource_stats.average_cost_per_unit = resource_stats.total_cost / Decimal::from(resource_stats.count);
        resource_stats.last_used_at = now;
        
        if let Some(bytes) = bytes {
            resource_stats.total_bytes = Some(
                resource_stats.total_bytes.unwrap_or(0) + bytes
            );
            self.total_data_transfer_bytes += bytes;
        }
        
        if let Some(duration) = duration_ms {
            resource_stats.total_duration_ms = Some(
                resource_stats.total_duration_ms.unwrap_or(0) + duration
            );
        }

        // Update category totals
        let category = resource_type.category();
        let category_stats = self.category_totals
            .entry(category.clone())
            .or_insert_with(|| CategoryUsageStats {
                category: category.clone(),
                total_count: 0,
                total_cost: Decimal::ZERO,
                cost_percentage: 0.0,
                resource_type_breakdown: HashMap::new(),
                peak_usage_timestamp: now,
                efficiency_score: 1.0,
            });

        category_stats.total_count += 1;
        category_stats.total_cost += cost;
        *category_stats.resource_type_breakdown
            .entry(resource_type)
            .or_insert(0) += 1;

        // Update totals
        self.total_cost += cost;
        self.total_requests += 1;
        self.updated_at = now;
        
        // Recalculate cost percentages
        self.recalculate_cost_percentages();
    }

    /// Calculate efficiency score for the usage pattern
    pub fn calculate_efficiency_score(&self) -> f64 {
        if self.total_requests == 0 {
            return 1.0;
        }

        let cost_per_request = self.total_cost.to_f64().unwrap_or(0.0) / self.total_requests as f64;
        let efficiency_threshold = 0.01; // $0.01 per request as baseline
        
        if cost_per_request <= efficiency_threshold {
            1.0
        } else {
            (efficiency_threshold / cost_per_request).min(1.0_f64)
        }
    }

    /// Get usage summary for reporting
    pub fn get_usage_summary(&self) -> UsageSummary {
        UsageSummary {
            time_period: self.time_period.clone(),
            total_requests: self.total_requests,
            total_cost: self.total_cost,
            total_data_transfer_gb: (self.total_data_transfer_bytes as f64) / 1_000_000_000.0,
            top_resource_types: self.get_top_resource_types(5),
            cost_breakdown: self.get_cost_breakdown(),
            efficiency_score: self.calculate_efficiency_score(),
        }
    }

    fn recalculate_cost_percentages(&mut self) {
        let total_cost_f64 = self.total_cost.to_f64().unwrap_or(0.0);
        if total_cost_f64 > 0.0 {
            for category_stats in self.category_totals.values_mut() {
                let category_cost_f64 = category_stats.total_cost.to_f64().unwrap_or(0.0);
                category_stats.cost_percentage = (category_cost_f64 / total_cost_f64) * 100.0;
            }
        }
    }

    fn get_top_resource_types(&self, limit: usize) -> Vec<(ResourceType, u64)> {
        let mut resource_counts: Vec<(ResourceType, u64)> = self.resource_usage
            .iter()
            .map(|(resource_type, stats)| (resource_type.clone(), stats.count))
            .collect();
        
        resource_counts.sort_by(|a, b| b.1.cmp(&a.1));
        resource_counts.truncate(limit);
        resource_counts
    }

    fn get_cost_breakdown(&self) -> Vec<(ResourceCategory, Decimal)> {
        self.category_totals
            .iter()
            .map(|(category, stats)| (category.clone(), stats.total_cost))
            .collect()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageSummary {
    pub time_period: TimePeriod,
    pub total_requests: u64,
    pub total_cost: Decimal,
    pub total_data_transfer_gb: f64,
    pub top_resource_types: Vec<(ResourceType, u64)>,
    pub cost_breakdown: Vec<(ResourceCategory, Decimal)>,
    pub efficiency_score: f64,
}

impl TimePeriod {
    pub fn current_minute() -> Self {
        TimePeriod::Minute { 
            timestamp: Utc::now().with_second(0).unwrap().with_nanosecond(0).unwrap()
        }
    }

    pub fn current_hour() -> Self {
        TimePeriod::Hour { 
            timestamp: Utc::now().with_minute(0).unwrap().with_second(0).unwrap().with_nanosecond(0).unwrap()
        }
    }

    pub fn current_day() -> Self {
        TimePeriod::Day { 
            date: Utc::now().date_naive()
        }
    }

    pub fn current_month() -> Self {
        let now = Utc::now();
        TimePeriod::Month { 
            year: now.year(),
            month: now.month(),
        }
    }

    pub fn current_year() -> Self {
        TimePeriod::Year { 
            year: Utc::now().year()
        }
    }
}

impl RealTimeUsageTracker {
    pub fn new(identifier: String) -> Self {
        let now = Utc::now();
        Self {
            identifier,
            current_minute: UsageWindow::new(now.with_second(0).unwrap().with_nanosecond(0).unwrap()),
            current_hour: UsageWindow::new(now.with_minute(0).unwrap().with_second(0).unwrap().with_nanosecond(0).unwrap()),
            current_day: UsageWindow::new(now.date_naive().and_hms_opt(0, 0, 0).unwrap().and_utc()),
            current_month: UsageWindow::new(now.date_naive().with_day(1).unwrap().and_hms_opt(0, 0, 0).unwrap().and_utc()),
            last_updated: now,
        }
    }

    pub fn record_usage(&mut self, resource_type: ResourceType, cost: Decimal) {
        let now = Utc::now();
        
        // Update windows if needed
        self.maybe_reset_windows(now);
        
        // Record in all applicable windows
        self.current_minute.add_usage(resource_type.clone(), cost);
        self.current_hour.add_usage(resource_type.clone(), cost);
        self.current_day.add_usage(resource_type.clone(), cost);
        self.current_month.add_usage(resource_type, cost);
        
        self.last_updated = now;
    }

    fn maybe_reset_windows(&mut self, now: DateTime<Utc>) {
        let minute_boundary = now.with_second(0).unwrap().with_nanosecond(0).unwrap();
        let hour_boundary = now.with_minute(0).unwrap().with_second(0).unwrap().with_nanosecond(0).unwrap();
        let day_boundary = now.date_naive().and_hms_opt(0, 0, 0).unwrap().and_utc();
        let month_boundary = now.date_naive().with_day(1).unwrap().and_hms_opt(0, 0, 0).unwrap().and_utc();

        if self.current_minute.window_start < minute_boundary {
            self.current_minute = UsageWindow::new(minute_boundary);
        }
        if self.current_hour.window_start < hour_boundary {
            self.current_hour = UsageWindow::new(hour_boundary);
        }
        if self.current_day.window_start < day_boundary {
            self.current_day = UsageWindow::new(day_boundary);
        }
        if self.current_month.window_start < month_boundary {
            self.current_month = UsageWindow::new(month_boundary);
        }
    }
}

impl UsageWindow {
    pub fn new(window_start: DateTime<Utc>) -> Self {
        Self {
            window_start,
            total_requests: 0,
            total_cost: Decimal::ZERO,
            resource_counts: HashMap::new(),
            category_counts: HashMap::new(),
        }
    }

    pub fn add_usage(&mut self, resource_type: ResourceType, cost: Decimal) {
        self.total_requests += 1;
        self.total_cost += cost;
        *self.resource_counts.entry(resource_type.clone()).or_insert(0) += 1;
        *self.category_counts.entry(resource_type.category()).or_insert(0) += 1;
    }
}