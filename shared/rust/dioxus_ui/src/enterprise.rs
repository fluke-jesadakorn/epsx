//! Frontend presentation. Data, access and mutation decisions stay in their existing owners.
use self::FrontendIcon as Icon;
use crate::components::stock_data_card::StockCardWatchlist;
use crate::pages::analytics::{AnalyticsRow, WatchlistData};
use crate::primitives::Icon as SharedIcon;
use dioxus::prelude::*;

/// Additional outline icons for the frontend; shared consumers keep their icon defaults.
#[component]
pub fn FrontendIcon(name: String, size: Option<u32>, class_name: Option<String>) -> Element {
    let path = match name.as_str() {
        "bookmark" => "<path d='M19 21l-7-4-7 4V5a2 2 0 0 1 2-2h10a2 2 0 0 1 2 2z'/>",
        "ellipsis" => "<circle cx='5' cy='12' r='1'/><circle cx='12' cy='12' r='1'/><circle cx='19' cy='12' r='1'/>",
        "grip-vertical" => "<circle cx='9' cy='5' r='1'/><circle cx='15' cy='5' r='1'/><circle cx='9' cy='12' r='1'/><circle cx='15' cy='12' r='1'/><circle cx='9' cy='19' r='1'/><circle cx='15' cy='19' r='1'/>",
        "panel-left" => "<rect x='3' y='3' width='18' height='18' rx='2'/><path d='M9 3v18'/>",
        "chart-no-axes-combined" => "<path d='M3 3v18h18M7 14l4-4 4 2 6-8M7 18v-1m4 1v-5m4 5v-3m4 3V9'/>",
        "list-filter" => "<path d='M4 5h16M7 12h10M10 19h4'/>",
        "trending-down" => "<path d='m3 5 6 6 4-4 8 10M15 17h6v-6'/>",
        _ => return rsx! { SharedIcon { name, size, class_name } },
    };
    let size = size.unwrap_or(20);
    rsx! { svg { width: "{size}", height: "{size}", view_box: "0 0 24 24",
        fill: "none", stroke: "currentColor", stroke_width: "1.7",
        stroke_linecap: "round", stroke_linejoin: "round",
        class: class_name.unwrap_or_default(), "aria-hidden": "true",
        dangerous_inner_html: path
    } }
}

#[component]
pub fn PageHeader(
    title: String,
    description: String,
    #[props(default)] children: Element,
) -> Element {
    rsx! { header { class: "fe-page-header",
        div { h1 { "{title}" } p { "{description}" } } {children}
    } }
}

#[component]
pub fn DataState(
    title: String,
    message: String,
    href: String,
    #[props(default = "Try again".to_string())] action: String,
) -> Element {
    let navigation = try_consume_context::<crate::fullstack::analytics::AnalyticsNavigation>();
    let target = href.clone();
    rsx! { section { class: "fe-state", role: "status",
        div { class: "fe-state-art", aria_hidden: "true",
            Icon { name: "database".to_string(), size: Some(24) }
        }
        h2 { "{title}" } p { "{message}" }
        a { class: "fe-button", href, onclick: move |event| {
            if crate::fullstack::shell::migrated_link(&target) { crate::fullstack::analytics::follow_link(event, navigation, &target); }
        },
            Icon { name: "arrow-right".to_string(), size: Some(16) } "{action}"
        }
    } }
}

/// Public platform introduction; deliberately independent of account and market data.
pub const HOME_TITLE: &str = "EPSX — Financial Technology Platform";
pub const HOME_DESCRIPTION: &str = "EPSX is a financial technology platform that brings company information and digital tools into one connected experience.";

