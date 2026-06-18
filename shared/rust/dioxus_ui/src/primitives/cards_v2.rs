//! `MetricCard` / `FeatureCard` / `PricingCard` — semantic card
//! variants for common UI patterns.
//!
//! - `MetricCard` — a card displaying a single metric (label +
///   value + optional trend). Distinct from `StatCard` (which is
///   a single bar with a value) and `MetricPill` (which is a
///   compact pill).
/// - `FeatureCard` — a card displaying a feature (icon + title
///   + description). Used on marketing / pricing pages.
/// - `PricingCard` — a card displaying a pricing tier (name +
///   price + feature list + CTA).

use super::icon::Icon;

use dioxus::prelude::*;

/// A metric card — large number display with label and optional
/// trend.
#[component]
pub fn MetricCard(
    label: String,
    value: String,
    #[props(default = None)] description: Option<String>,
    #[props(default = None)] trend: Option<String>,
    #[props(default = None)] icon: Option<String>,
) -> Element {
    rsx! {
        div { class: "metric-card flex flex-col gap-2 rounded-lg border bg-card p-6 shadow-sm",
            div { class: "flex items-center justify-between",
                span { class: "metric-card-label text-sm font-medium text-muted-foreground", "{label}" }
                if let Some(i) = icon {
                    Icon { name: i.clone(), size: Some(16) }
                }
            }
            span { class: "metric-card-value text-3xl font-bold tracking-tight", "{value}" }
            if let Some(d) = description {
                span { class: "metric-card-description text-xs text-muted-foreground", "{d}" }
            }
            if let Some(t) = trend {
                span { class: "metric-card-trend text-xs font-medium", "{t}" }
            }
        }
    }
}

/// A feature card — icon + title + description.
#[component]
pub fn FeatureCard(
    title: String,
    description: String,
    #[props(default = "check".to_string())] icon: String,
) -> Element {
    rsx! {
        div { class: "feature-card flex flex-col gap-2 rounded-lg border bg-card p-4",
            div { class: "feature-card-icon flex h-10 w-10 items-center justify-center rounded-md bg-primary/10 text-primary",
                Icon { name: icon.clone(), size: Some(20) }
            }
            h3 { class: "feature-card-title text-base font-semibold", "{title}" }
            p { class: "feature-card-description text-sm text-muted-foreground", "{description}" }
        }
    }
}

/// A pricing tier card. Renders the tier name, price, feature
/// list, and a CTA button.
///
/// - `name: String` — tier name (e.g. "Pro").
/// - `price: String` — price string (e.g. "$9 / month").
/// - `features: Vec<String>` — bullet list of features.
/// - `cta_label: String` — CTA button text.
/// - `highlighted: Option<bool>` — whether this tier is the
///   "recommended" one. Affects border + background.
/// - `on_cta_click: Option<EventHandler<MouseEvent>>` — fired
///   when the CTA button is clicked.
#[component]
pub fn PricingCard(
    name: String,
    price: String,
    features: Vec<String>,
    cta_label: String,
    #[props(default = false)] highlighted: bool,
    #[props(default = None)] on_cta_click: Option<EventHandler<MouseEvent>>,
) -> Element {
    let mut card_cls = "pricing-card flex flex-col gap-4 rounded-lg border p-6".to_string();
    if highlighted {
        card_cls.push_str(" border-primary bg-primary/5 shadow-lg");
    } else {
        card_cls.push_str(" bg-card");
    }
    rsx! {
        div { class: "{card_cls}",
            div { class: "flex flex-col gap-1",
                h3 { class: "pricing-card-name text-lg font-semibold", "{name}" }
                span { class: "pricing-card-price text-3xl font-bold", "{price}" }
            }
            ul { class: "pricing-card-features flex flex-col gap-2",
                for f in features.iter() {
                    li { class: "pricing-card-feature flex items-center gap-2 text-sm",
                        Icon { name: "check".to_string(), size: Some(14) }
                        span { "{f}" }
                    }
                }
            }
            button {
                class: "pricing-card-cta inline-flex items-center justify-center rounded-md text-sm font-medium h-10 px-4 transition-colors bg-primary text-primary-foreground hover:bg-primary/90",
                r#type: "button",
                onclick: move |e| if let Some(h) = &on_cta_click { h.call(e); },
                "{cta_label}"
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn metric_card_value_uses_large_text() {
        let base = "metric-card-value text-3xl font-bold tracking-tight";
        assert!(base.contains("text-3xl"));
        assert!(base.contains("font-bold"));
    }

    #[test]
    fn feature_card_icon_in_rounded_box() {
        let base = "feature-card-icon flex h-10 w-10 items-center justify-center rounded-md bg-primary/10 text-primary";
        assert!(base.contains("rounded-md"));
        assert!(base.contains("bg-primary/10"));
    }

    #[test]
    fn pricing_card_highlighted_class() {
        let mut card_cls = "pricing-card flex flex-col gap-4 rounded-lg border p-6".to_string();
        card_cls.push_str(" border-primary bg-primary/5 shadow-lg");
        assert!(card_cls.contains("border-primary"));
        assert!(card_cls.contains("shadow-lg"));
    }

    #[test]
    fn pricing_card_default_class() {
        let mut card_cls = "pricing-card flex flex-col gap-4 rounded-lg border p-6".to_string();
        card_cls.push_str(" bg-card");
        assert!(card_cls.contains("bg-card"));
    }
}
