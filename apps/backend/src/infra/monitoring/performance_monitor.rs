// Performance monitoring for permission system
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Performance metrics for permission operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PermissionMetrics {
    pub operation: String,
    pub duration_ms: u64,
    pub timestamp: DateTime<Utc>,
    pub success: bool,
    pub user_id: Option<String>,
    pub resource: Option<String>,
    pub action: Option<String>,
    pub cache_hit: bool,
    pub metadata: HashMap<String, String>,
}

/// Aggregated performance statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceStats {
    pub operation: String,
    pub total_requests: u64,
    pub successful_requests: u64,
    pub failed_requests: u64,
    pub average_duration_ms: f64,
    pub min_duration_ms: u64,
    pub max_duration_ms: u64,
    pub p95_duration_ms: u64,
    pub p99_duration_ms: u64,
    pub cache_hit_rate: f64,
    pub requests_per_second: f64,
    pub last_updated: DateTime<Utc>,
}

/// Database query performance tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryMetrics {
    pub query_type: String,
    pub table: String,
    pub duration_ms: u64,
    pub rows_affected: Option<u64>,
    pub timestamp: DateTime<Utc>,
    pub success: bool,
    pub query_plan_hash: Option<String>,
}

/// System health monitoring
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemHealthMetrics {
    pub cpu_usage_percent: f64,
    pub memory_usage_bytes: u64,
    pub memory_usage_percent: f64,
    pub active_connections: u32,
    pub cache_memory_bytes: u64,
    pub permission_cache_hit_rate: f64,
    pub average_response_time_ms: f64,
    pub timestamp: DateTime<Utc>,
}

/// Performance monitor configuration
#[derive(Debug, Clone)]
pub struct PerformanceConfig {
    pub enable_detailed_metrics: bool,
    pub metrics_retention_hours: u32,
    pub alert_threshold_ms: u64,
    pub slow_query_threshold_ms: u64,
    pub stats_update_interval_seconds: u64,
}

impl Default for PerformanceConfig {
    fn default() -> Self {
        Self {
            enable_detailed_metrics: true,
            metrics_retention_hours: 24,
            alert_threshold_ms: 1000,
            slow_query_threshold_ms: 500,
            stats_update_interval_seconds: 60,
        }
    }
}

/// Main performance monitoring service
pub struct PerformanceMonitor {
    config: PerformanceConfig,
    metrics: Arc<RwLock<Vec<PermissionMetrics>>>,
    query_metrics: Arc<RwLock<Vec<QueryMetrics>>>,
    stats_cache: Arc<RwLock<HashMap<String, PerformanceStats>>>,
    alert_callbacks: Arc<RwLock<Vec<Box<dyn Fn(&PermissionMetrics) + Send + Sync>>>>,
}

