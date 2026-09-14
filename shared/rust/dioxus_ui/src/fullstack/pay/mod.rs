//! Pay owns its component tree and reactive state. Browser interop is isolated
//! to wallet, clipboard and session storage adapters; it never renders UI.
mod home;
pub mod types;
mod ui;
mod wallet;
use super::LoadError;
use dioxus::prelude::*;
pub use types::Action;
pub use types::*;
pub use ui::PayApp;

#[cfg(feature = "server")]
type ProviderFuture<T> =
    std::pin::Pin<Box<dyn std::future::Future<Output = Result<T, LoadError>> + Send>>;
#[cfg(feature = "server")]
#[derive(Clone)]
pub struct PayProvider {
    pub read: std::sync::Arc<
        dyn Fn(Page, Credentials, http::HeaderMap) -> ProviderFuture<PageData> + Send + Sync,
    >,
    pub action: std::sync::Arc<
        dyn Fn(Action, Credentials, String, http::HeaderMap) -> ProviderFuture<ActionResult>
            + Send
            + Sync,
    >,
}
#[server(prefix = "/_server/pay", endpoint = "read")]
pub async fn read_pay(
    page: Page,
    credentials: Credentials,
) -> Result<Result<PageData, LoadError>, ServerFnError> {
    use dioxus_server::axum::Extension;
    let Extension(provider) =
        dioxus_fullstack::FullstackContext::extract::<Extension<PayProvider>, _>()
            .await
            .map_err(|_| ServerFnError::new("Pay provider unavailable"))?;
    let headers = dioxus_fullstack::FullstackContext::extract::<http::HeaderMap, _>()
        .await
        .map_err(|_| ServerFnError::new("Request context unavailable"))?;
    Ok((provider.read)(page, credentials, headers).await)
}
#[server(prefix = "/_server/pay", endpoint = "action")]
pub async fn act_pay(
    action: Action,
    credentials: Credentials,
    idempotency_key: String,
) -> Result<Result<ActionResult, LoadError>, ServerFnError> {
    use dioxus_server::axum::Extension;
    let Extension(provider) =
        dioxus_fullstack::FullstackContext::extract::<Extension<PayProvider>, _>()
            .await
            .map_err(|_| ServerFnError::new("Pay provider unavailable"))?;
    let headers = dioxus_fullstack::FullstackContext::extract::<http::HeaderMap, _>()
        .await
        .map_err(|_| ServerFnError::new("Request context unavailable"))?;
    Ok((provider.action)(action, credentials, idempotency_key, headers).await)
}

pub fn known_page_path(path: &str) -> bool {
    matches!(path, "/" | "/docs" | "/docs/merchant") || ui::page_for(path).is_some()
}
