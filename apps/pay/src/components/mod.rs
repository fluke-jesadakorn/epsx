//! Pay BFF page components — Dioxus wrappers around the ported
//! payment components in `shared/rust/dioxus_ui::payment::*`.
//!
//! wave49(slice-2): replaces the inline `r##"..."##` HTML strings
//! in the previous `apps/pay/src/main.rs` checkout/success/cancel
//! bodies. Each page is a Dioxus `#[component]` that:
//!   1. Accepts a typed prop (state, intent id, error).
//!   2. Renders the appropriate ported component(s) with that
//!      prop.
//!   3. Returns `Element` for `dioxus_ssr::render_element` to
//!      serialize.
//!
//! Slice-3 will add client-side interactivity (Reown AppKit wallet
//! connect, on-chain polling). For slice-2 these components are
//! static SSR shells — same visual structure as the ported
//! components but driven by server-fetched state, not signals.

pub mod cancel_screen;
pub mod checkout_form;
pub mod escrow_status;
pub mod success_screen;

pub use cancel_screen::PayCancelScreen;
pub use checkout_form::PayCheckoutForm;
pub use escrow_status::PayEscrowStatus;
pub use success_screen::PaySuccessScreen;