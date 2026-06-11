//! /payment + /payment/[type]/[id] — payment flow.

use crate::primitives::*;
use crate::feedback::*;
use crate::stepper::{Stepper, StepPanel, StepNavigation};

use dioxus::prelude::*;
use super::PageContext;
use super::PageMeta;
use crate::layout::main_layout::MainLayout;
use crate::layout::PageHeader;
use crate::auth::AuthGate;

pub fn render(ctx: &PageContext) -> (PageMeta, Element) {
    let meta = PageMeta::app("Payment");
    (meta, rsx! { RenderPayment { ctx: ctx.clone() } })
}

#[component]
fn RenderPayment(ctx: PageContext) -> Element {
    let mut step = use_signal(|| 0usize);
    let mut pay_type = use_signal(|| "subscription".to_string());
    let mut amount = use_signal(|| "29.00".to_string());
    let mut token = use_signal(|| "USDT".to_string());
    let steps = vec![("Method".to_string(), false), ("Details".to_string(), false), ("Confirm".to_string(), false), ("Complete".to_string(), false)];
    rsx! {
        MainLayout { ctx: ctx.clone(),
            AuthGate { user: ctx.user.clone(), feature: Some("payment".to_string()),
                required_permissions: Some(vec!["payments:read".to_string()]),
                return_url: Some(ctx.path.clone()),
                div { class: "container page-content max-w-3xl",
                    PageHeader { title: "Payment".to_string(), description: Some("Send a payment to any merchant or subscription plan".to_string()), icon: Some("credit-card".to_string()) }
                    div { class: "mb-6", Stepper { steps, current: *step.read() } }
                    div { class: "card card-glass", div { class: "card-body",
                        if *step.read() == 0 {
                            StepPanel { title: "Choose payment method".to_string(), description: Some("Select how you want to pay".to_string()),
                                div { class: "grid grid-cols-1 md:grid-cols-2 gap-4",
                                    button { class: "card card-glass p-4 text-left", r#type: "button", onclick: move |_| { pay_type.set("subscription".to_string()); step.set(1); }, div { class: "font-bold", "Subscription" } div { class: "text-sm text-muted-foreground", "Pay a subscription plan" } }
                                    button { class: "card card-glass p-4 text-left", r#type: "button", onclick: move |_| { pay_type.set("one-time".to_string()); step.set(1); }, div { class: "font-bold", "One-time payment" } div { class: "text-sm text-muted-foreground", "Pay a merchant once" } }
                                }
                            }
                        } else if *step.read() == 1 {
                            StepPanel { title: "Payment details".to_string(), description: Some("Enter the amount and token".to_string()),
                                Form { method: "POST".to_string(), action: "/api/v1/payments/confirm".to_string(),
                                    div { class: "field", label { class: "field-label", "Amount" } input { class: "input", name: "amount", r#type: "number", step: "0.01", required: true, value: "{amount.read()}", oninput: move |e| amount.set(e.value().to_string()) } }
                                    div { class: "field", label { class: "field-label", "Token" }
                                        SelectField { name: "token".to_string(), options: vec![("USDT".to_string(), "USDT".to_string()), ("USDC".to_string(), "USDC".to_string()), ("BNB".to_string(), "BNB".to_string())], value: Some(token.read().clone()), required: true, label: None, help: None, error: None, placeholder: None, onchange: None }
                                    }
                                    div { class: "flex justify-between mt-4",
                                        button { class: "btn btn-outline", r#type: "button", onclick: move |_| step.set(0), "Back" }
                                        button { class: "btn btn-primary", r#type: "button", onclick: move |_| step.set(2), "Next" }
                                    }
                                }
                            }
                        } else if *step.read() == 2 {
                            StepPanel { title: "Confirm payment".to_string(), description: Some("Review the details before submitting".to_string()),
                                div { class: "space-y-2",
                                    div { class: "flex justify-between", span { "Type" } span { class: "font-semibold", "{pay_type.read()}" } }
                                    div { class: "flex justify-between", span { "Amount" } span { class: "font-mono font-bold", "{amount.read()} {token.read()}" } }
                                    div { class: "flex justify-between", span { "Network fee" } span { class: "font-mono text-muted-foreground", "~0.0001 BNB" } }
                                }
                                div { class: "flex justify-between mt-4",
                                    button { class: "btn btn-outline", r#type: "button", onclick: move |_| step.set(1), "Back" }
                                    button { class: "btn btn-primary", r#type: "button", onclick: move |_| step.set(3), "Submit payment" }
                                }
                            }
                        } else {
                            div { class: "text-center py-8",
                                Icon { name: "check-circle".to_string(), size: Some(64) }
                                h2 { class: "text-2xl font-bold mt-4", "Payment submitted" }
                                p { class: "text-muted-foreground mt-2", "Your payment is being processed on-chain. You will be notified when it confirms." }
                                div { class: "mt-6 flex justify-center gap-2", a { class: "btn btn-primary", href: "/dashboard", "Back to dashboard" } a { class: "btn btn-outline", href: "/account", "View payment history" } }
                            }
                        }
                    } }
                }
            }
        }
    }
}

pub fn render_dynamic(ctx: &PageContext) -> (PageMeta, Element) {
    let meta = PageMeta::app("Payment");
    let ptype = ctx.params.get("type").cloned().unwrap_or_else(|| "subscription".to_string());
    let pid = ctx.params.get("id").cloned().unwrap_or_default();
    (meta, rsx! {
        MainLayout { ctx: ctx.clone(),
            AuthGate { user: ctx.user.clone(), feature: Some("payment".to_string()),
                required_permissions: Some(vec!["payments:read".to_string()]),
                return_url: Some(ctx.path.clone()),
                div { class: "container page-content max-w-2xl",
                    PageHeader { title: format!("{} payment", ptype), description: Some(format!("ID: {}", pid)), icon: Some("credit-card".to_string()) }
                    div { class: "card card-glass",
                        div { class: "card-body",
                            Form { method: "POST".to_string(), action: "/api/v1/payments/confirm".to_string(),
                                input { r#type: "hidden", name: "type", value: "{ptype}" }
                                input { r#type: "hidden", name: "id", value: "{pid}" }
                                div { class: "field",
                                    label { class: "field-label", "Amount" }
                                    input { class: "input", name: "amount", r#type: "number", step: "0.01", required: true, value: "29.00" }
                                }
                                div { class: "field",
                                    label { class: "field-label", "Token" }
                                    SelectField { name: "token".to_string(), options: vec![("USDT".to_string(), "USDT".to_string()), ("USDC".to_string(), "USDC".to_string())], value: Some("USDT".to_string()), required: true, label: None, help: None, error: None, placeholder: None, onchange: None }
                                }
                                button { class: "btn btn-primary btn-block", r#type: "submit", "Pay now" }
                            }
                        }
                    }
                }
            }
        }
    })
}
