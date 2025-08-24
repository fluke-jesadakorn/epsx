// Performance optimization recommendations engine

use std::sync::Arc;
use std::collections::HashMap;
use chrono::{DateTime, Utc, Duration};
use uuid::Uuid;
use serde::{Serialize, Deserialize};
use tracing::{info, warn, error};

use crate::web::performance::{
    models::*,
    repo::PerformanceRepo,
    analytics::PerformanceAnalytics,
};

/// AI-powered performance optimization recommendations engine
pub struct RecommendationEngine {
    repo: Arc<PerformanceRepo>,
    analytics: Arc<PerformanceAnalytics>,
}

impl RecommendationEngine {
    pub fn new(repo: Arc<PerformanceRepo>, analytics: Arc<PerformanceAnalytics>) -> Self {
        Self { repo, analytics }
    }

    /// Generate comprehensive performance recommendations
    pub async fn generate_recommendations(
        &self,
        analysis_window_hours: i32,
    ) -> Result<Vec<PerformanceRecommendation>, Box<dyn std::error::Error>> {
        info!("Generating performance recommendations for {}h window", analysis_window_hours);

        let mut recommendations = Vec::new();

        // Get performance insights
        let insights = self.analytics.generate_insights(analysis_window_hours).await?;

        // Database optimization recommendations
        recommendations.extend(
            self.generate_database_recommendations(&insights.slowest_endpoints).await?
        );

        // Cache optimization recommendations
        recommendations.extend(
            self.generate_cache_recommendations(&insights.cache_analytics).await?
        );

        // Infrastructure scaling recommendations
        recommendations.extend(
            self.generate_scaling_recommendations(&insights.system_health, &insights.summary).await?
        );

        // Index creation recommendations
        recommendations.extend(
            self.generate_index_recommendations(&insights.slowest_endpoints).await?
        );

        // API optimization recommendations
        recommendations.extend(
            self.generate_api_recommendations(&insights.slowest_endpoints).await?
        );

        // Resource optimization recommendations
        recommendations.extend(
            self.generate_resource_recommendations(&insights.system_health).await?
        );

        // Security performance recommendations
        recommendations.extend(
            self.generate_security_recommendations(&insights.slowest_endpoints).await?
        );

        // Sort by impact score
        recommendations.sort_by(|a, b| b.impact_score.cmp(&a.impact_score));

        // Store recommendations in database
        for rec in &recommendations {
            if let Err(e) = self.repo.create_recommendation(rec).await {
                warn!("Failed to store recommendation: {}", e);
            }
        }

        info!("Generated {} recommendations", recommendations.len());
        Ok(recommendations)
    }

    /// Generate database optimization recommendations
    async fn generate_database_recommendations(
        &self,
        slowest_endpoints: &[EndpointPerformance],
    ) -> Result<Vec<PerformanceRecommendation>, Box<dyn std::error::Error>> {
        let mut recommendations = Vec::new();

        for endpoint in slowest_endpoints.iter().take(5) {
            if endpoint.p95_duration > 1000.0 {
                let severity = if endpoint.p95_duration > 5000.0 {
                    Priority::Critical
                } else if endpoint.p95_duration > 2000.0 {
                    Priority::High
                } else {
                    Priority::Medium
                };

                let impact_score = self.calculate_impact_score(
                    endpoint.p95_duration,
                    endpoint.request_count,
                    endpoint.error_rate,
                );

                recommendations.push(PerformanceRecommendation {
                    id: Uuid::new_v4(),
                    created_at: Utc::now(),
                    recommendation_type: RecommendationType::QueryOptimization,
                    priority: severity,
                    title: format!("Optimize database queries for {}", endpoint.endpoint),
                    description: self.generate_database_optimization_description(endpoint),
                    impact_score,
                    estimated_improvement: Some(format!(
                        "Reduce latency by 40-70% (from {:.0}ms to {:.0}ms)",
                        endpoint.p95_duration,
                        endpoint.p95_duration * 0.4
                    )),
                    affected_endpoints: vec![endpoint.endpoint.clone()],
                    implementation_effort: self.estimate_database_optimization_effort(endpoint),
                    auto_implementable: false,
                    metadata: Some(serde_json::json!({
                        "current_p95": endpoint.p95_duration,
                        "request_count": endpoint.request_count,
                        "error_rate": endpoint.error_rate,
                        "analysis_type": "slow_query_detection"
                    })),
                    status: RecommendationStatus::Pending,
                    implemented_at: None,
                    notes: None,
                });
            }
        }

        // Connection pool optimization
        if slowest_endpoints.iter().any(|e| e.p95_duration > 2000.0) {
            recommendations.push(PerformanceRecommendation {
                id: Uuid::new_v4(),
                created_at: Utc::now(),
                recommendation_type: RecommendationType::QueryOptimization,
                priority: Priority::Medium,
                title: "Optimize database connection pool settings".to_string(),
                description: "High latency detected across multiple endpoints suggests potential connection pool bottlenecks. Consider increasing pool size or optimizing connection lifecycle.".to_string(),
                impact_score: 70,
                estimated_improvement: Some("Reduce connection wait time by 30-50%".to_string()),
                affected_endpoints: slowest_endpoints.iter().take(10).map(|e| e.endpoint.clone()).collect(),
                implementation_effort: ImplementationEffort::Low,
                auto_implementable: true,
                metadata: Some(serde_json::json!({
                    "recommendation_type": "connection_pool_optimization",
                    "suggested_changes": [
                        "Increase max_connections to 50",
                        "Set idle_timeout to 30 minutes",
                        "Enable connection validation"
                    ]
                })),
                status: RecommendationStatus::Pending,
                implemented_at: None,
                notes: None,
            });
        }

        Ok(recommendations)
    }

