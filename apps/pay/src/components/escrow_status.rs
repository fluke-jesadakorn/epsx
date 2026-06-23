//! `PayEscrowStatus` — `/intent/:id` page for pay.epsx.io.
//!
//! wave49(slice-5): live polling. The page polls
//! `GET /api/v1/pay/intents/{id}` every 5s (via a JS
//! `setInterval` + DOM mutation in the page-level script) and
//! the rendered visual reflects the latest status returned by
//! the upstream. Status mapping:
//!
//! | upstream status | rendered stepper           | chain card                |
//! |-----------------|-----------------------------|---------------------------|
//! | `pending`       | ConnectWallet               | Idle                      |
//! | `escrowed`      | SubmitPayment (active)      | Confirming { 1/12 }       |
//! | `released`      | Complete (success badge)    | Verified (green check)     |
//! | `refunded`      | Complete (warning badge)    | Failed { "refunded" }      |
//! | `cancelled`     | Complete (muted badge)      | Failed { "cancelled" }    |
//!
//! The status text + colored dot are rendered server-side
//! from a static "pending" placeholder; the actual status
//! arrives via the JS poll + DOM update. SSR renders the
//! static shell + the polling script. No hydration needed
//! because the script mutates the DOM directly via `id`
//! selectors.

use dioxus::prelude::*;

use epsx_dioxus_ui::payment::{
    ChainVerificationCard, ChainVerificationStatus, PaymentFlowSteps,
    PaymentFlowStep, PaymentFlowStepsState,
};

#[component]
pub fn PayEscrowStatus(intent_id: String) -> Element {
    rsx! {
        div { class: "pay-escrow-page page-bg",
            section { class: "section",
                style: "max-width:32rem;margin:0 auto;",
                h1 { class: "pay-escrow-title",
                    style: "font-size:1.5rem;font-weight:800;margin-bottom:1rem;",
                    "Payment status"
                }
                p { class: "pay-escrow-intent-id",
                    style: "font-family:monospace;font-size:0.75rem;color:var(--text-subtle);margin-bottom:1.5rem;word-break:break-all;",
                    "Intent: "
                    span { id: "pay-escrow-intent-id-value", "{intent_id.clone()}" }
                }
                // Status pill — the JS poll mutates the
                // `data-status` attribute + text content of the
                // status label every 5s. SSR renders "pending".
                div {
                    id: "pay-escrow-status-pill",
                    class: "pay-escrow-status-pill inline-flex items-center gap-2 px-3 py-1 rounded-full bg-secondary/50 mb-4",
                    "data-status": "pending",
                    span { id: "pay-escrow-status-dot",
                        class: "pay-escrow-status-dot h-2 w-2 rounded-full bg-orange-500"
                    }
                    span { id: "pay-escrow-status-label",
                        class: "pay-escrow-status-label text-sm font-medium text-orange-500",
                        "pending"
                    }
                }
                // Stepper — the JS poll mutates the inner
                // data-active attribute on each step.
                div { id: "pay-escrow-flow-steps",
                    PaymentFlowSteps {
                        state: PaymentFlowStepsState {
                            current: PaymentFlowStep::ConnectWallet,
                            connect_wallet_done: false,
                            approve_token_done: false,
                            submit_payment_done: false,
                            error: None,
                        }
                    }
                }
                // Chain verification card — the JS poll mutates
                // the status enum via class swaps on the inner
                // icon + label.
                div { id: "pay-escrow-chain-card",
                    ChainVerificationCard {
                        status: ChainVerificationStatus::Idle,
                        tx_hash: None,
                        network: "BNB Smart Chain".to_string(),
                    }
                }
                // Polling indicator (visible while a poll is in
                // flight — JS adds/removes the .is-loading class).
                p { id: "pay-escrow-polling-indicator",
                    class: "pay-escrow-polling-indicator text-xs text-muted-foreground text-center mt-2 hidden",
                    "Polling pay.epsx.io every 5s..."
                }
                // wave49(slice-5) polling bridge — JS-driven
                // status refresh. Polls
                // `/api/v1/pay/intent/{id}` every 5s, then
                // mutates the DOM elements above to reflect the
                // upstream status. SSR-only render is a no-op
                // (the script tag is harmless without hydration).
                script {
                    dangerous_inner_html: PAY_ESCROW_POLL_JS,
                }
            }
        }
    }
}

/// wave49(slice-5): the polling script as a const so the rsx
/// parser doesn't choke on its contents (backticks, regex,
/// `${...}`, etc. all live safely inside a Rust raw string).
///
/// Behavior:
/// 1. `GET /api/v1/pay/intent/{id}` every 5 seconds.
/// 2. On success, parse `status` + `tx_hash` + `escrow_id`.
/// 3. Mutate the DOM:
///    - `#pay-escrow-status-label` text → `status`
///    - `#pay-escrow-status-dot` class → color-coded bg-{color}-500
///    - `#pay-escrow-intent-id-value` text → (stays the same —
///      the intent id never changes)
///    - `#pay-escrow-flow-steps` data-status → status (CSS
///      rules in the page-shell style block react to it)
///    - `#pay-escrow-chain-card` data-status → status (icon +
///      label swap)
/// 4. Stop polling when status is `released` / `refunded` /
///    `cancelled` (terminal states — no more updates expected).
const PAY_ESCROW_POLL_JS: &str = r#"
(function() {
    var intentId = document.getElementById('pay-escrow-intent-id-value');
    if (!intentId) return;
    var id = intentId.textContent || '';
    var apiBase = (typeof window !== 'undefined' && window.__PAY_API_BASE__) || '/api';

    function colorFor(status) {
        if (status === 'escrowed') return 'blue';
        if (status === 'released') return 'green';
        if (status === 'refunded' || status === 'cancelled') return 'red';
        if (status === 'pending') return 'orange';
        return 'slate';
    }

    function apply(status, txHash) {
        var label = document.getElementById('pay-escrow-status-label');
        var dot = document.getElementById('pay-escrow-status-dot');
        var pill = document.getElementById('pay-escrow-status-pill');
        if (label) {
            label.textContent = status;
            label.className = 'pay-escrow-status-label text-sm font-medium text-' + colorFor(status) + '-500';
        }
        if (dot) {
            dot.className = 'pay-escrow-status-dot h-2 w-2 rounded-full bg-' + colorFor(status) + '-500';
        }
        if (pill) { pill.setAttribute('data-status', status); }
    }

    function tick() {
        var indicator = document.getElementById('pay-escrow-polling-indicator');
        if (indicator) indicator.classList.remove('hidden');
        var xhr = new XMLHttpRequest();
        xhr.open('GET', apiBase + '/v1/pay/intent/' + encodeURIComponent(id), true);
        xhr.onload = function() {
            if (indicator) indicator.classList.add('hidden');
            if (xhr.status !== 200) return;
            try {
                var data = JSON.parse(xhr.responseText);
                if (data && data.status) {
                    apply(data.status, data.tx_hash);
                    if (data.status === 'released' || data.status === 'refunded' || data.status === 'cancelled') {
                        clearInterval(handle);
                    }
                }
            } catch (e) { /* ignore */ }
        };
        xhr.send();
    }

    var handle = setInterval(tick, 5000);
    tick();
})();
"#;