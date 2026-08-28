//! Reusable production-style EPS ranking card.
//!
//! The component is deliberately presentation-only. Rank offsets, accessible
//! ranges, and watchlist authorization are supplied by backend-owned responses.

use dioxus::prelude::*;

use crate::primitives::Icon;

pub const ANALYTICS_SIGN_IN_PATH: &str = "/auth?return_url=%2Fanalytics";

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum StockCardWatchlist {
    SignedOut,
    Ready { is_watchlisted: bool },
    Unavailable,
}

#[derive(Clone)]
struct RankTheme {
    color: &'static str,
    glow: &'static str,
    label: String,
}

fn rank_theme(rank: i32) -> RankTheme {
    match rank {
        1 => RankTheme {
            color: "text-yellow-400",
            glow: "border-orange-400/30 shadow-orange-500/10",
            label: "CHAMPION".to_string(),
        },
        2 => RankTheme {
            color: "text-slate-300",
            glow: "border-blue-400/30 shadow-blue-500/10",
            label: "ELITE".to_string(),
        },
        3 => RankTheme {
            color: "text-amber-500",
            glow: "border-orange-400/30 shadow-orange-500/10",
            label: "LEGEND".to_string(),
        },
        4 => RankTheme {
            color: "text-emerald-400",
            glow: "border-emerald-400/30 shadow-emerald-500/10",
            label: "MASTER".to_string(),
        },
        5 => RankTheme {
            color: "text-teal-400",
            glow: "border-emerald-400/30 shadow-emerald-500/10",
            label: "EXPERT".to_string(),
        },
        _ => RankTheme {
            color: "text-blue-500 dark:text-blue-400",
            glow: "border-gray-200 dark:border-slate-700/80",
            label: format!("RANK #{rank}"),
        },
    }
}

pub fn format_percentage(value: f64) -> String {
    if value >= 0.0 {
        format!("+{value:.2}%")
    } else {
        format!("{value:.2}%")
    }
}

pub fn format_currency(value: f64, currency: &str) -> String {
    let absolute = value.abs();
    let fixed = format!("{absolute:.2}");
    let (integer, fraction) = fixed
        .split_once('.')
        .expect("two-decimal formatting always has a decimal point");
    let mut grouped_reversed = String::with_capacity(integer.len() + integer.len() / 3);
    for (index, character) in integer.chars().rev().enumerate() {
        if index > 0 && index % 3 == 0 {
            grouped_reversed.push(',');
        }
        grouped_reversed.push(character);
    }
    let grouped: String = grouped_reversed.chars().rev().collect();
    let sign = if value < 0.0 { "-" } else { "" };
    if currency.eq_ignore_ascii_case("USD") {
        format!("{sign}${grouped}.{fraction}")
    } else {
        format!("{sign}{currency} {grouped}.{fraction}")
    }
}

fn watchlist_control(symbol: &str, watchlist: &StockCardWatchlist) -> Element {
    let base_class = "stock-watchlist-control absolute right-3 top-3 z-20 inline-flex h-8 w-8 items-center justify-center rounded-full text-xl leading-none transition-colors hover:bg-black/[0.05] focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-pink-400 dark:hover:bg-white/10";
    match watchlist {
        StockCardWatchlist::SignedOut => rsx! {
            a {
                class: "{base_class} text-gray-400 hover:text-pink-400",
                href: ANALYTICS_SIGN_IN_PATH,
                "data-watchlist-signed-out": "true",
                "data-symbol": symbol,
                "aria-label": "Sign in to add {symbol} to watchlist",
                span { "aria-hidden": "true", "♡" }
            }
        },
        StockCardWatchlist::Ready { is_watchlisted } => {
            let label = if *is_watchlisted {
                format!("Remove {symbol} from watchlist")
            } else {
                format!("Add {symbol} to watchlist")
            };
            let color = if *is_watchlisted {
                "text-pink-500"
            } else {
                "text-gray-400 hover:text-pink-400"
            };
            let glyph = if *is_watchlisted { "♥" } else { "♡" };
            let watched = if *is_watchlisted { "true" } else { "false" };
            rsx! {
                button {
                    class: "{base_class} {color}",
                    r#type: "button",
                    "data-watchlist-toggle": "true",
                    "data-symbol": symbol,
                    "data-watchlisted": watched,
                    "aria-label": "{label}",
                    "aria-busy": "false",
                    span { "data-watchlist-glyph": "true", "aria-hidden": "true", "{glyph}" }
                }
                span {
                    class: "stock-watchlist-status sr-only",
                    role: "status",
                    "aria-live": "polite"
                }
            }
        }
        StockCardWatchlist::Unavailable => rsx! {
            button {
                class: "{base_class} cursor-not-allowed text-gray-600",
                r#type: "button",
                disabled: true,
                "data-watchlist-unavailable": "true",
                "data-symbol": symbol,
                "aria-label": "Watchlist unavailable for {symbol}",
                span { "aria-hidden": "true", "♡" }
            }
            span {
                class: "stock-watchlist-status sr-only",
                role: "status",
                "aria-live": "polite",
                "Watchlist is temporarily unavailable."
            }
        },
    }
}

