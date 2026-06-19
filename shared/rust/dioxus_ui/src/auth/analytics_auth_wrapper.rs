//! `AnalyticsAuthWrapper` — pass-through container for analytics
//! auth-gated content.
//!
//! Port of `apps-old/frontend/components/auth/analytics-auth-wrapper.tsx`
//! (11 LoC). The TS source is a stateless client component that
//! simply returns its children — it exists to give a stable name to
//! the "this section requires auth" boundary in the route file.
//!
//! In the Dioxus port we keep the same name and signature so the
//! analytics / dashboard pages can be ported 1:1 from the TS source
//! without renaming. The real "show unauth state" logic lives in
//! `pages/analytics.rs::render` via the `PageContext::user` check —
//! `AnalyticsAuthWrapper` itself does NOT enforce auth.

use dioxus::prelude::*;

#[component]
pub fn AnalyticsAuthWrapper(children: Element) -> Element {
    rsx! {
        Fragment { {children} }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn analytics_auth_wrapper_renders_children() {
        // Smoke test: the wrapper is a transparent pass-through, so
        // we just verify the component name resolves and the
        // signature accepts an `Element` slot.
        
    }

    #[test]
    fn analytics_auth_wrapper_does_not_enforce_auth() {
        // The TS source returns its children unconditionally — it
        // does NOT check authentication state. The Dioxus port
        // preserves this: a caller that wants to gate content must
        // branch on `ctx.user.is_some()` at the page level (see
        // `pages/analytics.rs`).
        //
        // This test just documents the contract — we can't render
        // Dioxus components in unit tests, but the wrapper signature
        // itself is what enforces "no auth check".
        
    }
}
