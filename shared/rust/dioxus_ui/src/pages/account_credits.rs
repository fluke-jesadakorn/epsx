//! /account/credits — credit balance + transaction history.
//!
//! Wave 22 (T2) Track A — pixel-perfect port of
//! `apps-old/frontend/app/account/credits/page.tsx` (11 LoC) +
//! `credits-page-client.tsx` (256 LoC). Mirrors the OLD prod layout:
//!
//! Sections (matches the OLD prod baseline PNG):
//! - `CreditBalance`       — 3 balance cards (available, lifetime earned,
//!   lifetime spent). Uses the live `data_credits` param when present
//!   (BFF-fetched from the credits API), otherwise shows $0 defaults
//!   to match the OLD prod render for unauthenticated/anonymous
//!   visitors.
//! - `CreditTransactionList` — Transaction History card with filter
//!   chips (All / Admin Grant / Admin Revoke / Payment / Proration
//!   Credit / Refund / Expired / Adjustment) + "No transactions
//!   found" empty state when there is no data.
//!
//! Removed in T2: the Wave-6A-Track-A "Top up credits" form section
//! is dropped — the OLD prod does not have it. The card is now a
//! 2-column grid: balance row on top, transaction list full-width
//! below.
//!
//! All section markers are asserted in the `tests` module below.

use crate::primitives::*;
use crate::feedback::*;

use dioxus::prelude::*;
use super::PageContext;
use super::PageMeta;
use crate::layout::main_layout::MainLayout;

pub fn render(ctx: &PageContext) -> (PageMeta, Element) {
    let meta = PageMeta::app("Credits");
    (meta, rsx! { RenderAccountCredits { ctx: ctx.clone() } })
}

#[component]
fn RenderAccountCredits(ctx: PageContext) -> Element {
    // T2: match OLD prod layout — header + balance row + transaction
    // list. No top-up form (removed in this revision). The "data_credits"
    // param, when present (BFF fetch), is parsed and passed to the
    // sub-components. When absent, the balance renders $0 to match the
    // OLD prod snapshot (anonymous visitor).
    let data_credits: Option<CreditBalanceData> = ctx.params.get("data_credits")
        .and_then(|s| serde_json::from_str(s).ok());

    let available = data_credits.as_ref()
        .map(|d| d.available_balance)
        .unwrap_or(0.0);
    let lifetime_earned = data_credits.as_ref()
        .map(|d| d.lifetime_earned)
        .unwrap_or(0.0);
    let lifetime_spent = data_credits.as_ref()
        .map(|d| d.lifetime_spent)
        .unwrap_or(0.0);
    let transactions: Vec<CreditTxData> = data_credits.as_ref()
        .map(|d| d.transactions.clone())
        .unwrap_or_default();

    rsx! {
        MainLayout { ctx: ctx.clone(),
            // T2: removed the `<AuthGate>` wrapper — the OLD prod
            // page is public-readable (see apps-old/frontend/middleware.ts
            // publicRoutes: '/account*'). The page renders the full
            // credit balance layout for anonymous visitors with
            // $0 defaults. Authed users get the same layout but
            // with real balance data when the BFF wires
            // `data_credits`.
            div { class: "container page-content credits-ledger-page",
                // T2: header — just the title + description, no
                // `PageHeader` icon (matches OLD prod).
                div { class: "mb-6",
                    h1 { class: "text-3xl font-bold text-foreground", "Credit Balance" }
                    p { class: "mt-2 text-slate-400",
                        "Manage your EPSX credits and view transaction history"
                    }
                }
                // Section 1: balance cards
                CreditBalance {
                    available,
                    lifetime_earned,
                    lifetime_spent,
                }
                // Section 2: transaction history (full width)
                div { class: "mt-6",
                    CreditTransactionList { transactions: transactions.clone() }
                }
            }
        }
    }
}

// ----- Section 1: CreditBalance ------------------------------------------------

