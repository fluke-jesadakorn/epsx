//! Node-free progressive enhancement for the SSR applications.
//!
//! The browser executes only wasm-bindgen output generated from this crate.
//! No generated JavaScript or WebAssembly is committed to the repository.

/// Canonical generated module name used by the BFFs and build tooling.
pub const GENERATED_MODULE: &str = "epsx_browser_runtime.js";

/// Accept only same-origin absolute-path redirects.
pub fn safe_return_path(raw: &str) -> &str {
    if raw.starts_with('/')
        && !raw.starts_with("//")
        && !raw.contains('\\')
        && !raw.starts_with("/auth")
    {
        raw
    } else {
        "/"
    }
}

#[cfg(target_arch = "wasm32")]
mod browser {
    use super::safe_return_path;
    use js_sys::{Array, Function, Object, Promise, Reflect};
    use serde::Deserialize;
    use serde_json::{json, Value};
    use wasm_bindgen::{closure::Closure, prelude::*, JsCast};
    use wasm_bindgen_futures::{spawn_local, JsFuture};
    use web_sys::{Document, Element, Event, Request, RequestInit, Response, Window};

    const GENERATED_WORKER: &str = "/runtime/epsx_service_worker.js";

    #[derive(Deserialize)]
    struct Challenge {
        message: String,
        nonce: String,
    }

    #[wasm_bindgen(start)]
    pub fn start() -> Result<(), JsValue> {
        let window = web_sys::window().ok_or("window unavailable")?;
        let document = window.document().ok_or("document unavailable")?;
        apply_theme(&window, &document);
        bind_clicks(&document)?;
        register_worker(&window);
        start_route_tasks(&window, &document);
        Ok(())
    }

    fn bind_clicks(document: &Document) -> Result<(), JsValue> {
        let closure = Closure::<dyn FnMut(Event)>::new(move |event: Event| {
            let Some(target) = event
                .target()
                .and_then(|target| target.dyn_into::<Element>().ok())
            else {
                return;
            };
            let Ok(Some(element)) = target.closest(
                "[data-epsx-action],[data-connect-wallet],[data-epsx-logout],[data-notification-mutation]",
            ) else {
                return;
            };
            let action = element
                .get_attribute("data-epsx-action")
                .or_else(|| {
                    element
                        .has_attribute("data-connect-wallet")
                        .then(|| "connect-wallet".into())
                })
                .or_else(|| {
                    element
                        .has_attribute("data-epsx-logout")
                        .then(|| "logout".into())
                })
                .or_else(|| {
                    element
                        .has_attribute("data-notification-mutation")
                        .then(|| "notification-mutation".into())
                });
            let Some(action) = action else { return };
            if !matches!(action.as_str(), "native-link" | "native-submit") {
                event.prevent_default();
            }
            dispatch_action(element, &action);
        });
        document.add_event_listener_with_callback("click", closure.as_ref().unchecked_ref())?;
        closure.forget();
        Ok(())
    }

    fn dispatch_action(element: Element, action: &str) {
        match action {
            "theme-toggle" => toggle_theme(),
            "toggle-nav" | "toggle-dropdown" => toggle_controlled(&element, "open"),
            "toggle-mobile-menu" => toggle_target(&element, "epsx-mobile-sheet", "open"),
            "open-sheet" | "open-modal" => set_named_target(&element, true),
            "close-sheet" | "close-modal" => set_named_target(&element, false),
            "activate-tab" => activate_tab(&element),
            "copy" => copy_value(&element),
            "share" => share_value(&element),
            "connect-wallet" => spawn_local(connect_wallet()),
            "logout" => {
                let target = element
                    .get_attribute("data-epsx-logout-target")
                    .unwrap_or_else(|| "/".into());
                spawn_local(logout(target));
            }
            "session-recover" => spawn_local(recover_session()),
            "notification-mutation" => spawn_local(notification_mutation(element)),
            "retry" => reload(),
            "back" => history_back(),
            _ => {}
        }
    }

    fn window_document() -> Option<(Window, Document)> {
        let window = web_sys::window()?;
        let document = window.document()?;
        Some((window, document))
    }

    fn apply_theme(window: &Window, document: &Document) {
        let stored = window
            .local_storage()
            .ok()
            .flatten()
            .and_then(|storage| storage.get_item("epsx-theme").ok().flatten());
        let dark = match stored.as_deref() {
            Some("light") => false,
            Some("dark") => true,
            _ => window
                .match_media("(prefers-color-scheme: dark)")
                .ok()
                .flatten()
                .is_none_or(|query| query.matches()),
        };
        if let Some(root) = document.document_element() {
            let _ = root.class_list().toggle_with_force("dark", dark);
        }
    }

