// Performance alerting system with real-time monitoring and notifications

use std::sync::Arc;
use std::collections::HashMap;
use chrono::{DateTime, Utc, Duration};
use uuid::Uuid;
use serde::{Serialize, Deserialize};
use tokio::time::{interval, Duration as TokioDuration};
use tracing::{info, warn, error, debug};

use crate::web::performance::{
    models::*,
    repo::PerformanceRepo,
};

/// Alert system for monitoring performance thresholds
pub struct AlertSystem {
    repo: Arc<PerformanceRepo>,
    alert_state: Arc<tokio::sync::Mutex<AlertState>>,
}

#[derive(Debug, Default)]
struct AlertState {
    active_alerts: HashMap<Uuid, DateTime<Utc>>,
    cooldown_state: HashMap<Uuid, DateTime<Utc>>,
    alert_counts: HashMap<String, u32>,
}

impl AlertSystem {
    pub fn new(repo: Arc<PerformanceRepo>) -> Self {
        Self {
            repo,
            alert_state: Arc::new(tokio::sync::Mutex::new(AlertState::default())),
        }
    }

    /// Start the alert monitoring loop
    pub async fn start_monitoring(&self) {
        info!("Starting performance alert monitoring");
        
        let mut interval = interval(TokioDuration::from_secs(60)); // Check every minute
        
        loop {
            interval.tick().await;
            
            if let Err(e) = self.check_all_alerts().await {
                error!("Error checking alerts: {}", e);
            }
        }
    }

    /// Check all configured alerts
    async fn check_all_alerts(&self) -> Result<(), Box<dyn std::error::Error>> {
        let configs = self.repo.get_alert_configs().await?;
        
        for config in configs {
            if !config.enabled {
                continue;
            }

            // Check if alert is in cooldown
            {
                let state = self.alert_state.lock().await;
                if let Some(last_triggered) = state.cooldown_state.get(&config.id) {
                    let cooldown_until = *last_triggered + Duration::minutes(config.cooldown_minutes as i64);
                    if Utc::now() < cooldown_until {
                        debug!("Alert {} is in cooldown until {}", config.name, cooldown_until);
                        continue;
                    }
                }
            }

            match self.evaluate_alert(&config).await {
                Ok(Some(alert)) => {
                    self.trigger_alert(alert, &config).await?;
                }
                Ok(None) => {
                    // Check if we should resolve any existing alerts
                    self.check_alert_resolution(&config).await?;
                }
                Err(e) => {
                    warn!("Failed to evaluate alert {}: {}", config.name, e);
                }
            }
        }

        Ok(())
    }

    /// Evaluate a specific alert configuration
    async fn evaluate_alert(
        &self,
        config: &PerformanceAlertConfig,
    ) -> Result<Option<PerformanceAlert>, Box<dyn std::error::Error>> {
        let end_time = Utc::now();
        let start_time = end_time - Duration::minutes(config.time_window_minutes as i64);

        let metric_value = match config.metric_type {
            MetricType::Latency => {
                self.calculate_latency_metric(config.endpoint_pattern.as_deref(), start_time, end_time).await?
            }
            MetricType::ErrorRate => {
                self.calculate_error_rate_metric(config.endpoint_pattern.as_deref(), start_time, end_time).await?
            }
            MetricType::CacheHitRate => {
                self.calculate_cache_hit_rate_metric(start_time, end_time).await?
            }
            MetricType::Throughput => {
                self.calculate_throughput_metric(config.endpoint_pattern.as_deref(), start_time, end_time).await?
            }
            MetricType::ResourceUsage => {
                self.calculate_resource_usage_metric().await?
            }
        };

        // Evaluate threshold
        if config.threshold_operator.evaluate(metric_value, config.threshold_value) {
            let alert = PerformanceAlert {
                id: Uuid::new_v4(),
                alert_config_id: config.id,
                triggered_at: Utc::now(),
                resolved_at: None,
                metric_value,
                threshold_value: config.threshold_value,
                endpoint: config.endpoint_pattern.clone(),
                time_window_start: start_time,
                time_window_end: end_time,
                alert_message: self.generate_alert_message(config, metric_value),
                severity: config.severity.clone(),
                acknowledged: false,
                acknowledged_by: None,
                acknowledged_at: None,
                notes: None,
                metadata: Some(serde_json::json!({
                    "metric_type": config.metric_type,
                    "threshold_operator": config.threshold_operator,
                    "evaluation_time": end_time
                })),
                created_at: Utc::now(),
            };

            Ok(Some(alert))
        } else {
            Ok(None)
        }
    }

