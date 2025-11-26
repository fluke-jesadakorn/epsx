pub mod notification_id;
pub mod notification_content;
pub mod delivery_channel;
pub mod notification_topic;
pub mod schedule_info;
pub mod user_preferences;

// Re-export all value objects for easier import
pub use notification_id::NotificationId;
pub use notification_content::{ NotificationContent, ContentUrgency };
pub use delivery_channel::{
  DeliveryChannel,
  DeliveryChannelType,
  DeliveryChannelConfig,
  MultiChannelConfig,
  RetryConfiguration,
  ContentLimits,
  DeliveryCost,
  PrivacyLevel,
};
pub use notification_topic::{
  NotificationTopic,
  TopicCategory,
  AccessLevel,
  SubscriberScale,
};
pub use schedule_info::{
  ScheduleInfo,
  ScheduleType,
  ScheduleStatus,
  DeliveryWindow,
};
pub use user_preferences::{
  UserNotificationPreferences,
  ChannelSettings,
  ContentPreferences,
  QuietHours,
  FrequencyLimits,
  PreferenceSummary,
  NotificationType,
};
