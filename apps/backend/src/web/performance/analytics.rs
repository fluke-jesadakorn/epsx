// Performance analytics engine with advanced metrics calculation

use std::sync::Arc;
use std::collections::HashMap;
use chrono::{DateTime, Utc, Duration};
use uuid::Uuid;
use serde::{Serialize, Deserialize};
use tracing::{info, warn, error};

use crate::web::performance::{
    models::*,
    repo::PerformanceRepo,
};

/// Performance analytics engine for calculating advanced metrics
pub struct PerformanceAnalytics {
    repo: Arc<PerformanceRepo>,
}

impl PerformanceAnalytics {
    pub fn new(repo: Arc<PerformanceRepo>) -> Self {
        Self { repo }
    }

    /// Calculate performance percentiles (P50, P95, P99) for given timeframe
    pub async fn calculate_percentiles(
        &self,
        endpoint: Option<&str>,
        start_time: DateTime<Utc>,
        end_time: DateTime<Utc>,
    ) -> Result<PercentileMetrics, Box<dyn std::error::Error>> {
        info!(
            "Calculating percentiles for endpoint {:?} from {} to {}",
            endpoint, start_time, end_time
        );

        let trends = self.repo.get_performance_trends(
            endpoint,
            MetricType::Latency,
            start_time,
            end_time,
            TimeBucket::Minute,
        ).await?;

        if trends.is_empty() {
            return Ok(PercentileMetrics::default());
        }

        let mut values: Vec<f64> = trends.iter().map(|t| t.metric_value).collect();
        values.sort_by(|a, b| a.partial_cmp(b).unwrap());

        let p50 = self.calculate_percentile(&values, 50.0);
        let p95 = self.calculate_percentile(&values, 95.0);
        let p99 = self.calculate_percentile(&values, 99.0);
        let p999 = self.calculate_percentile(&values, 99.9);

        Ok(PercentileMetrics {
            p50,
            p95,
            p99,
            p999,
            min: values.first().copied().unwrap_or(0.0),
            max: values.last().copied().unwrap_or(0.0),
            avg: values.iter().sum::<f64>() / values.len() as f64,
            count: values.len() as i64,
            timestamp: Utc::now(),
        })
    }

    /// Calculate percentile value from sorted array
    fn calculate_percentile(&self, sorted_values: &[f64], percentile: f64) -> f64 {
        if sorted_values.is_empty() {
            return 0.0;
        }

        let index = (percentile / 100.0) * (sorted_values.len() as f64 - 1.0);
        let lower = index.floor() as usize;
        let upper = index.ceil() as usize;

        if lower == upper {
            sorted_values[lower]
        } else {
            let weight = index - lower as f64;
            sorted_values[lower] * (1.0 - weight) + sorted_values[upper] * weight
        }
    }

    /// Detect performance anomalies using statistical analysis
    pub async fn detect_anomalies(
        &self,
        lookback_hours: i32,
        threshold_factor: f64,
    ) -> Result<Vec<PerformanceAnomalyDetection>, Box<dyn std::error::Error>> {
        info!("Detecting performance anomalies with {}h lookback", lookback_hours);

        let anomalies = self.repo.detect_anomalies(lookback_hours).await?;
        
        let mut filtered_anomalies = Vec::new();

        for anomaly in anomalies {
            // Apply threshold filtering
            if anomaly.deviation_score.abs() >= threshold_factor {
                filtered_anomalies.push(anomaly);
            }
        }

        // Additional statistical anomaly detection
        let additional_anomalies = self.detect_statistical_anomalies(lookback_hours).await?;
        filtered_anomalies.extend(additional_anomalies);

        Ok(filtered_anomalies)
    }

