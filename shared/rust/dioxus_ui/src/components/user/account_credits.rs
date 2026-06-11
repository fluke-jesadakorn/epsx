//! Sub-components extracted from `pages/account_credits.rs` during
//! Wave 6C Track E (1:1 user-side component parity).
//!
//! Six named sub-components: `RenderAccountCredits`, `CreditBalance`,
//! `CreditTopUp`, `QuickAmount`, `CreditTransactionList`,
//! `CreditTransactionRow`.

use crate::primitives::*;
use crate::feedback::*;
use crate::pages::PageContext;

use dioxus::prelude::*;

/// Page-level orchestrator for the `/account/credits` route. Composes
/// the 3 ledger sub-components (balance row, top-up form, transaction
/// list) under the auth gate and main layout.
#[component]
pub fn RenderAccountCredits(ctx: PageContext) -> Element {
    rsx! {
        crate::layout::main_layout::MainLayout { ctx: ctx.clone(),
            crate::auth::AuthGate { user: ctx.user.clone(), feature: Some("your credits".to_string()),
                required_permissions: Some(vec!["profile:read".to_string()]),
                return_url: Some(ctx.path.clone()),
                div { class: "container page-content credits-ledger-page",
                    crate::layout::PageHeader {
                        title: "Credit Balance".to_string(),
                        description: Some("Manage your EPSX credits and view transaction history".to_string()),
                        icon: Some("coins".to_string()),
                    }
                    CreditBalance {}
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

/// 3 balance cards (Available, Lifetime Earned, Lifetime Spent).
/// Ported from the source's "Balance Cards" row.
#[component]
pub fn CreditBalance() -> Element {
    rsx! {
        div { class: "credits-balance-row grid grid-cols-1 md:grid-cols-3 gap-4",
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

/// Top-up form (amount + confirm). Static placeholder — no live payment
/// integration in the port.
#[component]
pub fn CreditTopUp() -> Element {
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
pub fn QuickAmount(value: String) -> Element {
    rsx! {
        button { class: "btn btn-outline btn-sm", r#type: "button", "${value}" }
    }
}

/// Paginated credit transaction history. Static placeholder.
#[component]
pub fn CreditTransactionList() -> Element {
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
                div { class: "px-6 pt-4 flex flex-wrap gap-2",
                    for (_id, label) in filters {
                        button {
                            class: "credits-filter-chip btn btn-sm btn-outline",
                            r#type: "button",
                            "{label}"
                        }
                    }
                }
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
pub fn CreditTransactionRow(
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

#[cfg(test)]
mod tests {
    use super::*;

    /// Wave 6C Track E — `test_render_smoke` for the extracted
    /// account_credits sub-components.
    #[test]
    fn account_credits_subcomponents_render_smoke() {
        // CreditBalance
        let el = rsx! { CreditBalance {} };
        let html = dioxus_ssr::render_element(el);
        assert!(html.contains("credits-balance-row"), "CreditBalance missing section-marker");
        assert!(html.contains("Available Balance"));
        assert!(html.contains("Lifetime Earned"));
        assert!(html.contains("Lifetime Spent"));

        // CreditTopUp
        let el = rsx! { CreditTopUp {} };
        let html = dioxus_ssr::render_element(el);
        assert!(html.contains("credits-topup"), "CreditTopUp missing section-marker");
        assert!(html.contains("Top up credits"));

        // QuickAmount
        let el = rsx! { QuickAmount { value: "25".to_string() } };
        let html = dioxus_ssr::render_element(el);
        assert!(html.contains("$25"));

        // CreditTransactionList
        let el = rsx! { CreditTransactionList {} };
        let html = dioxus_ssr::render_element(el);
        assert!(html.contains("credits-transaction-list"), "CreditTransactionList missing section-marker");
        assert!(html.contains("Transaction History"));
        assert!(html.contains("Trade reward"));

        // CreditTransactionRow (credit)
        let el = rsx! { CreditTransactionRow {
            date: "2024-09-20".to_string(),
            title: "Trade reward".to_string(),
            reason: "Bonus".to_string(),
            amount: 50.0,
            kind: "grant".to_string(),
        } };
        let html = dioxus_ssr::render_element(el);
        assert!(html.contains("credits-ledger-row--credit"));
        assert!(html.contains("+50"));

        // CreditTransactionRow (debit)
        let el = rsx! { CreditTransactionRow {
            date: "2024-09-19".to_string(),
            title: "API call".to_string(),
            reason: "Data".to_string(),
            amount: -1.0,
            kind: "payment_debit".to_string(),
        } };
        let html = dioxus_ssr::render_element(el);
        assert!(html.contains("credits-ledger-row--debit"));
    }
}
