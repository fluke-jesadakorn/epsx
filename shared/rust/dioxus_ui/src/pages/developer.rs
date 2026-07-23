//! /developer + /developer/usage + /developer/docs — developer portal.
//!
//! Wave 6A Track B port: expands the page from a thin shell (130 LoC) to a
//! section-level port of the Next.js source (`app/developer/page.tsx` 27
//! LoC + 15 sub-components ~1,522 LoC; design doc target ≥500 LoC).
//!
//! ## Sections (per design doc)
//!
//! ### Overview (`render_overview`)
//! 1. Authenticated unavailable state — API key, plan, permission, and usage
//!    claims are withheld until the Rust route has backend-owned contracts.
//!
//! ### Usage (`render_usage`)
//! 7. Authenticated unavailable state — usage reporting is withheld until a
//!    production-owned data contract is available to the Rust page.
//!
//! ### Docs (`render_docs`)
//! 8. Endpoints sidebar + endpoint cards. Source: `docs-sidebar.tsx`
//!    74 LoC + `endpoint-section.tsx` (kept as inlined cards).
//!
//! ## Section markers (used by `tests::test_section_markers`)
//!   - `developer-overview-unavailable`
//!   - `developer-usage-unavailable`
//!   - `developer-docs`

use crate::primitives::*;

use super::PageContext;
use super::PageMeta;
use crate::auth::AuthGate;
use crate::layout::main_layout::MainLayout;
use crate::layout::{DeveloperShell, PageHeader};
use dioxus::prelude::*;
use std::sync::OnceLock;

// ─────────────────────────────────────────────────────────────────────────
// API docs types + endpoint catalog. Ported from
// `apps-old/frontend/components/developer/docs/data/endpoints.ts` (263
// LoC). The catalog is cached in a `OnceLock` so the per-page render
// path doesn't pay the construction cost on every request.
// ─────────────────────────────────────────────────────────────────────────

#[derive(Clone, Debug, PartialEq)]
pub struct EndpointParam {
    pub name: String,
    pub kind: String,
    pub required: bool,
    pub desc: String,
    pub default: Option<String>,
}