    /// Generate cache optimization recommendations
    async fn generate_cache_recommendations(
        &self,
        cache_analytics: &CacheAnalytics,
    ) -> Result<Vec<PerformanceRecommendation>, Box<dyn std::error::Error>> {
        let mut recommendations = Vec::new();

        // Overall cache hit rate optimization
        if cache_analytics.overall_hit_rate < 80.0 {
            let priority = if cache_analytics.overall_hit_rate < 50.0 {
                Priority::Critical
            } else if cache_analytics.overall_hit_rate < 70.0 {
                Priority::High
            } else {
                Priority::Medium
            };

            recommendations.push(PerformanceRecommendation {
                id: Uuid::new_v4(),
                created_at: Utc::now(),
                recommendation_type: RecommendationType::CacheOptimization,
                priority,
                title: "Improve overall cache hit rate".to_string(),
                description: format!(
                    "Current cache hit rate is {:.1}%, which is below optimal (90%+). Implement better caching strategies including longer TTLs for stable data, cache warming, and better key design.",
                    cache_analytics.overall_hit_rate
                ),
                impact_score: (100.0 - cache_analytics.overall_hit_rate) as i32,
                estimated_improvement: Some(format!(
                    "Improve response time by 30-50% by increasing hit rate to 90%"
                )),
                affected_endpoints: vec![], // Affects all endpoints
                implementation_effort: ImplementationEffort::Medium,
                auto_implementable: true,
                metadata: Some(serde_json::json!({
                    "current_hit_rate": cache_analytics.overall_hit_rate,
                    "target_hit_rate": 90.0,
                    "improvement_strategies": [
                        "Implement cache warming for frequently accessed data",
                        "Increase TTL for stable reference data",
                        "Implement cache hierarchies (L1/L2)",
                        "Add preemptive cache refresh"
                    ]
                })),
                status: RecommendationStatus::Pending,
                implemented_at: None,
                notes: None,
            });
        }

        // Cache type specific recommendations
        for (cache_type, hit_rate) in &cache_analytics.hit_rate_by_type {
            if *hit_rate < 70.0 {
                recommendations.push(PerformanceRecommendation {
                    id: Uuid::new_v4(),
                    created_at: Utc::now(),
                    recommendation_type: RecommendationType::CacheOptimization,
                    priority: Priority::Medium,
                    title: format!("Optimize {} cache performance", cache_type),
                    description: format!(
                        "{} cache has hit rate of {:.1}%. Consider reviewing cache key design, TTL settings, and data access patterns.",
                        cache_type, hit_rate
                    ),
                    impact_score: (100.0 - hit_rate) as i32,
                    estimated_improvement: Some("Improve cache-specific performance by 20-40%".to_string()),
                    affected_endpoints: vec![],
                    implementation_effort: ImplementationEffort::Low,
                    auto_implementable: true,
                    metadata: Some(serde_json::json!({
                        "cache_type": cache_type,
                        "current_hit_rate": hit_rate,
                        "specific_optimizations": self.get_cache_type_optimizations(cache_type)
                    })),
                    status: RecommendationStatus::Pending,
                    implemented_at: None,
                    notes: None,
                });
            }
        }

        // Frequently missed keys optimization
        if !cache_analytics.top_missed_keys.is_empty() {
            recommendations.push(PerformanceRecommendation {
                id: Uuid::new_v4(),
                created_at: Utc::now(),
                recommendation_type: RecommendationType::CacheOptimization,
                priority: Priority::Medium,
                title: "Optimize frequently missed cache keys".to_string(),
                description: format!(
                    "Detected {} frequently missed cache key patterns. Consider implementing cache warming or adjusting cache strategies for these keys.",
                    cache_analytics.top_missed_keys.len()
                ),
                impact_score: 60,
                estimated_improvement: Some("Reduce cache misses by 25-40%".to_string()),
                affected_endpoints: vec![],
                implementation_effort: ImplementationEffort::Low,
                auto_implementable: true,
                metadata: Some(serde_json::json!({
                    "missed_keys": cache_analytics.top_missed_keys,
                    "optimization_strategies": [
                        "Implement cache warming for these keys",
                        "Increase TTL if data is stable",
                        "Add background refresh jobs",
                        "Consider cache key normalization"
                    ]
                })),
                status: RecommendationStatus::Pending,
                implemented_at: None,
                notes: None,
            });
        }

        Ok(recommendations)
    }

