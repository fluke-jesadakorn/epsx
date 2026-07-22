//! Read-only admin payment-intent inventory.
//!
//! Payment, permission, plan, and financial policy remains owned by the Rust
//! services. This page renders only the canonical admin pay-intent response
//! supplied by the admin BFF. It deliberately exposes no mutation controls and
//! does not derive fiat revenue, plan names, or entitlement state.

use dioxus::prelude::*;
use serde::{Deserialize, Serialize};

use super::super::{PageContext, PageMeta};
use crate::auth::AuthGate;
use crate::components::admin::page_layout::{PageGradient, PageHeader, PageLayout, PageMaxWidth};
use crate::primitives::Icon;

pub const ADMIN_PAYMENTS_DATA_PARAM: &str = "data_admin_payment_intents";
pub const ADMIN_PAYMENTS_STATE_PARAM: &str = "data_admin_payment_intents_state";
pub const ADMIN_PAYMENTS_TAB_PARAM: &str = "admin_payment_intents_tab";
pub const ADMIN_PAYMENTS_PAYER_PARAM: &str = "admin_payment_intents_payer";
pub const ADMIN_PAYMENTS_STATUS_PARAM: &str = "admin_payment_intents_status";
pub const ADMIN_PAYMENTS_LIMIT_PARAM: &str = "admin_payment_intents_limit";
pub const ADMIN_PAYMENTS_OFFSET_PARAM: &str = "admin_payment_intents_offset";

pub const ADMIN_PAYMENTS_READY: &str = "ready";
pub const ADMIN_PAYMENTS_EMPTY: &str = "empty";
pub const ADMIN_PAYMENTS_UNAVAILABLE: &str = "unavailable";
pub const ADMIN_PAYMENTS_MALFORMED: &str = "malformed";

/// Exact service-owned fields returned by `GET /api/v1/admin/pay/intents`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct AdminPaymentIntent {
    pub id: String,
    pub chain_id: String,
    pub payer: String,
    pub payee: String,
    pub amount: String,
    pub token_address: String,
    pub status: String,
    pub escrow_id: Option<String>,
    pub tx_hash: Option<String>,
    pub description: Option<String>,
    pub expires_at: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct AdminPaymentIntentList {
    pub items: Vec<AdminPaymentIntent>,
    pub total: i64,
}

/// Decode and semantically validate the service boundary before any value is
/// presented as an authoritative payment record.
pub fn decode_admin_payment_intent_list(
    value: serde_json::Value,
) -> Option<AdminPaymentIntentList> {
    let payload: AdminPaymentIntentList = serde_json::from_value(value).ok()?;
    let total = usize::try_from(payload.total).ok()?;
    if total < payload.items.len() || payload.items.iter().any(|item| !item.is_well_formed()) {
        return None;
    }
    Some(payload)
}

