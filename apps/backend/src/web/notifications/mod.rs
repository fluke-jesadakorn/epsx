// Stateless notification system module
// Uses SSE for real-time delivery + email for persistence

pub mod sse_handlers;

pub use sse_handlers::{
    NotificationBroadcaster, 
    SSENotification, 
    NotificationType, 
    NotificationPriority,
    sse_notifications_handler,
    send_sse_notification_handler,
    broadcast_sse_notification_handler,
    sse_health_handler,
};