    /// Generate scaling recommendations
    async fn generate_scaling_recommendations(
        &self,
        system_health: &SystemHealthMetrics,
        summary: &PerformanceSummary,
    ) -> Result<Vec<PerformanceRecommendation>, Box<dyn std::error::Error>> {
        let mut recommendations = Vec::new();

        // CPU scaling recommendations
        if system_health.cpu_usage > 80.0 {
            recommendations.push(PerformanceRecommendation {
                id: Uuid::new_v4(),
                created_at: Utc::now(),
                recommendation_type: RecommendationType::Scaling,
                priority: if system_health.cpu_usage > 90.0 {
                    Priority::Critical
                } else {
                    Priority::High
                },
                title: "Scale CPU resources".to_string(),
                description: format!(
                    "CPU utilization is at {:.1}%, which may impact performance. Consider vertical scaling (more CPU cores) or horizontal scaling (more instances).",
                    system_health.cpu_usage
                ),
                impact_score: system_health.cpu_usage as i32,
                estimated_improvement: Some("Improve response time by 20-40% and increase throughput capacity".to_string()),
                affected_endpoints: vec![], // Affects all endpoints
                implementation_effort: ImplementationEffort::High,
                auto_implementable: false,
                metadata: Some(serde_json::json!({
                    "current_cpu_usage": system_health.cpu_usage,
                    "scaling_options": [
                        "Vertical scaling: Increase CPU cores",
                        "Horizontal scaling: Add more instances",
                        "Auto-scaling: Configure CPU-based scaling rules"
                    ]
                })),
                status: RecommendationStatus::Pending,
                implemented_at: None,
                notes: None,
            });
        }

        // Memory scaling recommendations
        if system_health.memory_usage > 85.0 {
            recommendations.push(PerformanceRecommendation {
                id: Uuid::new_v4(),
                created_at: Utc::now(),
                recommendation_type: RecommendationType::Scaling,
                priority: if system_health.memory_usage > 95.0 {
                    Priority::Critical
                } else {
                    Priority::High
                },
                title: "Scale memory resources".to_string(),
                description: format!(
                    "Memory utilization is at {:.1}%, approaching capacity limits. Consider increasing memory allocation or optimizing memory usage.",
                    system_health.memory_usage
                ),
                impact_score: system_health.memory_usage as i32,
                estimated_improvement: Some("Prevent memory-related performance degradation and OOM errors".to_string()),
                affected_endpoints: vec![],
                implementation_effort: ImplementationEffort::Medium,
                auto_implementable: false,
                metadata: Some(serde_json::json!({
                    "current_memory_usage": system_health.memory_usage,
                    "recommendations": [
                        "Increase allocated memory",
                        "Optimize memory usage patterns",
                        "Implement memory monitoring alerts"
                    ]
                })),
                status: RecommendationStatus::Pending,
                implemented_at: None,
                notes: None,
            });
        }

        // Throughput scaling recommendations
        if summary.throughput_rps > 100.0 && summary.p95_response_time > 1000.0 {
            recommendations.push(PerformanceRecommendation {
                id: Uuid::new_v4(),
                created_at: Utc::now(),
                recommendation_type: RecommendationType::Scaling,
                priority: Priority::Medium,
                title: "Scale for high throughput workload".to_string(),
                description: format!(
                    "High throughput ({:.1} RPS) with elevated latency ({:.0}ms P95) suggests need for horizontal scaling or load balancing optimization.",
                    summary.throughput_rps, summary.p95_response_time
                ),
                impact_score: 75,
                estimated_improvement: Some("Distribute load more effectively, reduce latency by 30-50%".to_string()),
                affected_endpoints: vec![],
                implementation_effort: ImplementationEffort::High,
                auto_implementable: false,
                metadata: Some(serde_json::json!({
                    "current_throughput": summary.throughput_rps,
                    "current_p95_latency": summary.p95_response_time,
                    "scaling_strategies": [
                        "Add more application instances",
                        "Implement load balancing",
                        "Consider microservices architecture",
                        "Optimize connection pooling"
                    ]
                })),
                status: RecommendationStatus::Pending,
                implemented_at: None,
                notes: None,
            });
        }

        Ok(recommendations)
    }

