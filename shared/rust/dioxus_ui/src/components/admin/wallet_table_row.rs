//! Admin `WalletTableRow` (renamed `AdminWalletTableRow` to avoid
//! collision with the inline `fn WalletTableRow` in
//! `pages::admin_pages::wallet_wallets`) — Wave 38a T1 admin wallet
//! domain port.
//!
//! Mirrors `apps-old/admin-frontend/components/wallet/wallet-table-row.tsx`.
//! Renders a single row inside the wallet table: checkbox + address
//! + label + plan badge + status + actions dropdown.
//!
//! ## Action handlers
//!
//! All action callbacks are `EventHandler<MouseEvent>` so the
//! parent can attach click handlers without bridging through a
//! closure. The callback receives a `MouseEvent` so the caller
//! can `stop_propagation()` if needed (matches the prod behavior
//! where row clicks shouldn't bubble to a parent Link).
//!
//! ## Tests
//!
//! `test_admin_wallet_table_row_renders_address` — the address
//! renders inside the row.
//! `test_admin_wallet_table_row_renders_label_badge_when_present` /
//! `_hides_label_badge_when_absent` — label badge visibility matches
//! the source data.
//! `test_admin_wallet_table_row_renders_status_active_disabled` —
//! the status column uses the right color class.
//! `test_admin_wallet_table_row_disabled_row_has_opacity` — disabled
//! wallets get the opacity-70 class.

use dioxus::prelude::*;

/// Wallet status string used by `AdminWalletTableRow`. Mirrors the
/// prod `WalletStatus` type.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum WalletRowStatus {
    Active,
    Disabled,
}

impl WalletRowStatus {
    pub fn from_str(s: &str) -> Self {
        match s {
            "disabled" => WalletRowStatus::Disabled,
            _ => WalletRowStatus::Active,
        }
    }

    /// Tailwind dot color (small circle before the label).
    pub fn dot_class(&self) -> &'static str {
        match self {
            WalletRowStatus::Active => "bg-success",
            WalletRowStatus::Disabled => "bg-warning",
        }
    }

    /// Tailwind text color for the status label.
    pub fn text_class(&self) -> &'static str {
        match self {
            WalletRowStatus::Active => "text-success",
            WalletRowStatus::Disabled => "text-warning",
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            WalletRowStatus::Active => "active",
            WalletRowStatus::Disabled => "disabled",
        }
    }
}

/// Wallet row data payload. Mirrors a subset of the prod
/// `WalletData` shape used by the table row.
#[derive(Clone, Debug, PartialEq)]
pub struct WalletRowData {
    pub wallet_address: String,
    pub label: Option<String>,
    pub plan: String,
    pub status: String,
}

#[component]
fn PlanBadge(plan: String) -> Element {
    rsx! {
        span { class: "px-2 py-0 text-[10px] uppercase font-bold tracking-wider rounded bg-primary/5 text-primary border border-primary/10",
            "{plan}"
        }
    }
}

#[component]
fn StatusPill(status: WalletRowStatus) -> Element {
    rsx! {
        div { class: "flex items-center gap-2",
            div { class: "h-1.5 w-1.5 rounded-full {status.dot_class()}" }
            span { class: "capitalize text-xs font-medium {status.text_class()}", "{status.label()}" }
        }
    }
}

