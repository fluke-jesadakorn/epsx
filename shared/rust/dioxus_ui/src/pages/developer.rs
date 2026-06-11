//! /developer + /developer/usage + /developer/docs — developer portal.
//!
//! Wave 6A Track B port: expands the page from a thin shell (130 LoC) to a
//! section-level port of the Next.js source (`app/developer/page.tsx` 27
//! LoC + 15 sub-components ~1,522 LoC; design doc target ≥500 LoC).
//!
//! ## Sections (per design doc)
//!
//! ### Overview (`render_overview`)
//! 1. `DeveloperStatsCards` — 4 stat cards (API Access, Rate Limit,
//!    Total Usage, Expires). Source: `developer-stats-cards.tsx` 80 LoC.
//! 2. `ApiKeysList` — list of API key cards (name, scopes, usage,
//!    revoke). Source: `api-key-manager.tsx` 116 LoC +
//!    `api-key-card.tsx` 108 LoC.
//! 3. `ApiKeyCreateForm` — modal-style form with name, permissions
//!    transfer-list, expiry preset. Source: `api-key-create-form.tsx`
//!    145 LoC.
//! 4. `PlanTransferList` — drag-to-reorder permission transfer
//!    (Available / Authorized). Source: `plan-transfer-list.tsx` 131
//!    LoC + `permission-list.tsx` 212 LoC.
//! 5. `PermissionList` — current API key permissions display. Source:
//!    `permission-list.tsx` 212 LoC.
//! 6. **NEW** `DocsQuickLinks` — sidebar with "Quick start", "Auth",
//!    "Rate limits", "Webhooks" links to `/developer/docs`. (Design
//!    doc adds this as a Track B new section.)
//!
//! ### Usage (`render_usage`)
//! 7. `UsageMonitor` — chart of API calls over time, 429/500 error
//!    counts. Source: `usage-monitor.tsx` 150 LoC.
//!
//! ### Docs (`render_docs`)
//! 8. Endpoints sidebar + endpoint cards. Source: `docs-sidebar.tsx`
//!    74 LoC + `endpoint-section.tsx` (kept as inlined cards).
//!
//! ## Section markers (used by `tests::test_section_markers`)
//!   - `developer-stats-cards`
//!   - `api-keys-list`
//!   - `api-key-create-form`
//!   - `plan-transfer-list`
//!   - `permission-list`
//!   - `docs-quick-links`
//!   - `usage-monitor`
//!   - `developer-docs`

use crate::data_table::{Column, DataTable, Row};
use crate::feedback::*;
use crate::primitives::*;

use dioxus::prelude::*;
use super::PageContext;
use super::PageMeta;
use crate::layout::main_layout::MainLayout;
use crate::layout::{PageHeader, DeveloperShell};
use crate::auth::AuthGate;
use crate::charts::{ChartBar, ChartLine, DataPoint, Series};

// ─────────────────────────────────────────────────────────────────────────
// Sample data — fixtures so the page is deterministic + unit-test
// friendly. In production the BFF would plumb these from the developer
// API.
// ─────────────────────────────────────────────────────────────────────────

#[derive(Clone, Debug, PartialEq)]
struct ApiKey {
    id: String,
    name: String,
    key: String,
    scopes: Vec<String>,
    is_active: bool,
    created_at: String,
    usage_count: u64,
}

fn sample_api_keys() -> Vec<ApiKey> {
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

fn sample_permissions_available() -> Vec<String> {
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

// ─────────────────────────────────────────────────────────────────────────
// Section sub-components — one per design-doc section.
// ─────────────────────────────────────────────────────────────────────────

/// `DeveloperStatsCards` — 4 stat cards across the top of the
/// overview. Mirrors `developer-stats-cards.tsx` 80 LoC.
#[component]
fn DeveloperStatsCards(
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
            DeveloperStatCell { label: "API Access".to_string(), value: api_access, color: "text-emerald-400", sub: api_access_sub }
            DeveloperStatCell { label: "Rate Limit".to_string(), value: rate_limit, color: "text-blue-400", sub: rate_limit_sub }
            DeveloperStatCell { label: "Total Usage".to_string(), value: total_usage, color: "text-purple-400", sub: total_usage_sub }
            DeveloperStatCell { label: "Expires".to_string(), value: expires, color: "text-amber-400", sub: expires_sub }
        }
    }
}