impl PerformanceMonitor {
    pub fn new(config: Option<PerformanceConfig>) -> Self {
        Self {
            config: config.unwrap_or_default(),
            metrics: Arc::new(RwLock::new(Vec::new())),
            query_metrics: Arc::new(RwLock::new(Vec::new())),
            stats_cache: Arc::new(RwLock::new(HashMap::new())),
            alert_callbacks: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Record a permission operation metric
    pub async fn record_permission_metric(&self, metric: PermissionMetrics) {
        if self.config.enable_detailed_metrics {
            // Check for slow operations
            if metric.duration_ms > self.config.alert_threshold_ms {
                self.trigger_alerts(&metric).await;
            }

            let mut metrics = self.metrics.write().await;
            metrics.push(metric);

            // Clean up old metrics
            self.cleanup_old_metrics(&mut metrics).await;
        }

        // Update stats cache
        self.update_stats_cache().await;
    }

    /// Record a database query metric
    pub async fn record_query_metric(&self, metric: QueryMetrics) {
        if metric.duration_ms > self.config.slow_query_threshold_ms {
            tracing::warn!(
                "Slow query detected: {} on {} took {}ms",
                metric.query_type,
                metric.table,
                metric.duration_ms
            );
        }

        let mut query_metrics = self.query_metrics.write().await;
        query_metrics.push(metric);

        // Keep only recent metrics
        let cutoff_time = Utc::now() - chrono::Duration::hours(self.config.metrics_retention_hours as i64);
        query_metrics.retain(|m| m.timestamp > cutoff_time);
    }

    /// Get performance statistics for an operation
    pub async fn get_stats(&self, operation: &str) -> Option<PerformanceStats> {
        let stats = self.stats_cache.read().await;
        stats.get(operation).cloned()
    }

    /// Get all performance statistics
    pub async fn get_all_stats(&self) -> HashMap<String, PerformanceStats> {
        let stats = self.stats_cache.read().await;
        stats.clone()
    }

    /// Get recent metrics for an operation
    pub async fn get_recent_metrics(&self, operation: &str, limit: usize) -> Vec<PermissionMetrics> {
        let metrics = self.metrics.read().await;
        metrics
            .iter()
            .filter(|m| m.operation == operation)
            .rev()
            .take(limit)
            .cloned()
            .collect()
    }

    /// Get database query statistics
    pub async fn get_query_stats(&self) -> HashMap<String, QueryPerformanceStats> {
        let query_metrics = self.query_metrics.read().await;
        let mut stats: HashMap<String, Vec<&QueryMetrics>> = HashMap::new();

        for metric in query_metrics.iter() {
            let key = format!("{}:{}", metric.query_type, metric.table);
            stats.entry(key).or_insert_with(Vec::new).push(metric);
        }

        let mut result = HashMap::new();
        for (key, metrics) in stats {
            if !metrics.is_empty() {
                let total_duration: u64 = metrics.iter().map(|m| m.duration_ms).sum();
                let successful = metrics.iter().filter(|m| m.success).count();
                let durations: Vec<u64> = metrics.iter().map(|m| m.duration_ms).collect();

                result.insert(key.clone(), QueryPerformanceStats {
                    query_type: key,
                    total_queries: metrics.len() as u64,
                    successful_queries: successful as u64,
                    average_duration_ms: total_duration as f64 / metrics.len() as f64,
                    min_duration_ms: *durations.iter().min().unwrap(),
                    max_duration_ms: *durations.iter().max().unwrap(),
                    p95_duration_ms: self.calculate_percentile(&durations, 95),
                    last_updated: Utc::now(),
                });
            }
        }

        result
    }

    /// Get system health metrics
    pub async fn get_system_health(&self) -> SystemHealthMetrics {
        let metrics = self.metrics.read().await;
        let recent_metrics: Vec<&PermissionMetrics> = metrics
            .iter()
            .filter(|m| m.timestamp > Utc::now() - chrono::Duration::minutes(5))
            .collect();

        let cache_hits = recent_metrics.iter().filter(|m| m.cache_hit).count();
        let cache_hit_rate = if recent_metrics.is_empty() {
            0.0
        } else {
            cache_hits as f64 / recent_metrics.len() as f64
        };

        let avg_response_time = if recent_metrics.is_empty() {
            0.0
        } else {
            recent_metrics.iter().map(|m| m.duration_ms).sum::<u64>() as f64 / recent_metrics.len() as f64
        };

        SystemHealthMetrics {
            cpu_usage_percent: self.get_cpu_usage(),
            memory_usage_bytes: self.get_memory_usage(),
            memory_usage_percent: self.get_memory_usage_percent(),
            active_connections: self.get_active_connections(),
            cache_memory_bytes: self.get_cache_memory_usage(),
            permission_cache_hit_rate: cache_hit_rate,
            average_response_time_ms: avg_response_time,
            timestamp: Utc::now(),
        }
    }

    /// Add alert callback for performance issues
    pub async fn add_alert_callback<F>(&self, callback: F)
    where
        F: Fn(&PermissionMetrics) + Send + Sync + 'static,
    {
        let mut callbacks = self.alert_callbacks.write().await;
        callbacks.push(Box::new(callback));
    }

    /// Update cached statistics
    async fn update_stats_cache(&self) {
        let metrics = self.metrics.read().await;
        let mut stats_cache = self.stats_cache.write().await;

        // Group metrics by operation
        let mut operation_metrics: HashMap<String, Vec<&PermissionMetrics>> = HashMap::new();
        for metric in metrics.iter() {
            operation_metrics
                .entry(metric.operation.clone())
                .or_insert_with(Vec::new)
                .push(metric);
        }

        // Calculate stats for each operation
        for (operation, op_metrics) in operation_metrics {
            if !op_metrics.is_empty() {
                let total_requests = op_metrics.len() as u64;
                let successful_requests = op_metrics.iter().filter(|m| m.success).count() as u64;
                let failed_requests = total_requests - successful_requests;

                let durations: Vec<u64> = op_metrics.iter().map(|m| m.duration_ms).collect();
                let total_duration: u64 = durations.iter().sum();
                let cache_hits = op_metrics.iter().filter(|m| m.cache_hit).count();

                let stats = PerformanceStats {
                    operation: operation.clone(),
                    total_requests,
                    successful_requests,
                    failed_requests,
                    average_duration_ms: total_duration as f64 / total_requests as f64,
                    min_duration_ms: *durations.iter().min().unwrap(),
                    max_duration_ms: *durations.iter().max().unwrap(),
                    p95_duration_ms: self.calculate_percentile(&durations, 95),
                    p99_duration_ms: self.calculate_percentile(&durations, 99),
                    cache_hit_rate: cache_hits as f64 / total_requests as f64,
                    requests_per_second: self.calculate_rps(&op_metrics),
                    last_updated: Utc::now(),
                };

                stats_cache.insert(operation, stats);
            }
        }
    }

    /// Trigger alerts for performance issues
    async fn trigger_alerts(&self, metric: &PermissionMetrics) {
        let callbacks = self.alert_callbacks.read().await;
        for callback in callbacks.iter() {
            callback(metric);
        }

        tracing::warn!(
            "Performance alert: {} operation took {}ms (threshold: {}ms)",
            metric.operation,
            metric.duration_ms,
            self.config.alert_threshold_ms
        );
    }

    /// Clean up old metrics
    async fn cleanup_old_metrics(&self, metrics: &mut Vec<PermissionMetrics>) {
        let cutoff_time = Utc::now() - chrono::Duration::hours(self.config.metrics_retention_hours as i64);
        metrics.retain(|m| m.timestamp > cutoff_time);
    }

    /// Calculate percentile from duration array
    fn calculate_percentile(&self, durations: &[u64], percentile: u8) -> u64 {
        if durations.is_empty() {
            return 0;
        }

        let mut sorted = durations.to_vec();
        sorted.sort_unstable();

        let index = (percentile as f64 / 100.0 * sorted.len() as f64) as usize;
        sorted.get(index.saturating_sub(1)).copied().unwrap_or(0)
    }

    /// Calculate requests per second
    fn calculate_rps(&self, metrics: &[&PermissionMetrics]) -> f64 {
        if metrics.is_empty() {
            return 0.0;
        }

        let time_window = Duration::from_secs(60); // 1 minute window
        let now = Utc::now();
        let recent_metrics: Vec<&PermissionMetrics> = metrics
            .iter()
            .filter(|m| now - m.timestamp < chrono::Duration::from_std(time_window).unwrap())
            .copied()
            .collect();

        recent_metrics.len() as f64 / 60.0
    }

    // System resource monitoring methods (simplified implementations)
    fn get_cpu_usage(&self) -> f64 {
        // In a real implementation, this would use system APIs
        // For now, return a placeholder
        0.0
    }

    fn get_memory_usage(&self) -> u64 {
        // In a real implementation, this would check actual memory usage
        0
    }

    fn get_memory_usage_percent(&self) -> f64 {
        // In a real implementation, this would calculate memory percentage
        0.0
    }

    fn get_active_connections(&self) -> u32 {
        // In a real implementation, this would check database connection pool
        0
    }

    fn get_cache_memory_usage(&self) -> u64 {
        // In a real implementation, this would check cache memory usage
        0
    }
}

/// Performance timing helper
pub struct PerformanceTimer {
    start: Instant,
    operation: String,
    monitor: Arc<PerformanceMonitor>,
    user_id: Option<String>,
    resource: Option<String>,
    action: Option<String>,
    metadata: HashMap<String, String>,
}

impl PerformanceTimer {
    pub fn new(operation: String, monitor: Arc<PerformanceMonitor>) -> Self {
        Self {
            start: Instant::now(),
            operation,
            monitor,
            user_id: None,
            resource: None,
            action: None,
            metadata: HashMap::new(),
        }
    }

    pub fn with_user_id(mut self, user_id: String) -> Self {
        self.user_id = Some(user_id);
        self
    }

    pub fn with_resource(mut self, resource: String) -> Self {
        self.resource = Some(resource);
        self
    }

    pub fn with_action(mut self, action: String) -> Self {
        self.action = Some(action);
        self
    }

    pub fn with_metadata(mut self, key: String, value: String) -> Self {
        self.metadata.insert(key, value);
        self
    }

    pub async fn finish(self, success: bool, cache_hit: bool) {
        let duration = self.start.elapsed();
        let metric = PermissionMetrics {
            operation: self.operation,
            duration_ms: duration.as_millis() as u64,
            timestamp: Utc::now(),
            success,
            user_id: self.user_id,
            resource: self.resource,
            action: self.action,
            cache_hit,
            metadata: self.metadata,
        };

        self.monitor.record_permission_metric(metric).await;
    }
}

/// Database query performance statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryPerformanceStats {
    pub query_type: String,
    pub total_queries: u64,
    pub successful_queries: u64,
    pub average_duration_ms: f64,
    pub min_duration_ms: u64,
    pub max_duration_ms: u64,
    pub p95_duration_ms: u64,
    pub last_updated: DateTime<Utc>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_performance_monitor_creation() {
        let monitor = PerformanceMonitor::new(None);
        let stats = monitor.get_all_stats().await;
        assert!(stats.is_empty());
    }

    #[tokio::test]
    async fn test_metric_recording() {
        let monitor = PerformanceMonitor::new(None);
        
        let metric = PermissionMetrics {
            operation: "permission_check".to_string(),
            duration_ms: 50,
            timestamp: Utc::now(),
            success: true,
            user_id: Some("user123".to_string()),
            resource: Some("posts".to_string()),
            action: Some("read".to_string()),
            cache_hit: true,
            metadata: HashMap::new(),
        };

        monitor.record_permission_metric(metric).await;
        
        let stats = monitor.get_stats("permission_check").await;
        assert!(stats.is_some());
        let stats = stats.unwrap();
        assert_eq!(stats.total_requests, 1);
        assert_eq!(stats.successful_requests, 1);
        assert!(stats.cache_hit_rate > 0.0);
    }

    #[tokio::test]
    async fn test_performance_timer() {
        let monitor = Arc::new(PerformanceMonitor::new(None));
        
        let timer = PerformanceTimer::new("test_operation".to_string(), monitor.clone())
            .with_user_id("user123".to_string())
            .with_resource("test_resource".to_string());

        // Simulate some work
        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
        
        timer.finish(true, false).await;
        
        let stats = monitor.get_stats("test_operation").await;
        assert!(stats.is_some());
        let stats = stats.unwrap();
        assert_eq!(stats.total_requests, 1);
        assert!(stats.average_duration_ms >= 10.0);
    }
}