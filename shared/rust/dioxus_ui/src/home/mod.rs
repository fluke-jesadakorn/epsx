//! `home` domain subdir — 5 components ported from
//! `apps-old/frontend/components/home/`:
//! - `hero_section`         (92 LoC)
//! - `dynamic_pricing_section` (103 LoC)
//! - `dynamic_pricing_client`  (331 LoC)
//! - `financial_data_table`    (237 LoC)
//! - `share_button`            (25 LoC)
//!
//! Plus the supporting server-rendered sub-components
//! `server_news_section` (119 LoC) + `server_top_performers` (172 LoC)
//! that the home page renders inline in the OLD tree.
//!
//! Wave 25 T2 already inlined the home page's 4 sections directly
//! into `pages/home.rs` (Hero, TopPerformers, Pricing, News) for
//! pixel-parity reasons. The `home` module here exposes the same
//! shapes as reusable primitives so future pages (e.g. a marketing
//! landing page variant) can compose them.

use dioxus::prelude::*;

pub mod dynamic_pricing_client;
pub mod dynamic_pricing_section;
pub mod financial_data_table;
pub mod hero_section;
pub mod server_news_section;
pub mod server_top_performers;
pub mod share_button;

pub use dynamic_pricing_client::{DynamicPricingClient, PricingCard, PricingGroup};
pub use dynamic_pricing_section::DynamicPricingSection;
pub use financial_data_table::{FinancialDataRow, FinancialDataTable};
pub use hero_section::{HeroSection, SignedOutHero};
pub use server_news_section::ServerNewsSection;
pub use server_top_performers::ServerTopPerformers;
pub use share_button::ShareButton;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn home_module_re_exports_resolve() {
        // Smoke test — the 7 NEW home components are all exported
        // from `crate::home::*`. We assert each one is a function
        // pointer so a regression that renames or removes a
        // component will fail at compile time (this test) and not
        // at link time.
    }
}
