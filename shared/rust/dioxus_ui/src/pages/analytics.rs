//! /analytics — analytics dashboard.
//!
//! Wave 6A Track B port: brings the page from a thin shell (78 LoC) to a
//! section-level port of the Next.js source (`apps/frontend/app/analytics/page.tsx`
//! 84 LoC + 19 sub-components ~2,951 LoC; design doc target ≥600 LoC).
//!
//! Sections (all in render order, matching the source layout):
//!
//! 1. `AnalyticsHeader` — page header with title, plan badge, live/AI
//!    pills, date range, "Export" trigger, filter toggle.
//!    Source: `app/analytics/page.tsx` lines 44-64 +
//!    `analytics-dashboard-wrapper.tsx` header.
//! 2. `AnalyticsPlanStatusBar` — current plan + visible rank range +
//!    upgrade CTA. Source: `plan-status-bar.tsx` 194 LoC.
//! 3. `FilterPanel` — left sidebar with sort / country / sector /
//!    min-eps / min-growth filters. Source: `filter-panel.tsx` 175 LoC
//!    + `filter-form.tsx` 187 LoC.
//! 4. `AnalyticsCardGrid` — 2-col responsive grid of stock ranking
//!    cards. Source: `analytics-card-grid.tsx` 75 LoC + data wiring
//!    from `card-dashboard-view.tsx` 133 LoC.
//! 5. `AnalyticsChart` — large time-series chart (uses `ChartLine` +
//!    `ChartBar` primitives). Source: `analytics-dashboard.tsx`.
//! 6. `AnalyticsTable` — paginated table of recent events (uses
//!    `DataTable` + `Pagination` primitives). Source:
//!    `analytics-dashboard.tsx` + `pagination.tsx` 144 LoC.
//! 7. `AnalyticsExportDialog` — modal with format selector (CSV, JSON,
//!    Parquet) + download button. Source:
//!    `analytics-export-dialog.tsx` 180 LoC. Backed by the new
//!    `<ExportDialog>` primitive (`data/export_dialog.rs`).
//! 8. `AnalyticsMetadata` — last refresh, processing time, data
//!    source, attribution. Source:
//!    `analytics-metadata-display.tsx` 152 LoC.
//!
//! Section markers (used by `tests::test_section_markers`):
//!   - `analytics-header`
//!   - `analytics-plan-status`
//!   - `analytics-filter-panel`
//!   - `analytics-card-grid`
//!   - `analytics-chart`
//!   - `analytics-table`
//!   - `analytics-export-dialog`
//!   - `analytics-metadata`

use crate::data::export_dialog::{ExportDialog, ExportFormat};
use crate::data::pagination::Pagination;
use crate::data_table::{Column, DataTable, Row};
use crate::feedback::*;
use crate::primitives::*;

use dioxus::prelude::*;
use super::PageContext;
use super::PageMeta;
use crate::layout::main_layout::MainLayout;
use crate::layout::PageHeader;
use crate::auth::AuthGate;
use crate::charts::{ChartBar, ChartLine, DataPoint, Series};

/// Sample ranked stock cards — these mirror the `SymbolCardData` shape
/// from the source. In a real BFF render the data is plumbed from
/// `getDashboardInitAction`; here we use static fixtures so the page
/// is deterministic and unit-test friendly.
fn sample_rankings() -> Vec<AnalyticsRankingCard> {
    vec![
        AnalyticsRankingCard {
            rank: 1,
            symbol: "AAPL".into(),
            company_name: "Apple Inc.".into(),
            eps_growth: 24.5,
            price: 192.34,
            currency: "USD".into(),
            tier_label: "Premium".into(),
        },
        AnalyticsRankingCard {
            rank: 2,
            symbol: "MSFT".into(),
            company_name: "Microsoft Corp.".into(),
            eps_growth: 21.2,
            price: 412.05,
            currency: "USD".into(),
            tier_label: "Premium".into(),
        },
        AnalyticsRankingCard {
            rank: 3,
            symbol: "NVDA".into(),
            company_name: "NVIDIA Corp.".into(),
            eps_growth: 56.8,
            price: 845.10,
            currency: "USD".into(),
            tier_label: "Premium".into(),
        },
        AnalyticsRankingCard {
            rank: 4,
            symbol: "GOOGL".into(),
            company_name: "Alphabet Inc.".into(),
            eps_growth: 18.4,
            price: 168.20,
            currency: "USD".into(),
            tier_label: "Premium".into(),
        },
        AnalyticsRankingCard {
            rank: 5,
            symbol: "META".into(),
            company_name: "Meta Platforms".into(),
            eps_growth: 31.7,
            price: 480.55,
            currency: "USD".into(),
            tier_label: "Premium".into(),
        },
        AnalyticsRankingCard {
            rank: 6,
            symbol: "TSLA".into(),
            company_name: "Tesla Inc.".into(),
            eps_growth: -4.1,
            price: 192.10,
            currency: "USD".into(),
            tier_label: "Standard".into(),
        },
    ]
}

