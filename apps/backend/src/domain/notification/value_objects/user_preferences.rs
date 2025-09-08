use std::fmt::{self, Display};
use std::collections::HashSet;
use chrono::{NaiveTime, Utc, DateTime};
use serde::{Deserialize, Serialize};
use crate::infrastructure::adapters::repositories::diesel::types::{NotificationType, DeliveryChannel as DeliveryChannelType};

/// User Notification Preferences Value Object
/// Encapsulates user's notification preferences, quiet hours, and channel settings
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UserNotificationPreferences {
    user_id: uuid::Uuid,
    channel_settings: ChannelSettings,
    content_preferences: ContentPreferences,
    quiet_hours: Option<QuietHours>,
    timezone: String,
    blocked_topics: HashSet<String>,
    frequency_limits: FrequencyLimits,
    last_updated: DateTime<Utc>,
}

impl UserNotificationPreferences {
    /// Create new user preferences with defaults
    pub fn new(user_id: uuid::Uuid) -> Self {
        Self {
            user_id,
            channel_settings: ChannelSettings::default(),
            content_preferences: ContentPreferences::default(),
            quiet_hours: None,
            timezone: "UTC".to_string(),
            blocked_topics: HashSet::new(),
            frequency_limits: FrequencyLimits::default(),
            last_updated: Utc::now(),
        }
    }

    /// Create preferences with timezone
    pub fn with_timezone(user_id: uuid::Uuid, timezone: String) -> Result<Self, String> {
        // Basic timezone validation
        if timezone.is_empty() {
            return Err("Timezone cannot be empty".to_string());
        }

        let mut prefs = Self::new(user_id);
        prefs.timezone = timezone;
        Ok(prefs)
    }

    /// Get user ID
    pub fn user_id(&self) -> uuid::Uuid {
        self.user_id
    }

    /// Get channel settings
    pub fn channel_settings(&self) -> &ChannelSettings {
        &self.channel_settings
    }

    /// Get mutable channel settings
    pub fn channel_settings_mut(&mut self) -> &mut ChannelSettings {
        self.touch();
        &mut self.channel_settings
    }

    /// Get content preferences
    pub fn content_preferences(&self) -> &ContentPreferences {
        &self.content_preferences
    }

    /// Get mutable content preferences
    pub fn content_preferences_mut(&mut self) -> &mut ContentPreferences {
        self.touch();
        &mut self.content_preferences
    }

    /// Get quiet hours
    pub fn quiet_hours(&self) -> Option<&QuietHours> {
        self.quiet_hours.as_ref()
    }

    /// Set quiet hours
    pub fn set_quiet_hours(&mut self, quiet_hours: QuietHours) -> Result<(), String> {
        quiet_hours.validate()?;
        self.quiet_hours = Some(quiet_hours);
        self.touch();
        Ok(())
    }

    /// Remove quiet hours
    pub fn remove_quiet_hours(&mut self) {
        self.quiet_hours = None;
        self.touch();
    }

    /// Get timezone
    pub fn timezone(&self) -> &str {
        &self.timezone
    }

    /// Set timezone
    pub fn set_timezone(&mut self, timezone: String) -> Result<(), String> {
        if timezone.is_empty() {
            return Err("Timezone cannot be empty".to_string());
        }
        self.timezone = timezone;
        self.touch();
        Ok(())
    }

    /// Get blocked topics
    pub fn blocked_topics(&self) -> &HashSet<String> {
        &self.blocked_topics
    }

    /// Block a topic
    pub fn block_topic(&mut self, topic: String) {
        self.blocked_topics.insert(topic);
        self.touch();
    }

    /// Unblock a topic
    pub fn unblock_topic(&mut self, topic: &str) {
        self.blocked_topics.remove(topic);
        self.touch();
    }

    /// Check if topic is blocked
    pub fn is_topic_blocked(&self, topic: &str) -> bool {
        self.blocked_topics.contains(topic)
    }

    /// Get frequency limits
    pub fn frequency_limits(&self) -> &FrequencyLimits {
        &self.frequency_limits
    }

