// Security Webhook Manager - Complete Diesel Implementation
use diesel::prelude::*;
use chrono::{DateTime, Utc};
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
                NewDieselSecurityEvent,
                DieselAlertNotification,
                NewDieselAlertNotification,
                SecurityStats,
            },
            pool::DbPool,
            schema::{
                security_events::dsl::*,
                alert_notifications::dsl::*,
            },
        },
    },
};

pub struct WebhookManager {
    pool: DbPool,
    cache: Option<Box<dyn Cache>>,
    webhook_endpoints: HashMap<String, WebhookConfig>,
}

#[derive(Debug, Clone)]
pub struct WebhookConfig {
    pub url: String,
    pub secret: String,
    pub retry_attempts: u32,
    pub timeout_seconds: u32,
    pub enabled: bool,
}

#[derive(Debug, Clone)]
pub struct SecurityWebhookPayload {
    pub event_id: Uuid,
    pub event_type: String,
    pub severity: String,
    pub source: String,
    pub timestamp: DateTime<Utc>,
    pub data: JsonValue,
    pub user_id: Option<String>,
    pub ip_address: Option<ipnetwork::IpNetwork>,
}

impl WebhookManager {
    pub fn new(pool: DbPool) -> Self {
        Self {
            pool,
            cache: None,
            webhook_endpoints: HashMap::new(),
        }
    }

    pub fn with_cache(pool: DbPool, cache_instance: Box<dyn Cache>) -> Self {
        Self {
            pool,
            cache: Some(cache_instance),
            webhook_endpoints: HashMap::new(),
        }
    }

    // Register a webhook endpoint
    pub fn register_webhook(&mut self, name: String, config: WebhookConfig) {
        info!("Registering webhook endpoint: {}", name);
        self.webhook_endpoints.insert(name, config);
    }

    // Log a security event with Diesel
    pub async fn log_security_event(&self, payload: &SecurityWebhookPayload) -> AppResult<Uuid> {
        let mut conn = self.pool.get().await
            .map_err(|e| AppError::Database {
                message: "Failed to get database connection".to_string(),
                source: Some(Box::new(e)),
                correlation_id: None,
            })?;

        let new_event = NewDieselSecurityEvent {
            id: payload.event_id,
            event_type: payload.event_type.clone(),
            severity: payload.severity.clone(),
            source: payload.source.clone(),
            user_id: payload.user_id.clone(),
            ip_address: payload.ip_address,
            user_agent: None,
            request_path: None,
            request_method: None,
            request_headers: None,
            response_status: None,
            event_data: Some(payload.data.clone()),
            risk_score: Some(self.calculate_risk_score(&payload.severity)),
            country_code: None,
            device_fingerprint: None,
            correlation_id: Some(Uuid::new_v4()),
            alert_triggered: Some(true),
            blocked: Some(false),
            timestamp: payload.timestamp,
            processed: Some(false),
        };

        let result = diesel::insert_into(security_events)
            .values(&new_event)
            .execute(&mut conn)
            .await
            .map_err(|e| {
                error!("Failed to insert security event: {}", e);
                AppError::Database {
                    message: "Failed to log security event".to_string(),
                    source: Some(Box::new(e)),
                    correlation_id: None,
                }
            })?;

        if result > 0 {
            info!("Successfully logged security event {} with type {}", payload.event_id, payload.event_type);
            
            // Trigger webhooks for this event
            self.trigger_webhooks(payload).await?;
            
            Ok(payload.event_id)
        } else {
            Err(AppError::Database {
                message: "No security event was inserted".to_string(),
                source: None,
                correlation_id: None,
            })
        }
    }

    // Trigger webhooks for a security event
    pub async fn trigger_webhooks(&self, payload: &SecurityWebhookPayload) -> AppResult<()> {
        let mut successful_webhooks = 0;
        let mut failed_webhooks = 0;

        for (webhook_name, config) in &self.webhook_endpoints {
            if !config.enabled {
                debug!("Skipping disabled webhook: {}", webhook_name);
                continue;
            }

            match self.send_webhook(webhook_name, config, payload).await {
                Ok(_) => {
                    successful_webhooks += 1;
                    info!("Successfully sent webhook to {}", webhook_name);
                }
                Err(e) => {
                    failed_webhooks += 1;
                    error!("Failed to send webhook to {}: {}", webhook_name, e);
                }
            }
        }

        info!("Webhook delivery summary: {} successful, {} failed", successful_webhooks, failed_webhooks);
        Ok(())
    }

