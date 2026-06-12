//! `epsx-contracts` — WAVE 9 / TRACK A STUB
//!
//! Real contents ship from `wave9/epsx-contracts`. This stub
//! exists on the `wave9/epsx-web-middleware` branch ONLY so
//! the `epsx-web-middleware` path dep resolves and the
//! workspace graph stays consistent under
//! `cargo check --workspace --bins`.
//!
//! See `wave9/epsx-contracts`'s deliverable.md for the real
//! shared-kernel surface (errors, permissions, constants,
//! traits, value objects).

#![doc = "Stub crate. Real surface ships from wave9/epsx-contracts."]

/// Stub permissions module. The real `epsx_contracts::permissions`
/// ships from Track A with `has_permission`, `is_admin`,
/// `has_any_permission`, `permission_platform`, and
/// `has_admin_platform_permission`.
pub mod permissions {
    /// Stub — real implementation ships from Track A.
    pub fn has_permission(granted: &[String], required: &str) -> bool {
        granted.iter().any(|p| p == required)
    }
}

/// Stub constants module. The real `epsx_contracts::constants`
/// ships from Track A.
pub mod constants {
    /// Stub — real constant ships from Track A.
    pub const MINUTE: u64 = 60;
    /// Stub — real constant ships from Track A.
    pub const HOUR: u64 = 60 * 60;
    /// Stub — real constant ships from Track A.
    pub const DAY: u64 = 24 * 60 * 60;
    /// Stub — real constant ships from Track A.
    pub const FREE_PLAN_ID: &str = "free";
    /// Stub — real constant ships from Track A.
    pub const FREE_PLAN_NAME: &str = "Free";
    /// Stub — real constant ships from Track A.
    pub const FREE_PLAN_RANKING_OFFSET: i32 = 0;
    /// Stub — real function ships from Track A.
    pub fn is_system_admin_plan(_plan_id: &str) -> bool {
        false
    }
}

/// Stub errors module. The real `epsx_contracts::errors` ships
/// from Track A.
pub mod errors {
    /// Stub error type.
    #[derive(Debug, thiserror::Error)]
    pub enum AppError {
        #[error("not implemented (stub)")]
        NotImplemented,
    }

    /// Stub result alias.
    pub type AppResult<T> = Result<T, AppError>;
}
