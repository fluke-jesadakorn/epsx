//! `/portfolio` — authenticated, owner-scoped watchlist management.
//!
//! The BFF supplies only the verified session owner's persisted symbols. This
//! page renders that contract and exposes progressive Watch/Unwatch controls;
//! it does not infer holdings, prices, ranks, or plan access in the frontend.

use super::PageContext;
use super::PageMeta;
use crate::components::auth_access_banner::AuthAccessBanner;
use crate::layout::main_layout::MainLayout;
use crate::pages::analytics::normalize_watchlist_symbol;
use crate::primitives::*;
use dioxus::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use uuid::Uuid;

const PORTFOLIO_SIGN_IN_PATH: &str = "/auth?return_url=%2Fportfolio";
const PORTFOLIO_SYMBOL_PATTERN: &str = "[A-Za-z0-9][A-Za-z0-9.-]{0,19}";
pub const PORTFOLIO_WATCHLIST_DATA_PARAM: &str = "data_portfolio_watchlist";
pub const PORTFOLIO_WATCHLIST_STATE_PARAM: &str = "data_portfolio_watchlist_state";

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct WatchlistGroupData {
    pub id: Uuid,
    pub name: String,
    pub position: i32,
    pub symbols: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct WatchlistLayoutData {
    pub groups: Vec<WatchlistGroupData>,
    pub ungrouped: Vec<String>,
    pub watched: usize,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct WatchlistGroupLayoutUpdate {
    pub id: Uuid,
    pub symbols: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct WatchlistLayoutUpdate {
    pub groups: Vec<WatchlistGroupLayoutUpdate>,
    pub ungrouped: Vec<String>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct WatchlistLayoutValidationError;

impl WatchlistLayoutData {
    pub fn validated(mut self) -> Result<Self, WatchlistLayoutValidationError> {
        if self.groups.len() > 200 || self.watched > 1_000 {
            return Err(WatchlistLayoutValidationError);
        }
        let mut group_ids = HashSet::new();
        let mut group_names = HashSet::new();
        let mut grouped = HashSet::new();
        for (position, group) in self.groups.iter_mut().enumerate() {
            let name = group.name.trim();
            if group.position
                != i32::try_from(position).map_err(|_| WatchlistLayoutValidationError)?
                || !(1..=50).contains(&name.chars().count())
                || name.chars().any(char::is_control)
                || !group_ids.insert(group.id)
                || !group_names.insert(name.to_lowercase())
                || group.symbols.len() > 1_000
            {
                return Err(WatchlistLayoutValidationError);
            }
            group.name = name.to_string();
            let mut local = HashSet::new();
            for symbol in &mut group.symbols {
                *symbol =
                    normalize_watchlist_symbol(symbol).ok_or(WatchlistLayoutValidationError)?;
                if !local.insert(symbol.clone()) {
                    return Err(WatchlistLayoutValidationError);
                }
                grouped.insert(symbol.clone());
            }
        }
        let mut ungrouped = HashSet::new();
        for symbol in &mut self.ungrouped {
            *symbol = normalize_watchlist_symbol(symbol).ok_or(WatchlistLayoutValidationError)?;
            if grouped.contains(symbol) || !ungrouped.insert(symbol.clone()) {
                return Err(WatchlistLayoutValidationError);
            }
        }
        if grouped.union(&ungrouped).count() != self.watched {
            return Err(WatchlistLayoutValidationError);
        }
        Ok(self)
    }

    pub fn memberships_for(&self, symbol: &str) -> usize {
        self.groups
            .iter()
            .filter(|group| group.symbols.iter().any(|candidate| candidate == symbol))
            .count()
    }
}

/// Inline CSS rules for Tailwind v2 CDN arbitrary-value classes
/// that the CDN doesn't generate. We inject these into the page so
/// `h-[400px]`-style dimensions render correctly.
const PORTFOLIO_INLINE_CSS: &str = r#"
.portfolio-prod-bg > div[style*="radial-gradient"] { opacity: 1 !important; }
.absolute.-top-40.-left-40 { width: 400px !important; height: 400px !important; }
.absolute.top-1\/3.-right-32 { width: 300px !important; height: 300px !important; }
.portfolio-prod-page { background: #f8fafc !important; color: #0f172a; }
.portfolio-prod-bg > div:first-child { background: linear-gradient(to bottom, #f8fafc, #f1f5f9, #f8fafc) !important; }
.portfolio-prod-title { color: #0f172a !important; }
.portfolio-prod-subtitle { color: #475569 !important; }
.portfolio-prod-header [data-portfolio-freshness="unavailable"] { color: #92400e !important; }
.portfolio-prod-header [data-portfolio-freshness="ready"] { color: #047857 !important; }
.portfolio-watchlist-search { background: rgba(30, 41, 59, 0.7) !important; border-color: #475569 !important; color: #94a3b8 !important; }
.portfolio-watchlist-input { background: transparent !important; color: #f8fafc !important; }
.portfolio-watchlist-input::placeholder { color: #94a3b8 !important; text-transform: none !important; }
.portfolio-watchlist-item { background: rgba(15, 23, 42, 0.82) !important; border-color: #334155 !important; }
.portfolio-watchlist-item h2 { color: #f8fafc !important; }
.portfolio-watchlist-item p { color: #94a3b8 !important; }
.portfolio-group { background: rgba(15, 23, 42, 0.66); border-color: #334155; }
.portfolio-drop-target { border-color: #10b981 !important; box-shadow: 0 0 0 3px rgb(16 185 129 / 0.15); }
.portfolio-dragging { opacity: .45; }
.portfolio-drop-placeholder { min-height: 5rem; border: 2px dashed #10b981; border-radius: 1rem; background: rgb(16 185 129 / .08); }
.portfolio-unavailable h2 { color: #ffffff !important; }
.portfolio-unavailable p { color: #cbd5e1 !important; }
.portfolio-new-group { background: rgba(15, 23, 42, 0.6) !important; border-color: #334155 !important; }
.portfolio-new-group label { color: #f1f5f9 !important; }
.portfolio-new-group input { background: #020617 !important; border-color: #475569 !important; color: #ffffff !important; }
.portfolio-empty-state, [data-watchlist-empty] { background: rgba(15, 23, 42, 0.82) !important; border-color: #334155 !important; }
.portfolio-empty-state h2, [data-watchlist-empty] h2 { color: #ffffff !important; }
.portfolio-empty-state p, [data-watchlist-empty] p { color: #94a3b8 !important; }
.portfolio-signin-card {
  background-color: #eff6ff !important;
  border-color: #bfdbfe !important;
}
.portfolio-prod-signin-title { color: #1e3a8a !important; }
.portfolio-prod-signin-sub,
.portfolio-prod-signin-footer { color: #1d4ed8 !important; }
.portfolio-prod-signin-link { color: #1d4ed8 !important; }
.portfolio-prod-signin-btn { background: #1d4ed8 !important; }
html.dark .portfolio-prod-page { background: #020617 !important; color: #f8fafc; }
html.dark .portfolio-prod-bg > div:first-child { background: linear-gradient(to bottom, #020617, #0f172a, #020617) !important; }
html.dark .portfolio-prod-title { color: #ffffff !important; }
html.dark .portfolio-prod-subtitle { color: #94a3b8 !important; }
html.dark .portfolio-prod-header [data-portfolio-freshness="unavailable"] { color: #fcd34d !important; }
html.dark .portfolio-prod-header [data-portfolio-freshness="ready"] { color: #6ee7b7 !important; }
html.dark .portfolio-prod-signin-title { color: #dbeafe !important; }
html.dark .portfolio-prod-signin-sub { color: #93c5fd !important; }
html.dark .portfolio-signin-card { background-color: rgb(30 58 138 / 0.2) !important; border-color: rgb(29 78 216) !important; }
html.dark .portfolio-prod-signin-footer,
html.dark .portfolio-prod-signin-link { color: #60a5fa !important; }
/* Wave 28 T2 — Tailwind v2 CDN doesn't generate the arbitrary-
   value `min-h-[300px]` class, so force it on the prod's
   `<RequireSignIn>` wrapper (which reserves 300px of vertical
   space for the signin card). */
.portfolio-prod-require-signin { min-height: 300px !important; }

/* --- UX Polish: Portfolio v2 enhancements (preserves all prod selectors above) --- */
.portfolio-prod-header { backdrop-filter: none; }
.portfolio-prod-icon { box-shadow: 0 10px 20px -10px rgba(16,185,129,0.5), 0 4px 8px -4px rgba(0,0,0,0.15); position: relative; }
.portfolio-prod-icon::after { content: ''; position: absolute; inset: -1px; border-radius: 0.75rem; padding: 1px; background: linear-gradient(135deg, rgba(255,255,255,0.4), rgba(255,255,255,0)); -webkit-mask: linear-gradient(#fff 0 0) content-box, linear-gradient(#fff 0 0); -webkit-mask-composite: xor; mask-composite: exclude; opacity: 0.6; pointer-events: none; }
.portfolio-watchlist-search { backdrop-filter: blur(12px); transition: border-color 0.2s, box-shadow 0.2s, background 0.2s; }
.portfolio-watchlist-search:focus-within { border-color: #10b981 !important; box-shadow: 0 0 0 3px rgb(16 185 129 / 0.15), 0 4px 12px rgb(0 0 0 / 0.15); background: rgba(15,23,42,0.9) !important; }
.portfolio-watchlist-input:focus { outline: none; }
.portfolio-watchlist-item { transition: transform 0.15s ease, border-color 0.15s ease, box-shadow 0.15s ease; backdrop-filter: blur(16px); }
.portfolio-watchlist-item:hover { transform: translateY(-1px); border-color: #475569 !important; box-shadow: 0 8px 24px rgb(0 0 0 / 0.22), 0 2px 8px rgb(0 0 0 / 0.15); }
.portfolio-watchlist-item:focus-within { border-color: #10b981 !important; }
.portfolio-group { backdrop-filter: blur(14px); transition: border-color 0.2s, box-shadow 0.2s; box-shadow: 0 8px 32px rgb(0 0 0 / 0.16); }
.portfolio-group:hover { border-color: #475569; }
.portfolio-group header input:focus { outline: 2px solid rgb(16 185 129 / 0.35); outline-offset: 0; border-color: #10b981 !important; box-shadow: 0 0 0 3px rgb(16 185 129 / 0.1); }
.portfolio-new-group { backdrop-filter: blur(14px); transition: border-color 0.2s; }
.portfolio-new-group:focus-within { border-color: #10b981 !important; }
.portfolio-empty-state, [data-watchlist-empty] { backdrop-filter: blur(14px); box-shadow: 0 12px 32px rgb(0 0 0 / 0.18); }
.portfolio-signin-card { box-shadow: 0 16px 40px rgb(29 78 216 / 0.12), 0 4px 12px rgb(0 0 0 / 0.06); transition: transform 0.2s ease, box-shadow 0.2s ease; }
.portfolio-signin-card:hover { transform: translateY(-1px); box-shadow: 0 20px 48px rgb(29 78 216 / 0.16), 0 8px 20px rgb(0 0 0 / 0.08); }
.portfolio-prod-signin-btn { transition: transform 0.15s ease, box-shadow 0.15s ease, background 0.15s ease; box-shadow: 0 8px 16px rgb(29 78 216 / 0.25); }
.portfolio-prod-signin-btn:hover { transform: translateY(-1px); box-shadow: 0 12px 24px rgb(29 78 216 / 0.35); filter: brightness(1.05); }
.portfolio-prod-signin-btn:active { transform: translateY(0); }
html.dark .portfolio-prod-icon { box-shadow: 0 10px 24px -10px rgba(16,185,129,0.4), 0 6px 16px rgb(0 0 0 / 0.35); }
@keyframes portfolio-orb-float { 0%, 100% { transform: translate(0,0) scale(1); opacity: 0.9; } 50% { transform: translate(6px,-8px) scale(1.02); opacity: 1; } }
.portfolio-prod-orb-1, .portfolio-prod-orb-2 { animation: portfolio-orb-float 18s ease-in-out infinite; }
.portfolio-prod-orb-2 { animation-delay: -9s; }
.portfolio-stat-pill { backdrop-filter: blur(8px); }
"#;

pub fn render(ctx: &PageContext) -> (PageMeta, Element) {
    let meta = PageMeta::app("Portfolio");
    let watchlist_state = ctx
        .param(PORTFOLIO_WATCHLIST_STATE_PARAM)
        .map(String::as_str)
        .unwrap_or("unavailable");
    let watchlist = ctx
        .param(PORTFOLIO_WATCHLIST_DATA_PARAM)
        .and_then(|raw| serde_json::from_str::<WatchlistLayoutData>(raw).ok())
        .and_then(|layout| layout.validated().ok());
    let ready_layout = (watchlist_state == "ready").then_some(watchlist).flatten();
    let freshness = if ctx.user.is_none() {
        "signed_out"
    } else if ready_layout.is_some() {
        "ready"
    } else {
        "unavailable"
    };
    let watched_count = ready_layout
        .as_ref()
        .map(|layout| layout.watched)
        .unwrap_or_default();
    (
        meta,
        rsx! {
            MainLayout { ctx: ctx.clone(),
                style { "{PORTFOLIO_INLINE_CSS}" }
                // Wave 25 T2 — match prod's bg-gray-50 dark:bg-slate-950
                // shell (we use the dark color directly because Tailwind
                // v2 CDN drops `dark:` variants). The fixed bg layer has
                // 3 gradient orbs + a radial dark overlay.
                div { class: "portfolio-prod-page relative min-h-screen bg-slate-950",
                    div { class: "fixed inset-0 z-0 portfolio-prod-bg",
                        div { class: "absolute inset-0 bg-gradient-to-b from-slate-950 via-slate-900 to-slate-950" }
                        div { class: "absolute -top-40 -left-40 h-[400px] w-[400px] rounded-full bg-emerald-600/15 blur-3xl portfolio-prod-orb-1" }
                        div { class: "absolute top-1/3 -right-32 h-[300px] w-[300px] rounded-full bg-teal-600/10 blur-3xl portfolio-prod-orb-2" }
                        div { class: "absolute inset-0 bg-[radial-gradient(ellipse_at_center,_transparent_0%,_rgba(0,0,0,0.3)_100%)]" }
                    }
                    div { class: "relative z-10",
                        div { class: "mx-auto max-w-7xl px-4 py-6 sm:py-8 portfolio-prod-container",
                            PortfolioHeader { freshness, watched_count }
                            if ctx.user.is_none() {
                                if ctx.wallet.address.is_none() {
                                    AuthAccessBanner { href: PORTFOLIO_SIGN_IN_PATH.to_string() }
                                }
                                // Value props — portfolio-specific benefits before the gate (keeps the
                                // signed-out truthfulness but adds hierarchy and scannability)
                                section { class: "mb-6 grid grid-cols-1 gap-3 sm:grid-cols-3",
                                    "aria-label": "Portfolio benefits",
                                    div { class: "flex items-start gap-3 rounded-2xl border border-slate-700/60 bg-slate-900/40 p-4 backdrop-blur-sm",
                                        div { class: "flex h-9 w-9 shrink-0 items-center justify-center rounded-xl bg-emerald-500/15 text-emerald-300",
                                            Icon { name: "heart".to_string(), size: Some(18) }
                                        }
                                        div {
                                            p { class: "text-sm font-semibold text-white", "Personal watchlist" }
                                            p { class: "mt-1 text-xs leading-relaxed text-slate-400", "Save symbols you care about. One place for everything." }
                                        }
                                    }
                                    div { class: "flex items-start gap-3 rounded-2xl border border-slate-700/60 bg-slate-900/40 p-4 backdrop-blur-sm",
                                        div { class: "flex h-9 w-9 shrink-0 items-center justify-center rounded-xl bg-teal-500/15 text-teal-300",
                                            Icon { name: "layers".to_string(), size: Some(18) }
                                        }
                                        div {
                                            p { class: "text-sm font-semibold text-white", "Organize with groups" }
                                            p { class: "mt-1 text-xs leading-relaxed text-slate-400", "Drag, reorder, and group symbols your way." }
                                        }
                                    }
                                    div { class: "flex items-start gap-3 rounded-2xl border border-slate-700/60 bg-slate-900/40 p-4 backdrop-blur-sm",
                                        div { class: "flex h-9 w-9 shrink-0 items-center justify-center rounded-xl bg-sky-500/15 text-sky-300",
                                            Icon { name: "refresh-cw".to_string(), size: Some(18) }
                                        }
                                        div {
                                            p { class: "text-sm font-semibold text-white", "Stay in sync" }
                                            p { class: "mt-1 text-xs leading-relaxed text-slate-400", "Persisted to your account. Available everywhere." }
                                        }
                                    }
                                }
                                div { class: "flex flex-col items-center justify-center min-h-[300px] p-2 sm:p-6 portfolio-prod-require-signin",
                                    div { class: "max-w-md w-full",
                                        PortfolioSignInCard {}
                                        p { class: "mt-4 text-center text-xs text-slate-500 dark:text-slate-400",
                                            "Secure, owner-scoped · no sample data is fabricated"
                                        }
                                    }
                                }
                            } else if let Some(layout) = ready_layout {
                                PortfolioWatchlist { layout }
                            } else {
                                PortfolioUnavailable { source_shape: true }
                            }
                        }
                    }
                }
            }
        },
    )
}

#[component]
fn PortfolioHeader(freshness: &'static str, watched_count: usize) -> Element {
    let (badge_class, badge_label, icon) = match freshness {
        "ready" => (
            "border-emerald-500/25 bg-emerald-500/10 text-emerald-300",
            format!("{watched_count} watched"),
            "check",
        ),
        "signed_out" => (
            "border-blue-500/20 bg-blue-500/10 text-blue-300",
            "Sign in to sync".to_string(),
            "user",
        ),
        _ => (
            "border-amber-500/20 bg-amber-500/10 text-amber-300",
            "Data unavailable".to_string(),
            "circle-alert",
        ),
    };
    rsx! {
        div { class: "portfolio-prod-header mb-6 flex flex-col gap-4 sm:flex-row sm:items-start sm:justify-between",
            div { class: "flex items-center gap-3",
                div { class: "portfolio-prod-icon flex h-10 w-10 items-center justify-center rounded-xl bg-gradient-to-br from-emerald-400 to-teal-500 ring-1 ring-white/10",
                    Icon { name: "heart".to_string(), size: Some(20), class_name: Some("text-white".to_string()) }
                }
                div {
                    h1 { class: "text-2xl font-bold tracking-tight text-white portfolio-prod-title", "Portfolio" }
                    p { class: "text-sm text-slate-400 portfolio-prod-subtitle",
                        "Track your watchlisted stocks"
                    }
                    if freshness == "ready" && watched_count > 0 {
                        p { class: "mt-1 hidden text-xs text-slate-500 dark:text-slate-400 sm:block",
                            "Organize with drag & drop · persisted to your account"
                        }
                    }
                }
            }
            div { class: "flex flex-wrap items-center gap-2 self-start sm:self-center",
                span {
                    class: "inline-flex w-max items-center gap-1.5 rounded-full border px-3 py-1.5 text-xs font-semibold shadow-sm {badge_class}",
                    "data-portfolio-freshness": freshness,
                    Icon { name: icon.to_string(), size: Some(14) }
                    "{badge_label}"
                }
                if freshness != "signed_out" {
                    span { class: "hidden items-center gap-1 text-xs text-slate-400 sm:inline-flex",
                        Icon { name: "shield".to_string(), size: Some(12), class_name: Some("text-slate-500".to_string()) }
                        "Owner-scoped"
                    }
                }
            }
        }
    }
}

#[component]
fn PortfolioWatchlist(layout: WatchlistLayoutData) -> Element {
    let empty = layout.watched == 0;
    let groups = layout.groups.clone();
    let group_count = layout.groups.len();
    rsx! {
        section {
            class: "space-y-5",
            "data-portfolio-state": if empty { "empty" } else { "ready" },
            "data-watchlist-organizer": "true",
            // Stats ribbon when there is content — fast scannability
            if !empty {
                div { class: "grid grid-cols-3 gap-3",
                    div { class: "portfolio-stat-pill flex flex-col items-center justify-center rounded-2xl border border-slate-700/60 bg-slate-900/50 px-3 py-3 backdrop-blur-sm",
                        span { class: "text-[11px] font-semibold uppercase tracking-widest text-slate-400", "Watched" }
                        span { class: "mt-1 text-xl font-black text-white", "{layout.watched}" }
                    }
                    div { class: "portfolio-stat-pill flex flex-col items-center justify-center rounded-2xl border border-slate-700/60 bg-slate-900/50 px-3 py-3 backdrop-blur-sm",
                        span { class: "text-[11px] font-semibold uppercase tracking-widest text-slate-400", "Groups" }
                        span { class: "mt-1 text-xl font-black text-white", "{group_count}" }
                    }
                    div { class: "portfolio-stat-pill flex flex-col items-center justify-center rounded-2xl border border-emerald-500/20 bg-emerald-500/10 px-3 py-3 backdrop-blur-sm",
                        span { class: "text-[11px] font-semibold uppercase tracking-widest text-emerald-300/80", "Ungrouped" }
                        span { class: "mt-1 text-xl font-black text-emerald-300", "{layout.ungrouped.len()}" }
                    }
                }
            }
            form {
                class: "portfolio-watchlist-search flex flex-col gap-3 rounded-2xl border border-slate-700/50 bg-slate-900/60 p-3 backdrop-blur-sm sm:flex-row sm:items-center sm:p-2.5",
                action: "/portfolio/watch",
                method: "post",
                "data-watchlist-form": "true",
                div { class: "flex min-w-0 flex-1 items-center gap-3 rounded-xl border border-slate-600/50 bg-slate-950/70 px-3 py-2.5 focus-within:border-emerald-500/50 focus-within:ring-2 focus-within:ring-emerald-500/20",
                    Icon { name: "search".to_string(), size: Some(18), class_name: Some("text-slate-500 shrink-0".to_string()) }
                    input {
                        id: "portfolio-watchlist-symbol",
                        class: "portfolio-watchlist-input min-w-0 flex-1 border-0 bg-transparent p-0 text-base font-semibold tracking-wide outline-none placeholder:normal-case",
                        r#type: "text",
                        name: "symbol",
                        maxlength: "20",
                        pattern: PORTFOLIO_SYMBOL_PATTERN,
                        autocomplete: "off",
                        spellcheck: "false",
                        required: true,
                        placeholder: "Enter a symbol, e.g. AAPL",
                        "aria-label": "Add symbol to watchlist",
                        style: "text-transform: uppercase;",
                    }
                }
                if !groups.is_empty() {
                    details { class: "relative shrink-0 sm:flex-none", "data-watch-group-picker": "true",
                        summary { class: "flex min-h-11 w-full cursor-pointer list-none items-center justify-center gap-1.5 rounded-xl border border-slate-600 bg-slate-800/70 px-4 py-2 text-sm font-semibold text-slate-200 hover:bg-slate-700/70 sm:w-auto",
                            Icon { name: "layers".to_string(), size: Some(14) }
                            "Choose groups"
                        }
                        div { class: "absolute right-0 z-30 mt-2 min-w-64 space-y-1 rounded-xl border border-slate-700 bg-slate-900 p-3 shadow-2xl",
                            p { class: "px-2 pb-2 text-xs font-semibold uppercase tracking-wide text-slate-400", "Add to groups" }
                            for group in groups.iter() {
                                label { class: "flex min-h-10 cursor-pointer items-center gap-3 rounded-lg px-2 py-1 text-sm text-slate-200 hover:bg-slate-800",
                                    input { r#type: "checkbox", name: "group_ids", value: "{group.id}", class: "rounded border-slate-600 text-emerald-600 focus:ring-emerald-500" }
                                    span { class: "truncate", "{group.name}" }
                                }
                            }
                            p { class: "border-t border-slate-800 px-2 pt-2 text-xs text-slate-500", "No selection means Ungrouped." }
                        }
                    }
                }
                button {
                    class: "inline-flex min-h-11 shrink-0 items-center justify-center gap-2 rounded-xl bg-emerald-600 px-6 py-2.5 font-semibold text-white shadow-lg shadow-emerald-900/20 transition-all hover:bg-emerald-500 hover:shadow-emerald-900/30 disabled:cursor-wait disabled:opacity-60 active:scale-[0.98] sm:self-stretch",
                    r#type: "submit",
                    "data-watchlist-add": "true",
                    "aria-busy": "false",
                    Icon { name: "plus".to_string(), size: Some(16) }
                    "Watch"
                }
            }
            p {
                class: "min-h-5 px-1 text-sm text-slate-400",
                role: "status",
                "aria-live": "polite",
                "data-watchlist-feedback": "true",
            }

            details { class: "group/create relative", "data-watchlist-new-group": "true",
                summary {
                    class: "flex min-h-11 w-full cursor-pointer list-none items-center justify-center gap-2 rounded-xl border border-dashed border-slate-600/70 text-sm font-semibold text-emerald-300 transition-colors hover:border-emerald-500/60 hover:text-emerald-200",
                    Icon { name: "plus".to_string(), size: Some(14) }
                    "Create a new group"
                }
                div { class: "portfolio-new-group mt-2 flex flex-col gap-3 rounded-2xl border border-slate-700 bg-slate-900/60 p-4 backdrop-blur-sm sm:flex-row sm:items-end",
                    div { class: "min-w-0 flex-1",
                        label { class: "mb-2 flex items-center gap-1.5 text-xs font-semibold text-slate-300", r#for: "portfolio-new-group",
                            Icon { name: "folder-open".to_string(), size: Some(12), class_name: Some("text-emerald-400".to_string()) }
                            "Group name"
                        }
                        input {
                            id: "portfolio-new-group",
                            class: "min-h-11 w-full rounded-xl border border-slate-600 bg-slate-950 px-3.5 text-white placeholder:text-slate-500 outline-none focus:border-emerald-500 focus:ring-2 focus:ring-emerald-500/20",
                            r#type: "text",
                            maxlength: "50",
                            placeholder: "e.g. Long term",
                            "data-watchlist-new-group-name": "true",
                        }
                    }
                    button {
                        class: "inline-flex min-h-11 items-center justify-center gap-1.5 rounded-xl bg-emerald-600 px-6 font-semibold text-white shadow-lg shadow-emerald-900/20 transition-all hover:bg-emerald-500 disabled:cursor-wait disabled:opacity-60 sm:self-auto",
                        r#type: "button",
                        "data-watchlist-group-create": "true",
                        "aria-busy": "false",
                        Icon { name: "check".to_string(), size: Some(14) }
                        "Create"
                    }
                }
            }

            if empty {
                div {
                    class: "portfolio-empty-state flex min-h-[340px] flex-col items-center justify-center rounded-3xl border border-dashed border-slate-700 bg-slate-900/50 p-8 text-center backdrop-blur-sm",
                    "data-watchlist-empty": "true",
                    div { class: "flex h-20 w-20 items-center justify-center rounded-3xl bg-emerald-500/10 text-emerald-300 ring-1 ring-emerald-500/20",
                        Icon { name: "heart".to_string(), size: Some(36) }
                    }
                    h2 { class: "mt-6 text-2xl font-semibold tracking-tight text-white", "No stocks watched yet" }
                    p { class: "mt-2 max-w-lg text-sm leading-relaxed text-slate-300",
                        "Enter a stock symbol above or use the heart on Analytics. Your watchlist is saved to your account."
                    }
                    div { class: "mt-6 flex flex-col items-center gap-3 sm:flex-row",
                        a { class: "inline-flex items-center justify-center gap-2 rounded-xl bg-emerald-600 px-6 py-3 text-sm font-semibold text-white shadow-lg shadow-emerald-900/20 transition-colors hover:bg-emerald-500", href: "/analytics",
                            Icon { name: "bar-chart-3".to_string(), size: Some(16) }
                            "Browse Analytics"
                        }
                        span { class: "hidden text-xs text-slate-500 sm:inline", "or" }
                        span { class: "text-xs text-slate-500", "try e.g. AAPL · MSFT · NVDA" }
                    }
                }
            } else {
                div { class: "space-y-5", "data-watchlist-groups": "true",
                    for group in groups.iter() {
                        WatchlistGroupSection {
                            group: group.clone(),
                            all_groups: groups.clone(),
                            layout: layout.clone(),
                        }
                    }
                    WatchlistUngroupedSection {
                        symbols: layout.ungrouped.clone(),
                        all_groups: groups.clone(),
                        layout: layout.clone(),
                    }
                }
            }
        }
    }
}

#[component]
fn WatchlistGroupSection(
    group: WatchlistGroupData,
    all_groups: Vec<WatchlistGroupData>,
    layout: WatchlistLayoutData,
) -> Element {
    rsx! {
        section {
            class: "portfolio-group overflow-hidden rounded-2xl border p-4 sm:p-5",
            "data-watchlist-group": "true",
            "data-group-id": "{group.id}",
            header {
                class: "mb-4 flex flex-col gap-3 sm:flex-row sm:items-center",
                draggable: "true",
                "data-group-id": "{group.id}",
                div { class: "flex min-w-0 flex-1 items-center gap-2",
                    button { class: "flex h-9 w-9 shrink-0 cursor-grab touch-none items-center justify-center rounded-lg border border-slate-700/60 bg-slate-800/60 text-slate-400 hover:bg-slate-700/80 hover:text-slate-200 active:cursor-grabbing", r#type: "button", title: "Drag group", "aria-label": "Drag {group.name}", "data-watchlist-group-handle": "true",
                        Icon { name: "menu".to_string(), size: Some(14) }
                    }
                    div { class: "flex min-w-0 flex-1 items-center gap-2 rounded-xl border border-transparent bg-transparent px-1 hover:border-slate-600/60 hover:bg-slate-800/30 focus-within:border-emerald-500/50 focus-within:bg-slate-800/60",
                        input {
                            class: "min-h-9 min-w-0 flex-1 rounded-lg bg-transparent px-2 text-[15px] font-bold tracking-tight text-slate-900 outline-none dark:text-white",
                            r#type: "text",
                            maxlength: "50",
                            value: "{group.name}",
                            "aria-label": "Rename {group.name}",
                            "data-watchlist-group-name": "true",
                        }
                        span { class: "mr-1 shrink-0 rounded-full bg-emerald-500/15 px-2.5 py-1 text-xs font-semibold text-emerald-300 ring-1 ring-emerald-500/20", "{group.symbols.len()}" }
                    }
                }
                div { class: "flex flex-wrap items-center gap-1.5",
                    button { class: "inline-flex min-h-9 items-center justify-center rounded-lg border border-slate-600 bg-slate-800/70 px-3 text-xs font-semibold text-slate-200 hover:bg-slate-700/80", r#type: "button", "data-watchlist-group-rename": "true", "data-group-id": "{group.id}",
                        Icon { name: "check".to_string(), size: Some(12) }
                        "Save name"
                    }
                    button { class: "inline-flex h-8 w-8 items-center justify-center rounded-lg border border-slate-600 bg-slate-800/50 text-slate-400 hover:bg-slate-700/60 hover:text-slate-200", r#type: "button", "data-watchlist-move-group": "up", "data-group-id": "{group.id}", title: "Move group up", "aria-label": "Move group up",
                        Icon { name: "arrow-up-down".to_string(), size: Some(14) }
                    }
                    button { class: "inline-flex h-8 w-8 items-center justify-center rounded-lg border border-slate-600 bg-slate-800/50 text-slate-400 hover:bg-slate-700/60 hover:text-slate-200", r#type: "button", "data-watchlist-move-group": "down", "data-group-id": "{group.id}", title: "Move group down", "aria-label": "Move group down",
                        Icon { name: "chevron-down".to_string(), size: Some(14) }
                    }
                    button { class: "inline-flex min-h-9 items-center justify-center gap-1 rounded-lg border border-red-500/20 bg-red-500/10 px-3 text-xs font-semibold text-red-300 hover:bg-red-500/15", r#type: "button", "data-watchlist-group-delete": "true", "data-group-id": "{group.id}",
                        Icon { name: "trash-2".to_string(), size: Some(12) }
                        "Delete"
                    }
                }
            }
            div {
                class: "grid min-h-20 grid-cols-1 gap-4 sm:grid-cols-2 lg:grid-cols-3",
                "data-watchlist-items": "true",
                "data-group-id": "{group.id}",
                for symbol in group.symbols.iter() {
                    PortfolioSymbolCard {
                        symbol: symbol.clone(),
                        current_group: Some(group.id),
                        all_groups: all_groups.clone(),
                        membership_count: layout.memberships_for(symbol),
                    }
                }
                if group.symbols.is_empty() {
                    p { class: "col-span-full py-5 text-center text-sm text-slate-500 dark:text-slate-400", "Drop stocks here" }
                }
            }
            p { class: "mt-3 min-h-5 text-sm", role: "status", "aria-live": "polite", "data-watchlist-group-feedback": "true" }
        }
    }
}

#[component]
fn WatchlistUngroupedSection(
    symbols: Vec<String>,
    all_groups: Vec<WatchlistGroupData>,
    layout: WatchlistLayoutData,
) -> Element {
    rsx! {
        section {
            class: "portfolio-group overflow-hidden rounded-2xl border border-dashed p-4 sm:p-5",
            "data-watchlist-group": "true",
            "data-group-id": "ungrouped",
            header { class: "mb-4 flex items-center justify-between gap-3",
                div { class: "flex items-center gap-3",
                    div { class: "hidden h-9 w-9 items-center justify-center rounded-xl bg-slate-700/50 text-slate-400 sm:flex",
                        Icon { name: "inbox".to_string(), size: Some(16) }
                    }
                    div {
                        h2 { class: "flex items-center gap-2 text-[15px] font-bold text-slate-900 dark:text-white", "Ungrouped" }
                        p { class: "text-xs text-slate-500 dark:text-slate-400", "Drag a symbol onto a group above to organize it" }
                    }
                }
                span { class: "rounded-full bg-slate-700 px-3 py-1 text-xs font-semibold text-slate-200 ring-1 ring-slate-600", "{symbols.len()}" }
            }
            div {
                class: "grid min-h-20 grid-cols-1 gap-4 sm:grid-cols-2 lg:grid-cols-3",
                "data-watchlist-items": "true",
                "data-group-id": "ungrouped",
                for symbol in symbols.iter() {
                    PortfolioSymbolCard {
                        symbol: symbol.clone(),
                        current_group: None,
                        all_groups: all_groups.clone(),
                        membership_count: layout.memberships_for(symbol),
                    }
                }
                if symbols.is_empty() {
                    p { class: "col-span-full py-5 text-center text-sm text-slate-500 dark:text-slate-400", "No ungrouped stocks" }
                }
            }
        }
    }
}

#[component]
fn PortfolioSymbolCard(
    symbol: String,
    current_group: Option<Uuid>,
    all_groups: Vec<WatchlistGroupData>,
    membership_count: usize,
) -> Element {
    let current_group_value = current_group
        .map(|id| id.to_string())
        .unwrap_or_else(|| "ungrouped".to_string());
    let membership_label = if membership_count == 1 {
        "1 group".to_string()
    } else {
        format!("{membership_count} groups")
    };
    rsx! {
        article {
            class: "portfolio-watchlist-item group flex flex-col gap-3 rounded-2xl border p-4 shadow-sm",
            draggable: "true",
            "data-watchlist-item": "true",
            "data-symbol": "{symbol}",
            "data-group-id": "{current_group_value}",
            "data-membership-count": "{membership_count}",
            div { class: "flex items-start justify-between gap-3",
                div { class: "min-w-0 flex-1",
                    div { class: "flex items-center gap-2",
                        h3 { class: "truncate text-[22px] font-black tracking-tight text-white", "{symbol}" }
                        if membership_count > 1 {
                            span { class: "inline-flex items-center rounded-full bg-amber-500/15 px-2 py-0.5 text-[11px] font-semibold text-amber-300 ring-1 ring-amber-500/20", "{membership_label}" }
                        }
                    }
                    p { class: "mt-1 text-xs text-slate-400",
                        if membership_count == 0 {
                            "Not in any group"
                        } else if membership_count == 1 {
                            "In 1 group"
                        } else {
                            "In {membership_count} groups"
                        }
                    }
                }
                button { class: "flex h-8 w-8 shrink-0 cursor-grab touch-none items-center justify-center rounded-lg border border-slate-700/60 bg-slate-800/60 text-slate-500 opacity-60 transition-opacity hover:bg-slate-700/70 hover:text-slate-300 group-hover:opacity-100 active:cursor-grabbing", r#type: "button", title: "Drag stock", "aria-label": "Drag {symbol}", "data-watchlist-item-handle": "true",
                    Icon { name: "menu".to_string(), size: Some(14) }
                }
            }
            if membership_count == 1 {
                div { class: "inline-flex w-fit items-center gap-1 rounded-full bg-slate-800 px-2.5 py-1 text-xs font-medium text-slate-300 ring-1 ring-slate-700",
                    Icon { name: "layers".to_string(), size: Some(12), class_name: Some("text-slate-500".to_string()) }
                    "{membership_label}"
                }
            } else if membership_count == 0 {
                div { class: "inline-flex w-fit items-center gap-1 rounded-full bg-slate-800/80 px-2.5 py-1 text-xs font-medium text-slate-400 ring-1 ring-slate-700/60",
                    Icon { name: "inbox".to_string(), size: Some(12), class_name: Some("text-slate-500".to_string()) }
                    "Ungrouped"
                }
            }
            details { class: "relative", "data-watchlist-add-groups-menu": "true",
                summary { class: "flex min-h-10 cursor-pointer list-none items-center justify-center gap-1.5 rounded-xl border border-slate-600 bg-slate-800/60 px-3 py-2 text-center text-xs font-semibold text-slate-200 hover:bg-slate-700/60",
                    Icon { name: "plus".to_string(), size: Some(12) }
                    "Add to groups"
                }
                div { class: "absolute left-0 right-0 z-20 mt-2 space-y-1 rounded-xl border border-slate-700 bg-slate-900 p-3 shadow-2xl",
                    p { class: "px-1 pb-1 text-xs font-semibold uppercase tracking-wide text-slate-400", "Memberships" }
                    for group in all_groups.iter() {
                        label { class: "flex min-h-9 cursor-pointer items-center gap-3 rounded-lg px-2 text-sm text-slate-200 hover:bg-slate-800",
                            input {
                                r#type: "checkbox",
                                value: "{group.id}",
                                checked: group.symbols.iter().any(|candidate| candidate == &symbol),
                                disabled: group.symbols.iter().any(|candidate| candidate == &symbol),
                                class: "rounded border-slate-600 bg-slate-800 text-emerald-600 focus:ring-emerald-500 disabled:opacity-40",
                                "data-watchlist-membership-choice": "true",
                            }
                            span { class: "truncate", "{group.name}" }
                            if group.symbols.iter().any(|candidate| candidate == &symbol) {
                                span { class: "ml-auto text-[11px] text-emerald-400", "added" }
                            }
                        }
                    }
                    if all_groups.is_empty() {
                        p { class: "px-2 py-2 text-sm text-slate-500", "Create a group first." }
                    } else {
                        button { class: "mt-2 inline-flex min-h-10 w-full items-center justify-center gap-1.5 rounded-xl bg-emerald-600 px-3 text-sm font-semibold text-white hover:bg-emerald-500", r#type: "button", "data-watchlist-groups-save": "true", "data-symbol": "{symbol}",
                            Icon { name: "check".to_string(), size: Some(14) }
                            "Save groups"
                        }
                    }
                }
            }
            div { class: "flex items-center gap-1 rounded-xl border border-slate-700/60 bg-slate-800/30 p-1",
                button { class: "inline-flex min-h-8 flex-1 items-center justify-center gap-1 rounded-lg bg-slate-800 px-2 text-xs font-medium text-slate-300 hover:bg-slate-700 hover:text-white", r#type: "button", "data-watchlist-move-item": "up",
                    Icon { name: "arrow-up-down".to_string(), size: Some(12) }
                    "Up"
                }
                div { class: "h-4 w-px bg-slate-700" }
                button { class: "inline-flex min-h-8 flex-1 items-center justify-center gap-1 rounded-lg bg-slate-800 px-2 text-xs font-medium text-slate-300 hover:bg-slate-700 hover:text-white", r#type: "button", "data-watchlist-move-item": "down",
                    "Down"
                    Icon { name: "chevron-down".to_string(), size: Some(12) }
                }
            }
            label { class: "space-y-1.5 text-xs font-semibold text-slate-400",
                span { class: "flex items-center gap-1", Icon { name: "arrow-right-left".to_string(), size: Some(12) } "Move to group" }
                select { class: "min-h-10 w-full rounded-xl border border-slate-700 bg-slate-900/60 px-3 py-2 text-sm text-slate-200 outline-none focus:border-emerald-500/50 focus:ring-2 focus:ring-emerald-500/20", "data-watchlist-move-to-group": "true",
                    option { value: "ungrouped", selected: current_group.is_none(), "Ungrouped" }
                    for group in all_groups.iter() {
                        option { value: "{group.id}", selected: current_group == Some(group.id), "{group.name}" }
                    }
                }
            }
            div { class: "flex flex-wrap gap-2 border-t border-slate-700/60 pt-3",
                if current_group.is_some() {
                    button { class: "inline-flex min-h-10 flex-1 items-center justify-center gap-1 rounded-xl border border-slate-600 bg-slate-800/60 px-3 text-xs font-semibold text-slate-300 hover:bg-slate-700/60", r#type: "button", "data-watchlist-remove-membership": "true",
                        Icon { name: "x".to_string(), size: Some(12) }
                        "Remove from this group"
                    }
                }
                form { class: "flex-1", action: "/portfolio/unwatch", method: "post",
                    input { r#type: "hidden", name: "symbol", value: "{symbol}" }
                    button {
                        class: "inline-flex min-h-10 w-full items-center justify-center gap-1.5 rounded-xl border border-pink-500/20 bg-pink-500/10 px-3 text-sm font-semibold text-pink-300 hover:bg-pink-500/15 disabled:cursor-wait disabled:opacity-60",
                        r#type: "submit",
                        "data-watchlist-toggle": "true",
                        "data-symbol": "{symbol}",
                        "data-watchlisted": "true",
                        "data-membership-count": "{membership_count}",
                        "aria-label": "Unwatch {symbol}",
                        "aria-busy": "false",
                        Icon { name: "trash".to_string(), size: Some(14) }
                        "Unwatch"
                    }
                }
            }
            p { class: "min-h-4 text-xs text-red-400", role: "status", "aria-live": "polite", "data-watchlist-item-feedback": "true" }
        }
    }
}

#[component]
fn PortfolioUnavailable(source_shape: bool) -> Element {
    rsx! {
        section {
            class: if source_shape {
                "portfolio-unavailable portfolio-source-preview overflow-hidden rounded-3xl border border-slate-700/60 bg-slate-900/30 backdrop-blur-sm shadow-xl shadow-black/10"
            } else {
                "portfolio-unavailable overflow-hidden rounded-3xl border border-slate-700/80 bg-slate-900/50 shadow-xl shadow-black/20 backdrop-blur-sm"
            },
            "data-portfolio-state": "unavailable",
            role: "alert",
            aria_labelledby: "portfolio-unavailable-title",
            div { class: "h-1 bg-gradient-to-r from-emerald-400 via-teal-400 to-cyan-400" }
            div { class: if source_shape { "space-y-6 p-4 sm:space-y-8 sm:p-8" } else { "space-y-8 p-5 sm:p-8" },
                div {
                    class: if source_shape { "portfolio-watchlist-search flex min-w-0 items-center gap-2 rounded-xl border border-slate-700 bg-slate-800/50 px-3 py-3 text-xs text-slate-400 opacity-75 sm:gap-3 sm:px-5 sm:py-4 sm:text-base" } else { "portfolio-watchlist-search flex items-center gap-3 rounded-2xl border border-slate-600 bg-slate-800/70 px-5 py-4 text-base text-slate-400 sm:text-xl" },
                    role: "searchbox",
                    aria_disabled: "true",
                    Icon { name: "search".to_string(), size: Some(if source_shape { 16 } else { 22 }), class_name: Some("text-slate-500".to_string()) }
                    span { class: "min-w-0 truncate", "Search stocks to add to watchlist…" }
                    span { class: "ml-auto hidden items-center gap-1 rounded-full bg-amber-500/10 px-2.5 py-1 text-xs font-medium text-amber-300 ring-1 ring-amber-500/20 sm:inline-flex",
                        Icon { name: "circle-alert".to_string(), size: Some(12) }
                        "Offline"
                    }
                }

                div { class: if source_shape { "flex min-h-[240px] flex-col items-center justify-center text-center sm:min-h-[320px]" } else { "flex min-h-[280px] flex-col items-center justify-center text-center sm:min-h-[360px]" },
                    div { class: if source_shape { "flex h-20 w-20 items-center justify-center rounded-3xl bg-slate-800 text-slate-400 ring-1 ring-slate-700/60" } else { "flex h-24 w-24 items-center justify-center rounded-3xl bg-slate-800 text-slate-400 ring-1 ring-slate-700/60" },
                        Icon { name: "heart".to_string(), size: Some(if source_shape { 34 } else { 48 }), class_name: Some("opacity-80".to_string()) }
                    }
                    h2 {
                        id: "portfolio-unavailable-title",
                        class: if source_shape { "mt-6 text-lg font-semibold tracking-tight text-white sm:mt-8 sm:text-2xl" } else { "mt-8 text-2xl font-semibold tracking-tight text-white sm:text-3xl" },
                        "No watchlist data available"
                    }
                    p { class: if source_shape { "mt-3 max-w-lg text-sm leading-relaxed text-slate-400 sm:text-base" } else { "mt-3 max-w-2xl text-base leading-relaxed text-slate-400 sm:text-lg" },
                        "Your saved watchlist is temporarily unavailable. Watch and Unwatch actions stay disabled until the connection is restored."
                    }
                    p { class: "sr-only",
                        "Your portfolio cannot be verified right now. No securities, prices, rankings, plan access, or watchlist membership are being inferred."
                    }
                }

                div { class: "rounded-2xl border border-slate-700/60 bg-slate-800/30 p-4 backdrop-blur-sm",
                    p { class: "text-xs font-semibold uppercase tracking-wide text-slate-400", "Try instead" }
                    nav {
                        class: "mt-3 flex flex-col gap-2 sm:flex-row",
                        aria_label: "Portfolio alternatives",
                        a {
                            class: "inline-flex flex-1 items-center justify-center gap-2 rounded-xl border border-slate-600 bg-slate-800 px-4 py-2.5 text-sm font-semibold text-slate-200 hover:bg-slate-700",
                            href: "/account",
                            Icon { name: "user".to_string(), size: Some(16) }
                            " Return to account"
                        }
                        a {
                            class: "inline-flex flex-1 items-center justify-center gap-2 rounded-xl border border-slate-700 bg-transparent px-4 py-2.5 text-sm font-semibold text-slate-300 hover:bg-slate-800/50 hover:text-slate-200",
                            href: "/contact",
                            Icon { name: "circle-help".to_string(), size: Some(16) }
                            " Contact support"
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn PortfolioBoundaryItem(icon: &'static str, title: &'static str, body: &'static str) -> Element {
    rsx! {
        div { class: "rounded-xl border border-slate-700 bg-slate-800/50 p-4",
            div { class: "flex items-center gap-2 font-semibold text-white",
                Icon { name: icon.to_string(), size: Some(18) }
                "{title}"
            }
            p { class: "mt-2 text-sm leading-6 text-slate-300", "{body}" }
            span { class: "mt-3 inline-flex rounded-full border border-amber-500/30 bg-amber-500/10 px-2 py-1 text-xs font-medium text-amber-400",
                "Unavailable"
            }
        }
    }
}

/// "Sign In Required" blue card. Mirrors prod's
/// `p-6 bg-blue-50 border border-blue-200 rounded-lg
/// dark:bg-blue-900/20 dark:border-blue-700` panel with a 🔐
/// emoji icon, "Sign In Required" heading, "To view your
/// portfolio, you need basic authentication." subtext, a bright
/// blue "Sign In" button, a blue "Learn More" link, and a small
/// blue "Need help?" footer.
///
/// Wave 28 T2 — replaced the gold 40px lock SVG with the prod's
/// `🔐` emoji span (the prod uses the literal emoji, not an SVG),
/// and changed the inner wrapper from `flex flex-col items-center
/// text-center` to the prod's `text-center space-y-4` shape.
#[component]
fn PortfolioSignInCard() -> Element {
    rsx! {
        div { class: "portfolio-prod-signin portfolio-signin-card overflow-hidden rounded-2xl border p-6 bg-blue-900/20 border-blue-700 backdrop-blur-sm",
            div { class: "text-center space-y-5",
                // 🔐 emoji icon (prod's actual markup — no SVG)
                div { class: "mx-auto flex h-14 w-14 items-center justify-center rounded-2xl bg-blue-500/15 ring-1 ring-blue-500/20",
                    span { class: "text-3xl", role: "img", aria_label: "Sign in required", "🔐" }
                }
                div { class: "space-y-2",
                    h3 { class: "portfolio-prod-signin-title text-lg font-semibold tracking-tight text-blue-100",
                        "Sign In Required"
                    }
                    p { class: "portfolio-prod-signin-sub text-sm leading-relaxed text-blue-300",
                        "To view your portfolio, you need basic authentication."
                    }
                }
                // Benefits hint — subtle, adds value without duplicating banner
                div { class: "grid grid-cols-3 gap-2 rounded-xl bg-blue-500/5 p-3 ring-1 ring-blue-500/10",
                    div { class: "flex flex-col items-center gap-1",
                        Icon { name: "heart".to_string(), size: Some(16), class_name: Some("text-blue-400".to_string()) }
                        span { class: "text-[11px] font-medium text-blue-300", "Watchlist" }
                    }
                    div { class: "flex flex-col items-center gap-1",
                        Icon { name: "layers".to_string(), size: Some(16), class_name: Some("text-blue-400".to_string()) }
                        span { class: "text-[11px] font-medium text-blue-300", "Groups" }
                    }
                    div { class: "flex flex-col items-center gap-1",
                        Icon { name: "shield".to_string(), size: Some(16), class_name: Some("text-blue-400".to_string()) }
                        span { class: "text-[11px] font-medium text-blue-300", "Synced" }
                    }
                }
                // Primary "Sign In" button — bright blue
                a { class: "portfolio-prod-signin-btn inline-flex w-full items-center justify-center gap-2 px-4 py-3 bg-blue-600 text-white rounded-xl hover:bg-blue-700 font-semibold text-center",
                    href: PORTFOLIO_SIGN_IN_PATH,
                    Icon { name: "log-in".to_string(), size: Some(16), class_name: Some("text-white".to_string()) }
                    "Sign In"
                }
                // "Learn More" link — blue text
                a { class: "portfolio-prod-signin-link inline-flex w-full items-center justify-center gap-1.5 px-4 py-2 text-blue-400 hover:text-blue-300 font-medium text-sm text-center",
                    href: "/contact",
                    "Learn More"
                    Icon { name: "arrow-right".to_string(), size: Some(12) }
                }
                // Footer — "Need help?"
                p { class: "portfolio-prod-signin-footer text-xs leading-relaxed text-blue-400 border-t border-blue-500/10 pt-4",
                    "Need help? Check our support documentation or contact support."
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::auth::user::AuthMethod;
    use crate::auth::User;

    fn authed_ctx() -> PageContext {
        PageContext {
            user: Some(User {
                id: "u1".to_string(),
                address: "0x1234…abcd".to_string(),
                chain_id: "56".to_string(),
                roles: vec!["user".to_string()],
                email: Some("test@epsx.io".to_string()),
                tier: Some("Pro".to_string()),
                permissions: vec![],
                last_login_at: None,
                auth_method: AuthMethod::Wallet,
                display_name: Some("Test".to_string()),
            }),
            path: "/portfolio".to_string(),
            ..Default::default()
        }
    }

    fn anon_ctx() -> PageContext {
        PageContext {
            user: None,
            path: "/portfolio".to_string(),
            ..Default::default()
        }
    }

    fn connected_anon_ctx() -> PageContext {
        PageContext {
            wallet: crate::auth::wallet_button::ConnectedWalletState {
                address: Some("0x1234567890abcdef1234567890abcdef12345678".to_string()),
                connector_id: Some("metaMask".to_string()),
                chain_id: Some(56),
                ..Default::default()
            },
            ..anon_ctx()
        }
    }

    fn watchlist_ctx(symbols: &[&str]) -> PageContext {
        let mut ctx = authed_ctx();
        ctx.params.insert(
            PORTFOLIO_WATCHLIST_STATE_PARAM.to_string(),
            "ready".to_string(),
        );
        ctx.params.insert(
            PORTFOLIO_WATCHLIST_DATA_PARAM.to_string(),
            serde_json::json!({
                "groups": [],
                "ungrouped": symbols,
                "watched": symbols.len()
            })
            .to_string(),
        );
        ctx
    }

    #[test]
    fn authenticated_portfolio_fails_closed_with_meaningful_alternatives() {
        let (_meta, el) = render(&authed_ctx());
        let html = dioxus_ssr::render_element(el);

        for marker in [
            "data-portfolio-state=\"unavailable\"",
            "Your portfolio cannot be verified right now",
            "No securities, prices, rankings, plan access, or watchlist membership are being inferred.",
            "aria-label=\"Portfolio alternatives\"",
            "href=\"/account\"",
            "href=\"/contact\"",
        ] {
            assert!(html.contains(marker), "missing truthful marker `{marker}`: {html}");
        }
        assert!(!html.contains("href=\"/portfolio\""));
        assert!(!html.contains("> Retry</a>"));
    }

    #[test]
    fn canned_and_malformed_portfolio_payloads_are_ignored() {
        for payload in [
            r#"{"holdings":[{"symbol":"CANNED_TICKER","price":"$987.65","rank":"Premium","eps":"EPS ▲"}]}"#,
            r#"{"watchlist":["CANNED_WATCHLIST_ITEM"],"live":true}"#,
            "{malformed",
        ] {
            let mut ctx = authed_ctx();
            ctx.params
                .insert("data_portfolio".to_string(), payload.to_string());
            let (_meta, el) = render(&ctx);
            let html = dioxus_ssr::render_element(el);

            assert!(html.contains("data-portfolio-state=\"unavailable\""));
            for forbidden in [
                "CANNED_TICKER",
                "$987.65",
                "Premium",
                "EPS ▲",
                "CANNED_WATCHLIST_ITEM",
                "portfolio-prod-stock-card",
                "portfolio-prod-search-input",
            ] {
                assert!(
                    !html.contains(forbidden),
                    "legacy payload or unsupported control `{forbidden}` must not render: {html}"
                );
            }
        }
    }

    #[test]
    fn authenticated_portfolio_has_no_sample_financial_or_entitlement_claims() {
        let (_meta, el) = render(&authed_ctx());
        let html = dioxus_ssr::render_element(el);

        for forbidden in [
            "AAPL",
            "MSFT",
            "NVDA",
            "$189.45",
            "+2.34%",
            "Your Watchlist",
            "Unlock Full Analytics Access",
            "Top 100 stock rankings",
            "Real-time EPS data",
            "AI-powered insights",
            "Sign In Free",
            ">Live<",
        ] {
            assert!(
                !html.contains(forbidden),
                "unsupported portfolio or entitlement claim `{forbidden}` must not render: {html}"
            );
        }
        assert!(html.contains("data-portfolio-freshness=\"unavailable\""));
    }

    #[test]
    fn authenticated_portfolio_renders_persisted_watchlist_and_mutation_controls() {
        let (_meta, el) = render(&watchlist_ctx(&["AAPL", "BRK.B"]));
        let html = dioxus_ssr::render_element(el);

        for marker in [
            "data-portfolio-state=\"ready\"",
            "data-portfolio-freshness=\"ready\"",
            "2 watched",
            "data-watchlist-form=\"true\"",
            "data-watchlist-add=\"true\"",
            "action=\"/portfolio/watch\"",
            "action=\"/portfolio/unwatch\"",
            "data-watchlist-toggle=\"true\"",
            "data-symbol=\"AAPL\"",
            "data-symbol=\"BRK.B\"",
            "Unwatch AAPL",
            "Ungrouped",
            "data-watchlist-new-group-name=\"true\"",
            "data-watchlist-group-create=\"true\"",
            "Move to group",
        ] {
            assert!(
                html.contains(marker),
                "missing watchlist marker `{marker}`: {html}"
            );
        }
        assert!(!html.contains("data-portfolio-state=\"unavailable\""));
        assert!(!html.contains("$189.45"));
        assert!(!html.contains("Premium"));
    }

    #[test]
    fn authenticated_empty_watchlist_invites_a_real_first_watch() {
        let (_meta, el) = render(&watchlist_ctx(&[]));
        let html = dioxus_ssr::render_element(el);

        for marker in [
            "data-portfolio-state=\"empty\"",
            "data-watchlist-empty=\"true\"",
            "0 watched",
            "No stocks watched yet",
            "Your watchlist is saved to your account.",
            ">Watch<",
            "href=\"/analytics\"",
        ] {
            assert!(
                html.contains(marker),
                "missing empty marker `{marker}`: {html}"
            );
        }
        assert!(!html.contains("Data unavailable"));
    }

    #[test]
    fn malformed_owner_watchlist_fails_closed() {
        let mut ctx = watchlist_ctx(&["AAPL"]);
        ctx.params.insert(
            PORTFOLIO_WATCHLIST_DATA_PARAM.to_string(),
            serde_json::json!({
                "groups": [],
                "ungrouped": ["../AAPL"],
                "watched": 1
            })
            .to_string(),
        );
        let (_meta, el) = render(&ctx);
        let html = dioxus_ssr::render_element(el);

        assert!(html.contains("data-portfolio-state=\"unavailable\""));
        assert!(!html.contains("data-watchlist-add=\"true\""));
        assert!(!html.contains("../AAPL"));
    }

    #[test]
    fn grouped_portfolio_exposes_drag_multi_group_and_keyboard_fallbacks() {
        let group_id = Uuid::new_v4();
        let mut ctx = authed_ctx();
        ctx.params.insert(
            PORTFOLIO_WATCHLIST_STATE_PARAM.to_string(),
            "ready".to_string(),
        );
        ctx.params.insert(
            PORTFOLIO_WATCHLIST_DATA_PARAM.to_string(),
            serde_json::json!({
                "groups": [{
                    "id": group_id,
                    "name": "Long term",
                    "position": 0,
                    "symbols": ["AAPL"]
                }],
                "ungrouped": ["MSFT"],
                "watched": 2
            })
            .to_string(),
        );
        let (_meta, element) = render(&ctx);
        let html = dioxus_ssr::render_element(element);
        for marker in [
            "data-watchlist-group-handle=\"true\"",
            "data-watchlist-item-handle=\"true\"",
            "data-watchlist-group-rename=\"true\"",
            "data-watchlist-group-delete=\"true\"",
            "data-watchlist-add-groups-menu=\"true\"",
            "data-watchlist-remove-membership=\"true\"",
            "data-watchlist-move-item=\"up\"",
            "data-watchlist-move-group=\"down\"",
            "Add to groups",
            "Remove from this group",
            "Unwatch",
            "Ungrouped",
        ] {
            assert!(
                html.contains(marker),
                "missing organizer marker `{marker}`: {html}"
            );
        }
    }

    #[test]
    fn signed_out_portfolio_keeps_truthful_require_sign_in_state() {
        let (_meta, el) = render(&anon_ctx());
        let html = dioxus_ssr::render_element(el);

        for marker in [
            "portfolio-prod-require-signin",
            "portfolio-prod-signin",
            "Sign In Required",
            "To view your portfolio, you need basic authentication.",
            "href=\"/auth?return_url=%2Fportfolio\"",
            "href=\"/contact\"",
            "aria-label=\"Sign in required\"",
        ] {
            assert!(
                html.contains(marker),
                "missing signed-out marker `{marker}`: {html}"
            );
        }

        assert_eq!(html.matches(PORTFOLIO_SIGN_IN_PATH).count(), 2);
        assert!(html.contains("Unlock Full Analytics Access"));
        assert!(!html.contains("href=\"/auth\""));
        assert!(!html.contains("data-portfolio-state=\"unavailable\""));
        assert!(!html.contains("portfolio-prod-stock-card"));
        assert!(!html.contains("portfolio-prod-search-input"));
        assert!(!html.contains("portfolio-prod-upsell"));
    }

    #[test]
    fn connected_wallet_without_session_still_uses_the_source_sign_in_gate() {
        let (_meta, el) = render(&connected_anon_ctx());
        let html = dioxus_ssr::render_element(el);

        assert!(html.contains("portfolio-prod-signin"));
        assert!(html.contains("Sign In Required"));
        assert!(!html.contains("data-portfolio-state=\"unavailable\""));
        assert!(!html.contains("Live preview"));
    }
}