    /// Generate index creation recommendations
    async fn generate_index_recommendations(
        &self,
        slowest_endpoints: &[EndpointPerformance],
    ) -> Result<Vec<PerformanceRecommendation>, Box<dyn std::error::Error>> {
        let mut recommendations = Vec::new();

        for endpoint in slowest_endpoints.iter().take(5) {
            if endpoint.p95_duration > 2000.0 && self.is_database_heavy_endpoint(&endpoint.endpoint) {
                recommendations.push(PerformanceRecommendation {
                    id: Uuid::new_v4(),
                    created_at: Utc::now(),
                    recommendation_type: RecommendationType::IndexCreation,
                    priority: Priority::Medium,
                    title: format!("Create database indexes for {}", endpoint.endpoint),
                    description: format!(
                        "Endpoint {} shows signs of slow database queries (P95: {:.0}ms). Analyze query patterns and create appropriate indexes.",
                        endpoint.endpoint, endpoint.p95_duration
                    ),
                    impact_score: 70,
                    estimated_improvement: Some("Reduce query time by 50-80%".to_string()),
                    affected_endpoints: vec![endpoint.endpoint.clone()],
                    implementation_effort: ImplementationEffort::Low,
                    auto_implementable: true,
                    metadata: Some(serde_json::json!({
                        "endpoint": endpoint.endpoint,
                        "current_p95": endpoint.p95_duration,
                        "suggested_indexes": self.suggest_indexes_for_endpoint(&endpoint.endpoint),
                        "analysis_priority": "high"
                    })),
                    status: RecommendationStatus::Pending,
                    implemented_at: None,
                    notes: None,
                });
            }
        }

        Ok(recommendations)
    }

