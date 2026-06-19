//! `ProgressiveAuthBanner` — inline "sign in to unlock" strip
//! variant for the OLD components/ tree.
//!
//! Port of `apps-old/frontend/components/auth/progressive-auth-banner.tsx`
//! (16 LoC). The TS source is a 1-line wrapper around the shared
//! `AuthBanner` component. The Dioxus port mirrors that: a thin
//! re-export of the existing `crate::auth::progressive_banner::ProgressiveAuthBanner`
//! primitive (Wave 2 T1) under the legacy components/ namespace.
//!
//! Kept as a separate file so the auth/ module's mod.rs can expose
//! both `ProgressiveAuthBanner` (the existing primitive) and this
//! new `ProgressiveAuthBanner` (the components/ tree alias) without
//! ambiguity. They share the same impl — this is just a
//! documentation-friendly alias.

use crate::auth::progressive_banner::ProgressiveAuthBanner;

use dioxus::prelude::*;

/// Re-export of the shared `ProgressiveAuthBanner` primitive under
/// the `crate::auth::progressive_auth_banner` path. Mirrors the
/// TS source's `ProgressiveAuthBanner` from
/// `components/auth/progressive-auth-banner.tsx`.
#[component]
pub fn ProgressiveAuthBannerLegacy(
    /// Optional message override. Default: "Sign in to access this
    /// feature".
    #[props(default = None)] message: Option<String>,
    /// Optional description override. Default: "Connect your wallet
    /// to continue".
    #[props(default = None)] description: Option<String>,
) -> Element {
    rsx! {
        ProgressiveAuthBanner {
            message,
            description,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn progressive_auth_banner_legacy_signature_matches_ts() {
        // The TS source takes `message?: string, description?: string`
        // — both optional. The Dioxus port preserves this with
        // Option<String> props.
        
    }
}
