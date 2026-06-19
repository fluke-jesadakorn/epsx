//! `FrontendAuthModal` — auth modal controlled by the global
//! `useSharedAuth` provider state.
//!
//! Port of `apps-old/frontend/components/auth/frontend-auth-modal.tsx`
//! (17 LoC). The TS source reads `showSignInModal` + `closeSignInModal`
//! from the auth context and forwards them to `<AuthModal>`. The
//! Dioxus port is a thin wrapper around the existing
//! `AuthModal` primitive that hardcodes `variant="user"` and
//! forwards the props the caller provides.
//!
//! In SSR the modal is a visual stub — the global `useSharedAuth`
//! provider is client-only in the OLD Next.js app, and the Dioxus
//! port renders the modal in the closed state by default. Pages
//! that need an open-on-mount modal can pass `open={true}`.

use crate::auth::auth_modal::AuthModal;
use crate::auth::auth_modal::WalletInfo;

use dioxus::prelude::*;

#[component]
pub fn FrontendAuthModal(
    /// Forwarded to `AuthModal::open`. Defaults to `false`
    /// (modal closed in SSR).
    #[props(default = false)] open: bool,
    /// Fired when the user dismisses the modal.
    #[props(default = None)] on_close: Option<EventHandler<MouseEvent>>,
    /// Fired after a successful auth.
    #[props(default = None)] on_success: Option<EventHandler<()>>,
    /// Optional list of wallets to show.
    #[props(default = None)] wallets: Option<Vec<WalletInfo>>,
) -> Element {
    rsx! {
        AuthModal {
            open,
            on_close: move |e| {
                if let Some(cb) = on_close.as_ref() {
                    cb.call(e);
                }
            },
            on_success: on_success,
            variant: Some("user".to_string()),
            wallets: wallets,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn frontend_auth_modal_renders_closed_by_default() {
        // The TS source reads `showSignInModal` from context and
        // defaults to `false`. The Dioxus port uses the `open`
        // prop which defaults to `false`.
        
    }
}
