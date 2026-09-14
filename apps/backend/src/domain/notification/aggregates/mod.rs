pub mod notification;

// Re-export the main aggregate and its types
pub use notification::{
    ChannelDeliveryStatus, DeliveryError, DeliveryResult, DeliveryTracking, Notification,
    NotificationMetadata, NotificationPriority, NotificationStatus,
};