    /// Calculate latency metric (P95)
    async fn calculate_latency_metric(
        &self,
        endpoint_pattern: Option<&str>,
        start_time: DateTime<Utc>,
        end_time: DateTime<Utc>,
    ) -> Result<f64, Box<dyn std::error::Error>> {
        if let Some(pattern) = endpoint_pattern {
            let endpoints = self.repo.get_slowest_endpoints(start_time, end_time, 1).await?;
            if let Some(endpoint) = endpoints.into_iter().find(|e| e.endpoint.contains(pattern)) {
                Ok(endpoint.p95_duration)
            } else {
                Ok(0.0)
            }
        } else {
            let summary = self.repo.get_performance_summary(start_time, end_time).await?;
            Ok(summary.p95_response_time)
        }
    }

    /// Calculate error rate metric
    async fn calculate_error_rate_metric(
        &self,
        endpoint_pattern: Option<&str>,
        start_time: DateTime<Utc>,
        end_time: DateTime<Utc>,
    ) -> Result<f64, Box<dyn std::error::Error>> {
        if let Some(pattern) = endpoint_pattern {
            let endpoints = self.repo.get_slowest_endpoints(start_time, end_time, 50).await?;
            if let Some(endpoint) = endpoints.into_iter().find(|e| e.endpoint.contains(pattern)) {
                Ok(endpoint.error_rate)
            } else {
                Ok(0.0)
            }
        } else {
            let summary = self.repo.get_performance_summary(start_time, end_time).await?;
            Ok(summary.error_rate)
        }
    }

    /// Calculate cache hit rate metric
    async fn calculate_cache_hit_rate_metric(
        &self,
        start_time: DateTime<Utc>,
        end_time: DateTime<Utc>,
    ) -> Result<f64, Box<dyn std::error::Error>> {
        let cache_analytics = self.repo.get_cache_analytics(start_time, end_time).await?;
        Ok(cache_analytics.overall_hit_rate)
    }

    /// Calculate throughput metric
    async fn calculate_throughput_metric(
        &self,
        endpoint_pattern: Option<&str>,
        start_time: DateTime<Utc>,
        end_time: DateTime<Utc>,
    ) -> Result<f64, Box<dyn std::error::Error>> {
        let summary = self.repo.get_performance_summary(start_time, end_time).await?;
        Ok(summary.throughput_rps)
    }

    /// Calculate resource usage metric
    async fn calculate_resource_usage_metric(&self) -> Result<f64, Box<dyn std::error::Error>> {
        let system_health = self.repo.get_system_health().await?;
        Ok(system_health.cpu_usage.max(system_health.memory_usage))
    }

    /// Generate alert message
    fn generate_alert_message(&self, config: &PerformanceAlertConfig, metric_value: f64) -> String {
        let metric_name = match config.metric_type {
            MetricType::Latency => "latency",
            MetricType::ErrorRate => "error rate",
            MetricType::CacheHitRate => "cache hit rate",
            MetricType::Throughput => "throughput",
            MetricType::ResourceUsage => "resource usage",
        };

        let operator_text = match config.threshold_operator {
            ThresholdOperator::GreaterThan => "exceeded",
            ThresholdOperator::LessThan => "dropped below",
            ThresholdOperator::GreaterThanOrEqual => "reached or exceeded",
            ThresholdOperator::LessThanOrEqual => "reached or dropped below",
            ThresholdOperator::Equal => "equals",
        };

        let endpoint_text = config.endpoint_pattern
            .as_ref()
            .map(|p| format!(" for endpoint pattern '{}'", p))
            .unwrap_or_default();

        format!(
            "{} {} has {} threshold of {:.2}. Current value: {:.2}{}",
            config.severity.to_string().to_uppercase(),
            metric_name,
            operator_text,
            config.threshold_value,
            metric_value,
            endpoint_text
        )
    }

    /// Trigger an alert
    async fn trigger_alert(
        &self,
        alert: PerformanceAlert,
        config: &PerformanceAlertConfig,
    ) -> Result<(), Box<dyn std::error::Error>> {
        info!(
            "Triggering alert: {} - {}",
            config.name,
            alert.alert_message
        );

        // Store alert in database
        self.repo.create_alert(&alert).await?;

        // Update alert state
        {
            let mut state = self.alert_state.lock().await;
            state.active_alerts.insert(alert.id, alert.triggered_at);
            state.cooldown_state.insert(config.id, alert.triggered_at);
            
            let count_key = format!("{}:{}", config.name, alert.triggered_at.format("%Y-%m-%d"));
            *state.alert_counts.entry(count_key).or_insert(0) += 1;
        }

        // Send notifications
        self.send_notifications(&alert, config).await?;

        Ok(())
    }

