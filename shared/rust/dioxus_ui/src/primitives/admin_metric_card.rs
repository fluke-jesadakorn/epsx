//! `<AdminMetricCard>` — admin-specific metric card primitive.
//!
//! Mirrors the design pattern from
//! `apps-old/admin-frontend/components/admin/developer-portal/portal-overview.tsx`
//! (the "StatCard" sub-component) and the
//! `components/wallet/wallet-stats-bar.tsx` stat cards used on the
//! wallets surface. The general-purpose [`StatCard`](super::stat_card)
//! already covers user-surface metrics, but admin pages have a few
//! extra conventions:
//!
//! 1. A `MetricTrend` indicator — `Up(f32) | Down(f32) | Flat` — that
//!    the `StatCard` API does not expose (it only has a string
//!    `change` label). Wave 6B admin pages need a strict-typed trend
//!    so the page-level component decides whether "5%" is good or bad
//!    (API errors/hour: DOWN is good; active users/hour: UP is good).
//! 2. An optional `sparkline_data: Vec<f32>` rendered as a small
//!    inline SVG sparkline at the bottom of the card. The general
//!    `StatCard` takes an opaque `Element` for that, which works but
//!    is awkward for a "wire 7 floats to a sparkline" use case.
//! 3. An explicit `icon: String` (lucide name) so admin pages
//!    can express "Active users in last 24h" → `users` icon, "API
//!    errors/hour" → `alert-triangle`, "Pending approvals" →
//!    `check-circle`, etc.
//!
//! Reused by:
//! * `pages/admin_pages/wallet_wallets.rs` — WalletStatsBar renders
//!   "Total wallets", "Active", "Disabled", "Subscribed" + a
//!   sparkline per card.
//! * `pages/admin_pages/developer_portal.rs` — DeveloperPortalStats
//!   renders the 4-portal-overview stat cards.
//!
//! The component is fully server-renderable — no signals, no
//! resources, no client-only state.

use dioxus::prelude::*;

use super::icon::Icon;

/// Trend indicator for an admin metric. Strict-typed so the page
/// code decides whether a positive or negative direction is "good".
///
/// * `Up(percent)` — value is rising. Page decides if good/bad.
/// * `Down(percent)` — value is falling. Page decides if good/bad.
/// * `Flat` — no change. Rendered as a muted em-dash.
#[derive(Clone, Debug, PartialEq)]
pub enum MetricTrend {
    Up(f32),
    Down(f32),
    Flat,
}

impl MetricTrend {
    /// Resolve a trend + a "is rising good?" flag into a `BadgeKind`
    /// the admin pages reuse for trend pills. Page code passes
    /// `is_up_good = true` for "active users" and
    /// `is_up_good = false` for "API errors".
    pub fn badge_kind(&self, is_up_good: bool) -> crate::BadgeKind {
        match self {
            MetricTrend::Up(_) => {
                if is_up_good { crate::BadgeKind::Success } else { crate::BadgeKind::Danger }
            }
            MetricTrend::Down(_) => {
                if is_up_good { crate::BadgeKind::Danger } else { crate::BadgeKind::Success }
            }
            MetricTrend::Flat => crate::BadgeKind::Outline,
        }
    }

    /// Convenience: returns the percent change value (the `f32`
    /// inside `Up`/`Down`) or 0.0 for `Flat`. Useful for pages that
    /// want to display the raw value rather than a `+/-` pill.
    pub fn percent(&self) -> f32 {
        match self {
            MetricTrend::Up(p) | MetricTrend::Down(p) => *p,
            MetricTrend::Flat => 0.0,
        }
    }
}

