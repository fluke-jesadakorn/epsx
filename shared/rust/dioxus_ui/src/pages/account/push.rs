//! Dioxus state around a narrow browser PushManager adapter. Network access
//! remains in authenticated server functions; no browser DOM writes occur.
use crate::fullstack::LoadError;
use dioxus::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PushStatus {
    pub enabled: bool,
    pub subscribed: bool,
    pub public_key: Option<String>,
    pub subscription_id: Option<String>,
    pub created_at: Option<String>,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PushSubscription {
    pub endpoint: String,
    pub p256dh: String,
    pub auth: String,
    pub user_agent: Option<String>,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum PushCommand {
    Status,
    Subscribe(PushSubscription),
    Unsubscribe { endpoint: String },
}
#[cfg(feature = "server")]
#[derive(Clone)]
pub struct PushProvider(pub PushProviderCallback);
#[server(prefix = "/_server/frontend", endpoint = "account-push")]
pub async fn browser_push(
    command: PushCommand,
) -> Result<Result<PushStatus, LoadError>, ServerFnError> {
    use dioxus_server::axum::Extension;
    let Extension(provider) =
        dioxus_fullstack::FullstackContext::extract::<Extension<PushProvider>, _>()
            .await
            .map_err(|_| ServerFnError::new("Push provider unavailable"))?;
    let headers = dioxus_fullstack::FullstackContext::extract::<http::HeaderMap, _>()
        .await
        .map_err(|_| ServerFnError::new("Request context unavailable"))?;
    Ok((provider.0)(command, headers).await)
}

#[derive(Clone, Deserialize)]
struct BrowserResult {
    subscription: Option<PushSubscription>,
    error: Option<String>,
}

/// Called only after the user activates Enable/Disable. This adapter only
/// interacts with Notification permission, the service worker and PushManager.
async fn subscription(enable: bool, key: Option<String>) -> Result<PushSubscription, String> {
    let key = serde_json::to_string(&key).map_err(|_| "Invalid push configuration.".to_string())?;
    let script = format!(
        r#"
try {{
    if (!isSecureContext || !('Notification' in window) || !('serviceWorker' in navigator) || !('PushManager' in window)) throw new Error('Browser notifications are not supported here.');
    const local = ['localhost','127.0.0.1','::1','[::1]','dev.epsx.io','dev-admin.epsx.io','dev-pay.epsx.io'].includes(location.hostname) || location.hostname.endsWith('.localhost');
    if (local) throw new Error('Browser push is disabled on local development origins.');
    const enable = {enable}, key = {key};
    if (enable && await Notification.requestPermission() !== 'granted') throw new Error('Notification permission was not granted. Check browser settings before retrying.');
    let registration = await navigator.serviceWorker.getRegistration('/');
    if (!registration && enable) registration = await navigator.serviceWorker.register('/runtime/epsx_service_worker_bootstrap.v3.js?rev=3', {{ type:'module', scope:'/' }});
    if (!registration) throw new Error('No subscription exists in this browser.');
    registration = await Promise.race([navigator.serviceWorker.ready, new Promise((_, reject) => setTimeout(() => reject(new Error('The notification worker is not ready. Try again.')), 8000))]);
    let subscription = await registration.pushManager.getSubscription();
    if (!subscription && enable) {{
        if (!key || !/^[A-Za-z0-9_-]+$/.test(key)) throw new Error('Invalid push configuration.');
        const padded = key.replace(/-/g,'+').replace(/_/g,'/') + '='.repeat((4-key.length%4)%4);
        const bytes = Uint8Array.from(atob(padded), character => character.charCodeAt(0));
        subscription = await registration.pushManager.subscribe({{ userVisibleOnly:true, applicationServerKey:bytes }});
    }}
    if (!subscription) throw new Error('No subscription exists in this browser.');
    const value = subscription.toJSON();
    dioxus.send({{ subscription:{{ endpoint:value.endpoint, p256dh:value.keys.p256dh, auth:value.keys.auth, user_agent:navigator.userAgent }}, error:null }});
}} catch (error) {{ dioxus.send({{subscription:null,error:error.message || 'Browser notification setup failed.'}}); }}
"#
    );
    let result: BrowserResult = document::eval(&script)
        .recv()
        .await
        .map_err(|_| "Could not access browser notifications.".to_string())?;
    result.subscription.ok_or_else(|| {
        result
            .error
            .unwrap_or_else(|| "Browser notification setup failed.".into())
    })
}

#[component]
pub fn BrowserPush() -> Element {
    let initial =
        use_server_future(move || async move { browser_push(PushCommand::Status).await })?;
    let seed = initial
        .read()
        .clone()
        .unwrap_or(Ok(Err(LoadError::Unavailable)))
        .unwrap_or(Err(LoadError::Unavailable));
    let mut state = use_signal(|| seed.ok());
    let mut message = use_signal(String::new);
    let mut pending = use_signal(|| false);
    let mut supported = use_signal(|| false);
    let change = use_callback(move |enable: bool| {
        if *pending.peek() {
            return;
        }
        pending.set(true);
        message.set(
            if enable {
                "Requesting browser notification permission…"
            } else {
                "Removing this browser's subscription…"
            }
            .into(),
        );
        let key = state
            .peek()
            .as_ref()
            .and_then(|value| value.public_key.clone());
        spawn(async move {
            let result = async {
                let subscription = subscription(enable, key).await?;
                let command = if enable { PushCommand::Subscribe(subscription) } else { PushCommand::Unsubscribe { endpoint: subscription.endpoint } };
                let response = browser_push(command).await.map_err(|_| "Notification service is unavailable.".to_string())?.map_err(|error| error.message().to_string())?;
                if enable && (!response.enabled || !response.subscribed) { return Err("The service did not confirm this subscription.".into()); }
                if !enable {
                    if response.subscribed { return Err("The service did not confirm removal.".into()); }
                    let unsubscribed = document::eval("try { const registration = await navigator.serviceWorker.getRegistration('/'); const sub = await registration?.pushManager.getSubscription(); dioxus.send(!sub || await sub.unsubscribe()); } catch (_) { dioxus.send(false); }").recv::<bool>().await.unwrap_or(false);
                    if !unsubscribed { state.set(Some(response)); return Err("The service removed this subscription, but browser cleanup failed. Retry Disable to clear the browser subscription.".into()); }
                }
                Ok(response)
            }.await;
            pending.set(false);
            match result {
                Ok(value) => {
                    state.set(Some(value));
                    message.set(if enable { "A browser push subscription is registered for this wallet. Delivery is not confirmed." } else { "This browser's subscription was removed." }.into());
                }
                Err(error) => message.set(error),
            }
        });
    });
    let enabled = state.read().as_ref().is_some_and(|value| value.enabled);
    let registered = state.read().as_ref().is_some_and(|value| value.subscribed);
    rsx! {
        div { class: "mt-5 rounded-2xl border border-border bg-card p-5 fe-surface", "data-section": "account-browser-push", "data-dioxus-push": "true", aria_busy: pending(),
            onmounted: move |_| { spawn(async move {
                let available = document::eval("dioxus.send(isSecureContext && 'Notification' in window && 'serviceWorker' in navigator && 'PushManager' in window && !['localhost','127.0.0.1','::1','[::1]','dev.epsx.io','dev-admin.epsx.io','dev-pay.epsx.io'].includes(location.hostname) && !location.hostname.endsWith('.localhost'));").recv::<bool>().await.unwrap_or(false);
                supported.set(available);
            }); },
            h3 { class: "font-semibold text-foreground fe-tone-text", "Browser notifications" }
            p { class: "mt-1 text-sm leading-6 text-muted-foreground fe-tone-muted", role: "status", aria_live: "polite",
                if !message().is_empty() { "{message}" }
                else if !enabled { "Browser push is unavailable until the notification service is configured." }
                else if !supported() { "Browser notifications are unsupported or disabled on this origin." }
                else if registered { "A browser push subscription is registered for this wallet." }
                else { "Browser push is ready. Enable it from this browser when you are ready." }
            }
            p { class: "mt-1 text-xs leading-5 text-muted-foreground fe-tone-muted", "Browser permission and subscription status are shown here; this does not confirm provider delivery." }
            div { class: "mt-4 flex flex-wrap gap-3",
                button { r#type: "button", class: "btn btn-sm btn-primary", disabled: pending() || !enabled || !supported(), onclick: move |_| change.call(true), "Enable browser notifications" }
                if enabled && supported() { button { r#type: "button", class: "btn btn-sm btn-outline", disabled: pending(), onclick: move |_| change.call(false), "Disable browser notifications" } }
            }
        }
    }
}

#[cfg(feature = "server")]
pub type PushProviderCallback = std::sync::Arc<
    dyn Fn(
            PushCommand,
            http::HeaderMap,
        ) -> std::pin::Pin<
            Box<dyn std::future::Future<Output = Result<PushStatus, LoadError>> + Send>,
        > + Send
        + Sync,
>;