    fn toggle_theme() {
        let Some((window, document)) = window_document() else {
            return;
        };
        let Some(root) = document.document_element() else {
            return;
        };
        let dark = !root.class_list().contains("dark");
        let _ = root.class_list().toggle_with_force("dark", dark);
        if let Ok(Some(storage)) = window.local_storage() {
            let _ = storage.set_item("epsx-theme", if dark { "dark" } else { "light" });
        }
        if let Ok(event) = web_sys::CustomEvent::new("epsx:theme") {
            let _ = document.dispatch_event(&event);
        }
    }

    fn toggle_controlled(element: &Element, class_name: &str) {
        let target = element
            .get_attribute("aria-controls")
            .and_then(|id| window_document()?.1.get_element_by_id(&id))
            .or_else(|| element.parent_element());
        let Some(target) = target else { return };
        let open = !target.class_list().contains(class_name);
        let _ = target.class_list().toggle_with_force(class_name, open);
        let _ = element.set_attribute("aria-expanded", if open { "true" } else { "false" });
    }

    fn toggle_target(element: &Element, id: &str, class_name: &str) {
        let Some((_, document)) = window_document() else {
            return;
        };
        let Some(target) = document.get_element_by_id(id) else {
            return;
        };
        let open = !target.class_list().contains(class_name);
        let _ = target.class_list().toggle_with_force(class_name, open);
        let _ = element.set_attribute("aria-expanded", if open { "true" } else { "false" });
    }

    fn set_named_target(element: &Element, open: bool) {
        let id = element
            .get_attribute("data-epsx-target")
            .or_else(|| element.get_attribute("aria-controls"));
        let Some((_, document)) = window_document() else {
            return;
        };
        let Some(target) = id.and_then(|id| document.get_element_by_id(&id)) else {
            return;
        };
        let _ = target.class_list().toggle_with_force("open", open);
        let _ = target.set_attribute("aria-hidden", if open { "false" } else { "true" });
    }

    fn activate_tab(element: &Element) {
        let Some((_, document)) = window_document() else {
            return;
        };
        let Some(group) = element.get_attribute("data-tab-group") else {
            return;
        };
        let Some(name) = element.get_attribute("data-tab-name") else {
            return;
        };
        let selector = format!("[data-tab-group=\"{group}\"]");
        if let Ok(nodes) = document.query_selector_all(&selector) {
            for index in 0..nodes.length() {
                if let Some(node) = nodes
                    .item(index)
                    .and_then(|node| node.dyn_into::<Element>().ok())
                {
                    let selected = node.get_attribute("data-tab-name").as_deref() == Some(&name);
                    let _ = node.class_list().toggle_with_force("active", selected);
                    let _ = node
                        .set_attribute("aria-selected", if selected { "true" } else { "false" });
                }
            }
        }
        let panel_selector = format!("[data-tab-panel-group=\"{group}\"]");
        if let Ok(nodes) = document.query_selector_all(&panel_selector) {
            for index in 0..nodes.length() {
                if let Some(node) = nodes
                    .item(index)
                    .and_then(|node| node.dyn_into::<Element>().ok())
                {
                    let selected = node.get_attribute("data-tab-panel").as_deref() == Some(&name);
                    let _ = node.toggle_attribute_with_force("hidden", !selected);
                }
            }
        }
    }

    fn copy_value(element: &Element) {
        let Some((window, _)) = window_document() else {
            return;
        };
        let Some(value) = element.get_attribute("data-copy") else {
            return;
        };
        let clipboard = window.navigator().clipboard();
        spawn_local(async move {
            let _ = JsFuture::from(clipboard.write_text(&value)).await;
        });
    }

    fn share_value(element: &Element) {
        let Some((window, _)) = window_document() else {
            return;
        };
        let value = element.get_attribute("data-share-text").unwrap_or_default();
        let clipboard = window.navigator().clipboard();
        spawn_local(async move {
            let _ = JsFuture::from(clipboard.write_text(&value)).await;
        });
    }

    async fn fetch_json(path: &str, method: &str, body: Option<Value>) -> Result<Value, JsValue> {
        let window = web_sys::window().ok_or("window unavailable")?;
        let options = RequestInit::new();
        options.set_method(method);
        options.set_credentials(web_sys::RequestCredentials::SameOrigin);
        if let Some(body) = body {
            options.set_body(&JsValue::from_str(&body.to_string()));
        }
        let request = Request::new_with_str_and_init(path, &options)?;
        request.headers().set("accept", "application/json")?;
        if method != "GET" {
            request.headers().set("content-type", "application/json")?;
        }
        let response = JsFuture::from(window.fetch_with_request(&request))
            .await?
            .dyn_into::<Response>()?;
        if !response.ok() {
            return Err(JsValue::from_str(&format!("HTTP {}", response.status())));
        }
        let value = JsFuture::from(response.json()?).await?;
        serde_wasm_bindgen::from_value(value).map_err(|error| JsValue::from_str(&error.to_string()))
    }

