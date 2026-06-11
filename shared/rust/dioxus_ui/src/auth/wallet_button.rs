//! Wallet connect button + connected-wallet dropdown.
//!
//! Wave 2 Track C additions over the Wave 1 scaffold:
//! - `ConnectButton` — the "Connect Wallet" CTA. Renders the
//!   orange→purple gradient per the TS shadcn source. `compact` and
//!   `full` size variants match the mobile/navbar layout uses.
//! - `ConnectedWalletDropdown` — provider card, copy / explorer
//!   actions, optional sign-in button, retry banner, nav links,
//!   disconnect. The full TS `wallet-provider-icon.tsx` is 379 lines
//!   — this Rust port keeps the structural parity but strips the
//!   async SIWE flow (the BFF handles that; the Rust UI just fires
//!   the appropriate callback).
//! - `WalletProvider` registry — `WalletProviderRegistry` static
//!   table mapping connector id → display name / icon glyph / color.
//!   Used by `ConnectedWalletDropdown` to render the provider card
//!   header.
//! - Required callbacks: `on_connect`, `on_copy`, `on_explorer`,
//!   `on_sign_in`, `on_retry`, `on_disconnect`, `on_navigate`.
//!
//! All Wave 1 call sites that use `WalletConnectButton` and
//! `ConnectedWalletDropdown` keep compiling because the original
//! signatures are preserved (with new params added as
//! `#[props(default = ...)]`).

use crate::primitives::icon::Icon;
use super::user::User;

use dioxus::prelude::*;

/// The `WalletProvider` registry maps a wagmi connector id
/// (e.g. `"metaMask"`, `"walletConnect"`) to the display glyph +
/// color used by the connected-wallet dropdown header card.
///
/// This is the Rust port of the `walletProviders` table in
/// `apps-old/frontend/components/nav/wallet-provider-icon.tsx` (the
/// 379-line file). We keep only the structural fields the UI needs;
/// the actual connector activation stays in the BFF / wagmi layer.
#[derive(Clone, Debug, PartialEq)]
pub struct WalletProvider {
    pub id: &'static str,
    pub name: &'static str,
    /// A short single-glyph "icon" (emoji in the TS source). Used
    /// inside a circular badge in the dropdown header.
    pub icon: &'static str,
    /// Tailwind class fragment (bg-orange-500, etc.) used to tint
    /// the provider card.
    pub color: &'static str,
}

/// Static provider registry. Mirrors the TS `walletProviders` map
/// exactly — when adding a new connector, append an entry here AND
/// in the TS source.
pub const WALLET_PROVIDERS: &[WalletProvider] = &[
    WalletProvider { id: "metaMask",     name: "MetaMask",      icon: "\u{1F98A}", color: "bg-orange-500" },
    WalletProvider { id: "walletConnect",name: "WalletConnect", icon: "\u{1F517}", color: "bg-blue-500" },
    WalletProvider { id: "injected",    name: "Browser Wallet",icon: "\u{1F310}", color: "bg-purple-500" },
    WalletProvider { id: "coinbase",    name: "Coinbase",      icon: "\u{1F535}", color: "bg-blue-600" },
    WalletProvider { id: "rainbow",     name: "Rainbow",       icon: "\u{1F308}", color: "bg-gradient-to-r from-pink-500 to-violet-500" },
];

/// Look up a provider by connector id. Falls back to `injected`
/// (Browser Wallet) when the id is unknown, matching the TS `??`
/// fallback in `wallet-provider-icon.tsx`.
pub fn wallet_provider_for(id: &str) -> &'static WalletProvider {
    let needle = id.to_ascii_lowercase();
    WALLET_PROVIDERS
        .iter()
        .find(|p| p.id.eq_ignore_ascii_case(&needle))
        .unwrap_or_else(|| WALLET_PROVIDERS.iter().find(|p| p.id == "injected").expect("injected always present"))
}

// === ConnectButton ===

/// Size variant for `ConnectButton`. `Compact` matches the navbar's
/// "Connect" pill; `Full` is the wide mobile drawer CTA. `Default`
/// is the desktop / "Connect Wallet" hero button.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ConnectButtonSize {
    Compact,
    Default,
    Full,
}

impl Default for ConnectButtonSize {
    fn default() -> Self { Self::Default }
}

