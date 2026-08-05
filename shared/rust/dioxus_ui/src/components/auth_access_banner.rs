//! Source-shaped sign-in prompt used on public analytics surfaces.
//!
//! This is presentation only: it does not assert a plan, ranking offset, or
//! permission. The destination owns authentication and the backend remains
//! the authority for any access granted after sign-in.

use crate::primitives::Icon;
use dioxus::prelude::*;

#[component]
pub fn AuthAccessBanner(href: String) -> Element {
    rsx! {
        section {
            class: "auth-access-banner mb-6 rounded-2xl border border-purple-500/50 bg-gradient-to-r from-purple-500/10 via-slate-950/40 to-pink-500/10 p-4 backdrop-blur-xl sm:p-5",
            aria_label: "Sign in for analytics access",
            div { class: "flex flex-col gap-4 sm:flex-row sm:items-center sm:justify-between",
                div { class: "flex min-w-0 items-start gap-3",
                    div { class: "flex h-12 w-12 shrink-0 items-center justify-center rounded-xl bg-gradient-to-br from-purple-600 to-pink-500 shadow-lg",
                        Icon { name: "lock".to_string(), size: Some(24), class_name: Some("text-white".to_string()) }
                    }
                    div { class: "min-w-0",
                        h2 { class: "text-base font-semibold text-white", "Unlock Full Analytics Access" }
                        p { class: "mt-1 text-sm text-slate-200", "Sign in to access owner-scoped rankings and premium analytics features." }
                        div { class: "mt-2 flex flex-wrap gap-x-4 gap-y-1 text-xs text-slate-300",
                            span { "↗ Top 100 stock rankings" }
                            span { "▥ Real-time EPS data" }
                            span { "✧ AI-powered insights" }
                        }
                    }
                }
                a {
                    class: "inline-flex shrink-0 items-center justify-center gap-2 rounded-xl bg-gradient-to-r from-purple-500 to-pink-500 px-5 py-3 text-sm font-semibold text-white shadow-lg transition hover:from-purple-600 hover:to-pink-600",
                    href: "{href}",
                    Icon { name: "log-in".to_string(), size: Some(16) }
                    "Sign In Free"
                }
            }
        }
    }
}