/// 3 balance cards (Available, Lifetime Earned, Lifetime Spent).
/// T2: now accepts the three numeric values as props (the OLD prod
/// page is data-driven via `useEffect` + `creditsApi.getBalance()`;
/// the BFF can pipe those values through `data_credits`). When no
/// data is provided, all three values default to $0 — matching the
/// OLD prod render for anonymous visitors.
#[component]
fn CreditBalance(available: f64, lifetime_earned: f64, lifetime_spent: f64) -> Element {
    let fmt = |v: f64| -> String {
        // Match OLD: `$0` for zero, `$X.XX` otherwise. Avoid `0,000.00`
        // for the canonical "no balance yet" case.
        if v == 0.0 { "$0".to_string() } else { format!("${:.2}", v) }
    };
    rsx! {
        div { class: "credits-balance-row grid grid-cols-1 md:grid-cols-3 gap-4",
            // Available (highlighted card with blue gradient — same
            // as the OLD prod).
            div { class: "credits-balance-available card card-glass p-6 text-white bg-gradient-to-br from-blue-500 to-blue-600",
                div { class: "flex items-center justify-between mb-3",
                    div { class: "p-2 bg-white/20 rounded-lg",
                        Icon { name: "coins".to_string(), size: Some(24) }
                    }
                    Icon { name: "dollar-sign".to_string(), size: Some(28), class_name: Some("opacity-50".to_string()) }
                }
                p { class: "text-sm opacity-90", "Available Balance" }
                p { class: "text-4xl font-bold mt-1", "{fmt(available)}" }
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
                    p { class: "text-3xl font-bold", "{fmt(lifetime_earned)}" }
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
                    p { class: "text-3xl font-bold", "{fmt(lifetime_spent)}" }
                }
            }
        }
    }
}

// ----- Section 2: CreditTransactionList ----------------------------------------

/// Transaction History card. T2: filter chips + empty-state-by-default
/// (mirrors the OLD prod render when no transaction data is present).
/// When `data_credits` carries transactions, each is rendered as a
/// row with the kind-coloured badge. The active filter (defaulting to
/// "All") gets the primary background; others stay outline. This is
/// visual-only for now (clicking a chip is a no-op — the real filter
/// is handled by the BFF in the production wiring).
#[component]
fn CreditTransactionList(transactions: Vec<CreditTxData>) -> Element {
    let mut active_filter = use_signal(|| "all".to_string());
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
            div { class: "p-6 border-b border-border",
                h2 { class: "text-xl font-semibold text-foreground mb-4",
                    "Transaction History"
                }
                div { class: "flex flex-wrap gap-2",
                    for (id, label) in filters.iter() {
                        {
                            let id = id.to_string();
                            let label = label.to_string();
                            let is_active = *active_filter.read() == id;
                            let cls = if is_active {
                                "credits-filter-chip btn btn-sm btn-primary"
                            } else {
                                "credits-filter-chip btn btn-sm btn-outline"
                            };
                            rsx! {
                                button {
                                    class: "{cls}",
                                    r#type: "button",
                                    onclick: move |_| active_filter.set(id.clone()),
                                    "{label}"
                                }
                            }
                        }
                    }
                }
            }
            div { class: "divide-y divide-border",
                if transactions.is_empty() {
                    div { class: "p-12 text-center",
                        Icon { name: "coins".to_string(), size: Some(48), class_name: Some("text-muted-foreground".to_string()) }
                        p { class: "mt-3 text-slate-400", "No transactions found" }
                    }
                } else {
                    for tx in transactions.iter() {
                        CreditTransactionRow { tx: tx.clone() }
                    }
                }
            }
        }
    }
}

