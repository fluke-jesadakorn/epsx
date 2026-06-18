//! Admin `WalletFilterBar` component — Wave 38a T1 admin wallet
//! domain port.
//!
//! Mirrors `apps-old/admin-frontend/components/wallet/wallet-filter-bar.tsx`.
//! Renders the filter row above the wallet list: search input +
//! status / platform / sort-by selects + sort-order toggle.
//!
//! ## Filter values
//!
//! - `status`: "all" | "active" | "disabled"
//! - `platform`: "all" | "analytics" | "pay" | "token" | "markets"
//! - `sort_by`: "created_at" | "last_auth_at" | "wallet_address"
//! - `sort_order`: "asc" | "desc"
//!
//! The full filter shape is wrapped in a `WalletFilters` struct so
//! callers can swap it as a single value.
//!
//! ## Tests
//!
//! `test_wallet_filter_bar_renders_search_input` — search input
//! renders with the placeholder.
//! `test_wallet_filter_bar_renders_4_controls` — status / platform
//! / sort-by / sort-order toggle all render.
//! `test_wallet_filter_bar_renders_current_search_value` — the
//! initial search text renders.
//! `test_wallet_filter_bar_propagates_class_name` — caller
//! `class_name` is appended.

use dioxus::prelude::*;

/// Filter values for the wallet list. Mirrors the prod
/// `WalletFilters` shape.
#[derive(Clone, Debug, PartialEq)]
pub struct WalletFilters {
    pub search: String,
    pub status: String,
    pub platform: String,
    pub sort_by: String,
    pub sort_order: String,
}

impl WalletFilters {
    /// Empty / default filter (all rows, sort by created_at desc).
    pub fn empty() -> Self {
        Self {
            search: String::new(),
            status: "all".to_string(),
            platform: "all".to_string(),
            sort_by: "created_at".to_string(),
            sort_order: "desc".to_string(),
        }
    }
}