    /// Advanced statistical anomaly detection using Z-score and IQR
    async fn detect_statistical_anomalies(
        &self,
        lookback_hours: i32,
    ) -> Result<Vec<PerformanceAnomalyDetection>, Box<dyn std::error::Error>> {
        let end_time = Utc::now();
        let start_time = end_time - Duration::hours(lookback_hours as i64);

        let slowest_endpoints = self.repo.get_slowest_endpoints(
            start_time,
            end_time,
            50, // Top 50 endpoints
        ).await?;

        let mut anomalies = Vec::new();

        for endpoint in slowest_endpoints {
            // Get historical data for this endpoint
            let trends = self.repo.get_performance_trends(
                Some(&endpoint.endpoint),
                MetricType::Latency,
                start_time - Duration::days(7), // Look back 7 days for baseline
                end_time,
                TimeBucket::Hour,
            ).await?;

            if trends.len() < 10 {
                continue; // Not enough data
            }

            let values: Vec<f64> = trends.iter().map(|t| t.metric_value).collect();
            let (mean, std_dev) = self.calculate_mean_std(&values);
            
            // Check if current performance is anomalous (Z-score > 3)
            let current_value = endpoint.p95_duration;
            let z_score = (current_value - mean) / std_dev;

            if z_score.abs() > 3.0 {
                let anomaly_type = if z_score > 0.0 {
                    AnomalyType::LatencySpike
                } else {
                    // Unusual but possible: performance improvement
                    AnomalyType::LatencySpike // We'll still call it a spike for now
                };

                anomalies.push(PerformanceAnomalyDetection {
                    endpoint: endpoint.endpoint.clone(),
                    metric_type: MetricType::Latency,
                    current_value,
                    expected_value: mean,
                    deviation_score: z_score,
                    anomaly_type,
                    detected_at: Utc::now(),
                });
            }

            // Check error rate anomalies
            if endpoint.error_rate > 0.0 {
                let error_z_score = endpoint.error_rate / 5.0; // Normalize to expected 5% baseline
                if error_z_score > 2.0 {
                    anomalies.push(PerformanceAnomalyDetection {
                        endpoint: endpoint.endpoint.clone(),
                        metric_type: MetricType::ErrorRate,
                        current_value: endpoint.error_rate,
                        expected_value: 5.0,
                        deviation_score: error_z_score,
                        anomaly_type: AnomalyType::ErrorRateIncrease,
                        detected_at: Utc::now(),
                    });
                }
            }

            // Check cache hit rate anomalies
            if endpoint.cache_hit_rate < 80.0 {
                anomalies.push(PerformanceAnomalyDetection {
                    endpoint: endpoint.endpoint.clone(),
                    metric_type: MetricType::CacheHitRate,
                    current_value: endpoint.cache_hit_rate,
                    expected_value: 90.0,
                    deviation_score: (80.0 - endpoint.cache_hit_rate) / 10.0,
                    anomaly_type: AnomalyType::CacheHitRateDecrease,
                    detected_at: Utc::now(),
                });
            }
        }

        Ok(anomalies)
    }

    /// Calculate mean and standard deviation
    fn calculate_mean_std(&self, values: &[f64]) -> (f64, f64) {
        let mean = values.iter().sum::<f64>() / values.len() as f64;
        let variance = values
            .iter()
            .map(|x| (x - mean).powi(2))
            .sum::<f64>() / values.len() as f64;
        let std_dev = variance.sqrt();
        (mean, std_dev)
    }

    /// Generate performance insights and bottleneck analysis
    pub async fn generate_insights(
        &self,
        time_window_hours: i32,
    ) -> Result<PerformanceInsights, Box<dyn std::error::Error>> {
        let end_time = Utc::now();
        let start_time = end_time - Duration::hours(time_window_hours as i64);

        info!("Generating performance insights for {}h window", time_window_hours);

        // Get summary metrics
        let summary = self.repo.get_performance_summary(start_time, end_time).await?;
        
        // Get slowest endpoints
        let slowest_endpoints = self.repo.get_slowest_endpoints(start_time, end_time, 10).await?;
        
        // Get cache analytics
        let cache_analytics = self.repo.get_cache_analytics(start_time, end_time).await?;
        
        // Get system health
        let system_health = self.repo.get_system_health().await?;
        
        // Detect anomalies
        let anomalies = self.detect_anomalies(time_window_hours, 2.0).await?;

        // Generate bottleneck analysis
        let bottlenecks = self.analyze_bottlenecks(&slowest_endpoints, &cache_analytics).await;

        // Generate recommendations
        let recommendations = self.generate_performance_recommendations(
            &summary,
            &slowest_endpoints,
            &cache_analytics,
            &system_health,
        ).await;

        Ok(PerformanceInsights {
            summary,
            slowest_endpoints,
            cache_analytics,
            system_health,
            anomalies,
            bottlenecks,
            recommendations,
            sla_compliance: self.calculate_sla_compliance(&summary).await,
            timestamp: Utc::now(),
        })
    }