    async fn wallet_request(method: &str, params: Array) -> Result<JsValue, JsValue> {
        let window = web_sys::window().ok_or("window unavailable")?;
        let provider = Reflect::get(window.as_ref(), &JsValue::from_str("ethereum"))?;
        if provider.is_null() || provider.is_undefined() {
            return Err(JsValue::from_str("No wallet detected"));
        }
        let request = Object::new();
        Reflect::set(
            &request,
            &JsValue::from_str("method"),
            &JsValue::from_str(method),
        )?;
        Reflect::set(&request, &JsValue::from_str("params"), &params)?;
        let function =
            Reflect::get(&provider, &JsValue::from_str("request"))?.dyn_into::<Function>()?;
        let promise = function.call1(&provider, &request)?.dyn_into::<Promise>()?;
        JsFuture::from(promise).await
    }

    async fn connect_wallet() {
        set_status("Connecting wallet…", false);
        let result = async {
            let accounts = wallet_request("eth_requestAccounts", Array::new()).await?;
            let accounts = Array::from(&accounts);
            let address = accounts.get(0).as_string().ok_or("wallet returned no account")?;
            let chain_hex = wallet_request("eth_chainId", Array::new())
                .await?
                .as_string()
                .ok_or("wallet returned no chain")?;
            let chain_id = u64::from_str_radix(chain_hex.trim_start_matches("0x"), 16)
                .map_err(|_| JsValue::from_str("invalid wallet chain"))?;
            let challenge: Challenge = serde_json::from_value(
                fetch_json("/api/v1/auth/challenge", "POST", Some(json!({"address": address}))).await?,
            )
            .map_err(|error| JsValue::from_str(&error.to_string()))?;
            let params = Array::new();
            params.push(&JsValue::from_str(&challenge.message));
            params.push(&JsValue::from_str(&address));
            let signature = wallet_request("personal_sign", params)
                .await?
                .as_string()
                .ok_or("wallet returned no signature")?;
            let session = fetch_json(
                "/api/v1/auth/siwe",
                "POST",
                Some(json!({
                    "message": challenge.message,
                    "signature": signature,
                    "address": address,
                    "nonce": challenge.nonce,
                    "chain_id": chain_id.to_string()
                })),
            )
            .await?;
            if session.get("authenticated") != Some(&Value::Bool(true)) {
                return Err(JsValue::from_str("verification did not establish a session"));
            }
            let Some((window, document)) = window_document() else { return Err(JsValue::from_str("browser unavailable")) };
            let cookie = format!(
                "epsx_wallet={}; Path=/; Max-Age=86400; SameSite=Lax",
                js_sys::encode_uri_component(&json!({"address": address, "connector_id": "injected", "chain_id": chain_id.to_string()}).to_string())
            );
            let _ = Reflect::set(document.as_ref(), &JsValue::from_str("cookie"), &JsValue::from_str(&cookie));
            let search = window.location().search().unwrap_or_default();
            let target = search
                .split('&')
                .find_map(|part| part.trim_start_matches('?').strip_prefix("return_url="))
                .map(|value| {
                    js_sys::decode_uri_component(value)
                        .ok()
                        .and_then(|value| value.as_string())
                        .unwrap_or_else(|| "/".to_string())
                })
                .unwrap_or_else(|| "/".to_string());
            window.location().replace(safe_return_path(&target))?;
            Ok::<(), JsValue>(())
        }
        .await;
        if let Err(error) = result {
            set_status(
                &error
                    .as_string()
                    .unwrap_or_else(|| "Wallet connection failed".into()),
                true,
            );
        }
    }

    async fn logout(target: String) {
        match fetch_json("/api/v1/auth/logout", "POST", Some(json!({}))).await {
            Ok(_) => {
                if let Some((window, _)) = window_document() {
                    let _ = window.location().assign(safe_return_path(&target));
                }
            }
            Err(error) => set_status(
                &error.as_string().unwrap_or_else(|| "Logout failed".into()),
                true,
            ),
        }
    }

    async fn recover_session() {
        if fetch_json("/api/v1/auth/refresh", "POST", Some(json!({})))
            .await
            .is_ok()
        {
            reload();
        } else {
            set_status("Your session could not be recovered. Sign in again.", true);
        }
    }

