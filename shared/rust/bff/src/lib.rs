//! Shared BFF utilities: security middleware, session cookies, JWT
//! verification, and common route builders. Used by both `apps/frontend`
//! and `apps/admin`.
//!
//! [`session`] is the canonical monolith authentication contract. The older
//! [`auth_helpers`] API remains available while callers migrate away from the
//! legacy shared-secret token format.

pub mod auth_helpers;
pub mod browser_auth;
pub mod cookies;
pub mod dev_bypass;
pub mod middleware;
pub mod session;
pub mod static_assets;