/// Wallet filter bar — search + status + platform + sort-by +
/// sort-order toggle. Caller passes the current filter + a change
/// handler.
#[component]
pub fn WalletFilterBar(
    /// Current filter state.
    filters: WalletFilters,
    /// Handler called with the new filter whenever any input
    /// changes.
    on_filter_change: EventHandler<WalletFilters>,
    /// Optional extra classes appended to the outer wrapper.
    class_name: Option<String>,
) -> Element {
    let mut cls =
        "flex flex-col sm:flex-row gap-3 bg-card p-4 rounded-2xl border border-border/20 shadow-lg".to_string();
    if let Some(extra) = class_name {
        if !extra.is_empty() {
            cls.push(' ');
            cls.push_str(&extra);
        }
    }

    let trigger_cls = "h-10 bg-muted/30 border-border/30 rounded-xl text-sm hover:border-border/50 transition-colors";

    let filters_for_search = filters.clone();
    let filters_for_status = filters.clone();
    let filters_for_platform = filters.clone();
    let filters_for_sort_by = filters.clone();
    let filters_for_sort_toggle = filters.clone();

    rsx! {
        div { class: "{cls}",
            // Search
            div { class: "relative flex-1 min-w-0",
                // search icon (lucide)
                svg {
                    class: "absolute left-3.5 top-1/2 -translate-y-1/2 h-4 w-4 text-muted-foreground pointer-events-none",
                    fill: "none",
                    stroke: "currentColor",
                    view_box: "0 0 24 24",
                    path {
                        stroke_linecap: "round",
                        stroke_linejoin: "round",
                        stroke_width: "2",
                        d: "M21 21l-4.35-4.35 M11 19a8 8 0 100-16 8 8 0 000 16z",
                    }
                }
                input {
                    r#type: "search",
                    placeholder: "Search address, label, or note...",
                    class: "w-full pl-10 h-10 bg-muted/30 border border-border/30 focus:border-[#1fc7d4]/50 transition-colors rounded-xl placeholder:text-muted-foreground/40 text-sm",
                    value: "{filters.search}",
                    oninput: move |e| {
                        let mut next = filters_for_search.clone();
                        next.search = e.value();
                        on_filter_change.call(next);
                    },
                }
            }
            // Controls
            div { class: "flex items-center gap-2 flex-shrink-0 flex-wrap",
                // Status select
                select {
                    class: "w-[120px] {trigger_cls}",
                    value: "{filters.status}",
                    onchange: move |e| {
                        let mut next = filters_for_status.clone();
                        next.status = e.value();
                        on_filter_change.call(next);
                    },
                    option { value: "all", "All Status" }
                    option { value: "active", "Active" }
                    option { value: "disabled", "Disabled" }
                }
                // Platform select
                select {
                    class: "w-[130px] {trigger_cls}",
                    value: "{filters.platform}",
                    onchange: move |e| {
                        let mut next = filters_for_platform.clone();
                        next.platform = e.value();
                        on_filter_change.call(next);
                    },
                    option { value: "all", "All Platforms" }
                    option { value: "analytics", "Analytics" }
                    option { value: "pay", "Pay" }
                    option { value: "token", "Token" }
                    option { value: "markets", "Markets" }
                }
                // Sort-by select
                select {
                    class: "w-[140px] {trigger_cls}",
                    value: "{filters.sort_by}",
                    onchange: move |e| {
                        let mut next = filters_for_sort_by.clone();
                        next.sort_by = e.value();
                        on_filter_change.call(next);
                    },
                    option { value: "created_at", "Date Created" }
                    option { value: "last_auth_at", "Last Active" }
                    option { value: "wallet_address", "Address" }
                }
                // Sort-order toggle
                button {
                    r#type: "button",
                    class: "h-10 w-10 rounded-xl bg-muted/30 border border-border/30 hover:border-[#1fc7d4]/40 hover:bg-muted/50 hover:text-[#1fc7d4] transition-colors flex items-center justify-center text-muted-foreground flex-shrink-0",
                    title: if filters.sort_order == "desc" { "Descending — click to toggle" } else { "Ascending — click to toggle" },
                    onclick: move |_| {
                        let mut next = filters_for_sort_toggle.clone();
                        next.sort_order = if next.sort_order == "desc" { "asc".to_string() } else { "desc".to_string() };
                        on_filter_change.call(next);
                    },
                    // arrows-up-down icon (lucide)
                    svg {
                        class: "h-4 w-4 transition-transform duration-300",
                        fill: "none",
                        stroke: "currentColor",
                        view_box: "0 0 24 24",
                        path {
                            stroke_linecap: "round",
                            stroke_linejoin: "round",
                            stroke_width: "2",
                            d: "M7 4v16 M3 8l4-4 4 4 M17 20V4 M21 16l-4 4-4-4",
                        }
                    }
                }
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

    fn sample_filters() -> WalletFilters {
        WalletFilters {
            search: "0xabc".to_string(),
            status: "active".to_string(),
            platform: "analytics".to_string(),
            sort_by: "last_auth_at".to_string(),
            sort_order: "asc".to_string(),
        }
    }

    /// Search input renders with the placeholder.
    #[test]
    fn test_wallet_filter_bar_renders_search_input() {
        fn render() -> Element {
            rsx! {
            WalletFilterBar {
                filters: sample_filters(),
                on_filter_change: EventHandler::new(|_: WalletFilters| {}),
            }
        }
        

        }

        let mut vdom = dioxus::prelude::VirtualDom::new(render);
        vdom.rebuild_in_place();
        let html = dioxus_ssr::render(&vdom);
        assert!(
            html.contains("Search address, label, or note..."),
            "Search placeholder should render. Got: {html}"
        );
    }

    /// All 4 controls render.
    #[test]
    fn test_wallet_filter_bar_renders_4_controls() {
        fn render() -> Element {
            rsx! {
            WalletFilterBar {
                filters: sample_filters(),
                on_filter_change: EventHandler::new(|_: WalletFilters| {}),
            }
        }
        

        }

        let mut vdom = dioxus::prelude::VirtualDom::new(render);
        vdom.rebuild_in_place();
        let html = dioxus_ssr::render(&vdom);
        assert!(html.contains("All Status"), "Status options render. Got: {html}");
        assert!(html.contains("All Platforms"), "Platform options render. Got: {html}");
        assert!(html.contains("Date Created"), "Sort-by options render. Got: {html}");
        assert!(
            html.contains("Ascending"),
            "Sort-order tooltip should reflect asc state. Got: {html}"
        );
    }

    /// The initial search value renders.
    #[test]
    fn test_wallet_filter_bar_renders_current_search_value() {
        fn render() -> Element {
            rsx! {
            WalletFilterBar {
                filters: sample_filters(),
                on_filter_change: EventHandler::new(|_: WalletFilters| {}),
            }
        }
        

        }

        let mut vdom = dioxus::prelude::VirtualDom::new(render);
        vdom.rebuild_in_place();
        let html = dioxus_ssr::render(&vdom);
        assert!(
            html.contains("0xabc"),
            "Initial search value should render. Got: {html}"
        );
    }

    /// Descending sort order renders the matching tooltip.
    #[test]
    fn test_wallet_filter_bar_descending_tooltip() {
        fn render() -> Element {
            let mut f = sample_filters();
            f.sort_order = "desc".to_string();
            rsx! {
            WalletFilterBar {
                filters: f,
                on_filter_change: EventHandler::new(|_: WalletFilters| {}),
            }
        }
        }

        let mut vdom = dioxus::prelude::VirtualDom::new(render);
        vdom.rebuild_in_place();
        let html = dioxus_ssr::render(&vdom);
        assert!(
            html.contains("Descending"),
            "Sort-order tooltip should reflect desc state. Got: {html}"
        );
    }

    /// `class_name` is propagated.
    #[test]
    fn test_wallet_filter_bar_propagates_class_name() {
        fn render() -> Element {
            rsx! {
            WalletFilterBar {
                filters: sample_filters(),
                on_filter_change: EventHandler::new(|_: WalletFilters| {}),
                class_name: Some("mt-4".to_string()),
            }
        }
        

        }

        let mut vdom = dioxus::prelude::VirtualDom::new(render);
        vdom.rebuild_in_place();
        let html = dioxus_ssr::render(&vdom);
        assert!(
            html.contains("mt-4"),
            "class_name should propagate. Got: {html}"
        );
    }

    /// Default filter is "all" / "created_at" / "desc".
    #[test]
    fn test_wallet_filters_empty_default() {
        let f = WalletFilters::empty();
        assert_eq!(f.search, "");
        assert_eq!(f.status, "all");
        assert_eq!(f.platform, "all");
        assert_eq!(f.sort_by, "created_at");
        assert_eq!(f.sort_order, "desc");
    }
}
