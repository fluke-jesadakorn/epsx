//! Layout chrome — navbar, footer, sidebar, dashboard shell, page header.

use dioxus::prelude::*;

pub mod navbar;
pub mod footer;
pub mod sidebar;
pub mod shell;
pub mod page_header;
pub mod breadcrumbs;
pub mod header;

pub use navbar::*;
pub use footer::*;
pub use sidebar::*;
pub use shell::*;
pub use page_header::*;
pub use breadcrumbs::*;
pub use header::*;