/// The "Connect Wallet" CTA. Renders an orange→purple gradient pill
/// per the TS shadcn source. When `on_click` is set the element is
/// a button; otherwise it's an anchor pointing to `href`
/// (default `/auth`).
#[component]
pub fn ConnectButton(
    /// Click handler. When set, the element becomes a button.
    #[props(default = None)] on_click: Option<EventHandler<MouseEvent>>,
    /// Href for the link fallback. Defaults to `/auth`.
    #[props(default = None)] href: Option<String>,
    /// Size variant. Defaults to `Default`.
    #[props(default = None)] size: Option<ConnectButtonSize>,
    /// Optional label override. Defaults to "Connect Wallet".
    #[props(default = None)] label: Option<String>,
    /// Disable the button. Defaults to `false`.
    #[props(default = false)] disabled: bool,
    /// Extra class names appended to the rendered element.
    #[props(default = None)] class_name: Option<String>,
) -> Element {
    let size_val = size.unwrap_or_default();
    let label_val = label.unwrap_or_else(|| {
        match size_val {
            ConnectButtonSize::Compact => "Connect".to_string(),
            _ => "Connect Wallet".to_string(),
        }
    });
    let href_val = href.unwrap_or_else(|| "/auth".to_string());
    let extra_cls = class_name.unwrap_or_default();
    let (size_cls, icon_size) = match size_val {
        ConnectButtonSize::Compact => ("connect-btn connect-btn-compact", 12u32),
        ConnectButtonSize::Default => ("connect-btn connect-btn-default", 16u32),
        ConnectButtonSize::Full => ("connect-btn connect-btn-full", 20u32),
    };
    let final_class = format!("{size_cls} {extra_cls}");
    rsx! {
        if let Some(h) = on_click.clone() {
            button {
                class: "{final_class}",
                r#type: "button",
                disabled: disabled,
                "aria-label": "Connect wallet",
                onclick: move |e| h.call(e),
                span { class: "connect-btn-icon", Icon { name: "wallet".to_string(), size: Some(icon_size) } }
                span { class: "connect-btn-label", "{label_val}" }
            }
        } else {
            a {
                class: "{final_class}",
                href: "{href_val}",
                "aria-label": "Connect wallet",
                span { class: "connect-btn-icon", Icon { name: "wallet".to_string(), size: Some(icon_size) } }
                span { class: "connect-btn-label", "{label_val}" }
            }
        }
    }
}

// === ConnectedWalletDropdown ===

/// Optional nav-link rendered inside `ConnectedWalletDropdown`.
/// Mirrors the `asChild` `Link` items in the TS shadcn
/// `DropdownMenuItem` rows.
#[derive(Clone, Debug, PartialEq)]
pub struct WalletNavLink {
    pub label: String,
    pub href: String,
    pub icon: Option<String>,
}

/// State bundle consumed by `ConnectedWalletDropdown`. Bundling
/// keeps the public API stable when new fields are added (callers
/// can use the `..Default::default()` pattern).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ConnectedWalletState {
    /// Connected wallet address. When `None`, the dropdown
    /// renders nothing (matches the TS `if (!displayAddress) return
    /// null;` guard).
    pub address: Option<String>,
    /// Connector id (e.g. "metaMask"). Used to pick the provider
    /// card variant.
    pub connector_id: Option<String>,
    /// Whether the user has completed the SIWE sign-in step.
    /// When `false`, the dropdown shows a "Sign In with Wallet"
    /// action.
    pub is_authenticated: bool,
    /// Current authentication attempt count. When `>= 3` and
    /// `last_error.is_some()`, the "Retry Authentication" row is
    /// shown.
    pub auth_retry_count: u32,
    /// Last authentication error message. When `Some`, the retry
    /// row shows on retry-count >= 3.
    pub last_error: Option<String>,
    /// Native BNB balance string, e.g. "0.1234". When `Some`, the
    /// admin variant shows a "Balance" row.
    pub balance: Option<String>,
    /// Current chain id. When `Some(56)` the network badge is green
    /// (BSC Mainnet live); `Some(97)` is yellow (BSC Testnet);
    /// any other value is gray.
    pub chain_id: Option<u64>,
    /// Whether the wallet is currently in the "authenticating"
    /// sub-state (signing / verifying). When `true`, the status
    /// row shows "Signing... (n/3)".
    pub is_authenticating: bool,
    /// User's role (e.g. "admin", "super_admin"). When set on the
    /// admin variant, the "Role" row is shown.
    pub role: Option<String>,
    /// User's tier level (e.g. "Pro", "Enterprise"). When set, the
    /// "Tier" row is shown.
    pub tier_level: Option<String>,
    /// Permission count. When > 0 the "Permissions" row is shown.
    pub perm_count: u32,
}

