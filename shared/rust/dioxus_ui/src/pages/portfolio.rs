//! `/portfolio` — authenticated, owner-scoped watchlist management.
//!
//! The BFF supplies only the verified session owner's persisted symbols. This
//! page renders that contract and exposes progressive Watch/Unwatch controls;
//! it does not infer holdings, prices, ranks, or plan access in the frontend.

pub mod hydrated;
use super::PageContext;
use super::PageMeta;
use crate::enterprise::FrontendIcon as Icon;
use crate::layout::main_layout::MainLayout;
use crate::pages::analytics::normalize_watchlist_symbol;
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

pub fn render(ctx: &PageContext) -> (PageMeta, Element) {
    let meta = PageMeta::app("Saved companies");
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
                div { class: "fe-saved-page portfolio-prod-container",
                    PortfolioHeader { freshness, watched_count }
                    if ctx.user.is_none() {
                        PortfolioSignInCard {}
                    } else if let Some(layout) = ready_layout {
                        PortfolioWatchlist { layout }
                    } else {
                        PortfolioUnavailable { source_shape: true }
                    }
                }
            }
        },
    )
}

#[component]
fn PortfolioHeader(freshness: String, watched_count: usize) -> Element {
    rsx! {
        header { class: "fe-saved-header",
            div {
                p { class: "fe-eyebrow", "YOUR COLLECTION" }
                h1 { "Saved companies" }
                p { "Companies you save, organized your way." }
            }
            div { class: "fe-saved-header-actions",
                span { class: "fe-badge", "data-portfolio-freshness": freshness,
                    if freshness == "ready" { "{watched_count} saved" }
                    else if freshness == "signed_out" { "Sign in to sync" }
                    else { "Data unavailable" }
                }
                crate::fullstack::shell::ShellLink { class: "fe-button", href: "/analytics", "Explore rankings"
                    Icon { name: "arrow-up-right".to_string(), size: Some(16) }
                }
            }
        }
    }
}

