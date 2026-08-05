use super::{PageContext, PageMeta};
use crate::layout::main_layout::MainLayout;
use crate::layout::marketing_bg::MarketingBackground;
use dioxus::prelude::*;

const TITLE: &str = "About Us - EPSX Analytics Platform";
const DESCRIPTION: &str = "Learn about EPSX DataTech Platform - comprehensive technology platform designed to manage the complete data lifecycle, from collection and storage to analysis and visualization.";
const KEYWORDS: &str =
    "EPSX, DataTech Platform, data analytics, business intelligence, data management";

/// Static `/about` page, pinned to
/// `origin/development@373bd231cb7a616c3d4c0ddc1d60e0099a88a5db`.
///
/// The source contains only the hero, DataTech lifecycle content, mission, and
/// vision. Team biographies, company statistics, roadmap milestones, values,
/// and hiring claims are intentionally absent because the pinned source does
/// not establish any of them.
pub fn render(ctx: &PageContext) -> (PageMeta, Element) {
    let mut meta = PageMeta::marketing("About");
    meta.title = TITLE.to_string();
    meta.description = DESCRIPTION.to_string();
    meta.keywords = Some(KEYWORDS.to_string());

    (
        meta,
        rsx! {
            div { class: "about-page",
                MainLayout { ctx: ctx.clone(),
                    MarketingBackground {
                        Hero {}
                        DataTechSection {}
                        MissionAndVision {}
                    }
                }
            }
        },
    )
}

#[component]
fn Hero() -> Element {
    rsx! {
        section { class: "about-hero-section", "aria-labelledby": "about-title",
            div { class: "container",
                div { class: "about-hero-content",
                    h1 { id: "about-title", class: "about-hero-title", "About EPSX" }
                    p { class: "about-hero-sub",
                        "Empowering businesses with advanced data analytics and comprehensive platform solutions"
                    }
                    div { class: "about-hero-underline", "aria-hidden": "true" }
                }
            }
        }
    }
}