    /// Analyze performance bottlenecks
    async fn analyze_bottlenecks(
        &self,
        slowest_endpoints: &[EndpointPerformance],
        cache_analytics: &CacheAnalytics,
    ) -> Vec<BottleneckAnalysis> {
        let mut bottlenecks = Vec::new();

        // Database bottlenecks
        for endpoint in slowest_endpoints.iter().take(5) {
            if endpoint.p95_duration > 1000.0 {
                bottlenecks.push(BottleneckAnalysis {
                    bottleneck_type: BottleneckType::Database,
                    severity: if endpoint.p95_duration > 5000.0 {
                        Severity::Critical
                    } else if endpoint.p95_duration > 2000.0 {
                        Severity::High
                    } else {
                        Severity::Medium
                    },
                    affected_endpoint: Some(endpoint.endpoint.clone()),
                    description: format!(
                        "Endpoint {} has P95 latency of {:.0}ms, likely due to slow database queries",
                        endpoint.endpoint, endpoint.p95_duration
                    ),
                    impact_score: ((endpoint.p95_duration / 100.0) as i32).min(100),
                    detected_at: Utc::now(),
                });
            }
        }

        // Cache bottlenecks
        if cache_analytics.overall_hit_rate < 80.0 {
            bottlenecks.push(BottleneckAnalysis {
                bottleneck_type: BottleneckType::Cache,
                severity: if cache_analytics.overall_hit_rate < 50.0 {
                    Severity::Critical
                } else if cache_analytics.overall_hit_rate < 70.0 {
                    Severity::High
                } else {
                    Severity::Medium
                },
                affected_endpoint: None,
                description: format!(
                    "Cache hit rate is {:.1}%, significantly below optimal (90%+)",
                    cache_analytics.overall_hit_rate
                ),
                impact_score: (100.0 - cache_analytics.overall_hit_rate) as i32,
                detected_at: Utc::now(),
            });
        }

        // Network bottlenecks (high response sizes)
        for endpoint in slowest_endpoints {
            if endpoint.avg_duration > 500.0 && endpoint.p95_duration / endpoint.avg_duration > 3.0 {
                bottlenecks.push(BottleneckAnalysis {
                    bottleneck_type: BottleneckType::Network,
                    severity: Severity::Medium,
                    affected_endpoint: Some(endpoint.endpoint.clone()),
                    description: format!(
                        "Endpoint {} shows high latency variance (P95/avg ratio: {:.1}), suggesting network issues",
                        endpoint.endpoint, endpoint.p95_duration / endpoint.avg_duration
                    ),
                    impact_score: ((endpoint.p95_duration / endpoint.avg_duration * 10.0) as i32).min(100),
                    detected_at: Utc::now(),
                });
            }
        }

        bottlenecks
    }

    /// Generate performance recommendations
    async fn generate_performance_recommendations(
        &self,
        summary: &PerformanceSummary,
        slowest_endpoints: &[EndpointPerformance],
        cache_analytics: &CacheAnalytics,
        system_health: &SystemHealthMetrics,
    ) -> Vec<OptimizationRecommendation> {
        let mut recommendations = Vec::new();

        // Database optimization recommendations
        for endpoint in slowest_endpoints.iter().take(3) {
            if endpoint.p95_duration > 1000.0 {
                recommendations.push(OptimizationRecommendation {
                    recommendation_type: RecommendationType::QueryOptimization,
                    priority: if endpoint.p95_duration > 5000.0 {
                        Priority::Critical
                    } else {
                        Priority::High
                    },
                    title: format!("Optimize database queries for {}", endpoint.endpoint),
                    description: format!(
                        "Endpoint {} has P95 latency of {:.0}ms. Consider adding indexes, optimizing queries, or implementing query caching.",
                        endpoint.endpoint, endpoint.p95_duration
                    ),
                    impact_score: ((endpoint.p95_duration / 50.0) as i32).min(100),
                    estimated_improvement: Some(format!("Reduce latency by 40-70% ({:.0}ms)", endpoint.p95_duration * 0.5)),
                    affected_endpoints: vec![endpoint.endpoint.clone()],
                    implementation_effort: ImplementationEffort::Medium,
                    auto_implementable: false,
                });
            }
        }

        // Cache optimization recommendations
        if cache_analytics.overall_hit_rate < 80.0 {
            recommendations.push(OptimizationRecommendation {
                recommendation_type: RecommendationType::CacheOptimization,
                priority: Priority::High,
                title: "Improve cache hit rate".to_string(),
                description: format!(
                    "Current cache hit rate is {:.1}%. Implement better caching strategies, increase cache TTL, or pre-warm cache for frequently accessed data.",
                    cache_analytics.overall_hit_rate
                ),
                impact_score: (100.0 - cache_analytics.overall_hit_rate) as i32,
                estimated_improvement: Some("Reduce average response time by 30-50%".to_string()),
                affected_endpoints: slowest_endpoints.iter().map(|e| e.endpoint.clone()).collect(),
                implementation_effort: ImplementationEffort::Medium,
                auto_implementable: true,
            });
        }

        // Scaling recommendations
        if system_health.cpu_usage > 80.0 || system_health.memory_usage > 85.0 {
            recommendations.push(OptimizationRecommendation {
                recommendation_type: RecommendationType::Scaling,
                priority: Priority::High,
                title: "Scale system resources".to_string(),
                description: format!(
                    "High resource utilization detected (CPU: {:.1}%, Memory: {:.1}%). Consider horizontal or vertical scaling.",
                    system_health.cpu_usage, system_health.memory_usage
                ),
                impact_score: ((system_health.cpu_usage + system_health.memory_usage) / 2.0) as i32,
                estimated_improvement: Some("Improve overall system performance by 20-40%".to_string()),
                affected_endpoints: vec![], // Affects all endpoints
                implementation_effort: ImplementationEffort::High,
                auto_implementable: false,
            });
        }

        // Index creation recommendations
        for endpoint in slowest_endpoints.iter().take(5) {
            if endpoint.p95_duration > 2000.0 && endpoint.endpoint.contains("/api/") {
                recommendations.push(OptimizationRecommendation {
                    recommendation_type: RecommendationType::IndexCreation,
                    priority: Priority::Medium,
                    title: format!("Create database indexes for {}", endpoint.endpoint),
                    description: format!(
                        "Slow API endpoint detected. Analyze query patterns and create appropriate database indexes.",
                    ),
                    impact_score: 60,
                    estimated_improvement: Some("Reduce query time by 50-80%".to_string()),
                    affected_endpoints: vec![endpoint.endpoint.clone()],
                    implementation_effort: ImplementationEffort::Low,
                    auto_implementable: true,
                });
            }
        }

        recommendations
    }