    /// Get mutable frequency limits
    pub fn frequency_limits_mut(&mut self) -> &mut FrequencyLimits {
        self.touch();
        &mut self.frequency_limits
    }

    /// Check if user should receive notification based on preferences
    pub fn should_receive_notification(
        &self,
        notification_type: &NotificationType,
        channel: &DeliveryChannelType,
        topic: Option<&str>,
        current_time: DateTime<Utc>,
    ) -> bool {
        // Check if channel is enabled
        if !self.channel_settings.is_channel_enabled(channel) {
            return false;
        }

        // Check if notification type is allowed
        if !self.content_preferences.allows_notification_type(notification_type) {
            return false;
        }

        // Check if topic is blocked
        if let Some(topic_name) = topic {
            if self.is_topic_blocked(topic_name) {
                return false;
            }
        }

        // Check quiet hours
        if let Some(quiet_hours) = &self.quiet_hours {
            if quiet_hours.is_in_quiet_period(current_time, &self.timezone) {
                // Allow urgent notifications during quiet hours
                match notification_type {
                    NotificationType::System | NotificationType::Security => true,
                    _ => false,
                }
            } else {
                true
            }
        } else {
            true
        }
    }

    /// Check if frequency limits allow sending notification
    pub fn can_send_notification(
        &self,
        channel: &DeliveryChannelType,
        recent_count: u32,
        window_hours: u32,
    ) -> bool {
        self.frequency_limits.allows_notification(channel, recent_count, window_hours)
    }

    /// Get last updated timestamp
    pub fn last_updated(&self) -> DateTime<Utc> {
        self.last_updated
    }

    /// Update the last modified timestamp
    fn touch(&mut self) {
        self.last_updated = Utc::now();
    }

    /// Get preference summary for analytics
    pub fn summary(&self) -> PreferenceSummary {
        PreferenceSummary {
            enabled_channels: self.channel_settings.enabled_channel_count(),
            allowed_types: self.content_preferences.allowed_type_count(),
            has_quiet_hours: self.quiet_hours.is_some(),
            blocked_topic_count: self.blocked_topics.len(),
            timezone: self.timezone.clone(),
        }
    }
}

/// Channel-specific settings
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ChannelSettings {
    fcm_push_enabled: bool,
    in_app_enabled: bool,
    email_enabled: bool,
    sms_enabled: bool,
}

impl ChannelSettings {
    /// Check if a specific channel is enabled
    pub fn is_channel_enabled(&self, channel: &DeliveryChannelType) -> bool {
        match channel {
            DeliveryChannelType::FcmPush => self.fcm_push_enabled,
            DeliveryChannelType::Push => self.fcm_push_enabled, // Map to FCM push
            DeliveryChannelType::InApp => self.in_app_enabled,
            DeliveryChannelType::Email => self.email_enabled,
            DeliveryChannelType::Sms => self.sms_enabled,
            DeliveryChannelType::SMS => self.sms_enabled, // Map to SMS
        }
    }

    /// Enable a channel
    pub fn enable_channel(&mut self, channel: &DeliveryChannelType) {
        match channel {
            DeliveryChannelType::FcmPush => self.fcm_push_enabled = true,
            DeliveryChannelType::Push => self.fcm_push_enabled = true, // Map to FCM push
            DeliveryChannelType::InApp => self.in_app_enabled = true,
            DeliveryChannelType::Email => self.email_enabled = true,
            DeliveryChannelType::Sms => self.sms_enabled = true,
            DeliveryChannelType::SMS => self.sms_enabled = true, // Map to SMS
        }
    }

    /// Disable a channel
    pub fn disable_channel(&mut self, channel: &DeliveryChannelType) {
        match channel {
            DeliveryChannelType::FcmPush => self.fcm_push_enabled = false,
            DeliveryChannelType::Push => self.fcm_push_enabled = false, // Map to FCM push
            DeliveryChannelType::InApp => self.in_app_enabled = false,
            DeliveryChannelType::Email => self.email_enabled = false,
            DeliveryChannelType::Sms => self.sms_enabled = false,
            DeliveryChannelType::SMS => self.sms_enabled = false, // Map to SMS
        }
    }

