/// Notification Bounded Context
/// 
/// This bounded context handles all aspects of notification delivery, user preferences,
/// scheduling, and multi-channel communication for the EPSX platform.
/// 
/// ## Core Concepts
/// 
/// - **Notification**: Main aggregate managing notification lifecycle from creation to delivery
/// - **NotificationTopic**: FCM topics for broadcasting notifications to user groups
/// - **UserNotificationPreferences**: User preferences, quiet hours, and channel settings
/// - **DeliveryChannel**: Multi-channel delivery with retry logic and content adaptation
/// - **ScheduleInfo**: Scheduling, expiry, and timing management for notifications
/// 
/// ## Supported Channels
/// 
/// - **FCM Push**: Real-time push notifications with rich content support
/// - **In-App**: Instant in-app notifications with no content limits
/// - **Email**: Rich HTML emails with attachment support
/// - **SMS**: Text messages with character limits and high delivery cost
/// 
/// ## Domain Events
/// 
/// The context publishes events for notification lifecycle, delivery status,
/// user preference changes, and topic subscription management
/// 
/// ## Integration
/// 
/// This bounded context integrates with:
/// - User Management (for user identification and permissions)
/// - Trading Analytics (for market alert notifications)
/// - External services (FCM, email providers, SMS gateways)

pub mod value_objects;
pub mod aggregates;

// Public exports from value objects
pub use value_objects::{
    NotificationId, NotificationContent, ContentUrgency,
    DeliveryChannel, DeliveryChannelType, MultiChannelConfig, RetryConfiguration,
    ContentLimits, DeliveryCost, PrivacyLevel,
    NotificationTopic, TopicCategory, AccessLevel, SubscriberScale,
    ScheduleInfo, ScheduleType, ScheduleStatus, DeliveryWindow,
    UserNotificationPreferences, ChannelSettings, ContentPreferences,
    QuietHours, FrequencyLimits, PreferenceSummary
};

// Public exports from aggregates
pub use aggregates::{
    Notification, NotificationMetadata, DeliveryTracking, ChannelDeliveryStatus,
    DeliveryError, DeliveryResult, NotificationStatus,
    NotificationCreated, NotificationScheduled, NotificationSending,
    NotificationDeliveryCompleted, NotificationExpired, NotificationPriorityUpdated,
    NotificationCancelled
};