// Domain events that trigger notification generation
// Event-driven notification system for automatic notification creation

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::dom::{
    ports::notification::{
        DomainNotification, DomainNotificationType, DomainNotificationPriority, 
        NotificationRecipient, NotificationError
    },
    values::{UserId, PayId, Currency},
};

/// Core event types that can trigger notifications
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum NotificationTriggerEvent {
    // User management events
    UserRegistered {
        user_id: UserId,
        email: String,
        created_at: DateTime<Utc>,
    },
    UserLoggedIn {
        user_id: UserId,
        login_time: DateTime<Utc>,
        ip_address: Option<String>,
        user_agent: Option<String>,
        is_new_device: bool,
    },
    PasswordChanged {
        user_id: UserId,
        changed_at: DateTime<Utc>,
        initiated_by: String,
    },
    
    // Payment events
    PaymentCompleted {
        user_id: UserId,
        payment_id: PayId,
        amount: rust_decimal::Decimal,
        currency: Currency,
        package_name: String,
        completed_at: DateTime<Utc>,
    },
    PaymentFailed {
        user_id: UserId,
        payment_id: PayId,
        amount: rust_decimal::Decimal,
        currency: Currency,
        failure_reason: String,
        failed_at: DateTime<Utc>,
    },
    SubscriptionRenewal {
        user_id: UserId,
        package_name: String,
        next_billing_date: DateTime<Utc>,
        amount: rust_decimal::Decimal,
    },
    SubscriptionCanceled {
        user_id: UserId,
        package_name: String,
        canceled_at: DateTime<Utc>,
        access_until: DateTime<Utc>,
    },
    
    // Permission and role events
    PermissionGranted {
        user_id: UserId,
        permission: String,
        granted_by: UserId,
        granted_at: DateTime<Utc>,
        expires_at: Option<DateTime<Utc>>,
    },
    PermissionRevoked {
        user_id: UserId,
        permission: String,
        revoked_by: UserId,
        revoked_at: DateTime<Utc>,
    },
    RoleAssigned {
        user_id: UserId,
        role: String,
        assigned_by: UserId,
        assigned_at: DateTime<Utc>,
    },
    FeatureAccessExpiring {
        user_id: UserId,
        feature_name: String,
        expires_at: DateTime<Utc>,
        days_remaining: i32,
    },
    FeatureAccessExpired {
        user_id: UserId,
        feature_name: String,
        expired_at: DateTime<Utc>,
        grace_period_days: i32,
    },
    
    // Analytics and trading events
    EPSThresholdReached {
        user_id: UserId,
        symbol: String,
        threshold_value: f64,
        current_value: f64,
        threshold_type: EPSThresholdType,
        triggered_at: DateTime<Utc>,
    },
    MarketAlert {
        user_id: UserId,
        alert_type: MarketAlertType,
        symbol: String,
        current_price: rust_decimal::Decimal,
        trigger_price: rust_decimal::Decimal,
        triggered_at: DateTime<Utc>,
    },
    WatchlistUpdate {
        user_id: UserId,
        symbol: String,
        change_type: String,
        previous_value: Option<rust_decimal::Decimal>,
        current_value: rust_decimal::Decimal,
    },
    
    // Security events
    SecurityThreatDetected {
        user_id: Option<UserId>,
        threat_type: String,
        severity: SecuritySeverity,
        details: HashMap<String, String>,
        detected_at: DateTime<Utc>,
    },
    BruteForceAttempt {
        target_user: Option<UserId>,
        ip_address: String,
        attempt_count: u32,
        detected_at: DateTime<Utc>,
    },
    SuspiciousActivity {
        user_id: UserId,
        activity_type: String,
        risk_score: f64,
        details: HashMap<String, String>,
        detected_at: DateTime<Utc>,
    },
    
    // System events
    SystemMaintenance {
        scheduled_start: DateTime<Utc>,
        estimated_duration: chrono::Duration,
        affected_services: Vec<String>,
        maintenance_type: MaintenanceType,
    },
    ServiceOutage {
        service_name: String,
        outage_start: DateTime<Utc>,
        estimated_resolution: Option<DateTime<Utc>>,
        severity: OutageSeverity,
    },
    PerformanceDegradation {
        service_name: String,
        metric_name: String,
        threshold_value: f64,
        current_value: f64,
        detected_at: DateTime<Utc>,
    },
}