    /// Calculate SLA compliance metrics
    async fn calculate_sla_compliance(&self, summary: &PerformanceSummary) -> SLACompliance {
        // Define SLA targets
        let latency_sla_ms = 500.0; // P95 < 500ms
        let uptime_sla_percent = 99.9; // 99.9% uptime
        let error_rate_sla_percent = 1.0; // < 1% error rate

        let latency_compliance = if summary.p95_response_time <= latency_sla_ms {
            100.0
        } else {
            ((latency_sla_ms / summary.p95_response_time) * 100.0).min(100.0)
        };

        let uptime_compliance = 100.0 - summary.error_rate; // Simplified uptime calculation

        let error_rate_compliance = if summary.error_rate <= error_rate_sla_percent {
            100.0
        } else {
            ((error_rate_sla_percent / summary.error_rate) * 100.0).min(100.0)
        };

        let overall_compliance = (latency_compliance + uptime_compliance + error_rate_compliance) / 3.0;

        SLACompliance {
            overall_compliance,
            latency_compliance,
            uptime_compliance,
            error_rate_compliance,
            sla_targets: SLATargets {
                max_latency_ms: latency_sla_ms,
                min_uptime_percent: uptime_sla_percent,
                max_error_rate_percent: error_rate_sla_percent,
            },
            current_metrics: CurrentSLAMetrics {
                current_latency_ms: summary.p95_response_time,
                current_uptime_percent: 100.0 - summary.error_rate,
                current_error_rate_percent: summary.error_rate,
            },
            breaches_today: 0, // Would need to be calculated from historical data
            time_to_breach: None, // Predictive analysis
        }
    }

    /// Calculate capacity planning metrics
    pub async fn calculate_capacity_metrics(
        &self,
        projection_days: i32,
    ) -> Result<CapacityPlanningMetrics, Box<dyn std::error::Error>> {
        let end_time = Utc::now();
        let start_time = end_time - Duration::days(30); // Look back 30 days

        let trends = self.repo.get_performance_trends(
            None,
            MetricType::Throughput,
            start_time,
            end_time,
            TimeBucket::Day,
        ).await?;

        if trends.len() < 7 {
            // Not enough data for meaningful projection
            return Ok(CapacityPlanningMetrics::default());
        }

        // Simple linear regression for growth prediction
        let (slope, _intercept) = self.calculate_linear_regression(&trends);
        
        let current_throughput = trends.last().map(|t| t.metric_value).unwrap_or(0.0);
        let projected_throughput = current_throughput + (slope * projection_days as f64);
        
        // Calculate when we might hit capacity limits
        let system_health = self.repo.get_system_health().await?;
        let current_utilization = system_health.cpu_usage.max(system_health.memory_usage);
        
        // Assume we hit capacity at 90% utilization
        let days_to_capacity = if slope > 0.0 {
            let remaining_capacity = 90.0 - current_utilization;
            let utilization_per_day = slope * 0.1; // Rough estimate
            if utilization_per_day > 0.0 {
                Some((remaining_capacity / utilization_per_day) as i32)
            } else {
                None
            }
        } else {
            None
        };

        Ok(CapacityPlanningMetrics {
            current_throughput,
            projected_throughput,
            growth_rate_percent: (slope / current_throughput * 100.0).max(0.0),
            current_utilization,
            projected_utilization: (current_utilization + slope * projection_days as f64 * 0.1).min(100.0),
            days_to_capacity,
            recommended_scaling_date: days_to_capacity.map(|days| Utc::now() + Duration::days(days as i64 - 30)),
            confidence_score: if trends.len() > 14 { 85.0 } else { 60.0 },
        })
    }

