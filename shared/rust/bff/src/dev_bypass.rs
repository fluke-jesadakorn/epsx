//! Dev-only auth bypass.
//!
//! When the env var `EPSX_DEV_AUTH_BYPASS=1` is set, BFFs treat every
//! incoming request as logged in as a hardcoded dev admin. This is for
//! the local pixel-recheck / visual-diff workflow — you don't want to
//! wire up SIWE, mint a JWT, set cookies, and click through a wallet
//! popup just to see what the dashboard looks like.
//!
//! The bypass is checked at the very top of each BFF's `current_user` /
//! `require_user` / `require_admin` / `require_editor` helpers. When the
//! env var is UNSET, behavior is unchanged from the prior shape — every
//! function falls through to the JWT-verify path. **No code revert is
//! needed to turn it off: unset the env var and restart.**
//!
//! Safety: the env var name has `EPSX_DEV_*` prefix and a one-line
//! startup log line is emitted so it's visible in process logs. Never
//! set this in production.
//!
//! The hardcoded user:
//! - `user_id`: `dev-bypass`
//! - `address`: `0x000000000000000000000000000000000000d3v1` (matches
//!   the 20-byte Ethereum address format so downstream wallets
//!   render it without validation errors)
//! - `chain_id`: `0x38` (BSC mainnet; matches the default in the
//!   wallet shim)
//! - `roles`: `["admin", "super_admin"]` — both admin role strings
//!   the BFFs recognize, so `permissions_for_roles` produces the
//!   full admin perm set on both BFFs.

use epsx_auth::AuthUser;

/// The env var that flips on the bypass. Set to `"1"` to enable.
/// Any other value is treated as unset.
pub const DEV_BYPASS_ENV: &str = "EPSX_DEV_AUTH_BYPASS";

/// Wave 24 t3p — env var that forces the dev BFF to behave as if the
/// request is unauthenticated, even when `EPSX_DEV_AUTH_BYPASS=1` is
/// also set. This is the pixel-recheck escape hatch for the
/// `redirect-chain-differs` issue: with the bypass on, every request
/// is treated as authed → `current_user()` returns `Some(dev-admin)`
/// → `user.is_none()` is always `false` → the SSR's
/// `needs_unauth_redirect` branch never fires → the dev baseline
/// differs from prod (prod is unauth, so prod 307-redirects to
/// `/auth?return_url=…` for protected pages).
///
/// Setting `EPSX_DEV_AUTH_FORCE_UNAUTH=1` flips that single bit:
/// `current_user()` returns `None` even though the dev-admin cookie
/// is still set on the request. The pixel-recheck harness can then
/// run with both vars set:
///
/// ```bash
/// EPSX_DEV_AUTH_BYPASS=1 EPSX_DEV_AUTH_FORCE_UNAUTH=1 \
///     bash tools/e2e/capture-dev.sh
/// ```
///
/// and get a dev baseline that 307-redirects the same way prod does
/// for protected pages, so the redirect chains match.
///
/// When `EPSX_DEV_AUTH_FORCE_UNAUTH=1` is set WITHOUT
/// `EPSX_DEV_AUTH_BYPASS=1`, behavior is the same as the bypass-off
/// default — every request falls through to the normal JWT-verify
/// path. The "force" semantics only kick in when the bypass is
/// already on.
///
/// Default is OFF, no behavior change when unset.
pub const DEV_FORCE_UNAUTH_ENV: &str = "EPSX_DEV_AUTH_FORCE_UNAUTH";

/// Returns `true` if the force-unauth flag is set, `false` otherwise.
/// Caller should treat this as a one-bit override on top of
/// `dev_bypass_user()`: when both the bypass and the force-unauth
/// flag are on, the caller must return `None` instead of
/// `dev_bypass_user()`.
pub fn is_dev_force_unauth_enabled() -> bool {
    std::env::var(DEV_FORCE_UNAUTH_ENV).ok().as_deref() == Some("1")
}

/// Hardcoded dev-only user. Returned by `dev_bypass_user()` when the
/// env var is set. NOT for production use.
fn dev_user() -> AuthUser {
    AuthUser {
        user_id: "dev-bypass".to_string(),
        address: "0x000000000000000000000000000000000000d3v1".to_string(),
        chain_id: "0x38".to_string(),
        // `admin` covers frontend BFF's `is_admin_role` (`admin` |
        // `super_admin`); we emit both so the BFFs and any future
        // role checks see the same shape.
        roles: vec!["admin".to_string(), "super_admin".to_string()],
    }
}