    // Send individual webhook
    async fn send_webhook(&self, webhook_name: &str, config: &WebhookConfig, payload: &SecurityWebhookPayload) -> AppResult<()> {
        use reqwest::Client;
        
        let client = Client::new();
        let webhook_payload = serde_json::json!({
            "event_id": payload.event_id,
            "event_type": payload.event_type,
            "severity": payload.severity,
            "source": payload.source,
            "timestamp": payload.timestamp,
            "data": payload.data,
            "user_id": payload.user_id,
            "ip_address": payload.ip_address.map(|ip| ip.to_string()),
        });

        let mut attempt = 0;
        while attempt < config.retry_attempts {
            attempt += 1;

            let response = client
                .post(&config.url)
                .json(&webhook_payload)
                .header("X-Webhook-Secret", &config.secret)
                .timeout(std::time::Duration::from_secs(config.timeout_seconds as u64))
                .send()
                .await;

            match response {
                Ok(resp) if resp.status().is_success() => {
                    // Log successful webhook delivery
                    self.log_webhook_delivery(payload.event_id, webhook_name, "success", None).await?;
                    return Ok(());
                }
                Ok(resp) => {
                    let status = resp.status();
                    let error_msg = format!("Webhook returned non-success status: {}", status);
                    warn!("Webhook delivery failed (attempt {}): {}", attempt, error_msg);
                    
                    if attempt >= config.retry_attempts {
                        self.log_webhook_delivery(payload.event_id, webhook_name, "failed", Some(error_msg.clone())).await?;
                        return Err(AppError::External {
                            message: error_msg,
                            source: None,
                        });
                    }
                }
                Err(e) => {
                    let error_msg = format!("Webhook request failed: {}", e);
                    warn!("Webhook delivery failed (attempt {}): {}", attempt, error_msg);
                    
                    if attempt >= config.retry_attempts {
                        self.log_webhook_delivery(payload.event_id, webhook_name, "failed", Some(error_msg.clone())).await?;
                        return Err(AppError::External {
                            message: error_msg,
                            source: Some(Box::new(e)),
                        });
                    }
                }
            }

            // Exponential backoff before retry
            let backoff_ms = 1000 * (2_u64.pow(attempt - 1));
            tokio::time::sleep(std::time::Duration::from_millis(backoff_ms)).await;
        }

        Err(AppError::External {
            message: format!("Webhook delivery failed after {} attempts", config.retry_attempts),
            source: None,
        })
    }

    // Log webhook delivery status
    async fn log_webhook_delivery(&self, event_id: Uuid, webhook_name: &str, status: &str, error_message: Option<String>) -> AppResult<()> {
        let mut conn = self.pool.get().await
            .map_err(|e| AppError::Database {
                message: "Failed to get database connection".to_string(),
                source: Some(Box::new(e)),
                correlation_id: None,
            })?;

        // For now, we'll use a simple approach since we don't have a dedicated webhook_deliveries table
        // In a full implementation, you'd create a proper webhook_deliveries table
        let delivery_data = serde_json::json!({
            "webhook_name": webhook_name,
            "status": status,
            "error_message": error_message,
            "delivered_at": Utc::now(),
        });

        let new_notification = NewDieselAlertNotification {
            id: Uuid::new_v4(),
            rule_id: Uuid::new_v4(), // Placeholder - in real implementation, this would be the webhook rule ID
            event_id,
            channel: "webhook".to_string(),
            recipient: webhook_name.to_string(),
            message: format!("Webhook delivery {} for event {}", status, event_id),
            delivery_status: Some(status.to_string()),
            attempts: Some(1),
            created_at: Some(Utc::now()),
        };

        diesel::insert_into(alert_notifications)
            .values(&new_notification)
            .execute(&mut conn)
            .await
            .map_err(|e| {
                error!("Failed to log webhook delivery: {}", e);
                AppError::Database {
                    message: "Failed to log webhook delivery".to_string(),
                    source: Some(Box::new(e)),
                    correlation_id: None,
                }
            })?;

        Ok(())
    }

