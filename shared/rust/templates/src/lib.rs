//! EPSX design system — shared HTML template helpers.
//!
//! Every BFF (frontend, admin, pay, preview) calls `design_system_head()` to
//! emit the same `<head>` block (source-compatible font stack, Tailwind CSS, CSS
//! variables, glassmorphism utilities, animations, dark/light mode FOUC
//! prevention).
//!
//! All visual changes across the platform should go through this module so we
//! can match the original Next.js design without duplicating CSS strings.

pub mod components;

fn escape_html_attribute(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

fn escape_html_text(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

/// Returns the full `<head>` block matching the original Next.js design.
///
/// Includes:
/// - the source app's effective Tailwind sans stack via `--font-sans`
/// - Tailwind v2.2.19 CDN (we keep the older CDN for stability with our
///   utility classes; the design intent is identical to v4)
/// - Complete CSS variable system for light + dark mode
/// - Glassmorphism, gradient text, gradient orbs, shadows, hover effects
/// - FOUC prevention script that applies the saved theme before first paint
/// - Toast / modal / dropdown / tab / chat-widget global controllers
pub const DESIGN_SYSTEM_CSS: &str = include_str!("design_system.css");

pub fn design_system_head(title: &str, description: &str) -> String {
    design_system_head_with_keywords(title, description, None)
}

/// Variant of [`design_system_head`] that preserves route-owned search
/// keywords when the canonical source defines them.
pub fn design_system_head_with_keywords(
    title: &str,
    description: &str,
    keywords: Option<&str>,
) -> String {
    let title = escape_html_text(title);
    let description = escape_html_attribute(description);
    let keywords_meta = keywords
        .map(escape_html_attribute)
        // Keep the separator as a real newline. A raw `\\n` sequence here
        // becomes visible text at the top of pages that include keywords.
        .map(|value| format!("<meta name=\"keywords\" content=\"{value}\" />\n"))
        .unwrap_or_default();
    format!(
        r##"<meta charset="utf-8" />
<meta name="viewport" content="width=device-width, initial-scale=1, maximum-scale=5, user-scalable=yes, viewport-fit=cover" />
<meta name="description" content="{description}" />
{keywords_meta}<meta name="theme-color" content="#ffffff" media="(prefers-color-scheme: light)" />
<meta name="theme-color" content="#000000" media="(prefers-color-scheme: dark)" />
<link rel="icon" href="/public/logos/epsx-icon.svg" type="image/svg+xml" />
<title>{title}</title>
<!-- Wave 28 T1: Tailwind v4 PostCSS pipeline — local CSS only.
     The CDN at jsdelivr is gone; Tailwind v4 utilities are now served from
     /public/dist/tailwind.css, compiled by `apps/frontend/build.rs` /
     `apps/admin/build.rs` from `apps/<app>/src/styles/index.css` via
     `@tailwindcss/postcss 4.1.18`. The /public prefix matches the BFF's
     `nest_service("/public", ServeDir::new("public"))` mount in
     `apps/frontend/src/main.rs` (and the equivalent for `apps/admin`).
     The CDN swap from Wave 25 was kept (Tailwind v2.2.19) until Wave 28
     confirmed the structural color drift from the v2 CDN — see the Wave
     28 honest verdict + T1 deliverable.
     Chrome aggressively caches /public/dist/tailwind.css without a
     Cache-Control header (ServeDir defaults to heuristic caching). Adding
     `?v=` busts the disk cache after each Tailwind rebuild; the HTML
     itself is `private, no-store` so the new link is always fetched. -->
<link rel="stylesheet" href="/public/dist/tailwind.css?v=3" />
<style>{DESIGN_SYSTEM_CSS}</style>"##,
        keywords_meta = keywords_meta,
    )
}

/// Returns the external wasm-bindgen module generated from the Rust browser runtime.
/// The referenced files live under `target/` and are never committed.
/// Bump the revision when the bootstrap/runtime contract changes so an already-open
/// browser cannot keep executing an older ES module graph after a local rebuild.
pub fn global_js() -> &'static str {
    r#"<script type="module" src="/runtime/epsx_browser_runtime_bootstrap.js?rev=4" data-epsx-generated-runtime="wasm-bindgen"></script>"#
}

/// Returns a theme toggle button handled by the Rust/WASM event delegate.
pub fn theme_toggle_button() -> &'static str {
    r##"<button id="epsx-theme-toggle" type="button" class="nav-link" data-epsx-theme-toggle data-epsx-action="theme-toggle" aria-label="Toggle theme" style="width:2.25rem;height:2.25rem;padding:0;justify-content:center;">
  <svg xmlns="http://www.w3.org/2000/svg" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="lucide lucide-sun" data-epsx-theme-icon="sun" aria-hidden="true"><circle cx="12" cy="12" r="4"/><path d="M12 2v2"/><path d="M12 20v2"/><path d="m4.93 4.93 1.41 1.41"/><path d="m17.66 17.66 1.41 1.41"/><path d="M2 12h2"/><path d="M20 12h2"/><path d="m6.34 17.66-1.41 1.41"/><path d="m19.07 4.93-1.41 1.41"/></svg>
  <svg xmlns="http://www.w3.org/2000/svg" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="lucide lucide-moon" data-epsx-theme-icon="moon" aria-hidden="true"><path d="M12 3a6 6 0 0 0 9 9 9 9 0 1 1-9-9Z"/></svg>
</button>"##
}

// Browser behavior is delegated to the generated Rust/WASM runtime via typed
// data attributes. Values remain ordinary HTML attributes, so the SSR output
// is useful when the runtime is unavailable and contains no executable code.

/// Returns a complete `<button>…</button>` HTML string that
/// copies `text` to the clipboard when clicked. The `label`
/// parameter is the resting label; generic copy buttons retain
/// the `epsx.copyText` inline feedback that restores after 2 s.
///
/// Usage from a Dioxus component:
/// ```ignore
/// rsx! {
///     span { class: "inline-block",
///         dangerous_inner_html: "{epsx_templates::copy_button_html(&text, \"Copy\")}" }
/// }
/// ```
pub fn copy_button_html(text: &str, label: &str) -> String {
    format!(
        r#"<button type="button" class="btn btn-sm btn-outline copy-btn" data-copy="{safe_text}" data-epsx-action="copy" aria-label="Copy to clipboard"><span>{label}</span></button>"#,
        safe_text = html_attr_escape(text),
        label = html_text_escape(label),
    )
}

/// Returns a complete `<button>…</button>` HTML string for the
/// contact page's "Copy email" button. Visually matches the
/// `contact-copy-btn` class so existing CSS still applies. The
/// caller renders the associated `contact-copy-email-status`
/// polite status region next to this stable-label button.
pub fn email_copy_button_html(email: &str) -> String {
    format!(
        r#"<button id="contact-copy-email-button" type="button" class="btn btn-ghost contact-copy-btn" data-copy="{safe_email}" data-copy-status-target="contact-copy-email-status" data-epsx-action="copy" aria-label="Copy email address" aria-describedby="contact-copy-email-status"><svg xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" data-lucide="copy"><rect width="14" height="14" x="8" y="8" rx="2" ry="2"></rect><path d="M4 16c-1.1 0-2-.9-2-2V4c0-1.1.9-2 2-2h10c1.1 0 2 .9 2 2"></path></svg><span>Copy</span></button>"#,
        safe_email = html_attr_escape(email),
    )
}

/// Returns a complete `<button>…</button>` HTML string for a
/// share button. Uses the Web Share API on mobile; on desktop
/// falls back to copying the URL to the clipboard.
pub fn share_button_html(text: &str, title: &str, label: &str) -> String {
    format!(
        r#"<button type="button" class="share-btn" data-share-text="{safe_text}" data-share-title="{safe_title}" data-epsx-action="share" aria-label="Share"><span>{label}</span></button>"#,
        safe_text = html_attr_escape(text),
        safe_title = html_attr_escape(title),
        label = html_text_escape(label),
    )
}

/// Returns a native submit button for the server-owned news search form.
pub fn news_search_submit_button_html(_form_id: &str, label: &str) -> String {
    format!(
        r#"<button type="submit" class="btn btn-outline">{label}</button>"#,
        label = html_text_escape(label),
    )
}

/// Returns a complete `<select data-epsx-navigate="1" …>…</select>`
/// HTML string. The `global_js` `bindNavigateSelects()` listener
/// picks it up on DOMContentLoaded and wires a `change` handler
/// that navigates to `<base_href>?<qp>=<value>`. Used by the
/// pagination `LimitSelector` and the payment page's Token picker.
pub fn navigate_select_html(
    base_href: &str,
    query_param: &str,
    current: &str,
    options: &[(String, String)],
) -> String {
    let mut opts = String::new();
    for (val, lbl) in options {
        let sel = if val == current { " selected" } else { "" };
        opts.push_str(&format!(
            r#"<option value="{val}"{sel}>{lbl}</option>"#,
            val = html_attr_escape(val),
            sel = sel,
            lbl = html_text_escape(lbl),
        ));
    }
    format!(
        r#"<select class="input input-sm" data-epsx-navigate="1" data-base-href="{base}" data-qp="{qp}">{opts}</select>"#,
        base = html_attr_escape(base_href),
        qp = html_attr_escape(query_param),
        opts = opts,
    )
}

/// Escape a string for safe inclusion in a double-quoted HTML
/// attribute value. The escape table covers `&`, `<`, `>`, `"`,
/// and `'`. Used by the builder fns above to neutralise the
/// `data-*` attribute values that mirror the user-supplied text.
fn html_attr_escape(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&#39;"),
            c => out.push(c),
        }
    }
    out
}

/// Public re-export of `html_attr_escape` for Dioxus components
/// that need to build raw HTML strings via `dangerous_inner_html`.
/// Prefer using the higher-level `copy_button_html` /
/// `share_button_html` / `email_copy_button_html` builders for
/// common cases; this is for bespoke markup.
pub fn html_attr_escape_pub(s: &str) -> String {
    html_attr_escape(s)
}

/// Public re-export of `html_text_escape` for the same reason.
pub fn html_text_escape_pub(s: &str) -> String {
    html_text_escape(s)
}

/// Escape a string for safe inclusion as HTML text content.
fn html_text_escape(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            c => out.push(c),
        }
    }
    out
}

