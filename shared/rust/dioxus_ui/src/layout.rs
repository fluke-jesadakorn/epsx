//! Layout chrome — navbar, footer, sidebar, dashboard shell, page header.

pub mod breadcrumbs;
pub mod footer;
pub mod navbar;
pub mod page_header;
pub mod shell;
pub mod sidebar;
// === wave2-chrome-track-a === new module (admin header)
pub mod header;
// === wave2-chrome-track-b === new modules (frontend nav cluster)
pub mod mobile_nav;
pub mod nav_actions;
pub mod nav_config;
pub mod navbar_skeleton;
// === wave6b-admin-pages-depth-track-a === new module (AdminShell primitive)
//
// `<AdminShell>` is the shared admin chrome (sidebar + breadcrumb header
// + main content area) used by every Wave 6B admin page. The pages
// consume it via `use crate::layout::admin_shell::AdminShell;`. CSS
// lives in `shared/rust/templates/src/lib.rs` under the
// `// === wave6b-admin-pages-depth-track-a ===` marker region. Track
// B/C/D do not touch this module.
pub mod admin_shell;
// === global session truth === SSR-injected verified/fixture/anonymous state
pub mod session_state;
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

pub use breadcrumbs::*;
pub use footer::*;
pub use navbar::*;
pub use page_header::*;
pub use shell::*;
pub use sidebar::*;
// === wave2-chrome-track-a === re-export (admin header)
pub use header::*;
// === wave2-chrome-track-b === re-exports (frontend nav cluster)
pub use mobile_nav::*;
pub use nav_actions::*;
pub use nav_config::*;
pub use navbar_skeleton::*;
// === wave3a-wiring-track-a ===
// Intentionally NOT re-exporting `main_layout::*` globally — see
// the comment on the `pub mod main_layout;` line above.
// === wave6b-admin-pages-depth-track-a === re-export (AdminShell primitive)
pub use admin_shell::*;
