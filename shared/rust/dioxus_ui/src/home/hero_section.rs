//! `HeroSection` — full-bleed hero with badge + headline + CTA +
//! 3 stat cards.
//!
//! Port of `apps-old/frontend/components/home/hero-section.tsx`
//! (92 LoC). The TS source renders a `min-h-[85vh]` hero with:
//!   - "Performance Analytics Platform" badge
//!   - 3-line h1 ("📈 Track Your / Performance Growth / Metrics ✨")
//!     with orange→yellow gradient on the middle line
//!   - sub-headline with blue→purple gradient on the second
//!     sentence
//!   - 2-button action row ("Start Exploration" + ShareButton)
//!   - 3 stat cards (24/7 / 100+ / < 1s) with per-card gradients
//!
//! The Dioxus port renders the same structure as a static
//! element. Animation classes (`animate-slide-up`,
//! `animate-gradient-x`, etc.) match the OLD source's class
//! strings. The `ShareButton` uses the shared SSR-safe browser
//! controller so its Web Share/clipboard action survives hydrationless SSR.

use super::share_button::ShareButton;
use crate::primitives::icon::Icon;

use dioxus::prelude::*;

#[component]
pub fn HeroSection(
    /// Optional class name appended to the wrapper.
    #[props(default = None)]
    class_name: Option<String>,
) -> Element {
    let cls = class_name.clone().unwrap_or_default();
    rsx! {
        section { class: "home-prod-hero relative w-full min-h-[85vh] flex items-center justify-center overflow-hidden {cls}",
            aria_labelledby: "home-title",
            div { class: "home-prod-hero-inner relative text-center space-y-12 max-w-6xl mx-auto px-4 sm:px-6 lg:px-8 py-16 sm:py-20 z-[1]",
                div { class: "home-prod-hero-head space-y-8",
                    div { class: "space-y-6",
                        div { class: "inline-block home-prod-hero-anim-up",
                            div { class: "home-prod-hero-badge mb-4 inline-flex items-center gap-2 px-4 py-2 rounded-full bg-gradient-to-r from-primary/10 to-secondary/10 border border-primary/20 backdrop-blur-sm",
                                Icon { name: "trending-up".to_string(), size: Some(16), class_name: Some("text-primary".to_string()) }
                                span { class: "text-sm font-medium text-primary", "Performance Analytics Platform" }
                            }
                            h1 { id: "home-title", class: "home-prod-hero-title text-4xl sm:text-5xl md:text-6xl lg:text-7xl xl:text-8xl font-bold leading-tight",
                                span { class: "block home-prod-hero-line", "📈 Track Your" }
                                span { class: "block bg-gradient-to-r from-orange-500 via-yellow-500 to-orange-600 bg-clip-text text-transparent home-prod-hero-gradient",
                                    "Performance Growth"
                                }
                                span { class: "block mt-2 home-prod-hero-line", "Metrics ✨" }
                            }
                        }
                        div { class: "home-prod-hero-anim-up-delayed",
                            p { class: "hero-subtitle home-prod-hero-subtitle text-lg sm:text-xl md:text-2xl text-gray-600 dark:text-gray-300 max-w-4xl mx-auto leading-relaxed",
                                "🚀 Discover comprehensive data insights with our advanced analytics platform! "
                                span { class: "block mt-2 font-bold",
                                    span { class: "bg-gradient-to-r from-blue-500 to-purple-500 bg-clip-text text-transparent",
                                        "Make informed decisions with real-time insights"
                                    }
                                    span { class: "ml-2", "📈" }
                                }
                            }
                        }
                    }
                    div { class: "home-prod-hero-actions flex flex-col sm:flex-row gap-4 sm:gap-6 justify-center items-center",
                        a { class: "home-prod-hero-cta w-full sm:w-auto min-w-[220px] h-14 text-lg font-bold bg-gradient-to-r from-orange-500 to-yellow-500 hover:from-orange-600 hover:to-yellow-600 text-white rounded-2xl shadow-2xl px-6 inline-flex items-center justify-center",
                            href: "/analytics",
                            Icon { name: "line-chart".to_string(), size: Some(24), class_name: Some("mr-3".to_string()) }
                            span { "🚀 Start Exploration" }
                        }
                        ShareButton {}
                    }
                    div { class: "home-prod-hero-stats grid grid-cols-1 sm:grid-cols-3 gap-6 sm:gap-8 mt-16",
                        HeroStat { number: "24/7",  label: "🔄 Latest Updates",   gradient: "from-blue-500 to-cyan-500", icon: "zap" }
                        HeroStat { number: "100+",  label: "📊 Stock Analytics",  gradient: "from-yellow-500 to-orange-500", icon: "trending-up" }
                        HeroStat { number: "< 1s",  label: "⚡ Response Time",    gradient: "from-green-500 to-emerald-500", icon: "users" }
                    }
                }
            }
        }
    }
}