/// Returns the standard EPSX logo (gradient text "EPSX").
/// Returns the EPSX hexagon-with-chart icon (matches epsx.io's `/logos/epsx-icon.svg`).
///
/// Wave 44 t2: use the production asset (`/logos/epsx-icon.svg`) directly.
/// Keeping this as a CSS background (rather than duplicating inline SVG
/// gradients in both desktop and mobile headers) avoids document-global
/// gradient ID collisions and makes the icon render consistently at every
/// breakpoint. A background also keeps the shell's text/attribute escaping
/// contract intact for hostile wallet values.
pub fn epsx_icon_svg() -> &'static str {
    r#"<span class="epsx-icon" role="img" aria-label="EPSX" style="background:url('/public/logos/epsx-icon.svg') center/contain no-repeat;"></span>"#
}

/// Lucide icon path data — `name` is the kebab-case lucide name (e.g. `chart-column`).
/// Returns the inner `<path>` content. Caller wraps in a `<svg>` with class.
/// We embed the 50+ icons we use; for anything else, return empty.
pub fn lucide_icon(name: &str) -> &'static str {
    match name {
        "chart-column" => {
            r#"<path d="M3 3v16a2 2 0 0 0 2 2h16"/><path d="M18 17V9"/><path d="M13 17V5"/><path d="M8 17v-3"/>"#
        }
        // Wave 28 T2 — register prod's exact icon shape for the
        // portfolio upsell banner (the 3-bar chart with no axis
        // labels). Path data from lucide.dev/chart-no-axes-column.
        "chart-no-axes-column" => {
            r#"<path d="M5 21V3"/><path d="M19 21V3"/><path d="M15 21V9"/><path d="M11 21V13"/><path d="M7 21V17"/>"#
        }
        "code" => r#"<path d="m16 18 6-6-6-6"/><path d="m8 6-6 6 6 6"/>"#,
        "building" => {
            r#"<path d="M6 22V4a2 2 0 0 1 2-2h8a2 2 0 0 1 2 2v18"/><path d="M6 12H4a2 2 0 0 0-2 2v6a2 2 0 0 0 2 2h2"/><path d="M18 9h2a2 2 0 0 1 2 2v9a2 2 0 0 1-2 2h-2"/><path d="M10 6h4"/><path d="M10 10h4"/><path d="M10 14h4"/><path d="M10 18h4"/>"#
        }
        "chevron-down" => r#"<path d="m6 9 6 6 6-6"/>"#,
        "chevron-right" => r#"<path d="m9 18 6-6-6-6"/>"#,
        "trending-up" => r#"<path d="M22 7 13.5 15.5 8.5 10.5 2 17"/><path d="M16 7h6v6"/>"#,
        "chart-line" | "line-chart" => {
            r#"<path d="M3 3v16a2 2 0 0 0 2 2h16"/><path d="m19 9-5 5-4-4-3 3"/>"#
        }
        "zap" => {
            r#"<path d="M4 14a1 1 0 0 1-.78-1.63l9.9-10.2a.5.5 0 0 1 .86.46l-1.92 6.02A1 1 0 0 0 13 10h7a1 1 0 0 1 .78 1.63l-9.9 10.2a.5.5 0 0 1-.86-.46l1.92-6.02A1 1 0 0 0 11 14z"/>"#
        }
        "users" => {
            r#"<path d="M16 21v-2a4 4 0 0 0-4-4H6a4 4 0 0 0-4 4v2"/><circle cx="9" cy="7" r="4"/><path d="M22 21v-2a4 4 0 0 0-3-3.87"/><path d="M16 3.13a4 4 0 0 1 0 7.75"/>"#
        }
        "calendar" => {
            r#"<path d="M8 2v4"/><path d="M16 2v4"/><rect width="18" height="18" x="3" y="4" rx="2"/><path d="M3 10h18"/>"#
        }
        "newspaper" => {
            r#"<path d="M4 22h16a2 2 0 0 0 2-2V4a2 2 0 0 0-2-2H8a2 2 0 0 0-2 2v16a2 2 0 0 1-2 2Zm0 0a2 2 0 0 1-2-2v-9c0-1.1.9-2 2-2h2"/><path d="M18 14h-8"/><path d="M15 18h-5"/><path d="M10 6h8v4h-8V6Z"/>"#
        }
        "pin" => {
            r#"<path d="M12 17v5"/><path d="M9 10.76a2 2 0 0 1-1.11 1.79l-1.78.9A2 2 0 0 0 5 15.24V16a1 1 0 0 0 1 1h12a1 1 0 0 0 1-1v-.76a2 2 0 0 0-1.11-1.79l-1.78-.9A2 2 0 0 1 15 10.76V7a1 1 0 0 1 1-1 2 2 0 0 0 0-4H8a2 2 0 0 0 0 4 1 1 0 0 1 1 1z"/>"#
        }
        "arrow-right" => r#"<path d="M5 12h14"/><path d="m12 5 7 7-7 7"/>"#,
        "arrow-up-down" => {
            r#"<path d="m21 16-4 4-4-4"/><path d="M17 20V4"/><path d="m3 8 4-4 4 4"/><path d="M7 4v16"/>"#
        }
        "info" => r#"<circle cx="12" cy="12" r="10"/><path d="M12 16v-4"/><path d="M12 8h.01"/>"#,
        "mail" => {
            r#"<rect width="20" height="16" x="2" y="4" rx="2"/><path d="m22 7-8.97 5.7a1.94 1.94 0 0 1-2.06 0L2 7"/>"#
        }
        "help-circle" => {
            r#"<circle cx="12" cy="12" r="10"/><path d="M9.09 9a3 3 0 0 1 5.83 1c0 2-3 3-3 3"/><path d="M12 17h.01"/>"#
        }
        "circle-help" => {
            r#"<circle cx="12" cy="12" r="10"/><path d="M9.09 9a3 3 0 0 1 5.83 1c0 2-3 3-3 3"/><path d="M12 17h.01"/>"#
        }
        "menu" => {
            r#"<line x1="4" x2="20" y1="12" y2="12"/><line x1="4" x2="20" y1="6" y2="6"/><line x1="4" x2="20" y1="18" y2="18"/>"#
        }
        "x" => r#"<path d="M18 6 6 18"/><path d="m6 6 12 12"/>"#,
        "sun" => {
            r#"<circle cx="12" cy="12" r="4"/><path d="M12 2v2"/><path d="M12 20v2"/><path d="m4.93 4.93 1.41 1.41"/><path d="m17.66 17.66 1.41 1.41"/><path d="M2 12h2"/><path d="M20 12h2"/><path d="m6.34 17.66-1.41 1.41"/><path d="m19.07 4.93-1.41 1.41"/>"#
        }
        "moon" => r#"<path d="M12 3a6 6 0 0 0 9 9 9 9 0 1 1-9-9Z"/>"#,
        "wallet" => {
            r#"<path d="M19 7V4a1 1 0 0 0-1-1H5a2 2 0 0 0 0 4h15a1 1 0 0 1 1 1v4h-3a2 2 0 0 0 0 4h3a1 1 0 0 0 1-1v-2a1 1 0 0 0-1-1"/><path d="M3 5v14a2 2 0 0 0 2 2h15a1 1 0 0 0 1-1v-4"/>"#
        }
        "log-out" => {
            r#"<path d="M9 21H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h4"/><polyline points="16 17 21 12 16 7"/><line x1="21" x2="9" y1="12" y2="12"/>"#
        }
        "user" => {
            r#"<path d="M19 21v-2a4 4 0 0 0-4-4H9a4 4 0 0 0-4 4v2"/><circle cx="12" cy="7" r="4"/>"#
        }
        "settings" => {
            r#"<path d="M12.22 2h-.44a2 2 0 0 0-2 2v.18a2 2 0 0 1-1 1.73l-.43.25a2 2 0 0 1-2 0l-.15-.08a2 2 0 0 0-2.73.73l-.22.38a2 2 0 0 0 .73 2.73l.15.1a2 2 0 0 1 1 1.72v.51a2 2 0 0 1-1 1.74l-.15.09a2 2 0 0 0-.73 2.73l.22.38a2 2 0 0 0 2.73.73l.15-.08a2 2 0 0 1 2 0l.43.25a2 2 0 0 1 1 1.73V20a2 2 0 0 0 2 2h.44a2 2 0 0 0 2-2v-.18a2 2 0 0 1 1-1.73l.43-.25a2 2 0 0 1 2 0l.15.08a2 2 0 0 0 2.73-.73l.22-.39a2 2 0 0 0-.73-2.73l-.15-.08a2 2 0 0 1-1-1.74v-.5a2 2 0 0 1 1-1.74l.15-.09a2 2 0 0 0 .73-2.73l-.22-.38a2 2 0 0 0-2.73-.73l-.15.08a2 2 0 0 1-2 0l-.43-.25a2 2 0 0 1-1-1.73V4a2 2 0 0 0-2-2z"/><circle cx="12" cy="12" r="3"/>"#
        }
        "check" => r#"<path d="M20 6 9 17l-5-5"/>"#,
        // Wave-49 TODO cleanup (alert.rs): shadcn's <Alert> uses
        // 'check-circle' for the Success variant. Register the
        // shape so the Alert component can render the exact lucide
        // name instead of the 'check' substitute.
        "check-circle" => {
            r#"<path d="M22 11.08V12a10 10 0 1 1-5.93-9.14"/><polyline points="22 4 12 14.01 9 11.01"/>"#
        }
        // Wave-49 TODO cleanup (alert.rs): shadcn's <Alert> uses
        // 'alert-triangle' for the Warning variant.
        "alert-triangle" => {
            r#"<path d="m21.73 18-8-14a2 2 0 0 0-3.48 0l-8 14A2 2 0 0 0 4 21h16a2 2 0 0 0 1.73-3Z"/><line x1="12" x2="12" y1="9" y2="13"/><line x1="12" x2="12.01" y1="17" y2="17"/>"#
        }
        "plus" => r#"<path d="M5 12h14"/><path d="M12 5v14"/>"#,
        "search" => r#"<circle cx="11" cy="11" r="8"/><path d="m21 21-4.3-4.3"/>"#,
        "heart" => {
            r#"<path d="M20.84 4.61a5.5 5.5 0 0 0-7.78 0L12 5.67l-1.06-1.06a5.5 5.5 0 0 0-7.78 7.78l1.06 1.06L12 21.23l7.78-7.78 1.06-1.06a5.5 5.5 0 0 0 0-7.78z"/>"#
        }
        "share" => {
            r#"<path d="M4 12v8a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2v-8"/><polyline points="16 6 12 2 8 6"/><line x1="12" x2="12" y1="2" y2="15"/>"#
        }
        "bell" => {
            r#"<path d="M6 8a6 6 0 0 1 12 0c0 7 3 9 3 9H3s3-2 3-9"/><path d="M10.3 21a1.94 1.94 0 0 0 3.4 0"/>"#
        }
        "book" => {
            r#"<path d="M4 19.5A2.5 2.5 0 0 1 6.5 17H20"/><path d="M6.5 2H20v20H6.5A2.5 2.5 0 0 1 4 19.5v-15A2.5 2.5 0 0 1 6.5 2z"/>"#
        }
        "key" => {
            r#"<circle cx="7.5" cy="15.5" r="5.5"/><path d="m21 2-9.6 9.6"/><path d="m15.5 7.5 3 3L22 7l-3-3"/>"#
        }
        "layout-dashboard" => {
            r#"<rect width="7" height="9" x="3" y="3" rx="1"/><rect width="7" height="5" x="14" y="3" rx="1"/><rect width="7" height="9" x="14" y="12" rx="1"/><rect width="7" height="5" x="3" y="16" rx="1"/>"#
        }
        "message-circle" => r#"<path d="M7.9 20A9 9 0 1 0 4 16.1L2 22Z"/>"#,
        "file-text" => {
            r#"<path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"/><polyline points="14 2 14 8 20 8"/><line x1="16" x2="8" y1="13" y2="13"/><line x1="16" x2="8" y1="17" y2="17"/><line x1="10" x2="8" y1="9" y2="9"/>"#
        }
        "file" => {
            r#"<path d="M14.5 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V7.5L14.5 2z"/><polyline points="14 2 14 8 20 8"/>"#
        }
        "folder-open" => {
            r#"<path d="m6 14 1.5-2.9A2 2 0 0 1 9.24 10H20a2 2 0 0 1 1.94 2.5l-1.54 6a2 2 0 0 1-1.95 1.5H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h3.9a2 2 0 0 1 1.69.9l.81 1.2a2 2 0 0 0 1.67.9H18a2 2 0 0 1 2 2v2"/>"#
        }
        "list" => {
            r#"<path d="M3 6h.01"/><path d="M3 12h.01"/><path d="M3 18h.01"/><path d="M8 6h13"/><path d="M8 12h13"/><path d="M8 18h13"/>"#
        }
        "upload" => {
            r#"<path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"/><polyline points="17 8 12 3 7 8"/><line x1="12" x2="12" y1="3" y2="15"/>"#
        }
        "save" => {
            r#"<path d="M15.2 3a2 2 0 0 1 1.4.6l3.8 3.8a2 2 0 0 1 .6 1.4V19a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2z"/><path d="M17 21v-8H7v8"/><path d="M7 3v5h8"/>"#
        }
        "history" => {
            r#"<path d="M3 12a9 9 0 1 0 9-9 9.75 9.75 0 0 0-6.74 2.74L3 8"/><path d="M3 3v5h5"/><path d="M12 7v5l4 2"/>"#
        }
        "credit-card" => {
            r#"<rect width="20" height="14" x="2" y="5" rx="2"/><line x1="2" x2="22" y1="10" y2="10"/>"#
        }
        "link" => {
            r#"<path d="M10 13a5 5 0 0 0 7.54.54l3-3a5 5 0 0 0-7.07-7.07l-1.72 1.71"/><path d="M14 11a5 5 0 0 0-7.54-.54l-3 3a5 5 0 0 0 7.07 7.07l1.71-1.71"/>"#
        }
        "external-link" => {
            r#"<path d="M15 3h6v6"/><path d="M10 14 21 3"/><path d="M18 13v6a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V8a2 2 0 0 1 2-2h6"/>"#
        }
        "briefcase" => {
            r#"<path d="M16 20V4a2 2 0 0 0-2-2h-4a2 2 0 0 0-2 2v16"/><rect width="20" height="14" x="2" y="6" rx="2"/>"#
        }
        // wave2-chrome-track-a: added icons required by admin sidebar/header parity.
        // All paths mirror the official lucide.dev SVG body.
        "home" => {
            r#"<path d="m3 9 9-7 9 7v11a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2z"/><polyline points="9 22 9 12 15 12 15 22"/>"#
        }
        "lock" => {
            r#"<rect width="18" height="11" x="3" y="11" rx="2" ry="2"/><path d="M7 11V7a5 5 0 0 1 10 0v4"/>"#
        }
        "shield" => r#"<path d="M12 22s8-4 8-10V5l-8-3-8 3v7c0 6 8 10 8 10z"/>"#,
        "globe" => {
            r#"<circle cx="12" cy="12" r="10"/><line x1="2" x2="22" y1="12" y2="12"/><path d="M12 2a15.3 15.3 0 0 1 4 10 15.3 15.3 0 0 1-4 10 15.3 15.3 0 0 1-4-10 15.3 15.3 0 0 1 4-10z"/>"#
        }
        "palette" => {
            r#"<circle cx="13.5" cy="6.5" r=".5"/><circle cx="17.5" cy="10.5" r=".5"/><circle cx="8.5" cy="7.5" r=".5"/><circle cx="6.5" cy="12.5" r=".5"/><path d="M12 2C6.5 2 2 6.5 2 12s4.5 10 10 10c.926 0 1.648-.746 1.648-1.688 0-.437-.18-.835-.437-1.125-.29-.289-.438-.652-.438-1.125a1.64 1.64 0 0 1 1.668-1.668h1.996c3.051 0 5.555-2.503 5.555-5.554C21.965 6.012 17.461 2 12 2z"/>"#
        }
        "send" => {
            r#"<line x1="22" x2="11" y1="2" y2="13"/><polygon points="22 2 15 22 11 13 2 9 22 2"/>"#
        }
        "coins" => {
            r#"<circle cx="8" cy="8" r="6"/><path d="M18.09 10.37A6 6 0 1 1 10.34 18"/><path d="M7 6h1v4"/><path d="m16.71 13.88.7.71-2.82 2.82"/>"#
        }
        "link-2" => {
            r#"<path d="M9 17H7A5 5 0 0 1 7 7h2"/><path d="M15 7h2a5 5 0 1 1 0 10h-2"/><line x1="8" x2="16" y1="12" y2="12"/>"#
        }
        "image" => {
            r#"<rect width="18" height="18" x="3" y="3" rx="2" ry="2"/><circle cx="9" cy="9" r="2"/><path d="m21 15-3.086-3.086a2 2 0 0 0-2.828 0L6 21"/>"#
        }
        "bar-chart-3" => {
            r#"<path d="M3 3v18h18"/><path d="M18 17V9"/><path d="M13 17V5"/><path d="M8 17v-3"/>"#
        }
        "bar-chart" | "bar-chart-2" | "chart-bar" => {
            r#"<path d="M3 3v18h18"/><path d="M18 17V9"/><path d="M13 17V5"/><path d="M8 17v-3"/>"#
        }
        "bug" => {
            r#"<path d="M12 20v-9"/><path d="M14 7a4 4 0 0 1 4 4v3a6 6 0 0 1-12 0v-3a4 4 0 0 1 4-4z"/><path d="M14.12 3.88 16 2"/><path d="M21 21a4 4 0 0 0-3.81-4"/><path d="M21 5a4 4 0 0 1-3.55 3.97"/><path d="M22 13h-4"/><path d="M3 21a4 4 0 0 1 3.81-4"/><path d="M3 5a4 4 0 0 0 3.55 3.97"/><path d="M6 13H2"/><path d="m8 2 1.88 1.88"/><path d="M9 7.13V6a3 3 0 1 1 6 0v1.13"/>"#
        }
        "book-open" => {
            r#"<path d="M2 3h6a4 4 0 0 1 4 4v14a3 3 0 0 0-3-3H2z"/><path d="M22 3h-6a4 4 0 0 0-4 4v14a3 3 0 0 1 3-3h7z"/>"#
        }
        // === wave5-page-depth-track-a === new icons required by the
        // expanded home / auth / about hero pages. All paths mirror
        // the official lucide.dev SVG body. No existing icons are
        // restyled.
        "share-2" => {
            r#"<circle cx="18" cy="5" r="3"/><circle cx="6" cy="12" r="3"/><circle cx="18" cy="19" r="3"/><line x1="8.59" x2="15.42" y1="13.51" y2="17.49"/><line x1="15.41" x2="8.59" y1="6.51" y2="10.49"/>"#
        }
        "clock" => r#"<circle cx="12" cy="12" r="10"/><polyline points="12 6 12 12 16 14"/>"#,
        "star" => {
            r#"<polygon points="12 2 15.09 8.26 22 9.27 17 14.14 18.18 21.02 12 17.77 5.82 21.02 7 14.14 2 9.27 8.91 8.26 12 2"/>"#
        }
        "circle-check" => r#"<circle cx="12" cy="12" r="10"/><path d="m9 12 2 2 4-4"/>"#,
        "rocket" => {
            r#"<path d="M4.5 16.5c-1.5 1.26-2 5-2 5s3.74-.5 5-2c.71-.84.7-2.13-.09-2.91a2.18 2.18 0 0 0-2.91-.09z"/><path d="m12 15-3-3a22 22 0 0 1 2-3.95A12.88 12.88 0 0 1 22 2c0 2.72-.78 7.5-6 11a22.35 22.35 0 0 1-4 2z"/><path d="M9 12H4s.55-3.03 2-4c1.62-1.08 5 0 5 0"/><path d="M12 15v5s3.03-.55 4-2c1.08-1.62 0-5 0-5"/>"#
        }
        "target" => {
            r#"<circle cx="12" cy="12" r="10"/><circle cx="12" cy="12" r="6"/><circle cx="12" cy="12" r="2"/>"#
        }
        "lightbulb" => {
            r#"<path d="M15 14c.2-1 .7-1.7 1.5-2.5 1-.9 1.5-2.2 1.5-3.5A6 6 0 0 0 6 8c0 1 .2 2.2 1.5 3.5.7.7 1.3 1.5 1.5 2.5"/><path d="M9 18h6"/><path d="M10 22h4"/>"#
        }
        "database" => {
            r#"<ellipse cx="12" cy="5" rx="9" ry="3"/><path d="M3 5V19A9 3 0 0 0 21 19V5"/><path d="M3 12A9 3 0 0 0 21 12"/>"#
        }
        "message-square" => {
            r#"<path d="M21 15a2 2 0 0 1-2 2H7l-4 4V5a2 2 0 0 1 2-2h14a2 2 0 0 1 2 2z"/>"#
        }
        "sparkles" => {
            r#"<path d="m12 3-1.9 5.8a2 2 0 0 1-1.3 1.3L3 12l5.8 1.9a2 2 0 0 1 1.3 1.3L12 21l1.9-5.8a2 2 0 0 1 1.3-1.3L21 12l-5.8-1.9a2 2 0 0 1-1.3-1.3z"/><path d="M5 3v4"/><path d="M19 17v4"/><path d="M3 5h4"/><path d="M17 19h4"/>"#
        }
        "gem" => {
            r#"<path d="M6 3h12l4 6-10 12L2 9Z"/><path d="m12 21 4-12-4-6-4 6 4 12Z"/><path d="M2 9h20"/>"#
        }
        "cpu" => {
            r#"<rect width="16" height="16" x="4" y="4" rx="2"/><rect width="6" height="6" x="9" y="9" rx="1"/><path d="M15 2v2"/><path d="M15 20v2"/><path d="M2 15h2"/><path d="M2 9h2"/><path d="M20 15h2"/><path d="M20 9h2"/><path d="M9 2v2"/><path d="M9 20v2"/>"#
        }
        "play" => r#"<polygon points="6 3 20 12 6 21 6 3"/>"#,
        "arrow-up-right" => r#"<path d="M7 7h10v10"/><path d="M7 17 17 7"/>"#,
        "circle-x" => {
            r#"<circle cx="12" cy="12" r="10"/><path d="m15 9-6 6"/><path d="m9 9 6 6"/>"#
        }
        "triangle-alert" => {
            r#"<path d="m21.73 18-8-14a2 2 0 0 0-3.48 0l-8 14A2 2 0 0 0 4 21h16a2 2 0 0 0 1.73-3Z"/><path d="M12 9v4"/><path d="M12 17h.01"/>"#
        }
        "wifi-off" => {
            r#"<line x1="2" x2="22" y1="2" y2="22"/><path d="M8.5 16.5a5 5 0 0 1 7 0"/><path d="M2 8.82a15 15 0 0 1 4.17-2.65"/><path d="M10.66 5c4.01-.36 8.14.9 11.34 3.76"/><path d="M16.85 11.25a10 10 0 0 1 2.22 1.68"/><path d="M5 13a10 10 0 0 1 5.24-2.76"/><line x1="12" x2="12.01" y1="20" y2="20"/>"#
        }
        // Additional names used by the migrated chat, notifications,
        // analytics and admin surfaces. Keep the aliases here so SSR and
        // client-rendered icons share one complete Lucide registry.
        "arrow-right-left" => {
            r#"<path d="m17 11 4-4-4-4"/><path d="M3 7h18"/><path d="m7 13-4 4 4 4"/><path d="M21 17H3"/>"#
        }
        "bell-off" => {
            r#"<path d="M13.73 21a2 2 0 0 1-3.46 0"/><path d="M18.63 13A17.89 17.89 0 0 1 18 8"/><path d="M6.26 6.26A5.86 5.86 0 0 0 6 8c0 7-3 9-3 9h14"/><path d="M18 8a6 6 0 0 0-9.33-5"/><path d="m1 1 22 22"/>"#
        }
        "bot" => {
            r#"<rect width="18" height="10" x="3" y="8" rx="2"/><path d="M12 4v4"/><path d="M8 12h.01"/><path d="M16 12h.01"/><path d="M7 16h10"/>"#
        }
        "check-check" => r#"<path d="M18 6 7 17l-5-5"/><path d="m22 10-7.5 7.5L13 16"/>"#,
        "circle-alert" => {
            r#"<circle cx="12" cy="12" r="10"/><line x1="12" x2="12" y1="8" y2="12"/><line x1="12" x2="12.01" y1="16" y2="16"/>"#
        }
        "copy" => {
            r#"<rect width="14" height="14" x="8" y="8" rx="2" ry="2"/><path d="M4 16c-1.1 0-2-.9-2-2V4c0-1.1.9-2 2-2h10c1.1 0 2 .9 2 2"/>"#
        }
        "edit" => {
            r#"<path d="M12 20h9"/><path d="M16.5 3.5a2.121 2.121 0 0 1 3 3L8 18l-4 1 1-4Z"/>"#
        }
        "eye" => {
            r#"<path d="M2.062 12.348a1 1 0 0 1 0-.696 10.75 10.75 0 0 1 19.876 0 1 1 0 0 1 0 .696 10.75 10.75 0 0 1-19.876 0"/><circle cx="12" cy="12" r="3"/>"#
        }
        "eye-off" => {
            r#"<path d="M10.733 5.076a10.744 10.744 0 0 1 8.14 2.633c1.17 1.03 2.13 2.33 2.8 3.77a1 1 0 0 1 0 .85 10.77 10.77 0 0 1-4.05 4.73"/><path d="M14.084 14.158a3 3 0 0 1-4.242-4.242"/><path d="M17.479 17.499a10.75 10.75 0 0 1-15.42-5.151 1 1 0 0 1 0-.696 10.75 10.75 0 0 1 4.446-5.15"/><path d="m2 2 20 20"/>"#
        }
        "file-question" => {
            r#"<path d="M15 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V7Z"/><path d="M14 2v6h6"/><path d="M9.1 9a3 3 0 1 1 5.8 1c0 2-2.9 2-2.9 4"/><path d="M12 18h.01"/>"#
        }
        "headset" => {
            r#"<path d="M3 14h3a2 2 0 0 1 2 2v3a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2Z"/><path d="M21 14h-3a2 2 0 0 0-2 2v3a2 2 0 0 0 2 2h1a2 2 0 0 0 2-2Z"/><path d="M3 14v-2a9 9 0 0 1 18 0v2"/><path d="M21 14v-2"/>"#
        }
        "inbox" => {
            r#"<polyline points="22 12 16 12 14 15 10 15 8 12 2 12"/><path d="M5.45 5.11 2 12v6a2 2 0 0 0 2 2h16a2 2 0 0 0 2-2v-6l-3.45-6.89A2 2 0 0 0 16.76 4H7.24a2 2 0 0 0-1.79 1.11Z"/>"#
        }
        "list-restart" => {
            r#"<path d="M21 6H3"/><path d="M7 12H3"/><path d="M7 18H3"/><path d="M16 12h5"/><path d="M16 18h5"/><path d="M13 12h.01"/><path d="M13 18h.01"/><path d="M16 3v3"/><path d="m19 4-3 3-3-3"/>"#
        }
        "loader" => {
            r#"<path d="M12 2v4"/><path d="m16.2 7.8 2.9-2.9"/><path d="M18 12h4"/><path d="m16.2 16.2 2.9 2.9"/><path d="M12 18v4"/><path d="m7.8 16.2-2.9 2.9"/><path d="M6 12H2"/><path d="m7.8 7.8-2.9-2.9"/>"#
        }
        "log-in" => {
            r#"<path d="m10 17 5-5-5-5"/><path d="M15 12H3"/><path d="M21 19V5a2 2 0 0 0-2-2h-6"/>"#
        }
        "pin-off" => {
            r#"<line x1="2" x2="22" y1="2" y2="22"/><path d="M12 17v5"/><path d="M9 10.76a2 2 0 0 1-1.11 1.79l-1.78.9A2 2 0 0 0 5 15.24V16h7"/><path d="M15 7v3.76a2 2 0 0 0 1.11 1.79l1.78.9A2 2 0 0 1 19 15.24V16h-3"/><path d="M8 2h8a2 2 0 0 1 0 4H8"/>"#
        }
        "shield-alert" => {
            r#"<path d="M20 13c0 5-3.5 7.5-7.66 8.95a1 1 0 0 1-.67-.01C7.5 20.5 4 18 4 13V6a1 1 0 0 1 1-1c2 0 4.5-1.2 6.24-2.72a1.17 1.17 0 0 1 1.52 0C14.51 3.81 17 5 19 5a1 1 0 0 1 1 1z"/><path d="M12 8v4"/><path d="M12 16h.01"/>"#
        }
        "sliders-horizontal" => {
            r#"<line x1="21" x2="14" y1="4" y2="4"/><line x1="10" x2="3" y1="4" y2="4"/><line x1="21" x2="12" y1="12" y2="12"/><line x1="8" x2="3" y1="12" y2="12"/><line x1="21" x2="16" y1="20" y2="20"/><line x1="12" x2="3" y1="20" y2="20"/><line x1="14" x2="14" y1="2" y2="6"/><line x1="8" x2="8" y1="10" y2="14"/><line x1="16" x2="16" y1="18" y2="22"/>"#
        }
        "tag" => {
            r#"<path d="M12.586 2.586A2 2 0 0 0 11.172 2H4a2 2 0 0 0-2 2v7.172a2 2 0 0 0 .586 1.414l8.704 8.704a2.426 2.426 0 0 0 3.42 0l6.58-6.58a2.426 2.426 0 0 0 0-3.42z"/><circle cx="7.5" cy="7.5" r=".5"/>"#
        }
        // === wave6b-admin-pages-depth-track-a === new icons required by
        // the 5 admin pages (dashboard, analytics, policies, settings,
        // media). All paths mirror the official lucide.dev SVG body.
        // No existing icons are restyled. The 4 additions:
        // - `download` — analytics export button + media browser
        //   "open" icon.
        // - `layers` — policies stats bar "Total Policies" card.
        // - `activity` — policies monitor "Evaluations (24h)" stat.
        // - `rotate-ccw` — settings dashboard "Reset Logic" button.
        "download" => {
            r#"<path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"/><polyline points="7 10 12 15 17 10"/><line x1="12" x2="12" y1="15" y2="3"/>"#
        }
        "layers" => {
            r#"<polygon points="12 2 2 7 12 12 22 7 12 2"/><polyline points="2 17 12 22 22 17"/><polyline points="2 12 12 17 22 12"/>"#
        }
        "activity" => r#"<polyline points="22 12 18 12 15 21 9 3 6 12 2 12"/>"#,
        "rotate-ccw" => {
            r#"<path d="M3 12a9 9 0 1 0 9-9 9.75 9.75 0 0 0-6.74 2.74L3 8"/><path d="M3 3v5h5"/>"#
        }
        // end wave6b-admin-pages-depth-track-a icon additions

        // === wave6b-admin-pages-depth-track-c === new icons required by
        // the financial-surface admin pages (payments + wallet_credits
        // + wallet_plans + wallet_access). All paths mirror the
        // official lucide.dev SVG body. No existing icons are
        // restyled.
        "refresh-cw" => {
            r#"<path d="M21 12a9 9 0 0 0-9-9 9.75 9.75 0 0 0-6.74 2.74L3 8"/><path d="M3 3v5h5"/><path d="M3 12a9 9 0 0 0 9 9 9.75 9.75 0 0 0 6.74-2.74L21 16"/><path d="M16 16h5v5"/>"#
        }
        "trash" => {
            r#"<path d="M3 6h18"/><path d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6"/><path d="M8 6V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2"/>"#
        }
        "trash-2" => {
            r#"<path d="M3 6h18"/><path d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6"/><path d="M8 6V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2"/><line x1="10" x2="10" y1="11" y2="17"/><line x1="14" x2="14" y1="11" y2="17"/>"#
        }
        "alert-circle" => {
            r#"<circle cx="12" cy="12" r="10"/><line x1="12" x2="12" y1="8" y2="12"/><line x1="12" x2="12.01" y1="16" y2="16"/>"#
        }
        "arrow-left" => r#"<path d="M19 12H5"/><path d="m12 19-7-7 7-7"/>"#,
        "user-check" => {
            r#"<path d="M16 21v-2a4 4 0 0 0-4-4H6a4 4 0 0 0-4 4v2"/><circle cx="9" cy="7" r="4"/><polyline points="16 11 18 13 22 9"/>"#
        }
        "shield-check" => {
            r#"<path d="M20 13c0 5-3.5 7.5-7.66 8.95a1 1 0 0 1-.67-.01C7.5 20.5 4 18 4 13V6a1 1 0 0 1 1-1c2 0 4.5-1.2 6.24-2.72a1.17 1.17 0 0 1 1.52 0C14.51 3.81 17 5 19 5a1 1 0 0 1 1 1z"/><path d="m9 12 2 2 4-4"/>"#
        }
        // === wave38b(t2): added shield-x for the 3 admin outlier
        // pages (access-denied, unauthorized, developer-portal/
        // api-keys/create). Prod renders this icon inside a
        // w-20 h-20 red-gradient shield container; see
        // `tools/e2e-admin/baselines/prod-admin/{admin-access-
        // denied,admin-unauthorized,admin-developer-portal-api-
        // keys-create}.html` for the exact class structure.
        "shield-x" => {
            r#"<path d="M20 13c0 5-3.5 7.5-7.66 8.95a1 1 0 0 1-.67-.01C7.5 20.5 4 18 4 13V6a1 1 0 0 1 1-1c2 0 4.5-1.2 6.24-2.72a1.17 1.17 0 0 1 1.52 0C14.51 3.81 17 5 19 5a1 1 0 0 1 1 1z"/><path d="m14.5 9.5-5 5"/><path d="m9.5 9.5 5 5"/>"#
        }
        _ => "",
    }
}

