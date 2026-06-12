//! Wave 10 (Track C) — kernel-level port adapters.
//!
//! Each adapter is a 1:1 wrapper around the corresponding
//! `UnifiedPermissionService` method. No new logic. The
//! non-trivial work is the DTO conversion (in the authority
//! adapter) and the `i32 → RankingOffset` value-object
//! conversion (in the ranking-offset adapter).
//!
//! Wiring lives in `infrastructure/container/simple_container.rs`
//! and `infrastructure/container/stateless_service_factory.rs`:
//!   - `UnifiedPermissionService` is still constructed once and
//!     shared.
//!   - The two adapters are constructed on top of the same
//!     `Arc<UnifiedPermissionService>`.
//!   - `web/payments/validation_handlers.rs`,
//!     `web/analytics/eps/rankings.rs`, and
//!     `web/analytics/eps/cache.rs` will be migrated to take
//!     the trait object instead of the concrete service in
//!     subsequent commits of this same branch.

pub mod in_process_authority_adapter;
pub mod in_process_ranking_offset_adapter;