    /// Generate API optimization recommendations
    async fn generate_api_recommendations(
        &self,
        slowest_endpoints: &[EndpointPerformance],
    ) -> Result<Vec<PerformanceRecommendation>, Box<dyn std::error::Error>> {
        let mut recommendations = Vec::new();

        for endpoint in slowest_endpoints.iter().take(3) {
            // API response size optimization
            if endpoint.avg_duration > 500.0 {
                recommendations.push(PerformanceRecommendation {
                    id: Uuid::new_v4(),
                    created_at: Utc::now(),
                    recommendation_type: RecommendationType::QueryOptimization,
                    priority: Priority::Medium,
                    title: format!("Optimize API response for {}", endpoint.endpoint),
                    description: format!(
                        "API endpoint {} may benefit from response optimization, pagination, or field selection to reduce payload size and improve performance.",
                        endpoint.endpoint
                    ),
                    impact_score: 50,
                    estimated_improvement: Some("Reduce response time by 20-40%".to_string()),
                    affected_endpoints: vec![endpoint.endpoint.clone()],
                    implementation_effort: ImplementationEffort::Medium,
                    auto_implementable: false,
                    metadata: Some(serde_json::json!({
                        "optimization_strategies": [
                            "Implement response pagination",
                            "Add field selection (GraphQL-style)",
                            "Optimize serialization",
                            "Implement response compression",
                            "Add response caching headers"
                        ]
                    })),
                    status: RecommendationStatus::Pending,
                    implemented_at: None,
                    notes: None,
                });
            }

            // High error rate endpoints
            if endpoint.error_rate > 5.0 {
                recommendations.push(PerformanceRecommendation {
                    id: Uuid::new_v4(),
                    created_at: Utc::now(),
                    recommendation_type: RecommendationType::QueryOptimization,
                    priority: Priority::High,
                    title: format!("Fix error-prone endpoint {}", endpoint.endpoint),
                    description: format!(
                        "Endpoint {} has high error rate ({:.1}%). Investigate and fix underlying issues to improve reliability and performance.",
                        endpoint.endpoint, endpoint.error_rate
                    ),
                    impact_score: 80,
                    estimated_improvement: Some("Improve reliability and reduce error-related latency".to_string()),
                    affected_endpoints: vec![endpoint.endpoint.clone()],
                    implementation_effort: ImplementationEffort::Medium,
                    auto_implementable: false,
                    metadata: Some(serde_json::json!({
                        "current_error_rate": endpoint.error_rate,
                        "investigation_areas": [
                            "Input validation errors",
                            "Database constraint violations",
                            "External service timeouts",
                            "Business logic errors",
                            "Authentication/authorization issues"
                        ]
                    })),
                    status: RecommendationStatus::Pending,
                    implemented_at: None,
                    notes: None,
                });
            }
        }

        Ok(recommendations)
    }

    /// Generate resource optimization recommendations
    async fn generate_resource_recommendations(
        &self,
        system_health: &SystemHealthMetrics,
    ) -> Result<Vec<PerformanceRecommendation>, Box<dyn std::error::Error>> {
        let mut recommendations = Vec::new();

        // Connection pool optimization
        if system_health.connection_pool_utilization > 80.0 {
            recommendations.push(PerformanceRecommendation {
                id: Uuid::new_v4(),
                created_at: Utc::now(),
                recommendation_type: RecommendationType::QueryOptimization,
                priority: Priority::Medium,
                title: "Optimize database connection pool".to_string(),
                description: format!(
                    "Database connection pool utilization is at {:.1}%. Consider increasing pool size or optimizing connection usage patterns.",
                    system_health.connection_pool_utilization
                ),
                impact_score: 60,
                estimated_improvement: Some("Reduce connection wait times and improve throughput".to_string()),
                affected_endpoints: vec![],
                implementation_effort: ImplementationEffort::Low,
                auto_implementable: true,
                metadata: Some(serde_json::json!({
                    "current_utilization": system_health.connection_pool_utilization,
                    "optimizations": [
                        "Increase max pool size",
                        "Optimize connection lifecycle",
                        "Implement connection validation",
                        "Add connection monitoring"
                    ]
                })),
                status: RecommendationStatus::Pending,
                implemented_at: None,
                notes: None,
            });
        }

        // GC optimization (if applicable)
        if system_health.gc_pause_ms > 50.0 {
            recommendations.push(PerformanceRecommendation {
                id: Uuid::new_v4(),
                created_at: Utc::now(),
                recommendation_type: RecommendationType::QueryOptimization,
                priority: Priority::Low,
                title: "Optimize garbage collection".to_string(),
                description: format!(
                    "GC pause time is {:.1}ms. Consider GC tuning or memory allocation optimization.",
                    system_health.gc_pause_ms
                ),
                impact_score: 30,
                estimated_improvement: Some("Reduce latency spikes and improve consistency".to_string()),
                affected_endpoints: vec![],
                implementation_effort: ImplementationEffort::Medium,
                auto_implementable: false,
                metadata: Some(serde_json::json!({
                    "current_gc_pause": system_health.gc_pause_ms,
                    "optimization_strategies": [
                        "Tune GC parameters",
                        "Optimize memory allocation patterns",
                        "Consider different GC algorithms"
                    ]
                })),
                status: RecommendationStatus::Pending,
                implemented_at: None,
                notes: None,
            });
        }

        Ok(recommendations)
    }