/// Returns a complete `<svg>` element for a Lucide icon.
/// `size` defaults to 16; pass a number string (e.g. "20") to override.
pub fn lucide(name: &str, size: &str, class: &str) -> String {
    lucide_with_attributes(name, size, class, "")
}

fn lucide_with_attributes(name: &str, size: &str, class: &str, attributes: &str) -> String {
    format!(
        r#"<svg xmlns="http://www.w3.org/2000/svg" width="{sz}" height="{sz}" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="lucide lucide-{name} {class}" {attributes} aria-hidden="true">{body}</svg>"#,
        sz = size,
        name = name,
        class = class,
        attributes = attributes,
        body = lucide_icon(name),
    )
}

pub fn logo(href: &str, size: &str) -> String {
    let cls = if size == "sm" {
        "logo-text-sm"
    } else {
        "logo-text"
    };
    format!(
        r#"<a href="{href}" class="flex items-center gap-2.5 group" style="text-decoration:none;">
  {icon}
  <span class="{cls}">EPSX</span>
</a>"#,
        href = href,
        cls = cls,
        icon = epsx_icon_svg(),
    )
}

/// Returns a theme-aware navbar wrapper opener. Use with `navbar_close()`.
pub fn navbar_open() -> &'static str {
    r#"<nav class="navbar"><div class="container-x flex items-center justify-between" style="height:3.5rem;">"#
}

