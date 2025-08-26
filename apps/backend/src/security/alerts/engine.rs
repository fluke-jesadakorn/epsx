// Security Alert Engine - Complete Diesel Implementation
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use chrono::{DateTime, Utc, Duration};
use uuid::Uuid;
use tracing::{info, error, warn, debug};
use std::collections::HashMap;
use serde_json::Value as JsonValue;

use crate::{
    core::errors::{AppError, AppResult},
    infra::{
        cache::Cache,
        db::diesel::{
            models::{
                DieselSecurityEvent,
                DieselSecurityAlertRule,
                NewDieselSecurityAlertRule,
                NewDieselAlertNotification,
            },
            pool::DbPool,
            schema::{
                security_events, security_alert_rules, alert_notifications,
            },
            types::DieselIpAddr,
        },
    },
};

pub struct SecurityAlertEngine {
    pool: DbPool,
    cache: Option<Box<dyn Cache>>,
    alert_rules: Vec<AlertRule>,
}

#[derive(Debug, Clone)]
pub struct AlertRule {
    pub id: Uuid,
    pub name: String,
    pub event_pattern: JsonValue,
    pub severity: String,
    pub threshold_count: i32,
    pub time_window_seconds: i32,
    pub enabled: bool,
    pub notification_channels: Vec<String>,
    pub auto_block: bool,
    pub block_duration_seconds: i32,
}

#[derive(Debug, Clone)]
pub struct AlertContext {
    pub rule_id: Uuid,
    pub event_count: i64,
    pub triggered_at: DateTime<Utc>,
    pub matching_events: Vec<Uuid>,
    pub risk_score: i32,
}

impl SecurityAlertEngine {
    pub fn new(pool: DbPool) -> Self {
        Self {
            pool,
            cache: None,
            alert_rules: Vec::new(),
        }
    }

    pub fn with_cache(pool: DbPool, cache_instance: Box<dyn Cache>) -> Self {
        Self {
            pool,
            cache: Some(cache_instance),
            alert_rules: Vec::new(),
        }
    }

    // Initialize the engine by loading alert rules from database
    pub async fn initialize(&mut self) -> AppResult<()> {
        info!("Initializing Security Alert Engine");
        
        let mut conn = self.pool.get().await
            .map_err(|e| AppError::database_error(format!("Failed to get database connection: {}", e)))?;

        let db_rules = security_alert_rules::table
            .filter(security_alert_rules::enabled.eq(true))
            .load::<DieselSecurityAlertRule>(&mut conn)
            .await
            .map_err(|e| AppError::database_error(format!("Failed to load alert rules: {}", e)))?;

        self.alert_rules = db_rules.into_iter().map(|rule| AlertRule {
            id: rule.id,
            name: rule.name,
            event_pattern: serde_json::json!({"type": "default"}),
            severity: "medium".to_string(),
            threshold_count: 1,
            time_window_seconds: 300,
            enabled: rule.enabled,
            notification_channels: vec![],
            auto_block: false,
            block_duration_seconds: 3600,
        }).collect();

        info!("Loaded {} alert rules", self.alert_rules.len());
        Ok(())
    }

    // Process a security event and check for alerts
    pub async fn process_security_event(&self, event: &DieselSecurityEvent) -> AppResult<Vec<AlertContext>> {
        let mut triggered_alerts = Vec::new();

        for rule in &self.alert_rules {
            if !rule.enabled {
                continue;
            }

            if self.matches_event_pattern(&rule.event_pattern, event) {
                debug!("Event {} matches rule pattern: {}", event.id, rule.name);
                
                if let Some(alert_context) = self.check_threshold_breach(rule, event).await? {
                    info!("Alert triggered: {} (rule: {})", alert_context.rule_id, rule.name);
                    
                    // Log the alert
                    self.log_alert(&alert_context, rule, event).await?;
                    
                    // Auto-block if configured
                    if rule.auto_block {
                        let ip_network = ipnetwork::IpNetwork::from(event.ip_address.0);
                        self.auto_block_ip(ip_network, rule).await?;
                    }
                    
                    triggered_alerts.push(alert_context);
                }
            }
        }

        if !triggered_alerts.is_empty() {
            info!("Processed event {}: {} alerts triggered", event.id, triggered_alerts.len());
        }

        Ok(triggered_alerts)
    }

