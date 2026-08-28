//! Node-free progressive enhancement for the SSR applications.
//!
//! The browser executes only wasm-bindgen output generated from this crate.
//! No generated JavaScript or WebAssembly is committed to the repository.

/// Canonical generated module name used by the BFFs and build tooling.
pub const GENERATED_MODULE: &str = "epsx_browser_runtime_bootstrap.js";
/// The recovery worker must control the entire browser surface, not only the
/// `/runtime/` directory that contains its generated module.
pub const GENERATED_WORKER_SCOPE: &str = "/";

/// Keep service workers off loopback development origins. Stale workers can
/// survive a rebuild and turn otherwise healthy localhost navigations into a
/// browser-level `ERR_FAILED` response.
pub fn service_workers_enabled(hostname: &str) -> bool {
    !matches!(
        hostname.trim().to_ascii_lowercase().as_str(),
        "localhost" | "127.0.0.1" | "::1" | "[::1]"
    )
}

pub fn chat_topic_icon_svg(name: &str) -> String {
    // Mirrors epsx_templates::lucide_icon for the 6 chat topic icons.
    // Returns a 20px SVG with currentColor stroke so the parent's `color` drives visibility.
    let body = match name {
        "message-circle" => r#"<path d="M7.9 20A9 9 0 1 0 4 16.1L2 22Z"/>"#,
        "credit-card" => {
            r#"<rect width="20" height="14" x="2" y="5" rx="2"/><line x1="2" x2="22" y1="10" y2="10"/>"#
        }
        "user" => {
            r#"<path d="M19 21v-2a4 4 0 0 0-4-4H9a4 4 0 0 0-4 4v2"/><circle cx="12" cy="7" r="4"/>"#
        }
        "bar-chart" | "bar-chart-2" | "bar-chart-3" | "chart-column" => {
            r#"<path d="M3 3v18h18"/><path d="M18 17V9"/><path d="M13 17V5"/><path d="M8 17v-3"/>"#
        }
        "bug" => {
            r#"<path d="M12 20v-9"/><path d="M14 7a4 4 0 0 1 4 4v3a6 6 0 0 1-12 0v-3a4 4 0 0 1 4-4z"/><path d="M14.12 3.88 16 2"/><path d="M21 21a4 4 0 0 0-3.81-4"/><path d="M21 5a4 4 0 0 1-3.55 3.97"/><path d="M22 13h-4"/><path d="M3 21a4 4 0 0 1 3.81-4"/><path d="M3 5a4 4 0 0 0 3.55 3.97"/><path d="M6 13H2"/><path d="m8 2 1.88 1.88"/><path d="M9 7.13V6a3 3 0 1 1 6 0v1.13"/>"#
        }
        "lightbulb" => {
            r#"<path d="M15 14c.2-1 .7-1.7 1.5-2.5 1-.9 1.5-2.2 1.5-3.5A6 6 0 0 0 6 8c0 1 .2 2.2 1.5 3.5.7.7 1.3 1.5 1.5 2.5"/><path d="M9 18h6"/><path d="M10 22h4"/>"#
        }
        "headset" => {
            r#"<path d="M3 14h3a2 2 0 0 1 2 2v3a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2Z"/><path d="M21 14h-3a2 2 0 0 0-2 2v3a2 2 0 0 0 2 2h1a2 2 0 0 0 2-2Z"/><path d="M3 14v-2a9 9 0 0 1 18 0v2"/><path d="M21 14v-2"/>"#
        }
        _ => r#"<path d="M7.9 20A9 9 0 1 0 4 16.1L2 22Z"/>"#,
    };
    format!(
        r#"<span class="epsx-icon" style="width:20px;height:20px;"><svg xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="lucide lucide-{name}" aria-hidden="true">{body}</svg></span>"#,
        name = name,
        body = body
    )
}

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

/// Turn the BFF's deliberately safe error code into a useful browser message.
/// The upstream response never contains tokens or provider details, so showing
/// this code helps a user distinguish a rejected signature from a verifier
/// outage without exposing authentication material. Known infrastructure
/// outages are surfaced with a clearer plain-text message; everything else
/// falls back to the closed code/HTTP pair for debugging.
pub fn auth_http_error(status: u16, code: Option<&str>) -> String {
    match code.map(str::trim).filter(|code| !code.is_empty()) {
        Some("auth_upstream_unavailable") => {
            "Sign-in service is temporarily unavailable. Please try again in a moment.".to_string()
        }
        Some("challenge_rejected") => {
            "Wallet challenge was rejected. Please reconnect and try again.".to_string()
        }
        Some("authentication_rejected") => {
            "Wallet signature was rejected. Please reconnect and try again.".to_string()
        }
        Some("missing_refresh_token") => {
            "Your session expired. Please reconnect your wallet.".to_string()
        }
        Some(code) => format!("Sign-in failed: {code} (HTTP {status})"),
        None => format!("Sign-in failed (HTTP {status})"),
    }
}

/// Classify a formatted BFF error as the closed transient upstream pair so the
/// browser runtime can decide whether a single retry is worthwhile. Matches any
/// `auth_upstream_unavailable` code (any HTTP status, e.g. 502, 530, 503) or
/// the friendly outage message so retries work either before or after
/// `auth_http_error` rewrites the code and regardless of the mapped gateway
/// status.
pub fn is_transient_upstream_error_pub(message: &str) -> bool {
    let trimmed = message.trim();
    let closed = trimmed.contains("auth_upstream_unavailable");
    let friendly = trimmed.starts_with("Sign-in service is temporarily unavailable");
    closed || friendly
}

/// Only provider transports implemented by the generated browser runtime may
/// be selected from DOM attributes. Decorative options must never silently
/// fall back to a different injected wallet.
pub fn supported_injected_wallet(raw: &str) -> Option<&'static str> {
    raw.trim()
        .eq_ignore_ascii_case("metamask")
        .then_some("metamask")
}

/// Wallet addresses are case-insensitive for the purpose of matching the
/// account exposed by an injected provider to the account selected for SIWE.
/// Empty values never match: treating two missing addresses as one identity
/// would let stale DOM/cookie state pass the browser-side consistency check.
pub fn same_wallet_address(left: &str, right: &str) -> bool {
    let left = left.trim();
    let right = right.trim();
    !left.is_empty() && !right.is_empty() && left.eq_ignore_ascii_case(right)
}

/// Prefer the provider's explicit selected address when it is one of the
/// permitted accounts. Falling back to the first permitted account preserves
/// the injected-provider convention without trusting a stale cached address.
pub fn select_wallet_account<'a>(
    accounts: &'a [String],
    selected_address: Option<&str>,
) -> Option<&'a str> {
    selected_address
        .filter(|selected| !selected.trim().is_empty())
        .and_then(|selected| {
            accounts
                .iter()
                .find(|account| same_wallet_address(account, selected))
        })
        .or_else(|| accounts.first())
        .map(String::as_str)
}

/// Canonical client-side shape check for watchlist symbols. The backend and
/// BFF remain authoritative; this prevents malformed values from reaching a
/// request path and keeps the progressive UI aligned with their contract.
pub fn normalize_watchlist_symbol(value: &str) -> Option<String> {
    let symbol = value.trim().to_ascii_uppercase();
    let mut characters = symbol.chars();
    let first = characters.next()?;
    if symbol.len() > 20
        || !first.is_ascii_alphanumeric()
        || !characters
            .all(|character| character.is_ascii_alphanumeric() || matches!(character, '.' | '-'))
    {
        return None;
    }
    Some(symbol)
}

pub fn watchlist_mutation(value: &str, currently_watched: bool) -> Option<(&'static str, String)> {
    let symbol = normalize_watchlist_symbol(value)?;
    if currently_watched {
        Some(("DELETE", format!("/api/users/watchlist/{symbol}")))
    } else {
        Some(("POST", "/api/users/watchlist".to_string()))
    }
}

/// Encode an ERC-20 `transfer(address,uint256)` call without floating-point
/// arithmetic. Checkout prices are bounded decimal strings from the backend.
pub fn erc20_transfer_calldata(
    receiver: &str,
    decimal_amount: &str,
    decimals: u8,
) -> Option<String> {
    if receiver.len() != 42
        || !receiver.starts_with("0x")
        || !receiver[2..].bytes().all(|byte| byte.is_ascii_hexdigit())
        || decimals > 38
    {
        return None;
    }
    let amount = decimal_amount.trim();
    if amount.is_empty() || amount.starts_with('-') || amount.starts_with('+') {
        return None;
    }
    let mut parts = amount.split('.');
    let whole = parts.next()?;
    let fraction = parts.next().unwrap_or_default();
    if parts.next().is_some()
        || whole.is_empty()
        || !whole.bytes().all(|byte| byte.is_ascii_digit())
        || !fraction.bytes().all(|byte| byte.is_ascii_digit())
        || fraction.len() > usize::from(decimals)
    {
        return None;
    }
    let scale = 10_u128.checked_pow(u32::from(decimals))?;
    let whole_units = whole.parse::<u128>().ok()?.checked_mul(scale)?;
    let fraction_units = if fraction.is_empty() {
        0
    } else {
        fraction
            .parse::<u128>()
            .ok()?
            .checked_mul(10_u128.checked_pow(u32::from(decimals) - fraction.len() as u32)?)?
    };
    let base_units = whole_units.checked_add(fraction_units)?;
    if base_units == 0 {
        return None;
    }
    let address_word = format!("{:0>64}", receiver[2..].to_ascii_lowercase());
    Some(format!("0xa9059cbb{address_word}{base_units:064x}"))
}

#[cfg(target_arch = "wasm32")]
mod browser {
    use super::{
        auth_http_error, erc20_transfer_calldata, normalize_watchlist_symbol, safe_return_path,
        same_wallet_address, select_wallet_account, service_workers_enabled,
        supported_injected_wallet, watchlist_mutation, GENERATED_WORKER_SCOPE,
    };
    use js_sys::{Array, Function, Object, Promise, Reflect, Uint8Array};
    use serde::Deserialize;
    use serde_json::{json, Value};
    use std::cell::{Cell, RefCell};
    use wasm_bindgen::{closure::Closure, prelude::*, JsCast};
    use wasm_bindgen_futures::{spawn_local, JsFuture};
    use web_sys::{
        DataTransfer, Document, DragEvent, Element, Event, File, FormData, HtmlButtonElement,
        HtmlElement, HtmlInputElement, HtmlSelectElement, HtmlTextAreaElement, KeyboardEvent,
        PointerEvent, Request, RequestInit, Response, Window,
    };

    const GENERATED_WORKER: &str = "/runtime/epsx_service_worker_bootstrap.v3.js?rev=3";

    thread_local! {
        static DEVELOPER_MUTATION_CONFIRMED: Cell<bool> = const { Cell::new(false) };
        static WATCHLIST_DRAG: RefCell<Option<WatchlistDragState>> = const { RefCell::new(None) };
        static WALLET_AUTH_IN_PROGRESS: Cell<bool> = const { Cell::new(false) };
        static WALLET_SYNC_IN_PROGRESS: Cell<bool> = const { Cell::new(false) };
    }

    struct WatchlistDragState {
        source: Element,
        snapshot: Value,
        kind: String,
    }

    struct WalletAuthGuard;

    impl WalletAuthGuard {
        fn begin() -> Self {
            WALLET_AUTH_IN_PROGRESS.with(|value| value.set(true));
            Self
        }
    }

    impl Drop for WalletAuthGuard {
        fn drop(&mut self) {
            WALLET_AUTH_IN_PROGRESS.with(|value| value.set(false));
        }
    }

