//! Admin payment module
//!
//! Contains DTOs for admin payment APIs.
//!
//! wave11(track-b): the admin subscription handlers
//! (`list_subscriptions_admin_handler`) live here. Pre-wave-11
//! the read handler lived in
//! `web/admin/plans/handlers.rs::list_subscriptions_handler` and
//! reached into `SubscriptionDb` directly. Track B moves the
//! payments-only read into the payments admin area and routes it
//! through `Arc<dyn SubscriptionRepositoryPort>`. The
//! `create_subscription_handler` stays in
//! `web/admin/plans/handlers.rs` for now (it does a
//! primary-DB `wallet_plan_assignments` UPSERT in the same
//! function) and is a wave-12+ follow-up.
//! See `docs/wave8-service-boundary/ROADMAP.md` §4 wave-11
//! preconditions item 3.

pub mod dtos;
pub mod subscription_admin_handlers;

pub use dtos::*;
pub use subscription_admin_handlers::list_subscriptions_admin_handler;
