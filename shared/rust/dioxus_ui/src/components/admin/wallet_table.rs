//! Admin `WalletTable` component — Wave 38a T1 admin wallet domain
//! port.
//!
//! Mirrors `apps-old/admin-frontend/components/wallet/wallet-table.tsx`.
//! Renders a sortable, scrollable wallet table with header columns
//! (Wallet & Label / Plan / Status / Actions) and an iterable body
//! of `AdminWalletTableRow` rows.
//!
//! Caller passes the wallet list + selected-address set + action
//! handlers. SSR-friendly: works without JavaScript hydration.
//!
//! ## Tests
//!
//! `test_wallet_table_renders_all_columns` — the 4 column headers
//! render.
//! `test_wallet_table_renders_rows_for_each_wallet` — one row per
//! wallet.
//! `test_wallet_table_empty_state` — empty list → "No wallets
//! found" message.

use dioxus::prelude::*;

use super::wallet_table_row::{AdminWalletTableRow, WalletRowData};

/// Wallet table — wraps a list of `AdminWalletTableRow` rows in the
/// design-system scrollable, rounded border, header-row layout.
#[component]
pub fn WalletTable(
    /// List of wallet rows to render.
    wallets: Vec<WalletRowData>,
    /// Set of selected wallet addresses.
    selected_addresses: Vec<String>,
    /// Toggle handler — receives the wallet address + new state.
    on_select_wallet: EventHandler<(String, bool)>,
    /// View-detail handler.
    on_view: EventHandler<WalletRowData>,
    /// Manage-access handler.
    on_manage: EventHandler<WalletRowData>,
    /// Disable handler.
    on_disable: EventHandler<WalletRowData>,
    /// Enable handler.
    on_enable: EventHandler<WalletRowData>,
    /// Edit-metadata handler.
    on_edit: EventHandler<WalletRowData>,
    /// Optional extra classes appended to the outer wrapper.
    class_name: Option<String>,
) -> Element {
    let mut cls =
        "w-full overflow-auto rounded-xl border border-border/60 bg-card".to_string();
    if let Some(extra) = class_name {
        if !extra.is_empty() {
            cls.push(' ');
            cls.push_str(&extra);
        }
    }
    rsx! {
        div { class: "{cls}",
            table { class: "w-full text-sm text-left border-collapse",
                thead {
                    tr { class: "border-b border-border/60 bg-muted/30",
                        th { class: "p-4 w-10", "" }
                        th { class: "p-4 font-semibold text-muted-foreground whitespace-nowrap", "Wallet & Label" }
                        th { class: "p-4 font-semibold text-muted-foreground whitespace-nowrap", "Plan" }
                        th { class: "p-4 font-semibold text-muted-foreground whitespace-nowrap", "Status" }
                        th { class: "p-4 font-semibold text-muted-foreground whitespace-nowrap text-right", "Actions" }
                    }
                }
                tbody { class: "divide-y divide-border/40",
                    if wallets.is_empty() {
                        tr {
                            td {
                                colspan: "5",
                                class: "p-8 text-center text-muted-foreground",
                                "No wallets found"
                            }
                        }
                    } else {
                        for w in wallets.iter() {
                            {
                                let is_selected = selected_addresses.contains(&w.wallet_address);
                                rsx! {
                                    AdminWalletTableRow {
                                        key: "{w.wallet_address}",
                                        wallet: w.clone(),
                                        is_selected: is_selected,
                                        on_select_wallet: on_select_wallet,
                                        on_view: on_view,
                                        on_manage: on_manage,
                                        on_disable: on_disable,
                                        on_enable: on_enable,
                                        on_edit: on_edit,
                                    }
                                }
                            }
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

    fn sample_wallets() -> Vec<WalletRowData> {
        vec![
            WalletRowData {
                wallet_address: "0xaaaa000000000000000000000000000000000001".to_string(),
                label: Some("A".to_string()),
                plan: "Premium".to_string(),
                status: "active".to_string(),
            },
            WalletRowData {
                wallet_address: "0xbbbb000000000000000000000000000000000002".to_string(),
                label: None,
                plan: "Basic".to_string(),
                status: "disabled".to_string(),
            },
        ]
    }

    /// All 4 column headers render.
    #[test]
    fn test_wallet_table_renders_all_columns() {
        fn render() -> Element {
            rsx! {
            WalletTable {
                wallets: sample_wallets(),
                selected_addresses: vec![],
                on_select_wallet: EventHandler::new(|_: (String, bool)| {}),
                on_view: EventHandler::new(|_: WalletRowData| {}),
                on_manage: EventHandler::new(|_: WalletRowData| {}),
                on_disable: EventHandler::new(|_: WalletRowData| {}),
                on_enable: EventHandler::new(|_: WalletRowData| {}),
                on_edit: EventHandler::new(|_: WalletRowData| {}),
            }
        }
        

        }

        let mut vdom = dioxus::prelude::VirtualDom::new(render);
        vdom.rebuild_in_place();
        let html = dioxus_ssr::render(&vdom);
        assert!(
            html.contains("Wallet &amp; Label")
                || html.contains("Wallet &#38; Label")
                || html.contains("Wallet & Label"),
            "Wallet & Label column should render. Got: {html}"
        );
        assert!(html.contains("Plan"), "Plan column should render. Got: {html}");
        assert!(html.contains("Status"), "Status column should render. Got: {html}");
        assert!(
            html.contains("Actions"),
            "Actions column should render. Got: {html}"
        );
    }

    /// One row per wallet.
    #[test]
    fn test_wallet_table_renders_rows_for_each_wallet() {
        fn render() -> Element {
            rsx! {
            WalletTable {
                wallets: sample_wallets(),
                selected_addresses: vec![],
                on_select_wallet: EventHandler::new(|_: (String, bool)| {}),
                on_view: EventHandler::new(|_: WalletRowData| {}),
                on_manage: EventHandler::new(|_: WalletRowData| {}),
                on_disable: EventHandler::new(|_: WalletRowData| {}),
                on_enable: EventHandler::new(|_: WalletRowData| {}),
                on_edit: EventHandler::new(|_: WalletRowData| {}),
            }
        }
        

        }

        let mut vdom = dioxus::prelude::VirtualDom::new(render);
        vdom.rebuild_in_place();
        let html = dioxus_ssr::render(&vdom);
        assert!(
            html.contains("0xaaaa"),
            "First wallet address should render. Got: {html}"
        );
        assert!(
            html.contains("0xbbbb"),
            "Second wallet address should render. Got: {html}"
        );
        assert!(html.contains("Premium"), "First plan should render. Got: {html}");
        assert!(html.contains("Basic"), "Second plan should render. Got: {html}");
    }

    /// Empty list → "No wallets found" empty state.
    #[test]
    fn test_wallet_table_empty_state() {
        fn render() -> Element {
            rsx! {
            WalletTable {
                wallets: vec![],
                selected_addresses: vec![],
                on_select_wallet: EventHandler::new(|_: (String, bool)| {}),
                on_view: EventHandler::new(|_: WalletRowData| {}),
                on_manage: EventHandler::new(|_: WalletRowData| {}),
                on_disable: EventHandler::new(|_: WalletRowData| {}),
                on_enable: EventHandler::new(|_: WalletRowData| {}),
                on_edit: EventHandler::new(|_: WalletRowData| {}),
            }
        }
        

        }

        let mut vdom = dioxus::prelude::VirtualDom::new(render);
        vdom.rebuild_in_place();
        let html = dioxus_ssr::render(&vdom);
        assert!(
            html.contains("No wallets found"),
            "Empty state should render. Got: {html}"
        );
    }

    /// `class_name` is propagated.
    #[test]
    fn test_wallet_table_propagates_class_name() {
        fn render() -> Element {
            rsx! {
            WalletTable {
                wallets: sample_wallets(),
                selected_addresses: vec![],
                on_select_wallet: EventHandler::new(|_: (String, bool)| {}),
                on_view: EventHandler::new(|_: WalletRowData| {}),
                on_manage: EventHandler::new(|_: WalletRowData| {}),
                on_disable: EventHandler::new(|_: WalletRowData| {}),
                on_enable: EventHandler::new(|_: WalletRowData| {}),
                on_edit: EventHandler::new(|_: WalletRowData| {}),
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
}
