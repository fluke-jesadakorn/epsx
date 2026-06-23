//! `PaySuccessScreen` — `/success` page for pay.epsx.io.
//!
//! wave49(slice-2): replaces the inline `pay_success_body()`
//! HTML string. Renders a green-themed confirmation panel
//! with a "View Dashboard" CTA + "Back to Home" CTA.
//!
//! Composes 2 ported components:
//!   - `CurrentAccessCard` (status: confirmed + active)
//!   - Inline lucide "check" icon for the success badge

use dioxus::prelude::*;

use epsx_dioxus_ui::payment::{CurrentAccessCard, CurrentAccessInfo};

#[component]
pub fn PaySuccessScreen(intent_id: Option<String>) -> Element {
    rsx! {
        div { class: "pay-success-page page-bg",
            section { class: "section",
                style: "display:flex;align-items:center;justify-content:center;min-height:80vh;",
                div { class: "pay-success-card",
                    style: "text-align:center;max-width:32rem;",
                    // Success badge
                    div { class: "pay-success-icon",
                        style: "width:5rem;height:5rem;border-radius:9999px;background:linear-gradient(135deg,#10b981,#34d399);display:flex;align-items:center;justify-content:center;margin:0 auto 1.5rem;box-shadow:0 20px 25px -5px rgba(16,185,129,0.5);",
                        i { class: "epsx-icon", dangerous_inner_html: epsx_templates::lucide("check", "28", "white") }
                    }
                    span { class: "pay-success-badge badge badge-success",
                        style: "margin-bottom:1rem;",
                        "Confirmed"
                    }
                    h1 { class: "pay-success-title",
                        style: "font-size:2.5rem;font-weight:800;margin-bottom:1rem;",
                        "Payment Successful"
                    }
                    p { class: "pay-success-description",
                        style: "color:var(--text-muted);font-size:1.125rem;margin-bottom:2rem;",
                        "Your payment has been confirmed on BSC. The recipient has been notified."
                    }
                    // Show the intent ID if provided (for support)
                    if let Some(id) = intent_id.as_ref() {
                        div { class: "pay-success-intent-id",
                            style: "font-family:monospace;font-size:0.75rem;color:var(--text-subtle);margin-bottom:1.5rem;word-break:break-all;",
                            "Intent: {id}"
                        }
                    }
                    // CTAs
                    div { class: "pay-success-ctas",
                        style: "display:inline-flex;gap:0.75rem;flex-wrap:wrap;justify-content:center;margin-bottom:2rem;",
                        a { class: "pay-success-cta-home btn btn-gradient btn-lg",
                            href: "/",
                            i { class: "epsx-icon", dangerous_inner_html: epsx_templates::lucide("home", "16", "currentColor") }
                            " Back to Home"
                        }
                        a { class: "pay-success-cta-dashboard btn btn-outline btn-lg",
                            href: "/dashboard",
                            i { class: "epsx-icon", dangerous_inner_html: epsx_templates::lucide("gauge", "16", "currentColor") }
                            " View Dashboard"
                        }
                    }
                    // CurrentAccessCard — show updated plan status
                    CurrentAccessCard {
                        info: CurrentAccessInfo {
                            plan_name: "EPSX Pay".to_string(),
                            tier: "Confirmed".to_string(),
                            expires_at: None,
                            renews_at: None,
                            is_active: true,
                        }
                    }
                }
            }
        }
    }
}