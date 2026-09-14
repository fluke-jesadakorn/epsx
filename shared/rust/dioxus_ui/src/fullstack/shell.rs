//! Dioxus-owned native enterprise chrome. Browser adapters only persist display
//! preferences or perform clipboard/focus operations; Dioxus owns all UI state.
use crate::layout::site_navbar::{SiteNavGroup, SiteNavItem, SiteNavbar};
use crate::{enterprise::FrontendIcon, routes::FrontendRoute};
use dioxus::prelude::*;

type NavItem = (&'static str, &'static str, &'static str);
const MARKET: &[NavItem] = &[
    ("/analytics", "Explore", "search"),
    ("/portfolio", "Saved companies", "heart"),
    ("/plans", "Plans", "layers"),
    ("/news", "News", "newspaper"),
];
const ACCOUNT: &[NavItem] = &[
    ("/account", "Settings", "settings"),
    ("/dashboard", "Overview", "layout-dashboard"),
    ("/profile", "Profile", "user"),
    ("/permissions", "Access", "shield"),
    ("/account/credits", "Credits", "coins"),
    ("/payment", "Billing", "wallet"),
    ("/account/payments", "Purchases", "file-text"),
];
const COMPANY: &[NavItem] = &[
    ("/about", "About", "info"),
    ("/contact", "Contact", "mail"),
    ("/chat", "Support", "message-circle"),
    ("/privacy", "Privacy", "shield"),
    ("/terms", "Terms", "file-text"),
];

pub fn migrated_link(href: &str) -> bool {
    crate::navigation::local_route(href)
        && href
            .split('#')
            .next()
            .unwrap_or(href)
            .parse::<FrontendRoute>()
            .is_ok_and(|route| !matches!(route, FrontendRoute::NotFoundView { .. }))
}
fn marketing(path: &str) -> bool {
    matches!(
        path,
        "/" | "/index" | "/about" | "/contact" | "/manual" | "/privacy" | "/terms" | "/offline"
    )
}
#[derive(Clone, Copy)]
pub struct AuthRevision(pub Signal<u64>);
#[derive(Clone, Copy)]
pub struct ThemeSignal(pub Signal<bool>);

#[derive(Clone, Copy)]
struct ShellState {
    wallet: Signal<bool>,
}

/// Migrated destinations use the Router; pending families retain native links.
#[component]
pub fn ShellLink(
    href: String,
    #[props(default)] class: String,
    #[props(default)] current: bool,
    #[props(default)] label: String,
    children: Element,
    #[props(extends = GlobalAttributes)] attributes: Vec<Attribute>,
    #[props(default)] onclick: Option<EventHandler<MouseEvent>>,
) -> Element {
    let state = try_use_context::<ShellState>();
    let close = move |event: MouseEvent| {
        if let Some(handler) = onclick {
            handler.call(event);
        }
        if let Some(mut state) = state {
            state.wallet.set(false);
        }
    };
    if state.is_some() && migrated_link(&href) {
        rsx! { Link { to: href, class, aria_current: current.then_some("page"), title: label, onclick: close, attributes, {children} } }
    } else {
        rsx! { crate::navigation::AppLink { href, class, aria_current: current.then_some("page"), title: label, onclick: close, attributes, {children} } }
    }
}
#[component]
fn WalletMenu(address: String) -> Element {
    let mut state = use_context::<ShellState>();
    let valid = address.len() == 42
        && address.starts_with("0x")
        && address[2..].bytes().all(|byte| byte.is_ascii_hexdigit());
    let short = if valid {
        format!("{}…{}", &address[..6], &address[38..])
    } else {
        "Your wallet".into()
    };
    let mut copied = use_signal(String::new);
    let mut disconnecting = use_signal(|| false);
    let mut logout_error = use_signal(String::new);
    let mut auth_revision = use_context::<AuthRevision>();
    let navigator = use_navigator();
    let copy_address = address.clone();
    rsx! { details { class: "fe-account-menu", open: (state.wallet)(),
        summary { class: "fe-wallet-trigger", aria_label: "Account menu", aria_expanded: (state.wallet)(),
            onclick: move |event| { event.prevent_default(); state.wallet.toggle(); },
            span { class: "fe-wallet-avatar", aria_hidden: "true", FrontendIcon { name: "wallet", size: 20 } }
            span { class: "fe-wallet-short", "{short}" } FrontendIcon { name: "chevron-down", size: 20 }
        }
        div { class: "fe-account-popover fe-wallet-popover",
            div { class: "fe-wallet-card",
                div { class: "fe-wallet-card-top", span { class: "fe-wallet-avatar", aria_hidden: "true", FrontendIcon { name: "wallet", size: 20 } }
                    div { strong { "Your wallet" } span { class: "fe-wallet-session", span { aria_hidden: "true" } "Signed in" } }
                }
                code { class: "fe-account-identity", title: address, "{short}" }
                if valid {
                    button { class: "fe-wallet-copy", r#type: "button", onclick: move |_| {
                        let address = copy_address.clone();
                        spawn(async move {
                            let script = format!("try {{ await navigator.clipboard.writeText('{}'); dioxus.send(true); }} catch (_) {{ dioxus.send(false); }}", address);
                            let success = document::eval(&script).recv::<bool>().await.unwrap_or(false);
                            copied.set(if success { "Address copied." } else { "Could not copy address." }.into());
                        });
                    }, FrontendIcon { name: "copy", size: 20 } span { "Copy address" } }
                    span { class: "fe-wallet-copy-status", role: "status", aria_live: "polite", "{copied}" }
                }
            }
            nav { class: "fe-wallet-links", aria_label: "Wallet account",
                ShellLink { href: "/dashboard", FrontendIcon { name: "layout-dashboard", size: 20 } span { "Overview" } }
                ShellLink { href: "/account", FrontendIcon { name: "user", size: 20 } span { "Account settings" } }
            }
            button { class: "fe-wallet-disconnect", r#type: "button", disabled: disconnecting(), onclick: move |_| {
                if *disconnecting.peek() { return; }
                disconnecting.set(true); logout_error.set(String::new());
                spawn(async move {
                    // End the extension's site permission before removing the
                    // server session; clearing cookies alone leaves MetaMask connected.
                    let _ = document::eval(include_str!("wallet_disconnect.js")).recv::<bool>().await;
                    match disconnect().await {
                        Ok(Ok(_)) => { let next = *auth_revision.0.peek() + 1; auth_revision.0.set(next); state.wallet.set(false); navigator.replace("/"); },
                        _ => { disconnecting.set(false); logout_error.set("Could not disconnect. Please try again.".into()); },
                    }
                });
            }, FrontendIcon { name: "log-out", size: 20 } span { "Disconnect" } }
            if !logout_error().is_empty() { p { role: "status", "{logout_error}" } }
        }
    } }
}
#[component]
pub fn FrontendShell() -> Element {
    let path = use_route::<FrontendRoute>().to_string();
    rsx! { FrontendShellContent { offline: path.split('?').next() == Some("/offline") } }
}
#[component]
fn FrontendShellContent(offline: ReadSignal<bool>) -> Element {
    let mut hydrated = use_signal(|| false);
    let mut dark = use_signal(|| true);
    use_context_provider(|| ThemeSignal(dark));
    use_effect(move || {
        let is_dark = dark();
        if !hydrated() {
            return;
        }
        spawn(async move {
            let _ = document::eval(if is_dark {
                "try { localStorage.setItem('epsx-theme','dark'); } catch (_) {}"
            } else {
                "try { localStorage.setItem('epsx-theme','light'); } catch (_) {}"
            });
        });
    });
    let mut state = ShellState {
        wallet: use_signal(|| false),
    };
    use_context_provider(|| state);
    let auth_revision = AuthRevision(use_signal(|| 0_u64));
    use_context_provider(|| auth_revision);
    use_context_provider(super::frontend_auth::SessionRecovery::default);
    let full_path = use_route::<FrontendRoute>().to_string();
    let path = full_path.split('?').next().unwrap_or(&full_path).to_owned();
    let is_marketing = marketing(&path);
    let is_auth = path == "/auth";
    let navigator = use_navigator();
    rsx! {
        document::Link { rel: "stylesheet", href: "/public/dist/tailwind.css" }
        div { class: if is_auth { "epsx-frontend fe-auth" } else if is_marketing { "epsx-frontend fe-marketing" } else { "epsx-frontend fe-topnav" },
            class: if dark() { "dark" },
            "data-theme": if dark() { "dark" } else { "light" }, "data-dioxus-hydrated": hydrated(),
            onmounted: move |_| {
                spawn(async move {
                    // Match the server's dark default until the user explicitly
                    // chooses a theme; hydration must not switch it to OS light.
                    let preferences = document::eval("let theme = 'dark'; try { theme = localStorage.getItem('epsx-theme') || 'dark'; } catch (_) {} dioxus.send(theme !== 'light');").recv::<bool>().await;
                    if let Ok(theme) = preferences { dark.set(theme); }
                    // Browser lifecycle adapter only; all page UI stays in Dioxus.
                    let _ = document::eval("if (isSecureContext && 'serviceWorker' in navigator) { const blocked = ['localhost','127.0.0.1','::1','[::1]','dev.epsx.io','dev-admin.epsx.io','dev-pay.epsx.io'].includes(location.hostname); if (!blocked) { navigator.serviceWorker.register('/runtime/epsx_service_worker_bootstrap.v3.js?rev=3', {type:'module',scope:'/'}).catch(() => {}); } }");
                    hydrated.set(true);
                });
            },
            onkeydown: move |event| { if event.key() == Key::Escape { state.wallet.set(false); } },
            crate::navigation::AppLink { class: "epsx-skip-link", href: "#epsx-main-content", "Skip to main content" }
            if !is_auth {
                SiteNavbar {
                    groups: [("Market", MARKET), ("Company", COMPANY), ("Account", ACCOUNT)].into_iter().map(|(label, items)| SiteNavGroup::new(label, items.iter().map(|(href, label, icon)| SiteNavItem::new(*href, *label).with_icon(*icon)).collect())).collect(),
                    path: full_path.clone(),
                    on_navigate: move |(event, href): (MouseEvent, String)| {
                        if migrated_link(&href) && event.modifiers().is_empty() && event.trigger_button() == Some(dioxus::html::input_data::MouseButton::Primary) {
                            event.prevent_default();
                            navigator.push(href);
                        }
                    },
                    actions: rsx! { SuspenseBoundary { fallback: |_| rsx! { span { role: "status", "Loading account…" } }, HeaderProfile { offline, path: full_path.clone(), dark } } },
                }
            }
            main { class: "fe-main", id: "epsx-main-content", tabindex: -1,

                div { class: "fe-content", "data-fe-page": path, SuspenseBoundary { fallback: |_| rsx! { crate::navigation::PageSkeleton {} }, Outlet::<FrontendRoute> {} } }
            }
            if is_marketing {
                footer { class: "fe-footer", div { strong { "EPSX" } p { "Financial Technology Platform" } }
                    nav { aria_label: "Footer", for (href, label) in [("/about","About"),("/news","News"),("/contact","Contact"),("/chat","Support"),("/terms","Terms"),("/privacy","Privacy")] { ShellLink { href, "{label}" } } }
                }
            }
        }
    }
}