    /// Get count of enabled channels
    pub fn enabled_channel_count(&self) -> u8 {
        let mut count = 0;
        if self.fcm_push_enabled { count += 1; }
        if self.in_app_enabled { count += 1; }
        if self.email_enabled { count += 1; }
        if self.sms_enabled { count += 1; }
        count
    }

    /// Check if any channels are enabled
    pub fn has_any_enabled(&self) -> bool {
        self.fcm_push_enabled || self.in_app_enabled || self.email_enabled || self.sms_enabled
    }
}

impl Default for ChannelSettings {
    fn default() -> Self {
        Self {
            fcm_push_enabled: true,  // Default enabled
            in_app_enabled: true,    // Default enabled
            email_enabled: false,    // Default disabled (requires explicit opt-in)
            sms_enabled: false,      // Default disabled (requires explicit opt-in)
        }
    }
}

/// Content preferences for notification types
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ContentPreferences {
    system_notifications: bool,
    admin_notifications: bool,
    security_notifications: bool,
    feature_notifications: bool,
    marketing_notifications: bool,
}

impl ContentPreferences {
    /// Check if notification type is allowed
    pub fn allows_notification_type(&self, notification_type: &NotificationType) -> bool {
        match notification_type {
            NotificationType::System => self.system_notifications,
            NotificationType::Admin => self.admin_notifications,
            NotificationType::Security => self.security_notifications,
            NotificationType::Feature => self.feature_notifications,
            NotificationType::Marketing => self.marketing_notifications,
            NotificationType::Info => self.system_notifications,
            NotificationType::Warning => self.system_notifications,
            NotificationType::Error => self.system_notifications,
            NotificationType::Success => self.system_notifications,
            NotificationType::General => self.system_notifications,
        }
    }

    /// Enable notification type
    pub fn enable_type(&mut self, notification_type: &NotificationType) {
        match notification_type {
            NotificationType::System => self.system_notifications = true,
            NotificationType::Admin => self.admin_notifications = true,
            NotificationType::Security => self.security_notifications = true,
            NotificationType::Feature => self.feature_notifications = true,
            NotificationType::Marketing => self.marketing_notifications = true,
            NotificationType::Info => self.system_notifications = true,
            NotificationType::Warning => self.system_notifications = true,
            NotificationType::Error => self.system_notifications = true,
            NotificationType::Success => self.system_notifications = true,
            NotificationType::General => self.system_notifications = true,
        }
    }

    /// Disable notification type
    pub fn disable_type(&mut self, notification_type: &NotificationType) {
        match notification_type {
            NotificationType::System => self.system_notifications = false,
            NotificationType::Admin => self.admin_notifications = false,
            NotificationType::Security => self.security_notifications = false,
            NotificationType::Feature => self.feature_notifications = false,
            NotificationType::Marketing => self.marketing_notifications = false,
            NotificationType::Info => self.system_notifications = false,
            NotificationType::Warning => self.system_notifications = false,
            NotificationType::Error => self.system_notifications = false,
            NotificationType::Success => self.system_notifications = false,
            NotificationType::General => self.system_notifications = false,
        }
    }

    /// Get count of allowed types
    pub fn allowed_type_count(&self) -> u8 {
        let mut count = 0;
        if self.system_notifications { count += 1; }
        if self.admin_notifications { count += 1; }
        if self.security_notifications { count += 1; }
        if self.feature_notifications { count += 1; }
        if self.marketing_notifications { count += 1; }
        count
    }
}

impl Default for ContentPreferences {
    fn default() -> Self {
        Self {
            system_notifications: true,   // Always enabled by default
            admin_notifications: true,    // Enabled for admins
            security_notifications: true, // Always enabled for security
            feature_notifications: true,  // Default enabled
            marketing_notifications: false, // Default disabled (requires opt-in)
        }
    }
}

/// Quiet hours configuration
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct QuietHours {
    start_time: NaiveTime,
    end_time: NaiveTime,
    enabled: bool,
}

impl QuietHours {
    /// Create new quiet hours
    pub fn new(start_time: NaiveTime, end_time: NaiveTime) -> Result<Self, String> {
        let quiet_hours = Self {
            start_time,
            end_time,
            enabled: true,
        };
        quiet_hours.validate()?;
        Ok(quiet_hours)
    }

