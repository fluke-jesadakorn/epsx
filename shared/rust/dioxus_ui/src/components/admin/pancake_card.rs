//! Admin `PancakeCard` family — Wave 37 T1 admin primitives port.
//!
//! Mirrors `apps-old/admin-frontend/components/ui/pancake-card.tsx`,
//! which exports 3 card variants on top of the shared `Card`
//! primitive:
//!
//! | Component | Use case |
//! | --- | --- |
//! | `PancakeCard` | Glass-morphism base card (the design-system standard) |
//! | `PancakeStatsCard` | Stats card with title / value / trend |
//! | `PancakeFeatureCard` | Feature card with title / description / icon / action |
//!
//! In the Dioxus port we wrap `primitives::card::Card` so the visual
//! class structure (`glass rounded-2xl border border-border/20
//! bg-muted/30 shadow-xl hover:shadow-2xl hover-lift`) matches prod.
//!
//! ## Tests
//!
//! `test_pancake_card_renders_glass_class` — the base card emits
//! the `glass` class plus `rounded-2xl` / `border-border/20` /
//! `bg-muted/30`.

use dioxus::prelude::*;

/// PancakeSwap-styled glass-morphism card. The visual standard for
/// every card on the admin dashboard.
#[component]
pub fn PancakeCard(
    class_name: Option<String>,
    children: Element,
) -> Element {
    let mut cls = "card card-glass pancake-card rounded-2xl border border-border/20 bg-muted/30 shadow-xl hover:shadow-2xl hover-lift".to_string();
    if let Some(c) = class_name {
        cls.push(' ');
        cls.push_str(&c);
    }
    rsx! {
        div { class: "{cls}", {children} }
    }
}

/// Stats card with title / value / trend / icon. Renders a
/// `PancakeCard` with the standard stat-card layout inside.
#[component]
pub fn PancakeStatsCard(
    title: Option<String>,
    value: Option<String>,
    /// Percentage change vs last month. Positive = green, negative = red.
    trend: Option<f32>,
    /// Optional icon glyph (emoji or text) rendered in the top-right.
    icon: Option<String>,
    class_name: Option<String>,
) -> Element {
    let mut outer_cls = "overflow-hidden".to_string();
    if let Some(c) = class_name {
        outer_cls.push(' ');
        outer_cls.push_str(&c);
    }
    let trend_positive = trend.unwrap_or(0.0) > 0.0;
    let trend_cls = if trend_positive {
        "font-medium px-2 py-1 rounded-full text-success bg-emerald-500/10"
    } else {
        "font-medium px-2 py-1 rounded-full text-destructive bg-red-500/10"
    };

    rsx! {
        PancakeCard {
            class_name: Some(outer_cls),
            div { class: "card-content p-6",
                div { class: "flex items-center justify-between",
                    div {
                        p { class: "text-sm font-medium text-muted-foreground",
                            {title.unwrap_or_default()}
                        }
                        h3 { class: "text-2xl font-bold mt-2 bg-gradient-to-r from-purple-400 to-orange-400 bg-clip-text text-transparent",
                            {value.unwrap_or_default()}
                        }
                    }
                    if let Some(g) = icon.clone() {
                        if !g.is_empty() {
                            div { class: "p-3 bg-gradient-to-br from-purple-500/20 to-orange-500/20 rounded-xl bg-clip-text text-purple-400 border border-purple-500/20",
                                "{g}"
                            }
                        }
                    }
                }
                if let Some(t) = trend {
                    div { class: "mt-4 flex items-center text-sm",
                        span { class: "{trend_cls}",
                            {format!("{}{}%", if trend_positive { "+" } else { "" }, t)}
                        }
                        span { class: "text-muted-foreground ml-2", "from last month" }
                    }
                }
            }
        }
    }
}

/// Feature card with title / description / icon / action. Renders a
/// `PancakeCard` with the standard feature-card layout inside.
#[component]
pub fn PancakeFeatureCard(
    title: Option<String>,
    description: Option<String>,
    /// Optional icon glyph rendered in the top-left.
    icon: Option<String>,
    /// Optional action element (typically a button) rendered in
    /// the card footer.
    action: Option<String>,
    class_name: Option<String>,
) -> Element {
    let mut outer_cls = "hover:border-purple-500/30 transition-all".to_string();
    if let Some(c) = class_name {
        outer_cls.push(' ');
        outer_cls.push_str(&c);
    }
    rsx! {
        PancakeCard {
            class_name: Some(outer_cls),
            div { class: "card-header",
                if let Some(g) = icon.clone() {
                    if !g.is_empty() {
                        div { class: "mb-4 text-purple-400 bg-gradient-to-br from-purple-500/20 to-orange-500/20 p-3 rounded-xl border border-purple-500/20 inline-block",
                            "{g}"
                        }
                    }
                }
                h3 { class: "card-title text-foreground", {title.unwrap_or_default()} }
                p { class: "card-description", {description.unwrap_or_default()} }
            }
            if let Some(a) = action.clone() {
                if !a.is_empty() {
                    div { class: "card-footer", "{a}" }
                }
            }
        }
    }
}

