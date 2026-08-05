//! `/payment` and `/payment/:type/:id` — fail-closed payment surfaces.
//!
//! The frontend does not own payment intent, price, token, entitlement, or
//! transaction state. Until the backend contracts for those values and their
//! authenticated mutation flow are available, both routes remain read-only.

use super::{PageContext, PageMeta};
use crate::auth::GlobalAuthGuard;
use crate::layout::main_layout::MainLayout;
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
    let meta = PageMeta::app("Payment");
    (
        meta,
        rsx! { PaymentUnavailablePage { ctx: ctx.clone(), route_context: None } },
    )
}

/// Render the dynamic payment route. Route segments are display-only context:
/// they are bounded, control-free, escaped by Dioxus, and never used to select
/// a redirect, form target, price, token, entitlement, or payment state.
pub fn render_dynamic(ctx: &PageContext) -> (PageMeta, Element) {
    let meta = PageMeta::app("Payment");
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
    let user_authenticated = ctx.user.is_some();
    rsx! {
        MainLayout { ctx,
            if user_authenticated {
                PaymentUnavailableContent { route_context }
            } else {
                PaymentAuthRequiredContent {}
            }
        }
    }
}

/// The source payment page stops before rendering any checkout copy when no
/// verified session is available. Keep the route's decorative shell, then
/// let the shared auth guard own the sign-in modal.
#[component]
fn PaymentAuthRequiredContent() -> Element {
    rsx! {
        div {
            class: "payment-auth-required relative flex min-h-screen items-center justify-center overflow-hidden bg-gradient-to-br from-purple-50 via-indigo-50 to-blue-50 dark:from-gray-900 dark:via-purple-900/20 dark:to-gray-800",
            "data-payment-state": "auth-required",
            div { class: "pointer-events-none absolute inset-0",
                div { class: "absolute left-10 top-10 h-32 w-32 rounded-full bg-gradient-to-br from-purple-400/30 to-indigo-500/30 blur-xl" }
                div { class: "absolute right-20 top-40 h-24 w-24 rounded-full bg-gradient-to-br from-blue-400/30 to-cyan-500/30 blur-xl" }
                div { class: "absolute bottom-20 left-20 h-40 w-40 rounded-full bg-gradient-to-br from-pink-400/30 to-purple-500/30 blur-xl" }
            }
            div { class: "relative z-10 mx-auto w-full max-w-6xl p-6",
                GlobalAuthGuard { user_authenticated: false }
            }
        }
    }
}

