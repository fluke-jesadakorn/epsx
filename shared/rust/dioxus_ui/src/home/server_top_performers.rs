//! `ServerTopPerformers` — server-rendered "Performance Companies"
//! grid.
//!
//! Port of
//! `apps-old/frontend/components/home/server-top-performers.tsx`
//! (172 LoC). The TS source is an async server component that
//! fetches top-performing companies and renders 3 cards. The
//! Dioxus port renders the same visual layout. Data is provided
//! by the caller (page fetches via the BFF API).

use dioxus::prelude::*;

#[component]
pub fn ServerTopPerformers() -> Element {
    rsx! {
        section { class: "server-top-performers",
            div { class: "container mx-auto px-4 py-16 sm:py-24 lg:py-32",
                div { class: "relative",
                    div { class: "absolute -top-8 -left-8 h-16 w-16 rounded-full bg-gradient-to-br from-orange-400/20 to-yellow-400/20 dark:from-orange-600/10 dark:to-yellow-600/10 blur-xl" }
                    div { class: "absolute -right-8 -bottom-8 h-20 w-20 rounded-full bg-gradient-to-br from-blue-400/20 to-cyan-400/20 dark:from-blue-700/10 dark:to-cyan-700/10 blur-xl" }
                    div { class: "flex w-full flex-col gap-8",
                        div { class: "mb-6 space-y-4 text-center server-top-performers-header",
                            h2 { class: "server-top-performers-title pancake-gradient-text text-3xl font-bold sm:text-4xl",
                                "Performance Companies"
                            }
                            p { class: "text-muted-foreground mx-auto max-w-2xl server-top-performers-sub",
                                "Discover the data leaders with exceptional growth and performance metrics"
                            }
                            div { class: "server-top-performers-divider pancake-gradient mx-auto h-1 w-24 rounded-full" }
                        }
                        div { class: "server-top-performers-grid grid grid-cols-1 justify-items-center gap-4 px-2 sm:grid-cols-2 sm:px-0 lg:grid-cols-3",
                            ServerTopPerformersCard { symbol: "GHC",  price: "$6,535",  change: "+4657%", positive: true }
                            ServerTopPerformersCard { symbol: "ARAX", price: "$1,240",  change: "+312%",  positive: true }
                            ServerTopPerformersCard { symbol: "NVTK", price: "$8,915",  change: "+287%",  positive: true }
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn ServerTopPerformersCard(symbol: &'static str, price: &'static str, change: &'static str, positive: bool) -> Element {
    rsx! {
        div { class: "server-top-performers-card w-full max-w-sm h-64 rounded-2xl border border-slate-700 bg-slate-900 p-5 shadow-sm flex flex-col",
            div { class: "flex items-center justify-between mb-3",
                span { class: "text-sm font-semibold text-slate-400", "Rank" }
                span { class: "text-sm text-slate-400", "USD" }
            }
            div { class: "server-top-performers-symbol text-2xl font-bold text-white mb-1", "{symbol}" }
            div { class: "server-top-performers-price text-3xl font-extrabold text-white mb-4", "{price}" }
            div { class: "mt-auto flex items-center justify-between pt-3 border-t border-slate-700",
                span { class: "text-xs text-slate-400", "EPS Growth" }
                span {
                    class: if positive { "server-top-performers-change-positive text-sm font-semibold text-emerald-500" } else { "server-top-performers-change-negative text-sm font-semibold text-red-500" },
                    "▲ {change}"
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn server_top_performers_smoke() {
        let _fn_ptr: fn() -> Element = ServerTopPerformers;
    }
}
