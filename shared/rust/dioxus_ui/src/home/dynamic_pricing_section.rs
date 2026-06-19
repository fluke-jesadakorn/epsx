//! `DynamicPricingSection` — server-rendered wrapper around
//! `DynamicPricingClient`.
//!
//! Port of
//! `apps-old/frontend/components/home/dynamic-pricing-section.tsx`
//! (103 LoC). The TS source is an async server component that
//! fetches plans via `getPublicPlansAction({ affiliate_code })`
//! and forwards the result to `DynamicPricingClient`. The Dioxus
//! port renders the same visual section title + the `DynamicPricingClient`
//! primitive. The data is provided by the caller (the BFF
//! fetches plans via `epsx_client::ServiceClient` and passes the
//! result through the page context).

use crate::home::dynamic_pricing_client::{DynamicPricingClient, PricingGroup};

use dioxus::prelude::*;

#[component]
pub fn DynamicPricingSection(
    /// Personal-tier plans (e.g. Free, Pro).
    #[props(default = Vec::new())] personal_plans: Vec<PricingGroup>,
    /// Enterprise-tier plans.
    #[props(default = Vec::new())] enterprise_plans: Vec<PricingGroup>,
    /// API-tier plans.
    #[props(default = Vec::new())] api_plans: Vec<PricingGroup>,
    /// Optional affiliate code. When `Some`, the client
    /// component will display the discounted price.
    #[props(default = None)] affiliate_code: Option<String>,
    /// Optional class names appended to the wrapper.
    #[props(default = None)] class_name: Option<String>,
) -> Element {
    let cls = class_name.clone().unwrap_or_default();
    rsx! {
        section { class: "home-prod-dynamic-pricing {cls}",
            div { class: "container mx-auto px-4 py-16 sm:py-24 lg:py-32",
                div { class: "text-center mb-12",
                    h2 { class: "home-prod-dynamic-pricing-title text-3xl sm:text-4xl font-bold mb-4",
                        "Plans that scale with you"
                    }
                    p { class: "home-prod-dynamic-pricing-sub text-muted-foreground max-w-2xl mx-auto",
                        "Choose the plan that fits your workflow."
                    }
                }
                DynamicPricingClient {
                    personal_plans: personal_plans,
                    enterprise_plans: enterprise_plans,
                    api_plans: api_plans,
                    affiliate_code: affiliate_code,
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dynamic_pricing_section_smoke() {
        
    }
}
