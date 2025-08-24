// Security Alert System Configuration
// Production-ready configuration examples and templates

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// Production Alert Configuration Examples
pub struct ProductionAlertConfig;

impl ProductionAlertConfig {
    /// Get default production alert rules
    pub fn default_alert_rules() -> Vec<AlertRuleTemplate> {
        vec![
            AlertRuleTemplate {
                name: "Critical Brute Force Attack".to_string(),
                description: "Detect multiple failed login attempts indicating brute force attack".to_string(),
                category: "BRUTE_FORCE".to_string(),
                severity: "CRITICAL".to_string(),
                conditions: serde_json::json!({
                    "event_type": "MULTIPLE_FAILED_LOGINS",
                    "time_window": 300,
                    "threshold": 10,
                    "operator": ">="
                }),
                aggregation_window_minutes: 5,
                cooldown_minutes: 30,
                max_alerts_per_hour: 6,
                tags: vec!["authentication".to_string(), "brute-force".to_string()],
                enabled: true,
            },
            AlertRuleTemplate {
                name: "Privilege Escalation Attempt".to_string(),
                description: "Alert on any attempt to escalate privileges".to_string(),
                category: "PRIVILEGE_ESCALATION".to_string(),
                severity: "CRITICAL".to_string(),
                conditions: serde_json::json!({
                    "event_type": "PRIVILEGE_ESCALATION_ATTEMPT"
                }),
                aggregation_window_minutes: 1,
                cooldown_minutes: 5,
                max_alerts_per_hour: 12,
                tags: vec!["authorization".to_string(), "escalation".to_string()],
                enabled: true,
            },
            AlertRuleTemplate {
                name: "Geographic Anomaly Detection".to_string(),
                description: "Alert on logins from unusual geographic locations".to_string(),
                category: "ANOMALY".to_string(),
                severity: "HIGH".to_string(),
                conditions: serde_json::json!({
                    "event_type": "GEOGRAPHIC_ANOMALY_DETECTED"
                }),
                aggregation_window_minutes: 10,
                cooldown_minutes: 60,
                max_alerts_per_hour: 4,
                tags: vec!["anomaly".to_string(), "geographic".to_string()],
                enabled: true,
            },
            AlertRuleTemplate {
                name: "Mass Account Lockout".to_string(),
                description: "Alert when multiple accounts are locked in short time period".to_string(),
                category: "AUTHENTICATION".to_string(),
                severity: "HIGH".to_string(),
                conditions: serde_json::json!({
                    "event_type": "ACCOUNT_LOCKED",
                    "time_window": 600,
                    "threshold": 5,
                    "operator": ">="
                }),
                aggregation_window_minutes: 10,
                cooldown_minutes: 30,
                max_alerts_per_hour: 8,
                tags: vec!["authentication".to_string(), "mass-lockout".to_string()],
                enabled: true,
            },
            AlertRuleTemplate {
                name: "SQL Injection Attempt".to_string(),
                description: "Alert on potential SQL injection attempts".to_string(),
                category: "INJECTION".to_string(),
                severity: "CRITICAL".to_string(),
                conditions: serde_json::json!({
                    "event_type": "SQL_INJECTION_ATTEMPT"
                }),
                aggregation_window_minutes: 1,
                cooldown_minutes: 10,
                max_alerts_per_hour: 20,
                tags: vec!["injection".to_string(), "sql".to_string()],
                enabled: true,
            },
            AlertRuleTemplate {
                name: "Bulk Data Access".to_string(),
                description: "Alert on unusual bulk data access patterns".to_string(),
                category: "DATA_BREACH".to_string(),
                severity: "HIGH".to_string(),
                conditions: serde_json::json!({
                    "event_type": "BULK_DATA_ACCESS",
                    "time_window": 300,
                    "threshold": 100,
                    "operator": ">="
                }),
                aggregation_window_minutes: 5,
                cooldown_minutes: 20,
                max_alerts_per_hour: 10,
                tags: vec!["data-breach".to_string(), "bulk-access".to_string()],
                enabled: true,
            },
            AlertRuleTemplate {
                name: "API Rate Limit Violations".to_string(),
                description: "Alert on excessive API rate limit violations".to_string(),
                category: "ANOMALY".to_string(),
                severity: "MEDIUM".to_string(),
                conditions: serde_json::json!({
                    "event_type": "RATE_LIMIT_EXCEEDED",
                    "time_window": 300,
                    "threshold": 20,
                    "operator": ">="
                }),
                aggregation_window_minutes: 5,
                cooldown_minutes: 15,
                max_alerts_per_hour: 12,
                tags: vec!["rate-limiting".to_string(), "api".to_string()],
                enabled: true,
            },
        ]
    }

