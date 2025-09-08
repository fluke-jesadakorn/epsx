// Event Payload Value Object
// Represents the data carried by real-time events

use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::UserId;

/// Event payload containing the actual event data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventPayload {
    event_type: EventType,
    data: serde_json::Value,
    metadata: HashMap<String, String>,
}

impl EventPayload {
    /// Create new event payload
    pub fn new(event_type: EventType, data: serde_json::Value) -> Self {
        Self {
            event_type,
            data,
            metadata: HashMap::new(),
        }
    }
    
    /// Add metadata to the event
    pub fn with_metadata(mut self, key: String, value: String) -> Self {
        self.metadata.insert(key, value);
        self
    }
    
    /// Get event type
    pub fn event_type(&self) -> &EventType {
        &self.event_type
    }
    
    /// Get event data
    pub fn data(&self) -> &serde_json::Value {
        &self.data
    }
    
    /// Get metadata
    pub fn metadata(&self) -> &HashMap<String, String> {
        &self.metadata
    }
    
    /// Validate payload size doesn't exceed limits
    pub fn validate_size(&self) -> Result<(), EventPayloadError> {
        let serialized = serde_json::to_string(self)
            .map_err(|e| EventPayloadError::SerializationFailed(e.to_string()))?;
        
        let size_kb = serialized.len() / 1024;
        if size_kb > super::super::RealtimeEventsBoundedContext::MAX_EVENT_PAYLOAD_SIZE_KB as usize {
            return Err(EventPayloadError::PayloadTooLarge {
                actual_kb: size_kb as u32,
                max_kb: super::super::RealtimeEventsBoundedContext::MAX_EVENT_PAYLOAD_SIZE_KB,
            });
        }
        
        Ok(())
    }
    
    /// Create payment event payloads
    pub fn payment_started(
        payment_id: String,
        user_id: UserId,
        amount: f64,
        currency: String,
    ) -> Self {
        let data = serde_json::json!({
            "payment_id": payment_id,
            "user_id": user_id.to_string(),
            "amount": amount,
            "currency": currency,
            "timestamp": Utc::now()
        });
        
        Self::new(EventType::PaymentStarted, data)
            .with_metadata("channel".to_string(), "payments".to_string())
    }
    
    pub fn payment_completed(
        payment_id: String,
        user_id: UserId,
        amount: f64,
        currency: String,
        transaction_id: String,
    ) -> Self {
        let data = serde_json::json!({
            "payment_id": payment_id,
            "user_id": user_id.to_string(),
            "amount": amount,
            "currency": currency,
            "transaction_id": transaction_id,
            "timestamp": Utc::now()
        });
        
        Self::new(EventType::PaymentCompleted, data)
            .with_metadata("channel".to_string(), "payments".to_string())
    }
    
    pub fn payment_failed(
        payment_id: String,
        user_id: UserId,
        amount: f64,
        currency: String,
        error_code: String,
        error_message: String,
    ) -> Self {
        let data = serde_json::json!({
            "payment_id": payment_id,
            "user_id": user_id.to_string(),
            "amount": amount,
            "currency": currency,
            "error_code": error_code,
            "error_message": error_message,
            "timestamp": Utc::now()
        });
        
        Self::new(EventType::PaymentFailed, data)
            .with_metadata("channel".to_string(), "payments".to_string())
    }
    
    /// Create notification event payload
    pub fn system_notification(
        title: String,
        message: String,
        level: NotificationLevel,
        target_user: Option<UserId>,
    ) -> Self {
        let data = serde_json::json!({
            "title": title,
            "message": message,
            "level": level,
            "target_user": target_user.map(|u| u.to_string()),
            "timestamp": Utc::now()
        });
        
        Self::new(EventType::SystemNotification, data)
            .with_metadata("channel".to_string(), "notifications".to_string())
    }
    
