use serde::{Deserialize, Serialize};
use std::fmt::{self, Display};
use std::collections::HashSet;

/// Delivery Channel Types - pure domain enums
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DeliveryChannelType {
    #[serde(rename = "fcm_push")]
    FcmPush,
    #[serde(rename = "push")]
    Push,
    #[serde(rename = "in_app")]
    InApp,
    #[serde(rename = "email")]
    Email,
    #[serde(rename = "sms")]
    Sms,
    #[serde(rename = "SMS")]
    SMS,
}

impl Display for DeliveryChannelType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            DeliveryChannelType::FcmPush => "FCM Push",
            DeliveryChannelType::Push => "Push",
            DeliveryChannelType::InApp => "In-App",
            DeliveryChannelType::Email => "Email",
            DeliveryChannelType::Sms => "SMS",
            DeliveryChannelType::SMS => "SMS",
        };
        write!(f, "{}", s)
    }
}

/// Delivery Channel Value Object
/// Wraps the delivery channel type with additional behavior and validation
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DeliveryChannel {
    channel_type: DeliveryChannelType,
    is_enabled: bool,
    retry_config: RetryConfiguration,
}

impl DeliveryChannel {
    /// Create new delivery channel
    pub fn new(channel_type: DeliveryChannelType) -> Self {
        let retry_config = RetryConfiguration::default_for_channel(&channel_type);
        
        Self {
            channel_type,
            is_enabled: true,
            retry_config,
        }
    }

    /// Create disabled channel
    pub fn disabled(channel_type: DeliveryChannelType) -> Self {
        Self {
            channel_type,
            is_enabled: false,
            retry_config: RetryConfiguration::no_retry(),
        }
    }

    /// Create with custom retry configuration
    pub fn with_retry_config(channel_type: DeliveryChannelType, retry_config: RetryConfiguration) -> Self {
        Self {
            channel_type,
            is_enabled: true,
            retry_config,
        }
    }

    /// Get the channel type
    pub fn channel_type(&self) -> &DeliveryChannelType {
        &self.channel_type
    }

    /// Check if channel is enabled
    pub fn is_enabled(&self) -> bool {
        self.is_enabled
    }

    /// Enable the channel
    pub fn enable(&mut self) {
        self.is_enabled = true;
    }

    /// Disable the channel
    pub fn disable(&mut self) {
        self.is_enabled = false;
    }

    /// Get retry configuration
    pub fn retry_config(&self) -> &RetryConfiguration {
        &self.retry_config
    }

    /// Check if this channel supports rich content (images, buttons, etc.)
    pub fn supports_rich_content(&self) -> bool {
        match self.channel_type {
            DeliveryChannelType::FcmPush => true,
            DeliveryChannelType::Push => true,
            DeliveryChannelType::InApp => true,
            DeliveryChannelType::Email => true,
            DeliveryChannelType::Sms => false,
            DeliveryChannelType::SMS => false,
        }
    }

    /// Check if this channel supports real-time delivery
    pub fn is_realtime(&self) -> bool {
        match self.channel_type {
            DeliveryChannelType::FcmPush => true,
            DeliveryChannelType::Push => true,
            DeliveryChannelType::InApp => true,
            DeliveryChannelType::Email => false,
            DeliveryChannelType::Sms => true,
            DeliveryChannelType::SMS => true,
        }
    }

    /// Get typical delivery time in seconds
    pub fn typical_delivery_time_seconds(&self) -> u32 {
        match self.channel_type {
            DeliveryChannelType::FcmPush => 5,      // Near instant
            DeliveryChannelType::Push => 5,         // Near instant
            DeliveryChannelType::InApp => 1,        // Instant
            DeliveryChannelType::Email => 60,       // 1 minute
            DeliveryChannelType::Sms => 30,         // 30 seconds
            DeliveryChannelType::SMS => 30,         // 30 seconds
        }
    }