#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct LogoutResult {
    pub upstream_revoked: bool,
}
#[cfg(feature = "server")]
#[derive(Clone)]
pub struct LogoutProvider(pub LogoutProviderCallback);
#[server(prefix = "/_server/frontend", endpoint = "logout")]
pub async fn disconnect() -> Result<Result<LogoutResult, super::LoadError>, ServerFnError> {
    use dioxus_server::axum::Extension;
    let Extension(provider) =
        dioxus_fullstack::FullstackContext::extract::<Extension<LogoutProvider>, _>()
            .await
            .map_err(|_| ServerFnError::new("Logout provider unavailable"))?;
    let headers = dioxus_fullstack::FullstackContext::extract::<http::HeaderMap, _>()
        .await
        .map_err(|_| ServerFnError::new("Request context unavailable"))?;
    Ok((provider.0)(headers).await)
}
#[component]
fn HeaderTools(path: String, user: Option<crate::auth::User>, dark: Signal<bool>) -> Element {
    let target = url::form_urlencoded::byte_serialize(path.as_bytes()).collect::<String>();
    rsx! { div { class: "fe-header-tools",
        if user.is_some() {
            ShellLink { class: "fe-icon-button", href: "/notifications", aria_label: "Notifications", title: "Notifications", FrontendIcon { name: "bell", size: 20 } }
        }
        button { class: "fe-icon-button", r#type: "button", aria_label: "Toggle theme", title: "Toggle theme", onclick: move |_| {
            dark.toggle(); let is_dark = dark();
            spawn(async move { let _ = document::eval(if is_dark { "try { localStorage.setItem('epsx-theme','dark'); } catch (_) {}" } else { "try { localStorage.setItem('epsx-theme','light'); } catch (_) {}" }); });
        }, FrontendIcon { name: "sun", size: 20 } }
        if let Some(user) = user { WalletMenu { address: user.address } }
        else { ShellLink { class: "fe-button fe-primary fe-connect-wallet", aria_label: "Connect wallet", title: "Connect wallet", href: "/auth?return_url={target}", FrontendIcon { name: "wallet", size: 20 } span { "Connect wallet" } } }
    } }
}

