use dioxus::prelude::*;

/// Theme + design tokens — exposes the same CSS variables the Next.js app uses.
///
/// Browser persistence and toggling are implemented by the generated
/// `epsx-browser-runtime` WASM module. SSR emits only CSS and typed data
/// attributes; there is no executable inline script.
pub const EPSX_CSS_VARS: &str = r#"
:root {
  /* These tokens are consumed directly by the frozen Tailwind v4 bundle
     and by the shared template CSS. Keep them as complete CSS colors rather
     than bare HSL channels: `background: var(--primary)` must stay valid. */
  --background: #ffffff;
  --foreground: #0f172a;
  --muted: #f1f5f9;
  --muted-foreground: #475569;
  --card: #ffffff;
  --card-foreground: #0f172a;
  --border: #e2e8f0;
  --input: #cbd5e1;
  --primary: #2563eb;
  --primary-foreground: #ffffff;
  --secondary: #f1f5f9;
  --secondary-foreground: #0f172a;
  --accent: #f1f5f9;
  --accent-foreground: #0f172a;
  --destructive: #dc2626;
  --destructive-foreground: #ffffff;
  --ring: #2563eb;
  --radius: 0.5rem;
  --epsx-blue-start: #488BFA;
  --epsx-blue-end: #A43FF3;
  --epsx-cyan: #3FC9F3;
  --epsx-purple: #A43FF3;
}
.dark {
  --background: #020617;
  --foreground: #e2e8f0;
  --muted: #0f172a;
  --muted-foreground: #94a3b8;
  --card: #020617;
  --card-foreground: #e2e8f0;
  --border: #1e293b;
  --input: #1e293b;
  --primary: #2563eb;
  --primary-foreground: #ffffff;
  --secondary: #0f172a;
  --secondary-foreground: #f8fafc;
  --accent: #1e293b;
  --accent-foreground: #f8fafc;
  --destructive: #b91c1c;
  --destructive-foreground: #ffffff;
  --ring: #60a5fa;
}
"#;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum ThemeMode {
    #[default]
    Dark,
    Light,
}

impl ThemeMode {
    pub fn from_name(s: &str) -> Self {
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

/// Inject the EPSX design tokens. The shared page shell already starts with
/// the deterministic dark class; Rust/WASM reconciles persisted preference.
#[component]
pub fn ThemeRoot(children: Element) -> Element {
    rsx! {
        style { "{EPSX_CSS_VARS}" }
        {children}
    }
}

/// SSR-local theme signal. The default is `Dark`. Hydration-less documents
/// do not synchronize it after render; client-side theme state is owned by
/// the marked controls and `epsx_templates::global_js`.
pub fn use_theme() -> Signal<ThemeMode> {
    use_signal(ThemeMode::default)
}

/// Click-target theme toggle. Renders a single sun/moon button. Clicking
/// it flips the `dark` class on `<html>`, updates `data-theme`, and
/// persists the new value through the Rust/WASM event delegate.
///
/// Usage (in the navbar / nav-actions):
/// ```ignore
/// UnifiedThemeToggle {}
/// ```
#[component]
pub fn UnifiedThemeToggle() -> Element {
    // This only selects a deterministic SSR icon fallback. The browser-side
    // controller reconciles it from the pre-paint DOM theme before use.
    let mode = use_theme();

    // The server cannot read the browser's persisted theme, so keep the
    // fallback name neutral. `global_js` reconciles the icon and exact
    // "Switch to … theme" action name from the live DOM theme on
    // DOMContentLoaded and after every toggle.
    let label = "Toggle theme";

    // Raw HTML is used only to preserve the existing SVG markup. Behavior is
    // declared with a non-executable action token.
    let sun_svg = epsx_templates::lucide("sun", "18", "").to_string();
    let moon_svg = epsx_templates::lucide("moon", "18", "").to_string();
    let sun_display = if *mode.read() == ThemeMode::Light {
        "none"
    } else {
        ""
    };
    let moon_display = if *mode.read() == ThemeMode::Dark {
        "none"
    } else {
        ""
    };
    let safe_label = epsx_templates::html_attr_escape_pub(label);
    let html = format!(
        r#"<button type="button" class="theme-toggle btn btn-icon btn-ghost" data-epsx-theme-toggle data-epsx-action="theme-toggle" aria-label="{label}" title="{label}"><span class="theme-toggle-icon theme-toggle-sun" data-epsx-theme-icon="sun" style="display:{sun_disp};width:1.125rem;height:1.125rem;">{sun}</span><span class="theme-toggle-icon theme-toggle-moon" data-epsx-theme-icon="moon" style="display:{moon_disp};width:1.125rem;height:1.125rem;">{moon}</span></button>"#,
        label = safe_label,
        sun_disp = sun_display,
        moon_disp = moon_display,
        sun = sun_svg,
        moon = moon_svg,
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
    fn unified_toggle_emits_the_shared_runtime_contract() {
        let html = dioxus_ssr::render_element(rsx! { UnifiedThemeToggle {} });
        assert_eq!(html.matches("data-epsx-theme-toggle").count(), 1);
        assert_eq!(html.matches(r#"data-epsx-theme-icon="sun""#).count(), 1);
        assert_eq!(html.matches(r#"data-epsx-theme-icon="moon""#).count(), 1);
        assert!(html.contains(r#"aria-label="Toggle theme""#));
        assert!(html.contains(r#"title="Toggle theme""#));
        assert!(html.contains(r#"data-epsx-action="theme-toggle""#));
        assert!(!html.contains("onclick="));
    }

    #[test]
    fn theme_mode_parse() {
        assert_eq!(ThemeMode::from_name("dark"), ThemeMode::Dark);
        assert_eq!(ThemeMode::from_name("light"), ThemeMode::Light);
        // Unknown values fall back to Light — the conservative default.
        assert_eq!(ThemeMode::from_name("garbage"), ThemeMode::Light);
        assert_eq!(ThemeMode::from_name(""), ThemeMode::Light);
    }

    #[test]
    fn theme_mode_as_str_round_trip() {
        for m in [ThemeMode::Light, ThemeMode::Dark] {
            assert_eq!(ThemeMode::from_name(m.as_str()), m);
        }
    }

    /// The CSS-vars string must still contain both `:root` and `.dark`
    /// blocks — the SSR theme switcher relies on both. A regression
    /// here breaks dark mode site-wide.
    #[test]
    fn css_vars_contain_light_and_dark_blocks() {
        assert!(EPSX_CSS_VARS.contains(":root"), "must declare :root vars");
        assert!(EPSX_CSS_VARS.contains(".dark"), "must declare .dark vars");
        assert!(EPSX_CSS_VARS.contains("--primary: #2563eb"));
        assert!(
            !EPSX_CSS_VARS.contains("--primary: 222.2"),
            "shared tokens must be complete CSS colors, not HSL channels"
        );
    }

    /// The shared shell must reference only the generated Rust/WASM module.
    #[test]
    fn shell_loads_generated_wasm_theme_runtime() {
        let tag = epsx_templates::global_js();
        assert!(tag.contains("/runtime/epsx_browser_runtime_bootstrap.js"));
        assert!(tag.contains("data-epsx-generated-runtime=\"wasm-bindgen\""));
        assert!(!tag.contains("localStorage"));
        assert!(!tag.contains("onclick="));
    }
}