    /// Create trading event payload
    pub fn stock_price_update(
        symbol: String,
        price: f64,
        change: f64,
        change_percent: f64,
        volume: u64,
    ) -> Self {
        let data = serde_json::json!({
            "symbol": symbol,
            "price": price,
            "change": change,
            "change_percent": change_percent,
            "volume": volume,
            "timestamp": Utc::now()
        });
        
        Self::new(EventType::StockPriceUpdate, data)
            .with_metadata("channel".to_string(), "trading".to_string())
    }
}

/// Types of events that can be broadcast
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EventType {
    // Payment events
    PaymentStarted,
    PaymentCompleted,
    PaymentFailed,
    
    // Subscription events
    SubscriptionUpgraded,
    SubscriptionExpired,
    
    // Feature expiration events
    FeatureExpirationWarning,
    FeatureExpired,
    GracePeriodStarted,
    GracePeriodEnding,
    
    // Trading events
    StockPriceUpdate,
    TradeExecuted,
    
    // Notification events
    SystemNotification,
    
    // System health events
    HealthAlert,
    
    // User-defined events
    Custom(String),
}

impl EventType {
    /// Get the channel for this event type
    pub fn default_channel(&self) -> &'static str {
        match self {
            Self::PaymentStarted | Self::PaymentCompleted | Self::PaymentFailed => "payments",
            Self::SubscriptionUpgraded | Self::SubscriptionExpired => "subscriptions",
            Self::FeatureExpirationWarning | Self::FeatureExpired 
                | Self::GracePeriodStarted | Self::GracePeriodEnding => "features",
            Self::StockPriceUpdate | Self::TradeExecuted => "trading",
            Self::SystemNotification => "notifications",
            Self::HealthAlert => "health",
            Self::Custom(_) => "custom",
        }
    }
    
    /// Check if this event type requires user targeting
    pub fn requires_user_targeting(&self) -> bool {
        match self {
            Self::PaymentStarted | Self::PaymentCompleted | Self::PaymentFailed 
                | Self::SubscriptionUpgraded | Self::SubscriptionExpired
                | Self::FeatureExpirationWarning | Self::FeatureExpired
                | Self::GracePeriodStarted | Self::GracePeriodEnding
                | Self::TradeExecuted => true,
            Self::StockPriceUpdate | Self::HealthAlert => false,
            Self::SystemNotification => false, // Can be broadcast or targeted
            Self::Custom(_) => false, // Depends on implementation
        }
    }
}

/// Notification levels for system notifications
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NotificationLevel {
    Info,
    Warning,
    Error,
    Success,
}

/// Errors that can occur with event payloads
#[derive(Debug, thiserror::Error)]
pub enum EventPayloadError {
    #[error("Payload too large: {actual_kb}KB exceeds maximum {max_kb}KB")]
    PayloadTooLarge { actual_kb: u32, max_kb: u32 },
    
    #[error("Serialization failed: {0}")]
    SerializationFailed(String),
    
    #[error("Invalid event data format")]
    InvalidFormat,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_payment_event_creation() {
        let user_id = UserId::from_numeric(123);
        let payload = EventPayload::payment_started(
            "pay_123".to_string(),
            user_id,
            100.0,
            "USD".to_string(),
        );
        
        assert_eq!(payload.event_type(), &EventType::PaymentStarted);
        assert_eq!(payload.metadata().get("channel"), Some(&"payments".to_string()));
        
        // Validate the payload can be serialized
        let serialized = serde_json::to_string(&payload).unwrap();
        assert!(!serialized.is_empty());
    }
    
    #[test]
    fn test_event_type_channels() {
        assert_eq!(EventType::PaymentStarted.default_channel(), "payments");
        assert_eq!(EventType::StockPriceUpdate.default_channel(), "trading");
        assert_eq!(EventType::SystemNotification.default_channel(), "notifications");
    }
    
    #[test]
    fn test_user_targeting_requirements() {
        assert!(EventType::PaymentStarted.requires_user_targeting());
        assert!(EventType::TradeExecuted.requires_user_targeting());
        assert!(!EventType::StockPriceUpdate.requires_user_targeting());
        assert!(!EventType::HealthAlert.requires_user_targeting());
    }
}