    /// Get maximum content length for this channel
    pub fn max_content_length(&self) -> Option<ContentLimits> {
        match self.channel_type {
            DeliveryChannelType::FcmPush => Some(ContentLimits {
                title_max: 50,
                body_max: 150,
                total_max: 4000, // FCM payload limit
            }),
            DeliveryChannelType::Push => Some(ContentLimits {
                title_max: 50,
                body_max: 150,
                total_max: 4000, // Push payload limit
            }),
            DeliveryChannelType::InApp => None, // No strict limits
            DeliveryChannelType::Email => Some(ContentLimits {
                title_max: 100, // Subject line
                body_max: 50000, // Reasonable email size
                total_max: 50000,
            }),
            DeliveryChannelType::Sms => Some(ContentLimits {
                title_max: 0, // No title in SMS
                body_max: 160, // Single SMS
                total_max: 160,
            }),
            DeliveryChannelType::SMS => Some(ContentLimits {
                title_max: 0, // No title in SMS
                body_max: 160, // Single SMS
                total_max: 160,
            }),
        }
    }

    /// Get cost tier for this delivery channel
    pub fn cost_tier(&self) -> DeliveryCost {
        match self.channel_type {
            DeliveryChannelType::FcmPush => DeliveryCost::Free,
            DeliveryChannelType::Push => DeliveryCost::Free,
            DeliveryChannelType::InApp => DeliveryCost::Free,
            DeliveryChannelType::Email => DeliveryCost::Low,
            DeliveryChannelType::Sms => DeliveryCost::High,
            DeliveryChannelType::SMS => DeliveryCost::High,
        }
    }

    /// Check if channel requires user opt-in
    pub fn requires_opt_in(&self) -> bool {
        match self.channel_type {
            DeliveryChannelType::FcmPush => true,
            DeliveryChannelType::Push => true,
            DeliveryChannelType::InApp => false,
            DeliveryChannelType::Email => true,
            DeliveryChannelType::Sms => true,
            DeliveryChannelType::SMS => true,
        }
    }

    /// Get privacy level for this channel
    pub fn privacy_level(&self) -> PrivacyLevel {
        match self.channel_type {
            DeliveryChannelType::FcmPush => PrivacyLevel::Medium,
            DeliveryChannelType::Push => PrivacyLevel::Medium,
            DeliveryChannelType::InApp => PrivacyLevel::Low,
            DeliveryChannelType::Email => PrivacyLevel::High,
            DeliveryChannelType::Sms => PrivacyLevel::High,
            DeliveryChannelType::SMS => PrivacyLevel::High,
        }
    }
}

/// Retry configuration for delivery attempts
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RetryConfiguration {
    pub max_attempts: u8,
    pub initial_delay_seconds: u32,
    pub max_delay_seconds: u32,
    pub backoff_multiplier: f32,
    pub retry_on_errors: HashSet<String>,
}

impl RetryConfiguration {
    /// No retry configuration
    pub fn no_retry() -> Self {
        Self {
            max_attempts: 1,
            initial_delay_seconds: 0,
            max_delay_seconds: 0,
            backoff_multiplier: 1.0,
            retry_on_errors: HashSet::new(),
        }
    }

