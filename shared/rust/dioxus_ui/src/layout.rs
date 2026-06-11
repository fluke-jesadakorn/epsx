//! Layout chrome — navbar, footer, sidebar, dashboard shell, page header.

use dioxus::prelude::*;

pub mod navbar;
pub mod footer;
pub mod sidebar;
pub mod shell;
pub mod page_header;
pub mod breadcrumbs;
// === wave2-chrome-track-b === new modules (frontend nav cluster)
pub mod nav_config;
pub mod navbar_skeleton;
pub mod nav_actions;
pub mod mobile_nav;

pub use navbar::*;
pub use footer::*;
pub use sidebar::*;
pub use shell::*;
pub use page_header::*;
pub use breadcrumbs::*;
// === wave2-chrome-track-b === re-exports (frontend nav cluster)
pub use nav_config::*;
pub use navbar_skeleton::*;
pub use nav_actions::*;
pub use mobile_nav::*;
