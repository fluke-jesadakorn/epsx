//! /wallet-management/credits — authenticated, read-only credit statistics.
//!
//! The page accepts only a strict, redacted projection from the admin BFF.
//! Ledger entries, actors, operation IDs, correlation IDs, balances by wallet,
//! and every financial mutation remain outside the PageContext contract.

use dioxus::prelude::*;
use serde::{Deserialize, Serialize};

use crate::auth::AuthGate;
use crate::components::admin::data_state_banner::{AdminDataState, AdminDataStateBanner};
use crate::components::admin::page_layout::{PageGradient, PageHeader};
use crate::primitives::Icon;

use super::super::{PageContext, PageMeta};
use super::wallet_hub::WalletManagementHub;

const WALLET_CREDITS_PATH: &str = "/wallet-management/credits";
const ADMIN_HOME_PATH: &str = "/";
const MAX_SAFE_MINOR_UNITS: i64 = 9_000_000_000_000_000_000;

pub const ADMIN_CREDITS_DATA_PARAM: &str = "data_admin_credits";
pub const ADMIN_CREDITS_STATE_PARAM: &str = "data_admin_credits_state";

pub const ADMIN_CREDITS_READY: &str = "ready";
pub const ADMIN_CREDITS_FORBIDDEN: &str = "forbidden";
pub const ADMIN_CREDITS_UNAVAILABLE: &str = "unavailable";
pub const ADMIN_CREDITS_MALFORMED: &str = "malformed";
pub const ADMIN_CREDITS_UNAUTHENTICATED: &str = "unauthenticated";
pub const ADMIN_CREDITS_UNAUTHORIZED: &str = "unauthorized";

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
    Unauthenticated,
    Unauthorized,
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
        Some(ADMIN_CREDITS_UNAUTHENTICATED) => CreditLoad::Unauthenticated,
        Some(ADMIN_CREDITS_UNAUTHORIZED) => CreditLoad::Unauthorized,
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
    let mutation = match ctx.query_param("mutation").as_deref() {
        Some("success") | Some("conflict") | Some("forbidden") | Some("unavailable")
        | Some("malformed") => ctx.query_param("mutation"),
        _ => None,
    };

    rsx! {
        WalletManagementHub {
            ctx: ctx.clone(),
            PageHeader {
                title: "Wallet credits".to_string(),
                subtitle: Some("Review backend-authoritative credit totals".to_string()),
                icon: Some("coins".to_string()),
                gradient: Some(PageGradient::Info),
                centered: Some(false),
                extra_actions: None,
                class_name: None,
            }
            CreditWorkspaceNav {}
            match load {
                CreditLoad::Ready(projection) => rsx! { CreditStatsReady { projection, mutation } },
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
                CreditLoad::Unauthenticated => rsx! {
                    AdminDataStateBanner {
                        state: AdminDataState::Unauthenticated,
                        subject: "Wallet credits".to_string(),
                        return_path: WALLET_CREDITS_PATH.to_string(),
                        retry_href: WALLET_CREDITS_PATH.to_string(),
                    }
                },
                CreditLoad::Unauthorized => rsx! {
                    AdminDataStateBanner {
                        state: AdminDataState::Unauthorized,
                        subject: "Wallet credits".to_string(),
                        return_path: WALLET_CREDITS_PATH.to_string(),
                        retry_href: WALLET_CREDITS_PATH.to_string(),
                    }
                },
            }
        }
    }
}

