use dioxus::prelude::*;

/// Theme + design tokens — exposes the same CSS variables the Next.js app uses.
///
/// Wave 23 T4: rewrote this module to actually work end-to-end. The original
/// shipped the `.dark` CSS-variable block but nothing in the crate ever added
/// the `dark` class to `<html>`, so dark mode was dead code. This file now:
///
/// 1. Ships the CSS variables (light + dark) — unchanged.
/// 2. `ThemeRoot` injects a one-shot inline script that reads the persisted
///    theme from `localStorage` (or `prefers-color-scheme`) and applies the
///    `dark` class to `<html>` before paint, eliminating the FOUC the prod
///    `theme-toggle.tsx` warns about.
/// 3. `UnifiedThemeToggle` is the click target the navbar (and the auth
///    pages) render. Clicking it flips the `dark` class on `<html>` and
///    persists the new value to `localStorage`.
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
/// persists the new value to `localStorage`. The internal `Signal` is
/// kept in sync with the DOM via a `use_effect` block.
///
/// Usage (in the navbar / nav-actions):
/// ```ignore
/// UnifiedThemeToggle {}
/// ```
#[component]
pub fn UnifiedThemeToggle() -> Element {
    let mut mode = use_theme();

    // On mount, sync the signal with whatever the pre-paint script
    // applied. This is browser-only; SSR keeps the default (Light).
    use_effect(move || {
        spawn(async move {
            let script =
                r#"document.documentElement.getAttribute('data-theme') || 'light'"#;
            if let Ok(value) = document::eval(script).join::<String>().await {
                mode.set(ThemeMode::from_str(value.trim()));
            }
        });
    });

    let onclick = move |_: MouseEvent| {
        let next = mode.read().toggle();
        // Apply to DOM + persist. Wrap in `spawn` so the await is
        // properly driven by the Dioxus runtime.
        spawn(async move {
            let script = format!(
                r#"
                (function() {{
                    try {{
                        var root = document.documentElement;
                        if ('{next}' === 'dark') {{
                            root.classList.add('dark');
                        }} else {{
                            root.classList.remove('dark');
                        }}
                        root.setAttribute('data-theme', '{next}');
                        localStorage.setItem('epsx-theme', '{next}');
                    }} catch (e) {{}}
                }})();
                "#,
                next = next.as_str()
            );
            let _ = document::eval(script.as_str()).await;
        });
        mode.set(next);
    };

    let (icon, label) = match *mode.read() {
        ThemeMode::Light => ("sun", "Switch to dark mode"),
        ThemeMode::Dark => ("moon", "Switch to light mode"),
    };

    rsx! {
        button {
            class: "theme-toggle btn btn-icon btn-ghost",
            r#type: "button",
            "aria-label": "{label}",
            title: "{label}",
            onclick: onclick,
            span { class: "theme-toggle-icon", dangerous_inner_html: "{epsx_templates::lucide(icon, \"18\", \"\")}" }
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
}