#[component]
pub fn StockDataCard(
    symbol: String,
    rank: i32,
    eps_growth: f64,
    price: f64,
    #[props(default = "USD".to_string())] currency: String,
    #[props(default = None)] days_until_next_action: Option<i32>,
    #[props(default = None)] progress_percentage: Option<f64>,
    #[props(default = None)] company_name: Option<String>,
    #[props(default = None)] watchlist: Option<StockCardWatchlist>,
) -> Element {
    let _ = eps_growth;
    let theme = rank_theme(rank);
    let action_progress = progress_percentage
        .or_else(|| {
            days_until_next_action
                .map(|days| ((90.0 - f64::from(days)) / 90.0 * 100.0).clamp(5.0, 100.0))
        })
        .unwrap_or(0.0)
        .clamp(0.0, 100.0);
    let action_width_style = format!("width: {action_progress:.2}%;");
    let action_label = days_until_next_action
        .map(|days| format!("{days} Days"))
        .unwrap_or_else(|| "N/A".to_string());
    let price_label = format_currency(price, &currency);
    let details_url = format!("https://www.tradingview.com/symbols/{symbol}");
    let header_label = if rank > 5 {
        theme.label.clone()
    } else {
        "Stock Symbol".to_string()
    };
    let card_padding = if rank <= 5 { "pt-8" } else { "pt-5" };
    let badge_gradient = if rank <= 3 {
        "from-blue-600 to-cyan-500"
    } else {
        "from-emerald-600 to-teal-500"
    };

    rsx! {
        article {
            class: "stock-data-card group relative flex h-full w-full flex-col overflow-hidden rounded-2xl border bg-white/90 shadow-lg backdrop-blur-xl transition-transform duration-300 hover:-translate-y-1 dark:bg-slate-900/90 {theme.glow}",
            "data-stock-card": "true",
            "data-rank": rank,
            "data-symbol": "{symbol}",
            if let Some(watchlist) = watchlist.as_ref() {
                {watchlist_control(&symbol, watchlist)}
            }
            if rank <= 5 {
                div { class: "absolute left-1/2 top-3 z-20 -translate-x-1/2 -translate-y-1/2",
                    div { class: "rounded-full bg-gradient-to-r {badge_gradient} px-4 py-1.5 text-[10px] font-bold uppercase tracking-wider text-white shadow-lg",
                        "{theme.label}"
                    }
                }
            }

            div { class: "relative z-10 flex h-full flex-col p-5 {card_padding}",
                div { class: "mb-4 text-center",
                    p { class: "mb-1 text-xs font-bold uppercase tracking-widest text-gray-500 dark:text-gray-400",
                        "{header_label}"
                    }
                    h3 { class: "break-words text-4xl font-black tracking-tighter {theme.color}", "{symbol}" }
                    if let Some(name) = company_name.as_deref().filter(|name| !name.is_empty()) {
                        p { class: "mx-auto mt-0.5 max-w-[90%] truncate text-xs font-medium text-gray-500 dark:text-gray-400",
                            "{name}"
                        }
                    }
                    p { class: "mt-0.5 text-sm font-semibold text-gray-600 dark:text-gray-300", "{price_label}" }
                }

                div { class: "mb-4 flex flex-grow flex-col justify-center",
                    div { class: "relative overflow-hidden rounded-2xl bg-gradient-to-br from-blue-50 via-indigo-50/50 to-white p-4 ring-1 ring-blue-200/50 transition-colors dark:from-blue-500/[0.08] dark:via-indigo-500/[0.05] dark:to-white/[0.02] dark:ring-white/10",
                        div { class: "pointer-events-none absolute -right-6 -top-6 h-20 w-20 rounded-full bg-blue-500/10 blur-2xl", "aria-hidden": "true" }
                        div { class: "relative flex items-center justify-between gap-3",
                            div { class: "flex items-center gap-2",
                                div { class: "flex h-10 w-10 items-center justify-center rounded-xl bg-gradient-to-br from-blue-600 to-cyan-500 shadow-md",
                                    Icon { name: "calendar".to_string(), size: Some(18), class_name: Some("text-white".to_string()) }
                                }
                                span { class: "text-xs font-bold uppercase tracking-widest text-blue-600 dark:text-blue-400", "Next Action" }
                            }
                            span { class: "whitespace-nowrap text-2xl font-black tracking-tight text-slate-900 dark:text-white tabular-nums", "{action_label}" }
                        }
                        div { class: "mt-4 h-2 w-full overflow-hidden rounded-full bg-slate-200 dark:bg-slate-700/50",
                            div {
                                class: "relative h-full rounded-full bg-gradient-to-r from-blue-600 to-cyan-400",
                                style: "{action_width_style}",
                                div { class: "absolute inset-0 bg-white/20" }
                            }
                        }
                    }
                }

                a {
                    class: "mt-auto flex w-full items-center justify-center gap-2 overflow-hidden rounded-xl bg-gradient-to-r from-blue-600 to-blue-700 py-3 text-sm font-bold text-white transition-all duration-300 hover:shadow-lg hover:shadow-blue-500/25",
                    href: "{details_url}",
                    target: "_blank",
                    rel: "noopener noreferrer",
                    "aria-label": "View {symbol} details on TradingView (opens in a new tab)",
                    "View Details"
                    Icon { name: "arrow-right".to_string(), size: Some(16) }
                }
            }

            div { class: "pointer-events-none absolute right-0 top-0 h-32 w-32 translate-x-10 -translate-y-10 rounded-full bg-blue-500/10 blur-3xl", "aria-hidden": "true" }
            div { class: "pointer-events-none absolute bottom-0 left-0 h-24 w-24 -translate-x-10 translate-y-10 rounded-full bg-purple-500/10 blur-3xl", "aria-hidden": "true" }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn currency_and_percentage_formatting_match_the_production_card() {
        assert_eq!(format_currency(1234.5, "USD"), "$1,234.50");
        assert_eq!(format_currency(-8.0, "USD"), "-$8.00");
        assert_eq!(format_percentage(12.345), "+12.35%");
        assert_eq!(format_percentage(-7.2), "-7.20%");
    }

    #[test]
    fn missing_next_action_is_explicit_and_home_can_hide_watchlist() {
        let html = dioxus_ssr::render_element(rsx! {
            StockDataCard {
                symbol: "LIVE".to_string(),
                rank: 100,
                eps_growth: -4.25,
                price: 1002.5,
                company_name: Some("Live Company".to_string()),
            }
        });
        assert!(html.contains("RANK #100"));
        assert!(html.contains("$1,002.50"));
        assert!(html.contains("N/A"));
        assert!(html.contains("Next Action"));
        assert!(!html.contains("Growth"));
        assert!(!html.contains("-4.25%"));
        assert!(!html.contains("+12.35%"));
        assert!(!html.contains("data-watchlist-toggle"));
        assert!(!html.contains("data-watchlist-signed-out"));
    }
}
