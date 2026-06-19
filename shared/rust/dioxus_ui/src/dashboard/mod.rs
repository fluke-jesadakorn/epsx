//! `dashboard` domain subdir — 1 component ported from
//! `apps-old/frontend/components/dashboard/`:
//! - `dashboard_client` (295 LoC)
//!
//! The TS source is a client component that fetches dashboard
//! stats + activity and renders the dashboard view. The Dioxus
//! port renders the same visual structure as a static shell with
//! typed data props that the page-level BFF call sites can fill
//! in from the dashboard service.

use dioxus::prelude::*;

pub mod dashboard_client;

pub use dashboard_client::{DashboardClient, DashboardStats};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dashboard_module_re_exports_resolve() {
        
    }
}
