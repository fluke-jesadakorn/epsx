//! Sub-components extracted from `pages/developer.rs` during
//! Wave 6C Track E (1:1 user-side component parity).
//!
//! Ten named sub-components: `DeveloperStatsCards`, `DeveloperStatCell`,
//! `ApiKeysList`, `ApiKeyCreateForm`, `PlanTransferList`,
//! `PermissionList`, `DocsQuickLinks`, `UsageMonitor`,
//! `DeveloperOverviewBody`, `DeveloperUsageBody`. Also the `ApiKey`
//! data type (made `pub` for module surface).

use crate::data_table::{Column, DataTable, Row};
use crate::feedback::*;
use crate::primitives::*;
use crate::pages::PageContext;
use crate::layout::main_layout::MainLayout;
use crate::layout::{PageHeader, DeveloperShell};
use crate::auth::AuthGate;
use crate::charts::{ChartBar, ChartLine, DataPoint, Series};

use dioxus::prelude::*;

/// `ApiKey` — shape consumed by `ApiKeysList` and `ApiKeyCreateForm`.
#[derive(Clone, Debug, PartialEq)]
pub struct ApiKey {
    pub id: String,
    pub name: String,
    pub key: String,
    pub scopes: Vec<String>,
    pub is_active: bool,
    pub created_at: String,
    pub usage_count: u64,
}

pub fn sample_api_keys() -> Vec<ApiKey> {
    vec![
        ApiKey {
            id: "k_prod".into(),
            name: "Production".into(),
            key: "epsx_live_4f8a2c1b9d3e7f5a".into(),
            scopes: vec!["read".into(), "write".into(), "analytics:read".into()],
            is_active: true,
            created_at: "2024-08-01".into(),
            usage_count: 142_310,
        },
        ApiKey {
            id: "k_staging".into(),
            name: "Staging".into(),
            key: "epsx_test_7c1d4e2f8a3b6c9d".into(),
            scopes: vec!["read".into(), "analytics:read".into()],
            is_active: true,
            created_at: "2024-08-15".into(),
            usage_count: 28_104,
        },
        ApiKey {
            id: "k_legacy".into(),
            name: "Legacy CI".into(),
            key: "epsx_live_2e5a8b1c4f7d3a9b".into(),
            scopes: vec!["read".into()],
            is_active: false,
            created_at: "2024-04-22".into(),
            usage_count: 0,
        },
    ]
}

pub fn sample_permissions_available() -> Vec<String> {
    vec![
        "read".into(),
        "write".into(),
        "delete".into(),
        "analytics:read".into(),
        "analytics:write".into(),
        "payments:read".into(),
        "payments:write".into(),
        "subscriptions:read".into(),
        "subscriptions:write".into(),
        "users:read".into(),
        "users:write".into(),
    ]
}

/// `DeveloperStatsCards` — 4 stat cards across the top of the overview.
#[component]
pub fn DeveloperStatsCards(
    api_access: String,
    api_access_sub: Option<String>,
    rate_limit: String,
    rate_limit_sub: Option<String>,
    total_usage: String,
    total_usage_sub: Option<String>,
    expires: String,
    expires_sub: Option<String>,
) -> Element {
    rsx! {
        div { class: "developer-stats-cards grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-4",
            "data-section": "developer-stats-cards",
            DeveloperStatCell { label: "API Access".to_string(), value: api_access, color: "text-emerald-400".to_string(), sub: api_access_sub }
            DeveloperStatCell { label: "Rate Limit".to_string(), value: rate_limit, color: "text-blue-400".to_string(), sub: rate_limit_sub }
            DeveloperStatCell { label: "Total Usage".to_string(), value: total_usage, color: "text-purple-400".to_string(), sub: total_usage_sub }
            DeveloperStatCell { label: "Expires".to_string(), value: expires, color: "text-amber-400".to_string(), sub: expires_sub }
        }
    }
}