impl AdminPaymentIntent {
    fn is_well_formed(&self) -> bool {
        [
            self.id.as_str(),
            self.chain_id.as_str(),
            self.payer.as_str(),
            self.payee.as_str(),
            self.amount.as_str(),
            self.token_address.as_str(),
            self.status.as_str(),
            self.created_at.as_str(),
            self.updated_at.as_str(),
        ]
        .iter()
        .all(|value| !value.trim().is_empty())
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct PaymentFilters {
    payer: Option<String>,
    status: Option<String>,
    limit: usize,
    offset: usize,
}

impl PaymentFilters {
    fn from_ctx(ctx: &PageContext) -> Self {
        Self {
            payer: ctx
                .params
                .get(ADMIN_PAYMENTS_PAYER_PARAM)
                .and_then(|value| safe_filter(value, 128)),
            status: ctx
                .params
                .get(ADMIN_PAYMENTS_STATUS_PARAM)
                .and_then(|value| safe_filter(value, 32)),
            limit: ctx
                .params
                .get(ADMIN_PAYMENTS_LIMIT_PARAM)
                .and_then(|value| value.parse().ok())
                .unwrap_or(20)
                .clamp(1, 100),
            offset: ctx
                .params
                .get(ADMIN_PAYMENTS_OFFSET_PARAM)
                .and_then(|value| value.parse().ok())
                .unwrap_or(0),
        }
    }

    fn page_url(&self, offset: usize) -> String {
        let mut pairs = vec![
            "tab=payments".to_string(),
            format!("limit={}", self.limit),
            format!("offset={offset}"),
        ];
        if let Some(payer) = &self.payer {
            pairs.push(format!("payer={payer}"));
        }
        if let Some(status) = &self.status {
            pairs.push(format!("status={status}"));
        }
        format!("/payments?{}", pairs.join("&"))
    }
}

fn safe_filter(value: &str, max_len: usize) -> Option<String> {
    (!value.is_empty()
        && value.len() <= max_len
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b':' | b'.' | b'_' | b'-')))
    .then(|| value.to_string())
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum PaymentLoad {
    Ready(AdminPaymentIntentList),
    Empty,
    Unavailable,
    Malformed,
}

fn payment_load(ctx: &PageContext) -> PaymentLoad {
    let state = ctx
        .params
        .get(ADMIN_PAYMENTS_STATE_PARAM)
        .map(String::as_str);
    match state {
        Some(ADMIN_PAYMENTS_READY) | Some(ADMIN_PAYMENTS_EMPTY) => {
            let Some(raw) = ctx.params.get(ADMIN_PAYMENTS_DATA_PARAM) else {
                return PaymentLoad::Malformed;
            };
            let Some(payload) = serde_json::from_str(raw)
                .ok()
                .and_then(decode_admin_payment_intent_list)
            else {
                return PaymentLoad::Malformed;
            };
            match (state, payload.items.is_empty(), payload.total) {
                (Some(ADMIN_PAYMENTS_READY), false, _) => PaymentLoad::Ready(payload),
                (Some(ADMIN_PAYMENTS_READY), true, total) if total > 0 => {
                    PaymentLoad::Ready(payload)
                }
                (Some(ADMIN_PAYMENTS_EMPTY), true, 0) => PaymentLoad::Empty,
                _ => PaymentLoad::Malformed,
            }
        }
        Some(ADMIN_PAYMENTS_MALFORMED) => PaymentLoad::Malformed,
        Some(ADMIN_PAYMENTS_UNAVAILABLE) | None => PaymentLoad::Unavailable,
        Some(_) => PaymentLoad::Malformed,
    }
}

pub fn render(ctx: &PageContext) -> (PageMeta, Element) {
    let meta = PageMeta::admin("Payments");
    (
        meta,
        rsx! {
            AuthGate {
                user: ctx.user.clone(),
                feature: Some("payment intent visibility".to_string()),
                required_permissions: Some(vec!["admin:payments:view".to_string()]),
                return_url: Some(ctx.path.clone()),
                RenderPaymentsHub { ctx: ctx.clone() }
            }
        },
    )
}

#[component]
fn RenderPaymentsHub(ctx: PageContext) -> Element {
    let active_tab = ctx
        .params
        .get(ADMIN_PAYMENTS_TAB_PARAM)
        .map(String::as_str)
        .unwrap_or("payments");
    let active_tab = match active_tab {
        "user-access" => "user-access",
        "payment-links" => "payment-links",
        _ => "payments",
    };
    let filters = PaymentFilters::from_ctx(&ctx);
    let refresh_url = filters.page_url(filters.offset);

    rsx! {
        PageLayout {
            max_width: Some(PageMaxWidth::SevenXl),
            PageHeader {
                title: "Payments Hub".to_string(),
                subtitle: Some("Review backend-authoritative payment intents".to_string()),
                icon: Some("credit-card".to_string()),
                gradient: Some(PageGradient::Primary),
                centered: Some(true),
                extra_actions: Some(rsx! {
                    a { class: "btn btn-sm btn-outline", href: refresh_url.clone(),
                        Icon { name: "refresh-cw".to_string(), size: Some(14) }
                        " Refresh"
                    }
                }),
                class_name: None,
            }
            PaymentsHubTabs { active: active_tab.to_string() }
            if active_tab == "payments" {
                PaymentsTab { load: payment_load(&ctx), filters }
            } else if active_tab == "user-access" {
                UnavailableTab {
                    title: "User access is unavailable".to_string(),
                    detail: "A canonical subscription/access read contract is not connected to this page yet.".to_string(),
                }
            } else {
                UnavailableTab {
                    title: "Payment links are unavailable".to_string(),
                    detail: "Payment-link reads and mutations remain disabled until their service contract is production-ready.".to_string(),
                }
            }
        }
    }
}

#[component]
fn PaymentsTab(load: PaymentLoad, filters: PaymentFilters) -> Element {
    rsx! {
        div { class: "space-y-6",
            PaymentFilterForm { filters: filters.clone() }
            match load {
                PaymentLoad::Ready(payload) => rsx! {
                    PaymentIntentList { payload, filters: filters.clone() }
                },
                PaymentLoad::Empty => rsx! {
                    section { class: "rounded-2xl border border-border/30 bg-card p-10 text-center", role: "status",
                        h2 { class: "text-lg font-semibold", "No payment intents found" }
                        p { class: "mt-2 text-sm text-muted-foreground", "No authoritative payment intents match the current filters." }
                    }
                },
                PaymentLoad::Unavailable => rsx! {
                    LoadProblem {
                        title: "Payment intents unavailable".to_string(),
                        detail: "The payment service could not provide an authoritative response. No records are being shown.".to_string(),
                        retry_url: filters.page_url(filters.offset),
                    }
                },
                PaymentLoad::Malformed => rsx! {
                    LoadProblem {
                        title: "Payment data could not be verified".to_string(),
                        detail: "The service response did not match the payment-intent contract. No records are being shown.".to_string(),
                        retry_url: filters.page_url(filters.offset),
                    }
                },
            }
        }
    }
}

#[component]
fn PaymentFilterForm(filters: PaymentFilters) -> Element {
    let payer = filters.payer.clone().unwrap_or_default();
    let status = filters.status.clone().unwrap_or_default();
    rsx! {
        form { class: "payments-filter-panel rounded-xl border border-border/20 bg-card p-4", method: "GET", action: "/payments",
            input { r#type: "hidden", name: "tab", value: "payments" }
            input { r#type: "hidden", name: "offset", value: "0" }
            div { class: "grid grid-cols-1 gap-4 md:grid-cols-4 md:items-end",
                label { class: "space-y-2 text-sm font-medium",
                    span { "Payer" }
                    input { class: "input w-full font-mono", r#type: "text", name: "payer", value: payer, maxlength: "128", placeholder: "Wallet address" }
                }
                label { class: "space-y-2 text-sm font-medium",
                    span { "Status" }
                    input { class: "input w-full", r#type: "text", name: "status", value: status, maxlength: "32", placeholder: "All statuses" }
                }
                label { class: "space-y-2 text-sm font-medium",
                    span { "Rows per page" }
                    select { class: "input w-full", name: "limit",
                        for value in [10usize, 20, 50, 100] {
                            option { value: "{value}", selected: filters.limit == value, "{value}" }
                        }
                    }
                }
                button { class: "btn btn-primary", r#type: "submit", "Apply filters" }
            }
        }
    }
}

#[component]
fn PaymentIntentList(payload: AdminPaymentIntentList, filters: PaymentFilters) -> Element {
    let total = usize::try_from(payload.total).unwrap_or(0);
    let current_page = filters.offset / filters.limit + 1;
    let total_pages = total.div_ceil(filters.limit).max(1);
    let previous_offset = filters.offset.saturating_sub(filters.limit);
    let next_offset = filters.offset.saturating_add(filters.limit);
    let has_previous = filters.offset > 0;
    let has_next = next_offset < total;

    rsx! {
        section { class: "payment-intents-list rounded-2xl border border-border/20 overflow-hidden bg-card",
            div { class: "h-[3px] bg-gradient-to-r from-[#1fc7d4] to-[#7645d9]" }
            div { class: "flex flex-wrap items-center justify-between gap-3 p-6",
                div {
                    h2 { class: "text-lg font-semibold", "Payment intents" }
                    p { class: "text-sm text-muted-foreground", "{payload.total} authoritative records" }
                }
                p { class: "text-sm text-muted-foreground", "Page {current_page} of {total_pages}" }
            }
            if payload.items.is_empty() {
                div { class: "border-t border-border/30 p-8 text-center", role: "status",
                    h3 { class: "font-semibold", "No payment intents on this page" }
                    p { class: "mt-2 text-sm text-muted-foreground", "The filtered inventory still contains records. Return to the first page or use Previous." }
                    a { class: "btn btn-sm btn-outline mt-4", href: filters.page_url(0), "Return to first page" }
                }
            } else {
                div { class: "hidden overflow-x-auto md:block",
                    table { class: "min-w-full",
                        caption { class: "sr-only", "Backend-authoritative payment intents" }
                        thead {
                            tr { class: "border-y border-border/30",
                                th { class: "px-4 py-3 text-left text-xs uppercase text-muted-foreground", scope: "col", "Intent" }
                                th { class: "px-4 py-3 text-left text-xs uppercase text-muted-foreground", scope: "col", "Payer / payee" }
                                th { class: "px-4 py-3 text-left text-xs uppercase text-muted-foreground", scope: "col", "Amount / token" }
                                th { class: "px-4 py-3 text-left text-xs uppercase text-muted-foreground", scope: "col", "Chain" }
                                th { class: "px-4 py-3 text-left text-xs uppercase text-muted-foreground", scope: "col", "Status" }
                                th { class: "px-4 py-3 text-left text-xs uppercase text-muted-foreground", scope: "col", "Created" }
                            }
                        }
                        tbody { class: "divide-y divide-border/30",
                            for intent in payload.items.iter() {
                                PaymentIntentRow { intent: intent.clone() }
                            }
                        }
                    }
                }
                div { class: "space-y-3 p-4 md:hidden",
                    for intent in payload.items.iter() {
                        PaymentIntentCard { intent: intent.clone() }
                    }
                }
            }
            nav { class: "flex items-center justify-between border-t border-border/30 p-4", aria_label: "Payment intent pagination",
                if has_previous {
                    a { class: "btn btn-sm btn-outline", href: filters.page_url(previous_offset), rel: "prev", "Previous" }
                } else {
                    span { class: "btn btn-sm btn-outline opacity-50", aria_disabled: "true", "Previous" }
                }
                if has_next {
                    a { class: "btn btn-sm btn-outline", href: filters.page_url(next_offset), rel: "next", "Next" }
                } else {
                    span { class: "btn btn-sm btn-outline opacity-50", aria_disabled: "true", "Next" }
                }
            }
        }
    }
}

#[component]
fn PaymentIntentRow(intent: AdminPaymentIntent) -> Element {
    rsx! {
        tr {
            td { class: "px-4 py-4 align-top",
                code { class: "text-xs", "{intent.id}" }
                if let Some(hash) = &intent.tx_hash {
                    p { class: "mt-1 font-mono text-xs text-muted-foreground", "Tx {hash}" }
                }
            }
            td { class: "px-4 py-4 align-top font-mono text-xs",
                p { "From {intent.payer}" }
                p { class: "mt-1 text-muted-foreground", "To {intent.payee}" }
            }
            td { class: "px-4 py-4 align-top",
                p { class: "font-semibold", "{intent.amount}" }
                code { class: "text-xs text-muted-foreground", "{intent.token_address}" }
            }
            td { class: "px-4 py-4 align-top", "{intent.chain_id}" }
            td { class: "px-4 py-4 align-top", StatusBadge { status: intent.status.clone() } }
            td { class: "px-4 py-4 align-top text-sm text-muted-foreground", "{intent.created_at}" }
        }
    }
}

#[component]
fn PaymentIntentCard(intent: AdminPaymentIntent) -> Element {
    rsx! {
        article { class: "rounded-xl border border-border/30 p-4",
            div { class: "flex items-start justify-between gap-3",
                code { class: "break-all text-xs", "{intent.id}" }
                StatusBadge { status: intent.status.clone() }
            }
            dl { class: "mt-4 grid grid-cols-[auto_1fr] gap-x-3 gap-y-2 text-sm",
                dt { class: "text-muted-foreground", "Payer" }
                dd { class: "break-all font-mono text-xs", "{intent.payer}" }
                dt { class: "text-muted-foreground", "Payee" }
                dd { class: "break-all font-mono text-xs", "{intent.payee}" }
                dt { class: "text-muted-foreground", "Amount" }
                dd { "{intent.amount}" }
                dt { class: "text-muted-foreground", "Token" }
                dd { class: "break-all font-mono text-xs", "{intent.token_address}" }
                dt { class: "text-muted-foreground", "Chain" }
                dd { "{intent.chain_id}" }
                dt { class: "text-muted-foreground", "Created" }
                dd { "{intent.created_at}" }
            }
        }
    }
}

#[component]
fn StatusBadge(status: String) -> Element {
    rsx! {
        span { class: "inline-flex rounded-full border border-border/40 bg-muted/40 px-2.5 py-1 text-xs font-semibold", "{status}" }
    }
}

#[component]
fn LoadProblem(title: String, detail: String, retry_url: String) -> Element {
    rsx! {
        section { class: "rounded-2xl border border-amber-500/30 bg-amber-500/5 p-8", role: "alert",
            h2 { class: "text-lg font-semibold", "{title}" }
            p { class: "mt-2 text-sm text-muted-foreground", "{detail}" }
            a { class: "btn btn-sm btn-outline mt-5", href: retry_url, "Try again" }
        }
    }
}

#[component]
fn UnavailableTab(title: String, detail: String) -> Element {
    rsx! {
        section { class: "rounded-2xl border border-border/30 bg-card p-10", role: "status",
            h2 { class: "text-lg font-semibold", "{title}" }
            p { class: "mt-2 max-w-2xl text-sm text-muted-foreground", "{detail}" }
        }
    }
}

#[component]
fn PaymentsHubTabs(active: String) -> Element {
    rsx! {
        nav { class: "flex flex-wrap gap-2 mb-6", aria_label: "Payments sections",
            a { class: if active == "payments" { "btn btn-primary btn-sm" } else { "btn btn-outline btn-sm" }, href: "/payments?tab=payments", "Payments" }
            a { class: if active == "user-access" { "btn btn-primary btn-sm" } else { "btn btn-outline btn-sm" }, href: "/payments?tab=user-access", "User Access" }
            a { class: if active == "payment-links" { "btn btn-primary btn-sm" } else { "btn btn-outline btn-sm" }, href: "/payments?tab=payment-links", "Payment Links" }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::auth::{user::AuthMethod, User};

    fn authed_ctx() -> PageContext {
        PageContext {
            user: Some(User {
                id: "admin-1".to_string(),
                address: "0xadmin".to_string(),
                chain_id: "56".to_string(),
                roles: vec![],
                email: None,
                tier: None,
                permissions: vec!["admin:payments:view".to_string()],
                last_login_at: None,
                auth_method: AuthMethod::Wallet,
                display_name: None,
            }),
            path: "/payments".to_string(),
            ..Default::default()
        }
    }

    fn intent(id: &str) -> AdminPaymentIntent {
        AdminPaymentIntent {
            id: id.to_string(),
            chain_id: "56".to_string(),
            payer: "0x1111111111111111111111111111111111111111".to_string(),
            payee: "0x2222222222222222222222222222222222222222".to_string(),
            amount: "1000000000000000000".to_string(),
            token_address: "0x3333333333333333333333333333333333333333".to_string(),
            status: "pending".to_string(),
            escrow_id: None,
            tx_hash: Some("0xtransaction".to_string()),
            description: None,
            expires_at: None,
            created_at: "2026-07-22T10:00:00Z".to_string(),
            updated_at: "2026-07-22T10:00:00Z".to_string(),
        }
    }

    fn with_load(state: &str, payload: Option<AdminPaymentIntentList>) -> PageContext {
        let mut ctx = authed_ctx();
        ctx.params
            .insert(ADMIN_PAYMENTS_STATE_PARAM.to_string(), state.to_string());
        if let Some(payload) = payload {
            ctx.params.insert(
                ADMIN_PAYMENTS_DATA_PARAM.to_string(),
                serde_json::to_string(&payload).unwrap(),
            );
        }
        ctx.params
            .insert(ADMIN_PAYMENTS_LIMIT_PARAM.to_string(), "20".to_string());
        ctx.params
            .insert(ADMIN_PAYMENTS_OFFSET_PARAM.to_string(), "0".to_string());
        ctx
    }

    fn render_html(ctx: &PageContext) -> String {
        let (_, element) = render(ctx);
        dioxus_ssr::render_element(element)
    }

    #[test]
    fn ready_renders_only_service_owned_fields_and_escapes_text() {
        let mut row = intent("intent-1");
        row.status = "<script>alert(1)</script>".to_string();
        let html = render_html(&with_load(
            ADMIN_PAYMENTS_READY,
            Some(AdminPaymentIntentList {
                items: vec![row],
                total: 1,
            }),
        ));

        assert!(html.contains("intent-1"), "rendered HTML: {html:?}");
        assert!(html.contains("1000000000000000000"));
        assert!(html.contains("&#60;script&#62;alert(1)&#60;/script&#62;"));
        assert!(!html.contains("<script>alert(1)</script>"));
        for invented in ["Total Revenue", "$45,231", "pi_abc123", "Pro Plan Monthly"] {
            assert!(
                !html.contains(invented),
                "invented value leaked: {invented}"
            );
        }
    }

    #[test]
    fn empty_unavailable_and_malformed_are_distinct() {
        let empty = render_html(&with_load(
            ADMIN_PAYMENTS_EMPTY,
            Some(AdminPaymentIntentList {
                items: vec![],
                total: 0,
            }),
        ));
        let unavailable = render_html(&with_load(ADMIN_PAYMENTS_UNAVAILABLE, None));
        let malformed = render_html(&with_load(ADMIN_PAYMENTS_MALFORMED, None));

        assert!(empty.contains("No payment intents found"));
        assert!(unavailable.contains("Payment intents unavailable"));
        assert!(!unavailable.contains("No payment intents found"));
        assert!(malformed.contains("Payment data could not be verified"));
        assert!(!malformed.contains("No payment intents found"));
    }

    #[test]
    fn inconsistent_state_or_payload_fails_closed_as_malformed() {
        let ctx = with_load(
            ADMIN_PAYMENTS_READY,
            Some(AdminPaymentIntentList {
                items: vec![],
                total: 0,
            }),
        );
        assert!(render_html(&ctx).contains("Payment data could not be verified"));

        let invalid = serde_json::json!({"items": [intent("intent-1")], "total": 0});
        assert!(decode_admin_payment_intent_list(invalid).is_none());
    }

    #[test]
    fn nonzero_total_empty_page_preserves_truth_and_recovery_navigation() {
        let mut ctx = with_load(
            ADMIN_PAYMENTS_READY,
            Some(AdminPaymentIntentList {
                items: vec![],
                total: 41,
            }),
        );
        ctx.params
            .insert(ADMIN_PAYMENTS_OFFSET_PARAM.to_string(), "40".to_string());

        let html = render_html(&ctx);
        assert!(html.contains("41 authoritative records"));
        assert!(html.contains("No payment intents on this page"));
        assert!(html.contains("Return to first page"));
        assert!(html.contains("offset=0"));
        assert!(html.contains("Previous"));
        assert!(!html.contains("No payment intents found"));
    }

    #[test]
    fn view_permission_is_required_without_manage_substitution() {
        let mut denied = with_load(ADMIN_PAYMENTS_UNAVAILABLE, None);
        denied.user.as_mut().unwrap().permissions = vec!["admin:payments:manage".to_string()];
        let html = render_html(&denied);
        assert!(html.contains("Permission required"));
        assert!(html.contains("admin:payments:view"));
        assert!(!html.contains("Payments Hub"));
    }

    #[test]
    fn unavailable_tabs_have_no_mutation_controls_or_sample_records() {
        for (tab, expected) in [
            ("user-access", "User access is unavailable"),
            ("payment-links", "Payment links are unavailable"),
        ] {
            let mut ctx = with_load(ADMIN_PAYMENTS_UNAVAILABLE, None);
            ctx.params
                .insert(ADMIN_PAYMENTS_TAB_PARAM.to_string(), tab.to_string());
            let html = render_html(&ctx);
            assert!(html.contains(expected));
            for forbidden in [
                "Create Payment Link",
                "Revoke link",
                "Pro Plan Monthly",
                "Users with access",
            ] {
                assert!(
                    !html.contains(forbidden),
                    "forbidden control/sample leaked: {forbidden}"
                );
            }
        }
    }

    #[test]
    fn pagination_preserves_only_normalized_filters() {
        let mut ctx = with_load(
            ADMIN_PAYMENTS_READY,
            Some(AdminPaymentIntentList {
                items: vec![intent("intent-1")],
                total: 41,
            }),
        );
        ctx.params.insert(
            ADMIN_PAYMENTS_PAYER_PARAM.to_string(),
            "0x1111111111111111111111111111111111111111".to_string(),
        );
        ctx.params.insert(
            ADMIN_PAYMENTS_STATUS_PARAM.to_string(),
            "pending".to_string(),
        );
        let html = render_html(&ctx);
        assert!(html.contains("offset=20"));
        assert!(html.contains("payer=0x1111111111111111111111111111111111111111"));
        assert!(html.contains("status=pending"));
        assert!(!html.contains("admin:payments:manage"));
    }
}
