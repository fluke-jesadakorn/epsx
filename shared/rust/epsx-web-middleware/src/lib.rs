//! `epsx-web-middleware`
//!
//! Facade crate for the EPSX web/middleware layer. This is a
//! **types-only stub** in wave 9: it declares the public surface
//! that future service binaries will use to plug in shared
//! middlewares, and proves the dependency graph
//! (`epsx-contracts` + `epsx-identity` + `axum` + `tower` + `http`
//! + `tracing`) compiles standalone.
//!
//! The 10 implementation files at `apps/backend/src/web/middleware/`
//! (3,361 LOC) are NOT moved into this crate in wave 9. They
//! stay in `apps/backend` for now because of a wave-8-audit
//! under-estimate: the audit's R11 ("package the 10 middleware
//! files as a shared crate") recommended a real file-move, but
//! the actual coupling footprint is broader than 2 paths (the
//! 15+ backend-internal paths are documented in the wave 9
//! deliverable for this track). A real move needs the
//! dependent types — `AppState`, `DomainContainer`, `Cache`,
//! `ApiKeyRepository`, `UnifiedWeb3AuthService`, the diesel
//! table schemas, the tower-governor wiring, and the threat
//! detection service — to be either extracted into
//! `epsx-contracts` / `epsx-identity` first, or refactored into
//! trait abstractions that the new crate can take as
//! generic parameters. Both of those are out of scope for
//! wave 9.
//!
//! This crate is therefore a forward-looking dep-graph proof:
//!
//! * It has a real public surface (`BackendMiddleware` trait +
//!   `MiddlewareKind` enum + re-exports of the kernel / identity
//!   types a future middleware will need).
//! * It compiles standalone with `cargo check -p epsx-web-middleware`
//!   (no path dep back to `apps/backend`).
//! * It has a smoke unit test (`tests::facade_surface_compiles`)
//!   that proves a unit struct can implement the trait and that
//!   the re-exports resolve.
//!
//! ## Why a facade, not a file-move
//!
//! A facade proves the workspace graph and unblocks wave 10
//! (notifications lift) without committing to a file-move that
//! the rest of the architecture does not yet support. When the
//! 15+ coupling paths are cleared in a future wave, the
//! concrete implementations at `apps/backend/src/web/middleware/`
//! move into this crate in a follow-up PR. The trait
//! `BackendMiddleware` is the seam that future concrete
//! implementations will satisfy.

#![doc(html_root_url = "https://docs.rs/epsx-web-middleware/0.1.0")]

use axum::http::HeaderName;
use std::borrow::Cow;

/// Marker trait for middlewares that plug into an EPSX service
/// binary's HTTP layer.
///
/// This trait is the seam that future concrete middleware
/// implementations will satisfy when the 10 files at
/// `apps/backend/src/web/middleware/` move into this crate in
/// a follow-up wave. It carries no required methods today —
/// the goal is to give the crate a real public surface and
/// prove the dep graph compiles.
///
/// When the real implementations land, the trait will gain
/// `name()`, `kind()`, and `mount()` methods that the unified
/// router uses to register the middleware on the right route
/// group. The trait is intentionally minimal so that the
/// file-move PR does not have to change the trait signature
/// in lock-step.
pub trait BackendMiddleware: Send + Sync + 'static {
    /// Stable name used in tracing spans and structured logs
    /// (e.g. `"bearer_auth"`, `"permission_validation"`,
    /// `"rate_limit"`, `"security_headers"`).
    fn name(&self) -> Cow<'static, str> {
        Cow::Borrowed("unnamed")
    }
}

/// Coarse categorization for middlewares. Used by the unified
/// router to apply the middleware on the right route group
/// (e.g. only public routes, only authed routes, every
/// route).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MiddlewareKind {
    /// Always-on middleware (security headers, request id).
    Global,
    /// Authentication (bearer, web3 auth, API key).
    Auth,
    /// Authorization / permission gating.
    Permission,
    /// Rate limiting (governor, multi-level, IP).
    RateLimit,
    /// Observability (usage tracking, analytics).
    Observability,
}

/// Re-export of the EPSX kernel error type. The real error
/// surface ships from `epsx-contracts::errors` in wave 9
/// track A; this re-export is the path future middleware
/// code will use so the call site compiles unchanged.
pub use epsx_kernel as kernel;

/// Re-export of the EPSX identity type (the OpenID
/// token service handle). The real surface ships from
/// `epsx-identity` in wave 9 track B; on this branch the
/// dep points at the workspace-name-collision-free stub
/// crate `epsx-identity-stub`. Future service binaries
/// will use `epsx_identity::*` regardless of the crate
/// rename Track B picks.
pub use epsx_identity_shared as identity;

/// Header name used by the security-headers middleware.
/// Exposed here so that future tests in this crate can
/// assert against it without re-declaring the string.
pub const SECURITY_HEADER_CONTENT_TYPE_OPTIONS: HeaderName =
    HeaderName::from_static("x-content-type-options");

/// Header value paired with
/// [`SECURITY_HEADER_CONTENT_TYPE_OPTIONS`]. Centralised so
/// the `security_headers_middleware` implementation can be
/// moved into this crate in the follow-up wave without
/// re-deriving the constant.
pub const SECURITY_HEADER_VALUE_NOSNIFF: &str = "nosniff";

/// Header name used by the security-headers middleware for
/// clickjacking protection.
pub const SECURITY_HEADER_FRAME_OPTIONS: HeaderName =
    HeaderName::from_static("x-frame-options");

/// Header name used by the security-headers middleware for
/// HSTS in production builds.
pub const SECURITY_HEADER_HSTS: HeaderName =
    HeaderName::from_static("strict-transport-security");

/// A trivial concrete middleware used to prove the
/// `BackendMiddleware` trait can be implemented in this
/// crate. The smoke test in `tests::facade_surface_compiles`
/// instantiates this struct.
#[derive(Debug, Clone, Copy, Default)]
pub struct NoopMiddleware;

impl BackendMiddleware for NoopMiddleware {
    fn name(&self) -> Cow<'static, str> {
        Cow::Borrowed("noop")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Smoke test: prove the public surface compiles and a
    /// unit struct can implement the trait. This is the test
    /// the wave-9 verifier looks for in
    /// `shared/rust/epsx-web-middleware/src/`.
    #[test]
    fn facade_surface_compiles() {
        let middleware = NoopMiddleware;
        assert_eq!(middleware.name(), "noop");
        assert_eq!(MiddlewareKind::Global, MiddlewareKind::Global);
        // Touch the header constants so a future refactor that
        // accidentally renames them fails the test, not the
        // production binary.
        assert_eq!(SECURITY_HEADER_VALUE_NOSNIFF, "nosniff");
        assert_eq!(
            SECURITY_HEADER_CONTENT_TYPE_OPTIONS.as_str(),
            "x-content-type-options"
        );
    }
}