    /// Default retry configuration for a channel
    pub fn default_for_channel(channel: &DeliveryChannelType) -> Self {
        match channel {
            DeliveryChannelType::FcmPush => Self {
                max_attempts: 3,
                initial_delay_seconds: 30,
                max_delay_seconds: 300,
                backoff_multiplier: 2.0,
                retry_on_errors: ["timeout", "server_error", "rate_limited"].iter().map(|s| s.to_string()).collect(),
            },
            DeliveryChannelType::Push => Self {
                max_attempts: 3,
                initial_delay_seconds: 30,
                max_delay_seconds: 300,
                backoff_multiplier: 2.0,
                retry_on_errors: ["timeout", "server_error", "rate_limited"].iter().map(|s| s.to_string()).collect(),
            },
            DeliveryChannelType::InApp => Self::no_retry(), // In-app is immediate
            DeliveryChannelType::Email => Self {
                max_attempts: 5,
                initial_delay_seconds: 60,
                max_delay_seconds: 3600,
                backoff_multiplier: 2.0,
                retry_on_errors: ["timeout", "server_error", "rate_limited", "temp_failure"].iter().map(|s| s.to_string()).collect(),
            },
            DeliveryChannelType::Sms => Self {
                max_attempts: 3,
                initial_delay_seconds: 120,
                max_delay_seconds: 1800,
                backoff_multiplier: 1.5,
                retry_on_errors: ["timeout", "server_error", "rate_limited"].iter().map(|s| s.to_string()).collect(),
            },
            DeliveryChannelType::SMS => Self {
                max_attempts: 3,
                initial_delay_seconds: 120,
                max_delay_seconds: 1800,
                backoff_multiplier: 1.5,
                retry_on_errors: ["timeout", "server_error", "rate_limited"].iter().map(|s| s.to_string()).collect(),
            },
        }
    }

    /// Calculate delay for attempt number
    pub fn delay_for_attempt(&self, attempt: u8) -> u32 {
        if attempt <= 1 {
            return 0;
        }
        
        let delay = self.initial_delay_seconds as f32 * self.backoff_multiplier.powi((attempt - 2) as i32);
        delay.min(self.max_delay_seconds as f32) as u32
    }

    /// Check if error should trigger retry
    pub fn should_retry_error(&self, error: &str) -> bool {
        self.retry_on_errors.iter().any(|pattern| error.to_lowercase().contains(&pattern.to_lowercase()))
    }
}

/// Content limits for different channels
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct ContentLimits {
    pub title_max: usize,
    pub body_max: usize,
    pub total_max: usize,
}

/// Delivery cost classification
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DeliveryCost {
    Free,
    Low,
    Medium,
    High,
}

impl DeliveryCost {
    pub fn as_str(&self) -> &'static str {
        match self {
            DeliveryCost::Free => "free",
            DeliveryCost::Low => "low",
            DeliveryCost::Medium => "medium",
            DeliveryCost::High => "high",
        }
    }
}

/// Privacy level for delivery channels
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PrivacyLevel {
    Low,    // In-app notifications
    Medium, // Push notifications
    High,   // Email, SMS
}

impl PrivacyLevel {
    pub fn as_str(&self) -> &'static str {
        match self {
            PrivacyLevel::Low => "low",
            PrivacyLevel::Medium => "medium",
            PrivacyLevel::High => "high",
        }
    }
}

impl Display for DeliveryChannel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let status = if self.is_enabled { "enabled" } else { "disabled" };
        write!(f, "{} ({})", self.channel_type, status)
    }
}

impl From<DeliveryChannelType> for DeliveryChannel {
    fn from(channel_type: DeliveryChannelType) -> Self {
        Self::new(channel_type)
    }
}

/// Multi-channel configuration for notifications
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MultiChannelConfig {
    channels: Vec<DeliveryChannel>,
    fallback_enabled: bool,
    primary_channel: Option<DeliveryChannelType>,
}

impl MultiChannelConfig {
    /// Create new multi-channel configuration
    pub fn new(channels: Vec<DeliveryChannel>) -> Self {
        Self {
            channels,
            fallback_enabled: true,
            primary_channel: None,
        }
    }

    /// Create single channel configuration
    pub fn single_channel(channel: DeliveryChannel) -> Self {
        let channel_type = channel.channel_type.clone();
        Self {
            channels: vec![channel],
            fallback_enabled: false,
            primary_channel: Some(channel_type),
        }
    }

    /// Get all enabled channels
    pub fn enabled_channels(&self) -> Vec<&DeliveryChannel> {
        self.channels.iter().filter(|c| c.is_enabled()).collect()
    }

