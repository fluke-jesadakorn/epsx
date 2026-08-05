// Redis-based notification system with database persistence
// Uses Redis pub/sub for real-time delivery + PostgreSQL for offline queue

pub mod offline_queue;
pub mod sse_handlers;

#[cfg(test)]
mod tests;

pub use sse_handlers::{
    sse_notifications_handler, NotificationPriority, NotificationType, SSENotification,
};

pub use offline_queue::{
    cleanup_old_notifications, fetch_queued_notifications, get_notification_stats,
    mark_as_acknowledged, mark_as_delivered,
};
