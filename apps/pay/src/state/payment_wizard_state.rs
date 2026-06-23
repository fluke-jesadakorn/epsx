//! Payment wizard state — typed state for the checkout flow.
//!
//! wave49(slice-2): replaces the inline `Signal<…>` typed state
//! scattered across the old `apps/pay/src/main.rs` checkout
//! inline HTML. Centralizes the wizard so the Dioxus checkout
//! component can drive the full stepper + connect modal +
//! approval + payment transaction sequence without losing
//! types or relying on ad-hoc `serde_json::Value` lookups.
//!
//! State machine (lifted from `UnifiedPaymentStep` in
//! `shared/rust/dioxus_ui::payment::unified_payment_flow`):
//!
//! ```text
//!   Idle
//!     → ConnectWallet
//!         → SelectPlan
//!             → Approve
//!                 → Pay
//!                     → Done | Failed(reason)
//! ```
//!
//! Slice-3 will add the on-chain polling step
//! (`chain_verification_card::ChainVerificationStatus`).
//! For slice-2 the state stays pre-on-chain; the checkout
//! component renders the `UnifiedPaymentFlow` with the
//! current step and lets the user click "Continue" to
//! advance.

use serde::{Deserialize, Serialize};

/// Wizard step — mirrors `UnifiedPaymentStep` but adds
/// `Failed { reason }` for the error path. The checkout
/// component maps this into `UnifiedPaymentStep` for the
/// visual stepper.
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub enum WizardStep {
    Idle,
    ConnectWallet,
    SelectPlan,
    Approve,
    Pay,
    Done,
    Failed { reason: String },
}

impl Default for WizardStep {
    fn default() -> Self {
        Self::Idle
    }
}

/// URL params parsed from the BFF SSR fallback (`?amount=…&currency=…&chain_id=…&token=…&intent=…`).
///
/// `intent` is set when the user is returning to the checkout after
/// creating an intent; `amount` + `currency` + `chain_id` + `token`
/// are set when the user lands cold (e.g. from a `pay.epsx.io/r/{slug}`
/// shareable link redirect — added in slice-3).
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct CheckoutUrlParams {
    pub amount: Option<String>,
    pub currency: Option<String>,
    pub chain_id: Option<String>,
    pub token: Option<String>,
    pub intent: Option<String>,
    pub description: Option<String>,
}

/// Top-level state for the `/checkout` page.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct PaymentWizardState {
    pub step: WizardStep,
    pub params: CheckoutUrlParams,
    /// Set once `POST /api/v1/pay/intent` returns a created intent.
    pub created_intent_id: Option<String>,
    /// Set once the wallet is connected (slice-3 will populate
    /// from the Reown AppKit `wagmi.connected` signal).
    pub wallet_address: Option<String>,
    /// Set once the on-chain tx is broadcast (slice-3).
    pub tx_hash: Option<String>,
}

impl PaymentWizardState {
    /// Construct from URL search params (parsed by the BFF SSR
    /// fallback before `VirtualDom::new_with_props`).
    pub fn from_search(query: &str) -> Self {
        let params: CheckoutUrlParams = serde_urlencoded::from_str(query)
            .unwrap_or_default();
        let step = if params.intent.is_some() {
            // Returning user — show the in-progress state.
            WizardStep::ConnectWallet
        } else {
            WizardStep::Idle
        };
        Self {
            step,
            params,
            created_intent_id: None,
            wallet_address: None,
            tx_hash: None,
        }
    }

    /// Advance to the next step. Returns `true` if the step
    /// changed, `false` if already at the target.
    pub fn advance(&mut self) -> bool {
        let next = match &self.step {
            WizardStep::Idle => WizardStep::ConnectWallet,
            WizardStep::ConnectWallet => WizardStep::SelectPlan,
            WizardStep::SelectPlan => WizardStep::Approve,
            WizardStep::Approve => WizardStep::Pay,
            WizardStep::Pay => WizardStep::Done,
            WizardStep::Done | WizardStep::Failed { .. } => return false,
        };
        self.step = next;
        true
    }

    /// Move to the failed state with a reason.
    pub fn fail(&mut self, reason: impl Into<String>) {
        self.step = WizardStep::Failed { reason: reason.into() };
    }

    /// Map to `UnifiedPaymentStep` for the visual stepper.
    pub fn unified_step(&self) -> epsx_dioxus_ui::payment::UnifiedPaymentStep {
        use epsx_dioxus_ui::payment::UnifiedPaymentStep as U;
        match &self.step {
            WizardStep::Idle => U::Idle,
            WizardStep::ConnectWallet => U::ConnectWallet,
            WizardStep::SelectPlan => U::SelectPlan,
            WizardStep::Approve => U::Approve,
            WizardStep::Pay => U::Pay,
            WizardStep::Done => U::Done,
            WizardStep::Failed { .. } => U::Pay, // visual fallback — error renders via `error` prop
        }
    }
}

