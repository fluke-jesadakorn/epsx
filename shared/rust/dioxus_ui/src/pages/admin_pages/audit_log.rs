//! `/audit-log` — authenticated, redacted, read-only audit inventory.
//!
//! The page renders only a strict backend projection. Actor/target identity,
//! network/device data, before/after state, arbitrary metadata, backend detail
//! fields, totals, export, and every mutation remain backend concerns.

use chrono::DateTime;
use dioxus::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

use crate::auth::AuthGate;
use crate::components::admin::data_state_banner::{AdminDataState, AdminDataStateBanner};
use crate::components::admin::page_layout::{PageGradient, PageHeader, PageLayout, PageMaxWidth};
use crate::primitives::Icon;

use super::super::{PageContext, PageMeta};

const AUDIT_PATH: &str = "/audit-log";
const MAX_AUDIT_ITEMS: usize = 20;
const MAX_CURSOR_CHARS: usize = 256;

pub const ADMIN_AUDIT_DATA_PARAM: &str = "data_admin_audit";
pub const ADMIN_AUDIT_STATE_PARAM: &str = "data_admin_audit_state";
pub const ADMIN_AUDIT_CATEGORY_PARAM: &str = "admin_audit_category";
pub const ADMIN_AUDIT_CURSOR_PARAM: &str = "admin_audit_cursor";

