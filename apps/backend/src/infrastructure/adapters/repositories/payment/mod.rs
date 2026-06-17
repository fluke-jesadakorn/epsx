//! In-process payment-bounded-context repository adapters.
//!
//! Wave 11 — Track B (outbound-leakage fold). Pre-wave-11 these
//! adapters sat in the central `infrastructure::adapters::repositories`
//! tree (or, in the case of `payment_context_repository_adapter`,
//! next to the payment adapter but at the central layer). Track B
//! groups the *subscription* adapter under this `payment/` subdir
//! so the bounded-context ownership is explicit on disk; the
//! `payment_repository_adapter`, `payment_context_repository_adapter`,
//! and `credit_repository_adapter` files are still at the central
//! layer for now (out of scope for this track) and are re-exported
//! here as a future-move marker.
//!
//! Audit references:
//!   - `docs/wave8-service-boundary/audit-payments.md` §3 row 1
//!     and row 3
//!   - `docs/wave8-service-boundary/ROADMAP.md` §4 wave-11
//!     preconditions item 3
//!   - `docs/wave8-service-boundary/ROADMAP.md` §12 (implementation
//!     report, this wave)

// The subscription adapter is the only one moved into this
// subdir by wave 11 / Track B. The other payment adapters stay
// at the central layer for one more wave.
pub mod subscription_repository_adapter;

// Re-exports of the payment / payment-context / credit adapters
// that still live at the central layer. They are surfaced under
// this `payment::` subdir as a forward-move marker so future
// `use` sites can write the destination path now; the actual
// file moves are wave-12+ work.
pub use super::payment_context_repository_adapter::{
    is_context_usable, NewPaymentContextDb, PaymentContextDb, PaymentContextRepositoryAdapter,
    PaymentContextSearchCriteria, UpdatePaymentContextDb,
};
pub use super::payment_repository_adapter::PaymentRepositoryAdapter;
pub use super::credit_repository_adapter::CreditRepositoryAdapter;
pub use subscription_repository_adapter::{
    PaymentSubscriptionRepositoryAdapter, SubscriptionSearchCriteria,
};