// Supporting enums and types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum EPSThresholdType {
    Growth,
    Decline,
    Volatility,
    Ranking,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum MarketAlertType {
    PriceTarget,
    VolumeSpike,
    MovingAverage,
    Support,
    Resistance,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SecuritySeverity {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum MaintenanceType {
    Scheduled,
    Emergency,
    SecurityUpdate,
    FeatureDeployment,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum OutageSeverity {
    Minor,
    Major,
    Critical,
}

/// Context data for notification generation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationContext {
    pub template_variables: HashMap<String, serde_json::Value>,
    pub user_preferences: Option<UserNotificationPreferences>,
    pub delivery_channels: Vec<DeliveryChannel>,
    pub priority_override: Option<DomainNotificationPriority>,
    pub expiration_override: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserNotificationPreferences {
    pub email_enabled: bool,
    pub push_enabled: bool,
    pub websocket_enabled: bool,
    pub digest_mode: bool,
    pub quiet_hours_start: Option<chrono::NaiveTime>,
    pub quiet_hours_end: Option<chrono::NaiveTime>,
    pub timezone: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum DeliveryChannel {
    Email,
    Push,
    WebSocket,
    SMS,
    InApp,
}

/// Event handler trait for generating notifications from domain events
#[async_trait]
pub trait NotificationEventHandler: Send + Sync {
    /// Generate notifications from a trigger event
    async fn handle_event(&self, event: NotificationTriggerEvent, context: NotificationContext) 
        -> Result<Vec<DomainNotification>, NotificationError>;
    
    /// Check if this handler can process the given event type
    fn can_handle(&self, event: &NotificationTriggerEvent) -> bool;
    
    /// Get the priority for this handler (higher priority handlers run first)
    fn priority(&self) -> i32 {
        0
    }
}

/// Default notification generator implementation
pub struct DefaultNotificationGenerator;

impl DefaultNotificationGenerator {
    pub fn new() -> Self {
        Self
    }
    
    /// Generate notification content based on event type
    fn generate_notification_content(&self, event: &NotificationTriggerEvent, _context: &NotificationContext) 
        -> Result<(String, String, DomainNotificationType, DomainNotificationPriority), NotificationError> {
        
        match event {
            NotificationTriggerEvent::UserRegistered { email, .. } => {
                Ok((
                    "Welcome to EPSX!".to_string(),
                    format!("Welcome {}! Your account has been successfully created.", email),
                    DomainNotificationType::AccountUpdate,
                    DomainNotificationPriority::Normal,
                ))
            }
            
            NotificationTriggerEvent::UserLoggedIn { is_new_device, .. } => {
                if *is_new_device {
                    Ok((
                        "New Device Login".to_string(),
                        "We noticed a login from a new device. If this wasn't you, please secure your account.".to_string(),
                        DomainNotificationType::SecurityAlert,
                        DomainNotificationPriority::High,
                    ))
                } else {
                    // Don't generate notification for regular logins
                    Err(NotificationError::InvalidRecipient("Regular login - no notification needed".to_string()))
                }
            }
            
            NotificationTriggerEvent::PaymentCompleted { amount, currency, package_name, .. } => {
                Ok((
                    "Payment Successful".to_string(),
                    format!("Your payment of {} {} for {} has been processed successfully.", amount, currency, package_name),
                    DomainNotificationType::PaymentNotification,
                    DomainNotificationPriority::Normal,
                ))
            }
            
            NotificationTriggerEvent::PaymentFailed { amount, currency, failure_reason, .. } => {
                Ok((
                    "Payment Failed".to_string(),
                    format!("Your payment of {} {} failed: {}. Please try again or contact support.", amount, currency, failure_reason),
                    DomainNotificationType::PaymentNotification,
                    DomainNotificationPriority::High,
                ))
            }
            
            NotificationTriggerEvent::PermissionGranted { permission, expires_at, .. } => {
                let expiry_text = if let Some(expiry) = expires_at {
                    format!(" (expires on {})", expiry.format("%Y-%m-%d"))
                } else {
                    "".to_string()
                };
                
                Ok((
                    "New Access Granted".to_string(),
                    format!("You have been granted access to {}{}", permission, expiry_text),
                    DomainNotificationType::ModuleAccessChanged,
                    DomainNotificationPriority::Normal,
                ))
            }
            
            NotificationTriggerEvent::FeatureAccessExpiring { feature_name, days_remaining, .. } => {
                let priority = if *days_remaining <= 1 {
                    DomainNotificationPriority::High
                } else if *days_remaining <= 7 {
                    DomainNotificationPriority::Normal
                } else {
                    DomainNotificationPriority::Low
                };
                
                Ok((
                    "Feature Access Expiring".to_string(),
                    format!("Your access to {} expires in {} days. Renew now to continue using this feature.", 
                           feature_name, days_remaining),
                    DomainNotificationType::FeatureExpiration,
                    priority,
                ))
            }
            
            NotificationTriggerEvent::EPSThresholdReached { symbol, threshold_value, current_value, threshold_type, .. } => {
                let threshold_desc = match threshold_type {
                    EPSThresholdType::Growth => "growth threshold",
                    EPSThresholdType::Decline => "decline threshold", 
                    EPSThresholdType::Volatility => "volatility threshold",
                    EPSThresholdType::Ranking => "ranking threshold",
                };
                
                Ok((
                    format!("EPS Alert: {}", symbol),
                    format!("{} has reached your {} of {}%. Current value: {}%", 
                           symbol, threshold_desc, threshold_value, current_value),
                    DomainNotificationType::QuotaWarning, // Using QuotaWarning for analytics alerts
                    DomainNotificationPriority::High,
                ))
            }
            
            NotificationTriggerEvent::SecurityThreatDetected { threat_type, severity, .. } => {
                let priority = match severity {
                    SecuritySeverity::Critical => DomainNotificationPriority::Critical,
                    SecuritySeverity::High => DomainNotificationPriority::High,
                    SecuritySeverity::Medium => DomainNotificationPriority::Normal,
                    SecuritySeverity::Low => DomainNotificationPriority::Low,
                };
                
                Ok((
                    "Security Alert".to_string(),
                    format!("Security threat detected: {}. Please review your account security.", threat_type),
                    DomainNotificationType::SecurityAlert,
                    priority,
                ))
            }
            
            NotificationTriggerEvent::SystemMaintenance { scheduled_start, estimated_duration, affected_services, .. } => {
                let service_list = affected_services.join(", ");
                let duration_hours = estimated_duration.num_hours();
                
                Ok((
                    "Scheduled Maintenance".to_string(),
                    format!("Maintenance scheduled for {} affecting: {}. Expected duration: {} hours.", 
                           scheduled_start.format("%Y-%m-%d %H:%M UTC"), service_list, duration_hours),
                    DomainNotificationType::SystemMaintenance,
                    DomainNotificationPriority::Normal,
                ))
            }
            
            _ => {
                // For unhandled events, create a generic notification
                Ok((
                    "System Notification".to_string(),
                    "You have a new system notification. Please check your account for details.".to_string(),
                    DomainNotificationType::SystemMaintenance,
                    DomainNotificationPriority::Low,
                ))
            }
        }
    }
    
    /// Determine notification recipients based on event
    fn determine_recipients(&self, event: &NotificationTriggerEvent) -> Vec<NotificationRecipient> {
        match event {
            NotificationTriggerEvent::UserRegistered { user_id, .. } |
            NotificationTriggerEvent::UserLoggedIn { user_id, .. } |
            NotificationTriggerEvent::PaymentCompleted { user_id, .. } |
            NotificationTriggerEvent::PaymentFailed { user_id, .. } |
            NotificationTriggerEvent::PermissionGranted { user_id, .. } |
            NotificationTriggerEvent::PermissionRevoked { user_id, .. } |
            NotificationTriggerEvent::RoleAssigned { user_id, .. } |
            NotificationTriggerEvent::FeatureAccessExpiring { user_id, .. } |
            NotificationTriggerEvent::FeatureAccessExpired { user_id, .. } |
            NotificationTriggerEvent::EPSThresholdReached { user_id, .. } |
            NotificationTriggerEvent::MarketAlert { user_id, .. } |
            NotificationTriggerEvent::WatchlistUpdate { user_id, .. } |
            NotificationTriggerEvent::SuspiciousActivity { user_id, .. } |
            NotificationTriggerEvent::SubscriptionRenewal { user_id, .. } |
            NotificationTriggerEvent::SubscriptionCanceled { user_id, .. } => {
                vec![NotificationRecipient::User(user_id.clone())]
            }
            
            NotificationTriggerEvent::SecurityThreatDetected { user_id: Some(user_id), .. } => {
                vec![NotificationRecipient::User(user_id.clone())]
            }
            
            NotificationTriggerEvent::SecurityThreatDetected { user_id: None, .. } |
            NotificationTriggerEvent::BruteForceAttempt { .. } => {
                vec![NotificationRecipient::AdminGroup]
            }
            
            NotificationTriggerEvent::SystemMaintenance { .. } |
            NotificationTriggerEvent::ServiceOutage { .. } |
            NotificationTriggerEvent::PerformanceDegradation { .. } => {
                vec![NotificationRecipient::Broadcast]
            }
            
            NotificationTriggerEvent::PasswordChanged { user_id, .. } => {
                vec![NotificationRecipient::User(user_id.clone())]
            }
        }
    }
}

#[async_trait]
impl NotificationEventHandler for DefaultNotificationGenerator {
    async fn handle_event(&self, event: NotificationTriggerEvent, context: NotificationContext) 
        -> Result<Vec<DomainNotification>, NotificationError> {
        
        let (title, message, notification_type, base_priority) = 
            self.generate_notification_content(&event, &context)?;
        
        // Use priority override if provided
        let priority = context.priority_override.unwrap_or(base_priority);
        
        let recipients = self.determine_recipients(&event);
        let mut notifications = Vec::new();
        
        for recipient in recipients {
            // Create template variables
            let mut template_vars = context.template_variables.clone();
            
            // Add event-specific template variables
            match &event {
                NotificationTriggerEvent::PaymentCompleted { payment_id, amount, currency, .. } => {
                    template_vars.insert("payment_id".to_string(), serde_json::Value::String(payment_id.to_string()));
                    template_vars.insert("amount".to_string(), serde_json::Value::String(amount.to_string()));
                    template_vars.insert("currency".to_string(), serde_json::Value::String(format!("{:?}", currency)));
                }
                NotificationTriggerEvent::EPSThresholdReached { symbol, current_value, threshold_value, .. } => {
                    template_vars.insert("symbol".to_string(), serde_json::Value::String(symbol.clone()));
                    template_vars.insert("current_value".to_string(), 
                        serde_json::Number::from_f64(*current_value)
                            .map(serde_json::Value::Number)
                            .unwrap_or(serde_json::Value::Number(serde_json::Number::from(0))));
                    template_vars.insert("threshold_value".to_string(), 
                        serde_json::Number::from_f64(*threshold_value)
                            .map(serde_json::Value::Number)
                            .unwrap_or(serde_json::Value::Number(serde_json::Number::from(0))));
                }
                _ => {}
            }
            
            let context_data = if template_vars.is_empty() {
                None
            } else {
                Some(serde_json::to_value(template_vars).unwrap_or_default())
            };
            
            let notification = DomainNotification {
                id: None,
                recipient: recipient.clone(),
                notification_type: notification_type.clone(),
                priority: priority.clone(),
                title: title.clone(),
                message: message.clone(),
                data: context_data,
                scheduled_for: None,
                expires_at: context.expiration_override,
            };
            
            notifications.push(notification);
        }
        
        Ok(notifications)
    }
    
    fn can_handle(&self, _event: &NotificationTriggerEvent) -> bool {
        // Default handler can handle all events
        true
    }
    
    fn priority(&self) -> i32 {
        // Default handler has lowest priority
        -1000
    }
}

/// Event dispatcher for routing notification events to handlers
pub struct NotificationEventDispatcher {
    handlers: Vec<Box<dyn NotificationEventHandler>>,
}

impl NotificationEventDispatcher {
    pub fn new() -> Self {
        Self {
            handlers: vec![Box::new(DefaultNotificationGenerator::new())],
        }
    }
    
    pub fn add_handler(&mut self, handler: Box<dyn NotificationEventHandler>) {
        self.handlers.push(handler);
        // Sort by priority (highest first)
        self.handlers.sort_by(|a, b| b.priority().cmp(&a.priority()));
    }
    
    pub async fn dispatch_event(&self, event: NotificationTriggerEvent, context: NotificationContext) 
        -> Result<Vec<DomainNotification>, NotificationError> {
        
        let mut all_notifications = Vec::new();
        
        for handler in &self.handlers {
            if handler.can_handle(&event) {
                match handler.handle_event(event.clone(), context.clone()).await {
                    Ok(mut notifications) => {
                        all_notifications.append(&mut notifications);
                        // For most events, we only want one handler to process it
                        // But some events might need multiple handlers (e.g., logging + notification)
                        break;
                    }
                    Err(e) => {
                        // Log error but continue to next handler
                        tracing::warn!("Notification handler failed: {}", e);
                        continue;
                    }
                }
            }
        }
        
        Ok(all_notifications)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dom::values::identifiers::PayId;
    
    #[tokio::test]
    async fn test_payment_completed_notification() {
        let generator = DefaultNotificationGenerator::new();
        let event = NotificationTriggerEvent::PaymentCompleted {
            user_id: Uuid::new_v4(),
            payment_id: PayId::new(),
            amount: rust_decimal::Decimal::new(2999, 2), // $29.99
            currency: Currency::USD,
            package_name: "Premium Plan".to_string(),
            completed_at: Utc::now(),
        };
        
        let context = NotificationContext {
            template_variables: HashMap::new(),
            user_preferences: None,
            delivery_channels: vec![DeliveryChannel::Email, DeliveryChannel::InApp],
            priority_override: None,
            expiration_override: None,
        };
        
        let notifications = generator.handle_event(event, context).await.unwrap();
        assert!(!notifications.is_empty());
        
        let notification = &notifications[0];
        assert_eq!(notification.notification_type, DomainNotificationType::PaymentNotification);
        assert!(notification.title.contains("Payment Successful"));
        assert!(notification.message.contains("Premium Plan"));
    }
    
    #[tokio::test]
    async fn test_eps_threshold_notification() {
        let generator = DefaultNotificationGenerator::new();
        let event = NotificationTriggerEvent::EPSThresholdReached {
            user_id: Uuid::new_v4(),
            symbol: "AAPL".to_string(),
            threshold_value: 5.0,
            current_value: 5.2,
            threshold_type: EPSThresholdType::Growth,
            triggered_at: Utc::now(),
        };
        
        let context = NotificationContext {
            template_variables: HashMap::new(),
            user_preferences: None,
            delivery_channels: vec![DeliveryChannel::Push, DeliveryChannel::InApp],
            priority_override: None,
            expiration_override: None,
        };
        
        let notifications = generator.handle_event(event, context).await.unwrap();
        assert!(!notifications.is_empty());
        
        let notification = &notifications[0];
        assert_eq!(notification.priority, DomainNotificationPriority::High);
        assert!(notification.title.contains("AAPL"));
        assert!(notification.message.contains("growth threshold"));
    }
    
    #[tokio::test]
    async fn test_security_threat_notification() {
        let generator = DefaultNotificationGenerator::new();
        let event = NotificationTriggerEvent::SecurityThreatDetected {
            user_id: Some(Uuid::new_v4()),
            threat_type: "Brute force attack".to_string(),
            severity: SecuritySeverity::High,
            details: HashMap::new(),
            detected_at: Utc::now(),
        };
        
        let context = NotificationContext {
            template_variables: HashMap::new(),
            user_preferences: None,
            delivery_channels: vec![DeliveryChannel::Email, DeliveryChannel::Push],
            priority_override: None,
            expiration_override: None,
        };
        
        let notifications = generator.handle_event(event, context).await.unwrap();
        assert!(!notifications.is_empty());
        
        let notification = &notifications[0];
        assert_eq!(notification.notification_type, DomainNotificationType::SecurityAlert);
        assert_eq!(notification.priority, DomainNotificationPriority::High);
        assert!(notification.title.contains("Security Alert"));
    }
    
    #[tokio::test]
    async fn test_event_dispatcher() {
        let mut dispatcher = NotificationEventDispatcher::new();
        
        let event = NotificationTriggerEvent::UserRegistered {
            user_id: Uuid::new_v4(),
            email: "test@example.com".to_string(),
            created_at: Utc::now(),
        };
        
        let context = NotificationContext {
            template_variables: HashMap::new(),
            user_preferences: None,
            delivery_channels: vec![DeliveryChannel::Email],
            priority_override: None,
            expiration_override: None,
        };
        
        let notifications = dispatcher.dispatch_event(event, context).await.unwrap();
        assert!(!notifications.is_empty());
        
        let notification = &notifications[0];
        assert!(notification.title.contains("Welcome"));
        assert!(notification.message.contains("test@example.com"));
    }
}