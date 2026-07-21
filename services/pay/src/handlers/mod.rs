//! Handler modules for epsx-pay-svc.
//!
//! wave49(slice-3): top-level dispatch of handler sub-modules.
//! Each submodule owns a coherent endpoint group:
//!
//! - [`intents`]     — `pay_intents` CRUD (5 endpoints)
//! - [`escrows`]     — `escrows` lifecycle (7 endpoints)
//! - [`pay_links`]   — shareable payment URLs (3 endpoints, slice-3 new)
//! - [`pay_history`] — per-address payment history (1 endpoint, slice-3 new)
//! - [`pay_admin`]   — admin force-operations (4 endpoints, slice-3 new)
//! - [`pay_webhooks`]— on-chain event handler (1 endpoint, slice-3 new)

pub mod escrows;
pub mod intents;
pub mod pay_admin;
pub mod pay_history;
pub mod pay_links;
pub mod pay_webhooks;
