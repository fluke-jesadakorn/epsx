use crate::primitives::*;

use super::PageContext;
use super::PageMeta;
use crate::auth::ConnectButton;
use crate::auth::ConnectButtonSize;
use crate::layout::main_layout::AuthLayout;
use crate::theme::UnifiedThemeToggle;
use dioxus::prelude::*;

pub const AUTH_PAGE_SESSION_STATE_PARAM: &str = "auth_page_session_state";
pub const AUTH_PAGE_SESSION_STATE_SIGNED_OUT: &str = "signed_out";
pub const AUTH_PAGE_SESSION_STATE_RECOVERING: &str = "recovering";
pub const AUTH_PAGE_SESSION_STATE_VERIFIER_UNAVAILABLE: &str = "verifier_unavailable";

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum AuthPageSessionState {
    #[default]
    SignedOut,
    Recovering,
    VerifierUnavailable,
}

impl AuthPageSessionState {
    fn from_context(ctx: &PageContext) -> Self {
        match ctx
            .params
            .get(AUTH_PAGE_SESSION_STATE_PARAM)
            .map(String::as_str)
        {
            None | Some(AUTH_PAGE_SESSION_STATE_SIGNED_OUT) => Self::SignedOut,
            Some(AUTH_PAGE_SESSION_STATE_RECOVERING) => Self::Recovering,
            Some(AUTH_PAGE_SESSION_STATE_VERIFIER_UNAVAILABLE) => Self::VerifierUnavailable,
            Some(_) => Self::VerifierUnavailable,
        }
    }

    const fn as_str(self) -> &'static str {
        match self {
            Self::SignedOut => AUTH_PAGE_SESSION_STATE_SIGNED_OUT,
            Self::Recovering => AUTH_PAGE_SESSION_STATE_RECOVERING,
            Self::VerifierUnavailable => AUTH_PAGE_SESSION_STATE_VERIFIER_UNAVAILABLE,
        }
    }
}

/// Auth page (`/auth`). Wave 5 Track A port — see
/// `docs/wave5-page-depth/design.md` §"Track A — Hero pages" /
/// `auth_page.rs`. Two-column layout:
///   - LEFT: marketing pitch (hero copy + 3 value props + 1 testimonial)
///   - RIGHT: the wallet-only SIWE auth form
///
/// Wave 50 — wired up the full SIWE flow:
/// - The `<ConnectButton data_connect_wallet=true>` renders a raw
///   `<button data-connect-wallet="true">` consumed by the generated
///   Rust/WASM browser runtime.
/// - Loading + error banners are rendered statically with stable ids;
///   Rust/WASM updates the live region throughout challenge, signing, and
///   verification. The initial document remains useful without enhancement.
#[component]
pub fn AuthPage() -> Element {
    rsx! { RenderAuth { session_state: AuthPageSessionState::SignedOut } }
}

