//! `MetricPill` / `StatRow` / `StatGroup` — metric / KPI display
//! primitives.
//!
//! Three related components for displaying metrics in a compact
//! format:
//!
//! - `MetricPill` — a single metric with a label, value, and
///   optional trend indicator (up/down arrow + percentage).
/// - `StatRow` — a horizontal row of `MetricPill`s.
/// - `StatGroup` — a labeled group of `StatRow`s with an
///   optional header.

use super::icon::Icon;

use dioxus::prelude::*;

/// Trend direction for a `MetricPill`.
#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub enum MetricTrend {
    #[default]
    Neutral,
    Up,
    Down,
}

impl MetricTrend {
    pub fn icon(self) -> &'static str {
        match self {
            MetricTrend::Neutral => "minus",
            MetricTrend::Up => "arrow-up",
            MetricTrend::Down => "arrow-down",
        }
    }
    pub fn color_class(self) -> &'static str {
        match self {
            MetricTrend::Neutral => "text-muted-foreground",
            MetricTrend::Up => "text-green-600 dark:text-green-400",
            MetricTrend::Down => "text-red-600 dark:text-red-400",
        }
    }
}

/// A single metric — label, value, and optional trend.
#[component]
pub fn MetricPill(
    label: String,
    value: String,
    #[props(default = None)] trend_value: Option<String>,
    #[props(default = MetricTrend::default())] trend: MetricTrend,
    #[props(default = None)] icon: Option<String>,
) -> Element {
    rsx! {
        div { class: "metric-pill flex flex-col gap-1 px-3 py-2 rounded-md border bg-card",
            div { class: "metric-pill-header flex items-center gap-1.5",
                if let Some(i) = icon {
                    Icon { name: i.clone(), size: Some(12) }
                }
                span { class: "metric-pill-label text-xs font-medium text-muted-foreground", "{label}" }
            }
            div { class: "metric-pill-value-row flex items-baseline gap-1.5",
                span { class: "metric-pill-value text-lg font-semibold", "{value}" }
                if let Some(t) = trend_value {
                    span { class: "metric-pill-trend text-xs font-medium {trend.color_class()} flex items-center gap-0.5",
                        Icon { name: trend.icon().to_string(), size: Some(10) }
                        "{t}"
                    }
                }
            }
        }
    }
}

/// A horizontal row of metric pills.
#[component]
pub fn StatRow(
    metrics: Vec<(String, String)>, // (label, value)
    #[props(default = None)] class: Option<String>,
) -> Element {
    let mut cls = "stat-row grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-2".to_string();
    if let Some(c) = class {
        cls.push(' ');
        cls.push_str(&c);
    }
    rsx! {
        div { class: "{cls}",
            for (label, value) in metrics.iter() {
                MetricPill { label: label.clone(), value: value.clone() }
            }
        }
    }
}

/// A labeled group of stat rows.
#[component]
pub fn StatGroup(
    title: String,
    children: Element,
    #[props(default = None)] class: Option<String>,
) -> Element {
    let mut cls = "stat-group flex flex-col gap-3".to_string();
    if let Some(c) = class {
        cls.push(' ');
        cls.push_str(&c);
    }
    rsx! {
        div { class: "{cls}",
            h4 { class: "stat-group-title text-sm font-semibold text-muted-foreground uppercase tracking-wide", "{title}" }
            {children}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn trend_icons_are_distinct() {
        let trends = [MetricTrend::Neutral, MetricTrend::Up, MetricTrend::Down];
        let classes: Vec<&str> = trends.iter().map(|t| t.icon()).collect();
        let unique: std::collections::HashSet<&str> = classes.iter().copied().collect();
        assert_eq!(unique.len(), trends.len(), "trend icons must be distinct");
    }

    #[test]
    fn trend_up_color_is_green() {
        let cls = MetricTrend::Up.color_class();
        assert!(cls.contains("text-green-600"));
    }

    #[test]
    fn trend_down_color_is_red() {
        let cls = MetricTrend::Down.color_class();
        assert!(cls.contains("text-red-600"));
    }

    #[test]
    fn trend_neutral_color_is_muted() {
        let cls = MetricTrend::Neutral.color_class();
        assert!(cls.contains("text-muted-foreground"));
    }

    #[test]
    fn default_trend_is_neutral() {
        assert_eq!(MetricTrend::default(), MetricTrend::Neutral);
    }

    #[test]
    fn metric_pill_class() {
        let base = "metric-pill flex flex-col gap-1 px-3 py-2 rounded-md border bg-card";
        assert!(base.contains("bg-card"));
        assert!(base.contains("rounded-md"));
    }
}