/// Returns a theme-aware navbar wrapper closer.
pub fn navbar_close() -> &'static str {
    r#"</div></nav>"#
}

/// Returns the page background wrapper opener (gradient bg + orbs).
/// Matches epsx.io: `bg-gradient-to-br from-blue-50 via-orange-50 to-yellow-50`
/// (light) / `dark:from-slate-900 dark:via-slate-800 dark:to-slate-900` (dark).
pub fn page_bg_open() -> &'static str {
    r#"<div class="page-bg relative min-h-screen overflow-hidden bg-gradient-to-br from-blue-50 via-orange-50 to-yellow-50 dark:from-slate-900 dark:via-slate-800 dark:to-slate-900">"#
}

/// Closes the page background wrapper.
pub fn page_bg_close() -> &'static str {
    "</div>"
}

/// Returns three decorative gradient orbs positioned behind the hero.
pub fn hero_orbs() -> &'static str {
    r#"<div class="orb orb-orange" style="width:24rem;height:24rem;top:-6rem;left:-6rem;"></div>
<div class="orb orb-blue" style="width:20rem;height:20rem;top:8rem;right:-4rem;"></div>
<div class="orb orb-purple" style="width:18rem;height:18rem;bottom:0;left:33%;"></div>"#
}

/// Returns a standard footer (matches the FOOTER_LINKS + brand block from
/// the original `nav-config.ts`).
pub fn footer() -> &'static str {
    r##"<footer class="footer">
  <div class="container-x">
    <div style="display:grid;grid-template-columns:repeat(auto-fit, minmax(200px, 1fr));gap:2rem;margin-bottom:2rem;">
      <div>
        <a href="/" style="text-decoration:none;">
          <span class="logo-text">EPSX</span>
        </a>
        <p style="margin-top:0.75rem;font-size:0.875rem;max-width:18rem;">
          Public information and reference content for EPSX.
        </p>
      </div>
      <div>
        <h2 id="footer-platform-heading" style="font-size:0.875rem;font-weight:600;color:var(--text);margin:0 0 0.75rem;">Platform</h2>
        <nav aria-labelledby="footer-platform-heading" style="display:flex;flex-direction:column;gap:0.5rem;">
          <a href="/analytics" class="footer-link">Rankings</a>
          <a href="/portfolio" class="footer-link">Portfolio</a>
          <a href="/plans" class="footer-link">Plans</a>
          <a href="/news" class="footer-link">News</a>
        </nav>
      </div>
      <div>
        <h2 id="footer-developers-heading" style="font-size:0.875rem;font-weight:600;color:var(--text);margin:0 0 0.75rem;">Developers</h2>
        <nav aria-labelledby="footer-developers-heading" style="display:flex;flex-direction:column;gap:0.5rem;">
          <a href="/developer" class="footer-link">API Keys</a>
          <a href="/developer/docs" class="footer-link">Documentation</a>
          <a href="/chat" class="footer-link">Support</a>
        </nav>
      </div>
      <div>
        <h2 id="footer-company-heading" style="font-size:0.875rem;font-weight:600;color:var(--text);margin:0 0 0.75rem;">Company</h2>
        <nav aria-labelledby="footer-company-heading" style="display:flex;flex-direction:column;gap:0.5rem;">
          <a href="/about" class="footer-link">About</a>
          <a href="/contact" class="footer-link">Contact</a>
          <a href="/terms" class="footer-link">Terms of Service</a>
          <a href="/privacy" class="footer-link">Privacy Policy</a>
        </nav>
      </div>
    </div>
    <div style="border-top:1px solid var(--epsx-border);padding-top:1.5rem;display:flex;flex-wrap:wrap;gap:1rem;justify-content:space-between;align-items:center;font-size:0.8125rem;">
      <span>&copy; EPSX. All rights reserved.</span>
      <span>Built on BSC</span>
    </div>
  </div>
