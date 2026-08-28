//! Dioxus fullstack entry for `dx serve --hot-reload`.
//!
//! Big-Bang Phase 2: This binary is the HMR dev surface (<500ms).
//! Production stays on `bff-frontend` (Axum + dioxus_ssr) until Phase 2
//! server_fn migration completes. Both share `epsx-dioxus-ui` + `templates`.

use dioxus::prelude::*;
use epsx_dioxus_ui::app::FrontendApp;

#[component]
fn App() -> Element {
    rsx! { FrontendApp {} }
}

fn main() {
    // `FrontendApp` with `initial_context: None` falls back to `PageContext::default()`
    // via `try_consume_context` in `routes.rs:get_ctx`. Hot reload patches `rsx!` without
    // `cargo run` restart — edit `stock_data_card.rs:225` `Next Action` hero and see <500ms.
    dioxus::launch(App);
}

// Example server function for pilot migration (home analytics).
// This will replace `ssr.rs:fetch_page_data` `load_home_analytics` HashMap insert.
//
//  #[server(GetHomeRankings)]
//  pub async fn get_home_rankings() -> Result<epsx_dioxus_ui::pages::analytics::AnalyticsResponse, ServerFnError> {
//      // Server-only: ServiceClient::new(ClientConfig { base_url: std::env::var("API_URL")? })
//      // Fallback to `PageContext` HashMap until migrated page-by-page.
//      Ok(epsx_dioxus_ui::pages::analytics::AnalyticsResponse::default())
//  }