#[component]
pub fn HomeHero() -> Element {
    let navigation = try_consume_context::<crate::fullstack::analytics::AnalyticsNavigation>();
    rsx! {
        section { class: "fe-hero", aria_labelledby: "home-title",
            div { class: "fe-hero-copy",
                p { class: "fe-kicker", span { class: "fe-kicker-line", aria_hidden: "true" } "Financial Technology Platform" }
                h1 { id: "home-title", "Financial technology. " span { "Connected by design." } }
                p { class: "fe-lead", "{HOME_DESCRIPTION}" }
                div { class: "fe-actions",
                    a { class: "fe-button fe-primary", href: "/analytics", onclick: move |event| crate::fullstack::analytics::follow_link(event, navigation, "/analytics"),
                        "Explore platform" Icon { name: "arrow-up-right".to_string(), size: Some(18) }
                    }
                    a { class: "fe-button fe-hero-secondary", href: "/about", onclick: move |event| crate::fullstack::analytics::follow_link(event, navigation, "/about"),
                        "About EPSX" Icon { name: "arrow-right".to_string(), size: Some(18) }
                    }
                }
            }
            div { class: "fe-hero-art", aria_hidden: "true",
                svg { class: "fe-hero-architecture", view_box: "0 0 600 560", fill: "none", xmlns: "http://www.w3.org/2000/svg", "focusable": "false",
                    // Static, decorative geometry: no simulated product controls or data.
                    g { class: "fe-hero-wire", stroke_width: "1",
                        path { d: "M300 34 566 187V365L300 520 34 365V187L300 34ZM34 187 300 342 566 187M300 342V520" }
                        path { d: "M167 111 433 265V443M433 111 167 265V443M34 276 300 431 566 276" }
                    }
                    ellipse { class: "fe-hero-shadow", cx: "300", cy: "430", rx: "195", ry: "65" }
                    path { class: "fe-hero-plane fe-hero-plane-base", d: "M80 343 300 216 520 343 300 470Z" }
                    path { class: "fe-hero-plane fe-hero-plane-middle", d: "M80 282 300 155 520 282 300 409Z" }
                    g { class: "fe-hero-connectors", stroke_width: "1.5", stroke_dasharray: "3 7",
                        path { d: "M80 214V343M520 214V343M300 341V470M300 87V155" }
                    }
                    path { class: "fe-hero-plane-edge", d: "M80 214 300 341 520 214V233L300 360 80 233Z" }
                    path { class: "fe-hero-plane fe-hero-plane-top", d: "M80 214 300 87 520 214 300 341Z" }
                    g { class: "fe-hero-circuit", stroke_width: "1.5", stroke_linecap: "round", stroke_linejoin: "round",
                        path { d: "M130 214 216 264 268 234M300 116V160M470 214 384 264 332 234M300 314V270" }
                        path { d: "M173 189 216 164 248 182M427 189 384 164 352 182" }
                    }
                    g { transform: "translate(300 214) matrix(.866 .5 -.866 .5 0 0)",
                        rect { class: "fe-hero-core-halo", x: "-94", y: "-94", width: "188", height: "188", rx: "24" }
                        rect { class: "fe-hero-core", x: "-72", y: "-72", width: "144", height: "144", rx: "20" }
                        image { href: "/public/logos/epsx-icon.svg", x: "-46", y: "-46", width: "92", height: "92" }
                    }
                    for (x, y) in [(130, 214), (300, 116), (470, 214), (300, 314)] {
                        g { transform: "translate({x} {y})",
                            ellipse { class: "fe-hero-node-halo", rx: "15", ry: "9" }
                            ellipse { class: "fe-hero-node", rx: "5", ry: "3" }
                        }
                    }
                    g { class: "fe-hero-orbit", stroke_width: "1.5",
                        path { d: "M79 126 108 109 137 126 108 143Z M467 363 496 346 525 363 496 380Z" }
                        path { d: "M108 109V86M496 380V403" }
                    }
                    circle { class: "fe-hero-node", cx: "108", cy: "82", r: "3" }
                    circle { class: "fe-hero-node", cx: "496", cy: "407", r: "3" }
                }
            }
        }
    }
}

#[component]
pub fn CompanyIdentity(row: AnalyticsRow) -> Element {
    let monogram: String = row.symbol.chars().take(2).collect();
    rsx! { div { class: "fe-company",
        span { class: "fe-company-monogram", aria_hidden: "true", "{monogram}" }
        div { strong { "{row.symbol}" }
            if let Some(name) = &row.company_name { span { class: "fe-company-name", title: "{name}", "{name}" } }
        }
    } }
}

fn report_calendar_date(timestamp: i64) -> Option<chrono::NaiveDate> {
    use chrono::Datelike;
    let date = chrono::DateTime::from_timestamp(timestamp, 0)?.date_naive();
    (timestamp > 0 && date.year() <= 9999).then_some(date)
}

/// Presentation-only dates. Ranking and access decisions remain with the backend.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NextAction {
    Api(chrono::NaiveDate),
    Estimated {
        date: chrono::NaiveDate,
        previous: chrono::NaiveDate,
    },
    Unavailable,
}