#[component]
fn CreditWorkspaceNav() -> Element {
    rsx! {
        nav { class: "flex gap-1 overflow-x-auto border-b border-border/30", aria_label: "Credit workspace",
            a { class: "relative flex items-center gap-2 whitespace-nowrap px-4 py-3 text-sm font-semibold text-[#1fc7d4]", href: "#credit-overview", aria_current: "page",
                Icon { name: "bar-chart-3".to_string(), size: Some(16) }
                "Overview"
                span { class: "absolute bottom-0 left-0 right-0 h-[2px] bg-gradient-to-r from-[#1fc7d4] to-[#7645d9]", aria_hidden: "true" }
            }
            a { class: "flex items-center gap-2 whitespace-nowrap px-4 py-3 text-sm font-semibold text-muted-foreground hover:text-foreground", href: "#credit-grant",
                Icon { name: "plus".to_string(), size: Some(16) }
                "Grant Credits"
            }
            span { class: "flex cursor-not-allowed items-center gap-2 whitespace-nowrap px-4 py-3 text-sm font-semibold text-muted-foreground opacity-50", title: "Credit history requires a wallet-ledger read contract", aria_disabled: "true",
                Icon { name: "clock".to_string(), size: Some(16) }
                "Credit History"
            }
        }
    }
}