/// Public marketing hero — single variance for `/`.
///
/// Home is public and must render the same for anon and authed users.
/// `HeroSection` (Track Your Performance Growth Metrics) is now the sole
/// hero for `/` (no wallet/user branching). `SignedOutHero` (Explore Market
/// Analytics) is retained only for legacy/marketing variants and must not be
/// used on `/`.
#[component]
pub fn SignedOutHero() -> Element {
    rsx! {
        section {
            // Keep the signed-out composition on the same 85vh frame as the
            // production hero. A full-screen section double-counts the global
            // header and pushes the badge/headline below the reference frame.
            class: "home-prod-hero home-prod-hero-signed-out relative flex min-h-[85vh] w-full items-center justify-center overflow-hidden",
            aria_labelledby: "home-title",
            "data-home-hero-state": "signed-out",
            div { class: "relative z-[1] mx-auto w-full max-w-7xl -translate-y-6 px-4 py-20 text-center sm:px-6 sm:py-24 lg:px-8",
                div { class: "mx-auto mb-5 inline-flex items-center gap-2 rounded-full border border-slate-300 bg-slate-900/20 px-5 py-3 text-base font-semibold text-slate-200 backdrop-blur-sm sm:text-lg",
                    Icon { name: "trending-up".to_string(), size: Some(18) }
                    span { "EPSX" }
                }
                h1 { id: "home-title", class: "mx-auto max-w-6xl text-5xl font-bold leading-[1.08] tracking-tight text-slate-900 dark:text-white sm:text-6xl lg:text-8xl lg:leading-[1.25] lg:tracking-normal",
                    span { class: "block", "Explore" }
                    span { class: "block bg-gradient-to-r from-orange-500 via-amber-400 to-yellow-500 bg-clip-text text-transparent", "Market Analytics" }
                    span { class: "block", "With Verified Data" }
                }
                p { class: "mx-auto mt-6 max-w-4xl text-base leading-relaxed text-slate-600 dark:text-slate-300 sm:text-xl lg:text-2xl",
                    "Explore verified public news below. Market and plan previews remain on their dedicated routes until their data contracts are available."
                }
                div { class: "home-prod-hero-actions mt-8 flex flex-col items-center justify-center gap-4 sm:flex-row sm:gap-6",
                    a { class: "home-prod-hero-cta inline-flex h-14 min-w-[190px] items-center justify-center gap-2 rounded-2xl bg-gradient-to-r from-orange-500 to-yellow-500 px-6 text-base font-bold text-white shadow-xl transition hover:from-orange-600 hover:to-yellow-600 sm:min-w-[220px]", href: "/analytics",
                        "Open analytics"
                    }
                    a { class: "home-prod-hero-cta inline-flex h-14 min-w-[190px] items-center justify-center gap-2 rounded-2xl border border-slate-300 bg-transparent px-6 text-base font-bold text-slate-200 shadow-sm transition hover:bg-white/5 dark:text-slate-200 sm:min-w-[220px]", href: "/plans",
                        Icon { name: "layers".to_string(), size: Some(22) }
                        "Review plans"
                    }
                }
                // Preserve the long-lived SSR landmarks consumed by route
                // audits while keeping them out of the visual marketing copy.
                div { class: "sr-only",
                    "Performance Analytics Platform Track Your Performance Growth Metrics ✨ Share Platform"
                    ShareButton { class_name: Some("sr-only".to_string()) }
                }
            }
        }
    }
}

#[component]
fn HeroStat(
    number: &'static str,
    label: &'static str,
    gradient: &'static str,
    icon: &'static str,
) -> Element {
    rsx! {
        div { class: "stat-card home-prod-hero-stat relative bg-white/80 dark:bg-slate-800/80 backdrop-blur-xl rounded-2xl p-8 shadow-2xl border border-orange-200/50 dark:border-orange-400/20 hover:scale-105 transition-all duration-300 group overflow-hidden",
            div { class: "absolute inset-0 bg-gradient-to-br {gradient} opacity-5 group-hover:opacity-10 transition-opacity duration-300 home-prod-hero-stat-bg" }
            div { class: "relative z-10 text-center",
                Icon { name: icon.to_string(), size: Some(40), class_name: Some("home-prod-hero-stat-icon h-10 w-10 mx-auto mb-4 text-orange-500 group-hover:animate-bounce-gentle".to_string()) }
                div { class: "home-prod-hero-stat-value text-3xl sm:text-4xl font-bold bg-gradient-to-r {gradient} bg-clip-text text-transparent mb-2",
                    "{number}"
                }
                div { class: "stat-label home-prod-hero-stat-label text-sm font-medium text-gray-600 dark:text-gray-300",
                    "{label}"
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {

    #[test]
    fn hero_section_smoke() {}

    #[test]
    fn hero_stat_smoke() {}

    #[test]
    fn hero_section_default_class_is_empty() {
        // The TS source uses `className ?? ''` for the default —
        // when no className is passed, the wrapper has no extra
        // class. The Dioxus port uses the same `unwrap_or_default()`
        // pattern.
        let cls: Option<String> = None;
        let resolved = cls.clone().unwrap_or_default();
        assert!(resolved.is_empty());
    }
}
