//! /wallet-management/access — authenticated, read-only access assignments.
//!
//! The page consumes only a strict redacted projection. Wallet identity,
//! assignment actor, optimistic version, timestamps, correlation IDs, and all
//! assign/revoke operations stay in the backend/BFF boundary.

use chrono::DateTime;
use dioxus::prelude::*;
use serde::{Deserialize, Serialize};

use crate::auth::AuthGate;
use crate::components::admin::data_state_banner::{AdminDataState, AdminDataStateBanner};
use crate::components::admin::page_layout::{PageGradient, PageHeader};
use crate::primitives::Icon;

use super::super::{PageContext, PageMeta};
use super::wallet_hub::WalletManagementHub;

const WALLET_ACCESS_PATH: &str = "/wallet-management/access";
const PLANS_PATH: &str = "/wallet-management/access/plans";
const ADMIN_HOME_PATH: &str = "/";
const MAX_ASSIGNMENTS: usize = 1_000;
const MAX_PLAN_NAME_CHARS: usize = 100;
const MAX_PERMISSION_CHARS: usize = 128;
const MAX_TIMESTAMP_CHARS: usize = 64;

pub const ADMIN_ACCESS_DATA_PARAM: &str = "data_admin_access";
pub const ADMIN_ACCESS_STATE_PARAM: &str = "data_admin_access_state";

pub const ADMIN_ACCESS_READY: &str = "ready";
pub const ADMIN_ACCESS_FORBIDDEN: &str = "forbidden";
pub const ADMIN_ACCESS_UNAVAILABLE: &str = "unavailable";
pub const ADMIN_ACCESS_MALFORMED: &str = "malformed";
pub const ADMIN_ACCESS_UNAUTHENTICATED: &str = "unauthenticated";
pub const ADMIN_ACCESS_UNAUTHORIZED: &str = "unauthorized";

/// Backend-owned fields needed to target a versioned assignment mutation.
/// Actor and update timestamps remain excluded from page state.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AdminAccessAssignmentProjection {
    pub wallet_address: String,
    pub plan_id: String,
    pub plan_name: String,
    pub permission: String,
    pub expires_at: Option<String>,
    pub version: i64,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AdminAccessProjection {
    pub items: Vec<AdminAccessAssignmentProjection>,
}

pub fn decode_admin_access_projection(value: serde_json::Value) -> Option<AdminAccessProjection> {
    let projection: AdminAccessProjection = serde_json::from_value(value).ok()?;
    if projection.items.len() > MAX_ASSIGNMENTS
        || projection.items.iter().any(|item| !item.is_well_formed())
    {
        return None;
    }
    Some(projection)
}

impl AdminAccessAssignmentProjection {
    fn is_well_formed(&self) -> bool {
        valid_wallet(&self.wallet_address)
            && valid_uuid(&self.plan_id)
            && valid_text(&self.plan_name, MAX_PLAN_NAME_CHARS)
            && valid_permission(&self.permission)
            && self.version >= 0
            && self.expires_at.as_deref().is_none_or(valid_timestamp)
    }
}

fn valid_wallet(value: &str) -> bool {
    value.len() == 42
        && value.starts_with("0x")
        && value[2..].bytes().all(|byte| byte.is_ascii_hexdigit())
}

fn valid_text(value: &str, max_chars: usize) -> bool {
    !value.trim().is_empty()
        && value.trim() == value
        && value.chars().count() <= max_chars
        && !value.chars().any(char::is_control)
}

fn valid_permission(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= MAX_PERMISSION_CHARS
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b':' | b'_' | b'-'))
}

fn valid_timestamp(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= MAX_TIMESTAMP_CHARS
        && !value.chars().any(char::is_control)
        && DateTime::parse_from_rfc3339(value).is_ok()
}

