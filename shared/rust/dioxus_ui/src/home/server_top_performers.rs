//! `ServerTopPerformers` — truthful unavailable state for the market preview.
//!
//! Port of
//! The legacy component rendered hard-coded market rows. Ranking data is now
//! backend-owned, so this compatibility component must not manufacture rows
//! until a verified ranking DTO is available.

use dioxus::prelude::*;

#[component]
pub fn ServerTopPerformers() -> Element {
    rsx! {
        section {
            class: "server-top-performers",
            "data-market-state": "unavailable",
            div { class: "container mx-auto px-4 py-16 sm:py-24 lg:py-32",
                div { class: "relative",
                    div { class: "flex w-full flex-col gap-8 text-center",
                        h2 { class: "server-top-performers-title text-3xl font-bold sm:text-4xl",
                            "Market analytics unavailable"
                        }
                        p { class: "text-muted-foreground mx-auto max-w-2xl server-top-performers-sub",
                            "No ranking records are shown until the backend returns a verified market response."
                        }
                        a {
                            class: "mx-auto inline-flex items-center rounded-xl border border-cyan-400/30 px-5 py-3 font-semibold text-cyan-300",
                            href: "/analytics",
                            "Open analytics"
                        }
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
    fn server_top_performers_smoke() {
        let _fn_ptr: fn() -> Element = ServerTopPerformers;
    }
}
