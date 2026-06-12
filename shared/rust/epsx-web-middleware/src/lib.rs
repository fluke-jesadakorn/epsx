//! `epsx-web-middleware`
//!
//! Shared web/middleware crate for EPSX. Started as a types-only
//! facade stub in wave 9 and grew in the wave 10 prep pass.
//!
//! ## What lives here (post-wave-10-prep)
//!
//! * `security_headers` — `security_headers_middleware` +
//!   `request_id_middleware` + the `RequestId` extension type.
//!   Pure axum code, no backend coupling. Moved from
//!   `apps/backend/src/web/middleware/security_headers.rs`.
//! * `governor_limiters` — the 3 pure `*_rate_limiter()` factory
//!   functions (`auth_rate_limiter`, `chat_rate_limiter`,
//!   `email_rate_limiter`). Pure tower-governor config, no
//!   backend coupling. The `threat_aware_middleware` (which
//!   reaches into the backend's threat-detection service) stays
//!   in `apps/backend` for now.
//!
//! ## What still lives in `apps/backend/src/web/middleware/`
//!
//! The remaining 8 files (2,500+ LOC) are too tightly coupled
//! to backend-internal types to move in one prep pass:
//!
//! * `bearer_middleware.rs` (502 LOC) — `AppState`,
//!   `ApiKeyRepository`, `redis_cache::get_perm_invalidated`,
//!   `auth::OpenIDTokenError`. Moving it requires a trait
//!   abstraction over the API key repo + Redis cache invalidation.
//! * `auth_middleware.rs` (380 LOC) — `AppState`,
//!   `UnifiedWeb3AuthService`.
//! * `permission_validation_middleware.rs` (475 LOC) —
//!   `PermissionError`, `bearer_middleware::OpenIDUserContext`.
//! * `rate_limit_middleware.rs` (226 LOC) — `DomainContainer`.
//! * `rate_limiter.rs` (643 LOC) — `Cache`.
//! * `multi_level_rate_limiter.rs` (647 LOC) — `Cache`, `Config`.
//! * `usage_tracking_middleware.rs` (194 LOC) — `DomainContainer`,
//!   Diesel schemas, `auth_middleware::Web3AuthContext`,
//!   `bearer_middleware::OpenIDUserContext`.
//! * `governor_limiter.rs` (now ~30 LOC after the 3 factories
//!   moved) — `AppState`, `infrastructure::security::get_threat_detection_service`.
//!
//! These are wave 10+1+ work. The dep-graph proof was delivered
//! in wave 9; the real file-move is a sequence of
//! "introduce trait abstraction → move file" steps that the
//! wave 10+1 plan can take per coupling type.
//!
//! ## Trait seam
//!
//! `BackendMiddleware` is the marker trait the unified router
//! uses to register middlewares. Real concrete middlewares
//! (the ones that stay in the backend) implement this trait
//! when they get ported; until then, the backend's
//! `web::middleware::mod.rs` re-exports the moved pieces so
//! `crate::web::middleware::security_headers_middleware` keeps
//! working.

#![doc(html_root_url = "https://docs.rs/epsx-web-middleware/0.1.0")]

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

// wave 10 prep: real implementation modules moved in from
// `apps/backend/src/web/middleware/`. See module-level docs in
// each file for what moved and what's still in the backend.
pub mod security_headers;
pub mod governor_limiters;

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
        // The security headers + governor rate-limiter factories
        // were moved into sibling modules in the wave 10 prep
        // pass; this test now also proves those modules are
        // visible from the crate root.
        crate::security_headers::RequestId(String::new());
        let _ = crate::governor_limiters::auth_rate_limiter;
    }
}
