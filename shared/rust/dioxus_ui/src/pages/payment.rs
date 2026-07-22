//! `/payment` and `/payment/:type/:id` — fail-closed payment surfaces.
//!
//! The frontend does not own payment intent, price, token, entitlement, or
//! transaction state. Until the backend contracts for those values and their
//! authenticated mutation flow are available, both routes remain read-only.

use super::{PageContext, PageMeta};
use crate::layout::{main_layout::MainLayout, PageHeader};
use crate::primitives::Icon;
use dioxus::prelude::*;

const MAX_ROUTE_CONTEXT_BYTES: usize = 128;
const MAX_ROUTE_CONTEXT_CHARS: usize = 64;

#[derive(Clone, Debug, PartialEq, Eq)]
struct PaymentRouteContext {
    payment_type: Option<String>,
    reference: Option<String>,
}

/// Render the legacy payment entry point without forwarding request data or
/// implying that a payment intent, price, token, or current access exists.
pub fn render(ctx: &PageContext) -> (PageMeta, Element) {
    let meta = PageMeta::app("Payment unavailable");
    (
        meta,
        rsx! { PaymentUnavailablePage { ctx: ctx.clone(), route_context: None } },
    )
}

/// Render the dynamic payment route. Route segments are display-only context:
/// they are bounded, control-free, escaped by Dioxus, and never used to select
/// a redirect, form target, price, token, entitlement, or payment state.
pub fn render_dynamic(ctx: &PageContext) -> (PageMeta, Element) {
    let meta = PageMeta::app("Payment unavailable");
    let route_context = PaymentRouteContext {
        payment_type: bounded_route_context(ctx.params.get("type")),
        reference: bounded_route_context(ctx.params.get("id")),
    };
    let route_context = (route_context.payment_type.is_some() || route_context.reference.is_some())
        .then_some(route_context);

    (
        meta,
        rsx! { PaymentUnavailablePage { ctx: ctx.clone(), route_context } },
    )
}

fn bounded_route_context(value: Option<&String>) -> Option<String> {
    let value = value?.trim();
    if value.is_empty()
        || value.len() > MAX_ROUTE_CONTEXT_BYTES
        || value.chars().count() > MAX_ROUTE_CONTEXT_CHARS
        || !value.is_ascii()
        || value.chars().any(char::is_control)
    {
        return None;
    }
    Some(value.to_string())
}

#[component]
fn PaymentUnavailablePage(ctx: PageContext, route_context: Option<PaymentRouteContext>) -> Element {
    rsx! {
        MainLayout { ctx,
            PaymentUnavailableContent { route_context }
        }
    }
}