/// Returns `Some(AuthUser)` if the bypass is enabled, `None` otherwise.
///
/// Callers should short-circuit the normal JWT-verify path when this
/// returns `Some`, e.g.:
///
/// ```ignore
/// pub fn current_user(headers, jwt) -> Option<AuthUser> {
///     if let Some(user) = epsx_bff::dev_bypass::dev_bypass_user() {
///         return Some(user);
///     }
///     // ...normal JWT-verify path
/// }
/// ```
pub fn dev_bypass_user() -> Option<AuthUser> {
    let on = std::env::var(DEV_BYPASS_ENV).ok().as_deref() == Some("1");
    if on {
        Some(dev_user())
    } else {
        None
    }
}

/// Returns true if the bypass is enabled. Useful for emitting a
/// one-line startup banner in main() so it's obvious in process logs
/// when the bypass is hot. Caller controls the log level.
pub fn is_dev_bypass_enabled() -> bool {
    std::env::var(DEV_BYPASS_ENV).ok().as_deref() == Some("1")
}

#[cfg(test)]
mod tests {
    //! The dev bypass is the kind of code that should be impossible
    //! to leave on by accident. These tests pin:
    //! - default OFF (unset env var)
    //! - any value other than `"1"` is OFF
    //! - `"1"` flips it ON and the returned user is the hardcoded admin
    //! - the helper is idempotent (callable from multiple BFFs in the
    //!   same process without state issues — it just reads the env var
    //!   every call).

    use super::*;
    use std::sync::Mutex;

    // `std::env::set_var` is not thread-safe; serialize all tests that
    // mutate the env. (per memory/tokio-runtime-quirks.md)
    static ENV_LOCK: Mutex<()> = Mutex::new(());

    #[test]
    fn off_by_default() {
        let _g = ENV_LOCK.lock().unwrap();
        std::env::remove_var(DEV_BYPASS_ENV);
        assert!(dev_bypass_user().is_none());
        assert!(!is_dev_bypass_enabled());
    }

    #[test]
    fn off_when_set_to_other_values() {
        let _g = ENV_LOCK.lock().unwrap();
        for v in ["0", "true", "yes", "on", "TRUE", "1\n"] {
            std::env::set_var(DEV_BYPASS_ENV, v);
            // `as_deref() == Some("1")` is strict — only literal "1" turns it on.
            // (Side note: "1\n" is what `env` would produce if the user
            //  added a trailing newline; we don't tolerate it.)
            let expected = v == "1";
            assert_eq!(
                dev_bypass_user().is_some(),
                expected,
                "value {v:?} should give expected={expected}"
            );
        }
        std::env::remove_var(DEV_BYPASS_ENV);
    }

    #[test]
    fn on_when_set_to_one() {
        let _g = ENV_LOCK.lock().unwrap();
        std::env::set_var(DEV_BYPASS_ENV, "1");
        let u = dev_bypass_user().expect("bypass should be on");
        assert_eq!(u.user_id, "dev-bypass");
        assert_eq!(u.address, "0x000000000000000000000000000000000000d3v1");
        assert_eq!(u.chain_id, "0x38");
        assert!(u.is_admin());
        assert!(u.roles.contains(&"admin".to_string()));
        assert!(u.roles.contains(&"super_admin".to_string()));
        std::env::remove_var(DEV_BYPASS_ENV);
    }

    #[test]
    fn idempotent_returns_same_user_each_call() {
        let _g = ENV_LOCK.lock().unwrap();
        std::env::set_var(DEV_BYPASS_ENV, "1");
        let a = dev_bypass_user().unwrap();
        let b = dev_bypass_user().unwrap();
        assert_eq!(a.user_id, b.user_id);
        assert_eq!(a.roles, b.roles);
        std::env::remove_var(DEV_BYPASS_ENV);
    }

    #[test]
    fn turn_off_then_on_works() {
        let _g = ENV_LOCK.lock().unwrap();
        std::env::remove_var(DEV_BYPASS_ENV);
        assert!(dev_bypass_user().is_none());
        std::env::set_var(DEV_BYPASS_ENV, "1");
        assert!(dev_bypass_user().is_some());
        std::env::remove_var(DEV_BYPASS_ENV);
        assert!(dev_bypass_user().is_none());
    }

    // === Wave 24 t3p — force-unauth env var ===

    #[test]
    fn force_unauth_off_by_default() {
        let _g = ENV_LOCK.lock().unwrap();
        std::env::remove_var(DEV_FORCE_UNAUTH_ENV);
        assert!(!is_dev_force_unauth_enabled());
    }

    #[test]
    fn force_unauth_strict_one() {
        let _g = ENV_LOCK.lock().unwrap();
        for v in ["0", "true", "yes", "on", "TRUE", "1\n"] {
            std::env::set_var(DEV_FORCE_UNAUTH_ENV, v);
            let expected = v == "1";
            assert_eq!(
                is_dev_force_unauth_enabled(),
                expected,
                "value {v:?} should give expected={expected}"
            );
        }
        std::env::remove_var(DEV_FORCE_UNAUTH_ENV);
    }
}