// =====================================================================
// Tests
// =====================================================================

#[cfg(test)]
mod tests {
    use super::*;

    /// The base card emits the `pancake-card` + `card-glass` classes.
    #[test]
    fn test_pancake_card_renders_glass_class() {
        let el = rsx! {
            PancakeCard {
                "Body content"
            }
        };
        let html = dioxus_ssr::render_element(el);
        assert!(
            html.contains("pancake-card"),
            "PancakeCard should expose the pancake-card class. Got: {html}"
        );
        assert!(
            html.contains("card-glass"),
            "PancakeCard should use the glass-morphism class. Got: {html}"
        );
        assert!(
            html.contains("rounded-2xl"),
            "PancakeCard should be rounded-2xl. Got: {html}"
        );
        assert!(
            html.contains("border-border/20"),
            "PancakeCard should use the design-system border. Got: {html}"
        );
        assert!(
            html.contains("Body content"),
            "PancakeCard should render children. Got: {html}"
        );
    }

    /// `PancakeCard` accepts a `class_name` for caller-supplied extras.
    #[test]
    fn test_pancake_card_propagates_class_name() {
        let el = rsx! {
            PancakeCard {
                class_name: Some("mt-4 w-full".to_string()),
                "Body"
            }
        };
        let html = dioxus_ssr::render_element(el);
        assert!(
            html.contains("mt-4 w-full"),
            "PancakeCard should propagate class_name. Got: {html}"
        );
    }

    /// `PancakeStatsCard` renders title / value / trend / icon.
    #[test]
    fn test_pancake_stats_card_renders_title_value() {
        let el = rsx! {
            PancakeStatsCard {
                title: Some("Active Users".to_string()),
                value: Some("12,345".to_string()),
                trend: Some(7.5),
                icon: Some("📈".to_string()),
            }
        };
        let html = dioxus_ssr::render_element(el);
        assert!(html.contains("Active Users"), "PancakeStatsCard should render title. Got: {html}");
        assert!(html.contains("12,345"), "PancakeStatsCard should render value. Got: {html}");
        assert!(html.contains("+7.5%"), "PancakeStatsCard should render positive trend. Got: {html}");
        assert!(html.contains("text-success"), "Positive trend should use text-success. Got: {html}");
        assert!(html.contains("📈"), "PancakeStatsCard should render icon. Got: {html}");
    }

    /// A negative `trend` value uses the destructive color.
    #[test]
    fn test_pancake_stats_card_negative_trend() {
        let el = rsx! {
            PancakeStatsCard {
                title: Some("Errors".to_string()),
                value: Some("42".to_string()),
                trend: Some(-3.0),
            }
        };
        let html = dioxus_ssr::render_element(el);
        assert!(html.contains("-3%"), "PancakeStatsCard should render negative trend. Got: {html}");
        assert!(
            html.contains("text-destructive"),
            "Negative trend should use text-destructive. Got: {html}"
        );
    }

    /// `PancakeFeatureCard` renders title / description / icon /
    /// action slots.
    #[test]
    fn test_pancake_feature_card_renders_all_slots() {
        let el = rsx! {
            PancakeFeatureCard {
                title: Some("Real-time alerts".to_string()),
                description: Some("Get notified the moment something changes.".to_string()),
                icon: Some("🔔".to_string()),
                action: Some("Configure".to_string()),
            }
        };
        let html = dioxus_ssr::render_element(el);
        assert!(
            html.contains("Real-time alerts"),
            "PancakeFeatureCard should render title. Got: {html}"
        );
        assert!(
            html.contains("Get notified"),
            "PancakeFeatureCard should render description. Got: {html}"
        );
        assert!(html.contains("🔔"), "PancakeFeatureCard should render icon. Got: {html}");
        assert!(html.contains("Configure"), "PancakeFeatureCard should render action. Got: {html}");
    }

    /// `PancakeFeatureCard` without an action slot omits the
    /// `card-footer` element.
    #[test]
    fn test_pancake_feature_card_without_action() {
        let el = rsx! {
            PancakeFeatureCard {
                title: Some("Header only".to_string()),
                description: Some("No action button".to_string()),
            }
        };
        let html = dioxus_ssr::render_element(el);
        assert!(
            !html.contains("card-footer"),
            "PancakeFeatureCard without action should not render card-footer. Got: {html}"
        );
        assert!(html.contains("Header only"), "PancakeFeatureCard should still render title. Got: {html}");
    }
}