impl NextAction {
    pub fn from_row(row: &AnalyticsRow) -> Self {
        if let Some(next) = row.next_earnings_date.and_then(report_calendar_date) {
            return Self::Api(next);
        }
        if let Some(previous) = row.last_earnings_date.and_then(report_calendar_date) {
            if let Some(next) = previous.checked_add_days(chrono::Days::new(90)) {
                use chrono::Datelike;
                if next.year() <= 9999 {
                    return Self::Estimated {
                        date: next,
                        previous,
                    };
                }
            }
        }
        Self::Unavailable
    }

    pub fn date(self) -> Option<chrono::NaiveDate> {
        match self {
            Self::Api(date) | Self::Estimated { date, .. } => Some(date),
            Self::Unavailable => None,
        }
    }

    pub fn relative_to(self, today: chrono::NaiveDate) -> String {
        match self
            .date()
            .map(|date| date.signed_duration_since(today).num_days())
        {
            Some(0) => "Today".into(),
            Some(days) if days < 0 => "Date passed".into(),
            Some(1) => "In 1 day".into(),
            Some(days) => format!("In {days} days"),
            None => "Save this company to revisit".into(),
        }
    }
}

#[component]
fn NextActionDate(action: NextAction, today: chrono::NaiveDate) -> Element {
    rsx! { div { class: "fe-next-action", "data-next-action-source": match action { NextAction::Api(_) => "api", NextAction::Estimated { .. } => "estimated", NextAction::Unavailable => "unavailable" },
        div { class: "fe-event-date",
            if let Some(date) = action.date() {
                time { datetime: date.to_string(), {date.format("%d %B %Y").to_string()} }
                if matches!(action, NextAction::Estimated { .. }) { span { class: "fe-badge", "Estimated" } }
            } else { span { "Date not available" } }
        }
        small { if action.date().is_some() { "Company report · " } "{action.relative_to(today)}" }
    } }
}

#[component]
fn ReportContent(
    row: AnalyticsRow,
    today: chrono::NaiveDate,
    watch: Option<StockCardWatchlist>,
    #[props(default = "/analytics".to_string())] return_path: String,
) -> Element {
    let action = NextAction::from_row(&row);
    rsx! { div { class: "fe-report-content",
        div { class: "fe-report-heading",
            div { p { class: "fe-eyebrow", "{row.symbol} / COMPANY DETAILS" }
                h3 { "{row.company_name.as_deref().unwrap_or(&row.symbol)}" }
            }
            if let Some(state) = watch { div { class: "fe-watch", FrontendWatch { symbol: row.symbol.clone(), state, return_path: return_path.clone() } } }
        }
        div { class: "fe-event-details",
            div { p { class: "fe-eyebrow", "NEXT ACTION" } NextActionDate { action, today } }
            div { class: "fe-event-context",
                ReportDateContext { action, today }
            }
        }
    } }
}

fn watch_state(
    row: &AnalyticsRow,
    signed_in: bool,
    watchlist: &Option<WatchlistData>,
    state: &str,
) -> StockCardWatchlist {
    if !signed_in {
        StockCardWatchlist::SignedOut
    } else if state == "ready" && watchlist.is_some() {
        StockCardWatchlist::Ready {
            is_watchlisted: watchlist.as_ref().is_some_and(|list| {
                list.symbols
                    .iter()
                    .any(|symbol| symbol.eq_ignore_ascii_case(&row.symbol))
            }),
        }
    } else {
        StockCardWatchlist::Unavailable
    }
}

#[component]
fn FrontendWatch(
    symbol: String,
    #[props(default)] in_place: bool,
    state: StockCardWatchlist,
    #[props(default = "/analytics".to_string())] return_path: String,
) -> Element {
    if try_use_context::<crate::fullstack::shell::AuthRevision>().is_some() {
        if let StockCardWatchlist::Ready { is_watchlisted } = state {
            return rsx! { crate::pages::portfolio::hydrated::WatchButton { symbol, initially_saved: is_watchlisted, in_place } };
        }
    }
    let mut query = url::form_urlencoded::Serializer::new(String::new());
    query.append_pair("return_url", &return_path);
    let sign_in = format!("/auth?{}", query.finish());
    match state {
        StockCardWatchlist::SignedOut => rsx! {
            a { href: sign_in, "data-watchlist-signed-out": "true",
                "data-symbol": symbol.clone(), aria_label: "Sign in to save {symbol}",
                Icon { name: "bookmark".to_string(), size: Some(17) } span { "Save" }
            }
        },
        StockCardWatchlist::Ready { is_watchlisted } => rsx! {
            button { r#type: "button", "data-watchlist-toggle": "true", "data-symbol": symbol.clone(),
                "data-watchlist-in-place": in_place.then_some("true"),
                "data-watchlisted": if is_watchlisted { "true" } else { "false" },
                aria_pressed: if is_watchlisted { "true" } else { "false" }, aria_busy: "false",
                aria_label: if is_watchlisted { format!("Saved · Remove {symbol} from saved companies") } else { format!("Save {symbol}") },
                Icon { name: "bookmark".to_string(), size: Some(17) }
                span { "data-watchlist-label": "true", if is_watchlisted { "Saved" } else { "Save" } }
            }
        },
        StockCardWatchlist::Unavailable => rsx! {
            button { r#type: "button", disabled: true, "data-watchlist-unavailable": "true", "data-symbol": symbol.clone(),
                aria_label: "Saving unavailable for {symbol}",
                Icon { name: "bookmark".to_string(), size: Some(17) } span { "Save" }
            }
        },
    }
}

