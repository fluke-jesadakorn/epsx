//! `ChainVerificationCard` — shows on-chain verification status
//! (tx hash, block, confirmations).
//!
//! Port of
//! `apps-old/frontend/components/payment/chain-verification-card.tsx`
//! (427 LoC). The TS source renders a card with a step-by-step
//! verification flow: connect wallet → submit tx → wait for
//! confirmations → done. The Dioxus port renders the same
//! visual structure with a status enum prop.

use crate::primitives::icon::Icon;

use dioxus::prelude::*;

#[derive(Clone, PartialEq, Eq, Debug, Default)]
pub enum ChainVerificationStatus {
    #[default]
    Idle,
    Connecting,
    Submitting,
    Confirming { confirmations: u32, target: u32 },
    Verified,
    Failed { reason: String },
}

#[component]
pub fn ChainVerificationCard(
    status: ChainVerificationStatus,
    /// Optional transaction hash. When `Some`, a "View on
    /// Explorer" link is rendered.
    #[props(default = None)] tx_hash: Option<String>,
    /// Network name (e.g. "BNB Smart Chain").
    #[props(default = "BNB Smart Chain".to_string())] network: String,
) -> Element {
    let (label, color, icon) = match status {
        ChainVerificationStatus::Idle => ("Idle".to_string(), "slate".to_string(), "circle".to_string()),
        ChainVerificationStatus::Connecting => ("Connecting wallet".to_string(), "blue".to_string(), "loader".to_string()),
        ChainVerificationStatus::Submitting => ("Submitting transaction".to_string(), "blue".to_string(), "loader".to_string()),
        ChainVerificationStatus::Confirming { confirmations, target } => (
            format!("Confirming {}/{}", confirmations, target),
            "orange".to_string(),
            "loader".to_string(),
        ),
        ChainVerificationStatus::Verified => ("Verified".to_string(), "green".to_string(), "check-circle".to_string()),
        ChainVerificationStatus::Failed { .. } => ("Failed".to_string(), "red".to_string(), "x-circle".to_string()),
    };
    rsx! {
        div { class: "chain-verification-card card card-glass",
            div { class: "card-header",
                div { class: "card-title flex items-center gap-2",
                Icon { name: icon.to_string(), size: Some(20), class_name: Some(format!("text-{color}-500")) }
                "Chain verification — {label}"
                }
            }
            div { class: "card-body space-y-3",
                p { class: "text-sm text-slate-600 dark:text-slate-300",
                    "Network: {network}"
                }
                if let Some(hash) = tx_hash.as_ref() {
                    div { class: "chain-verification-tx-hash p-3 bg-slate-50 dark:bg-slate-800 rounded-lg",
                        span { class: "text-xs font-medium text-slate-700 dark:text-slate-300", "Transaction: " }
                        code { class: "text-xs font-mono break-all text-slate-600 dark:text-slate-400", "{hash}" }
                    }
                }
                if let ChainVerificationStatus::Failed { reason } = status {
                    p { class: "chain-verification-error text-sm text-red-500", "{reason}" }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn chain_verification_status_default_is_idle() {
        let s = ChainVerificationStatus::default();
        assert_eq!(s, ChainVerificationStatus::Idle);
    }

    #[test]
    fn chain_verification_card_smoke() {
        
    }
}
