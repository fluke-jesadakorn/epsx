// Redis-based notification system with database persistence
// Uses Redis pub/sub for real-time delivery + PostgreSQL for offline queue

pub mod sse_handlers;
pub mod redis_broadcaster;
pub mod offline_queue;

pub use sse_handlers::{
    SSENotification,
    NotificationType,
    NotificationPriority,
    sse_notifications_handler,
    sse_health_handler,
};

pub use redis_broadcaster::RedisNotificationBroadcaster;
pub use offline_queue::{
    fetch_queued_notifications,
    mark_as_delivered,
    mark_as_acknowledged,
    cleanup_old_notifications,
    get_notification_stats,
};