    /// Get default webhook configurations
    pub fn default_webhook_configs() -> Vec<WebhookConfigTemplate> {
        vec![
            WebhookConfigTemplate {
                name: "Critical Alerts Slack".to_string(),
                description: "Send critical alerts to security team Slack channel".to_string(),
                url: "https://hooks.slack.com/services/YOUR/SLACK/WEBHOOK".to_string(),
                auth_type: "NONE".to_string(),
                auth_config: serde_json::json!({}),
                custom_headers: HashMap::from([
                    ("Content-Type".to_string(), "application/json".to_string()),
                ]),
                payload_template: Some(serde_json::json!({
                    "text": "🚨 Critical Security Alert",
                    "blocks": [
                        {
                            "type": "section",
                            "text": {
                                "type": "mrkdwn",
                                "text": "*{{alert.title}}*\n{{alert.description}}"
                            }
                        },
                        {
                            "type": "section",
                            "fields": [
                                {
                                    "type": "mrkdwn",
                                    "text": "*Severity:*\n{{alert.severity}}"
                                },
                                {
                                    "type": "mrkdwn",
                                    "text": "*Source IP:*\n{{alert.source_ip}}"
                                }
                            ]
                        }
                    ]
                })),
                severity_filter: vec!["CRITICAL".to_string()],
                category_filter: vec![],
                retry_attempts: 3,
                timeout_seconds: 30,
            },
            WebhookConfigTemplate {
                name: "PagerDuty Critical Incidents".to_string(),
                description: "Create PagerDuty incidents for critical alerts".to_string(),
                url: "https://events.pagerduty.com/v2/enqueue".to_string(),
                auth_type: "CUSTOM".to_string(),
                auth_config: serde_json::json!({
                    "headers": {
                        "Authorization": "Token YOUR_PAGERDUTY_API_KEY"
                    }
                }),
                custom_headers: HashMap::from([
                    ("Content-Type".to_string(), "application/json".to_string()),
                ]),
                payload_template: Some(serde_json::json!({
                    "routing_key": "YOUR_PAGERDUTY_ROUTING_KEY",
                    "event_action": "trigger",
                    "payload": {
                        "summary": "{{alert.title}}",
                        "source": "EPSX Security System",
                        "severity": "critical",
                        "component": "security-alerts",
                        "group": "{{alert.category}}",
                        "class": "security",
                        "custom_details": {
                            "alert_id": "{{alert.id}}",
                            "severity": "{{alert.severity}}",
                            "source_ip": "{{alert.source_ip}}",
                            "user_id": "{{alert.user_id}}",
                            "triggered_at": "{{alert.triggered_at}}"
                        }
                    }
                })),
                severity_filter: vec!["CRITICAL".to_string()],
                category_filter: vec![],
                retry_attempts: 5,
                timeout_seconds: 45,
            },
            WebhookConfigTemplate {
                name: "Security Dashboard Updates".to_string(),
                description: "Real-time updates to security dashboard".to_string(),
                url: "https://your-dashboard.com/api/security/alerts".to_string(),
                auth_type: "HMAC".to_string(),
                auth_config: serde_json::json!({
                    "secret": "your-hmac-secret-key-min-64-chars",
                    "algorithm": "sha256"
                }),
                custom_headers: HashMap::from([
                    ("Content-Type".to_string(), "application/json".to_string()),
                    ("X-Source".to_string(), "EPSX-Security-System".to_string()),
                ]),
                payload_template: None, // Use default payload
                severity_filter: vec!["HIGH".to_string(), "CRITICAL".to_string()],
                category_filter: vec![],
                retry_attempts: 3,
                timeout_seconds: 30,
            },
        ]
    }