#[derive(Clone, Debug, PartialEq)]
struct AnalyticsRankingCard {
    rank: u32,
    symbol: String,
    company_name: String,
    eps_growth: f64,
    price: f64,
    currency: String,
    tier_label: String,
}

/// Sample recent events for the events table. Mirrors
/// `SymbolCardData.quarterly_performance` shape.
fn sample_events() -> Vec<(String, String, String, String, String)> {
    vec![
        ("AAPL".into(), "Q3 2025 EPS beat".into(), "2025-10-30".into(), "Bullish".into(), "+12.4%".into()),
        ("MSFT".into(), "Q3 2025 revenue".into(), "2025-10-25".into(), "Bullish".into(), "+8.2%".into()),
        ("NVDA".into(), "AI guidance raise".into(), "2025-10-18".into(), "Bullish".into(), "+24.1%".into()),
        ("GOOGL".into(), "Search ad growth".into(), "2025-10-12".into(), "Neutral".into(), "+1.3%".into()),
        ("META".into(), "Reality Labs loss".into(), "2025-10-08".into(), "Bearish".into(), "-3.2%".into()),
        ("TSLA".into(), "Delivery miss".into(), "2025-10-02".into(), "Bearish".into(), "-7.5%".into()),
    ]
}

/// PnL over time series (matches `CardDashboardView` chart wiring).
fn sample_pnl_series() -> Vec<Series> {
    let points: Vec<DataPoint> = (0..30)
        .map(|i| DataPoint {
            x: i as f64,
            y: 100.0 + (i as f64 * 1.4) + (i as f64 * 0.55).sin() * 6.0,
            label: None,
        })
        .collect();
    vec![Series {
        name: "P&L".into(),
        color: "#22d3ee".into(),
        points,
    }]
}

/// Volume by day series.
fn sample_volume_series() -> Vec<(String, f64)> {
    (0..7)
        .map(|i| (format!("D{}", i + 1), 100.0 + (i as f64 * 30.0) + (i as f64 * 0.9).sin() * 40.0))
        .collect()
}

// ─────────────────────────────────────────────────────────────────────────
// Section sub-components — each `#[component] fn` corresponds to one
// design-doc section. Keeping them small and named makes the page
// `render()` self-documenting and gives the test layer stable
// section-marker class names.
// ─────────────────────────────────────────────────────────────────────────

