//! /account/credits — credit balance + transaction history (the
//! "credit ledger" widget).
//!
//! Wave 6A Track A — port of
//! `apps-old/frontend/app/account/credits/page.tsx` (11 LoC) +
//! `credits-page-client.tsx` (256 LoC, the actual widget; the design
//! doc's "credits-management.tsx 431 LoC" reference is the equivalent
//! in their current branch — we port the active source as-is).
//!
//! Sections:
//! - `CreditBalance`       — 3 balance cards (available, earned, spent)
//! - `CreditTopUp`         — top-up form (amount + confirm button)
//! - `CreditTransactionList` — paginated transaction history
//!
//! All section markers are asserted in the `tests` module below.

use crate::primitives::*;
use crate::feedback::*;

use dioxus::prelude::*;
use super::PageContext;
use super::PageMeta;
use crate::layout::main_layout::MainLayout;
use crate::layout::PageHeader;
use crate::auth::AuthGate;

pub fn render(ctx: &PageContext) -> (PageMeta, Element) {
    let meta = PageMeta::app("Credits");
    (meta, rsx! { RenderAccountCredits { ctx: ctx.clone() } })
}

#[component]
fn RenderAccountCredits(ctx: PageContext) -> Element {
    // The "credit ledger" widget. The source uses 3 sub-components —
    // balance, transaction list, top-up form. We port them as 3
    // separate `#[component]` functions below, then orchestrate them
    // here in the same layout as the source.
    rsx! {
        MainLayout { ctx: ctx.clone(),
            AuthGate { user: ctx.user.clone(), feature: Some("your credits".to_string()),
                required_permissions: Some(vec!["profile:read".to_string()]),
                return_url: Some(ctx.path.clone()),
                div { class: "container page-content credits-ledger-page",
                    PageHeader {
                        title: "Credit Balance".to_string(),
                        description: Some("Manage your EPSX credits and view transaction history".to_string()),
                        icon: Some("coins".to_string()),
                    }
                    // Section 1: balance cards
                    CreditBalance {}
                    // Section 2 + 3: top-up form + transaction list, side by side
                    div { class: "grid grid-cols-1 lg:grid-cols-3 gap-4 mt-4",
                        div { class: "lg:col-span-1",
                            CreditTopUp {}
                        }
                        div { class: "lg:col-span-2",
                            CreditTransactionList {}
                        }
                    }
                }
            }
        }
    }
}

// ----- Section 1: CreditBalance ------------------------------------------------

/// 3 balance cards (Available, Lifetime Earned, Lifetime Spent).
/// Ported from the source's "Balance Cards" row.
#[component]
fn CreditBalance() -> Element {
    rsx! {
        div { class: "credits-balance-row grid grid-cols-1 md:grid-cols-3 gap-4",
            // Available (highlighted card with primary colour)
            div { class: "credits-balance-available card card-glass p-6 text-white bg-gradient-to-br from-blue-500 to-blue-600",
                div { class: "flex items-center justify-between mb-3",
                    div { class: "p-2 bg-white/20 rounded-lg",
                        Icon { name: "coins".to_string(), size: Some(24) }
                    }
                    Icon { name: "dollar-sign".to_string(), size: Some(28), class_name: Some("opacity-50".to_string()) }
                }
                p { class: "text-sm opacity-90", "Available Balance" }
                p { class: "text-4xl font-bold mt-1", "$1,250.00" }
            }
            // Lifetime earned
            div { class: "credits-balance-earned card card-glass",
                div { class: "card-body",
                    div { class: "flex items-center justify-between mb-2",
                        div { class: "p-2 bg-emerald-50 dark:bg-emerald-900/20 rounded-lg",
                            Icon { name: "trending-up".to_string(), size: Some(20), class_name: Some("text-emerald-600 dark:text-emerald-400".to_string()) }
                        }
                    }
                    p { class: "text-sm text-muted-foreground", "Lifetime Earned" }
                    p { class: "text-3xl font-bold", "$1,590.00" }
                }
            }
            // Lifetime spent
            div { class: "credits-balance-spent card card-glass",
                div { class: "card-body",
                    div { class: "flex items-center justify-between mb-2",
                        div { class: "p-2 bg-orange-50 dark:bg-orange-900/20 rounded-lg",
                            Icon { name: "trending-down".to_string(), size: Some(20), class_name: Some("text-orange-600 dark:text-orange-400".to_string()) }
                        }
                    }
                    p { class: "text-sm text-muted-foreground", "Lifetime Spent" }
                    p { class: "text-3xl font-bold", "$340.00" }
                }
            }
        }
    }
}

