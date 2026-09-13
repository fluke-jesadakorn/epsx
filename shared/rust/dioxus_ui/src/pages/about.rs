use super::{PageContext, PageMeta};
use crate::{enterprise::PageHeader, layout::main_layout::MainLayout};
use dioxus::prelude::*;

pub fn render(ctx: &PageContext) -> (PageMeta, Element) {
    let mut meta = PageMeta::marketing("About EPSX");
    meta.description = "EPSX is a Financial Technology Platform for company rankings and companies you want to revisit.".into();
    (
        meta,
        rsx! { MainLayout { ctx: ctx.clone(), HydratedAbout {} } },
    )
}

#[component]
pub fn HydratedAbout() -> Element {
    rsx! {
        document::Title { "About EPSX — EPSX" }
        document::Meta { name: "description", content: "EPSX is a Financial Technology Platform for company rankings and companies you want to revisit." }

                div { class: "fe-page fe-reading-page about-page",
                    PageHeader { title: "Company rankings, made easier.", description: "Financial Technology Platform" }
                    section { class: "fe-panel",
                        h2 { "Explore, understand, keep track" }
                        p { "Explore EPSX rankings, see upcoming company reports, and save companies to revisit. Filter by country or sector, then open company details for more context." }
                        p { "Save the companies that interest you and organize them into groups. Your saved companies are a collection you choose and maintain." }
                        crate::fullstack::shell::ShellLink { class: "fe-button fe-primary", href: "/analytics", "Explore rankings" }
                    }
                    section { class: "fe-panel",
                        h2 { "A clearer next step" }
                        p { "Next action shows the next company report date. If only the previous report date is available, we estimate 90 calendar days later and label it Estimated. Dates may change." }
                        p { "Rankings are generated using EPSX’s proprietary methodology." }
                    }
                    section { class: "fe-panel",
                        h2 { "Build with the EPSX API" }
                        p { "Create API keys, review request usage and browse the API reference. Available scopes and rate limits are shown in your developer account." }
                        div { class: "fe-actions",
                            crate::fullstack::shell::ShellLink { class: "fe-button", href: "/developer", "API keys" }
                            crate::fullstack::shell::ShellLink { class: "fe-button", href: "/developer/docs", "API documentation" }
                        }
                    }
                    section { class: "fe-panel",
                        h2 { "Here to help" }
                        p { "Find help with your account, data access or payments through support." }
                        div { class: "fe-actions", crate::fullstack::shell::ShellLink { class: "fe-button", href: "/chat", "Open support" } crate::fullstack::shell::ShellLink { class: "fe-button", href: "/contact", "Contact" } }
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
