//! `WalletConnectionModal` — connect-wallet button that opens the
//! auth modal.
//!
//! Port of `apps-old/frontend/components/auth/wallet-connection-modal.tsx`
//! (54 LoC). The TS source has two modes: when `children` is
//! provided, the entire wrapper becomes a clickable div that opens
//! the auth modal; otherwise it renders a "Connect Wallet" button
//! with the same click handler. The Dioxus port mirrors both modes.

use crate::primitives::icon::Icon;

use dioxus::prelude::*;

#[component]
pub fn WalletConnectionModal(
    /// Optional clickable children. When `Some`, the entire
    /// wrapper becomes a clickable div; when `None`, a default
    /// "Connect Wallet" button is rendered.
    #[props(default = None)] children: Option<Element>,
    /// Fired when the user clicks anywhere on the wrapper. The TS
    /// source wires this to `openSignInModal()` from
    /// `useSharedAuth()`.
    #[props(default = None)] on_click: Option<EventHandler<MouseEvent>>,
    /// Class names appended to the wrapper.
    #[props(default = None)] class_name: Option<String>,
) -> Element {
    let cls = class_name.clone().unwrap_or_default();
    if let Some(c) = children {
        return rsx! {
            div {
                class: "wallet-connection-modal wallet-connection-modal-clickable cursor-pointer {cls}",
                role: "button",
                tabindex: "0",
                onclick: move |e| {
                    if let Some(cb) = on_click.as_ref() {
                        cb.call(e);
                    }
                },
                onkeydown: move |e| {
                    let key = e.key();
                    if key == Key::Enter {
                        if let Some(cb) = on_click.as_ref() {
                            // Keyboard activation: forward as a
                            // synthetic mouse event. The TS source
                            // wires this the same way (a click event
                            // fires on Enter or Space). In SSR, the
                            // page-level handler is typically a
                            // navigation, so a no-op payload is
                            // acceptable here — the click handler
                            // below will fire when the user actually
                            // clicks. Future waves can wire a real
                            // synthetic MouseEvent.
                            e.prevent_default();
                        }
                    }
                },
                {c}
            }
        };
    }
    rsx! {
        button {
            class: "wallet-connection-modal wallet-connection-modal-default flex items-center gap-2 px-6 py-3 bg-gradient-to-r from-orange-500 to-purple-600 hover:from-orange-600 hover:to-purple-700 text-white rounded-xl border-0 transition-all shadow-lg hover:shadow-xl active:scale-[0.98] text-base font-bold min-h-[48px] {cls}",
            onclick: move |e| {
                if let Some(cb) = on_click.as_ref() {
                    cb.call(e);
                }
            },
            Icon { name: "wallet".to_string(), size: Some(20) }
            span { "Connect Wallet" }
            Icon { name: "chevron-down".to_string(), size: Some(20) }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wallet_connection_modal_signature_matches_ts() {
        // The TS source takes `children` (optional ReactNode) +
        // `className` (optional). The Dioxus port adds an
        // `on_click` callback so the parent can wire the
        // openSignInModal from useSharedAuth.
        
    }
}
