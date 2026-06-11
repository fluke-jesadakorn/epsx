//! Tests for Wave 3b Track A — per-page auth-gate enrichment.
//!
//! Each frontend user page (`/account`, `/profile`, `/dashboard`, etc.)
//! now wraps its body in `<AuthGate>` with:
//! - `required_permissions: Some(vec!["<perm>".to_string()])` — the
//!   `domain:action` schema from `docs/wave3b-gates/design.md` §1.
//! - `return_url: Some(ctx.path.clone())` — the path the SIWE flow
//!   should bounce the user back to after sign-in.
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

use crate::auth::User;
use crate::pages::PageContext;

/// Convenience: build a `User` with the given permissions list. The
/// gate's `has_permission` check is exact-match against
/// `User::permissions`, so the test user must explicitly hold the
/// permission the page gates on.
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

/// Build an authenticated `PageContext` pointing at `/account`. The
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
/// marker. The account page's `<PageHeader>` always renders the title
/// `"Account"`; the gate panel never does (its headline is
/// `"Sign in required"`).
const ACCOUNT_HEADER: &str = "Account";

/// Gate panel's default headline — emitted inside the
/// `<h2 class="auth-gate-title">` only when the gate fires.
const GATE_HEADLINE: &str = "Sign in required";

#[test]
fn account_page_renders_body_when_authenticated() {
    // The /account page gates on `profile:read` (see
    // `pages/account.rs` + `docs/wave3b-gates/design.md` §1). The
    // test user must hold that permission to pass the gate.
    let user = test_user_with(&["profile:read"]);
    let ctx = authed_ctx("/account", user);
    let (_meta, element) = crate::pages::account::render(&ctx);
    let html = render_to_string(element);
    assert!(
        html.contains(ACCOUNT_HEADER),
        "Authenticated /account should render the page body header. Got: {}",
        html
    );
    assert!(
        !html.contains(GATE_HEADLINE),
        "Authenticated /account should NOT render the gate headline. Got: {}",
        html
    );
}

#[test]
fn account_page_renders_gate_when_signed_out() {
    // No `user` — the gate fires and shows the "Sign in required"
    // panel. The page body must NOT leak through.
    let ctx = anon_ctx("/account");
    let (_meta, element) = crate::pages::account::render(&ctx);
    let html = render_to_string(element);
    assert!(
        html.contains(GATE_HEADLINE),
        "Signed-out /account should render the gate headline. Got: {}",
        html
    );
    assert!(
        !html.contains(ACCOUNT_HEADER),
        "Signed-out /account must NOT render the page body header. Got: {}",
        html
    );
}

/// Secondary assertion: the return_url is plumbed into the
/// `?next=<path>` query string on the connect link. This is the
/// bounce-back path the SIWE flow needs.
#[test]
fn account_page_return_url_bounces_back_to_path() {
    let ctx = anon_ctx("/account");
    let (_meta, element) = crate::pages::account::render(&ctx);
    let html = render_to_string(element);
    // The gate builds `connect_href = format!("/auth?next={}", u)`
    // when return_url is Some and non-empty. We expect the literal
    // substring `/auth?next=/account` somewhere in the gate markup.
    assert!(
        html.contains("/auth?next=/account"),
        "Signed-out /account should expose the return_url as /auth?next=/account. Got: {}",
        html
    );
}

/// The required_permissions list is shown in the gate panel when the
/// user is signed in but missing a permission. This is the "you are
/// signed in but lack the right role" UX.
#[test]
fn account_page_required_permissions_shown_when_missing() {
    // The user is signed in but does NOT have `profile:read`. The
    // gate's case-2 fires: signed in + missing permission = "Permission
    // required" panel listing the missing permission.
    let user = test_user_with(&[]); // no permissions at all
    let ctx = authed_ctx("/account", user);
    let (_meta, element) = crate::pages::account::render(&ctx);
    let html = render_to_string(element);
    assert!(
        html.contains("Permission required"),
        "Signed-in /account without profile:read should render the 'Permission required' panel. Got: {}",
        html
    );
    // The missing permission should be listed.
    assert!(
        html.contains("profile:read"),
        "Permission panel should list the missing permission. Got: {}",
        html
    );
    // The page body must NOT leak through.
    assert!(
        !html.contains(ACCOUNT_HEADER),
        "Signed-in /account without profile:read must NOT render the page body header. Got: {}",
        html
    );
}