    /// Get default notification channel configurations
    pub fn default_notification_channels() -> Vec<NotificationChannelTemplate> {
        vec![
            NotificationChannelTemplate {
                name: "Security Team Email".to_string(),
                description: "Email notifications for security team".to_string(),
                channel_type: "EMAIL".to_string(),
                config: serde_json::json!({
                    "recipients": ["security@yourcompany.com", "soc@yourcompany.com"],
                    "smtp_config": {
                        "host": "smtp.yourcompany.com",
                        "port": 587,
                        "username": "alerts@yourcompany.com",
                        "password": "your-smtp-password",
                        "tls": true
                    },
                    "from_email": "security-alerts@yourcompany.com",
                    "from_name": "EPSX Security System"
                }),
                severity_filter: vec!["HIGH".to_string(), "CRITICAL".to_string()],
                category_filter: vec![],
                time_restrictions: Some(serde_json::json!({
                    "business_hours_only": false,
                    "timezone": "UTC"
                })),
            },
            NotificationChannelTemplate {
                name: "Microsoft Teams Security Channel".to_string(),
                description: "Teams notifications for security alerts".to_string(),
                channel_type: "TEAMS".to_string(),
                config: serde_json::json!({
                    "webhook_url": "https://yourcompany.webhook.office.com/webhookb2/YOUR-WEBHOOK-URL",
                    "message_format": "adaptive_card"
                }),
                severity_filter: vec!["MEDIUM".to_string(), "HIGH".to_string(), "CRITICAL".to_string()],
                category_filter: vec![],
                time_restrictions: Some(serde_json::json!({
                    "business_hours_only": true,
                    "timezone": "America/New_York",
                    "excluded_hours": [22, 23, 0, 1, 2, 3, 4, 5, 6, 7]
                })),
            },
        ]
    }

    /// Get default escalation rules
    pub fn default_escalation_rules() -> Vec<EscalationRuleTemplate> {
        vec![
            EscalationRuleTemplate {
                name: "Critical Alert Escalation".to_string(),
                description: "Escalate unacknowledged critical alerts after 5 minutes".to_string(),
                severity: "CRITICAL".to_string(),
                category: None,
                unacknowledged_minutes: 5,
                escalate_to_severity: Some("CRITICAL".to_string()),
                notify_channels: vec![], // Will be filled with actual channel IDs
                assign_to: Some("security-lead".to_string()),
                is_active: true,
            },
            EscalationRuleTemplate {
                name: "High Alert Escalation".to_string(),
                description: "Escalate unacknowledged high alerts after 15 minutes".to_string(),
                severity: "HIGH".to_string(),
                category: None,
                unacknowledged_minutes: 15,
                escalate_to_severity: Some("CRITICAL".to_string()),
                notify_channels: vec![],
                assign_to: None,
                is_active: true,
            },
        ]
    }