    async fn notification_mutation(element: Element) {
        let Some(action) = element.get_attribute("data-notification-mutation") else {
            return;
        };
        let method = if action == "acknowledge" {
            "PUT"
        } else {
            "POST"
        };
        let path = match action.as_str() {
            "mark-all" => "/api/v1/notifications/mark-all-read".into(),
            "clear-all" => "/api/v1/notifications/clear-all".into(),
            "read" | "unread" | "acknowledge" | "dismiss" | "delete" => {
                let Some(id) = element.get_attribute("data-notification-id") else {
                    return;
                };
                if id.is_empty() || id.len() > 128 || id.chars().any(char::is_whitespace) {
                    return;
                }
                format!("/api/v1/notifications/{id}/{action}")
            }
            _ => return,
        };
        if fetch_json(&path, method, Some(json!({}))).await.is_ok() {
            reload();
        } else {
            set_status("Notification changes could not be saved.", true);
        }
    }

    fn start_route_tasks(window: &Window, document: &Document) {
        if document
            .query_selector("[data-epsx-session-recovery]")
            .ok()
            .flatten()
            .is_some()
        {
            spawn_local(recover_session());
        }
        if let Ok(Some(node)) = document.query_selector("[data-notification-count]") {
            spawn_local(load_notification_count(node));
        }
        if let Ok(Some(node)) = document.query_selector("[data-payment-status-endpoint]") {
            spawn_local(poll_payment(node));
        }
        if let Ok(Some(node)) = document.query_selector("[data-notifications-live-status]") {
            start_notification_stream(node);
        }
        let _ = window;
    }

    async fn load_notification_count(node: Element) {
        let endpoint = node
            .get_attribute("data-notification-count-endpoint")
            .unwrap_or_else(|| "/api/v1/notifications/unread-count".into());
        if let Ok(value) = fetch_json(&endpoint, "GET", None).await {
            if let Some(count) = value.get("count").and_then(Value::as_u64) {
                let display = if count > 99 {
                    "99+".to_string()
                } else {
                    count.to_string()
                };
                node.set_text_content(Some(&display));
                let _ = node.remove_attribute("hidden");
            }
        }
    }

    async fn poll_payment(node: Element) {
        let Some(endpoint) = node.get_attribute("data-payment-status-endpoint") else {
            return;
        };
        if !endpoint.starts_with('/') || endpoint.starts_with("//") {
            return;
        }
        if let Ok(value) = fetch_json(&endpoint, "GET", None).await {
            if let Some(status) = value.get("status").and_then(Value::as_str) {
                node.set_text_content(Some(status));
                let _ = node.set_attribute("data-payment-status", status);
            }
        }
    }

    fn start_notification_stream(status: Element) {
        let Ok(source) = web_sys::EventSource::new("/api/v1/notifications/stream") else {
            return;
        };
        let closure = Closure::<dyn FnMut(Event)>::new(move |_event| {
            status.set_text_content(Some("New notification received. Refreshing…"));
            reload();
        });
        source.set_onmessage(Some(closure.as_ref().unchecked_ref()));
        closure.forget();
        std::mem::forget(source);
    }

    fn register_worker(window: &Window) {
        if !window.is_secure_context() {
            return;
        }
        let container = window.navigator().service_worker();
        let options = web_sys::RegistrationOptions::new();
        options.set_type("module");
        let _ = container.register_with_options(GENERATED_WORKER, &options);
    }

    fn set_status(message: &str, error: bool) {
        let Some((_, document)) = window_document() else {
            return;
        };
        let node = document
            .query_selector("[data-epsx-runtime-status]")
            .ok()
            .flatten()
            .or_else(|| document.get_element_by_id("wallet-status"));
        if let Some(node) = node {
            node.set_text_content(Some(message));
            let _ = node.set_attribute("role", if error { "alert" } else { "status" });
        }
    }

    fn reload() {
        if let Some((window, _)) = window_document() {
            let _ = window.location().reload();
        }
    }

    fn history_back() {
        if let Some((window, _)) = window_document() {
            let _ = window.history().and_then(|history| history.back());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn redirects_remain_same_origin_paths() {
        assert_eq!(safe_return_path("/dashboard?tab=one"), "/dashboard?tab=one");
        assert_eq!(safe_return_path("https://evil.example"), "/");
        assert_eq!(safe_return_path("//evil.example"), "/");
        assert_eq!(safe_return_path("/auth"), "/");
        assert_eq!(safe_return_path("/safe\\evil"), "/");
    }
}
