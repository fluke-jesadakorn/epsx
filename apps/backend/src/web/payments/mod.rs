//! Payment Web Handlers
//!
//! This module provides comprehensive payment validation and management API endpoints
//! using the existing PaymentVerifier for blockchain transaction validation

pub mod admin;
pub mod validation_handlers;
pub mod subscription_handlers;
pub mod admin_handlers;
pub mod user_payment_handlers;
pub mod submit_tx_handler;
pub mod get_tx_status_handler;
pub mod upgrade_service;
pub mod credit_handlers;

// Re-export handler functions for router integration
pub use validation_handlers::{
    validate_payment_handler,
    activate_subscription_handler,
    get_payment_details_handler,

};
pub use subscription_handlers::{
    get_user_plans_handler,
    get_plan_expiry_status_handler,
    cancel_plan_handler,
    get_upgrade_preview_handler,
    execute_plan_switch_handler,
};
pub use admin_handlers::{
    admin_list_payments_handler,
    admin_get_payment_details_handler,
    admin_update_payment_status_handler,
    admin_process_refund_handler,
    admin_list_subscriptions_handler,
    admin_get_payment_analytics_handler,
};
pub use user_payment_handlers::{
    get_user_payment_history,
};
pub use submit_tx_handler::{
    submit_transaction_handler,
};
pub use get_tx_status_handler::{
    get_transaction_status_handler,
};
pub use credit_handlers::{
    get_credit_balance,
    get_credit_history,
    admin_get_user_credits,
    admin_grant_credits,
    admin_revoke_credits,
    admin_get_credit_stats,
};