impl ConnectedWalletState {
    /// Wave 3a Track B — derive a server-side `ConnectedWalletState`
    /// from the inbound HTTP request headers.
    ///
    /// In Wave 3a the real wagmi-equivalent client writes a
    /// `WalletInfo` cookie on connect; we would parse it here to
    /// populate `address` / `connector_id` / `chain_id`. For this
    /// wave the parser is a no-op (returns `Self::default()`) so the
    /// BFF SSR path can be wired without waiting on the client-side
    /// cookie contract. A follow-up wave (3b or later) will replace
    /// the body with a real `WalletInfo` cookie parser — the
    /// signature is stable.
    ///
    /// `is_authenticated` is intentionally NOT derived from cookies
    /// here; it tracks the SIWE session lifetime, not the wallet
    /// connection lifetime. The BFF is expected to set it from
    /// `user.is_some()` after calling this helper.
    ///
    /// The function MUST accept the `HeaderMap` so callers compile
    /// even though it is currently ignored.
    pub fn from_cookies(headers: &axum::http::HeaderMap) -> Self {
        // Stub: real cookie read happens here in a follow-up wave. The
        // wagmi-equivalent client writes a WalletInfo cookie when
        // connected; we will parse it here. Until then, return the
        // default (all fields None / false / 0) so SSR never crashes
        // on a missing cookie.
        let _ = headers;
        Self::default()
    }
}