    /// Get start time
    pub fn start_time(&self) -> NaiveTime {
        self.start_time
    }

    /// Get end time  
    pub fn end_time(&self) -> NaiveTime {
        self.end_time
    }

    /// Check if quiet hours are enabled
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// Enable quiet hours
    pub fn enable(&mut self) {
        self.enabled = true;
    }

    /// Disable quiet hours
    pub fn disable(&mut self) {
        self.enabled = false;
    }

    /// Check if given time is in quiet period
    pub fn is_in_quiet_period(&self, datetime: DateTime<Utc>, timezone: &str) -> bool {
        if !self.enabled {
            return false;
        }

        // For simplicity, convert to naive time (in practice, you'd use proper timezone handling)
        let time = datetime.time();

        if self.start_time <= self.end_time {
            // Same day quiet hours (e.g., 22:00 to 07:00)
            time >= self.start_time && time <= self.end_time
        } else {
            // Overnight quiet hours (e.g., 23:00 to 06:00)
            time >= self.start_time || time <= self.end_time
        }
    }

    /// Validate quiet hours configuration
    fn validate(&self) -> Result<(), String> {
        if self.start_time == self.end_time {
            return Err("Quiet hours start and end time cannot be the same".to_string());
        }
        Ok(())
    }

    /// Get duration of quiet hours
    pub fn duration_hours(&self) -> f32 {
        if self.start_time <= self.end_time {
            // Same day
            let duration = self.end_time.signed_duration_since(self.start_time);
            duration.num_minutes() as f32 / 60.0
        } else {
            // Overnight
            let to_midnight = NaiveTime::from_hms_opt(23, 59, 59).unwrap()
                .signed_duration_since(self.start_time);
            let from_midnight = self.end_time
                .signed_duration_since(NaiveTime::from_hms_opt(0, 0, 0).unwrap());
            (to_midnight.num_minutes() + from_midnight.num_minutes()) as f32 / 60.0
        }
    }
}

/// Frequency limits for notifications
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FrequencyLimits {
    max_per_hour: u32,
    max_per_day: u32,
    max_push_per_hour: u32,
    max_email_per_day: u32,
    max_sms_per_day: u32,
}

impl FrequencyLimits {
    /// Check if notification is allowed based on recent count
    pub fn allows_notification(
        &self,
        channel: &DeliveryChannelType,
        recent_count: u32,
        window_hours: u32,
    ) -> bool {
        let limit = match (channel, window_hours) {
            (DeliveryChannelType::FcmPush, 1) => self.max_push_per_hour,
            (DeliveryChannelType::Email, 24) => self.max_email_per_day,
            (DeliveryChannelType::Sms, 24) => self.max_sms_per_day,
            (_, 1) => self.max_per_hour,
            (_, 24) => self.max_per_day,
            _ => return true, // No limit for other windows
        };

        recent_count < limit
    }
}

impl Default for FrequencyLimits {
    fn default() -> Self {
        Self {
            max_per_hour: 10,        // Max 10 notifications per hour
            max_per_day: 50,         // Max 50 notifications per day
            max_push_per_hour: 5,    // Max 5 push notifications per hour
            max_email_per_day: 10,   // Max 10 emails per day
            max_sms_per_day: 3,      // Max 3 SMS per day
        }
    }
}

/// Summary of user preferences for analytics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PreferenceSummary {
    pub enabled_channels: u8,
    pub allowed_types: u8,
    pub has_quiet_hours: bool,
    pub blocked_topic_count: usize,
    pub timezone: String,
}

impl Display for UserNotificationPreferences {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let summary = self.summary();
        write!(f, "User {} preferences: {} channels, {} types, {} blocked topics",
               self.user_id,
               summary.enabled_channels,
               summary.allowed_types,
               summary.blocked_topic_count)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;
    use uuid::Uuid;

    #[test]
    fn test_new_preferences() {
        let user_id = Uuid::new_v4();
        let prefs = UserNotificationPreferences::new(user_id);
        
        assert_eq!(prefs.user_id(), user_id);
        assert_eq!(prefs.timezone(), "UTC");
        assert!(prefs.channel_settings().has_any_enabled());
    }