    /// Get environment-specific configuration
    pub fn get_environment_config(env: &str) -> EnvironmentConfig {
        match env.to_lowercase().as_str() {
            "production" => EnvironmentConfig {
                alert_retention_days: 365,
                webhook_timeout_seconds: 45,
                max_concurrent_alerts: 1000,
                enable_rate_limiting: true,
                enable_correlation: true,
                enable_ml_detection: true,
                log_level: "INFO".to_string(),
                metrics_enabled: true,
                health_check_interval_minutes: 5,
                cache_ttl_minutes: 60,
            },
            "staging" => EnvironmentConfig {
                alert_retention_days: 90,
                webhook_timeout_seconds: 30,
                max_concurrent_alerts: 500,
                enable_rate_limiting: true,
                enable_correlation: true,
                enable_ml_detection: false,
                log_level: "DEBUG".to_string(),
                metrics_enabled: true,
                health_check_interval_minutes: 10,
                cache_ttl_minutes: 30,
            },
            "development" => EnvironmentConfig {
                alert_retention_days: 30,
                webhook_timeout_seconds: 15,
                max_concurrent_alerts: 100,
                enable_rate_limiting: false,
                enable_correlation: false,
                enable_ml_detection: false,
                log_level: "DEBUG".to_string(),
                metrics_enabled: false,
                health_check_interval_minutes: 30,
                cache_ttl_minutes: 10,
            },
            _ => Self::get_environment_config("development"),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AlertRuleTemplate {
    pub name: String,
    pub description: String,
    pub category: String,
    pub severity: String,
    pub conditions: serde_json::Value,
    pub aggregation_window_minutes: i32,
    pub cooldown_minutes: i32,
    pub max_alerts_per_hour: i32,
    pub tags: Vec<String>,
    pub enabled: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WebhookConfigTemplate {
    pub name: String,
    pub description: String,
    pub url: String,
    pub auth_type: String,
    pub auth_config: serde_json::Value,
    pub custom_headers: HashMap<String, String>,
    pub payload_template: Option<serde_json::Value>,
    pub severity_filter: Vec<String>,
    pub category_filter: Vec<String>,
    pub retry_attempts: i32,
    pub timeout_seconds: i32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NotificationChannelTemplate {
    pub name: String,
    pub description: String,
    pub channel_type: String,
    pub config: serde_json::Value,
    pub severity_filter: Vec<String>,
    pub category_filter: Vec<String>,
    pub time_restrictions: Option<serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EscalationRuleTemplate {
    pub name: String,
    pub description: String,
    pub severity: String,
    pub category: Option<String>,
    pub unacknowledged_minutes: i32,
    pub escalate_to_severity: Option<String>,
    pub notify_channels: Vec<Uuid>,
    pub assign_to: Option<String>,
    pub is_active: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EnvironmentConfig {
    pub alert_retention_days: i32,
    pub webhook_timeout_seconds: u64,
    pub max_concurrent_alerts: usize,
    pub enable_rate_limiting: bool,
    pub enable_correlation: bool,
    pub enable_ml_detection: bool,
    pub log_level: String,
    pub metrics_enabled: bool,
    pub health_check_interval_minutes: u64,
    pub cache_ttl_minutes: i64,
}

/// Production deployment checklist and recommendations
pub struct ProductionDeploymentGuide;

impl ProductionDeploymentGuide {
    pub fn get_deployment_checklist() -> Vec<DeploymentCheckItem> {
        vec![
            DeploymentCheckItem {
                category: "Database".to_string(),
                item: "Run security alerts migration (010_security_alerts_and_webhooks.sql)".to_string(),
                critical: true,
                completed: false,
            },
            DeploymentCheckItem {
                category: "Configuration".to_string(),
                item: "Set up webhook endpoints with proper authentication".to_string(),
                critical: true,
                completed: false,
            },
            DeploymentCheckItem {
                category: "Configuration".to_string(),
                item: "Configure SMTP settings for email notifications".to_string(),
                critical: false,
                completed: false,
            },
            DeploymentCheckItem {
                category: "Configuration".to_string(),
                item: "Set up Slack webhook URLs for team notifications".to_string(),
                critical: false,
                completed: false,
            },
            DeploymentCheckItem {
                category: "Configuration".to_string(),
                item: "Configure PagerDuty integration for critical alerts".to_string(),
                critical: true,
                completed: false,
            },
            DeploymentCheckItem {
                category: "Security".to_string(),
                item: "Generate secure HMAC secrets for webhook authentication".to_string(),
                critical: true,
                completed: false,
            },
            DeploymentCheckItem {
                category: "Security".to_string(),
                item: "Set up proper rate limiting and CORS configuration".to_string(),
                critical: true,
                completed: false,
            },
            DeploymentCheckItem {
                category: "Monitoring".to_string(),
                item: "Configure alert system health monitoring".to_string(),
                critical: true,
                completed: false,
            },
            DeploymentCheckItem {
                category: "Testing".to_string(),
                item: "Test webhook deliveries to all configured endpoints".to_string(),
                critical: true,
                completed: false,
            },
            DeploymentCheckItem {
                category: "Testing".to_string(),
                item: "Verify alert rule triggers with test security events".to_string(),
                critical: true,
                completed: false,
            },
            DeploymentCheckItem {
                category: "Documentation".to_string(),
                item: "Document incident response procedures".to_string(),
                critical: false,
                completed: false,
            },
            DeploymentCheckItem {
                category: "Training".to_string(),
                item: "Train security team on alert dashboard and response procedures".to_string(),
                critical: false,
                completed: false,
            },
        ]
    }

    pub fn get_performance_tuning_recommendations() -> Vec<PerformanceTuning> {
        vec![
            PerformanceTuning {
                component: "Database".to_string(),
                recommendation: "Create appropriate indexes on security_alerts table for common queries".to_string(),
                impact: "High".to_string(),
                effort: "Low".to_string(),
            },
            PerformanceTuning {
                component: "Webhooks".to_string(),
                recommendation: "Implement connection pooling for webhook HTTP clients".to_string(),
                impact: "Medium".to_string(),
                effort: "Medium".to_string(),
            },
            PerformanceTuning {
                component: "Caching".to_string(),
                recommendation: "Use Redis for alert correlation cache in high-volume environments".to_string(),
                impact: "High".to_string(),
                effort: "Medium".to_string(),
            },
            PerformanceTuning {
                component: "Processing".to_string(),
                recommendation: "Tune alert processing queue size based on expected volume".to_string(),
                impact: "Medium".to_string(),
                effort: "Low".to_string(),
            },
            PerformanceTuning {
                component: "Monitoring".to_string(),
                recommendation: "Set up metrics collection for alert processing performance".to_string(),
                impact: "Medium".to_string(),
                effort: "Medium".to_string(),
            },
        ]
    }
}

#[derive(Debug, Serialize)]
pub struct DeploymentCheckItem {
    pub category: String,
    pub item: String,
    pub critical: bool,
    pub completed: bool,
}

#[derive(Debug, Serialize)]
pub struct PerformanceTuning {
    pub component: String,
    pub recommendation: String,
    pub impact: String,
    pub effort: String,
}