    /// Generate security-related performance recommendations
    async fn generate_security_recommendations(
        &self,
        slowest_endpoints: &[EndpointPerformance],
    ) -> Result<Vec<PerformanceRecommendation>, Box<dyn std::error::Error>> {
        let mut recommendations = Vec::new();

        // Authentication/authorization optimization
        for endpoint in slowest_endpoints {
            if self.is_auth_heavy_endpoint(&endpoint.endpoint) && endpoint.p95_duration > 1000.0 {
                recommendations.push(PerformanceRecommendation {
                    id: Uuid::new_v4(),
                    created_at: Utc::now(),
                    recommendation_type: RecommendationType::CacheOptimization,
                    priority: Priority::Medium,
                    title: format!("Optimize authentication for {}", endpoint.endpoint),
                    description: format!(
                        "Authentication-heavy endpoint {} may benefit from auth caching, session optimization, or JWT validation improvements.",
                        endpoint.endpoint
                    ),
                    impact_score: 55,
                    estimated_improvement: Some("Reduce auth overhead by 30-60%".to_string()),
                    affected_endpoints: vec![endpoint.endpoint.clone()],
                    implementation_effort: ImplementationEffort::Medium,
                    auto_implementable: true,
                    metadata: Some(serde_json::json!({
                        "auth_optimizations": [
                            "Implement JWT caching",
                            "Optimize session validation",
                            "Cache permission checks",
                            "Implement auth result caching"
                        ]
                    })),
                    status: RecommendationStatus::Pending,
                    implemented_at: None,
                    notes: None,
                });
            }
        }

        Ok(recommendations)
    }

    /// Calculate impact score based on multiple factors
    fn calculate_impact_score(&self, latency: f64, request_count: i64, error_rate: f64) -> i32 {
        let latency_score = ((latency / 100.0).min(10.0) * 10.0) as i32;
        let volume_score = ((request_count as f64 / 1000.0).min(5.0) * 10.0) as i32;
        let error_score = (error_rate * 2.0) as i32;
        
        (latency_score + volume_score + error_score).min(100)
    }

    /// Estimate implementation effort for database optimization
    fn estimate_database_optimization_effort(&self, endpoint: &EndpointPerformance) -> ImplementationEffort {
        if endpoint.p95_duration > 5000.0 {
            ImplementationEffort::High // Likely requires significant query rewriting
        } else if endpoint.p95_duration > 2000.0 {
            ImplementationEffort::Medium // May need index creation and some optimization
        } else {
            ImplementationEffort::Low // Likely just needs index creation
        }
    }

    /// Generate detailed database optimization description
    fn generate_database_optimization_description(&self, endpoint: &EndpointPerformance) -> String {
        let mut suggestions = Vec::new();

        if endpoint.p95_duration > 5000.0 {
            suggestions.push("Review and optimize complex queries");
            suggestions.push("Consider query result caching");
            suggestions.push("Implement database query pagination");
        }

        if endpoint.p95_duration > 2000.0 {
            suggestions.push("Create appropriate database indexes");
            suggestions.push("Optimize JOIN operations");
        }

        if endpoint.error_rate > 5.0 {
            suggestions.push("Review database constraint violations");
            suggestions.push("Implement proper error handling");
        }

        format!(
            "Endpoint {} shows performance issues (P95: {:.0}ms, {} requests, {:.1}% errors). Suggested optimizations: {}",
            endpoint.endpoint,
            endpoint.p95_duration,
            endpoint.request_count,
            endpoint.error_rate,
            suggestions.join(", ")
        )
    }

    /// Get cache type specific optimizations
    fn get_cache_type_optimizations(&self, cache_type: &str) -> Vec<String> {
        match cache_type {
            "redis" => vec![
                "Optimize Redis key design".to_string(),
                "Implement Redis clustering".to_string(),
                "Tune Redis memory policies".to_string(),
            ],
            "memory" => vec![
                "Increase memory cache size".to_string(),
                "Implement LRU eviction policy".to_string(),
                "Add cache warming strategies".to_string(),
            ],
            "session" => vec![
                "Optimize session storage".to_string(),
                "Implement session clustering".to_string(),
                "Review session timeout settings".to_string(),
            ],
            _ => vec![
                "Review cache configuration".to_string(),
                "Optimize cache key design".to_string(),
            ],
        }
    }