#[component]
fn ReportDateContext(action: NextAction, today: chrono::NaiveDate) -> Element {
    rsx! {
                match action {
                    NextAction::Api(_) => rsx! { strong { "Company report date" } p { "Date supplied by the data provider. It may change." } },
                    NextAction::Estimated { previous, .. } => rsx! {
                        strong { "Estimated 90 days after the previous company report." }
                        p { "Previous company report: " time { datetime: previous.to_string(), {previous.format("%d %B %Y").to_string()} } }
                    },
                    NextAction::Unavailable => rsx! { strong { "No usable report date is available." } p { "Save this company and check back for a date." } },
                }
                if action.date().is_some_and(|date| date < today) {
                    p { "This date has passed. The date alone does not tell us whether a report has been published." }
                }
                p { "Use this date to plan when to revisit. Saving a company does not set a reminder or subscribe you to notifications. Dates use UTC." }
    }
}

/// Match the production symbol link, encoding the symbol as one path segment.
fn tradingview_symbol_url(symbol: &str) -> String {
    let mut url =
        url::Url::parse("https://www.tradingview.com/symbols/").expect("static TradingView URL");
    url.path_segments_mut()
        .expect("TradingView URL has a path")
        .pop_if_empty()
        .push(symbol);
    url.into()
}

/// Shared ranking cards for Explore and the public homepage previews.
#[component]
pub fn RankingCards(
    rows: Vec<AnalyticsRow>,
    #[props(default)] signed_in: bool,
    #[props(default)] watchlist: Option<WatchlistData>,
    #[props(default)] watchlist_state: String,
    #[props(default = true)] show_watch: bool,
    #[props(default = "/analytics".to_string())] return_path: String,
) -> Element {
    let today = use_server_cached(|| chrono::Utc::now().date_naive());
    rsx! {
        div { class: "fe-ranking-results", "data-section": "analytics-card-grid",
            div { class: "fe-ranking-grid", aria_label: "Company rankings",
                for row in &rows {
                    {
                        rsx! { article { class: "fe-ranking-card", "data-stock-card": "true", "data-symbol": row.symbol.clone(), "data-rank": "{row.rank}",
                            header { class: "fe-ranking-card-top",
                                span { class: "fe-card-rank", "Rank " strong { "#{row.rank}" } }
                                if show_watch {
                                    div { class: "fe-watch", FrontendWatch { symbol: row.symbol.clone(), state: watch_state(row, signed_in, &watchlist, &watchlist_state), in_place: true, return_path: return_path.clone() } }
                                }
                            }
                            div { class: "fe-card-company",
                                h2 { class: "fe-card-symbol", "{row.symbol}" }
                                p { class: "fe-card-company-name", title: row.company_name.clone().unwrap_or_default(), {row.company_name.as_deref().unwrap_or("Company name unavailable")} }
                            }
                            CardNextAction { action: NextAction::from_row(row), today, previous_report: row.last_earnings_date }
                            a { class: "fe-card-details", href: tradingview_symbol_url(&row.symbol),
                                target: "_blank", rel: "noopener noreferrer", "data-tradingview-details": "true",
                                aria_label: "View details for {row.symbol} on TradingView (opens in a new tab)",
                                title: "Open {row.symbol} on TradingView in a new tab",
                                "View details" Icon { name: "arrow-up-right".to_string(), size: Some(16) }
                            }
                            if show_watch { p { class: "fe-card-feedback", "data-fe-company-feedback": "card", "data-symbol": row.symbol.clone(), role: "status", aria_live: "polite" } }
                        } }
                    }
                }
            }
            if show_watch { p { class: "fe-watch-feedback", "data-watchlist-feedback": "true", role: "status", aria_live: "polite" } }
        }
    }
}