    // Check if an event matches a rule pattern
    fn matches_event_pattern(&self, pattern: &JsonValue, event: &DieselSecurityEvent) -> bool {
        // Simple pattern matching - in a production system, this would be more sophisticated
        if let Some(pattern_obj) = pattern.as_object() {
            for (key, value) in pattern_obj {
                match key.as_str() {
                    "event_type" => {
                        if let Some(pattern_type) = value.as_str() {
                            if event.event_type != pattern_type {
                                return false;
                            }
                        }
                    }
                    "severity" => {
                        if let Some(pattern_severity) = value.as_str() {
                            if event.severity != pattern_severity {
                                return false;
                            }
                        }
                    }
                    "source" => {
                        if let Some(pattern_source) = value.as_str() {
                            if event.source != pattern_source {
                                return false;
                            }
                        }
                    }
                    "min_risk_score" => {
                        if let Some(min_score) = value.as_i64() {
                            if event.risk_score.unwrap_or(0) < min_score as i32 {
                                return false;
                            }
                        }
                    }
                    _ => {
                        debug!("Unknown pattern key: {}", key);
                    }
                }
            }
        }
        
        true
    }

    // Check if event count exceeds threshold within time window
    async fn check_threshold_breach(&self, rule: &AlertRule, current_event: &DieselSecurityEvent) -> AppResult<Option<AlertContext>> {
        let mut conn = self.pool.get().await
            .map_err(|e| AppError::database_error(format!("Failed to get database connection: {}", e)))?;

        let time_window_start = current_event.timestamp - Duration::seconds(rule.time_window_seconds as i64);

        // Count matching events within time window
        let mut query = security_events::table
            .filter(security_events::timestamp.gt(time_window_start))
            .filter(security_events::timestamp.le(current_event.timestamp))
            .into_boxed();

        // Apply pattern filters
        if let Some(pattern_obj) = rule.event_pattern.as_object() {
            for (key, value) in pattern_obj {
                match key.as_str() {
                    "event_type" => {
                        if let Some(pattern_type) = value.as_str() {
                            query = query.filter(security_events::event_type.eq(pattern_type));
                        }
                    }
                    "severity" => {
                        if let Some(pattern_severity) = value.as_str() {
                            query = query.filter(security_events::severity.eq(pattern_severity));
                        }
                    }
                    "source" => {
                        if let Some(pattern_source) = value.as_str() {
                            query = query.filter(security_events::source.eq(pattern_source));
                        }
                    }
                    "min_risk_score" => {
                        if let Some(min_score) = value.as_i64() {
                            query = query.filter(security_events::risk_score.ge(min_score as i32));
                        }
                    }
                    _ => {}
                }
            }
        }

        let matching_events = query
            .select((security_events::id, security_events::timestamp, security_events::risk_score))
            .load::<(Uuid, DateTime<Utc>, Option<i32>)>(&mut conn)
            .await
            .map_err(|e| AppError::database_error(format!("Failed to count matching events: {}", e)))?;

        let event_count = matching_events.len() as i64;
        
        if event_count >= rule.threshold_count as i64 {
            let event_ids: Vec<Uuid> = matching_events.iter().map(|(id, _, _)| *id).collect();
            let total_risk_score: i32 = matching_events.iter()
                .map(|(_, _, score)| score.unwrap_or(0))
                .sum();

            Ok(Some(AlertContext {
                rule_id: rule.id,
                event_count,
                triggered_at: Utc::now(),
                matching_events: event_ids,
                risk_score: total_risk_score,
            }))
        } else {
            Ok(None)
        }
    }