</footer>"##
}

/// Renders the EPSX.io-style sticky header.
/// Matches: `sticky top-0 z-50 border-b border-slate-200/60 bg-white/95 backdrop-blur-md dark:border-slate-800 dark:bg-slate-950/95`
/// - Logo (EPSX icon + gradient text)
/// - 2 dropdowns: Market / Company (Developer lives in the wallet/personal menu)
/// - Theme toggle (sun/moon)
/// - Connect button (orange gradient)
pub fn epsx_header() -> String {
    epsx_header_for_session(false)
}

/// Render the public header with a truthful browser-session action.
/// Authentication remains server-derived; public controls use the native auth
/// route and contain no permissions or entitlement logic.
pub fn epsx_header_for_session(is_authenticated: bool) -> String {
    epsx_header_for_session_and_return_target(is_authenticated, "/")
}

/// Render the public header while preserving a safe same-origin return target
/// for signed-out Connect actions. Existing callers can continue to use
/// [`epsx_header_for_session`], which returns to `/`.
pub fn epsx_header_for_session_and_return_target(
    is_authenticated: bool,
    return_target: &str,
) -> String {
    epsx_header_for_session_and_wallet(is_authenticated, return_target, None)
}

/// Render the public header with an optional browser-connected wallet.
///
/// The wallet cookie only describes the connected provider; it never grants
/// access. When present without a server-authenticated session we show the
/// compact wallet identity alongside an explicit sign-in label in the header
/// and leave the verification action to the native `/auth` route.
pub fn epsx_header_for_session_and_wallet(
    is_authenticated: bool,
    return_target: &str,
    wallet_address: Option<&str>,
) -> String {
    epsx_header_for_session_and_wallet_with_network(
        is_authenticated,
        return_target,
        wallet_address,
        false,
    )
}

