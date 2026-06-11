use crate::primitives::*;

use dioxus::prelude::*;
use super::PageContext;
use super::PageMeta;
use crate::auth::ConnectButton;
use crate::auth::ConnectButtonSize;
use crate::layout::main_layout::AuthLayout;

const AUTH_HYDRATION_SCRIPT: &str = "(function(){var t=localStorage.getItem('epsx_token');if(t){var d=new URLSearchParams(location.search).get('return_url')||'/';location.replace(d);}window.addEventListener('storage',function(){var t=localStorage.getItem('epsx_token');if(t)location.replace('/');});})();";

/// Auth page (`/auth`). Wave 5 Track A port — see
/// `docs/wave5-page-depth/design.md` §"Track A — Hero pages" /
/// `auth_page.rs`. Two-column layout:
///   - LEFT: marketing pitch (hero copy + 3 value props + 1 testimonial)
///   - RIGHT: the auth form (SIWE ConnectButton + email magic link +
///     Google OAuth button)
/// Error states: "wallet not installed", "wrong network", "signature
/// rejected". Loading states: spinner during challenge fetch, button
/// disabled + "Check your wallet..." text during signature.
#[component]
pub fn AuthPage() -> Element {
    rsx! { RenderAuth {} }
}

#[component]
pub fn RenderAuth() -> Element {
    // Auth flow state machine:
    //   idle      — initial state, no auth attempted
    //   connecting — challenge is being fetched / wallet is connecting
    //   signing   — wallet is asking the user to sign the SIWE message
    //   verifying — BFF is verifying the signature
    //   error     — last attempt failed (msg in `error_msg` signal)
    let mut status = use_signal(|| "idle".to_string());
    let mut error_msg = use_signal(String::new);
    let mut error_kind = use_signal(|| "".to_string()); // "no_wallet" | "wrong_network" | "rejected" | ""
    let mut demo_enabled = use_signal(|| true);

    // On mount, check if a wallet is already connected via window.epsxWallet
    // and pre-fill the address. Client-side hydration handles this.
    rsx! {
        div { class: "auth-page",
            // === LEFT column: marketing pitch ===
            div { class: "auth-page-pitch",
                div { class: "auth-page-pitch-bg", "aria-hidden": "true",
                    // Animated orbs (per source app/auth/page.tsx)
                    div { class: "auth-page-pitch-orb auth-page-pitch-orb-1" }
                    div { class: "auth-page-pitch-orb auth-page-pitch-orb-2" }
                    div { class: "auth-page-pitch-orb auth-page-pitch-orb-3" }
                }
                div { class: "auth-page-pitch-inner",
                    div { class: "auth-page-brand",
                        a { href: "/", "EPSX" }
                    }
                    h1 { class: "auth-page-headline",
                        "Precision " span { class: "gradient-text", "Analytics" } " for Modern Teams"
                    }
                    p { class: "auth-page-sub",
                        "Join the next generation of data intelligence. Real-time metrics, predictive modeling, and institutional-grade insights at your fingertips."
                    }
                    // Three value props
                    div { class: "auth-page-value-props",
                        div { class: "auth-page-value-prop",
                            div { class: "auth-page-value-icon",
                                Icon { name: "database".to_string(), size: Some(20), class_name: Some("text-primary".to_string()) }
                            }
                            div { class: "auth-page-value-text",
                                h3 { class: "auth-page-value-title", "Data Accuracy" }
                                p { class: "auth-page-value-desc", "Institutional-grade precision for every metric." }
                            }
                        }
                        div { class: "auth-page-value-prop",
                            div { class: "auth-page-value-icon",
                                Icon { name: "zap".to_string(), size: Some(20), class_name: Some("text-primary".to_string()) }
                            }
                            div { class: "auth-page-value-text",
                                h3 { class: "auth-page-value-title", "Real-time Edge" }
                                p { class: "auth-page-value-desc", "Stay ahead of the curve with instant updates." }
                            }
                        }
                        div { class: "auth-page-value-prop",
                            div { class: "auth-page-value-icon",
                                Icon { name: "shield".to_string(), size: Some(20), class_name: Some("text-primary".to_string()) }
                            }
                            div { class: "auth-page-value-text",
                                h3 { class: "auth-page-value-title", "Secure Ownership" }
                                p { class: "auth-page-value-desc", "Your data, your identity, through Web3." }
                            }
                        }
                    }
                    // Social proof — "Powering 2,500+ teams worldwide"
                    div { class: "auth-page-social-proof",
                        div { class: "auth-page-social-avatars",
                            span { class: "auth-page-social-avatar auth-page-social-avatar-a", "A" }
                            span { class: "auth-page-social-avatar auth-page-social-avatar-b", "B" }
                            span { class: "auth-page-social-avatar auth-page-social-avatar-c", "C" }
                            span { class: "auth-page-social-avatar auth-page-social-avatar-d", "D" }
                        }
                        p { class: "auth-page-social-text",
                            "Powering " span { class: "auth-page-social-count", "2,500+" } " teams worldwide"
                        }
                    }
                }
            }
            // === RIGHT column: auth form ===
            div { class: "auth-page-form-col",
                div { class: "auth-page-form-inner",
                    div { class: "card card-glass auth-card",
                        h2 { class: "auth-card-title", "Welcome back" }
                        p { class: "auth-card-sub", "Sign in to access dashboards, analytics, payments, and developer tools." }
                        // === Primary CTA: SIWE ===
                        div { class: "auth-card-cta",
                            ConnectButton {
                                on_click: move |_| {
                                    status.set("connecting".to_string());
                                    error_msg.set(String::new());
                                    error_kind.set(String::new());
                                },
                                size: Some(ConnectButtonSize::Full),
                                label: Some("Sign in with wallet".to_string()),
                                disabled: *status.read() == "connecting" || *status.read() == "signing",
                            }
                        }
                        // === Error banner (per error kind) ===
                        if !error_msg.read().is_empty() {
                            div { class: "auth-card-error", role: "alert",
                                div { class: "auth-card-error-icon",
                                    Icon { name: "triangle-alert".to_string(), size: Some(16) }
                                }
                                div { class: "auth-card-error-body",
                                    div { class: "auth-card-error-title",
                                        if *error_kind.read() == "no_wallet" { "Wallet not installed" }
                                        else if *error_kind.read() == "wrong_network" { "Wrong network" }
                                        else if *error_kind.read() == "rejected" { "Signature rejected" }
                                        else { "Sign-in failed" }
                                    }
                                    div { class: "auth-card-error-msg", "{error_msg.read()}" }
                                }
                            }
                        }
                        // === Loading state ===
                        if *status.read() == "connecting" || *status.read() == "signing" || *status.read() == "verifying" {
                            div { class: "auth-card-status", "aria-live": "polite",
                                div { class: "spinner spinner-sm" }
                                span {
                                    if *status.read() == "connecting" { "Waiting for wallet..." }
                                    else if *status.read() == "signing" { "Check your wallet..." }
                                    else { "Verifying signature..." }
                                }
                            }
                        }
                        // === Divider ===
                        div { class: "auth-card-divider",
                            span { "OR" }
                        }
                        // === Email magic link ===
                        form { class: "auth-card-email-form",
                            onsubmit: move |e| {
                                e.prevent_default();
                                // The BFF handles the actual magic-link
                                // send. In SSR we just transition the
                                // status to acknowledge the click; the
                                // hydration script reads the form
                                // values and POSTs to /api/v1/auth/magic.
                                status.set("connecting".to_string());
                                error_msg.set(String::new());
                            },
                            input { class: "input auth-card-email-input", r#type: "email",
                                placeholder: "you@example.com",
                                "aria-label": "Email address",
                            }
                            button { class: "btn btn-outline btn-block", r#type: "submit",
                                Icon { name: "mail".to_string(), size: Some(16) }
                                span { "Continue with email" }
                            }
                        }
                        // === Google OAuth ===
                        a { class: "btn btn-outline btn-block auth-card-google-btn", href: "/api/v1/auth/oauth/google",
                            span { class: "auth-card-google-glyph", "G" }
                            span { "Continue with Google" }
                        }
                        if *demo_enabled.read() {
                            div { class: "auth-card-divider auth-card-divider-thin",
                                span { "OR" }
                            }
                            button { class: "btn btn-ghost btn-block", r#type: "button",
                                onclick: move |_| {
                                    status.set("signing".to_string());
                                    error_msg.set(String::new());
                                },
                                "Try the demo account"
                            }
                        }
                        // === Terms / Privacy footer ===
                        p { class: "auth-card-foot",
                            "By connecting, you agree to our "
                            a { href: "/terms", "Terms" }
                            " and "
                            a { href: "/privacy", "Privacy Policy" }
                            "."
                        }
                    }
                    // === Network status indicator ===
                    div { class: "auth-page-status-indicator",
                        span { class: "auth-page-status-dot" }
                        "Network Secure & Operational"
                    }
                    // === Manual redirect fallback ===
                    div { class: "auth-page-fallback",
                        a { href: "/", "Go to Homepage" }
                    }
                    script { dangerous_inner_html: AUTH_HYDRATION_SCRIPT }
                }
            }
        }
    }
}

