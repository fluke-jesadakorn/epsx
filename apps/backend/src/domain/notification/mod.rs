/// Web3-first Notification Bounded Context
/// 
/// This bounded context handles all aspects of notification delivery, user preferences,
/// scheduling, and wallet-based communication for the EPSX platform using serverless architecture.
/// 
/// ## Core Concepts
/// 
/// - **Notification**: Main aggregate managing notification lifecycle from creation to delivery
/// - **NotificationTopic**: Wallet-based topics for broadcasting notifications to user groups
/// - **UserNotificationPreferences**: User preferences, quiet hours, and channel settings
/// - **DeliveryChannel**: Multi-channel delivery with retry logic and content adaptation
/// - **ScheduleInfo**: Scheduling, expiry, and timing management for notifications
/// 
/// ## Supported Channels (Web3-first)
/// 
/// - **Wallet Notifications**: Primary notification channel for connected wallets
/// - **Web Push**: Browser-native push notifications via Web Push API
/// - **In-App**: Database-stored notifications for wallet-based app display
/// - **WebSocket**: Real-time notifications for active wallet connections
/// 
/// ## Domain Events
/// 
/// The context publishes events for notification lifecycle, delivery status,
/// user preference changes, and topic subscription management
/// 
/// ## Integration
/// 
/// This bounded context integrates with:
/// - User Management (for wallet identification and permissions)
/// - Trading Analytics (for market alert notifications)
/// - Web3 Services (for wallet-based delivery and blockchain events)

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