/// Render the public header with an optional non-interactive network label.
///
/// The development navigation renders the chain selector in the tablet and
/// desktop action cluster. The Rust BFF cannot switch a wallet network from
/// an SSR-only shell, so it exposes the verified local target as a label and
/// leaves network mutation to a future hydrated wallet integration.
pub fn epsx_header_for_session_and_wallet_with_network(
    is_authenticated: bool,
    return_target: &str,
    wallet_address: Option<&str>,
    show_network: bool,
) -> String {
    let auth_href = auth_href_for_return_target(return_target);
    let auth_href = html_attr_escape(&auth_href);
    let current_path = return_target
        .split_once(['?', '#'])
        .map_or(return_target, |(path, _)| path);

    let nav_item = |href: &str, icon_name: &str, label: &str, description: &str| {
        format!(
            r##"
      <a href="{href}" class="epsx-nav-item">
        {icon}
        <div>
          <div class="item-label">{label}</div>
          <div class="item-desc">{description}</div>
        </div>
      </a>"##,
            icon = lucide(icon_name, "16", "item-icon"),
        )
    };

    // Market dropdown items (rankings, portfolio)
    let market_items = format!(
        "{}{}",
        nav_item(
            "/analytics",
            "chart-column",
            "Rankings",
            "Rankings availability"
        ),
        nav_item(
            "/portfolio",
            "trending-up",
            "Portfolio",
            "Portfolio availability"
        )
    );

    // Company dropdown items
    let company_items = format!(
        "{}{}{}{}",
        nav_item("/about", "info", "About", "Mission &amp; vision"),
        nav_item("/news", "newspaper", "News", "News availability"),
        nav_item("/contact", "mail", "Contact", "Contact by email"),
        nav_item("/chat", "help-circle", "Support", "Support status")
    );

    let logo = epsx_icon_svg();
    let notification_action = if is_authenticated {
        format!(
            r##"<a href="/notifications" class="epsx-theme-btn epsx-notification-link" aria-label="Notifications" data-epsx-notification-badge-target="true">
        {icon}
        <span class="epsx-notification-badge" data-epsx-notification-unread-badge="true" data-notification-count="true" data-state="unavailable" aria-hidden="true" hidden></span>
      </a>"##,
            icon = lucide("bell", "16", "epsx-action-icon"),
        )
    } else {
        String::new()
    };
    let (desktop_auth, compact_auth) = if is_authenticated {
        if let Some(address) = wallet_address.filter(|value| !value.trim().is_empty()) {
            let safe_address = html_attr_escape(address.trim());
            let safe_short = html_attr_escape(&short_wallet_address(address));
            (
                format!(
                    r##"<div class="epsx-desktop-session hidden md:flex items-center gap-1.5">
        <div class="epsx-session-menu-wrap">
          <button class="epsx-wallet-pill epsx-session-trigger" type="button" aria-label="Wallet menu for {safe_short}" aria-haspopup="menu" aria-expanded="false" aria-controls="epsx-session-menu-desktop" data-epsx-action="toggle-dropdown">
            {wallet_icon}
            <span>{safe_short}</span>
            {chevron_icon}
          </button>
          <div id="epsx-session-menu-desktop" class="epsx-session-menu" role="menu" data-epsx-dropdown aria-hidden="true" hidden>
            <div class="epsx-session-summary">
              <div class="epsx-session-label">{summary_icon} Wallet</div>
              <code class="epsx-session-address">{safe_address}</code>
            </div>
            <div class="epsx-session-actions">
              <a href="/account" class="epsx-session-menu-item" role="menuitem">{account_icon} Account</a>
              <a href="/developer" class="epsx-session-menu-item" role="menuitem">{developer_icon} Developer</a>
              <a href="/developer/docs" class="epsx-session-menu-item" role="menuitem">{docs_icon} Documentation</a>
              <button class="epsx-session-menu-item" type="button" role="menuitem" data-epsx-action="copy" data-copy="{safe_address}">{copy_icon} Copy address</button>
              <button class="epsx-session-menu-item epsx-session-sign-out" type="button" role="menuitem" data-epsx-logout>{logout_icon} Sign out</button>
            </div>
          </div>
        </div>
      </div>"##,
                    wallet_icon = lucide("wallet", "16", "epsx-action-icon"),
                    chevron_icon = lucide("chevron-down", "12", "epsx-session-chevron"),
                    summary_icon = lucide("wallet", "12", "epsx-action-icon"),
                    account_icon = lucide("user", "16", "epsx-action-icon"),
                    developer_icon = lucide("code", "16", "epsx-action-icon"),
                    docs_icon = lucide("book", "16", "epsx-action-icon"),
                    copy_icon = lucide("copy", "16", "epsx-action-icon"),
                    logout_icon = lucide("log-out", "16", "epsx-action-icon"),
                ),
                format!(
                    r##"<div class="epsx-tablet-session hidden sm:flex md:hidden items-center gap-1.5">
        <div class="epsx-session-menu-wrap">
          <button class="epsx-wallet-pill epsx-session-trigger" type="button" aria-label="Wallet menu for {safe_short}" aria-haspopup="menu" aria-expanded="false" aria-controls="epsx-session-menu-tablet" data-epsx-action="toggle-dropdown" style="height:2rem;padding:0 0.75rem;font-size:0.75rem;border-radius:1rem;">
            {wallet_icon}
            <span>{safe_short}</span>
            {chevron_icon}
          </button>
          <div id="epsx-session-menu-tablet" class="epsx-session-menu" role="menu" data-epsx-dropdown aria-hidden="true" hidden>
            <div class="epsx-session-summary">
              <div class="epsx-session-label">{summary_icon} Wallet</div>
              <code class="epsx-session-address">{safe_address}</code>
            </div>
            <div class="epsx-session-actions">
              <a href="/account" class="epsx-session-menu-item" role="menuitem">{account_icon} Account</a>
              <a href="/developer" class="epsx-session-menu-item" role="menuitem">{developer_icon} Developer</a>
              <a href="/developer/docs" class="epsx-session-menu-item" role="menuitem">{docs_icon} Documentation</a>
              <button class="epsx-session-menu-item" type="button" role="menuitem" data-epsx-action="copy" data-copy="{safe_address}">{copy_icon} Copy address</button>
              <button class="epsx-session-menu-item epsx-session-sign-out" type="button" role="menuitem" data-epsx-logout>{logout_icon} Sign out</button>
            </div>
          </div>
        </div>
      </div>"##,
                    wallet_icon = lucide("wallet", "12", "epsx-action-icon"),
                    chevron_icon = lucide("chevron-down", "10", "epsx-session-chevron"),
                    summary_icon = lucide("wallet", "12", "epsx-action-icon"),
                    account_icon = lucide("user", "14", "epsx-action-icon"),
                    developer_icon = lucide("code", "14", "epsx-action-icon"),
                    docs_icon = lucide("book", "14", "epsx-action-icon"),
                    copy_icon = lucide("copy", "14", "epsx-action-icon"),
                    logout_icon = lucide("log-out", "14", "epsx-action-icon"),
                ),
            )
        } else {
            (
                format!(
                    r##"<div class="epsx-desktop-session hidden md:flex items-center gap-1.5">
        <a href="/developer" class="epsx-theme-btn" aria-label="Developer">{developer_icon}</a>
        <a href="/developer/docs" class="epsx-theme-btn" aria-label="Documentation">{docs_icon}</a>
        <a href="/account" class="epsx-theme-btn" aria-label="Account">{account_icon}</a>
        <button class="epsx-connect-btn" type="button" data-epsx-logout>{logout_icon} Sign out</button>
      </div>"##,
                    account_icon = lucide("user", "16", "epsx-action-icon"),
                    developer_icon = lucide("code", "16", "epsx-action-icon"),
                    docs_icon = lucide("book", "16", "epsx-action-icon"),
                    logout_icon = lucide("log-out", "16", "epsx-action-icon"),
                ),
                format!(
                    r##"<div class="epsx-tablet-session hidden sm:flex md:hidden items-center gap-1.5">
        <a href="/developer" class="epsx-theme-btn" aria-label="Developer" style="height:2rem;padding:0 0.55rem;">{developer_icon}</a>
        <a href="/developer/docs" class="epsx-theme-btn" aria-label="Documentation" style="height:2rem;padding:0 0.55rem;">{docs_icon}</a>
        <button class="epsx-connect-btn" type="button" data-epsx-logout style="height:2rem;padding:0 0.75rem;font-size:0.75rem;border-radius:1rem;">{logout_icon} Sign out</button>
      </div>"##,
                    developer_icon = lucide("code", "12", "epsx-action-icon"),
                    docs_icon = lucide("book", "12", "epsx-action-icon"),
                    logout_icon = lucide("log-out", "12", "epsx-action-icon"),
                ),
            )
        }
    } else if let Some(address) = wallet_address.filter(|value| !value.trim().is_empty()) {
        let short = short_wallet_address(address);
        let safe_short = html_attr_escape(&short);
        (
            format!(
                r##"<div class="epsx-desktop-session hidden md:flex items-center gap-1.5">
        <a href="{auth_href}" class="epsx-wallet-pill" data-epsx-wallet-pill aria-label="Wallet {safe_short} connected; sign in required">
          {wallet_icon}
          Sign in · {safe_short}
        </a>
      </div>"##,
                wallet_icon = lucide("wallet", "16", "epsx-action-icon"),
            ),
            format!(
                r##"<div class="epsx-tablet-session hidden sm:flex md:hidden items-center gap-1.5">
        <a href="{auth_href}" class="epsx-wallet-pill" data-epsx-wallet-pill aria-label="Wallet {safe_short} connected; sign in required" style="height:2rem;padding:0 0.75rem;font-size:0.75rem;border-radius:1rem;">
          {wallet_icon}
          Sign in · {safe_short}
        </a>
      </div>"##,
                wallet_icon = lucide("wallet", "12", "epsx-action-icon"),
            ),
        )
    } else {
        (
            format!(
                r##"<div class="epsx-desktop-session hidden md:flex items-center gap-1.5">
        <a href="{auth_href}" class="epsx-connect-btn" data-epsx-auth-link style="text-decoration:none;">
          {wallet_icon}
          Connect
        </a>
      </div>"##,
                wallet_icon = lucide("wallet", "16", "epsx-action-icon"),
            ),
            format!(
                r##"<div class="epsx-tablet-session hidden sm:flex md:hidden items-center gap-1.5">
        <a href="{auth_href}" class="epsx-connect-btn" data-epsx-auth-link style="height:2rem;padding:0 0.75rem;font-size:0.75rem;border-radius:1rem;text-decoration:none;">
          {wallet_icon}
          Connect
        </a>
      </div>"##,
                wallet_icon = lucide("wallet", "12", "epsx-action-icon"),
            ),
        )
    };
    let network_indicator = if show_network {
        format!(
            r##"<div class="epsx-network-badge" data-epsx-network="bsc-testnet" aria-label="Current network: BSC Testnet">
        {icon}
        <span>BSC Testnet</span>
      </div>"##,
            icon = lucide("link", "16", "epsx-action-icon"),
        )
    } else {
        String::new()
    };
    let mobile_auth = if is_authenticated {
        let authenticated_wallet = wallet_address
            .filter(|value| !value.trim().is_empty())
            .map(|address| {
                format!(
                    "{} Wallet {}",
                    lucide("wallet", "16", "epsx-mobile-icon"),
                    html_attr_escape(&short_wallet_address(address))
                )
            })
            .unwrap_or_else(|| format!("{} Account", lucide("user", "16", "epsx-mobile-icon")));
        format!(
            r##"<a href="/account" class="epsx-mobile-link">
        {authenticated_wallet}
      </a>
      <a href="/developer" class="epsx-mobile-link">
        {developer_icon} Developer
      </a>
      <a href="/developer/docs" class="epsx-mobile-link">
        {docs_icon} Documentation
      </a>
      <button class="epsx-mobile-link" type="button" data-epsx-logout style="width:100%;border:0;background:transparent;text-align:left;">
        {logout_icon} Sign out
      </button>"##,
            developer_icon = lucide("code", "16", "epsx-mobile-icon"),
            docs_icon = lucide("book", "16", "epsx-mobile-icon"),
            logout_icon = lucide("log-out", "16", "epsx-mobile-icon"),
        )
    } else if let Some(address) = wallet_address.filter(|value| !value.trim().is_empty()) {
        format!(
            r##"<a href="{auth_href}" class="epsx-mobile-link" data-epsx-wallet-pill>
        {wallet_icon} Sign in {short}
      </a>"##,
            wallet_icon = lucide("wallet", "16", "epsx-mobile-icon"),
            short = html_attr_escape(&short_wallet_address(address)),
        )
    } else {
        format!(
            r##"<a href="{auth_href}" class="epsx-mobile-connect" data-epsx-auth-link>
        {wallet_icon} Connect
      </a>"##,
            wallet_icon = lucide("wallet", "16", "epsx-mobile-icon"),
        )
    };
    let path_is_active = |href: &str| {
        current_path == href
            || current_path
                .strip_prefix(href)
                .is_some_and(|suffix| suffix.starts_with('/'))
    };
    let group_is_active = |label: &str| match label {
        "Market" => path_is_active("/analytics") || path_is_active("/portfolio"),
        "Company" => ["/about", "/news", "/contact", "/chat"]
            .into_iter()
            .any(path_is_active),
        _ => false,
    };
    let nav_block = |label: &str, icon: &str, items: &str| -> String {
        let id = label.to_ascii_lowercase();
        let active = group_is_active(label);
        let active_class = if active { " active" } else { "" };
        format!(
            r##"<div class="epsx-nav-wrap" data-nav="{label}">
  <button id="epsx-nav-{id}-trigger" class="epsx-nav-trigger{active_class}" type="button" aria-expanded="false" aria-controls="epsx-nav-{id}-panel" data-epsx-action="toggle-nav">
    {nav_icon}
    {label}
    {chevron_icon}
  </button>
  <div id="epsx-nav-{id}-panel" class="epsx-nav-menu" aria-labelledby="epsx-nav-{id}-trigger" hidden>{items}</div>
</div>"##,
            id = id,
            label = label,
            active_class = active_class,
            nav_icon = lucide(icon, "16", "nav-icon"),
            chevron_icon = lucide("chevron-down", "12", "nav-chev"),
            items = items,
        )
    };
    let mobile_item = |href: &str, icon_name: &str, label: &str| -> String {
        let active_class = if path_is_active(href) { " active" } else { "" };
        format!(
            r##"<a href="{href}" class="epsx-mobile-link{active_class}">
        {icon} {label}
      </a>"##,
            icon = lucide(icon_name, "16", "epsx-mobile-icon"),
        )
    };
    let mobile_group = |label: &str, icon_name: &str, items: &str| -> String {
        let id = label.to_ascii_lowercase();
        let active = group_is_active(label);
        let active_class = if active { " active" } else { "" };
        let expanded = if active { "true" } else { "false" };
        let hidden = if active { "" } else { " hidden" };
        format!(
            r##"<div class="epsx-mobile-group">
      <button id="epsx-mobile-{id}-trigger" class="epsx-mobile-group-trigger{active_class}" type="button" aria-expanded="{expanded}" aria-controls="epsx-mobile-{id}-panel" data-epsx-action="toggle-nav">
        <span class="epsx-mobile-group-label">
          {icon} {label}
        </span>
        {chevron}
      </button>
      <div id="epsx-mobile-{id}-panel" class="epsx-mobile-group-items" aria-labelledby="epsx-mobile-{id}-trigger"{hidden}>
        {items}
      </div>
    </div>"##,
            icon = lucide(icon_name, "16", "epsx-mobile-icon"),
            chevron = lucide("chevron-right", "16", "epsx-mobile-chevron"),
        )
    };
    let mobile_market_items = format!(
        "{}{}",
        mobile_item("/analytics", "chart-line", "Rankings"),
        mobile_item("/portfolio", "trending-up", "Portfolio")
    );
    let mobile_company_items = format!(
        "{}{}{}{}",
        mobile_item("/about", "info", "About"),
        mobile_item("/news", "newspaper", "News"),
        mobile_item("/contact", "mail", "Contact"),
        mobile_item("/chat", "help-circle", "Support")
    );
    let theme_sun = lucide_with_attributes("sun", "16", "sun", r#"data-epsx-theme-icon="sun""#);
    let theme_moon = lucide_with_attributes("moon", "16", "moon", r#"data-epsx-theme-icon="moon""#);

    format!(
        r##"<header class="epsx-header" data-epsx-authenticated="{authenticated}">
  <div class="mx-auto flex h-14 max-w-7xl items-center justify-between px-4 md:px-6">
    <div class="epsx-desktop-navigation hidden lg:flex items-center gap-6">
      <a href="/" class="flex items-center gap-2.5 group" style="text-decoration:none;">
        {logo}
        <span class="text-xl font-black tracking-widest text-transparent bg-clip-text bg-gradient-to-r from-[#488BFA] to-[#A43FF3] leading-none mt-0.5">EPSX</span>
      </a>

      <nav class="flex items-center gap-0.5" aria-label="Primary">
        {market}
        {company}
      </nav>
    </div>

    <a href="/" class="epsx-compact-brand lg:hidden flex items-center gap-2.5 group" style="text-decoration:none;">
      {logo}
      <span class="text-xl font-black tracking-widest text-transparent bg-clip-text bg-gradient-to-r from-[#488BFA] to-[#A43FF3] leading-none mt-0.5">EPSX</span>
    </a>

    <div class="flex items-center gap-2">
      {notification_action}
      <button class="epsx-theme-btn" type="button" data-epsx-theme-toggle data-epsx-action="theme-toggle" aria-label="Toggle theme">
        {theme_sun}
        {theme_moon}
      </button>
      {network_indicator}
      {desktop_auth}
      {compact_auth}
      <!-- Mobile menu toggle (< 1024px) -->
      <button class="epsx-theme-btn lg:hidden" type="button" aria-label="Open menu" aria-expanded="false" aria-controls="epsx-mobile-sheet" data-epsx-action="toggle-mobile-menu" id="epsx-mobile-menu-btn" style="width:2.25rem;height:2.25rem;padding:0;">
        {menu_icon}
      </button>
    </div>
  </div>
</header>
<div id="epsx-mobile-sheet" class="epsx-mobile-sheet">
  <div class="epsx-mobile-sheet-inner" role="dialog" aria-modal="true" aria-label="Mobile navigation">
    <div class="epsx-mobile-sheet-header">
      <a href="/" class="flex items-center gap-2.5 group" style="text-decoration:none;">
        {logo}
        <span class="text-xl font-black tracking-widest text-transparent bg-clip-text bg-gradient-to-r from-[#488BFA] to-[#A43FF3] leading-none mt-0.5">EPSX</span>
      </a>
      <button class="epsx-theme-btn" type="button" aria-label="Close menu" aria-controls="epsx-mobile-sheet" data-epsx-action="toggle-mobile-menu">
        {close_icon}
      </button>
    </div>
    <nav class="epsx-mobile-navigation" aria-label="Mobile">
      {mobile_market}
      {mobile_company}
    </nav>
    <div class="epsx-mobile-session">
      {mobile_auth}
    </div>
  </div>
</div>"##,
        logo = logo,
        market = nav_block("Market", "chart-column", &market_items),
        company = nav_block("Company", "building", &company_items),
        notification_action = notification_action,
        desktop_auth = desktop_auth,
        compact_auth = compact_auth,
        mobile_auth = mobile_auth,
        authenticated = is_authenticated,
        theme_sun = theme_sun,
        theme_moon = theme_moon,
        menu_icon = lucide("menu", "18", "epsx-mobile-menu-icon"),
        close_icon = lucide("x", "18", "epsx-mobile-menu-icon"),
        mobile_market = mobile_group("Market", "chart-column", &mobile_market_items),
        mobile_company = mobile_group("Company", "building", &mobile_company_items),
    )
}

fn short_wallet_address(address: &str) -> String {
    let trimmed = address.trim();
    if trimmed.chars().count() <= 10 {
        return trimmed.to_string();
    }
    let prefix: String = trimmed.chars().take(6).collect();
    let suffix: String = trimmed
        .chars()
        .rev()
        .take(4)
        .collect::<String>()
        .chars()
        .rev()
        .collect();
    format!("{prefix}…{suffix}")
}

/// Purple-to-teal prompt shown when a wallet is connected but the SIWE
/// session is not authenticated yet. The link is server-safe and preserves
/// the current same-origin return target.
pub fn epsx_wallet_sign_in_banner(return_target: &str) -> String {
    let href = html_attr_escape(&auth_href_for_return_target(return_target));
    format!(
        r##"<div class="epsx-sign-in-banner" role="region" aria-label="Sign-in prompt">
  <span class="font-medium opacity-90">Your wallet is connected —</span>
  <a href="{href}" class="epsx-sign-in-banner-action">Sign In with Wallet</a>
  <span class="opacity-70">to access all features</span>
</div>"##
    )
}

fn auth_href_for_return_target(candidate: &str) -> String {
    let target = safe_shell_return_target(candidate);
    format!("/auth?return_url={}", percent_encode_query_value(target))
}

fn safe_shell_return_target(candidate: &str) -> &str {
    let route_path = candidate
        .split_once(['?', '#'])
        .map_or(candidate, |(path, _)| path);
    if candidate.is_empty()
        || !candidate.starts_with('/')
        || candidate.starts_with("//")
        || candidate.contains('\\')
        || candidate.chars().any(char::is_control)
        || route_path == "/auth"
    {
        "/"
    } else {
        candidate
    }
}

fn percent_encode_query_value(value: &str) -> String {
    let mut encoded = String::with_capacity(value.len());
    for byte in value.bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                encoded.push(byte as char);
            }
            _ => encoded.push_str(&format!("%{byte:02X}")),
        }
    }
    encoded
}