// ----- Section 2: CreditTopUp --------------------------------------------------

/// Top-up form (amount + confirm). Static placeholder — no live payment
/// integration in the port.
#[component]
fn CreditTopUp() -> Element {
    rsx! {
        div { class: "credits-topup card card-glass",
            div { class: "card-header", h3 { class: "card-title", "Top up credits" } }
            div { class: "card-body",
                p { class: "text-sm text-muted-foreground mb-3",
                    "Purchase additional credits to use across the platform."
                }
                div { class: "form-row",
                    label { class: "form-label", "Amount (USD)" }
                    input { class: "input", r#type: "number", min: "5", step: "1", value: "50" }
                }
                div { class: "grid grid-cols-3 gap-2 mt-3",
                    QuickAmount { value: "10" }
                    QuickAmount { value: "50" }
                    QuickAmount { value: "100" }
                }
                button { class: "btn btn-primary btn-block mt-4", r#type: "button",
                    Icon { name: "plus".to_string(), size: Some(16) }
                    " Add credits"
                }
                p { class: "text-xs text-muted-foreground mt-2 text-center",
                    "Charged to your default payment method"
                }
            }
        }
    }
}

#[component]
fn QuickAmount(value: String) -> Element {
    rsx! {
        button { class: "btn btn-outline btn-sm", r#type: "button", "${value}" }
    }
}

// ----- Section 3: CreditTransactionList ----------------------------------------

/// Paginated credit transaction history. Static placeholder (no live
/// data) but the structure mirrors the source: filter chips at the top,
/// a row per transaction with icon + amount + reason, an empty state
/// when there are no rows.
#[component]
fn CreditTransactionList() -> Element {
    let transactions = vec![
        ("2024-09-20", "Trade reward",        "Subscription bonus payout", 50.0,  "grant"),
        ("2024-09-19", "API call",            "Market data API",          -1.0,  "payment_debit"),
        ("2024-09-18", "Premium feature",     "Analytics export",       -100.0,  "payment_debit"),
        ("2024-09-15", "Subscription bonus",  "Pro plan renewal",        500.0,  "grant"),
    ];
    let filters = vec![
        ("all",              "All"),
        ("grant",            "Admin Grant"),
        ("revoke",           "Admin Revoke"),
        ("payment_debit",    "Payment"),
        ("proration_credit", "Proration Credit"),
        ("refund",           "Refund"),
        ("expiry",           "Expired"),
        ("adjustment",       "Adjustment"),
    ];
    rsx! {
        div { class: "credits-transaction-list card card-glass",
            div { class: "card-header",
                h3 { class: "card-title", "Transaction History" }
            }
            div { class: "card-body p-0",
                // Filter chips
                div { class: "px-6 pt-4 flex flex-wrap gap-2",
                    for (_id, label) in filters {
                        button {
                            class: "credits-filter-chip btn btn-sm btn-outline",
                            r#type: "button",
                            "{label}"
                        }
                    }
                }
                // Rows
                div { class: "divide-y divide-border mt-4",
                    if transactions.is_empty() {
                        div { class: "p-8 text-center",
                            Icon { name: "coins".to_string(), size: Some(40), class_name: Some("text-muted-foreground".to_string()) }
                            p { class: "text-sm text-muted-foreground mt-2", "No transactions found" }
                        }
                    } else {
                        for (date, title, reason, amount, kind) in transactions {
                            CreditTransactionRow {
                                date: date.to_string(),
                                title: title.to_string(),
                                reason: reason.to_string(),
                                amount: amount,
                                kind: kind.to_string(),
                            }
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn CreditTransactionRow(
    date: String,
    title: String,
    reason: String,
    amount: f64,
    kind: String,
) -> Element {
    let is_credit = amount > 0.0;
    let amount_text = format!("{}{:.0}", if is_credit { "+" } else { "" }, amount);
    let kind_class = match kind.as_str() {
        "grant" => "text-emerald-600 dark:text-emerald-400",
        "refund" | "proration_credit" => "text-emerald-600 dark:text-emerald-400",
        "payment_debit" => "text-orange-600 dark:text-orange-400",
        "revoke" | "expiry" => "text-red-600 dark:text-red-400",
        _ => "text-purple-600 dark:text-purple-400",
    };
    let row_class = if is_credit { "credits-ledger-row credits-ledger-row--credit" } else { "credits-ledger-row credits-ledger-row--debit" };
    rsx! {
        div { class: "{row_class} p-4 flex items-start justify-between",
            div { class: "flex items-start gap-3 flex-1",
                div { class: "p-2 rounded-lg bg-primary/10",
                    Icon {
                        name: (if is_credit { "arrow-down-right" } else { "arrow-up-right" }).to_string(),
                        size: Some(20),
                        class_name: Some(kind_class.to_string()),
                    }
                }
                div { class: "flex-1",
                    div { class: "font-semibold", "{title}" }
                    p { class: "text-sm text-muted-foreground", "{reason}" }
                    p { class: "text-xs text-muted-foreground mt-1", "{date}" }
                }
            }
            div { class: "text-right",
                div { class: "text-lg font-bold {kind_class}", "${amount_text}" }
                span { class: "credits-ledger-kind text-xs text-muted-foreground", "{kind}" }
            }
        }
    }
}

// =============================================================================
// Tests
// =============================================================================
//
// - `test_render_smoke` — render(&empty_ctx()) returns non-empty Element.
// - `test_section_markers` — SSR'd HTML contains every section-marker
//   class the design doc claims.

#[cfg(test)]
mod tests {
    use super::*;
    use crate::auth::user::User;
    use crate::auth::user::AuthMethod;

    fn empty_ctx() -> PageContext {
        PageContext {
            user: None,
            path: "/account/credits".to_string(),
            ..Default::default()
        }
    }

    fn authed_ctx() -> PageContext {
        PageContext {
            user: Some(User {
                id: "u-1".to_string(),
                address: "0x1234abcd".to_string(),
                chain_id: "1".to_string(),
                roles: vec!["user".to_string()],
                email: Some("test@epsx.io".to_string()),
                tier: Some("pro".to_string()),
                permissions: vec!["profile:read".to_string()],
                last_login_at: None,
                auth_method: AuthMethod::default(),
                display_name: Some("EPSX tester".to_string()),
            }),
            path: "/account/credits".to_string(),
            ..Default::default()
        }
    }

    #[test]
    fn test_render_smoke() {
        let (_meta, el) = render(&authed_ctx());
        let html = dioxus_ssr::render_element(el);
        assert!(!html.is_empty(), "account/credits must render non-empty HTML. Got: {}", html);
        assert!(html.len() > 100, "account/credits HTML is suspiciously short ({} bytes).", html.len());
    }

    #[test]
    fn test_section_markers() {
        let (_meta, el) = render(&authed_ctx());
        let html = dioxus_ssr::render_element(el);
        for marker in &[
            "credits-ledger-page",
            "credits-balance-row",
            "credits-balance-available",
            "credits-balance-earned",
            "credits-balance-spent",
            "credits-topup",
            "credits-transaction-list",
            "credits-ledger-row",
        ] {
            // The marker may be a single class on its div, the first
            // of multiple classes, or in the middle of a class list —
            // accept any of those forms.
            let needle_a = format!("class=\"{}\"", marker);
            let needle_b = format!("class=\"{mark} ", mark = marker);
            let needle_c = format!(" {}\"", marker);
            let needle_d = format!(" {} ", marker);
            assert!(
                html.contains(&needle_a)
                    || html.contains(&needle_b)
                    || html.contains(&needle_c)
                    || html.contains(&needle_d),
                "account/credits must contain section marker '{}'. Got: {}",
                marker, html
            );
        }
    }
}
