//! /wallet-management/credits — authenticated, read-only credit statistics.
//!
//! The page accepts only a strict, redacted projection from the admin BFF.
//! Ledger entries, actors, operation IDs, correlation IDs, balances by wallet,
//! and every financial mutation remain outside the PageContext contract.

use dioxus::prelude::*;
use serde::{Deserialize, Serialize};

use crate::auth::AuthGate;
use crate::components::admin::page_layout::{PageGradient, PageHeader, PageLayout, PageMaxWidth};
use crate::primitives::Icon;

use super::super::{PageContext, PageMeta};

const WALLET_CREDITS_PATH: &str = "/wallet-management/credits";
const ADMIN_HOME_PATH: &str = "/";
const MAX_SAFE_MINOR_UNITS: i64 = 9_000_000_000_000_000_000;

pub const ADMIN_CREDITS_DATA_PARAM: &str = "data_admin_credits";
pub const ADMIN_CREDITS_STATE_PARAM: &str = "data_admin_credits_state";

pub const ADMIN_CREDITS_READY: &str = "ready";
pub const ADMIN_CREDITS_FORBIDDEN: &str = "forbidden";
pub const ADMIN_CREDITS_UNAVAILABLE: &str = "unavailable";
pub const ADMIN_CREDITS_MALFORMED: &str = "malformed";

/// Redacted read-only fields from CreditStatsResponse. Financial amounts are
/// retained as backend-owned minor units; correlation IDs and ledger identity
/// are intentionally not carried into page HTML.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AdminCreditStatsProjection {
    pub outstanding_minor: i64,
    pub granted_today_minor: i64,
    pub revoked_today_minor: i64,
    pub active_accounts: i64,
}

