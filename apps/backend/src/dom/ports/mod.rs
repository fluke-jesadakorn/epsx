// Domain ports module - abstractions for external dependencies
pub mod cache;
pub mod notification;

pub use cache::{DomainCache, DomainCacheError, DomainCacheStats};
pub use notification::{
    NotificationPort, DomainNotification, NotificationRecipient, 
    DomainNotificationType, DomainNotificationPriority, NotificationStatus, NotificationError
};