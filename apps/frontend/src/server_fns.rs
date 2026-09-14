//! Server functions for Dioxus fullstack big-bang.
//!
//! Each `#[server]` replaces one `ssr.rs:fetch_page_data` branch that previously
//! did `params.insert("data_X", serde_json::to_string(outcome))` (HashMap<String,String>).
//! Pilot: `get_home_rankings` + `get_home_news`. Next: analytics, news, portfolio.
//!
//! Server-only: `ServiceClient` + `JwksVerifier` via `server_context` headers.
//! Client gets <500ms HMR via `dx serve --hot-reload` without `cargo run` restart.

use dioxus::prelude::*;

// TODO(Phase 2B): Enable once `ssr.rs` HashMap is removed. Example:
//
// #[server(GetHomeRankings)]
// pub async fn get_home_rankings() -> Result<epsx_dioxus_ui::pages::analytics::AnalyticsResponse, ServerFnError> {
//     let api_url = std::env::var("API_URL").unwrap_or_else(|_| "http://127.0.0.1:8080".into());
//     let client = epsx_client::ServiceClient::new(epsx_client::ClientConfig {
//         base_url: api_url,
//         timeout: std::time::Duration::from_secs(15),
//     });
//     // `ServiceClient::get_plain("/api/analytics/rankings?page=1&limit=3")`
//     // -> `AnalyticsResponse::validated`
//     Err(ServerFnError::new("not yet migrated: use ssr.rs fallback"))
// }

// Placeholder to make `cargo check` pass today (no `#[server]` yet).
pub fn _placeholder() {}