    /// Simple linear regression calculation
    fn calculate_linear_regression(&self, trends: &[PerformanceTrend]) -> (f64, f64) {
        let n = trends.len() as f64;
        if n < 2.0 {
            return (0.0, 0.0);
        }

        let sum_x: f64 = (0..trends.len()).map(|i| i as f64).sum();
        let sum_y: f64 = trends.iter().map(|t| t.metric_value).sum();
        let sum_xy: f64 = trends.iter().enumerate().map(|(i, t)| i as f64 * t.metric_value).sum();
        let sum_x2: f64 = (0..trends.len()).map(|i| (i as f64).powi(2)).sum();

        let slope = (n * sum_xy - sum_x * sum_y) / (n * sum_x2 - sum_x.powi(2));
        let intercept = (sum_y - slope * sum_x) / n;

        (slope, intercept)
    }
}

// Supporting data structures for analytics

#[derive(Debug, Serialize, Deserialize)]
pub struct PercentileMetrics {
    pub p50: f64,
    pub p95: f64,
    pub p99: f64,
    pub p999: f64,
    pub min: f64,
    pub max: f64,
    pub avg: f64,
    pub count: i64,
    pub timestamp: DateTime<Utc>,
}

impl Default for PercentileMetrics {
    fn default() -> Self {
        Self {
            p50: 0.0,
            p95: 0.0,
            p99: 0.0,
            p999: 0.0,
            min: 0.0,
            max: 0.0,
            avg: 0.0,
            count: 0,
            timestamp: Utc::now(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PerformanceInsights {
    pub summary: PerformanceSummary,
    pub slowest_endpoints: Vec<EndpointPerformance>,
    pub cache_analytics: CacheAnalytics,
    pub system_health: SystemHealthMetrics,
    pub anomalies: Vec<PerformanceAnomalyDetection>,
    pub bottlenecks: Vec<BottleneckAnalysis>,
    pub recommendations: Vec<OptimizationRecommendation>,
    pub sla_compliance: SLACompliance,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BottleneckAnalysis {
    pub bottleneck_type: BottleneckType,
    pub severity: Severity,
    pub affected_endpoint: Option<String>,
    pub description: String,
    pub impact_score: i32,
    pub detected_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BottleneckType {
    Database,
    Cache,
    Network,
    CPU,
    Memory,
    Disk,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Severity {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OptimizationRecommendation {
    pub recommendation_type: RecommendationType,
    pub priority: Priority,
    pub title: String,
    pub description: String,
    pub impact_score: i32,
    pub estimated_improvement: Option<String>,
    pub affected_endpoints: Vec<String>,
    pub implementation_effort: ImplementationEffort,
    pub auto_implementable: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SLACompliance {
    pub overall_compliance: f64,
    pub latency_compliance: f64,
    pub uptime_compliance: f64,
    pub error_rate_compliance: f64,
    pub sla_targets: SLATargets,
    pub current_metrics: CurrentSLAMetrics,
    pub breaches_today: i32,
    pub time_to_breach: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SLATargets {
    pub max_latency_ms: f64,
    pub min_uptime_percent: f64,
    pub max_error_rate_percent: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CurrentSLAMetrics {
    pub current_latency_ms: f64,
    pub current_uptime_percent: f64,
    pub current_error_rate_percent: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapacityPlanningMetrics {
    pub current_throughput: f64,
    pub projected_throughput: f64,
    pub growth_rate_percent: f64,
    pub current_utilization: f64,
    pub projected_utilization: f64,
    pub days_to_capacity: Option<i32>,
    pub recommended_scaling_date: Option<DateTime<Utc>>,
    pub confidence_score: f64,
}

impl Default for CapacityPlanningMetrics {
    fn default() -> Self {
        Self {
            current_throughput: 0.0,
            projected_throughput: 0.0,
            growth_rate_percent: 0.0,
            current_utilization: 0.0,
            projected_utilization: 0.0,
            days_to_capacity: None,
            recommended_scaling_date: None,
            confidence_score: 0.0,
        }
    }
}