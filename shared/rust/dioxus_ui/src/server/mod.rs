//! Unified fullstack server functions — shared between `bff-frontend` :3000 and `bff-admin` :3001.
//!
//! Phase 1 scaffolding for easy maintenance / reduced codebase:
//! All `#[server]` live fetches live here once, validated once, and are consumed
//! via `use_server_future` in `pages/*` (frontend `A7 B1-B7` + admin `A8 B1-B7`).
//! Existing pilots `pages/home.rs:get_home_rankings` + `pages/analytics.rs:get_analytics_rankings`
//! stay as re-exports until Phase 2 moves every `ssr.rs:fetch_page_data` HashMap branch here.
//!
//! Shared helpers ensure 1 runtime guard, 1 API_URL, 1 `validated()` path, 1 `reqwest 0.12` call.

pub mod admin;
pub mod analytics;
pub mod commerce;
pub mod communication;
pub mod content;
pub mod identity;
pub mod plans;

use std::sync::{Mutex, OnceLock};
#[cfg(not(target_arch = "wasm32"))]
use std::time::Duration;

use dioxus::prelude::ServerFnError;

/// Test override for `API_URL` — `wiremock` `MockServer::uri()` without global `env` race.
/// `serial_test` serialises tests that mutate this.
static API_URL_OVERRIDE: Mutex<Option<String>> = Mutex::new(None);

#[cfg(test)]
pub fn set_api_base_for_test(url: String) {
    *API_URL_OVERRIDE.lock().unwrap() = Some(url);
}

#[cfg(test)]
pub fn clear_api_base_for_test() {
    *API_URL_OVERRIDE.lock().unwrap() = None;
}

/// Shared `API_URL` helper — test override else `API_URL` env else `http://127.0.0.1:8080` trimmed.
pub fn api_base() -> String {
    if let Some(url) = API_URL_OVERRIDE.lock().unwrap().clone() {
        return url.trim_end_matches('/').to_string();
    }
    let url = std::env::var("API_URL")
        .unwrap_or_else(|_| "http://127.0.0.1:8080".to_string())
        .trim_end_matches('/')
        .to_string();
    // Debug for hybrid HMR — remove after verify
    eprintln!(
        "[dioxus_ui::api_base] API_URL={} (env API_URL={:?})",
        url,
        std::env::var("API_URL")
    );
    url
}

/// Runtime guard — single validated place. `dioxus_ssr` unit tests run without a
/// `tokio` runtime and hit this `Err` → page renders `Unavailable` truthfully.
/// In WASM `not(feature="server")` there is no `tokio`, guard is `Ok`.
pub fn require_runtime() -> Result<(), ServerFnError> {
    #[cfg(feature = "server")]
    {
        if tokio::runtime::Handle::try_current().is_err() {
            return Err(ServerFnError::new("no runtime".to_string()));
        }
    }
    Ok(())
}

/// Best-performance + easy-maintenance auth forward: `dx` extracts `Cookie`/
/// `Authorization` from the incoming `Request` and forwards to backend `:8080`
/// in 1 hop (no BFF double fetch). Public routes ignore headers.
#[cfg(feature = "server")]
pub async fn forwarded_headers() -> http::HeaderMap {
    // Real `dx` extract via `FullstackContext::extract::<HeaderMap>()`.
    // Works inside `#[server]` handlers and SSR streaming (`dx serve`
    // live reads `server/*` directly via `use_server_future`).
    // Keep `::new()` fallback for `cargo test` `dioxus_ssr` without runtime.
    match dioxus_fullstack::FullstackContext::extract::<http::HeaderMap, _>().await {
        Ok(map) => map,
        Err(_) => http::HeaderMap::new(),
    }
}

#[cfg(not(feature = "server"))]
pub async fn forwarded_headers() -> http::HeaderMap {
    http::HeaderMap::new()
}

static CLIENT: OnceLock<reqwest::Client> = OnceLock::new();

#[cfg(not(target_arch = "wasm32"))]
fn pooled_client() -> &'static reqwest::Client {
    CLIENT.get_or_init(|| {
        reqwest::Client::builder()
            .timeout(Duration::from_secs(15))
            .pool_idle_timeout(Duration::from_secs(90))
            .build()
            .expect("pooled reqwest client builds with hickory-dns rustls-tls")
    })
}

#[cfg(target_arch = "wasm32")]
fn pooled_client() -> &'static reqwest::Client {
    CLIENT.get_or_init(reqwest::Client::new)
}

pub fn auth_forward_headers(all: &http::HeaderMap) -> http::HeaderMap {
    let mut out = http::HeaderMap::new();
    for key in ["cookie", "authorization"] {
        if let Some(v) = all.get(key) {
            out.insert(http::HeaderName::from_static(key), v.clone());
        }
    }
    out
}

pub async fn fetch_value(url: String) -> Result<serde_json::Value, ServerFnError> {
    require_runtime()?;
    let headers = forwarded_headers().await;
    let fwd = auth_forward_headers(&headers);
    let client = pooled_client();
    let mut req = client.get(&url);
    for (k, v) in fwd.iter() {
        req = req.header(k.as_str(), v.as_bytes());
    }
    let resp = req
        .send()
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;
    let value: serde_json::Value = resp
        .json()
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;
    Ok(value)
}
