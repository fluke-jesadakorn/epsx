use crate::primitives::*;

use super::PageContext;
use super::PageMeta;
use crate::auth::ConnectButton;
use crate::auth::ConnectButtonSize;
use crate::layout::main_layout::AuthLayout;
use dioxus::prelude::*;

/// Auth page (`/auth`). Wave 5 Track A port — see
/// `docs/wave5-page-depth/design.md` §"Track A — Hero pages" /
/// `auth_page.rs`. Two-column layout:
///   - LEFT: marketing pitch (hero copy + 3 value props + 1 testimonial)
///   - RIGHT: the auth form (SIWE ConnectButton + email magic link +
///     Google OAuth button)
///
/// Wave 50 — wired up the full SIWE flow:
/// - The `<ConnectButton data_connect_wallet=true>` renders a raw
///   `<button data-connect-wallet="true">` (the page-shell
///   `wallet_shim()` attaches the click → `window.epsx.connectWallet()`).
/// - Loading + error banners are rendered statically with stable ids;
///   an inline `<script>` listens for the `epsx:wallet:status` events
///   the shim broadcasts and toggles `hidden` + content. No Dioxus
///   hydration is required — every interactive state is SSR-safe.
#[component]
pub fn AuthPage() -> Element {
    rsx! { RenderAuth {} }
}

#[component]
pub fn RenderAuth() -> Element {
    // The component is purely declarative — every interactive state
    // (loading / error / success) is driven by the inline script
    // below. The Dioxus `use_signal` calls were removed because
    // their closures don't survive SSR; the SSR'd HTML carries the
    // initial state (idle, no error, no loading) and JS toggles the
    // visibility from there.

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
                        h2 { class: "auth-card-title", "Welcome to EPSX" }
                        p { class: "auth-card-sub", "Secure authentication via Web3" }
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
                                data_connect_wallet: Some(true),
                            }
                        }
                        // === Loading state (hidden by default) ===
                        // The inline script at the bottom of the page
                        // toggles `hidden` and updates the message as
                        // the SIWE flow progresses through challenge →
                        // signing → verifying. Static markup so the SSR
                        // snapshot carries the full UI even with
                        // hydration off.
                        div {
                            id: "auth-card-status",
                            class: "auth-card-status",
                            "aria-live": "polite",
                            hidden: true,
                            div { class: "spinner spinner-sm" }
                            span {
                                id: "auth-card-status-msg",
                                "Waiting for wallet..."
                            }
                        }
                        // === Error banner (hidden by default) ===
                        div {
                            id: "auth-card-error",
                            class: "auth-card-error",
                            role: "alert",
                            hidden: true,
                            div { class: "auth-card-error-icon",
                                Icon { name: "triangle-alert".to_string(), size: Some(16) }
                            }
                            div { class: "auth-card-error-body",
                                div {
                                    id: "auth-card-error-title",
                                    class: "auth-card-error-title",
                                    "Sign-in failed"
                                }
                                div {
                                    id: "auth-card-error-msg",
                                    class: "auth-card-error-msg",
                                    ""
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
                        "Network Secure & Operational"
                    }
                    // === Manual redirect fallback ===
                    div { class: "auth-page-fallback",
                        a { href: "/", "Go to Homepage" }
                    }
                    // === Wave 50 — wallet status event listener ===
                    //
                    // Toggles the loading + error banner visibility
                    // based on `epsx:wallet:status` events broadcast by
                    // the page-shell `wallet_shim()`. Pure DOM
                    // manipulation — no Dioxus hydration needed.
                    script { dangerous_inner_html: WALLET_STATUS_LISTENER_SCRIPT }
                }
            }
        }
    }
}