/// `<AdminMetricCard>` — server-renderable admin metric card.
///
/// Layout (top → bottom):
/// 1. Row: optional icon + label (small uppercase) on the left,
///    trend pill (Up/Down/Flat) on the right.
/// 2. Value: large bold number/string.
/// 3. Optional sparkline (7-floats max, rendered as an inline SVG
///    polyline).
#[component]
pub fn AdminMetricCard(
    /// Small uppercase label, e.g. "Active users (24h)".
    label: String,
    /// Big value, e.g. "1,234" or "12.4%". Pre-formatted.
    value: String,
    /// Optional trend indicator. `None` → no trend pill rendered.
    #[props(default = None)]
    trend: Option<MetricTrend>,
    /// Optional sparkline data points. Renders a polyline
    /// sparkline. `Vec<f32>` is normalized to the card width.
    #[props(default = None)]
    sparkline_data: Option<Vec<f32>>,
    /// Optional lucide icon name. Rendered top-left.
    #[props(default = None)]
    icon: Option<String>,
) -> Element {
    // Trend pill — derive a class + display text from the trend.
    let (trend_pill, trend_text) = match &trend {
        Some(MetricTrend::Up(p)) => (
            "admin-metric-trend admin-metric-trend-up",
            format!("+{:.1}%", p),
        ),
        Some(MetricTrend::Down(p)) => (
            "admin-metric-trend admin-metric-trend-down",
            format!("-{:.1}%", p),
        ),
        Some(MetricTrend::Flat) => (
            "admin-metric-trend admin-metric-trend-flat",
            "—".to_string(),
        ),
        None => ("", String::new()),
    };

    // Build the sparkline polyline points (if data present).
    // We use a 200x40 viewBox and scale floats into it.
    let sparkline_points: String = sparkline_data
        .as_ref()
        .map(|pts| {
            if pts.is_empty() {
                return String::new();
            }
            let min_v = pts.iter().cloned().fold(f32::INFINITY, f32::min);
            let max_v = pts.iter().cloned().fold(f32::NEG_INFINITY, f32::max);
            let range = (max_v - min_v).max(0.0001);
            let n = pts.len();
            pts.iter()
                .enumerate()
                .map(|(i, v)| {
                    let x = if n > 1 { (i as f32) * (200.0 / (n as f32 - 1.0)) } else { 0.0 };
                    let y = 40.0 - ((v - min_v) / range) * 40.0;
                    format!("{:.1},{:.1}", x, y)
                })
                .collect::<Vec<_>>()
                .join(" ")
        })
        .unwrap_or_default();

    rsx! {
        div { class: "card card-stats admin-metric-card hover-scale",
            div { class: "admin-metric-card-header",
                div { class: "admin-metric-card-label-row",
                    if let Some(ic) = &icon {
                        span { class: "admin-metric-card-icon",
                            Icon { name: ic.clone(), size: Some(18), class_name: Some("text-primary".to_string()) }
                        }
                    }
                    span { class: "admin-metric-card-label text-sm text-muted-foreground", "{label}" }
                }
                if !trend_text.is_empty() {
                    span { class: "{trend_pill} text-xs font-semibold", "{trend_text}" }
                }
            }
            p { class: "admin-metric-card-value text-2xl font-semibold mt-1", "{value}" }
            if !sparkline_points.is_empty() {
                div { class: "admin-metric-card-sparkline mt-2",
                    svg {
                        width: "100%",
                        height: "40",
                        view_box: "0 0 200 40",
                        xmlns: "http://www.w3.org/2000/svg",
                        polyline {
                            points: "{sparkline_points}",
                            fill: "none",
                            stroke: "currentColor",
                            "stroke-width": "2",
                            "stroke-linejoin": "round",
                            "stroke-linecap": "round",
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

    /// Smoke test: AdminMetricCard must render both the label and the
    /// value in the SSR'd HTML. Mirrors the Wave 6A
    /// `test_render_smoke` pattern.
    #[test]
    fn admin_metric_card_renders_value_and_label() {
        let el = rsx! {
            AdminMetricCard {
                label: "Active users (24h)".to_string(),
                value: "1,234".to_string(),
                trend: Some(MetricTrend::Up(12.5)),
                sparkline_data: Some(vec![1.0, 2.0, 3.0, 4.0, 5.0]),
                icon: Some("users".to_string()),
            }
        };
        let html = dioxus_ssr::render_element(el);
        assert!(
            html.contains("Active users (24h)"),
            "AdminMetricCard must render the label. Got: {}",
            html
        );
        assert!(
            html.contains("1,234"),
            "AdminMetricCard must render the value. Got: {}",
            html
        );
        // Trend pill text — `+12.5%` is derived from the Up(12.5).
        assert!(
            html.contains("+12.5%"),
            "AdminMetricCard must render the Up trend pill. Got: {}",
            html
        );
        // The sparkline polyline must be in the rendered SVG.
        assert!(
            html.contains("<polyline"),
            "AdminMetricCard must render the sparkline polyline. Got: {}",
            html
        );
    }

    /// `Flat` trend renders as an em-dash (`—`) and the value
    /// still shows. Sparkline is omitted when `sparkline_data` is
    /// `None`.
    #[test]
    fn admin_metric_card_flat_trend_and_no_sparkline() {
        let el = rsx! {
            AdminMetricCard {
                label: "Pending approvals".to_string(),
                value: "0".to_string(),
                trend: Some(MetricTrend::Flat),
                sparkline_data: None,
                icon: None,
            }
        };
        let html = dioxus_ssr::render_element(el);
        assert!(html.contains("Pending approvals"));
        assert!(html.contains("0"));
        // Em-dash is the Flat pill text.
        assert!(
            html.contains("—"),
            "Flat trend must render an em-dash. Got: {}",
            html
        );
        // No sparkline polyline.
        assert!(
            !html.contains("<polyline"),
            "AdminMetricCard with no sparkline_data must NOT render a polyline. Got: {}",
            html
        );
    }
}
