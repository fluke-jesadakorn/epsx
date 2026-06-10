// Notification handlers - unified API
// Split into focused modules for better maintainability

mod notification_types;
mod notification_admin;
mod notification_user;

// Re-export types
pub use notification_types::*;

// Re-export admin handlers
pub use notification_admin::{
    send_notification_handler,
    get_all_notifications_handler,
    get_notification_stats_handler,
    delete_admin_notification_handler,
};

// Re-export user handlers
pub use notification_user::{
    get_user_notifications_handler,
    mark_notification_read_handler,
    mark_notification_unread_handler,
    delete_notification_handler,
    get_unread_count_handler,
    mark_all_notifications_read_handler,
    clear_all_notifications_handler,
    acknowledge_notification_handler,
};
