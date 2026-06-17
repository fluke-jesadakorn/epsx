//! Notification port adapters.
//!
//! Wave 10 service-boundary refactor. The current implementation is
//! the in-process adapter (`in_process_adapter`); a future
//! `http_adapter` is added in the integration gate when the
//! `epsx-notifications` service binary is wired up.

pub mod in_process_adapter;

pub use in_process_adapter::InProcessNotificationAdapter;
