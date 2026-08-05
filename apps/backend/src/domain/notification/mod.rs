pub mod aggregates;
pub mod events;
pub mod repository_ports;
/// Web3-first Notification Bounded Context
///
/// This bounded context handles all aspects of notification delivery, user preferences,
/// scheduling, and wallet-based communication for the EPSX platform using serverless architecture.
///
/// ## Core Concepts
///
/// - **Notification**: Main aggregate managing notification lifecycle from creation to delivery
/// - **NotificationTopic**: Wallet-based topics for broadcasting notifications to user plans
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

// Public exports from value objects
pub use value_objects::{
    AccessLevel, ChannelSettings, ContentLimits, ContentPreferences, ContentUrgency,
    DeliveryChannel, DeliveryChannelType, DeliveryCost, DeliveryWindow, FrequencyLimits,
    MultiChannelConfig, NotificationContent, NotificationId, NotificationTopic, NotificationType,
    PreferenceSummary, PrivacyLevel, QuietHours, RetryConfiguration, ScheduleInfo, ScheduleStatus,
    ScheduleType, SubscriberScale, TopicCategory, UserNotificationPreferences,
};

// Public exports from aggregates
pub use aggregates::{
    ChannelDeliveryStatus, DeliveryError, DeliveryResult, DeliveryTracking, Notification,
    NotificationMetadata, NotificationPriority, NotificationStatus,
};

// Public exports from events
pub use events::notification_events::{
    NotificationCancelled, NotificationCreated, NotificationDeliveryCompleted, NotificationExpired,
    NotificationPriorityUpdated, NotificationScheduled, NotificationSending,
};

// Public exports from repository ports
pub use repository_ports::{NotificationRepositoryPort, NotificationSearchCriteria};