/// A standard page shell. Returns the complete `<!DOCTYPE html>...<body>...</body></html>`
/// wrapper used by every BFF page. BFFs just supply the `<nav>` content and
/// the body content.
pub fn page_shell(
    title: &str,
    description: &str,
    nav: &str,
    body: &str,
    include_footer: bool,
) -> String {
    page_shell_with_body_class(title, description, nav, body, include_footer, "")
}

/// Same as `page_shell` but lets the caller add a class to the `<body>` tag.
/// Pass `body_class = "page-bg"` to apply the gradient page background.
pub fn page_shell_with_body_class(
    title: &str,
    description: &str,
    nav: &str,
    body: &str,
    include_footer: bool,
    body_class: &str,
) -> String {
    page_shell_with_body_class_and_keywords(
        title,
        description,
        None,
        nav,
        body,
        include_footer,
        body_class,
    )
}

/// Same as [`page_shell_with_body_class`] with optional route-owned keywords.
pub fn page_shell_with_body_class_and_keywords(
    title: &str,
    description: &str,
    keywords: Option<&str>,
    nav: &str,
    body: &str,
    include_footer: bool,
    body_class: &str,
) -> String {
    let footer_html = if include_footer { footer() } else { "" };
    format!(
        r##"<!DOCTYPE html>
<html lang="en" class="dark">
<head>
{head}
{js}
</head>
<body class="min-h-screen {body_class}">
<a class="epsx-skip-link" href="#epsx-main-content">Skip to main content</a>
{nav}
<main id="epsx-main-content" tabindex="-1" style="min-height:calc(100vh - 3.5rem);">
{body}
</main>
{footer}
</body>
</html>"##,
        head = design_system_head_with_keywords(title, description, keywords),
        js = global_js(),
        nav = nav,
        body = body,
        footer = footer_html,
        body_class = body_class,
    )
}