/// `AnalyticsHeader` — title, plan badge, live/AI pills, date range,
/// "Export" trigger, filter toggle.
#[component]
fn AnalyticsHeader(
    on_export_click: EventHandler,
    on_filter_toggle: EventHandler,
    filters_active: bool,
) -> Element {
    rsx! {
        div {
            class: "analytics-header flex flex-col gap-4 sm:flex-row sm:items-center sm:justify-between mb-6",
            "data-section": "analytics-header",
            div { class: "flex items-center gap-3",
                div { class: "flex h-10 w-10 items-center justify-center rounded-xl bg-gradient-to-br from-purple-500 to-pink-500",
                    Icon { name: "bar-chart-3".to_string(), size: Some(20), class_name: Some("text-white".to_string()) }
                }
                div {
                    h1 { class: "text-2xl font-bold text-foreground", "Analytics" }
                    p { class: "text-sm text-slate-400", "Top-performing stocks by EPS growth" }
                }
            }
            div { class: "flex flex-wrap gap-2",
                // Date range picker
                div { class: "analytics-header-date flex items-center gap-1.5 rounded-lg border border-border/30 bg-card px-3 py-1.5",
                    Icon { name: "calendar".to_string(), size: Some(14), class_name: Some("text-muted-foreground".to_string()) }
                    span { class: "text-xs font-medium text-muted-foreground", "Last 30 days" }
                }
                // Live pill
                div { class: "flex items-center gap-1.5 rounded-lg border border-emerald-500/20 bg-emerald-500/10 px-3 py-1.5",
                    Icon { name: "trending-up".to_string(), size: Some(14), class_name: Some("text-emerald-400".to_string()) }
                    span { class: "text-xs font-medium text-emerald-400", "Live" }
                }
                // AI-powered pill
                div { class: "flex items-center gap-1.5 rounded-lg border border-purple-500/20 bg-purple-500/10 px-3 py-1.5",
                    Icon { name: "sparkles".to_string(), size: Some(14), class_name: Some("text-purple-400".to_string()) }
                    span { class: "text-xs font-medium text-purple-400", "AI-Powered" }
                }
                // Export trigger
                button {
                    r#type: "button",
                    class: "analytics-header-export inline-flex items-center gap-1.5 rounded-lg border border-purple-200 bg-white/80 px-3 py-1.5 text-xs font-semibold text-purple-700 shadow-sm hover:bg-purple-50",
                    onclick: move |_| on_export_click.call(()),
                    Icon { name: "download".to_string(), size: Some(14) }
                    "Export"
                }
                // Filter toggle
                button {
                    r#type: "button",
                    class: "analytics-header-filters inline-flex items-center gap-1.5 rounded-lg border border-border/30 bg-card px-3 py-1.5 text-xs font-medium",
                    onclick: move |_| on_filter_toggle.call(()),
                    Icon { name: "filter".to_string(), size: Some(14) }
                    "Filters"
                    if filters_active { span { class: "analytics-header-filters-badge ml-1 rounded-full bg-purple-500/20 px-1.5 py-0.5 text-[10px] font-semibold text-purple-300", "•" } }
                }
            }
        }
    }
}

/// `AnalyticsPlanStatusBar` — current plan + visible rank range +
/// upgrade CTA. Mirrors `plan-status-bar.tsx` 194 LoC.
#[component]
fn AnalyticsPlanStatusBar(plan_name: String, rank_range: String, locked_ranks: Option<String>) -> Element {
    rsx! {
        div { class: "analytics-plan-status flex flex-col gap-3 sm:flex-row sm:items-center sm:justify-between rounded-2xl border border-purple-500/20 bg-gradient-to-r from-purple-500/10 via-pink-500/10 to-orange-500/10 p-4 mb-4",
            "data-section": "analytics-plan-status",
            div { class: "flex items-center gap-3",
                div { class: "flex h-10 w-10 items-center justify-center rounded-xl bg-gradient-to-br from-purple-500 to-pink-500 shadow-lg",
                    Icon { name: "shield".to_string(), size: Some(20), class_name: Some("text-white".to_string()) }
                }
                div {
                    h3 { class: "text-base font-bold text-foreground", "{plan_name}" }
                    div { class: "mt-1 flex items-center gap-2 text-sm text-slate-400",
                        Icon { name: "sparkles".to_string(), size: Some(14) }
                        span { "Viewing: " }
                        span { class: "font-semibold text-foreground", "{rank_range}" }
                        if let Some(locked) = &locked_ranks {
                            span { class: "text-slate-500", "•" }
                            div { class: "flex items-center gap-1 text-amber-400",
                                Icon { name: "lock".to_string(), size: Some(12) }
                                span { "{locked}" }
                            }
                        }
                    }
                }
            }
            if let Some(locked) = &locked_ranks {
                a {
                    class: "inline-flex items-center gap-2 rounded-xl bg-gradient-to-r from-purple-500 via-pink-500 to-orange-500 px-4 py-2 text-sm font-semibold text-white shadow-lg hover:shadow-xl",
                    href: "/plans",
                    Icon { name: "rocket".to_string(), size: Some(14) }
                    "Unlock {locked}"
                }
            }
        }
    }
}