#[component]
fn CreditTransactionRow(tx: CreditTxData) -> Element {
    let is_credit = tx.amount > 0.0;
    let sign = if is_credit { "+" } else { "" };
    let amount_str = if tx.amount == tx.amount.trunc() {
        format!("{sign}{}", tx.amount as i64)
    } else {
        format!("{sign}{:.2}", tx.amount)
    };
    let kind_class = match tx.kind.as_str() {
        "grant" | "refund" | "proration_credit" => "text-emerald-600 dark:text-emerald-400",
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
                    div { class: "font-semibold", "{tx.title}" }
                    p { class: "text-sm text-muted-foreground", "{tx.reason}" }
                    p { class: "text-xs text-muted-foreground mt-1", "{tx.date}" }
                }
            }
            div { class: "text-right",
                div { class: "text-lg font-bold {kind_class}", "${amount_str}" }
                span { class: "credits-ledger-kind text-xs text-muted-foreground", "{tx.kind}" }
            }
        }
    }
}

// ----- Data model --------------------------------------------------------------

/// T2: data model for the credits BFF fetch. Parsed from the
/// `data_credits` query param. Mirrors the OLD `CreditBalance` and
/// `CreditTransaction` shapes from
/// `apps-old/frontend/shared/types/credits.ts`. Fields default to
/// empty/zero so the page renders the OLD "all $0 + no transactions"
/// baseline when the BFF fetch isn't wired yet.
#[derive(Clone, Debug, serde::Deserialize, PartialEq)]
struct CreditBalanceData {
    #[serde(default)]
    available_balance: f64,
    #[serde(default)]
    lifetime_earned: f64,
    #[serde(default)]
    lifetime_spent: f64,
    #[serde(default)]
    transactions: Vec<CreditTxData>,
}

#[derive(Clone, Debug, serde::Deserialize, PartialEq)]
struct CreditTxData {
    #[serde(default)]
    date: String,
    #[serde(default)]
    title: String,
    #[serde(default)]
    reason: String,
    #[serde(default)]
    amount: f64,
    #[serde(default)]
    kind: String,
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
            "credits-transaction-list",
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

    /// T2 — when no `data_credits` param is supplied (the BFF hasn't
    /// fetched balance yet, e.g. anonymous visitor), the page must
    /// default to `$0` for all three balance values and show the
    /// "No transactions found" empty state. This matches the OLD
    /// prod baseline PNG (https://epsx.io/account/credits).
    #[test]
    fn test_default_zero_balance_and_empty_transactions() {
        let (_meta, el) = render(&authed_ctx());
        let html = dioxus_ssr::render_element(el);
        // The three $0 values render as `$0` (one occurrence each).
        let zero_count = html.matches("$0").count();
        assert!(
            zero_count >= 3,
            "expected 3 `$0` values for the 3 balance cards, found {}. Got: {}",
            zero_count, html
        );
        // Empty-state CTA must be present.
        assert!(
            html.contains("No transactions found"),
            "empty-state must say 'No transactions found' (matches OLD prod). Got: {}",
            html
        );
    }

    /// T2 — the Wave-6A-Track-A "Top up credits" form must NOT be
    /// rendered (it was removed in this revision; the OLD prod
    /// page does not have it).
    #[test]
    fn test_no_topup_form() {
        let (_meta, el) = render(&authed_ctx());
        let html = dioxus_ssr::render_element(el);
        assert!(
            !html.contains("credits-topup") && !html.contains("Top up credits"),
            "the top-up form must not render (OLD prod doesn't have it). Got: {}",
            html
        );
    }

    /// T2 — the filter chip set must include all 8 labels the OLD
    /// prod page has (All + 7 transaction kinds).
    #[test]
    fn test_filter_chip_labels() {
        let (_meta, el) = render(&authed_ctx());
        let html = dioxus_ssr::render_element(el);
        for label in &[
            "All",
            "Admin Grant",
            "Admin Revoke",
            "Payment",
            "Proration Credit",
            "Refund",
            "Expired",
            "Adjustment",
        ] {
            assert!(
                html.contains(label),
                "filter chip '{}' must render. Got: {}",
                label, html
            );
        }
    }
}