pub fn decode_admin_credit_stats_projection(
    value: serde_json::Value,
) -> Option<AdminCreditStatsProjection> {
    let projection: AdminCreditStatsProjection = serde_json::from_value(value).ok()?;
    let amounts = [
        projection.outstanding_minor,
        projection.granted_today_minor,
        projection.revoked_today_minor,
    ];
    if amounts
        .iter()
        .any(|amount| !(0..=MAX_SAFE_MINOR_UNITS).contains(amount))
        || !(0..=MAX_SAFE_MINOR_UNITS).contains(&projection.active_accounts)
    {
        return None;
    }
    Some(projection)
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum CreditLoad {
    Ready(AdminCreditStatsProjection),
    Forbidden,
    Unavailable,
    Malformed,
}

fn credit_load(ctx: &PageContext) -> CreditLoad {
    match ctx
        .params
        .get(ADMIN_CREDITS_STATE_PARAM)
        .map(String::as_str)
    {
        Some(ADMIN_CREDITS_READY) => {
            let Some(raw) = ctx.params.get(ADMIN_CREDITS_DATA_PARAM) else {
                return CreditLoad::Malformed;
            };
            serde_json::from_str(raw)
                .ok()
                .and_then(decode_admin_credit_stats_projection)
                .map(CreditLoad::Ready)
                .unwrap_or(CreditLoad::Malformed)
        }
        Some(ADMIN_CREDITS_FORBIDDEN) => CreditLoad::Forbidden,
        Some(ADMIN_CREDITS_MALFORMED) => CreditLoad::Malformed,
        Some(ADMIN_CREDITS_UNAVAILABLE) | None => CreditLoad::Unavailable,
        Some(_) => CreditLoad::Malformed,
    }
}

pub fn render(ctx: &PageContext) -> (PageMeta, Element) {
    let meta = PageMeta::admin("Credits");
    (
        meta,
        rsx! {
            AuthGate {
                user: ctx.user.clone(),
                feature: Some("the private wallet credits workspace".to_string()),
                return_url: Some(WALLET_CREDITS_PATH.to_string()),
                RenderWalletCredits { ctx: ctx.clone() }
            }
        },
    )
}

#[component]
fn RenderWalletCredits(ctx: PageContext) -> Element {
    let load = credit_load(&ctx);

    rsx! {
        PageLayout {
            max_width: Some(PageMaxWidth::SevenXl),
            PageHeader {
                title: "Wallet credits".to_string(),
                subtitle: Some("Review backend-authoritative credit totals".to_string()),
                icon: Some("coins".to_string()),
                gradient: Some(PageGradient::Info),
                centered: Some(false),
                extra_actions: None,
                class_name: None,
            }
            match load {
                CreditLoad::Ready(projection) => rsx! { CreditStatsReady { projection } },
                CreditLoad::Forbidden => rsx! {
                    CreditProblem {
                        state: ADMIN_CREDITS_FORBIDDEN,
                        title: "Credit statistics access was denied".to_string(),
                        detail: "The backend did not authorize this session to read credit totals.".to_string(),
                    }
                },
                CreditLoad::Unavailable => rsx! {
                    CreditProblem {
                        state: ADMIN_CREDITS_UNAVAILABLE,
                        title: "Credit statistics are unavailable".to_string(),
                        detail: "The wallet backend could not provide authoritative credit totals. No amounts are being shown.".to_string(),
                    }
                },
                CreditLoad::Malformed => rsx! {
                    CreditProblem {
                        state: ADMIN_CREDITS_MALFORMED,
                        title: "Credit data could not be verified".to_string(),
                        detail: "The backend response did not match the strict redacted credit contract. No amounts are being shown.".to_string(),
                    }
                },
            }
        }
    }
}

#[component]
fn CreditStatsReady(projection: AdminCreditStatsProjection) -> Element {
    rsx! {
        section {
            class: "overflow-hidden rounded-2xl border border-border/30 bg-card shadow-xl",
            role: "status",
            aria_labelledby: "admin-credit-stats-title",
            "data-admin-wallet-credits-state": ADMIN_CREDITS_READY,
            div { class: "h-1 bg-gradient-to-r from-[#1fc7d4] via-[#7645d9] to-[#ffb237]", aria_hidden: "true" }
            div { class: "p-5 sm:p-6",
                h2 { id: "admin-credit-stats-title", class: "text-lg font-semibold text-foreground", "Credit totals" }
                p { class: "mt-1 max-w-3xl text-sm leading-6 text-muted-foreground",
                    "Amounts are displayed exactly as backend-owned minor units. No fiat conversion or ledger interpretation is applied here."
                }
            }
            dl { class: "grid grid-cols-1 gap-px border-t border-border/30 bg-border/30 sm:grid-cols-2 xl:grid-cols-4",
                CreditMetric { label: "Outstanding minor units", value: format_minor(projection.outstanding_minor) }
                CreditMetric { label: "Granted today, minor units", value: format_minor(projection.granted_today_minor) }
                CreditMetric { label: "Revoked today, minor units", value: format_minor(projection.revoked_today_minor) }
                CreditMetric { label: "Active credit accounts", value: format_minor(projection.active_accounts) }
            }
            p { class: "border-t border-border/30 px-5 py-4 text-xs leading-5 text-muted-foreground sm:px-6",
                "This read-only projection contains no wallet ledger rows or credit operation controls."
            }
        }
    }
}

#[component]
fn CreditMetric(label: &'static str, value: String) -> Element {
    rsx! {
        div { class: "min-w-0 bg-card p-5 sm:p-6",
            dt { class: "text-sm font-medium text-muted-foreground", "{label}" }
            dd { class: "mt-2 break-words text-2xl font-black tracking-tight text-foreground", "{value}" }
        }
    }
}

fn format_minor(value: i64) -> String {
    let digits = value.to_string();
    let first_group = digits.len() % 3;
    let mut formatted = String::with_capacity(digits.len() + digits.len() / 3);
    for (index, byte) in digits.bytes().enumerate() {
        if index > 0 && index % 3 == first_group {
            formatted.push(',');
        }
        formatted.push(char::from(byte));
    }
    formatted
}

#[component]
fn CreditProblem(state: &'static str, title: String, detail: String) -> Element {
    rsx! {
        section {
            class: "rounded-2xl border border-amber-500/25 bg-amber-500/10 p-6 sm:p-8",
            role: if state == ADMIN_CREDITS_FORBIDDEN { "alert" } else { "status" },
            aria_labelledby: "admin-credit-problem-title",
            "data-admin-wallet-credits-state": state,
            div { class: "flex flex-col gap-5 sm:flex-row sm:items-start",
                div {
                    class: "flex h-12 w-12 shrink-0 items-center justify-center rounded-xl border border-amber-500/25 bg-background/60 text-amber-700 dark:text-amber-300",
                    aria_hidden: "true",
                    Icon { name: "shield-alert".to_string(), size: Some(24) }
                }
                div { class: "min-w-0",
                    h2 { id: "admin-credit-problem-title", class: "text-xl font-bold text-foreground", "{title}" }
                    p { class: "mt-2 max-w-3xl text-sm leading-6 text-muted-foreground", "{detail}" }
                    nav { class: "mt-5 flex flex-wrap gap-3", aria_label: "Credit statistics recovery",
                        a { class: "btn btn-sm btn-outline", href: WALLET_CREDITS_PATH, "Retry statistics" }
                        a { class: "btn btn-sm btn-ghost", href: ADMIN_HOME_PATH, "Admin home" }
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::auth::user::{AuthMethod, User};

    fn signed_in() -> PageContext {
        PageContext {
            user: Some(User {
                id: "credits-session".to_string(),
                address: "0xsession".to_string(),
                chain_id: "56".to_string(),
                auth_method: AuthMethod::Wallet,
                ..Default::default()
            }),
            path: WALLET_CREDITS_PATH.to_string(),
            ..Default::default()
        }
    }

    fn html(ctx: &PageContext) -> String {
        dioxus_ssr::render_element(render(ctx).1)
    }

    fn with_state(state: &str, data: Option<serde_json::Value>) -> PageContext {
        let mut ctx = signed_in();
        ctx.params
            .insert(ADMIN_CREDITS_STATE_PARAM.to_string(), state.to_string());
        if let Some(data) = data {
            ctx.params
                .insert(ADMIN_CREDITS_DATA_PARAM.to_string(), data.to_string());
        }
        ctx
    }

    fn projection() -> serde_json::Value {
        serde_json::json!({
            "outstanding_minor": 1200,
            "granted_today_minor": 500,
            "revoked_today_minor": 100,
            "active_accounts": 3,
        })
    }

    #[test]
    fn strict_credit_projection_accepts_zero_and_redacts_unknown_fields() {
        assert!(decode_admin_credit_stats_projection(serde_json::json!({
            "outstanding_minor": 0,
            "granted_today_minor": 0,
            "revoked_today_minor": 0,
            "active_accounts": 0,
        }))
        .is_some());
        assert!(decode_admin_credit_stats_projection(serde_json::json!({
            "outstanding_minor": 1,
            "granted_today_minor": 0,
            "revoked_today_minor": 0,
            "active_accounts": 1,
            "correlation_id": "private-operation-id",
        }))
        .is_none());
        assert!(decode_admin_credit_stats_projection(serde_json::json!({
            "outstanding_minor": -1,
            "granted_today_minor": 0,
            "revoked_today_minor": 0,
            "active_accounts": 0,
        }))
        .is_none());
    }

    #[test]
    fn ready_projection_is_read_only_and_financial_state_is_explicit() {
        let rendered = html(&with_state(ADMIN_CREDITS_READY, Some(projection())));
        assert!(rendered.contains("data-admin-wallet-credits-state=\"ready\""));
        assert!(rendered.contains("1,200"));
        assert!(rendered.contains("Amounts are displayed exactly as backend-owned minor units"));
        assert!(!rendered.contains("<form"));
        assert!(!rendered.contains("<button"));
        assert!(!rendered.contains("grant"));
        assert!(!rendered.contains("revoke"));
    }

    #[test]
    fn forbidden_unavailable_and_malformed_never_render_stale_amounts() {
        for (state, title) in [
            (
                ADMIN_CREDITS_FORBIDDEN,
                "Credit statistics access was denied",
            ),
            (
                ADMIN_CREDITS_UNAVAILABLE,
                "Credit statistics are unavailable",
            ),
            (ADMIN_CREDITS_MALFORMED, "Credit data could not be verified"),
        ] {
            let rendered = html(&with_state(state, Some(projection())));
            assert!(rendered.contains(&format!("data-admin-wallet-credits-state=\"{state}\"")));
            assert!(rendered.contains(title));
            assert!(!rendered.contains("1,200"));
        }
    }

    #[test]
    fn signed_out_projection_and_hostile_params_stay_private() {
        let mut ctx = with_state(ADMIN_CREDITS_READY, Some(projection()));
        ctx.user = None;
        ctx.query = "amount=999999&action=grant".to_string();
        ctx.params
            .insert("legacy_data".to_string(), "secret".to_string());
        let rendered = html(&ctx);
        assert!(rendered.contains("Sign in required"));
        assert!(!rendered.contains("data-admin-wallet-credits-state"));
        assert!(!rendered.contains("1,200"));
        assert!(!rendered.contains("secret"));
    }
}
