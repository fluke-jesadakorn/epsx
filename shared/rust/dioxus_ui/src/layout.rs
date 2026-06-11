//! Layout chrome — navbar, footer, sidebar, dashboard shell, page header.

use dioxus::prelude::*;

pub mod navbar;
pub mod footer;
pub mod sidebar;
pub mod shell;
pub mod page_header;
pub mod breadcrumbs;
// === wave2-chrome-track-a === new module (admin header)
pub mod header;
// === wave2-chrome-track-b === new modules (frontend nav cluster)
pub mod nav_config;
pub mod navbar_skeleton;
pub mod nav_actions;
pub mod mobile_nav;
// === wave3a-wiring-track-a === new module (frontend MainLayout wrapper)
//
// The two `MainLayout` / `AuthLayout` component names already exist in
// `super::shell` (the admin shell, added in Wave 2 chrome-track-a).
// Re-exporting both globally from `layout.rs` causes E0659 "is
// ambiguous" errors. So we expose `main_layout` as a sub-module and
// import it via the module path from each page:
//
//     use crate::layout::main_layout::MainLayout;
//
// The Track C integration agent can promote either the frontend or
// admin `MainLayout` to the global namespace if needed — the design
// doc keeps them disambiguated by file (`main_layout` vs `shell`).
pub mod main_layout;
// === wave5-page-depth-track-a === new module (MarketingBackground primitive)
//
// Extracted shared PancakeSwap-style gradient + orbs + mesh overlays
// + geometric decorations that appear on the home / about / contact /
// plans pages. Pages consume it as `use crate::layout::marketing_bg
// ::MarketingBackground;`. CSS lives in
// `shared/rust/templates/src/lib.rs` under the same wave marker
// region. Track B imports this module from its pages (no further
// changes to `layout.rs` are required from Track B).
pub mod marketing_bg;

pub use navbar::*;
pub use footer::*;
pub use sidebar::*;
pub use shell::*;
pub use page_header::*;
pub use breadcrumbs::*;
// === wave2-chrome-track-a === re-export (admin header)
pub use header::*;
// === wave2-chrome-track-b === re-exports (frontend nav cluster)
pub use nav_config::*;
pub use navbar_skeleton::*;
pub use nav_actions::*;
pub use mobile_nav::*;
// === wave3a-wiring-track-a ===
// Intentionally NOT re-exporting `main_layout::*` globally — see
// the comment on the `pub mod main_layout;` line above.