    /// Get channels by cost tier
    pub fn channels_by_cost(&self, max_cost: DeliveryCost) -> Vec<&DeliveryChannel> {
        self.channels
            .iter()
            .filter(|c| c.is_enabled() && Self::cost_is_within_limit(c.cost_tier(), max_cost))
            .collect()
    }

    /// Get realtime channels only
    pub fn realtime_channels(&self) -> Vec<&DeliveryChannel> {
        self.enabled_channels()
            .into_iter()
            .filter(|c| c.is_realtime())
            .collect()
    }

    /// Check if fallback is enabled
    pub fn fallback_enabled(&self) -> bool {
        self.fallback_enabled
    }

    /// Get primary channel
    pub fn primary_channel(&self) -> Option<&DeliveryChannelType> {
        self.primary_channel.as_ref()
    }

    /// Set fallback enabled
    pub fn set_fallback_enabled(&mut self, enabled: bool) {
        self.fallback_enabled = enabled;
    }

    fn cost_is_within_limit(channel_cost: DeliveryCost, max_cost: DeliveryCost) -> bool {
        match (channel_cost, max_cost) {
            (DeliveryCost::Free, _) => true,
            (DeliveryCost::Low, DeliveryCost::Free) => false,
            (DeliveryCost::Low, _) => true,
            (DeliveryCost::Medium, DeliveryCost::Free | DeliveryCost::Low) => false,
            (DeliveryCost::Medium, _) => true,
            (DeliveryCost::High, DeliveryCost::High) => true,
            (DeliveryCost::High, _) => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_delivery_channel_creation() {
        let channel = DeliveryChannel::new(DeliveryChannelType::FcmPush);
        assert!(channel.is_enabled());
        assert_eq!(channel.channel_type(), &DeliveryChannelType::FcmPush);
    }

    #[test]
    fn test_channel_properties() {
        let fcm = DeliveryChannel::new(DeliveryChannelType::FcmPush);
        assert!(fcm.supports_rich_content());
        assert!(fcm.is_realtime());
        assert_eq!(fcm.cost_tier(), DeliveryCost::Free);
        assert!(fcm.requires_opt_in());

        let sms = DeliveryChannel::new(DeliveryChannelType::Sms);
        assert!(!sms.supports_rich_content());
        assert!(sms.is_realtime());
        assert_eq!(sms.cost_tier(), DeliveryCost::High);
    }

    #[test]
    fn test_content_limits() {
        let fcm = DeliveryChannel::new(DeliveryChannelType::FcmPush);
        let limits = fcm.max_content_length().unwrap();
        assert_eq!(limits.title_max, 50);
        assert_eq!(limits.body_max, 150);

        let sms = DeliveryChannel::new(DeliveryChannelType::Sms);
        let sms_limits = sms.max_content_length().unwrap();
        assert_eq!(sms_limits.body_max, 160);
        assert_eq!(sms_limits.title_max, 0);
    }

    #[test]
    fn test_retry_configuration() {
        let email_retry = RetryConfiguration::default_for_channel(&DeliveryChannelType::Email);
        assert_eq!(email_retry.max_attempts, 5);
        assert!(email_retry.should_retry_error("timeout"));
        assert!(!email_retry.should_retry_error("invalid_address"));

        let delay = email_retry.delay_for_attempt(3);
        assert!(delay > email_retry.initial_delay_seconds);
    }

    #[test]
    fn test_multi_channel_config() {
        let channels = vec![
            DeliveryChannel::new(DeliveryChannelType::FcmPush),
            DeliveryChannel::new(DeliveryChannelType::Email),
            DeliveryChannel::disabled(DeliveryChannelType::Sms),
        ];

        let config = MultiChannelConfig::new(channels);
        assert_eq!(config.enabled_channels().len(), 2);
        
        let free_channels = config.channels_by_cost(DeliveryCost::Free);
        assert_eq!(free_channels.len(), 1);
        
        let realtime = config.realtime_channels();
        assert_eq!(realtime.len(), 1); // Only FCM is enabled and realtime
    }
}