pub(crate) fn valid_uuid(value: &str) -> bool {
    let bytes = value.as_bytes();
    bytes.len() == 36
        && bytes.iter().enumerate().all(|(index, byte)| {
            matches!(index, 8 | 13 | 18 | 23)
                .then_some(*byte == b'-')
                .unwrap_or_else(|| byte.is_ascii_hexdigit())
        })
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum AccessLoad {
    Ready(AdminAccessProjection),
    Forbidden,
    Unavailable,
    Malformed,
    Unauthenticated,
    Unauthorized,
}

fn access_load(ctx: &PageContext) -> AccessLoad {
    match ctx.params.get(ADMIN_ACCESS_STATE_PARAM).map(String::as_str) {
        Some(ADMIN_ACCESS_READY) => {
            let Some(raw) = ctx.params.get(ADMIN_ACCESS_DATA_PARAM) else {
                return AccessLoad::Malformed;
            };
            serde_json::from_str(raw)
                .ok()
                .and_then(decode_admin_access_projection)
                .map(AccessLoad::Ready)
                .unwrap_or(AccessLoad::Malformed)
        }
        Some(ADMIN_ACCESS_FORBIDDEN) => AccessLoad::Forbidden,
        Some(ADMIN_ACCESS_MALFORMED) => AccessLoad::Malformed,
        Some(ADMIN_ACCESS_UNAUTHENTICATED) => AccessLoad::Unauthenticated,
        Some(ADMIN_ACCESS_UNAUTHORIZED) => AccessLoad::Unauthorized,
        Some(ADMIN_ACCESS_UNAVAILABLE) | None => AccessLoad::Unavailable,
        Some(_) => AccessLoad::Malformed,
    }
}

pub fn render(ctx: &PageContext) -> (PageMeta, Element) {
    let meta = PageMeta::admin("Wallet access");
    (
        meta,
        rsx! {
            AuthGate {
                user: ctx.user.clone(),
                feature: Some("the private wallet access workspace".to_string()),
                return_url: Some(WALLET_ACCESS_PATH.to_string()),
                RenderWalletAccess { ctx: ctx.clone() }
            }
        },
    )
}

#[component]
fn RenderWalletAccess(ctx: PageContext) -> Element {
    let load = access_load(&ctx);

    rsx! {
        WalletManagementHub {
            ctx: ctx.clone(),
            PageHeader {
                title: "Wallet access".to_string(),
                subtitle: Some("Review backend-authoritative access assignments".to_string()),
                icon: Some("shield".to_string()),
                gradient: Some(PageGradient::Purple),
                centered: Some(false),
                extra_actions: None,
                class_name: None,
            }
            match load {
                AccessLoad::Ready(projection) => rsx! { AccessReady { projection } },
                AccessLoad::Forbidden => rsx! {
                    AccessProblem {
                        state: ADMIN_ACCESS_FORBIDDEN,
                        title: "Wallet access was denied".to_string(),
                        detail: "The backend did not authorize this session to read access assignments.".to_string(),
                    }
                },
                AccessLoad::Unavailable => rsx! {
                    AccessProblem {
                        state: ADMIN_ACCESS_UNAVAILABLE,
                        title: "Wallet access is unavailable".to_string(),
                        detail: "The subscription backend could not provide an authoritative assignment response. No assignments are being shown.".to_string(),
                    }
                },
                AccessLoad::Malformed => rsx! {
                    AccessProblem {
                        state: ADMIN_ACCESS_MALFORMED,
                        title: "Wallet access data could not be verified".to_string(),
                        detail: "The backend response did not match the strict redacted access contract. No assignments are being shown.".to_string(),
                    }
                },
                AccessLoad::Unauthenticated => rsx! {
                    AdminDataStateBanner {
                        state: AdminDataState::Unauthenticated,
                        subject: "Wallet access".to_string(),
                        return_path: WALLET_ACCESS_PATH.to_string(),
                        retry_href: WALLET_ACCESS_PATH.to_string(),
                    }
                },
                AccessLoad::Unauthorized => rsx! {
                    AdminDataStateBanner {
                        state: AdminDataState::Unauthorized,
                        subject: "Wallet access".to_string(),
                        return_path: WALLET_ACCESS_PATH.to_string(),
                        retry_href: WALLET_ACCESS_PATH.to_string(),
                    }
                },
            }
        }
    }
}

#[component]
fn AccessReady(projection: AdminAccessProjection) -> Element {
    if projection.items.is_empty() {
        return rsx! {
            section {
                class: "rounded-2xl border border-border/30 bg-card p-10 text-center shadow-xl",
                role: "status",
                "data-admin-wallet-access-state": ADMIN_ACCESS_READY,
                h2 { class: "text-xl font-semibold text-foreground", "No access assignments returned" }
                p { class: "mx-auto mt-2 max-w-xl text-sm leading-6 text-muted-foreground",
                    "The backend returned an authoritative empty assignment projection."
                }
                a { class: "btn btn-outline mt-5", href: PLANS_PATH, "Review plan definitions" }
            }
        };
    }

    rsx! {
        section {
            class: "overflow-hidden rounded-2xl border border-border/30 bg-card shadow-xl",
            aria_labelledby: "admin-wallet-access-title",
            "data-admin-wallet-access-state": ADMIN_ACCESS_READY,
            div { class: "h-1 bg-gradient-to-r from-[#7645d9] via-[#1fc7d4] to-[#ed4b9e]", aria_hidden: "true" }
            div { class: "p-5 sm:p-6",
                h2 { id: "admin-wallet-access-title", class: "text-lg font-semibold text-foreground", "Access assignments" }
                p { class: "mt-1 text-sm leading-6 text-muted-foreground",
                    "Assignments are backend-authoritative. New assignments require the backend version and are audited there."
                }
                form { method: "post", action: WALLET_ACCESS_PATH, class: "mt-5 grid gap-3 rounded-xl border border-border/20 bg-background/30 p-4 md:grid-cols-5",
                    input { r#type: "hidden", name: "operation", value: "access_assign" }
                    input { r#type: "hidden", name: "idempotency_key", value: format!("admin.access.assign.{}", uuid::Uuid::new_v4()) }
                    input { class: "input input-bordered", name: "wallet_address", maxlength: 42, placeholder: "Wallet 0x...", required: true }
                    input { class: "input input-bordered", name: "plan_id", maxlength: 36, placeholder: "Plan UUID", required: true }
                    input { class: "input input-bordered", name: "permission", maxlength: 128, placeholder: "Permission", required: true }
                    input { class: "input input-bordered", name: "expected_version", r#type: "number", min: 0, placeholder: "Version", required: true }
                    button { r#type: "submit", class: "btn btn-primary", "Assign access" }
                }
            }
            ul { class: "divide-y divide-border/30 border-t border-border/30", aria_label: "Wallet access assignments",
                for item in projection.items {
                    AccessAssignmentRow { item }
                }
            }
        }
    }
}

#[component]
fn AccessAssignmentRow(item: AdminAccessAssignmentProjection) -> Element {
    let expiry = item
        .expires_at
        .unwrap_or_else(|| "No expiration reported".to_string());

    rsx! {
        li { class: "grid gap-4 p-5 sm:grid-cols-[minmax(0,1fr)_minmax(0,1fr)_auto] sm:items-center",
            div {
                p { class: "text-xs font-medium uppercase tracking-wide text-muted-foreground", "Plan" }
                p { class: "mt-1 break-words font-semibold text-foreground", "{item.plan_name}" }
                p { class: "mt-1 break-all text-xs text-muted-foreground", "Wallet {item.wallet_address} · Plan {item.plan_id}" }
            }
            div {
                p { class: "text-xs font-medium uppercase tracking-wide text-muted-foreground", "Permission" }
                p { class: "mt-1 break-all font-mono text-sm text-foreground", "{item.permission}" }
            }
            div { class: "sm:text-right",
                p { class: "text-xs font-medium uppercase tracking-wide text-muted-foreground", "Expires" }
                p { class: "mt-1 text-sm text-foreground", "{expiry}" }
            }
            div { class: "flex flex-wrap gap-2 sm:justify-end",
                form { method: "post", action: WALLET_ACCESS_PATH,
                    input { r#type: "hidden", name: "operation", value: "access_revoke" }
                    input { r#type: "hidden", name: "wallet_address", value: item.wallet_address.clone() }
                    input { r#type: "hidden", name: "plan_id", value: item.plan_id.clone() }
                    input { r#type: "hidden", name: "permission", value: item.permission.clone() }
                    input { r#type: "hidden", name: "expected_version", value: item.version.to_string() }
                    input { r#type: "hidden", name: "idempotency_key", value: format!("admin.access.revoke.{}", uuid::Uuid::new_v4()) }
                    button { r#type: "submit", class: "btn btn-sm btn-outline", "Revoke assignment" }
                }
            }
        }
    }
}

#[component]
fn AccessProblem(state: &'static str, title: String, detail: String) -> Element {
    rsx! {
        div {
            "data-admin-wallet-access-state": state,
            section {
                class: "rounded-xl border border-amber-500/25 bg-amber-500/10 px-5 py-4",
                role: if state == ADMIN_ACCESS_FORBIDDEN { "alert" } else { "status" },
                aria_labelledby: "admin-wallet-access-problem-title",
                div { class: "flex flex-col gap-4 sm:flex-row sm:items-center sm:justify-between",
                    div { class: "min-w-0",
                        h2 { id: "admin-wallet-access-problem-title", class: "font-semibold text-foreground", "{title}" }
                        p { class: "mt-1 max-w-3xl text-sm text-muted-foreground", "{detail}" }
                    }
                    nav { class: "flex shrink-0 flex-wrap gap-2", aria_label: "Wallet access recovery",
                        a { class: "btn btn-sm btn-outline", href: WALLET_ACCESS_PATH, "Retry access read" }
                        a { class: "btn btn-sm btn-ghost", href: ADMIN_HOME_PATH, "Admin home" }
                    }
                }
            }
            section { class: "mt-6 overflow-hidden rounded-2xl border border-border/20 bg-card shadow-xl", aria_label: "Wallet access assignments unavailable",
                div { class: "h-[3px] bg-gradient-to-r from-[#7645d9] to-[#1fc7d4]", aria_hidden: "true" }
                div { class: "flex flex-col gap-4 p-6 sm:flex-row sm:items-center sm:justify-between",
                    div {
                        h3 { class: "text-xs font-bold uppercase tracking-[0.2em] text-[#7645d9]", "Access Assignments" }
                        p { class: "mt-2 text-sm text-muted-foreground", "No verified assignment inventory is available." }
                    }
                    a { class: "btn btn-sm btn-outline", href: PLANS_PATH,
                        Icon { name: "layers".to_string(), size: Some(15) }
                        " Review plan definitions"
                    }
                }
                div { class: "border-t border-border/30 px-6 py-12 text-center",
                    Icon { name: "shield".to_string(), size: Some(34) }
                    p { class: "mt-3 font-mono text-sm text-amber-400", "Assignments unavailable" }
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
                id: "access-session".to_string(),
                address: "0xsession".to_string(),
                chain_id: "56".to_string(),
                auth_method: AuthMethod::Wallet,
                ..Default::default()
            }),
            path: WALLET_ACCESS_PATH.to_string(),
            ..Default::default()
        }
    }

    fn html(ctx: &PageContext) -> String {
        dioxus_ssr::render_element(render(ctx).1)
    }

    fn projection() -> serde_json::Value {
        serde_json::json!({
            "items": [{
                "wallet_address": "0x1234567890abcdef1234567890abcdef12345678",
                "plan_id": "00000000-0000-0000-0000-000000000001",
                "plan_name": "Pro",
                "permission": "admin:payments:view",
                "expires_at": "2026-12-31T00:00:00Z",
                "version": 2,
            }]
        })
    }

    fn with_state(state: &str, data: Option<serde_json::Value>) -> PageContext {
        let mut ctx = signed_in();
        ctx.params
            .insert(ADMIN_ACCESS_STATE_PARAM.to_string(), state.to_string());
        if let Some(data) = data {
            ctx.params
                .insert(ADMIN_ACCESS_DATA_PARAM.to_string(), data.to_string());
        }
        ctx
    }

    #[test]
    fn strict_access_projection_redacts_actor_wallet_version_and_unknown_fields() {
        assert!(decode_admin_access_projection(projection()).is_some());
        assert!(decode_admin_access_projection(serde_json::json!({
            "items": [{
                "plan_id": "00000000-0000-0000-0000-000000000001",
                "plan_name": "Pro",
                "permission": "admin:payments:view",
                "expires_at": null,
                "wallet_address": "0xprivate",
                "assigned_by": "operator",
                "version": 7,
            }]
        }))
        .is_none());
        assert!(decode_admin_access_projection(serde_json::json!({
            "items": [{
                "plan_id": "not-a-uuid",
                "plan_name": "Pro",
                "permission": "admin:payments:view",
                "expires_at": null,
            }]
        }))
        .is_none());
    }

    #[test]
    fn ready_access_projection_has_versioned_revoke_control() {
        let rendered = html(&with_state(ADMIN_ACCESS_READY, Some(projection())));
        assert!(rendered.contains("data-admin-wallet-access-state=\"ready\""));
        assert!(rendered.contains("admin:payments:view"));
        assert!(rendered.contains("Wallet 0x1234567890abcdef1234567890abcdef12345678"));
        assert!(rendered.contains("Revoke assignment"));
        assert!(rendered.contains("value=\"access_assign\""));
        assert!(rendered.contains("value=\"access_revoke\""));
        assert!(rendered.contains("expected_version"));
        assert!(rendered.contains("value=\"2\""));
        assert!(!rendered.contains("value=\"2/\""));
        assert!(!rendered.contains("onclick="));
    }

    #[test]
    fn empty_and_error_states_are_truthful_and_hide_stale_rows() {
        let empty = html(&with_state(
            ADMIN_ACCESS_READY,
            Some(serde_json::json!({ "items": [] })),
        ));
        assert!(empty.contains("No access assignments returned"));
        for (state, title) in [
            (ADMIN_ACCESS_FORBIDDEN, "Wallet access was denied"),
            (ADMIN_ACCESS_UNAVAILABLE, "Wallet access is unavailable"),
            (
                ADMIN_ACCESS_MALFORMED,
                "Wallet access data could not be verified",
            ),
        ] {
            let rendered = html(&with_state(state, Some(projection())));
            assert!(rendered.contains(&format!("data-admin-wallet-access-state=\"{state}\"")));
            assert!(rendered.contains(title));
            assert!(rendered.contains("Access Assignments"));
            assert!(rendered.contains("Review plan definitions"));
            assert!(rendered.contains("Assignments unavailable"));
            assert!(!rendered.contains("admin:payments:view"));
        }
    }

    #[test]
    fn unauthenticated_and_unauthorized_decode_to_shared_banner_states() {
        for state in [ADMIN_ACCESS_UNAUTHENTICATED, ADMIN_ACCESS_UNAUTHORIZED] {
            let rendered = html(&with_state(state, Some(projection())));
            assert!(rendered.contains(&format!("data-admin-data-state=\"{state}\"")));
            assert!(rendered.contains("Sign in"));
            assert!(!rendered.contains("admin:payments:view"));
        }
    }

    #[test]
    fn signed_out_and_hostile_context_never_leaks_access_state() {
        let mut ctx = with_state(ADMIN_ACCESS_READY, Some(projection()));
        ctx.user = None;
        ctx.query = "action=grant&wallet=private".to_string();
        ctx.params
            .insert("legacy".to_string(), "private-plan".to_string());
        let rendered = html(&ctx);
        assert!(rendered.contains("Sign in required"));
        assert!(!rendered.contains("data-admin-wallet-access-state"));
        assert!(!rendered.contains("admin:payments:view"));
        assert!(!rendered.contains("private-plan"));
    }
}
