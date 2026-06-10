//! Admin Payment Management Handlers
//!
//! Comprehensive admin interface for managing payments, subscriptions, and financial operations
//! Requires admin permissions and provides detailed analytics and management capabilities

pub mod types;
pub mod payment_handlers;
pub mod subscription_handlers;
pub mod analytics_handlers;

// Re-export all public handler functions
pub use payment_handlers::{
    admin_list_payments_handler,
    admin_get_payment_details_handler,
    admin_update_payment_status_handler,
    admin_process_refund_handler,
};

pub use subscription_handlers::admin_list_subscriptions_handler;

pub use analytics_handlers::admin_get_payment_analytics_handler;

// Re-export commonly used types
pub use types::{
    AdminPaymentListParams,
    AdminPaymentListResponse,
    AdminPaymentInfo,
    AdminPaymentDetailsResponse,
    AdminSubscriptionListResponse,
    PaymentAnalyticsResponse,
    RefundPaymentRequest,
    UpdatePaymentStatusRequest,
};
