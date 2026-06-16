use dioxus::prelude::*;

/// Theme + design tokens — exposes the same CSS variables the Next.js app uses.
///
/// Wave 23 T4 v2: rewritten to use INLINE `onclick="epsx.toggleTheme()"`
/// (via `epsx_templates::global_js`), not a Dioxus `onclick:` closure. The
/// closure was being stripped by SSR (Dioxus is hydration-less in this
/// project per `docs/wave3a-wiring/design.md`), so the toggle button was
/// visually present but a no-op at runtime. The inline pattern works
/// because the JS function lives in the global script block that
/// `epsx_templates::global_js()` emits on every page.
///
/// This module now:
/// 1. Ships the CSS variables (light + dark) — unchanged.
/// 2. `ThemeRoot` injects a one-shot inline script that reads the persisted
///    theme from `localStorage` (or `prefers-color-scheme`) and applies the
///    `dark` class to `<html>` before paint, eliminating the FOUC the prod
///    `theme-toggle.tsx` warns about.
/// 3. `UnifiedThemeToggle` is the click target the navbar (and the auth
///    pages) render. It now renders the button as raw HTML via
///    `dangerous_inner_html` with `onclick="epsx.toggleTheme()"` baked in,
///    so the click handler actually fires on the client.
/// 4. `use_theme` exposes a reactive `Signal<ThemeMode>` so other components
///    (e.g. the admin settings page) can reflect the active mode.
pub const EPSX_CSS_VARS: &str = r#"
:root {
  --background: 0 0% 100%;
  --foreground: 222.2 47.4% 11.2%;
  --muted: 210 40% 96.1%;
  --muted-foreground: 215.4 16.3% 46.9%;
  --card: 0 0% 100%;
  --card-foreground: 222.2 47.4% 11.2%;
  --border: 214.3 31.8% 91.4%;
  --input: 214.3 31.8% 91.4%;
  --primary: 222.2 47.4% 11.2%;
  --primary-foreground: 210 40% 98%;
  --secondary: 210 40% 96.1%;
  --secondary-foreground: 222.2 47.4% 11.2%;
  --accent: 210 40% 96.1%;
  --accent-foreground: 222.2 47.4% 11.2%;
  --destructive: 0 84.2% 60.2%;
  --destructive-foreground: 210 40% 98%;
  --ring: 215 20.2% 65.1%;
  --radius: 0.5rem;
  --epsx-blue-start: #488BFA;
  --epsx-blue-end: #A43FF3;
  --epsx-cyan: #3FC9F3;
  --epsx-purple: #A43FF3;
}
.dark {
  --background: 224 71% 4%;
  --foreground: 213 31% 91%;
  --muted: 223 47% 11%;
  --muted-foreground: 215.4 16.3% 56.9%;
  --card: 224 71% 4%;
  --card-foreground: 213 31% 91%;
  --border: 216 34% 17%;
  --input: 216 34% 17%;
  --primary: 210 40% 98%;
  --primary-foreground: 222.2 47.4% 1.2%;
  --secondary: 222.2 47.4% 11.2%;
  --secondary-foreground: 210 40% 98%;
  --accent: 216 34% 17%;
  --accent-foreground: 210 40% 98%;
  --destructive: 0 63% 31%;
  --destructive-foreground: 210 40% 98%;
  --ring: 216 34% 17%;
}
"#;

const THEME_STORAGE_KEY: &str = "epsx-theme";

/// Inline pre-paint script — runs before any Dioxus hydration. Reads the
/// persisted theme from `localStorage` (or `prefers-color-scheme`) and
/// applies the `dark` class to `<html>`. This is the FOUC-prevention
/// pattern `apps-old/frontend/components/theme-toggle.tsx` uses.
const THEME_BOOT_SCRIPT: &str = r#"
(function() {
    try {
        var stored = localStorage.getItem('epsx-theme');
        var mode = stored;
        if (!mode) {
            var prefersDark = window.matchMedia && window.matchMedia('(prefers-color-scheme: dark)').matches;
            mode = prefersDark ? 'dark' : 'light';
        }
        var root = document.documentElement;
        if (mode === 'dark') {
            root.classList.add('dark');
        } else {
            root.classList.remove('dark');
        }
        root.setAttribute('data-theme', mode);
    } catch (e) {
        // localStorage may be disabled (e.g. SSR-only render, sandboxed
        // iframe); fall through without modifying the DOM.
    }
})();
"#;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum ThemeMode {
    #[default]
    Light,
    Dark,
}