/// The connected-wallet dropdown card. Renders the full TS layout
/// (provider card header, copy + explorer actions, optional sign-in
/// row, retry banner, nav links, disconnect) condensed down to a
/// Dioxus-friendly structure.
///
/// All callbacks are `EventHandler<()>` because the actual SIWE
/// state is owned by the parent — the dropdown is a presentational
/// shell. The parent wires the callback to its own SIWE flow.
#[component]
pub fn ConnectedWalletDropdown(
    /// Connection state. When `state.address.is_none()`, the
    /// component renders nothing.
    state: ConnectedWalletState,
    /// Optional nav links rendered between the actions and the
    /// disconnect button. Use this for "Account Settings" /
    /// "Developer Portal" entries.
    #[props(default = None)] nav_links: Option<Vec<WalletNavLink>>,
    /// Click handler for the "Copy" action.
    #[props(default = None)] on_copy: Option<EventHandler<MouseEvent>>,
    /// Click handler for the "View on Explorer" action.
    #[props(default = None)] on_explorer: Option<EventHandler<MouseEvent>>,
    /// Click handler for the "Sign In with Wallet" action.
    #[props(default = None)] on_sign_in: Option<EventHandler<MouseEvent>>,
    /// Click handler for the "Retry Authentication" action.
    #[props(default = None)] on_retry: Option<EventHandler<MouseEvent>>,
    /// Click handler for the "Disconnect" action.
    #[props(default = None)] on_disconnect: Option<EventHandler<MouseEvent>>,
    /// Extra class names for the outer wrapper.
    #[props(default = None)] class_name: Option<String>,
    /// Whether the copy was just performed (shows "Copied!" instead
    /// of "Copy"). Defaults to `false`.
    #[props(default = false)] copied: bool,
    /// Show the nav links section. Defaults to `true` when at
    /// least one nav link is provided.
    #[props(default = true)] show_nav_links: bool,
    /// Show the role/tier/permissions meta grid (admin variant).
    /// Defaults to `false`.
    #[props(default = false)] show_meta: bool,
) -> Element {
    let address = match &state.address {
        Some(a) if !a.is_empty() => a.clone(),
        _ => return rsx! { Fragment {} },
    };
    let extra_cls = class_name.unwrap_or_default();
    let provider = wallet_provider_for(state.connector_id.as_deref().unwrap_or("injected"));

    // Derive status text + color, matching the TS `getStatus()` helper.
    let (status_text, status_color) = if state.auth_retry_count >= 3 && state.last_error.is_some() {
        ("Auth Failed", "wallet-status-error")
    } else if state.is_authenticating {
        ("Signing...", "wallet-status-warning")
    } else if state.is_authenticated {
        ("Authenticated", "wallet-status-success")
    } else {
        ("Connected", "wallet-status-neutral")
    };

    let show_sign_in_row = !state.is_authenticated && !state.is_authenticating;
    let show_retry_row = state.auth_retry_count >= 3 && state.last_error.is_some();

    let network_class = match state.chain_id {
        Some(56) => "wallet-network-live",
        Some(97) => "wallet-network-testnet",
        Some(_) => "wallet-network-other",
        None => "",
    };
    let network_label_text: String = match state.chain_id {
        Some(56) => "BSC Mainnet".to_string(),
        Some(97) => "BSC Testnet".to_string(),
        Some(other) => format!("Chain {other}"),
        None => String::new(),
    };

    let link_list: Vec<WalletNavLink> = nav_links.clone().unwrap_or_default();
    let has_links = show_nav_links && !link_list.is_empty();

    rsx! {
        div { class: "connected-wallet-dropdown {extra_cls}",
            // Provider card header
            div { class: "wallet-provider-card {provider.color}",
                div { class: "wallet-provider-icon",
                    span { class: "wallet-provider-glyph", "{provider.icon}" }
                }
                div { class: "wallet-provider-meta",
                    div { class: "wallet-provider-name", "{provider.name}" }
                    div { class: "wallet-provider-address", "{address}" }
                    div { class: "wallet-provider-status {status_color}",
                        if status_text == "Authenticated" {
                            span { class: "wallet-status-dot" }
                        }
                        "{status_text}"
                    }
                }
            }

            // Copy / Explorer quick actions
            div { class: "wallet-actions-row",
                button {
                    class: "wallet-action-btn",
                    r#type: "button",
                    onclick: move |e| {
                        if let Some(h) = &on_copy { h.call(e); }
                    },
                    if copied {
                        Icon { name: "check".to_string(), size: Some(14) }
                        span { "Copied!" }
                    } else {
                        Icon { name: "copy".to_string(), size: Some(14) }
                        span { "Copy" }
                    }
                }
                button {
                    class: "wallet-action-btn",
                    r#type: "button",
                    onclick: move |e| {
                        if let Some(h) = &on_explorer { h.call(e); }
                    },
                    Icon { name: "external-link".to_string(), size: Some(14) }
                    span { "Explorer" }
                }
            }

            // Optional meta grid (admin variant — role / tier / perms / balance)
            if show_meta {
                div { class: "wallet-meta-grid",
                    if let Some(role) = &state.role {
                        if !role.is_empty() {
                            div { class: "wallet-meta-cell",
                                div { class: "wallet-meta-label", "Role" }
                                div { class: "wallet-meta-value wallet-meta-value-role",
                                    "{role.replace('_', \" \")}"
                                }
                            }
                        }
                    }
                    if let Some(tier) = &state.tier_level {
                        if !tier.is_empty() {
                            div { class: "wallet-meta-cell",
                                div { class: "wallet-meta-label", "Tier" }
                                div { class: "wallet-meta-value wallet-meta-value-tier", "{tier}" }
                            }
                        }
                    }
                    if state.perm_count > 0 {
                        div { class: "wallet-meta-cell",
                            div { class: "wallet-meta-label", "Permissions" }
                            div { class: "wallet-meta-value", "{state.perm_count}" }
                        }
                    }
                    if let Some(bal) = &state.balance {
                        if !bal.is_empty() {
                            div { class: "wallet-meta-cell",
                                div { class: "wallet-meta-label", "Balance" }
                                div { class: "wallet-meta-value", "{bal} BNB" }
                            }
                        }
                    }
                }
                if !network_label_text.is_empty() {
                    div { class: "wallet-network-badge {network_class}",
                        span { class: "wallet-network-dot" }
                        span { "{network_label_text}" }
                    }
                }
            }

            // Sign-in row (connected but not authenticated)
            if show_sign_in_row {
                if let Some(h) = on_sign_in.clone() {
                    button {
                        class: "wallet-signin-row",
                        r#type: "button",
                        onclick: move |e| h.call(e),
                        Icon { name: "wallet".to_string(), size: Some(16) }
                        div { class: "wallet-signin-meta",
                            div { class: "wallet-signin-title", "Sign In with Wallet" }
                            div { class: "wallet-signin-sub", "Authenticate to access all features" }
                        }
                    }
                }
            }

            // Retry row (auth failed >= 3 attempts). The whole row
            // is wrapped in a `polite` live region so screen readers
            // announce the appearance of the retry CTA when the
            // auth-attempt count crosses the threshold. The visible
            // count text below ("Retry Authentication" / "Clear error
            // and try again") is the spoken text when the region
            // updates.
            if show_retry_row {
                div { class: "wallet-retry-row-wrap", "aria-live": "polite",
                    if let Some(h) = on_retry.clone() {
                        button {
                            class: "wallet-retry-row",
                            r#type: "button",
                            onclick: move |e| h.call(e),
                            Icon { name: "refresh-cw".to_string(), size: Some(16) }
                            div { class: "wallet-retry-meta",
                                div { class: "wallet-retry-title", "Retry Authentication" }
                                div { class: "wallet-retry-sub", "Clear error and try again" }
                            }
                        }
                    }
                }
            }

            // Nav links (Account / Developer / ...)
            if has_links {
                div { class: "wallet-nav-links",
                    for link in link_list.iter() {
                        a {
                            class: "wallet-nav-link",
                            href: "{link.href}",
                            if let Some(icon) = &link.icon {
                                Icon { name: icon.clone(), size: Some(16) }
                            }
                            span { "{link.label}" }
                        }
                    }
                }
            }

            // Disconnect
            if let Some(h) = on_disconnect.clone() {
                button {
                    class: "wallet-disconnect-btn",
                    r#type: "button",
                    onclick: move |e| h.call(e),
                    Icon { name: "log-out".to_string(), size: Some(16) }
                    span { "Disconnect" }
                }
            }
        }
    }
}

