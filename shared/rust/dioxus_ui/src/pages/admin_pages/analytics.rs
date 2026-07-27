//! Authenticated, backend-projected analytics read state for `/analytics`.
//!
//! This leaf consumes only the strict JSON projection placed in
//! `PageContext.params` by the admin BFF. It never derives entitlement from
//! frontend roles, invents missing metrics, or emits mutation controls.

use dioxus::prelude::*;
use serde::{Deserialize, Serialize};

use crate::auth::AuthGate;
use crate::layout::admin_shell::AdminShell;
use crate::primitives::Icon;

use super::super::{PageContext, PageMeta};

const ANALYTICS_PATH: &str = "/analytics";
const MAX_COUNT: i64 = 9_000_000_000_000_000;

pub const ADMIN_ANALYTICS_DATA_PARAM: &str = "data_admin_analytics";
pub const ADMIN_ANALYTICS_STATE_PARAM: &str = "data_admin_analytics_state";

pub const ADMIN_ANALYTICS_READY: &str = "ready";
pub const ADMIN_ANALYTICS_EMPTY: &str = "empty";
pub const ADMIN_ANALYTICS_FORBIDDEN: &str = "forbidden";
pub const ADMIN_ANALYTICS_UNAVAILABLE: &str = "unavailable";
pub const ADMIN_ANALYTICS_MALFORMED: &str = "malformed";

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AdminAnalyticsUserStats {
    pub total: i64,
    pub active: i64,
    pub today_connections: i64,
    pub total_users: i64,
    pub active_users: i64,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AdminAnalyticsPermissionStats {
    pub total: i64,
    pub total_plans: i64,
    pub total_permissions: i64,
    pub active_permissions: i64,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AdminAnalyticsPlanStats {
    pub total_plans: i64,
    pub active_plans: i64,
    pub total_memberships: i64,
    pub active_memberships: i64,
    pub recent_assignments: i64,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AdminAnalyticsDeveloperStats {
    pub total_api_keys: i64,
    pub active_api_keys: i64,
}

/// A deliberately empty typed placeholder. The current backend does not
/// expose operational telemetry in this contract. A non-null telemetry block
/// is rejected until it has a reviewed, typed source rather than being shown
/// as an unverified health claim.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AdminAnalyticsSystemMetrics {}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AdminAnalyticsSnapshot {
    #[serde(default)]
    pub user_stats: Option<AdminAnalyticsUserStats>,
    #[serde(default)]
    pub permission_analytics: Option<AdminAnalyticsPermissionStats>,
    #[serde(default)]
    pub plan_stats: Option<AdminAnalyticsPlanStats>,
    #[serde(default)]
    pub system_metrics: Option<AdminAnalyticsSystemMetrics>,
    #[serde(default)]
    pub developer_portal: Option<AdminAnalyticsDeveloperStats>,
}

pub fn decode_admin_analytics_projection(
    value: serde_json::Value,
) -> Option<AdminAnalyticsSnapshot> {
    let projection: AdminAnalyticsSnapshot = serde_json::from_value(value).ok()?;
    if projection
        .user_stats
        .as_ref()
        .is_some_and(|stats| !valid_user_stats(stats))
        || projection
            .permission_analytics
            .as_ref()
            .is_some_and(|stats| !valid_permission_stats(stats))
        || projection
            .plan_stats
            .as_ref()
            .is_some_and(|stats| !valid_plan_stats(stats))
        || projection
            .developer_portal
            .as_ref()
            .is_some_and(|stats| !valid_developer_stats(stats))
    {
        return None;
    }
    Some(projection)
}

fn valid_count(value: i64) -> bool {
    (0..=MAX_COUNT).contains(&value)
}

fn valid_user_stats(stats: &AdminAnalyticsUserStats) -> bool {
    [
        stats.total,
        stats.active,
        stats.today_connections,
        stats.total_users,
        stats.active_users,
    ]
    .into_iter()
    .all(valid_count)
        && stats.total == stats.total_users
        && stats.active == stats.active_users
        && stats.active <= stats.total
}

fn valid_permission_stats(stats: &AdminAnalyticsPermissionStats) -> bool {
    [
        stats.total,
        stats.total_plans,
        stats.total_permissions,
        stats.active_permissions,
    ]
    .into_iter()
    .all(valid_count)
        && stats.total == stats.total_permissions
        && stats.active_permissions <= stats.total_permissions
}

fn valid_plan_stats(stats: &AdminAnalyticsPlanStats) -> bool {
    [
        stats.total_plans,
        stats.active_plans,
        stats.total_memberships,
        stats.active_memberships,
        stats.recent_assignments,
    ]
    .into_iter()
    .all(valid_count)
        && stats.active_plans <= stats.total_plans
        && stats.active_memberships <= stats.total_memberships
        && stats.recent_assignments <= stats.total_memberships
}

fn valid_developer_stats(stats: &AdminAnalyticsDeveloperStats) -> bool {
    valid_count(stats.total_api_keys)
        && valid_count(stats.active_api_keys)
        && stats.active_api_keys <= stats.total_api_keys
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum AnalyticsLoad {
    Ready(AdminAnalyticsSnapshot),
    Empty,
    Forbidden,
    Unavailable,
    Malformed,
}

fn analytics_load(ctx: &PageContext) -> AnalyticsLoad {
    let state = ctx
        .params
        .get(ADMIN_ANALYTICS_STATE_PARAM)
        .map(String::as_str);
    match state {
        Some(ADMIN_ANALYTICS_READY) | Some(ADMIN_ANALYTICS_EMPTY) => {
            let Some(raw) = ctx.params.get(ADMIN_ANALYTICS_DATA_PARAM) else {
                return AnalyticsLoad::Malformed;
            };
            let Some(snapshot) = serde_json::from_str(raw)
                .ok()
                .and_then(decode_admin_analytics_projection)
            else {
                return AnalyticsLoad::Malformed;
            };
            let has_data = has_data(&snapshot);
            match (state, has_data) {
                (Some(ADMIN_ANALYTICS_READY), true) => AnalyticsLoad::Ready(snapshot),
                (Some(ADMIN_ANALYTICS_EMPTY), false) => AnalyticsLoad::Empty,
                _ => AnalyticsLoad::Malformed,
            }
        }
        Some(ADMIN_ANALYTICS_FORBIDDEN) => AnalyticsLoad::Forbidden,
        Some(ADMIN_ANALYTICS_MALFORMED) => AnalyticsLoad::Malformed,
        Some(ADMIN_ANALYTICS_UNAVAILABLE) | None => AnalyticsLoad::Unavailable,
        Some(_) => AnalyticsLoad::Malformed,
    }
}

fn has_data(snapshot: &AdminAnalyticsSnapshot) -> bool {
    snapshot.user_stats.is_some()
        || snapshot.permission_analytics.is_some()
        || snapshot.plan_stats.is_some()
        || snapshot.developer_portal.is_some()
}

pub fn render(ctx: &PageContext) -> (PageMeta, Element) {
    let meta = PageMeta::admin("Analytics");
    (meta, rsx! { RenderAnalytics { ctx: ctx.clone() } })
}

#[component]
fn RenderAnalytics(ctx: PageContext) -> Element {
    let load = analytics_load(&ctx);
    rsx! {
        AuthGate {
            user: ctx.user.clone(),
            feature: Some("the private admin analytics workspace".to_string()),
            return_url: Some(ANALYTICS_PATH.to_string()),
            AdminShell {
                ctx: ctx.clone(),
                page_title: "Analytics".to_string(),
                breadcrumbs: vec![
                    ("Dashboard".to_string(), "/".to_string()),
                    ("Analytics".to_string(), ANALYTICS_PATH.to_string()),
                ],
                match load {
                    AnalyticsLoad::Ready(snapshot) => rsx! { AnalyticsReady { snapshot } },
                    AnalyticsLoad::Empty => rsx! { AnalyticsEmpty {} },
                    AnalyticsLoad::Forbidden => rsx! {
                        AnalyticsProblem {
                            state: ADMIN_ANALYTICS_FORBIDDEN,
                            title: "Analytics access was denied".to_string(),
                            detail: "The backend did not authorize this session to read analytics.".to_string(),
                        }
                    },
                    AnalyticsLoad::Unavailable => rsx! {
                        AnalyticsProblem {
                            state: ADMIN_ANALYTICS_UNAVAILABLE,
                            title: "Platform analytics are unavailable".to_string(),
                            detail: "The backend did not provide an authoritative analytics response. No values are being shown.".to_string(),
                        }
                    },
                    AnalyticsLoad::Malformed => rsx! {
                        AnalyticsProblem {
                            state: ADMIN_ANALYTICS_MALFORMED,
                            title: "Analytics data could not be verified".to_string(),
                            detail: "The backend response did not match the strict analytics projection. No values are being shown.".to_string(),
                        }
                    },
                }
            }
        }
    }
}

#[component]
fn AnalyticsReady(snapshot: AdminAnalyticsSnapshot) -> Element {
    rsx! {
        div {
            class: "container page-content admin-analytics py-8",
            "data-admin-analytics-state": ADMIN_ANALYTICS_READY,
            p { class: "text-sm text-muted-foreground", "Backend-authoritative analytics snapshot" }
            div { class: "mt-6 grid gap-5 md:grid-cols-2 xl:grid-cols-4",
                if let Some(stats) = snapshot.user_stats {
                    AnalyticsGroup {
                        title: "Users".to_string(),
                        items: vec![
                            ("Total users".to_string(), format_count(stats.total)),
                            ("Active users".to_string(), format_count(stats.active)),
                            ("Connections today".to_string(), format_count(stats.today_connections)),
                        ],
                    }
                }
                if let Some(stats) = snapshot.permission_analytics {
                    AnalyticsGroup {
                        title: "Permissions".to_string(),
                        items: vec![
                            ("Plans".to_string(), format_count(stats.total_plans)),
                            ("Permissions".to_string(), format_count(stats.total_permissions)),
                            ("Active permissions".to_string(), format_count(stats.active_permissions)),
                        ],
                    }
                }
                if let Some(stats) = snapshot.plan_stats {
                    AnalyticsGroup {
                        title: "Plans".to_string(),
                        items: vec![
                            ("Total plans".to_string(), format_count(stats.total_plans)),
                            ("Active plans".to_string(), format_count(stats.active_plans)),
                            ("Memberships".to_string(), format_count(stats.total_memberships)),
                            ("Recent assignments".to_string(), format_count(stats.recent_assignments)),
                        ],
                    }
                }
                if let Some(stats) = snapshot.developer_portal {
                    AnalyticsGroup {
                        title: "Developer access".to_string(),
                        items: vec![
                            ("API keys".to_string(), format_count(stats.total_api_keys)),
                            ("Active API keys".to_string(), format_count(stats.active_api_keys)),
                        ],
                    }
                }
            }
        }
    }
}

#[component]
fn AnalyticsGroup(title: String, items: Vec<(String, String)>) -> Element {
    rsx! {
        section {
            class: "rounded-2xl border border-border/30 bg-card p-5 shadow-sm",
            aria_labelledby: "analytics-group-title",
            h2 { id: "analytics-group-title", class: "text-sm font-semibold text-foreground", "{title}" }
            dl { class: "mt-4 space-y-4",
                for (label, value) in items {
                    div {
                        dt { class: "text-xs text-muted-foreground", "{label}" }
                        dd { class: "mt-1 text-2xl font-black tracking-tight text-foreground", "{value}" }
                    }
                }
            }
        }
    }
}

#[component]
fn AnalyticsEmpty() -> Element {
    rsx! {
        section {
            class: "container page-content admin-analytics py-8",
            role: "status",
            "data-admin-analytics-state": ADMIN_ANALYTICS_EMPTY,
            div { class: "rounded-2xl border border-border/30 bg-card p-8 text-center",
                Icon { name: "bar-chart-3".to_string(), size: Some(30) }
                h2 { class: "mt-4 text-xl font-semibold text-foreground", "No analytics data is available" }
                p { class: "mt-2 text-sm leading-6 text-muted-foreground", "The backend returned an authoritative empty analytics snapshot." }
                a { class: "btn btn-sm btn-outline mt-5", href: ANALYTICS_PATH, "Refresh analytics" }
            }
        }
    }
}

#[component]
fn AnalyticsProblem(state: &'static str, title: String, detail: String) -> Element {
    rsx! {
        section {
            class: "container page-content admin-analytics py-8",
            role: "alert",
            "data-admin-analytics-state": state,
            div { class: "rounded-2xl border border-amber-500/25 bg-amber-500/10 p-8",
                h2 { class: "text-xl font-semibold text-foreground", "{title}" }
                p { class: "mt-3 max-w-3xl text-sm leading-6 text-muted-foreground", "{detail}" }
                nav { class: "mt-6 flex flex-wrap gap-3", aria_label: "Analytics recovery",
                    a { class: "btn btn-sm btn-outline", href: ANALYTICS_PATH,
                        Icon { name: "refresh-cw".to_string(), size: Some(15) }
                        " Check again"
                    }
                    a { class: "btn btn-sm btn-ghost", href: "/", "Admin home" }
                }
            }
        }
    }
}

fn format_count(value: i64) -> String {
    let digits = value.to_string();
    let mut formatted = String::with_capacity(digits.len() + digits.len() / 3);
    for (index, character) in digits.chars().enumerate() {
        if index > 0 && (digits.len() - index).is_multiple_of(3) {
            formatted.push(',');
        }
        formatted.push(character);
    }
    formatted
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::*;
    use crate::auth::user::{AuthMethod, User};
    use serde_json::json;

    fn signed_in_ctx() -> PageContext {
        PageContext {
            user: Some(User {
                id: "analytics-session".to_string(),
                address: "0x1234abcd5678ef90".to_string(),
                chain_id: "56".to_string(),
                roles: vec![],
                email: None,
                tier: None,
                permissions: vec![],
                last_login_at: None,
                auth_method: AuthMethod::Wallet,
                display_name: None,
            }),
            path: ANALYTICS_PATH.to_string(),
            ..Default::default()
        }
    }

    fn html(ctx: &PageContext) -> String {
        let (_, element) = render(ctx);
        dioxus_ssr::render_element(element)
    }

    fn ready_snapshot() -> AdminAnalyticsSnapshot {
        AdminAnalyticsSnapshot {
            user_stats: Some(AdminAnalyticsUserStats {
                total: 120,
                active: 100,
                today_connections: 12,
                total_users: 120,
                active_users: 100,
            }),
            permission_analytics: None,
            plan_stats: None,
            system_metrics: None,
            developer_portal: None,
        }
    }

    fn with_state(
        mut ctx: PageContext,
        state: &str,
        data: Option<AdminAnalyticsSnapshot>,
    ) -> PageContext {
        ctx.params
            .insert(ADMIN_ANALYTICS_STATE_PARAM.to_string(), state.to_string());
        if let Some(data) = data {
            ctx.params.insert(
                ADMIN_ANALYTICS_DATA_PARAM.to_string(),
                serde_json::to_string(&data).unwrap(),
            );
        }
        ctx
    }

    #[test]
    fn decoder_accepts_only_bounded_backend_projection() {
        assert!(
            decode_admin_analytics_projection(serde_json::to_value(ready_snapshot()).unwrap())
                .is_some()
        );

        let mut unknown = serde_json::to_value(ready_snapshot()).unwrap();
        unknown["sample_series"] = json!([1, 2, 3]);
        assert!(decode_admin_analytics_projection(unknown).is_none());

        let mut negative = serde_json::to_value(ready_snapshot()).unwrap();
        negative["user_stats"]["active"] = json!(-1);
        assert!(decode_admin_analytics_projection(negative).is_none());

        let mut fabricated_telemetry = serde_json::to_value(ready_snapshot()).unwrap();
        fabricated_telemetry["system_metrics"] = json!({"health_percentage": 99.9});
        assert!(decode_admin_analytics_projection(fabricated_telemetry).is_none());
    }

    #[test]
    fn signed_out_route_keeps_analytics_projection_private() {
        let rendered = html(&PageContext {
            path: ANALYTICS_PATH.to_string(),
            ..Default::default()
        });
        assert!(rendered.contains("Sign in required"));
        assert!(!rendered.contains("data-admin-analytics-state"));
        assert!(!rendered.contains("120"));
    }

    #[test]
    fn ready_projection_renders_backend_values_without_entitlement_or_mutation_ui() {
        let rendered = html(&with_state(
            signed_in_ctx(),
            ADMIN_ANALYTICS_READY,
            Some(ready_snapshot()),
        ));
        assert!(rendered.contains("data-admin-analytics-state=\"ready\""));
        assert!(rendered.contains("120"));
        assert!(rendered.contains("100"));
        assert!(rendered.contains("12"));
        for forbidden in [
            "Permission required",
            "admin:analytics:view",
            "Upgrade",
            "Export",
            "Save",
            "Delete",
            "<form",
            "<input",
        ] {
            assert!(
                !rendered.contains(forbidden),
                "unsupported analytics UI: {forbidden}"
            );
        }
    }

    #[test]
    fn explicit_states_are_truthful_and_mismatched_data_fails_closed() {
        let empty = html(&with_state(
            signed_in_ctx(),
            ADMIN_ANALYTICS_EMPTY,
            Some(AdminAnalyticsSnapshot {
                user_stats: None,
                permission_analytics: None,
                plan_stats: None,
                system_metrics: None,
                developer_portal: None,
            }),
        ));
        assert!(empty.contains("data-admin-analytics-state=\"empty\""));

        for state in [ADMIN_ANALYTICS_FORBIDDEN, ADMIN_ANALYTICS_UNAVAILABLE] {
            let rendered = html(&with_state(signed_in_ctx(), state, None));
            assert!(rendered.contains(&format!("data-admin-analytics-state=\"{state}\"")));
            assert!(!rendered.contains("120"));
        }

        let malformed = html(&with_state(signed_in_ctx(), ADMIN_ANALYTICS_READY, None));
        assert!(malformed.contains("data-admin-analytics-state=\"malformed\""));

        let mut hostile = signed_in_ctx();
        hostile.query = "role=admin&permission=admin:analytics:view".to_string();
        hostile.params = HashMap::from([(
            ADMIN_ANALYTICS_DATA_PARAM.to_string(),
            "{\"sample_series\":[\"HOSTILE\"]}".to_string(),
        )]);
        let rendered = html(&hostile);
        assert!(rendered.contains("data-admin-analytics-state=\"unavailable\""));
        assert!(!rendered.contains("HOSTILE"));
        assert!(!rendered.contains("admin:analytics:view"));
    }

    #[test]
    fn authenticated_page_owns_one_shell_and_safe_recovery_links() {
        let rendered = html(&signed_in_ctx());
        assert_eq!(
            rendered
                .matches("class=\"admin-shell admin-shell-page\"")
                .count(),
            1
        );
        assert!(rendered.contains("href=\"/analytics\""));
        assert!(rendered.contains("href=\"/\""));
        assert!(!rendered.contains("javascript:"));
        assert!(!rendered.contains("onclick="));
    }
}