    #[test]
    fn test_channel_settings() {
        let user_id = Uuid::new_v4();
        let mut prefs = UserNotificationPreferences::new(user_id);
        
        assert!(prefs.channel_settings().is_channel_enabled(&DeliveryChannelType::FcmPush));
        assert!(prefs.channel_settings().is_channel_enabled(&DeliveryChannelType::InApp));
        
        prefs.channel_settings_mut().disable_channel(&DeliveryChannelType::FcmPush);
        assert!(!prefs.channel_settings().is_channel_enabled(&DeliveryChannelType::FcmPush));
    }

    #[test]
    fn test_content_preferences() {
        let user_id = Uuid::new_v4();
        let mut prefs = UserNotificationPreferences::new(user_id);
        
        assert!(prefs.content_preferences().allows_notification_type(&NotificationType::System));
        assert!(!prefs.content_preferences().allows_notification_type(&NotificationType::Marketing));
        
        prefs.content_preferences_mut().enable_type(&NotificationType::Marketing);
        assert!(prefs.content_preferences().allows_notification_type(&NotificationType::Marketing));
    }

    #[test]
    fn test_quiet_hours() {
        let start = NaiveTime::from_hms_opt(22, 0, 0).unwrap();
        let end = NaiveTime::from_hms_opt(7, 0, 0).unwrap();
        let quiet_hours = QuietHours::new(start, end).unwrap();
        
        // Test overnight quiet hours
        let night_time = Utc.with_ymd_and_hms(2024, 1, 1, 23, 30, 0).unwrap();
        let morning_time = Utc.with_ymd_and_hms(2024, 1, 1, 6, 30, 0).unwrap();
        let day_time = Utc.with_ymd_and_hms(2024, 1, 1, 12, 0, 0).unwrap();
        
        assert!(quiet_hours.is_in_quiet_period(night_time, "UTC"));
        assert!(quiet_hours.is_in_quiet_period(morning_time, "UTC"));
        assert!(!quiet_hours.is_in_quiet_period(day_time, "UTC"));
    }

    #[test]
    fn test_topic_blocking() {
        let user_id = Uuid::new_v4();
        let mut prefs = UserNotificationPreferences::new(user_id);
        
        assert!(!prefs.is_topic_blocked("marketing"));
        
        prefs.block_topic("marketing".to_string());
        assert!(prefs.is_topic_blocked("marketing"));
        
        prefs.unblock_topic("marketing");
        assert!(!prefs.is_topic_blocked("marketing"));
    }

    #[test]
    fn test_should_receive_notification() {
        let user_id = Uuid::new_v4();
        let mut prefs = UserNotificationPreferences::new(user_id);
        
        // Block marketing notifications
        prefs.content_preferences_mut().disable_type(&NotificationType::Marketing);
        
        let current_time = Utc::now();
        
        // Should not receive marketing notifications
        assert!(!prefs.should_receive_notification(
            &NotificationType::Marketing,
            &DeliveryChannelType::FcmPush,
            None,
            current_time
        ));
        
        // Should receive system notifications
        assert!(prefs.should_receive_notification(
            &NotificationType::System,
            &DeliveryChannelType::FcmPush,
            None,
            current_time
        ));
    }

    #[test]
    fn test_frequency_limits() {
        let limits = FrequencyLimits::default();
        
        // Should allow under limit
        assert!(limits.allows_notification(&DeliveryChannelType::FcmPush, 3, 1));
        
        // Should block over limit
        assert!(!limits.allows_notification(&DeliveryChannelType::FcmPush, 10, 1));
    }

    #[test]
    fn test_quiet_hours_duration() {
        let start = NaiveTime::from_hms_opt(22, 0, 0).unwrap();
        let end = NaiveTime::from_hms_opt(7, 0, 0).unwrap();
        let quiet_hours = QuietHours::new(start, end).unwrap();
        
        let duration = quiet_hours.duration_hours();
        assert!((duration - 9.0).abs() < 0.1); // 22:00 to 07:00 = 9 hours
    }
}