/// Inline `<script>` that listens for the `epsx:wallet:status` events
/// the page-shell `wallet_shim()` broadcasts and toggles the loading
/// + error banner visibility.
///
/// Status shape:
///   `{ status: 'idle'|'challenge'|'signing'|'verifying'|'success'|'error',
///      kind?: 'no_wallet'|'wrong_network'|'rejected'|'error',
///      message?: string,
///      address?: string }`
///
/// The script is intentionally minimal — it only manipulates the four
/// known elements (CTA button + status banner + error title + error
/// message). No innerHTML injection, no fetch, no library deps.
const WALLET_STATUS_LISTENER_SCRIPT: &str = r#"(function(){
  function $(id){ return document.getElementById(id); }
  var ctaBtn = document.getElementById('auth-card-cta-btn') ||
               document.querySelector('[data-connect-wallet]');
  var statusEl = $('auth-card-status');
  var statusMsg = $('auth-card-status-msg');
  var errorEl = $('auth-card-error');
  var errorTitle = $('auth-card-error-title');
  var errorMsg = $('auth-card-error-msg');

  function show(d) {
    if (!d) return;
    if (d.status === 'challenge') {
      statusMsg.textContent = 'Requesting challenge...';
      statusEl.hidden = false;
      errorEl.hidden = true;
      if (ctaBtn) ctaBtn.disabled = true;
    } else if (d.status === 'signing') {
      statusMsg.textContent = 'Check your wallet...';
      statusEl.hidden = false;
      errorEl.hidden = true;
      if (ctaBtn) ctaBtn.disabled = true;
    } else if (d.status === 'verifying') {
      statusMsg.textContent = 'Verifying signature...';
      statusEl.hidden = false;
      errorEl.hidden = true;
      if (ctaBtn) ctaBtn.disabled = true;
    } else if (d.status === 'success') {
      // Page reload handled by the shim.
      statusEl.hidden = true;
      errorEl.hidden = true;
    } else if (d.status === 'error') {
      var title = 'Sign-in failed';
      if (d.kind === 'no_wallet') title = 'Wallet not installed';
      else if (d.kind === 'wrong_network') title = 'Wrong network';
      else if (d.kind === 'rejected') title = 'Signature rejected';
      errorTitle.textContent = title;
      errorMsg.textContent = d.message || 'Authentication failed. Please try again.';
      errorEl.hidden = false;
      statusEl.hidden = true;
      if (ctaBtn) ctaBtn.disabled = false;
    } else if (d.status === 'idle') {
      statusEl.hidden = true;
      errorEl.hidden = true;
      if (ctaBtn) ctaBtn.disabled = false;
    }
  }

  document.addEventListener('epsx:wallet:status', function(e) {
    show((e && e.detail) || {});
  });
})();"#;

pub fn render(ctx: &PageContext) -> (PageMeta, Element) {
    let meta = PageMeta::marketing("Sign in");
    (
        meta,
        rsx! {
            AuthLayout { ctx: ctx.clone(),
                RenderAuth {}
            }
        },
    )
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

    /// Wave 5 — `test_auth_options`. Originally exposed SIWE +
    /// email magic link + Google OAuth. Wave 49 T1 (Plan 13) changed
    /// /auth to match prod's wallet-only design (Welcome to EPSX /
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
        assert!(
            html.contains("Precision"),
            "Auth page must render the pitch headline. Got: {}",
            html
        );
        // Three value props.
        for value in &["Data Accuracy", "Real-time Edge", "Secure Ownership"] {
            assert!(
                html.contains(value),
                "Auth page pitch must include value prop '{}'. Got: {}",
                value,
                html
            );
        }
        // Social proof.
        assert!(
            html.contains("2,500+"),
            "Auth page must render the '2,500+' social proof. Got: {}",
            html
        );
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

    /// The auth page must include the inline `<script>` that listens
    /// for `epsx:wallet:status` events. Without this script the user
    /// has no visual feedback when the SIWE flow progresses through
    /// challenge → signing → verifying.
    #[test]
    fn test_includes_wallet_status_listener_script() {
        let ctx = PageContext {
            user: None,
            path: "/auth".to_string(),
            ..Default::default()
        };
        let (_meta, el) = render(&ctx);
        let html = dioxus_ssr::render_element(el);
        assert!(
            html.contains("epsx:wallet:status"),
            "Auth page must include the wallet status listener script. Got: {html}"
        );
        // Verify the script toggles all 5 status labels.
        for label in &[
            "Requesting challenge...",
            "Check your wallet...",
            "Verifying signature...",
            "Wallet not installed",
            "Signature rejected",
        ] {
            assert!(
                html.contains(label),
                "Wallet status listener script must reference '{}'. Got: {html}",
                label
            );
        }
    }

    /// The auth page must render the loading + error banner elements
    /// with stable ids so the JS listener can toggle them. Initial
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
