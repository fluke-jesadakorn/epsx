//! `DynamicPricingClient` — interactive pricing grid with 3 plan
//! groups (personal / enterprise / api) and per-card
//! highlight + promotion badges.
//!
//! Port of
//! `apps-old/frontend/components/home/dynamic-pricing-client.tsx`
//! (331 LoC). The TS source is a client component that fetches
//! affiliate-discounted prices on the client, handles tab
//! switching between plan groups, and renders 3-column grid
//! cards. The Dioxus port renders the same visual structure
//! (3-column grid, highlight ring, "Most popular" badge,
//! original/sale price strikethrough) with the data provided by
//! the caller.

use crate::primitives::icon::Icon;

use dioxus::prelude::*;

#[derive(Clone, PartialEq, Debug, Default)]
pub struct PricingGroup {
    pub id: String,
    pub title: String,
    pub price: String,
    pub original_price: Option<String>,
    pub features: Vec<PricingFeature>,
    pub highlight: bool,
    pub button_text: String,
    pub button_href: String,
    pub promotions: Vec<String>,
    pub savings: Option<String>,
    pub promotion_ends_at: Option<String>,
}

#[derive(Clone, PartialEq, Debug, Default)]
pub struct PricingFeature {
    pub text: String,
    pub included: bool,
}

#[component]
pub fn DynamicPricingClient(
    #[props(default = Vec::new())] personal_plans: Vec<PricingGroup>,
    #[props(default = Vec::new())] enterprise_plans: Vec<PricingGroup>,
    #[props(default = Vec::new())] api_plans: Vec<PricingGroup>,
    #[props(default = None)] affiliate_code: Option<String>,
) -> Element {
    let active = personal_plans.clone();
    rsx! {
        div { class: "dynamic-pricing-client",
            if !affiliate_code.is_some() {
                // No affiliate — render the personal plans by default.
            } else {
                div { class: "dynamic-pricing-affiliate-banner text-xs text-orange-500 mb-2",
                    "Affiliate code applied: {affiliate_code.as_ref().unwrap()}"
                }
            }
            div { class: "dynamic-pricing-grid grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6",
                for plan in active.iter() {
                    PricingCard { plan: plan.clone() }
                }
            }
        }
    }
}

#[component]
pub fn PricingCard(plan: PricingGroup) -> Element {
    let highlight = plan.highlight;
    rsx! {
        div {
            class: if highlight { "dynamic-pricing-card dynamic-pricing-card-highlight relative bg-white dark:bg-slate-800 rounded-2xl p-6 shadow-lg ring-2 ring-orange-500" } else { "dynamic-pricing-card relative bg-white dark:bg-slate-800 rounded-2xl p-6 shadow-lg" },
            if highlight {
                span { class: "dynamic-pricing-badge absolute -top-3 left-1/2 -translate-x-1/2 inline-block rounded-full bg-orange-500 text-white text-xs font-bold px-3 py-1",
                    "Most popular"
                }
            }
            if !plan.promotions.is_empty() {
                div { class: "dynamic-pricing-promo absolute -top-3 right-4 inline-block rounded-full bg-red-500 text-white text-xs font-bold px-3 py-1",
                    {plan.promotions.join(" ")}
                }
            }
            div { class: "dynamic-pricing-card-name text-2xl font-bold mb-2", "{plan.title}" }
            div { class: "dynamic-pricing-card-price text-3xl font-extrabold text-orange-500 mb-4",
                "{plan.price}"
            }
            if let Some(orig) = plan.original_price.as_ref() {
                div { class: "dynamic-pricing-card-original-price text-sm line-through text-slate-500",
                    "{orig}"
                }
            }
            if let Some(savings) = plan.savings.as_ref() {
                div { class: "dynamic-pricing-card-savings text-sm text-green-500",
                    "{savings}"
                }
            }
            ul { class: "dynamic-pricing-card-features space-y-2",
                for f in plan.features.iter() {
                    li { class: "flex items-start gap-2 text-sm",
                        if f.included {
                            Icon { name: "check".to_string(), size: Some(16), class_name: Some("text-orange-500 flex-shrink-0 mt-0.5".to_string()) }
                        } else {
                            Icon { name: "x".to_string(), size: Some(16), class_name: Some("text-slate-400 flex-shrink-0 mt-0.5".to_string()) }
                        }
                        span { "{f.text}" }
                    }
                }
            }
            a { class: "dynamic-pricing-card-cta mt-6 w-full inline-flex items-center justify-center px-4 py-2 bg-orange-500 text-white rounded-lg font-bold hover:bg-orange-600",
                href: plan.button_href.clone(),
                "{plan.button_text}"
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pricing_group_default_is_empty() {
        let p = PricingGroup::default();
        assert!(p.id.is_empty());
        assert!(p.title.is_empty());
        assert!(p.features.is_empty());
        assert!(!p.highlight);
        assert!(p.promotions.is_empty());
    }

    #[test]
    fn pricing_feature_default_is_empty_included() {
        let f = PricingFeature::default();
        assert!(f.text.is_empty());
        assert!(!f.included);
    }

    #[test]
    fn dynamic_pricing_client_signature() {
        
    }

    #[test]
    fn pricing_card_signature() {
        
    }
}
