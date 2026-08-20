//! Node-free progressive enhancement for the SSR applications.
//!
//! The browser executes only wasm-bindgen output generated from this crate.
//! No generated JavaScript or WebAssembly is committed to the repository.

/// Canonical generated module name used by the BFFs and build tooling.
pub const GENERATED_MODULE: &str = "epsx_browser_runtime_bootstrap.js";

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
    use js_sys::{Array, Function, Object, Promise, Reflect, Uint8Array};
    use serde::Deserialize;
    use serde_json::{json, Value};
    use wasm_bindgen::{closure::Closure, prelude::*, JsCast};
    use wasm_bindgen_futures::{spawn_local, JsFuture};
    use web_sys::{Document, Element, Event, Request, RequestInit, Response, Window};

    const GENERATED_WORKER: &str = "/runtime/epsx_service_worker_bootstrap.js";

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
            let Ok(Some(element)) = target.closest(concat!(
                "[data-epsx-action],[data-connect-wallet],[data-epsx-logout],",
                "[data-notification-mutation],[data-manual-screenshot],",
                "[data-manual-dialog-close],[data-docs-sidebar-toggle],",
                "[data-docs-sidebar-overlay],[data-docs-section-link],",
                "[data-docs-endpoint-toggle],[data-docs-code-tab],",
                "[data-docs-copy-code],[data-docs-copy-response],[data-push-action]"
            )) else {
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
                })
                .or_else(|| {
                    element
                        .has_attribute("data-manual-screenshot")
                        .then(|| "manual-open".into())
                })
                .or_else(|| {
                    element
                        .has_attribute("data-manual-dialog-close")
                        .then(|| "manual-close".into())
                })
                .or_else(|| {
                    element
                        .has_attribute("data-docs-sidebar-toggle")
                        .then(|| "docs-sidebar-toggle".into())
                })
                .or_else(|| {
                    element
                        .has_attribute("data-docs-sidebar-overlay")
                        .then(|| "docs-sidebar-close".into())
                })
                .or_else(|| {
                    element
                        .has_attribute("data-docs-section-link")
                        .then(|| "docs-section".into())
                })
                .or_else(|| {
                    element
                        .has_attribute("data-docs-endpoint-toggle")
                        .then(|| "docs-endpoint".into())
                })
                .or_else(|| {
                    element
                        .has_attribute("data-docs-code-tab")
                        .then(|| "docs-tab".into())
                })
                .or_else(|| {
                    element
                        .has_attribute("data-docs-copy-code")
                        .then(|| "docs-copy-code".into())
                })
                .or_else(|| {
                    element
                        .has_attribute("data-docs-copy-response")
                        .then(|| "docs-copy-response".into())
                })
                .or_else(|| {
                    element.get_attribute("data-push-action").map(|action| {
                        if action == "enable" {
                            "push-enable".into()
                        } else {
                            "push-disable".into()
                        }
                    })
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
            "create-checkout" => spawn_local(create_checkout(element)),
            "manual-open" => open_manual_dialog(&element),
            "manual-close" => close_manual_dialog(),
            "docs-sidebar-toggle" => set_docs_sidebar(&element, None),
            "docs-sidebar-close" => set_docs_sidebar(&element, Some(false)),
            "docs-section" => activate_docs_section(&element),
            "docs-endpoint" => toggle_docs_endpoint(&element),
            "docs-tab" => activate_docs_tab(&element),
            "docs-copy-code" => copy_docs_value(&element, false),
            "docs-copy-response" => copy_docs_value(&element, true),
            "push-enable" => spawn_local(change_push_subscription(true)),
            "push-disable" => spawn_local(change_push_subscription(false)),
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
        let _ = target.set_attribute("aria-hidden", if open { "false" } else { "true" });
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

    fn open_manual_dialog(trigger: &Element) {
        let Some((_, document)) = window_document() else {
            return;
        };
        let Ok(Some(dialog)) = document.query_selector("[data-manual-dialog]") else {
            return;
        };
        if let Ok(Some(image)) = dialog.query_selector("[data-manual-dialog-image]") {
            if let Some(source) = trigger.get_attribute("data-screenshot-src") {
                let _ = image.set_attribute("src", &source);
            }
            if let Some(alt) = trigger.get_attribute("data-screenshot-alt") {
                let _ = image.set_attribute("alt", &alt);
            }
        }
        if let Ok(Some(title)) = dialog.query_selector("[data-manual-dialog-title]") {
            title.set_text_content(trigger.get_attribute("data-screenshot-alt").as_deref());
        }
        let _ = dialog.remove_attribute("hidden");
        set_body_overflow("hidden");
        if let Ok(Some(close)) = dialog.query_selector("[data-manual-dialog-close]") {
            if let Ok(close) = close.dyn_into::<web_sys::HtmlElement>() {
                let _ = close.focus();
            }
        }
    }

    fn close_manual_dialog() {
        let Some((_, document)) = window_document() else {
            return;
        };
        if let Ok(Some(dialog)) = document.query_selector("[data-manual-dialog]") {
            let _ = dialog.set_attribute("hidden", "");
        }
        set_body_overflow("");
    }

    fn set_body_overflow(value: &str) {
        let Some((_, document)) = window_document() else {
            return;
        };
        let Some(body) = document.body() else { return };
        if let Ok(style) = Reflect::get(body.as_ref(), &JsValue::from_str("style")) {
            let _ = Reflect::set(
                &style,
                &JsValue::from_str("overflow"),
                &JsValue::from_str(value),
            );
        }
    }

    fn set_docs_sidebar(trigger: &Element, requested: Option<bool>) {
        let Some((_, document)) = window_document() else {
            return;
        };
        let Ok(Some(sidebar)) = document.query_selector("[data-docs-sidebar]") else {
            return;
        };
        let open = requested.unwrap_or_else(|| !sidebar.class_list().contains("open"));
        let _ = sidebar.class_list().toggle_with_force("open", open);
        if let Ok(Some(toggle)) = document.query_selector("[data-docs-sidebar-toggle]") {
            let _ = toggle.set_attribute("aria-expanded", if open { "true" } else { "false" });
            let _ = toggle.set_attribute(
                "aria-label",
                if open {
                    "Close API reference navigation"
                } else {
                    "Open API reference navigation"
                },
            );
        }
        if let Ok(Some(overlay)) = document.query_selector("[data-docs-sidebar-overlay]") {
            if open {
                let _ = overlay.remove_attribute("hidden");
            } else {
                let _ = overlay.set_attribute("hidden", "");
            }
        }
        let _ = trigger;
    }

    fn activate_docs_section(link: &Element) {
        let Some((_, document)) = window_document() else {
            return;
        };
        if let Ok(links) = document.query_selector_all("[data-docs-section-link]") {
            for index in 0..links.length() {
                if let Some(item) = links
                    .item(index)
                    .and_then(|node| node.dyn_into::<Element>().ok())
                {
                    let _ = item.class_list().toggle_with_force("active", item == *link);
                }
            }
        }
        set_docs_sidebar(link, Some(false));
    }

    fn toggle_docs_endpoint(button: &Element) {
        let Some((_, document)) = window_document() else {
            return;
        };
        let Some(id) = button.get_attribute("aria-controls") else {
            return;
        };
        let Some(body) = document.get_element_by_id(&id) else {
            return;
        };
        let open = button.get_attribute("aria-expanded").as_deref() != Some("true");
        let _ = button.set_attribute("aria-expanded", if open { "true" } else { "false" });
        if open {
            let _ = body.remove_attribute("hidden");
        } else {
            let _ = body.set_attribute("hidden", "");
        }
        if let Ok(Some(chevron)) = button.query_selector(".docs-endpoint-card-chevron") {
            chevron.set_text_content(Some(if open { "▾" } else { "▸" }));
        }
    }

    fn activate_docs_tab(tab: &Element) {
        let Some(language) = tab.get_attribute("data-docs-code-tab") else {
            return;
        };
        let Ok(Some(example)) = tab.closest(".docs-code-example") else {
            return;
        };
        for (selector, attribute) in [
            ("[data-docs-code-tab]", "data-docs-code-tab"),
            ("[data-docs-code-panel]", "data-docs-code-panel"),
        ] {
            let Ok(nodes) = example.query_selector_all(selector) else {
                continue;
            };
            for index in 0..nodes.length() {
                let Some(node) = nodes
                    .item(index)
                    .and_then(|node| node.dyn_into::<Element>().ok())
                else {
                    continue;
                };
                let selected = node.get_attribute(attribute).as_deref() == Some(&language);
                if attribute == "data-docs-code-tab" {
                    let _ = node.class_list().toggle_with_force("active", selected);
                    let _ = node
                        .set_attribute("aria-selected", if selected { "true" } else { "false" });
                    let _ = node.set_attribute("tabindex", if selected { "0" } else { "-1" });
                } else if selected {
                    let _ = node.remove_attribute("hidden");
                } else {
                    let _ = node.set_attribute("hidden", "");
                }
            }
        }
    }

    fn copy_docs_value(button: &Element, response: bool) {
        let container_selector = if response {
            ".docs-response-example"
        } else {
            ".docs-code-example"
        };
        let value_selector = if response {
            "code"
        } else {
            "[data-docs-code-panel]:not([hidden]) code"
        };
        let value = button
            .closest(container_selector)
            .ok()
            .flatten()
            .and_then(|container| container.query_selector(value_selector).ok().flatten())
            .and_then(|node| node.text_content());
        if let Some(value) = value {
            let _ = button.set_attribute("data-copy", &value);
            copy_value(button);
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

    async fn create_checkout(element: Element) {
        let amount = element.get_attribute("data-amount").unwrap_or_default();
        let currency = element.get_attribute("data-currency").unwrap_or_default();
        let token = element.get_attribute("data-token").unwrap_or_default();
        let chain_id = element.get_attribute("data-chain-id").unwrap_or_default();
        let description = element
            .get_attribute("data-description")
            .unwrap_or_default();
        let api_base = element
            .get_attribute("data-api-base")
            .unwrap_or_else(|| "/api".into());
        let fallback = element
            .get_attribute("data-pay-url")
            .unwrap_or_else(|| "/pay".into());
        if !api_base.starts_with('/') || api_base.starts_with("//") {
            set_status("Checkout is unavailable.", true);
            return;
        }
        let endpoint = format!("{}/v1/pay/intent", api_base.trim_end_matches('/'));
        match fetch_json(
            &endpoint,
            "POST",
            Some(json!({
                "amount": amount,
                "currency": currency,
                "token": token,
                "chain_id": chain_id,
                "description": description
            })),
        )
        .await
        {
            Ok(value) => {
                let intent = value
                    .get("intent")
                    .and_then(|value| value.get("id"))
                    .and_then(Value::as_str);
                let target = intent
                    .map(|id| format!("/pay?intent={}", js_sys::encode_uri_component(id)))
                    .unwrap_or(fallback);
                if let Some((window, _)) = window_document() {
                    let _ = window.location().assign(&target);
                }
            }
            Err(_) => set_status("Checkout could not be started.", true),
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
        if let Ok(Some(node)) = document.query_selector("[data-epsx-notification-push]") {
            spawn_local(load_push_status(node));
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
        loop {
            set_payment_loading(true);
            if let Ok(value) = fetch_json(&endpoint, "GET", None).await {
                let payload = value.get("intent").unwrap_or(&value);
                if let Some(status) = payload.get("status").and_then(Value::as_str) {
                    apply_payment_status(&node, status);
                    set_payment_loading(false);
                    if matches!(status, "released" | "refunded" | "cancelled") {
                        break;
                    }
                }
            }
            set_payment_loading(false);
            delay(5_000).await;
        }
    }

    fn apply_payment_status(node: &Element, status: &str) {
        let color = match status {
            "escrowed" => "blue",
            "released" => "green",
            "refunded" | "cancelled" => "red",
            "pending" => "orange",
            _ => "slate",
        };
        let _ = node.set_attribute("data-status", status);
        let _ = node.set_attribute("data-payment-status", status);
        let Some((_, document)) = window_document() else {
            return;
        };
        if let Some(label) = document.get_element_by_id("pay-escrow-status-label") {
            label.set_text_content(Some(status));
            label.set_class_name(&format!(
                "pay-escrow-status-label text-sm font-medium text-{color}-500"
            ));
        }
        if let Some(dot) = document.get_element_by_id("pay-escrow-status-dot") {
            dot.set_class_name(&format!(
                "pay-escrow-status-dot h-2 w-2 rounded-full bg-{color}-500"
            ));
        }
        for id in ["pay-escrow-flow-steps", "pay-escrow-chain-card"] {
            if let Some(target) = document.get_element_by_id(id) {
                let _ = target.set_attribute("data-status", status);
            }
        }
    }

    fn set_payment_loading(loading: bool) {
        let Some((_, document)) = window_document() else {
            return;
        };
        if let Some(indicator) = document.get_element_by_id("pay-escrow-polling-indicator") {
            let _ = indicator.class_list().toggle_with_force("hidden", !loading);
        }
    }

    async fn delay(milliseconds: i32) {
        let promise = Promise::new(&mut |resolve, _reject| {
            if let Some(window) = web_sys::window() {
                let _ = window
                    .set_timeout_with_callback_and_timeout_and_arguments_0(&resolve, milliseconds);
            } else {
                let _ = resolve.call0(&JsValue::UNDEFINED);
            }
        });
        let _ = JsFuture::from(promise).await;
    }

    async fn load_push_status(root: Element) {
        set_push_state(
            &root,
            "checking",
            "Checking whether browser push is available…",
        );
        let Ok(value) = fetch_json("/api/v1/notifications/push", "GET", None).await else {
            set_push_state(
                &root,
                "unavailable",
                "Browser push availability could not be verified.",
            );
            return;
        };
        let enabled = value.get("enabled").and_then(Value::as_bool);
        let subscribed = value.get("subscribed").and_then(Value::as_bool);
        let public_key = value.get("public_key").and_then(Value::as_str);
        if enabled.is_none()
            || subscribed.is_none()
            || enabled != Some(public_key.is_some())
            || (enabled == Some(false) && subscribed == Some(true))
        {
            set_push_state(
                &root,
                "unavailable",
                "Browser push availability could not be verified.",
            );
            return;
        }
        if !enabled.unwrap_or(false) {
            set_push_state(
                &root,
                "unavailable",
                "Browser push is unavailable until the notification service is configured.",
            );
            return;
        }
        let Some(public_key) = public_key.filter(|key| valid_base64url_key(key)) else {
            set_push_state(
                &root,
                "unavailable",
                "Browser push configuration is invalid.",
            );
            return;
        };
        let _ = root.set_attribute("data-push-public-key", public_key);
        if subscribed.unwrap_or(false) {
            set_push_state(
                &root,
                "subscribed",
                "A browser push subscription is registered for this wallet.",
            );
        } else {
            set_push_state(
                &root,
                "ready",
                "Browser push is ready. Enable it from this browser when you are ready.",
            );
        }
    }

    async fn change_push_subscription(enable: bool) {
        let Some((window, document)) = window_document() else {
            return;
        };
        let Ok(Some(root)) = document.query_selector("[data-epsx-notification-push]") else {
            return;
        };
        set_push_state(
            &root,
            "pending",
            if enable {
                "Requesting browser notification permission…"
            } else {
                "Removing the browser push subscription…"
            },
        );
        let result = async {
            if enable {
                request_notification_permission(&window).await?;
            }
            let registration = JsFuture::from(window.navigator().service_worker().ready()?).await?;
            let manager = Reflect::get(&registration, &JsValue::from_str("pushManager"))?;
            let mut subscription = call_promise_method(&manager, "getSubscription", None).await?;
            if enable && (subscription.is_null() || subscription.is_undefined()) {
                let key = root
                    .get_attribute("data-push-public-key")
                    .and_then(|value| decode_base64url(&value))
                    .ok_or_else(|| JsValue::from_str("invalid push key"))?;
                let options = Object::new();
                Reflect::set(
                    &options,
                    &JsValue::from_str("userVisibleOnly"),
                    &JsValue::TRUE,
                )?;
                let bytes = Uint8Array::from(key.as_slice());
                Reflect::set(
                    &options,
                    &JsValue::from_str("applicationServerKey"),
                    bytes.as_ref(),
                )?;
                subscription =
                    call_promise_method(&manager, "subscribe", Some(options.as_ref())).await?;
            }
            let body = push_subscription_body(&subscription)?;
            if enable {
                let value =
                    fetch_json("/api/v1/notifications/push", "PUT", Some(body.clone())).await?;
                if value.get("enabled") != Some(&Value::Bool(true))
                    || value.get("subscribed") != Some(&Value::Bool(true))
                {
                    return Err(JsValue::from_str("push subscription was rejected"));
                }
            } else {
                let endpoint = body
                    .get("endpoint")
                    .and_then(Value::as_str)
                    .ok_or_else(|| JsValue::from_str("missing subscription endpoint"))?;
                let value = fetch_json(
                    "/api/v1/notifications/push",
                    "DELETE",
                    Some(json!({"endpoint": endpoint})),
                )
                .await?;
                if value.get("subscribed") != Some(&Value::Bool(false)) {
                    return Err(JsValue::from_str("push unsubscribe was rejected"));
                }
                let _ = call_promise_method(&subscription, "unsubscribe", None).await;
            }
            Ok::<(), JsValue>(())
        }
        .await;
        match result {
            Ok(()) if enable => set_push_state(
                &root,
                "subscribed",
                "A browser push subscription is registered for this wallet.",
            ),
            Ok(()) => {
                let _ = root.remove_attribute("data-push-public-key");
                load_push_status(root).await;
            }
            Err(_) if enable => set_push_state(
                &root,
                "ready",
                "The browser subscription could not be registered. Try again when the service is available.",
            ),
            Err(_) => set_push_state(
                &root,
                "subscribed",
                "The browser push subscription could not be removed.",
            ),
        }
    }

    async fn request_notification_permission(window: &Window) -> Result<(), JsValue> {
        if !window.is_secure_context() {
            return Err(JsValue::from_str("secure context required"));
        }
        let notification = Reflect::get(window.as_ref(), &JsValue::from_str("Notification"))?;
        if notification.is_null() || notification.is_undefined() {
            return Err(JsValue::from_str("notifications unsupported"));
        }
        let mut permission = Reflect::get(&notification, &JsValue::from_str("permission"))?
            .as_string()
            .unwrap_or_default();
        if permission == "default" {
            let request = Reflect::get(&notification, &JsValue::from_str("requestPermission"))?
                .dyn_into::<Function>()?;
            let promise = request.call0(&notification)?.dyn_into::<Promise>()?;
            permission = JsFuture::from(promise)
                .await?
                .as_string()
                .unwrap_or_default();
        }
        if permission == "granted" {
            Ok(())
        } else {
            Err(JsValue::from_str("notification permission not granted"))
        }
    }

    async fn call_promise_method(
        receiver: &JsValue,
        method: &str,
        argument: Option<&JsValue>,
    ) -> Result<JsValue, JsValue> {
        let function =
            Reflect::get(receiver, &JsValue::from_str(method))?.dyn_into::<Function>()?;
        let value = if let Some(argument) = argument {
            function.call1(receiver, argument)?
        } else {
            function.call0(receiver)?
        };
        JsFuture::from(value.dyn_into::<Promise>()?).await
    }

    fn push_subscription_body(subscription: &JsValue) -> Result<Value, JsValue> {
        if subscription.is_null() || subscription.is_undefined() {
            return Err(JsValue::from_str("browser subscription unavailable"));
        }
        let to_json =
            Reflect::get(subscription, &JsValue::from_str("toJSON"))?.dyn_into::<Function>()?;
        let raw = to_json.call0(subscription)?;
        let value: Value = serde_wasm_bindgen::from_value(raw)
            .map_err(|error| JsValue::from_str(&error.to_string()))?;
        let endpoint = value
            .get("endpoint")
            .and_then(Value::as_str)
            .unwrap_or_default();
        let p256dh = value
            .pointer("/keys/p256dh")
            .and_then(Value::as_str)
            .unwrap_or_default();
        let auth = value
            .pointer("/keys/auth")
            .and_then(Value::as_str)
            .unwrap_or_default();
        let valid_endpoint = url::Url::parse(endpoint).is_ok_and(|url| {
            url.scheme() == "https"
                && url.username().is_empty()
                && url.password().is_none()
                && url.query().is_none()
                && url.fragment().is_none()
                && endpoint.len() <= 2_048
        });
        if !valid_endpoint || !valid_base64url_key(p256dh) || !valid_base64url_key(auth) {
            return Err(JsValue::from_str("invalid browser subscription"));
        }
        Ok(json!({"endpoint": endpoint, "p256dh": p256dh, "auth": auth}))
    }

    fn set_push_state(root: &Element, state: &str, message: &str) {
        let _ = root.set_attribute("data-push-state", state);
        if let Ok(Some(status)) = root.query_selector("[data-push-status]") {
            status.set_text_content(Some(message));
        }
        for (action, visible, enabled) in [
            ("enable", state != "subscribed", state == "ready"),
            ("disable", state == "subscribed", state == "subscribed"),
        ] {
            let selector = format!("[data-push-action=\"{action}\"]");
            let Ok(Some(button)) = root.query_selector(&selector) else {
                continue;
            };
            if visible {
                let _ = button.remove_attribute("hidden");
            } else {
                let _ = button.set_attribute("hidden", "");
            }
            if enabled {
                let _ = button.remove_attribute("disabled");
            } else {
                let _ = button.set_attribute("disabled", "");
            }
        }
    }

    fn valid_base64url_key(value: &str) -> bool {
        !value.is_empty()
            && value.len() <= 256
            && value
                .bytes()
                .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_'))
    }

    fn decode_base64url(value: &str) -> Option<Vec<u8>> {
        if !valid_base64url_key(value) || value.len() % 4 == 1 {
            return None;
        }
        let mut output = Vec::with_capacity(value.len() * 3 / 4);
        let mut accumulator = 0_u32;
        let mut bits = 0_u8;
        for byte in value.bytes() {
            let digit = match byte {
                b'A'..=b'Z' => byte - b'A',
                b'a'..=b'z' => byte - b'a' + 26,
                b'0'..=b'9' => byte - b'0' + 52,
                b'-' => 62,
                b'_' => 63,
                _ => return None,
            };
            accumulator = (accumulator << 6) | u32::from(digit);
            bits += 6;
            if bits >= 8 {
                bits -= 8;
                output.push((accumulator >> bits) as u8);
                accumulator &= (1_u32 << bits).saturating_sub(1);
            }
        }
        (!output.is_empty()).then_some(output)
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
