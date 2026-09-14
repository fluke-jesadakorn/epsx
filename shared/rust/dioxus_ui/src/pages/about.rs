use super::{PageContext, PageMeta};
use crate::layout::main_layout::MainLayout;
use dioxus::prelude::*;

pub fn render(ctx: &PageContext) -> (PageMeta, Element) {
    let mut meta = PageMeta::marketing("About EPSX");
    meta.description = "EPSX is a Financial Technology Platform to explore company rankings, follow upcoming reports, and organize companies you want to revisit.".into();
    (
        meta,
        rsx! { MainLayout { ctx: ctx.clone(), HydratedAbout {} } },
    )
}

#[component]
pub fn HydratedAbout() -> Element {
    rsx! {
        document::Title { "About EPSX — EPSX" }
        document::Meta { name: "description", content: "EPSX is a Financial Technology Platform to explore company rankings, follow upcoming reports, and organize companies you want to revisit." }

        div { class: "about-editorial",
            section { class: "about-hero",
                div { class: "about-intro",
                    p { class: "about-eyebrow", "ABOUT EPSX / FINANCIAL TECHNOLOGY" }
                    h1 { "Less searching." br {} span { "More perspective." } }
                    p { class: "about-lead", "A clearer way to explore company rankings, follow upcoming reports, and keep the companies that matter in view." }
                    div { class: "about-actions",
                        crate::fullstack::shell::ShellLink { class: "fe-button fe-primary", href: "/analytics", "Explore rankings ↗" }
                        crate::navigation::AppLink { class: "about-text-link", href: "#about-approach", "Meet the platform ↓" }
                    }
                }
                div { class: "about-art", "aria-hidden": "true",
                    div { class: "about-art-top", span { "THE EPSX PERSPECTIVE" } span { "01 — 03" } }
                    div { class: "about-orbit about-orbit-one" }
                    div { class: "about-orbit about-orbit-two" }
                    div { class: "about-art-core", "E", span { "+" } }
                    div { class: "about-art-tag about-tag-one", span { "01" } "Discover" }
                    div { class: "about-art-tag about-tag-two", span { "02" } "Understand" }
                    div { class: "about-art-tag about-tag-three", span { "03" } "Keep in view" }
                    p { class: "about-art-caption", "Turn information into perspective." }
                }
            }
            section { class: "about-approach", id: "about-approach",
                div { class: "about-section-heading",
                    p { class: "about-eyebrow", "BUILT AROUND YOUR CURIOSITY" }
                    h2 { "From a wider view" br {} "to your own shortlist." }
                    p { "Research is a process. EPSX brings the next steps together, so you can pick up where your curiosity takes you." }
                }
                div { class: "about-steps",
                    article { class: "about-step", span { class: "about-step-number", "01 / DISCOVER" } h3 { "Find your starting point." } p { "Explore company rankings and filter by country or sector. Open company details when something catches your attention." } }
                    article { class: "about-step", span { class: "about-step-number", "02 / UNDERSTAND" } h3 { "See what’s coming next." } p { "View upcoming company report dates alongside the rankings, with estimated dates clearly marked." } }
                    article { class: "about-step", span { class: "about-step-number", "03 / FOLLOW" } h3 { "Make it your own." } p { "Save companies that interest you and organize them into groups. Build a collection you can return to and refine." } }
                }
            }
            section { class: "about-bottom-grid",
                article { class: "about-api",
                    p { class: "about-eyebrow", "FOR BUILDERS" }
                    div { class: "about-code-mark", "{{ / }}" }
                    h2 { "Your workflow." br {} "Powered by the EPSX API." }
                    p { "Developer API access is coming in a future release." }
                    p { class: "about-eyebrow", "COMING SOON" }
                }
                article { class: "about-transparency",
                    p { class: "about-eyebrow", "A LITTLE CONTEXT" }
                    h2 { "Clarity includes" br {} "the details." }
                    p { "Rankings use EPSX’s proprietary methodology." }
                    p { "Next action shows the next company report date. When only a previous report date is available, we estimate 90 calendar days later and label it Estimated. Dates may change." }
                    div { class: "about-help",
                        h3 { "A question along the way?" }
                        p { "We’re here for account, data access, and payment questions." }
                        crate::fullstack::shell::ShellLink { class: "about-text-link", href: "/chat", "Open support ↗" }
                        crate::fullstack::shell::ShellLink { class: "about-text-link", href: "/contact", "Contact →" }
                    }
                }
            }
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn describes_actual_capabilities() {
        let (meta, body) = render(&PageContext::default());
        let html = dioxus_ssr::render_element(body);
        assert!(meta.description.contains("Financial Technology Platform"));
        assert!(html.contains("href=\"/analytics\""));
        for claim in ["guaranteed", "real-time", "institutional-grade", "IoT"] {
            assert!(!html.contains(claim));
        }
    }
}
