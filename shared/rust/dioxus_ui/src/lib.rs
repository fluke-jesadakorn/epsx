//! EPSX Dioxus 0.7 UI component library.
//!
//! 1:1 UX/UI parity with the Next.js frontend (`apps/frontend`) and admin
//! (`apps/admin-frontend`). Every component reuses the existing
//! `epsx_templates::components` CSS strings (Tailwind v2.2.19 CDN classes +
//! the glassmorphism/gradient utilities emitted by `design_system_head`).
//!
//! High-level structure:
//! - [`primitives`] — Button, Card, Badge, Input, StatCard, Tabs, Skeleton,
//!   Icon (matches the existing 33-lucide-name set).
//! - [`layout`] — Navbar, Footer, Sidebar, DashboardShell, PageHeader.
//! - [`feedback`] — Toast, Modal, Spinner, Progress, EmptyState.
//! - [`data`] — Table, Pagination, FilterBar.
//! - [`auth`] — AuthModal, AuthGate, WalletConnectButton, ProgressiveAuthBanner.
//! - [`i18n`] — Minimal EN-only string table (placeholder; matches the
//!   original which has no i18n framework configured).
//! - [`pages`] — Top-level rsx! functions for every page in the Next.js apps.
//!   These are the public surface BFFs use.

pub mod primitives;
pub mod layout;
pub mod feedback;
pub mod data;
pub mod auth;
pub mod chat;
pub mod i18n;
pub mod pages;
pub mod theme;
// === wave6c-1to1-track-b === new module (admin sub-component clusters)
//
// 1:1 component parity for the admin pages: the per-page page file
// in `pages/admin_pages/*.rs` consumes sub-components from
// `components::admin::{dashboard,analytics,policies}::*`. Track C/D
// add their own sub-modules (payments, wallet, settings, media,
// wallets, chat, developer, audit, auth, news, notifications) —
// the integration gate concatenates the `pub mod` lines.
pub mod components;

#[cfg(test)]
mod tests;

pub use primitives::*;
pub use layout::*;
pub use feedback::*;
pub use data::*;
pub use auth::*;
pub use chat::*;
pub use theme::*;