#[component]
fn CardNextAction(
    action: NextAction,
    today: chrono::NaiveDate,
    #[props(default)] previous_report: Option<i64>,
) -> Element {
    let relative = if action.date().is_some() {
        action.relative_to(today)
    } else {
        "Date not available".into()
    };
    rsx! { div { class: "fe-card-next", "data-next-action-source": match action { NextAction::Api(_) => "api", NextAction::Estimated { .. } => "estimated", NextAction::Unavailable => "unavailable" },
        p { class: "fe-card-next-label", Icon { name: "calendar".to_string(), size: Some(13) } "Next action"
            if matches!(action, NextAction::Estimated { .. }) { span { class: "fe-badge", "Estimated" } }
        }
        strong { class: "fe-card-countdown", "{relative}" }
        p { class: "fe-card-report-date", "Company report"
            if let Some(date) = action.date() {
                span { aria_hidden: "true", " · " }
                time { datetime: date.to_string(), {date.format("%d %B %Y").to_string()} }
            }
        }
        ReportProgress { action, today, previous_report }

    } }
}

/// A calendar interval, never a company rating or a report-completion signal.
fn report_progress_days(
    action: NextAction,
    previous_report: Option<i64>,
    today: chrono::NaiveDate,
) -> Option<(i64, i64)> {
    let end = action.date()?;
    let start = match action {
        NextAction::Estimated { previous, .. } => previous,
        _ => previous_report.and_then(report_calendar_date)?,
    };
    let total = end.signed_duration_since(start).num_days();
    if total <= 0 || today < start {
        return None;
    }
    let elapsed = today
        .signed_duration_since(start)
        .num_days()
        .clamp(0, total);
    Some((elapsed, total))
}

#[component]
fn ReportProgress(
    action: NextAction,
    today: chrono::NaiveDate,
    previous_report: Option<i64>,
) -> Element {
    if let Some((elapsed, total)) = report_progress_days(action, previous_report, today) {
        let state = if action.date().is_some_and(|date| date < today) {
            "past"
        } else {
            "active"
        };
        let explanation = if state == "past" {
            "The report date has passed. This does not indicate that the report has been published."
                .to_string()
        } else {
            format!("{elapsed} of {total} calendar days elapsed between the previous company report and the {}report date.", if matches!(action, NextAction::Estimated { .. }) { "estimated " } else { "next " })
        };
        rsx! {
            progress { class: "fe-report-progress", "data-report-progress": state,
                value: "{elapsed}", max: "{total}", aria_label: "Company report timeline",
                aria_valuetext: explanation.clone(), title: explanation.clone(),
                "{elapsed} of {total} days"
            }
            p { class: "fe-progress-explanation", "{explanation}" }
        }
    } else {
        rsx! {
            div { class: "fe-report-progress fe-report-progress-unavailable", "data-report-progress": "unavailable",
                role: "img", aria_label: "Report timeline unavailable", title: "A timeline needs valid previous and next report dates." }
            p { class: "fe-progress-explanation", "A timeline is unavailable without valid previous and next report dates." }
        }
    }
}

