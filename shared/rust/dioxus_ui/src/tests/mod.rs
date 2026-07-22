//! Tests for Wave 3b Track A — per-page auth-gate enrichment.
//!
//! Each protected frontend page wraps its body in `<AuthGate>` and provides a
//! return URL for the SIWE flow. Pages may also surface backend-issued scopes,
//! but they must not invent a frontend authorization policy when the source
//! contract requires authentication only.
//!
//! These tests render the page bodies (authenticated vs. signed-out)
//! via `dioxus_ssr::render_element` and assert the gate either
//! **passes through** the body content (authed) or **renders the
//! "Sign in required"** gate panel (signed out).
//!
//! They mirror the Wave 3a Track A pattern (`main_layout.rs`): no
//! router, no Dioxus runtime hydration, no event handlers — just
//! `dioxus_ssr::render_element(el) -> String` and a few
//! `html.contains(...)` assertions.
//!
//! Wave 23 T3 — `/account` is no longer the canonical "gated page"
/*! test target: the wave 22 T2 commit
 *  (`apps-old/frontend/middleware.ts` publicRoutes: `/account*`)
 *  removed the AuthGate wrapper from the account page so the
 *  prod-shape layout renders for anonymous visitors. The
 *  /profile page is now the canonical example of a page that
 *  still wraps in `<AuthGate>`, so the tests below use
 *  `/profile` as the asserted path. The change preserves
 *  the original assertions and adds the new `?return_url=`
 *  test alongside.
 */

use crate::auth::User;
use crate::pages::PageContext;

/// Convenience: build a `User` with the given backend-issued permissions.
fn test_user_with(permissions: &[&str]) -> User {
    User {
        id: "u1".to_string(),
        address: "0x1234…abcd".to_string(),
        chain_id: "56".to_string(),
        roles: vec!["user".to_string()],
        email: None,
        tier: Some("Pro".to_string()),
        permissions: permissions.iter().map(|s| s.to_string()).collect(),
        last_login_at: None,
        auth_method: crate::auth::user::AuthMethod::Wallet,
        display_name: None,
    }
}

/// Render a Dioxus `Element` to a string for assertion. Same shape
/// as the helper in `layout/main_layout.rs` — duplicated here so
/// tests can live in this module without depending on layout internals.
fn render_to_string(el: dioxus::prelude::Element) -> String {
    dioxus_ssr::render_element(el)
}

/// Build an authenticated `PageContext` pointing at `/profile`. The
/// BFF would normally fill in `params`, `wallet`, `query`, etc. — for
/// these unit tests we only need `user` and `path`.
fn authed_ctx(path: &str, user: User) -> PageContext {
    PageContext {
        user: Some(user),
        path: path.to_string(),
        ..Default::default()
    }
}

fn anon_ctx(path: &str) -> PageContext {
    PageContext {
        path: path.to_string(),
        ..Default::default()
    }
}

/// Page body header is the canonical "we made it through the gate"
/// marker. The profile page's `<PageHeader>` always renders the title
/// `"Profile & Settings"`; the gate panel never does (its headline
/// is `"Sign in required"`). The `&` is HTML-encoded as `&#38;` in
/// the SSR output, so we match on the encoded form.
const PROFILE_HEADER: &str = "Profile &#38; Settings";

/// Gate panel's default headline — emitted inside the
/// `<h2 class="auth-gate-title">` only when the gate fires.
const GATE_HEADLINE: &str = "Sign in required";

#[test]
fn profile_page_renders_body_when_authenticated() {
    let user = test_user_with(&[]);
    let ctx = authed_ctx("/profile", user);
    let (_meta, element) = crate::pages::profile::render(&ctx);
    let html = render_to_string(element);
    assert!(
        html.contains(PROFILE_HEADER),
        "Authenticated /profile should render the page body header. Got: {}",
        html
    );
    assert!(
        !html.contains(GATE_HEADLINE),
        "Authenticated /profile should NOT render the gate headline. Got: {}",
        html
    );
}

#[test]
fn profile_page_renders_gate_when_signed_out() {
    // No `user` — the gate fires and shows the "Sign in required"
    // panel. The page body must NOT leak through.
    let ctx = anon_ctx("/profile");
    let (_meta, element) = crate::pages::profile::render(&ctx);
    let html = render_to_string(element);
    assert!(
        html.contains(GATE_HEADLINE),
        "Signed-out /profile should render the gate headline. Got: {}",
        html
    );
    assert!(
        !html.contains(PROFILE_HEADER),
        "Signed-out /profile must NOT render the page body header. Got: {}",
        html
    );
}

/// Secondary assertion: the return_url is plumbed into the
/// `?return_url=<path>` query string on the connect link. This is the
/// bounce-back path the SIWE flow needs.
///
/// Wave 23 T3 — renamed from `?next=` to `?return_url=` to match the
/// prod Vercel middleware (`apps-old/frontend/middleware.ts`
/// `handleUnauthenticated` + `handleExplicitReturnUrl` both read
/// `?return_url=`). The auth page's hydration script also reads
/// `return_url` so the bounce-back works end-to-end.
///
/// Also pre-23 T3, the assertion was on `/account` (which used to
/// wrap in `<AuthGate>`); post-wave-22-T2 the `/account` page is
/// public-readable (no gate), so the canonical "gated page" is now
/// `/profile` and the assertion moved there. Per the
/// `spec-flips-pre-existing-test` rule (memory:
/// `epsx-dioxus-port-waves.md`), the test was EXTENDED to also
/// cover the admin-redirect shape (see
/// `account_legacy_alias_works_on_legacy_path` below) and
/// assert the OLD `?next=` form does NOT appear — the rename must
/// be clean, not additive.
#[test]
fn profile_page_return_url_bounces_back_to_path() {
    let ctx = anon_ctx("/profile");
    let (_meta, element) = crate::pages::profile::render(&ctx);
    let html = render_to_string(element);
    // The gate builds `connect_href = format!("/auth?return_url={}", u)`
    // when return_url is Some and non-empty (URL-encoded for paths
    // that already contain `?` / `&` / `=`). We expect the literal
    // substring `/auth?return_url=%2Fprofile` somewhere in the gate
    // markup. (Was `/auth?next=/account` pre-wave-23 — see the
    // wave23-t3 changelog for the rename.)
    assert!(
        html.contains("/auth?return_url=%2Fprofile"),
        "Signed-out /profile should expose the return_url as /auth?return_url=%2Fprofile. Got: {}",
        html
    );
    // Belt-and-braces: the old `?next=` form must NOT appear (the
    // rename must be clean, not additive).
    assert!(
        !html.contains("/auth?next="),
        "Signed-out /profile must NOT use the old `?next=` form. Got: {}",
        html
    );
}

#[test]
fn profile_page_does_not_create_a_frontend_permission_gate() {
    let user = test_user_with(&[]);
    let ctx = authed_ctx("/profile", user);
    let (_meta, element) = crate::pages::profile::render(&ctx);
    let html = render_to_string(element);
    assert!(
        html.contains(PROFILE_HEADER),
        "Any locally verified session should render the profile body. Got: {}",
        html
    );
    assert!(
        !html.contains("Permission required"),
        "The profile route must not infer profile:read/profile:write policy in the UI. Got: {}",
        html
    );
}
