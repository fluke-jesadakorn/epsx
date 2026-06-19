//! `payment` domain subdir — 5+ components ported from
//! `apps-old/frontend/components/payment/`:
//! - `chain_verification_card` (427 LoC)
//! - `current_access_card`    (214 LoC)
//! - `payment_flow_steps`     (783 LoC)
//! - `plan_comparison_card`   (525 LoC)
//! - `unified_payment_flow`   (252 LoC)
//! - `upgrade_banner`         (181 LoC)
//!
//! All 6 components are ported as visual stubs with typed prop
//! shapes that the page-level BFF call sites can fill in from
//! the payment / subscription service clients.

use dioxus::prelude::*;

pub mod chain_verification_card;
pub mod current_access_card;
pub mod payment_flow_steps;
pub mod plan_comparison_card;
pub mod unified_payment_flow;
pub mod upgrade_banner;

pub use chain_verification_card::{ChainVerificationCard, ChainVerificationStatus};
pub use current_access_card::{CurrentAccessCard, CurrentAccessInfo};
pub use payment_flow_steps::{PaymentFlowStep, PaymentFlowSteps, PaymentFlowStepsState};
pub use plan_comparison_card::{PlanComparisonCard, PlanComparisonRow};
pub use unified_payment_flow::{UnifiedPaymentFlow, UnifiedPaymentStep};
pub use upgrade_banner::{UpgradeBanner, UpgradeUrgency};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn payment_module_re_exports_resolve() {
        // Smoke test: all 6 NEW payment components are exported
        // from `crate::payment::*`. Compile-time check.
        
        
        
        
        
        
    }
}
