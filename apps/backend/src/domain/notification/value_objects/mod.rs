pub mod delivery_channel;
pub mod notification_content;
pub mod notification_id;
pub mod notification_topic;
pub mod schedule_info;
pub mod user_preferences;

// Re-export all value objects for easier import
pub use delivery_channel::{
    ContentLimits, DeliveryChannel, DeliveryChannelConfig, DeliveryChannelType, DeliveryCost,
    MultiChannelConfig, PrivacyLevel, RetryConfiguration,
};
pub use notification_content::{ContentUrgency, NotificationContent};
pub use notification_id::NotificationId;
pub use notification_topic::{AccessLevel, NotificationTopic, SubscriberScale, TopicCategory};
pub use schedule_info::{DeliveryWindow, ScheduleInfo, ScheduleStatus, ScheduleType};
pub use user_preferences::{
    ChannelSettings, ContentPreferences, FrequencyLimits, NotificationType, PreferenceSummary,
    QuietHours, UserNotificationPreferences,
};
