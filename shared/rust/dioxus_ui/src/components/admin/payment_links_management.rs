//! Admin `PaymentLinksManagement` family — Wave 38b T2 admin domain port.
//!
//! Mirrors
//! `apps-old/admin-frontend/components/payments/payment-links-management.tsx`,
//! which exports the table/mobile card view of payment links +
//! a master `PaymentLinksManagement` orchestrator.
//!
//! | Component | Use case |
//! | --- | --- |
//! | `PaymentLinksTable` | Desktop table of links (slug / context / amount / uses / status / actions) |
//! | `PaymentLinksMobileCards` | Mobile card list |
//! | `PaymentLinksHeaderRow` | Header row with title + count badge |
//!
//! ## Tests
//!
//! Each component gets a colocated `#[cfg(test)] mod tests` block
//! with smoke render + key prop handling.

use dioxus::prelude::*;
use crate::primitives::icon::Icon;

// ============================================================================
// Data shape
// ============================================================================

#[derive(Clone, Debug, PartialEq)]
pub struct PaymentLink {
    pub id: String,
    pub slug: String,
    pub context_type: String,
    pub context_id: Option<String>,
    pub name: String,
    pub amount: f64,
    pub currency: String,
    pub is_active: bool,
    pub is_usable: bool,
    pub uses: u32,
    pub max_uses: Option<u32>,
    pub created_at: String,
    pub expires_at: Option<String>,
}

// ============================================================================
// Helpers
// ============================================================================

fn format_pl_date(date_str: String) -> String {
    date_str
}

fn is_pl_expired(expires_at: Option<String>) -> bool {
    match expires_at {
        Some(s) => s.is_empty(),
        None => false,
    }
}

// ============================================================================
// PaymentLinksHeaderRow
// ============================================================================
//
// Title + count badge at the top of the links section.

#[component]
pub fn PaymentLinksHeaderRow(count: u32) -> Element {
    rsx! {
        div { class: "payment-links-header-row flex flex-col sm:flex-row items-start sm:items-center justify-between mb-6 gap-3",
            h2 { class: "text-xs font-bold text-[#7645d9] uppercase tracking-[0.2em]", "All Payment Links" }
            span { class: "px-3 py-1 bg-muted/50 rounded-full border border-border/40 text-xs font-bold text-muted-foreground",
                "{count} links"
            }
        }
    }
}

// ============================================================================
// PaymentLinksTable
// ============================================================================
//
// Desktop table. Mirrors the source's `PaymentLinksTable` (slug,
// context, amount, uses, status, actions).