#[component]
pub fn RenderAuth(session_state: AuthPageSessionState) -> Element {
    // The component is purely declarative — every interactive state
    // (loading / error / success) is driven by the generated Rust/WASM
    // runtime. The SSR document carries the initial idle state.

    rsx! {
        div {
            class: "auth-page",
            "data-auth-session-state": session_state.as_str(),
            "aria-busy": if session_state == AuthPageSessionState::Recovering { "true" } else { "false" },
            // The standalone auth page has no navbar, but the source design
            // still exposes the shared theme control in the upper-right
            // corner. Keep it in the page shell so it remains available on
            // both the desktop marketing layout and compact auth card.
            div { class: "auth-page-theme-toggle", UnifiedThemeToggle {} }
            // The development auth page paints these ambient orbs at the
            // page level, so they remain visible on compact screens where
            // the desktop marketing column is intentionally hidden.
            div { class: "auth-page-background", "aria-hidden": "true",
                div { class: "auth-page-background-orb auth-page-background-orb-1" }
                div { class: "auth-page-background-orb auth-page-background-orb-2" }
                div { class: "auth-page-background-orb auth-page-background-orb-3" }
            }
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
                            a { href: "/",
                                span { class: "auth-brand-icon", aria_hidden: "true",
                                    Icon { name: "cpu".to_string(), size: Some(30), class_name: Some("text-white".to_string()) }
                                }
                                "EPSX"
                            }
                    }
                    h1 { class: "auth-page-headline",
                        span { class: "auth-page-headline-line",
                            "Precision " span { class: "gradient-text", "Analytics" }
                        }
                        br {}
                        span { class: "auth-page-headline-line", "For Modern Teams" }
                    }
                    p { class: "auth-page-sub",
                        "Join the next generation of data intelligence. Real-time metrics, predictive modeling, and institutional-grade insights at your fingertips."
                    }
                    // Four value props (matches the development auth page).
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
                        div { class: "auth-page-value-prop",
                            div { class: "auth-page-value-icon",
                                Icon { name: "globe".to_string(), size: Some(20), class_name: Some("text-primary".to_string()) }
                            }
                            div { class: "auth-page-value-text",
                                h3 { class: "auth-page-value-title", "Global Coverage" }
                                p { class: "auth-page-value-desc", "Comprehensive coverage across all data sources." }
                            }
                        }
                    }
                    // Source social-proof row.
                    div { class: "auth-page-social-proof",
                        div { class: "auth-page-social-avatars",
                            span { class: "auth-page-social-avatar auth-page-social-avatar-a", "A" }
                            span { class: "auth-page-social-avatar auth-page-social-avatar-b", "B" }
                            span { class: "auth-page-social-avatar auth-page-social-avatar-c", "C" }
                            span { class: "auth-page-social-avatar auth-page-social-avatar-d", "D" }
                        }
                        p { class: "auth-page-social-text",
                            "Built for teams using modern data workflows"
                        }
                    }
                }
            }
            // === RIGHT column: auth form ===
            div { class: "auth-page-form-col",
                div { class: "auth-page-form-inner",
                    // Mobile-only heading from the development auth page.
                    div { class: "auth-page-mobile-header",
                        div { class: "auth-page-mobile-brand",
                            a { href: "/",
                                span { class: "auth-brand-icon", aria_hidden: "true",
                                    Icon { name: "cpu".to_string(), size: Some(28), class_name: Some("text-white".to_string()) }
                                }
                                "EPSX"
                            }
                        }
                        h2 { "Welcome Back" }
                        p { "Connect your wallet to access the platform" }
                    }
                    div { class: "card card-glass auth-card",
                        // Wave 49 — Plan 13 (T1) — re-ported dev /auth
                        // to match prod's wallet-only design.
                        //   - Title: "Welcome back" → "Welcome to EPSX"
                        //   - Sub:   "Sign in to access dashboards…" →
                        //            "Secure authentication via Web3"
                        //   - CTA:   "Sign in with wallet" → "Connect Wallet"
                        //   - Removed: OR divider, email form, Google OAuth,
                        //              "Try the demo account" button
                        //   - Added: 3-feature security list
                        //            (Secure Web3 Login Flow / No Account
                        //             Credentials Needed / Decentralized
                        //             Data Privacy)
                    // This single fix repairs 7 routes that all
                        // 307-redirect to /auth when the user is
                        // unauthenticated (UNAUTH_REDIRECT_PATHS in
                        // apps/frontend/src/ssr.rs):
                        //   /about, /contact, /offline, /auth,
                        //   /permissions, /profile, /notifications
                        div { class: "auth-card-mobile-icon", aria_hidden: "true",
                            Icon { name: "lock".to_string(), size: Some(32), class_name: Some("text-primary".to_string()) }
                        }
                        div { class: "auth-card-desktop-heading",
                            h2 { class: "auth-card-title", "Welcome to EPSX" }
                            p { class: "auth-card-sub", "Secure authentication via Web3" }
                        }
                        // === Primary CTA: SIWE (wallet-only) ===
                        // Wave 50 — `data_connect_wallet=true` makes
                        // ConnectButton emit a raw `<button
                        // data-connect-wallet="true">` element. The
                        // page-shell `wallet_shim()` script attaches a
                        // click listener that calls
                        // `window.epsx.connectWallet()` (the full
                        // EIP-4361 challenge → sign → verify flow).
                        // This survives SSR (Dioxus onclick closures
                        // get stripped; data-* + external JS does not).
                        div { class: "auth-card-cta",
                            ConnectButton {
                                size: Some(ConnectButtonSize::Full),
                                label: Some("Connect Wallet".to_string()),
                                disabled: session_state != AuthPageSessionState::SignedOut,
                                data_connect_wallet: Some(true),
                                data_provider: Some("metamask".to_string()),
                            }
                        }
                        div { class: "auth-card-divider auth-card-divider-thin", aria_hidden: "true" }
                        // === Loading state (hidden by default) ===
                        // Rust/WASM toggles `hidden` and updates the message as
                        // SIWE progresses through challenge, signing, and verification.
                        div {
                            id: "auth-card-status",
                            class: "auth-card-status",
                            "data-epsx-runtime-status": "true",
                            role: "status",
                            "aria-live": "polite",
                            hidden: session_state != AuthPageSessionState::Recovering,
                            div { class: "spinner spinner-sm" }
                            span {
                                id: "auth-card-status-msg",
                                if session_state == AuthPageSessionState::Recovering {
                                    "Restoring your session..."
                                } else {
                                    "Waiting for wallet..."
                                }
                            }
                        }
                        // === Error banner (hidden by default) ===
                        div {
                            id: "auth-card-error",
                            class: "auth-card-error",
                            role: "alert",
                            tabindex: "-1",
                            hidden: session_state != AuthPageSessionState::VerifierUnavailable,
                            div { class: "auth-card-error-icon",
                                Icon { name: "triangle-alert".to_string(), size: Some(16) }
                            }
                            div { class: "auth-card-error-body",
                                div {
                                    id: "auth-card-error-title",
                                    class: "auth-card-error-title",
                                    if session_state == AuthPageSessionState::VerifierUnavailable {
                                        "Sign-in temporarily unavailable"
                                    } else {
                                        "Sign-in failed"
                                    }
                                }
                                div {
                                    id: "auth-card-error-msg",
                                    class: "auth-card-error-msg",
                                    if session_state == AuthPageSessionState::VerifierUnavailable {
                                        "We cannot verify your session right now. Please try again later."
                                    } else {
                                        ""
                                    }
                                }
                            }
                        }
                        // === 3-feature security list (prod design) ===
                        ul { class: "auth-card-features", role: "list",
                            li { class: "auth-card-feature",
                                span { class: "auth-card-feature-icon", "✓" }
                                span { "Secure Web3 Login Flow" }
                            }
                            li { class: "auth-card-feature",
                                span { class: "auth-card-feature-icon", "✓" }
                                span { "No Account Credentials Needed" }
                            }
                            li { class: "auth-card-feature",
                                span { class: "auth-card-feature-icon", "✓" }
                                span { "Decentralized Data Privacy" }
                            }
                        }
                        // Mobile source layout exposes the four value props
                        // as compact cards below the wallet benefits.
                        div { class: "auth-card-mobile-features",
                            for (icon, title) in [
                                ("database", "Data Accuracy"),
                                ("zap", "Real-time Edge"),
                                ("shield", "Secure Ownership"),
                                ("globe", "Global Coverage"),
                            ] {
                                div { class: "auth-card-mobile-feature",
                                    div { class: "auth-card-mobile-feature-icon",
                                        Icon { name: icon.to_string(), size: Some(20), class_name: Some("text-primary".to_string()) }
                                    }
                                    h4 { "{title}" }
                                }
                            }
                        }
                        // === Terms / Privacy footer ===
                        p { class: "auth-card-foot",
                            "By connecting, you agree to our "
                            a { href: "/terms", "Terms" }
                            " and "
                            a { href: "/privacy", "Privacy" }
                            "."
                        }
                    }
                    // === Network status indicator ===
                    div { class: "auth-page-status-indicator",
                        span { class: "auth-page-status-dot" }
                        span { class: "auth-page-status-wide", "Wallet-based sign-in" }
                        span { class: "auth-page-status-compact", "Secure Connection" }
                        span { class: "auth-page-status-wallet", "Wallet-based sign-in" }
                    }
                    // === Manual redirect fallback ===
                    div { class: "auth-page-fallback",
                        a { href: "/", "Go to Homepage" }
                    }
                }
            }
        }
    }
}