pub const ADMIN_AUDIT_READY: &str = "ready";
pub const ADMIN_AUDIT_EMPTY: &str = "empty";
pub const ADMIN_AUDIT_FORBIDDEN: &str = "forbidden";
pub const ADMIN_AUDIT_UNAVAILABLE: &str = "unavailable";
pub const ADMIN_AUDIT_MALFORMED: &str = "malformed";
pub const ADMIN_AUDIT_UNAUTHENTICATED: &str = "unauthenticated";
pub const ADMIN_AUDIT_UNAUTHORIZED: &str = "unauthorized";

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AdminAuditSummary {
    pub id: String,
    pub category: String,
    pub action: String,
    pub resource_type: String,
    pub effect: String,
    pub occurred_at: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AdminAuditList {
    pub items: Vec<AdminAuditSummary>,
    pub next_cursor: Option<String>,
    pub has_more: bool,
}

pub fn decode_admin_audit_projection(value: serde_json::Value) -> Option<AdminAuditList> {
    let projection: AdminAuditList = serde_json::from_value(value).ok()?;
    if projection.items.len() > MAX_AUDIT_ITEMS
        || projection.has_more != projection.next_cursor.is_some()
        || projection.has_more && projection.items.is_empty()
        || projection
            .next_cursor
            .as_deref()
            .is_some_and(|cursor| !valid_cursor(cursor))
    {
        return None;
    }

    let mut ids = HashSet::with_capacity(projection.items.len());
    let mut previous_key: Option<(DateTime<chrono::FixedOffset>, String)> = None;
    for item in &projection.items {
        if !valid_uuid(&item.id)
            || !ids.insert(item.id.clone())
            || !valid_category(&item.category)
            || !bounded_control_free(&item.action, 50)
            || !bounded_control_free(&item.resource_type, 50)
            || !matches!(item.effect.as_str(), "success" | "failure" | "denied")
        {
            return None;
        }
        let occurred_at = DateTime::parse_from_rfc3339(&item.occurred_at).ok()?;
        let key = (occurred_at, item.id.clone());
        if previous_key
            .as_ref()
            .is_some_and(|previous| previous <= &key)
        {
            return None;
        }
        previous_key = Some(key);
    }
    Some(projection)
}

fn valid_uuid(value: &str) -> bool {
    value.len() == 36
        && value.bytes().enumerate().all(|(index, byte)| match index {
            8 | 13 | 18 | 23 => byte == b'-',
            _ => byte.is_ascii_hexdigit(),
        })
}

fn valid_category(value: &str) -> bool {
    matches!(
        value,
        "auth"
            | "developer"
            | "notification"
            | "payment"
            | "permission"
            | "plan"
            | "support"
            | "system"
            | "wallet"
    )
}

fn valid_cursor(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= MAX_CURSOR_CHARS
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_'))
}

fn bounded_control_free(value: &str, max_chars: usize) -> bool {
    !value.trim().is_empty()
        && value.chars().count() <= max_chars
        && !value.chars().any(char::is_control)
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
struct AuditLocation {
    category: Option<String>,
    cursor: Option<String>,
}

impl AuditLocation {
    fn from_ctx(ctx: &PageContext) -> Self {
        Self {
            category: ctx
                .params
                .get(ADMIN_AUDIT_CATEGORY_PARAM)
                .filter(|value| valid_category(value))
                .cloned(),
            cursor: ctx
                .params
                .get(ADMIN_AUDIT_CURSOR_PARAM)
                .filter(|value| valid_cursor(value))
                .cloned(),
        }
    }

    fn href(&self, cursor: Option<&str>) -> String {
        let mut pairs = Vec::with_capacity(2);
        if let Some(category) = &self.category {
            pairs.push(format!("category={category}"));
        }
        if let Some(cursor) = cursor {
            pairs.push(format!("cursor={cursor}"));
        }
        if pairs.is_empty() {
            AUDIT_PATH.to_string()
        } else {
            format!("{AUDIT_PATH}?{}", pairs.join("&"))
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum AuditLoad {
    Ready(AdminAuditList),
    Empty,
    Unauthenticated,
    Unauthorized,
    Forbidden,
    Unavailable,
    Malformed,
}

fn audit_load(ctx: &PageContext) -> AuditLoad {
    let state = ctx.params.get(ADMIN_AUDIT_STATE_PARAM).map(String::as_str);
    match state {
        Some(ADMIN_AUDIT_READY) | Some(ADMIN_AUDIT_EMPTY) => {
            let Some(raw) = ctx.params.get(ADMIN_AUDIT_DATA_PARAM) else {
                return AuditLoad::Malformed;
            };
            let Some(projection) = serde_json::from_str(raw)
                .ok()
                .and_then(decode_admin_audit_projection)
            else {
                return AuditLoad::Malformed;
            };
            match (state, projection.items.is_empty()) {
                (Some(ADMIN_AUDIT_READY), false) => AuditLoad::Ready(projection),
                (Some(ADMIN_AUDIT_EMPTY), true) => AuditLoad::Empty,
                _ => AuditLoad::Malformed,
            }
        }
        Some(ADMIN_AUDIT_FORBIDDEN) => AuditLoad::Forbidden,
        Some(ADMIN_AUDIT_MALFORMED) => AuditLoad::Malformed,
        Some(ADMIN_AUDIT_UNAUTHENTICATED) => AuditLoad::Unauthenticated,
        Some(ADMIN_AUDIT_UNAUTHORIZED) => AuditLoad::Unauthorized,
        Some(ADMIN_AUDIT_UNAVAILABLE) | None => AuditLoad::Unavailable,
        Some(_) => AuditLoad::Malformed,
    }
}

pub fn render(ctx: &PageContext) -> (PageMeta, Element) {
    let meta = PageMeta::admin("Audit log");
    (
        meta,
        rsx! {
            AuthGate {
                user: ctx.user.clone(),
                feature: Some("the private audit workspace".to_string()),
                return_url: Some(AUDIT_PATH.to_string()),
                RenderAuditLog { ctx: ctx.clone() }
            }
        },
    )
}

#[component]
fn RenderAuditLog(ctx: PageContext) -> Element {
    let location = AuditLocation::from_ctx(&ctx);
    let load = audit_load(&ctx);

    rsx! {
        PageLayout {
            max_width: Some(PageMaxWidth::SevenXl),
            PageHeader {
                title: "Audit Log".to_string(),
                subtitle: Some("Track all admin actions, permission changes, and system events".to_string()),
                icon: Some("file-text".to_string()),
                gradient: Some(PageGradient::Indigo),
                centered: Some(false),
                extra_actions: None,
                class_name: None,
            }
            AuditFilters { location: location.clone() }
            match load {
                AuditLoad::Ready(projection) => rsx! {
                    AuditReady { projection, location }
                },
                AuditLoad::Empty => rsx! {
                    AuditEmpty { location }
                },
                AuditLoad::Forbidden => rsx! {
                    AuditProblem {
                        state: ADMIN_AUDIT_FORBIDDEN,
                        title: "Audit access was denied".to_string(),
                        detail: "The backend did not authorize this session to read the redacted audit inventory.".to_string(),
                        retry_href: location.href(location.cursor.as_deref()),
                    }
                },
                AuditLoad::Unauthenticated | AuditLoad::Unauthorized => {
                    let state = if matches!(load, AuditLoad::Unauthenticated) {
                        AdminDataState::Unauthenticated
                    } else {
                        AdminDataState::Unauthorized
                    };
                    rsx! {
                        AdminDataStateBanner {
                            state,
                            subject: "Audit log".to_string(),
                            return_path: AUDIT_PATH.to_string(),
                            retry_href: AUDIT_PATH.to_string(),
                        }
                    }
                }
                AuditLoad::Unavailable => rsx! {
                    AuditProblem {
                        state: ADMIN_AUDIT_UNAVAILABLE,
                        title: "Audit records are unavailable".to_string(),
                        detail: "The audit backend could not provide an authoritative response. No records are being shown.".to_string(),
                        retry_href: location.href(location.cursor.as_deref()),
                    }
                },
                AuditLoad::Malformed => rsx! {
                    AuditProblem {
                        state: ADMIN_AUDIT_MALFORMED,
                        title: "Audit data could not be verified".to_string(),
                        detail: "The backend response did not match the redacted audit contract. No records are being shown.".to_string(),
                        retry_href: location.href(None),
                    }
                },
            }
        }
    }
}

#[component]
fn AuditFilters(location: AuditLocation) -> Element {
    let refresh_href = location.href(None);
    rsx! {
        section { class: "rounded-xl border border-border/20 bg-card p-4 shadow-xl", aria_label: "Audit log filters",
            div { class: "flex flex-col gap-3 lg:flex-row",
                div { class: "relative flex-1",
                    span { class: "pointer-events-none absolute left-3 top-1/2 -translate-y-1/2 text-muted-foreground", aria_hidden: "true",
                        Icon { name: "search".to_string(), size: Some(16) }
                    }
                    input {
                        class: "w-full rounded-xl border border-border/50 bg-muted/30 py-2.5 pl-9 pr-4 text-sm text-muted-foreground",
                        r#type: "text",
                        disabled: true,
                        placeholder: "Search by actor, action, or target...",
                        title: "Search requires an analytics-owned redacted search contract",
                        aria_label: "Audit search unavailable",
                    }
                }
                AuditCategoryNav { selected: location.category.clone() }
            }
            div { class: "mt-3 flex flex-col gap-3 sm:flex-row",
                div { class: "flex flex-1 items-center gap-2",
                    input { class: "min-w-0 flex-1 rounded-xl border border-border/50 bg-muted/30 px-3 py-2 text-sm", r#type: "date", disabled: true, aria_label: "Audit date from unavailable", title: "Date filtering is not exposed by the analytics service yet" }
                    span { class: "text-sm text-muted-foreground", "to" }
                    input { class: "min-w-0 flex-1 rounded-xl border border-border/50 bg-muted/30 px-3 py-2 text-sm", r#type: "date", disabled: true, aria_label: "Audit date to unavailable", title: "Date filtering is not exposed by the analytics service yet" }
                }
                div { class: "flex gap-2",
                    a { class: "btn btn-sm bg-gradient-to-r from-[#7645d9] to-[#5a33b8] text-white", href: refresh_href,
                        Icon { name: "refresh-cw".to_string(), size: Some(15) }
                        " Refresh"
                    }
                    button { class: "btn btn-sm btn-outline", r#type: "button", disabled: true, title: "Export requires a backend-owned redacted export contract",
                        Icon { name: "download".to_string(), size: Some(15) }
                        " Export"
                    }
                }
            }
        }
    }
}

#[component]
fn AuditCategoryNav(selected: Option<String>) -> Element {
    const CATEGORIES: [(&str, &str); 9] = [
        ("auth", "Auth"),
        ("developer", "Developer"),
        ("notification", "Notifications"),
        ("payment", "Payments"),
        ("permission", "Permissions"),
        ("plan", "Plans"),
        ("support", "Support"),
        ("system", "System"),
        ("wallet", "Wallets"),
    ];
    rsx! {
        nav { class: "flex gap-2 overflow-x-auto pb-1 lg:pb-0", aria_label: "Audit category",
            a {
                class: if selected.is_none() { "btn btn-sm btn-primary" } else { "btn btn-sm btn-outline" },
                href: AUDIT_PATH,
                aria_current: selected.is_none().then_some("page"),
                "All activity"
            }
            for (category, label) in CATEGORIES {
                a {
                    class: if selected.as_deref() == Some(category) { "btn btn-sm btn-primary" } else { "btn btn-sm btn-outline" },
                    href: format!("{AUDIT_PATH}?category={category}"),
                    aria_current: (selected.as_deref() == Some(category)).then_some("page"),
                    "{label}"
                }
            }
        }
    }
}

#[component]
fn AuditReady(projection: AdminAuditList, location: AuditLocation) -> Element {
    let next_href = projection
        .next_cursor
        .as_deref()
        .map(|cursor| location.href(Some(cursor)));
    rsx! {
        section {
            class: "overflow-hidden rounded-2xl border border-border/30 bg-card shadow-xl",
            aria_label: "Redacted audit inventory",
            "data-audit-log-state": ADMIN_AUDIT_READY,
            div { class: "h-[3px] bg-gradient-to-r from-[#7645d9] via-[#1fc7d4] to-[#31d0aa]" }
            div { class: "flex flex-wrap items-center justify-between gap-3 p-5",
                div {
                    h2 { class: "text-lg font-semibold text-foreground", "Platform activity" }
                    p { class: "text-sm text-muted-foreground", "Newest verified summaries first" }
                }
                p { class: "text-xs text-muted-foreground", "Sensitive identity and detail fields are redacted" }
            }
            div { class: "hidden grid-cols-12 gap-4 border-t border-border/30 bg-muted/20 px-5 py-3 text-xs font-semibold uppercase tracking-wide text-muted-foreground md:grid", aria_hidden: "true",
                span { class: "col-span-3", "Time" }
                span { class: "col-span-3", "Action" }
                span { class: "col-span-3", "Resource" }
                span { class: "col-span-2", "Category" }
                span { class: "col-span-1 text-right", "Result / Details" }
            }
            ul { class: "divide-y divide-border/30", aria_label: "Audit summaries",
                for item in projection.items {
                    AuditRow { item }
                }
            }
            nav { class: "flex flex-wrap items-center justify-between gap-3 border-t border-border/30 p-4", aria_label: "Audit pagination",
                a { class: "btn btn-sm btn-outline", href: location.href(None), "Return to newest" }
                if let Some(next_href) = next_href {
                    a { class: "btn btn-sm btn-outline", href: next_href, rel: "next", "Older activity" }
                } else {
                    span { class: "btn btn-sm btn-outline opacity-50", aria_disabled: "true", "No older activity" }
                }
            }
        }
    }
}

#[component]
fn AuditRow(item: AdminAuditSummary) -> Element {
    let effect_class = match item.effect.as_str() {
        "success" => "border-green-500/30 bg-green-500/10 text-green-800 dark:text-green-300",
        "failure" => "border-red-500/30 bg-red-500/10 text-red-800 dark:text-red-300",
        _ => "border-amber-500/30 bg-amber-500/10 text-amber-900 dark:text-amber-300",
    };
    let occurred_at = item.occurred_at.clone();
    let action = item.action.clone();
    let resource_type = item.resource_type.clone();
    let category = item.category.clone();
    let effect = item.effect.clone();
    rsx! {
        li {
            details { class: "group",
                summary { class: "cursor-pointer list-none p-5 marker:content-none [&::-webkit-details-marker]:hidden",
                    div { class: "grid gap-3 md:grid-cols-12 md:items-center md:gap-4",
                        div { class: "md:col-span-3",
                            span { class: "sr-only", "Occurred: " }
                            p { class: "text-xs font-medium uppercase tracking-wide text-muted-foreground md:hidden", aria_hidden: "true", "Time" }
                            time { class: "text-sm text-foreground", datetime: occurred_at.clone(), "{occurred_at}" }
                        }
                        div { class: "min-w-0 md:col-span-3",
                            span { class: "sr-only", "Action: " }
                            p { class: "text-xs font-medium uppercase tracking-wide text-muted-foreground md:hidden", aria_hidden: "true", "Action" }
                            p { class: "break-words text-sm font-semibold text-foreground", "{action}" }
                        }
                        div { class: "min-w-0 md:col-span-3",
                            span { class: "sr-only", "Resource: " }
                            p { class: "text-xs font-medium uppercase tracking-wide text-muted-foreground md:hidden", aria_hidden: "true", "Resource" }
                            p { class: "break-words text-sm text-muted-foreground", "{resource_type}" }
                        }
                        div { class: "md:col-span-2",
                            span { class: "sr-only", "Category: " }
                            p { class: "text-xs font-medium uppercase tracking-wide text-muted-foreground md:hidden", aria_hidden: "true", "Category" }
                            span { class: "inline-flex rounded-full border border-primary/20 bg-primary/10 px-2.5 py-1 text-xs font-semibold text-primary", "{category}" }
                        }
                        div { class: "flex items-center justify-between gap-2 md:col-span-1 md:justify-end md:text-right",
                            span { class: "sr-only", "Effect: " }
                            span { class: "inline-flex rounded-full border px-2 py-1 text-xs font-semibold {effect_class}", "{effect}" }
                            span { class: "text-muted-foreground transition-transform group-open:rotate-180", aria_hidden: "true",
                                Icon { name: "chevron-down".to_string(), size: Some(15) }
                            }
                        }
                    }
                }
                div { class: "border-t border-border/30 bg-muted/15 px-5 py-4",
                    div { class: "mb-4 flex items-start gap-3 rounded-xl border border-border/30 bg-card/50 p-3",
                        Icon { name: "shield-check".to_string(), size: Some(18) }
                        div {
                            p { class: "text-sm font-semibold text-foreground", "Redacted event details" }
                            p { class: "mt-1 text-xs text-muted-foreground", "Only the backend-authorized summary is available. Sensitive identity and supplemental event fields remain hidden." }
                        }
                    }
                    dl { class: "grid gap-3 sm:grid-cols-2 lg:grid-cols-4",
                        AuditDetailField { label: "Time".to_string(), value: item.occurred_at }
                        AuditDetailField { label: "Action".to_string(), value: item.action }
                        AuditDetailField { label: "Resource".to_string(), value: item.resource_type }
                        AuditDetailField { label: "Category / result".to_string(), value: format!("{} / {}", item.category, item.effect) }
                    }
                }
            }
        }
    }
}

#[component]
fn AuditDetailField(label: String, value: String) -> Element {
    rsx! {
        div { class: "min-w-0 rounded-lg border border-border/20 bg-background/40 p-3",
            dt { class: "text-xs font-semibold uppercase tracking-wide text-muted-foreground", "{label}" }
            dd { class: "mt-1 break-words text-sm text-foreground", "{value}" }
        }
    }
}

#[component]
fn AuditEmpty(location: AuditLocation) -> Element {
    let filtered = location.category.is_some();
    let continued = location.cursor.is_some();
    rsx! {
        section {
            class: "rounded-2xl border border-border/30 bg-card p-10 text-center shadow-xl",
            role: "status",
            "data-audit-log-state": ADMIN_AUDIT_EMPTY,
            Icon { name: "history".to_string(), size: Some(30) }
            h2 { class: "mt-4 text-xl font-semibold text-foreground", "No audit activity found" }
            p { class: "mx-auto mt-2 max-w-xl text-sm text-muted-foreground",
                if continued {
                    "No older records exist for this continuation."
                } else if filtered {
                    "The selected category has no authoritative audit summaries."
                } else {
                    "The backend authoritatively returned an empty audit inventory."
                }
            }
            if filtered || continued {
                a { class: "btn btn-primary mt-5", href: AUDIT_PATH, "View newest activity" }
            }
        }
    }
}

#[component]
fn AuditProblem(state: &'static str, title: String, detail: String, retry_href: String) -> Element {
    rsx! {
        section {
            class: "rounded-2xl border border-border/30 bg-card p-10 text-center shadow-xl",
            role: if state == ADMIN_AUDIT_FORBIDDEN { "alert" } else { "status" },
            "data-audit-log-state": state,
            Icon { name: "shield".to_string(), size: Some(30) }
            h2 { class: "mt-4 text-xl font-semibold text-foreground", "{title}" }
            p { class: "mx-auto mt-2 max-w-2xl text-sm text-muted-foreground", "{detail}" }
            div { class: "mt-6 flex flex-wrap justify-center gap-3",
                a { class: "btn btn-primary", href: retry_href, "Try again" }
                a { class: "btn btn-outline", href: AUDIT_PATH, "Reset audit view" }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::auth::user::{AuthMethod, User};

    fn signed_in_ctx() -> PageContext {
        PageContext {
            user: Some(User {
                id: "u1".to_string(),
                address: "0x1234".to_string(),
                chain_id: "56".to_string(),
                roles: vec!["user".to_string()],
                email: None,
                tier: None,
                permissions: vec![],
                last_login_at: None,
                auth_method: AuthMethod::Wallet,
                display_name: None,
            }),
            path: AUDIT_PATH.to_string(),
            ..Default::default()
        }
    }

    fn html(ctx: &PageContext) -> String {
        let (_, element) = render(ctx);
        dioxus_ssr::render_element(element)
    }

    fn projection() -> AdminAuditList {
        AdminAuditList {
            items: vec![AdminAuditSummary {
                id: "00000000-0000-0000-0000-000000000002".to_string(),
                category: "system".to_string(),
                action: "settings.updated".to_string(),
                resource_type: "settings".to_string(),
                effect: "success".to_string(),
                occurred_at: "2026-07-22T12:00:00Z".to_string(),
            }],
            next_cursor: Some("cursor_token_2".to_string()),
            has_more: true,
        }
    }

    #[test]
    fn signed_out_route_keeps_audit_state_private() {
        let rendered = html(&PageContext {
            path: AUDIT_PATH.to_string(),
            ..Default::default()
        });
        assert!(rendered.contains("Sign in required"));
        assert!(!rendered.contains("data-audit-log-state"));
        assert!(!rendered.contains("Platform activity"));
    }

    #[test]
    fn ready_state_is_redacted_responsive_and_cursor_driven() {
        let mut ctx = signed_in_ctx();
        ctx.params.insert(
            ADMIN_AUDIT_STATE_PARAM.to_string(),
            ADMIN_AUDIT_READY.to_string(),
        );
        ctx.params.insert(
            ADMIN_AUDIT_DATA_PARAM.to_string(),
            serde_json::to_string(&projection()).unwrap(),
        );
        let rendered = html(&ctx);
        assert!(rendered.contains("data-audit-log-state=\"ready\""));
        assert!(rendered.contains("settings.updated"));
        assert!(rendered.contains("datetime=\"2026-07-22T12:00:00Z\""));
        assert!(rendered.contains("cursor=cursor_token_2"));
        assert!(rendered.contains("<details"));
        assert!(rendered.contains("Redacted event details"));
        assert!(rendered.contains("Audit search unavailable"));
        assert!(rendered.contains("Export"));
        assert!(rendered.contains("disabled"));
        for label in [
            "Occurred: ",
            "Action: ",
            "Resource: ",
            "Category: ",
            "Effect: ",
        ] {
            assert!(rendered.contains(label), "missing accessible {label}");
        }
        for forbidden in [
            "00000000-0000-0000-0000-000000000002",
            "wallet_address",
            "resource_id",
            "ip_address",
            "user_agent",
            "before_state",
            "after_state",
            "metadata",
            "href=\"/audit-log/export",
            "action=\"/audit-log",
        ] {
            assert!(!rendered.contains(forbidden), "leaked {forbidden}");
        }
    }

    #[test]
    fn malformed_or_hostile_projection_fails_closed() {
        let mut ctx = signed_in_ctx();
        ctx.params.insert(
            ADMIN_AUDIT_STATE_PARAM.to_string(),
            ADMIN_AUDIT_READY.to_string(),
        );
        ctx.params.insert(
            ADMIN_AUDIT_DATA_PARAM.to_string(),
            r#"{"items":[{"id":"00000000-0000-0000-0000-000000000001","category":"system","action":"ok","resource_type":"settings","effect":"success","occurred_at":"2026-07-22T12:00:00Z","actor":"0xsecret"}],"next_cursor":null,"has_more":false}"#.to_string(),
        );
        let rendered = html(&ctx);
        assert!(rendered.contains("data-audit-log-state=\"malformed\""));
        assert!(!rendered.contains("0xsecret"));
        assert!(!rendered.contains(">ok<"));
    }

    #[test]
    fn unauthenticated_and_unauthorized_decode_and_render_the_shared_banner() {
        let mut unauthenticated = signed_in_ctx();
        unauthenticated.params.insert(
            ADMIN_AUDIT_STATE_PARAM.to_string(),
            ADMIN_AUDIT_UNAUTHENTICATED.to_string(),
        );
        assert_eq!(audit_load(&unauthenticated), AuditLoad::Unauthenticated);
        let rendered = html(&unauthenticated);
        assert!(rendered.contains("data-admin-data-state=\"unauthenticated\""));
        assert!(rendered.contains("Sign in required"));
        assert!(!rendered.contains("data-audit-log-state"));

        let mut unauthorized = signed_in_ctx();
        unauthorized.params.insert(
            ADMIN_AUDIT_STATE_PARAM.to_string(),
            ADMIN_AUDIT_UNAUTHORIZED.to_string(),
        );
        assert_eq!(audit_load(&unauthorized), AuditLoad::Unauthorized);
        let rendered = html(&unauthorized);
        assert!(rendered.contains("data-admin-data-state=\"unauthorized\""));
        assert!(rendered.contains("Session expired"));
        assert!(!rendered.contains("data-audit-log-state"));
    }

    #[test]
    fn category_navigation_and_problem_recovery_are_native_links() {
        let mut ctx = signed_in_ctx();
        ctx.params.insert(
            ADMIN_AUDIT_STATE_PARAM.to_string(),
            ADMIN_AUDIT_FORBIDDEN.to_string(),
        );
        ctx.params.insert(
            ADMIN_AUDIT_CATEGORY_PARAM.to_string(),
            "permission".to_string(),
        );
        let rendered = html(&ctx);
        assert!(rendered.contains("href=\"/audit-log?category=permission\""));
        assert!(rendered.contains("href=\"/audit-log?category=support\""));
        assert!(rendered.contains("Audit access was denied"));
        assert!(!rendered.contains("onclick="));
        assert!(!rendered.contains("javascript:"));
    }
}