#[component]
pub fn PaymentLinksTable(links: Vec<PaymentLink>) -> Element {
    rsx! {
        div { class: "payment-links-table hidden sm:block overflow-x-auto",
            table { class: "min-w-full",
                thead {
                    tr { class: "border-b border-border/50",
                        th { class: "px-4 py-3 text-left text-xs font-semibold text-muted-foreground uppercase tracking-wider", "Slug" }
                        th { class: "px-4 py-3 text-left text-xs font-semibold text-muted-foreground uppercase tracking-wider", "Context" }
                        th { class: "px-4 py-3 text-left text-xs font-semibold text-muted-foreground uppercase tracking-wider", "Amount" }
                        th { class: "px-4 py-3 text-left text-xs font-semibold text-muted-foreground uppercase tracking-wider", "Uses" }
                        th { class: "px-4 py-3 text-left text-xs font-semibold text-muted-foreground uppercase tracking-wider", "Status" }
                        th { class: "px-4 py-3 text-left text-xs font-semibold text-muted-foreground uppercase tracking-wider", "Actions" }
                    }
                }
                tbody { class: "divide-y divide-border/50",
                    for link in links.iter() {
                        {
                            let is_active = link.is_active;
                            let status_cls = if is_active {
                                "bg-success/10 text-success border border-success/20"
                            } else {
                                "bg-muted text-muted-foreground border border-border/50"
                            };
                            let status_text = if is_active { "Active" } else { "Inactive" };
                            let max_uses_display = match link.max_uses { Some(m) => format!("{} / {}", link.uses, m), None => format!("{}", link.uses) };
                            let context_display = match &link.context_id {
                                Some(cid) => format!("{} ({})", link.context_type, cid),
                                None => link.context_type.clone(),
                            };
                            rsx! {
                                tr { key: "{link.id}", class: "payment-links-table-row hover:bg-muted/30 transition-colors",
                                    td { class: "px-4 py-4 whitespace-nowrap",
                                        div { class: "font-mono text-xs text-foreground", "{link.slug}" }
                                        div { class: "text-xs text-muted-foreground mt-0.5", "{link.name}" }
                                    }
                                    td { class: "px-4 py-4 whitespace-nowrap text-sm text-muted-foreground", "{context_display}" }
                                    td { class: "px-4 py-4 whitespace-nowrap text-sm font-bold text-[#1fc7d4]",
                                        {
                                            let amt = format!("{:.2}", link.amount);
                                            rsx! { "{amt} {link.currency}" }
                                        }
                                    }
                                    td { class: "px-4 py-4 whitespace-nowrap text-sm text-foreground", "{max_uses_display}" }
                                    td { class: "px-4 py-4 whitespace-nowrap",
                                        span { class: "inline-flex items-center px-2.5 py-1 rounded-full text-xs font-semibold {status_cls}", "{status_text}" }
                                    }
                                    td { class: "px-4 py-4 whitespace-nowrap",
                                        button {
                                            class: "p-1.5 rounded-lg hover:bg-red-500/10 text-muted-foreground hover:text-red-400 transition-colors",
                                            r#type: "button",
                                            title: "Revoke",
                                            Icon { name: "trash-2".to_string(), size: Some(16) }
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
}

// ============================================================================
// PaymentLinksMobileCards
// ============================================================================
//
// Mobile card list. Mirrors the source's `PaymentLinksMobileCards`.

#[component]
pub fn PaymentLinksMobileCards(links: Vec<PaymentLink>) -> Element {
    rsx! {
        div { class: "payment-links-mobile-cards block sm:hidden space-y-3",
            for link in links.iter() {
                {
                    let is_active = link.is_active;
                    let status_cls = if is_active {
                        "bg-success/10 text-success border border-success/20"
                    } else {
                        "bg-muted text-muted-foreground border border-border/50"
                    };
                    let status_text = if is_active { "Active" } else { "Inactive" };
                    rsx! {
                        div { key: "{link.id}", class: "p-4 bg-muted/30 border border-border/50 rounded-2xl",
                            div { class: "flex items-center justify-between mb-3",
                                div { class: "font-mono text-xs text-foreground truncate", "{link.slug}" }
                                span { class: "inline-flex items-center px-2.5 py-1 rounded-full text-xs font-semibold {status_cls}", "{status_text}" }
                            }
                            div { class: "text-sm font-medium text-foreground mb-1", "{link.name}" }
                            div { class: "grid grid-cols-2 gap-3",
                                div { class: "bg-card rounded-xl p-3 border border-border/50",
                                    div { class: "text-sm font-medium text-muted-foreground", "Amount" }
                                    div { class: "text-lg font-bold text-primary",
                                        {
                                            let amt = format!("{:.2}", link.amount);
                                            rsx! { "{amt} {link.currency}" }
                                        }
                                    }
                                }
                                div { class: "bg-card rounded-xl p-3 border border-border/50",
                                    div { class: "text-sm font-medium text-muted-foreground", "Uses" }
                                    div { class: "text-lg font-bold text-secondary",
                                        "{link.uses}"
                                    }
                                }
                            }
                            div { class: "mt-3 text-xs text-muted-foreground",
                                "{format_pl_date(link.created_at.clone())}"
                            }
                        }
                    }
                }
            }
        }
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_link() -> PaymentLink {
        PaymentLink {
            id: "pl_1".to_string(),
            slug: "pro-plan-monthly".to_string(),
            context_type: "plan".to_string(),
            context_id: Some("plan-pro".to_string()),
            name: "Pro Plan Monthly".to_string(),
            amount: 29.0,
            currency: "USDT".to_string(),
            is_active: true,
            is_usable: true,
            uses: 12,
            max_uses: Some(100),
            created_at: "2024-09-20 10:32".to_string(),
            expires_at: None,
        }
    }

    /// `PaymentLinksHeaderRow` renders title + count badge.
    #[test]
    fn payment_links_header_row_renders_title_and_count() {
        fn harness() -> Element {
            rsx! { PaymentLinksHeaderRow { count: 5u32 } }
        }
        let mut vdom = dioxus::prelude::VirtualDom::new(harness);
        vdom.rebuild_in_place();
        let html = dioxus_ssr::render(&vdom);
        assert!(html.contains("payment-links-header-row"), "PaymentLinksHeaderRow must render container class. Got: {html}");
        assert!(html.contains("All Payment Links"), "PaymentLinksHeaderRow must render title. Got: {html}");
        assert!(html.contains("5 links"), "PaymentLinksHeaderRow must render count. Got: {html}");
    }

    /// `PaymentLinksTable` renders the 6 column headers + a row.
    #[test]
    fn payment_links_table_renders_headers_and_row() {
        fn harness() -> Element {
            rsx! { PaymentLinksTable { links: vec![sample_link()] } }
        }
        let mut vdom = dioxus::prelude::VirtualDom::new(harness);
        vdom.rebuild_in_place();
        let html = dioxus_ssr::render(&vdom);
        assert!(html.contains("payment-links-table"), "PaymentLinksTable must render container class. Got: {html}");
        for col in &["Slug", "Context", "Amount", "Uses", "Status", "Actions"] {
            assert!(html.contains(col), "PaymentLinksTable must render `{col}` header. Got: {html}");
        }
        assert!(html.contains("pro-plan-monthly"), "PaymentLinksTable must render slug. Got: {html}");
        assert!(html.contains("Pro Plan Monthly"), "PaymentLinksTable must render name. Got: {html}");
        assert!(html.contains("29.00 USDT"), "PaymentLinksTable must render formatted amount. Got: {html}");
        assert!(html.contains("12 / 100"), "PaymentLinksTable must render uses / max_uses. Got: {html}");
        assert!(html.contains("Active"), "PaymentLinksTable must render Active status. Got: {html}");
        assert!(html.contains("bg-success/10"), "PaymentLinksTable active must use success class. Got: {html}");
    }

    /// `PaymentLinksTable` with inactive link shows Inactive.
    #[test]
    fn payment_links_table_renders_inactive_status() {
        fn harness() -> Element {
            let mut link = sample_link();
            link.is_active = false;
            rsx! { PaymentLinksTable { links: vec![link] } }
        }
        let mut vdom = dioxus::prelude::VirtualDom::new(harness);
        vdom.rebuild_in_place();
        let html = dioxus_ssr::render(&vdom);
        assert!(html.contains("Inactive"), "PaymentLinksTable inactive must show Inactive. Got: {html}");
        assert!(html.contains("bg-muted"), "PaymentLinksTable inactive must use muted class. Got: {html}");
    }

    /// `PaymentLinksTable` with unlimited max_uses shows just the count.
    #[test]
    fn payment_links_table_renders_unlimited_max_uses() {
        fn harness() -> Element {
            let mut link = sample_link();
            link.max_uses = None;
            rsx! { PaymentLinksTable { links: vec![link] } }
        }
        let mut vdom = dioxus::prelude::VirtualDom::new(harness);
        vdom.rebuild_in_place();
        let html = dioxus_ssr::render(&vdom);
        assert!(html.contains("12</td>") || html.contains(">12<"), "PaymentLinksTable must render uses count alone when max_uses is None. Got: {html}");
    }

    /// `PaymentLinksTable` with empty links renders headers but no rows.
    #[test]
    fn payment_links_table_empty_renders_no_rows() {
        fn harness() -> Element {
            rsx! { PaymentLinksTable { links: vec![] } }
        }
        let mut vdom = dioxus::prelude::VirtualDom::new(harness);
        vdom.rebuild_in_place();
        let html = dioxus_ssr::render(&vdom);
        assert!(html.contains("payment-links-table"), "PaymentLinksTable empty must render container. Got: {html}");
        assert!(!html.contains("payment-links-table-row"), "PaymentLinksTable empty must omit rows. Got: {html}");
    }

    /// `PaymentLinksMobileCards` renders the mobile card layout.
    #[test]
    fn payment_links_mobile_cards_renders_layout() {
        fn harness() -> Element {
            rsx! { PaymentLinksMobileCards { links: vec![sample_link()] } }
        }
        let mut vdom = dioxus::prelude::VirtualDom::new(harness);
        vdom.rebuild_in_place();
        let html = dioxus_ssr::render(&vdom);
        assert!(html.contains("payment-links-mobile-cards"), "PaymentLinksMobileCards must render container class. Got: {html}");
        assert!(html.contains("Amount"), "PaymentLinksMobileCards must render Amount label. Got: {html}");
        assert!(html.contains("Uses"), "PaymentLinksMobileCards must render Uses label. Got: {html}");
        assert!(html.contains("29.00 USDT"), "PaymentLinksMobileCards must render amount. Got: {html}");
    }

    /// `PaymentLinksMobileCards` with empty links renders empty body.
    #[test]
    fn payment_links_mobile_cards_empty_renders_nothing() {
        fn harness() -> Element {
            rsx! { PaymentLinksMobileCards { links: vec![] } }
        }
        let mut vdom = dioxus::prelude::VirtualDom::new(harness);
        vdom.rebuild_in_place();
        let html = dioxus_ssr::render(&vdom);
        assert!(html.contains("payment-links-mobile-cards"), "PaymentLinksMobileCards empty must render container. Got: {html}");
    }
}
