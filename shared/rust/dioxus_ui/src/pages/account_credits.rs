//! `/account/credits` — owner-scoped credit balance and ledger.
//!
//! The frontend BFF injects independently validated balance and history
//! projections from the authenticated backend routes. Missing or malformed
//! data stays unavailable and is never converted into a zero balance or an
//! empty ledger.

use chrono::DateTime;
use dioxus::prelude::*;
use serde_json::Value;

use super::{PageContext, PageMeta};
use crate::enterprise::FrontendIcon;
use crate::layout::main_layout::MainLayout;
use crate::primitives::*;

const ACCOUNT_PATH: &str = "/account";
const CREDITS_SIGN_IN_PATH: &str = "/auth?return_url=%2Faccount%2Fcredits";
pub const ACCOUNT_CREDIT_BALANCE_DATA_PARAM: &str = "data_account_credit_balance";
pub const ACCOUNT_CREDIT_BALANCE_STATE_PARAM: &str = "data_account_credit_balance_state";
pub const ACCOUNT_CREDIT_HISTORY_DATA_PARAM: &str = "data_account_credit_history";
pub const ACCOUNT_CREDIT_HISTORY_STATE_PARAM: &str = "data_account_credit_history_state";
pub const ACCOUNT_CREDIT_READY: &str = "ready";
pub const ACCOUNT_CREDIT_EMPTY: &str = "empty";
pub const ACCOUNT_CREDIT_UNAVAILABLE: &str = "unavailable";
pub const ACCOUNT_CREDIT_MALFORMED: &str = "malformed";
pub const ACCOUNT_CREDIT_HISTORY_MAX_ITEMS: usize = 20;

const MAX_WALLET_LEN: usize = 128;
const MAX_VALUE_LEN: usize = 128;
const MAX_TEXT_LEN: usize = 512;

#[derive(Clone, Debug, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
pub struct CreditBalanceProjection {
    pub wallet_address: String,
    pub balance: String,
    pub pending_balance: String,
    pub available_balance: String,
    pub lifetime_earned: String,
    pub lifetime_spent: String,
    pub last_transaction_at: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
pub struct CreditTransactionProjection {
    pub id: String,
    pub amount: String,
    pub balance_after: String,
    pub tx_type: String,
    pub reference_type: Option<String>,
    pub reason: Option<String>,
    pub expires_at: Option<String>,
    pub created_at: String,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
pub struct CreditHistoryProjection {
    pub transactions: Vec<CreditTransactionProjection>,
    pub count: usize,
}

#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct CreditHistoryWire {
    success: bool,
    data: Vec<CreditTransactionWire>,
    count: usize,
}

#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct CreditTransactionWire {
    id: String,
    wallet_address: String,
    #[serde(deserialize_with = "credit_decimal_text")]
    amount: String,
    #[serde(deserialize_with = "credit_decimal_text")]
    balance_after: String,
    tx_type: String,
    reference_id: Option<String>,
    reference_type: Option<String>,
    reason: Option<String>,
    granted_by: Option<String>,
    expires_at: Option<String>,
    created_at: String,
}

fn credit_decimal_text<'de, D: serde::Deserializer<'de>>(
    deserializer: D,
) -> Result<String, D::Error> {
    use serde::Deserialize;
    match Value::deserialize(deserializer)? {
        Value::String(value) => Ok(value),
        Value::Number(value) => Ok(value.to_string()),
        _ => Err(serde::de::Error::custom(
            "credit amount must be a decimal string or number",
        )),
    }
}

pub fn decode_credit_balance(
    value: Value,
    expected_owner: &str,
) -> Option<CreditBalanceProjection> {
    let balance: CreditBalanceProjection = serde_json::from_value(value).ok()?;
    if !valid_owner(&balance.wallet_address, expected_owner)
        || !valid_decimal(&balance.balance)
        || !valid_decimal(&balance.pending_balance)
        || !valid_decimal(&balance.available_balance)
        || !valid_decimal(&balance.lifetime_earned)
        || !valid_decimal(&balance.lifetime_spent)
        || !balance
            .last_transaction_at
            .as_deref()
            .is_none_or(valid_timestamp)
    {
        return None;
    }
    Some(balance)
}