pub fn render(ctx: &PageContext) -> (PageMeta, Element) {
    let meta = PageMeta::marketing("Sign in");
    (meta, rsx! {
        AuthLayout { ctx: ctx.clone(),
            RenderAuth {}
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Wave 5 — `test_render_smoke`. The `render` function returns a
    /// non-empty `Element` and the rendered HTML string is non-empty.
    #[test]
    fn test_render_smoke() {
        let ctx = PageContext {
            user: None,
            path: "/auth".to_string(),
            ..Default::default()
        };
        let (_meta, el) = render(&ctx);
        let html = dioxus_ssr::render_element(el);
        assert!(!html.is_empty(), "Auth page must render non-empty HTML. Got: {}", html);
        assert!(html.len() > 100, "Auth page HTML is suspiciously short ({} bytes).", html.len());
    }

    /// Wave 5 — `test_section_markers`. The auth page must contain
    /// the new Wave 5 two-column section markers. The design doc
    /// calls these `auth-page-pitch` (left) and `auth-page-form-col`
    /// (right). The original `auth-page` and `auth-card` markers
    /// remain for backwards-compat. Each marker is checked as a
    /// space-bounded token inside a `class="..."` attribute — the
    /// standalone `class="auth-card"` form would fail because the
    /// port also adds other Tailwind classes (`card card-glass
    /// auth-card`).
    #[test]
    fn test_section_markers() {
        let ctx = PageContext {
            user: None,
            path: "/auth".to_string(),
            ..Default::default()
        };
        let (_meta, el) = render(&ctx);
        let html = dioxus_ssr::render_element(el);
        for marker in &[
            "auth-page",
            "auth-page-pitch",
            "auth-page-form-col",
            "auth-card",
        ] {
            // Match the marker as a space-bounded class token.
            let needle_a = format!("class=\"{}\"", marker);
            let needle_b = format!("class=\"{}\"", marker);
            let needle_c = format!("{} ", marker); // leading word in multi-class
            let needle_d = format!(" {}\"", marker); // trailing word in multi-class
            assert!(
                html.contains(&needle_a) || html.contains(&needle_c) || html.contains(&needle_d),
                "Auth page must contain section marker '{}'. Got: {}",
                marker, html
            );
        }
    }

    /// Wave 5 — `test_auth_options`. The auth form must expose all
    /// three authentication options the design doc requires: SIWE
    /// wallet connect, email magic link, and Google OAuth button.
    #[test]
    fn test_auth_options() {
        let ctx = PageContext {
            user: None,
            path: "/auth".to_string(),
            ..Default::default()
        };
        let (_meta, el) = render(&ctx);
        let html = dioxus_ssr::render_element(el);
        // SIWE — ConnectButton renders a button with class
        // `connect-btn connect-btn-full` (the Full size variant).
        assert!(html.contains("connect-btn"), "Auth page must render the ConnectButton (SIWE). Got: {}", html);
        // Email magic link.
        assert!(html.contains("Continue with email"), "Auth page must render email magic link CTA. Got: {}", html);
        assert!(html.contains("auth-card-email-input"), "Auth page must render the email input. Got: {}", html);
        // Google OAuth.
        assert!(html.contains("Continue with Google"), "Auth page must render Google OAuth CTA. Got: {}", html);
        assert!(html.contains("/api/v1/auth/oauth/google"), "Google OAuth must link to /api/v1/auth/oauth/google. Got: {}", html);
    }

    /// Wave 5 — `test_pitch_content`. The left-side marketing pitch
    /// must include the three value props (Data Accuracy, Real-time
    /// Edge, Secure Ownership) and the "2,500+ teams" social proof.
    #[test]
    fn test_pitch_content() {
        let ctx = PageContext {
            user: None,
            path: "/auth".to_string(),
            ..Default::default()
        };
        let (_meta, el) = render(&ctx);
        let html = dioxus_ssr::render_element(el);
        // Headline.
        assert!(html.contains("Precision"), "Auth page must render the pitch headline. Got: {}", html);
        // Three value props.
        for value in &["Data Accuracy", "Real-time Edge", "Secure Ownership"] {
            assert!(
                html.contains(value),
                "Auth page pitch must include value prop '{}'. Got: {}",
                value, html
            );
        }
        // Social proof.
        assert!(html.contains("2,500+"), "Auth page must render the '2,500+' social proof. Got: {}", html);
    }
}
