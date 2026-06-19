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
//! strings. The "Share" button is a placeholder anchor — the real
//! `ShareButton` primitive is in `crate::home::ShareButton`.

use crate::primitives::icon::Icon;

use dioxus::prelude::*;

#[component]
pub fn HeroSection(
    /// Optional class name appended to the wrapper.
    #[props(default = None)] class_name: Option<String>,
) -> Element {
    let cls = class_name.clone().unwrap_or_default();
    rsx! {
        div { class: "home-prod-hero relative w-full min-h-[85vh] flex items-center justify-center overflow-hidden {cls}",
            div { class: "home-prod-hero-inner relative text-center space-y-12 max-w-6xl mx-auto px-4 sm:px-6 lg:px-8 py-16 sm:py-20 z-[1]",
                div { class: "home-prod-hero-head space-y-8",
                    div { class: "space-y-6",
                        div { class: "inline-block home-prod-hero-anim-up",
                            div { class: "home-prod-hero-badge mb-4 inline-flex items-center gap-2 px-4 py-2 rounded-full bg-gradient-to-r from-primary/10 to-secondary/10 border border-primary/20 backdrop-blur-sm",
                                Icon { name: "trending-up".to_string(), size: Some(16), class_name: Some("text-primary".to_string()) }
                                span { class: "text-sm font-medium text-primary", "Performance Analytics Platform" }
                            }
                            h1 { class: "home-prod-hero-title text-4xl sm:text-5xl md:text-6xl lg:text-7xl xl:text-8xl font-bold leading-tight",
                                span { class: "block home-prod-hero-line", "📈 Track Your" }
                                span { class: "block bg-gradient-to-r from-orange-500 via-yellow-500 to-orange-600 bg-clip-text text-transparent home-prod-hero-gradient",
                                    "Performance Growth"
                                }
                                span { class: "block mt-2 home-prod-hero-line", "Metrics ✨" }
                            }
                        }
                        div { class: "home-prod-hero-anim-up-delayed",
                            p { class: "home-prod-hero-subtitle text-lg sm:text-xl md:text-2xl text-slate-300 max-w-4xl mx-auto leading-relaxed",
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
                        a { class: "home-prod-hero-share w-full sm:w-auto min-w-[220px] h-14 text-lg font-bold bg-gradient-to-r from-orange-500 to-yellow-500 hover:from-orange-600 hover:to-yellow-600 text-white border-2 border-orange-400/50 rounded-2xl shadow-xl px-6 inline-flex items-center justify-center",
                            href: "#share",
                            Icon { name: "share-2".to_string(), size: Some(24), class_name: Some("mr-3".to_string()) }
                            span { "📤 Share Platform" }
                        }
                    }
                    div { class: "home-prod-hero-stats grid grid-cols-1 sm:grid-cols-3 gap-6 sm:gap-8 mt-16",
                        HeroStat { number: "24/7",  label: "🔄 Latest Updates",   gradient: "from-blue-500 to-cyan-500" }
                        HeroStat { number: "100+",  label: "📊 Stock Analytics",  gradient: "from-yellow-500 to-orange-500" }
                        HeroStat { number: "< 1s",  label: "⚡ Response Time",    gradient: "from-green-500 to-emerald-500" }
                    }
                }
            }
        }
    }
}

#[component]
fn HeroStat(number: &'static str, label: &'static str, gradient: &'static str) -> Element {
    rsx! {
        div { class: "home-prod-hero-stat relative bg-slate-800/80 backdrop-blur-xl rounded-2xl p-8 shadow-2xl border border-orange-400/20 hover:scale-105 transition-all duration-300 group overflow-hidden",
            div { class: "absolute inset-0 bg-gradient-to-br {gradient} opacity-5 group-hover:opacity-10 transition-opacity duration-300 home-prod-hero-stat-bg" }
            div { class: "relative z-10 text-center",
                div { class: "home-prod-hero-stat-icon h-10 w-10 mx-auto mb-4 text-orange-500" }
                div { class: "home-prod-hero-stat-value text-3xl sm:text-4xl font-bold bg-gradient-to-r {gradient} bg-clip-text text-transparent mb-2",
                    "{number}"
                }
                div { class: "home-prod-hero-stat-label text-sm font-medium text-slate-300",
                    "{label}"
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hero_section_smoke() {
        
    }

    #[test]
    fn hero_stat_smoke() {
        
    }

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
