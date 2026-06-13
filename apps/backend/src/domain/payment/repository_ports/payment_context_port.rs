//! `PaymentContextRepositoryPort` — payment context (V2 dynamic
//! payment link) port.
//!
//! Wave 11 — Track B (outbound-leakage fold). Pre-wave-11 a
//! narrow `PaymentContextRepositoryPort` lived in
//! `repository_ports/mod.rs` (with a domain-aggregate surface
//! that no live caller exercised) and the admin
//! `web/admin/payment_link_handlers.rs` reached around it
//! entirely to use the concrete
//! `PaymentContextRepositoryAdapter` directly. Track B
//! consolidates the port surface on the concrete adapter's
//! method set, using the Diesel DTOs (`PaymentContextDb`,
//! `NewPaymentContextDb`, `UpdatePaymentContextDb`) directly so
//! the admin handler (and the new `web/payments/payment_link_handlers.rs`
//! replacement) can plug in via `Arc<dyn PaymentContextRepositoryPort>`
//! without a domain↔DB shim.
//!
//! ## Trait shape
//!
//! `Send + Sync` so the port is `Arc<dyn PaymentContextRepositoryPort>`
//! in DI graphs. `#[async_trait]` for object-safety. `AppResult<T>`
//! (re-exported from `epsx_contracts::errors`) for cross-crate
//! compatibility — same as the existing `PaymentRepositoryPort`
//! and the wave-11 `SubscriptionRepositoryPort`.
//!
//! The 9 methods map 1:1 to the concrete
//! `PaymentContextRepositoryAdapter` (see
//! `infrastructure::adapters::repositories::payment_context_repository_adapter.rs`).
//! `is_context_usable` is a free helper on the adapter module;
//! it stays a free function (no port surface), the way
//! `compute_link_hash` does.
//!
//! Audit references:
//!   - `docs/wave8-service-boundary/audit-payments.md` §3 row 1
//!     (`web/admin/payment_link_handlers.rs` reaching into
//!     `payment_context_repository_adapter` directly)
//!   - `docs/wave8-service-boundary/ROADMAP.md` §4 wave-11
//!     preconditions item 3
//!   - `docs/wave8-service-boundary/ROADMAP.md` §12 (implementation
//!     report, this wave)

use async_trait::async_trait;
use epsx_contracts::errors::AppResult;
use uuid::Uuid;

use crate::infrastructure::adapters::repositories::payment_context_repository_adapter::{
    NewPaymentContextDb, PaymentContextDb, PaymentContextSearchCriteria,
    UpdatePaymentContextDb,
};

/// Port for payment context (dynamic payment link) operations.
///
/// The methods mirror the concrete `PaymentContextRepositoryAdapter`
/// 1:1. The handler layer (admin `payment_link_handlers`) goes
/// through `Arc<dyn PaymentContextRepositoryPort>` in DI; the
/// in-process `PaymentContextRepositoryAdapter` is the default
/// implementation.
///
/// `find_by_slug` is the hot path for the public
/// `GET /api/public/payment-links/{slug}` route — the public
/// handler takes an `Arc<dyn PaymentContextRepositoryPort>`
/// from `AppState` and dispatches directly to this method.
#[async_trait]
pub trait PaymentContextRepositoryPort: Send + Sync {
    /// Save a new payment context. Returns the persisted row
    /// (with DB-generated timestamps + `version`).
    async fn save(&self, context: NewPaymentContextDb) -> AppResult<PaymentContextDb>;

    /// Find a payment context by ID.
    async fn find_by_id(&self, id: Uuid) -> AppResult<Option<PaymentContextDb>>;

    /// Find a payment context by slug. Hot path for the public
    /// `GET /api/public/payment-links/{slug}` route.
    async fn find_by_slug(&self, slug: &str) -> AppResult<Option<PaymentContextDb>>;

    /// List payment contexts matching the search criteria.
    async fn find_all(
        &self,
        criteria: PaymentContextSearchCriteria,
    ) -> AppResult<Vec<PaymentContextDb>>;

    /// Update a payment context by ID.
    async fn update(
        &self,
        id: Uuid,
        changeset: UpdatePaymentContextDb,
    ) -> AppResult<PaymentContextDb>;

    /// Soft-delete a payment context (sets `is_active = false`).
    async fn soft_delete(&self, id: Uuid) -> AppResult<()>;

    /// Increment the `current_uses` counter on a payment context.
    async fn increment_usage(&self, id: Uuid) -> AppResult<PaymentContextDb>;

    /// Count payment contexts matching the search criteria.
    async fn count(&self, criteria: PaymentContextSearchCriteria) -> AppResult<i64>;

    /// Find expired but still-active payment contexts. Used by
    /// the cleanup job (no live caller in the wave-11 source
    /// tree; kept on the port for parity with the adapter).
    async fn find_expired(&self) -> AppResult<Vec<PaymentContextDb>>;
}

// -----------------------------------------------------------------------------
// Compile-time object-safety probe.
// -----------------------------------------------------------------------------
#[cfg(test)]
mod object_safety {
    use super::*;
    use std::sync::Arc;

    /// Type-asserts that `PaymentContextRepositoryPort` is
    /// dyn-compatible by holding an
    /// `Arc<dyn PaymentContextRepositoryPort>` reference. If the
    /// trait ever gains a non-object-safe method (e.g. a generic
    /// method or a `Self` in argument position) this function
    /// will fail to compile.
    fn _assert_object_safe(_: Arc<dyn PaymentContextRepositoryPort>) {}
}