    #[derive(Deserialize)]
    struct WalletCookie {
        #[serde(default)]
        address: String,
        #[serde(default)]
        connector_id: String,
        #[serde(default)]
        chain_id: Option<String>,
    }

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
        bind_keys(&document)?;
        bind_watchlist_changes(&document)?;
        bind_watchlist_drag(&document)?;
        bind_watchlist_pointer_drag(&document)?;
        bind_wallet_provider(&document);
        let _ = bind_chat(&document);
        register_worker(&window);
        start_route_tasks(&window, &document);
        Ok(())
    }

    fn bind_clicks(document: &Document) -> Result<(), JsValue> {
        let click_document = document.clone();
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
                "[data-docs-copy-code],[data-docs-copy-response],[data-push-action],",
                "[data-developer-create],[data-developer-revoke],[data-developer-try],",
                "[data-watchlist-add],[data-watchlist-toggle],",
                "[data-watchlist-group-create],[data-watchlist-group-rename],",
                "[data-watchlist-group-delete],[data-watchlist-groups-save],",
                "[data-watchlist-remove-membership],[data-watchlist-move-item],",
                "[data-watchlist-move-group]"
            )) else {
                close_dropdowns(&click_document, None);
                close_nav_groups(&click_document, None);
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
                })
                .or_else(|| {
                    element
                        .has_attribute("data-developer-create")
                        .then(|| "developer-create".into())
                })
                .or_else(|| {
                    element
                        .has_attribute("data-developer-revoke")
                        .then(|| "developer-revoke".into())
                })
                .or_else(|| {
                    element
                        .has_attribute("data-developer-try")
                        .then(|| "developer-try".into())
                })
                .or_else(|| {
                    element
                        .has_attribute("data-watchlist-add")
                        .then(|| "watchlist-add".into())
                })
                .or_else(|| {
                    element
                        .has_attribute("data-watchlist-toggle")
                        .then(|| "watchlist-toggle".into())
                })
                .or_else(|| {
                    element
                        .has_attribute("data-watchlist-group-create")
                        .then(|| "watchlist-group-create".into())
                })
                .or_else(|| {
                    element
                        .has_attribute("data-watchlist-group-rename")
                        .then(|| "watchlist-group-rename".into())
                })
                .or_else(|| {
                    element
                        .has_attribute("data-watchlist-group-delete")
                        .then(|| "watchlist-group-delete".into())
                })
                .or_else(|| {
                    element
                        .has_attribute("data-watchlist-groups-save")
                        .then(|| "watchlist-groups-save".into())
                })
                .or_else(|| {
                    element
                        .has_attribute("data-watchlist-remove-membership")
                        .then(|| "watchlist-remove-membership".into())
                })
                .or_else(|| {
                    element
                        .get_attribute("data-watchlist-move-item")
                        .map(|direction| format!("watchlist-move-item-{direction}"))
                })
                .or_else(|| {
                    element
                        .get_attribute("data-watchlist-move-group")
                        .map(|direction| format!("watchlist-move-group-{direction}"))
                });
            let Some(action) = action else { return };
            if action != "toggle-dropdown" {
                close_dropdowns(&click_document, None);
            }
            if action != "toggle-nav" {
                close_nav_groups(&click_document, None);
            }
            if !matches!(action.as_str(), "native-link" | "native-submit") {
                event.prevent_default();
            }
            dispatch_action(element, &action);
        });
        document.add_event_listener_with_callback("click", closure.as_ref().unchecked_ref())?;
        closure.forget();
        Ok(())
    }

    fn bind_keys(document: &Document) -> Result<(), JsValue> {
        let key_document = document.clone();
        let closure = Closure::<dyn FnMut(KeyboardEvent)>::new(move |event: KeyboardEvent| {
            if event.key() == "Escape" {
                cancel_watchlist_drag();
                close_dropdowns(&key_document, None);
                close_nav_groups(&key_document, None);
            }
        });
        document.add_event_listener_with_callback("keydown", closure.as_ref().unchecked_ref())?;
        closure.forget();
        Ok(())
    }

    fn dispatch_action(element: Element, action: &str) {
        match action {
            "theme-toggle" => toggle_theme(),
            "toggle-nav" => toggle_nav(&element),
            "toggle-dropdown" => toggle_dropdown(&element),
            "toggle-mobile-menu" => toggle_target(&element, "epsx-mobile-sheet", "open"),
            "open-sheet" | "open-modal" => set_named_target(&element, true),
            "close-sheet" | "close-modal" => set_named_target(&element, false),
            "activate-tab" => activate_tab(&element),
            "copy" => copy_value(&element),
            "share" => share_value(&element),
            "connect-wallet" => spawn_local(connect_wallet(element)),
            "logout" => {
                let target = element
                    .get_attribute("data-epsx-logout-target")
                    .unwrap_or_else(|| "/".into());
                spawn_local(logout(target));
            }
            "session-recover" => spawn_local(recover_session()),
            "notification-mutation" => spawn_local(notification_mutation(element)),
            "create-checkout" => spawn_local(create_checkout(element)),
            "submit-plan-payment" => spawn_local(submit_plan_payment(element)),
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
            "developer-create" => spawn_local(create_developer_key(element)),
            "developer-revoke" => spawn_local(revoke_developer_key(element)),
            "developer-try" => spawn_local(try_developer_operation(element)),
            "watchlist-add" | "watchlist-toggle" => spawn_local(update_watchlist(element)),
            "watchlist-group-create" => spawn_local(create_watchlist_group(element)),
            "watchlist-group-rename" => spawn_local(rename_watchlist_group(element)),
            "watchlist-group-delete" => spawn_local(delete_watchlist_group(element)),
            "watchlist-groups-save" => spawn_local(save_symbol_groups(element)),
            "watchlist-remove-membership" => spawn_local(remove_group_membership(element)),
            "watchlist-move-item-up" => move_watchlist_item_keyboard(element, false),
            "watchlist-move-item-down" => move_watchlist_item_keyboard(element, true),
            "watchlist-move-group-up" => move_watchlist_group_keyboard(element, false),
            "watchlist-move-group-down" => move_watchlist_group_keyboard(element, true),
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
        let _ = target.toggle_attribute_with_force("hidden", !open);
        let _ = target.set_attribute("aria-hidden", if open { "false" } else { "true" });
        let _ = element.set_attribute("aria-expanded", if open { "true" } else { "false" });
    }

    /// Desktop navigation menus are mutually exclusive. Mobile accordions and
    /// admin sidebar groups also use `toggle-nav`, so they retain the generic
    /// controlled-element behavior unless the trigger belongs to an
    /// `.epsx-nav-wrap` desktop group.
    fn toggle_nav(element: &Element) {
        let Some((_, document)) = window_document() else {
            return;
        };
        let Some(target_id) = element.get_attribute("aria-controls") else {
            return;
        };
        let Some(target) = document.get_element_by_id(&target_id) else {
            return;
        };
        let Ok(Some(wrapper)) = element.closest(".epsx-nav-wrap") else {
            toggle_controlled(element, "open");
            return;
        };
        let open = element.get_attribute("aria-expanded").as_deref() != Some("true")
            || target.has_attribute("hidden");
        close_nav_groups(&document, open.then_some(target_id.as_str()));
        let _ = wrapper.class_list().toggle_with_force("open", open);
        let _ = target.class_list().toggle_with_force("open", open);
        let _ = target.toggle_attribute_with_force("hidden", !open);
        let _ = target.set_attribute("aria-hidden", if open { "false" } else { "true" });
        let _ = element.set_attribute("aria-expanded", if open { "true" } else { "false" });
    }

    fn close_nav_groups(document: &Document, except_panel_id: Option<&str>) {
        let Ok(groups) = document.query_selector_all(".epsx-nav-wrap") else {
            return;
        };
        for index in 0..groups.length() {
            let Some(wrapper) = groups
                .item(index)
                .and_then(|node| node.dyn_into::<Element>().ok())
            else {
                continue;
            };
            let trigger = wrapper
                .query_selector(".epsx-nav-trigger[aria-controls]")
                .ok()
                .flatten();
            let panel_id = trigger
                .as_ref()
                .and_then(|trigger| trigger.get_attribute("aria-controls"));
            if panel_id.as_deref() == except_panel_id {
                continue;
            }
            let _ = wrapper.class_list().remove_1("open");
            if let Some(trigger) = trigger {
                let _ = trigger.set_attribute("aria-expanded", "false");
            }
            if let Some(panel) = panel_id.and_then(|id| document.get_element_by_id(&id)) {
                let _ = panel.class_list().remove_1("open");
                let _ = panel.set_attribute("hidden", "");
                let _ = panel.set_attribute("aria-hidden", "true");
            }
        }
    }

    fn toggle_dropdown(element: &Element) {
        let Some((_, document)) = window_document() else {
            return;
        };
        let Some(target_id) = element.get_attribute("aria-controls") else {
            return;
        };
        let Some(target) = document.get_element_by_id(&target_id) else {
            return;
        };
        let open = target.has_attribute("hidden") || !target.class_list().contains("open");
        close_dropdowns(&document, open.then_some(target_id.as_str()));
        if open {
            let _ = target.class_list().add_1("open");
            let _ = target.remove_attribute("hidden");
            let _ = target.set_attribute("aria-hidden", "false");
            let _ = element.set_attribute("aria-expanded", "true");
        }
    }

    fn close_dropdowns(document: &Document, except_id: Option<&str>) {
        if let Ok(nodes) = document.query_selector_all("[data-epsx-dropdown]") {
            for index in 0..nodes.length() {
                let Some(dropdown) = nodes
                    .item(index)
                    .and_then(|node| node.dyn_into::<Element>().ok())
                else {
                    continue;
                };
                if dropdown.id().as_str() == except_id.unwrap_or_default() {
                    continue;
                }
                let _ = dropdown.class_list().remove_1("open");
                let _ = dropdown.set_attribute("hidden", "");
                let _ = dropdown.set_attribute("aria-hidden", "true");
            }
        }
        if let Ok(nodes) =
            document.query_selector_all("[data-epsx-action=\"toggle-dropdown\"][aria-controls]")
        {
            for index in 0..nodes.length() {
                let Some(trigger) = nodes
                    .item(index)
                    .and_then(|node| node.dyn_into::<Element>().ok())
                else {
                    continue;
                };
                if trigger.get_attribute("aria-controls").as_deref() == except_id {
                    continue;
                }
                let _ = trigger.set_attribute("aria-expanded", "false");
            }
        }
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

    async fn fetch_json_with_headers(
        path: &str,
        method: &str,
        body: Option<Value>,
        headers: &[(&str, &str)],
    ) -> Result<Value, JsValue> {
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
        for (name, value) in headers {
            request.headers().set(name, value)?;
        }
        let response = JsFuture::from(window.fetch_with_request(&request))
            .await?
            .dyn_into::<Response>()?;
        if !response.ok() {
            let status = response.status();
            let code = match response.json() {
                Ok(body) => JsFuture::from(body)
                    .await
                    .ok()
                    .and_then(|body| serde_wasm_bindgen::from_value::<Value>(body).ok())
                    .and_then(|body| body.get("error").and_then(Value::as_str).map(str::to_owned)),
                Err(_) => None,
            };
            return Err(JsValue::from_str(&auth_http_error(status, code.as_deref())));
        }
        let value = JsFuture::from(response.json()?).await?;
        serde_wasm_bindgen::from_value(value).map_err(|error| JsValue::from_str(&error.to_string()))
    }

    async fn fetch_json(path: &str, method: &str, body: Option<Value>) -> Result<Value, JsValue> {
        fetch_json_with_headers(path, method, body, &[]).await
    }

    /// Retry a fetch once after a short delay when the BFF returns the
    /// closed "auth_upstream_unavailable" code (any 5xx, e.g. 502, 530).
    /// This handles the startup race where the user clicks Connect Wallet
    /// before the backend has finished warming up, plus short network blips
    /// and Cloudflare tunnel 530 errors. Deterministic failures (4xx,
    /// malformed JSON) are surfaced immediately without retry.
    async fn fetch_json_with_upstream_retry(
        path: &str,
        method: &str,
        body: Option<Value>,
    ) -> Result<Value, JsValue> {
        match fetch_json(path, method, body.clone()).await {
            Ok(value) => Ok(value),
            Err(error) => {
                let message = error.as_string().unwrap_or_default();
                if is_transient_upstream_error(&message) {
                    delay(750).await;
                    return fetch_json(path, method, body).await;
                }
                Err(error)
            }
        }
    }

    fn is_transient_upstream_error(message: &str) -> bool {
        let trimmed = message.trim();
        trimmed.contains("auth_upstream_unavailable")
            || trimmed.starts_with("Sign-in service is temporarily unavailable")
    }

    fn form_value(form: &Element, name: &str) -> Option<String> {
        form.query_selector(&format!("[name=\"{name}\"]"))
            .ok()
            .flatten()
            .and_then(|element| {
                element
                    .clone()
                    .dyn_into::<HtmlInputElement>()
                    .ok()
                    .map(|input| input.value())
                    .or_else(|| {
                        element
                            .dyn_into::<HtmlTextAreaElement>()
                            .ok()
                            .map(|input| input.value())
                    })
            })
    }

    fn new_idempotency_key(prefix: &str) -> String {
        format!(
            "developer.{prefix}.{}.{:016x}",
            js_sys::Date::now() as u64,
            (js_sys::Math::random() * u64::MAX as f64) as u64
        )
    }

    async fn create_developer_key(button: Element) {
        let Ok(Some(form)) = button.closest("[data-developer-create-form]") else {
            return;
        };
        let name = form_value(&form, "name").unwrap_or_default();
        let description = form_value(&form, "description").filter(|value| !value.is_empty());
        let expires_at = form_value(&form, "expires_at").filter(|value| !value.is_empty());
        let idempotency_key = form_value(&form, "idempotency_key")
            .filter(|value| !value.is_empty())
            .unwrap_or_else(|| new_idempotency_key("create"));
        let mut scopes = Vec::new();
        if let Ok(nodes) = form.query_selector_all("input[name=\"scopes\"]") {
            for index in 0..nodes.length() {
                let Some(input) = nodes
                    .item(index)
                    .and_then(|node| node.dyn_into::<HtmlInputElement>().ok())
                else {
                    continue;
                };
                if input.checked() {
                    scopes.push(Value::String(input.value()));
                }
            }
        }
        let result = fetch_json_with_headers(
            "/api/v1/developer/keys",
            "POST",
            Some(json!({
                "name": name,
                "description": description,
                "scopes": scopes,
                "expires_at": expires_at
            })),
            &[("idempotency-key", idempotency_key.as_str())],
        )
        .await;
        match result {
            Ok(value) => {
                let secret = value
                    .get("data")
                    .and_then(|data| data.get("secret"))
                    .and_then(Value::as_str);
                let Some((_, document)) = window_document() else {
                    return;
                };
                let Some(panel) = document.get_element_by_id("developer-secret-once") else {
                    return;
                };
                if let Some(secret) = secret {
                    if let Some(node) = document.get_element_by_id("developer-secret-value") {
                        node.set_text_content(Some(secret));
                    }
                    if let Some(copy) = document.get_element_by_id("developer-secret-copy") {
                        let _ = copy.set_attribute("data-copy", secret);
                    }
                } else if let Some(node) = document.get_element_by_id("developer-secret-value") {
                    node.set_text_content(Some(
                        "This request was already completed; the secret cannot be shown again.",
                    ));
                }
                let _ = panel.remove_attribute("hidden");
            }
            Err(error) => set_status(
                &error
                    .as_string()
                    .unwrap_or_else(|| "API key could not be created.".into()),
                true,
            ),
        }
    }

    async fn revoke_developer_key(button: Element) {
        let Some(id) = button.get_attribute("data-key-id") else {
            return;
        };
        let confirmed = web_sys::window()
            .and_then(|window| {
                window
                    .confirm_with_message("Revoke this API key now? This cannot be undone.")
                    .ok()
            })
            .unwrap_or(false);
        if !confirmed {
            return;
        }
        let idempotency_key = button
            .closest("[data-developer-revoke-form]")
            .ok()
            .flatten()
            .and_then(|form| form_value(&form, "idempotency_key"))
            .unwrap_or_else(|| new_idempotency_key("revoke"));
        if fetch_json_with_headers(
            &format!("/api/v1/developer/keys/{id}/revoke"),
            "POST",
            Some(json!({"reason": "Revoked from Developer Portal"})),
            &[("idempotency-key", idempotency_key.as_str())],
        )
        .await
        .is_ok()
        {
            reload();
        } else {
            set_status("The API key could not be revoked.", true);
        }
    }

    async fn try_developer_operation(button: Element) {
        let Some(operation_id) = button.get_attribute("data-operation-id") else {
            return;
        };
        let mutation = button.get_attribute("data-operation-mutation").as_deref() == Some("true");
        if mutation {
            let already_confirmed = DEVELOPER_MUTATION_CONFIRMED.with(Cell::get);
            if !already_confirmed {
                let confirmed = web_sys::window()
                    .and_then(|window| {
                        window
                            .confirm_with_message(
                                "Try It is about to mutate backend data. Continue for this tab?",
                            )
                            .ok()
                    })
                    .unwrap_or(false);
                if !confirmed {
                    return;
                }
                DEVELOPER_MUTATION_CONFIRMED.with(|value| value.set(true));
            }
        }
        let Some((_, document)) = window_document() else {
            return;
        };
        let api_key = document
            .get_element_by_id("developer-try-api-key")
            .and_then(|element| element.dyn_into::<HtmlInputElement>().ok())
            .map(|input| input.value())
            .unwrap_or_default();
        let Ok(Some(container)) = button.closest("article") else {
            return;
        };
        let query = container
            .query_selector("[data-try-query]")
            .ok()
            .flatten()
            .and_then(|element| element.dyn_into::<HtmlInputElement>().ok())
            .map(|input| input.value())
            .filter(|value| !value.is_empty());
        let response = container
            .query_selector("[data-try-response]")
            .ok()
            .flatten();
        let body_text = container
            .query_selector("[data-try-body]")
            .ok()
            .flatten()
            .and_then(|element| element.dyn_into::<HtmlTextAreaElement>().ok())
            .map(|input| input.value())
            .filter(|value| !value.is_empty());
        let body = match body_text {
            Some(value) => match serde_json::from_str::<Value>(&value) {
                Ok(value) => Some(value),
                Err(_) => {
                    if let Some(response) = response.as_ref() {
                        response.set_text_content(Some("Request body must be valid JSON."));
                        let _ = response.remove_attribute("hidden");
                    }
                    return;
                }
            },
            None => None,
        };
        let idempotency_key = mutation.then(|| new_idempotency_key("try"));
        let result = fetch_json(
            "/api/v1/developer/try",
            "POST",
            Some(json!({
                "operation_id": operation_id,
                "api_key": api_key,
                "query": query,
                "body": body,
                "confirm_mutation": mutation,
                "idempotency_key": idempotency_key
            })),
        )
        .await;
        if let Some(response) = response {
            let text = match result {
                Ok(value) => serde_json::to_string_pretty(&value)
                    .unwrap_or_else(|_| "Response could not be displayed.".into()),
                Err(error) => error
                    .as_string()
                    .unwrap_or_else(|| "Request failed.".into()),
            };
            response.set_text_content(Some(&text));
            let _ = response.remove_attribute("hidden");
        }
    }

    fn provider_flag(provider: &JsValue, name: &str) -> bool {
        Reflect::get(provider, &JsValue::from_str(name))
            .ok()
            .and_then(|value| value.as_bool())
            .unwrap_or(false)
    }

    fn injected_wallet_provider(connector: &str) -> Result<JsValue, JsValue> {
        let connector = supported_injected_wallet(connector)
            .ok_or_else(|| JsValue::from_str("This wallet connector is not available"))?;
        let window = web_sys::window().ok_or("window unavailable")?;
        let ethereum = Reflect::get(window.as_ref(), &JsValue::from_str("ethereum"))?;
        if ethereum.is_null() || ethereum.is_undefined() {
            return Err(JsValue::from_str(
                "MetaMask was not detected. Install or enable the MetaMask extension and reload.",
            ));
        }

        if connector == "metamask" {
            let providers = Reflect::get(&ethereum, &JsValue::from_str("providers"))
                .unwrap_or(JsValue::UNDEFINED);
            if Array::is_array(&providers) {
                let providers = Array::from(&providers);
                for index in 0..providers.length() {
                    let provider = providers.get(index);
                    if provider_flag(&provider, "isMetaMask") {
                        return Ok(provider);
                    }
                }
            }
            if provider_flag(&ethereum, "isMetaMask") {
                return Ok(ethereum);
            }
            if Reflect::get(&ethereum, &JsValue::from_str("request"))
                .ok()
                .is_some_and(|f| f.is_function())
            {
                return Ok(ethereum);
            }
            return Err(JsValue::from_str(
                "MetaMask was not detected. Enable MetaMask for this site and reload.",
            ));
        }

        Err(JsValue::from_str("This wallet connector is not available"))
    }

    async fn wallet_request(
        provider: &JsValue,
        method: &str,
        params: Array,
    ) -> Result<JsValue, JsValue> {
        let request = Object::new();
        Reflect::set(
            &request,
            &JsValue::from_str("method"),
            &JsValue::from_str(method),
        )?;
        Reflect::set(&request, &JsValue::from_str("params"), &params)?;
        let function =
            Reflect::get(provider, &JsValue::from_str("request"))?.dyn_into::<Function>()?;
        let promise = function.call1(provider, &request)?.dyn_into::<Promise>()?;
        JsFuture::from(promise).await
    }

    fn provider_selected_address(provider: &JsValue) -> Option<String> {
        Reflect::get(provider, &JsValue::from_str("selectedAddress"))
            .ok()
            .and_then(|value| value.as_string())
            .filter(|value| !value.trim().is_empty())
    }

    fn account_from_provider_result(
        provider: &JsValue,
        result: JsValue,
    ) -> Result<String, JsValue> {
        if !Array::is_array(&result) {
            return Err(JsValue::from_str("wallet returned malformed accounts"));
        }
        let accounts = Array::from(&result)
            .iter()
            .filter_map(|value| value.as_string())
            .filter(|value| !value.trim().is_empty())
            .collect::<Vec<_>>();
        select_wallet_account(&accounts, provider_selected_address(provider).as_deref())
            .map(str::to_string)
            .ok_or_else(|| JsValue::from_str("wallet returned no account"))
    }

    async fn current_provider_account(provider: &JsValue) -> Result<String, JsValue> {
        let accounts = wallet_request(provider, "eth_accounts", Array::new()).await?;
        account_from_provider_result(provider, accounts)
    }

    async fn request_provider_account(
        provider: &JsValue,
        force_selection: bool,
    ) -> Result<String, JsValue> {
        if force_selection {
            // Admin authentication must not silently reuse a persisted account
            // permission. EIP-2255 asks MetaMask to show its account selector
            // again, so a wallet switch is confirmed before SIWE begins.
            let permissions = Array::new();
            let requested = Object::new();
            Reflect::set(
                &requested,
                &JsValue::from_str("eth_accounts"),
                &Object::new(),
            )?;
            permissions.push(&requested);
            wallet_request(provider, "wallet_requestPermissions", permissions).await?;
        }
        let accounts = wallet_request(provider, "eth_requestAccounts", Array::new()).await?;
        account_from_provider_result(provider, accounts)
    }

    async fn ensure_provider_account(
        provider: &JsValue,
        expected_address: &str,
    ) -> Result<(), JsValue> {
        let current = current_provider_account(provider).await?;
        if same_wallet_address(&current, expected_address) {
            Ok(())
        } else {
            Err(JsValue::from_str(
                "MetaMask account changed during sign-in. Please try again.",
            ))
        }
    }

    fn document_cookie(document: &Document, name: &str) -> Option<String> {
        let cookies = Reflect::get(document.as_ref(), &JsValue::from_str("cookie"))
            .ok()?
            .as_string()?;
        cookies.split(';').find_map(|part| {
            let (key, value) = part.trim().split_once('=')?;
            (key == name).then(|| value.to_string())
        })
    }

    fn wallet_cookie(document: &Document) -> Option<WalletCookie> {
        let encoded = document_cookie(document, "epsx_wallet")?;
        let decoded = js_sys::decode_uri_component(&encoded).ok()?.as_string()?;
        serde_json::from_str(&decoded).ok()
    }

    fn write_wallet_cookie(document: &Document, address: &str, connector: &str, chain_id: u64) {
        let cookie = format!(
            "epsx_wallet={}; Path=/; Max-Age=86400; SameSite=Lax",
            js_sys::encode_uri_component(
                &json!({
                    "address": address,
                    "connector_id": connector,
                    "chain_id": chain_id.to_string()
                })
                .to_string()
            )
        );
        let _ = Reflect::set(
            document.as_ref(),
            &JsValue::from_str("cookie"),
            &JsValue::from_str(&cookie),
        );
    }

    fn clear_wallet_cookie(document: &Document) {
        let _ = Reflect::set(
            document.as_ref(),
            &JsValue::from_str("cookie"),
            &JsValue::from_str("epsx_wallet=; Path=/; Max-Age=0; SameSite=Lax"),
        );
    }

    fn session_wallet(document: &Document) -> Option<String> {
        document
            .query_selector("[data-epsx-session-wallet]")
            .ok()
            .flatten()
            .and_then(|element| element.get_attribute("data-epsx-session-wallet"))
            .filter(|value| !value.trim().is_empty())
    }

    async fn provider_chain_id(provider: &JsValue) -> Result<u64, JsValue> {
        let chain_hex = wallet_request(provider, "eth_chainId", Array::new())
            .await?
            .as_string()
            .ok_or("wallet returned no chain")?;
        u64::from_str_radix(chain_hex.trim_start_matches("0x"), 16)
            .map_err(|_| JsValue::from_str("invalid wallet chain"))
    }

    async fn clear_session_after_wallet_change(document: &Document) {
        clear_wallet_cookie(document);
        set_status(
            "Wallet account changed. Ending the previous admin session…",
            false,
        );
        // The admin BFF clears its HttpOnly session cookies even when the
        // upstream logout call fails. Always reload after the response so the
        // server renders from the newly-cleared local session.
        let _ = fetch_json("/api/v1/auth/logout", "POST", Some(json!({}))).await;
        reload();
    }

    async fn synchronize_wallet_state(provider: JsValue, explicit_disconnect: bool) {
        if WALLET_AUTH_IN_PROGRESS.with(Cell::get)
            || WALLET_SYNC_IN_PROGRESS.with(|value| value.replace(true))
        {
            return;
        }

        let Some((_, document)) = window_document() else {
            WALLET_SYNC_IN_PROGRESS.with(|value| value.set(false));
            return;
        };
        let stored = wallet_cookie(&document);
        let authenticated_wallet = session_wallet(&document);
        if stored.is_none() && authenticated_wallet.is_none() {
            WALLET_SYNC_IN_PROGRESS.with(|value| value.set(false));
            return;
        }

        let account = if explicit_disconnect {
            None
        } else {
            current_provider_account(&provider).await.ok()
        };

        if authenticated_wallet.as_deref().is_some_and(|session| {
            explicit_disconnect
                || account
                    .as_deref()
                    .is_some_and(|current| !same_wallet_address(session, current))
        }) {
            clear_session_after_wallet_change(&document).await;
            WALLET_SYNC_IN_PROGRESS.with(|value| value.set(false));
            return;
        }

        if let Some(stored) = stored {
            match account {
                Some(account) => {
                    let chain_id = provider_chain_id(&provider).await.ok();
                    let stored_chain = stored.chain_id.as_deref().and_then(|id| id.parse().ok());
                    let identity_changed = !same_wallet_address(&stored.address, &account)
                        || !stored.connector_id.eq_ignore_ascii_case("metamask");
                    let chain_changed =
                        chain_id.is_some_and(|chain_id| Some(chain_id) != stored_chain);
                    if identity_changed || chain_changed {
                        write_wallet_cookie(
                            &document,
                            &account,
                            "metamask",
                            chain_id.or(stored_chain).unwrap_or_default(),
                        );
                        // A network switch during checkout must not discard the
                        // in-flight payment. Identity changes still require a
                        // full server render; chain-only changes update state.
                        if identity_changed {
                            reload();
                        }
                    }
                }
                None if explicit_disconnect || authenticated_wallet.is_none() => {
                    clear_wallet_cookie(&document);
                    reload();
                }
                None => {}
            }
        }
        WALLET_SYNC_IN_PROGRESS.with(|value| value.set(false));
    }

    fn bind_wallet_provider(document: &Document) {
        let watches_provider = document
            .query_selector("[data-wallet-provider-watch=\"metamask\"], [data-epsx-session-wallet]")
            .ok()
            .flatten()
            .is_some();
        if !watches_provider {
            return;
        }
        let Ok(provider) = injected_wallet_provider("metamask") else {
            if wallet_cookie(document).is_some() && session_wallet(document).is_none() {
                clear_wallet_cookie(document);
                reload();
            }
            return;
        };
        let Ok(on) = Reflect::get(&provider, &JsValue::from_str("on"))
            .and_then(|value| value.dyn_into::<Function>())
        else {
            return;
        };

        let accounts_provider = provider.clone();
        let accounts_changed = Closure::<dyn FnMut(JsValue)>::new(move |accounts: JsValue| {
            if WALLET_AUTH_IN_PROGRESS.with(Cell::get) {
                return;
            }
            let explicit_disconnect =
                Array::is_array(&accounts) && Array::from(&accounts).length() == 0;
            let provider = accounts_provider.clone();
            spawn_local(synchronize_wallet_state(provider, explicit_disconnect));
        });
        let _ = on.call2(
            &provider,
            &JsValue::from_str("accountsChanged"),
            accounts_changed.as_ref().unchecked_ref(),
        );
        accounts_changed.forget();

        let chain_provider = provider.clone();
        let chain_changed = Closure::<dyn FnMut(JsValue)>::new(move |_chain_id: JsValue| {
            if WALLET_AUTH_IN_PROGRESS.with(Cell::get) {
                return;
            }
            let provider = chain_provider.clone();
            spawn_local(synchronize_wallet_state(provider, false));
        });
        let _ = on.call2(
            &provider,
            &JsValue::from_str("chainChanged"),
            chain_changed.as_ref().unchecked_ref(),
        );
        chain_changed.forget();

        spawn_local(synchronize_wallet_state(provider, false));
    }

    fn set_wallet_busy(element: &Element, busy: bool) {
        let _ = element.set_attribute("aria-busy", if busy { "true" } else { "false" });
        if let Some(button) = element.dyn_ref::<HtmlButtonElement>() {
            button.set_disabled(busy);
        }
    }

    async fn connect_wallet(button: Element) {
        let _auth_guard = WalletAuthGuard::begin();
        set_wallet_busy(&button, true);
        set_status("Connecting wallet…", false);
        let result = async {
            let connector_attr = button.get_attribute("data-provider");
            let connector_raw = connector_attr
                .as_deref()
                .filter(|s| !s.trim().is_empty())
                .unwrap_or("metamask");
            let connector = supported_injected_wallet(connector_raw)
                .ok_or_else(|| JsValue::from_str("This wallet connector is not available"))?;
            let provider = injected_wallet_provider(connector)?;
            let force_selection = button
                .get_attribute("data-force-wallet-selection")
                .as_deref()
                == Some("true");
            let address = request_provider_account(&provider, force_selection).await?;
            let chain_id = provider_chain_id(&provider).await?;
            let challenge: Challenge = serde_json::from_value(
                fetch_json_with_upstream_retry(
                    "/api/v1/auth/challenge",
                    "POST",
                    Some(json!({"address": address})),
                )
                .await?,
            )
            .map_err(|error| JsValue::from_str(&error.to_string()))?;
            ensure_provider_account(&provider, &address).await?;
            let params = Array::new();
            params.push(&JsValue::from_str(&challenge.message));
            params.push(&JsValue::from_str(&address));
            let signature = wallet_request(&provider, "personal_sign", params)
                .await?
                .as_string()
                .ok_or("wallet returned no signature")?;
            ensure_provider_account(&provider, &address).await?;
            let session = fetch_json_with_upstream_retry(
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
                return Err(JsValue::from_str(
                    "verification did not establish a session",
                ));
            }
            let Some((window, document)) = window_document() else {
                return Err(JsValue::from_str("browser unavailable"));
            };
            write_wallet_cookie(&document, &address, connector, chain_id);
            let target = button
                .get_attribute("data-return-url")
                .filter(|url| !url.trim().is_empty())
                .or_else(|| {
                    let search = window.location().search().ok()?;
                    let query = search.strip_prefix('?').unwrap_or(&search);
                    query.split('&').find_map(|pair| {
                        let (key, value) = pair.split_once('=')?;
                        if key == "return_url" {
                            js_sys::decode_uri_component(value)
                                .ok()
                                .and_then(|v| v.as_string())
                        } else {
                            None
                        }
                    })
                })
                .unwrap_or_else(|| "/".to_string());
            window.location().replace(safe_return_path(&target))?;
            Ok::<(), JsValue>(())
        }
        .await;
        if let Err(error) = result {
            set_wallet_busy(&button, false);
            set_status(
                &error
                    .as_string()
                    .unwrap_or_else(|| "Wallet connection failed".into()),
                true,
            );
        }
    }

    async fn logout(target: String) {
        let document = window_document().map(|(_, document)| document);
        if let Some(document) = document.as_ref() {
            // Disconnect is explicit user intent. Do not preserve a browser-
            // readable wallet selection after the HttpOnly session is gone.
            clear_wallet_cookie(document);
        }
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

    fn bind_watchlist_changes(document: &Document) -> Result<(), JsValue> {
        let closure = Closure::<dyn FnMut(Event)>::new(move |event: Event| {
            let Some(target) = event
                .target()
                .and_then(|target| target.dyn_into::<Element>().ok())
            else {
                return;
            };
            let Ok(Some(select)) = target.closest("[data-watchlist-move-to-group]") else {
                return;
            };
            let Some(value) = select
                .dyn_ref::<HtmlSelectElement>()
                .map(HtmlSelectElement::value)
            else {
                return;
            };
            move_watchlist_item_to_group(select, &value);
        });
        document.add_event_listener_with_callback("change", closure.as_ref().unchecked_ref())?;
        closure.forget();
        Ok(())
    }

    fn elements(root: &Element, selector: &str) -> Vec<Element> {
        let Ok(nodes) = root.query_selector_all(selector) else {
            return vec![];
        };
        (0..nodes.length())
            .filter_map(|index| nodes.item(index))
            .filter_map(|node| node.dyn_into::<Element>().ok())
            .collect()
    }

    fn document_elements(document: &Document, selector: &str) -> Vec<Element> {
        let Ok(nodes) = document.query_selector_all(selector) else {
            return vec![];
        };
        (0..nodes.length())
            .filter_map(|index| nodes.item(index))
            .filter_map(|node| node.dyn_into::<Element>().ok())
            .collect()
    }

    fn list_symbols(list: &Element) -> Vec<Value> {
        elements(list, "[data-watchlist-item]")
            .into_iter()
            .filter(|card| !card.has_attribute("data-layout-exclude"))
            .filter_map(|card| card.get_attribute("data-symbol"))
            .map(Value::String)
            .collect()
    }

    fn collect_watchlist_layout() -> Option<Value> {
        let (_, document) = window_document()?;
        let container = document
            .query_selector("[data-watchlist-groups]")
            .ok()
            .flatten()?;
        let mut groups = Vec::new();
        let mut ungrouped = Vec::new();
        for section in elements(&container, "[data-watchlist-group]") {
            let group_id = section.get_attribute("data-group-id")?;
            let list = section
                .query_selector("[data-watchlist-items]")
                .ok()
                .flatten()?;
            let symbols = list_symbols(&list);
            if group_id == "ungrouped" {
                ungrouped = symbols;
            } else {
                groups.push(json!({"id": group_id, "symbols": symbols}));
            }
        }
        Some(json!({"groups": groups, "ungrouped": ungrouped}))
    }

    fn layout_is_canonical(value: &Value) -> bool {
        value.pointer("/data/groups").is_some_and(Value::is_array)
            && value
                .pointer("/data/ungrouped")
                .is_some_and(Value::is_array)
            && value.pointer("/data/watched").is_some_and(Value::is_u64)
    }

    fn organizer_feedback(element: &Element, message: &str, error: bool) {
        let item_status = element
            .closest("[data-watchlist-item]")
            .ok()
            .flatten()
            .and_then(|item| {
                item.query_selector("[data-watchlist-item-feedback]")
                    .ok()
                    .flatten()
            });
        let group_status = element
            .closest("[data-watchlist-group]")
            .ok()
            .flatten()
            .and_then(|group| {
                group
                    .query_selector("[data-watchlist-group-feedback]")
                    .ok()
                    .flatten()
            });
        let page_status = window_document().and_then(|(_, document)| {
            document
                .query_selector("[data-watchlist-feedback]")
                .ok()
                .flatten()
        });
        for status in [item_status, group_status, page_status]
            .into_iter()
            .flatten()
        {
            status.set_text_content(Some(message));
            let _ = status.set_attribute("role", if error { "alert" } else { "status" });
        }
    }

    fn find_card(document: &Document, group_id: &str, symbol: &str) -> Option<Element> {
        document
            .query_selector(&format!(
                "[data-watchlist-item][data-group-id=\"{group_id}\"][data-symbol=\"{symbol}\"]:not([data-restore-used])"
            ))
            .ok()
            .flatten()
    }

    fn restore_watchlist_layout(snapshot: &Value) {
        let Some((_, document)) = window_document() else {
            return;
        };
        let Some(container) = document
            .query_selector("[data-watchlist-groups]")
            .ok()
            .flatten()
        else {
            return;
        };
        let ungrouped_section = document
            .query_selector("[data-watchlist-group][data-group-id=\"ungrouped\"]")
            .ok()
            .flatten();
        if let Some(groups) = snapshot.get("groups").and_then(Value::as_array) {
            for group in groups {
                let Some(id) = group.get("id").and_then(Value::as_str) else {
                    continue;
                };
                if let Ok(Some(section)) = document
                    .query_selector(&format!("[data-watchlist-group][data-group-id=\"{id}\"]"))
                {
                    let _ = container.insert_before(&section, ungrouped_section.as_deref());
                }
            }
        }
        for card in document_elements(&document, "[data-watchlist-item]") {
            let _ = card.remove_attribute("data-restore-used");
        }
        let restore_group = |group_id: &str, symbols: &[Value]| {
            let Ok(Some(list)) = document.query_selector(&format!(
                "[data-watchlist-items][data-group-id=\"{group_id}\"]"
            )) else {
                return;
            };
            for value in symbols {
                let Some(symbol) = value.as_str() else {
                    continue;
                };
                let card = find_card(&document, group_id, symbol).or_else(|| {
                    document
                        .query_selector(&format!(
                            "[data-watchlist-item][data-symbol=\"{symbol}\"]:not([data-restore-used])"
                        ))
                        .ok()
                        .flatten()
                });
                if let Some(card) = card {
                    let _ = card.set_attribute("data-restore-used", "true");
                    let _ = card.set_attribute("data-group-id", group_id);
                    let _ = card.remove_attribute("data-layout-exclude");
                    let _ = card.remove_attribute("hidden");
                    if let Some(select) = card
                        .query_selector("[data-watchlist-move-to-group]")
                        .ok()
                        .flatten()
                        .and_then(|select| select.dyn_into::<HtmlSelectElement>().ok())
                    {
                        select.set_value(group_id);
                    }
                    let _ = list.append_child(&card);
                }
            }
        };
        if let Some(groups) = snapshot.get("groups").and_then(Value::as_array) {
            for group in groups {
                if let (Some(id), Some(symbols)) = (
                    group.get("id").and_then(Value::as_str),
                    group.get("symbols").and_then(Value::as_array),
                ) {
                    restore_group(id, symbols);
                }
            }
        }
        if let Some(symbols) = snapshot.get("ungrouped").and_then(Value::as_array) {
            restore_group("ungrouped", symbols);
        }
        for card in document_elements(&document, "[data-watchlist-item]") {
            let _ = card.remove_attribute("data-restore-used");
        }
    }

    async fn persist_watchlist_layout(trigger: Element, body: Value, rollback: Option<Value>) {
        set_watchlist_busy(&trigger, true);
        organizer_feedback(&trigger, "Saving…", false);
        match fetch_json("/api/users/watchlist/layout", "PUT", Some(body)).await {
            Ok(value) if layout_is_canonical(&value) => reload(),
            _ => {
                if let Some(snapshot) = rollback.as_ref() {
                    restore_watchlist_layout(snapshot);
                }
                set_watchlist_busy(&trigger, false);
                organizer_feedback(
                    &trigger,
                    "This change could not be saved. The previous layout was restored.",
                    true,
                );
            }
        }
    }

    fn save_current_watchlist_layout(trigger: Element, rollback: Value) {
        let Some(body) = collect_watchlist_layout() else {
            restore_watchlist_layout(&rollback);
            organizer_feedback(&trigger, "The current layout could not be read.", true);
            return;
        };
        spawn_local(persist_watchlist_layout(trigger, body, Some(rollback)));
    }

    async fn create_watchlist_group(element: Element) {
        let Some((_, document)) = window_document() else {
            return;
        };
        let Some(input) = document
            .query_selector("[data-watchlist-new-group-name]")
            .ok()
            .flatten()
            .and_then(|input| input.dyn_into::<HtmlInputElement>().ok())
        else {
            return;
        };
        let name = input.value().trim().to_string();
        if name.is_empty() || name.chars().count() > 50 || name.chars().any(char::is_control) {
            organizer_feedback(&element, "Enter a group name with 1–50 characters.", true);
            return;
        }
        set_watchlist_busy(&element, true);
        organizer_feedback(&element, "Saving…", false);
        match fetch_json(
            "/api/users/watchlist/groups",
            "POST",
            Some(json!({"name": name})),
        )
        .await
        {
            Ok(value) if layout_is_canonical(&value) => reload(),
            _ => {
                set_watchlist_busy(&element, false);
                organizer_feedback(
                    &element,
                    "The group could not be created. Its name may already be in use.",
                    true,
                );
            }
        }
    }

    async fn rename_watchlist_group(element: Element) {
        let Some(group_id) = element.get_attribute("data-group-id") else {
            return;
        };
        let Some(name) = element
            .closest("[data-watchlist-group]")
            .ok()
            .flatten()
            .and_then(|group| {
                group
                    .query_selector("[data-watchlist-group-name]")
                    .ok()
                    .flatten()
            })
            .and_then(|input| input.dyn_into::<HtmlInputElement>().ok())
            .map(|input| input.value().trim().to_string())
        else {
            return;
        };
        if name.is_empty() || name.chars().count() > 50 || name.chars().any(char::is_control) {
            organizer_feedback(&element, "Enter a group name with 1–50 characters.", true);
            return;
        }
        set_watchlist_busy(&element, true);
        organizer_feedback(&element, "Saving…", false);
        match fetch_json(
            &format!("/api/users/watchlist/groups/{group_id}"),
            "PUT",
            Some(json!({"name": name})),
        )
        .await
        {
            Ok(value) if layout_is_canonical(&value) => reload(),
            _ => {
                set_watchlist_busy(&element, false);
                organizer_feedback(
                    &element,
                    "The group name could not be saved. It may already be in use.",
                    true,
                );
            }
        }
    }

    async fn delete_watchlist_group(element: Element) {
        let Some(group_id) = element.get_attribute("data-group-id") else {
            return;
        };
        let confirmed = web_sys::window()
            .and_then(|window| {
                window
                    .confirm_with_message(
                        "Delete this group? Stocks without another group will move to Ungrouped.",
                    )
                    .ok()
            })
            .unwrap_or(false);
        if !confirmed {
            return;
        }
        set_watchlist_busy(&element, true);
        organizer_feedback(&element, "Removing…", false);
        match fetch_json(
            &format!("/api/users/watchlist/groups/{group_id}"),
            "DELETE",
            None,
        )
        .await
        {
            Ok(value) if layout_is_canonical(&value) => reload(),
            _ => {
                set_watchlist_busy(&element, false);
                organizer_feedback(&element, "The group could not be deleted.", true);
            }
        }
    }

    fn remove_symbol(values: &mut Vec<Value>, symbol: &str) {
        values.retain(|value| value.as_str() != Some(symbol));
    }

    async fn save_symbol_groups(element: Element) {
        let Some(card) = element.closest("[data-watchlist-item]").ok().flatten() else {
            return;
        };
        let Some(symbol) = card.get_attribute("data-symbol") else {
            return;
        };
        let selected: std::collections::HashSet<String> =
            elements(&card, "[data-watchlist-membership-choice]")
                .into_iter()
                .filter_map(|choice| choice.dyn_into::<HtmlInputElement>().ok())
                .filter(|choice| choice.checked())
                .map(|choice| choice.value())
                .collect();
        let Some(mut body) = collect_watchlist_layout() else {
            return;
        };
        if let Some(groups) = body.get_mut("groups").and_then(Value::as_array_mut) {
            for group in groups {
                let Some(id) = group.get("id").and_then(Value::as_str).map(str::to_string) else {
                    continue;
                };
                let Some(symbols) = group.get_mut("symbols").and_then(Value::as_array_mut) else {
                    continue;
                };
                remove_symbol(symbols, &symbol);
                if selected.contains(&id) {
                    symbols.push(Value::String(symbol.clone()));
                }
            }
        }
        if let Some(ungrouped) = body.get_mut("ungrouped").and_then(Value::as_array_mut) {
            remove_symbol(ungrouped, &symbol);
            if selected.is_empty() {
                ungrouped.push(Value::String(symbol));
            }
        }
        persist_watchlist_layout(element, body, None).await;
    }

    fn card_occurrences(symbol: &str) -> Vec<Element> {
        let Some((_, document)) = window_document() else {
            return vec![];
        };
        document_elements(
            &document,
            &format!("[data-watchlist-item][data-symbol=\"{symbol}\"]"),
        )
        .into_iter()
        .filter(|card| !card.has_attribute("data-layout-exclude"))
        .collect()
    }

    fn group_list(group_id: &str) -> Option<Element> {
        let (_, document) = window_document()?;
        document
            .query_selector(&format!(
                "[data-watchlist-items][data-group-id=\"{group_id}\"]"
            ))
            .ok()
            .flatten()
    }

    fn exclude_card(card: &Element) {
        let _ = card.set_attribute("data-layout-exclude", "true");
        let _ = card.set_attribute("hidden", "");
    }

    fn move_card_to_group_dom(card: &Element, target_group: &str, before: Option<&Element>) {
        let Some(symbol) = card.get_attribute("data-symbol") else {
            return;
        };
        let Some(list) = group_list(target_group) else {
            return;
        };
        if target_group == "ungrouped" {
            for other in card_occurrences(&symbol) {
                if !card.is_same_node(Some(&other)) {
                    exclude_card(&other);
                }
            }
            let _ = card.remove_attribute("data-layout-exclude");
            let _ = card.remove_attribute("hidden");
            let _ = card.set_attribute("data-group-id", target_group);
            let _ = list.insert_before(card, before.map(|element| &**element));
            return;
        }
        let existing = card_occurrences(&symbol).into_iter().find(|candidate| {
            !card.is_same_node(Some(candidate))
                && candidate.get_attribute("data-group-id").as_deref() == Some(target_group)
        });
        if let Some(existing) = existing {
            exclude_card(card);
            let _ = list.insert_before(&existing, before.map(|element| &**element));
        } else {
            let _ = card.set_attribute("data-group-id", target_group);
            let _ = list.insert_before(card, before.map(|element| &**element));
        }
    }

    async fn remove_group_membership(element: Element) {
        let Some(card) = element.closest("[data-watchlist-item]").ok().flatten() else {
            return;
        };
        let Some(symbol) = card.get_attribute("data-symbol") else {
            return;
        };
        let Some(snapshot) = collect_watchlist_layout() else {
            return;
        };
        let occurrences = card_occurrences(&symbol).len();
        if occurrences <= 1 {
            move_card_to_group_dom(&card, "ungrouped", None);
        } else {
            exclude_card(&card);
        }
        let Some(body) = collect_watchlist_layout() else {
            restore_watchlist_layout(&snapshot);
            return;
        };
        persist_watchlist_layout(element, body, Some(snapshot)).await;
    }

    fn move_watchlist_item_keyboard(element: Element, down: bool) {
        let Some(card) = element.closest("[data-watchlist-item]").ok().flatten() else {
            return;
        };
        let Some(list) = card.parent_element() else {
            return;
        };
        let cards: Vec<_> = elements(&list, "[data-watchlist-item]")
            .into_iter()
            .filter(|candidate| !candidate.has_attribute("data-layout-exclude"))
            .collect();
        let Some(index) = cards
            .iter()
            .position(|candidate| card.is_same_node(Some(candidate)))
        else {
            return;
        };
        let sibling = if down {
            cards.get(index + 1)
        } else {
            index.checked_sub(1).and_then(|index| cards.get(index))
        };
        let Some(sibling) = sibling else {
            return;
        };
        let Some(snapshot) = collect_watchlist_layout() else {
            return;
        };
        if down {
            let _ = list.insert_before(sibling, Some(&card));
        } else {
            let _ = list.insert_before(&card, Some(sibling));
        }
        save_current_watchlist_layout(element, snapshot);
    }

    fn move_watchlist_group_keyboard(element: Element, down: bool) {
        let Some(section) = element.closest("[data-watchlist-group]").ok().flatten() else {
            return;
        };
        let Some(container) = section.parent_element() else {
            return;
        };
        let groups: Vec<_> = elements(&container, "[data-watchlist-group]")
            .into_iter()
            .filter(|group| group.get_attribute("data-group-id").as_deref() != Some("ungrouped"))
            .collect();
        let Some(index) = groups
            .iter()
            .position(|group| section.is_same_node(Some(group)))
        else {
            return;
        };
        let sibling = if down {
            groups.get(index + 1)
        } else {
            index.checked_sub(1).and_then(|index| groups.get(index))
        };
        let Some(sibling) = sibling else {
            return;
        };
        let Some(snapshot) = collect_watchlist_layout() else {
            return;
        };
        if down {
            let _ = container.insert_before(sibling, Some(&section));
        } else {
            let _ = container.insert_before(&section, Some(sibling));
        }
        save_current_watchlist_layout(element, snapshot);
    }

    fn move_watchlist_item_to_group(element: Element, target_group: &str) {
        let Some(card) = element.closest("[data-watchlist-item]").ok().flatten() else {
            return;
        };
        if card.get_attribute("data-group-id").as_deref() == Some(target_group) {
            return;
        }
        let Some(snapshot) = collect_watchlist_layout() else {
            return;
        };
        move_card_to_group_dom(&card, target_group, None);
        save_current_watchlist_layout(element, snapshot);
    }

    fn clear_watchlist_drop_highlights() {
        let Some((_, document)) = window_document() else {
            return;
        };
        for element in document_elements(&document, ".portfolio-drop-target") {
            let _ = element.class_list().remove_1("portfolio-drop-target");
        }
    }

    fn remove_watchlist_placeholder() {
        let Some((_, document)) = window_document() else {
            return;
        };
        for placeholder in document_elements(&document, "[data-watchlist-drop-placeholder]") {
            if let Some(parent) = placeholder.parent_node() {
                let _ = parent.remove_child(&placeholder);
            }
        }
    }

    fn position_watchlist_placeholder(target: &Element, kind: &str) {
        let Some((_, document)) = window_document() else {
            return;
        };
        let Some(placeholder) = document
            .query_selector("[data-watchlist-drop-placeholder]")
            .ok()
            .flatten()
        else {
            return;
        };
        if kind == "group" {
            if let Ok(Some(group)) = target.closest("[data-watchlist-group]") {
                if let Some(parent) = group.parent_node() {
                    let _ = parent.insert_before(&placeholder, Some(&group));
                }
            }
        } else if let Ok(Some(card)) = target.closest("[data-watchlist-item]") {
            if let Some(parent) = card.parent_node() {
                let _ = parent.insert_before(&placeholder, Some(&card));
            }
        } else if let Ok(Some(list)) = target.closest("[data-watchlist-items]") {
            let _ = list.append_child(&placeholder);
        }
    }

    fn cleanup_watchlist_drag(source: &Element) {
        let _ = source.class_list().remove_1("portfolio-dragging");
        clear_watchlist_drop_highlights();
        remove_watchlist_placeholder();
    }

    fn cancel_watchlist_drag() {
        WATCHLIST_DRAG.with(|state| {
            if let Some(state) = state.borrow_mut().take() {
                restore_watchlist_layout(&state.snapshot);
                cleanup_watchlist_drag(&state.source);
            }
        });
    }

    fn watchlist_drag_target(target: &Element, kind: &str) -> Option<Element> {
        if kind == "group" {
            target
                .closest("[data-watchlist-group]")
                .ok()
                .flatten()
                .filter(|group| {
                    group.get_attribute("data-group-id").as_deref() != Some("ungrouped")
                })
        } else {
            target
                .closest("[data-watchlist-item]")
                .ok()
                .flatten()
                .or_else(|| target.closest("[data-watchlist-items]").ok().flatten())
        }
    }

    fn perform_watchlist_drop(state: WatchlistDragState, target: Element) {
        let target = if target.has_attribute("data-watchlist-drop-placeholder") {
            target
                .next_sibling()
                .and_then(|node| node.dyn_into::<Element>().ok())
                .or_else(|| target.parent_element())
                .unwrap_or(target)
        } else {
            target
        };
        cleanup_watchlist_drag(&state.source);
        if state.kind == "group" {
            let Some(target_group) = target.closest("[data-watchlist-group]").ok().flatten() else {
                return;
            };
            if state.source.is_same_node(Some(&target_group))
                || target_group.get_attribute("data-group-id").as_deref() == Some("ungrouped")
            {
                return;
            }
            if let Some(container) = target_group.parent_element() {
                let _ = container.insert_before(&state.source, Some(&target_group));
                save_current_watchlist_layout(state.source, state.snapshot);
            }
            return;
        }
        let destination_card = target.closest("[data-watchlist-item]").ok().flatten();
        let destination_list = target
            .closest("[data-watchlist-items]")
            .ok()
            .flatten()
            .or_else(|| {
                destination_card
                    .as_ref()
                    .and_then(|card| card.parent_element())
            });
        let Some(destination_list) = destination_list else {
            return;
        };
        let Some(group_id) = destination_list.get_attribute("data-group-id") else {
            return;
        };
        let before = destination_card
            .as_ref()
            .filter(|card| !state.source.is_same_node(Some(card)));
        move_card_to_group_dom(&state.source, &group_id, before);
        save_current_watchlist_layout(state.source, state.snapshot);
    }

    fn start_watchlist_drag(source: Element, kind: &str) {
        let Some(snapshot) = collect_watchlist_layout() else {
            return;
        };
        cancel_watchlist_drag();
        let _ = source.class_list().add_1("portfolio-dragging");
        if let Some((_, document)) = window_document() {
            if let Ok(placeholder) = document.create_element("div") {
                let _ = placeholder.set_attribute("data-watchlist-drop-placeholder", "true");
                placeholder.set_class_name("portfolio-drop-placeholder");
                if let Some(parent) = source.parent_node() {
                    let _ = parent.insert_before(&placeholder, Some(&source));
                }
            }
        }
        WATCHLIST_DRAG.with(|state| {
            *state.borrow_mut() = Some(WatchlistDragState {
                source,
                snapshot,
                kind: kind.to_string(),
            });
        });
    }

    fn bind_watchlist_drag(document: &Document) -> Result<(), JsValue> {
        let start = Closure::<dyn FnMut(DragEvent)>::new(move |event: DragEvent| {
            let Some(target) = event
                .target()
                .and_then(|target| target.dyn_into::<Element>().ok())
            else {
                return;
            };
            if let Ok(Some(card)) = target.closest("[data-watchlist-item]") {
                if let Some(data) = event.data_transfer() {
                    let _ = data.set_data("text/plain", "watchlist-item");
                }
                start_watchlist_drag(card, "item");
            } else if let Ok(Some(handle)) = target.closest("[data-watchlist-group-handle]") {
                if let Ok(Some(group)) = handle.closest("[data-watchlist-group]") {
                    if let Some(data) = event.data_transfer() {
                        let _ = data.set_data("text/plain", "watchlist-group");
                    }
                    start_watchlist_drag(group, "group");
                }
            }
        });
        document.add_event_listener_with_callback("dragstart", start.as_ref().unchecked_ref())?;
        start.forget();

        let over = Closure::<dyn FnMut(DragEvent)>::new(move |event: DragEvent| {
            let Some(target) = event
                .target()
                .and_then(|target| target.dyn_into::<Element>().ok())
            else {
                return;
            };
            WATCHLIST_DRAG.with(|state| {
                let state = state.borrow();
                let Some(state) = state.as_ref() else {
                    return;
                };
                if let Some(drop_target) = watchlist_drag_target(&target, &state.kind) {
                    event.prevent_default();
                    clear_watchlist_drop_highlights();
                    let _ = drop_target.class_list().add_1("portfolio-drop-target");
                    position_watchlist_placeholder(&drop_target, &state.kind);
                }
            });
        });
        document.add_event_listener_with_callback("dragover", over.as_ref().unchecked_ref())?;
        over.forget();

        let drop = Closure::<dyn FnMut(DragEvent)>::new(move |event: DragEvent| {
            let Some(target) = event
                .target()
                .and_then(|target| target.dyn_into::<Element>().ok())
            else {
                return;
            };
            let state = WATCHLIST_DRAG.with(|state| state.borrow_mut().take());
            if let Some(state) = state {
                event.prevent_default();
                perform_watchlist_drop(state, target);
            }
        });
        document.add_event_listener_with_callback("drop", drop.as_ref().unchecked_ref())?;
        drop.forget();

        let end = Closure::<dyn FnMut(DragEvent)>::new(move |_event: DragEvent| {
            cancel_watchlist_drag();
        });
        document.add_event_listener_with_callback("dragend", end.as_ref().unchecked_ref())?;
        end.forget();
        Ok(())
    }

    fn bind_watchlist_pointer_drag(document: &Document) -> Result<(), JsValue> {
        let down = Closure::<dyn FnMut(PointerEvent)>::new(move |event: PointerEvent| {
            let Some(target) = event
                .target()
                .and_then(|target| target.dyn_into::<Element>().ok())
            else {
                return;
            };
            if let Ok(Some(handle)) = target.closest("[data-watchlist-item-handle]") {
                if let Ok(Some(card)) = handle.closest("[data-watchlist-item]") {
                    event.prevent_default();
                    start_watchlist_drag(card, "item");
                }
            } else if let Ok(Some(handle)) = target.closest("[data-watchlist-group-handle]") {
                if let Ok(Some(group)) = handle.closest("[data-watchlist-group]") {
                    event.prevent_default();
                    start_watchlist_drag(group, "group");
                }
            }
        });
        document.add_event_listener_with_callback("pointerdown", down.as_ref().unchecked_ref())?;
        down.forget();

        let move_document = document.clone();
        let pointer_move = Closure::<dyn FnMut(PointerEvent)>::new(move |event: PointerEvent| {
            WATCHLIST_DRAG.with(|state| {
                let state = state.borrow();
                let Some(state) = state.as_ref() else {
                    return;
                };
                let Some(target) = move_document
                    .element_from_point(event.client_x() as f32, event.client_y() as f32)
                else {
                    return;
                };
                if let Some(drop_target) = watchlist_drag_target(&target, &state.kind) {
                    clear_watchlist_drop_highlights();
                    let _ = drop_target.class_list().add_1("portfolio-drop-target");
                    position_watchlist_placeholder(&drop_target, &state.kind);
                }
                if let Some(window) = web_sys::window() {
                    let height = window
                        .inner_height()
                        .ok()
                        .and_then(|height| height.as_f64())
                        .unwrap_or(0.0);
                    let y = f64::from(event.client_y());
                    if y < 64.0 {
                        window.scroll_by_with_x_and_y(0.0, -22.0);
                    } else if height > 0.0 && y > height - 64.0 {
                        window.scroll_by_with_x_and_y(0.0, 22.0);
                    }
                }
            });
        });
        document.add_event_listener_with_callback(
            "pointermove",
            pointer_move.as_ref().unchecked_ref(),
        )?;
        pointer_move.forget();

        let up_document = document.clone();
        let up = Closure::<dyn FnMut(PointerEvent)>::new(move |event: PointerEvent| {
            let state = WATCHLIST_DRAG.with(|state| state.borrow_mut().take());
            let Some(state) = state else {
                return;
            };
            let target =
                up_document.element_from_point(event.client_x() as f32, event.client_y() as f32);
            if let Some(target) = target {
                perform_watchlist_drop(state, target);
            } else {
                restore_watchlist_layout(&state.snapshot);
                cleanup_watchlist_drag(&state.source);
            }
        });
        document.add_event_listener_with_callback("pointerup", up.as_ref().unchecked_ref())?;
        up.forget();
        let cancel = Closure::<dyn FnMut(PointerEvent)>::new(move |_event: PointerEvent| {
            cancel_watchlist_drag();
        });
        document
            .add_event_listener_with_callback("pointercancel", cancel.as_ref().unchecked_ref())?;
        cancel.forget();
        Ok(())
    }

    fn set_watchlist_busy(element: &Element, busy: bool) {
        let _ = element.set_attribute("aria-busy", if busy { "true" } else { "false" });
        if let Some(button) = element.dyn_ref::<HtmlButtonElement>() {
            button.set_disabled(busy);
        }
        if let Some(select) = element.dyn_ref::<HtmlSelectElement>() {
            select.set_disabled(busy);
        }
    }

    fn set_watchlist_feedback(element: &Element, message: &str, error: bool) {
        let local_status = element
            .closest("[data-stock-card]")
            .ok()
            .flatten()
            .and_then(|card| {
                card.query_selector(".stock-watchlist-status")
                    .ok()
                    .flatten()
            });
        let page_status = window_document().and_then(|(_, document)| {
            document
                .query_selector("[data-watchlist-feedback]")
                .ok()
                .flatten()
        });
        for status in [local_status, page_status].into_iter().flatten() {
            status.set_text_content(Some(message));
            let _ = status.set_attribute("role", if error { "alert" } else { "status" });
        }
    }

    async fn update_watchlist(element: Element) {
        let watch_form = element.closest("[data-watchlist-form]").ok().flatten();
        let raw_symbol = element.get_attribute("data-symbol").or_else(|| {
            watch_form
                .as_ref()
                .and_then(|form| form_value(form, "symbol"))
        });
        let Some(symbol) = raw_symbol.and_then(|value| normalize_watchlist_symbol(&value)) else {
            set_watchlist_feedback(
                &element,
                "Enter a valid stock symbol using letters, numbers, dots, or hyphens.",
                true,
            );
            return;
        };
        let currently_watched =
            element.get_attribute("data-watchlisted").as_deref() == Some("true");
        let membership_count = element
            .get_attribute("data-membership-count")
            .and_then(|value| value.parse::<usize>().ok())
            .unwrap_or_default();
        if currently_watched && membership_count > 1 {
            let confirmed = web_sys::window()
                .and_then(|window| {
                    window
                        .confirm_with_message(&format!(
                            "Unwatch {symbol}? It will be removed from all {membership_count} groups."
                        ))
                        .ok()
                })
                .unwrap_or(false);
            if !confirmed {
                return;
            }
        }
        let Some((method, path)) = watchlist_mutation(&symbol, currently_watched) else {
            return;
        };
        set_watchlist_busy(&element, true);
        let idle_label = element.text_content().unwrap_or_else(|| {
            if currently_watched {
                "Unwatch"
            } else {
                "Watch"
            }
            .into()
        });
        let _ = element.set_attribute("data-watchlist-idle-label", &idle_label);
        element.set_text_content(Some(if currently_watched {
            "Removing…"
        } else {
            "Saving…"
        }));
        set_watchlist_feedback(
            &element,
            if currently_watched {
                "Removing from your watchlist…"
            } else {
                "Saving to your watchlist…"
            },
            false,
        );
        let group_ids: Vec<Value> = watch_form
            .as_ref()
            .map(|form| {
                elements(form, "input[name=\"group_ids\"]")
                    .into_iter()
                    .filter_map(|input| input.dyn_into::<HtmlInputElement>().ok())
                    .filter(|input| input.checked())
                    .map(|input| Value::String(input.value()))
                    .collect()
            })
            .unwrap_or_default();
        let body = (!currently_watched).then(|| json!({"symbol": symbol, "group_ids": group_ids}));
        let persisted = fetch_json(&path, method, body)
            .await
            .ok()
            .and_then(|value| {
                value
                    .pointer("/data/symbols")
                    .and_then(Value::as_array)
                    .map(|symbols| {
                        symbols.iter().any(|candidate| {
                            candidate
                                .as_str()
                                .and_then(normalize_watchlist_symbol)
                                .as_deref()
                                == Some(symbol.as_str())
                        })
                    })
            })
            .is_some_and(|contains_symbol| contains_symbol != currently_watched);
        if persisted {
            set_watchlist_feedback(
                &element,
                if currently_watched {
                    "Removed from your watchlist."
                } else {
                    "Saved to your watchlist."
                },
                false,
            );
            reload();
        } else {
            set_watchlist_busy(&element, false);
            if let Some(label) = element.get_attribute("data-watchlist-idle-label") {
                element.set_text_content(Some(&label));
            }
            set_watchlist_feedback(
                &element,
                "Your watchlist could not be updated. Please try again.",
                true,
            );
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

    fn set_plan_payment_status(message: &str, error: bool, state: &str) {
        let Some((_, document)) = window_document() else {
            return;
        };
        if let Some(node) = document.get_element_by_id("plan-payment-status") {
            node.set_text_content(Some(message));
            let _ = node.set_attribute("role", if error { "alert" } else { "status" });
            let _ = node.set_attribute("data-payment-state", state);
        }
    }

    fn plan_payment_busy(button: &Element, busy: bool) {
        let _ = button.set_attribute("aria-busy", if busy { "true" } else { "false" });
        if let Some(button) = button.dyn_ref::<HtmlButtonElement>() {
            button.set_disabled(busy);
        }
    }

    fn chain_metadata(chain_id: u64) -> Option<(&'static str, &'static str, &'static str)> {
        match chain_id {
            31_337 => Some(("EPSX Local", "http://127.0.0.1:8545", "ETH")),
            56 => Some(("BNB Smart Chain", "https://bsc-dataseed.binance.org", "BNB")),
            97 => Some((
                "BNB Smart Chain Testnet",
                "https://data-seed-prebsc-1-s1.bnbchain.org:8545",
                "tBNB",
            )),
            _ => None,
        }
    }

    async fn ensure_payment_chain(
        provider: &JsValue,
        chain_id: u64,
        chain_hex: &str,
    ) -> Result<(), JsValue> {
        if provider_chain_id(provider).await? == chain_id {
            return Ok(());
        }
        let switch_params = Array::new();
        let switch_target = Object::new();
        Reflect::set(
            &switch_target,
            &JsValue::from_str("chainId"),
            &JsValue::from_str(chain_hex),
        )?;
        switch_params.push(&switch_target);
        if wallet_request(provider, "wallet_switchEthereumChain", switch_params)
            .await
            .is_err()
        {
            let (chain_name, rpc_url, symbol) = chain_metadata(chain_id)
                .ok_or_else(|| JsValue::from_str("Unsupported payment network"))?;
            let add_params = Array::new();
            let network = Object::new();
            Reflect::set(
                &network,
                &JsValue::from_str("chainId"),
                &JsValue::from_str(chain_hex),
            )?;
            Reflect::set(
                &network,
                &JsValue::from_str("chainName"),
                &JsValue::from_str(chain_name),
            )?;
            let native_currency = Object::new();
            Reflect::set(
                &native_currency,
                &JsValue::from_str("name"),
                &JsValue::from_str(symbol),
            )?;
            Reflect::set(
                &native_currency,
                &JsValue::from_str("symbol"),
                &JsValue::from_str(symbol),
            )?;
            Reflect::set(
                &native_currency,
                &JsValue::from_str("decimals"),
                &JsValue::from_f64(18.0),
            )?;
            Reflect::set(
                &network,
                &JsValue::from_str("nativeCurrency"),
                &native_currency,
            )?;
            let rpc_urls = Array::new();
            rpc_urls.push(&JsValue::from_str(rpc_url));
            Reflect::set(&network, &JsValue::from_str("rpcUrls"), &rpc_urls)?;
            add_params.push(&network);
            wallet_request(provider, "wallet_addEthereumChain", add_params).await?;
        }
        if provider_chain_id(provider).await? != chain_id {
            return Err(JsValue::from_str(
                "MetaMask is on the wrong payment network",
            ));
        }
        Ok(())
    }

    async fn poll_plan_payment(tx_hash: &str) -> Result<&'static str, JsValue> {
        let endpoint = format!("/api/v1/payments/status/{tx_hash}");
        for _ in 0..120 {
            let value = fetch_json(&endpoint, "GET", None).await?;
            let status = value
                .pointer("/data/status")
                .and_then(Value::as_str)
                .ok_or_else(|| JsValue::from_str("Payment status response was malformed"))?;
            match status {
                "confirmed" => return Ok("confirmed"),
                "failed" | "expired" => return Ok("failed"),
                "pending" | "confirming" => {
                    let confirmations = value
                        .pointer("/data/confirmations")
                        .and_then(Value::as_i64)
                        .unwrap_or_default();
                    set_plan_payment_status(
                        &format!("Waiting for on-chain confirmation… ({confirmations} confirmed)"),
                        false,
                        status,
                    );
                }
                _ => return Err(JsValue::from_str("Payment returned an unknown status")),
            }
            delay(5_000).await;
        }
        Err(JsValue::from_str(
            "Confirmation is taking longer than expected. Your transaction is still being monitored.",
        ))
    }

    async fn submit_plan_payment(button: Element) {
        plan_payment_busy(&button, true);
        set_plan_payment_status("Connecting to MetaMask…", false, "connecting");
        let result = async {
            let plan_id = button
                .get_attribute("data-plan-id")
                .ok_or_else(|| JsValue::from_str("Plan ID is unavailable"))?;
            let amount = button
                .get_attribute("data-amount")
                .ok_or_else(|| JsValue::from_str("Payment amount is unavailable"))?;
            let receiver = button
                .get_attribute("data-receiver-address")
                .ok_or_else(|| JsValue::from_str("Payment receiver is unavailable"))?;
            let token = button
                .get_attribute("data-token-address")
                .ok_or_else(|| JsValue::from_str("Payment token is unavailable"))?;
            let session_wallet = button
                .get_attribute("data-session-wallet")
                .ok_or_else(|| JsValue::from_str("Your verified wallet session is unavailable"))?;
            let chain_id = button
                .get_attribute("data-chain-id")
                .and_then(|value| value.parse::<u64>().ok())
                .ok_or_else(|| JsValue::from_str("Payment network is unavailable"))?;
            let chain_hex = button
                .get_attribute("data-chain-hex")
                .ok_or_else(|| JsValue::from_str("Payment network is unavailable"))?;
            let decimals = button
                .get_attribute("data-token-decimals")
                .and_then(|value| value.parse::<u8>().ok())
                .ok_or_else(|| JsValue::from_str("Payment token precision is unavailable"))?;
            let calldata = erc20_transfer_calldata(&receiver, &amount, decimals)
                .ok_or_else(|| JsValue::from_str("Backend payment terms are malformed"))?;

            let provider = injected_wallet_provider("metamask")?;
            let account = request_provider_account(&provider, false).await?;
            if !same_wallet_address(&account, &session_wallet) {
                return Err(JsValue::from_str(
                    "MetaMask must use the same wallet as your signed-in EPSX session.",
                ));
            }
            set_plan_payment_status("Checking the payment network…", false, "network");
            ensure_payment_chain(&provider, chain_id, &chain_hex).await?;
            ensure_provider_account(&provider, &session_wallet).await?;

            set_plan_payment_status(
                "Confirm the stablecoin transfer in MetaMask…",
                false,
                "wallet-confirmation",
            );
            let transaction = Object::new();
            Reflect::set(
                &transaction,
                &JsValue::from_str("from"),
                &JsValue::from_str(&account),
            )?;
            Reflect::set(
                &transaction,
                &JsValue::from_str("to"),
                &JsValue::from_str(&token),
            )?;
            Reflect::set(
                &transaction,
                &JsValue::from_str("data"),
                &JsValue::from_str(&calldata),
            )?;
            Reflect::set(
                &transaction,
                &JsValue::from_str("value"),
                &JsValue::from_str("0x0"),
            )?;
            let params = Array::new();
            params.push(&transaction);
            let tx_hash = wallet_request(&provider, "eth_sendTransaction", params)
                .await?
                .as_string()
                .filter(|value| {
                    value.len() == 66
                        && value.starts_with("0x")
                        && value[2..].bytes().all(|byte| byte.is_ascii_hexdigit())
                })
                .ok_or_else(|| {
                    JsValue::from_str("MetaMask returned an invalid transaction hash")
                })?;

            set_plan_payment_status(
                "Transaction sent. Registering it for backend verification…",
                false,
                "submitted",
            );
            let submitted = fetch_json(
                "/api/v1/payments/submit",
                "POST",
                Some(json!({"transaction_hash": tx_hash, "plan_id": plan_id})),
            )
            .await?;
            let status = submitted
                .pointer("/data/status")
                .and_then(Value::as_str)
                .ok_or_else(|| JsValue::from_str("Payment submission response was malformed"))?;
            if status != "confirmed" && poll_plan_payment(&tx_hash).await? != "confirmed" {
                return Err(JsValue::from_str(
                    "The transaction was not confirmed. No plan access was activated.",
                ));
            }
            Ok::<(), JsValue>(())
        }
        .await;

        match result {
            Ok(()) => {
                set_plan_payment_status(
                    "Payment confirmed. Your plan and expanded stock-ranking access are active.",
                    false,
                    "confirmed",
                );
                button.set_text_content(Some("Access activated"));
                delay(1_500).await;
                if let Some((window, _)) = window_document() {
                    let _ = window.location().assign("/analytics");
                }
            }
            Err(error) => {
                plan_payment_busy(&button, false);
                set_plan_payment_status(
                    &error
                        .as_string()
                        .unwrap_or_else(|| "Payment could not be completed.".into()),
                    true,
                    "error",
                );
            }
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

    fn update_chat_submit_state(document: &Document) {
        let topic = document
            .query_selector("[data-chat-topic-input]")
            .ok()
            .flatten()
            .and_then(|el| {
                el.dyn_ref::<HtmlInputElement>()
                    .map(|i| i.value())
                    .or_else(|| el.get_attribute("value"))
            })
            .unwrap_or_default();
        let subject = document
            .query_selector("[data-chat-subject]")
            .ok()
            .flatten()
            .and_then(|el| {
                el.dyn_ref::<HtmlInputElement>()
                    .map(|i| i.value())
                    .or_else(|| el.dyn_ref::<HtmlTextAreaElement>().map(|t| t.value()))
            })
            .unwrap_or_default();
        let message = document
            .query_selector("[data-chat-message]")
            .ok()
            .flatten()
            .and_then(|el| {
                el.dyn_ref::<HtmlTextAreaElement>()
                    .map(|t| t.value())
                    .or_else(|| el.dyn_ref::<HtmlInputElement>().map(|i| i.value()))
            })
            .unwrap_or_default();
        let ready =
            !topic.trim().is_empty() && !subject.trim().is_empty() && !message.trim().is_empty();
        if let Ok(Some(btn)) = document.query_selector("[data-chat-submit]") {
            if ready {
                let _ = btn.remove_attribute("disabled");
                if let Some(b) = btn.dyn_ref::<HtmlButtonElement>() {
                    b.set_disabled(false);
                }
            } else {
                let _ = btn.set_attribute("disabled", "");
                if let Some(b) = btn.dyn_ref::<HtmlButtonElement>() {
                    b.set_disabled(true);
                }
            }
        }
    }

    fn handle_chat_file(document: &Document, file: &web_sys::File) {
        let max_bytes = 5 * 1024 * 1024;
        let size = file.size() as usize;
        let name = file.name();
        let lower = name.to_ascii_lowercase();
        let allowed = ["jpg", "jpeg", "png", "gif", "webp", "pdf"];
        let ext = lower.rsplit('.').next().unwrap_or_default();
        let type_ok = allowed.contains(&ext);
        let error_el = document
            .query_selector("[data-chat-file-error]")
            .ok()
            .flatten();
        let list_el = document
            .query_selector("[data-chat-file-list]")
            .ok()
            .flatten();
        if !type_ok {
            if let Some(err) = error_el {
                err.set_text_content(Some(
                    "Unsupported file type. Use JPG, PNG, GIF, WebP or PDF.",
                ));
                let _ = err.remove_attribute("hidden");
            }
            if let Some(input) = document
                .query_selector("[data-chat-file-input]")
                .ok()
                .flatten()
                .and_then(|el| el.dyn_into::<HtmlInputElement>().ok())
            {
                input.set_value("");
            }
            if let Some(list) = list_el {
                list.set_inner_html("");
                let _ = list.set_attribute("hidden", "");
            }
            return;
        }
        if size > max_bytes {
            if let Some(err) = error_el {
                err.set_text_content(Some("File is too large. Max 5MB."));
                let _ = err.remove_attribute("hidden");
            }
            if let Some(input) = document
                .query_selector("[data-chat-file-input]")
                .ok()
                .flatten()
                .and_then(|el| el.dyn_into::<HtmlInputElement>().ok())
            {
                input.set_value("");
            }
            if let Some(list) = list_el {
                list.set_inner_html("");
                let _ = list.set_attribute("hidden", "");
            }
            return;
        }
        if let Some(err) = error_el {
            let _ = err.set_attribute("hidden", "");
            err.set_text_content(None);
        }
        if let Some(list) = list_el {
            let kb = (size as f64 / 1024.0).ceil() as usize;
            let display = if kb > 1024 {
                format!("{:.1} MB", size as f64 / (1024.0 * 1024.0))
            } else {
                format!("{kb} KB")
            };
            let html = format!(
                r#"<div class="chat-topic-file-item" style="display:flex;align-items:center;justify-content:space-between;gap:0.5rem;padding:0.5rem 0.625rem;margin-top:0.5rem;background:rgba(255,255,255,0.06);border:1px solid rgba(255,255,255,0.08);border-radius:0.5rem;"><span style="font-size:0.75rem;overflow:hidden;text-overflow:ellipsis;white-space:nowrap;">{} ({})</span><button type="button" data-chat-file-remove style="background:transparent;border:0;color:#94a3b8;cursor:pointer;font-size:1rem;line-height:1;">&times;</button></div>"#,
                {
                    let mut s = name.clone();
                    s = s
                        .replace('&', "&amp;")
                        .replace('<', "&lt;")
                        .replace('>', "&gt;")
                        .replace('"', "&quot;");
                    s
                },
                display
            );
            list.set_inner_html(&html);
            let _ = list.remove_attribute("hidden");
        }
    }

    fn filter_chat_conversations(document: &Document) {
        let search = document
            .query_selector("[data-chat-search]")
            .ok()
            .flatten()
            .and_then(|el| el.dyn_ref::<HtmlInputElement>().map(|i| i.value()))
            .unwrap_or_default()
            .to_ascii_lowercase()
            .trim()
            .to_string();
        let status = document
            .query_selector("[data-chat-filter-status]")
            .ok()
            .flatten()
            .and_then(|el| el.dyn_ref::<HtmlSelectElement>().map(|s| s.value()))
            .unwrap_or_else(|| "all".into());
        let topic = document
            .query_selector("[data-chat-filter-topic]")
            .ok()
            .flatten()
            .and_then(|el| el.dyn_ref::<HtmlSelectElement>().map(|s| s.value()))
            .unwrap_or_else(|| "all".into());
        let Ok(nodes) = document.query_selector_all("[data-conversation-card]") else {
            return;
        };
        for i in 0..nodes.length() {
            let Some(card) = nodes.item(i).and_then(|n| n.dyn_into::<Element>().ok()) else {
                continue;
            };
            let subj = card
                .get_attribute("data-conversation-subject")
                .unwrap_or_default();
            let card_status = card
                .get_attribute("data-conversation-status")
                .unwrap_or_default();
            let card_topic = card
                .get_attribute("data-conversation-topic")
                .unwrap_or_default();
            let matches_search = search.is_empty() || subj.contains(&search);
            let matches_status = status == "all" || card_status == status;
            let matches_topic = topic == "all" || card_topic == topic;
            let visible = matches_search && matches_status && matches_topic;
            if visible {
                let _ = card.remove_attribute("hidden");
                card.set_attribute("style", "").ok();
            } else {
                let _ = card.set_attribute("hidden", "");
            }
        }
    }

    async fn fetch_with_form_data(path: &str, form: web_sys::FormData) -> Result<Value, JsValue> {
        let window = web_sys::window().ok_or("window unavailable")?;
        let opts = RequestInit::new();
        opts.set_method("POST");
        opts.set_credentials(web_sys::RequestCredentials::SameOrigin);
        opts.set_body(&form);
        let request = Request::new_with_str_and_init(path, &opts)?;
        request.headers().set("accept", "application/json")?;
        let resp = JsFuture::from(window.fetch_with_request(&request))
            .await?
            .dyn_into::<Response>()?;
        if !resp.ok() {
            let status = resp.status();
            let code = match resp.json() {
                Ok(body) => JsFuture::from(body)
                    .await
                    .ok()
                    .and_then(|b| serde_wasm_bindgen::from_value::<Value>(b).ok())
                    .and_then(|b| b.get("error").and_then(Value::as_str).map(str::to_owned)),
                Err(_) => None,
            };
            return Err(JsValue::from_str(&auth_http_error(status, code.as_deref())));
        }
        let v = JsFuture::from(resp.json()?).await?;
        serde_wasm_bindgen::from_value(v).map_err(|e| JsValue::from_str(&e.to_string()))
    }

    async fn chat_create_with_file(
        _form: Element,
        topic_id: String,
        subject: String,
        message: String,
    ) -> Result<(), JsValue> {
        let window = web_sys::window().ok_or("window unavailable")?;
        let document = window.document().ok_or("document unavailable")?;
        // 1) create conversation via JSON
        let payload = json!({
            "topic_id": topic_id,
            "subject": subject,
            "message": message
        });
        let created: Value =
            fetch_json("/api/v1/chat/conversations", "POST", Some(payload)).await?;
        let conv_id = created
            .get("data")
            .and_then(|d| d.get("id"))
            .and_then(Value::as_str)
            .or_else(|| created.get("id").and_then(Value::as_str))
            .ok_or_else(|| JsValue::from_str("Conversation was created but id was missing"))?
            .to_string();
        // 2) optional file upload
        let file_opt: Option<File> = document
            .query_selector("[data-chat-file-input]")
            .ok()
            .flatten()
            .and_then(|el| {
                let input = el.dyn_into::<HtmlInputElement>().ok()?;
                if let Some(files) = input.files() {
                    if files.length() > 0 {
                        return files.get(0);
                    }
                }
                // fallback: dropped file stored as property __droppedFile
                Reflect::get(input.as_ref(), &JsValue::from_str("__droppedFile"))
                    .ok()
                    .and_then(|v| {
                        if v.is_undefined() || v.is_null() {
                            None
                        } else {
                            v.dyn_into::<File>().ok()
                        }
                    })
            });
        if let Some(file) = file_opt {
            let form = web_sys::FormData::new()?;
            form.append_with_blob_and_filename("file", &file, &file.name())?;
            // BFF upload route
            let upload_path = format!("/api/v1/chat/conversations/{conv_id}/upload");
            match fetch_with_form_data(&upload_path, form).await {
                Ok(_) => {}
                Err(e) => {
                    // upload failed but conversation exists: still redirect and show warning via query
                    let msg = e.as_string().unwrap_or_else(|| "File upload failed".into());
                    web_sys::console::warn_1(&JsValue::from_str(&format!(
                        "chat file upload failed: {msg}"
                    )));
                }
            }
        }
        let target = format!("/chat/{conv_id}?chat=created");
        window.location().assign(&target)?;
        Ok(())
    }

    fn bind_chat(document: &Document) -> Result<(), JsValue> {
        let click_doc = document.clone();
        let click_closure = Closure::<dyn Fn(Event)>::new(move |event: Event| {
            let Some(target) = event.target().and_then(|t| t.dyn_into::<Element>().ok()) else {
                return;
            };
            if let Ok(Some(btn)) = target.closest("[data-chat-topic-select]") {
                event.prevent_default();
                let topic_id = btn
                    .get_attribute("data-chat-topic-select")
                    .unwrap_or_default();
                let label = btn.get_attribute("data-topic-label").unwrap_or_default();
                let desc = btn
                    .get_attribute("data-topic-description")
                    .unwrap_or_default();
                let icon = btn
                    .get_attribute("data-topic-icon")
                    .unwrap_or_else(|| "message-circle".into());
                let bg = btn
                    .get_attribute("data-topic-icon-bg")
                    .unwrap_or_else(|| "rgba(59,130,246,0.15)".into());
                let fg = btn
                    .get_attribute("data-topic-icon-fg")
                    .unwrap_or_else(|| "#60a5fa".into());
                if let Ok(Some(input)) = click_doc.query_selector("[data-chat-topic-input]") {
                    let _ = input.set_attribute("value", &topic_id);
                    if let Some(html) = input.dyn_ref::<HtmlInputElement>() {
                        html.set_value(&topic_id);
                    }
                }
                if let Ok(Some(icon_el)) = click_doc.query_selector("[data-chat-selected-icon]") {
                    let _ = icon_el.set_attribute(
                        "style",
                        &format!("background:{}; color:{}; border:1px solid rgba(255,255,255,0.12); box-shadow:0 4px 12px rgba(0,0,0,0.14), inset 0 1px 0 rgba(255,255,255,0.10);", bg, fg),
                    );
                    let _ = icon_el.set_attribute("data-icon", &icon);
                    icon_el.set_inner_html(&crate::chat_topic_icon_svg(&icon));
                }
                if let Ok(Some(label_el)) = click_doc.query_selector("[data-chat-selected-label]") {
                    label_el.set_text_content(Some(&label));
                }
                if let Ok(Some(desc_el)) = click_doc.query_selector("[data-chat-selected-desc]") {
                    desc_el.set_text_content(Some(&desc));
                }
                if let Ok(Some(panel)) =
                    click_doc.query_selector("[data-chat-topic-selector-panel]")
                {
                    let _ = panel.set_attribute("hidden", "");
                }
                if let Ok(Some(wrap)) = click_doc.query_selector("[data-chat-topic-form-wrap]") {
                    let _ = wrap.remove_attribute("hidden");
                }
                if let Ok(Some(subject)) = click_doc.query_selector("[data-chat-subject]") {
                    if let Some(html) = subject.dyn_ref::<web_sys::HtmlElement>() {
                        let _ = html.focus();
                    }
                }
                update_chat_submit_state(&click_doc);
                return;
            }
            if target.closest("[data-chat-back]").ok().flatten().is_some() {
                event.prevent_default();
                if let Ok(Some(panel)) =
                    click_doc.query_selector("[data-chat-topic-selector-panel]")
                {
                    let _ = panel.remove_attribute("hidden");
                }
                if let Ok(Some(wrap)) = click_doc.query_selector("[data-chat-topic-form-wrap]") {
                    let _ = wrap.set_attribute("hidden", "");
                }
                if let Ok(Some(input)) = click_doc.query_selector("[data-chat-topic-input]") {
                    if let Some(html) = input.dyn_ref::<HtmlInputElement>() {
                        html.set_value("");
                    } else {
                        let _ = input.set_attribute("value", "");
                    }
                }
                for sel in ["[data-chat-subject]", "[data-chat-message]"] {
                    if let Ok(Some(el)) = click_doc.query_selector(sel) {
                        if let Some(html) = el.dyn_ref::<HtmlInputElement>() {
                            html.set_value("");
                        } else if let Some(html) = el.dyn_ref::<HtmlTextAreaElement>() {
                            html.set_value("");
                        }
                    }
                }
                if let Ok(Some(file_input)) = click_doc.query_selector("[data-chat-file-input]") {
                    if let Some(html) = file_input.dyn_ref::<HtmlInputElement>() {
                        html.set_value("");
                    }
                }
                if let Ok(Some(list)) = click_doc.query_selector("[data-chat-file-list]") {
                    list.set_inner_html("");
                    let _ = list.set_attribute("hidden", "");
                }
                if let Ok(Some(err)) = click_doc.query_selector("[data-chat-file-error]") {
                    let _ = err.set_attribute("hidden", "");
                    err.set_text_content(None);
                }
                if let Ok(Some(status)) = click_doc.query_selector("[data-chat-form-status]") {
                    let _ = status.set_attribute("hidden", "");
                }
                update_chat_submit_state(&click_doc);
                return;
            }
            if let Ok(Some(zone)) = target.closest("[data-chat-dropzone]") {
                if target
                    .closest("[data-chat-file-remove]")
                    .ok()
                    .flatten()
                    .is_some()
                {
                    return;
                }
                if target.has_attribute("data-chat-file-input") {
                    return;
                }
                if let Ok(Some(file_input)) = click_doc.query_selector("[data-chat-file-input]") {
                    if let Some(html) = file_input.dyn_ref::<web_sys::HtmlElement>() {
                        let _ = html.click();
                    }
                }
                let _ = zone;
            }
            if target
                .closest("[data-chat-file-remove]")
                .ok()
                .flatten()
                .is_some()
            {
                event.prevent_default();
                if let Ok(Some(file_input)) = click_doc.query_selector("[data-chat-file-input]") {
                    if let Some(html) = file_input.dyn_ref::<HtmlInputElement>() {
                        html.set_value("");
                        let _ = Reflect::delete_property(
                            html.as_ref(),
                            &JsValue::from_str("__droppedFile"),
                        );
                    }
                }
                if let Ok(Some(list)) = click_doc.query_selector("[data-chat-file-list]") {
                    list.set_inner_html("");
                    let _ = list.set_attribute("hidden", "");
                }
                if let Ok(Some(err)) = click_doc.query_selector("[data-chat-file-error]") {
                    let _ = err.set_attribute("hidden", "");
                }
                update_chat_submit_state(&click_doc);
            }
        });
        document
            .add_event_listener_with_callback("click", click_closure.as_ref().unchecked_ref())?;
        click_closure.forget();

        let input_doc = document.clone();
        let input_closure = Closure::<dyn FnMut(Event)>::new(move |event: Event| {
            let Some(target) = event.target().and_then(|t| t.dyn_into::<Element>().ok()) else {
                return;
            };
            if target
                .closest("[data-chat-subject]")
                .ok()
                .flatten()
                .is_some()
                || target
                    .closest("[data-chat-message]")
                    .ok()
                    .flatten()
                    .is_some()
            {
                update_chat_submit_state(&input_doc);
            }
            if target.get_attribute("data-chat-search").is_some()
                || target.get_attribute("data-chat-filter-status").is_some()
                || target.get_attribute("data-chat-filter-topic").is_some()
            {
                filter_chat_conversations(&input_doc);
            }
        });
        document
            .add_event_listener_with_callback("input", input_closure.as_ref().unchecked_ref())?;
        input_closure.forget();

        let change_doc = document.clone();
        let change_closure = Closure::<dyn FnMut(Event)>::new(move |event: Event| {
            let Some(target) = event.target().and_then(|t| t.dyn_into::<Element>().ok()) else {
                return;
            };
            if target.get_attribute("data-chat-file-input").is_some() {
                if let Some(input) = target.dyn_ref::<HtmlInputElement>() {
                    let _ = Reflect::delete_property(
                        input.as_ref(),
                        &JsValue::from_str("__droppedFile"),
                    );
                    if let Some(files) = input.files() {
                        if files.length() > 0 {
                            if let Some(file) = files.get(0) {
                                handle_chat_file(&change_doc, &file);
                            }
                        }
                    }
                }
                update_chat_submit_state(&change_doc);
            }
            if target.get_attribute("data-chat-filter-status").is_some()
                || target.get_attribute("data-chat-filter-topic").is_some()
            {
                filter_chat_conversations(&change_doc);
            }
        });
        document
            .add_event_listener_with_callback("change", change_closure.as_ref().unchecked_ref())?;
        change_closure.forget();

        let dragover_doc = document.clone();
        let dragover_closure = Closure::<dyn FnMut(DragEvent)>::new(move |event: DragEvent| {
            let Some(target) = event.target().and_then(|t| t.dyn_into::<Element>().ok()) else {
                return;
            };
            if target
                .closest("[data-chat-dropzone]")
                .ok()
                .flatten()
                .is_some()
            {
                event.prevent_default();
                if let Ok(Some(zone)) = dragover_doc.query_selector("[data-chat-dropzone]") {
                    let _ = zone.class_list().add_1("drag-over");
                }
            }
        });
        document.add_event_listener_with_callback(
            "dragover",
            dragover_closure.as_ref().unchecked_ref(),
        )?;
        dragover_closure.forget();

        let dragleave_doc = document.clone();
        let dragleave_closure = Closure::<dyn FnMut(DragEvent)>::new(move |event: DragEvent| {
            let Some(target) = event.target().and_then(|t| t.dyn_into::<Element>().ok()) else {
                return;
            };
            if target
                .closest("[data-chat-dropzone]")
                .ok()
                .flatten()
                .is_some()
            {
                if let Ok(Some(zone)) = dragleave_doc.query_selector("[data-chat-dropzone]") {
                    let _ = zone.class_list().remove_1("drag-over");
                }
            }
            let _ = target;
        });
        document.add_event_listener_with_callback(
            "dragleave",
            dragleave_closure.as_ref().unchecked_ref(),
        )?;
        dragleave_closure.forget();

        let drop_doc = document.clone();
        let drop_closure = Closure::<dyn FnMut(DragEvent)>::new(move |event: DragEvent| {
            event.prevent_default();
            let Some(target) = event.target().and_then(|t| t.dyn_into::<Element>().ok()) else {
                return;
            };
            if target
                .closest("[data-chat-dropzone]")
                .ok()
                .flatten()
                .is_none()
            {
                return;
            }
            if let Ok(Some(zone)) = drop_doc.query_selector("[data-chat-dropzone]") {
                let _ = zone.class_list().remove_1("drag-over");
            }
            if let Some(dt) = event.data_transfer() {
                if let Some(files) = dt.files() {
                    if files.length() > 0 {
                        if let Some(file) = files.get(0) {
                            // also set the file input's files via DataTransfer workaround is non-trivial; just handle directly
                            handle_chat_file(&drop_doc, &file);
                            // store file into input via DataTransfer is not easily done; we keep it in memory by creating a new FileList via hack: set input value and keep file in closure would be ideal but we reuse input files check on submit via a temporary global. For now, we create a DataTransfer to set files
                            if let Ok(Some(input)) =
                                drop_doc.query_selector("[data-chat-file-input]")
                            {
                                if let Some(html) = input.dyn_ref::<HtmlInputElement>() {
                                    // cannot programmatically set files due to security; we instead store file in a custom property on the input
                                    let _ =
                                        html.set_attribute("data-dropped-file-name", &file.name());
                                    // keep the file object in a property via js_sys::Reflect
                                    let _ = Reflect::set(
                                        html.as_ref(),
                                        &JsValue::from_str("__droppedFile"),
                                        &file,
                                    );
                                }
                            }
                            update_chat_submit_state(&drop_doc);
                        }
                    }
                }
            }
        });
        document.add_event_listener_with_callback("drop", drop_closure.as_ref().unchecked_ref())?;
        drop_closure.forget();

        let submit_doc = document.clone();
        let submit_closure = Closure::<dyn FnMut(Event)>::new(move |event: Event| {
            let Some(form) = event.target().and_then(|t| t.dyn_into::<Element>().ok()) else {
                return;
            };
            if form.get_attribute("data-chat-create-form").is_none() {
                return;
            }
            event.prevent_default();
            let doc = submit_doc.clone();
            let topic = doc
                .query_selector("[data-chat-topic-input]")
                .ok()
                .flatten()
                .map(|el| {
                    el.dyn_ref::<HtmlInputElement>()
                        .map(|i| i.value())
                        .unwrap_or_else(|| el.get_attribute("value").unwrap_or_default())
                })
                .unwrap_or_default();
            let subject = doc
                .query_selector("[data-chat-subject]")
                .ok()
                .flatten()
                .and_then(|el| {
                    el.dyn_ref::<HtmlInputElement>()
                        .map(|i| i.value())
                        .or_else(|| el.dyn_ref::<HtmlTextAreaElement>().map(|t| t.value()))
                })
                .unwrap_or_default();
            let message = doc
                .query_selector("[data-chat-message]")
                .ok()
                .flatten()
                .and_then(|el| {
                    el.dyn_ref::<HtmlTextAreaElement>()
                        .map(|t| t.value())
                        .or_else(|| el.dyn_ref::<HtmlInputElement>().map(|i| i.value()))
                })
                .unwrap_or_default();
            if topic.trim().is_empty() || subject.trim().is_empty() || message.trim().is_empty() {
                if let Ok(Some(status)) = doc.query_selector("[data-chat-form-status]") {
                    status.set_text_content(Some("Please fill in topic, subject and message."));
                    let _ = status.remove_attribute("hidden");
                }
                return;
            }
            if let Ok(Some(btn)) = doc.query_selector("[data-chat-submit]") {
                let _ = btn.set_attribute("disabled", "");
                if let Some(b) = btn.dyn_ref::<HtmlButtonElement>() {
                    b.set_disabled(true);
                }
            }
            if let Ok(Some(status)) = doc.query_selector("[data-chat-form-status]") {
                status.set_text_content(Some("Starting conversation…"));
                let _ = status.remove_attribute("hidden");
            }
            let doc_clone = doc.clone();
            let form_clone = form.clone();
            spawn_local(async move {
                let doc2 = doc_clone;
                let res = chat_create_with_file(form_clone, topic, subject, message).await;
                if let Err(err) = res {
                    if let Ok(Some(status)) = doc2.query_selector("[data-chat-form-status]") {
                        status.set_text_content(Some(
                            &err.as_string()
                                .unwrap_or_else(|| "Failed to start conversation.".into()),
                        ));
                        let _ = status.remove_attribute("hidden");
                    }
                    if let Ok(Some(btn)) = doc2.query_selector("[data-chat-submit]") {
                        let _ = btn.remove_attribute("disabled");
                        if let Some(b) = btn.dyn_ref::<HtmlButtonElement>() {
                            b.set_disabled(false);
                        }
                    }
                }
            });
        });
        document
            .add_event_listener_with_callback("submit", submit_closure.as_ref().unchecked_ref())?;
        submit_closure.forget();

        update_chat_submit_state(document);
        filter_chat_conversations(document);
        Ok(())
    }

    async fn load_notification_count(node: Element) {
        let endpoint = node
            .get_attribute("data-notification-count-endpoint")
            .unwrap_or_else(|| "/api/v1/notifications/unread-count".into());
        // The badge lives inside the notification link target. Keep the target's
        // accessible label and visual state in sync with the badge count.
        let target: Option<Element> = node
            .closest(r#"[data-epsx-notification-badge-target="true"]"#)
            .ok()
            .flatten()
            .or_else(|| node.parent_element());
        let set_unavailable = |badge: &Element, target: Option<&Element>| {
            badge.set_text_content(Some(""));
            let _ = badge.set_attribute("hidden", "");
            let _ = badge.set_attribute("aria-hidden", "true");
            let _ = badge.set_attribute("data-state", "unavailable");
            if let Some(target) = target {
                let _ = target.set_attribute("aria-label", "Notifications");
                // Reset bell to muted when no unread
                if let Ok(Some(icon)) = target.query_selector(".epsx-action-icon") {
                    let _ = icon.remove_attribute("style");
                }
            }
        };
        let show_count = |badge: &Element, target: Option<&Element>, count: u64| {
            if count == 0 {
                set_unavailable(badge, target);
                // Keep data-state as available but hidden to match production's
                // zero-count available state (badge hidden, target labelled).
                let _ = badge.set_attribute("data-state", "available");
                return;
            }
            let display = if count > 99 {
                "99+".to_string()
            } else {
                count.to_string()
            };
            badge.set_text_content(Some(&display));
            let _ = badge.remove_attribute("hidden");
            let _ = badge.set_attribute("aria-hidden", "false");
            let _ = badge.set_attribute("data-state", "available");
            if let Some(target) = target {
                let _ =
                    target.set_attribute("aria-label", &format!("Notifications, {count} unread"));
                // Match production: orange bell when there are unread
                if let Ok(Some(icon)) = target.query_selector(".epsx-action-icon") {
                    let _ = icon.set_attribute("style", "color:#f97316");
                }
            }
        };
        match fetch_json(&endpoint, "GET", None).await {
            Ok(value) => {
                if let Some(count) = value.get("count").and_then(Value::as_u64) {
                    show_count(&node, target.as_ref(), count);
                } else {
                    set_unavailable(&node, target.as_ref());
                }
            }
            Err(_) => set_unavailable(&node, target.as_ref()),
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
        let hostname = window.location().hostname().unwrap_or_default();
        if !service_workers_enabled(&hostname) {
            let window = window.clone();
            spawn_local(async move {
                cleanup_local_workers(window).await;
            });
            return;
        }
        let container = window.navigator().service_worker();
        let options = web_sys::RegistrationOptions::new();
        options.set_type("module");
        options.set_scope(GENERATED_WORKER_SCOPE);
        let _ = container.register_with_options(GENERATED_WORKER, &options);
    }

    async fn cleanup_local_workers(window: Window) {
        const RELOAD_MARKER: &str = "epsx-local-worker-cleanup-v1";
        let container = window.navigator().service_worker();
        let Ok(registrations) = JsFuture::from(container.get_registrations()).await else {
            return;
        };
        let mut removed = false;
        for value in Array::from(&registrations).iter() {
            let Ok(registration) = value.dyn_into::<web_sys::ServiceWorkerRegistration>() else {
                continue;
            };
            let Ok(promise) = registration.unregister() else {
                continue;
            };
            let Ok(result) = JsFuture::from(promise).await else {
                continue;
            };
            removed |= result.as_bool().unwrap_or(false);
        }

        let storage = window.session_storage().ok().flatten();
        if removed {
            let already_reloaded = storage
                .as_ref()
                .and_then(|storage| storage.get_item(RELOAD_MARKER).ok().flatten())
                .is_some();
            if !already_reloaded {
                if let Some(storage) = storage {
                    let _ = storage.set_item(RELOAD_MARKER, "1");
                }
                let _ = window.location().reload();
            }
        } else if let Some(storage) = storage {
            let _ = storage.remove_item(RELOAD_MARKER);
        }
    }

    fn set_status(message: &str, error: bool) {
        let Some((_, document)) = window_document() else {
            return;
        };

        // The auth page renders both regions hidden during SSR. Keep their
        // structure intact (spinner/title/icon) and reveal the appropriate
        // region instead of replacing a hidden parent node's entire content.
        if let (Some(status), Some(error_panel)) = (
            document.get_element_by_id("auth-card-status"),
            document.get_element_by_id("auth-card-error"),
        ) {
            if error {
                let _ = status.set_attribute("hidden", "");
                let _ = error_panel.remove_attribute("hidden");
                if let Some(error_message) = document.get_element_by_id("auth-card-error-msg") {
                    error_message.set_text_content(Some(message));
                }
            } else {
                let _ = error_panel.set_attribute("hidden", "");
                let _ = status.remove_attribute("hidden");
                if let Some(status_message) = document.get_element_by_id("auth-card-status-msg") {
                    status_message.set_text_content(Some(message));
                }
            }
            return;
        }

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

    #[test]
    fn auth_http_errors_preserve_only_the_safe_reason_code() {
        assert_eq!(
            auth_http_error(502, Some("invalid_upstream_token")),
            "Sign-in failed: invalid_upstream_token (HTTP 502)"
        );
        assert_eq!(auth_http_error(503, None), "Sign-in failed (HTTP 503)");
    }

    #[test]
    fn auth_http_errors_surface_friendly_text_for_known_outages() {
        assert_eq!(
            auth_http_error(502, Some("auth_upstream_unavailable")),
            "Sign-in service is temporarily unavailable. Please try again in a moment."
        );
        assert_eq!(
            auth_http_error(502, Some("auth_upstream_unavailable")),
            "Sign-in service is temporarily unavailable. Please try again in a moment."
        );
        assert_eq!(
            auth_http_error(502, Some(" challenge_rejected ")),
            "Wallet challenge was rejected. Please reconnect and try again."
        );
        assert_eq!(
            auth_http_error(400, Some("authentication_rejected")),
            "Wallet signature was rejected. Please reconnect and try again."
        );
        assert_eq!(
            auth_http_error(401, Some("missing_refresh_token")),
            "Your session expired. Please reconnect your wallet."
        );
        assert_eq!(auth_http_error(502, Some("")), "Sign-in failed (HTTP 502)");
    }

    #[test]
    fn transient_upstream_error_only_matches_closed_pair() {
        assert!(super::is_transient_upstream_error_pub(
            "Sign-in service is temporarily unavailable. Please try again in a moment."
        ));
        assert!(super::is_transient_upstream_error_pub(
            "  auth_upstream_unavailable (HTTP 502)  "
        ));
        assert!(super::is_transient_upstream_error_pub(
            "auth_upstream_unavailable (HTTP 530)"
        ));
        assert!(super::is_transient_upstream_error_pub(
            "Sign-in failed: auth_upstream_unavailable (HTTP 503)"
        ));
        assert!(!super::is_transient_upstream_error_pub(
            "Sign-in failed: challenge_rejected (HTTP 400)"
        ));
        assert!(!super::is_transient_upstream_error_pub(
            "Sign-in failed (HTTP 503)"
        ));
        assert!(!super::is_transient_upstream_error_pub(""));
    }

    #[test]
    fn injected_wallet_selection_is_closed_to_metamask() {
        assert_eq!(supported_injected_wallet("metamask"), Some("metamask"));
        assert_eq!(supported_injected_wallet(" MetaMask "), Some("metamask"));
        for unsupported in ["", "safe", "walletconnect", "base", "evil"] {
            assert_eq!(supported_injected_wallet(unsupported), None);
        }
    }

    #[test]
    fn wallet_identity_matching_is_case_insensitive_but_never_empty() {
        assert!(same_wallet_address("0xAaBb", " 0xaabb "));
        assert!(!same_wallet_address("", ""));
        assert!(!same_wallet_address("0x1111", "0x2222"));
    }

    #[test]
    fn provider_selected_account_wins_over_stale_account_order() {
        let accounts = vec!["0xold".to_string(), "0xCURRENT".to_string()];
        assert_eq!(
            select_wallet_account(&accounts, Some("0xcurrent")),
            Some("0xCURRENT")
        );
        assert_eq!(select_wallet_account(&accounts, None), Some("0xold"));
        assert_eq!(select_wallet_account(&[], Some("0xcurrent")), None);
    }

    #[test]
    fn service_workers_stay_off_loopback_development_origins() {
        for hostname in ["localhost", "LOCALHOST", "127.0.0.1", "::1", "[::1]"] {
            assert!(!service_workers_enabled(hostname));
        }
        assert!(service_workers_enabled("epsx.io"));
    }

    #[test]
    fn watchlist_symbols_and_mutation_paths_are_bounded() {
        for (raw, expected) in [
            (" aapl ", Some("AAPL")),
            ("BRK.B", Some("BRK.B")),
            ("btc-usd", Some("BTC-USD")),
            ("../AAPL", None),
            ("AAPL/US", None),
            ("", None),
            ("ABCDEFGHIJKLMNOPQRSTU", None),
        ] {
            assert_eq!(normalize_watchlist_symbol(raw).as_deref(), expected);
        }
        assert_eq!(
            watchlist_mutation(" aapl ", false),
            Some(("POST", "/api/users/watchlist".to_string()))
        );
        assert_eq!(
            watchlist_mutation("brk.b", true),
            Some(("DELETE", "/api/users/watchlist/BRK.B".to_string()))
        );
        assert_eq!(watchlist_mutation("../bad", true), None);
    }

    #[test]
    fn erc20_transfer_encoding_preserves_decimal_precision() {
        let calldata =
            erc20_transfer_calldata("0x1111111111111111111111111111111111111111", "9.90", 18)
                .expect("valid transfer calldata");
        assert_eq!(calldata.len(), 138);
        assert!(calldata.starts_with(
            "0xa9059cbb0000000000000000000000001111111111111111111111111111111111111111"
        ));
        assert!(calldata.ends_with("8963dd8c2c5e0000"));

        assert!(erc20_transfer_calldata(
            "0x1111111111111111111111111111111111111111",
            "1.0000000000000000001",
            18
        )
        .is_none());
        assert!(erc20_transfer_calldata("0xdeadbeef", "1.00", 18).is_none());
        assert!(
            erc20_transfer_calldata("0x1111111111111111111111111111111111111111", "0", 18)
                .is_none()
        );
    }
}