/// `FilterPanel` — sort + country + sector + min-eps + min-growth.
/// Mirrors `filter-panel.tsx` 175 LoC + `filter-form.tsx` 187 LoC.
#[component]
fn FilterPanel(
    active_count: u32,
    on_clear: EventHandler,
    on_apply: EventHandler,
) -> Element {
    rsx! {
        div { class: "analytics-filter-panel card card-glass mb-6",
            "data-section": "analytics-filter-panel",
            div { class: "card-header flex flex-col gap-3 p-3 sm:flex-row sm:items-end sm:gap-3 sm:p-4",
                // Sort By
                div { class: "flex-1 min-w-0",
                    label { class: "form-label", "Sort By" }
                    select { class: "input",
                        option { value: "growth_factor", "EPS Growth" }
                        option { value: "current_eps", "Current EPS" }
                        option { value: "market_cap", "Market Cap" }
                        option { value: "ranking_position", "Ranking Position" }
                    }
                }
                // Country
                div { class: "flex-1 min-w-0",
                    label { class: "form-label", "Country" }
                    select { class: "input",
                        option { value: "", "All Countries" }
                        option { value: "us", "United States" }
                        option { value: "gb", "United Kingdom" }
                        option { value: "jp", "Japan" }
                        option { value: "de", "Germany" }
                        option { value: "fr", "France" }
                    }
                }
                // Sector
                div { class: "flex-1 min-w-0",
                    label { class: "form-label", "Sector" }
                    select { class: "input",
                        option { value: "", "All Sectors" }
                        option { value: "Technology", "Technology" }
                        option { value: "Healthcare", "Healthcare" }
                        option { value: "Financials", "Financials" }
                        option { value: "Energy", "Energy" }
                        option { value: "Consumer", "Consumer" }
                    }
                }
                // Min EPS
                div { class: "flex-1 min-w-0",
                    label { class: "form-label", r#for: "analytics-filter-min-eps", "Min EPS ($)" }
                    input {
                        class: "input",
                        id: "analytics-filter-min-eps",
                        r#type: "number",
                        step: "0.01",
                        placeholder: "e.g. 1.00",
                    }
                }
                // Min growth
                div { class: "flex-1 min-w-0",
                    label { class: "form-label", r#for: "analytics-filter-min-growth", "Min Growth (%)" }
                    input {
                        class: "input",
                        id: "analytics-filter-min-growth",
                        r#type: "number",
                        step: "0.1",
                        placeholder: "e.g. 5.0",
                    }
                }
                // Actions
                div { class: "flex items-end gap-2",
                    button {
                        r#type: "button",
                        class: "btn btn-primary analytics-filter-apply",
                        onclick: move |_| on_apply.call(()),
                        Icon { name: "search".to_string(), size: Some(14) }
                        " Apply"
                    }
                    if active_count > 0 {
                        button {
                            r#type: "button",
                            class: "btn btn-outline analytics-filter-clear",
                            onclick: move |_| on_clear.call(()),
                            Icon { name: "rotate-ccw".to_string(), size: Some(14) }
                            span { class: "hidden sm:inline", " Reset" }
                        }
                    }
                }
            }
            if active_count > 0 {
                {
                    let plural = if active_count != 1 { "s" } else { "" };
                    let filter_text = format!("{active_count} filter{plural} active");
                    rsx! {
                        div { class: "border-t border-white/5 px-4 py-2 flex items-center gap-2 text-xs text-slate-400",
                            div { class: "h-1.5 w-1.5 rounded-full bg-emerald-400" }
                            span { "{filter_text}" }
                        }
                    }
                }
            }
        }
    }
}

/// `AnalyticsCardGrid` — 2-col responsive grid of ranking cards.
#[component]
fn AnalyticsCardGrid(cards: Vec<AnalyticsRankingCard>) -> Element {
    rsx! {
        div { class: "analytics-card-grid grid grid-cols-1 gap-4 sm:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4 mb-6",
            "data-section": "analytics-card-grid",
            if cards.is_empty() {
                div { class: "col-span-full py-12 text-center",
                    div { class: "mx-auto mb-4 flex h-16 w-16 items-center justify-center rounded-2xl bg-muted",
                        Icon { name: "sparkles".to_string(), size: Some(32), class_name: Some("text-slate-400".to_string()) }
                    }
                    p { class: "text-slate-400", "No rankings data available" }
                }
            } else {
                for card in cards.iter() {
                    {
                        let card = card.clone();
                        let tier_class = if card.tier_label == "Premium" { "analytics-card-tier-premium" } else { "analytics-card-tier-standard" };
                        let growth_class = if card.eps_growth >= 0.0 { "text-emerald-400" } else { "text-red-400" };
                        let growth_prefix = if card.eps_growth >= 0.0 { "+" } else { "" };
                        rsx! {
                            div {
                                class: "analytics-card card card-glass hover-scale {tier_class}",
                                key: "{card.symbol}",
                                div { class: "card-body p-4",
                                    div { class: "flex items-start justify-between mb-2",
                                        div {
                                            div { class: "flex items-center gap-2",
                                                span { class: "font-semibold text-foreground", "{card.symbol}" }
                                                span { class: "rounded-full bg-purple-500/10 px-2 py-0.5 text-[10px] font-semibold text-purple-400", "{card.tier_label}" }
                                            }
                                            p { class: "mt-0.5 text-xs text-muted-foreground truncate", "{card.company_name}" }
                                        }
                                        div { class: "flex items-center gap-2",
                                            // Wave 23 T2 — TradingView external link, mirrors the
                                            // prod `card-dashboard-sections.tsx` link
                                            // `href={`https://www.tradingview.com/symbols/${cardData.symbol}`}`.
                                            // Renders a small icon-only link in the corner of each card.
                                            a {
                                                class: "text-muted-foreground hover:text-yellow-400 transition-colors",
                                                href: "https://www.tradingview.com/symbols/{card.symbol}",
                                                target: "_blank",
                                                rel: "noopener noreferrer",
                                                title: "View {card.symbol} on TradingView",
                                                "aria-label": "View {card.symbol} on TradingView",
                                                Icon { name: "external-link".to_string(), size: Some(14) }
                                            }
                                            span { class: "text-xs text-muted-foreground", "#{card.rank}" }
                                        }
                                    }
                                    div { class: "flex items-end justify-between",
                                        div {
                                            p { class: "text-2xl font-bold text-foreground", "${card.price:.2}" }
                                            p { class: "text-xs text-muted-foreground", "EPS growth" }
                                        }
                                        div { class: "text-right",
                                            p { class: "text-lg font-bold {growth_class}", "{growth_prefix}{card.eps_growth:.1}%" }
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

/// `AnalyticsChart` — large time-series chart, P&L over time +
/// volume by day side-by-side. Uses `ChartLine` + `ChartBar`.
#[component]
fn AnalyticsChart(series: Vec<Series>, volume: Vec<(String, f64)>) -> Element {
    rsx! {
        div { class: "analytics-chart grid grid-cols-1 lg:grid-cols-2 gap-4 mb-6",
            "data-section": "analytics-chart",
            div { class: "card card-glass",
                div { class: "card-header", h3 { class: "card-title", "P&L over time" } }
                div { class: "card-body",
                    ChartLine { series, width: 480, height: 220 }
                }
            }
            div { class: "card card-glass",
                div { class: "card-header", h3 { class: "card-title", "Volume by day" } }
                div { class: "card-body",
                    ChartBar { data: volume, width: 480, height: 220 }
                }
            }
        }
    }
}

/// `AnalyticsTable` — paginated table of recent events. Uses the
/// `DataTable` + `Pagination` primitives.
#[component]
fn AnalyticsTable(
    events: Vec<(String, String, String, String, String)>,
    current_page: u32,
    total_pages: u32,
    base_href: String,
) -> Element {
    // Build DataTable rows from the events tuple.
    let rows: Vec<Row> = events
        .iter()
        .enumerate()
        .map(|(i, e)| Row {
            id: format!("event-{i}"),
            cells: vec![e.0.clone(), e.1.clone(), e.2.clone(), e.3.clone(), e.4.clone()],
        })
        .collect();
    let columns = vec![
        Column { key: "symbol".into(), label: "Symbol".into(), sortable: true, ..Default::default() },
        Column { key: "event".into(), label: "Event".into(), sortable: true, ..Default::default() },
        Column { key: "date".into(), label: "Date".into(), sortable: true, ..Default::default() },
        Column { key: "sentiment".into(), label: "Sentiment".into(), sortable: true, ..Default::default() },
        Column { key: "move".into(), label: "Move".into(), sortable: true, ..Default::default() },
    ];
    rsx! {
        div { class: "analytics-table card card-glass mb-6",
            "data-section": "analytics-table",
            div { class: "card-header", h3 { class: "card-title", "Recent events" } }
            div { class: "card-body p-0",
                DataTable { columns, rows, striped: true, hover: true, page_size: 5 }
            }
            div { class: "p-3 border-t border-border/10",
                Pagination {
                    current_page,
                    total_pages,
                    base_href,
                    query_param: Some("page".to_string()),
                }
            }
        }
    }
}

/// `AnalyticsMetadata` — last refresh, processing time, data source,
/// attribution. Mirrors `analytics-metadata-display.tsx` 152 LoC.
#[component]
fn AnalyticsMetadata(
    processing_time_ms: u32,
    last_refresh: String,
    data_source: String,
    attribution: String,
) -> Element {
    rsx! {
        div { class: "analytics-metadata relative overflow-hidden rounded-3xl border border-purple-200/50 bg-white/80 p-6 shadow-2xl backdrop-blur-xl",
            "data-section": "analytics-metadata",
            div { class: "absolute inset-0 bg-gradient-to-br from-purple-50/50 via-transparent to-blue-50/50" }
            div { class: "relative z-10",
                div { class: "mb-4 text-center sm:text-left",
                    h2 { class: "mb-2 text-xl font-bold sm:text-2xl",
                        span { class: "mr-2", "🚀" }
                        span { class: "bg-gradient-to-r from-purple-500 via-blue-500 to-purple-600 bg-clip-text text-transparent",
                            "Advanced Analytics Engine"
                        }
                    }
                    p { class: "text-sm text-muted-foreground",
                        "Powered by Diesel ORM with real-time processing and intelligent caching"
                    }
                }
                div { class: "grid grid-cols-1 gap-4 sm:grid-cols-2 lg:grid-cols-4",
                    AnalyticsMetadataCard { icon: "zap", label: "Processing Time", value: format!("{processing_time_ms} ms"), color: "green" }
                    AnalyticsMetadataCard { icon: "wifi", label: "Real-time", value: "HTTP Only".to_string(), color: "blue" }
                    AnalyticsMetadataCard { icon: "database", label: "Data Source", value: data_source, color: "orange" }
                    AnalyticsMetadataCard { icon: "globe", label: "Markets", value: attribution, color: "purple" }
                }
                div { class: "mt-4 flex items-center gap-2 text-xs text-muted-foreground",
                    Icon { name: "clock".to_string(), size: Some(12) }
                    span { "Last refresh: " }
                    span { class: "font-mono text-foreground", "{last_refresh}" }
                }
            }
        }
    }
}

/// Per-cell card for `AnalyticsMetadata`. Mirrors the `MetricCard`
/// sub-component in the source (lines 68-120).
#[component]
fn AnalyticsMetadataCard(icon: String, label: String, value: String, color: String) -> Element {
    let (border, bg, icon_bg, label_color, value_color) = match color.as_str() {
        "blue" => (
            "border-blue-200/50",
            "from-blue-50/80 to-cyan-50/80",
            "from-blue-500 to-cyan-500",
            "text-blue-700",
            "text-blue-800",
        ),
        "orange" => (
            "border-orange-200/50",
            "from-orange-50/80 to-yellow-50/80",
            "from-orange-500 to-yellow-500",
            "text-orange-700",
            "text-orange-800",
        ),
        "purple" => (
            "border-purple-200/50",
            "from-purple-50/80 to-pink-50/80",
            "from-purple-500 to-pink-500",
            "text-purple-700",
            "text-purple-800",
        ),
        _ => (
            "border-green-200/50",
            "from-green-50/80 to-emerald-50/80",
            "from-green-500 to-emerald-500",
            "text-green-700",
            "text-green-800",
        ),
    };
    rsx! {
        div { class: "rounded-2xl border {border} bg-gradient-to-br {bg} p-4 backdrop-blur-sm",
            div { class: "flex items-center gap-3 mb-2",
                div { class: "rounded-xl bg-gradient-to-r {icon_bg} p-2",
                    Icon { name: icon, size: Some(14), class_name: Some("text-white".to_string()) }
                }
                span { class: "text-sm font-semibold {label_color}", "{label}" }
            }
            p { class: "text-lg font-bold {value_color}", "{value}" }
        }
    }
}

/// `AnalyticsPageBody` — the body of the analytics page. Lives in a
/// `#[component]` fn so we can use `use_signal` (the parent's `render`
/// is a plain fn and has no Dioxus runtime).
#[component]
fn AnalyticsPageBody(ctx: PageContext) -> Element {
    // Local UI state — filter panel visibility, export dialog open.
    let mut filters_open = use_signal(|| true);
    let mut export_open = use_signal(|| false);
    let mut active_filters = use_signal(|| 2u32);

    // Sample data — would come from a BFF in production.
    let rankings = sample_rankings();
    let events = sample_events();
    let pnl_series = sample_pnl_series();
    let volume_series = sample_volume_series();

    rsx! {
        MainLayout { ctx: ctx.clone(),
            AuthGate { user: ctx.user.clone(), feature: Some("analytics".to_string()),
                required_permissions: Some(vec!["analytics:read".to_string()]),
                return_url: Some(ctx.path.clone()),
                div { class: "container page-content",
                    // 1. Header
                    AnalyticsHeader {
                        on_export_click: move |_| export_open.set(true),
                        on_filter_toggle: move |_| {
                            let cur = *filters_open.read();
                            filters_open.set(!cur);
                        },
                        filters_active: *active_filters.read() > 0,
                    }
                    // 2. Plan status
                    AnalyticsPlanStatusBar {
                        plan_name: "Pro".to_string(),
                        rank_range: "Ranks 5+".to_string(),
                        locked_ranks: Some("ranks 1-4".to_string()),
                    }
                    // 3. Filter panel (collapsible)
                    if *filters_open.read() {
                        FilterPanel {
                            active_count: *active_filters.read(),
                            on_clear: move |_| active_filters.set(0),
                            on_apply: move |_| {},
                        }
                    }
                    // 4. Card grid
                    AnalyticsCardGrid { cards: rankings }
                    // 5. Chart (P&L + volume)
                    AnalyticsChart { series: pnl_series, volume: volume_series }
                    // 6. Table (recent events + pagination)
                    AnalyticsTable {
                        events,
                        current_page: 1,
                        total_pages: 12,
                        base_href: "/analytics".to_string(),
                    }
                    // 7. Metadata
                    AnalyticsMetadata {
                        processing_time_ms: 124,
                        last_refresh: "2025-10-31 14:32 UTC".to_string(),
                        data_source: "Analytics API".to_string(),
                        attribution: "Global Markets".to_string(),
                    }
                    // PageHeader (existing — kept for breadcrumb continuity)
                    PageHeader {
                        title: "Analytics".to_string(),
                        description: Some("Your trading performance and platform usage".to_string()),
                        icon: Some("chart-line".to_string()),
                    }
                    // Legacy 4-up stat cards (kept; matches the previous
                    // page shape and avoids regressing the small
                    // `/analytics` smoke test that other tools may hit).
                    div { class: "grid grid-cols-1 md:grid-cols-4 gap-4 mb-6",
                        StatCard { label: "Total trades".to_string(), value: "1,234".to_string(), icon: Some("repeat".to_string()) }
                        StatCard { label: "Win rate".to_string(), value: "62%".to_string(), icon: Some("target".to_string()) }
                        StatCard { label: "Avg return".to_string(), value: "+3.4%".to_string(), icon: Some("trending-up".to_string()) }
                        StatCard { label: "Best trade".to_string(), value: "+24.5%".to_string(), icon: Some("award".to_string()) }
                    }
                    // 8. Export dialog (modal — only rendered when open)
                    // The wrapper div is always present so the
                    // `analytics-export-dialog` section marker is in
                    // the DOM even when the dialog is closed. This
                    // matches the design doc's "section always
                    // present" contract.
                    div { class: "analytics-export-dialog-marker", "data-section": "analytics-export-dialog",
                        ExportDialog {
                            open: *export_open.read(),
                            on_close: move |_| export_open.set(false),
                            on_export: move |_fmt: ExportFormat| {
                                // Caller is responsible for the actual
                                // download; for the port, just close.
                                export_open.set(false);
                            },
                            title: Some("Export Analytics Data".to_string()),
                            description: Some("Choose what to export and in which format.".to_string()),
                        }
                    }
                }
            }
        }
    }
}

/// Top-level `render` — composes the 8 sections. The dialog state is
/// local to `AnalyticsPageBody` (the page is a self-contained unit;
/// in a real hydration pass it would lift into a global store).
pub fn render(ctx: &PageContext) -> (PageMeta, Element) {
    let meta = PageMeta::app("Analytics");
    let body = rsx! { AnalyticsPageBody { ctx: ctx.clone() } };
    (meta, body)
}

// === wave6-auth-pages-depth-track-b ===
// Unit tests for the analytics page. Required by the design doc:
//   - test_render_smoke: render() returns a non-empty Element
//   - test_section_markers: the rendered HTML contains the 8
//     section-marker class names defined above.
#[cfg(test)]
mod tests {
    use super::*;

    fn empty_ctx() -> PageContext {
        PageContext {
            path: "/analytics".to_string(),
            ..Default::default()
        }
    }

    /// Authed context — the page is wrapped in `<AuthGate>` (gated on
    /// `analytics:read`). For the section-marker test to see the body,
    /// the test user must hold the `analytics:read` permission. The
    /// gate's `has_permission` check is exact-match, so we set it
    /// explicitly.
    fn authed_ctx() -> PageContext {
        PageContext {
            user: Some(crate::auth::User {
                id: "test-user".to_string(),
                address: "0xtest".to_string(),
                chain_id: "56".to_string(),
                roles: vec!["user".to_string()],
                email: None,
                tier: Some("Pro".to_string()),
                permissions: vec!["analytics:read".to_string()],
                last_login_at: None,
                auth_method: crate::auth::AuthMethod::Wallet,
                display_name: Some("Test".to_string()),
            }),
            path: "/analytics".to_string(),
            ..Default::default()
        }
    }

    fn render_to_string(ctx: &PageContext) -> String {
        let (_meta, el) = render(ctx);
        dioxus_ssr::render_element(el)
    }

    /// Wave 6A — `test_render_smoke`. The `render` function returns
    /// a non-empty HTML string.
    #[test]
    fn test_render_smoke() {
        let ctx = authed_ctx();
        let (_meta, el) = render(&ctx);
        let html = dioxus_ssr::render_element(el);
        assert!(
            !html.trim().is_empty(),
            "analytics page should render non-empty HTML"
        );
    }

    /// Wave 6A — `test_section_markers`. The rendered HTML must
    /// contain each of the 8 design-doc section markers.
    #[test]
    fn test_section_markers() {
        let html = render_to_string(&authed_ctx());
        for marker in &[
            "analytics-header",
            "analytics-plan-status",
            "analytics-filter-panel",
            "analytics-card-grid",
            "analytics-chart",
            "analytics-table",
            "analytics-export-dialog",
            "analytics-metadata",
        ] {
            assert!(
                html.contains(marker),
                "analytics page should contain section marker `{marker}`. Got: {html}"
            );
        }
    }
}
