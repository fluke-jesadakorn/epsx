//! Runtime-only safety gates shared by the browser-facing services.

/// Allow a debug binary to exercise production runtime semantics against
/// loopback-only E2E fixtures. Release binaries can never enable this gate,
/// even if the environment variable is present.
pub fn production_shaped_e2e_enabled() -> bool {
    production_shaped_e2e_enabled_for(
        cfg!(debug_assertions),
        std::env::var("EPSX_PRODUCTION_SHAPED_E2E").ok().as_deref(),
    )
}

fn production_shaped_e2e_enabled_for(debug_build: bool, value: Option<&str>) -> bool {
    debug_build && value == Some("1")
}

#[cfg(test)]
mod tests {
    use super::production_shaped_e2e_enabled_for;

    #[test]
    fn production_shaped_gate_requires_debug_build_and_explicit_opt_in() {
        assert!(production_shaped_e2e_enabled_for(true, Some("1")));
        assert!(!production_shaped_e2e_enabled_for(false, Some("1")));
        assert!(!production_shaped_e2e_enabled_for(true, Some("true")));
        assert!(!production_shaped_e2e_enabled_for(true, None));
    }
}
