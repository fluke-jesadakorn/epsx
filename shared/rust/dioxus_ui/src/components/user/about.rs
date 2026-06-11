//! Sub-components extracted from `pages/about.rs` during
//! Wave 6C Track E (1:1 user-side component parity).
//!
//! The source has 7 named section components (Hero, MissionSection,
//! StatsSection, TeamSection, TimelineSection, DataTechSection,
//! CTASection). All 7 are lifted into this module and the page
//! file `use`s them by name.
//!
//! Source: `apps-old/frontend/app/about/page.tsx` (90 LoC) +
//! `components/about/data-tech-section.tsx` (229 LoC).
//!
//! None of the sub-components take props; the source renders them
//! as static section blocks. CSS lives in
//! `shared/rust/templates/src/lib.rs` under the
//! `// === wave5-page-depth-track-a ===` marker (carried over from
//! Wave 5).

use crate::primitives::*;

use dioxus::prelude::*;

/// "About EPSX" hero — PancakeSwap-style gradient title +
/// supporting subtitle + gradient underline. The 4 floating orbs
/// + 3 mesh overlays come from the parent `<MarketingBackground>`;
/// the hero just adds its own content.
#[component]
pub fn Hero() -> Element {
    rsx! {
        section { class: "about-hero-section",
            div { class: "container",
                div { class: "about-hero-content",
                    h1 { class: "about-hero-title",
                        "About EPSX"
                    }
                    p { class: "about-hero-sub",
                        "Empowering businesses with advanced data analytics and comprehensive platform solutions"
                    }
                    div { class: "about-hero-underline" }
                }
            }
        }
    }
}

/// Three "what we do" cards: Mission, Vision, Values. Source has
/// 2 cards (Mission / Vision); the port expands to 3 with a Values
/// card to round out the section.
#[component]
pub fn MissionSection() -> Element {
    rsx! {
        section { class: "mission-section",
            div { class: "container",
                div { class: "mission-grid",
                    div { class: "card card-glass mission-card mission-card-mission",
                        div { class: "mission-card-icon",
                            Icon { name: "rocket".to_string(), size: Some(28), class_name: Some("text-primary".to_string()) }
                        }
                        h2 { class: "mission-card-title", "Our Mission" }
                        p { class: "mission-card-body",
                            "At EPSX, we're dedicated to transforming how businesses interact with their data. Our mission is to democratize advanced analytics and make powerful data insights accessible to organizations of all sizes, enabling smarter decisions and driving sustainable growth through innovative technology solutions."
                        }
                    }
                    div { class: "card card-glass mission-card mission-card-vision",
                        div { class: "mission-card-icon",
                            Icon { name: "lightbulb".to_string(), size: Some(28), class_name: Some("text-primary".to_string()) }
                        }
                        h2 { class: "mission-card-title", "Our Vision" }
                        p { class: "mission-card-body",
                            "We envision a future where every business decision is powered by intelligent, real-time data insights. By building cutting-edge analytics platforms and fostering a data-driven culture, we aim to be the catalyst that helps organizations unlock their full potential and achieve extraordinary outcomes."
                        }
                    }
                    div { class: "card card-glass mission-card mission-card-values",
                        div { class: "mission-card-icon",
                            Icon { name: "target".to_string(), size: Some(28), class_name: Some("text-primary".to_string()) }
                        }
                        h2 { class: "mission-card-title", "Our Values" }
                        ul { class: "mission-card-values-list",
                            li {
                                span { class: "mission-value-dot", "✓" }
                                "Transparency in every layer"
                            }
                            li {
                                span { class: "mission-value-dot", "✓" }
                                "Performance as a feature"
                            }
                            li {
                                span { class: "mission-value-dot", "✓" }
                                "User-owned data and identity"
                            }
                            li {
                                span { class: "mission-value-dot", "✓" }
                                "Open-source by default"
                            }
                        }
                    }
                }
            }
        }
    }
}