#[component]
fn CreditStatsReady(projection: AdminCreditStatsProjection, mutation: Option<String>) -> Element {
    rsx! {
        section {
            id: "credit-overview",
            class: "overflow-hidden rounded-2xl border border-border/30 bg-card shadow-xl",
            role: "status",
            aria_labelledby: "admin-credit-stats-title",
            "data-admin-wallet-credits-state": ADMIN_CREDITS_READY,
            if let Some(state) = mutation {
                p { class: "border-b border-amber-500/30 bg-amber-500/5 px-5 py-3 text-sm", role: if state == "forbidden" { "alert" } else { "status" },
                    "data-admin-wallet-credits-mutation-state": state,
                    "Credit mutation: {state}"
                }
            }
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
            form { id: "credit-grant", method: "post", action: "/wallet-management/credits", class: "grid gap-3 border-t border-border/30 p-5 sm:grid-cols-2 lg:grid-cols-5",
                input { r#type: "hidden", name: "operation", value: "credit_grant" }
                input { r#type: "hidden", name: "idempotency_key", value: format!("admin.credits.grant.{}", uuid::Uuid::new_v4()) }
                input { class: "input input-bordered", name: "wallet_address", maxlength: 42, placeholder: "Wallet 0x...", required: true }
                input { class: "input input-bordered", name: "expected_version", r#type: "number", min: 0, placeholder: "Version", required: true }
                input { class: "input input-bordered", name: "amount_minor", r#type: "number", min: 1, placeholder: "Minor units", required: true }
                input { class: "input input-bordered", name: "reason", maxlength: 500, placeholder: "Reason", required: true }
                button { r#type: "submit", class: "btn btn-primary", "Grant credits" }
            }
            form { method: "post", action: "/wallet-management/credits", class: "grid gap-3 border-t border-border/30 p-5 sm:grid-cols-2 lg:grid-cols-5",
                input { r#type: "hidden", name: "operation", value: "credit_revoke" }
                input { r#type: "hidden", name: "idempotency_key", value: format!("admin.credits.revoke.{}", uuid::Uuid::new_v4()) }
                input { class: "input input-bordered", name: "wallet_address", maxlength: 42, placeholder: "Wallet 0x...", required: true }
                input { class: "input input-bordered", name: "expected_version", r#type: "number", min: 0, placeholder: "Version", required: true }
                input { class: "input input-bordered", name: "amount_minor", r#type: "number", min: 1, placeholder: "Minor units", required: true }
                input { class: "input input-bordered", name: "reason", maxlength: 500, placeholder: "Reason", required: true }
                button { r#type: "submit", class: "btn btn-outline", "Revoke credits" }
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
        div {
            "data-admin-wallet-credits-state": state,
            section {
                class: "rounded-xl border border-amber-500/25 bg-amber-500/10 px-5 py-4",
                role: if state == ADMIN_CREDITS_FORBIDDEN { "alert" } else { "status" },
                aria_labelledby: "admin-credit-problem-title",
                div { class: "flex flex-col gap-4 sm:flex-row sm:items-center sm:justify-between",
                    div { class: "min-w-0",
                        h2 { id: "admin-credit-problem-title", class: "font-semibold text-foreground", "{title}" }
                        p { class: "mt-1 max-w-3xl text-sm text-muted-foreground", "{detail}" }
                    }
                    nav { class: "flex shrink-0 flex-wrap gap-2", aria_label: "Credit statistics recovery",
                        a { class: "btn btn-sm btn-outline", href: WALLET_CREDITS_PATH, "Retry statistics" }
                        a { class: "btn btn-sm btn-ghost", href: ADMIN_HOME_PATH, "Admin home" }
                    }
                }
            }
            section { id: "credit-overview", class: "mt-6", aria_label: "Credit overview unavailable",
                div { class: "grid grid-cols-1 gap-4 md:grid-cols-2 lg:grid-cols-4",
                    for (label, icon, gradient) in [
                        ("Total Credits Outstanding", "coins", "from-[#1fc7d4] to-[#7645d9]"),
                        ("Credits Granted Today", "trending-up", "from-[#31d0aa] to-[#1fc7d4]"),
                        ("Credits Used Today", "trending-down", "from-[#ffb237] to-[#ed4b9e]"),
                        ("Active Users with Credits", "users", "from-[#7645d9] to-[#ed4b9e]"),
                    ] {
                        article { class: "overflow-hidden rounded-xl border border-border/20 bg-card p-5 shadow-xl",
                            span { class: "mb-3 inline-flex h-10 w-10 items-center justify-center rounded-lg bg-gradient-to-r text-white {gradient}", aria_hidden: "true",
                                Icon { name: icon.to_string(), size: Some(20) }
                            }
                            p { class: "text-[10px] font-bold uppercase tracking-[0.15em] text-muted-foreground", "{label}" }
                            p { class: "mt-2 font-mono text-xl font-black text-amber-400", "Unavailable" }
                        }
                    }
                }
                div { class: "mt-6 flex flex-wrap gap-3",
                    a { class: "btn btn-sm bg-gradient-to-r from-[#7645d9] to-[#5a33b8] text-white", href: WALLET_CREDITS_PATH,
                        Icon { name: "refresh-cw".to_string(), size: Some(15) }
                        " Refresh Stats"
                    }
                    button { class: "btn btn-sm btn-outline", r#type: "button", disabled: true, "Export Report (Coming Soon)" }
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
    fn ready_projection_is_authoritative_and_exposes_bounded_credit_mutations() {
        let rendered = html(&with_state(ADMIN_CREDITS_READY, Some(projection())));
        assert!(rendered.contains("data-admin-wallet-credits-state=\"ready\""));
        assert!(rendered.contains("Overview"));
        assert!(rendered.contains("Grant Credits"));
        assert!(rendered.contains("Credit History"));
        assert!(rendered.contains("1,200"));
        assert!(rendered.contains("Amounts are displayed exactly as backend-owned minor units"));
        assert!(rendered.contains("<form"));
        assert!(rendered.contains("Grant credits"));
        assert!(rendered.contains("Revoke credits"));
        assert!(rendered.contains("expected_version"));
        assert!(rendered.contains("idempotency_key"));
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
            assert!(rendered.contains("Total Credits Outstanding"));
            assert!(rendered.contains("Refresh Stats"));
            assert!(rendered.contains("Unavailable"));
            assert!(!rendered.contains("1,200"));
        }
    }

    #[test]
    fn unauthenticated_and_unauthorized_decode_to_shared_banner_states() {
        for state in [ADMIN_CREDITS_UNAUTHENTICATED, ADMIN_CREDITS_UNAUTHORIZED] {
            let rendered = html(&with_state(state, Some(projection())));
            assert!(rendered.contains(&format!("data-admin-data-state=\"{state}\"")));
            assert!(rendered.contains("Sign in"));
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