/// Minimal `serde_urlencoded` shim — avoids pulling in the full
/// `serde_urlencoded` crate. Parses `?amount=10&currency=USDT` style
/// query strings into `CheckoutUrlParams`. Empty / malformed
/// inputs return `Default::default()` (no error propagated).
mod serde_urlencoded {
    use super::CheckoutUrlParams;

    pub fn from_str(query: &str) -> Result<CheckoutUrlParams, ()> {
        let query = query.trim_start_matches('?');
        if query.is_empty() {
            return Ok(CheckoutUrlParams::default());
        }
        let mut out = CheckoutUrlParams::default();
        for pair in query.split('&') {
            if let Some((k, v)) = pair.split_once('=') {
                let key = url_decode(k);
                let val = url_decode(v);
                match key.as_str() {
                    "amount" => out.amount = Some(val),
                    "currency" => out.currency = Some(val),
                    "chain_id" => out.chain_id = Some(val),
                    "token" => out.token = Some(val),
                    "intent" => out.intent = Some(val),
                    "description" => out.description = Some(val),
                    _ => {}
                }
            }
        }
        Ok(out)
    }

    fn url_decode(s: &str) -> String {
        // Minimal percent-decoder — handles the common case of
        // `%20` → space, `%2F` → `/`. Anything more exotic
        // passes through unchanged.
        let mut out = String::with_capacity(s.len());
        let bytes = s.as_bytes();
        let mut i = 0;
        while i < bytes.len() {
            if bytes[i] == b'%' && i + 2 < bytes.len() {
                if let (Some(h), Some(l)) = (hex(bytes[i + 1]), hex(bytes[i + 2])) {
                    out.push((h * 16 + l) as char);
                    i += 3;
                    continue;
                }
            }
            out.push(bytes[i] as char);
            i += 1;
        }
        out
    }

    fn hex(b: u8) -> Option<u8> {
        match b {
            b'0'..=b'9' => Some(b - b'0'),
            b'a'..=b'f' => Some(b - b'a' + 10),
            b'A'..=b'F' => Some(b - b'A' + 10),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wizard_step_default_is_idle() {
        assert_eq!(WizardStep::default(), WizardStep::Idle);
    }

    #[test]
    fn advance_walks_full_path() {
        let mut s = PaymentWizardState::default();
        assert!(s.advance());
        assert_eq!(s.step, WizardStep::ConnectWallet);
        assert!(s.advance());
        assert_eq!(s.step, WizardStep::SelectPlan);
        assert!(s.advance());
        assert_eq!(s.step, WizardStep::Approve);
        assert!(s.advance());
        assert_eq!(s.step, WizardStep::Pay);
        assert!(s.advance());
        assert_eq!(s.step, WizardStep::Done);
        assert!(!s.advance()); // Done is terminal
    }

    #[test]
    fn fail_sets_failed_state() {
        let mut s = PaymentWizardState::default();
        s.advance();
        s.advance();
        s.fail("user rejected tx");
        match &s.step {
            WizardStep::Failed { reason } => assert_eq!(reason, "user rejected tx"),
            other => panic!("expected Failed, got {:?}", other),
        }
        assert!(!s.advance()); // terminal
    }

    #[test]
    fn from_search_parses_amount_and_currency() {
        let s = PaymentWizardState::from_search("amount=10.50&currency=USDT&chain_id=56&token=USDT");
        assert_eq!(s.params.amount.as_deref(), Some("10.50"));
        assert_eq!(s.params.currency.as_deref(), Some("USDT"));
        assert_eq!(s.params.chain_id.as_deref(), Some("56"));
        assert_eq!(s.params.token.as_deref(), Some("USDT"));
        assert_eq!(s.step, WizardStep::Idle);
    }

    #[test]
    fn from_search_with_intent_starts_at_connect() {
        let s = PaymentWizardState::from_search("intent=0xabc123");
        assert_eq!(s.params.intent.as_deref(), Some("0xabc123"));
        assert_eq!(s.step, WizardStep::ConnectWallet);
    }

    #[test]
    fn from_search_with_empty_returns_default() {
        let s = PaymentWizardState::from_search("");
        assert!(s.params.amount.is_none());
        assert_eq!(s.step, WizardStep::Idle);
    }

    #[test]
    fn unified_step_maps_correctly() {
        let mut s = PaymentWizardState::default();
        assert_eq!(s.unified_step(), epsx_dioxus_ui::payment::UnifiedPaymentStep::Idle);
        s.advance();
        assert_eq!(s.unified_step(), epsx_dioxus_ui::payment::UnifiedPaymentStep::ConnectWallet);
        s.advance();
        s.advance();
        s.advance();
        s.advance();
        assert_eq!(s.unified_step(), epsx_dioxus_ui::payment::UnifiedPaymentStep::Done);
    }
}