impl ThemeMode {
    pub fn from_str(s: &str) -> Self {
        match s {
            "dark" => ThemeMode::Dark,
            _ => ThemeMode::Light,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            ThemeMode::Light => "light",
            ThemeMode::Dark => "dark",
        }
    }

    pub fn toggle(&self) -> Self {
        match self {
            ThemeMode::Light => ThemeMode::Dark,
            ThemeMode::Dark => ThemeMode::Light,
        }
    }
}

/// Inject the EPSX design tokens + a pre-paint theme bootstrap script.
/// Drop this near the top of every SSR document so the CSS vars are
/// available and the `dark` class is applied before any visible paint.
#[component]
pub fn ThemeRoot(children: Element) -> Element {
    rsx! {
        style { "{EPSX_CSS_VARS}" }
        // Pre-paint theme bootstrap. `script` with no `type` is treated
        // as JavaScript by the browser. Dioxus SSR will emit the literal
        // `<script>…</script>` so the parser runs it before the first
        // paint of the body content below.
        script { dangerous_inner_html: "{THEME_BOOT_SCRIPT}" }
        {children}
    }
}

/// Reactive theme signal. The default is `Light`; after the first
/// client-side mount the value should be synced from
/// `document.documentElement.getAttribute('data-theme')` via
/// `use_effect` (see `UnifiedThemeToggle`).
pub fn use_theme() -> Signal<ThemeMode> {
    use_signal(ThemeMode::default)
}

// `current_dom_theme` removed — replaced by the inline
// `use_effect` inside `UnifiedThemeToggle`. A standalone
// `fn current_dom_theme()` cannot return a value from an
// `async` block; callers that need the live value should
// use `use_theme` + `use_effect` (see below).