impl EndpointParam {
    /// Helper for building a `EndpointParam` with the most common
    /// shape (no default). Mirrors the source's `param()` helper.
    pub fn param(name: &str, kind: &str, required: bool, desc: &str) -> Self {
        Self {
            name: name.to_string(),
            kind: kind.to_string(),
            required,
            desc: desc.to_string(),
            default: None,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct EndpointDef {
    pub method: String,
    pub path: String,
    pub title: String,
    pub desc: String,
    pub tier: String,
    pub params: Vec<EndpointParam>,
    pub headers: Vec<(String, bool, String)>,
    /// JSON-serialized response example, pre-serialized so the
    /// component doesn't have to plumb a JSON value through
    /// Dioxus's signal/type system.
    pub response_example: String,
    pub rate_limits: Vec<(String, String)>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct EndpointCategory {
    pub id: String,
    pub title: String,
    pub desc: String,
    pub endpoints: Vec<EndpointDef>,
}

/// Immutable source snapshot used by the documentation renderer. The route is
/// intentionally static until A5 supplies a generated, runtime-owned OpenAPI
/// contract; keeping the pin beside the catalog makes drift explicit instead
/// of pretending the unused BFF fixture is authoritative.
pub const DEVELOPER_DOCS_SOURCE_BASELINE: &str =
    "origin/development@373bd231cb7a616c3d4c0ddc1d60e0099a88a5db";
const DEVELOPER_DOCS_EXAMPLE_ORIGIN: &str = "https://api.example.invalid";

fn endpoint_categories() -> Vec<EndpointCategory> {
    // Default rate limits per tier — mirrors the source's
    // `defaultRateLimits` constant.
    let default_rate_limits: Vec<(String, String)> = vec![
        ("free".into(), "30/min".into()),
        ("basic".into(), "60/min".into()),
        ("premium".into(), "120/min".into()),
        ("enterprise".into(), "600/min".into()),
    ];
    let bearer = ("Authorization".to_string(), true, "Bearer <api_key>".to_string());
    let optional_bearer = (
        "Authorization".to_string(),
        false,
        "Bearer <api_key> — optional, unlocks premium columns".to_string(),
    );

    vec![
        EndpointCategory {
            id: "auth".into(),
            title: "Authentication".into(),
            desc: "Pinned Authorization Bearer-header reference; accepted credential types are not verified here.".into(),
            endpoints: vec![EndpointDef {
                method: "GET".into(),
                path: "/api/auth/session/verify".into(),
                title: "Verify session".into(),
                desc: "Verify that your API key is valid and return associated permissions.".into(),
                tier: "free".into(),
                params: vec![],
                headers: vec![bearer.clone()],
                response_example: r#"{"success":true,"data":{"wallet_address":"0x1234...abcd","permissions":["epsx:analytics:read","epsx:export:csv"],"auth_method":"api_key"}}"#.into(),
                rate_limits: default_rate_limits.clone(),
            }],
        },
        EndpointCategory {
            id: "analytics".into(),
            title: "Analytics".into(),
            desc: "Market data, stock rankings, filters, countries, and sector breakdowns.".into(),
            endpoints: vec![
                EndpointDef {
                    method: "GET".into(),
                    path: "/api/analytics/rankings".into(),
                    title: "Get stock rankings".into(),
                    desc: "Returns paginated EPS rankings with optional filters. Free tier gets limited columns; premium unlocks all fields.".into(),
                    tier: "free".into(),
                    params: vec![
                        EndpointParam { default: Some("1".into()), ..EndpointParam::param("page", "number", false, "Page number") },
                        EndpointParam { default: Some("20".into()), ..EndpointParam::param("per_page", "number", false, "Results per page (max 100)") },
                        EndpointParam { default: Some("eps_growth".into()), ..EndpointParam::param("sort_by", "string", false, "Sort column (e.g. eps_growth, market_cap)") },
                        EndpointParam { default: Some("desc".into()), ..EndpointParam::param("sort_dir", "string", false, "asc or desc") },
                        EndpointParam::param("country", "string", false, "ISO country code filter (e.g. US, TH)"),
                        EndpointParam::param("sector", "string", false, "Sector filter"),
                        EndpointParam::param("search", "string", false, "Search by ticker or company name"),
                    ],
                    headers: vec![optional_bearer.clone()],
                    response_example: r#"{"success":true,"data":{"items":[{"ticker":"AAPL","name":"Apple Inc.","country":"US","sector":"Technology","eps_growth":12.5,"market_cap":3200000000000,"rank":1}],"pagination":{"page":1,"per_page":20,"total":5420,"total_pages":271}}}"#.into(),
                    rate_limits: vec![
                        ("free".into(), "10/min".into()),
                        ("basic".into(), "60/min".into()),
                        ("premium".into(), "120/min".into()),
                        ("enterprise".into(), "600/min".into()),
                    ],
                },
                EndpointDef {
                    method: "GET".into(),
                    path: "/api/analytics/filters".into(),
                    title: "Get filter options".into(),
                    desc: "Returns available filter values for countries, sectors, and sort columns.".into(),
                    tier: "free".into(),
                    params: vec![],
                    headers: vec![optional_bearer.clone()],
                    response_example: r#"{"success":true,"data":{"countries":[{"code":"US","name":"United States","count":2100}],"sectors":[{"name":"Technology","count":450}],"sort_options":["eps_growth","market_cap","revenue"]}}"#.into(),
                    rate_limits: default_rate_limits.clone(),
                },
                EndpointDef {
                    method: "GET".into(),
                    path: "/api/analytics/countries".into(),
                    title: "Get countries".into(),
                    desc: "Returns list of countries with stock data available.".into(),
                    tier: "free".into(),
                    params: vec![],
                    headers: vec![optional_bearer.clone()],
                    response_example: r#"{"success":true,"data":[{"code":"US","name":"United States","count":2100}]}"#.into(),
                    rate_limits: default_rate_limits.clone(),
                },
                EndpointDef {
                    method: "GET".into(),
                    path: "/api/analytics/sectors".into(),
                    title: "Get sectors".into(),
                    desc: "Returns available sector categories.".into(),
                    tier: "free".into(),
                    params: vec![],
                    headers: vec![optional_bearer.clone()],
                    response_example: r#"{"success":true,"data":[{"name":"Technology","count":450}]}"#.into(),
                    rate_limits: default_rate_limits.clone(),
                },
            ],
        },
        EndpointCategory {
            id: "portfolio".into(),
            title: "Portfolio & Watchlist".into(),
            desc: "Manage your stock watchlist. Requires authentication.".into(),
            endpoints: vec![
                EndpointDef {
                    method: "GET".into(),
                    path: "/api/users/watchlist".into(),
                    title: "Get watchlist".into(),
                    desc: "Returns current user watchlist with stock data.".into(),
                    tier: "basic".into(),
                    params: vec![],
                    headers: vec![bearer.clone()],
                    response_example: r#"{"success":true,"data":{"items":[{"ticker":"AAPL","name":"Apple Inc.","added_at":"2025-01-15T10:30:00Z"}],"count":1}}"#.into(),
                    rate_limits: default_rate_limits.clone(),
                },
                EndpointDef {
                    method: "POST".into(),
                    path: "/api/users/watchlist".into(),
                    title: "Add to watchlist".into(),
                    desc: "Add a stock ticker to your watchlist.".into(),
                    tier: "basic".into(),
                    params: vec![EndpointParam::param("ticker", "string", true, "Stock ticker symbol (e.g. AAPL)")],
                    headers: vec![bearer.clone()],
                    response_example: r#"{"success":true,"data":{"ticker":"AAPL","added_at":"2025-01-15T10:30:00Z"}}"#.into(),
                    rate_limits: default_rate_limits.clone(),
                },
                EndpointDef {
                    method: "DELETE".into(),
                    path: "/api/users/watchlist".into(),
                    title: "Remove from watchlist".into(),
                    desc: "Remove a stock ticker from your watchlist.".into(),
                    tier: "basic".into(),
                    params: vec![EndpointParam::param("ticker", "string", true, "Stock ticker symbol to remove")],
                    headers: vec![bearer.clone()],
                    response_example: r#"{"success":true,"data":{"removed":true}}"#.into(),
                    rate_limits: default_rate_limits.clone(),
                },
            ],
        },
        EndpointCategory {
            id: "user".into(),
            title: "User".into(),
            desc: "User profile and access information.".into(),
            endpoints: vec![
                EndpointDef {
                    method: "GET".into(),
                    path: "/api/users/profile".into(),
                    title: "Get profile".into(),
                    desc: "Returns the authenticated user profile including wallet address and plan info.".into(),
                    tier: "free".into(),
                    params: vec![],
                    headers: vec![bearer.clone()],
                    response_example: r#"{"success":true,"data":{"wallet_address":"0x1234...abcd","plans":[{"name":"Premium","slug":"premium"}],"created_at":"2025-01-01T00:00:00Z"}}"#.into(),
                    rate_limits: default_rate_limits.clone(),
                },
                EndpointDef {
                    method: "GET".into(),
                    path: "/api/users/access-overview".into(),
                    title: "Get access overview".into(),
                    desc: "Returns a summary of permissions and plan features available to the user.".into(),
                    tier: "free".into(),
                    params: vec![],
                    headers: vec![bearer.clone()],
                    response_example: r#"{"success":true,"data":{"permissions":["epsx:analytics:read"],"plans":[{"name":"Premium","features":["Full rankings","CSV export"]}]}}"#.into(),
                    rate_limits: default_rate_limits.clone(),
                },
            ],
        },
    ]
}

/// Public accessor for the cached endpoint catalog. Caches the
/// categories behind a `OnceLock` so the per-page render path
/// doesn't pay the construction cost on every request.
pub fn cached_endpoint_categories() -> &'static Vec<EndpointCategory> {
    static CACHE: OnceLock<Vec<EndpointCategory>> = OnceLock::new();
    CACHE.get_or_init(endpoint_categories)
}

/// Public method-color helper. Mirrors the `methodColor` map at
/// `usage-monitor.tsx:13-17` and `endpoint-card.tsx:10-14`.
pub fn method_color_class(method: &str) -> &'static str {
    match method {
        "GET" => "bg-blue-500/10 text-blue-500",
        "POST" => "bg-green-500/10 text-green-500",
        "DELETE" => "bg-red-500/10 text-red-500",
        _ => "text-muted-foreground",
    }
}

/// Public tier-color helper. Mirrors the pinned `tier-badge.tsx` mapping.
pub fn tier_color_class(tier: &str) -> &'static str {
    match tier {
        "free" => "bg-cyan-500/10 text-cyan-400 border-cyan-500/20",
        "basic" => "bg-green-500/10 text-green-400 border-green-500/20",
        "premium" => "bg-purple-500/10 text-purple-400 border-purple-500/20",
        "enterprise" => "bg-orange-500/10 text-orange-400 border-orange-500/20",
        _ => "text-muted-foreground",
    }
}

fn code_snippet(endpoint: &EndpointDef, language: &str) -> String {
    let url = format!("{DEVELOPER_DOCS_EXAMPLE_ORIGIN}{}", endpoint.path);
    match language {
        "javascript" => {
            let mut options = vec![
                format!("  method: '{}'", endpoint.method),
                "  headers: { 'Authorization': 'Bearer YOUR_API_KEY' }".to_string(),
            ];
            if endpoint.method == "POST" {
                options.push("  body: JSON.stringify({ ticker: 'AAPL' })".to_string());
            }
            format!(
                "const res = await fetch('{url}', {{\n{}\n}});\nconst data = await res.json();",
                options.join(",\n")
            )
        }
        "python" => {
            let mut lines = vec![
                "import requests".to_string(),
                String::new(),
                format!("url = \"{url}\""),
                "headers = {\"Authorization\": \"Bearer YOUR_API_KEY\"}".to_string(),
            ];
            let request = match endpoint.method.as_str() {
                "POST" => "res = requests.post(url, headers=headers, json={\"ticker\": \"AAPL\"})",
                "DELETE" => "res = requests.delete(url, headers=headers, params={\"ticker\": \"AAPL\"})",
                _ => "res = requests.get(url, headers=headers)",
            };
            lines.push(request.to_string());
            lines.push("data = res.json()".to_string());
            lines.join("\n")
        }
        _ => {
            let mut lines = vec![
                format!("curl -X {} \"{url}\"", endpoint.method),
                "  -H \"Authorization: Bearer YOUR_API_KEY\"".to_string(),
            ];
            if endpoint.method == "POST" {
                lines.push("  -H \"Content-Type: application/json\"".to_string());
                lines.push("  -d '{\"ticker\": \"AAPL\"}'".to_string());
            }
            lines.join(" \\\n")
        }
    }
}

fn pretty_response(response: &str) -> String {
    serde_json::from_str::<serde_json::Value>(response)
        .and_then(|value| serde_json::to_string_pretty(&value))
        .unwrap_or_else(|_| response.to_string())
}

// ─────────────────────────────────────────────────────────────────────────
// Developer overview (`/developer`) — truthful migration state.
// ─────────────────────────────────────────────────────────────────────────

/// Authenticated state for developer API management.
///
/// The pinned source populates this surface through authenticated plan and
/// API-key actions. Until equivalent backend-owned Rust contracts exist, the
/// page must not render compatibility payloads or simulate mutations locally.
#[component]
fn DeveloperOverviewUnavailable() -> Element {
    rsx! {
        section {
            class: "developer-overview-unavailable rounded-2xl border border-border/20 bg-card p-8 shadow-xl",
            "data-section": "developer-overview-unavailable",
            role: "status",
            "aria-live": "polite",
            "aria-labelledby": "developer-overview-unavailable-title",
            div { class: "mx-auto max-w-2xl text-center",
                div { class: "mx-auto mb-4 flex h-12 w-12 items-center justify-center rounded-full bg-muted",
                    Icon { name: "code".to_string(), size: Some(24) }
                }
                h2 {
                    id: "developer-overview-unavailable-title",
                    class: "text-xl font-bold text-foreground",
                    "Developer tools unavailable"
                }
                p { class: "mt-3 text-sm text-muted-foreground",
                    "API key and plan management are not available right now. No keys, secrets, plan assignments, permissions, usage, rate limits, or expiration values are shown."
                }
                nav {
                    class: "mt-6 flex flex-wrap justify-center gap-3",
                    "aria-label": "Developer page actions",
                    a {
                        class: "btn btn-primary",
                        href: "/developer",
                        "Retry"
                    }
                    a {
                        class: "btn btn-outline",
                        href: "/developer/docs",
                        "Read API documentation"
                    }
                }
            }
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────
// Usage sub-page (`/developer/usage`) — truthful migration state.
// ─────────────────────────────────────────────────────────────────────────

/// Authenticated state for usage reporting.
///
/// No metrics are rendered until the Rust page has a production-owned,
/// authenticated usage contract. Retry and documentation remain ordinary
/// links so they work in server-rendered output without hydration.
#[component]
fn DeveloperUsageUnavailable() -> Element {
    rsx! {
        section {
            class: "developer-usage-unavailable rounded-2xl border border-border/20 bg-card p-8 shadow-xl",
            "data-section": "developer-usage-unavailable",
            role: "status",
            "aria-live": "polite",
            "aria-labelledby": "developer-usage-unavailable-title",
            div { class: "mx-auto max-w-2xl text-center",
                div { class: "mx-auto mb-4 flex h-12 w-12 items-center justify-center rounded-full bg-muted",
                    Icon { name: "chart-line".to_string(), size: Some(24) }
                }
                h2 {
                    id: "developer-usage-unavailable-title",
                    class: "text-xl font-bold text-foreground",
                    "Usage data unavailable"
                }
                p { class: "mt-3 text-sm text-muted-foreground",
                    "Authenticated usage reporting is temporarily unavailable. No request counts, limits, reliability figures, or activity rows are shown."
                }
                nav {
                    class: "mt-6 flex flex-wrap justify-center gap-3",
                    "aria-label": "Usage page actions",
                    a {
                        class: "btn btn-primary",
                        href: "/developer/usage",
                        "Retry"
                    }
                    a {
                        class: "btn btn-outline",
                        href: "/developer/docs",
                        "Read API documentation"
                    }
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

/// `DeveloperOverviewBody` — authentication-only overview state.
#[component]
fn DeveloperOverviewBody(ctx: PageContext) -> Element {
    rsx! {
        MainLayout { ctx: ctx.clone(),
            AuthGate {
                user: ctx.user.clone(),
                feature: Some("the developer portal".to_string()),
                return_url: Some(ctx.path.clone()),
                DeveloperShell { current_path: ctx.path.clone(),
                    div { class: "container page-content space-y-6",
                        PageHeader {
                            title: "Developer portal".to_string(),
                            description: Some("Developer API management is not currently available.".to_string()),
                            icon: Some("code".to_string()),
                        }
                        DeveloperOverviewUnavailable {}
                    }
                }
            }
        }
    }
}

/// `DeveloperUsageBody` — authentication-only usage state.
#[component]
fn DeveloperUsageBody(ctx: PageContext) -> Element {
    rsx! {
        MainLayout { ctx: ctx.clone(),
            AuthGate {
                user: ctx.user.clone(),
                feature: Some("API usage".to_string()),
                return_url: Some(ctx.path.clone()),
                DeveloperShell { current_path: ctx.path.clone(),
                    div { class: "developer-usage-prod container page-content space-y-6",
                        PageHeader {
                            title: "API usage".to_string(),
                            description: Some("Usage reporting is not currently available.".to_string()),
                            icon: Some("chart-line".to_string()),
                        }
                        DeveloperUsageUnavailable {}
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
/// Wave 22 T4 — replaces the 3-card stub (auth/payments/
/// subscriptions) with the real `ENDPOINT_CATEGORIES` (4
/// categories, 10 endpoints) ported from
/// `apps-old/frontend/components/developer/docs/data/endpoints.ts`.
/// Renders a sidebar (categories nav + quick-start card) on the
/// left and a stacked list of `EndpointCard` components on the
/// right. Each card is collapsible with a click on the header
/// row. Mirrors the `endpoint-card.tsx` + `docs-sidebar.tsx` +
/// `api-docs.tsx` source structure.
pub fn render_docs(ctx: &PageContext) -> (PageMeta, Element) {
    // The pinned page has no route-owned metadata and therefore inherits the
    // exact root metadata from `app/layout.tsx`.
    let mut meta = PageMeta::app("API documentation");
    meta.title = "EPSX - Stock Analytics Platform".to_string();
    meta.description = "Advanced stock data analytics platform".to_string();
    // Next's metadata generator serializes keyword arrays with `join(',')`.
    meta.keywords = Some("stock analytics,financial data,EPSX,market insights".to_string());
    let categories = cached_endpoint_categories();

    (meta, rsx! {
        MainLayout { ctx: ctx.clone(),
            DeveloperShell { current_path: ctx.path.clone(),
                div {
                    class: "developer-docs-page container page-content",
                    "data-docs-source-baseline": DEVELOPER_DOCS_SOURCE_BASELINE,
                    // 8. Endpoints sidebar + endpoint cards.
                    div { class: "developer-docs flex gap-6",
                        "data-section": "developer-docs",
                        DocsSidebar { categories: categories.clone() }
                        div { class: "min-w-0 flex-1 space-y-8",
                            // Hero
                            div { class: "developer-docs-hero mb-8",
                                div { class: "h-[3px] w-16 rounded-full bg-gradient-to-r from-[#7645d9] to-[#1fc7d4]" }
                                h1 { class: "mt-3 text-3xl font-bold text-foreground", "API Reference" }
                                p { class: "mt-2 text-muted-foreground",
                                    "Pinned migration reference for reviewing endpoint, schema, and example structure."
                                }
                            }
                            aside {
                                class: "developer-docs-reference-warning rounded-2xl border border-amber-500/40 bg-amber-500/10 p-5",
                                "data-docs-reference-warning": "true",
                                role: "note",
                                "aria-labelledby": "developer-docs-reference-warning-title",
                                h2 {
                                    id: "developer-docs-reference-warning-title",
                                    class: "text-base font-semibold text-foreground",
                                    "Pinned migration reference"
                                }
                                p { class: "mt-2 text-sm text-muted-foreground",
                                    "Endpoint, authentication, tier, rate-limit, schema, and example content below comes from the pinned migration source "
                                    code { class: "font-mono text-xs", "{DEVELOPER_DOCS_SOURCE_BASELINE}" }
                                    ". It is not a verified production contract. Do not use real credentials."
                                }
                            }
                            // Auth guide card
                            div { class: "developer-docs-auth-card rounded-2xl border border-border/20 bg-card p-5 shadow-xl",
                                h3 { class: "text-sm font-semibold text-foreground", "Authentication" }
                                p { class: "mt-1 text-sm text-muted-foreground",
                                    "The pinned reference shows an "
                                    code { class: "rounded bg-background px-1.5 py-0.5 text-xs", "Authorization: Bearer <token>" }
                                    " header. Accepted credential types and middleware behavior are not verified here."
                                }
                                pre { class: "developer-docs-curl mt-3 rounded-xl bg-slate-900 p-3 font-mono text-xs text-gray-300",
                                    "curl -H \"Authorization: Bearer YOUR_API_KEY\" https://api.example.invalid/api/analytics/rankings"
                                }
                            }
                            // Endpoint sections
                            div { class: "space-y-10",
                                for cat in categories.iter() {
                                    EndpointSection { category: cat.clone() }
                                }
                            }
                        }
                    }
                }
            }
        }
    })
}

/// `DocsSidebar` — left rail with category links + a quick-start
/// card. Mirrors `apps-old/frontend/components/developer/docs/docs-sidebar.tsx`.
#[component]
fn DocsSidebar(categories: Vec<EndpointCategory>) -> Element {
    rsx! {
        button {
            r#type: "button",
            class: "docs-sidebar-toggle",
            "data-docs-sidebar-toggle": "true",
            "aria-controls": "developer-docs-sidebar",
            "aria-expanded": "false",
            "aria-label": "Open API reference navigation",
            span { "☰" }
        }
        aside {
            id: "developer-docs-sidebar",
            class: "docs-sidebar",
            "data-docs-sidebar": "true",
            "aria-label": "API reference sections",
            div { class: "px-4 py-4",
                h3 { class: "mb-3 text-xs font-semibold uppercase tracking-wider text-muted-foreground", "API Reference" }
                nav { class: "space-y-1", "aria-label": "Endpoint categories",
                    for cat in categories.iter() {
                        a {
                            class: "docs-sidebar-link flex w-full items-center gap-2 rounded-lg px-3 py-2 text-left text-sm text-muted-foreground transition-colors hover:bg-background hover:text-foreground",
                            href: "#section-{cat.id}",
                            "data-docs-section-link": "{cat.id}",
                            key: "{cat.id}",
                            span { class: "h-1.5 w-1.5 rounded-full bg-current opacity-50" }
                            "{cat.title}"
                            span { class: "ml-auto text-xs text-muted-foreground/50", "{cat.endpoints.len()}" }
                        }
                    }
                }
            }
            // Quick start card
            div { class: "docs-sidebar-quickstart mx-1 mt-4 rounded-xl border border-border/10 bg-background p-3",
                p { class: "text-xs font-medium text-foreground", "Header reference" }
                p { class: "mt-1 text-[11px] leading-relaxed text-muted-foreground",
                    "Pinned Bearer-header example only. Do not use real credentials."
                }
                code { class: "mt-2 block rounded-lg bg-slate-900 p-2 font-mono text-[10px] text-gray-300",
                    "Authorization: Bearer <key>"
                }
            }
        }
        button {
            r#type: "button",
            class: "docs-sidebar-overlay",
            "data-docs-sidebar-overlay": "true",
            "aria-label": "Close API reference navigation",
            hidden: true,
        }
    }
}

/// `EndpointSection` — one section per category. Mirrors
/// `apps-old/frontend/components/developer/docs/endpoint-section.tsx`.
#[component]
fn EndpointSection(category: EndpointCategory) -> Element {
    rsx! {
        section { class: "docs-endpoint-section space-y-4", id: "section-{category.id}",
            key: "{category.id}",
            div { class: "docs-endpoint-section-header",
                h2 { class: "text-2xl font-bold text-foreground", "{category.title}" }
                p { class: "mt-1 text-sm text-muted-foreground", "{category.desc}" }
            }
            for ep in category.endpoints.iter() {
                EndpointCard { endpoint: ep.clone() }
            }
        }
    }
}

/// `EndpointCard` — collapsible card per endpoint. Mirrors
/// `apps-old/frontend/components/developer/docs/endpoint-card.tsx`.
/// The header row shows the method badge, the path, the tier
/// badge, and a chevron. Click toggles the expanded body which
/// renders params table, rate limits, example curl, and the
/// response example.
#[component]
fn EndpointCard(endpoint: EndpointDef) -> Element {
    let method_cls = method_color_class(&endpoint.method);
    let tier_cls = tier_color_class(&endpoint.tier);
    let card_id = format!(
        "docs-endpoint-{}-{}",
        endpoint.method.to_ascii_lowercase(),
        endpoint.path.trim_matches('/').replace('/', "-")
    );
    let body_id = format!("{card_id}-body");
    let curl = code_snippet(&endpoint, "curl");
    let javascript = code_snippet(&endpoint, "javascript");
    let python = code_snippet(&endpoint, "python");
    let response = pretty_response(&endpoint.response_example);
    rsx! {
        article { class: "docs-endpoint-card rounded-2xl border border-border/20 bg-card shadow-xl",
            id: "{card_id}",
            key: "{endpoint.method}-{endpoint.path}",
            button {
                r#type: "button",
                class: "flex w-full items-center gap-3 px-5 py-4 text-left",
                "data-docs-endpoint-toggle": "true",
                "aria-expanded": "false",
                "aria-controls": "{body_id}",
                span { class: "rounded-lg px-2.5 py-1 text-xs font-bold {method_cls}", "{endpoint.method}" }
                code { class: "flex-1 font-mono text-sm text-foreground", "{endpoint.path}" }
                span { class: "inline-flex items-center rounded-full border px-2 py-0.5 text-xs font-medium capitalize {tier_cls}", "{endpoint.tier}" }
                span { class: "docs-endpoint-card-chevron h-4 w-4 text-muted-foreground", "aria-hidden": "true", "▸" }
            }
            div {
                id: "{body_id}",
                class: "docs-endpoint-card-body border-t border-border/10 px-5 py-4 space-y-5",
                hidden: true,
                p { class: "text-sm text-muted-foreground", "{endpoint.desc}" }
                // Params table
                if !endpoint.params.is_empty() {
                    div { class: "docs-endpoint-card-params",
                        h4 { class: "mb-2 text-xs font-semibold uppercase tracking-wider text-muted-foreground", "Parameters" }
                        div { class: "overflow-x-auto rounded-xl border border-border/10",
                            table { class: "w-full text-sm",
                                thead {
                                    tr { class: "border-b border-border/10 text-left text-xs text-muted-foreground",
                                        th { class: "px-3 py-2 font-medium", "Name" }
                                        th { class: "px-3 py-2 font-medium", "Type" }
                                        th { class: "px-3 py-2 font-medium", "Required" }
                                        th { class: "px-3 py-2 font-medium", "Description" }
                                    }
                                }
                                tbody {
                                    for p in endpoint.params.iter() {
                                        tr { class: "border-b border-border/5",
                                            td { class: "px-3 py-2 font-mono text-xs text-foreground", "{p.name}" }
                                            td { class: "px-3 py-2 text-xs text-muted-foreground", "{p.kind}" }
                                            td { class: "px-3 py-2",
                                                if p.required {
                                                    span { class: "text-xs text-red-400", "yes" }
                                                } else {
                                                    span { class: "text-xs text-muted-foreground/50", "no" }
                                                }
                                            }
                                            td { class: "px-3 py-2 text-xs text-muted-foreground",
                                                "{p.desc}"
                                                if let Some(default) = &p.default { " (default: {default})" }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                // Rate limits
                div { class: "docs-endpoint-card-rate-limits",
                    h4 { class: "mb-2 text-xs font-semibold uppercase tracking-wider text-muted-foreground", "Rate Limits" }
                    div { class: "flex flex-wrap gap-2",
                        for (tier, limit) in endpoint.rate_limits.iter() {
                            span { class: "rounded-lg bg-background px-2.5 py-1 text-xs text-muted-foreground",
                                span { class: "capitalize", "{tier}" }
                                ": "
                                span { class: "font-mono", "{limit}" }
                            }
                        }
                    }
                }
                // Example
                div { class: "docs-endpoint-card-example",
                    h4 { class: "mb-2 text-xs font-semibold uppercase tracking-wider text-muted-foreground", "Example" }
                    div { class: "docs-code-example",
                        div {
                            class: "docs-code-toolbar",
                            role: "tablist",
                            "aria-label": "Code language",
                            for (lang, label) in [("curl", "cURL"), ("javascript", "JavaScript"), ("python", "Python")] {
                                button {
                                    r#type: "button",
                                    class: if lang == "curl" { "docs-code-tab active" } else { "docs-code-tab" },
                                    role: "tab",
                                    "data-docs-code-tab": "{lang}",
                                    "aria-selected": if lang == "curl" { "true" } else { "false" },
                                    tabindex: if lang == "curl" { "0" } else { "-1" },
                                    "{label}"
                                }
                            }
                            button { r#type: "button", class: "docs-copy-button", "data-docs-copy-code": "true", span { "Copy" } }
                        }
                        pre { class: "docs-code-panel", "data-docs-code-panel": "curl", code { "{curl}" } }
                        pre { class: "docs-code-panel", "data-docs-code-panel": "javascript", hidden: true, code { "{javascript}" } }
                        pre { class: "docs-code-panel", "data-docs-code-panel": "python", hidden: true, code { "{python}" } }
                    }
                }
                // Response example
                div { class: "docs-endpoint-card-response",
                    h4 { class: "mb-2 text-xs font-semibold uppercase tracking-wider text-muted-foreground", "Response" }
                    div { class: "docs-response-example",
                        button { r#type: "button", class: "docs-copy-button docs-response-copy", "data-docs-copy-response": "true", span { "Copy" } }
                        pre { class: "docs-response-panel", code { "{response}" } }
                    }
                }

                div {
                    class: "docs-live-request-unavailable rounded-2xl border border-border/20 bg-card p-4 shadow-xl",
                    "data-docs-live-request-notice": "true",
                    role: "note",
                    "aria-label": "Live requests unavailable",
                    h4 { class: "text-sm font-semibold text-foreground", "Live requests unavailable" }
                    p { class: "mt-1 text-sm text-muted-foreground",
                        "This endpoint is reference-only. It does not accept credentials or send a request."
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn empty_ctx() -> PageContext {
        PageContext {
            path: "/developer".to_string(),
            ..Default::default()
        }
    }

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

    fn usage_ctx() -> PageContext {
        let mut ctx = authed_ctx();
        ctx.path = "/developer/usage".to_string();
        ctx
    }

    fn render_overview_to_string(ctx: &PageContext) -> String {
        let (_, element) = render_overview(ctx);
        dioxus_ssr::render_element(element)
    }

    #[test]
    fn developer_overview_authenticated_state_is_truthfully_unavailable() {
        let mut ctx = authed_ctx();
        ctx.user
            .as_mut()
            .expect("authenticated fixture")
            .permissions
            .clear();
        let html = render_overview_to_string(&ctx);

        assert!(html.contains("developer-overview-unavailable"));
        assert!(html.contains("Developer tools unavailable"));
        assert!(html.contains("No keys, secrets, plan assignments, permissions, usage, rate limits, or expiration values are shown."));
        assert!(!html.contains("Permission required"));
    }

    #[test]
    fn developer_overview_ignores_payload_fixtures_secrets_and_business_claims() {
        let mut ctx = authed_ctx();
        ctx.params.insert(
            "data_developer".to_string(),
            r#"{
                "stats": {
                    "tier": "fixture-tier-probe",
                    "rate_limit": "fixture-rate-probe",
                    "total_usage": 987654321,
                    "expires": "fixture-expiry-probe"
                },
                "api_keys": [{
                    "id": "fixture-key-id",
                    "name": "fixture-key-name",
                    "key": "epsx_secret_overview_probe",
                    "scopes": ["backend:permission:probe"],
                    "is_active": true,
                    "created_at": "fixture-created-probe",
                    "usage_count": 7654321
                }]
            }"#
            .to_string(),
        );
        let html = render_overview_to_string(&ctx);

        for forbidden in [
            "fixture-tier-probe",
            "fixture-rate-probe",
            "987654321",
            "fixture-expiry-probe",
            "fixture-key-id",
            "fixture-key-name",
            "epsx_secret_overview_probe",
            "backend:permission:probe",
            "fixture-created-probe",
            "7654321",
            "epsx_live_4f8a2c1b9d3e7f5a",
            "170,414",
            "1000/min",
            "50,000/day",
            "2026-08-15",
            "288 days left",
            "Pro, Enterprise",
        ] {
            assert!(
                !html.contains(forbidden),
                "rendered overview fixture or claim: {forbidden}"
            );
        }
    }

    #[test]
    fn developer_overview_unavailable_state_is_accessible_and_native() {
        let html = dioxus_ssr::render_element(rsx! { DeveloperOverviewUnavailable {} });

        assert!(html.contains("role=\"status\""));
        assert!(html.contains("aria-live=\"polite\""));
        assert!(html.contains("aria-labelledby=\"developer-overview-unavailable-title\""));
        assert!(html.contains("href=\"/developer\""));
        assert!(html.contains("href=\"/developer/docs\""));
        assert!(html.contains(">Retry</a>"));
        assert!(html.contains(">Read API documentation</a>"));

        for control in ["<button", "<form", "<input", "onclick=", "oninput="] {
            assert!(
                !html.contains(control),
                "rendered local mutation control: {control}"
            );
        }
        for mutation in [
            "Create key",
            "Create API Key",
            "Revoke Key",
            "Refresh",
            "30 Days",
            "90 Days",
            "1 Year",
        ] {
            assert!(
                !html.contains(mutation),
                "rendered unsupported mutation: {mutation}"
            );
        }
    }

    #[test]
    fn developer_overview_signed_out_uses_native_auth_gate() {
        let html = render_overview_to_string(&empty_ctx());

        assert!(html.contains("class=\"auth-gate "));
        assert!(html.contains("role=\"alert\""));
        assert!(html.contains("Sign in required"));
        assert!(html.contains("href=\"/auth?return_url=%2Fdeveloper\""));
        assert!(!html.contains("developer-overview-unavailable"));
    }

    fn render_usage_to_string(ctx: &PageContext) -> String {
        let (_, element) = render_usage(ctx);
        dioxus_ssr::render_element(element)
    }

    #[test]
    fn developer_usage_authenticated_state_is_truthfully_unavailable() {
        let mut ctx = usage_ctx();
        ctx.user
            .as_mut()
            .expect("authenticated fixture")
            .permissions
            .clear();
        let html = render_usage_to_string(&ctx);

        assert!(html.contains("developer-usage-unavailable"));
        assert!(html.contains("Usage data unavailable"));
        assert!(html.contains(
            "No request counts, limits, reliability figures, or activity rows are shown."
        ));
        assert!(!html.contains("Permission required"));
    }

    #[test]
    fn developer_usage_ignores_payloads_and_renders_no_metrics_or_sample_rows() {
        let mut ctx = usage_ctx();
        ctx.params.insert(
            "data_developer_usage".to_string(),
            r#"{
                "summary": {
                    "calls_today": 12481,
                    "calls_30d": 358910,
                    "error_rate": "0.42%",
                    "success_rate": "99.6%",
                    "uptime": "99.99%"
                },
                "per_key": [{
                    "name": "Production",
                    "key": "epsx_secret_usage_probe",
                    "calls_today": 8231
                }],
                "history": [{"date": "2025-01-15", "calls": 9812}]
            }"#
            .to_string(),
        );
        let html = render_usage_to_string(&ctx);

        for forbidden in [
            "170414",
            "2891",
            "0.42%",
            "99.6%",
            "99.99%",
            "1000/min",
            "234/min",
            "12481",
            "358910",
            "8231",
            "9812",
            "Production",
            "epsx_secret_usage_probe",
            "Usage History",
            "Usage by API Key",
            "Top Endpoints",
            "usage-monitor",
        ] {
            assert!(
                !html.contains(forbidden),
                "rendered usage fixture or claim: {forbidden}"
            );
        }
    }

    #[test]
    fn developer_usage_unavailable_state_is_accessible_and_native() {
        let html = dioxus_ssr::render_element(rsx! { DeveloperUsageUnavailable {} });

        assert!(html.contains("role=\"status\""));
        assert!(html.contains("aria-live=\"polite\""));
        assert!(html.contains("aria-labelledby=\"developer-usage-unavailable-title\""));
        assert!(html.contains("href=\"/developer/usage\""));
        assert!(html.contains("href=\"/developer/docs\""));
        assert!(html.contains(">Retry</a>"));
        assert!(html.contains(">Read API documentation</a>"));
        for control in ["<button", "<form", "<input", "onclick="] {
            assert!(!html.contains(control), "rendered inert control: {control}");
        }
    }

    #[test]
    fn developer_usage_signed_out_uses_native_auth_gate() {
        let ctx = PageContext {
            path: "/developer/usage".to_string(),
            ..Default::default()
        };
        let html = render_usage_to_string(&ctx);

        assert!(html.contains("class=\"auth-gate "));
        assert!(html.contains("role=\"alert\""));
        assert!(html.contains("Sign in required"));
        assert!(html.contains("href=\"/auth?return_url=%2Fdeveloper%2Fusage\""));
        assert!(!html.contains("developer-usage-unavailable"));
        for simulated_wallet_ui in ["Select Wallet", "WalletConnect", "Base Account"] {
            assert!(!html.contains(simulated_wallet_ui));
        }
    }

    /// Wave 22 T4 — `test_docs_categories`. Renders the
    /// /developer/docs body and asserts the real `ENDPOINT_CATEGORIES`
    /// (4 categories, auth + analytics) are present, the auth
    /// card is rendered, and a real endpoint path shows up.
    #[test]
    fn test_docs_categories() {
        let ctx = authed_ctx();
        let (meta, el) = render_docs(&ctx);
        let html = dioxus_ssr::render_element(el);
        assert_eq!(meta.title, "EPSX - Stock Analytics Platform");
        assert_eq!(meta.description, "Advanced stock data analytics platform");
        assert_eq!(
            meta.keywords.as_deref(),
            Some("stock analytics,financial data,EPSX,market insights")
        );
        assert!(html.contains(DEVELOPER_DOCS_SOURCE_BASELINE));
        // 4 category titles from ENDPOINT_CATEGORIES. The `&` is
        // HTML-encoded by dioxus_ssr as `&#38;`, so we check for
        // the encoded form for "Portfolio & Watchlist".
        for title in &["Authentication", "Analytics", "Portfolio &#38; Watchlist", "User"] {
            assert!(
                html.contains(title),
                "docs page should render category title `{title}`. Got (truncated): {}",
                &html[..html.len().min(2000)]
            );
        }
        // Auth card is rendered.
        assert!(
            html.contains("developer-docs-auth-card"),
            "docs page should render the auth card"
        );
        // Real endpoint path from the catalog shows up.
        assert!(
            html.contains("/api/auth/session/verify"),
            "docs page should render a real endpoint path from the catalog"
        );
        // Quick-start card.
        assert!(
            html.contains("docs-sidebar-quickstart"),
            "docs page should render the quick-start sidebar card"
        );
        assert_eq!(html.matches("docs-endpoint-card rounded-2xl").count(), 10);
        assert_eq!(html.matches("data-docs-endpoint-toggle=\"true\"").count(), 10);
        assert_eq!(html.matches("docs-endpoint-card-params").count(), 3);
        assert_eq!(html.matches("docs-endpoint-card-rate-limits").count(), 10);
        assert_eq!(html.matches("data-docs-code-tab=").count(), 30);
        assert_eq!(html.matches("data-docs-code-panel=").count(), 30);
        assert_eq!(html.matches("data-docs-copy-code=\"true\"").count(), 10);
        assert_eq!(html.matches("data-docs-copy-response=\"true\"").count(), 10);
        assert_eq!(html.matches("docs-response-panel").count(), 10);
        assert_eq!(html.matches("data-docs-reference-warning=\"true\"").count(), 1);
        assert_eq!(html.matches("data-docs-live-request-notice=\"true\"").count(), 10);
        assert_eq!(html.matches("role=\"note\"").count(), 11);
        assert!(html.contains("It is not a verified production contract."));
        assert!(html.contains("Do not use real credentials."));
        assert_eq!(html.matches("Live requests unavailable").count(), 20);
        assert_eq!(html.matches("https://api.example.invalid").count(), 31);
        assert!(html.contains(
            "curl -H &#34;Authorization: Bearer YOUR_API_KEY&#34; https://api.example.invalid/api/analytics/rankings"
        ));
        assert!(!html.contains("api.epsx.io"));
        assert!(!html.contains("same endpoints, same data"));
        assert!(!html.contains("works like a JWT"));
        for removed_control in [
            "docs-field-control",
            "<select",
            "<input",
            "docs-send-button",
            "Try it",
            "Try It",
            "Send Request",
            "No key",
            "-api-key\"",
            "<form",
            "docs-request-executor",
            "data-docs-fetch",
        ] {
            assert!(
                !html.contains(removed_control),
                "docs page retained live-request control or executor: {removed_control}"
            );
        }
        // The ten JavaScript fetch calls remain inert text inside the preserved
        // language examples; there is no form, field, send control, or executor.
        assert_eq!(html.matches("const res = await fetch").count(), 10);
        assert!(!html.contains("REST endpoints, request/response schemas, and examples"));
        assert!(!html.contains("API documentation</h1>"));
    }

    /// Wave 22 T4 — `test_endpoint_catalog_units`. Cached catalog
    /// must have 4 categories, 10 endpoints, and
    /// contain a `param()` helper signature.
    #[test]
    fn test_endpoint_catalog_units() {
        let cats = cached_endpoint_categories();
        assert_eq!(cats.len(), 4, "expected 4 endpoint categories");
        let auth = cats.iter().find(|c| c.id == "auth").expect("auth category");
        assert_eq!(auth.title, "Authentication");
        assert!(auth.endpoints.iter().any(|e| e.path == "/api/auth/session/verify"));
        // Auth category has 1 endpoint, Analytics has 4, Portfolio has 3, User has 2.
        let analytics = cats.iter().find(|c| c.id == "analytics").expect("analytics category");
        assert_eq!(analytics.endpoints.len(), 4);
        let portfolio = cats.iter().find(|c| c.id == "portfolio").expect("portfolio category");
        assert_eq!(portfolio.endpoints.len(), 3);
        let user = cats.iter().find(|c| c.id == "user").expect("user category");
        assert_eq!(user.endpoints.len(), 2);
        // Total = 1 + 4 + 3 + 2 = 10 in the pinned source.
        let total: usize = cats.iter().map(|c| c.endpoints.len()).sum();
        assert_eq!(total, 10, "endpoint catalog must keep the exact pinned endpoint count");

        // param() helper unit-check.
        let p = EndpointParam::param("ticker", "string", true, "test");
        assert_eq!(p.name, "ticker");
        assert_eq!(p.kind, "string");
        assert!(p.required);
        assert_eq!(p.desc, "test");
        assert!(p.default.is_none());

        // method_color_class + tier_color_class return non-empty strings.
        assert!(!method_color_class("GET").is_empty());
        assert!(!method_color_class("POST").is_empty());
        assert!(!method_color_class("DELETE").is_empty());
        assert!(!tier_color_class("free").is_empty());
        assert!(!tier_color_class("enterprise").is_empty());
    }

    #[test]
    fn developer_docs_catalog_matches_pinned_visible_contract() {
        let cats = cached_endpoint_categories();
        let ids: Vec<&str> = cats.iter().map(|category| category.id.as_str()).collect();
        assert_eq!(ids, vec!["auth", "analytics", "portfolio", "user"]);
        let endpoints: Vec<(&str, &str)> = cats
            .iter()
            .flat_map(|category| category.endpoints.iter())
            .map(|endpoint| (endpoint.method.as_str(), endpoint.path.as_str()))
            .collect();
        assert_eq!(
            endpoints,
            vec![
                ("GET", "/api/auth/session/verify"),
                ("GET", "/api/analytics/rankings"),
                ("GET", "/api/analytics/filters"),
                ("GET", "/api/analytics/countries"),
                ("GET", "/api/analytics/sectors"),
                ("GET", "/api/users/watchlist"),
                ("POST", "/api/users/watchlist"),
                ("DELETE", "/api/users/watchlist"),
                ("GET", "/api/users/profile"),
                ("GET", "/api/users/access-overview"),
            ]
        );
        let rankings = &cats[1].endpoints[0];
        let defaults: Vec<Option<&str>> = rankings
            .params
            .iter()
            .take(4)
            .map(|param| param.default.as_deref())
            .collect();
        assert_eq!(
            defaults,
            vec![Some("1"), Some("20"), Some("eps_growth"), Some("desc")]
        );
    }

    #[test]
    fn developer_docs_code_examples_use_non_routable_reference_origin() {
        let post = &cached_endpoint_categories()[2].endpoints[1];
        assert_eq!(
            code_snippet(post, "curl"),
            "curl -X POST \"https://api.example.invalid/api/users/watchlist\" \\\n  -H \"Authorization: Bearer YOUR_API_KEY\" \\\n  -H \"Content-Type: application/json\" \\\n  -d '{\"ticker\": \"AAPL\"}'"
        );
        assert_eq!(
            code_snippet(post, "javascript"),
            "const res = await fetch('https://api.example.invalid/api/users/watchlist', {\n  method: 'POST',\n  headers: { 'Authorization': 'Bearer YOUR_API_KEY' },\n  body: JSON.stringify({ ticker: 'AAPL' })\n});\nconst data = await res.json();"
        );
        assert_eq!(
            code_snippet(post, "python"),
            "import requests\n\nurl = \"https://api.example.invalid/api/users/watchlist\"\nheaders = {\"Authorization\": \"Bearer YOUR_API_KEY\"}\nres = requests.post(url, headers=headers, json={\"ticker\": \"AAPL\"})\ndata = res.json()"
        );
        let pretty = pretty_response(&post.response_example);
        assert!(pretty.contains("\n  \"success\": true"));
    }
}
