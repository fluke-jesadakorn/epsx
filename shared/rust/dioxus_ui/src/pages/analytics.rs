//! `/analytics` — server-rendered, backend-authorized EPS ranking cards.

use std::collections::HashSet;

use chrono::DateTime;
use dioxus::prelude::*;
use serde::{Deserialize, Serialize};

use super::{PageContext, PageMeta};
use crate::components::stock_data_card::{StockCardWatchlist, StockDataCard};
use crate::enterprise::{DataState, MarketTable, PageHeader};
use crate::layout::main_layout::MainLayout;
use crate::primitives::Icon;

pub const ANALYTICS_DATA_PARAM: &str = "data_analytics";
pub const ANALYTICS_STATE_PARAM: &str = "data_analytics_state";
pub const ANALYTICS_FILTERS_DATA_PARAM: &str = "data_analytics_filters";
pub const ANALYTICS_FILTERS_STATE_PARAM: &str = "data_analytics_filters_state";
pub const ANALYTICS_QUERY_PARAM: &str = "data_analytics_query";
pub const ANALYTICS_WATCHLIST_DATA_PARAM: &str = "data_analytics_watchlist";
pub const ANALYTICS_WATCHLIST_STATE_PARAM: &str = "data_analytics_watchlist_state";

#[server]
pub async fn get_analytics_rankings(
    page: u32,
    limit: u32,
) -> Result<AnalyticsResponse, ServerFnError> {
    if tokio::runtime::Handle::try_current().is_err() {
        return Err(ServerFnError::new(
            "no runtime, fallback to PageContext".to_string(),
        ));
    }
    let api_url = std::env::var("API_URL").unwrap_or_else(|_| "http://127.0.0.1:8080".to_string());
    let url = format!(
        "{}/api/analytics/rankings?page={}&limit={}",
        api_url.trim_end_matches('/'),
        page,
        limit
    );
    let resp = reqwest::get(&url)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;
    let value: serde_json::Value = resp
        .json()
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;
    let response: AnalyticsResponse =
        serde_json::from_value(value).map_err(|e| ServerFnError::new(e.to_string()))?;
    response
        .validated()
        .map_err(|_| ServerFnError::new("validation failed".to_string()))
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AnalyticsValidationError;

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct AnalyticsResponse {
    pub success: bool,
    pub data: Vec<AnalyticsRow>,
    pub pagination: AnalyticsPagination,
    pub metadata: AnalyticsMetadata,
    pub access_info: Option<AnalyticsAccessInfo>,
    pub message: Option<String>,
    pub processing_time_ms: u64,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct AnalyticsRow {
    pub rank: i32,
    pub symbol: String,
    pub company_name: Option<String>,
    pub latest_date: String,
    pub value: f64,
    pub active_status: String,
    pub quarterly_performance: Vec<QuarterlyPerformance>,
    pub next_quarter_estimate: Option<NextQuarterEstimate>,
    pub next_earnings_date: Option<i64>,
    pub last_earnings_date: Option<i64>,
    pub next_earnings_date_formatted: Option<String>,
    pub days_until_next_earnings: Option<i32>,
    pub progress_percentage: Option<f64>,
    pub current_eps: Option<f64>,
    pub growth_factor: Option<f64>,
    pub price_current: Option<f64>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct QuarterlyPerformance {
    pub quarter: String,
    pub date: String,
    pub price: f64,
    pub eps: f64,
    pub eps_growth: f64,
    pub price_growth: f64,
    pub announcement_date: Option<String>,
    pub announcement_timestamp: Option<i64>,
    #[serde(default)]
    pub is_estimated: bool,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct NextQuarterEstimate {
    pub quarter: String,
    pub estimated_eps: f64,
    pub announcement_date: String,
    pub announcement_timestamp: i64,
    pub days_until_announcement: i32,
    pub estimated_price_target: Option<f64>,
    pub confidence: String,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct AnalyticsMetadata {
    pub available_countries: Vec<String>,
    pub available_sectors: Vec<String>,
    pub request_timestamp: String,
    pub data_source: String,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct AnalyticsAccessInfo {
    pub min_accessible_rank: i32,
    pub locked_ranks_count: i32,
    #[serde(default)]
    pub max_accessible_rank: Option<i32>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct AnalyticsPagination {
    pub page: i32,
    pub limit: i32,
    pub total: i64,
    #[serde(rename = "totalPages")]
    pub total_pages: i32,
    #[serde(rename = "hasNext")]
    pub has_next: bool,
    #[serde(rename = "hasPrev")]
    pub has_prev: bool,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct AnalyticsFilters {
    pub countries: Vec<AnalyticsCountry>,
    pub sectors: Vec<String>,
    #[serde(default)]
    pub exchanges: Vec<String>,
    #[serde(default)]
    pub stock_types: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct AnalyticsCountry {
    pub value: String,
    pub label: String,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize, PartialEq, Eq)]
pub struct AnalyticsQueryState {
    pub page: u32,
    pub limit: Option<u32>,
    pub country: Option<String>,
    pub sector: Option<String>,
    pub sort_by: Option<String>,
    pub min_eps: Option<String>,
    pub min_growth: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct WatchlistData {
    pub symbols: Vec<String>,
}

fn safe_text(value: &str, max: usize) -> bool {
    !value.is_empty()
        && value.chars().count() <= max
        && !value.chars().any(|character| character.is_control())
}

pub fn normalize_watchlist_symbol(value: &str) -> Option<String> {
    let symbol = value.trim().to_ascii_uppercase();
    let mut chars = symbol.chars();
    let first = chars.next()?;
    if symbol.len() > 20
        || !first.is_ascii_alphanumeric()
        || !chars
            .all(|character| character.is_ascii_alphanumeric() || matches!(character, '.' | '-'))
    {
        return None;
    }
    Some(symbol)
}

impl WatchlistData {
    pub fn validated(self) -> Result<Self, AnalyticsValidationError> {
        let mut seen = HashSet::new();
        let mut symbols = Vec::with_capacity(self.symbols.len());
        if self.symbols.len() > 1_000 {
            return Err(AnalyticsValidationError);
        }
        for symbol in self.symbols {
            let symbol = normalize_watchlist_symbol(&symbol).ok_or(AnalyticsValidationError)?;
            if seen.insert(symbol.clone()) {
                symbols.push(symbol);
            }
        }
        Ok(Self { symbols })
    }
}

impl AnalyticsFilters {
    pub fn validated(self) -> Result<Self, AnalyticsValidationError> {
        if self.countries.len() > 500
            || self.sectors.len() > 500
            || self.exchanges.len() > 500
            || self.stock_types.len() > 500
        {
            return Err(AnalyticsValidationError);
        }
        let mut country_values = HashSet::new();
        for country in &self.countries {
            if !safe_text(&country.value, 64)
                || !safe_text(&country.label, 128)
                || !country_values.insert(country.value.as_str())
            {
                return Err(AnalyticsValidationError);
            }
        }
        for value in self
            .sectors
            .iter()
            .chain(self.exchanges.iter())
            .chain(self.stock_types.iter())
        {
            if !safe_text(value, 128) {
                return Err(AnalyticsValidationError);
            }
        }
        Ok(self)
    }
}

impl AnalyticsResponse {
    pub fn validated(self) -> Result<Self, AnalyticsValidationError> {
        let pagination = &self.pagination;
        if !self.success
            || pagination.page < 1
            || !(1..=100).contains(&pagination.limit)
            || pagination.total < 0
            || pagination.total_pages < 0
            || self.data.len() > pagination.limit as usize
            || pagination.total < self.data.len() as i64
            || !safe_text(&self.metadata.request_timestamp, 128)
            || !safe_text(&self.metadata.data_source, 128)
            || DateTime::parse_from_rfc3339(&self.metadata.request_timestamp).is_err()
        {
            return Err(AnalyticsValidationError);
        }
        if let Some(access) = &self.access_info {
            if access.min_accessible_rank < 0
                || access.locked_ranks_count < 0
                || access
                    .max_accessible_rank
                    .is_some_and(|maximum| maximum < access.min_accessible_rank)
            {
                return Err(AnalyticsValidationError);
            }
        }
        if self.metadata.available_countries.len() > 500
            || self.metadata.available_sectors.len() > 500
            || self
                .metadata
                .available_countries
                .iter()
                .chain(self.metadata.available_sectors.iter())
                .any(|value| !safe_text(value, 128))
        {
            return Err(AnalyticsValidationError);
        }
        for row in &self.data {
            if row.rank < 1
                || !valid_ranking_symbol(&row.symbol)
                || !safe_text(&row.latest_date, 128)
                || !safe_text(&row.active_status, 64)
                || row
                    .company_name
                    .as_deref()
                    .is_some_and(|value| !safe_text(value, 256))
                || row.quarterly_performance.len() > 32
                || row
                    .progress_percentage
                    .is_some_and(|value| !(0.0..=100.0).contains(&value))
            {
                return Err(AnalyticsValidationError);
            }
            for quarter in &row.quarterly_performance {
                if !safe_text(&quarter.quarter, 64)
                    || !safe_text(&quarter.date, 128)
                    || quarter
                        .announcement_date
                        .as_deref()
                        .is_some_and(|value| !safe_text(value, 128))
                {
                    return Err(AnalyticsValidationError);
                }
            }
            if let Some(estimate) = &row.next_quarter_estimate {
                if !safe_text(&estimate.quarter, 64)
                    || !safe_text(&estimate.announcement_date, 128)
                    || !safe_text(&estimate.confidence, 64)
                {
                    return Err(AnalyticsValidationError);
                }
            }
        }
        Ok(self)
    }
}

fn valid_ranking_symbol(value: &str) -> bool {
    safe_text(value, 32)
        && value.chars().all(|character| {
            character.is_ascii_alphanumeric() || matches!(character, '.' | '-' | '_' | ':' | '/')
        })
}

impl AnalyticsQueryState {
    pub fn from_normalized_query(query: &str) -> Result<Self, AnalyticsValidationError> {
        let mut state = Self {
            page: 1,
            ..Default::default()
        };
        if query.is_empty() {
            return Ok(state);
        }
        let url = url::Url::parse(&format!("https://frontend.invalid/?{query}"))
            .map_err(|_| AnalyticsValidationError)?;
        let mut seen = HashSet::new();
        for (key, value) in url.query_pairs() {
            if !seen.insert(key.to_string()) {
                return Err(AnalyticsValidationError);
            }
            match key.as_ref() {
                "page" => state.page = value.parse().map_err(|_| AnalyticsValidationError)?,
                "limit" => state.limit = Some(value.parse().map_err(|_| AnalyticsValidationError)?),
                "country" => state.country = Some(value.into_owned()),
                "sector" => state.sector = Some(value.into_owned()),
                "sort_by" => state.sort_by = Some(value.into_owned()),
                "min_eps" => state.min_eps = Some(value.into_owned()),
                "min_growth" => state.min_growth = Some(value.into_owned()),
                _ => return Err(AnalyticsValidationError),
            }
        }
        if state.page == 0 || state.limit.is_some_and(|limit| !(1..=100).contains(&limit)) {
            return Err(AnalyticsValidationError);
        }
        Ok(state)
    }

    pub fn page_url(&self, page: u32, authoritative_limit: u32) -> String {
        self.url_with(
            page,
            authoritative_limit,
            self.country.as_deref(),
            self.sector.as_deref(),
        )
    }

    pub fn is_custom_view(&self) -> bool {
        self.sort_by.is_some() || self.min_eps.is_some() || self.min_growth.is_some()
    }

    pub fn default_ranking_url(&self, authoritative_limit: u32) -> String {
        Self {
            country: self.country.clone(),
            sector: self.sector.clone(),
            ..Default::default()
        }
        .page_url(1, authoritative_limit)
    }

    pub fn clear_frontend_filters_url(&self, authoritative_limit: u32) -> String {
        Self::default().page_url(1, authoritative_limit)
    }

    pub fn reset_url(&self, authoritative_limit: u32) -> String {
        self.url_with(1, authoritative_limit, None, None)
    }

    fn url_with(
        &self,
        page: u32,
        authoritative_limit: u32,
        country: Option<&str>,
        sector: Option<&str>,
    ) -> String {
        let mut serializer = url::form_urlencoded::Serializer::new(String::new());
        serializer.append_pair("page", &page.to_string());
        serializer.append_pair("limit", &authoritative_limit.to_string());
        if let Some(country) = country {
            serializer.append_pair("country", country);
        }
        if let Some(sector) = sector {
            serializer.append_pair("sector", sector);
        }
        if let Some(sort_by) = &self.sort_by {
            serializer.append_pair("sort_by", sort_by);
        }
        if let Some(min_eps) = &self.min_eps {
            serializer.append_pair("min_eps", min_eps);
        }
        if let Some(min_growth) = &self.min_growth {
            serializer.append_pair("min_growth", min_growth);
        }
        format!("/analytics?{}", serializer.finish())
    }
}

pub fn render(ctx: &PageContext) -> (PageMeta, Element) {
    let mut meta = PageMeta::app("Company rankings");
    meta.description =
        "Explore EPSX rankings, see upcoming company reports, and save companies to revisit."
            .into();
    let surface = render_surface_mode(ctx, true);
    let body = rsx! {
        MainLayout { ctx: ctx.clone(), {surface} }
    };
    (meta, body)
}

/// Production analytics workspace without an outer application shell.
///
/// The admin BFF already owns its authenticated shell, while the public
/// frontend wraps this same surface in `MainLayout`. Keeping one body avoids
/// the two routes drifting into different information architectures again.
pub fn render_surface(ctx: &PageContext) -> Element {
    render_surface_mode(ctx, false)
}

fn render_surface_mode(ctx: &PageContext, enterprise: bool) -> Element {
    let response = ctx
        .param(ANALYTICS_DATA_PARAM)
        .and_then(|raw| serde_json::from_str::<AnalyticsResponse>(raw).ok())
        .and_then(|response| response.validated().ok());
    let filters = ctx
        .param(ANALYTICS_FILTERS_DATA_PARAM)
        .and_then(|raw| serde_json::from_str::<AnalyticsFilters>(raw).ok())
        .and_then(|filters| filters.validated().ok());
    let query = ctx
        .param(ANALYTICS_QUERY_PARAM)
        .and_then(|raw| serde_json::from_str::<AnalyticsQueryState>(raw).ok())
        .unwrap_or_else(|| AnalyticsQueryState {
            page: 1,
            ..Default::default()
        });
    let watchlist = ctx
        .param(ANALYTICS_WATCHLIST_DATA_PARAM)
        .and_then(|raw| serde_json::from_str::<WatchlistData>(raw).ok())
        .and_then(|watchlist| watchlist.validated().ok());
    let state = ctx
        .param(ANALYTICS_STATE_PARAM)
        .map(String::as_str)
        .unwrap_or("unavailable");
    let filters_state = ctx
        .param(ANALYTICS_FILTERS_STATE_PARAM)
        .map(String::as_str)
        .unwrap_or("unavailable")
        .to_string();
    let watchlist_state = ctx
        .param(ANALYTICS_WATCHLIST_STATE_PARAM)
        .map(String::as_str)
        .unwrap_or(if ctx.user.is_some() {
            "unavailable"
        } else {
            "signed_out"
        })
        .to_string();

    let body = match (state, response) {
        ("ready", Some(response)) if !response.data.is_empty() => rsx! {
            AnalyticsPage {
                enterprise,
                signed_in: ctx.user.is_some(),
                response,
                filters,
                filters_state,
                query,
                watchlist,
                watchlist_state,
            }
        },
        ("empty", Some(response)) if response.data.is_empty() => rsx! {
            AnalyticsPage {
                enterprise,
                signed_in: ctx.user.is_some(),
                response,
                filters,
                filters_state,
                query,
                watchlist,
                watchlist_state,
            }
        },
        ("malformed" | "restricted", _) => rsx! {
            AnalyticsFailurePage {
                enterprise,
                signed_in: ctx.user.is_some(),
                failure_state: state.to_string(),
                filters,
                filters_state,
                query,
            }
        },
        _ => rsx! {
            AnalyticsFailurePage {
                enterprise,
                signed_in: ctx.user.is_some(),
                failure_state: "unavailable".to_string(),
                filters,
                filters_state,
                query,
            }
        },
    };
    body
}

#[component]
pub(crate) fn AnalyticsPage(
    enterprise: bool,
    signed_in: bool,
    response: AnalyticsResponse,
    filters: Option<AnalyticsFilters>,
    filters_state: String,
    query: AnalyticsQueryState,
    watchlist: Option<WatchlistData>,
    watchlist_state: String,
) -> Element {
    if enterprise {
        let empty = response.data.is_empty();
        let reset = query.clear_frontend_filters_url(response.pagination.limit.max(1) as u32);
        return rsx! {
            div { class: "fe-page fe-explore", "data-section": "analytics-rankings",
                "data-analytics-state": if empty { "empty" } else { "ready" },
                FrontendExploreHeading {}
                section { class: "fe-data-surface fe-ranking-surface",
                    details { class: "fe-filter-disclosure", open: true,
                        summary { Icon { name: "sliders-horizontal".to_string(), size: Some(16) } "Filters" }
                        AnalyticsFilterForm { enterprise: true, filters, filters_state,
                            query: query.clone(), authoritative_limit: response.pagination.limit }
                    }
                    if empty {
                        DataState { title: "No companies available for this view",
                            message: "Try another country or clear the filters. Check your available rank range below.",
                            href: reset, action: "Clear filters" }
                    } else {
                        AnalyticsCardGrid {
                            rows: response.data,
                            signed_in,
                            watchlist,
                            watchlist_state,
                            sign_in_path: format!(
                                "/auth?return_url={}",
                                url::form_urlencoded::byte_serialize(
                                    query.page_url(
                                        response.pagination.page.max(1) as u32,
                                        response.pagination.limit.max(1) as u32,
                                    ).as_bytes(),
                                ).collect::<String>(),
                            ),
                        }
                        AnalyticsPaginationNav { pagination: response.pagination, query }
                    }
                }
                FrontendAccessNote { access: response.access_info }
            }
        };
    }
    let effective_response = response;
    let is_empty = effective_response.data.is_empty();
    rsx! {
        div { class: "analytics-page relative min-h-screen",
            if !enterprise { AnalyticsBackground {} }
            div { class: "page-content relative z-10 mx-auto max-w-7xl px-4 py-6 sm:py-8",
                AnalyticsHeader { enterprise, metadata: Some(effective_response.metadata.clone()) }
                if enterprise { details { class: "fe-help",
                    summary { "What do these figures mean?" }
                    p { "EPS means earnings per share. EPS change describes the change between reporting periods, not an investment return. Open quarterly details to review dates and estimates." }
                }
                }
                section {
                    class: "analytics-rankings overflow-visible",
                    "data-section": "analytics-rankings",
                    "data-analytics-state": if is_empty { "empty" } else { "ready" },
                    AnalyticsAccessStatus { access: effective_response.access_info.clone() }
                    AnalyticsFilterForm {
                        enterprise,
                        filters,
                        filters_state,
                        query: query.clone(),
                        authoritative_limit: effective_response.pagination.limit,
                    }
                    if is_empty {
                        div { class: "py-12 text-center", "data-section": "analytics-empty-state",
                            div { class: "mx-auto mb-4 flex h-16 w-16 items-center justify-center rounded-2xl bg-slate-800/50",
                                Icon { name: "sparkles".to_string(), size: Some(32), class_name: Some("text-slate-400".to_string()) }
                            }
                            p { class: "text-slate-400", "No rankings match the selected filters" }
                        }
                    } else {
                        if enterprise {
                            MarketTable { rows: effective_response.data.clone(), signed_in, watchlist, watchlist_state }
                        } else {
                            AnalyticsCardGrid { rows: effective_response.data.clone(), signed_in, watchlist, watchlist_state }
                        }
                        AnalyticsPaginationNav {
                            pagination: effective_response.pagination.clone(),
                            query,
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn FrontendExploreHeading() -> Element {
    rsx! { PageHeader { title: "Company rankings", description: "Rankings are generated using EPSX’s proprietary methodology.",
    } }
}

#[component]
fn FrontendAccessNote(access: Option<AnalyticsAccessInfo>) -> Element {
    let navigation = try_consume_context::<crate::fullstack::analytics::AnalyticsNavigation>();
    rsx! {
        if let Some(access) = access {
            div { class: "fe-access-note", "data-section": "analytics-plan-status",
                Icon { name: "shield-check".to_string(), size: Some(14) }
                if let Some(maximum) = access.max_accessible_rank {
                    span { "Viewing: Ranks {access.min_accessible_rank}-{maximum}" }
                } else if access.min_accessible_rank <= 1 {
                    span { "Viewing: All ranks" }
                } else { span { "Viewing: Ranks {access.min_accessible_rank}+" } }
                if access.locked_ranks_count > 0 {
                    span { "Ranks 1-{access.locked_ranks_count} locked" }
                    a { href: "/plans", onclick: move |event| crate::fullstack::analytics::follow_link(event, navigation, "/plans"), "Review plans" }
                }
            }
        }
    }
}

#[component]
fn AnalyticsBackground() -> Element {
    rsx! {
        div { class: "pointer-events-none absolute inset-0 z-0 overflow-hidden", "aria-hidden": "true",
            div { class: "absolute inset-0 bg-gradient-to-b from-white via-gray-50 to-white dark:from-slate-950 dark:via-slate-900 dark:to-slate-950" }
            div { class: "absolute -left-40 -top-40 h-[400px] w-[400px] rounded-full bg-purple-600/15 blur-3xl" }
            div { class: "absolute -right-32 top-1/3 h-[300px] w-[300px] rounded-full bg-blue-600/10 blur-3xl" }
        }
    }
}

#[component]
fn AnalyticsHeader(enterprise: bool, metadata: Option<AnalyticsMetadata>) -> Element {
    rsx! {
        header { class: "analytics-header mb-6 flex flex-col gap-4 sm:flex-row sm:items-center sm:justify-between", "data-section": "analytics-header",
            div { class: "flex min-w-0 items-center gap-3",
                div { class: "flex h-10 w-10 shrink-0 items-center justify-center rounded-xl bg-gradient-to-br from-purple-500 to-pink-500 shadow-lg",
                    Icon { name: "bar-chart-3".to_string(), size: Some(20), class_name: Some("text-white".to_string()) }
                }
                div { class: "min-w-0",
                    h1 { class: "text-2xl font-bold text-foreground", if enterprise { "Explore" } else { "Analytics" } }
                    p { class: "max-w-full text-sm text-slate-600 dark:text-slate-400", if enterprise { "Explore reported company data. Filter the list and review quarterly details." } else { "Top-performing stocks by EPS growth" } }
                }
            }
            div { class: "flex gap-2",
                if metadata.is_some() {
                    span {
                        class: "inline-flex items-center gap-1.5 rounded-lg border border-emerald-500/20 bg-emerald-500/10 px-3 py-1.5 text-xs font-medium text-emerald-700 dark:text-emerald-400",
                        "data-analytics-freshness": "backend",
                        Icon { name: "database".to_string(), size: Some(14) }
                        if enterprise { "Data available" } else { "Update data" }
                    }
                } else {
                    span {
                        class: "inline-flex items-center gap-1.5 rounded-lg border border-amber-500/20 bg-amber-500/10 px-3 py-1.5 text-xs font-medium text-amber-700 dark:text-amber-300",
                        "data-analytics-freshness": "unavailable",
                        Icon { name: "circle-alert".to_string(), size: Some(14) }
                        "Data unavailable"
                    }
                }
            }
        }
    }
}

#[component]
fn AnalyticsAccessStatus(access: Option<AnalyticsAccessInfo>) -> Element {
    let navigation = try_consume_context::<crate::fullstack::analytics::AnalyticsNavigation>();
    let viewing = access.as_ref().map(|access| {
        if let Some(maximum) = access.max_accessible_rank {
            format!("Ranks {}-{maximum}", access.min_accessible_rank.max(1))
        } else if access.min_accessible_rank <= 1 {
            "All ranks".to_string()
        } else {
            format!("Ranks {}+", access.min_accessible_rank)
        }
    });
    let locked = access
        .as_ref()
        .and_then(|access| match access.locked_ranks_count {
            0 => None,
            1 => Some("Rank 1 locked".to_string()),
            count => Some(format!("Ranks 1-{count} locked")),
        });
    rsx! {
        section { class: "relative mb-4 overflow-hidden rounded-2xl border border-slate-500/30 bg-gradient-to-r from-slate-500/10 via-gray-500/10 to-purple-500/10 backdrop-blur-xl", "data-section": "analytics-plan-status",
            div { class: "relative flex flex-col gap-4 p-4 sm:flex-row sm:items-center sm:justify-between",
                div { class: "flex items-center gap-4",
                    div { class: "flex h-12 w-12 shrink-0 items-center justify-center rounded-xl bg-gradient-to-br from-slate-400 via-gray-500 to-slate-600 shadow-lg",
                        Icon { name: "shield".to_string(), size: Some(24), class_name: Some("text-white".to_string()) }
                    }
                    div {
                        h2 { class: "text-base font-bold text-slate-900 dark:text-white", "Rankings access" }
                        div { class: "mt-1 flex flex-wrap items-center gap-2 text-sm",
                            Icon { name: "sparkles".to_string(), size: Some(14), class_name: Some("text-slate-400".to_string()) }
                            if let Some(viewing) = viewing {
                                span { class: "text-slate-700 dark:text-slate-300", "Viewing: "
                                    span { class: "font-semibold text-slate-900 dark:text-white", "{viewing}" }
                                }
                            } else {
                                span { class: "text-slate-600 dark:text-slate-400", "Access summary unavailable" }
                            }
                            if let Some(locked) = locked {
                                span { class: "text-slate-500", "•" }
                                span { class: "flex items-center gap-1 text-slate-600 dark:text-slate-400",
                                    Icon { name: "lock".to_string(), size: Some(12) }
                                    "{locked}"
                                }
                            }
                        }
                    }
                }
                if access.as_ref().is_some_and(|access| access.locked_ranks_count > 0) {
                    a { class: "inline-flex w-full shrink-0 items-center justify-center gap-2 rounded-xl bg-gradient-to-r from-purple-600 to-pink-500 px-4 py-2.5 text-sm font-semibold text-white shadow-lg shadow-purple-900/20 sm:w-auto", href: "/plans", onclick: move |event| crate::fullstack::analytics::follow_link(event, navigation, "/plans"),
                        Icon { name: "rocket".to_string(), size: Some(16) }
                        "Review plans"
                    }
                }
            }
        }
    }
}

#[component]
fn AnalyticsFilterForm(
    enterprise: bool,
    filters: Option<AnalyticsFilters>,
    filters_state: String,
    query: AnalyticsQueryState,
    authoritative_limit: i32,
    #[props(default = false)] suppress_error: bool,
) -> Element {
    let navigation = try_consume_context::<crate::fullstack::analytics::AnalyticsNavigation>();
    let country_query = query.clone();
    let ready = filters_state == "ready" && filters.is_some();
    let active_count = usize::from(query.country.is_some()) + usize::from(query.sector.is_some());
    let reset_url = if enterprise {
        query.clear_frontend_filters_url(authoritative_limit.max(1) as u32)
    } else {
        query.reset_url(authoritative_limit.max(1) as u32)
    };
    rsx! {
        form {
            class: if enterprise { "fe-filter-form" } else { "mb-6 rounded-xl border border-border/20 bg-card/80 backdrop-blur-sm" },
            action: "/analytics",
            method: "get",
            onsubmit: move |event| crate::fullstack::analytics::submit_filters(event, navigation),
            "data-section": "analytics-filters",
            "data-analytics-filters-state": if ready { "ready" } else { "unavailable" },
            h2 { class: "flex items-center gap-2 px-3 pt-3 text-sm font-medium text-slate-700 dark:text-slate-300 sm:hidden",
                Icon { name: "sliders-horizontal".to_string(), size: Some(16), class_name: Some("text-slate-400".to_string()) }
                "Filters"
                if active_count > 0 {
                    span { class: "rounded-full bg-purple-500/20 px-2 py-0.5 text-xs font-semibold text-purple-300", "{active_count}" }
                }
            }
            input { r#type: "hidden", name: "page", value: "1" }
            input { r#type: "hidden", name: "limit", value: "{authoritative_limit}" }
            if enterprise {
                if let Some(value) = &query.sector {
                    input { r#type: "hidden", name: "sector", value }
                }
            }

            if let Some(value) = &query.min_eps {
                input { r#type: "hidden", name: "min_eps", value: "{value}" }
            }
            if let Some(value) = &query.min_growth {
                input { r#type: "hidden", name: "min_growth", value: "{value}" }
            }
            div { class: "flex flex-col gap-3 p-3 sm:flex-row sm:items-end sm:gap-3 sm:p-4",
                if let Some(value) = &query.sort_by {
                    input { r#type: "hidden", name: "sort_by", value }
                }
                div { class: "min-w-0 flex-1",
                    label { class: "mb-1.5 flex items-center gap-1.5 text-xs font-medium text-slate-600 dark:text-slate-400", r#for: "analytics-country",
                        Icon { name: "globe".to_string(), size: Some(12) }
                        "Country"
                    }
                    select {
                        id: "analytics-country",
                        name: "country",
                        "data-analytics-country-autosubmit": (enterprise && navigation.is_none()).then_some("true"),
                        onchange: move |event: FormEvent| {
                            if let Some(navigation) = navigation {
                                let mut next = country_query.clone();
                                let value = event.value();
                                next.country = (!value.is_empty()).then_some(value);
                                navigation.0.call(next.page_url(1, authoritative_limit.max(1) as u32));
                            }
                        },
                        disabled: !ready,
                        class: "h-9 w-full rounded-lg border border-border/20 bg-muted/30 px-3 text-sm text-foreground",
                        option { value: "", selected: query.country.is_none(), "All Countries" }
                        if enterprise {
                            if let Some(country) = &query.country {
                                if !filters.as_ref().is_some_and(|filters| filters.countries.iter().any(|option| &option.value == country)) {
                                    option { value: country.clone(), selected: true, "{country}" }
                                }
                            }
                        }
                        if let Some(filters) = &filters {
                            for country in &filters.countries {
                                option {
                                    value: "{country.value}",
                                    selected: query.country.as_deref() == Some(country.value.as_str()),
                                    "{country.label}"
                                }
                            }
                        }
                    }
                }
                if !enterprise {
                div { class: "min-w-0 flex-1",
                    label { class: "mb-1.5 flex items-center gap-1.5 text-xs font-medium text-slate-600 dark:text-slate-400", r#for: "analytics-sector",
                        Icon { name: "sparkles".to_string(), size: Some(12) }
                        "Sector"
                    }
                    select {
                        id: "analytics-sector",
                        name: "sector",
                        disabled: !ready,
                        class: "h-9 w-full rounded-lg border border-border/20 bg-muted/30 px-3 text-sm text-foreground",
                        option { value: "", selected: query.sector.is_none(), "All Sectors" }
                        if enterprise {
                            if let Some(sector) = &query.sector {
                                if !filters.as_ref().is_some_and(|filters| filters.sectors.contains(sector)) {
                                    option { value: sector.clone(), selected: true, "{sector}" }
                                }
                            }
                        }
                        if let Some(filters) = &filters {
                            for sector in &filters.sectors {
                                option {
                                    value: "{sector}",
                                    selected: query.sector.as_deref() == Some(sector.as_str()),
                                    "{sector}"
                                }
                            }
                        }
                    }
                }
                }
                div { class: "flex items-end gap-2",
                    if !enterprise {
                    button {
                        class: "inline-flex h-9 items-center gap-1.5 rounded-lg bg-purple-600 px-4 text-sm font-medium text-white transition-colors hover:bg-purple-500 disabled:pointer-events-none disabled:opacity-40",
                        r#type: "submit",
                        disabled: !ready,
                        Icon { name: "search".to_string(), size: Some(14) }
                        "Apply"
                    }
                    } else {
                        noscript {
                            button { r#type: "submit", class: "fe-button fe-primary", disabled: !ready, "Update country" }
                        }
                    }
                    if active_count > 0 || (enterprise && query.is_custom_view()) {
                        a {
                            class: "inline-flex h-9 items-center gap-1.5 rounded-lg border border-gray-200 bg-gray-100 px-3 text-sm font-medium text-slate-600 transition-colors hover:bg-gray-200 dark:border-white/[0.08] dark:bg-slate-800/60 dark:text-slate-300 dark:hover:bg-slate-700/60",
                            href: "{reset_url}",
                            onclick: move |event| crate::fullstack::analytics::follow_link(event, navigation, &reset_url),
                            aria_label: "Reset filters",
                            Icon { name: "rotate-ccw".to_string(), size: Some(14) }
                            span { class: "hidden sm:inline", if enterprise { "Clear filters" } else { "Reset" } }
                        }
                    }
                }
            }
            if enterprise {
                div { class: "fe-ranking-order",
                    if query.is_custom_view() {
                        span { class: "fe-badge", "Custom view" }
                        span { "Your link’s custom conditions are applied." }
                        a { href: query.default_ranking_url(authoritative_limit.max(1) as u32), onclick: { let target = query.default_ranking_url(authoritative_limit.max(1) as u32); move |event| crate::fullstack::analytics::follow_link(event, navigation, &target) }, "Use EPSX ranking" }
                    } else { span { "Order: EPSX ranking" } }
                    if let Some(country) = &query.country { span { class: "fe-filter-chip", "Country: {country}" } }
                    if let Some(sector) = &query.sector { span { class: "fe-filter-chip", "Sector: {sector}" } }
                }
            }
            if !ready && !suppress_error {
                p { class: "border-t border-gray-200 px-4 py-2 text-xs text-amber-700 dark:border-white/[0.04] dark:text-amber-300", role: "status",
                    "Filter options are temporarily unavailable."
                }
            }
        }
    }
}

pub(super) fn row_card_values(row: &AnalyticsRow) -> (f64, f64, Option<i32>, Option<f64>) {
    let latest = row.quarterly_performance.first();
    let growth = latest
        .map(|quarter| quarter.eps_growth)
        .or(row.growth_factor)
        .unwrap_or(0.0);
    let price = latest
        .map(|quarter| quarter.price)
        .or(row.price_current)
        .unwrap_or(row.value);
    let days = row
        .next_quarter_estimate
        .as_ref()
        .map(|estimate| estimate.days_until_announcement)
        .or(row.days_until_next_earnings)
        .filter(|days| *days >= 0);
    (growth, price, days, row.progress_percentage)
}

#[component]
fn AnalyticsCardGrid(
    rows: Vec<AnalyticsRow>,
    signed_in: bool,
    watchlist: Option<WatchlistData>,
    watchlist_state: String,
    #[props(default = crate::components::stock_data_card::ANALYTICS_SIGN_IN_PATH.to_string())]
    sign_in_path: String,
) -> Element {
    let watched = watchlist
        .map(|watchlist| watchlist.symbols.into_iter().collect::<HashSet<_>>())
        .unwrap_or_default();
    rsx! {
        section {
            class: "analytics-card-grid grid gap-6",
            style: "grid-template-columns: repeat(auto-fit, minmax(min(100%, 300px), 1fr)); gap: 24px;",
            "data-section": "analytics-card-grid",
            "aria-label": "EPS growth rankings",
            for row in rows {
                {
                    let (growth, price, days, progress) = row_card_values(&row);
                    let watchlist = if !signed_in {
                        StockCardWatchlist::SignedOut
                    } else if watchlist_state == "ready" {
                        StockCardWatchlist::Ready {
                            is_watchlisted: watched.contains(&row.symbol.to_ascii_uppercase()),
                        }
                    } else {
                        StockCardWatchlist::Unavailable
                    };
                    rsx! {
                        StockDataCard {
                            symbol: row.symbol,
                            rank: row.rank,
                            eps_growth: growth,
                            price,
                            company_name: row.company_name,
                            days_until_next_action: days,
                            progress_percentage: progress,
                            watchlist: Some(watchlist),
                            sign_in_path: sign_in_path.clone(),
                        }
                    }
                }
            }
        }
    }
}

#[derive(Clone, Copy)]
enum PageToken {
    Page(u32),
    Ellipsis,
}

fn visible_pages(page: u32, total_pages: u32) -> Vec<PageToken> {
    if total_pages <= 7 {
        return (1..=total_pages).map(PageToken::Page).collect();
    }
    let mut pages = vec![PageToken::Page(1)];
    let start = page.saturating_sub(2).max(2);
    let end = page.saturating_add(2).min(total_pages - 1);
    if start > 2 {
        pages.push(PageToken::Ellipsis);
    }
    pages.extend((start..=end).map(PageToken::Page));
    if end < total_pages - 1 {
        pages.push(PageToken::Ellipsis);
    }
    pages.push(PageToken::Page(total_pages));
    pages
}

fn preserved_hidden_filters(query: &AnalyticsQueryState) -> Element {
    rsx! {
        if let Some(value) = &query.country {
            input { r#type: "hidden", name: "country", value: "{value}" }
        }
        if let Some(value) = &query.sector {
            input { r#type: "hidden", name: "sector", value: "{value}" }
        }
        if let Some(value) = &query.sort_by {
            input { r#type: "hidden", name: "sort_by", value: "{value}" }
        }
        if let Some(value) = &query.min_eps {
            input { r#type: "hidden", name: "min_eps", value: "{value}" }
        }
        if let Some(value) = &query.min_growth {
            input { r#type: "hidden", name: "min_growth", value: "{value}" }
        }
    }
}

#[component]
fn AnalyticsPaginationNav(pagination: AnalyticsPagination, query: AnalyticsQueryState) -> Element {
    let page = pagination.page.max(1) as u32;
    let limit = pagination.limit.max(1) as u32;
    let total_pages = pagination.total_pages.max(1) as u32;
    let start = i64::from(page.saturating_sub(1)) * i64::from(limit) + 1;
    let end = (i64::from(page) * i64::from(limit)).min(pagination.total);
    let previous_url = query.page_url(page.saturating_sub(1).max(1), limit);
    let next_url = query.page_url(page.saturating_add(1).min(total_pages), limit);
    let standard_limits = [10_u32, 25, 50, 100];
    let requested_query =
        try_consume_context::<crate::fullstack::analytics::AnalyticsRequestedQuery>();
    let selected_limit = requested_query
        .and_then(|requested| AnalyticsQueryState::from_normalized_query(&(requested.0)()).ok())
        .and_then(|requested| requested.limit)
        .unwrap_or(limit);
    let navigation = try_consume_context::<crate::fullstack::analytics::AnalyticsNavigation>();
    let limit_query = query.clone();
    let loading = try_consume_context::<crate::fullstack::analytics::AnalyticsLoading>();
    let controls_disabled = loading.is_some_and(|state| !(state.ready)() || (state.pending)());
    rsx! {
        nav {
            class: "mt-8 rounded-xl border border-gray-200 bg-white p-4 backdrop-blur-sm dark:border-white/[0.06] dark:bg-slate-900/80",
            "aria-label": "Analytics pagination",
            "data-section": "analytics-pagination",
            div { class: "mb-4 flex flex-col items-center justify-between gap-3 sm:flex-row",
                p { class: "text-sm text-slate-600 dark:text-slate-400",
                    "Showing "
                    span { class: "font-medium text-slate-700 dark:text-slate-200", "{start}-{end}" }
                    " of {pagination.total}"
                }
                form {
                    class: "flex items-center gap-2",
                    action: "/analytics",
                    method: "get",
                    onsubmit: move |event| crate::fullstack::analytics::submit_filters(event, navigation),
                    input { r#type: "hidden", name: "page", value: "1" }
                    {preserved_hidden_filters(&query)}
                    label { class: "text-xs text-slate-600 dark:text-slate-400", r#for: "analytics-limit", "Per page" }
                    select {
                        id: "analytics-limit",
                        disabled: controls_disabled,
                        name: "limit",
                        class: "h-9 rounded-lg border border-gray-200 bg-gray-100 px-3 text-sm text-slate-700 dark:border-white/[0.08] dark:bg-slate-800/60 dark:text-slate-200",
                        "data-analytics-limit": navigation.is_none().then_some("true"),
                        onchange: move |event: FormEvent| {
                            if let (Some(navigation), Ok(limit)) = (navigation, event.value().parse::<u32>()) {
                                navigation.0.call(limit_query.page_url(1, limit));
                            }
                        },
                        if !standard_limits.contains(&selected_limit) {
                            option { value: "{selected_limit}", selected: true, "{selected_limit}" }
                        }
                        for candidate in standard_limits {
                            option { value: "{candidate}", selected: candidate == selected_limit, "{candidate}" }
                        }
                    }
                    button { class: "sr-only", r#type: "submit", "Apply page size" }
                }
            }
            div { class: "flex items-center justify-center gap-1",
                if pagination.has_prev {
                    a {
                        class: "flex h-9 items-center gap-1 rounded-lg border border-gray-200 bg-gray-100 px-3 text-sm font-medium text-slate-700 transition-colors hover:bg-gray-200 hover:text-slate-900 dark:border-white/[0.08] dark:bg-slate-800/60 dark:text-slate-300 dark:hover:bg-slate-700/60 dark:hover:text-white",
                        href: "{previous_url}",
                        onclick: move |event| crate::fullstack::analytics::follow_link(event, navigation, &previous_url),
                        rel: "prev",
                        aria_label: "Previous page",
                        Icon { name: "chevron-left".to_string(), size: Some(16) }
                        span { class: "hidden sm:block", "Prev" }
                    }
                } else {
                    span { class: "flex h-9 items-center gap-1 rounded-lg border border-gray-200 bg-gray-100 px-3 text-sm font-medium text-slate-600 opacity-50 dark:border-white/[0.08] dark:bg-slate-800/60 dark:text-slate-500 dark:opacity-30", aria_disabled: "true",
                        Icon { name: "chevron-left".to_string(), size: Some(16) }
                        span { class: "hidden sm:block", "Prev" }
                    }
                }
                div { class: "mx-1 flex items-center gap-1",
                    for (index, token) in visible_pages(page, total_pages).into_iter().enumerate() {
                        match token {
                            PageToken::Ellipsis => rsx! {
                                span { class: "flex h-9 w-9 items-center justify-center text-sm text-slate-600 dark:text-slate-500", "data-page-gap": index, "…" }
                            },
                            PageToken::Page(candidate) if candidate == page => rsx! {
                                span {
                                    class: "flex h-9 w-9 items-center justify-center rounded-lg border border-purple-500 bg-purple-600 text-sm font-medium text-white",
                                    "aria-current": "page",
                                    "{candidate}"
                                }
                            },
                            PageToken::Page(candidate) => {
                                let href = query.page_url(candidate, limit);
                                rsx! {
                                    a {
                                        class: "flex h-9 w-9 items-center justify-center rounded-lg border border-gray-200 bg-gray-100 text-sm font-medium text-slate-700 transition-colors hover:bg-gray-200 hover:text-slate-900 dark:border-white/[0.08] dark:bg-slate-800/60 dark:text-slate-300 dark:hover:bg-slate-700/60 dark:hover:text-white",
                                        href: "{href}",
                        onclick: move |event| crate::fullstack::analytics::follow_link(event, navigation, &href),
                                        "aria-label": "Page {candidate}",
                                        "{candidate}"
                                    }
                                }
                            }
                        }
                    }
                }
                if pagination.has_next {
                    a {
                        class: "flex h-9 items-center gap-1 rounded-lg border border-gray-200 bg-gray-100 px-3 text-sm font-medium text-slate-700 transition-colors hover:bg-gray-200 hover:text-slate-900 dark:border-white/[0.08] dark:bg-slate-800/60 dark:text-slate-300 dark:hover:bg-slate-700/60 dark:hover:text-white",
                        href: "{next_url}",
                        onclick: move |event| crate::fullstack::analytics::follow_link(event, navigation, &next_url),
                        rel: "next",
                        aria_label: "Next page",
                        span { class: "hidden sm:block", "Next" }
                        Icon { name: "chevron-right".to_string(), size: Some(16) }
                    }
                } else {
                    span { class: "flex h-9 items-center gap-1 rounded-lg border border-gray-200 bg-gray-100 px-3 text-sm font-medium text-slate-600 opacity-50 dark:border-white/[0.08] dark:bg-slate-800/60 dark:text-slate-500 dark:opacity-30", aria_disabled: "true",
                        span { class: "hidden sm:block", "Next" }
                        Icon { name: "chevron-right".to_string(), size: Some(16) }
                    }
                }
            }
        }
    }
}

#[component]
fn AnalyticsFailurePage(
    enterprise: bool,
    #[props(default)] signed_in: bool,
    failure_state: String,
    filters: Option<AnalyticsFilters>,
    filters_state: String,
    query: AnalyticsQueryState,
) -> Element {
    if enterprise {
        let retry = query.page_url(query.page.max(1), query.limit.unwrap_or(10));
        let encoded: String = url::form_urlencoded::byte_serialize(retry.as_bytes()).collect();
        let restricted_href = if signed_in {
            "/plans".to_string()
        } else {
            format!("/auth?return_url={encoded}")
        };
        return rsx! {
            div { class: "fe-page fe-explore",
                FrontendExploreHeading {}
                section { class: "fe-data-surface", "data-section": "analytics-failure",
                    "data-analytics-state": failure_state.clone(),
                    details { class: "fe-filter-disclosure", open: true,
                        summary { "Filters" }
                        AnalyticsFilterForm { enterprise: true, filters, filters_state, query,
                            authoritative_limit: 10, suppress_error: true }
                    }
                    if failure_state == "restricted" {
                        DataState { title: "Access to this data is restricted",
                            message: if signed_in { "Review the available plans to see which company rankings are included." } else { "Sign in to check the access available to your account." },
                            href: restricted_href, action: if signed_in { "Review plans" } else { "Sign in" } }
                    } else {
                        DataState { title: "Company data is unavailable",
                            message: "We couldn’t load this data. Please try again.", href: retry }
                    }
                }
            }
        };
    }
    let malformed = failure_state == "malformed";
    rsx! {
        div { class: "analytics-page relative min-h-screen",
            if !enterprise { AnalyticsBackground {} }
            div { class: "page-content relative z-10 mx-auto max-w-7xl px-4 py-6 sm:py-8",
                AnalyticsHeader { enterprise, metadata: None }
                section {
                    "data-section": "analytics-failure",
                    "data-analytics-state": "{failure_state}",
                    role: "alert",
                    AnalyticsAccessStatus { access: None }
                    AnalyticsFilterForm {
                        enterprise,
                        filters,
                        filters_state,
                        query,
                        authoritative_limit: 10,
                    }
                    if enterprise {
                        if failure_state == "restricted" {
                            DataState { title: "Access to this data is restricted", message: "Your current access does not include this data. Review the available plans for access details.", href: "/plans", action: "Review plans" }
                        } else {
                            DataState { title: "Company data is unavailable", message: "We couldn’t load this data. Please try again.", href: "/analytics" }
                        }
                    } else {
                    div { class: "mb-6 rounded-xl border border-amber-500/25 bg-amber-500/10 px-5 py-4",
                        h2 { class: "font-semibold text-foreground",
                            if malformed { "Ranking data could not be validated" } else { "Rankings are temporarily unavailable" }
                        }
                        p { class: "mt-1 max-w-3xl text-sm text-muted-foreground",
                            if malformed {
                                "We couldn’t load this data. Please try again."
                            } else {
                                "The analytics dependency could not be reached. No sample market data is being shown."
                            }
                        }
                    }
                    AnalyticsUnavailableGrid {}
                    }
                }
            }
        }
    }
}

#[component]
fn AnalyticsUnavailableGrid() -> Element {
    rsx! {
        section {
            class: "grid grid-cols-1 gap-4 sm:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4",
            "aria-label": "Unavailable EPS growth rankings",
            aria_hidden: "true",
            for tier in ["CHAMPION", "ELITE", "LEGEND", "MASTER"] {
                article { class: "relative overflow-hidden rounded-2xl border border-border/40 bg-card/80 p-5 text-center shadow-xl",
                    span { class: "absolute left-1/2 top-0 -translate-x-1/2 rounded-b-lg bg-gradient-to-r from-slate-700 to-slate-500 px-4 py-1 text-[10px] font-bold tracking-widest text-white", "{tier}" }
                    div { class: "pt-4",
                        p { class: "text-[10px] font-bold uppercase tracking-[0.2em] text-muted-foreground", "Stock Symbol" }
                        p { class: "mt-2 text-2xl font-black text-muted-foreground", "Unavailable" }
                        p { class: "mt-1 text-xs text-muted-foreground", "Company data unavailable" }
                    }
                    div { class: "mt-6 relative overflow-hidden rounded-2xl bg-gradient-to-br from-blue-50 via-indigo-50/50 to-white p-4 ring-1 ring-blue-200/50 dark:from-blue-500/[0.08] dark:via-indigo-500/[0.05] dark:to-white/[0.02] dark:ring-white/10",
                        div { class: "pointer-events-none absolute -right-6 -top-6 h-20 w-20 rounded-full bg-blue-500/10 blur-2xl", "aria-hidden": "true" }
                        div { class: "flex items-center justify-between gap-3",
                            div { class: "flex items-center gap-2",
                                div { class: "flex h-10 w-10 items-center justify-center rounded-xl bg-gradient-to-br from-blue-600 to-cyan-500 opacity-40",
                                    span { class: "text-white", "📅" }
                                }
                                p { class: "text-xs font-bold uppercase tracking-widest text-blue-600 dark:text-blue-400", "Next Action" }
                            }
                            p { class: "text-2xl font-black tracking-tight text-muted-foreground", "N/A" }
                        }
                        div { class: "mt-4 h-2 w-full overflow-hidden rounded-full bg-slate-200 dark:bg-slate-700/50",
                            div { class: "h-full w-[42%] rounded-full bg-gradient-to-r from-blue-600 to-cyan-400 opacity-30" }
                        }
                    }
                    button { class: "btn btn-ghost mt-4 w-full cursor-not-allowed opacity-50", r#type: "button", disabled: true, "View Details →" }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::auth::user::{AuthMethod, User};

    fn page_ctx() -> PageContext {
        PageContext {
            path: "/analytics".to_string(),
            ..Default::default()
        }
    }

    fn signed_in_ctx() -> PageContext {
        PageContext {
            user: Some(User {
                id: "analytics-user".to_string(),
                address: "0xanalytics".to_string(),
                chain_id: "1".to_string(),
                roles: vec!["user".to_string()],
                email: None,
                tier: None,
                permissions: vec![],
                last_login_at: None,
                auth_method: AuthMethod::Wallet,
                display_name: None,
            }),
            ..page_ctx()
        }
    }

    fn ranking(rank: i32, symbol: &str, growth: f64) -> serde_json::Value {
        serde_json::json!({
            "rank": rank,
            "symbol": symbol,
            "company_name": format!("{symbol} Company"),
            "latest_date": "2026-07-27",
            "value": 90.0,
            "active_status": "TRACK",
            "quarterly_performance": [{
                "quarter": "Q2",
                "date": "2026-06-30",
                "price": 1234.5,
                "eps": 2.5,
                "eps_growth": growth,
                "price_growth": 4.0,
                "announcement_date": "Jul 20, 2026",
                "announcement_timestamp": 1784505600,
                "is_estimated": false
            }],
            "next_quarter_estimate": {
                "quarter": "2026-Q3",
                "estimated_eps": 3.0,
                "announcement_date": "Oct 20, 2026",
                "announcement_timestamp": 1792454400,
                "days_until_announcement": 45,
                "estimated_price_target": 1300.0,
                "confidence": "High"
            },
            "next_earnings_date": 1792454400,
            "last_earnings_date": 1784505600,
            "next_earnings_date_formatted": "Oct 20, 2026",
            "days_until_next_earnings": 45,
            "progress_percentage": 50.0,
            "current_eps": 2.5,
            "growth_factor": growth,
            "price_current": 1234.5
        })
    }

    fn response(rows: Vec<serde_json::Value>, total: i64, total_pages: i32) -> String {
        serde_json::json!({
            "success": true,
            "data": rows,
            "pagination": {
                "page": 1,
                "limit": 10,
                "total": total,
                "totalPages": total_pages,
                "hasNext": total_pages > 1,
                "hasPrev": false
            },
            "metadata": {
                "available_countries": ["america"],
                "available_sectors": ["Technology"],
                "request_timestamp": "2026-07-27T00:00:00Z",
                "data_source": "live"
            },
            "access_info": {
                "min_accessible_rank": 100,
                "locked_ranks_count": 99,
                "max_accessible_rank": null
            },
            "message": "live",
            "processing_time_ms": 3
        })
        .to_string()
    }

    fn filters() -> String {
        serde_json::json!({
            "countries": [{"value": "america", "label": "United States"}],
            "sectors": ["Technology", "Healthcare"],
            "exchanges": ["NASDAQ"],
            "stock_types": ["common"]
        })
        .to_string()
    }

    fn ready_ctx(rows: Vec<serde_json::Value>, total: i64, total_pages: i32) -> PageContext {
        let mut ctx = page_ctx();
        ctx.params
            .insert(ANALYTICS_STATE_PARAM.to_string(), "ready".to_string());
        ctx.params.insert(
            ANALYTICS_DATA_PARAM.to_string(),
            response(rows, total, total_pages),
        );
        ctx.params.insert(
            ANALYTICS_FILTERS_STATE_PARAM.to_string(),
            "ready".to_string(),
        );
        ctx.params
            .insert(ANALYTICS_FILTERS_DATA_PARAM.to_string(), filters());
        ctx.params.insert(
            ANALYTICS_QUERY_PARAM.to_string(),
            serde_json::to_string(&AnalyticsQueryState {
                page: 1,
                limit: Some(10),
                country: Some("america".to_string()),
                sector: None,
                sort_by: Some("growth_factor".to_string()),
                min_eps: None,
                min_growth: None,
            })
            .unwrap(),
        );
        ctx
    }

    fn html(ctx: &PageContext) -> String {
        dioxus_ssr::render_element(render(ctx).1)
    }

    #[test]
    fn hydrated_pagination_waits_for_events_and_pending_requests() {
        #[component]
        fn Harness(ready: bool, pending: bool) -> Element {
            let ready = use_signal(|| ready);
            let pending = use_signal(|| pending);
            use_context_provider(|| crate::fullstack::analytics::AnalyticsLoading {
                ready,
                pending,
            });
            render(&ready_ctx(vec![ranking(100, "LIVE", 1.0)], 22, 3)).1
        }
        for (ready, pending, disabled) in [
            (false, false, true),
            (true, true, true),
            (true, false, false),
        ] {
            let rendered = dioxus_ssr::render_element(rsx! { Harness { ready, pending } });
            let select = rendered.split("id=\"analytics-limit\"").nth(1).unwrap();
            let attributes = select.split('>').next().unwrap();
            assert_eq!(attributes.contains("disabled"), disabled);
            assert_eq!(rendered.matches("data-stock-card=\"true\"").count(), 1);
        }
    }

    #[test]
    fn frontend_cards_preserve_backend_access_without_financial_figures() {
        let rendered = html(&ready_ctx(
            vec![ranking(100, "LIVE", 42.25), ranking(101, "LOSS", -7.5)],
            22,
            3,
        ));
        assert!(rendered.contains("data-analytics-state=\"ready\""));
        assert_eq!(rendered.matches("data-stock-card=\"true\"").count(), 2);
        for value in [
            "analytics-card-grid",
            "View LIVE details on TradingView (opens in a new tab)",
            "LIVE Company",
            "Next Action",
            "<details",
            "Ranks 100+",
            "Ranks 1-99 locked",
            "$1,234.50",
        ] {
            assert!(rendered.contains(value), "missing {value}");
        }
        for value in [
            "<dialog",
            "42.25%",
            "-7.50%",
            "CHAMPION",
            "fe-ranking-card",
            "2026-07-27T00:00:00Z",
            "AI-Powered",
        ] {
            assert!(!rendered.contains(value));
        }
    }

    #[test]
    fn public_cards_preserve_backend_rank_order_in_one_responsive_grid() {
        let rows = (100..=109)
            .map(|rank| ranking(rank, &format!("LIVE{rank}"), f64::from(rank - 99)))
            .collect();
        let rendered = html(&ready_ctx(rows, 100, 10));

        assert_eq!(rendered.matches("data-stock-card=\"true\"").count(), 10);
        assert!(!rendered.contains("<table"));
        let mut previous = 0;
        for rank in 100..=109 {
            let position = rendered.find(&format!("data-rank=\"{rank}\"")).unwrap();
            assert!(
                position > previous,
                "rank {rank} must preserve backend order"
            );
            previous = position;
        }
    }

    #[test]
    fn filters_and_pagination_preserve_supported_fields_and_authoritative_limit() {
        let rendered = html(&ready_ctx(vec![ranking(100, "LIVE", 1.0)], 22, 3));
        assert!(rendered.contains(">United States</option>"));
        assert!(!rendered.contains("id=\"analytics-sector\""));
        assert!(rendered.contains("data-analytics-country-autosubmit=\"true\""));
        assert!(rendered.contains("name=\"page\" value=\"1\""));
        assert!(rendered.contains("name=\"limit\" value=\"10\""));
        assert!(rendered.contains("Showing "));
        assert!(rendered.contains("1-10"));
        assert!(rendered.contains(
            "href=\"/analytics?page=2&#38;limit=10&#38;country=america&#38;sort_by=growth_factor\""
        ));
        assert!(rendered.contains("aria-label=\"Page 3\""));
        assert!(rendered.contains("data-analytics-limit=\"true\""));

        let state = AnalyticsQueryState {
            page: 4,
            limit: Some(100),
            country: Some("united states".into()),
            sector: Some("Health Care".into()),
            sort_by: Some("eps_growth".into()),
            min_eps: Some("1.5".into()),
            min_growth: Some("-2".into()),
        };
        assert_eq!(
            state.page_url(2, 25),
            "/analytics?page=2&limit=25&country=united+states&sector=Health+Care&sort_by=eps_growth&min_eps=1.5&min_growth=-2"
        );
    }

    #[test]
    fn empty_malformed_and_unavailable_states_are_distinct() {
        let mut empty = ready_ctx(vec![], 0, 0);
        empty
            .params
            .insert(ANALYTICS_STATE_PARAM.to_string(), "empty".to_string());
        let empty_html = html(&empty);
        assert!(empty_html.contains("data-analytics-state=\"empty\""));
        assert!(empty_html.contains("No companies available for this view"));

        let mut malformed = page_ctx();
        malformed
            .params
            .insert(ANALYTICS_STATE_PARAM.to_string(), "malformed".to_string());
        let malformed_html = html(&malformed);
        assert!(malformed_html.contains("data-analytics-state=\"malformed\""));
        assert!(malformed_html.contains("We couldn’t load this data"));

        let mut restricted = ready_ctx(vec![ranking(1, "HIDDEN", 1.0)], 1, 1);
        restricted
            .params
            .insert(ANALYTICS_STATE_PARAM.to_string(), "restricted".into());
        let restricted_html = html(&restricted);
        assert!(restricted_html.contains("data-analytics-state=\"restricted\""));
        assert!(restricted_html.contains("Access to this data is restricted"));
        assert!(!restricted_html.contains("HIDDEN"));

        let unavailable_html = html(&page_ctx());
        assert!(unavailable_html.contains("data-analytics-state=\"unavailable\""));
        assert!(unavailable_html.contains("Company data is unavailable"));

        assert_eq!(
            unavailable_html
                .matches("Company data is unavailable")
                .count(),
            1
        );
        assert!(!unavailable_html.contains("Data available"));
        assert!(!unavailable_html.contains(">Live<"));
        assert!(!unavailable_html.contains("data-stock-card=\"true\""));
    }

    #[test]
    fn watchlist_controls_fail_closed_and_signed_out_hearts_link_to_auth() {
        let signed_out = html(&ready_ctx(vec![ranking(100, "LIVE", 1.0)], 1, 1));
        assert!(signed_out.contains("data-watchlist-signed-out=\"true\""));
        assert!(signed_out.contains("href=\"/auth?return_url=%2Fanalytics%3Fpage%3D1%26limit%3D10%26country%3Damerica%26sort_by%3Dgrowth_factor\""));

        let mut signed_in = signed_in_ctx();
        signed_in.params = ready_ctx(vec![ranking(100, "LIVE", 1.0)], 1, 1).params;
        signed_in.params.insert(
            ANALYTICS_WATCHLIST_STATE_PARAM.to_string(),
            "ready".to_string(),
        );
        signed_in.params.insert(
            ANALYTICS_WATCHLIST_DATA_PARAM.to_string(),
            serde_json::json!({"symbols": ["live"]}).to_string(),
        );
        let ready = html(&signed_in);
        assert!(ready.contains("data-watchlist-toggle=\"true\""));
        assert!(ready.contains("data-watchlisted=\"true\""));
        assert!(ready.contains("Remove LIVE from watchlist"));

        signed_in.params.insert(
            ANALYTICS_WATCHLIST_STATE_PARAM.to_string(),
            "unavailable".to_string(),
        );
        signed_in.params.remove(ANALYTICS_WATCHLIST_DATA_PARAM);
        let unavailable = html(&signed_in);
        assert!(unavailable.contains("data-watchlist-unavailable=\"true\""));
        assert!(!unavailable.contains("data-watchlist-toggle=\"true\""));
    }

    #[test]
    fn exchange_symbols_with_slashes_do_not_reject_a_rankings_page() {
        let rows = vec![ranking(165, "TMM/A", 25.0)];
        let payload: AnalyticsResponse = serde_json::from_str(&response(rows.clone(), 1, 1)).unwrap();
        assert!(payload.validated().is_ok());
        let rendered = html(&ready_ctx(rows, 1, 1));
        assert!(rendered.contains("data-symbol=\"TMM/A\""));
        assert!(rendered.contains("https://www.tradingview.com/symbols/TMM%2FA"));
        assert!(!rendered.contains("data-analytics-state=\"unavailable\""));
    }

    #[test]
    fn semantic_validation_rejects_partial_and_invalid_payloads() {
        let mut ctx = page_ctx();
        ctx.params
            .insert(ANALYTICS_STATE_PARAM.to_string(), "ready".to_string());
        for payload in [
            r#"{"success":true,"data":[{"rank":100,"symbol":"PARTIAL"}]}"#,
            r#"{"success":true,"data":[],"pagination":{"page":1,"limit":0,"total":0,"totalPages":0,"hasNext":false,"hasPrev":false},"metadata":{"available_countries":[],"available_sectors":[],"request_timestamp":"x","data_source":"x"},"access_info":null,"message":null,"processing_time_ms":0}"#,
        ] {
            ctx.params
                .insert(ANALYTICS_DATA_PARAM.to_string(), payload.to_string());
            let rendered = html(&ctx);
            assert!(rendered.contains("data-analytics-state=\"unavailable\""));
            assert!(!rendered.contains("PARTIAL"));
        }
    }
}

#[cfg(test)]
mod frontend_ranking_query_tests {
    use super::*;
    #[test]
    fn custom_conditions_survive_forms_and_pagination_until_explicit_reset() {
        let query = AnalyticsQueryState::from_normalized_query(
            "country=america&sector=Technology&sort_by=price&min_eps=2&min_growth=4",
        )
        .unwrap();
        assert!(query.is_custom_view());
        let next = query.page_url(2, 10);
        for filter in [
            "sort_by=price",
            "min_eps=2",
            "min_growth=4",
            "country=america",
            "sector=Technology",
        ] {
            assert!(next.contains(filter));
        }
        assert_eq!(
            query.default_ranking_url(10),
            "/analytics?page=1&limit=10&country=america&sector=Technology"
        );
        assert_eq!(
            query.clear_frontend_filters_url(10),
            "/analytics?page=1&limit=10"
        );
        let html = dioxus_ssr::render_element(
            rsx! { AnalyticsFilterForm { enterprise: true, filters: None, filters_state: "unavailable".to_string(), query, authoritative_limit: 10 } },
        );
        assert!(html.contains("Custom view"));
        for name in ["sort_by", "min_eps", "min_growth"] {
            assert!(html.contains(&format!("name=\"{name}\"")));
        }
        assert!(!html.contains("analytics-sort"));
        assert!(!html.contains("EPS growth"));
        assert!(!html.contains("Order: EPSX ranking"));
    }
    #[test]
    fn default_view_lets_backend_choose_order() {
        let query = AnalyticsQueryState::default();
        let html = dioxus_ssr::render_element(
            rsx! { AnalyticsFilterForm { enterprise: true, filters: None, filters_state: "unavailable".to_string(), query, authoritative_limit: 10 } },
        );
        assert!(html.contains("Order: EPSX ranking"));
        assert!(!html.contains("name=\"sort_by\""));
    }
}