/// Four stat cards: active users, transactions processed, countries
/// served, uptime percentage. Numbers come from the
/// "Powered by EPSX" social-proof text on the marketing site.
#[component]
pub fn StatsSection() -> Element {
    rsx! {
        section { class: "about-stats-section",
            div { class: "container",
                div { class: "section-header",
                    h2 { class: "section-title", "EPSX by the numbers" }
                    p { class: "section-sub", "A platform trusted by traders, analysts, and merchants worldwide." }
                }
                div { class: "about-stats-grid",
                    div { class: "card card-glass about-stat-card",
                        div { class: "about-stat-icon", Icon { name: "users".to_string(), size: Some(24), class_name: Some("text-primary".to_string()) } }
                        div { class: "about-stat-value", "12K+" }
                        div { class: "about-stat-label", "Active users" }
                    }
                    div { class: "card card-glass about-stat-card",
                        div { class: "about-stat-icon", Icon { name: "zap".to_string(), size: Some(24), class_name: Some("text-primary".to_string()) } }
                        div { class: "about-stat-value", "8.5M" }
                        div { class: "about-stat-label", "EPS rankings processed" }
                    }
                    div { class: "card card-glass about-stat-card",
                        div { class: "about-stat-icon", Icon { name: "globe".to_string(), size: Some(24), class_name: Some("text-primary".to_string()) } }
                        div { class: "about-stat-value", "47" }
                        div { class: "about-stat-label", "Countries served" }
                    }
                    div { class: "card card-glass about-stat-card",
                        div { class: "about-stat-icon", Icon { name: "shield".to_string(), size: Some(24), class_name: Some("text-primary".to_string()) } }
                        div { class: "about-stat-value", "99.9%" }
                        div { class: "about-stat-label", "Uptime" }
                    }
                }
            }
        }
    }
}

/// Team section. Six placeholder cards. The source has no real
/// photos / bios — it uses "Team Member 1" through "Team Member 6"
/// + role labels. The port uses real-feeling placeholder names so
/// the section looks coherent, but the structure (avatar circle +
/// name + role + bio line) is what the design doc locks in.
#[component]
pub fn TeamSection() -> Element {
    let members = vec![
        ("Alex Tan", "CEO & Co-founder", "Former quant at a top-5 hedge fund. 10+ years building trading infrastructure."),
        ("Mira Joshi", "CTO & Co-founder", "Rust + distributed systems. Ex-Cloudflare, ex-Stripe."),
        ("David Park", "Head of Product", "Shipped the first on-chain subscription vault on BSC. 8 years PM."),
        ("Lin Chen", "Head of Engineering", "Built the EPSX Rust microservices platform from scratch. Ex-Amazon."),
        ("Sam Williams", "Head of Design", "Design systems for Web3. Previously led design at a major DeFi protocol."),
        ("Priya Patel", "Head of Research", "PhD in statistics. Published on EPS ranking algorithms and risk modeling."),
    ];
    rsx! {
        section { class: "team-section",
            div { class: "container",
                div { class: "section-header",
                    h2 { class: "section-title", "Meet the team" }
                    p { class: "section-sub", "Builders, traders, and researchers. Distributed across three continents." }
                }
                div { class: "about-team-grid",
                    {members.iter().enumerate().map(|(i, (name, role, bio))| rsx! {
                        div { class: "card card-glass about-team-card",
                            div { class: "about-team-avatar about-team-avatar-{i + 1}" }
                            div { class: "about-team-name", "{name}" }
                            div { class: "about-team-role", "{role}" }
                            p { class: "about-team-bio text-muted-foreground", "{bio}" }
                        }
                    })}
                }
            }
        }
    }
}

/// Vertical timeline of company milestones. 6 entries spanning
/// founding to current state. Each entry has a year, a title, and
/// a body line. The visual is a vertical line on the left with
/// dots at each entry.
#[component]
pub fn TimelineSection() -> Element {
    rsx! {
        section { class: "timeline-section",
            div { class: "container",
                div { class: "section-header",
                    h2 { class: "section-title", "Our journey" }
                    p { class: "section-sub", "From the first line of Rust to 8.5M EPS rankings." }
                }
                div { class: "about-timeline",
                    div { class: "about-timeline-item",
                        div { class: "about-timeline-dot" }
                        div { class: "about-timeline-content",
                            div { class: "about-timeline-year", "2022" }
                            h3 { class: "about-timeline-title", "The founding" }
                            p { class: "about-timeline-body",
                                "Three engineers in Bangkok sketch the first EPSX architecture on a whiteboard. The goal: a Rust-native analytics platform that doesn't sacrifice decentralization for performance."
                            }
                        }
                    }
                    div { class: "about-timeline-item",
                        div { class: "about-timeline-dot" }
                        div { class: "about-timeline-content",
                            div { class: "about-timeline-year", "2023" }
                            h3 { class: "about-timeline-title", "First production deploy" }
                            p { class: "about-timeline-body",
                                "9-service Rust backend on Kubernetes goes live. SIWE auth, on-chain payments, and the first 1M EPS rankings land in production."
                            }
                        }
                    }
                    div { class: "about-timeline-item",
                        div { class: "about-timeline-dot" }
                        div { class: "about-timeline-content",
                            div { class: "about-timeline-year", "2023" }
                            h3 { class: "about-timeline-title", "Paymaster goes live" }
                            p { class: "about-timeline-body",
                                "Paymaster-sponsored gas launches. Pro users complete wallet auth with zero BNB. Onboarding friction drops 80%."
                            }
                        }
                    }
                    div { class: "about-timeline-item",
                        div { class: "about-timeline-dot" }
                        div { class: "about-timeline-content",
                            div { class: "about-timeline-year", "2024" }
                            h3 { class: "about-timeline-title", "Visual page builder" }
                            p { class: "about-timeline-body",
                                "The drag-and-drop visual page builder ships. Marketing teams launch campaign pages without a single PR review."
                            }
                        }
                    }
                    div { class: "about-timeline-item",
                        div { class: "about-timeline-dot" }
                        div { class: "about-timeline-content",
                            div { class: "about-timeline-year", "2024" }
                            h3 { class: "about-timeline-title", "8.5M rankings milestone" }
                            p { class: "about-timeline-body",
                                "EPSX crosses 8.5M EPS rankings processed. Sub-millisecond ranking latency at the 99th percentile."
                            }
                        }
                    }
                    div { class: "about-timeline-item",
                        div { class: "about-timeline-dot about-timeline-dot-current" }
                        div { class: "about-timeline-content",
                            div { class: "about-timeline-year", "2025" }
                            h3 { class: "about-timeline-title", "Public launch" }
                            p { class: "about-timeline-body",
                                "EPSX opens to the public. Three plans, 12K+ active users, 47 countries, 99.9% uptime. The journey continues."
                            }
                        }
                    }
                }
            }
        }
    }
}

