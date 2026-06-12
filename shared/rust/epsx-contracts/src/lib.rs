//! EPSX shared kernel contracts.
//!
//! This crate is the canonical home for cross-cutting types that are shared
//! between every EPSX service: the unified `AppError`, the permission
//! wildcard-matchers, the plan/permission constants, the domain-driven
//! traits (`AggregateRoot`, `DomainEvent`, `Specification`, `ValueObject`),
//! the set of value objects, and telemetry primitives.
//!
//! ## Why a separate crate?
//!
//! The wave-8 service-boundary audit
//! (`docs/wave8-service-boundary/ROADMAP.md`) recommends that the shared
//! kernel become a workspace member *before* any service is lifted, so
//! that services do not have to depend on the full `epsx` binary crate to
//! import these types. This crate is the first step of that extraction.
//!
//! ## Re-export shims
//!
//! While the extraction is in progress, the original `apps/backend/src/core/*`
//! and `apps/backend/src/domain/shared_kernel/*` files still re-export every
//! item from this crate. The 146-file `use crate::core::errors::*` →
//! `use epsx_contracts::*` bulk rename is wave-10 work and is intentionally
//! *not* part of this track.

#![allow(clippy::needless_return)]

pub mod errors;
pub mod permissions;
pub mod constants;
pub mod telemetry;
pub mod traits;
pub mod value_object;
pub mod value_objects;

// wave10(track-c): cross-cutting kernel-level ports (ROADMAP §5 R1 + R6).
// These traits are the *stable contract* that the future epsx-identity
// binary will serve over HTTP / gRPC. The in-process adapters in
// apps/backend/src/infrastructure/adapters/permission/ wrap
// UnifiedPermissionService 1:1 today.
pub mod permission_authority_port;
pub mod wallet_ranking_offset_query;

// Re-export the value-object trait + error at the crate root for ergonomics,
// mirroring what `shared_kernel/mod.rs` did before the extraction.
pub use value_object::{ValueObject, ValueObjectError};