// === wave3a-wiring-track-b ===
//
// Wave 3a Track B — BFF plumbing for wallet state. This track adds
// `PageContext::wallet` and `ConnectedWalletState::from_cookies(...)`.
// The track does NOT add any CSS — the stub reads cookies as a
// no-op and the navbar cluster already exists from Wave 2. The
// marker block is reserved here per the Wave 3a CSS region
// convention (see `docs/wave3a-wiring/design.md` §3).

#[cfg(test)]
mod page_head_tests {
    use super::*;

    #[test]
    fn page_head_escapes_metadata_and_contains_no_inline_script() {
        let head = design_system_head_with_keywords(
            "Title </title>",
            "Description <unsafe>",
            Some("analytics & markets"),
        );
        assert!(head.contains("Title &lt;/title&gt;"));
        assert!(head.contains("Description &lt;unsafe&gt;"));
        assert!(!head.contains("<script"));
    }

    #[test]
    fn shared_theme_css_has_no_merge_artifacts_and_keeps_light_auth_readable() {
        let head = design_system_head("Title", "Description");
        assert!(
            !head.lines().any(|line| line.trim() == "======="),
            "standalone merge markers invalidate the following CSS rule"
        );
        assert!(head.contains("color: var(--text) !important;"));
        assert!(head.contains("html:not(.dark) .auth-modal-headline"));
        assert!(head.contains("html:not(.dark) .wallet-option"));
    }

    #[test]
    fn legacy_template_tokens_do_not_shadow_tailwind_hsl_channels() {
        let head = design_system_head("Title", "Description");
        assert!(head.contains("--epsx-border:"));
        assert!(head.contains("--epsx-primary:"));
        assert!(!head
            .lines()
            .any(|line| line.trim_start().starts_with("--border:")));
        assert!(!head
            .lines()
            .any(|line| line.trim_start().starts_with("--primary:")));
        assert!(head.contains(".admin-header-chrome"));
        assert!(head.contains(".admin-app-shell .btn-primary"));
    }

    #[test]
    fn admin_wallet_address_has_an_explicit_desktop_display_rule() {
        let head = design_system_head("Title", "Description");
        assert!(head.contains(".admin-wallet-short-address,"));
        assert!(
            head.contains(".admin-wallet-connect-label { display: inline; white-space: nowrap; }")
        );
        assert!(head.contains("@media (max-width: 1023px)"));
        assert!(head.contains(".admin-wallet-connect-label { display: none; }"));
    }

    #[test]
    fn shell_loads_only_the_generated_rust_wasm_module() {
        let shell = page_shell("Title", "Description", "", "<p>body</p>", false);
        assert_eq!(shell.matches("<script").count(), 1);
        assert!(shell.contains("epsx_browser_runtime_bootstrap.js?rev=4"));
        assert!(shell.contains("data-epsx-generated-runtime=\"wasm-bindgen\""));
        assert!(!shell.contains("onclick=\""));
    }

    #[test]
    fn progressive_controls_use_escaped_data_contracts() {
        let copy = copy_button_html("\" autofocus onfocus=\"unsafe", "Copy <value>");
        assert!(copy.contains("data-epsx-action=\"copy\""));
        assert!(copy.contains("&quot; autofocus onfocus=&quot;unsafe"));
        assert!(copy.contains("Copy &lt;value&gt;"));
        assert!(!copy.contains("onclick=\""));

        let share = share_button_html("share text", "share title", "Share");
        assert!(share.contains("data-epsx-action=\"share\""));
        assert!(!share.contains("onclick=\""));

        let theme = theme_toggle_button();
        assert!(theme.contains("data-epsx-action=\"theme-toggle\""));
        assert!(theme.contains("<svg"));
        assert!(!theme.contains("data-lucide"));
        assert!(!theme.contains("onclick=\""));
    }

    #[test]
    fn public_header_renders_inline_svg_for_every_navigation_surface() {
        let signed_out = epsx_header_for_session_and_wallet_with_network(
            false,
            "/analytics?country=america",
            None,
            true,
        );
        let signed_in = epsx_header_for_session_and_wallet_with_network(
            true,
            "/analytics?country=america",
            Some("0x1234567890abcdef1234567890abcdef1234abcd"),
            true,
        );

        for header in [&signed_out, &signed_in] {
            assert!(header.contains("<svg"));
            assert!(!header.contains("data-lucide"));
            assert!(!header.contains("<i "));
            assert_eq!(header.matches("data-epsx-action=\"toggle-nav\"").count(), 4);
            assert_eq!(
                header
                    .matches("data-epsx-action=\"toggle-mobile-menu\"")
                    .count(),
                2
            );
            assert!(header.contains("data-epsx-action=\"theme-toggle\""));
        }

        assert_eq!(signed_out.matches("data-epsx-auth-link").count(), 3);
        assert!(signed_in.contains("href=\"/notifications\""));
        assert!(signed_in.contains("href=\"/account\""));
        assert!(signed_in.contains("href=\"/developer\""));
        assert!(signed_in.contains("href=\"/developer/docs\""));
        assert!(signed_in.contains("Wallet menu for 0x1234…abcd"));
        assert_eq!(
            signed_in
                .matches("data-epsx-action=\"toggle-dropdown\"")
                .count(),
            2
        );
        assert_eq!(signed_in.matches("data-copy=\"").count(), 2);
        assert_eq!(signed_in.matches("data-epsx-logout").count(), 3);
        assert!(!signed_in.contains("class=\"epsx-connect-btn\" type=\"button\" data-epsx-logout"));
    }

    #[test]
    fn connected_wallet_header_still_exposes_required_sign_in_action() {
        let header = epsx_header_for_session_and_wallet(
            false,
            "/developer?tab=keys",
            Some("0x1234567890abcdef1234567890abcdef1234abcd"),
        );

        assert!(header.contains("Sign in · 0x1234…abcd"));
        assert!(header.contains("Wallet 0x1234…abcd connected; sign in required"));
        assert!(header.contains("/auth?return_url=%2Fdeveloper%3Ftab%3Dkeys"));
        assert!(!header.contains("data-epsx-authenticated=\"true\""));
    }

    #[test]
    fn public_header_responsive_contract_has_no_navigation_gap() {
        let header = epsx_header();
        assert!(header.contains("epsx-desktop-navigation hidden lg:flex"));
        assert!(header.contains("epsx-compact-brand lg:hidden flex"));
        assert!(header.contains("class=\"epsx-theme-btn lg:hidden\""));
        assert!(header.contains("aria-controls=\"epsx-mobile-sheet\""));
        assert!(header.contains("id=\"epsx-mobile-market-trigger\""));
        assert!(header.contains("id=\"epsx-mobile-company-trigger\""));
        assert!(!header.contains("id=\"epsx-mobile-developer-trigger\""));
        assert!(
            !header.contains("id=\"epsx-mobile-sheet\" class=\"epsx-mobile-sheet\" aria-hidden")
        );

        let shell = page_shell("Title", "Description", &header, "body", false);
        assert!(shell.contains("@media (min-width: 1024px)"));
        assert!(shell.contains(".epsx-header #epsx-mobile-menu-btn { display: none !important; }"));
        assert!(shell.contains("width: 85vw;"));
        assert!(shell.contains("height: 100dvh;"));
        assert!(shell.contains(
            "@media (min-width: 1024px) { .epsx-mobile-sheet { display: none !important; } }"
        ));
    }

    #[test]
    fn navigation_menu_css_matches_the_runtime_open_state() {
        let header = epsx_header();
        assert!(header.contains("class=\"epsx-nav-menu\""));
        assert!(header.contains(" hidden>"));

        let shell = page_shell("Title", "Description", &header, "body", false);
        assert!(shell.contains(".epsx-nav-menu.open { display: block; }"));
    }

    #[test]
    fn shell_keeps_a_single_accessible_main_target() {
        let shell = page_shell("Title", "Description", "<nav>nav</nav>", "body", true);
        assert_eq!(shell.matches("id=\"epsx-main-content\"").count(), 1);
        assert_eq!(shell.matches("href=\"#epsx-main-content\"").count(), 1);
        assert!(shell.contains("tabindex=\"-1\""));
    }
}