#[component]
pub fn MarketTable(
    rows: Vec<AnalyticsRow>,
    #[props(default)] signed_in: bool,
    #[props(default)] watchlist: Option<WatchlistData>,
    #[props(default)] watchlist_state: String,
    #[props(default = true)] show_watch: bool,
    #[props(default = "/analytics".to_string())] return_path: String,
) -> Element {
    // One UTC reference date per render; the native browser runtime preserves SSR text.
    let today = chrono::Utc::now().date_naive();
    rsx! { div { class: "fe-market-results", "data-section": "analytics-card-grid",
        if show_watch { p { class: "fe-watch-feedback", "data-watchlist-feedback": "true", role: "status", aria_live: "polite" } }
        table { class: "fe-market-table", aria_label: "Company rankings",
            caption { class: "sr-only", "Company rankings and next company report dates. Rankings are generated using EPSX’s proprietary methodology." }
            thead { tr {
                th { scope: "col", class: "fe-expand-cell", span { class: "sr-only", "Details" } }
                th { scope: "col", class: "fe-rank", "Rank" }
                th { scope: "col", "Company" }
                th { scope: "col", class: "fe-event-cell", "Next action" }
                if show_watch { th { scope: "col", class: "fe-watch-cell", span { class: "sr-only", "Save company" } } }
            } }
            tbody {
                for row in &rows {
                    {
                        let watch = watch_state(row, signed_in, &watchlist, &watchlist_state);
                        let target = format!("fe-event-{}", row.symbol);
                        rsx! {
                            tr { class: "fe-company-row", "data-stock-card": "true", "data-symbol": row.symbol.clone(), "data-rank": "{row.rank}",
                                td { class: "fe-expand-cell",
                                    button { class: "fe-expand", r#type: "button", "data-epsx-action": "fe-report-toggle",
                                        aria_expanded: "false", aria_controls: target.clone(), aria_label: "Company details · {row.symbol}",
                                        Icon { name: "chevron-right".to_string(), size: Some(16) }
                                    }
                                }
                                td { class: "fe-rank", "{row.rank}" }
                                td { CompanyIdentity { row: row.clone() } }
                                td { class: "fe-event-cell", NextActionDate { action: NextAction::from_row(row), today } }
                                if show_watch { td { class: "fe-watch-cell", div { class: "fe-watch", FrontendWatch { symbol: row.symbol.clone(), state: watch.clone(), return_path: return_path.clone() } } } }
                            }
                            tr { id: target, hidden: true, class: "fe-detail-row",
                                td { colspan: if show_watch { "5" } else { "4" }, ReportContent { row: row.clone(), today, watch: show_watch.then_some(watch), return_path: return_path.clone() } }
                            }
                        }
                    }
                }
            }
        }
        div { class: "fe-mobile-market", aria_label: "Company rankings",
            for row in &rows {
                {
                    let watch = watch_state(row, signed_in, &watchlist, &watchlist_state);
                    rsx! { article { class: "fe-market-card", "data-stock-card": "true", "data-symbol": row.symbol.clone(), "data-rank": "{row.rank}",
                        header {
                            span { class: "fe-rank", "{row.rank}" }
                            CompanyIdentity { row: row.clone() }
                            if show_watch { div { class: "fe-watch", FrontendWatch { symbol: row.symbol.clone(), state: watch.clone(), return_path: return_path.clone() } } }
                        }
                        div { class: "fe-mobile-event", span { class: "fe-eyebrow", "Next action" } NextActionDate { action: NextAction::from_row(row), today } }
                        details { class: "fe-report",
                            summary { "Company details · {row.symbol}" Icon { name: "chevron-down".to_string(), size: Some(14) } }
                            ReportContent { row: row.clone(), today, watch: show_watch.then_some(watch), return_path: return_path.clone() }
                        }
                    } }
                }
            }
        }
    } }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn explore_cards_have_native_tradingview_links_and_preserve_save_return_links() {
        let mut first = row();
        first.company_name = Some("A very long company & research name <example>".into());
        let mut second = first.clone();
        second.rank = 42;
        second.symbol = "BRK.B".into();
        let path = "/analytics?page=2&country=america&sector=Technology&sort_by=price&min_eps=2";
        let html = dioxus_ssr::render_element(
            rsx! { RankingCards { rows: vec![first, second], return_path: path.to_string() } },
        );
        assert_eq!(html.matches("data-stock-card=\"true\"").count(), 2);
        assert_eq!(html.matches("data-tradingview-details=\"true\"").count(), 2);
        assert_eq!(html.matches("target=\"_blank\"").count(), 2);
        assert_eq!(html.matches("rel=\"noopener noreferrer\"").count(), 2);
        assert!(html.contains("href=\"https://www.tradingview.com/symbols/BRK.B\""));
        assert!(html.contains("on TradingView (opens in a new tab)"));
        assert!(!html.contains("<dialog"));
        assert!(!html.contains("fe-company-open"));
        assert!(html.contains("View details"));
        assert!(html.contains("%2Fanalytics%3Fpage%3D2%26country%3Damerica%26sector%3DTechnology%26sort_by%3Dprice%26min_eps%3D2"));
        assert!(!html.contains("<table"));
        assert!(!html.contains("99999"));
        assert!(!html.contains("data-watchlist-toggle"));
        assert!(html.find("data-rank=\"7\"").unwrap() < html.find("data-rank=\"42\"").unwrap());
    }

    #[test]
    fn only_explore_save_controls_opt_into_confirmed_in_place_updates() {
        let state = StockCardWatchlist::Ready {
            is_watchlisted: true,
        };
        let default = dioxus_ssr::render_element(
            rsx! { FrontendWatch { symbol: "TEST".to_string(), state: state.clone() } },
        );
        let card = dioxus_ssr::render_element(
            rsx! { FrontendWatch { symbol: "TEST".to_string(), state, in_place: true } },
        );
        assert!(!default.contains("data-watchlist-in-place"));
        assert!(card.contains("data-watchlist-in-place=\"true\""));
        assert!(card.contains("aria-pressed=\"true\""));
        assert!(card.contains("data-watchlist-label=\"true\">Saved"));
    }

    #[test]
    fn card_days_are_primary_while_shared_date_presentation_remains_absolute_first() {
        let today = chrono::NaiveDate::from_ymd_opt(2026, 9, 8).unwrap();
        let action = NextAction::Api(today + chrono::Days::new(63));
        let card = dioxus_ssr::render_element(rsx! { CardNextAction { action, today } });
        let shared = dioxus_ssr::render_element(rsx! { NextActionDate { action, today } });
        assert!(card.find("In 63 days").unwrap() < card.find("10 November 2026").unwrap());
        assert!(shared.find("10 November 2026").unwrap() < shared.find("In 63 days").unwrap());
        for (action, expected) in [
            (NextAction::Api(today), "Today"),
            (NextAction::Api(today - chrono::Days::new(1)), "Date passed"),
            (NextAction::Unavailable, "Date not available"),
        ] {
            let html = dioxus_ssr::render_element(rsx! { CardNextAction { action, today } });
            assert!(html.contains(expected));
            assert!(!html.contains("Confirmed"));
        }
    }

    #[test]
    fn report_progress_uses_actual_api_interval_and_ninety_day_estimate() {
        let today = date("2026-09-08");
        let api = NextAction::Api(date("2026-11-26"));
        assert_eq!(
            report_progress_days(api, Some(timestamp("2026-08-27")), today),
            Some((12, 91))
        );
        let estimated = NextAction::Estimated {
            previous: date("2024-02-29"),
            date: date("2024-05-29"),
        };
        assert_eq!(
            report_progress_days(estimated, None, date("2024-03-30")),
            Some((30, 90))
        );
        assert_eq!(
            report_progress_days(api, Some(timestamp("2026-08-27") + 86399), today),
            Some((12, 91))
        );
    }

    #[test]
    fn missing_or_inconsistent_report_intervals_are_not_fake_percentages() {
        let action = NextAction::Api(date("2026-11-26"));
        for previous in [
            None,
            Some(0),
            Some(-1),
            Some(i64::MAX),
            Some(timestamp("2026-11-26")),
            Some(timestamp("2026-12-01")),
            Some(timestamp("2026-10-01")),
        ] {
            assert_eq!(
                report_progress_days(action, previous, date("2026-09-08")),
                None
            );
        }
        let html = dioxus_ssr::render_element(
            rsx! { ReportProgress { action, today: date("2026-09-08"), previous_report: None } },
        );
        assert!(html.contains("Report timeline unavailable"));
        assert!(!html.contains("<progress"));
        assert!(!html.contains("aria-valuenow"));
    }

    #[test]
    fn timeline_caps_at_date_without_claiming_report_publication() {
        let action = NextAction::Api(date("2026-09-08"));
        for today in ["2026-09-08", "2026-09-09"] {
            assert_eq!(
                report_progress_days(action, Some(timestamp("2026-08-01")), date(today)),
                Some((38, 38))
            );
        }
        let html = dioxus_ssr::render_element(
            rsx! { ReportProgress { action, today: date("2026-09-09"), previous_report: Some(timestamp("2026-08-01")) } },
        );
        assert!(html.contains("data-report-progress=\"past\""));
        assert!(html.contains("does not indicate that the report has been published"));
    }

    fn row() -> AnalyticsRow {
        serde_json::from_value(serde_json::json!({"rank":7,"symbol":"TEST","latest_date":"2099-06-30","value":99999.0,"active_status":"active","quarterly_performance":[],"current_eps":12345.67,"price_current":98765.43,"growth_factor":3.0})).unwrap()
    }
    fn date(value: &str) -> chrono::NaiveDate {
        chrono::NaiveDate::parse_from_str(value, "%Y-%m-%d").unwrap()
    }
    fn timestamp(value: &str) -> i64 {
        date(value)
            .and_hms_opt(0, 0, 0)
            .unwrap()
            .and_utc()
            .timestamp()
    }
    #[test]
    fn api_date_wins_and_utc_ignores_provider_relative_fields() {
        let mut row = row();
        row.next_earnings_date = Some(timestamp("2026-09-08") + 86399);
        row.last_earnings_date = Some(timestamp("2026-08-01"));
        row.days_until_next_earnings = Some(500);
        row.next_earnings_date_formatted = Some("2099-01-01".into());
        let action = NextAction::from_row(&row);
        assert_eq!(action, NextAction::Api(date("2026-09-08")));
        assert_eq!(action.relative_to(date("2026-09-08")), "Today");
        assert_eq!(action.relative_to(date("2026-09-09")), "Date passed");
        assert_eq!(action.relative_to(date("2026-09-07")), "In 1 day");
        assert_eq!(action.relative_to(date("2026-09-01")), "In 7 days");
    }
    #[test]
    fn fallback_adds_ninety_calendar_days_across_month_year_and_leap_day() {
        for (previous, expected) in [
            ("2023-12-01", "2024-02-29"),
            ("2024-02-29", "2024-05-29"),
            ("2026-11-15", "2027-02-13"),
            ("2026-01-31", "2026-05-01"),
        ] {
            let mut row = row();
            row.last_earnings_date = Some(timestamp(previous));
            let action = NextAction::from_row(&row);
            assert_eq!(
                action,
                NextAction::Estimated {
                    date: date(expected),
                    previous: date(previous)
                }
            );
        }
    }
    #[test]
    fn invalid_timestamps_fall_back_or_stay_unavailable() {
        for invalid in [
            None,
            Some(0),
            Some(-1),
            Some(i64::MAX),
            Some(1_800_000_000_000),
        ] {
            let mut row = row();
            row.next_earnings_date = invalid;
            row.last_earnings_date = invalid;
            assert_eq!(NextAction::from_row(&row), NextAction::Unavailable);
            row.last_earnings_date = Some(timestamp("2026-01-01"));
            assert_eq!(NextAction::from_row(&row).date(), Some(date("2026-04-01")));
        }
        let mut row = row();
        row.last_earnings_date = Some(timestamp("9999-12-01"));
        assert_eq!(NextAction::from_row(&row), NextAction::Unavailable);
    }
    #[test]
    fn unrelated_backend_estimate_and_request_date_are_never_fallback_dates() {
        let mut row = row();
        row.next_quarter_estimate = serde_json::from_value(serde_json::json!({"quarter":"Q4","estimated_eps":54321.0,"announcement_date":"2099-12-01","announcement_timestamp":timestamp("2099-12-01"),"days_until_announcement":40,"confidence":"high"})).ok();
        assert_eq!(NextAction::from_row(&row), NextAction::Unavailable);
        let html = dioxus_ssr::render_element(rsx! { MarketTable { rows: vec![row] } });
        for hidden in [
            "99999",
            "12345.67",
            "98765.43",
            "54321",
            "2099-12-01",
            "2099-06-30",
            "EPS change",
            ">EPS<",
            "Score",
            "Confirmed",
            "polyline",
            "Quarterly details",
        ] {
            assert!(!html.contains(hidden), "unexpected presentation: {hidden}");
        }
        assert!(html.contains("Date not available"));
        assert!(html.contains("aria-controls=\"fe-event-TEST\""));
        assert!(html.contains("data-watchlist-signed-out"));
    }
    #[test]
    fn save_sign_in_links_preserve_current_filters_page_and_custom_conditions() {
        let return_path = "/analytics?page=2&limit=10&country=america&sort_by=price&min_eps=2";
        let html = dioxus_ssr::render_element(
            rsx! { MarketTable { rows: vec![row()], return_path: return_path.to_string() } },
        );
        let href = "/auth?return_url=%2Fanalytics%3Fpage%3D2%26limit%3D10%26country%3Damerica%26sort_by%3Dprice%26min_eps%3D2";
        assert_eq!(html.matches(href).count(), 4);
    }
    #[test]
    fn past_estimate_keeps_its_date_and_explains_source_without_publication_claim() {
        let mut row = row();
        row.last_earnings_date = Some(timestamp("2026-01-01"));
        let html = dioxus_ssr::render_element(
            rsx! { ReportContent { row, today: date("2026-09-08"), watch: Some(StockCardWatchlist::SignedOut) } },
        );
        for expected in [
            "01 April 2026",
            "Date passed",
            "Estimated 90 days after the previous company report.",
            "01 January 2026",
            "does not tell us whether a report has been published",
            "Save",
        ] {
            assert!(html.contains(expected), "missing {expected}");
        }
    }
}