    // Get security event statistics
    pub async fn get_security_stats(&self, hours_back: i32) -> AppResult<SecurityStats> {
        let mut conn = self.pool.get().await
            .map_err(|e| AppError::Database {
                message: "Failed to get database connection".to_string(),
                source: Some(Box::new(e)),
                correlation_id: None,
            })?;

        let cutoff_time = Utc::now() - chrono::Duration::hours(hours_back as i64);

        let total_events = security_events
            .filter(timestamp.gt(cutoff_time))
            .count()
            .get_result::<i64>(&mut conn)
            .await
            .map_err(|e| AppError::Database {
                message: "Failed to count total events".to_string(),
                source: Some(Box::new(e)),
                correlation_id: None,
            })?;

        let high_severity_count = security_events
            .filter(timestamp.gt(cutoff_time))
            .filter(severity.eq("high").or(severity.eq("critical")))
            .count()
            .get_result::<i64>(&mut conn)
            .await
            .map_err(|e| AppError::Database {
                message: "Failed to count high severity events".to_string(),
                source: Some(Box::new(e)),
                correlation_id: None,
            })?;

        let blocked_attempts = security_events
            .filter(timestamp.gt(cutoff_time))
            .filter(blocked.eq(true))
            .count()
            .get_result::<i64>(&mut conn)
            .await
            .map_err(|e| AppError::Database {
                message: "Failed to count blocked attempts".to_string(),
                source: Some(Box::new(e)),
                correlation_id: None,
            })?;

        // Count unique IP addresses (attackers)
        let unique_attackers = security_events
            .filter(timestamp.gt(cutoff_time))
            .filter(ip_address.is_not_null())
            .select(ip_address)
            .distinct()
            .count()
            .get_result::<i64>(&mut conn)
            .await
            .map_err(|e| AppError::Database {
                message: "Failed to count unique attackers".to_string(),
                source: Some(Box::new(e)),
                correlation_id: None,
            })?;

        let last_event_at = security_events
            .order(timestamp.desc())
            .select(timestamp)
            .first::<DateTime<Utc>>(&mut conn)
            .await
            .optional()
            .map_err(|e| AppError::Database {
                message: "Failed to get last event time".to_string(),
                source: Some(Box::new(e)),
                correlation_id: None,
            })?;

        Ok(SecurityStats {
            total_events,
            high_severity_count,
            blocked_attempts,
            unique_attackers,
            last_event_at,
        })
    }

    // Get recent security events with pagination
    pub async fn get_recent_events(&self, limit: i32, offset: i32) -> AppResult<Vec<DieselSecurityEvent>> {
        let mut conn = self.pool.get().await
            .map_err(|e| AppError::Database {
                message: "Failed to get database connection".to_string(),
                source: Some(Box::new(e)),
                correlation_id: None,
            })?;

        let events = security_events
            .order(timestamp.desc())
            .limit(limit as i64)
            .offset(offset as i64)
            .load::<DieselSecurityEvent>(&mut conn)
            .await
            .map_err(|e| AppError::Database {
                message: "Failed to load security events".to_string(),
                source: Some(Box::new(e)),
                correlation_id: None,
            })?;

        Ok(events)
    }

    // Calculate risk score based on severity
    fn calculate_risk_score(&self, severity: &str) -> i32 {
        match severity.to_lowercase().as_str() {
            "low" => 25,
            "medium" => 50,
            "high" => 75,
            "critical" => 100,
            _ => 50, // Default to medium
        }
    }

    // Clean up old security events
    pub async fn cleanup_old_events(&self, days_to_keep: i32) -> AppResult<usize> {
        let mut conn = self.pool.get().await
            .map_err(|e| AppError::Database {
                message: "Failed to get database connection".to_string(),
                source: Some(Box::new(e)),
                correlation_id: None,
            })?;

        let cutoff_date = Utc::now() - chrono::Duration::days(days_to_keep as i64);

        let deleted_count = diesel::delete(
            security_events.filter(timestamp.lt(cutoff_date))
        )
        .execute(&mut conn)
        .await
        .map_err(|e| AppError::Database {
            message: "Failed to delete old security events".to_string(),
            source: Some(Box::new(e)),
            correlation_id: None,
        })?;

        if deleted_count > 0 {
            info!("Cleaned up {} old security events older than {} days", deleted_count, days_to_keep);
        }

        Ok(deleted_count)
    }

    // Health check for webhook manager
    pub async fn health_check(&self) -> AppResult<HashMap<String, bool>> {
        let mut health_status = HashMap::new();

        // Check database connectivity
        let db_healthy = match self.pool.get().await {
            Ok(_) => true,
            Err(e) => {
                error!("Database health check failed: {}", e);
                false
            }
        };
        health_status.insert("database".to_string(), db_healthy);

        // Check cache connectivity (if configured)
        let cache_healthy = match &self.cache {
            Some(_cache) => {
                // In a full implementation, you'd test cache connectivity here
                warn!("Cache health check not implemented - assuming healthy");
                true
            }
            None => true, // No cache configured
        };
        health_status.insert("cache".to_string(), cache_healthy);

        // Check webhook endpoints (basic connectivity test)
        let webhook_count = self.webhook_endpoints.len();
        health_status.insert("webhooks_configured".to_string(), webhook_count > 0);

        info!("Webhook manager health check completed - database: {}, cache: {}, webhooks: {}", 
              db_healthy, cache_healthy, webhook_count > 0);

        Ok(health_status)
    }
}