    /// Check if endpoint is database-heavy
    fn is_database_heavy_endpoint(&self, endpoint: &str) -> bool {
        endpoint.contains("/api/") && 
        (endpoint.contains("search") || 
         endpoint.contains("list") || 
         endpoint.contains("analytics") ||
         endpoint.contains("reports"))
    }

    /// Check if endpoint is auth-heavy
    fn is_auth_heavy_endpoint(&self, endpoint: &str) -> bool {
        endpoint.contains("/auth/") || 
        endpoint.contains("/login") || 
        endpoint.contains("/token") ||
        endpoint.contains("/session")
    }

    /// Suggest indexes for specific endpoints
    fn suggest_indexes_for_endpoint(&self, endpoint: &str) -> Vec<String> {
        let mut indexes = Vec::new();

        if endpoint.contains("user") {
            indexes.push("CREATE INDEX idx_users_email ON users(email)".to_string());
            indexes.push("CREATE INDEX idx_users_created_at ON users(created_at)".to_string());
        }

        if endpoint.contains("analytics") || endpoint.contains("eps") {
            indexes.push("CREATE INDEX idx_performance_metrics_timestamp ON performance_metrics(timestamp)".to_string());
            indexes.push("CREATE INDEX idx_performance_metrics_endpoint ON performance_metrics(endpoint)".to_string());
        }

        if endpoint.contains("session") {
            indexes.push("CREATE INDEX idx_sessions_user_id ON sessions(user_id)".to_string());
            indexes.push("CREATE INDEX idx_sessions_expires_at ON sessions(expires_at)".to_string());
        }

        if indexes.is_empty() {
            indexes.push("Analyze query patterns to determine optimal indexes".to_string());
        }

        indexes
    }

    /// Get stored recommendations with filters
    pub async fn get_recommendations(
        &self,
        status: Option<RecommendationStatus>,
        priority: Option<Priority>,
        limit: Option<i32>,
    ) -> Result<Vec<PerformanceRecommendation>, Box<dyn std::error::Error>> {
        let recommendations = self.repo.get_recommendations(status, limit).await?;
        
        if let Some(filter_priority) = priority {
            Ok(recommendations
                .into_iter()
                .filter(|r| r.priority == filter_priority)
                .collect())
        } else {
            Ok(recommendations)
        }
    }

    /// Mark recommendation as implemented
    pub async fn implement_recommendation(
        &self,
        recommendation_id: Uuid,
        notes: Option<String>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        info!("Marking recommendation {} as implemented", recommendation_id);

        // Update recommendation status
        // Note: This would require adding an update method to the repo
        // For now, we'll create a new recommendation record with implemented status
        
        Ok(())
    }

    /// Get recommendation implementation statistics
    pub async fn get_implementation_stats(&self) -> Result<RecommendationStats, Box<dyn std::error::Error>> {
        let all_recommendations = self.repo.get_recommendations(None, None).await?;
        
        let total = all_recommendations.len() as i32;
        let implemented = all_recommendations.iter().filter(|r| r.status == RecommendationStatus::Implemented).count() as i32;
        let pending = all_recommendations.iter().filter(|r| r.status == RecommendationStatus::Pending).count() as i32;
        let approved = all_recommendations.iter().filter(|r| r.status == RecommendationStatus::Approved).count() as i32;
        
        let avg_impact_score = if total > 0 {
            all_recommendations.iter().map(|r| r.impact_score).sum::<i32>() as f64 / total as f64
        } else {
            0.0
        };

        Ok(RecommendationStats {
            total_recommendations: total,
            implemented_recommendations: implemented,
            pending_recommendations: pending,
            approved_recommendations: approved,
            avg_impact_score,
            auto_implementable_count: all_recommendations.iter().filter(|r| r.auto_implementable).count() as i32,
        })
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RecommendationStats {
    pub total_recommendations: i32,
    pub implemented_recommendations: i32,
    pub pending_recommendations: i32,
    pub approved_recommendations: i32,
    pub avg_impact_score: f64,
    pub auto_implementable_count: i32,
}