// === Wave 1 backwards-compat re-exports ===

/// Wave 1 API — kept verbatim so existing pages keep compiling.
/// New code should prefer `ConnectButton` + `ConnectedWalletDropdown`.
#[component]
pub fn WalletConnectButton(
    user: Option<User>,
    on_connect: Option<EventHandler<MouseEvent>>,
) -> Element {
    if let Some(u) = user {
        rsx! {
            a { class: "btn btn-primary wallet-connect-legacy", href: "/profile",
                span { Icon { name: "wallet".to_string(), size: Some(16) } }
                span { "{u.short_address()}" }
            }
        }
    } else {
        rsx! {
            ConnectButton {
                on_click: on_connect,
                size: Some(ConnectButtonSize::Compact),
                label: Some("Connect Wallet".to_string()),
                class_name: Some("wallet-connect-legacy".to_string()),
            }
        }
    }
}

/// Wave 1 legacy helper — kept as a non-`#[component]` plain
/// function. The TS source used to render this as a static pill;
/// the new `ConnectedWalletDropdown` is the rich version. Internal
/// callers can compose it; external code should migrate to the
/// new `ConnectedWalletDropdown` API. Re-exported as
/// `connected_wallet_pill` from `auth.rs`.
pub fn connected_wallet_pill(user: User) -> Element {
    rsx! {
        div { class: "connected-wallet",
            div { class: "wallet-pill",
                span { class: "wallet-status-dot" }
                span { class: "wallet-address", "{user.short_address()}" }
            }
            div { class: "wallet-balance",
                span { "0.00" }
                span { class: "text-muted-foreground text-sm", "BNB" }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Wave 3a Track B — the `from_cookies` stub must return
    /// `Default::default()` for an empty `HeaderMap`. This locks in
    /// the contract that SSR never panics on a missing cookie.
    #[test]
    fn from_cookies_returns_default_for_empty_headers() {
        let state = ConnectedWalletState::from_cookies(&axum::http::HeaderMap::new());
        let default = ConnectedWalletState::default();
        assert_eq!(state, default);
        // Defensive: also assert the field-level shape, so a future
        // change to `Default` impl (e.g. by adding a new `#[derive]`
        // default value) doesn't silently desync the test.
        assert!(state.address.is_none());
        assert!(state.connector_id.is_none());
        assert!(!state.is_authenticated);
        assert_eq!(state.auth_retry_count, 0);
        assert!(state.last_error.is_none());
        assert!(state.balance.is_none());
        assert!(state.chain_id.is_none());
        assert!(!state.is_authenticating);
        assert!(state.role.is_none());
        assert!(state.tier_level.is_none());
        assert_eq!(state.perm_count, 0);
    }
}