/// Wallet table row. Renamed `AdminWalletTableRow` to avoid
/// collision with the inline `fn WalletTableRow` in
/// `wallet_wallets.rs`. Migration: delete the inline fn and `use
/// crate::components::admin::AdminWalletTableRow`.
#[component]
pub fn AdminWalletTableRow(
    /// Row payload (address + label + plan + status).
    wallet: WalletRowData,
    /// Whether the row's checkbox is checked.
    is_selected: bool,
    /// Toggle handler — receives the wallet address + the new
    /// selected state.
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
) -> Element {
    let status_kind = WalletRowStatus::from_str(&wallet.status);
    let is_disabled = status_kind == WalletRowStatus::Disabled;
    let mut cls = "group hover:bg-muted/30 transition-colors".to_string();
    if is_selected {
        cls.push_str(" bg-primary/5");
    }
    if is_disabled {
        cls.push_str(" opacity-70");
    }

    let addr_for_select = wallet.wallet_address.clone();
    let addr_for_checkbox = wallet.wallet_address.clone();
    let wallet_for_view = wallet.clone();
    let wallet_for_manage = wallet.clone();
    let wallet_for_disable = wallet.clone();
    let wallet_for_enable = wallet.clone();
    let wallet_for_edit = wallet.clone();
    let label_value = wallet.label.clone().unwrap_or_default();
    let address_display = wallet.wallet_address.clone();

    rsx! {
        tr { class: "{cls}",
            td { class: "p-4",
                input {
                    r#type: "checkbox",
                    class: "h-4 w-4 rounded border-border text-primary focus:ring-primary cursor-pointer",
                    checked: is_selected,
                    onchange: move |e| {
                        on_select_wallet.call((addr_for_select.clone(), e.checked()));
                    },
                }
            }
            td { class: "p-4",
                div { class: "flex flex-col min-w-0",
                    div { class: "flex items-center gap-2",
                        span { class: "font-mono text-xs font-medium text-foreground truncate max-w-[120px] md:max-w-none",
                            "{address_display}"
                        }
                        if !label_value.is_empty() {
                            span { class: "hidden sm:inline-flex",
                                {
                                    let label_for_badge = label_value.clone();
                                    rsx! {
                                        crate::components::admin::WalletLabelBadge {
                                            label: label_for_badge,
                                            size: Some("sm".to_string()),
                                        }
                                    }
                                }
                            }
                        }
                    }
                    if !label_value.is_empty() {
                        span { class: "text-[10px] text-muted-foreground mt-0.5 sm:hidden truncate",
                            "{label_value}"
                        }
                    }
                }
            }
            td { class: "p-4",
                PlanBadge { plan: wallet.plan.clone() }
            }
            td { class: "p-4",
                StatusPill { status: status_kind }
            }
            td { class: "p-4 text-right",
                div { class: "flex items-center justify-end gap-1",
                    button {
                        r#type: "button",
                        class: "h-8 w-8 text-muted-foreground hover:text-primary transition-colors rounded-md hover:bg-muted",
                        title: "View Details",
                        onclick: move |_| {
                            on_view.call(wallet_for_view.clone());
                        },
                        // eye icon (lucide)
                        svg {
                            class: "h-4 w-4 mx-auto",
                            fill: "none",
                            stroke: "currentColor",
                            view_box: "0 0 24 24",
                            path {
                                stroke_linecap: "round",
                                stroke_linejoin: "round",
                                stroke_width: "2",
                                d: "M2 12s3-7 10-7 10 7 10 7-3 7-10 7-10-7-10-7z",
                            }
                            circle {
                                cx: "12",
                                cy: "12",
                                r: "3",
                            }
                        }
                    }
                    button {
                        r#type: "button",
                        class: "h-8 w-8 text-muted-foreground hover:text-primary transition-colors rounded-md hover:bg-muted",
                        title: "Edit Metadata",
                        onclick: move |_| {
                            on_edit.call(wallet_for_edit.clone());
                        },
                        // pencil icon (lucide)
                        svg {
                            class: "h-4 w-4 mx-auto",
                            fill: "none",
                            stroke: "currentColor",
                            view_box: "0 0 24 24",
                            path {
                                stroke_linecap: "round",
                                stroke_linejoin: "round",
                                stroke_width: "2",
                                d: "M12 20h9 M16.5 3.5a2.121 2.121 0 113 3L7 19l-4 1 1-4 12.5-12.5z",
                            }
                        }
                    }
                    // Dropdown trigger (rendered as a button for
                    // SSR simplicity — full Radix dropdown is not
                    // needed for the visual structure).
                    button {
                        r#type: "button",
                        class: "h-8 w-8 text-muted-foreground hover:text-primary transition-colors rounded-md hover:bg-muted",
                        title: "More actions",
                        onclick: move |_| {
                            // Default to Manage when dropdown is
                            // not implemented in SSR.
                            on_manage.call(wallet_for_manage.clone());
                        },
                        svg {
                            class: "h-4 w-4 mx-auto",
                            fill: "none",
                            stroke: "currentColor",
                            view_box: "0 0 24 24",
                            path {
                                stroke_linecap: "round",
                                stroke_linejoin: "round",
                                stroke_width: "2",
                                d: "M12 12h.01 M12 6h.01 M12 18h.01",
                            }
                        }
                    }
                    // Hidden spacer to expose an Enable/Disable
                    // button for keyboard/screen-reader users
                    // (the prod dropdown opens a menu with these).
                    if is_disabled {
                        button {
                            r#type: "button",
                            class: "h-8 px-2 text-xs rounded-md text-success hover:bg-success/10 hidden",
                            title: "Re-enable",
                            "data-action": "reenable",
                            onclick: move |_| {
                                on_enable.call(wallet_for_enable.clone());
                            },
                            "🔓 Re-enable"
                        }
                    } else {
                        button {
                            r#type: "button",
                            class: "h-8 px-2 text-xs rounded-md text-warning hover:bg-warning/10 hidden",
                            title: "Disable",
                            "data-action": "disable",
                            onclick: move |_| {
                                on_disable.call(wallet_for_disable.clone());
                            },
                            "⚠️ Disable"
                        }
                    }
                    // SSR-friendly data attribute so callers /
                    // tests can detect which row is which.
                    span { class: "sr-only", "data-wallet-address": "{addr_for_checkbox}" }
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

    fn sample_row() -> WalletRowData {
        WalletRowData {
            wallet_address: "0x1234567890abcdef1234567890abcdef12345678".to_string(),
            label: Some("VIP".to_string()),
            plan: "Premium".to_string(),
            status: "active".to_string(),
        }
    }

    /// The address renders inside the row.
    #[test]
    fn test_admin_wallet_table_row_renders_address() {
        fn render() -> Element {
            rsx! {
            AdminWalletTableRow {
                wallet: sample_row(),
                is_selected: false,
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
            html.contains("0x1234567890abcdef"),
            "Address should render in the row. Got: {html}"
        );
    }

    /// Label badge renders when the row has a label.
    #[test]
    fn test_admin_wallet_table_row_renders_label_badge_when_present() {
        fn render() -> Element {
            rsx! {
            AdminWalletTableRow {
                wallet: sample_row(),
                is_selected: false,
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
            html.contains("VIP"),
            "Label badge text should render. Got: {html}"
        );
    }

    /// No label → no badge slot.
    #[test]
    fn test_admin_wallet_table_row_hides_label_badge_when_absent() {
        fn render() -> Element {
            let mut row = sample_row();
            row.label = None;
            rsx! {
            AdminWalletTableRow {
                wallet: row,
                is_selected: false,
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
            !html.contains("VIP"),
            "Label badge should not render when label is None. Got: {html}"
        );
    }

    /// Status pill renders the right text class for active.
    #[test]
    fn test_admin_wallet_table_row_renders_status_active() {
        fn render() -> Element {
            rsx! {
            AdminWalletTableRow {
                wallet: sample_row(),
                is_selected: false,
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
            html.contains("text-success"),
            "Active row should use text-success. Got: {html}"
        );
        assert!(
            html.contains("active"),
            "Active row should render the active label. Got: {html}"
        );
    }

    /// Disabled row gets the opacity-70 class.
    #[test]
    fn test_admin_wallet_table_row_disabled_row_has_opacity() {
        fn render() -> Element {
            let mut row = sample_row();
            row.status = "disabled".to_string();
            rsx! {
            AdminWalletTableRow {
                wallet: row,
                is_selected: false,
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
            html.contains("opacity-70"),
            "Disabled row should have opacity-70. Got: {html}"
        );
        assert!(
            html.contains("text-warning"),
            "Disabled row should use text-warning. Got: {html}"
        );
    }

    /// Plan badge text renders.
    #[test]
    fn test_admin_wallet_table_row_renders_plan_badge() {
        fn render() -> Element {
            rsx! {
            AdminWalletTableRow {
                wallet: sample_row(),
                is_selected: false,
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
            html.contains("Premium"),
            "Plan badge should render the plan name. Got: {html}"
        );
        assert!(
            html.contains("uppercase"),
            "Plan badge should use uppercase styling. Got: {html}"
        );
    }

    /// Checkbox `checked` state matches `is_selected`.
    #[test]
    fn test_admin_wallet_table_row_checkbox_state() {
        fn render() -> Element {
            rsx! {
            AdminWalletTableRow {
                wallet: sample_row(),
                is_selected: true,
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
        let html_selected = dioxus_ssr::render(&vdom);
        assert!(
            html_selected.contains("checked"),
            "Selected row should have checked attribute. Got: {html_selected}"
        );
        assert!(
            html_selected.contains("bg-primary/5"),
            "Selected row should have selected bg. Got: {html_selected}"
        );
    }
}