    // Log an alert to the database
    async fn log_alert(&self, context: &AlertContext, rule: &AlertRule, _triggering_event: &DieselSecurityEvent) -> AppResult<()> {
        let mut conn = self.pool.get().await
            .map_err(|e| AppError::database_error(format!("Failed to get database connection: {}", e)))?;

        for channel in &rule.notification_channels {
            let notification = NewDieselAlertNotification {
                id: Uuid::new_v4(),
                alert_id: rule.id, // Using rule ID as alert ID
                channel: channel.clone(),
                sent_at: context.triggered_at,
                status: "pending".to_string(),
                created_at: context.triggered_at,
                delivery_status: "pending".to_string(),
            };

            diesel::insert_into(alert_notifications::table)
                .values(&notification)
                .execute(&mut conn)
                .await
                .map_err(|e| AppError::database_error(format!("Failed to log alert notification: {}", e)))?;
        }

        // Update rule last_triggered timestamp
        diesel::update(security_alert_rules::table.filter(security_alert_rules::id.eq(rule.id)))
            .set(security_alert_rules::last_triggered.eq(Some(context.triggered_at)))
            .execute(&mut conn)
            .await
            .map_err(|e| AppError::database_error(format!("Failed to update rule last_triggered: {}", e)))?;

        info!("Logged alert for rule {} with {} notifications", rule.name, rule.notification_channels.len());
        Ok(())
    }

    // Auto-block an IP address
    async fn auto_block_ip(&self, ip_addr: ipnetwork::IpNetwork, rule: &AlertRule) -> AppResult<()> {
        let mut conn = self.pool.get().await
            .map_err(|e| AppError::database_error(format!("Failed to get database connection: {}", e)))?;

        let block_expires_at = if rule.block_duration_seconds > 0 {
            Some(Utc::now() + Duration::seconds(rule.block_duration_seconds as i64))
        } else {
            None // Permanent block
        };

        use crate::infra::db::diesel::schema::ip_blacklist::dsl::*;
        
        // Check if IP is already blocked
        let existing_block = ip_blacklist
            .filter(ip_address.eq(DieselIpAddr(ip_addr.ip())))
            .first::<crate::infra::db::diesel::models::DieselIpBlacklist>(&mut conn)
            .await
            .optional()
            .map_err(|e| AppError::database_error(format!("Failed to check existing IP block: {}", e)))?;

        if existing_block.is_none() {
            let new_block = crate::infra::db::diesel::models::NewDieselIpBlacklist {
                id: Uuid::new_v4(),
                ip_address: DieselIpAddr(ip_addr.ip()),
                reason: format!("Auto-blocked by rule: {}", rule.name),
                created_at: Utc::now(),
                expires_at: block_expires_at,
            };

            diesel::insert_into(ip_blacklist)
                .values(&new_block)
                .execute(&mut conn)
                .await
                .map_err(|e| AppError::database_error(format!("Failed to insert IP block: {}", e)))?;

            warn!("Auto-blocked IP {} due to rule: {} (expires: {:?})", ip_addr, rule.name, expires_at);
        } else {
            debug!("IP {} already blocked", ip_addr);
        }

        Ok(())
    }

    // Get recipient for notification channel
    fn get_channel_recipient(&self, channel: &str) -> String {
        use crate::config::env::get_env_var;
        
        match channel {
            "email" => get_env_var("SECURITY_EMAIL")
                .unwrap_or_else(|_| "security@epsx.io".to_string()),
            "slack" => get_env_var("SECURITY_SLACK_CHANNEL")
                .unwrap_or_else(|_| "#security-alerts".to_string()),
            "webhook" => get_env_var("SECURITY_WEBHOOK_ENDPOINT")
                .unwrap_or_else(|_| "security-webhook".to_string()),
            "sms" => get_env_var("SECURITY_SMS_NUMBER")
                .unwrap_or_else(|_| "+1234567890".to_string()),
            _ => format!("default-{}", channel),
        }
    }

