//! Admin `WalletDetailHeader` component — Wave 38a T1 admin wallet
//! domain port.
//!
//! Mirrors `apps-old/admin-frontend/components/wallet/wallet-detail-header.tsx`.
//! Renders the wallet detail page header row: back-link + title
//! + subtitle + refresh button.
//!
//! ## Tests
//!
//! `test_wallet_detail_header_renders_title` — the title and
//! subtitle render.
//! `test_wallet_detail_header_renders_back_link` — the back-link
//! renders with the `←` glyph.
//! `test_wallet_detail_header_renders_refresh_button` — the
//! refresh button renders.
//! `test_wallet_detail_header_propagates_class_name` — caller
//! `class_name` is appended.

use dioxus::prelude::*;

/// Wallet detail page header — back-link + title + subtitle +
/// refresh button.
#[component]
pub fn WalletDetailHeader(
    /// Page title (e.g. "Wallet Details").
    title: String,
    /// Page subtitle / description.
    subtitle: String,
    /// Whether the refresh button is in the loading state
    /// (renders the spin animation).
    is_refreshing: Option<bool>,
    /// Click handler for the refresh button.
    on_refresh: EventHandler<MouseEvent>,
    /// Optional extra classes appended to the outer wrapper.
    class_name: Option<String>,
) -> Element {
    let is_refreshing = is_refreshing.unwrap_or(false);
    let mut cls = "flex items-center gap-4".to_string();
    if let Some(extra) = class_name {
        if !extra.is_empty() {
            cls.push(' ');
            cls.push_str(&extra);
        }
    }
    rsx! {
        div { class: "{cls}",
            // Back link
            a {
                href: "/wallet-management",
                class: "p-2 rounded-xl bg-card border border-border/40 hover:bg-muted/30 transition-colors",
                title: "Back to wallets",
                // arrow-left icon (lucide)
                svg {
                    class: "h-5 w-5",
                    fill: "none",
                    stroke: "currentColor",
                    view_box: "0 0 24 24",
                    path {
                        stroke_linecap: "round",
                        stroke_linejoin: "round",
                        stroke_width: "2",
                        d: "M19 12H5 M12 19l-7-7 7-7",
                    }
                }
            }
            // Title block
            div { class: "flex-1",
                h1 { class: "text-2xl font-bold flex items-center gap-2",
                    span { "👛" }
                    "{title}"
                }
                p { class: "text-sm text-muted-foreground", "{subtitle}" }
            }
            // Refresh button
            button {
                r#type: "button",
                class: "px-4 py-2 rounded-md border border-border bg-card text-sm font-medium hover:bg-muted flex items-center gap-2 disabled:opacity-50",
                disabled: is_refreshing,
                onclick: move |e| on_refresh.call(e),
                // refresh-cw icon (lucide)
                svg {
                    class: if is_refreshing { "h-4 w-4 animate-spin" } else { "h-4 w-4" },
                    fill: "none",
                    stroke: "currentColor",
                    view_box: "0 0 24 24",
                    path {
                        stroke_linecap: "round",
                        stroke_linejoin: "round",
                        stroke_width: "2",
                        d: "M23 4v6h-6 M1 20v-6h6 M3.51 9a9 9 0 0114.85-3.36L23 10 M1 14l4.64 4.36A9 9 0 0020.49 15",
                    }
                }
                "Refresh"
            }
        }
    }
}

// =====================================================================
// Tests
// =====================================================================

#[cfg(test)]
mod tests {
    use super::*;

    /// Title and subtitle render.
    #[test]
    fn test_wallet_detail_header_renders_title() {
        fn render() -> Element {
            rsx! {
            WalletDetailHeader {
                title: "Wallet Details".to_string(),
                subtitle: "Manage wallet access and plans".to_string(),
                on_refresh: EventHandler::new(|_: MouseEvent| {}),
            }
        }
        

        }

        let mut vdom = dioxus::prelude::VirtualDom::new(render);
        vdom.rebuild_in_place();
        let html = dioxus_ssr::render(&vdom);
        assert!(
            html.contains("Wallet Details"),
            "Title should render. Got: {html}"
        );
        assert!(
            html.contains("Manage wallet access and plans"),
            "Subtitle should render. Got: {html}"
        );
        assert!(html.contains("👛"), "Wallet emoji should render. Got: {html}");
    }

    /// Back link renders.
    #[test]
    fn test_wallet_detail_header_renders_back_link() {
        fn render() -> Element {
            rsx! {
            WalletDetailHeader {
                title: "T".to_string(),
                subtitle: "S".to_string(),
                on_refresh: EventHandler::new(|_: MouseEvent| {}),
            }
        }
        

        }

        let mut vdom = dioxus::prelude::VirtualDom::new(render);
        vdom.rebuild_in_place();
        let html = dioxus_ssr::render(&vdom);
        assert!(
            html.contains("href=\"/wallet-management\""),
            "Back link href should render. Got: {html}"
        );
        assert!(
            html.contains("Back to wallets"),
            "Back link tooltip should render. Got: {html}"
        );
    }

    /// Refresh button renders.
    #[test]
    fn test_wallet_detail_header_renders_refresh_button() {
        fn render() -> Element {
            rsx! {
            WalletDetailHeader {
                title: "T".to_string(),
                subtitle: "S".to_string(),
                on_refresh: EventHandler::new(|_: MouseEvent| {}),
            }
        }
        

        }

        let mut vdom = dioxus::prelude::VirtualDom::new(render);
        vdom.rebuild_in_place();
        let html = dioxus_ssr::render(&vdom);
        assert!(
            html.contains("Refresh"),
            "Refresh button label should render. Got: {html}"
        );
    }

    /// Refreshing state renders the spin class.
    #[test]
    fn test_wallet_detail_header_refreshing_state() {
        fn render() -> Element {
            rsx! {
            WalletDetailHeader {
                title: "T".to_string(),
                subtitle: "S".to_string(),
                is_refreshing: Some(true),
                on_refresh: EventHandler::new(|_: MouseEvent| {}),
            }
        }
        

        }

        let mut vdom = dioxus::prelude::VirtualDom::new(render);
        vdom.rebuild_in_place();
        let html = dioxus_ssr::render(&vdom);
        assert!(
            html.contains("animate-spin"),
            "Refreshing state should render animate-spin. Got: {html}"
        );
        assert!(
            html.contains("disabled"),
            "Refresh button should be disabled while refreshing. Got: {html}"
        );
    }

    /// `class_name` is propagated.
    #[test]
    fn test_wallet_detail_header_propagates_class_name() {
        fn render() -> Element {
            rsx! {
            WalletDetailHeader {
                title: "T".to_string(),
                subtitle: "S".to_string(),
                on_refresh: EventHandler::new(|_: MouseEvent| {}),
                class_name: Some("mb-6".to_string()),
            }
        }
        

        }

        let mut vdom = dioxus::prelude::VirtualDom::new(render);
        vdom.rebuild_in_place();
        let html = dioxus_ssr::render(&vdom);
        assert!(
            html.contains("mb-6"),
            "class_name should propagate. Got: {html}"
        );
    }
}