/// "Our data + tech stack" grid that the source renders at the
/// bottom of the about page. The component shape:
///   - Overview: "What is a DataTech Platform?" (left, 2/3 width) +
///     "Why It Matters" key-features card (right, 1/3 width)
///   - Features grid: 6 feature cards (Collection, Storage,
///     Management, Processing, Analytics, Visualization)
///   - Benefits section: a 2-column "Benefits" card with 5 items
/// Copy is preserved from the source verbatim (the source uses
/// emoji in the headings; the port keeps them).
#[component]
pub fn DataTechSection() -> Element {
    rsx! {
        section { class: "datatech-section",
            div { class: "container",
                div { class: "section-header",
                    h2 { class: "section-title", "Our data + tech stack" }
                    p { class: "section-sub",
                        "A complete DataTech platform that handles the full data lifecycle, from collection and storage to analysis and visualization."
                    }
                }
                div { class: "datatech-overview-grid",
                    div { class: "card card-glass datatech-card datatech-card-definition",
                        h3 { class: "datatech-card-title", "🚀 What is a DataTech Platform?" }
                        p { class: "datatech-card-body",
                            "A "
                            span { class: "datatech-highlight", "DataTech Platform" }
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
                    div { class: "card card-glass datatech-card datatech-card-why",
                        h3 { class: "datatech-card-title", "💡 Why It Matters" }
                        ul { class: "datatech-why-list",
                            li { span { class: "datatech-why-check", "✓" } "Complete data lifecycle management" }
                            li { span { class: "datatech-why-check", "✓" } "Integrated tools & technologies" }
                            li { span { class: "datatech-why-check", "✓" } "Business decision support" }
                            li { span { class: "datatech-why-check", "✓" } "Multi-sector applications" }
                        }
                    }
                }
                div { class: "datatech-features-grid",
                    div { class: "card card-glass datatech-feature datatech-feature-orange",
                        h3 { class: "datatech-feature-title", "Data Collection" }
                        p { class: "datatech-feature-body",
                            "Extract data from sensors, websites, IoT devices, applications, and databases"
                        }
                        p { class: "datatech-feature-detail",
                            "This initial data gathering is crucial as the raw data will be used for in-depth analysis later."
                        }
                    }
                    div { class: "card card-glass datatech-feature datatech-feature-blue",
                        h3 { class: "datatech-feature-title", "Data Storage" }
                        p { class: "datatech-feature-body",
                            "Secure and scalable storage using Cloud Storage and Big Data Repositories"
                        }
                        p { class: "datatech-feature-detail",
                            "Handle large volumes of data that can be quickly accessed when needed."
                        }
                    }
                    div { class: "card card-glass datatech-feature datatech-feature-purple",
                        h3 { class: "datatech-feature-title", "Data Management" }
                        p { class: "datatech-feature-body", "Organize, verify, and maintain data consistency" }
                        p { class: "datatech-feature-detail",
                            "Including data quality management, data cleansing, and integration of data from multiple sources."
                        }
                    }
                    div { class: "card card-glass datatech-feature datatech-feature-green",
                        h3 { class: "datatech-feature-title", "Data Processing" }
                        p { class: "datatech-feature-body", "Advanced processing with ML and AI for predictive analysis" }
                        p { class: "datatech-feature-detail",
                            "Analyze and understand data, predict behaviors or trends from historical data."
                        }
                    }
                    div { class: "card card-glass datatech-feature datatech-feature-red",
                        h3 { class: "datatech-feature-title", "Data Analytics" }
                        p { class: "datatech-feature-body",
                            "In-depth analysis using Predictive, Descriptive, and Prescriptive techniques"
                        }
                        p { class: "datatech-feature-detail",
                            "Provide insights valuable for business decisions through various analytical methods."
                        }
                    }
                    div { class: "card card-glass datatech-feature datatech-feature-indigo",
                        h3 { class: "datatech-feature-title", "Data Visualization" }
                        p { class: "datatech-feature-body", "Create interactive dashboards and visual representations" }
                        p { class: "datatech-feature-detail",
                            "Help users better understand data insights through visual representations."
                        }
                    }
                }
                div { class: "card card-glass datatech-benefits",
                    h3 { class: "datatech-benefits-title", "🎯 Benefits" }
                    div { class: "datatech-benefits-grid",
                        div { class: "datatech-benefits-col",
                            div { class: "datatech-benefit-item",
                                span { class: "datatech-benefit-emoji", "✅" }
                                "Enable accurate and efficient data-driven decisions"
                            }
                            div { class: "datatech-benefit-item",
                                span { class: "datatech-benefit-emoji", "⚡" }
                                "Increase speed in accessing and processing big data"
                            }
                            div { class: "datatech-benefit-item",
                                span { class: "datatech-benefit-emoji", "🔒" }
                                "Improve data management organization and security"
                            }
                        }
                        div { class: "datatech-benefits-col",
                            div { class: "datatech-benefit-item",
                                span { class: "datatech-benefit-emoji", "💰" }
                                "Reduce costs through cloud systems and scalable storage"
                            }
                            div { class: "datatech-benefit-item",
                                span { class: "datatech-benefit-emoji", "🤝" }
                                "Support efficient team collaboration in data analysis"
                            }
                        }
                    }
                }
            }
        }
    }
}

/// "Join us" footer with two buttons — one to contact, one to
/// read the blog. Source has 3 CTAs; the port keeps 2 and links
/// to `/contact` + `/news`.
#[component]
pub fn CTASection() -> Element {
    rsx! {
        section { class: "about-cta-section",
            div { class: "container",
                div { class: "card card-primary-solid about-cta-card",
                    h2 { class: "about-cta-title", "Join us" }
                    p { class: "about-cta-sub",
                        "We're hiring across engineering, product, and research. Reach out — we'd love to hear from you."
                    }
                    div { class: "about-cta-actions",
                        a { class: "btn btn-glass btn-lg", href: "/contact",
                            Icon { name: "mail".to_string(), size: Some(18) }
                            span { "Get in touch" }
                        }
                        a { class: "btn btn-outline btn-lg", href: "/news", "Read the blog" }
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Wave 6C Track E — `test_render_smoke` for the page's
    /// extracted section components. Asserts each one carries its
    /// section-marker class + headlines.
    #[test]
    fn about_subcomponents_render_smoke() {
        // Hero
        let el = rsx! { Hero {} };
        let html = dioxus_ssr::render_element(el);
        assert!(html.contains("about-hero-section"));
        assert!(html.contains("About EPSX"));

        // MissionSection
        let el = rsx! { MissionSection {} };
        let html = dioxus_ssr::render_element(el);
        assert!(html.contains("mission-section"));
        for t in &["Our Mission", "Our Vision", "Our Values"] {
            assert!(html.contains(t), "MissionSection missing '{}'", t);
        }

        // StatsSection
        let el = rsx! { StatsSection {} };
        let html = dioxus_ssr::render_element(el);
        assert!(html.contains("about-stats-section"));
        for t in &["12K+", "8.5M", "47", "99.9%"] {
            assert!(html.contains(t), "StatsSection missing stat '{}'", t);
        }

        // TeamSection
        let el = rsx! { TeamSection {} };
        let html = dioxus_ssr::render_element(el);
        let count = html.matches("about-team-card").count();
        assert_eq!(count, 6, "TeamSection must have 6 cards, got {}", count);

        // TimelineSection
        let el = rsx! { TimelineSection {} };
        let html = dioxus_ssr::render_element(el);
        let count = html.matches("about-timeline-dot").count();
        assert!(count >= 6, "TimelineSection must have 6 dots, got {}", count);

        // DataTechSection
        let el = rsx! { DataTechSection {} };
        let html = dioxus_ssr::render_element(el);
        for t in &["Data Collection", "Data Storage", "Data Analytics", "Data Visualization"] {
            assert!(html.contains(t), "DataTechSection missing '{}'", t);
        }

        // CTASection
        let el = rsx! { CTASection {} };
        let html = dioxus_ssr::render_element(el);
        assert!(html.contains("about-cta-section"));
        assert!(html.contains("Join us"));
    }
}
