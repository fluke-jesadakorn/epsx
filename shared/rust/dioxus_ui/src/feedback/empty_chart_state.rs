//! `EmptyChartState` — chart-shaped placeholder with a "No data yet"
//! message and an optional call-to-action link.
//!
//! Mirrors the empty-state pattern that appears in the Next.js
//! portfolio performance chart and any other "we'll render a chart
//! here once you connect a data source" surface. Reusable across
//! `pages/portfolio.rs` and any future "data-onboarding" chart slot.
//!
//! New in Wave 6A Track D — see
//! `docs/wave6-auth-pages-depth/design.md` §"Track D" + the
//! `feedback/` module re-export in `feedback.rs`.

use dioxus::prelude::*;

/// Renders a chart-shaped placeholder showing a faint grid, a
/// centered icon, the supplied title, and (optionally) a CTA link.
///
/// - `title` — required headline shown above the CTA (e.g.
///   `"No portfolio data yet"`).
/// - `cta_label` — when `Some`, render a button-shaped link with this
///   label. When `None`, the CTA is omitted.
/// - `cta_href` — the URL the CTA links to. When `None`, the CTA is
///   omitted. Both fields must be `Some` for the CTA to appear; this
///   guards against rendering a button that goes nowhere.
#[component]
pub fn EmptyChartState(
    title: String,
    cta_label: Option<String>,
    cta_href: Option<String>,
) -> Element {
    rsx! {
        div { class: "empty-chart-state",
            div { class: "empty-chart-state-grid",
                div { class: "empty-chart-state-icon" }
            }
            p { class: "empty-chart-state-title", "{title}" }
            if let (Some(label), Some(href)) = (cta_label.clone(), cta_href.clone()) {
                a { class: "btn btn-primary empty-chart-state-cta", href: "{href}", "{label}" }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn render_to_string(el: Element) -> String {
        dioxus_ssr::render_element(el)
    }

    /// `EmptyChartState` must render the supplied `title` text so
    /// callers can assert on the section marker (the title is the
    /// canonical "we made it through the empty-state" marker,
    /// mirroring the `Account` / `Permissions` headers used in
    /// `tests/mod.rs`).
    #[test]
    fn empty_chart_state_renders_with_title() {
        let html = render_to_string(rsx! {
            EmptyChartState {
                title: "No portfolio data yet".to_string(),
                cta_label: Some("Connect wallet".to_string()),
                cta_href: Some("/auth".to_string()),
            }
        });
        assert!(
            html.contains("No portfolio data yet"),
            "EmptyChartState should render the title text. Got: {}",
            html
        );
        assert!(
            html.contains("Connect wallet"),
            "EmptyChartState should render the CTA label. Got: {}",
            html
        );
        assert!(
            html.contains("empty-chart-state"),
            "EmptyChartState should emit the empty-chart-state class for CSS targeting. Got: {}",
            html
        );
    }

    /// CTA must NOT render when either `cta_label` or `cta_href` is
    /// `None` — guards against rendering a button that goes nowhere.
    #[test]
    fn empty_chart_state_omits_cta_when_href_missing() {
        let html = render_to_string(rsx! {
            EmptyChartState {
                title: "Standalone".to_string(),
                cta_label: Some("Go".to_string()),
                cta_href: None,
            }
        });
        assert!(html.contains("Standalone"));
        assert!(
            !html.contains(">Go</a>"),
            "CTA should be omitted when cta_href is None. Got: {}",
            html
        );
    }
}
