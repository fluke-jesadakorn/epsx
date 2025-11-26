pub mod notification;

// Re-export the main aggregate and its types
pub use notification::{
    Notification, NotificationMetadata, DeliveryTracking, ChannelDeliveryStatus,
    DeliveryError, DeliveryResult, NotificationStatus, NotificationPriority
};