#[component]
fn PaymentUnavailableContent(route_context: Option<PaymentRouteContext>) -> Element {
    rsx! {
        div { class: "container page-content max-w-4xl payment-unavailable-page",
            PageHeader {
                title: "Payment flow unavailable".to_string(),
                description: Some("Checkout is temporarily unavailable. No payment will be created from this page.".to_string()),
                icon: Some("alert-triangle".to_string())
            }

            section {
                class: "card card-glass overflow-hidden",
                "aria-labelledby": "payment-unavailable-title",
                role: "alert",
                div { class: "h-1.5 bg-gradient-to-r from-orange-500 via-pink-500 to-purple-600" }
                div { class: "card-body space-y-6",
                    div { class: "flex flex-col sm:flex-row sm:items-start gap-4",
                        div { class: "h-14 w-14 shrink-0 rounded-2xl bg-warning/10 text-warning flex items-center justify-center",
                            Icon { name: "alert-triangle".to_string(), size: Some(28) }
                        }
                        div { class: "space-y-2",
                            div { class: "flex flex-wrap items-center gap-2",
                                h2 {
                                    id: "payment-unavailable-title",
                                    class: "text-xl font-bold",
                                    "Checkout is not available right now"
                                }
                                span { class: "badge badge-warning", "Unavailable" }
                            }
                            p { class: "text-muted-foreground",
                                "This page has not loaded or created a payment intent. No amount, asset, account access, or transaction result is asserted here."
                            }
                        }
                    }

                    if let Some(context) = route_context {
                        div {
                            class: "rounded-xl border border-border bg-muted/40 p-4 space-y-3",
                            role: "note",
                            h3 { class: "font-semibold", "Unverified route context" }
                            p { class: "text-sm text-muted-foreground",
                                "These route labels are shown only to help you identify the link you followed. They do not confirm that a payment or intent exists."
                            }
                            dl { class: "grid grid-cols-1 sm:grid-cols-2 gap-3 text-sm",
                                if let Some(payment_type) = context.payment_type {
                                    div { class: "rounded-lg bg-background p-3",
                                        dt { class: "text-muted-foreground", "Requested type" }
                                        dd { class: "mt-1 font-mono break-all", "{payment_type}" }
                                    }
                                }
                                if let Some(reference) = context.reference {
                                    div { class: "rounded-lg bg-background p-3",
                                        dt { class: "text-muted-foreground", "Requested reference" }
                                        dd { class: "mt-1 font-mono break-all", "{reference}" }
                                    }
                                }
                            }
                        }
                    }

                    div { class: "grid grid-cols-1 md:grid-cols-3 gap-3",
                        PaymentBoundaryItem {
                            icon: "database".to_string(),
                            title: "Verified checkout details".to_string(),
                            body: "Amount, asset, and recipient must come from a verified checkout.".to_string()
                        }
                        PaymentBoundaryItem {
                            icon: "wallet".to_string(),
                            title: "Wallet confirmation".to_string(),
                            body: "A wallet submission alone cannot assert payment completion.".to_string()
                        }
                        PaymentBoundaryItem {
                            icon: "check-circle".to_string(),
                            title: "Confirmed payment".to_string(),
                            body: "Access changes require a verified completed payment.".to_string()
                        }
                    }

                    nav {
                        class: "flex flex-col sm:flex-row gap-3 pt-2",
                        "aria-label": "Payment alternatives",
                        a { class: "btn btn-primary", href: "/plans",
                            Icon { name: "layers".to_string(), size: Some(16) }
                            " Browse plans"
                        }
                        a { class: "btn btn-outline", href: "/account",
                            Icon { name: "user".to_string(), size: Some(16) }
                            " Return to account"
                        }
                        a { class: "btn btn-ghost", href: "/",
                            Icon { name: "home".to_string(), size: Some(16) }
                            " Go home"
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn PaymentBoundaryItem(icon: String, title: String, body: String) -> Element {
    rsx! {
        div { class: "rounded-xl border border-border p-4",
            div { class: "flex items-center gap-2 font-semibold",
                Icon { name: icon, size: Some(18) }
                "{title}"
            }
            p { class: "mt-2 text-sm text-muted-foreground", "{body}" }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn page_ctx(path: &str) -> PageContext {
        PageContext {
            path: path.to_string(),
            ..Default::default()
        }
    }

    fn render_content(route_context: Option<PaymentRouteContext>) -> String {
        dioxus_ssr::render_element(rsx! { PaymentUnavailableContent { route_context } })
    }

    fn assert_no_payment_mutation(html: &str) {
        for forbidden in [
            "<form",
            "<input",
            "<select",
            "<button",
            "onclick=",
            "location.href",
            "/api/v1/payments/confirm",
            "payments:read",
            "29.00",
            "USDT",
            "Current plan",
            "Payment submitted",
            "Payment successful",
            "pay.epsx.io",
        ] {
            assert!(
                !html.contains(forbidden),
                "unsafe payment output: {forbidden}"
            );
        }
    }

    fn assert_fail_closed_content(html: &str) {
        assert!(!html.contains("<script"));
        assert_no_payment_mutation(html);
    }

    #[test]
    fn payment_entry_is_read_only_and_ignores_query_data() {
        let mut ctx = page_ctx("/payment");
        ctx.query = "amount=999&token=EVIL&next=https://attacker.invalid".to_string();
        let (_, element) = render(&ctx);
        let html = dioxus_ssr::render_element(element);

        assert!(html.contains("Payment flow unavailable"));
        assert!(html.contains("role=\"alert\""));
        assert!(html.contains("href=\"/plans\""));
        assert!(html.contains("href=\"/account\""));
        assert!(!html.contains("attacker.invalid"));
        assert!(!html.contains("amount=999"));
        assert!(!html.contains("token=EVIL"));
        assert_no_payment_mutation(&html);

        let content = render_content(None);
        assert_fail_closed_content(&content);
    }

    #[test]
    fn dynamic_context_is_bounded_and_html_escaped() {
        let context = PaymentRouteContext {
            payment_type: bounded_route_context(Some(
                &"plan<script>alert('x')</script>".to_string(),
            )),
            reference: bounded_route_context(Some(&"ref<&\"'42".to_string())),
        };
        let html = render_content(Some(context));

        assert!(html.contains("plan&#60;script&#62;alert(&#39;x&#39;)&#60;/script&#62;"));
        assert!(html.contains("ref&#60;&#38;&#34;&#39;42"));
        assert!(!html.contains("<script>alert('x')</script>"));
        assert!(html.contains("do not confirm that a payment or intent exists"));
        assert_fail_closed_content(&html);

        let overlong = "x".repeat(MAX_ROUTE_CONTEXT_CHARS + 1);
        assert_eq!(bounded_route_context(Some(&overlong)), None);
        assert_eq!(
            bounded_route_context(Some(&"safe\nunsafe".to_string())),
            None
        );
        assert_eq!(
            bounded_route_context(Some(&"bidirectional-\u{202e}value".to_string())),
            None
        );
    }

    #[test]
    fn dynamic_route_does_not_require_an_invented_frontend_permission() {
        let mut ctx = page_ctx("/payment/plan/reference");
        ctx.params.insert("type".to_string(), "plan".to_string());
        ctx.params.insert("id".to_string(), "reference".to_string());
        let (_, element) = render_dynamic(&ctx);
        let html = dioxus_ssr::render_element(element);

        assert!(html.contains("Checkout is not available right now"));
        assert!(html.contains("Requested type"));
        assert!(html.contains("plan"));
        assert!(!html.contains("Permission required"));
        assert_no_payment_mutation(&html);
    }
}