    /// Send alert notifications
    async fn send_notifications(
        &self,
        alert: &PerformanceAlert,
        config: &PerformanceAlertConfig,
    ) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(channels) = &config.notification_channels {
            if let Ok(channels_array) = serde_json::from_value::<Vec<String>>(channels.clone()) {
                for channel in channels_array {
                    match channel.as_str() {
                        "email" => self.send_email_notification(alert, config).await?,
                        "slack" => self.send_slack_notification(alert, config).await?,
                        "webhook" => self.send_webhook_notification(alert, config).await?,
                        _ => warn!("Unknown notification channel: {}", channel),
                    }
                }
            }
        }

        Ok(())
    }

    /// Send email notification
    async fn send_email_notification(
        &self,
        alert: &PerformanceAlert,
        config: &PerformanceAlertConfig,
    ) -> Result<(), Box<dyn std::error::Error>> {
        info!("Sending email notification for alert: {}", config.name);
        
        // TODO: Implement email sending using the existing email service
        // For now, just log the notification
        info!(
            "EMAIL ALERT: {} - {} (Severity: {:?})",
            config.name,
            alert.alert_message,
            alert.severity
        );

        Ok(())
    }

    /// Send Slack notification
    async fn send_slack_notification(
        &self,
        alert: &PerformanceAlert,
        config: &PerformanceAlertConfig,
    ) -> Result<(), Box<dyn std::error::Error>> {
        info!("Sending Slack notification for alert: {}", config.name);
        
        let severity_emoji = match alert.severity {
            AlertSeverity::Critical => "🚨",
            AlertSeverity::High => "⚠️",
            AlertSeverity::Medium => "🟡",
            AlertSeverity::Low => "🔵",
        };

        let message = format!(
            "{} *PERFORMANCE ALERT*\n\
            *Alert:* {}\n\
            *Message:* {}\n\
            *Severity:* {:?}\n\
            *Triggered:* {}\n\
            *Metric Value:* {:.2}\n\
            *Threshold:* {:.2}",
            severity_emoji,
            config.name,
            alert.alert_message,
            alert.severity,
            alert.triggered_at.format("%Y-%m-%d %H:%M:%S UTC"),
            alert.metric_value,
            alert.threshold_value
        );

        // TODO: Implement actual Slack API call
        info!("SLACK ALERT: {}", message);

        Ok(())
    }

    /// Send webhook notification
    async fn send_webhook_notification(
        &self,
        alert: &PerformanceAlert,
        config: &PerformanceAlertConfig,
    ) -> Result<(), Box<dyn std::error::Error>> {
        info!("Sending webhook notification for alert: {}", config.name);
        
        let payload = serde_json::json!({
            "alert_id": alert.id,
            "alert_name": config.name,
            "message": alert.alert_message,
            "severity": alert.severity,
            "metric_value": alert.metric_value,
            "threshold_value": alert.threshold_value,
            "triggered_at": alert.triggered_at,
            "endpoint": alert.endpoint,
            "metadata": alert.metadata
        });

        // TODO: Implement actual webhook HTTP POST
        info!("WEBHOOK ALERT: {}", payload);

        Ok(())
    }

    /// Check if alerts should be resolved
    async fn check_alert_resolution(
        &self,
        config: &PerformanceAlertConfig,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let active_alerts = self.repo.get_active_alerts().await?;
        
        for alert in active_alerts {
            if alert.alert_config_id != config.id {
                continue;
            }

            // Re-evaluate the metric
            let current_metric = match config.metric_type {
                MetricType::Latency => {
                    let end_time = Utc::now();
                    let start_time = end_time - Duration::minutes(config.time_window_minutes as i64);
                    self.calculate_latency_metric(config.endpoint_pattern.as_deref(), start_time, end_time).await?
                }
                MetricType::ErrorRate => {
                    let end_time = Utc::now();
                    let start_time = end_time - Duration::minutes(config.time_window_minutes as i64);
                    self.calculate_error_rate_metric(config.endpoint_pattern.as_deref(), start_time, end_time).await?
                }
                MetricType::CacheHitRate => {
                    let end_time = Utc::now();
                    let start_time = end_time - Duration::minutes(config.time_window_minutes as i64);
                    self.calculate_cache_hit_rate_metric(start_time, end_time).await?
                }
                MetricType::Throughput => {
                    let end_time = Utc::now();
                    let start_time = end_time - Duration::minutes(config.time_window_minutes as i64);
                    self.calculate_throughput_metric(config.endpoint_pattern.as_deref(), start_time, end_time).await?
                }
                MetricType::ResourceUsage => {
                    self.calculate_resource_usage_metric().await?
                }
            };

            // Check if condition is no longer met
            if !config.threshold_operator.evaluate(current_metric, config.threshold_value) {
                self.resolve_alert(alert.id).await?;
            }
        }

        Ok(())
    }

    /// Resolve an alert
    async fn resolve_alert(&self, alert_id: Uuid) -> Result<(), Box<dyn std::error::Error>> {
        info!("Resolving alert: {}", alert_id);
        
        self.repo.resolve_alert(alert_id).await?;
        
        // Update state
        {
            let mut state = self.alert_state.lock().await;
            state.active_alerts.remove(&alert_id);
        }

        Ok(())
    }

    /// Get alert dashboard data
    pub async fn get_alert_dashboard(&self) -> Result<AlertDashboard, Box<dyn std::error::Error>> {
        let active_alerts = self.repo.get_active_alerts().await?;
        
        let mut alerts_by_severity = HashMap::new();
        for alert in &active_alerts {
            *alerts_by_severity.entry(alert.severity.clone()).or_insert(0) += 1;
        }

        // Get recent resolutions (last 24 hours)
        let recent_resolutions = Vec::new(); // TODO: Implement in repo

        // Get alert trends (last 7 days)
        let alert_trends = Vec::new(); // TODO: Implement trend analysis

        Ok(AlertDashboard {
            active_alerts,
            alerts_by_severity,
            recent_resolutions,
            alert_trends,
        })
    }

    /// Acknowledge an alert
    pub async fn acknowledge_alert(
        &self,
        alert_id: Uuid,
        acknowledged_by: Uuid,
        notes: Option<String>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        info!("Acknowledging alert: {} by user: {}", alert_id, acknowledged_by);
        
        self.repo.acknowledge_alert(alert_id, acknowledged_by, notes).await?;
        
        Ok(())
    }

    /// Create new alert configuration
    pub async fn create_alert_config(
        &self,
        config: CreateAlertConfigRequest,
    ) -> Result<Uuid, Box<dyn std::error::Error>> {
        info!("Creating new alert configuration: {}", config.name);
        
        // Validate configuration
        self.validate_alert_config(&config)?;
        
        let config_id = self.repo.create_alert_config(&config).await?;
        
        info!("Alert configuration created with ID: {}", config_id);
        
        Ok(config_id)
    }

    /// Validate alert configuration
    fn validate_alert_config(&self, config: &CreateAlertConfigRequest) -> Result<(), Box<dyn std::error::Error>> {
        if config.name.trim().is_empty() {
            return Err("Alert name cannot be empty".into());
        }

        if config.threshold_value < 0.0 {
            return Err("Threshold value cannot be negative".into());
        }

        if config.time_window_minutes <= 0 {
            return Err("Time window must be positive".into());
        }

        if config.cooldown_minutes < 0 {
            return Err("Cooldown cannot be negative".into());
        }

        // Validate metric type specific constraints
        match config.metric_type {
            MetricType::ErrorRate | MetricType::CacheHitRate => {
                if config.threshold_value > 100.0 {
                    return Err("Percentage values cannot exceed 100".into());
                }
            }
            MetricType::Latency => {
                if config.threshold_value > 60000.0 {
                    return Err("Latency threshold seems too high (>60s)".into());
                }
            }
            _ => {}
        }

        Ok(())
    }

    /// Get alert statistics
    pub async fn get_alert_statistics(
        &self,
        days: i32,
    ) -> Result<AlertStatistics, Box<dyn std::error::Error>> {
        let active_count = self.repo.count_active_alerts().await?;
        
        // TODO: Implement more comprehensive statistics from database
        Ok(AlertStatistics {
            active_alerts: active_count,
            total_alerts_last_24h: 0,
            resolved_alerts_last_24h: 0,
            avg_resolution_time_minutes: 0.0,
            most_frequent_alert_type: "latency".to_string(),
            alert_frequency_by_severity: HashMap::new(),
        })
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AlertStatistics {
    pub active_alerts: i64,
    pub total_alerts_last_24h: i32,
    pub resolved_alerts_last_24h: i32,
    pub avg_resolution_time_minutes: f64,
    pub most_frequent_alert_type: String,
    pub alert_frequency_by_severity: HashMap<AlertSeverity, i32>,
}

impl ToString for AlertSeverity {
    fn to_string(&self) -> String {
        match self {
            AlertSeverity::Low => "low".to_string(),
            AlertSeverity::Medium => "medium".to_string(),
            AlertSeverity::High => "high".to_string(),
            AlertSeverity::Critical => "critical".to_string(),
        }
    }
}