#[component]
fn DataTechSection() -> Element {
    rsx! {
        section { class: "datatech-section", "aria-label": "DataTech Platform",
            div { class: "container",
                div { class: "datatech-overview-grid",
                    article { class: "card card-glass datatech-card datatech-card-definition", "aria-labelledby": "datatech-definition-title",
                        h2 { id: "datatech-definition-title", class: "datatech-card-title",
                            span { "aria-hidden": "true", "🚀 " }
                            "What is a DataTech Platform?"
                        }
                        p { class: "datatech-card-body",
                            "A "
                            strong { class: "datatech-highlight", "DataTech Platform" }
                            " is a comprehensive technology ecosystem designed to handle your complete data journey."
                        }
                        p { class: "datatech-card-body",
                            "From initial "
                            span { class: "datatech-text-orange", "collection and storage" }
                            " to advanced "
                            span { class: "datatech-text-blue", "analysis and visualization" }
                            ", these platforms integrate cutting-edge tools to maximize data value."
                        }
                    }
                    article { class: "card card-glass datatech-card datatech-card-why", "aria-labelledby": "datatech-why-title",
                        h2 { id: "datatech-why-title", class: "datatech-card-title",
                            span { "aria-hidden": "true", "💡 " }
                            "Why It Matters"
                        }
                        ul { class: "datatech-why-list",
                            li { span { class: "datatech-why-check", "aria-hidden": "true", "✓" } "Complete data lifecycle management" }
                            li { span { class: "datatech-why-check", "aria-hidden": "true", "✓" } "Integrated tools & technologies" }
                            li { span { class: "datatech-why-check", "aria-hidden": "true", "✓" } "Business decision support" }
                            li { span { class: "datatech-why-check", "aria-hidden": "true", "✓" } "Multi-sector applications" }
                        }
                    }
                }

                div { class: "datatech-features-grid",
                    FeatureCard {
                        id: "collection",
                        class_name: "datatech-feature-orange",
                        title: "Data Collection",
                        description: "Extract data from sensors, websites, IoT devices, applications, and databases",
                        details: "This initial data gathering is crucial as the raw data will be used for in-depth analysis later."
                    }
                    FeatureCard {
                        id: "storage",
                        class_name: "datatech-feature-blue",
                        title: "Data Storage",
                        description: "Secure and scalable storage using Cloud Storage and Big Data Repositories",
                        details: "Handle large volumes of data that can be quickly accessed when needed."
                    }
                    FeatureCard {
                        id: "management",
                        class_name: "datatech-feature-purple",
                        title: "Data Management",
                        description: "Organize, verify, and maintain data consistency",
                        details: "Including data quality management, data cleansing, and integration of data from multiple sources."
                    }
                    FeatureCard {
                        id: "processing",
                        class_name: "datatech-feature-green",
                        title: "Data Processing",
                        description: "Advanced processing with ML and AI for predictive analysis",
                        details: "Analyze and understand data, predict behaviors or trends from historical data."
                    }
                    FeatureCard {
                        id: "analytics",
                        class_name: "datatech-feature-red",
                        title: "Data Analytics",
                        description: "In-depth analysis using Predictive, Descriptive, and Prescriptive techniques",
                        details: "Provide insights valuable for business decisions through various analytical methods."
                    }
                    FeatureCard {
                        id: "visualization",
                        class_name: "datatech-feature-indigo",
                        title: "Data Visualization",
                        description: "Create interactive dashboards and visual representations",
                        details: "Help users better understand data insights through visual representations."
                    }
                }

                article { class: "card card-glass datatech-benefits", "aria-labelledby": "datatech-benefits-title",
                    h2 { id: "datatech-benefits-title", class: "datatech-benefits-title",
                        span { "aria-hidden": "true", "🎯 " }
                        "Benefits"
                    }
                    div { class: "datatech-benefits-grid",
                        ul { class: "datatech-benefits-col",
                            Benefit { emoji: "✅", text: "Enable accurate and efficient data-driven decisions" }
                            Benefit { emoji: "⚡", text: "Increase speed in accessing and processing big data" }
                            Benefit { emoji: "🔒", text: "Improve data management organization and security" }
                        }
                        ul { class: "datatech-benefits-col",
                            Benefit { emoji: "💰", text: "Reduce costs through cloud systems and scalable storage" }
                            Benefit { emoji: "🤝", text: "Support efficient team collaboration in data analysis" }
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn FeatureCard(
    id: &'static str,
    class_name: &'static str,
    title: &'static str,
    description: &'static str,
    details: &'static str,
) -> Element {
    let heading_id = format!("datatech-{id}-title");
    rsx! {
        article { class: "card card-glass datatech-feature {class_name}", "aria-labelledby": heading_id.clone(),
            h2 { id: heading_id, class: "datatech-feature-title", "{title}" }
            p { class: "datatech-feature-body", "{description}" }
            p { class: "datatech-feature-detail", "{details}" }
        }
    }
}

#[component]
fn Benefit(emoji: &'static str, text: &'static str) -> Element {
    rsx! {
        li { class: "datatech-benefit-item",
            span { class: "datatech-benefit-emoji", "aria-hidden": "true", "{emoji}" }
            "{text}"
        }
    }
}

#[component]
fn MissionAndVision() -> Element {
    rsx! {
        section { class: "mission-section", "aria-label": "EPSX mission and vision",
            div { class: "container",
                div { class: "mission-grid",
                    article { class: "card card-glass mission-card mission-card-mission", "aria-labelledby": "about-mission-title",
                        h2 { id: "about-mission-title", class: "mission-card-title", "Our Mission" }
                        p { class: "mission-card-body",
                            "At EPSX, we're dedicated to transforming how businesses interact with their data. Our mission is to democratize advanced analytics and make powerful data insights accessible to organizations of all sizes, enabling smarter decisions and driving sustainable growth through innovative technology solutions."
                        }
                    }
                    article { class: "card card-glass mission-card mission-card-vision", "aria-labelledby": "about-vision-title",
                        h2 { id: "about-vision-title", class: "mission-card-title", "Our Vision" }
                        p { class: "mission-card-body",
                            "We envision a future where every business decision is powered by intelligent, real-time data insights. By building cutting-edge analytics platforms and fostering a data-driven culture, we aim to be the catalyst that helps organizations unlock their full potential and achieve extraordinary outcomes."
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

    fn rendered_about() -> (PageMeta, String) {
        let ctx = PageContext {
            path: "/about".to_string(),
            ..Default::default()
        };
        let (meta, element) = render(&ctx);
        (meta, dioxus_ssr::render_element(element))
    }

    #[test]
    fn pinned_metadata_is_exact() {
        let (meta, _) = rendered_about();
        assert_eq!(meta.title, TITLE);
        assert_eq!(meta.description, DESCRIPTION);
        assert_eq!(meta.keywords.as_deref(), Some(KEYWORDS));
    }

    #[test]
    fn pinned_content_and_order_are_exact() {
        let (_, html) = rendered_about();
        let ordered = [
            "About EPSX",
            "What is a DataTech Platform?",
            "Data Collection",
            "Data Storage",
            "Data Management",
            "Data Processing",
            "Data Analytics",
            "Data Visualization",
            "Benefits",
            "Our Mission",
            "Our Vision",
        ];
        let mut cursor = 0;
        for text in ordered {
            let offset = html[cursor..]
                .find(text)
                .unwrap_or_else(|| panic!("missing pinned source text: {text}"));
            cursor += offset + text.len();
        }
        for text in [
            "Empowering businesses with advanced data analytics and comprehensive platform solutions",
            "Complete data lifecycle management",
            "Enable accurate and efficient data-driven decisions",
            "sustainable growth through innovative technology solutions",
            "achieve extraordinary outcomes",
        ] {
            assert!(html.contains(text), "missing pinned source copy: {text}");
        }
    }

    #[test]
    fn invented_company_claims_do_not_render() {
        let (_, html) = rendered_about();
        for text in [
            "Meet the team",
            "Alex Tan",
            "EPSX by the numbers",
            "12K+",
            "Our journey",
            "The founding",
            "Our Values",
            "Join us",
            "We're hiring",
        ] {
            assert!(
                !html.contains(text),
                "invented claim must not render: {text}"
            );
        }
    }

    #[test]
    fn accessible_landmarks_and_heading_hierarchy_are_stable() {
        let (_, html) = rendered_about();
        assert_eq!(html.matches("<h1").count(), 1);
        assert_eq!(html.matches("<h2").count(), 11);
        assert_eq!(html.matches("<article").count(), 11);
        assert!(html.contains("aria-labelledby=\"about-title\""));
        assert!(html.contains("aria-label=\"DataTech Platform\""));
        assert!(html.contains("aria-label=\"EPSX mission and vision\""));
        assert!(!html.contains("progressive-auth-banner"));
    }
}
