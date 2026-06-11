//! Domain-grouped component clusters.
//!
//! Reusable sub-components extracted from the inlined page JSX in
//! `pages/admin_pages/*.rs` to mirror the Next.js source's
//! sub-component tree 1:1.
//!
//! Each cluster (`admin/`, future `user/`) is one folder per source
//! domain (dashboard, analytics, policies, ...) — the per-page page
//! file consumes them via the module path:
//!
//! ```ignore
//! use crate::components::admin::dashboard::*;
//! ```
//!
//! Conventions for clusters:
//! - Each cluster module exposes one or more `#[component]` fns
//!   that mirror the named TSX sub-component in the source
//!   (e.g. `AdminStatsCards`, `AnalyticsHeader`, `PolicyBuilder`).
//! - Typed props (no `String` soup where the source has typed
//!   fields). CSS classes are passed through as `class` / `class_name`
//!   props when the source has a `className` knob.
//! - Section markers (`<section id="...">` or a class name like
//!   `admin-stats-cards`) are preserved on the outer wrapper so the
//!   page's `test_section_markers` test continues to pass without
//!   modification.
//! - Each cluster's `tests` module adds a `test_render_smoke` for
//!   every new sub-component. The parent page's section-marker test
//!   stays as-is.

pub mod admin;

pub use admin::*;
