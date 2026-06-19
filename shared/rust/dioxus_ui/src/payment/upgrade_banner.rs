//! `UpgradeBanner` — full-bleed banner prompting the user to
//! upgrade their plan.
//!
//! Port of
//! `apps-old/frontend/components/payment/upgrade-banner.tsx`
//! (181 LoC). The TS source renders a gradient banner with a
//! "Upgrade to {plan}" CTA. The Dioxus port renders the same
//! structure with a `UpgradeUrgency` prop.

use crate::primitives::icon::Icon;

use dioxus::prelude::*;

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum UpgradeUrgency {
    #[default]
    Low,
    Medium,
    High,
}

#[component]
pub fn UpgradeBanner(
    /// Plan name to upgrade to.
    plan_name: String,
    /// Urgency level — drives the color scheme.
    #[props(default = UpgradeUrgency::Medium)] urgency: UpgradeUrgency,
    /// Optional message override.
    #[props(default = None)] message: Option<String>,
    /// CTA href. Defaults to "/plans".
    #[props(default = "/plans".to_string())] href: String,
) -> Element {
    let (gradient, icon) = match urgency {
        UpgradeUrgency::Low => ("from-slate-600 to-slate-700", "info"),
        UpgradeUrgency::Medium => ("from-orange-500 to-purple-600", "trending-up"),
        UpgradeUrgency::High => ("from-red-500 to-pink-600", "alert-triangle"),
    };
    let msg = message.unwrap_or_else(|| {
        format!("Upgrade to {} to unlock more features.", plan_name)
    });
    rsx! {
        div {
            class: format!("upgrade-banner upgrade-banner-urgency-{:?} relative rounded-2xl p-6 bg-gradient-to-r {gradient} text-white shadow-xl overflow-hidden", urgency),
            div { class: "upgrade-banner-content flex flex-col sm:flex-row items-center justify-between gap-4 relative z-10",
                div { class: "upgrade-banner-message flex items-center gap-3",
                    Icon { name: icon.to_string(), size: Some(24), class_name: Some("text-white".to_string()) }
                    div {
                        h3 { class: "upgrade-banner-title text-lg font-bold", "Upgrade to {plan_name}" }
                        p { class: "upgrade-banner-sub text-sm opacity-90", "{msg}" }
                    }
                }
                a {
                    class: "upgrade-banner-cta inline-flex items-center gap-2 px-6 py-3 bg-white text-slate-900 font-bold rounded-xl hover:bg-slate-100 transition-colors",
                    href: href,
                    "View plans"
                    Icon { name: "arrow-right".to_string(), size: Some(16) }
                }
            }
            div { class: "upgrade-banner-blob absolute top-0 right-0 -mr-16 -mt-16 h-48 w-48 rounded-full bg-white/10 blur-3xl" }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn upgrade_urgency_default_is_low() {
        let u = UpgradeUrgency::default();
        assert_eq!(u, UpgradeUrgency::Low);
    }

    #[test]
    fn upgrade_banner_smoke() {
        
    }
}