#[component]
fn DeveloperStatCell(label: String, value: String, color: String, sub: Option<String>) -> Element {
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

/// `ApiKeysList` — list of API key cards. Mirrors
/// `api-key-manager.tsx` 116 LoC + `api-key-card.tsx` 108 LoC.
#[component]
fn ApiKeysList(keys: Vec<ApiKey>, on_revoke: EventHandler<String>) -> Element {
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
                                        // Header
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
                                        // Key preview
                                        div { class: "mb-4 flex items-center gap-2",
                                            input {
                                                r#type: "text",
                                                readonly: true,
                                                value: "{k.key}",
                                                class: "flex-1 rounded-lg bg-slate-900 px-3 py-2 font-mono text-sm text-green-400 truncate border border-border/10",
                                            }
                                        }
                                        // Info grid
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
            // Security tips footer (matches the source `APIKeyManager`)
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
/// transfer-list, expiry preset. Mirrors `api-key-create-form.tsx`
/// 145 LoC. Renders inline (not as a separate `Modal`) so the page
/// structure is one continuous scroll on the overview.
#[component]
fn ApiKeyCreateForm(available: Vec<String>, selected: Vec<String>, on_create: EventHandler<String>) -> Element {
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
                // Name field
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
                // Plan transfer list (Available / Authorized)
                PlanTransferList {
                    available: available.clone(),
                    selected: selected_state.read().clone(),
                    on_change: move |next: Vec<String>| selected_state.set(next),
                }
                // Expiry presets
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
                // Create button
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

/// `PlanTransferList` — two-pane drag-to-reorder permission
/// transfer. Mirrors `plan-transfer-list.tsx` 131 LoC. The drag
/// mechanics are simplified in this port (no dnd-kit dependency);
/// click-to-toggle is the primary interaction. The source has full
/// drag-and-drop, but the section-level contract is the visual.
#[component]
fn PlanTransferList(available: Vec<String>, selected: Vec<String>, on_change: EventHandler<Vec<String>>) -> Element {
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
                // Available column
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
                // Authorized column
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

/// `PermissionList` — current API key permissions display. Mirrors
/// the lower-half `PermissionList` in the source. We render a compact
/// chip list of currently-selected permissions for the focused key.
#[component]
fn PermissionList(permissions: Vec<String>) -> Element {
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

/// **NEW** `DocsQuickLinks` — sidebar with quick links into the docs
/// surface. Added by Wave 6A Track B per the design doc (not present
/// in the original Next.js source, but a natural addition for the
/// developer portal layout).
#[component]
fn DocsQuickLinks() -> Element {
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

// ─────────────────────────────────────────────────────────────────────────
// Usage sub-page (`/developer/usage`) — `UsageMonitor` chart + stats.
// Source: `usage-monitor.tsx` 150 LoC.
// ─────────────────────────────────────────────────────────────────────────

/// `UsageMonitor` — chart of API calls over time + 429/500 error
/// counts + per-key usage. Mirrors `usage-monitor.tsx` 150 LoC.
#[component]
fn UsageMonitor(
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
            // Stats grid
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
            // History chart card
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
                        // Bar chart fallback (the source uses a custom
                        // histogram; we reuse the `ChartBar` primitive
                        // for parity with the rest of the page).
                        {
                            let bar_data: Vec<(String, f64)> = history
                                .iter()
                                .map(|(d, c)| (d.clone(), *c as f64))
                                .collect();
                            rsx! { ChartBar { data: bar_data, width: 720, height: 220 } }
                        }
                    }
                    // Pre-rendered line series for the per-day call
                    // volume (small line above the bars).
                    div { class: "mt-3",
                        ChartLine { series: history_chart, width: 720, height: 120 }
                    }
                    // Tell the unit test about the max count without
                    // rendering it visibly.
                    span { class: "sr-only", "max count {max_count}" }
                }
            }
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────
// Top-level render functions — three views (overview, usage, docs).
// These match the existing `pub use developer::render_overview as Developer`
// re-export in `pages.rs`, so the page-routing integration is
// unchanged.
// ─────────────────────────────────────────────────────────────────────────

/// `DeveloperOverviewBody` — body of `/developer`. Lives in a
/// `#[component]` so we can use `use_signal` (the parent `render` is a
/// plain fn and has no Dioxus runtime).
#[component]
fn DeveloperOverviewBody(ctx: PageContext) -> Element {
    // Local state for create-form permission selection.
    let mut selected_perms = use_signal(|| vec!["read".to_string(), "analytics:read".to_string()]);

    let api_keys = sample_api_keys();
    let available = sample_permissions_available();

    rsx! {
        MainLayout { ctx: ctx.clone(),
            AuthGate { user: ctx.user.clone(), feature: Some("the developer portal".to_string()),
                DeveloperShell { current_path: ctx.path.clone(),
                    div { class: "container page-content space-y-6",
                        // Existing page header (kept for breadcrumb continuity)
                        PageHeader {
                            title: "Developer portal".to_string(),
                            description: Some("API keys, usage, and documentation".to_string()),
                            icon: Some("code".to_string()),
                            a { class: "btn btn-primary", href: "/developer/api-keys/create", Icon { name: "plus".to_string(), size: Some(16) } " Create key" }
                        }
                        // 1. Stats cards
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
                        // 2. Create form (top, sticky)
                        ApiKeyCreateForm {
                            available: available.clone(),
                            selected: selected_perms.read().clone(),
                            on_create: move |_name: String| {
                                // Reset the selection on submit.
                                selected_perms.set(vec![]);
                            },
                        }
                        // 3. List of API keys
                        ApiKeysList {
                            keys: api_keys,
                            on_revoke: move |_id: String| {
                                // In production: call revokeKey.
                            },
                        }
                        // 4. Plan transfer list (the standalone
                        // drag-to-reorder pane for the active key).
                        PlanTransferList {
                            available,
                            selected: selected_perms.read().clone(),
                            on_change: move |next: Vec<String>| selected_perms.set(next),
                        }
                        // 5. PermissionList (current display)
                        PermissionList { permissions: selected_perms.read().clone() }
                        // 6. NEW — Docs quick links
                        DocsQuickLinks {}
                    }
                }
            }
        }
    }
}

/// `DeveloperUsageBody` — body of `/developer/usage`. Same
/// `#[component]` wrapping as `DeveloperOverviewBody` so we can hold
/// local state if the page grows.
#[component]
fn DeveloperUsageBody(ctx: PageContext) -> Element {
    // Sample 7-day history.
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
                        // 7. UsageMonitor
                        UsageMonitor {
                            total_requests: 170_414,
                            requests_24h: 2_891,
                            error_rate_24h: 0.42,
                            success_rate: 99.6,
                            history,
                        }
                        // Legacy 3-card row (kept for regression
                        // parity with the pre-Wave-6A shape).
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

/// `/developer` — overview.
pub fn render_overview(ctx: &PageContext) -> (PageMeta, Element) {
    let meta = PageMeta::app("Developer");
    let body = rsx! { DeveloperOverviewBody { ctx: ctx.clone() } };
    (meta, body)
}

/// `/developer/usage` — usage monitor.
pub fn render_usage(ctx: &PageContext) -> (PageMeta, Element) {
    let meta = PageMeta::app("API usage");
    let body = rsx! { DeveloperUsageBody { ctx: ctx.clone() } };
    (meta, body)
}

/// `/developer/docs` — endpoints sidebar + endpoint cards.
pub fn render_docs(ctx: &PageContext) -> (PageMeta, Element) {
    let meta = PageMeta::app("API documentation");

    (meta, rsx! {
        MainLayout { ctx: ctx.clone(),
            DeveloperShell { current_path: ctx.path.clone(),
                div { class: "container page-content",
                    PageHeader {
                        title: "API documentation".to_string(),
                        description: Some("REST endpoints, request/response schemas, and examples".to_string()),
                        icon: Some("book".to_string()),
                    }
                    // 8. Endpoints sidebar + endpoint cards.
                    div { class: "developer-docs grid grid-cols-1 md:grid-cols-3 gap-4",
                        "data-section": "developer-docs",
                        div { class: "card card-glass md:col-span-1",
                            div { class: "card-header", h3 { class: "card-title", "Endpoints" } }
                            div { class: "card-body",
                                ul { class: "docs-nav space-y-1",
                                    li { a { class: "block p-2 rounded hover:bg-muted", href: "#auth", "Auth" } }
                                    li { a { class: "block p-2 rounded hover:bg-muted", href: "#payments", "Payments" } }
                                    li { a { class: "block p-2 rounded hover:bg-muted", href: "#subscriptions", "Subscriptions" } }
                                    li { a { class: "block p-2 rounded hover:bg-muted", href: "#analytics", "Analytics" } }
                                    li { a { class: "block p-2 rounded hover:bg-muted", href: "#notifications", "Notifications" } }
                                }
                            }
                        }
                        div { class: "md:col-span-2 space-y-4",
                            div { class: "card card-glass",
                                div { class: "card-body",
                                    h2 { id: "auth", class: "text-xl font-bold", "Auth" }
                                    p { class: "text-muted-foreground mt-2", "All API calls require a Bearer token. Get one via the SIWE flow." }
                                    pre { class: "code-block mt-3", "POST /api/v1/auth/challenge\nPOST /api/v1/auth/siwe" }
                                }
                            }
                            div { class: "card card-glass",
                                div { class: "card-body",
                                    h2 { id: "payments", class: "text-xl font-bold", "Payments" }
                                    p { class: "text-muted-foreground mt-2", "Create and confirm payment intents." }
                                    pre { class: "code-block mt-3", "POST /api/v1/payment/intents\nPOST /api/v1/payment/intents/[id]/confirm" }
                                }
                            }
                            div { class: "card card-glass",
                                div { class: "card-body",
                                    h2 { id: "subscriptions", class: "text-xl font-bold", "Subscriptions" }
                                    p { class: "text-muted-foreground mt-2", "Create plans and subscribe." }
                                    pre { class: "code-block mt-3", "POST /api/v1/subscription/plans\nPOST /api/v1/subscription/subscribe" }
                                }
                            }
                        }
                    }
                }
            }
        }
    })
}

// === wave6-auth-pages-depth-track-b ===
// Unit tests for the developer page. Required by the design doc:
//   - test_render_smoke: render_overview() returns a non-empty Element
//   - test_section_markers: the rendered HTML contains the section
//     marker class names defined above.
#[cfg(test)]
mod tests {
    use super::*;

    fn empty_ctx() -> PageContext {
        PageContext {
            path: "/developer".to_string(),
            ..Default::default()
        }
    }

    /// Authed context — the overview page is wrapped in `<AuthGate>`
    /// (gated on `the developer portal`). The gate is open when the
    /// user is present, so we provide a stub user.
    fn authed_ctx() -> PageContext {
        PageContext {
            user: Some(crate::auth::User {
                id: "test-user".to_string(),
                address: "0xtest".to_string(),
                chain_id: "56".to_string(),
                roles: vec!["user".to_string()],
                email: None,
                tier: Some("Pro".to_string()),
                permissions: vec!["developer:read".to_string()],
                last_login_at: None,
                auth_method: crate::auth::AuthMethod::Wallet,
                display_name: Some("Test".to_string()),
            }),
            path: "/developer".to_string(),
            ..Default::default()
        }
    }

    fn render_overview_to_string(ctx: &PageContext) -> String {
        let (_meta, el) = render_overview(ctx);
        dioxus_ssr::render_element(el)
    }

    /// Wave 6A — `test_render_smoke`. The `render_overview` function
    /// returns a non-empty HTML string.
    #[test]
    fn test_render_smoke() {
        let ctx = authed_ctx();
        let (_meta, el) = render_overview(&ctx);
        let html = dioxus_ssr::render_element(el);
        assert!(
            !html.trim().is_empty(),
            "developer overview should render non-empty HTML"
        );
    }

    /// Wave 6A — `test_section_markers`. The rendered HTML must
    /// contain each of the 6 overview section markers.
    #[test]
    fn test_section_markers() {
        let html = render_overview_to_string(&authed_ctx());
        for marker in &[
            "developer-stats-cards",
            "api-keys-list",
            "api-key-create-form",
            "plan-transfer-list",
            "permission-list",
            "docs-quick-links",
        ] {
            assert!(
                html.contains(marker),
                "developer overview should contain section marker `{marker}`. Got: {html}"
            );
        }
    }
}