#[component]
fn PaymentUnavailableContent(route_context: Option<PaymentRouteContext>) -> Element {
    rsx! {
        div { class: "payment-prod-page relative min-h-screen overflow-hidden px-4 pb-20 sm:px-6",
            style: "background: radial-gradient(circle at 25% 25%, rgba(118,69,217,0.20), transparent 34%), radial-gradient(circle at 75% 70%, rgba(31,199,212,0.12), transparent 35%), linear-gradient(135deg, #111827 0%, #1f1530 55%, #111827 100%);",
            style { "
                .payment-alternatives {{ display: flex; align-items: center; justify-content: center; gap: 0.75rem; margin: 3rem auto 0; flex-wrap: wrap; }}
                .payment-alternatives a {{ text-decoration: none; }}
                .payment-security {{
                    display: grid;
                    grid-template-columns: repeat(3, minmax(0, 1fr));
                    gap: 1rem;
                    width: 100%;
                    padding: 2rem;
                    border: 1px solid rgba(148, 163, 184, 0.22);
                    border-radius: 1rem;
                    background: rgba(15, 23, 42, 0.55);
                    backdrop-filter: blur(12px);
                }}
                .payment-boundary-item {{
                    display: flex;
                    flex-direction: column;
                    gap: 0.75rem;
                    min-width: 0;
                    padding: 1.25rem;
                    border: 1px solid rgba(148, 163, 184, 0.18);
                    border-radius: 1rem;
                    color: #cbd5e1;
                }}
                .payment-boundary-title {{ display: flex; align-items: center; gap: 0.5rem; color: #f8fafc; font-weight: 600; }}
                .payment-boundary-body {{ margin: 0; color: #cbd5e1; font-size: 0.95rem; line-height: 1.55; }}
                @media (max-width: 639px) {{
                    .payment-alternatives {{ flex-direction: column; align-items: stretch; }}
                    .payment-alternatives a:first-child {{ width: 100%; }}
                    .payment-alternatives a:last-child {{ align-self: center; }}
                    .payment-security {{ grid-template-columns: 1fr; padding: 1rem; gap: 1rem; }}
                    .payment-boundary-item {{ padding: 1.25rem; }}
                }}
            " }
            div { class: "relative z-10 mx-auto max-w-6xl py-12",
                header { class: "mx-auto mb-12 max-w-4xl text-center",
                    div { class: "mx-auto mb-6 flex h-20 w-20 items-center justify-center rounded-full bg-gradient-to-br from-purple-500 to-blue-600 text-white shadow-2xl shadow-purple-500/30",
                        Icon { name: "gem".to_string(), size: Some(40) }
                    }
                    h1 { class: "mb-4 bg-gradient-to-r from-purple-500 via-blue-500 to-cyan-400 bg-clip-text text-4xl font-black text-transparent lg:text-5xl",
                        "Choose Your Plan"
                    }
                    p { class: "mx-auto max-w-2xl text-lg text-slate-200",
                        "Unlock powerful analytics, API access, and premium features with blockchain-secured payments"
                    }
                }

                section {
                    class: "mx-auto max-w-lg rounded-2xl border border-red-700/50 bg-slate-800/80 p-8 text-center shadow-xl backdrop-blur-xl",
                    "aria-labelledby": "payment-unavailable-title",
                    role: "alert",
                    div { class: "mx-auto mb-4 flex h-16 w-16 items-center justify-center rounded-full bg-red-500/20 text-red-400",
                        Icon { name: "alert-triangle".to_string(), size: Some(32) }
                    }
                    h2 { id: "payment-unavailable-title", class: "text-xl font-bold text-red-400", "Failed to Load Plans" }
                    p { class: "mt-2 text-base text-slate-300",
                        "fetch failed. Please refresh or try again."
                    }
                    a { class: "mt-4 inline-flex items-center justify-center rounded-lg bg-blue-600 px-6 py-2 font-semibold text-white hover:bg-blue-500", href: "/plans",
                        "Try Again"
                    }
                    p { class: "sr-only",
                        "Payment flow unavailable. Checkout is not available right now. This page has not loaded or created a payment intent. No amount, asset, account access, or transaction result is asserted here."
                    }
                }

                if let Some(context) = route_context {
                    div { class: "sr-only", role: "note",
                        "Unverified route context. These route labels do not confirm that a payment or intent exists."
                        if let Some(payment_type) = context.payment_type { span { " Requested type: {payment_type}." } }
                        if let Some(reference) = context.reference { span { " Requested reference: {reference}." } }
                    }
                }

                div { class: "payment-security mx-auto mt-16 max-w-6xl",
                    PaymentBoundaryItem { icon: "database".to_string(), title: "Verified checkout".to_string(), body: "Details are shown only from a verified checkout response.".to_string() }
                    PaymentBoundaryItem { icon: "wallet".to_string(), title: "Wallet confirmation".to_string(), body: "A wallet submission alone cannot assert completion.".to_string() }
                    PaymentBoundaryItem { icon: "check-circle".to_string(), title: "Access after confirmation".to_string(), body: "Entitlements require a verified completed payment.".to_string() }
                }

                // The source payment composition keeps these recovery links
                // visible below the security boundary. They do not imply a
                // payment result and remain safe while checkout is offline.
                nav { class: "payment-alternatives", "aria-label": "Payment alternatives",
                    a { class: "btn btn-outline", href: "/account", Icon { name: "user".to_string(), size: Some(16) } " Return to account" }
                    a { class: "btn btn-ghost", href: "/", Icon { name: "home".to_string(), size: Some(16) } " Go home" }
                }
            }
        }
    }
}

#[component]
fn PaymentBoundaryItem(icon: String, title: String, body: String) -> Element {
    rsx! {
        div { class: "payment-boundary-item",
            div { class: "payment-boundary-title",
                Icon { name: icon, size: Some(18) }
                "{title}"
            }
            p { class: "payment-boundary-body", "{body}" }
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

    fn signed_in_ctx(path: &str) -> PageContext {
        PageContext {
            user: Some(crate::auth::User {
                id: "payment-user".to_string(),
                address: "0xpayment".to_string(),
                chain_id: "56".to_string(),
                roles: vec!["user".to_string()],
                email: None,
                tier: None,
                permissions: vec![],
                last_login_at: None,
                auth_method: crate::auth::user::AuthMethod::Wallet,
                display_name: None,
            }),
            ..page_ctx(path)
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

    fn assert_no_payment_claims(html: &str) {
        for forbidden in [
            "<form",
            "<input",
            "<select",
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
                "unauthenticated payment output contains checkout state: {forbidden}"
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

        assert!(html.contains("data-payment-state=\"auth-required\""));
        assert!(html.contains("frontend-auth-gate"));
        assert!(
            !html.contains("<main"),
            "payment body must not nest the shared shell main landmark"
        );
        assert!(!html.contains("Choose Your Plan"));
        assert!(!html.contains("Failed to Load Plans"));
        assert!(!html.contains("attacker.invalid"));
        assert!(!html.contains("amount=999"));
        assert!(!html.contains("token=EVIL"));
        assert_no_payment_claims(&html);

        let (_, authenticated_element) = render(&signed_in_ctx("/payment"));
        let authenticated_html = dioxus_ssr::render_element(authenticated_element);
        assert!(authenticated_html.contains("Payment flow unavailable"));
        assert!(authenticated_html.contains("role=\"alert\""));
        assert!(authenticated_html.contains("href=\"/plans\""));
        assert!(authenticated_html.contains("href=\"/account\""));
        assert!(authenticated_html.contains("payment-alternatives"));
        assert_no_payment_mutation(&authenticated_html);

        assert_fail_closed_content(&render_content(None));
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
        let mut ctx = signed_in_ctx("/payment/plan/reference");
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