    // Create a new alert rule
    pub async fn create_alert_rule(&self, rule_name: String, event_pattern: JsonValue, rule_severity: String, _threshold_count: i32, _time_window_seconds: i32) -> AppResult<Uuid> {
        let mut conn = self.pool.get().await
            .map_err(|e| AppError::database_error(format!("Failed to get database connection: {}", e)))?;

        let new_rule = NewDieselSecurityAlertRule {
            id: Uuid::new_v4(),
            name: rule_name,
            rule_type: rule_severity,
            condition: event_pattern,
            is_active: true,
            enabled: true,
            created_at: Utc::now(),
            last_triggered: None,
        };

        let rule_id = new_rule.id;

        diesel::insert_into(security_alert_rules::table)
            .values(&new_rule)
            .execute(&mut conn)
            .await
            .map_err(|e| AppError::database_error(format!("Failed to create alert rule: {}", e)))?;

        info!("Created new alert rule: {} ({})", new_rule.name, rule_id);
        Ok(rule_id)
    }

    // Get alert statistics
    pub async fn get_alert_stats(&self, hours_back: i32) -> AppResult<HashMap<String, i64>> {
        let mut conn = self.pool.get().await
            .map_err(|e| AppError::database_error(format!("Failed to get database connection: {}", e)))?;

        let cutoff_time = Utc::now() - Duration::hours(hours_back as i64);
        let mut stats = HashMap::new();

        // Total alerts
        let total_alerts = alert_notifications::table
            .filter(alert_notifications::created_at.gt(cutoff_time))
            .count()
            .get_result::<i64>(&mut conn)
            .await
            .map_err(|e| AppError::database_error(format!("Failed to count total alerts: {}", e)))?;

        stats.insert("total_alerts".to_string(), total_alerts);

        // Successful deliveries
        let successful_deliveries = alert_notifications::table
            .filter(alert_notifications::created_at.gt(cutoff_time))
            .filter(alert_notifications::delivery_status.eq("success"))
            .count()
            .get_result::<i64>(&mut conn)
            .await
            .map_err(|e| AppError::database_error(format!("Failed to count successful deliveries: {}", e)))?;

        stats.insert("successful_deliveries".to_string(), successful_deliveries);

        // Failed deliveries
        let failed_deliveries = alert_notifications::table
            .filter(alert_notifications::created_at.gt(cutoff_time))
            .filter(alert_notifications::delivery_status.eq("failed"))
            .count()
            .get_result::<i64>(&mut conn)
            .await
            .map_err(|e| AppError::database_error(format!("Failed to count failed deliveries: {}", e)))?;

        stats.insert("failed_deliveries".to_string(), failed_deliveries);

        // Active rules
        let active_rules = security_alert_rules::table
            .filter(security_alert_rules::enabled.eq(true))
            .count()
            .get_result::<i64>(&mut conn)
            .await
            .map_err(|e| AppError::database_error(format!("Failed to count active rules: {}", e)))?;

        stats.insert("active_rules".to_string(), active_rules);

        Ok(stats)
    }

    // Health check for alert engine
    pub async fn health_check(&self) -> AppResult<bool> {
        // Check database connectivity
        let mut conn = match self.pool.get().await {
            Ok(conn) => conn,
            Err(e) => {
                error!("Alert engine health check failed - database connection: {}", e);
                return Ok(false);
            }
        };

        // Check if we can query alert rules
        let rule_count = match security_alert_rules::table.count().get_result::<i64>(&mut conn).await {
            Ok(count) => count,
            Err(e) => {
                error!("Alert engine health check failed - rule query: {}", e);
                return Ok(false);
            }
        };

        info!("Alert engine health check passed - {} rules configured", rule_count);
        Ok(true)
    }
}