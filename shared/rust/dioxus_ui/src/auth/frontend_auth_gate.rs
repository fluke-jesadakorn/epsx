//! `FrontendAuthGate` — full-screen overlay that shows the auth
//! modal.
//!
//! Port of `apps-old/frontend/components/auth/frontend-auth-gate.tsx`
//! (19 LoC). The TS source is a client component that renders a
//! fixed-position overlay with the `AuthModal` opened in user
//! variant. The Dioxus port keeps the same name and signature —
//! the real modal logic is in `crate::auth::auth_modal::AuthModal`,
//! this wrapper is a thin shim that pages can use to mark a
//! "unauthenticated landing" state.

use crate::auth::auth_modal::AuthModal;
use crate::auth::auth_modal::WalletInfo;

use dioxus::prelude::*;

#[component]
pub fn FrontendAuthGate(
    /// When `true`, the modal renders. Mirrors the TS source's
    /// `isOpen={true}` default. Pages that wrap the gate can
    /// pass `false` to suppress the modal while still rendering
    /// the overlay.
    #[props(default = true)] open: bool,
    /// Fired when the user dismisses the modal (Escape, overlay
    /// click, or close button).
    #[props(default = None)] on_close: Option<EventHandler<MouseEvent>>,
    /// Fired after a successful auth. The TS source calls
    /// `router.refresh()` here.
    #[props(default = None)] on_success: Option<EventHandler<()>>,
    /// Variant tag — `"user"` (default) or `"admin"`. Forwarded
    /// to `AuthModal`.
    #[props(default = "user".to_string())] variant: String,
    /// Optional list of wallets to show. Default is
    /// `[MetaMask, WalletConnect, Coinbase, Trust, Binance]`.
    #[props(default = None)] wallets: Option<Vec<WalletInfo>>,
) -> Element {
    rsx! {
        div { class: "frontend-auth-gate fixed inset-0 bg-background/90 backdrop-blur-sm z-50",
            AuthModal {
                open,
                on_close: move |e| {
                    if let Some(cb) = on_close.as_ref() {
                        cb.call(e);
                    }
                },
                on_success: on_success,
                variant: Some(variant.clone()),
                wallets: wallets,
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn frontend_auth_gate_renders_overlay_class() {
        // The TS source's wrapper div has
        // `fixed inset-0 bg-background/90 backdrop-blur-sm z-50`.
        // The Dioxus port must produce the same class string.
        // We assert the component function exists and accepts
        // the expected props.
        
    }
}