/// Click-target theme toggle. Renders a single sun/moon button. Clicking
/// it flips the `dark` class on `<html>`, updates `data-theme`, and
/// persists the new value to `localStorage` — all driven by
/// `epsx.toggleTheme()` (defined in `epsx_templates::global_js`),
/// which is wired in via a literal `onclick="epsx.toggleTheme()"`
/// HTML attribute. This avoids the Dioxus-SSR-stripped-closure trap
/// (Dioxus is hydration-less; the `onclick: move |_| {}` macro form
/// produces a button with no runtime handler).
///
/// Usage (in the navbar / nav-actions):
/// ```ignore
/// UnifiedThemeToggle {}
/// ```
#[component]
pub fn UnifiedThemeToggle() -> Element {
    let mode = use_theme();

    // Compute the icon + label for the current mode. The `data-theme`
    // attribute is set by the pre-paint `THEME_BOOT_SCRIPT` (see
    // `ThemeRoot`); the icon visibility toggling is also handled by
    // `epsx.toggleTheme()` in `global_js` (it calls `updateThemeIcon`
    // on the same element after flipping the class).
    let (_icon, label) = match *mode.read() {
        ThemeMode::Light => ("sun", "Switch to dark mode"),
        ThemeMode::Dark  => ("moon", "Switch to light mode"),
    };

    // Build the full button as a raw HTML string so we can emit a
    // literal `onclick="epsx.toggleTheme()"` attribute (the Dioxus
    // `onclick:` macro attribute gets stripped at SSR time). Render
    // via `dangerous_inner_html` on a wrapping `<span>`.
    let sun_svg  = epsx_templates::lucide("sun",  "18", "").to_string();
    let moon_svg = epsx_templates::lucide("moon", "18", "").to_string();
    let sun_display  = if *mode.read() == ThemeMode::Light { "none" } else { "" };
    let moon_display = if *mode.read() == ThemeMode::Dark  { "none" } else { "" };
    let safe_label = epsx_templates::html_attr_escape_pub(label);
    let html = format!(
        r#"<button type="button" class="theme-toggle btn btn-icon btn-ghost" aria-label="{label}" title="{label}" onclick="epsx.toggleTheme()"><span class="theme-toggle-icon theme-toggle-sun"  style="display:{sun_disp};width:1.125rem;height:1.125rem;">{sun}</span><span class="theme-toggle-icon theme-toggle-moon" style="display:{moon_disp};width:1.125rem;height:1.125rem;">{moon}</span></button>"#,
        label     = safe_label,
        sun_disp  = sun_display,
        moon_disp = moon_display,
        sun       = sun_svg,
        moon      = moon_svg,
    );

    rsx! {
        span { class: "theme-toggle-wrap inline-flex",
            dangerous_inner_html: "{html}"
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn theme_mode_toggle() {
        assert_eq!(ThemeMode::Light.toggle(), ThemeMode::Dark);
        assert_eq!(ThemeMode::Dark.toggle(), ThemeMode::Light);
    }

    #[test]
    fn theme_mode_parse() {
        assert_eq!(ThemeMode::from_str("dark"), ThemeMode::Dark);
        assert_eq!(ThemeMode::from_str("light"), ThemeMode::Light);
        // Unknown values fall back to Light — the conservative default.
        assert_eq!(ThemeMode::from_str("garbage"), ThemeMode::Light);
        assert_eq!(ThemeMode::from_str(""), ThemeMode::Light);
    }

    #[test]
    fn theme_mode_as_str_round_trip() {
        for m in [ThemeMode::Light, ThemeMode::Dark] {
            assert_eq!(ThemeMode::from_str(m.as_str()), m);
        }
    }

    /// The CSS-vars string must still contain both `:root` and `.dark`
    /// blocks — the SSR theme switcher relies on both. A regression
    /// here breaks dark mode site-wide.
    #[test]
    fn css_vars_contain_light_and_dark_blocks() {
        assert!(EPSX_CSS_VARS.contains(":root"), "must declare :root vars");
        assert!(EPSX_CSS_VARS.contains(".dark"), "must declare .dark vars");
    }

    /// The boot script must reference the same localStorage key the
    /// `UnifiedThemeToggle` writes — otherwise the two sides fall out
    /// of sync and the user sees a flash of the wrong theme on reload.
    #[test]
    fn boot_script_uses_canonical_storage_key() {
        assert!(
            THEME_BOOT_SCRIPT.contains(THEME_STORAGE_KEY),
            "boot script must read epsx-theme from localStorage"
        );
    }

    /// Wave 23 T4 v2 — `epsx_templates::global_js` (in
    /// `shared/rust/templates/src/lib.rs`) must export
    /// `epsx.toggleTheme`, `epsx.setTheme`, `epsx.currentTheme`.
    /// The `UnifiedThemeToggle` component renders the inline
    /// `onclick="epsx.toggleTheme()"` attribute; if any of these
    /// three names disappear from the global namespace the click
    /// handler stops firing under SSR (Dioxus closures are
    /// stripped). This test catches that regression at compile-
    /// test time without needing a live browser.
    #[test]
    fn global_js_exports_theme_namespace() {
        let js = epsx_templates::global_js();
        assert!(js.contains("function toggleTheme()"), "global_js must define function toggleTheme");
        assert!(js.contains("function setTheme(t)"), "global_js must define function setTheme");
        assert!(js.contains("function currentTheme()"), "global_js must define function currentTheme");
        // The return statement must include toggleTheme + setTheme
        // in the public object so callers can access them via
        // `window.epsx.toggleTheme()`.
        assert!(js.contains("toggleTheme,"), "global_js return must include toggleTheme");
        assert!(js.contains("setTheme,"), "global_js return must include setTheme");
    }

    /// Wave 23 T4 v2 — the `setTheme` function in `global_js` must
    /// write to the same localStorage key the boot script reads.
    /// A mismatch causes the two halves of the theme system to
    /// silently desync (the user sees a flash of the wrong theme
    /// on reload). The previous T4's `setTheme` wrote
    /// `epsx_theme` (underscore) while the boot script read
    /// `epsx-theme` (hyphen) — a 1-char typo that broke the
    /// whole feature. This test pins the two halves together.
    #[test]
    fn set_theme_writes_to_canonical_storage_key() {
        let js = epsx_templates::global_js();
        assert!(
            js.contains("localStorage.setItem('epsx-theme'"),
            "setTheme() must write 'epsx-theme' (canonical key, matches the boot script)"
        );
    }
}
