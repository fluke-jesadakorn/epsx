// Real-time event definitions for payment tracking and notifications

use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use std::collections::HashMap;

/// Real-time event types for the trading platform
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum RealtimeEvent {
    /// Payment processing events
    PaymentStarted {
        payment_id: String,
        user_id: String,
        amount: f64,
        currency: String,
        timestamp: DateTime<Utc>,
    },
    PaymentCompleted {
        payment_id: String,
        user_id: String,
        amount: f64,
        currency: String,
        transaction_id: String,
        timestamp: DateTime<Utc>,
    },
    PaymentFailed {
        payment_id: String,
        user_id: String,
        amount: f64,
        currency: String,
        error_code: String,
        error_message: String,
        timestamp: DateTime<Utc>,
    },
    
    /// Subscription events
    SubscriptionUpgraded {
        user_id: String,
        old_tier: String,
        new_tier: String,
        timestamp: DateTime<Utc>,
    },
    SubscriptionExpired {
        user_id: String,
        tier: String,
        timestamp: DateTime<Utc>,
    },
    
    /// Stock trading events
    StockPriceUpdate {
        symbol: String,
        price: f64,
        change: f64,
        change_percent: f64,
        volume: u64,
        timestamp: DateTime<Utc>,
    },
    TradeExecuted {
        user_id: String,
        symbol: String,
        action: String, // "buy" or "sell"
        quantity: u32,
        price: f64,
        total_amount: f64,
        timestamp: DateTime<Utc>,
    },
    
    /// Notification events
    SystemNotification {
        title: String,
        message: String,
        level: NotificationLevel,
        target_user: Option<String>, // None for broadcast
        metadata: HashMap<String, serde_json::Value>,
        timestamp: DateTime<Utc>,
    },
    
    /// System health events
    HealthAlert {
        component: String, 
        status: String,
        message: String,
        severity: String,
        timestamp: DateTime<Utc>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NotificationLevel {
    Info,
    Warning,
    Error,
    Success,
}

/// Event metadata for routing and processing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventMetadata {
    pub event_id: String,
    pub timestamp: DateTime<Utc>,
    pub source: String,
    pub user_id: Option<String>,
    pub session_id: Option<String>,
    pub retry_count: u32,
}

/// Wrapper for events with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventMessage {
    pub metadata: EventMetadata,
    pub event: RealtimeEvent,
}

impl EventMessage {
    pub fn new(event: RealtimeEvent, source: String) -> Self {
        Self {
            metadata: EventMetadata {
                event_id: uuid::Uuid::new_v4().to_string(),
                timestamp: Utc::now(),
                source,
                user_id: None,
                session_id: None,
                retry_count: 0,
            },
            event,
        }
    }
    
    pub fn with_user_id(mut self, user_id: String) -> Self {
        self.metadata.user_id = Some(user_id);
        self
    }
    
    pub fn with_session_id(mut self, session_id: String) -> Self {
        self.metadata.session_id = Some(session_id);
        self
    }
}

/// Payment tracking specific events
impl RealtimeEvent {
    pub fn payment_started(payment_id: String, user_id: String, amount: f64, currency: String) -> Self {
        Self::PaymentStarted {
            payment_id,
            user_id,
            amount,
            currency,
            timestamp: Utc::now(),
        }
    }
    
    pub fn payment_completed(payment_id: String, user_id: String, amount: f64, currency: String, transaction_id: String) -> Self {
        Self::PaymentCompleted {
            payment_id,
            user_id,
            amount,
            currency,
            transaction_id,
            timestamp: Utc::now(),
        }
    }
    
    pub fn payment_failed(payment_id: String, user_id: String, amount: f64, currency: String, error_code: String, error_message: String) -> Self {
        Self::PaymentFailed {
            payment_id,
            user_id,
            amount,
            currency,
            error_code,
            error_message,
            timestamp: Utc::now(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn should_create_payment_started_event() {
        let event = RealtimeEvent::payment_started(
            "pay_123".to_string(),
            "user_456".to_string(),
            99.99,
            "USD".to_string()
        );
        
        match event {
            RealtimeEvent::PaymentStarted { payment_id, user_id, amount, currency, .. } => {
                assert_eq!(payment_id, "pay_123");
                assert_eq!(user_id, "user_456");
                assert_eq!(amount, 99.99);
                assert_eq!(currency, "USD");
            },
            _ => panic!("Expected PaymentStarted event"),
        }
    }
    
    #[test]
    fn should_create_event_message_with_metadata() {
        let event = RealtimeEvent::payment_started(
            "pay_123".to_string(),
            "user_456".to_string(),
            99.99,
            "USD".to_string()
        );
        
        let message = EventMessage::new(event, "payment-service".to_string())
            .with_user_id("user_456".to_string());
        
        assert_eq!(message.metadata.source, "payment-service");
        assert_eq!(message.metadata.user_id, Some("user_456".to_string()));
        assert!(!message.metadata.event_id.is_empty());
    }
    
    #[test]
    fn should_serialize_event_message() {
        let event = RealtimeEvent::SystemNotification {
            title: "Test".to_string(),
            message: "Test message".to_string(),
            level: NotificationLevel::Info,
            target_user: None,
            metadata: HashMap::new(),
            timestamp: Utc::now(),
        };
        
        let message = EventMessage::new(event, "test".to_string());
        let json = serde_json::to_string(&message).unwrap();
        
        assert!(json.contains("SystemNotification"));
        assert!(json.contains("Test message"));
    }
}