pub fn decode_credit_history(
    value: Value,
    expected_owner: &str,
    max_items: usize,
) -> Option<CreditHistoryProjection> {
    if max_items == 0 || expected_owner.is_empty() {
        return None;
    }
    let history: CreditHistoryWire = serde_json::from_value(value).ok()?;
    if !history.success
        || history.data.len() > max_items
        || history.count != history.data.len()
        || !history.data.iter().all(|row| {
            valid_owner(&row.wallet_address, expected_owner)
                && valid_text(&row.id, MAX_VALUE_LEN)
                && valid_decimal(&row.amount.to_string())
                && valid_decimal(&row.balance_after.to_string())
                && valid_text(&row.tx_type, MAX_VALUE_LEN)
                && row
                    .reference_id
                    .as_deref()
                    .is_none_or(|value| valid_text(value, MAX_VALUE_LEN))
                && row
                    .reference_type
                    .as_deref()
                    .is_none_or(|value| valid_text(value, MAX_VALUE_LEN))
                && row
                    .reason
                    .as_deref()
                    .is_none_or(|value| valid_text(value, MAX_TEXT_LEN))
                && row
                    .granted_by
                    .as_deref()
                    .is_none_or(|value| valid_text(value, MAX_WALLET_LEN))
                && row.expires_at.as_deref().is_none_or(valid_timestamp)
                && valid_timestamp(&row.created_at)
        })
    {
        return None;
    }

    Some(CreditHistoryProjection {
        count: history.count,
        transactions: history
            .data
            .into_iter()
            .map(|row| CreditTransactionProjection {
                id: row.id,
                amount: row.amount.to_string(),
                balance_after: row.balance_after.to_string(),
                tx_type: row.tx_type,
                reference_type: row.reference_type,
                reason: row.reason,
                expires_at: row.expires_at,
                created_at: row.created_at,
            })
            .collect(),
    })
}

fn valid_owner(value: &str, expected_owner: &str) -> bool {
    valid_text(value, MAX_WALLET_LEN) && value.eq_ignore_ascii_case(expected_owner)
}

fn valid_decimal(value: &str) -> bool {
    valid_text(value, MAX_VALUE_LEN) && value.parse::<f64>().is_ok_and(f64::is_finite)
}

fn valid_text(value: &str, max_len: usize) -> bool {
    !value.is_empty()
        && value.trim() == value
        && value.len() <= max_len
        && !value.chars().any(char::is_control)
}