#[component]
fn PortfolioWatchlist(layout: WatchlistLayoutData) -> Element {
    let controls = try_use_context::<hydrated::PortfolioControls>();
    let mut new_name = use_signal(String::new);
    let mut new_open = use_signal(|| false);
    use_effect(move || {
        if let Some(controls) = controls {
            if (controls.completed)() > 0 {
                new_open.set(false);
                new_name.set(String::new());
            }
        }
    });
    let add = move |event: FormEvent| {
        if let Some(controls) = controls {
            event.prevent_default();
            let values = event.values();
            let symbol = values
                .iter()
                .find(|(name, _)| name == "symbol")
                .and_then(|(_, value)| match value {
                    dioxus::html::FormValue::Text(value) => Some(value.clone()),
                    _ => None,
                })
                .unwrap_or_default();
            let group_ids = values
                .iter()
                .filter(|(name, _)| name == "group_ids")
                .filter_map(|(_, value)| match value {
                    dioxus::html::FormValue::Text(value) => value.parse().ok(),
                    _ => None,
                })
                .collect();
            controls
                .change
                .call(hydrated::WatchlistCommand::Save { symbol, group_ids });
        }
    };
    let empty = layout.watched == 0;
    let groups = layout.groups.clone();
    let group_count = layout.groups.len();
    rsx! {
        section {
            class: "fe-saved-organizer",
            "data-fe-saved-board": "true",
            aria_busy: "false",
            "data-portfolio-state": if empty { "empty" } else { "ready" },
            "data-watchlist-organizer": "true",
            // Stats ribbon when there is content — fast scannability
            if !empty {
                div { class: "fe-watch-counts fe-saved-counts",
                    div { class: "portfolio-stat-pill flex flex-col items-center justify-center rounded-2xl border border-slate-700/60 bg-slate-900/50 px-3 py-3 backdrop-blur-sm fe-surface",
                        span { class: "text-[11px] font-semibold uppercase tracking-widest text-slate-400 fe-tone-muted", "Saved" }
                        span { class: "mt-1 text-xl font-black text-white fe-tone-text", "{layout.watched}" }
                    }
                    div { class: "portfolio-stat-pill flex flex-col items-center justify-center rounded-2xl border border-slate-700/60 bg-slate-900/50 px-3 py-3 backdrop-blur-sm fe-surface",
                        span { class: "text-[11px] font-semibold uppercase tracking-widest text-slate-400 fe-tone-muted", "Groups" }
                        span { class: "mt-1 text-xl font-black text-white fe-tone-text", "{group_count}" }
                    }
                    div { class: "portfolio-stat-pill flex flex-col items-center justify-center rounded-2xl border border-emerald-500/20 bg-emerald-500/10 px-3 py-3 backdrop-blur-sm",
                        span { class: "text-[11px] font-semibold uppercase tracking-widest text-emerald-300/80 fe-tone-positive", "Ungrouped" }
                        span { class: "mt-1 text-xl font-black text-emerald-300 fe-tone-positive", span { "data-fe-ungrouped-count": "true", "{layout.ungrouped.len()}" } }
                    }
                }
            }
            form {
                class: "portfolio-watchlist-search fe-saved-add flex flex-col gap-3 rounded-2xl border border-slate-700/50 bg-slate-900/60 p-3 backdrop-blur-sm sm:flex-row sm:items-center sm:p-2.5 fe-surface",
                action: "/portfolio/watch",
                method: "post",
                "data-watchlist-form": "true", onsubmit: add,
                div { class: "flex min-w-0 flex-1 items-center gap-3 rounded-xl border border-slate-600/50 bg-slate-950/70 px-3 py-2.5 focus-within:border-emerald-500/50 focus-within:ring-2 focus-within:ring-emerald-500/20 fe-surface",
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
                        "aria-label": "Save a company by symbol",
                        style: "text-transform: uppercase;",
                    }
                }
                if !groups.is_empty() {
                    details { class: "relative shrink-0 sm:flex-none", "data-watch-group-picker": "true",
                        summary { class: "flex min-h-11 w-full cursor-pointer list-none items-center justify-center gap-1.5 rounded-xl border border-slate-600 bg-slate-800/70 px-4 py-2 text-sm font-semibold text-slate-200 hover:bg-slate-700/70 sm:w-auto fe-surface",
                            Icon { name: "layers".to_string(), size: Some(14) }
                            "Choose groups"
                        }
                        div { class: "absolute right-0 z-30 mt-2 min-w-64 space-y-1 rounded-xl border border-slate-700 bg-slate-900 p-3 shadow-2xl fe-surface",
                            p { class: "px-2 pb-2 text-xs font-semibold uppercase tracking-wide text-slate-400 fe-tone-muted", "Add to groups" }
                            for group in groups.iter() {
                                label { class: "flex min-h-10 cursor-pointer items-center gap-3 rounded-lg px-2 py-1 text-sm text-slate-200 hover:bg-slate-800",
                                    input { r#type: "checkbox", name: "group_ids", value: "{group.id}", class: "rounded border-slate-600 text-emerald-600 focus:ring-emerald-500 fe-tone-positive" }
                                    span { class: "truncate", "{group.name}" }
                                }
                            }
                            p { class: "border-t border-slate-800 px-2 pt-2 text-xs text-slate-500 fe-tone-muted", "No selection means Ungrouped." }
                        }
                    }
                }
                button {
                    class: "inline-flex min-h-11 shrink-0 items-center justify-center gap-2 rounded-xl bg-emerald-600 px-6 py-2.5 font-semibold text-white shadow-lg shadow-emerald-900/20 transition-all hover:bg-emerald-500 hover:shadow-emerald-900/30 disabled:cursor-wait disabled:opacity-60 active:scale-[0.98] sm:self-stretch fe-tone-text fe-action-primary",
                    r#type: "submit",
                    "data-watchlist-add": "true", disabled: controls.map(|value|(value.pending)()).unwrap_or(false),
                    "aria-busy": "false",
                    Icon { name: "plus".to_string(), size: Some(16) }
                    "Save"
                }
            }
            div { class: "fe-saved-help",
                p { id: "fe-saved-drag-help",
                    Icon { name: "grip-vertical".to_string(), size: Some(16) }
                    "Drag to reorder or move between groups."
                }
                p { role: "status", aria_live: "polite", aria_atomic: "true",
                    "data-watchlist-feedback": "true", "Changes save automatically."
                }
            }
            p { class: "sr-only", id: "fe-saved-keyboard-help",
                "Press Space on a drag handle to pick up a card. Use arrow keys to choose a position, Tab to change groups, and Space or Enter to drop. Escape cancels. You can also use Move to group and the Actions menu."
            }

            details { class: "fe-saved-create group/create relative", "data-watchlist-new-group": "true", open:new_open(),
                summary { onclick:move |event| {if controls.is_some(){event.prevent_default();new_open.toggle();}},
                    class: "flex min-h-11 w-full cursor-pointer list-none items-center justify-center gap-2 rounded-xl border border-dashed border-slate-600/70 text-sm font-semibold text-emerald-300 transition-colors hover:border-emerald-500/60 hover:text-emerald-200 fe-tone-positive",
                    Icon { name: "plus".to_string(), size: Some(14) }
                    "New group"
                }
                div { class: "portfolio-new-group mt-2 flex flex-col gap-3 rounded-2xl border border-slate-700 bg-slate-900/60 p-4 backdrop-blur-sm sm:flex-row sm:items-end fe-surface",
                    div { class: "min-w-0 flex-1",
                        label { class: "mb-2 flex items-center gap-1.5 text-xs font-semibold text-slate-300 fe-tone-muted", r#for: "portfolio-new-group",
                            Icon { name: "folder-open".to_string(), size: Some(12), class_name: Some("text-emerald-400".to_string()) }
                            "Group name"
                        }
                        input {
                            id: "portfolio-new-group",
                            class: "min-h-11 w-full rounded-xl border border-slate-600 bg-slate-950 px-3.5 text-white placeholder:text-slate-500 outline-none focus:border-emerald-500 focus:ring-2 focus:ring-emerald-500/20 fe-surface fe-tone-text",
                            r#type: "text",
                            maxlength: "50",
                            placeholder: "e.g. Long term",
                            "data-watchlist-new-group-name": "true", value: new_name(), oninput: move |event| new_name.set(event.value()),
                        }
                    }
                    button {
                        class: "inline-flex min-h-11 items-center justify-center gap-1.5 rounded-xl bg-emerald-600 px-6 font-semibold text-white shadow-lg shadow-emerald-900/20 transition-all hover:bg-emerald-500 disabled:cursor-wait disabled:opacity-60 sm:self-auto fe-tone-text fe-action-primary",
                        r#type: "button",
                        "data-watchlist-group-create": "true", onclick: move |_| { if let Some(controls)=controls { controls.change.call(hydrated::WatchlistCommand::CreateGroup{name:new_name()}); } },
                        "aria-busy": "false",
                        Icon { name: "check".to_string(), size: Some(14) }
                        "Create"
                    }
                }
            }

            if empty {
                div {
                    class: "portfolio-empty-state flex min-h-[340px] flex-col items-center justify-center rounded-3xl border border-dashed border-slate-700 bg-slate-900/50 p-8 text-center backdrop-blur-sm fe-surface",
                    "data-watchlist-empty": "true",
                    div { class: "flex h-20 w-20 items-center justify-center rounded-3xl bg-emerald-500/10 text-emerald-300 ring-1 ring-emerald-500/20 fe-tone-positive",
                        Icon { name: "heart".to_string(), size: Some(36) }
                    }
                    h2 { class: "mt-6 text-2xl font-semibold tracking-tight text-white fe-tone-text", "No saved companies yet" }
                    p { class: "mt-2 max-w-lg text-sm leading-relaxed text-slate-300 fe-tone-muted",
                        "Enter a company symbol above or use Save in Explore. Your saved companies are kept in your account."
                    }
                    div { class: "mt-6 flex flex-col items-center gap-3 sm:flex-row",
                        a { class: "inline-flex items-center justify-center gap-2 rounded-xl bg-emerald-600 px-6 py-3 text-sm font-semibold text-white shadow-lg shadow-emerald-900/20 transition-colors hover:bg-emerald-500 fe-tone-text fe-action-primary", href: "/analytics",
                            Icon { name: "bar-chart-3".to_string(), size: Some(16) }
                            "Explore rankings"
                        }
                        span { class: "hidden text-xs text-slate-500 sm:inline fe-tone-muted", "or" }
                        span { class: "text-xs text-slate-500 fe-tone-muted", "try e.g. AAPL · MSFT · NVDA" }
                    }
                }
            } else {
                div { class: "fe-saved-groups", "data-watchlist-groups": "true",
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
    let controls = try_use_context::<hydrated::PortfolioControls>();
    let mut name = use_signal(|| group.name.clone());
    let drop_layout = layout.clone();
    let drop_group = move |event: DragEvent| {
        event.prevent_default();
        if let Some(mut controls) = controls {
            let dragged = (controls.dragged)();
            controls.dragged.set(None);
            match dragged {
                Some(hydrated::Dragged::Item(symbol, source)) => controls
                    .organize
                    .call(hydrated::Organize::MoveTo(symbol, source, Some(group.id))),
                Some(hydrated::Dragged::Group(id)) => {
                    if let (Some(from), Some(to)) = (
                        drop_layout
                            .groups
                            .iter()
                            .position(|candidate| candidate.id == id),
                        drop_layout
                            .groups
                            .iter()
                            .position(|candidate| candidate.id == group.id),
                    ) {
                        controls.organize.call(hydrated::Organize::GroupMove(
                            id,
                            to as isize - from as isize,
                        ));
                    }
                }
                None => {}
            }
        }
    };
    rsx! {
        section {
            class: "portfolio-group fe-saved-group", ondragover: move |event| event.prevent_default(), ondrop: drop_group,
            "data-watchlist-group": "true",
            "data-group-id": "{group.id}",
            header {
                class: "mb-4 flex flex-col gap-3 sm:flex-row sm:items-center",
                draggable: "false",
                "data-group-id": "{group.id}",
                div { class: "flex min-w-0 flex-1 items-center gap-2",
                    button { class: "flex h-9 w-9 shrink-0 cursor-grab touch-none items-center justify-center rounded-lg border border-slate-700/60 bg-slate-800/60 text-slate-400 hover:bg-slate-700/80 hover:text-slate-200 active:cursor-grabbing fe-surface fe-tone-muted", r#type: "button", title: "Drag group", "aria-label": "Drag {group.name}", "data-watchlist-group-handle": "true", onkeydown: move |event| {if let Some(controls)=controls {let key=event.key();if key==Key::Tab && (controls.dragged)().is_none(){return;}
                    if matches!(key,Key::Enter|Key::Escape|Key::Tab|Key::ArrowUp|Key::ArrowDown|Key::ArrowLeft|Key::ArrowRight)||key==Key::Character(" ".into()){event.prevent_default();controls.keyboard.call((hydrated::Dragged::Group(group.id),key));}}}, draggable: "true", ondragstart: move |_| { if let Some(mut controls)=controls { controls.dragged.set(Some(hydrated::Dragged::Group(group.id))); } }, aria_describedby: "fe-saved-keyboard-help",
                        Icon { name: "grip-vertical".to_string(), size: Some(16) }
                    }
                    div { class: "flex min-w-0 flex-1 items-center gap-2 rounded-xl border border-transparent bg-transparent px-1 hover:border-slate-600/60 hover:bg-slate-800/30 focus-within:border-emerald-500/50 focus-within:bg-slate-800/60",
                        input {
                            class: "min-h-9 min-w-0 flex-1 rounded-lg bg-transparent px-2 text-[15px] font-bold tracking-tight text-slate-900 outline-none dark:text-white fe-tone-text",
                            r#type: "text",
                            maxlength: "50",
                            value: name(), oninput: move |event| name.set(event.value()),
                            "aria-label": "Rename {group.name}",
                            "data-watchlist-group-name": "true",
                        }
                        span { class: "mr-1 shrink-0 rounded-full bg-emerald-500/15 px-2.5 py-1 text-xs font-semibold text-emerald-300 ring-1 ring-emerald-500/20 fe-tone-positive", span { "data-fe-group-count": "true", "{group.symbols.len()}" } }
                    }
                }
                div { class: "flex flex-wrap items-center gap-1.5",
                    button { class: "inline-flex min-h-9 items-center justify-center rounded-lg border border-slate-600 bg-slate-800/70 px-3 text-xs font-semibold text-slate-200 hover:bg-slate-700/80 fe-surface", r#type: "button", "data-watchlist-group-rename": "true", onclick: move |_| { if let Some(controls)=controls { controls.change.call(hydrated::WatchlistCommand::RenameGroup{id:group.id,name:name()}); } }, "data-group-id": "{group.id}",
                        Icon { name: "check".to_string(), size: Some(12) }
                        "Save name"
                    }
                    button { class: "inline-flex h-8 w-8 items-center justify-center rounded-lg border border-slate-600 bg-slate-800/50 text-slate-400 hover:bg-slate-700/60 hover:text-slate-200 fe-surface fe-tone-muted", r#type: "button", "data-watchlist-move-group": "up", onclick: move |_| { if let Some(controls)=controls { controls.organize.call(hydrated::Organize::GroupMove(group.id,-1)); } }, "data-group-id": "{group.id}", title: "Move group up", "aria-label": "Move group up",
                        Icon { name: "arrow-up-down".to_string(), size: Some(14) }
                    }
                    button { class: "inline-flex h-8 w-8 items-center justify-center rounded-lg border border-slate-600 bg-slate-800/50 text-slate-400 hover:bg-slate-700/60 hover:text-slate-200 fe-surface fe-tone-muted", r#type: "button", "data-watchlist-move-group": "down", onclick: move |_| { if let Some(controls)=controls { controls.organize.call(hydrated::Organize::GroupMove(group.id,1)); } }, "data-group-id": "{group.id}", title: "Move group down", "aria-label": "Move group down",
                        Icon { name: "chevron-down".to_string(), size: Some(14) }
                    }
                    button { class: "inline-flex min-h-9 items-center justify-center gap-1 rounded-lg border border-red-500/20 bg-red-500/10 px-3 text-xs font-semibold text-red-300 hover:bg-red-500/15 fe-tone-danger", r#type: "button", "data-watchlist-group-delete": "true", onclick: move |_| { if let Some(controls)=controls { controls.change.call(hydrated::WatchlistCommand::DeleteGroup{id:group.id}); } }, "data-group-id": "{group.id}",
                        Icon { name: "trash-2".to_string(), size: Some(12) }
                        "Delete"
                    }
                }
            }
            div {
                class: "fe-saved-card-grid",
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
                p { class: "fe-saved-drop-empty", "data-fe-empty-group": "true", hidden: !group.symbols.is_empty(),
                    Icon { name: "plus".to_string(), size: Some(20) } "Drop companies here"
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
    let controls = try_use_context::<hydrated::PortfolioControls>();
    rsx! {
        section {
            class: "portfolio-group fe-saved-group fe-saved-ungrouped",
            "data-watchlist-group": "true",
            "data-group-id": "ungrouped", ondragover: move |event|event.prevent_default(), ondrop:move |event| {event.prevent_default();if let Some(mut controls)=controls {let dragged=(controls.dragged)();controls.dragged.set(None);if let Some(hydrated::Dragged::Item(symbol,source))=dragged {controls.organize.call(hydrated::Organize::MoveTo(symbol,source,None));}}},
            header { class: "mb-4 flex items-center justify-between gap-3",
                div { class: "flex items-center gap-3",
                    div { class: "hidden h-9 w-9 items-center justify-center rounded-xl bg-slate-700/50 text-slate-400 sm:flex fe-fill-neutral fe-tone-muted",
                        Icon { name: "inbox".to_string(), size: Some(16) }
                    }
                    div {
                        h2 { class: "flex items-center gap-2 text-[15px] font-bold text-slate-900 dark:text-white fe-tone-text", "Ungrouped" }
                        p { class: "text-xs text-slate-500 dark:text-slate-400 fe-tone-muted", "Drag a symbol onto a group above to organize it" }
                    }
                }
                span { class: "rounded-full bg-slate-700 px-3 py-1 text-xs font-semibold text-slate-200 ring-1 ring-slate-600 fe-fill-neutral", span { "data-fe-group-count": "true", "{symbols.len()}" } }
            }
            div {
                class: "fe-saved-card-grid",
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
                p { class: "fe-saved-drop-empty", "data-fe-empty-group": "true", hidden: !symbols.is_empty(),
                    Icon { name: "inbox".to_string(), size: Some(20) } "Drop companies here to keep them ungrouped."
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
    let controls = try_use_context::<hydrated::PortfolioControls>();
    let mut memberships = use_signal(|| {
        all_groups
            .iter()
            .filter(|group| group.symbols.contains(&symbol))
            .map(|group| group.id)
            .collect::<Vec<Uuid>>()
    });
    let owned_symbol = use_signal(|| symbol.clone());
    let organize = move |action| {
        if let Some(controls) = controls {
            controls.organize.call(action);
        }
    };
    let current_group_value = current_group
        .map(|id| id.to_string())
        .unwrap_or_else(|| "ungrouped".to_string());
    let membership_label = if membership_count == 1 {
        "1 group".to_string()
    } else {
        format!("{membership_count} groups")
    };
    rsx! {
        article { class: "fe-saved-card", draggable: "false",
            "data-watchlist-item": "true", "data-symbol": "{symbol}",
            "data-group-id": "{current_group_value}", "data-membership-count": "{membership_count}",
            span { class: "fe-saved-card-status", Icon { name: "bookmark".to_string(), size: Some(14) } "Saved" }
            button { class: "fe-icon-button fe-saved-handle", r#type: "button", "data-watchlist-item-handle": "true", onkeydown: move |event| {if let Some(controls)=controls {let key=event.key();if key==Key::Tab && (controls.dragged)().is_none(){return;}
                    if matches!(key,Key::Enter|Key::Escape|Key::Tab|Key::ArrowUp|Key::ArrowDown|Key::ArrowLeft|Key::ArrowRight)||key==Key::Character(" ".into()){event.prevent_default();controls.keyboard.call((hydrated::Dragged::Item(owned_symbol(),current_group),key));}}}, draggable: "true", ondragstart: move |_| {if let Some(mut controls)=controls { controls.dragged.set(Some(hydrated::Dragged::Item(owned_symbol(),current_group))); } },
                aria_label: "Drag {symbol}", aria_describedby: "fe-saved-keyboard-help", title: "Drag to organize",
                Icon { name: "grip-vertical".to_string(), size: Some(16) }
            }
            div { class: "fe-saved-card-identity",
                h3 { "{symbol}" }
                p { "data-fe-membership-label": "true", if membership_count > 0 { "{membership_label}" } else { "Ready to organize" } }
            }
            label { class: "fe-saved-card-move",
                span { class: "flex items-center gap-1", Icon { name: "arrow-right-left".to_string(), size: Some(12) } "Move to group" }
                select { class: "min-h-10 w-full rounded-xl border border-slate-700 bg-slate-900/60 px-3 py-2 text-sm text-slate-200 outline-none focus:border-emerald-500/50 focus:ring-2 focus:ring-emerald-500/20 fe-surface", "data-watchlist-move-to-group": "true", onchange: move |event| { let ids=event.value().parse::<Uuid>().ok().into_iter().collect(); organize(hydrated::Organize::Membership(owned_symbol(),ids)); }, aria_label: "Move {symbol} to group",
                    option { value: "ungrouped", selected: current_group.is_none(), "Ungrouped" }
                    for group in all_groups.iter().cloned() {
                        option { value: "{group.id}", selected: current_group == Some(group.id), "{group.name}" }
                    }
                }
            }

            details { class: "fe-row-menu fe-saved-card-menu",
                summary { class: "fe-icon-button", aria_label: "Actions for {symbol}",
                    Icon { name: "ellipsis".to_string(), size: Some(18) }
                }
                div { class: "fe-row-menu-panel",
            details { class: "relative", "data-watchlist-add-groups-menu": "true",
                summary { class: "flex min-h-10 cursor-pointer list-none items-center justify-center gap-1.5 rounded-xl border border-slate-600 bg-slate-800/60 px-3 py-2 text-center text-xs font-semibold text-slate-200 hover:bg-slate-700/60 fe-surface",
                    Icon { name: "plus".to_string(), size: Some(12) }
                    "Add to groups"
                }
                div { class: "absolute left-0 right-0 z-20 mt-2 space-y-1 rounded-xl border border-slate-700 bg-slate-900 p-3 shadow-2xl fe-surface",
                    p { class: "px-1 pb-1 text-xs font-semibold uppercase tracking-wide text-slate-400 fe-tone-muted", "Memberships" }
                    for group in all_groups.iter().cloned() {
                        label { class: "flex min-h-9 cursor-pointer items-center gap-3 rounded-lg px-2 text-sm text-slate-200 hover:bg-slate-800",
                            input {
                                r#type: "checkbox",
                                value: "{group.id}",
                                checked: group.symbols.iter().any(|candidate| candidate == &symbol),
                                disabled: group.symbols.iter().any(|candidate| candidate == &symbol),
                                class: "rounded border-slate-600 bg-slate-800 text-emerald-600 focus:ring-emerald-500 disabled:opacity-40 fe-fill-neutral fe-tone-positive",
                                "data-watchlist-membership-choice": "true", onchange: move |event| { let mut values=memberships(); values.retain(|id|id!=&group.id); if event.checked(){values.push(group.id);} memberships.set(values); },
                            }
                            span { class: "truncate", "{group.name}" }
                            span { class: "ml-auto text-[11px] text-emerald-400 fe-tone-positive", "data-fe-membership-added": "true", hidden: !group.symbols.iter().any(|candidate| candidate == &symbol), "added" }
                        }
                    }
                    if all_groups.is_empty() {
                        p { class: "px-2 py-2 text-sm text-slate-500 fe-tone-muted", "Create a group first." }
                    } else {
                        button { class: "mt-2 inline-flex min-h-10 w-full items-center justify-center gap-1.5 rounded-xl bg-emerald-600 px-3 text-sm font-semibold text-white hover:bg-emerald-500 fe-tone-text fe-action-primary", r#type: "button", "data-watchlist-groups-save": "true", onclick: move |_| organize(hydrated::Organize::Membership(owned_symbol(),memberships())), "data-symbol": "{symbol}",
                            Icon { name: "check".to_string(), size: Some(14) }
                            "Save groups"
                        }
                    }
                }
            }
            div { class: "flex items-center gap-1 rounded-xl border border-slate-700/60 bg-slate-800/30 p-1 fe-surface",
                button { class: "inline-flex min-h-8 flex-1 items-center justify-center gap-1 rounded-lg bg-slate-800 px-2 text-xs font-medium text-slate-300 hover:bg-slate-700 hover:text-white fe-fill-neutral fe-tone-muted", r#type: "button", "data-watchlist-move-item": "up", onclick: move |_| organize(hydrated::Organize::ItemMove(owned_symbol(),current_group,-1)),
                    Icon { name: "arrow-up".to_string(), size: Some(12) }
                    "Up"
                }
                div { class: "h-4 w-px bg-slate-700 fe-fill-neutral" }
                button { class: "inline-flex min-h-8 flex-1 items-center justify-center gap-1 rounded-lg bg-slate-800 px-2 text-xs font-medium text-slate-300 hover:bg-slate-700 hover:text-white fe-fill-neutral fe-tone-muted", r#type: "button", "data-watchlist-move-item": "down", onclick: move |_| organize(hydrated::Organize::ItemMove(owned_symbol(),current_group,1)),
                    Icon { name: "chevron-down".to_string(), size: Some(12) }
                    "Down"
                }
            }
            div { class: "fe-saved-menu-removals flex flex-col gap-2 border-t border-slate-700/60 pt-3",
                {
                    rsx! { button { hidden: current_group.is_none(), class: "inline-flex min-h-10 flex-1 items-center justify-center gap-1 rounded-xl border border-slate-600 bg-slate-800/60 px-3 text-xs font-semibold text-slate-300 hover:bg-slate-700/60 fe-surface fe-tone-muted", r#type: "button", "data-watchlist-remove-membership": "true", onclick: move |_| {if let Some(id)=current_group {organize(hydrated::Organize::RemoveMembership(owned_symbol(),id));}},
                        Icon { name: "x".to_string(), size: Some(12) }
                        "Remove from this group"
                    } }
                }
                form { class: "flex-1", action: "/portfolio/unwatch", method: "post", onsubmit: move |event| { if let Some(controls)=controls {event.prevent_default();controls.change.call(hydrated::WatchlistCommand::Remove{symbol:owned_symbol()});} },
                    input { r#type: "hidden", name: "symbol", value: "{symbol}" }
                    button {
                        class: "inline-flex min-h-10 w-full items-center justify-center gap-1.5 rounded-xl border border-pink-500/20 bg-pink-500/10 px-3 text-sm font-semibold text-pink-300 hover:bg-pink-500/15 disabled:cursor-wait disabled:opacity-60 fe-tone-accent",
                        r#type: "submit",
                        "data-watchlist-toggle": "true",
                        "data-symbol": "{symbol}",
                        "data-watchlisted": "true",
                        "data-membership-count": "{membership_count}",
                        "aria-label": "Remove {symbol} from saved companies",
                        "aria-busy": "false",
                        Icon { name: "trash".to_string(), size: Some(14) }
                        "Remove saved company"
                    }
                }
            }

                }
            }
            p { class: "fe-saved-card-feedback", role: "status", aria_live: "polite", "data-watchlist-item-feedback": "true" }
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
            div { class: "h-1 bg-gradient-to-r from-emerald-400 via-teal-400 to-cyan-400 fe-fill-neutral" }
            div { class: if source_shape { "space-y-6 p-4 sm:space-y-8 sm:p-8" } else { "space-y-8 p-5 sm:p-8" },
                div {
                    class: if source_shape { "portfolio-watchlist-search flex min-w-0 items-center gap-2 rounded-xl border border-slate-700 bg-slate-800/50 px-3 py-3 text-xs text-slate-400 opacity-75 sm:gap-3 sm:px-5 sm:py-4 sm:text-base" } else { "portfolio-watchlist-search flex items-center gap-3 rounded-2xl border border-slate-600 bg-slate-800/70 px-5 py-4 text-base text-slate-400 sm:text-xl" },
                    role: "searchbox",
                    aria_disabled: "true",
                    Icon { name: "search".to_string(), size: Some(if source_shape { 16 } else { 22 }), class_name: Some("text-slate-500".to_string()) }
                    span { class: "min-w-0 truncate", "Enter a company symbol to save…" }
                    span { class: "ml-auto hidden items-center gap-1 rounded-full bg-amber-500/10 px-2.5 py-1 text-xs font-medium text-amber-300 ring-1 ring-amber-500/20 sm:inline-flex fe-tone-warning",
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
                        "Saved companies are unavailable"
                    }
                    p { class: if source_shape { "mt-3 max-w-lg text-sm leading-relaxed text-slate-400 sm:text-base" } else { "mt-3 max-w-2xl text-base leading-relaxed text-slate-400 sm:text-lg" },
                        "We couldn’t load your saved companies. Try again to continue saving and organizing."
                    }
                    p { class: "sr-only",
                        "Your saved companies could not be verified. Your list will appear when it is available."
                    }
                }

                div { class: "rounded-2xl border border-slate-700/60 bg-slate-800/30 p-4 backdrop-blur-sm fe-surface",
                    p { class: "text-xs font-semibold uppercase tracking-wide text-slate-400 fe-tone-muted", "Try instead" }
                    nav {
                        class: "mt-3 flex flex-col gap-2 sm:flex-row",
                        aria_label: "Portfolio alternatives",
                        a {
                            class: "inline-flex flex-1 items-center justify-center gap-2 rounded-xl border border-slate-600 bg-slate-800 px-4 py-2.5 text-sm font-semibold text-slate-200 hover:bg-slate-700 fe-surface",
                            href: "/account",
                            Icon { name: "user".to_string(), size: Some(16) }
                            " Return to account"
                        }
                        a {
                            class: "inline-flex flex-1 items-center justify-center gap-2 rounded-xl border border-slate-700 bg-transparent px-4 py-2.5 text-sm font-semibold text-slate-300 hover:bg-slate-800/50 hover:text-slate-200 fe-tone-muted",
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
        div { class: "rounded-xl border border-slate-700 bg-slate-800/50 p-4 fe-surface",
            div { class: "flex items-center gap-2 font-semibold text-white fe-tone-text",
                Icon { name: icon.to_string(), size: Some(18) }
                "{title}"
            }
            p { class: "mt-2 text-sm leading-6 text-slate-300 fe-tone-muted", "{body}" }
            span { class: "mt-3 inline-flex rounded-full border border-amber-500/30 bg-amber-500/10 px-2 py-1 text-xs font-medium text-amber-400 fe-tone-warning",
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
        section { class: "fe-saved-welcome portfolio-prod-require-signin", "data-portfolio-state": "signed_out",
            div { class: "fe-saved-welcome-copy portfolio-prod-signin",
                span { class: "fe-saved-welcome-icon", Icon { name: "bookmark".to_string(), size: Some(24) } }
                h2 { "A space for companies you want to revisit." }
                p { "Save companies, build your own groups, and move cards into an order that makes sense to you." }
                a { class: "fe-button fe-primary", href: PORTFOLIO_SIGN_IN_PATH, "Sign in to get started"
                    Icon { name: "arrow-right".to_string(), size: Some(16) }
                }
                p { class: "fe-saved-welcome-note", Icon { name: "lock".to_string(), size: Some(14) } "Sign in to view and organize your saved companies." }
            }
            div { class: "fe-saved-welcome-art", aria_hidden: "true",
                div { class: "fe-saved-art-heading", span { "Your collection" } Icon { name: "ellipsis".to_string(), size: Some(20) } }
                div { class: "fe-saved-art-grid",
                    div { class: "fe-saved-art-card", Icon { name: "bookmark".to_string(), size: Some(16) } i {} i {} }
                    div { class: "fe-saved-art-slot" }
                    div { class: "fe-saved-art-card fe-saved-art-lifted", Icon { name: "grip-vertical".to_string(), size: Some(18) } i {} i {} }
                }
                p { "Pick up. Move. Make it yours." }
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
            "Your saved companies could not be verified",
            "Your list will appear when it is available.",
            "aria-label=\"Portfolio alternatives\"",
            "href=\"/account\"",
            "href=\"/contact\"",
        ] {
            assert!(
                html.contains(marker),
                "missing truthful marker `{marker}`: {html}"
            );
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
            "2 saved",
            "data-watchlist-form=\"true\"",
            "data-watchlist-add=\"true\"",
            "action=\"/portfolio/watch\"",
            "action=\"/portfolio/unwatch\"",
            "data-watchlist-toggle=\"true\"",
            "data-symbol=\"AAPL\"",
            "data-symbol=\"BRK.B\"",
            "Remove AAPL from saved companies",
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
            "0 saved",
            "No saved companies yet",
            "Your saved companies are kept in your account.",
            ">Save<",
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
            "data-fe-saved-board=\"true\"",
            "class=\"fe-saved-card\"",
            "fe-saved-keyboard-help",
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
            "Remove saved company",
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
            "A space for companies you want to revisit.",
            "Sign in to view and organize your saved companies.",
            "href=\"/auth?return_url=%2Fportfolio\"",
            "lucide-lock",
        ] {
            assert!(
                html.contains(marker),
                "missing signed-out marker `{marker}`: {html}"
            );
        }

        assert_eq!(html.matches(PORTFOLIO_SIGN_IN_PATH).count(), 1);
        assert!(!html.contains("Unlock Full Analytics Access"));
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
        assert!(html.contains("Sign in to view and organize your saved companies."));
        assert!(!html.contains("data-portfolio-state=\"unavailable\""));
        assert!(!html.contains("Live preview"));
    }
}
