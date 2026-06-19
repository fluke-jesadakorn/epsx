//! `CurrentAccessCard` — shows the user's current plan + access
//! expiry.
//!
//! Port of
//! `apps-old/frontend/components/payment/current-access-card.tsx`
//! (214 LoC). The TS source renders a card with the user's
//! current plan, tier badge, and "expires on" date. The Dioxus
//! port renders the same structure with a typed `CurrentAccessInfo`
//! data prop.

use crate::primitives::icon::Icon;

use dioxus::prelude::*;

#[derive(Clone, PartialEq, Debug, Default)]
pub struct CurrentAccessInfo {
    pub plan_name: String,
    pub tier: String,
    pub expires_at: Option<String>,
    pub renews_at: Option<String>,
    pub is_active: bool,
}

#[component]
pub fn CurrentAccessCard(info: CurrentAccessInfo) -> Element {
    let status_label = if info.is_active { "Active" } else { "Inactive" };
    let status_color = if info.is_active { "green" } else { "slate" };
    rsx! {
        div { class: "current-access-card card card-glass",
            div { class: "card-header",
                div { class: "card-title flex items-center gap-2",
                    Icon { name: "shield-check".to_string(), size: Some(20), class_name: Some("text-orange-500".to_string()) }
                    "Your current access"
                }
            }
            div { class: "card-body space-y-4",
                div { class: "current-access-plan-row flex items-center justify-between",
                    div {
                        div { class: "text-xs font-medium text-slate-500", "Plan" }
                        div { class: "text-lg font-bold text-foreground", "{info.plan_name}" }
                    }
                    span {
                        class: format!("current-access-tier inline-flex items-center px-2 py-0.5 rounded-full text-xs font-medium bg-{status_color}-500/10 text-{status_color}-500"),
                        "{info.tier}"
                    }
                }
                div { class: "current-access-status flex items-center gap-2",
                    span { class: format!("h-2 w-2 rounded-full bg-{status_color}-500") }
                    span { class: "text-sm text-slate-600 dark:text-slate-300", "{status_label}" }
                }
                if let Some(exp) = info.expires_at.as_ref() {
                    div { class: "current-access-expires text-sm text-slate-500",
                        "Expires: {exp}"
                    }
                }
                if let Some(ren) = info.renews_at.as_ref() {
                    div { class: "current-access-renews text-sm text-slate-500",
                        "Renews: {ren}"
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
    fn current_access_info_default() {
        let i = CurrentAccessInfo::default();
        assert!(i.plan_name.is_empty());
        assert!(!i.is_active);
        assert!(i.expires_at.is_none());
    }

    #[test]
    fn current_access_card_smoke() {
        
    }
}