fn valid_timestamp(value: &str) -> bool {
    valid_text(value, MAX_VALUE_LEN) && DateTime::parse_from_rfc3339(value).is_ok()
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CreditBalanceLoad {
    SignedOut,
    Ready(CreditBalanceProjection),
    Unavailable,
    Malformed,
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum CreditHistoryLoad {
    SignedOut,
    Ready(CreditHistoryProjection),
    Empty,
    Unavailable,
    Malformed,
}

pub fn credit_balance_load(ctx: &PageContext) -> CreditBalanceLoad {
    let Some(user) = ctx.user.as_ref() else {
        return CreditBalanceLoad::SignedOut;
    };
    match ctx
        .params
        .get(ACCOUNT_CREDIT_BALANCE_STATE_PARAM)
        .map(String::as_str)
    {
        Some(ACCOUNT_CREDIT_READY) => ctx
            .params
            .get(ACCOUNT_CREDIT_BALANCE_DATA_PARAM)
            .and_then(|raw| serde_json::from_str(raw).ok())
            .and_then(|value| decode_credit_balance(value, &user.address))
            .map(CreditBalanceLoad::Ready)
            .unwrap_or(CreditBalanceLoad::Malformed),
        Some(ACCOUNT_CREDIT_MALFORMED) => CreditBalanceLoad::Malformed,
        Some(ACCOUNT_CREDIT_UNAVAILABLE) | None => CreditBalanceLoad::Unavailable,
        Some(_) => CreditBalanceLoad::Malformed,
    }
}

fn credit_history_load(ctx: &PageContext) -> CreditHistoryLoad {
    let Some(user) = ctx.user.as_ref() else {
        return CreditHistoryLoad::SignedOut;
    };
    match ctx
        .params
        .get(ACCOUNT_CREDIT_HISTORY_STATE_PARAM)
        .map(String::as_str)
    {
        Some(ACCOUNT_CREDIT_READY) | Some(ACCOUNT_CREDIT_EMPTY) => {
            let expected_empty = ctx
                .params
                .get(ACCOUNT_CREDIT_HISTORY_STATE_PARAM)
                .is_some_and(|state| state == ACCOUNT_CREDIT_EMPTY);
            let history = ctx
                .params
                .get(ACCOUNT_CREDIT_HISTORY_DATA_PARAM)
                .and_then(|raw| serde_json::from_str(raw).ok())
                .and_then(|value| {
                    decode_credit_history(value, &user.address, ACCOUNT_CREDIT_HISTORY_MAX_ITEMS)
                });
            match history {
                Some(history) if expected_empty == history.transactions.is_empty() => {
                    if expected_empty {
                        CreditHistoryLoad::Empty
                    } else {
                        CreditHistoryLoad::Ready(history)
                    }
                }
                _ => CreditHistoryLoad::Malformed,
            }
        }
        Some(ACCOUNT_CREDIT_MALFORMED) => CreditHistoryLoad::Malformed,
        Some(ACCOUNT_CREDIT_UNAVAILABLE) | None => CreditHistoryLoad::Unavailable,
        Some(_) => CreditHistoryLoad::Malformed,
    }
}

pub fn render(ctx: &PageContext) -> (PageMeta, Element) {
    let meta = PageMeta::app("Credits");
    (meta, rsx! { RenderAccountCredits { ctx: ctx.clone() } })
}

#[component]
fn RenderAccountCredits(ctx: PageContext) -> Element {
    let owner = ctx.user.as_ref().map(|user| user.address.clone());
    let balance = credit_balance_load(&ctx);
    let history = credit_history_load(&ctx);

    rsx! {
        MainLayout { ctx: ctx.clone(),
            // The source credits route uses a centered max-w-6xl frame with
            // a 1.5rem inset and top breathing room below the header. Keep
            // that geometry even when the owner-scoped data is unavailable.
            div { class: "page-content credits-ledger-page mx-auto max-w-6xl px-6 pt-6 fe-page-layout",
                div { class: "mb-6",
                    h1 { class: "text-3xl font-bold text-foreground fe-tone-text fe-type-title", "Credit Balance" }
                    p { class: "mt-2 text-slate-400 fe-tone-muted",
                        "Manage your EPSX credits and view transaction history"
                    }
                }

                if owner.is_some() {
                    CreditsOwnerView { balance, history }
                } else {
                    CreditsSignedOut {}
                }
            }
        }
    }
}

#[component]
fn CreditsSignedOut() -> Element {
    rsx! {
        section {
            class: "credits-access-state card card-glass overflow-hidden fe-surface",
            "data-credits-state": "signed-out",
            aria_labelledby: "credits-signed-out-title",
            role: "status",
            div { class: "p-10 sm:p-12 text-center",
                div { class: "mx-auto flex h-14 w-14 items-center justify-center rounded-2xl bg-primary/10 text-primary",
                    Icon { name: "coins".to_string(), size: Some(28) }
                }
                p { class: "mt-5 text-xs font-semibold uppercase tracking-widest text-primary",
                    "Private account data"
                }
                h2 {
                    id: "credits-signed-out-title",
                    class: "mt-2 text-2xl font-semibold text-foreground fe-tone-text",
                    "Sign in to view your credits"
                }
                p { class: "mx-auto mt-3 max-w-xl text-sm leading-6 text-muted-foreground fe-tone-muted",
                    "Credit balances and ledger activity are private to the wallet that owns them."
                }
                crate::fullstack::shell::ShellLink {
                    class: "btn btn-primary mt-6",
                    href: CREDITS_SIGN_IN_PATH,
                    "Sign in"
                }
            }
        }
    }
}

#[component]
fn CreditsOwnerView(balance: CreditBalanceLoad, history: CreditHistoryLoad) -> Element {
    let state = match (&balance, &history) {
        (CreditBalanceLoad::Ready(_), CreditHistoryLoad::Ready(_) | CreditHistoryLoad::Empty) => {
            "ready"
        }
        (CreditBalanceLoad::Malformed, _) | (_, CreditHistoryLoad::Malformed) => "malformed",
        _ => "unavailable",
    };
    rsx! {
        section {
            class: "credits-owner-content",
            "data-credits-state": state,
            aria_labelledby: "credits-transaction-title",

            div { class: "credits-balance-row grid grid-cols-1 gap-4 md:grid-cols-3",
                match balance {
                    CreditBalanceLoad::Ready(balance) => rsx! {
                        CreditBalanceCard { marker: "credits-balance-available", icon: "coins", label: "Available Balance", value: balance.available_balance, highlighted: true }
                        CreditBalanceCard { marker: "credits-balance-earned", icon: "trending-up", label: "Lifetime Earned", value: balance.lifetime_earned, highlighted: false }
                        CreditBalanceCard { marker: "credits-balance-spent", icon: "trending-down", label: "Lifetime Spent", value: balance.lifetime_spent, highlighted: false }
                    },
                    _ => rsx! {
                        UnavailableBalanceCard { marker: "credits-balance-available", icon: "coins", label: "Available Balance", highlighted: true }
                        UnavailableBalanceCard { marker: "credits-balance-earned", icon: "trending-up", label: "Lifetime Earned", highlighted: false }
                        UnavailableBalanceCard { marker: "credits-balance-spent", icon: "trending-down", label: "Lifetime Spent", highlighted: false }
                    },
                }
            }

            div { class: "credits-transaction-list card card-glass mt-6 overflow-hidden fe-surface",
                div { class: "border-b border-border p-6",
                    h2 { id: "credits-transaction-title", class: "text-xl font-semibold text-foreground fe-tone-text", "Transaction History" }
                }
                match history {
                    CreditHistoryLoad::Ready(history) => rsx! { CreditHistoryList { history } },
                    CreditHistoryLoad::Empty => rsx! {
                        CreditsMessage { state: "empty", title: "No credit transactions yet", detail: "Transactions will appear here when credits are earned or spent." }
                    },
                    CreditHistoryLoad::Malformed => rsx! {
                        CreditsMessage { state: "malformed", title: "Credit history is unavailable", detail: "We couldn’t load your credit history. Please try again." }
                    },
                    CreditHistoryLoad::Unavailable | CreditHistoryLoad::SignedOut => rsx! {
                        CreditsMessage { state: "unavailable", title: "Credit history is temporarily unavailable", detail: "We couldn’t load your credit history. Please try again." }
                    },
                }
            }
        }
    }
}

#[component]
fn CreditBalanceCard(
    marker: &'static str,
    icon: &'static str,
    label: &'static str,
    value: String,
    highlighted: bool,
) -> Element {
    let card_class = if highlighted {
        format!(
            "{marker} card card-glass border-primary/20 bg-gradient-to-br from-primary/10 to-transparent"
        )
    } else {
        format!("{marker} card card-glass")
    };
    rsx! {
        div { class: "{card_class} fe-surface",
            div { class: "card-body",
                div { class: "mb-3 flex items-center justify-between",
                    div { class: "rounded-lg bg-primary/10 p-2 text-primary",
                        FrontendIcon { name: icon.to_string(), size: Some(20) }
                    }
                    span { class: "rounded-full border border-emerald-500/30 bg-emerald-500/10 px-2 py-1 text-xs font-medium text-emerald-500 fe-tone-positive", "Verified" }
                }
                p { class: "text-sm text-muted-foreground fe-tone-muted", "{label}" }
                p { class: "mt-1 text-2xl font-semibold text-foreground fe-tone-text", "data-credit-value": value.clone(),
                    "{value} credits"
                }
            }
        }
    }
}

#[component]
fn CreditHistoryList(history: CreditHistoryProjection) -> Element {
    rsx! {
        ol { class: "divide-y divide-border", "data-credit-history-count": history.count,
            for transaction in history.transactions {
                li { class: "p-5 sm:p-6",
                    article { class: "flex flex-col justify-between gap-3 sm:flex-row sm:items-start",
                        div { class: "min-w-0",
                            p { class: "font-semibold text-foreground fe-tone-text", "{transaction.tx_type}" }
                            if let Some(reason) = transaction.reason {
                                p { class: "mt-1 text-sm text-muted-foreground fe-tone-muted", "{reason}" }
                            }
                            p { class: "mt-1 font-mono text-xs text-muted-foreground break-all fe-tone-muted", "{transaction.id}" }
                        }
                        div { class: "sm:text-right",
                            p { class: "font-semibold text-foreground fe-tone-text", "{transaction.amount} credits" }
                            p { class: "mt-1 text-xs text-muted-foreground fe-tone-muted", "Balance {transaction.balance_after}" }
                            time { class: "mt-1 block text-xs text-muted-foreground fe-tone-muted", datetime: transaction.created_at.clone(), "{transaction.created_at}" }
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn CreditsMessage(state: &'static str, title: &'static str, detail: &'static str) -> Element {
    let role = if matches!(state, "unavailable" | "malformed") {
        "alert"
    } else {
        "status"
    };
    rsx! {
        div { class: "p-8 sm:p-10 text-center", "data-credit-history-state": state, role,
            Icon { name: "coins".to_string(), size: Some(36), class_name: Some("text-muted-foreground".to_string()) }
            h3 { class: "mt-3 text-lg font-semibold text-foreground fe-tone-text", "{title}" }
            p { class: "mx-auto mt-2 max-w-xl text-sm leading-6 text-muted-foreground fe-tone-muted", "{detail}" }
            crate::fullstack::shell::ShellLink { class: "btn btn-outline mt-5", href: ACCOUNT_PATH, "Back to account" }
        }
    }
}

#[component]
fn UnavailableBalanceCard(
    marker: &'static str,
    icon: &'static str,
    label: &'static str,
    highlighted: bool,
) -> Element {
    let card_class = if highlighted {
        format!(
            "{marker} card card-glass border-primary/20 bg-gradient-to-br from-primary/10 to-transparent"
        )
    } else {
        format!("{marker} card card-glass")
    };

    rsx! {
        div { class: "{card_class} fe-surface",
            div { class: "card-body",
                div { class: "mb-3 flex items-center justify-between",
                    div { class: "rounded-lg bg-primary/10 p-2 text-primary",
                        FrontendIcon { name: icon.to_string(), size: Some(20) }
                    }
                    span {
                        class: "rounded-full border border-amber-500/30 bg-amber-500/10 px-2 py-1 text-xs font-medium text-amber-500 fe-tone-warning",
                        "Unavailable"
                    }
                }
                p { class: "text-sm text-muted-foreground fe-tone-muted", "{label}" }
                p {
                    class: "mt-1 text-2xl font-semibold text-foreground fe-tone-text",
                    "data-credit-value": "unavailable",
                    "Not available"
                }
            }
        }
    }
}

#[cfg(feature = "server")]
pub type CreditsProviderCallback = std::sync::Arc<
    dyn Fn(
            http::HeaderMap,
        ) -> std::pin::Pin<
            Box<
                dyn std::future::Future<Output = Result<CreditsData, crate::fullstack::LoadError>>
                    + Send,
            >,
        > + Send
        + Sync,
>;

#[cfg(test)]
mod tests {
    fn has_link(html: &str, href: &str, label: &str, rel: Option<&str>) -> bool {
        html.split("<a ")
            .filter_map(|tail| tail.split_once("</a>"))
            .any(|(anchor, _)| {
                let Some((attributes, text)) = anchor.split_once('>') else {
                    return false;
                };
                attributes.contains(&format!("href=\"{href}\""))
                    && text == label
                    && rel.is_none_or(|value| attributes.contains(&format!("rel=\"{value}\"")))
            })
    }

    use super::*;
    use crate::auth::user::{AuthMethod, User};

    fn signed_out_ctx() -> PageContext {
        PageContext {
            user: None,
            path: "/account/credits".to_string(),
            ..Default::default()
        }
    }

    fn authed_ctx_with_address(address: &str) -> PageContext {
        PageContext {
            user: Some(User {
                id: "u-1".to_string(),
                address: address.to_string(),
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

    fn render_html(ctx: &PageContext) -> String {
        let (_meta, element) = render(ctx);
        dioxus_ssr::render_element(element)
    }

    fn assert_no_inferred_financial_state(html: &str) {
        for forbidden in [
            "$0",
            "No transactions found",
            "credits-filter-chip",
            "Admin Grant",
            "Admin Revoke",
            "Proration Credit",
            "Daily credit grant",
            "$250",
            "$1,250",
        ] {
            assert!(
                !html.contains(forbidden),
                "credits page must not render inferred, canned, or inert content `{forbidden}`. Got: {html}"
            );
        }
    }

    #[test]
    fn signed_out_is_a_status_with_native_return_link() {
        let html = render_html(&signed_out_ctx());

        assert!(html.contains("data-credits-state=\"signed-out\""));
        assert!(html.contains("role=\"status\""));
        assert!(html.contains("Sign in to view your credits"));
        assert!(html.contains("href=\"/auth?return_url=%2Faccount%2Fcredits\""));
        assert!(!html.contains("data-credits-state=\"unavailable\""));
        assert_no_inferred_financial_state(&html);
    }

    #[test]
    fn authenticated_state_is_an_alert_with_meaningful_account_navigation() {
        let html = render_html(&authed_ctx_with_address("0x1234abcd"));

        assert!(html.contains("data-credits-state=\"unavailable\""));
        assert!(html.contains("role=\"alert\""));
        assert!(html.contains("Credit history is temporarily unavailable"));
        assert!(html.contains("We couldn’t load your credit history. Please try again."));
        assert!(has_link(&html, "/account", "Back to account", None));
        assert!(!html.contains("href=\"/account/credits\""));
        assert!(!html.contains(">Retry</a>"));
        assert_no_inferred_financial_state(&html);
    }

    #[test]
    fn canned_partial_and_malformed_params_are_never_financial_data() {
        let mut canned = authed_ctx_with_address("0x1234abcd");
        canned.params.insert(
            "data_credits".to_string(),
            r#"{"available_balance":250,"lifetime_earned":1250,"transactions":[{"title":"Daily credit grant","amount":250}]}"#.to_string(),
        );
        let canned_html = render_html(&canned);

        let mut malformed = authed_ctx_with_address("0x1234abcd");
        malformed
            .params
            .insert("data_credits".to_string(), "{not-json".to_string());
        let malformed_html = render_html(&malformed);

        for html in [&canned_html, &malformed_html] {
            assert!(html.contains("data-credits-state=\"unavailable\""));
            assert!(html.contains("data-credit-value=\"unavailable\""));
            assert_no_inferred_financial_state(html);
        }
    }

    #[test]
    fn unavailable_layout_stays_source_like_without_inert_filter_controls() {
        let html = render_html(&authed_ctx_with_address("0x1234abcd"));

        for marker in [
            "credits-ledger-page",
            "credits-balance-row",
            "credits-balance-available",
            "credits-balance-earned",
            "credits-balance-spent",
            "credits-transaction-list",
        ] {
            assert!(
                html.contains(marker),
                "truthful unavailable page must retain source-like section `{marker}`. Got: {html}"
            );
        }
        assert_eq!(html.matches("data-credit-value=\"unavailable\"").count(), 3);
        assert_no_inferred_financial_state(&html);
    }

    #[test]
    fn dynamic_owner_is_not_emitted_by_unavailable_state() {
        let html = render_html(&authed_ctx_with_address("<script>alert('owner')</script>"));

        assert!(!html.contains("<script>alert('owner')</script>"));
        assert!(!html.contains("&#60;script&#62;alert"));
        assert_no_inferred_financial_state(&html);
    }

    #[test]
    fn verified_zero_balance_and_empty_history_render_as_authoritative_data() {
        let mut ctx = authed_ctx_with_address("0x1234abcd");
        ctx.params.insert(
            ACCOUNT_CREDIT_BALANCE_STATE_PARAM.to_string(),
            ACCOUNT_CREDIT_READY.to_string(),
        );
        ctx.params.insert(
            ACCOUNT_CREDIT_BALANCE_DATA_PARAM.to_string(),
            serde_json::json!({
                "wallet_address": "0x1234ABCD",
                "balance": "0",
                "pending_balance": "0",
                "available_balance": "0",
                "lifetime_earned": "0",
                "lifetime_spent": "0",
                "last_transaction_at": null
            })
            .to_string(),
        );
        ctx.params.insert(
            ACCOUNT_CREDIT_HISTORY_STATE_PARAM.to_string(),
            ACCOUNT_CREDIT_EMPTY.to_string(),
        );
        ctx.params.insert(
            ACCOUNT_CREDIT_HISTORY_DATA_PARAM.to_string(),
            serde_json::json!({"success": true, "data": [], "count": 0}).to_string(),
        );

        let html = render_html(&ctx);
        assert!(html.contains("data-credits-state=\"ready\""));
        assert_eq!(html.matches(">Verified<").count(), 3);
        assert_eq!(html.matches(">0 credits<").count(), 3);
        assert!(html.contains("No credit transactions yet"));
        assert!(!html.contains("data-credit-value=\"unavailable\""));
    }

    #[test]
    fn decoders_reject_cross_owner_and_non_empty_count_mismatch() {
        assert!(decode_credit_balance(
            serde_json::json!({
                "wallet_address": "0xother",
                "balance": "0",
                "pending_balance": "0",
                "available_balance": "0",
                "lifetime_earned": "0",
                "lifetime_spent": "0",
                "last_transaction_at": null
            }),
            "0x1234abcd"
        )
        .is_none());
        assert!(decode_credit_history(
            serde_json::json!({"success": true, "data": [], "count": 1}),
            "0x1234abcd",
            ACCOUNT_CREDIT_HISTORY_MAX_ITEMS
        )
        .is_none());
    }

    #[test]
    fn credit_history_accepts_backend_decimal_strings_without_losing_precision() {
        let mut wire = serde_json::json!({"success":true,"count":1,"data":[{"id":"tx-rehearsal","wallet_address":"0x1234abcd","amount":"9007199254740993.25","balance_after":"9007199254740994.50","tx_type":"grant","reference_id":null,"reference_type":null,"reason":null,"granted_by":null,"expires_at":null,"created_at":"2026-09-13T00:00:00Z"}]});
        let read = decode_credit_history(wire.clone(), "0x1234abcd", 20).unwrap();
        assert_eq!(read.transactions[0].amount, "9007199254740993.25");
        wire["data"][0]["amount"] = serde_json::json!(1.25);
        assert!(decode_credit_history(wire.clone(), "0x1234abcd", 20).is_some());
        wire["data"][0]["amount"] = serde_json::json!("NaN");
        assert!(decode_credit_history(wire, "0x1234abcd", 20).is_none());
    }
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct CreditsData {
    pub balance: Result<CreditBalanceProjection, crate::fullstack::LoadError>,
    pub history: Result<CreditHistoryProjection, crate::fullstack::LoadError>,
}
#[cfg(feature = "server")]
#[derive(Clone)]
pub struct CreditsProvider(pub CreditsProviderCallback);
#[server(prefix = "/_server/frontend", endpoint = "credits")]
pub async fn read_credits(
) -> Result<Result<CreditsData, crate::fullstack::LoadError>, ServerFnError> {
    use dioxus_server::axum::Extension;
    let Extension(provider) =
        dioxus_fullstack::FullstackContext::extract::<Extension<CreditsProvider>, _>()
            .await
            .map_err(|_| ServerFnError::new("Credits provider unavailable"))?;
    let headers = dioxus_fullstack::FullstackContext::extract::<http::HeaderMap, _>()
        .await
        .map_err(|_| ServerFnError::new("Request context unavailable"))?;
    Ok((provider.0)(headers).await)
}
#[component]
pub fn HydratedCredits() -> Element {
    let mut result = use_server_future(move || async move { read_credits().await })?;
    let snapshot = result.read().clone();
    rsx! {
        document::Title { "Credits — EPSX" }
        document::Meta { name: "description", content: "Manage your EPSX credits and view transaction history." }
        div { class: "page-content credits-ledger-page mx-auto max-w-6xl px-6 pt-6 fe-page-layout", "data-dioxus-credits": "true",
            div { class: "mb-6",
                h1 { class: "text-3xl font-bold text-foreground fe-tone-text fe-type-title", "Credit Balance" }
                p { class: "mt-2 text-slate-400 fe-tone-muted", "Manage your EPSX credits and view transaction history" }
            }
            match snapshot {
                Some(Ok(Ok(data))) => {
                    let balance = match data.balance {
                        Ok(value) => CreditBalanceLoad::Ready(value),
                        Err(crate::fullstack::LoadError::Malformed) => CreditBalanceLoad::Malformed,
                        Err(_) => CreditBalanceLoad::Unavailable,
                    };
                    let history = match data.history {
                        Ok(value) if value.transactions.is_empty() => CreditHistoryLoad::Empty,
                        Ok(value) => CreditHistoryLoad::Ready(value),
                        Err(crate::fullstack::LoadError::Malformed) => CreditHistoryLoad::Malformed,
                        Err(_) => CreditHistoryLoad::Unavailable,
                    };
                    rsx! { CreditsOwnerView { balance, history } }
                },
                Some(Ok(Err(crate::fullstack::LoadError::Unauthenticated))) => rsx! { CreditsSignedOut {} },
                None => rsx! { p { role: "status", "Loading credits…" } },
                _ => rsx! { p { role: "status", "Credit information is temporarily unavailable." } },
            }
            button { r#type: "button", class: "fe-button", onclick: move |_| result.restart(), "Refresh credits" }
        }
    }
}
