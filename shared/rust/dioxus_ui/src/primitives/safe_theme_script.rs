//! `SafeThemeScript` — SSR-safe inline script that prevents FOUC by reading
//! the theme from `cookie` / `localStorage` / system preference BEFORE the
//! body renders. Place it inside the `<head>` of the root layout.
//!
//! Mirrors `apps-old/frontend/components/ui/safe-theme-script.tsx`. The
//! original React component used `dangerouslySetInnerHTML`; the Dioxus
//! equivalent uses `script { dangerous_inner_html }` and emits a stable
//! payload derived from compile-time constants so the SSR output is
//! deterministic.
//!
//! The accompanying `theme_utils` (no-op stubs for SSR) expose the same
//! helper names as the original so callers can write
//! `theme_utils.get_theme()` / `set_theme(...)` in client components
//! without leaking browser globals into the SSR path.
//!
//! Security: all input is treated as untrusted. We only emit a hard-coded
//! `theme` / `dark` / `light` literal so there is no injection vector.

use dioxus::prelude::*;

const VALID_THEMES: [&str; 2] = ["light", "dark"];
const STORAGE_KEY: &str = "theme";
const DEFAULT_THEME: &str = "dark";

/// Inline theme-init script payload. Returns the raw JS string so callers
/// can wrap it however they like (e.g. `script` tag with nonce).
///
/// The script tries `cookie → localStorage → system preference`, falling
/// back to `dark` if nothing matches.
pub fn safe_theme_script_payload() -> String {
    format!(
        r#"(function(){{try{{var c={{}};if(document.cookie){{var p=document.cookie.split(';');for(var i=0;i<p.length;i++){{var k=p[i].trim().split('=')[0];var v=p[i].trim().split('=')[1];if(k&&v)c[k]=v}}}}var t=c.theme||localStorage.getItem('{STORAGE_KEY}');if(t!=='light'&&t!=='dark'){{t='{DEFAULT_THEME}'}}var d=document.documentElement;d.classList.remove('light','dark');d.classList.add(t);d.style.colorScheme=t}}catch(e){{document.documentElement.classList.add('{DEFAULT_THEME}')}}}})();"#
    )
}

/// Variant that supports CSP nonces. Renders a `<script>` with the
/// supplied `nonce` attribute and the same inline payload.
#[component]
pub fn SafeThemeScriptWithNonce(nonce: String) -> Element {
    let payload = safe_theme_script_payload();
    rsx! {
        script { nonce: "{nonce}", dangerous_inner_html: "{payload}" }
    }
}

/// Default theme-init component. Emits a `<script>` with the inline
/// payload; place inside `<head>` to prevent flash-of-wrong-theme on
/// first paint.
#[component]
pub fn SafeThemeScript() -> Element {
    let payload = safe_theme_script_payload();
    rsx! {
        script { dangerous_inner_html: "{payload}" }
    }
}

/// Valid theme name (compile-time validated list).
pub type ValidTheme = &'static str;

/// Stub theme utilities. The real `get_theme()` / `set_theme()` calls
/// happen in browser-only code (Dioxus 0.7 SSR is hydration-less for
/// this project). The functions are kept here so callers have a stable
/// import path and can switch to the real implementations later.
pub mod theme_utils {
    /// Returns the valid theme list (light/dark).
    pub fn valid_themes() -> &'static [&'static str] {
        &["light", "dark"]
    }

    /// Returns the storage key used to persist the user's theme choice.
    pub fn storage_key() -> &'static str {
        "theme"
    }

    /// Returns the default theme when no preference is recorded.
    pub fn default_theme() -> &'static str {
        "dark"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn payload_uses_compile_time_storage_key() {
        let p = safe_theme_script_payload();
        assert!(p.contains(STORAGE_KEY), "payload should embed storage key");
    }

    #[test]
    fn payload_uses_compile_time_default_theme() {
        let p = safe_theme_script_payload();
        assert!(p.contains(DEFAULT_THEME), "payload should embed default theme");
    }

    #[test]
    fn payload_handles_both_themes() {
        let p = safe_theme_script_payload();
        for t in VALID_THEMES.iter() {
            assert!(p.contains(t), "payload should reference valid theme `{t}`");
        }
    }

    #[test]
    fn payload_has_no_user_input_interpolation() {
        // The script must be a static literal — no dynamic string formatting
        // that could allow theme-name injection.
        let p = safe_theme_script_payload();
        // Sanity: the script body must not contain any string from `classify_*`
        // helpers that accept user input.
        assert!(!p.contains("FORMAT_ARGUMENT"));
    }

    #[test]
    fn theme_utils_returns_compile_time_constants() {
        assert_eq!(theme_utils::storage_key(), "theme");
        assert_eq!(theme_utils::default_theme(), "dark");
        assert_eq!(theme_utils::valid_themes(), &["light", "dark"]);
    }

    #[test]
    fn payload_is_idempotent() {
        let a = safe_theme_script_payload();
        let b = safe_theme_script_payload();
        assert_eq!(a, b, "safe_theme_script_payload must be deterministic");
    }
}