#[cfg(feature = "server")]
pub type LogoutProviderCallback = std::sync::Arc<
    dyn Fn(
            http::HeaderMap,
        ) -> std::pin::Pin<
            Box<dyn std::future::Future<Output = Result<LogoutResult, super::LoadError>> + Send>,
        > + Send
        + Sync,
>;

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Clone, PartialEq, Routable)]
    enum TestRoute {
        #[route("/")]
        PublicMenus {},
    }

    #[component]
    fn PublicMenus() -> Element {
        let state = ShellState {
            wallet: use_signal(|| false),
        };
        use_context_provider(|| state);
        rsx! { SiteNavbar {
            groups: [("Market", MARKET), ("Company", COMPANY)].into_iter().map(|(label, items)| SiteNavGroup::new(label, items.iter().map(|(href,label,icon)| SiteNavItem::new(*href,*label).with_icon(*icon)).collect())).collect(),
            path: "/".to_string(), actions: rsx! {},
        } }
    }

    #[test]
    fn public_menus_are_exclusive_before_hydration_and_keep_native_links() {
        let mut dom = VirtualDom::new(|| rsx! { Router::<TestRoute> {} });
        dom.rebuild_in_place();
        let html = dioxus_ssr::render(&dom);
        assert_eq!(html.matches("name=\"epsx-primary-menu\"").count(), 2);
        assert_eq!(html.matches("<details").count(), 3);
        assert!(!html.contains(" open"));
        for href in [
            "/analytics",
            "/portfolio",
            "/plans",
            "/news",
            "/about",
            "/contact",
            "/chat",
        ] {
            assert!(html.contains(&format!("href=\"{href}\"")), "missing {href}");
        }
    }
}

#[component]
fn HeaderProfile(offline: ReadSignal<bool>, path: String, dark: Signal<bool>) -> Element {
    let auth_revision = use_context::<AuthRevision>();
    let mut profile = use_server_future(move || {
        let offline = offline();
        async move {
            if offline {
                Ok(Err(crate::fullstack::LoadError::Unauthenticated))
            } else {
                crate::pages::profile::read_profile().await
            }
        }
    })?;
    use_effect(move || {
        if (auth_revision.0)() > 0 && !offline() {
            profile.restart();
        }
    });
    let user = if offline() {
        None
    } else {
        profile
            .read()
            .as_ref()
            .and_then(|result| result.as_ref().ok())
            .and_then(|result| result.as_ref().ok())
            .cloned()
    };
    rsx! { HeaderTools { path, user, dark } }
}