#[component]
pub fn DeveloperStatCell(label: String, value: String, color: String, sub: Option<String>) -> Element {
    rsx! {
        div { class: "rounded-2xl border border-border/20 bg-card p-5 shadow-xl",
            p { class: "text-xs font-medium text-muted-foreground", "{label}" }
            p { class: "mt-2 text-2xl font-bold {color}", "{value}" }
            if let Some(s) = &sub {
                p { class: "mt-1 text-[11px] text-muted-foreground/60", "{s}" }
            }
        }
    }
}

/// `ApiKeysList` — list of API key cards.
#[component]
pub fn ApiKeysList(keys: Vec<ApiKey>, on_revoke: EventHandler<String>) -> Element {
    rsx! {
        div { class: "api-keys-list rounded-2xl border border-border/20 bg-card shadow-xl",
            "data-section": "api-keys-list",
            div { class: "flex items-center justify-between border-b border-border/10 px-5 py-4",
                h3 { class: "text-lg font-semibold text-foreground", "Your API Keys" }
                button {
                    r#type: "button",
                    class: "rounded-lg bg-background px-3 py-1.5 text-xs font-medium text-muted-foreground hover:text-foreground",
                    "Refresh"
                }
            }
            div { class: "p-5 space-y-3",
                if keys.is_empty() {
                    p { class: "py-10 text-center text-sm text-muted-foreground",
                        "No API keys yet. Create your first key above."
                    }
                } else {
                    for k in keys.iter() {
                        {
                            let k = k.clone();
                            let k_id = k.id.clone();
                            rsx! {
                                div {
                                    key: "{k.id}",
                                    class: "api-key-card rounded-2xl border border-border/10 bg-card transition-shadow hover:shadow-xl",
                                    div { class: "p-5",
                                        div { class: "mb-4 flex items-start justify-between",
                                            div {
                                                div { class: "flex items-center gap-2",
                                                    h3 { class: "font-semibold text-foreground", "{k.name}" }
                                                    span { class: if k.is_active { "rounded-full border border-green-500/30 bg-green-500/10 px-2 py-0.5 text-xs font-medium text-green-400" } else { "rounded-full border border-red-500/30 bg-red-500/10 px-2 py-0.5 text-xs font-medium text-red-400" },
                                                        if k.is_active { "Active" } else { "Revoked" }
                                                    }
                                                }
                                                p { class: "mt-0.5 text-xs text-muted-foreground", "Created {k.created_at}" }
                                            }
                                        }
                                        div { class: "mb-4 flex items-center gap-2",
                                            input {
                                                r#type: "text",
                                                readonly: true,
                                                value: "{k.key}",
                                                class: "flex-1 rounded-lg bg-slate-900 px-3 py-2 font-mono text-sm text-green-400 truncate border border-border/10",
                                            }
                                        }
                                        div { class: "grid grid-cols-2 gap-4 border-t border-border/10 pt-4",
                                            div {
                                                p { class: "text-[10px] font-semibold uppercase tracking-widest text-muted-foreground/60", "Permissions" }
                                                div { class: "mt-1.5 flex flex-wrap gap-1",
                                                    for s in k.scopes.iter().take(3) {
                                                        span { class: "rounded border border-purple-500/30 bg-purple-500/5 px-1.5 py-0.5 text-[10px] font-mono text-purple-400", "{s}" }
                                                    }
                                                    if k.scopes.len() > 3 {
                                                        span { class: "text-[10px] text-muted-foreground", "+{k.scopes.len() - 3}" }
                                                    }
                                                }
                                            }
                                            div { class: "text-right",
                                                p { class: "text-[10px] font-semibold uppercase tracking-widest text-muted-foreground/60", "Usage" }
                                                p { class: "mt-1.5 text-lg font-bold text-foreground", "{k.usage_count}" }
                                            }
                                        }
                                    }
                                    if k.is_active {
                                        div { class: "border-t border-border/10 px-5 py-3 text-right",
                                            button {
                                                r#type: "button",
                                                class: "text-xs font-medium text-red-400/80 hover:text-red-400",
                                                onclick: move |_| on_revoke.call(k_id.clone()),
                                                "Revoke Key"
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
            div { class: "rounded-2xl border border-blue-500/20 bg-blue-500/5 p-5 m-5 mt-0",
                h4 { class: "mb-2 text-sm font-semibold text-blue-300", "Security Best Practices" }
                ul { class: "list-disc list-inside space-y-1 text-xs text-blue-300/80",
                    li { "Never commit API keys to version control" }
                    li { "Use environment variables for key storage" }
                    li { "Rotate keys regularly for production apps" }
                    li { "Revoke unused keys promptly" }
                }
            }
        }
    }
}

/// `ApiKeyCreateForm` — modal-style form with name, permissions
/// transfer-list, expiry preset.
#[component]
pub fn ApiKeyCreateForm(available: Vec<String>, selected: Vec<String>, on_create: EventHandler<String>) -> Element {
    let mut name = use_signal(String::new);
    let mut selected_state = use_signal(|| selected.clone());
    let mut expiry_preset = use_signal(|| "30 Days".to_string());

    rsx! {
        div { class: "api-key-create-form rounded-2xl border border-border/20 bg-card shadow-xl",
            "data-section": "api-key-create-form",
            div { class: "flex items-center gap-3 border-b border-border/10 px-5 py-4",
                div { class: "flex h-10 w-10 items-center justify-center rounded-xl bg-gradient-to-br from-purple-500 to-purple-600 shadow-lg",
                    Icon { name: "plus".to_string(), size: Some(20), class_name: Some("text-white".to_string()) }
                }
                h3 { class: "text-lg font-semibold text-foreground", "Create API Key" }
            }
            div { class: "space-y-5 p-5",
                div {
                    label { class: "mb-1 block text-sm font-medium text-muted-foreground", r#for: "api-key-name", "Key Name" }
                    input {
                        class: "input",
                        id: "api-key-name",
                        r#type: "text",
                        placeholder: "e.g. Production Server",
                        value: "{name.read()}",
                        oninput: move |e| name.set(e.value().to_string()),
                    }
                }
                PlanTransferList {
                    available: available.clone(),
                    selected: selected_state.read().clone(),
                    on_change: move |next: Vec<String>| selected_state.set(next),
                }
                div {
                    label { class: "mb-2 block text-sm font-medium text-muted-foreground", "Expiration" }
                    div { class: "flex flex-wrap gap-2",
                        for preset in ["30 Days", "90 Days", "1 Year", "Never"].iter() {
                            {
                                let preset = preset.to_string();
                                let is_active = *expiry_preset.read() == preset;
                                let active_class = if is_active {
                                    "rounded-lg border border-purple-500/50 bg-purple-500/10 px-3 py-1.5 text-xs font-medium text-purple-300"
                                } else {
                                    "rounded-lg border border-border/30 bg-background px-3 py-1.5 text-xs font-medium text-muted-foreground hover:bg-accent"
                                };
                                rsx! {
                                    button {
                                        r#type: "button",
                                        class: "{active_class}",
                                        onclick: move |_| expiry_preset.set(preset.clone()),
                                        "{preset}"
                                    }
                                }
                            }
                        }
                    }
                }
                button {
                    r#type: "button",
                    class: "btn btn-primary w-full api-key-create-submit",
                    disabled: name.read().trim().is_empty() || selected_state.read().is_empty(),
                    onclick: move |_| on_create.call(name.read().clone()),
                    "Create API Key"
                }
            }
        }
    }
}

/// `PlanTransferList` — two-pane drag-to-reorder permission transfer.
#[component]
pub fn PlanTransferList(available: Vec<String>, selected: Vec<String>, on_change: EventHandler<Vec<String>>) -> Element {
    rsx! {
        div { class: "plan-transfer-list space-y-3",
            "data-section": "plan-transfer-list",
            div { class: "flex items-center gap-2",
                div { class: "p-1.5 rounded-lg bg-amber-500/20 border border-amber-500/30",
                    Icon { name: "shield-check".to_string(), size: Some(14), class_name: Some("text-amber-500".to_string()) }
                }
                span { class: "text-sm font-semibold text-foreground", "Select Permissions" }
            }
            div { class: "grid grid-cols-1 sm:grid-cols-2 gap-4",
                div { class: "permission-list",
                    div { class: "flex items-center justify-between px-1 mb-1",
                        h3 { class: "text-xs uppercase font-bold tracking-widest text-muted-foreground",
                            "Available "
                            span { class: "ml-1 px-2 py-0.5 rounded-full bg-muted text-[10px]", "{available.len()}" }
                        }
                    }
                    div { class: "permission-list-box flex flex-col gap-2 p-2 rounded-2xl border border-border/30 bg-muted/30 h-[250px] overflow-y-auto",
                        for perm in available.iter() {
                            {
                                let perm = perm.clone();
                                let perm_for_click = perm.clone();
                                let selected = selected.clone();
                                let on_change = on_change.clone();
                                let is_selected = selected.contains(&perm);
                                rsx! {
                                    button {
                                        r#type: "button",
                                        key: "avail-{perm}",
                                        class: if is_selected { "flex items-center justify-between p-3 rounded-xl border border-purple-500/50 bg-purple-500/10 text-sm" } else { "flex items-center justify-between p-3 rounded-xl border border-border/30 bg-background text-sm hover:border-amber-300" },
                                        onclick: move |_| {
                                            let mut next = selected.clone();
                                            if next.contains(&perm_for_click) {
                                                next.retain(|p| p != &perm_for_click);
                                            } else {
                                                next.push(perm_for_click.clone());
                                            }
                                            on_change.call(next);
                                        },
                                        span { class: "font-mono text-xs text-foreground", "{perm}" }
                                        Icon { name: "arrow-right".to_string(), size: Some(14), class_name: Some("text-muted-foreground".to_string()) }
                                    }
                                }
                            }
                        }
                    }
                }
                div { class: "permission-list",
                    div { class: "flex items-center justify-between px-1 mb-1",
                        h3 { class: "text-xs uppercase font-bold tracking-widest text-amber-600",
                            "Authorized "
                            span { class: "ml-1 px-2 py-0.5 rounded-full bg-amber-100 text-amber-600 border border-amber-200 text-[10px]", "{selected.len()}" }
                        }
                    }
                    div { class: "permission-list-box flex flex-col gap-2 p-2 rounded-2xl border border-amber-200/30 bg-amber-50/5 h-[250px] overflow-y-auto",
                        if selected.is_empty() {
                            div { class: "flex flex-col items-center justify-center h-full text-amber-500/30",
                                Icon { name: "shield".to_string(), size: Some(32) }
                                p { class: "text-sm italic", "Drag or tap to authorize" }
                            }
                        } else {
                            for perm in selected.iter() {
                                {
                                    let perm = perm.clone();
                                    let perm_for_click = perm.clone();
                                    let selected = selected.clone();
                                    let on_change = on_change.clone();
                                    rsx! {
                                        div {
                                            key: "auth-{perm}",
                                            class: "flex items-center justify-between p-3 rounded-xl border border-amber-300 bg-amber-50 text-sm",
                                            span { class: "font-mono text-xs text-amber-700", "{perm}" }
                                            button {
                                                r#type: "button",
                                                class: "text-amber-700 hover:text-red-500",
                                                onclick: move |_| {
                                                    let mut next = selected.clone();
                                                    next.retain(|p| p != &perm_for_click);
                                                    on_change.call(next);
                                                },
                                                "×"
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

/// `PermissionList` — current API key permissions display.
#[component]
pub fn PermissionList(permissions: Vec<String>) -> Element {
    rsx! {
        div { class: "permission-list-display rounded-2xl border border-border/20 bg-card p-5 shadow-xl",
            "data-section": "permission-list",
            h4 { class: "mb-2 text-sm font-semibold text-foreground", "Current API key permissions" }
            if permissions.is_empty() {
                p { class: "text-xs text-muted-foreground", "No permissions granted" }
            } else {
                div { class: "flex flex-wrap gap-1.5",
                    for p in permissions.iter() {
                        span { class: "rounded border border-purple-500/30 bg-purple-500/5 px-2 py-0.5 text-[10px] font-mono text-purple-400", "{p}" }
                    }
                }
            }
        }
    }
}

/// `DocsQuickLinks` — sidebar with quick links into the docs surface.
#[component]
pub fn DocsQuickLinks() -> Element {
    rsx! {
        div { class: "docs-quick-links rounded-2xl border border-border/20 bg-card p-5 shadow-xl",
            "data-section": "docs-quick-links",
            h4 { class: "mb-2 text-sm font-semibold text-foreground", "Documentation" }
            p { class: "mb-3 text-xs text-muted-foreground", "Jump into the reference." }
            ul { class: "docs-quick-links-list space-y-1",
                li { a { class: "docs-quick-link block rounded-lg px-3 py-2 text-sm text-muted-foreground hover:bg-muted hover:text-foreground", href: "/developer/docs#quick-start", "Quick start" } }
                li { a { class: "docs-quick-link block rounded-lg px-3 py-2 text-sm text-muted-foreground hover:bg-muted hover:text-foreground", href: "/developer/docs#auth", "Auth" } }
                li { a { class: "docs-quick-link block rounded-lg px-3 py-2 text-sm text-muted-foreground hover:bg-muted hover:text-foreground", href: "/developer/docs#rate-limits", "Rate limits" } }
                li { a { class: "docs-quick-link block rounded-lg px-3 py-2 text-sm text-muted-foreground hover:bg-muted hover:text-foreground", href: "/developer/docs#webhooks", "Webhooks" } }
            }
        }
    }
}

/// `UsageMonitor` — chart of API calls over time + 429/500 error
/// counts + per-key usage.
#[component]
pub fn UsageMonitor(
    total_requests: u64,
    requests_24h: u64,
    error_rate_24h: f64,
    success_rate: f64,
    history: Vec<(String, u32)>,
) -> Element {
    let max_count = history.iter().map(|(_, c)| *c).max().unwrap_or(1).max(1);
    let history_chart: Vec<Series> = vec![Series {
        name: "API calls".into(),
        color: "#22d3ee".into(),
        points: history
            .iter()
            .enumerate()
            .map(|(i, (_d, c))| DataPoint {
                x: i as f64,
                y: *c as f64,
                label: None,
            })
            .collect(),
    }];
    rsx! {
        div { class: "usage-monitor space-y-6",
            "data-section": "usage-monitor",
            div { class: "grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-4",
                div { class: "rounded-2xl border border-border/20 bg-card p-5 shadow-xl",
                    p { class: "text-xs font-medium text-muted-foreground", "Total Requests" }
                    p { class: "mt-2 text-2xl font-bold text-emerald-400", "{total_requests}" }
                    p { class: "mt-1 text-[11px] text-muted-foreground/60", "All time" }
                }
                div { class: "rounded-2xl border border-border/20 bg-card p-5 shadow-xl",
                    p { class: "text-xs font-medium text-muted-foreground", "Requests (24h)" }
                    p { class: "mt-2 text-2xl font-bold text-blue-400", "{requests_24h}" }
                    p { class: "mt-1 text-[11px] text-muted-foreground/60", "3 keys (2 active)" }
                }
                div { class: "rounded-2xl border border-border/20 bg-card p-5 shadow-xl",
                    p { class: "text-xs font-medium text-muted-foreground", "Error Rate (24h)" }
                    p { class: "mt-2 text-2xl font-bold text-purple-400", "{error_rate_24h:.2}%" }
                    p { class: "mt-1 text-[11px] text-muted-foreground/60", "Failed requests" }
                }
                div { class: "rounded-2xl border border-border/20 bg-card p-5 shadow-xl",
                    p { class: "text-xs font-medium text-muted-foreground", "Success Rate" }
                    p { class: "mt-2 text-2xl font-bold text-amber-400", "{success_rate:.1}%" }
                    p { class: "mt-1 text-[11px] text-muted-foreground/60", "Global average" }
                }
            }
            div { class: "rounded-2xl border border-border/20 bg-card shadow-xl",
                div { class: "flex items-center justify-between border-b border-border/10 px-5 py-4",
                    h3 { class: "text-sm font-semibold text-foreground", "Usage History" }
                    div { class: "flex gap-1",
                        for r in [7u32, 30, 90].iter() {
                            button {
                                r#type: "button",
                                class: "rounded-md px-2.5 py-1 text-xs font-medium text-muted-foreground hover:text-foreground",
                                "{r}d"
                            }
                        }
                    }
                }
                div { class: "p-5",
                    if history.is_empty() {
                        p { class: "text-center text-sm text-muted-foreground", "No data" }
                    } else {
                        {
                            let bar_data: Vec<(String, f64)> = history
                                .iter()
                                .map(|(d, c)| (d.clone(), *c as f64))
                                .collect();
                            rsx! { ChartBar { data: bar_data, width: 720, height: 220 } }
                        }
                    }
                    div { class: "mt-3",
                        ChartLine { series: history_chart, width: 720, height: 120 }
                    }
                    span { class: "sr-only", "max count {max_count}" }
                }
            }
        }
    }
}

/// `DeveloperOverviewBody` — body of `/developer`.
#[component]
pub fn DeveloperOverviewBody(ctx: PageContext) -> Element {
    let mut selected_perms = use_signal(|| vec!["read".to_string(), "analytics:read".to_string()]);

    let api_keys = sample_api_keys();
    let available = sample_permissions_available();

    rsx! {
        MainLayout { ctx: ctx.clone(),
            AuthGate { user: ctx.user.clone(), feature: Some("the developer portal".to_string()),
                DeveloperShell { current_path: ctx.path.clone(),
                    div { class: "container page-content space-y-6",
                        PageHeader {
                            title: "Developer portal".to_string(),
                            description: Some("API keys, usage, and documentation".to_string()),
                            icon: Some("code".to_string()),
                            a { class: "btn btn-primary", href: "/developer/api-keys/create", Icon { name: "plus".to_string(), size: Some(16) } " Create key" }
                        }
                        DeveloperStatsCards {
                            api_access: "Active".to_string(),
                            api_access_sub: Some("Pro, Enterprise".to_string()),
                            rate_limit: "1000/min".to_string(),
                            rate_limit_sub: Some("50,000/day".to_string()),
                            total_usage: "170,414".to_string(),
                            total_usage_sub: Some("3 API keys".to_string()),
                            expires: "2026-08-15".to_string(),
                            expires_sub: Some("288 days left".to_string()),
                        }
                        ApiKeyCreateForm {
                            available: available.clone(),
                            selected: selected_perms.read().clone(),
                            on_create: move |_name: String| {
                                selected_perms.set(vec![]);
                            },
                        }
                        ApiKeysList {
                            keys: api_keys,
                            on_revoke: move |_id: String| {},
                        }
                        PlanTransferList {
                            available,
                            selected: selected_perms.read().clone(),
                            on_change: move |next: Vec<String>| selected_perms.set(next),
                        }
                        PermissionList { permissions: selected_perms.read().clone() }
                        DocsQuickLinks {}
                    }
                }
            }
        }
    }
}

/// `DeveloperUsageBody` — body of `/developer/usage`.
#[component]
pub fn DeveloperUsageBody(ctx: PageContext) -> Element {
    let history: Vec<(String, u32)> = (0..7)
        .map(|i| {
            let day = format!("D{}", i + 1);
            let count = 800 + (i * 50) + (i * 73 % 200) as u32;
            (day, count)
        })
        .collect();

    rsx! {
        MainLayout { ctx: ctx.clone(),
            AuthGate { user: ctx.user.clone(), feature: Some("usage stats".to_string()),
                DeveloperShell { current_path: ctx.path.clone(),
                    div { class: "container page-content space-y-6",
                        PageHeader {
                            title: "API usage".to_string(),
                            description: Some("Per-day call volume and rate-limit status".to_string()),
                            icon: Some("chart-line".to_string()),
                        }
                        UsageMonitor {
                            total_requests: 170_414,
                            requests_24h: 2_891,
                            error_rate_24h: 0.42,
                            success_rate: 99.6,
                            history,
                        }
                        div { class: "grid grid-cols-1 md:grid-cols-3 gap-4 mt-4",
                            StatCard { label: "Rate limit".to_string(), value: "1000/min".to_string(), icon: Some("gauge".to_string()) }
                            StatCard { label: "Current usage".to_string(), value: "234/min".to_string(), icon: Some("activity".to_string()) }
                            StatCard { label: "Errors (24h)".to_string(), value: "3".to_string(), icon: Some("alert-triangle".to_string()) }
                        }
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Render an `Element` produced by a `fn() -> Element` harness
    /// via a real Dioxus scope. Components with `EventHandler` props
    /// (ApiKeysList, ApiKeyCreateForm, PlanTransferList) need this —
    /// bare `dioxus_ssr::render_element(rsx! { ... })` panics with
    /// "Must be called from inside a Dioxus runtime".
    fn render_html(harness: fn() -> Element) -> String {
        let mut vdom = dioxus::prelude::VirtualDom::new(harness);
        vdom.rebuild_in_place();
        dioxus_ssr::render(&vdom)
    }

    /// Wave 6C Track E — `test_render_smoke` for the extracted
    /// developer sub-components.
    #[test]
    fn developer_subcomponents_render_smoke() {
        // DeveloperStatsCards — no EventHandler, render_element is fine.
        let el = rsx! {
            DeveloperStatsCards {
                api_access: "Active".to_string(),
                api_access_sub: None,
                rate_limit: "1000".to_string(),
                rate_limit_sub: None,
                total_usage: "100".to_string(),
                total_usage_sub: None,
                expires: "2026".to_string(),
                expires_sub: None,
            }
        };
        let html = dioxus_ssr::render_element(el);
        assert!(html.contains("developer-stats-cards"));
        assert!(html.contains("API Access"));

        // ApiKeysList (empty) — has EventHandler<String>.
        let html = render_html(|| rsx! { ApiKeysList { keys: vec![], on_revoke: |_| {} } });
        assert!(html.contains("api-keys-list"));
        assert!(html.contains("No API keys yet"));

        // ApiKeysList (with sample)
        let html = render_html(|| rsx! { ApiKeysList { keys: sample_api_keys(), on_revoke: |_| {} } });
        assert!(html.contains("Production"));

        // PermissionList — no EventHandler.
        let el = rsx! { PermissionList { permissions: vec!["read".to_string(), "write".to_string()] } };
        let html = dioxus_ssr::render_element(el);
        assert!(html.contains("permission-list"));
        assert!(html.contains("Current API key permissions"));

        // DocsQuickLinks
        let el = rsx! { DocsQuickLinks {} };
        let html = dioxus_ssr::render_element(el);
        assert!(html.contains("docs-quick-links"));
        assert!(html.contains("Quick start"));

        // PlanTransferList — has EventHandler<Vec<String>>.
        let html = render_html(|| rsx! {
            PlanTransferList {
                available: sample_permissions_available(),
                selected: vec!["read".to_string()],
                on_change: |_| {},
            }
        });
        assert!(html.contains("plan-transfer-list"));
        assert!(html.contains("Available"));
    }
}
