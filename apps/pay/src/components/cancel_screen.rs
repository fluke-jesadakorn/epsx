//! `PayCancelScreen` — `/cancel` page for pay.epsx.io.
//!
//! wave49(slice-2): replaces the inline `pay_cancel_body()`
//! HTML string. Renders a red-themed cancellation panel with
//! "Try Again" + "Back to Home" CTAs.

use dioxus::prelude::*;

#[component]
pub fn PayCancelScreen() -> Element {
    rsx! {
        div { class: "pay-cancel-page page-bg",
            section { class: "section",
                style: "display:flex;align-items:center;justify-content:center;min-height:80vh;",
                div { class: "pay-cancel-card",
                    style: "text-align:center;max-width:32rem;",
                    // Cancel badge
                    div { class: "pay-cancel-icon",
                        style: "width:5rem;height:5rem;border-radius:9999px;background:linear-gradient(135deg,#ef4444,#f87171);display:flex;align-items:center;justify-content:center;margin:0 auto 1.5rem;box-shadow:0 20px 25px -5px rgba(239,68,68,0.5);",
                        i { class: "epsx-icon", dangerous_inner_html: epsx_templates::lucide("x", "28", "white") }
                    }
                    span { class: "pay-cancel-badge badge badge-danger",
                        style: "margin-bottom:1rem;",
                        "Cancelled"
                    }
                    h1 { class: "pay-cancel-title",
                        style: "font-size:2.5rem;font-weight:800;margin-bottom:1rem;",
                        "Payment Cancelled"
                    }
                    p { class: "pay-cancel-description",
                        style: "color:var(--text-muted);font-size:1.125rem;margin-bottom:2rem;",
                        "Your payment was not completed. No funds have been transferred."
                    }
                    // CTAs
                    div { class: "pay-cancel-ctas",
                        style: "display:inline-flex;gap:0.75rem;flex-wrap:wrap;justify-content:center;",
                        a { class: "pay-cancel-cta-home btn btn-gradient btn-lg",
                            href: "/",
                            i { class: "epsx-icon", dangerous_inner_html: epsx_templates::lucide("home", "16", "currentColor") }
                            " Back to Home"
                        }
                        button { class: "pay-cancel-cta-retry btn btn-outline btn-lg",
                            // SSR-only render — slice-3 will hydrate
                            // this with a `use_navigator` callback.
                            r#type: "button",
                            onclick: |_| {},
                            i { class: "epsx-icon", dangerous_inner_html: epsx_templates::lucide("arrow-left", "16", "currentColor") }
                            " Try Again"
                        }
                    }
                }
            }
        }
    }
}