pub fn render(ctx: &PageContext) -> (PageMeta, Element) {
    let meta = PageMeta::marketing("Sign in");
    let session_state = AuthPageSessionState::from_context(ctx);
    (
        meta,
        rsx! {
            AuthLayout { ctx: ctx.clone(),
                RenderAuth { session_state }
            }
        },
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn render_with_session_state(session_state: Option<&str>) -> String {
        let mut ctx = PageContext {
            user: None,
            path: "/auth".to_string(),
            ..Default::default()
        };
        if let Some(session_state) = session_state {
            ctx.params.insert(
                AUTH_PAGE_SESSION_STATE_PARAM.to_string(),
                session_state.to_string(),
            );
        }
        let (_meta, el) = render(&ctx);
        dioxus_ssr::render_element(el)
    }

    #[test]
    fn auth_page_signed_out_is_actionable() {
        let html = render_with_session_state(Some(AUTH_PAGE_SESSION_STATE_SIGNED_OUT));
        assert!(html.contains("data-auth-session-state=\"signed_out\""));
        assert!(html.contains("data-connect-wallet=\"true\""));
        assert!(!html.contains("disabled=\"true\""));
        assert!(!html.contains("disabled=\"disabled\""));
        assert!(html.contains("Wallet-based sign-in"));
        assert!(html.contains("Secure Connection"));
        assert!(!html.contains("Network Secure"));
        for class in [
            "auth-page-pitch",
            "auth-page-sub",
            "auth-page-value-title",
            "auth-page-value-desc",
            "auth-page-social-text",
            "auth-card-title",
            "auth-card-sub",
            "auth-card-foot",
            "auth-page-status-indicator",
            "auth-page-fallback",
        ] {
            assert!(
                html.contains(&format!("class=\"{class}\"")),
                "missing {class}"
            );
        }
    }

    #[test]
    fn auth_page_recovering_is_announced_and_disables_connect() {
        let html = render_with_session_state(Some(AUTH_PAGE_SESSION_STATE_RECOVERING));
        assert!(html.contains("data-auth-session-state=\"recovering\""));
        assert!(html.contains("aria-busy=\"true\""));
        assert!(html.contains("role=\"status\""));
        assert!(html.contains("class=\"auth-card-status\""));
        assert!(html.contains("Restoring your session..."));
        assert!(html.contains("disabled=\"true\"") || html.contains("disabled=\"disabled\""));
    }

    #[test]
    fn auth_page_verifier_unavailable_is_fixed_and_disables_connect() {
        let html = render_with_session_state(Some(AUTH_PAGE_SESSION_STATE_VERIFIER_UNAVAILABLE));
        assert!(html.contains("data-auth-session-state=\"verifier_unavailable\""));
        assert!(html.contains("class=\"auth-card-error\""));
        assert!(html.contains("class=\"auth-card-error-title\""));
        assert!(html.contains("class=\"auth-card-error-msg\""));
        assert!(html.contains("Sign-in temporarily unavailable"));
        assert!(html.contains("We cannot verify your session right now. Please try again later."));
        assert!(html.contains("disabled=\"true\"") || html.contains("disabled=\"disabled\""));
        for forbidden in ["access_token", "refresh_token", "permission", "plan"] {
            assert!(!html.contains(forbidden));
        }
    }

    #[test]
    fn auth_page_unknown_present_state_fails_closed() {
        let html = render_with_session_state(Some("future-open-state"));
        assert!(html.contains("data-auth-session-state=\"verifier_unavailable\""));
        assert!(html.contains("Sign-in temporarily unavailable"));
        assert!(html.contains("disabled=\"true\"") || html.contains("disabled=\"disabled\""));
        assert!(!html.contains("future-open-state"));
    }

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
        assert!(
            !html.is_empty(),
            "Auth page must render non-empty HTML. Got: {}",
            html
        );
        assert!(
            html.len() > 100,
            "Auth page HTML is suspiciously short ({} bytes).",
            html.len()
        );
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
            let needle_c = format!("{} ", marker); // leading word in multi-class
            let needle_d = format!(" {}\"", marker); // trailing word in multi-class
            assert!(
                html.contains(&needle_a) || html.contains(&needle_c) || html.contains(&needle_d),
                "Auth page must contain section marker '{}'. Got: {}",
                marker,
                html
            );
        }
    }

    /// Wave 5 — `test_auth_options`. The current contract is the
    /// wallet-only design (Welcome to EPSX /
    /// Connect Wallet / 3-feature security list). The 3 auth options
    /// test now asserts the wallet-only CTA + 3-feature security
    /// list, matching the prod baseline PNG.
    #[test]
    fn test_auth_options() {
        let ctx = PageContext {
            user: None,
            path: "/auth".to_string(),
            ..Default::default()
        };
        let (_meta, el) = render(&ctx);
        let html = dioxus_ssr::render_element(el);
        // Primary CTA: SIWE ConnectButton renders as
        // `connect-btn connect-btn-full` (Full size variant).
        assert!(
            html.contains("connect-btn"),
            "Auth page must render the ConnectButton (SIWE). Got: {}",
            html
        );
        assert!(
            html.contains("Connect Wallet"),
            "Auth page must render the wallet-only Connect Wallet CTA. Got: {}",
            html
        );
        // 3-feature security list (matches prod design).
        assert!(
            html.contains("Secure Web3 Login Flow"),
            "Auth page must render 'Secure Web3 Login Flow' feature. Got: {}",
            html
        );
        assert!(
            html.contains("No Account Credentials Needed"),
            "Auth page must render 'No Account Credentials Needed' feature. Got: {}",
            html
        );
        assert!(
            html.contains("Decentralized Data Privacy"),
            "Auth page must render 'Decentralized Data Privacy' feature. Got: {}",
            html
        );
    }

    /// Wave 5 — `test_pitch_content`. The left-side marketing pitch
    /// must include the three value props (Data Accuracy, Real-time
    /// Edge, Secure Ownership) and a non-numeric product-fit statement.
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
        assert!(
            html.contains("Precision"),
            "Auth page must render the pitch headline. Got: {}",
            html
        );
        // Four source value props.
        for value in &[
            "Data Accuracy",
            "Real-time Edge",
            "Secure Ownership",
            "Global Coverage",
        ] {
            assert!(
                html.contains(value),
                "Auth page pitch must include value prop '{}'. Got: {}",
                value,
                html
            );
        }
        // Product fit, without an unsupported numeric customer claim.
        assert!(
            html.contains("Built for teams using modern data workflows"),
            "Auth page must render the truthful product-fit statement. Got: {}",
            html
        );
        assert!(!html.contains("2,500+"));
    }

    // ── Wave 50 — SSR-friendly wallet wiring tests ────────────────

    /// The ConnectButton on the auth page must carry
    /// `data-connect-wallet="true"` so the page-shell `wallet_shim()`
    /// script attaches a click listener that calls
    /// `window.epsx.connectWallet()`. Without this attribute, the
    /// click handler is a Dioxus closure (stripped at SSR time) and
    /// clicking the button is a visual no-op.
    #[test]
    fn test_connect_button_has_data_connect_wallet() {
        let ctx = PageContext {
            user: None,
            path: "/auth".to_string(),
            ..Default::default()
        };
        let (_meta, el) = render(&ctx);
        let html = dioxus_ssr::render_element(el);
        assert!(
            html.contains("data-connect-wallet=\"true\""),
            "Auth page ConnectButton must emit data-connect-wallet=\"true\" so the wallet shim can wire the click. Got: {html}"
        );
    }

    /// The generated Rust/WASM runtime writes status into a stable live region.
    #[test]
    fn test_includes_wallet_runtime_status_contract() {
        let ctx = PageContext {
            user: None,
            path: "/auth".to_string(),
            ..Default::default()
        };
        let (_meta, el) = render(&ctx);
        let html = dioxus_ssr::render_element(el);
        assert!(html.contains("data-epsx-runtime-status=\"true\""));
        assert!(html.contains("aria-live=\"polite\""));
        assert!(!html.contains("<script"));
    }

    /// The auth page must render the loading + error banner elements
    /// with stable ids so the Rust/WASM runtime can update them. Initial
    /// state: `hidden` (idle, no error, no loading).
    #[test]
    fn test_status_and_error_elements_present_with_stable_ids() {
        let ctx = PageContext {
            user: None,
            path: "/auth".to_string(),
            ..Default::default()
        };
        let (_meta, el) = render(&ctx);
        let html = dioxus_ssr::render_element(el);
        for id in &[
            "auth-card-status",
            "auth-card-status-msg",
            "auth-card-error",
            "auth-card-error-title",
            "auth-card-error-msg",
        ] {
            assert!(
                html.contains(&format!("id=\"{id}\"")),
                "Auth page must render element with id=\"{id}\". Got: {html}"
            );
        }
        // Initial state: hidden=true on both banners.
        // We can't easily assert the `hidden` attribute per-element
        // without parsing — but the static text "Sign-in failed"
        // (the default error title) and "Waiting for wallet..." (the
        // default status msg) must both be present.
        assert!(
            html.contains("Waiting for wallet..."),
            "Auth page must include the initial loading message. Got: {html}"
        );
        assert!(
            html.contains("Sign-in failed"),
            "Auth page must include the default error title. Got: {html}"
        );
    }
}
