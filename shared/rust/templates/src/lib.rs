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
     28 honest verdict + T1 deliverable. -->
<link rel="stylesheet" href="/public/dist/tailwind.css" />
<style>
  :root {{
    /* Brand palette (matches original) */
    --epsx-blue-start: #488BFA;
    --epsx-blue-end:   #A43FF3;
    --epsx-orange:     #f97316;
    --epsx-yellow:     #eab308;
    --epsx-amber:      #f59e0b;
    --epsx-purple:     #a855f7;
    --epsx-cyan:       #06b6d4;
    --epsx-green:      #10b981;
    --epsx-red:        #ef4444;
    --epsx-pink:       #ec4899;

    /* Light mode tokens */
    --bg:              #ffffff;
    --bg-secondary:    #f8fafc;
    --bg-tertiary:     #f1f5f9;
    --surface:         rgba(255, 255, 255, 0.80);
    --surface-hover:   rgba(255, 255, 255, 0.95);
    --surface-solid:   #ffffff;
    --epsx-border:          #e2e8f0;
    --epsx-border-strong:   #cbd5e1;
    --text:            #0f172a;
    --text-muted:      #475569;
    --text-subtle:     #64748b;
    /* White text on blue-500 is only 3.68:1.  Use the darker pair for
     * light-theme controls so normal-sized button copy clears WCAG AA. */
    --epsx-primary:         #2563eb;
    --epsx-primary-hover:   #1d4ed8;
    --focus-ring:      #c2410c;
    --shadow-sm:       0 1px 2px 0 rgba(0, 0, 0, 0.05);
    --shadow:          0 4px 6px -1px rgba(0, 0, 0, 0.1), 0 2px 4px -2px rgba(0, 0, 0, 0.05);
    --shadow-lg:       0 10px 15px -3px rgba(0, 0, 0, 0.1), 0 4px 6px -4px rgba(0, 0, 0, 0.05);
    --shadow-xl:       0 20px 25px -5px rgba(0, 0, 0, 0.1), 0 8px 10px -6px rgba(0, 0, 0, 0.05);
    --shadow-2xl:      0 25px 50px -12px rgba(0, 0, 0, 0.25);
    --shadow-orange:   0 20px 25px -5px rgba(249, 115, 22, 0.25);

    /* Gradients */
    --gradient-brand:  linear-gradient(135deg, #488BFA 0%, #A43FF3 100%);
    --gradient-warm:   linear-gradient(135deg, #f97316 0%, #eab308 50%, #ea580c 100%);
    --gradient-cool:   linear-gradient(135deg, #3b82f6 0%, #06b6d4 100%);
    --gradient-purple: linear-gradient(135deg, #a855f7 0%, #ec4899 100%);
    --gradient-page:   linear-gradient(135deg, #eff6ff 0%, #fff7ed 50%, #fefce8 100%);
    --gradient-card:   linear-gradient(135deg, rgba(59,130,246,0.05) 0%, rgba(168,85,247,0.05) 100%);

    /* Glassmorphism */
    --glass-bg:        rgba(255, 255, 255, 0.80);
    --glass-border:    rgba(249, 115, 22, 0.20);
    --glass-blur:      12px;
    --glass-shadow:    0 8px 32px rgba(0, 0, 0, 0.08);

    /* The pinned source's Tailwind `font-sans` body utility wins over its
     * lower-layer `next/font` declaration. Match the effective computed stack
     * recorded in source layout evidence. */
    --font-sans:       ui-sans-serif, system-ui, sans-serif, "Apple Color Emoji", "Segoe UI Emoji", "Segoe UI Symbol", "Noto Color Emoji";
    --font-mono:       ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, "Liberation Mono", "Courier New", monospace;

    /* Wave 26 T1 — design tokens for the v3-style gradient
     * utilities (used by `from-purple-900/40` / `via-pink-900/40`
     * etc. on the portfolio upsell banner). Pure CSS-var, not
     * the opacity-modified utility classes themselves — the
     * v3 color overrides live further down. `--pancake-gradient`
     * is named to match the prod `pancake-gradient-text` /
     * `pancake-gradient` utility classes; `--glass-bg` and
     * `--glass-border` are intentionally NOT redeclared here —
     * the existing declarations above (lines 108-109) are the
     * canonical values used by `.card-glass`, `.btn-glass`, and
     * 4 other rule sets. Re-declaring them with a 0.05 alpha
     * (which the previous draft did) silently broke every
     * glassmorphism surface. */
    --pancake-gradient: linear-gradient(135deg, var(--epsx-blue-start), var(--epsx-blue-end));
  }}

  html.dark {{
    /* Wave 49 T1 (Plan 13): dev previously set --bg to #030712
     * (very dark blue with cyan tint). Prod (epsx.io) uses a
     * neutral warm-dark background (sampled #211511 / #171717 /
     * #1c1411 / #121212 across marketing pages). Changing --bg
     * to a warm-neutral grey fixes the auth/about/contact/offline
     * pages' background pixel diff in one shot — those pages
     * inherit --bg from html rather than using MarketingBackground.
     * Side effects: every dark-mode page now has a slightly warmer
     * tone; visually this matches prod's design language. */
    --bg:              #1c1917;
    --bg-secondary:    #171717;
    --bg-tertiary:     #262626;
    --surface:         rgba(15, 23, 42, 0.80);
    --surface-hover:   rgba(15, 23, 42, 0.95);
    --surface-solid:   #0f172a;
    --epsx-border:          #1e293b;
    --epsx-border-strong:   #334155;
    --text:            #f1f5f9;
    --text-muted:      #94a3b8;
    --text-subtle:     #64748b;
    --focus-ring:      #f97316;
    --shadow-sm:       0 1px 2px 0 rgba(0, 0, 0, 0.4);
    --shadow:          0 4px 6px -1px rgba(0, 0, 0, 0.5), 0 2px 4px -2px rgba(0, 0, 0, 0.3);
    --shadow-lg:       0 10px 15px -3px rgba(0, 0, 0, 0.6), 0 4px 6px -4px rgba(0, 0, 0, 0.4);
    --shadow-xl:       0 20px 25px -5px rgba(0, 0, 0, 0.7), 0 8px 10px -6px rgba(0, 0, 0, 0.4);
    --shadow-2xl:      0 25px 50px -12px rgba(0, 0, 0, 0.8);
    --shadow-orange:   0 20px 25px -5px rgba(249, 115, 22, 0.50);
    --gradient-page:   linear-gradient(135deg, #1c1917 0%, #171717 50%, #0f0f0f 100%);
    --gradient-card:   linear-gradient(135deg, rgba(59,130,246,0.10) 0%, rgba(168,85,247,0.10) 100%);
    --glass-bg:        rgba(15, 23, 42, 0.80);
    --glass-border:    rgba(249, 115, 22, 0.30);
    --glass-shadow:    0 8px 32px rgba(0, 0, 0, 0.40);
  }}

  /* Wave 24 T4' — dark mode overrides for the light-pastel rules
   * defined further down (`.marketing-bg-fixed`, `.glass-bg`,
   * `.surface`, etc.). The base rules were originally authored
   * for the light theme (epsx.io light-mode marketing pages) and
   * never received a `html.dark` override, so on the dark dev
   * BFF the page rendered as a pastel rainbow on a black body
   * background — the dominant cause of the /`/` and /`/about`/
   * pixel diff before this fix. */
  html.dark .marketing-bg-fixed {{
    /* Wave 49 T1 (Plan 13): dev previously used a slate→indigo→stone
     * gradient (#0f172a → #1e1b4b → #0c0a09) which produced visibly
     * purple/blue backgrounds (sampled #201e2c / #1e1b49 / #17193e
     * across the viewport). Prod (epsx.io) uses a neutral warm-dark
     * background (sampled #211511 / #171717 / #1c1411 / #121212) with
     * an orange radial-glow overlay from `.marketing-bg-gradient`.
     * The fix drops to a flat warm-neutral base color so the existing
     * orange radial overlay reads correctly and matches prod. */
    background: #171717;
  }}
  /* Wave 24 T4' — the orbs/meshes on `<MarketingBackground>` were
   * authored for a light pastel background (where their warm hues
   * blended in). On the dark background they read as bright orange
   * / purple blurs that prod (epsx.io's home page) does not have.
   * Prod's home uses a flat dark page with a single subtle mesh
   * overlay; we approximate that by dimming the orbs + meshes in
   * dark mode. The base rules below define the 4 orbs at 20-30%
   * opacity; the dark override drops them to 4-6% so they read
   * as ambient depth, not foreground haze. */
  html.dark .marketing-orb-orange {{
    background: linear-gradient(to bottom right, rgba(251, 146, 60, 0.04), rgba(250, 204, 21, 0.04));
  }}
  html.dark .marketing-orb-blue {{
    background: linear-gradient(to bottom right, rgba(96, 165, 250, 0.04), rgba(34, 211, 238, 0.04));
  }}
  html.dark .marketing-orb-purple {{
    background: linear-gradient(to bottom right, rgba(192, 132, 252, 0.04), rgba(244, 114, 182, 0.04));
  }}
  html.dark .marketing-orb-green {{
    background: linear-gradient(to bottom right, rgba(74, 222, 128, 0.03), rgba(16, 185, 129, 0.03));
  }}
  html.dark .marketing-mesh-orange {{ background: radial-gradient(circle at 25% 25%, rgba(255, 133, 27, 0.03) 0%, transparent 50%); }}
  html.dark .marketing-mesh-blue   {{ background: radial-gradient(circle at 75% 75%, rgba(59, 130, 246, 0.03) 0%, transparent 50%); }}
  html.dark .marketing-mesh-purple {{ background: radial-gradient(circle at 50% 50%, rgba(168, 85, 247, 0.02) 0%, transparent 60%); }}
  html.dark .marketing-shape-square {{
    background: linear-gradient(to bottom right, rgba(251, 146, 60, 0.02), rgba(250, 204, 21, 0.02));
  }}
  html.dark .marketing-shape-circle {{
    background: linear-gradient(to bottom right, rgba(96, 165, 250, 0.03), rgba(34, 211, 238, 0.03));
  }}
  /* Hero section orbs - dimmed for dark mode (still ambient depth, no foreground haze) */
  html.dark .hero-orb-1 {{ background: rgba(251, 146, 60, 0.06); opacity: 0.5; }}
  html.dark .hero-orb-2 {{ background: rgba(96, 165, 250, 0.06); opacity: 0.5; }}
  html.dark .hero-orb-3 {{ background: rgba(192, 132, 252, 0.04); opacity: 0.5; }}
  html.dark .hero-orb-4 {{ background: rgba(74, 222, 128, 0.04); opacity: 0.5; }}
  html.dark .glass-bg {{
    background: rgba(15, 23, 42, 0.65);
  }}
  html.dark .card-glass {{
    background: rgba(15, 23, 42, 0.55);
    border-color: rgba(249, 115, 22, 0.30);
  }}
  html.dark .card-insight {{
    background: rgba(15, 23, 42, 0.55);
    border-color: rgba(148, 163, 184, 0.18);
  }}
  html.dark .card {{
    background: rgba(15, 23, 42, 0.55);
    border-color: rgba(148, 163, 184, 0.18);
  }}
  /* Wave 24 T4' — light-mode color hardcodes (the `.about-hero-sub`,
   * `.mission-card-body`, etc. use `rgb(...)` literals for text
   * color so they don't auto-flip on dark mode). Override to
   * light text colors when dark mode is active. */
  html.dark .about-hero-sub {{ color: rgb(203, 213, 225); }}
  html.dark .mission-card-body {{ color: rgb(203, 213, 225); }}
  html.dark .mission-card-values-list li {{ color: rgb(203, 213, 225); }}
  html.dark .about-team-name {{ color: rgb(241, 245, 249); }}
  html.dark .about-team-bio {{ color: rgb(203, 213, 225); }}
  html.dark .about-timeline-title {{ color: rgb(241, 245, 249); }}
  html.dark .about-timeline-body {{ color: rgb(203, 213, 225); }}
  html.dark .about-stat-label {{ color: rgb(148, 163, 184); }}
  html.dark .datatech-card-body {{ color: rgb(203, 213, 225); }}
  html.dark .datatech-why-list li {{ color: rgb(203, 213, 225); }}
  html.dark .datatech-feature-body {{ color: rgb(203, 213, 225); }}
  html.dark .datatech-benefit-item {{ color: rgb(203, 213, 225); }}
  html.dark .section-title {{ color: rgb(241, 245, 249); }}
  html.dark .section-sub {{ color: rgb(148, 163, 184); }}
  html.dark .hero-subtitle {{ color: rgb(203, 213, 225); }}
  html.dark .hero-stat-label {{ color: rgb(203, 213, 225); }}
  html.dark .feature-title {{ color: rgb(241, 245, 249); }}
  html.dark .feature-description {{ color: rgb(203, 213, 225); }}
  html.dark .pricing-teaser-tier {{ color: rgb(148, 163, 184); }}
  html.dark .pricing-teaser-price {{ color: rgb(241, 245, 249); }}
  html.dark .pricing-teaser-features li {{ color: rgb(203, 213, 225); }}
  html.dark .news-preview-title {{ color: rgb(241, 245, 249); }}
  html.dark .news-preview-excerpt {{ color: rgb(203, 213, 225); }}
  html.dark .performer-symbol {{ color: rgb(241, 245, 249); }}
  html.dark .performer-price {{ color: rgb(203, 213, 225); }}

  * {{ box-sizing: border-box; }}
  html, body {{
    background: var(--bg);
    color: var(--text);
    font-family: var(--font-sans);
    -webkit-font-smoothing: antialiased;
    -moz-osx-font-smoothing: grayscale;
    overflow-x: hidden;
  }}
  body {{ min-height: 100vh; }}

  .epsx-skip-link {{
    position: fixed;
    top: 0.75rem;
    left: 0.75rem;
    z-index: 10000;
    padding: 0.75rem 1rem;
    border-radius: 0.5rem;
    background: #111827;
    color: #fff;
    font-weight: 700;
    text-decoration: none;
    transform: translateY(calc(-100% - 1.5rem));
    transition: transform 0.15s ease;
  }}
  .epsx-skip-link:focus, .epsx-skip-link:focus-visible {{
    transform: translateY(0);
    outline: 3px solid #f97316;
    outline-offset: 2px;
  }}
  #epsx-main-content {{ scroll-margin-top: 3.5rem; }}

  /* === Gradient text === */
  .gradient-text {{
    background: var(--gradient-warm);
    -webkit-background-clip: text;
    background-clip: text;
    -webkit-text-fill-color: transparent;
    color: transparent;
  }}
  .gradient-text-brand {{
    background: var(--gradient-brand);
    -webkit-background-clip: text;
    background-clip: text;
    -webkit-text-fill-color: transparent;
    color: transparent;
  }}
  .gradient-text-cool {{
    background: var(--gradient-cool);
    -webkit-background-clip: text;
    background-clip: text;
    -webkit-text-fill-color: transparent;
    color: transparent;
  }}
  .gradient-text-purple {{
    background: var(--gradient-purple);
    -webkit-background-clip: text;
    background-clip: text;
    -webkit-text-fill-color: transparent;
    color: transparent;
  }}

  /* Source pages use these pancake utility names for the brand gradient
     on section headings and short divider rules. */
  .pancake-gradient-text {{
    background: var(--pancake-gradient);
    -webkit-background-clip: text;
    background-clip: text;
    -webkit-text-fill-color: transparent;
    color: transparent;
  }}
  .pancake-gradient {{
    background: var(--pancake-gradient);
  }}

  /* === Wave 26 T1 — v3-style gradient utility overrides ===
   *
   * The Tailwind v2.2.19 CDN generates opacity-modified gradient
   * stops using the OLD v2 formula which produces colors that
   * don't match Tailwind v3+ PostCSS. The portfolio
   * `bg-gradient-to-r from-purple-900/40 via-purple-800/30 to-pink-900/40`
   * upsell banner (T2's wave-25 anon-state nudge) is the worst
   * offender — its v2-CDN render is dark muddy purple instead
   * of prod's bright royal-purple → hot-pink sunset, and the
   * 9pp portfolio regression comes from this single rule.
   *
   * Tailwind v3+ emits the 5 `--tw-gradient-*` custom props
   * (from/via/to/stops/position) for every color+opacity class.
   * The v2-CDN only emits the `--tw-gradient-from` color but
   * uses a different alpha-blend math (it composites onto white
   * instead of onto the gradient background).
   *
   * Fix: ship the v3-correct color values inline for the
   * high-frequency combinations so the v2-CDN render matches v3.
   * We target purple-900/40 + pink-900/40 (the portfolio upsell
   * banner) and 6 other widely-used color+opacity combinations.
   *
   * `!important` is required to win the cascade over the v2-CDN
   * `linear-gradient(...)` it composes on `.bg-gradient-to-r`
   * (v2-CDN uses a `style` attr, not a class rule, so the
   * cascade order is opposite of v3). */
  .from-purple-900\/40 {{
    --tw-gradient-from: rgb(88 28 135 / 0.4) var(--tw-gradient-from-position);
    --tw-gradient-to:   rgb(88 28 135 / 0)   var(--tw-gradient-to-position);
    --tw-gradient-stops: var(--tw-gradient-from), var(--tw-gradient-to);
  }}
  .via-purple-800\/30 {{
    --tw-gradient-via:  rgb(107 33 168 / 0.3) var(--tw-gradient-via-position);
    --tw-gradient-to:   rgb(107 33 168 / 0)   var(--tw-gradient-to-position);
    --tw-gradient-stops: var(--tw-gradient-from), var(--tw-gradient-via), var(--tw-gradient-to);
  }}
  .to-pink-900\/40 {{
    --tw-gradient-to:   rgb(157 23 77 / 0.4) var(--tw-gradient-to-position);
  }}
  .via-pink-900\/40 {{
    --tw-gradient-via:  rgb(157 23 77 / 0.4) var(--tw-gradient-via-position);
    --tw-gradient-to:   rgb(157 23 77 / 0)   var(--tw-gradient-to-position);
    --tw-gradient-stops: var(--tw-gradient-from), var(--tw-gradient-via), var(--tw-gradient-to);
  }}
  .from-blue-900\/20 {{
    --tw-gradient-from: rgb(30 58 138 / 0.2) var(--tw-gradient-from-position);
    --tw-gradient-to:   rgb(30 58 138 / 0)   var(--tw-gradient-to-position);
    --tw-gradient-stops: var(--tw-gradient-from), var(--tw-gradient-to);
  }}
  .from-blue-900\/80 {{
    --tw-gradient-from: rgb(30 58 138 / 0.8) var(--tw-gradient-from-position);
    --tw-gradient-to:   rgb(30 58 138 / 0)   var(--tw-gradient-to-position);
    --tw-gradient-stops: var(--tw-gradient-from), var(--tw-gradient-to);
  }}
  .from-indigo-900\/40 {{
    --tw-gradient-from: rgb(49 46 129 / 0.4) var(--tw-gradient-from-position);
    --tw-gradient-to:   rgb(49 46 129 / 0)   var(--tw-gradient-to-position);
    --tw-gradient-stops: var(--tw-gradient-from), var(--tw-gradient-to);
  }}
  .from-emerald-500\/20 {{
    --tw-gradient-from: rgb(16 185 129 / 0.2) var(--tw-gradient-from-position);
    --tw-gradient-to:   rgb(16 185 129 / 0)   var(--tw-gradient-to-position);
    --tw-gradient-stops: var(--tw-gradient-from), var(--tw-gradient-to);
  }}
  .via-orange-500\/40 {{
    --tw-gradient-via:  rgb(249 115 22 / 0.4) var(--tw-gradient-via-position);
    --tw-gradient-to:   rgb(249 115 22 / 0)   var(--tw-gradient-to-position);
    --tw-gradient-stops: var(--tw-gradient-from), var(--tw-gradient-via), var(--tw-gradient-to);
  }}

  /* === Wave 26 T1 — v3-style gradient utility overrides ===
   *
   * (Concrete `.glass` / `.pancake-gradient-text` utility
   * classes were removed in attempt 2 — the v3-color fix below
   * is the only T1 CSS-level visual change. The previous
   * `.glass` class shadowed the 6+ existing rule sets
   * (`.card-glass`, `.btn-glass`, ...) that share the
   * `--glass-bg` / `--glass-border` vars, so adding a 7th
   * `.glass` rule created cascade conflicts on the dev BFF.
   * The `.pancake-gradient-text` class is referenced by zero
   * pages; it was a no-op shipped to "complete" the subtask
   * 1.4 spec.)
   *
   * === Gradient orbs (decorative blur) === */
  .orb {{
    position: absolute;
    border-radius: 9999px;
    filter: blur(80px);
    opacity: 0.4;
    pointer-events: none;
    z-index: 0;
  }}
  .orb-orange {{ background: var(--epsx-orange); }}
  .orb-blue   {{ background: var(--epsx-blue-start); }}
  .orb-purple {{ background: var(--epsx-purple); }}
  .orb-yellow {{ background: var(--epsx-yellow); }}

  /* === Glassmorphism card === */
  .card-glass {{
    background: var(--glass-bg);
    backdrop-filter: blur(var(--glass-blur));
    -webkit-backdrop-filter: blur(var(--glass-blur));
    border: 1px solid var(--glass-border);
    box-shadow: var(--glass-shadow);
    border-radius: 1rem;
  }}
  .card-insight {{
    background: var(--surface);
    backdrop-filter: blur(12px);
    -webkit-backdrop-filter: blur(12px);
    border: 1px solid var(--epsx-border);
    box-shadow: var(--shadow);
    border-radius: 1rem;
    padding: 1.5rem;
    transition: all 0.3s ease;
  }}
  .card-insight h2, .card-insight h3, .card-insight h4 {{
    color: var(--text);
    margin-top: 2rem;
    margin-bottom: 0.75rem;
    line-height: 1.3;
  }}
  .card-insight h2 {{ font-size: 1.625rem; font-weight: 700; }}
  .card-insight h3 {{ font-size: 1.25rem; font-weight: 700; }}
  .card-insight h4 {{ font-size: 1.0625rem; font-weight: 700; }}
  .card-insight p, .card-insight li {{
    color: var(--text-muted);
  }}
  .card-insight a {{
    color: var(--accent);
  }}
  .card-insight code {{
    background: rgba(255,255,255,0.05);
    padding: 0.125rem 0.375rem;
    border-radius: 0.25rem;
    font-size: 0.875em;
    color: var(--text);
  }}
  .card-insight pre {{
    background: rgba(0,0,0,0.3);
    border: 1px solid var(--epsx-border);
    border-radius: 0.5rem;
    padding: 1rem;
    overflow-x: auto;
    font-size: 0.875rem;
    margin: 1rem 0;
  }}
  .card-insight ul, .card-insight ol {{
    margin: 0.75rem 0 1rem 1.5rem;
  }}
  .card-insight li {{ margin-bottom: 0.375rem; }}
  .card-insight hr {{
    border: 0;
    border-top: 1px solid var(--epsx-border);
    margin: 2rem 0;
  }}
  .card-insight blockquote {{
    border-left: 3px solid var(--accent);
    padding-left: 1rem;
    color: var(--text);
    font-style: italic;
    margin: 1rem 0;
  }}
  .card-insight:hover {{
    box-shadow: var(--shadow-xl);
    transform: translateY(-2px);
  }}
  .card-insight-gradient {{
    background: var(--gradient-card);
    backdrop-filter: blur(12px);
    -webkit-backdrop-filter: blur(12px);
    border: 1px solid var(--epsx-border);
    box-shadow: var(--shadow);
    border-radius: 1.5rem;
    padding: 1.5rem;
  }}

  /* === Buttons === */
  .btn {{
    display: inline-flex;
    align-items: center;
    justify-content: center;
    gap: 0.5rem;
    padding: 0.625rem 1.25rem;
    border-radius: 0.75rem;
    font-weight: 600;
    font-size: 0.875rem;
    line-height: 1.25rem;
    transition: all 0.2s ease;
    cursor: pointer;
    border: 1px solid transparent;
    text-decoration: none;
    white-space: nowrap;
  }}
  .btn:hover {{ transform: translateY(-1px); }}
  .btn:active {{ transform: translateY(0); }}
  .btn-primary {{
    background: var(--epsx-primary);
    color: white;
  }}
  .btn-primary:hover {{ background: var(--epsx-primary-hover); box-shadow: var(--shadow); }}
  .btn-gradient {{
    background: var(--gradient-warm);
    color: white;
    border: none;
  }}
  .btn-gradient:hover {{ box-shadow: var(--shadow-orange); }}
  .btn-brand {{
    background: var(--gradient-brand);
    color: white;
    border: none;
  }}
  .btn-brand:hover {{ box-shadow: 0 10px 25px -5px rgba(168, 85, 247, 0.4); }}
  .btn-cool {{
    background: var(--gradient-cool);
    color: white;
    border: none;
  }}
  .btn-outline {{
    background: transparent;
    color: var(--text) !important;
    border: 1px solid var(--epsx-border-strong);
  }}
  .btn-outline:hover {{ background: var(--bg-secondary); border-color: var(--text-muted); }}
  .btn-ghost {{
    background: transparent;
    color: var(--text);
    border: 1px solid transparent;
  }}
  .btn-ghost:hover {{ background: var(--bg-secondary); }}
  .btn-glass {{
    background: var(--glass-bg);
    backdrop-filter: blur(var(--glass-blur));
    -webkit-backdrop-filter: blur(var(--glass-blur));
    color: var(--text);
    border: 1px solid var(--glass-border);
  }}
  .btn-glass:hover {{ background: var(--surface-hover); }}
  .btn-danger {{
    background: var(--epsx-red);
    color: white;
  }}
  .btn-danger:hover {{ background: #dc2626; box-shadow: var(--shadow); }}
  .btn-sm {{ padding: 0.375rem 0.75rem; font-size: 0.8125rem; border-radius: 0.5rem; }}
  .btn-lg {{ padding: 0.875rem 2rem; font-size: 1rem; border-radius: 0.875rem; }}
  .btn-xl {{ padding: 1rem 2.5rem; font-size: 1.125rem; border-radius: 1rem; min-height: 3.5rem; }}
  .btn-block {{ width: 100%; }}

  /* === Badges === */
  .badge {{
    display: inline-flex;
    align-items: center;
    gap: 0.25rem;
    padding: 0.25rem 0.625rem;
    border-radius: 9999px;
    font-size: 0.75rem;
    font-weight: 600;
    line-height: 1;
  }}
  .badge-primary {{ background: rgba(37,99,235,0.12); color: #1d4ed8; }}
  .badge-success {{ background: rgba(5,150,105,0.12); color: #047857; }}
  .badge-warning {{ background: rgba(217,119,6,0.12); color: #92400e; }}
  .badge-danger  {{ background: rgba(220,38,38,0.12); color: #b91c1c; }}
  .badge-info    {{ background: rgba(2,132,199,0.12); color: #0369a1; }}
  .badge-purple  {{ background: rgba(147,51,234,0.12); color: #7e22ce; }}
  .badge-pink    {{ background: rgba(219,39,119,0.12); color: #be185d; }}
  .badge-pending {{ background: rgba(217,119,6,0.12); color: #92400e; }}
  .badge-active  {{ background: rgba(5,150,105,0.12); color: #047857; }}
  html.dark .badge-primary {{ background: rgba(59,130,246,0.15); color: #60a5fa; }}
  html.dark .badge-success {{ background: rgba(16,185,129,0.15); color: #34d399; }}
  html.dark .badge-warning,
  html.dark .badge-pending {{ background: rgba(245,158,11,0.15); color: #fbbf24; }}
  html.dark .badge-danger {{ background: rgba(239,68,68,0.15); color: #f87171; }}
  html.dark .badge-info {{ background: rgba(6,182,212,0.15); color: #22d3ee; }}
  html.dark .badge-purple {{ background: rgba(168,85,247,0.15); color: #c084fc; }}
  html.dark .badge-pink {{ background: rgba(236,72,153,0.15); color: #f472b6; }}
  .badge-glass   {{
    background: var(--glass-bg);
    backdrop-filter: blur(8px);
    border: 1px solid var(--glass-border);
    color: var(--text);
  }}
  .badge-pill {{
    display: inline-flex;
    align-items: center;
    gap: 0.375rem;
    padding: 0.375rem 0.875rem;
    border-radius: 9999px;
    font-size: 0.8125rem;
    font-weight: 500;
    background: var(--surface);
    border: 1px solid var(--epsx-border);
    color: var(--text-muted);
    backdrop-filter: blur(8px);
  }}

  /* === Form === */
  .input {{
    width: 100%;
    padding: 0.625rem 1rem;
    border-radius: 0.625rem;
    background: var(--bg-secondary);
    border: 1px solid var(--epsx-border);
    color: var(--text);
    font-family: inherit;
    font-size: 0.875rem;
    transition: all 0.2s ease;
  }}
  .input:focus {{
    outline: none;
    border-color: var(--epsx-primary);
    box-shadow: 0 0 0 3px rgba(59,130,246,0.15);
  }}
  .btn:focus-visible, .input:focus-visible {{
    outline: 3px solid var(--focus-ring);
    outline-offset: 2px;
  }}
  .btn:disabled {{
    opacity: 0.5;
    pointer-events: none;
  }}
  .input:disabled {{
    opacity: 0.5;
    cursor: not-allowed;
  }}
  .label {{
    display: block;
    font-size: 0.8125rem;
    font-weight: 500;
    color: var(--text-muted);
    margin-bottom: 0.375rem;
  }}
  .input-icon-wrap {{
    position: relative;
  }}
  .input-icon-wrap .input {{ padding-left: 2.5rem; }}
  .input-icon-wrap .icon {{
    position: absolute;
    left: 0.875rem;
    top: 50%;
    transform: translateY(-50%);
    color: var(--text-subtle);
    pointer-events: none;
  }}

  /* === Toast === */
  .toast-host {{
    position: fixed;
    bottom: 1.25rem;
    right: 1.25rem;
    z-index: 99999;
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
    max-width: 24rem;
  }}
  .toast {{
    background: var(--surface-solid);
    border: 1px solid var(--epsx-border);
    border-radius: 0.75rem;
    box-shadow: var(--shadow-lg);
    padding: 0.875rem 1rem;
    color: var(--text);
    display: flex;
    gap: 0.625rem;
    align-items: flex-start;
    animation: slideInRight 0.25s ease-out;
  }}
  .toast-success {{ border-left: 3px solid var(--epsx-green); }}
  .toast-error   {{ border-left: 3px solid var(--epsx-red); }}
  .toast-info    {{ border-left: 3px solid var(--epsx-primary); }}
  .toast-warning {{ border-left: 3px solid var(--epsx-amber); }}

  /* === Modal === */
  .modal-backdrop {{
    position: fixed;
    inset: 0;
    background: rgba(0,0,0,0.6);
    backdrop-filter: blur(4px);
    z-index: 99998;
    display: flex;
    align-items: center;
    justify-content: center;
    padding: 1rem;
    animation: fadeIn 0.2s ease;
  }}
  .modal {{
    background: var(--surface-solid);
    border: 1px solid var(--epsx-border);
    border-radius: 1rem;
    box-shadow: var(--shadow-2xl);
    max-width: 32rem;
    width: 100%;
    max-height: 90vh;
    overflow: auto;
    animation: scaleIn 0.2s ease;
  }}

  /* === EPSX wallet select modal (matches epsx.io's `auth-modal`) === */
  .epsx-modal {{
    background: var(--surface-solid);
    border: 1px solid var(--epsx-border);
    border-radius: 1.5rem;
    box-shadow: 0 0 50px -12px rgba(0,0,0,0.5);
    max-width: 28rem;
    width: 100%;
    overflow: hidden;
    animation: scaleIn 0.2s ease;
  }}
  /* Match epsx.io's exact modal: #191923 dark bg regardless of light/dark */
  .auth-modal-inner {{
    background: #191923;
    color: #ffffff;
    border-radius: 1.5rem;
    box-shadow: 0 0 50px -12px rgba(0,0,0,0.5);
    overflow: hidden;
    isolation: isolate;
  }}
  @media (min-width: 640px) {{ .auth-modal-inner {{ border-radius: 1.5rem; }} }}
  .auth-modal-content {{ padding: 1.5rem 2rem; }}
  .auth-step {{ display: block; }}
  .auth-step-header {{
    display: flex; align-items: center; gap: 0.625rem;
    margin-bottom: 1.25rem;
  }}
  .auth-step-number {{
    width: 1.75rem; height: 1.75rem;
    display: inline-flex; align-items: center; justify-content: center;
    background: rgba(139,92,246,0.2); color: #8b5cf6;
    font-size: 0.875rem; font-weight: 600;
    border-radius: 9999px;
    flex-shrink: 0;
  }}
  .auth-step-label {{ font-size: 1rem; font-weight: 600; color: #ffffff; }}
  .auth-wallets {{ display: flex; flex-direction: column; gap: 0.625rem; }}
  .auth-wallet-btn {{
    width: 100%;
    display: flex; align-items: center; gap: 0.875rem;
    padding: 1rem 1.25rem;
    background: rgba(255,255,255,0.05);
    border: 1px solid transparent;
    border-radius: 1rem;
    color: #ffffff;
    font-size: 1rem; font-weight: 500;
    cursor: pointer;
    transition: all 0.2s ease;
    text-align: left;
  }}
  .auth-wallet-btn:hover {{
    background: rgba(255,255,255,0.10);
    border-color: rgba(139,92,246,0.4);
    transform: translateY(-1px);
  }}
  .auth-wallet-btn:disabled {{
    cursor: not-allowed;
    opacity: 0.45;
    transform: none;
  }}
  .auth-wallet-btn:disabled:hover {{
    background: rgba(255,255,255,0.05);
    border-color: transparent;
    transform: none;
  }}
  .auth-wallet-icon {{
    font-size: 1.5rem;
    width: 1.5rem; height: 1.5rem;
    display: inline-flex; align-items: center; justify-content: center;
    flex-shrink: 0;
  }}
  .auth-wallet-name {{ font-size: 1rem; font-weight: 500; color: #ffffff; }}

  /* Admin wallet gate — the production desktop composition is a 60/40
   * split (marketing copy on the left, wallet card on the right). The
   * Tailwind utility subset used by the local SSR build does not emit all
   * responsive `lg:*` rules, so keep this route's structural breakpoints
   * explicit. Mobile/tablet retain the focused wallet-selection layout. */
  .wave25-t3-auth-overlay {{ flex-direction: column; }}
  .wave25-t3-auth-left {{ display: none !important; }}
  .wave25-t3-auth-right {{ width: 100% !important; max-width: 56rem !important; }}
  .wave25-t3-desktop-card-heading {{ display: none !important; }}
  .wave25-t3-auth-card-inner {{ max-width: 26.25rem; }}
  /* The reference tablet layout keeps the focused card compact: the
   * generated utility CSS otherwise applies `sm:` sizing at 768px and
   * makes the card, lock mark, and wallet buttons noticeably too large. */
  @media (min-width: 640px) and (max-width: 1023px) {{
    .wave25-t3-auth-right {{ padding: 3rem 1rem !important; }}
    .wave25-t3-auth-heading {{ margin-bottom: 2rem !important; }}
    .wave25-t3-auth-heading .wave25-t3-auth-lock {{ width: 3rem !important; height: 3rem !important; }}
    .wave25-t3-auth-heading .wave25-t3-auth-lock + span {{ font-size: 1.5rem !important; }}
    .wave25-t3-auth-heading h1 {{ font-size: 1.875rem !important; }}
    .wave25-t3-auth-heading p {{ font-size: 1rem !important; }}
    .wave25-t3-auth-card-inner {{ max-width: 28rem !important; margin-left: auto; margin-right: auto; padding: 2rem !important; }}
    .wave25-t3-auth-card-inner .auth-wallet-btn {{ min-height: 4.375rem !important; }}
  }}
  @media (min-width: 1024px) {{
    .wave25-t3-auth-overlay {{
      flex-direction: row;
      align-items: stretch !important;
      justify-content: stretch !important;
    }}
    .wave25-t3-auth-left {{
      display: flex !important;
      width: 60% !important;
      padding: 5rem 6rem !important;
    }}
    .wave25-t3-auth-left .group > div:first-child {{
      border-color: rgba(148,163,184,0.08) !important;
      background: rgba(255,255,255,0.05) !important;
    }}
    .wave25-t3-auth-right {{
      width: 40% !important;
      max-width: none !important;
      padding: 1.5rem !important;
      border-left: 1px solid rgba(148,163,184,0.20);
      background: rgba(255,255,255,0.02);
      backdrop-filter: blur(24px);
    }}
    .wave25-t3-auth-heading {{ display: none !important; }}
    .wave25-t3-auth-card {{ width: 100% !important; }}
    .wave25-t3-auth-card-inner {{
      width: 100% !important;
      max-width: 28rem !important;
      margin-left: auto;
      margin-right: auto;
      background: rgb(20,25,36) !important;
    }}
    .wave25-t3-auth-card-inner .auth-step-header {{ margin-bottom: 1rem !important; }}
    .wave25-t3-auth-card-inner .auth-wallets {{ gap: 0.75rem !important; }}
    .wave25-t3-auth-card-inner .auth-wallet-btn {{ min-height: 70px !important; }}
    .wave25-t3-auth-card-inner .auth-wallet-icon {{ height: 2.25rem !important; }}
    .wave25-t3-desktop-card-heading {{ display: block !important; }}
  }}

  .auth-modal-footer {{
    padding: 1rem 2rem 1.5rem;
    background: #191923;
    text-align: center;
  }}
  .auth-footer-text {{ font-size: 0.75rem; color: rgba(255,255,255,0.4); margin: 0; }}

  html.dark .epsx-modal {{ background: rgba(15,23,42,0.95); border-color: rgba(51,65,85,0.5); }}
  .epsx-modal-header {{
    padding: 1.25rem 1.5rem 0.5rem;
    display: flex; align-items: center; gap: 0.75rem;
  }}
  .epsx-modal-step {{
    width: 1.75rem; height: 1.75rem;
    display: inline-flex; align-items: center; justify-content: center;
    background: rgba(139,92,246,0.2); color: #8b5cf6;
    font-size: 0.875rem; font-weight: 600;
    border-radius: 9999px;
  }}
  .epsx-modal-title {{
    font-size: 1.0625rem; font-weight: 600;
    color: var(--text);
  }}
  html.dark .epsx-modal-title {{ color: white; }}
  .epsx-modal-body {{ padding: 1rem 1.5rem; min-height: 12rem; }}
  .epsx-wallet-list {{ display: flex; flex-direction: column; gap: 0.75rem; }}
  .epsx-wallet-btn {{
    width: 100%;
    display: flex; align-items: center; gap: 0.875rem;
    padding: 1rem 1.25rem;
    background: var(--bg-secondary);
    border: 1px solid var(--epsx-border);
    border-radius: 1rem;
    color: var(--text);
    font-size: 1rem; font-weight: 500;
    cursor: pointer;
    transition: all 0.2s ease;
  }}
  .epsx-wallet-btn:hover {{
    border-color: rgba(139,92,246,0.5);
    transform: translateY(-2px);
  }}
  html.dark .epsx-wallet-btn {{ background: rgba(30,41,59,0.5); border-color: rgba(51,65,85,0.5); color: white; }}
  .epsx-wallet-btn .wallet-icon {{
    width: 1.5rem; height: 1.5rem;
    display: inline-flex; align-items: center; justify-content: center;
    flex-shrink: 0;
  }}
  .epsx-modal-footer {{
    padding: 1rem 1.5rem;
    border-top: 1px solid var(--epsx-border);
    text-align: center;
    font-size: 0.8125rem;
    color: var(--text-subtle);
  }}
  html.dark .epsx-modal-footer {{ border-top-color: rgba(51,65,85,0.5); color: #94a3b8; }}

  /* === Dropdown === */
  .dropdown-menu {{
    position: absolute;
    background: var(--surface-solid);
    border: 1px solid var(--epsx-border);
    border-radius: 0.75rem;
    box-shadow: var(--shadow-xl);
    padding: 0.375rem;
    min-width: 12rem;
    z-index: 9999;
    animation: scaleIn 0.15s ease;
  }}
  .dropdown-item {{
    display: flex;
    align-items: center;
    gap: 0.5rem;
    padding: 0.5rem 0.625rem;
    border-radius: 0.5rem;
    font-size: 0.875rem;
    color: var(--text-muted);
    cursor: pointer;
    transition: all 0.15s ease;
    text-decoration: none;
  }}
  .dropdown-item:hover {{ background: var(--bg-secondary); color: var(--text); }}
  .dropdown-item.active {{ background: var(--bg-secondary); color: var(--text); }}

  /* === Tabs === */
  .tabs-nav {{
    display: flex;
    gap: 0.25rem;
    border-bottom: 1px solid var(--epsx-border);
    margin-bottom: 1rem;
  }}
  .tab {{
    padding: 0.625rem 1rem;
    font-size: 0.875rem;
    font-weight: 500;
    color: var(--text-muted);
    border-bottom: 2px solid transparent;
    cursor: pointer;
    transition: all 0.15s ease;
  }}
  .tab:hover {{ color: var(--text); }}
  .tab.active {{ color: var(--epsx-orange); border-bottom-color: var(--epsx-orange); }}

  /* === Skeleton === */
  .skeleton {{
    background: linear-gradient(90deg, var(--bg-secondary) 25%, var(--bg-tertiary) 50%, var(--bg-secondary) 75%);
    background-size: 200% 100%;
    animation: shimmer 1.5s infinite;
    border-radius: 0.5rem;
  }}
  @keyframes shimmer {{
    0% {{ background-position: 200% 0; }}
    100% {{ background-position: -200% 0; }}
  }}

  /* === Page background === */
  .page-bg {{
    background: var(--gradient-page);
    min-height: 100vh;
  }}

  /* === App-page background (Wave 49 T2 / Plan 13) ===
   * Prod's app pages (/account, /account/credits, /analytics, /chat*,
   * /notifications, /permissions, /profile) render with a subtle
   * purple/magenta radial-glow body gradient instead of the warm-
   * neutral `--gradient-page` used by marketing pages (/, /plans,
   * /privacy, /terms, etc.). Sampled prod corners:
   *   /account         #13182b → #401c68 → #412148 (top-left → center → bot-right)
   *   /analytics       #13072b → #0f172b → #050617 (deep purple-navy)
   *   /permissions     #201511 → #151318 → #151217 (warm-brown → dark gray)
   * The dominant signal is a purple radial glow centered ~50%/50%
   * with low alpha, sitting on a deep navy base. Implemented as a
   * 2-layer background: a deep-purple linear gradient base + a soft
   * purple radial overlay matching prod's center hot-spot. */
  .page-bg-app {{
    background:
      radial-gradient(ellipse 80% 60% at 50% 40%, rgba(139, 92, 246, 0.18) 0%, rgba(168, 85, 247, 0.08) 35%, transparent 70%),
      linear-gradient(135deg, #0a0e1a 0%, #13182b 30%, #1a1138 60%, #2a1748 100%);
    min-height: 100vh;
  }}
  /* Light-mode equivalent — sampled prod light-mode /account uses a
   * subtle blue/violet pastel gradient. The class is rarely used in
   * light mode (most captures are dark) but defined for parity. */
  html:not(.dark) .page-bg-app {{
    background: linear-gradient(135deg, #f5f3ff 0%, #ede9fe 50%, #fce7f3 100%);
  }}
  .account-prod-page {{
    background: linear-gradient(135deg, #111827 0%, #581c87 48%, #111827 100%);
  }}
  html:not(.dark) .account-prod-page {{
    background: linear-gradient(135deg, #fffbea 0%, #fff7ed 48%, #fdf2f8 100%);
  }}
  html:not(.dark) .account-prod-page .text-slate-300 {{
    color: #475569;
  }}

  /* === Section === */
  .section {{ padding: 4rem 1.5rem; }}
  .section-tight {{ padding: 2.5rem 1.5rem; }}
  @media (min-width: 640px) {{ .section {{ padding: 5rem 2rem; }} }}
  @media (min-width: 1024px) {{ .section {{ padding: 6rem 2rem; }} }}

  /* === Container ===
   * Tailwind 2.2.19's .container has width:100% per breakpoint but does
   * NOT center itself (no margin: 0 auto). Wave 22 home/auth/contact/about
   * pages use the bare `.container` class — the Dioxus page source emits
   * it directly. Add the centering + width caps here so they match the
   * 80rem max-width used by `.container-x`.
   * See: https://v2.tailwindcss.com/docs/container
   */
  .container {{ margin-left: auto; margin-right: auto; padding-left: 1rem; padding-right: 1rem; }}
  @media (min-width: 640px)  {{ .container {{ max-width: 640px;  padding-left: 1.5rem; padding-right: 1.5rem; }} }}
  @media (min-width: 768px)  {{ .container {{ max-width: 768px;  padding-left: 1.5rem; padding-right: 1.5rem; }} }}
  @media (min-width: 1024px) {{ .container {{ max-width: 1024px; padding-left: 2rem;   padding-right: 2rem;   }} }}
  @media (min-width: 1280px) {{ .container {{ max-width: 1280px; padding-left: 2rem;   padding-right: 2rem;   }} }}
  @media (min-width: 1536px) {{ .container {{ max-width: 80rem;  padding-left: 2rem;   padding-right: 2rem;   }} }}
  .container-x {{ max-width: 80rem; margin-left: auto; margin-right: auto; padding-left: 1rem; padding-right: 1rem; }}
  @media (min-width: 640px) {{ .container-x {{ padding-left: 1.5rem; padding-right: 1.5rem; }} }}
  @media (min-width: 1024px) {{ .container-x {{ padding-left: 2rem; padding-right: 2rem; }} }}

  /* === Animations === */
  @keyframes gradient-x {{
    0%, 100% {{ background-position: 0% 50%; }}
    50%      {{ background-position: 100% 50%; }}
  }}
  .animate-gradient-x {{
    background-size: 200% 200%;
    animation: gradient-x 3s ease infinite;
  }}
  @keyframes fadeIn {{
    from {{ opacity: 0; }}
    to   {{ opacity: 1; }}
  }}
  .animate-fade-in {{ animation: fadeIn 0.5s ease-out; }}
  @keyframes slideInRight {{
    from {{ opacity: 0; transform: translateX(20px); }}
    to   {{ opacity: 1; transform: translateX(0); }}
  }}
  @keyframes scaleIn {{
    from {{ opacity: 0; transform: scale(0.95); }}
    to   {{ opacity: 1; transform: scale(1); }}
  }}
  .animate-scale-in {{ animation: scaleIn 0.3s ease-out; }}
  @keyframes slideUp {{
    from {{ opacity: 0; transform: translateY(20px); }}
    to   {{ opacity: 1; transform: translateY(0); }}
  }}
  .animate-slide-up {{ animation: slideUp 0.6s ease-out; }}
  .animate-slide-up-delayed {{ animation: slideUp 0.8s ease-out 0.15s both; }}
  .animate-slide-up-delayed-2 {{ animation: slideUp 0.8s ease-out 0.3s both; }}
  @keyframes zoomIn {{
    from {{ opacity: 0; transform: scale(0.95); }}
    to   {{ opacity: 1; transform: scale(1); }}
  }}
  .animate-zoom-in {{ animation: zoomIn 0.3s ease-out; }}
  @keyframes zoomOut {{
    from {{ opacity: 1; transform: scale(1); }}
    to   {{ opacity: 0; transform: scale(0.95); }}
  }}
  .animate-zoom-out {{ animation: zoomOut 0.3s ease-out; }}
  @keyframes slideIn {{
    from {{ opacity: 0; transform: translateY(-8px); }}
    to   {{ opacity: 1; transform: translateY(0); }}
  }}
  .animate-slide-in {{ animation: slideIn 0.3s ease-out; }}
  @keyframes bounceIn {{
    0%   {{ transform: scale(0); }}
    50%  {{ transform: scale(1.2); }}
    100% {{ transform: scale(1); }}
  }}
  .animate-bounce-in {{ animation: bounceIn 0.5s ease-out; }}
  @keyframes pulseGlow {{
    0%, 100% {{ box-shadow: 0 0 0 0 rgba(249,115,22,0.5); }}
    50%      {{ box-shadow: 0 0 0 12px rgba(249,115,22,0); }}
  }}
  .animate-pulse-glow {{ animation: pulseGlow 2s infinite; }}
  .epsx-loader {{
    width: 2rem; height: 2rem;
    border: 3px solid rgba(139,92,246,0.2);
    border-top-color: #8b5cf6;
    border-radius: 9999px;
    animation: spin 0.8s linear infinite;
  }}
  @keyframes bounceGentle {{
    0%, 100% {{ transform: translateY(0); }}
    50%      {{ transform: translateY(-4px); }}
  }}
  .animate-bounce-gentle:hover {{ animation: bounceGentle 0.6s ease infinite; }}
  .hover-scale {{ transition: transform 0.2s ease; }}
  .hover-scale:hover {{ transform: scale(1.05); }}

  /* === Navbar === */
  .navbar {{
    position: sticky;
    top: 0;
    z-index: 50;
    background: var(--surface);
    backdrop-filter: blur(16px);
    -webkit-backdrop-filter: blur(16px);
    border-bottom: 1px solid var(--epsx-border);
  }}
  .nav-link {{
    display: inline-flex;
    align-items: center;
    gap: 0.375rem;
    padding: 0.375rem 0.75rem;
    border-radius: 0.5rem;
    font-size: 0.875rem;
    font-weight: 500;
    color: var(--text-muted);
    transition: all 0.15s ease;
    text-decoration: none;
    cursor: pointer;
    background: transparent;
    border: none;
  }}
  .nav-link:hover {{ color: var(--text); background: var(--bg-secondary); }}
  .nav-link.active {{ color: var(--text); }}

  /* === Desktop nav dropdown === */
  .nav-dropdown-wrap {{ position: relative; }}
  .nav-dropdown-trigger {{
    display: inline-flex;
    align-items: center;
    gap: 0.375rem;
    padding: 0.375rem 0.75rem;
    border-radius: 0.5rem;
    font-size: 0.875rem;
    font-weight: 500;
    color: var(--text-muted);
    background: transparent;
    border: none;
    cursor: pointer;
    transition: all 0.15s ease;
    text-decoration: none;
  }}
  .nav-dropdown-trigger:hover {{ color: var(--text); background: var(--bg-secondary); }}
  .nav-dropdown-trigger.active {{ color: var(--text); }}
  .nav-dropdown-trigger > i {{ color: var(--epsx-orange); font-size: 0.9rem; }}
  .nav-dropdown-trigger .chev {{ font-size: 0.7rem; transition: transform 0.2s ease; }}
  .nav-dropdown-wrap.open .nav-dropdown-trigger .chev {{ transform: rotate(180deg); }}

  .nav-dropdown {{
    position: absolute;
    top: calc(100% + 0.5rem);
    left: 0;
    min-width: 14rem;
    background: var(--surface-solid);
    border: 1px solid var(--epsx-border);
    border-radius: 0.75rem;
    box-shadow: var(--shadow-xl);
    padding: 0.375rem;
    z-index: 9999;
    display: none;
    animation: scaleIn 0.15s ease;
  }}
  .nav-dropdown-wrap.open .nav-dropdown {{ display: block; }}
  .nav-dropdown-item {{
    display: flex;
    align-items: flex-start;
    gap: 0.625rem;
    padding: 0.625rem 0.75rem;
    border-radius: 0.5rem;
    font-size: 0.875rem;
    color: var(--text-muted);
    transition: all 0.15s ease;
    text-decoration: none;
  }}
  .nav-dropdown-item:hover {{ background: var(--bg-secondary); color: var(--text); }}
  .nav-dropdown-item.active {{ background: var(--bg-secondary); color: var(--text); }}
  .nav-dropdown-item .item-icon {{
    flex-shrink: 0;
    color: var(--epsx-orange);
    font-size: 0.95rem;
    margin-top: 0.1rem;
  }}
  .nav-dropdown-item .item-label {{ font-weight: 500; line-height: 1.2; color: var(--text); }}
  .nav-dropdown-item .item-desc {{
    font-size: 0.75rem;
    color: var(--text-subtle);
    margin-top: 0.125rem;
    line-height: 1.3;
  }}

  /* === Mobile nav accordion === */
  .nav-accordion {{ display: flex; flex-direction: column; }}
  .nav-accordion-trigger {{
    width: 100%;
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 0.625rem 0.75rem;
    border-radius: 0.5rem;
    font-size: 0.875rem;
    font-weight: 500;
    color: var(--text-muted);
    background: transparent;
    border: none;
    cursor: pointer;
    transition: all 0.15s ease;
    text-align: left;
  }}
  .nav-accordion-trigger:hover {{ color: var(--text); background: var(--bg-secondary); }}
  .nav-accordion-trigger.active {{ color: var(--text); }}
  .nav-accordion-trigger .trigger-label {{ display: flex; align-items: center; gap: 0.5rem; }}
  .nav-accordion-trigger .trigger-label > i {{ color: var(--epsx-orange); font-size: 0.9rem; }}
  .nav-accordion-trigger .chev {{
    color: var(--epsx-orange);
    font-size: 0.75rem;
    transition: transform 0.2s ease;
  }}
  .nav-accordion.open .nav-accordion-trigger .chev {{ transform: rotate(90deg); }}

  .nav-accordion-content {{
    display: none;
    margin-left: 0.75rem;
    padding-left: 0.75rem;
    border-left: 1px solid var(--epsx-border);
    margin-top: 0.25rem;
    margin-bottom: 0.5rem;
    flex-direction: column;
    gap: 0.125rem;
  }}
  .nav-accordion.open .nav-accordion-content {{ display: flex; }}
  .nav-accordion-content a {{
    display: flex;
    align-items: center;
    gap: 0.625rem;
    padding: 0.5rem 0.75rem;
    border-radius: 0.5rem;
    font-size: 0.875rem;
    color: var(--text-muted);
    text-decoration: none;
    transition: all 0.15s ease;
  }}
  .nav-accordion-content a:hover {{ background: var(--bg-secondary); color: var(--text); }}
  .nav-accordion-content a.active {{ background: var(--bg-secondary); color: var(--text); }}
  .nav-accordion-content a > i {{ color: var(--epsx-orange); font-size: 0.9rem; flex-shrink: 0; }}

  /* === Footer === */
  .footer {{
    background: var(--bg-secondary);
    border-top: 1px solid var(--epsx-border);
    padding: 3rem 1.5rem 1.5rem;
    color: var(--text-muted);
  }}
  .footer-link {{
    color: var(--text-muted);
    text-decoration: none;
    font-size: 0.875rem;
    transition: color 0.15s ease;
  }}
  .footer-link:hover {{ color: var(--text); }}

  /* === Logo === */
  .logo-text {{
    font-size: 1.25rem;
    font-weight: 900;
    letter-spacing: 0.1em;
    background: var(--gradient-brand);
    -webkit-background-clip: text;
    background-clip: text;
    -webkit-text-fill-color: transparent;
  }}
  .logo-text-sm {{
    font-size: 1rem;
    font-weight: 900;
    letter-spacing: 0.1em;
    background: var(--gradient-brand);
    -webkit-background-clip: text;
    background-clip: text;
    -webkit-text-fill-color: transparent;
  }}

  /* === EPSX icon (hexagon w/ chart) === */
  .epsx-icon {{
    width: 2rem; height: 2rem;
    display: inline-flex; align-items: center; justify-content: center;
    flex-shrink: 0;
  }}

  /* === Sticky header (matches epsx.io `sticky top-0 z-50 border-b border-slate-200/60 bg-white/95 backdrop-blur-md dark:border-slate-800 dark:bg-slate-950/95`) === */
  .epsx-header {{
    position: sticky; top: 0; z-index: 50;
    border-bottom: 1px solid rgba(226, 232, 240, 0.6);
    background: rgba(255, 255, 255, 0.95);
    backdrop-filter: blur(12px); -webkit-backdrop-filter: blur(12px);
  }}
  html.dark .epsx-header {{
    border-bottom-color: rgba(30, 41, 59, 1);
    background: rgba(2, 6, 23, 0.95);
  }}

  /* === Nav trigger (epsx.io: rounded-md, 8.5px icon, slate colors) === */
  .epsx-nav-trigger {{
    display: flex; align-items: center; gap: 0.25rem;
    height: 2rem;
    padding: 0.375rem 0.75rem;
    font-size: 0.875rem; font-weight: 500;
    color: var(--text-muted);
    border-radius: 0.375rem;
    background: transparent; border: none; cursor: pointer;
    transition: color 0.15s ease;
  }}
  html:not(.dark) .epsx-nav-trigger {{ color: #475569; }}
  .epsx-nav-trigger:hover {{ color: var(--text); }}
  .epsx-nav-trigger.active {{ color: var(--text); }}
  html.dark .epsx-nav-trigger {{ color: #94a3b8; }}
  html.dark .epsx-nav-trigger:hover,
  html.dark .epsx-nav-trigger.active {{ color: white; }}
  .epsx-nav-trigger .nav-icon {{ color: var(--epsx-orange); width: 1rem; height: 1rem; flex-shrink: 0; }}
  .epsx-nav-trigger .nav-chev {{ color: var(--epsx-orange); width: 0.75rem; height: 0.75rem; transition: transform 0.2s ease; }}
  /* === Position anchor for the absolute-positioned `.epsx-nav-menu` ===
     Without `position: relative` here, the dropdown menu's
     `position: absolute; top: calc(100% + 0.5rem); left: 0;` would
     anchor to the nearest positioned ancestor — which is the sticky
     `<header class="epsx-header">` (sticky elements ARE a positioning
     context). The result was the menu rendering at top-left of the
     page (100% of the 3.5rem sticky header height) instead of
     directly under the trigger button. Adding `position: relative`
     to `.epsx-nav-wrap` makes the wrap the containing block so
     `top: 100%` = the wrap's full height (the trigger's bottom
     edge + 0.5rem) and `left: 0` = the trigger's left edge —
     matching prod's Radix `align: "start"; sideOffset: 8` exactly. */
  .epsx-nav-wrap {{ position: relative; display: inline-block; }}
  .epsx-nav-wrap.open .epsx-nav-trigger .nav-chev {{ transform: rotate(180deg); }}

  /* === Nav dropdown content (epsx.io Radix menu) === */
  .epsx-nav-menu {{
    position: absolute; top: calc(100% + 0.5rem); left: 0;
    min-width: 13rem;
    background: var(--surface-solid);
    border: 1px solid var(--epsx-border);
    border-radius: 0.5rem;
    box-shadow: var(--shadow-xl);
    padding: 0.375rem;
    z-index: 9999;
    display: none;
    animation: scaleIn 0.15s ease;
  }}
  .epsx-nav-wrap.open .epsx-nav-menu,
  .epsx-nav-menu.open {{ display: block; }}
  html.dark .epsx-nav-menu {{ background: #0f172a; border-color: #334155; }}

  .epsx-nav-item {{
    display: flex; align-items: flex-start; gap: 0.625rem;
    padding: 0.5rem 0.625rem;
    border-radius: 0.375rem;
    font-size: 0.875rem;
    color: var(--text-muted);
    text-decoration: none;
    transition: background 0.15s ease, color 0.15s ease;
  }}
  .epsx-nav-item:hover {{ background: var(--bg-secondary); color: var(--text); }}
  html.dark .epsx-nav-item {{ color: #cbd5e1; }}
  html.dark .epsx-nav-item:hover {{ background: #1e293b; color: white; }}
  .epsx-nav-item .item-icon {{ color: var(--epsx-orange); width: 1rem; height: 1rem; flex-shrink: 0; margin-top: 0.125rem; }}
  .epsx-nav-item .item-label {{ font-weight: 500; line-height: 1.2; color: var(--text); }}
  .epsx-nav-item .item-desc {{ font-size: 0.75rem; color: var(--text-subtle); margin-top: 0.125rem; line-height: 1.3; }}
  html.dark .epsx-nav-item .item-label {{ color: white; }}
  html.dark .epsx-nav-item .item-desc {{ color: #94a3b8; }}

  /* === Connect button (orange gradient pill, like epsx.io) === */
  .epsx-connect-btn {{
    display: inline-flex; align-items: center; gap: 0.5rem;
    height: 2.5rem; padding: 0 1rem;
    border-radius: 1rem; font-size: 0.875rem; font-weight: 500;
    color: white;
    background: linear-gradient(135deg, #fb923c 0%, #ea580c 100%);
    border: 0; cursor: pointer;
    box-shadow: 0 10px 15px -3px rgba(249,115,22,0.25);
    transition: all 0.2s ease;
  }}
  .epsx-connect-btn:hover {{
    background: linear-gradient(135deg, #f97316 0%, #c2410c 100%);
    box-shadow: 0 20px 25px -5px rgba(249,115,22,0.4);
  }}

  /* Connected-but-not-signed-in wallet identity. */
  .epsx-wallet-pill {{
    display: inline-flex; align-items: center; gap: 0.5rem;
    height: 2.5rem; padding: 0 0.875rem; border-radius: 1rem;
    color: var(--text); background: var(--bg-secondary);
    border: 1px solid var(--epsx-border); text-decoration: none;
    font-size: 0.8125rem; font-weight: 600;
  }}
  .epsx-wallet-pill:hover {{ color: var(--text); border-color: var(--epsx-orange); }}
  .epsx-wallet-pill svg, .epsx-wallet-pill i {{ color: var(--epsx-orange); }}
  html.dark .epsx-wallet-pill {{ color: #e2e8f0; background: #1e293b; border-color: #334155; }}

  /* Authenticated wallet identity. Sign-out lives inside this disclosure so
     the public header keeps the same compact wallet shape as production. */
  .epsx-session-menu-wrap {{ position: relative; }}
  .epsx-session-trigger {{ cursor: pointer; white-space: nowrap; }}
  .epsx-session-chevron {{ width: 0.75rem; height: 0.75rem; transition: transform 0.15s ease; }}
  .epsx-session-trigger[aria-expanded="true"] .epsx-session-chevron {{ transform: rotate(180deg); }}
  .epsx-session-menu {{
    position: absolute; top: calc(100% + 0.625rem); right: 0; z-index: 9999;
    width: min(18rem, calc(100vw - 2rem)); overflow: hidden;
    border: 1px solid var(--epsx-border); border-radius: 1rem;
    background: var(--surface-solid); color: var(--text);
    box-shadow: var(--shadow-xl); transform-origin: top right;
  }}
  .epsx-session-menu[hidden] {{ display: none !important; }}
  .epsx-session-menu.open {{ animation: scaleIn 0.15s ease; }}
  html.dark .epsx-session-menu {{ background: #0f172a; border-color: #334155; }}
  .epsx-session-summary {{ padding: 0.75rem; border-bottom: 1px solid var(--epsx-border); }}
  .epsx-session-label {{
    display: flex; align-items: center; gap: 0.375rem; margin-bottom: 0.25rem;
    color: var(--epsx-orange); font-size: 0.625rem; font-weight: 700;
    letter-spacing: 0.08em; text-transform: uppercase;
  }}
  .epsx-session-address {{
    display: block; overflow-wrap: anywhere; color: var(--text);
    font-family: var(--font-mono); font-size: 0.75rem; line-height: 1.4;
  }}
  .epsx-session-actions {{ padding: 0.25rem; }}
  .epsx-session-menu-item {{
    display: flex; width: 100%; align-items: center; gap: 0.625rem;
    min-height: 2.5rem; padding: 0.625rem 0.75rem; border: 0;
    border-radius: 0.625rem; background: transparent; color: var(--text);
    font: inherit; font-size: 0.8125rem; text-align: left;
    text-decoration: none; cursor: pointer;
  }}
  .epsx-session-menu-item:hover {{ background: var(--bg-secondary); color: var(--text); }}
  .epsx-session-menu-item svg {{ color: var(--epsx-orange); flex-shrink: 0; }}
  .epsx-session-sign-out {{ color: var(--epsx-red); }}
  .epsx-session-sign-out svg {{ color: var(--epsx-red); }}
  html.dark .epsx-session-menu-item:hover {{ background: #1e293b; color: white; }}

  /* Wallet-connected prompt from the development navigation client. */
  .epsx-sign-in-banner {{
    position: sticky; top: 3.5rem; z-index: 40;
    display: flex; align-items: center; justify-content: center; gap: 0.75rem;
    padding: 0.75rem 1.5rem; color: white; font-size: 1rem;
    background: linear-gradient(90deg, #5a33b8, #7645d9 52%, #1a9bab);
    box-shadow: 0 10px 20px rgba(15,23,42,0.18);
  }}
  .epsx-sign-in-banner-action {{
    color: white; font-weight: 700; text-decoration: none;
    border-radius: 0.375rem; padding: 0.25rem 1rem;
    background: rgba(255,255,255,0.2); transition: background 0.15s ease;
  }}
  .epsx-sign-in-banner-action:hover {{ background: rgba(255,255,255,0.3); }}
  @media (max-width: 640px) {{
    .epsx-sign-in-banner {{ flex-wrap: wrap; gap: 0.4rem 0.65rem; font-size: 0.875rem; text-align: center; }}
  }}

  /* === Theme toggle (sun/moon) === */
  .epsx-theme-btn {{
    display: inline-flex; align-items: center; justify-content: center;
    width: 2.5rem; height: 2.5rem;
    padding: 0.5rem;
    border-radius: 1rem;
    background: transparent;
    color: #475569;
    border: 0;
    cursor: pointer;
    transition: all 0.15s ease;
  }}
  .epsx-theme-btn:hover {{ background: #fefce8; color: #ea580c; }}
  html.dark .epsx-theme-btn {{ background: rgba(30, 41, 59, 0.5); color: #cbd5e1; }}
  html.dark .epsx-theme-btn:hover {{ background: rgba(255, 255, 255, 0.1); color: #fb923c; }}
  .epsx-theme-btn [data-epsx-theme-icon] {{ color: var(--epsx-orange); }}
  .epsx-theme-btn [data-epsx-theme-icon="sun"],
  #epsx-theme-toggle [data-epsx-theme-icon="sun"] {{ display: none !important; }}
  .epsx-theme-btn [data-epsx-theme-icon="moon"],
  #epsx-theme-toggle [data-epsx-theme-icon="moon"] {{ display: inline-flex !important; }}
  html.dark .epsx-theme-btn [data-epsx-theme-icon="sun"],
  html.dark #epsx-theme-toggle [data-epsx-theme-icon="sun"] {{ display: inline-flex !important; }}
  html.dark .epsx-theme-btn [data-epsx-theme-icon="moon"],
  html.dark #epsx-theme-toggle [data-epsx-theme-icon="moon"] {{ display: none !important; }}
  /* Development's chain selector is read-only in the SSR shell. Keep its
     tablet/desktop label visually aligned with the source action cluster;
     hydrated wallet switching can replace this slot later. */
  .epsx-network-badge {{
    display: inline-flex; align-items: center; gap: 0.5rem;
    color: #94a3b8; font-size: 0.875rem; font-weight: 500;
    white-space: nowrap; line-height: 1;
  }}
  .epsx-network-badge svg {{ color: var(--epsx-orange); flex-shrink: 0; }}
  html.dark .epsx-network-badge {{ color: #94a3b8; }}
  @media (max-width: 767px) {{ .epsx-network-badge {{ display: none; }} }}
  .epsx-desktop-navigation {{ display: none !important; }}
  .epsx-compact-brand {{ display: flex !important; }}
  .epsx-desktop-session {{ display: none !important; }}
  .epsx-tablet-session {{ display: none !important; }}
  .epsx-header #epsx-mobile-menu-btn {{ display: inline-flex !important; }}
  @media (min-width: 640px) and (max-width: 767px) {{
    .epsx-tablet-session {{ display: flex !important; }}
  }}
  @media (min-width: 768px) {{
    .epsx-desktop-session {{ display: flex !important; }}
  }}
  @media (min-width: 1024px) {{
    .epsx-desktop-navigation {{ display: flex !important; }}
    .epsx-compact-brand {{ display: none !important; }}
    .epsx-header #epsx-mobile-menu-btn {{ display: none !important; }}
  }}
  /* The development mobile nav keeps the header minimal: theme, wallet,
     and notification actions move into the sheet below 640px. */
  @media (max-width: 639px) {{
    .epsx-header [data-epsx-theme-toggle],
    .epsx-header .epsx-notification-link {{ display: none; }}
    .epsx-header > div > div:last-child > div[class~="md:hidden"] {{ display: none !important; }}
    .epsx-header #epsx-mobile-menu-btn {{
      background: transparent; border-color: transparent; color: var(--epsx-orange);
    }}
    html.dark .epsx-header #epsx-mobile-menu-btn {{ background: transparent; border-color: transparent; color: var(--epsx-orange); }}
  }}
  .epsx-notification-link {{ position: relative; text-decoration: none; }}
  .epsx-notification-badge {{
    position: absolute; top: -0.35rem; right: -0.35rem;
    min-width: 1.1rem; height: 1.1rem; padding: 0 0.25rem;
    display: inline-flex; align-items: center; justify-content: center;
    border-radius: 9999px; border: 2px solid var(--bg);
    background: #dc2626; color: white;
    font-size: 0.625rem; font-weight: 700; line-height: 1;
  }}
  .epsx-notification-badge[hidden] {{ display: none !important; }}

  /* === Mobile menu sheet (< 1024px) === */
  .epsx-mobile-sheet {{
    position: fixed;
    inset: 0;
    z-index: 100;
    background: rgba(0,0,0,0.5);
    backdrop-filter: blur(4px);
    -webkit-backdrop-filter: blur(4px);
    display: none;
    align-items: stretch;
    justify-content: flex-end;
    padding: 0;
  }}
  .epsx-mobile-sheet.open {{ display: flex; }}
  @media (min-width: 1024px) {{ .epsx-mobile-sheet {{ display: none !important; }} }}
  .epsx-mobile-sheet-inner {{
    display: flex;
    flex-direction: column;
    width: 85vw;
    max-width: 24rem;
    height: 100dvh;
    max-height: none;
    margin-left: auto;
    overflow: hidden;
    background: #fff;
    border-left: 1px solid #e2e8f0;
    border-radius: 0;
    padding: 0;
    box-shadow: -25px 0 50px -12px rgba(0,0,0,0.35);
  }}
  html.dark .epsx-mobile-sheet-inner {{ background: #0f172a; border-left-color: #334155; }}
  .epsx-mobile-sheet-header {{
    display: flex;
    min-height: 4rem;
    align-items: center;
    justify-content: space-between;
    gap: 1rem;
    padding: 1rem;
    border-bottom: 1px solid var(--epsx-border);
  }}
  .epsx-mobile-navigation {{
    flex: 1;
    overflow-y: auto;
    padding: 1rem;
  }}
  .epsx-mobile-group {{ margin-bottom: 0.25rem; }}
  .epsx-mobile-group-trigger {{
    display: flex;
    width: 100%;
    align-items: center;
    justify-content: space-between;
    gap: 0.5rem;
    min-height: 2.5rem;
    padding: 0.625rem 0.75rem;
    border: 0;
    border-radius: 0.375rem;
    background: transparent;
    color: #475569;
    font: inherit;
    font-size: 0.875rem;
    font-weight: 500;
    cursor: pointer;
  }}
  .epsx-mobile-group-trigger:hover {{ color: #0f172a; background: #f8fafc; }}
  .epsx-mobile-group-trigger.active {{ color: #0f172a; }}
  html.dark .epsx-mobile-group-trigger {{ color: #94a3b8; }}
  html.dark .epsx-mobile-group-trigger:hover,
  html.dark .epsx-mobile-group-trigger.active {{ color: #fff; background: #1e293b; }}
  .epsx-mobile-group-label {{ display: flex; align-items: center; gap: 0.5rem; }}
  .epsx-mobile-group-trigger .epsx-mobile-icon,
  .epsx-mobile-group-trigger .epsx-mobile-chevron {{ color: var(--epsx-orange); flex-shrink: 0; }}
  .epsx-mobile-chevron {{ transition: transform 0.2s ease; }}
  .epsx-mobile-group-trigger[aria-expanded="true"] .epsx-mobile-chevron {{ transform: rotate(90deg); }}
  .epsx-mobile-group-items {{
    display: flex;
    flex-direction: column;
    gap: 0.125rem;
    margin-left: 0.75rem;
    padding-left: 0.75rem;
    border-left: 1px solid var(--epsx-border);
  }}
  .epsx-mobile-group-items[hidden] {{ display: none !important; }}
  .epsx-mobile-link {{
    display: flex; align-items: center; gap: 0.625rem;
    padding: 0.5rem 0.75rem;
    border-radius: 0.375rem;
    color: #64748b;
    text-decoration: none;
    font-size: 0.875rem;
    font-weight: 400;
    transition: background 0.15s ease;
  }}
  .epsx-mobile-link.active {{ color: #0f172a; background: #f1f5f9; }}
  html.dark .epsx-mobile-link {{ color: #94a3b8; }}
  html.dark .epsx-mobile-link.active {{ color: #fff; background: #1e293b; }}
  .epsx-mobile-link .epsx-mobile-icon,
  #epsx-mobile-menu-btn .epsx-mobile-menu-icon {{
    color: var(--epsx-orange);
    flex-shrink: 0;
  }}
  .epsx-mobile-link:hover, .epsx-mobile-link:active {{ background: var(--bg-secondary); }}
  .epsx-mobile-session {{ padding: 1rem; border-top: 1px solid var(--epsx-border); }}
  .epsx-mobile-session .epsx-mobile-link {{ width: 100%; min-height: 2.5rem; }}
  .epsx-mobile-session .epsx-mobile-connect {{
    display: flex;
    width: 100%;
    min-height: 2.5rem;
    align-items: center;
    justify-content: center;
    gap: 0.5rem;
    padding: 0 1rem;
    border: 0;
    border-radius: 1rem;
    background: linear-gradient(90deg, #fb923c, #ea580c);
    color: #fff;
    font-size: 0.875rem;
    font-weight: 500;
    text-decoration: none;
  }}
  .epsx-mobile-session .epsx-mobile-connect .epsx-mobile-icon {{ color: #fff; }}
  #epsx-mobile-menu-btn {{ border-radius: 0.375rem; background: transparent; }}
  html.dark .epsx-header #epsx-mobile-menu-btn {{ background: transparent; }}
  .epsx-header :where(a[href], button):focus-visible, .epsx-mobile-sheet :where(a[href], button):focus-visible, .footer a[href]:focus-visible {{
    outline: 3px solid var(--focus-ring);
    outline-offset: 2px;
  }}

  /* === Hero entrance animations (epsx.io staggered) === */
  @keyframes epsx-slide-up {{
    0% {{ opacity: 0; transform: translateY(20px); }}
    to {{ opacity: 1; transform: translateY(0); }}
  }}
  @keyframes epsx-fade-in {{
    0% {{ opacity: 0; }}
    to {{ opacity: 1; }}
  }}
  @keyframes epsx-gradient-x {{
    0%, to {{ background-position: 0% 50%; }}
    50% {{ background-position: 100% 50%; }}
  }}
  .animate-slide-up       {{ animation: epsx-slide-up .6s ease-out both; }}
  .animate-slide-up-d1    {{ animation: epsx-slide-up .6s ease-out .2s both; }}
  .animate-slide-up-d2    {{ animation: epsx-slide-up .6s ease-out .4s both; }}
  .animate-fade-in-d3     {{ animation: epsx-fade-in .6s ease-out .6s both; }}
  .animate-gradient-x {{
    background-size: 200% 200%;
    animation: epsx-gradient-x 3s ease infinite;
  }}

  /* === Hero gradient text (orange→yellow→orange) === */
  .hero-gradient-text {{
    background: linear-gradient(90deg, #f97316 0%, #eab308 50%, #ea580c 100%);
    background-size: 200% 200%;
    -webkit-background-clip: text;
    background-clip: text;
    -webkit-text-fill-color: transparent;
    color: transparent;
  }}

  /* === Hero stat card (rounded-2xl, glass with colored overlay) === */
  .stat-card {{
    position: relative;
    background: rgba(255,255,255,0.8);
    backdrop-filter: blur(20px); -webkit-backdrop-filter: blur(20px);
    border-radius: 1rem;
    padding: 2rem;
    box-shadow: 0 25px 50px -12px rgba(0,0,0,0.25);
    border: 1px solid rgba(249,115,22,0.5);
    transition: all 0.3s ease;
    overflow: hidden;
  }}
  html.dark .stat-card {{
    background: rgba(30, 41, 59, 0.8);
    border-color: rgba(251, 146, 60, 0.2);
  }}
  .stat-card:hover {{ transform: scale(1.05); }}
  .stat-card .stat-overlay {{
    position: absolute; inset: 0;
    background: linear-gradient(135deg, var(--c1, #3b82f6) 0%, var(--c2, #06b6d4) 100%);
    opacity: 0.05;
    transition: opacity 0.3s ease;
    pointer-events: none;
  }}
  .stat-card:hover .stat-overlay {{ opacity: 0.1; }}
  .stat-card .stat-content {{ position: relative; z-index: 10; text-align: center; }}
  .stat-card .stat-icon {{
    height: 2.5rem; width: 2.5rem;
    margin: 0 auto 1rem;
    color: var(--epsx-orange);
    transition: color 0.3s ease;
  }}
  .stat-card .stat-num {{
    font-size: 1.875rem; font-weight: 700;
    background: linear-gradient(90deg, var(--c1, #3b82f6), var(--c2, #06b6d4));
    -webkit-background-clip: text; background-clip: text;
    -webkit-text-fill-color: transparent;
    margin-bottom: 0.5rem;
  }}
  @media (min-width: 640px) {{
    .stat-card .stat-num {{ font-size: 2.25rem; }}
  }}
  .stat-card .stat-label {{
    font-size: 0.875rem; font-weight: 500;
    color: var(--text-muted);
  }}
  html.dark .stat-card .stat-label {{ color: #cbd5e1; }}

  /* === Dioxus homepage source-parity surfaces ===
     The Rust SSR emits the development hero's responsive utility classes,
     while these stable selectors keep the same glass surfaces available when
     a cached Tailwind bundle is served during a local rebuild. */
  .home-prod-plan-card {{
    background: rgba(255,255,255,0.8);
    border-color: rgba(249,115,22,0.5);
    color: var(--text);
  }}
  .home-prod-plan-title {{ color: var(--text); }}
  .home-prod-plan-sub {{ color: var(--text-muted); }}
  html:not(.dark) .home-prod-plan-card > div > p,
  html:not(.dark) .home-prod-plan-card > ul {{ color: #475569; }}
  html:not(.dark) .home-prod-plan-card > div > h3 {{ color: #7e22ce; }}
  /* The local utility bundle intentionally contains only a bounded subset of
     arbitrary responsive classes. Pin the reference landing measurements in
     stable selectors so the Rust SSR remains visually deterministic. */
  @media (min-width: 1024px) {{
    /* The connected hero is vertically centered beneath the sign-in banner.
       Keep the inner frame at its flex-center position; translating it upward
       makes the desktop state sit 12px above the supplied reference capture. */
    .home-prod-hero:not(.home-prod-hero-signed-out) > .home-prod-hero-inner {{ transform: none; }}
    .home-prod-hero-signed-out h1 {{ font-size: 96px !important; line-height: 120px !important; letter-spacing: normal !important; }}
  }}
  @media (min-width: 768px) {{
    /* The source hero uses md:text-2xl + leading-relaxed. Keep the
       connected SSR hero at that authored size even when the compact
       local utility bundle omits responsive text utilities. */
    .home-prod-hero-subtitle {{ font-size: 24px !important; line-height: 39px !important; }}
  }}
  @media (min-width: 640px) {{
    .home-prod-hero-signed-out .home-prod-hero-cta {{ min-width: 220px !important; height: 56px !important; padding-top: 0 !important; padding-bottom: 0 !important; font-size: 1rem !important; }}
  }}
  html.dark .home-prod-plan-card {{
    background: rgba(30,41,59,0.7);
    border-color: rgba(251,146,60,0.2);
  }}
  html.dark .home-prod-plan-title {{ color: #ffffff; }}
  html.dark .home-prod-plan-sub {{ color: #cbd5e1; }}

  /* Empty/error news states used a dark translucent gradient in both themes,
     leaving slate-300 and white copy washed out over the light page. */
  html:not(.dark) .home-prod-news[data-home-news-state="empty"] > div,
  html:not(.dark) .home-prod-news[data-home-news-state="unavailable"] > div {{
    background: rgba(255,255,255,0.82) !important;
    border-color: #cbd5e1 !important;
    box-shadow: var(--shadow-lg);
  }}
  html:not(.dark) .home-prod-news[data-home-news-state] > div > p {{ color: #475569; }}
  html:not(.dark) .home-prod-news[data-home-news-state] > div a {{
    color: #0e7490;
    border-color: #94a3b8;
  }}

  /* This shared sign-in banner is rendered on several public data surfaces.
     Its essential gradient colors must not depend on optional Tailwind
     utilities being present in the frozen browser bundle. */
  html:not(.dark) .auth-access-banner {{
    border-color: #d8b4fe !important;
    background: linear-gradient(90deg, #faf5ff 0%, #ffffff 50%, #fdf2f8 100%) !important;
  }}
  .auth-access-banner-icon,
  .auth-access-banner-cta {{
    background: linear-gradient(90deg, #7e22ce 0%, #be185d 100%) !important;
    color: #ffffff !important;
  }}
  .auth-access-banner-cta:hover {{
    background: linear-gradient(90deg, #6b21a8 0%, #9d174d 100%) !important;
  }}
  html.dark .auth-access-banner {{
    border-color: rgba(168, 85, 247, 0.5) !important;
    background: linear-gradient(90deg, rgba(168,85,247,0.15) 0%, rgba(15,23,42,0.82) 50%, rgba(236,72,153,0.15) 100%) !important;
  }}

  /* === Company card (Performance Companies) === */
  .company-card {{
    position: relative; border-radius: 1rem; padding: 1.25rem;
    display: flex; flex-direction: column;
    background: rgba(255,255,255,0.12);
    backdrop-filter: blur(20px); -webkit-backdrop-filter: blur(20px);
    border: 1px solid rgba(255,255,255,0.2);
    transition: all 0.3s ease;
  }}
  html.dark .company-card {{
    background: rgba(255,255,255,0.08);
    border-color: rgba(255,255,255,0.1);
  }}
  .company-card:hover {{
    transform: translateY(-4px);
    box-shadow: 0 20px 25px -5px rgba(59,130,246,0.1);
  }}
  .company-card .row-card {{
    padding: 0.75rem; border-radius: 0.75rem;
    background: rgba(255,255,255,0.05);
    transition: background 0.15s ease;
  }}
  .company-card .row-card:hover {{ background: rgba(255,255,255,0.1); }}
  .company-card .row-icon {{
    padding: 0.375rem; border-radius: 0.375rem;
    display: inline-flex; align-items: center; justify-content: center;
  }}
  .company-card .progress-track {{
    height: 0.375rem; width: 100%;
    background: rgba(229,231,235,1);
    border-radius: 9999px; overflow: hidden;
  }}
  html.dark .company-card .progress-track {{ background: rgba(55,65,81,0.5); }}
  .company-card .progress-fill {{
    height: 100%; border-radius: 9999px; position: relative;
  }}
  .company-card .progress-shine {{
    position: absolute; inset: 0; background: rgba(255,255,255,0.2);
  }}
  .company-card .view-btn {{
    width: 100%; padding: 0.75rem;
    border-radius: 0.75rem; font-weight: 700; font-size: 0.875rem;
    color: white; transition: all 0.3s ease;
    background: linear-gradient(90deg, #2563eb 0%, #1d4ed8 100%);
    border: 0; cursor: pointer;
  }}
  .company-card .view-btn:hover {{ box-shadow: 0 10px 15px -3px rgba(59,130,246,0.25); }}

  /* === Pricing card === */
  .pricing-card {{
    position: relative;
    background: var(--surface-solid);
    border: 1px solid var(--epsx-border);
    border-radius: 1rem;
    padding: 1.5rem;
    display: flex; flex-direction: column;
    transition: all 0.3s ease;
  }}
  html.dark .pricing-card {{ background: rgba(15,23,42,0.6); border-color: rgba(255,255,255,0.1); }}
  .pricing-card:hover {{ transform: translateY(-4px); box-shadow: var(--shadow-xl); }}
  .pricing-card .sale-badge {{
    position: absolute; top: -0.5rem; left: 1rem;
    background: linear-gradient(90deg, #ef4444 0%, #ec4899 100%);
    color: white;
    font-size: 0.75rem; font-weight: 700;
    padding: 0.25rem 0.5rem;
    border-radius: 0.25rem;
    letter-spacing: 0.05em;
  }}
  .pricing-card .price-title {{
    font-size: 1.125rem; font-weight: 700;
    color: var(--text);
    text-transform: uppercase; letter-spacing: 0.1em;
    margin-bottom: 1rem;
  }}
  html.dark .pricing-card .price-title {{ color: #f1f5f9; }}
  .pricing-card .price-amount {{
    display: flex; align-items: baseline; gap: 0.25rem;
    color: var(--epsx-blue-start);
    font-weight: 900; font-size: 2.5rem;
  }}
  .pricing-card .price-amount .currency {{ font-size: 1.5rem; }}
  .pricing-card .price-amount .suffix {{ font-size: 0.875rem; font-weight: 700; }}
  .pricing-card .promo-badge {{
    display: inline-block;
    background: linear-gradient(90deg, #ef4444 0%, #ec4899 100%);
    color: white; font-size: 0.75rem; font-weight: 700;
    padding: 0.125rem 0.5rem; border-radius: 0.25rem;
    margin-left: 0.5rem;
  }}
  .pricing-card .price-original {{
    color: var(--text-subtle);
    text-decoration: line-through;
    font-size: 0.875rem;
    margin-top: 0.25rem;
  }}
  .pricing-card .price-savings {{
    color: var(--text-subtle);
    font-size: 0.875rem;
    margin-top: 0.25rem;
  }}
  .pricing-card .countdown {{
    color: var(--text-subtle);
    font-size: 0.75rem;
    margin-top: 0.5rem;
  }}
  .pricing-card .features {{
    list-style: none; padding: 0; margin: 1.5rem 0;
    display: flex; flex-direction: column; gap: 0.625rem;
  }}
  .pricing-card .features li {{
    display: flex; align-items: center; gap: 0.5rem;
    font-size: 0.875rem;
    color: var(--text-muted);
  }}
  html.dark .pricing-card .features li {{ color: #cbd5e1; }}
  .pricing-card .features .check {{ color: #10b981; width: 1rem; height: 1rem; flex-shrink: 0; }}
  .pricing-card .cta-btn {{
    width: 100%; padding: 0.75rem; border-radius: 0.5rem;
    color: white; font-weight: 600; font-size: 0.875rem;
    background: linear-gradient(90deg, #06b6d4 0%, #3b82f6 100%);
    border: 0; cursor: pointer;
    transition: all 0.3s ease;
  }}
  .pricing-card .cta-btn:hover {{
    background: linear-gradient(90deg, #0891b2 0%, #2563eb 100%);
    box-shadow: 0 10px 20px rgba(6,182,212,0.3);
  }}

  /* === Section heading helpers === */
  .epsx-h2 {{
    font-size: 1.875rem; font-weight: 700; color: var(--text); text-align: center;
  }}
  @media (min-width: 640px) {{ .epsx-h2 {{ font-size: 2.25rem; }} }}
  html.dark .epsx-h2 {{ color: var(--text); }}
  .epsx-h2-orange {{
    font-size: 1.875rem; font-weight: 700; text-align: center;
  }}
  @media (min-width: 640px) {{ .epsx-h2-orange {{ font-size: 3rem; }} }}
  .epsx-h2-pink-purple {{
    font-size: 1.875rem; font-weight: 700; text-align: center;
    background: linear-gradient(90deg, #f97316 0%, #ec4899 50%, #a855f7 100%);
    -webkit-background-clip: text; background-clip: text;
    -webkit-text-fill-color: transparent;
  }}
  @media (min-width: 640px) {{ .epsx-h2-pink-purple {{ font-size: 3rem; }} }}
  .epsx-h2-purple {{
    font-size: 1.875rem; font-weight: 700; text-align: center;
    background: linear-gradient(90deg, #a855f7 0%, #8b5cf6 50%, #d946ef 100%);
    -webkit-background-clip: text; background-clip: text;
    -webkit-text-fill-color: transparent;
  }}
  @media (min-width: 640px) {{ .epsx-h2-purple {{ font-size: 3rem; }} }}
  .epsx-section-underline {{
    margin: 0.75rem auto 0;
    width: 6rem; height: 0.25rem; border-radius: 9999px;
  }}
  .epsx-section-underline.warm  {{ background: linear-gradient(90deg, #f97316 0%, #eab308 100%); }}
  .epsx-section-underline.pink  {{ background: linear-gradient(90deg, #ec4899 0%, #a855f7 100%); }}
  .epsx-section-underline.purple{{ background: linear-gradient(90deg, #a855f7 0%, #d946ef 100%); }}

  .epsx-section {{ padding: 4rem 1rem; }}
  @media (min-width: 640px) {{ .epsx-section {{ padding: 6rem 1.5rem; }} }}
  @media (min-width: 1024px) {{ .epsx-section {{ padding: 8rem 1.5rem; }} }}

  /* === News cards (epsx.io: featured 2/3 + 2 small 1/3 each) === */
  .news-featured {{
    position: relative; border-radius: 1.5rem; overflow: hidden;
    height: 320px;
    background: linear-gradient(135deg, rgba(118,69,217,0.2) 0%, rgba(31,199,212,0.1) 50%, rgba(15,23,42,0.6) 100%);
    border: 1px solid rgba(255,255,255,0.1);
  }}
  @media (min-width: 640px) {{ .news-featured {{ height: 400px; }} }}
  .news-featured img,
  .news-small img {{
    position: absolute; inset: 0;
    width: 100%; height: 100%; object-fit: cover;
    transition: transform 0.7s ease;
  }}
  .news-featured:hover img,
  .news-small:hover img {{ transform: scale(1.05); }}
  .news-overlay {{
    position: absolute; inset: 0;
    background: linear-gradient(180deg, transparent 0%, rgba(0,0,0,0.2) 50%, rgba(0,0,0,0.8) 100%);
  }}
  .news-caption {{ position: absolute; bottom: 0; left: 0; right: 0; padding: 1.5rem; }}
  @media (min-width: 640px) {{ .news-caption {{ padding: 2rem; }} }}
  .news-featured-tag {{
    display: inline-flex; align-items: center; gap: 0.375rem;
    color: #1fc7d4;
    font-size: 0.75rem; font-weight: 500;
    text-transform: uppercase; letter-spacing: 0.05em;
  }}
  .news-tag {{
    display: inline-block;
    background: rgba(255,255,255,0.1);
    backdrop-filter: blur(4px); -webkit-backdrop-filter: blur(4px);
    color: rgba(255,255,255,0.8);
    font-size: 0.75rem;
    padding: 0.125rem 0.5rem;
    border-radius: 9999px;
  }}
  .news-title {{
    color: white; font-weight: 700;
    transition: color 0.15s ease;
  }}
  .news-featured .news-title {{
    font-size: 1.25rem; line-height: 1.2;
  }}
  @media (min-width: 640px) {{ .news-featured .news-title {{ font-size: 1.5rem; }} }}
  .news-small .news-title {{
    font-size: 0.875rem; line-height: 1.25;
  }}
  a:hover .news-title {{ color: #1fc7d4; }}
  .news-excerpt {{
    color: rgba(255,255,255,0.7);
    font-size: 0.875rem; line-height: 1.4;
    margin: 0.5rem 0 0.75rem;
  }}
  .news-date {{ color: rgba(255,255,255,0.5); font-size: 0.75rem; }}
  .news-small {{
    position: relative; border-radius: 1rem; overflow: hidden;
    height: 180px;
    background: linear-gradient(135deg, rgba(118,69,217,0.2) 0%, rgba(31,199,212,0.1) 50%, rgba(15,23,42,0.6) 100%);
    border: 1px solid rgba(255,255,255,0.1);
  }}
  .news-small .news-caption {{ padding: 1rem; }}
  .news-small .news-date {{ margin-top: 0.25rem; display: block; }}

  /* === Decorative blob blur (epsx.io) === */
  .epsx-blob {{
    position: absolute;
    border-radius: 9999px;
    filter: blur(24px);
    pointer-events: none;
  }}

  /* === Tables === */
  .table-wrap {{ overflow-x: auto; border-radius: 0.75rem; border: 1px solid var(--epsx-border); }}
  .table {{ width: 100%; border-collapse: collapse; font-size: 0.875rem; }}
  .table th {{
    background: var(--bg-secondary);
    color: var(--text-muted);
    text-align: left;
    padding: 0.75rem 1rem;
    font-weight: 600;
    font-size: 0.75rem;
    text-transform: uppercase;
    letter-spacing: 0.05em;
    border-bottom: 1px solid var(--epsx-border);
  }}
  .table td {{
    padding: 0.875rem 1rem;
    border-bottom: 1px solid var(--epsx-border);
    color: var(--text);
  }}
  .table tr:last-child td {{ border-bottom: none; }}
  .table tr:hover {{ background: var(--bg-secondary); }}

  /* === Mobile sheet === */
  .mobile-sheet {{
    position: fixed;
    inset: 0;
    z-index: 9999;
    display: none;
  }}
  .mobile-sheet.open {{ display: block; }}
  .mobile-sheet .backdrop {{
    position: absolute;
    inset: 0;
    background: rgba(0,0,0,0.5);
    backdrop-filter: blur(4px);
  }}
  .mobile-sheet .panel {{
    position: absolute;
    right: 0;
    top: 0;
    bottom: 0;
    width: 85vw;
    max-width: 24rem;
    background: var(--surface-solid);
    border-left: 1px solid var(--epsx-border);
    padding: 1rem;
    overflow-y: auto;
    animation: slideInRight 0.25s ease-out;
  }}
  .mobile-sheet .hamburger {{ display: block; }}
  @media (min-width: 1024px) {{ .mobile-sheet .hamburger {{ display: none; }} }}

  /* === Utility === */
  .text-balance {{ text-wrap: balance; }}
  .text-pretty  {{ text-wrap: pretty; }}
  .divide-y > * + * {{ border-top: 1px solid var(--epsx-border); }}
  .ring-1 {{ box-shadow: 0 0 0 1px var(--epsx-border); }}
  .scrollbar-thin::-webkit-scrollbar {{ width: 6px; height: 6px; }}
  .scrollbar-thin::-webkit-scrollbar-thumb {{ background: var(--epsx-border-strong); border-radius: 3px; }}

  /* =================================================================
   * Wave 1 — Track A: form & input primitive parity
   * ----------------------------------------------------------------
   * Block comment cataloguing every new class added by the Track A
   * Dioxus primitives (form, input, select, combobox, date_picker,
   * stepper, checkbox, switch, misc). Reuses the existing --epsx-*
   * CSS custom properties and the global Tailwind v2 utility set.
   * Each entry is: `<class>` — `<purpose>` — `<consumer primitive file>`.
   * ----------------------------------------------------------------
   *  input-error             — invalid input border + ring            (primitives/input.rs)
   *  input-with-icon         — left padding when an icon is present   (primitives/input.rs)
   *  label-required          — red "*" indicator on required labels   (primitives/form.rs :: Label)
   *  form-section            — boxed subsection of a long form        (primitives/form.rs :: FormSection)
   *  form-section-header     — header row holding title + description (primitives/form.rs :: FormSection)
   *  form-section-title      — h3 title inside the section header     (primitives/form.rs :: FormSection)
   *  form-section-description— muted description under the section title(primitives/form.rs :: FormSection)
   *  form-section-body       — content area below the section header  (primitives/form.rs :: FormSection)
   *  form-row                — responsive 1/2/3-column grid for fields(primitives/form.rs :: FormRow)
   *  input-group             — label + control + trailing-button row  (primitives/form.rs :: InputGroup)
   *  input-group-label       — label rendered above the control row   (primitives/form.rs :: InputGroup)
   *  input-group-control     — flex row holding the control(s)        (primitives/form.rs :: InputGroup)
   *  input-group-help        — inline help text below the control row (primitives/form.rs :: InputGroup)
   *  input-group-error       — red error text below the control row   (primitives/form.rs :: InputGroup)
   *  radio-group             — vertical stack of radio rows           (primitives/form.rs :: RadioGroup)
   *  radio-group-label       — group label rendered above the rows    (primitives/form.rs :: RadioGroup)
   *  radio-group-help        — help text below the radio stack        (primitives/form.rs :: RadioGroup)
   *  radio-group-error       — error text below the radio stack       (primitives/form.rs :: RadioGroup)
   *  radio-row               — single radio row (label + input)       (primitives/form.rs :: RadioGroup)
   *  radio-row.selected      — visual cue for the currently-selected row(primitives/form.rs :: RadioGroup)
   *  radio-row-label         — label text inside a radio row          (primitives/form.rs :: RadioGroup)
   *  multiselect             — top-level wrapper around a multi-select(primitives/select.rs :: MultiSelect)
   *  multiselect-control     — flex row holding chips + trigger       (primitives/select.rs :: MultiSelect)
   *  multiselect-chip        — single chip for a selected value       (primitives/select.rs :: MultiSelect)
   *  multiselect-chip-remove — × button inside a chip                 (primitives/select.rs :: MultiSelect)
   *  multiselect-trigger     — "Add…" button that opens the dropdown  (primitives/select.rs :: MultiSelect)
   *  multiselect-menu        — dropdown panel listing the options     (primitives/select.rs :: MultiSelect)
   *  multiselect-option      — single option inside the dropdown      (primitives/select.rs :: MultiSelect)
   *  multiselect-option.selected — visual cue for selected options   (primitives/select.rs :: MultiSelect)
   *  combobox-async          — modifier on a combobox with async load (primitives/combobox.rs :: ComboboxAsync)
   *  combobox-loading        — "Loading…" item inside the menu        (primitives/combobox.rs :: ComboboxAsync)
   *  combobox-empty          — "No matches" item inside the menu      (primitives/combobox.rs :: ComboboxAsync)
   *  combobox-multi          — modifier on a multi-select combobox    (primitives/combobox.rs :: ComboboxMulti)
   *  combobox-multi-control  — flex row holding chips + search input  (primitives/combobox.rs :: ComboboxMulti)
   *  combobox-multi-chip     — single chip in a multi-select combobox (primitives/combobox.rs :: ComboboxMulti)
   *  combobox-multi-chip-remove — × button inside a multi chip      (primitives/combobox.rs :: ComboboxMulti)
   *  combobox-multi-input    — trailing search input after the chips  (primitives/combobox.rs :: ComboboxMulti)
   *  datetime-picker         — flex row holding the date + time inputs(primitives/date_picker.rs :: DateTimePicker)
   *  stepper-wrap            — outer wrapper around the progress bar  (primitives/stepper.rs)
   *                            and the row of step circles
   *  stepper-progress        — linear progress bar above the stepper  (primitives/stepper.rs)
   *  rating-interactive      — hover-able, clickable rating           (primitives/misc.rs :: Rating)
   *  rating-disabled         — non-interactive, dimmed rating         (primitives/misc.rs :: Rating)
   *  switch-sm / switch-md / switch-lg — size variants of the switch  (primitives/switch.rs)
   *  state-checked / state-unchecked    — checked/unchecked visual state for SwitchRoot(primitives/switch.rs)
   *  kbd-combo               — wrapper for multi-key keyboard shortcut(primitives/misc.rs :: KbdCombo)
   *  kbd-combo-sep           — "+" separator between combo keys       (primitives/misc.rs :: KbdCombo)
   *  slider-field            — vertical layout wrapper around a slider(primitives/misc.rs :: Slider)
   *  checkbox-indeterminate  — partial-fill visual state              (primitives/checkbox.rs)
   *  =================================================================
   */

  /* === Input variants === */
  .input-error {{
    border-color: var(--epsx-red);
    box-shadow: 0 0 0 3px rgba(239,68,68,0.15);
  }}
  .input-with-icon {{ padding-left: 2.5rem; }}
  .input-icon {{
    position: absolute;
    left: 0.875rem;
    top: 50%;
    transform: translateY(-50%);
    color: var(--text-subtle);
    pointer-events: none;
  }}
  .label-required {{
    color: var(--epsx-red);
    margin-left: 0.25rem;
  }}

  /* === Form section / row / input-group === */
  .form-section {{
    border: 1px solid var(--epsx-border);
    border-radius: 0.75rem;
    padding: 1.25rem 1.25rem 1rem;
    background: var(--bg-secondary);
  }}
  .form-section-header {{
    margin-bottom: 1rem;
  }}
  .form-section-title {{
    font-size: 1rem;
  }}
  /*
   * Wave 1 — Track C: interactive primitive parity
   * ----------------------------------------------------------------
   * Block comment cataloguing every new class added by the Track C
   * Dioxus primitives (dropdown, modal, tabs, tooltip, overlays,
   * rich_text). Reuses the existing --epsx-* CSS custom properties,
   * the global Tailwind v2 utility set, and the existing shadcn-style
   * class naming. Each entry is:
   *   `<class>` — `<purpose>` — `<consumer primitive file>`
   * ----------------------------------------------------------------
   *  dropdown-label            — non-interactive label inside menu  (primitives/dropdown.rs)
   *  dropdown-item-inset       — left-padded item (Radix `inset`)  (primitives/dropdown.rs)
   *  dropdown-item-check       — leading check column             (primitives/dropdown.rs)
   *  dropdown-item-checked     — modifier on a checked item         (primitives/dropdown.rs)
   *  dropdown-checkbox-item    — checkbox-style item container      (primitives/dropdown.rs)
   *  dropdown-menu-side-top    — render the menu above the trigger  (primitives/dropdown.rs)
   *  dropdown-menu-side-bottom — render below the trigger (default) (primitives/dropdown.rs)
   *  dropdown-menu-align-start — align menu to the start of trigger (primitives/dropdown.rs)
   *  dropdown-menu-align-end   — align menu to the end of trigger   (primitives/dropdown.rs)
   *  dropdown-menu-align-center— center menu under trigger (default)(primitives/dropdown.rs)
   *
   *  modal-overlay             — full-viewport click-to-dismiss     (primitives/modal.rs)
   *  modal-header              — title + close button row           (primitives/modal.rs)
   *  modal-title               — dialog title heading               (primitives/modal.rs)
   *  modal-close               — "✕" close button                   (primitives/modal.rs)
   *  modal-description         — subtitle paragraph                 (primitives/modal.rs)
   *  modal-body                — content area below header          (primitives/modal.rs)
   *  modal-footer              — right-aligned action row           (primitives/modal.rs)
   *  modal-sm/lg/xl/full       — width size variants                (primitives/modal.rs)
   *
   *  tabs                      — top-level tab list                 (primitives/tabs.rs)
   *  tab                       — individual tab button              (primitives/tabs.rs)
   *  tab-active                — modifier on the active tab         (primitives/tabs.rs)
   *  tab-icon                  — leading icon inside a tab          (primitives/tabs.rs)
   *  tabs-vertical             — vertical layout orientation        (primitives/tabs.rs)
   *
   *  tooltip-wrapper           — hover/focus reveal wrapper         (primitives/tooltip.rs)
   *  tooltip-content           — the bubble itself                  (primitives/tooltip.rs)
   *  tooltip-open              — modifier when the bubble is shown  (primitives/tooltip.rs)
   *  tooltip-side-top          — bubble above the trigger           (primitives/tooltip.rs)
   *  tooltip-side-bottom       — bubble below (default)             (primitives/tooltip.rs)
   *  tooltip-side-left         — bubble to the left                 (primitives/tooltip.rs)
   *  tooltip-side-right        — bubble to the right                (primitives/tooltip.rs)
   *  tooltip-align-start       — align bubble to start              (primitives/tooltip.rs)
   *  tooltip-align-end         — align bubble to end                (primitives/tooltip.rs)
   *  tooltip-align-center      — center bubble (default)            (primitives/tooltip.rs)
   *
   *  popover                   — top-level popover wrapper          (primitives/overlays.rs)
   *  popover-trigger           — click-to-open trigger              (primitives/overlays.rs)
   *  popover-content           — the popover body                   (primitives/overlays.rs)
   *  popover-content-side-top  — render above the trigger           (primitives/overlays.rs)
   *  popover-content-side-bottom — render below the trigger         (primitives/overlays.rs)
   *  popover-content-side-left — render to the left                 (primitives/overlays.rs)
   *  popover-content-side-right— render to the right                (primitives/overlays.rs)
   *  popover-content-align-start — align popover to start           (primitives/overlays.rs)
   *  popover-content-align-end — align popover to end               (primitives/overlays.rs)
   *  popover-content-align-center — center popover (default)        (primitives/overlays.rs)
   *
   *  hover-card                — hover-triggered card wrapper       (primitives/overlays.rs)
   *  hover-card-content        — the card body                      (primitives/overlays.rs)
   *
   *  accordion                 — stack of collapsible sections      (primitives/overlays.rs)
   *  accordion-item            — single section                     (primitives/overlays.rs)
   *  accordion-trigger         — section header button              (primitives/overlays.rs)
   *  accordion-content         — section body                       (primitives/overlays.rs)
   *  accordion-icon            — "+/-" glyph at section header      (primitives/overlays.rs)
   *  accordion-item.open       — modifier when a section is open    (primitives/overlays.rs)
   *
   *  collapsible               — single collapsible section         (primitives/overlays.rs)
   *  collapsible-trigger       — section header button              (primitives/overlays.rs)
   *  collapsible-content       — section body                       (primitives/overlays.rs)
   *  collapsible.open          — modifier when the section is open  (primitives/overlays.rs)
   *
   *  command-palette-overlay   — full-screen modal scrim            (primitives/overlays.rs)
   *  command-palette           — dialog container (centered card)   (primitives/overlays.rs)
   *  command-input             — search input at the top            (primitives/overlays.rs)
   *  command-list              — scrollable list of items           (primitives/overlays.rs)
   *  command-item              — individual command row             (primitives/overlays.rs)
   *  command-item.active       — modifier on the focused row        (primitives/overlays.rs)
   *  command-empty             — "No matches" placeholder row       (primitives/overlays.rs)
   *  command-hint              — right-aligned shortcut hint        (primitives/overlays.rs)
   *
   *  rich-text-editor          — top-level wrapper around the RTE   (primitives/rich_text.rs)
   *  rte-toolbar               — formatting button bar above the    (primitives/rich_text.rs)
   *                              textarea
   *  rte-preview               — rendered markdown preview pane     (primitives/rich_text.rs)
   * =================================================================
   */

  /* === Dropdown: modifiers introduced by Track C === */
  .dropdown-label {{
    padding: 0.5rem 0.75rem;
    font-size: 0.75rem;
    font-weight: 600;
    color: var(--text-muted);
    text-transform: uppercase;
    letter-spacing: 0.04em;
  }}
  .dropdown-item-inset {{ padding-left: 2rem; }}
  .dropdown-item-check {{
    position: absolute;
    left: 0.5rem;
    top: 50%;
    transform: translateY(-50%);
    width: 1rem;
    height: 1rem;
    display: inline-flex;
    align-items: center;
    justify-content: center;
    color: var(--epsx-primary);
  }}
  .dropdown-checkbox-item {{
    position: relative;
  }}
  .dropdown-item-checked {{
    background: var(--bg-secondary);
  }}
  .dropdown-menu-side-top    {{ transform-origin: bottom; }}
  .dropdown-menu-side-bottom {{ transform-origin: top; }}
  .dropdown-menu-align-start  {{ left: 0; }}
  .dropdown-menu-align-end    {{ right: 0; }}
  .dropdown-menu-align-center {{ left: 50%; transform: translateX(-50%); }}

  /* === Modal: layout + size variants + slot styling === */
  .modal-overlay {{
    position: fixed;
    inset: 0;
    background: rgba(15, 23, 42, 0.55);
    backdrop-filter: blur(4px);
    -webkit-backdrop-filter: blur(4px);
    z-index: 1000;
    display: flex;
    align-items: center;
    justify-content: center;
    padding: 1rem;
    animation: fadeIn 0.15s ease-out;
  }}
  .modal-header {{
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 1rem;
    padding: 1.25rem 1.5rem;
    border-bottom: 1px solid var(--epsx-border);
  }}
  .modal-title {{
    font-size: 1.125rem;
    font-weight: 600;
    color: var(--text);
    margin: 0;
  }}
  .form-section-description {{
    font-size: 0.875rem;
    color: var(--text-muted);
    margin: 0.25rem 0 0;
  }}
  .form-section-body {{
    margin-top: 0.5rem;
  }}
  .form-row {{
    display: grid;
    grid-template-columns: 1fr;
    gap: 1rem;
  }}
  @media (min-width: 768px) {{
    .form-row {{ grid-template-columns: repeat(2, minmax(0, 1fr)); }}
    .form-row[data-cols="3"], .form-row.md\:grid-cols-3 {{ grid-template-columns: repeat(3, minmax(0, 1fr)); }}
  }}
  .input-group {{
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
  }}
  .input-group-label {{
    font-size: 0.875rem;
    font-weight: 500;
    color: var(--text-muted);
  }}
  .input-group-control {{
    display: flex;
    gap: 0.5rem;
    align-items: center;
  }}
  .input-group-help {{
    font-size: 0.75rem;
    color: var(--text-muted);
  }}
  .input-group-error {{
    font-size: 0.75rem;
    color: var(--epsx-red);
  }}

  /* === Radio group === */
  .radio-group {{
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
  }}
  .radio-group-label {{
    font-size: 0.875rem;
    font-weight: 500;
    color: var(--text-muted);
  }}
  .radio-group-help {{
    font-size: 0.75rem;
    color: var(--text-muted);
  }}
  .radio-group-error {{
    font-size: 0.75rem;
    color: var(--epsx-red);
  }}
  .radio-row {{
    display: flex;
    align-items: center;
    gap: 0.5rem;
    padding: 0.375rem 0.5rem;
    border-radius: 0.5rem;
    cursor: pointer;
    transition: background-color 0.15s ease;
  }}
  .radio-row:hover {{ background: var(--bg-secondary); }}
  .radio-row.selected {{
    background: rgba(59,130,246,0.10);
    color: var(--text);
  }}
  html.dark .radio-row.selected {{
    background: rgba(59,130,246,0.20);
  }}
  .radio-row-label {{ color: var(--text); font-size: 0.875rem; }}

  /* === Multiselect === */
  .multiselect {{
    display: flex;
    flex-direction: column;
    gap: 0.375rem;
  }}
  .multiselect-control {{
    min-height: 2.5rem;
    padding: 0.375rem;
    background: var(--bg-secondary);
    border: 1px solid var(--epsx-border);
    border-radius: 0.625rem;
  }}
  .multiselect-control:focus-within {{
    border-color: var(--epsx-primary);
    box-shadow: 0 0 0 3px rgba(59,130,246,0.15);
  }}
  .multiselect-chip {{
    display: inline-flex;
    align-items: center;
    gap: 0.25rem;
    padding: 0.125rem 0.5rem;
    border-radius: 9999px;
    background: rgba(59,130,246,0.15);
    color: #3b82f6;
    font-size: 0.75rem;
    font-weight: 600;
  }}
  .multiselect-chip-remove {{
    display: inline-flex;
    align-items: center;
    justify-content: center;
    width: 1rem;
    height: 1rem;
    padding: 0;
    margin-left: 0.125rem;
    background: transparent;
    border: none;
    border-radius: 9999px;
    color: inherit;
    cursor: pointer;
    font-size: 0.875rem;
    line-height: 1;
  }}
  .multiselect-chip-remove:hover {{ background: rgba(59,130,246,0.25); }}
  .multiselect-trigger {{
    background: transparent;
    border: 1px dashed var(--epsx-border-strong);
    border-radius: 9999px;
    padding: 0.125rem 0.625rem;
    color: var(--text-muted);
    font-size: 0.75rem;
    cursor: pointer;
  }}
  .multiselect-trigger:hover {{ background: var(--bg-secondary); color: var(--text); }}
  .multiselect-trigger:disabled {{ opacity: 0.5; cursor: not-allowed; }}
  .multiselect-menu {{
    position: absolute;
    z-index: 50;
    margin-top: 0.25rem;
    max-height: 16rem;
    overflow: auto;
    background: var(--surface-solid);
    border: 1px solid var(--epsx-border);
    border-radius: 0.625rem;
    box-shadow: var(--shadow-lg);
    padding: 0.25rem;
    min-width: 12rem;
    list-style: none;
  }}
  .multiselect-option {{
    padding: 0.375rem 0.625rem;
    border-radius: 0.375rem;
    font-size: 0.875rem;
    color: var(--text);
    cursor: pointer;
  }}
  .multiselect-option:hover {{ background: var(--bg-secondary); }}
  .multiselect-option.selected {{
    background: rgba(59,130,246,0.15);
    color: #3b82f6;
  }}

  /* === Combobox variants === */
  .combobox-async .combobox-menu {{
    min-width: 12rem;
  }}
  .combobox-loading,
  .combobox-empty {{
    padding: 0.5rem 0.75rem;
    font-size: 0.875rem;
    color: var(--text-muted);
    list-style: none;
  }}
  .combobox-multi {{
    display: flex;
    flex-direction: column;
    gap: 0.375rem;
  }}
  .combobox-multi-control {{
    min-height: 2.5rem;
    padding: 0.375rem;
    background: var(--bg-secondary);
    border: 1px solid var(--epsx-border);
    border-radius: 0.625rem;
  }}
  .combobox-multi-control:focus-within {{
    border-color: var(--epsx-primary);
    box-shadow: 0 0 0 3px rgba(59,130,246,0.15);
  }}
  .combobox-multi-chip {{
    display: inline-flex;
    align-items: center;
    gap: 0.25rem;
    padding: 0.125rem 0.5rem;
    border-radius: 9999px;
    background: rgba(59,130,246,0.15);
    color: #3b82f6;
    font-size: 0.75rem;
    font-weight: 600;
  }}
  .combobox-multi-chip-remove {{
    display: inline-flex;
    align-items: center;
    justify-content: center;
    width: 1rem;
    height: 1rem;
    padding: 0;
    margin-left: 0.125rem;
    background: transparent;
    border: none;
    border-radius: 9999px;
    color: inherit;
    cursor: pointer;
    font-size: 0.875rem;
    line-height: 1;
  }}
  .combobox-multi-chip-remove:hover {{ background: rgba(59,130,246,0.25); }}
  .combobox-multi-input {{
    background: transparent;
    border: none;
    outline: none;
    color: var(--text);
    font-size: 0.875rem;
    flex: 1;
    min-width: 8rem;
  }}

  /* === DateTimePicker === */
  .datetime-picker {{
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
  }}
  .datetime-picker .input {{ min-width: 0; }}

  /* === Stepper (progress bar variant + per-step icons) === */
  .stepper-wrap {{
    display: flex;
    flex-direction: column;
    gap: 0.75rem;
    width: 100%;
  }}
  .stepper-progress {{
    width: 100%;
    height: 0.25rem;
    background: var(--bg-tertiary);
    border-radius: 9999px;
    overflow: hidden;
  }}
  .stepper-progress .progress-bar {{
    height: 100%;
    background: var(--gradient-brand);
    border-radius: 9999px;
    transition: width 0.3s ease;
  }}
  .step-circle.flex.items-center.justify-center {{
    display: inline-flex;
    align-items: center;
    justify-content: center;
  }}

  /* === Rating interactive / disabled === */
  .rating-interactive .rating-star {{
    cursor: pointer;
    transition: transform 0.15s ease, color 0.15s ease;
  }}
  .rating-interactive .rating-star:hover {{
    transform: scale(1.1);
  }}
  .rating-interactive .rating-star:focus {{
    outline: 2px solid var(--epsx-primary);
    outline-offset: 2px;
    border-radius: 0.25rem;
  }}
  .rating-disabled {{
    opacity: 0.6;
    cursor: not-allowed;
  }}

  /* === Switch size variants + states === */
  .SwitchRoot {{
    display: inline-flex;
    align-items: center;
    gap: 0.5rem;
    cursor: pointer;
    user-select: none;
  }}
  .SwitchInput {{
    appearance: none;
    -webkit-appearance: none;
    background: var(--bg-tertiary);
    border-radius: 9999px;
    position: relative;
    cursor: pointer;
    transition: background-color 0.2s ease;
    flex-shrink: 0;
  }}
  .SwitchInput:checked {{ background: var(--epsx-primary); }}
  .SwitchInput:disabled {{ opacity: 0.5; cursor: not-allowed; }}
  .SwitchInput::after {{
    content: '';
    position: absolute;
    top: 2px;
    left: 2px;
    background: white;
    border-radius: 9999px;
    transition: transform 0.2s ease;
  }}
  .SwitchInput:checked::after {{ transform: translateX(100%); }}
  .SwitchThumb {{ display: none; }}
  .switch-sm .SwitchInput {{ width: 2rem; height: 1.125rem; }}
  .switch-sm .SwitchInput::after {{ width: calc(1.125rem - 4px); height: calc(1.125rem - 4px); }}
  .switch-md .SwitchInput {{ width: 2.5rem; height: 1.375rem; }}
  .switch-md .SwitchInput::after {{ width: calc(1.375rem - 4px); height: calc(1.375rem - 4px); }}
  .switch-lg .SwitchInput {{ width: 3rem; height: 1.625rem; }}
  .switch-lg .SwitchInput::after {{ width: calc(1.625rem - 4px); height: calc(1.625rem - 4px); }}
  .SwitchLabel {{ font-size: 0.875rem; color: var(--text); }}
  .state-checked {{ /* presentational hook for future styling */ }}
  .state-unchecked {{ /* presentational hook for future styling */ }}

  /* === Kbd combo (multi-key shortcut) === */
  .kbd-combo {{
    display: inline-flex;
    align-items: center;
    gap: 0.25rem;
  }}
  .kbd-combo-sep {{
    font-size: 0.75rem;
    color: var(--text-muted);
  }}
  .kbd {{
    display: inline-flex;
    align-items: center;
    justify-content: center;
    min-width: 1.5rem;
    height: 1.5rem;
    padding: 0 0.375rem;
    font-family: ui-monospace, SFMono-Regular, "SF Mono", Menlo, monospace;
    font-size: 0.75rem;
    font-weight: 600;
    color: var(--text);
    background: var(--bg-secondary);
    border: 1px solid var(--epsx-border-strong);
    border-bottom-width: 2px;
    border-radius: 0.375rem;
  }}

  /* === Slider (a11y + visual) === */
  .slider-field {{
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
  }}
  .slider {{
    width: 100%;
    height: 0.375rem;
    appearance: none;
    -webkit-appearance: none;
    background: var(--bg-tertiary);
    border-radius: 9999px;
    cursor: pointer;
  }}
  .slider::-webkit-slider-thumb {{
    appearance: none;
    -webkit-appearance: none;
    width: 1.125rem;
    height: 1.125rem;
    border-radius: 9999px;
    background: var(--epsx-primary);
    border: 2px solid var(--surface-solid);
    box-shadow: var(--shadow);
  }}
  .slider::-moz-range-thumb {{
    width: 1.125rem;
    height: 1.125rem;
    border-radius: 9999px;
    background: var(--epsx-primary);
    border: 2px solid var(--surface-solid);
    box-shadow: var(--shadow);
  }}
  .slider:focus {{
    outline: 2px solid var(--epsx-primary);
    outline-offset: 2px;
  }}
  .slider:disabled {{ opacity: 0.5; cursor: not-allowed; }}

  /* === Checkbox indeterminate (visual) === */
  .checkbox-indeterminate {{
    background: var(--epsx-primary);
    position: relative;
    color: white;
  }}
  .checkbox-indeterminate::after {{
    content: '';
    position: absolute;
    left: 50%;
    top: 50%;
    width: 60%;
    height: 2px;
    background: currentColor;
    transform: translate(-50%, -50%);
    border-radius: 1px;
  }}
  .modal-close {{
    display: inline-flex;
    align-items: center;
    justify-content: center;
    width: 2rem;
    height: 2rem;
    border: 0;
    background: transparent;
    color: var(--text-muted);
    border-radius: 0.375rem;
    cursor: pointer;
    font-size: 1.125rem;
    line-height: 1;
  }}
  .modal-close:hover {{
    background: var(--bg-secondary);
    color: var(--text);
  }}
  .modal-description {{
    color: var(--text-muted);
  }}
  .modal-body {{
    padding: 1.25rem 1.5rem;
  }}
  .modal-footer {{
    padding: 1rem 1.5rem;
    border-top: 1px solid var(--epsx-border);
    background: var(--bg-secondary);
    border-bottom-left-radius: inherit;
    border-bottom-right-radius: inherit;
  }}
  .modal-sm   {{ max-width: 24rem; }}
  .modal-lg   {{ max-width: 48rem; }}
  .modal-xl   {{ max-width: 64rem; }}
  .modal-full {{ max-width: 95vw; max-height: 95vh; width: 100%; }}

  /* === Tabs === */
  .tabs {{
    display: inline-flex;
    align-items: center;
    gap: 0.25rem;
    border-bottom: 1px solid var(--epsx-border);
  }}
  .tab {{
    display: inline-flex;
    align-items: center;
    gap: 0.375rem;
    padding: 0.5rem 0.875rem;
    font-size: 0.875rem;
    font-weight: 500;
    color: var(--text-muted);
    background: transparent;
    border: 0;
    border-bottom: 2px solid transparent;
    cursor: pointer;
    transition: color 0.15s, border-color 0.15s;
  }}
  .tab:hover {{ color: var(--text); }}
  .tab-active {{
    color: var(--epsx-orange);
    border-bottom-color: var(--epsx-orange);
  }}
  .tab-icon {{
    display: inline-flex;
    align-items: center;
    color: var(--text-muted);
  }}
  .tabs-vertical {{
    flex-direction: column;
    align-items: stretch;
    border-bottom: 0;
    border-right: 1px solid var(--epsx-border);
  }}
  .tabs-vertical .tab {{
    border-bottom: 0;
    border-right: 2px solid transparent;
    justify-content: flex-start;
  }}
  .tabs-vertical .tab.tab-active {{
    border-right-color: var(--epsx-orange);
  }}

  /* === Tooltip: hover/focus reveal + side/align modifiers === */
  .tooltip-wrapper {{
    position: relative;
    display: inline-flex;
  }}
  .tooltip-content {{
    position: absolute;
    z-index: 1100;
    background: var(--text);
    color: var(--surface-solid);
    padding: 0.375rem 0.625rem;
    border-radius: 0.375rem;
    font-size: 0.75rem;
    line-height: 1.2;
    white-space: nowrap;
    pointer-events: none;
    opacity: 0;
    transform: translateY(2px);
    transition: opacity 0.15s ease, transform 0.15s ease;
    transition-delay: var(--tooltip-delay, 0ms);
    box-shadow: var(--shadow-lg);
  }}
  .tooltip-wrapper:hover .tooltip-content,
  .tooltip-wrapper:focus-within .tooltip-content,
  .tooltip-content.tooltip-open {{
    opacity: 1;
    transform: translateY(0);
  }}
  .tooltip-side-top    {{ bottom: 100%; left: 50%; transform: translate(-50%, -2px); margin-bottom: 0.375rem; }}
  .tooltip-side-bottom {{ top: 100%; left: 50%; transform: translate(-50%, 2px); margin-top: 0.375rem; }}
  .tooltip-side-left   {{ right: 100%; top: 50%; transform: translate(-2px, -50%); margin-right: 0.375rem; }}
  .tooltip-side-right  {{ left: 100%; top: 50%; transform: translate(2px, -50%); margin-left: 0.375rem; }}
  .tooltip-align-start  {{ left: 0; transform: translateX(0); }}
  .tooltip-align-end    {{ left: auto; right: 0; transform: translateX(0); }}
  .tooltip-align-center {{ left: 50%; transform: translateX(-50%); }}
  .tooltip-side-top.tooltip-align-start,
  .tooltip-side-bottom.tooltip-align-start {{
    left: 0; transform: translateX(0);
  }}
  .tooltip-side-top.tooltip-align-end,
  .tooltip-side-bottom.tooltip-align-end {{
    left: auto; right: 0; transform: translateX(0);
  }}
  .tooltip-side-left.tooltip-align-start,
  .tooltip-side-right.tooltip-align-start {{
    top: 0; transform: translateY(0);
  }}
  .tooltip-side-left.tooltip-align-end,
  .tooltip-side-right.tooltip-align-end {{
    top: auto; bottom: 0; transform: translateY(0);
  }}

  /* === Popover === */
  .popover {{
    position: relative;
    display: inline-block;
  }}
  .popover-content {{
    position: absolute;
    z-index: 900;
    min-width: 12rem;
    max-width: 24rem;
    background: var(--surface-solid);
    border: 1px solid var(--epsx-border);
    border-radius: 0.5rem;
    box-shadow: var(--shadow-lg);
    padding: 0.75rem;
    animation: fadeIn 0.12s ease-out;
  }}
  .popover-content-side-top    {{ bottom: 100%; margin-bottom: 0.375rem; }}
  .popover-content-side-bottom {{ top: 100%; margin-top: 0.375rem; }}
  .popover-content-side-left   {{ right: 100%; margin-right: 0.375rem; top: 0; }}
  .popover-content-side-right  {{ left: 100%; margin-left: 0.375rem; top: 0; }}
  .popover-content-align-start  {{ left: 0; }}
  .popover-content-align-end    {{ right: 0; }}
  .popover-content-align-center {{ left: 50%; transform: translateX(-50%); }}

  /* === HoverCard === */
  .hover-card {{
    position: relative;
    display: inline-block;
  }}
  .hover-card-content {{
    position: absolute;
    z-index: 850;
    top: 100%;
    left: 50%;
    transform: translate(-50%, 0.25rem);
    min-width: 16rem;
    max-width: 24rem;
    background: var(--surface-solid);
    border: 1px solid var(--epsx-border);
    border-radius: 0.5rem;
    box-shadow: var(--shadow-lg);
    padding: 0.75rem;
    opacity: 0;
    pointer-events: none;
    transition: opacity 0.15s ease, transform 0.15s ease;
    transition-delay: var(--hover-card-open-delay, 200ms);
  }}
  .hover-card:hover .hover-card-content,
  .hover-card:focus-within .hover-card-content,
  .hover-card-content[data-visible="true"] {{
    opacity: 1;
    pointer-events: auto;
    transform: translate(-50%, 0.5rem);
    transition-delay: 0ms;
  }}

  /* === Accordion === */
  .accordion {{
    display: flex;
    flex-direction: column;
    border: 1px solid var(--epsx-border);
    border-radius: 0.5rem;
    overflow: hidden;
  }}
  .accordion-item {{
    border-bottom: 1px solid var(--epsx-border);
  }}
  .accordion-item:last-child {{
    border-bottom: 0;
  }}
  .accordion-trigger {{
    display: flex;
    align-items: center;
    justify-content: space-between;
    width: 100%;
    padding: 0.75rem 1rem;
    background: var(--surface-solid);
    color: var(--text);
    border: 0;
    text-align: left;
    font-size: 0.875rem;
    font-weight: 500;
    cursor: pointer;
    transition: background 0.12s;
  }}
  .accordion-trigger:hover {{
    background: var(--bg-secondary);
  }}
  .accordion-icon {{
    color: var(--text-muted);
    font-size: 1rem;
    line-height: 1;
  }}
  .accordion-content {{
    padding: 0.75rem 1rem 1rem;
    color: var(--text);
    font-size: 0.875rem;
  }}
  .accordion-item.open .accordion-trigger {{
    background: var(--bg-secondary);
  }}

  /* === Collapsible === */
  .collapsible {{
    display: flex;
    flex-direction: column;
  }}
  .collapsible-trigger {{
    cursor: pointer;
  }}
  .collapsible-content {{
    padding: 0.5rem 0 0;
  }}

  /* === Command palette === */
  .command-palette-overlay {{
    position: fixed;
    inset: 0;
    background: rgba(15, 23, 42, 0.55);
    backdrop-filter: blur(4px);
    -webkit-backdrop-filter: blur(4px);
    z-index: 1200;
    display: flex;
    align-items: flex-start;
    justify-content: center;
    padding-top: 12vh;
    padding-left: 1rem;
    padding-right: 1rem;
    animation: fadeIn 0.12s ease-out;
  }}
  .command-palette {{
    width: 100%;
    max-width: 36rem;
    background: var(--surface-solid);
    border: 1px solid var(--epsx-border);
    border-radius: 0.75rem;
    box-shadow: var(--shadow-xl);
    overflow: hidden;
    display: flex;
    flex-direction: column;
    max-height: 70vh;
  }}
  .command-input {{
    width: 100%;
    padding: 1rem 1.25rem;
    font-size: 0.95rem;
    color: var(--text);
    background: var(--surface-solid);
    border: 0;
    border-bottom: 1px solid var(--epsx-border);
    outline: none;
  }}
  .command-input:focus {{
    border-bottom-color: var(--epsx-orange);
  }}
  .command-list {{
    flex: 1 1 auto;
    overflow-y: auto;
    padding: 0.375rem;
  }}
  .command-item {{
    display: flex;
    align-items: center;
    gap: 0.5rem;
    padding: 0.5rem 0.75rem;
    font-size: 0.875rem;
    color: var(--text);
    border-radius: 0.375rem;
    cursor: pointer;
    text-decoration: none;
  }}
  .command-item:hover,
  .command-item.active {{
    background: var(--bg-secondary);
    color: var(--text);
  }}
  .command-empty {{
    font-size: 0.875rem;
  }}
  .command-hint {{
    font-size: 0.75rem;
    color: var(--text-muted);
    font-family: ui-monospace, SFMono-Regular, "SF Mono", Menlo, monospace;
  }}

  /* === Rich text editor === */
  .rich-text-editor {{
    display: flex;
    flex-direction: column;
  }}
  .rte-toolbar {{
    display: flex;
    align-items: center;
    flex-wrap: wrap;
  }}
  .rte-preview {{
    background: var(--bg);
    color: var(--text);
    line-height: 1.6;
  }}
  .rte-preview h1 {{ font-size: 1.75rem; font-weight: 700; margin: 1rem 0 0.5rem; }}
  .rte-preview h2 {{ font-size: 1.5rem;  font-weight: 700; margin: 1rem 0 0.5rem; }}
  .rte-preview h3 {{ font-size: 1.25rem; font-weight: 600; margin: 0.75rem 0 0.5rem; }}
  .rte-preview p  {{ margin: 0 0 0.75rem; }}
  .rte-preview ul {{ margin: 0 0 0.75rem; padding-left: 1.5rem; list-style: disc; }}
  .rte-preview li {{ margin: 0.25rem 0; }}
  .rte-preview code {{
    font-family: ui-monospace, SFMono-Regular, "SF Mono", Menlo, monospace;
    background: var(--bg-tertiary);
    padding: 0.125rem 0.375rem;
    border-radius: 0.25rem;
    font-size: 0.85em;
  }}
  .rte-preview a {{
    color: var(--epsx-primary);
    text-decoration: underline;
  }}

  /* wave2-chrome-track-a — admin shell chrome.
     Adds the design-token-backed classes used by the new
     `AdminSidebar`, `Header`, `Breadcrumb`, `MainLayout`, and
     `AdminFooter` components in
     `shared/rust/dioxus_ui/src/layout/`. All rules are additive —
     no existing class is restyled. */
  .admin-shell {{
    display: flex;
    height: 100vh;
    width: 100%;
    overflow: hidden;
    background: var(--bg);
  }}
  .admin-main {{
    flex: 1 1 auto;
    display: flex;
    flex-direction: column;
    min-width: 0;
    height: 100%;
    overflow: hidden;
  }}
  .admin-header {{
    position: sticky;
    top: 0;
    z-index: 40;
    display: flex;
    height: 4rem;
    align-items: center;
    justify-content: space-between;
    padding-left: 1.5rem;
    padding-right: 1.5rem;
    gap: 0.75rem;
    border-bottom: 1px solid var(--epsx-border);
    background: var(--surface-solid);
  }}
  .admin-header-chrome {{
    padding-left: 0;
    padding-right: 0;
    background: hsl(var(--card));
  }}
  .admin-header-left {{ min-width: 0; flex: 1 1 auto; display: flex; align-items: center; gap: 0.5rem; }}
  .admin-header-right {{ display: flex; align-items: center; gap: 0.5rem; flex-shrink: 0; }}
  .admin-page-title {{ font-size: 1.125rem; font-weight: 600; color: var(--text); margin: 0; }}
  .admin-user-badge {{
    font-family: ui-monospace, SFMono-Regular, "SF Mono", Menlo, monospace;
    font-size: 0.75rem;
    color: var(--text-muted);
    background: var(--bg-tertiary);
    padding: 0.25rem 0.5rem;
    border-radius: 0.375rem;
  }}
  .admin-content {{
    flex: 1 1 auto;
    overflow-y: auto;
    overflow-x: hidden;
    padding: 0;
    min-height: 0;
  }}
  .admin-sidebar-cta {{
    text-decoration: none;
  }}
  html:not(.dark) .admin-sidebar-cta {{ background: #0e7490 !important; }}
  .admin-nav-row {{
    text-decoration: none;
  }}
  .admin-nav-row-active {{
    font-weight: 600;
  }}
  html:not(.dark) .admin-nav-row:not(.admin-nav-row-active) {{ color: #475569; }}
  html:not(.dark) .admin-nav-row-active,
  html:not(.dark) .admin-nav-children a[aria-current="page"] {{ color: #0e7490 !important; }}
  html:not(.dark) aside[aria-label="Sidebar"] .text-\[\#FF512F\] {{ color: #c2410c !important; }}
  .admin-nav-row[aria-expanded="true"] .admin-nav-chevron {{
    transform: rotate(90deg);
  }}
  .admin-nav-children[hidden] {{
    display: none !important;
  }}
  .admin-footer {{
    flex-shrink: 0;
  }}
  .admin-header-bell {{
    position: relative;
  }}
  .admin-header-theme-toggle {{
    color: #f97316;
  }}
  .admin-theme-icon {{ display: inline-flex; align-items: center; justify-content: center; }}
  .admin-theme-icon-sun {{ display: none; }}
  html.dark .admin-theme-icon-sun {{ display: inline-flex; }}
  html.dark .admin-theme-icon-moon {{ display: none; }}
  .admin-header-menu-wrap {{ position: relative; }}
  .admin-header-popover {{
    position: absolute;
    top: calc(100% + 0.625rem);
    right: 0;
    z-index: 99999;
    overflow: hidden;
    border: 1px solid var(--epsx-border);
    border-radius: 0.75rem;
    background: var(--surface-solid);
    color: var(--text);
    box-shadow: 0 18px 48px -16px rgba(0, 0, 0, 0.55);
    transform-origin: top right;
  }}
  .admin-header-popover[hidden] {{ display: none !important; }}
  .admin-header-popover.open {{ animation: scaleIn 0.15s ease; }}
  .admin-header-bell-icon {{ color: #f97316; }}
  .admin-notifications-menu {{ width: min(22rem, calc(100vw - 2rem)); }}
  .admin-notifications-heading {{
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 0.75rem;
    padding: 0.75rem 1rem;
    border-bottom: 1px solid var(--epsx-border);
  }}
  .admin-notifications-heading h2 {{ margin: 0; font-size: 0.875rem; font-weight: 600; }}
  .admin-notifications-heading span {{ font-size: 0.75rem; color: #f97316; }}
  .admin-notifications-empty {{
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 0.25rem;
    padding: 2rem 1rem;
    text-align: center;
    color: var(--text-muted);
  }}
  .admin-notifications-empty p {{ margin: 0.5rem 0 0; color: var(--text); font-size: 0.875rem; font-weight: 600; }}
  .admin-notifications-empty span {{ font-size: 0.75rem; }}
  .admin-notifications-list {{ max-height: 24rem; overflow-y: auto; }}
  .admin-notification-item {{
    display: flex;
    flex-direction: column;
    gap: 0.25rem;
    padding: 0.75rem 1rem;
    border-bottom: 1px solid var(--epsx-border);
    color: var(--text);
    text-decoration: none;
  }}
  .admin-notification-item:hover {{ background: var(--bg-tertiary); }}
  .admin-notification-item-unread {{ box-shadow: inset 3px 0 #f97316; }}
  .admin-notification-item strong {{ font-size: 0.8125rem; }}
  .admin-notification-item span {{ font-size: 0.75rem; color: var(--text-muted); }}
  .admin-notifications-view-all {{
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 0.5rem;
    padding: 0.75rem 1rem;
    color: var(--text-muted);
    font-size: 0.8125rem;
    font-weight: 600;
    text-decoration: none;
  }}
  .admin-notifications-view-all:hover {{ background: var(--bg-tertiary); color: var(--text); }}
  .admin-wallet-connect {{
    display: inline-flex;
    align-items: center;
    gap: 0.5rem;
    padding: 0.5rem 1.25rem;
    border-radius: 1rem;
    background: #1fc7d4;
    color: #fff;
    font-size: 0.875rem;
    font-weight: 700;
    text-decoration: none;
    box-shadow: 0 10px 24px -12px rgba(31, 199, 212, 0.55);
    transition: transform 0.2s ease, box-shadow 0.2s ease;
  }}
  .admin-wallet-connect:hover {{ transform: scale(1.02); box-shadow: 0 12px 28px -12px rgba(31, 199, 212, 0.75); }}
  .admin-wallet-trigger {{
    display: inline-flex;
    align-items: center;
    gap: 0.5rem;
    min-height: 2.25rem;
    padding: 0.5rem 0.625rem;
    border: 0;
    border-radius: 0.75rem;
    background: transparent;
    color: var(--text);
    font-size: 0.875rem;
    font-weight: 600;
    cursor: pointer;
    transition: background 0.15s ease;
  }}
  .admin-wallet-trigger:hover,
  .admin-wallet-trigger[aria-expanded="true"] {{ background: var(--bg-tertiary); }}
  .admin-wallet-trigger[aria-expanded="true"] .admin-wallet-chevron {{ transform: rotate(180deg); }}
  .admin-wallet-trigger > .epsx-icon:first-child {{ color: var(--text); }}
  .admin-wallet-short-address,
  .admin-wallet-connect-label {{ display: inline; white-space: nowrap; }}
  .admin-wallet-chevron {{ transition: transform 0.2s ease; }}
  .admin-wallet-menu {{ width: 16rem; }}
  .admin-wallet-accent {{ height: 3px; background: linear-gradient(90deg, #ffb237, #f97316); }}
  .admin-wallet-address-block {{ padding: 0.625rem 0.75rem; border-bottom: 1px solid var(--epsx-border); }}
  .admin-wallet-label {{
    display: flex;
    align-items: center;
    gap: 0.375rem;
    margin-bottom: 0.25rem;
    color: #f97316;
    font-size: 0.625rem;
    font-weight: 700;
    letter-spacing: 0.05em;
    text-transform: uppercase;
  }}
  .admin-wallet-address-block p {{
    margin: 0;
    overflow-wrap: anywhere;
    color: var(--text);
    font-family: ui-monospace, SFMono-Regular, Menlo, monospace;
    font-size: 0.75rem;
    line-height: 1.4;
  }}
  .admin-wallet-menu .wallet-meta-grid {{
    padding: 0.625rem 0.75rem;
    border-top: 0;
    background: transparent;
  }}
  .admin-wallet-menu .wallet-meta-label {{ color: var(--text-muted); }}
  .admin-wallet-menu .wallet-meta-value {{ color: var(--text); }}
  .admin-wallet-menu .wallet-meta-value-role {{ color: #a78bfa; }}
  .admin-wallet-menu .wallet-meta-value-tier {{ color: #0891b2; }}
  .admin-wallet-menu .wallet-network-badge {{
    border-top-color: var(--epsx-border);
    background: transparent;
    color: var(--text-muted);
  }}
  .admin-wallet-actions {{ padding: 0.25rem; }}
  .admin-wallet-menu-item {{
    display: flex;
    width: 100%;
    align-items: center;
    gap: 0.5rem;
    padding: 0.5rem 0.75rem;
    border: 0;
    border-radius: 0.375rem;
    background: transparent;
    color: var(--text);
    font: inherit;
    font-size: 0.8125rem;
    text-align: left;
    text-decoration: none;
    cursor: pointer;
  }}
  .admin-wallet-menu-item:hover {{ background: var(--bg-tertiary); }}
  .admin-wallet-menu-separator {{ height: 1px; margin: 0.25rem 0; background: var(--epsx-border); }}
  .admin-wallet-disconnect {{ color: #ef4444; }}
  @media (max-width: 1023px) {{
    .admin-wallet-short-address,
    .admin-wallet-connect-label {{ display: none; }}
  }}
  .admin-wallet-disconnect:hover {{ background: rgba(239, 68, 68, 0.1); }}
  .admin-app-shell .btn-primary {{
    border-color: transparent;
    background: linear-gradient(90deg, #7645d9, #5a33b8);
    color: #fff;
  }}
  .admin-app-shell .btn-primary:hover {{
    background: linear-gradient(90deg, #8455de, #6740c2);
  }}
  @media (max-width: 639px) {{
    .admin-notifications-menu {{ position: fixed; top: 4.5rem; right: 1rem; }}
  }}
  .developer-shell {{
    display: flex;
    height: 100vh;
    width: 100%;
    overflow: hidden;
    background: var(--bg);
  }}
  .developer-main {{
    flex: 1 1 auto;
    overflow-y: auto;
    padding: 1.5rem;
    min-width: 0;
  }}

  /* === wave2-chrome-track-b === frontend nav cluster (NavigationClient,
     DesktopNav, MobileNav, NavActions, NavbarSkeleton, NavGroup data). */
  /* Mobile nav group accordion */
  .mobile-nav-group {{ display: block; }}
  .mobile-nav-group-trigger {{
    width: 100%;
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 0.625rem 0.75rem;
    border-radius: 0.375rem;
    font-size: 0.875rem;
    font-weight: 500;
    color: var(--text-muted);
    background: transparent;
    border: none;
    cursor: pointer;
    text-align: left;
    transition: color 0.15s ease, background 0.15s ease;
  }}
  .mobile-nav-group-trigger:hover {{ color: var(--text); }}
  .mobile-nav-group-trigger.active {{ color: var(--text); }}
  html.dark .mobile-nav-group-trigger {{ color: #94a3b8; }}
  html.dark .mobile-nav-group-trigger:hover {{ color: white; }}
  .mobile-nav-group-trigger .chev {{
    transition: transform 0.2s ease;
  }}
  .mobile-nav-group-trigger .chev.rotate-90 {{ transform: rotate(90deg); }}

  /* Sign-in banner (purple→teal gradient; matches epsx.io CTA strip) */
  .signin-banner {{
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 0.75rem;
    padding: 0.75rem 1.5rem;
    font-size: 1rem;
    color: #fff;
    background: linear-gradient(90deg, #5a33b8 0%, #7645d9 50%, #1a9bab 100%);
    box-shadow: 0 4px 16px rgba(0, 0, 0, 0.18);
  }}
  html.dark .signin-banner {{
    background: linear-gradient(90deg, rgba(118,69,217,0.9) 0%, #5a33b8 50%, rgba(31,199,212,0.8) 100%);
  }}
  .signin-banner-cta {{
    border-radius: 0.375rem;
    background: rgba(255, 255, 255, 0.2);
    padding: 0.25rem 1rem;
    font-weight: 700;
    color: #fff;
    border: none;
    cursor: pointer;
    transition: background 0.15s ease;
  }}
  .signin-banner-cta:hover {{ background: rgba(255, 255, 255, 0.3); }}

  /* ============================================================
   * wave2-chrome-track-c
   * ------------------------------------------------------------
   * CSS for the Wave 2 Track C auth cluster
   * (auth_modal, auth_gate, access_denied, progressive_banner,
   * user, wallet_button). The Track C rust components emit
   * markup using these class names; keep additions in this
   * block so the integration step can merge Tracks A/B/C
   * cleanly without manual conflict resolution.
   * ============================================================ */

  /* --- auth modal (focus trap, role=dialog, gradient) --- */
  .auth-modal {{
    background: var(--surface-solid, #191923);
    color: var(--text);
    border: 1px solid var(--epsx-border);
    border-radius: 1.5rem;
    box-shadow: 0 0 50px -12px rgba(0,0,0,0.5);
    max-width: 56rem;
    width: 100%;
    overflow: hidden;
    animation: scaleIn 0.2s ease;
    isolation: isolate;
  }}
  .auth-modal-grid {{
    display: grid;
    grid-template-columns: 1fr;
  }}
  @media (min-width: 1024px) {{
    .auth-modal-grid {{ grid-template-columns: 3fr 2fr; }}
  }}
  .auth-modal-aside {{
    display: none;
    padding: 3rem 2.5rem;
    background:
      radial-gradient(at top left, rgba(118,69,217,0.18), transparent 60%),
      radial-gradient(at bottom right, rgba(31,199,212,0.18), transparent 60%);
    border-right: 1px solid var(--epsx-border);
  }}
  @media (min-width: 1024px) {{
    .auth-modal-aside {{ display: flex; flex-direction: column; justify-content: center; }}
  }}
  .auth-modal-brand {{
    display: flex; align-items: center; gap: 0.75rem;
    margin-bottom: 2rem;
  }}
  .auth-modal-headline {{
    font-size: 2.25rem; font-weight: 800; line-height: 1.1;
    margin: 0 0 1rem 0;
  }}
  .auth-modal-sub {{ color: rgba(255,255,255,0.65); margin: 0 0 1.5rem 0; line-height: 1.5; }}
  .auth-modal-features {{ list-style: none; padding: 0; margin: 0; display: flex; flex-direction: column; gap: 0.75rem; }}
  .auth-modal-features li {{ display: flex; align-items: center; gap: 0.5rem; color: rgba(255,255,255,0.85); }}
  .auth-modal-content {{ padding: 2rem; position: relative; background: rgba(255,255,255,0.02); }}
  .auth-modal-title {{ font-size: 1.25rem; font-weight: 600; margin: 0 0 0.5rem 0; color: #ffffff; }}
  .auth-modal-description {{ color: rgba(255,255,255,0.55); margin: 0 0 1rem 0; }}
  .auth-modal-divider {{
    margin: 1.5rem 0 1rem;
    text-align: center;
    color: rgba(255,255,255,0.4);
    font-size: 0.75rem; font-weight: 600;
    letter-spacing: 0.1em;
  }}
  .auth-demo-btn {{ margin-top: 0.25rem; }}

  /* --- wallet option row (button styled as card) --- */
  .wallet-list {{ display: flex; flex-direction: column; gap: 0.625rem; margin: 0.5rem 0 0 0; padding: 0; list-style: none; }}
  .wallet-option {{
    width: 100%;
    display: flex; align-items: center; gap: 0.875rem;
    padding: 1rem 1.25rem;
    background: rgba(255,255,255,0.05);
    border: 1px solid transparent;
    border-radius: 1rem;
    color: #ffffff;
    font-size: 1rem; font-weight: 500;
    cursor: pointer;
    transition: all 0.2s ease;
    text-align: left;
  }}
  .wallet-option:not(:disabled):hover {{
    background: rgba(255,255,255,0.10);
    border-color: rgba(139,92,246,0.4);
    transform: translateY(-1px);
  }}
  .wallet-option:disabled {{ opacity: 0.6; cursor: not-allowed; }}
  .wallet-icon {{
    display: inline-flex; align-items: center; justify-content: center;
    width: 1.75rem; height: 1.75rem;
    background: rgba(255,255,255,0.06);
    border-radius: 0.5rem;
  }}
  .wallet-name {{ flex: 1; }}
  .wallet-chev {{ opacity: 0.5; }}

  /* The modal surface follows the active theme. Its original copy and wallet
     rows were hard-coded white, which made the complete sign-in flow vanish
     when `--surface-solid` switched to white. */
  html:not(.dark) .auth-modal-headline,
  html:not(.dark) .auth-modal-title {{ color: #0f172a; }}
  html:not(.dark) .auth-modal-sub,
  html:not(.dark) .auth-modal-description {{ color: #475569; }}
  html:not(.dark) .auth-modal-features li {{ color: #334155; }}
  html:not(.dark) .auth-modal-content {{ background: rgba(248,250,252,0.8); }}
  html:not(.dark) .wallet-option {{
    background: #f1f5f9;
    border-color: #e2e8f0;
    color: #0f172a;
  }}
  html:not(.dark) .wallet-option:not(:disabled):hover {{
    background: #e2e8f0;
    border-color: #8b5cf6;
  }}
  html:not(.dark) .wallet-icon {{ background: #e2e8f0; }}

  /* --- auth gate (sign-in / permission-missing / admin variants) --- */
  .auth-gate {{
    display: flex; flex-direction: column; align-items: center; justify-content: center;
    text-align: center;
    padding: 3rem 1.5rem;
    max-width: 32rem; margin: 4rem auto;
    background: var(--surface-solid, #191923);
    border: 1px solid var(--epsx-border);
    border-radius: 1.5rem;
    box-shadow: 0 0 50px -12px rgba(0,0,0,0.5);
    color: var(--text);
  }}
  .auth-gate-icon {{
    width: 4rem; height: 4rem;
    display: inline-flex; align-items: center; justify-content: center;
    background: rgba(139,92,246,0.12);
    color: #8b5cf6;
    border-radius: 9999px;
    margin-bottom: 1.25rem;
  }}
  .auth-gate-title {{ font-size: 1.5rem; font-weight: 700; margin: 0 0 0.75rem 0; }}
  .auth-gate-description {{ color: var(--text-muted); margin: 0 0 1.5rem 0; line-height: 1.5; }}
  .auth-gate-perms {{
    text-align: left;
    background: var(--bg-secondary);
    border: 1px solid var(--epsx-border);
    border-radius: 0.75rem;
    padding: 0.875rem 1rem;
    margin: 0 0 1.5rem 0;
    width: 100%;
  }}
  .auth-gate-perms p {{ margin: 0 0 0.5rem 0; font-size: 0.75rem; color: var(--text-muted); text-transform: uppercase; letter-spacing: 0.05em; }}
  .auth-gate-perms ul {{ margin: 0; padding-left: 1.25rem; font-family: ui-monospace, SFMono-Regular, Menlo, monospace; font-size: 0.8125rem; color: var(--text); }}
  .auth-gate-actions {{ display: flex; gap: 0.75rem; flex-wrap: wrap; justify-content: center; }}
  .auth-gate-badge {{
    display: inline-block;
    background: linear-gradient(135deg, #7645d9, #1fc7d4);
    color: #ffffff;
    font-size: 0.6875rem; font-weight: 700;
    padding: 0.25rem 0.625rem;
    border-radius: 9999px;
    text-transform: uppercase; letter-spacing: 0.05em;
    margin-bottom: 0.75rem;
  }}
  .auth-gate-admin .auth-gate-icon {{ background: rgba(118,69,217,0.15); color: #a78bfa; }}
  .auth-gate-missing .auth-gate-icon {{ background: rgba(245,158,11,0.15); color: #fbbf24; }}
  html.dark .auth-gate {{ color: #ffffff; }}
  html.dark .auth-gate-description {{ color: rgba(255,255,255,0.65); }}
  html.dark .auth-gate-perms {{ background: rgba(255,255,255,0.04); }}
  html.dark .auth-gate-perms p {{ color: rgba(255,255,255,0.55); }}
  html.dark .auth-gate-perms ul {{ color: rgba(255,255,255,0.85); }}

  /* The wallet selector is intentionally a dark surface in either theme. Its
     semantic foreground utilities must therefore stay light in light mode. */
  .auth-modal-inner .text-foreground {{ color: #ffffff !important; }}
  .auth-modal-inner .text-muted-foreground {{ color: #cbd5e1 !important; }}
  .auth-modal-inner .auth-step-number {{ color: #c4b5fd; }}

  /* --- access denied (full page panel) --- */
  .access-denied {{
    display: flex; flex-direction: column; align-items: center; justify-content: center;
    text-align: center;
    padding: 4rem 1.5rem;
    max-width: 32rem; margin: 0 auto;
  }}
  .access-denied-icon {{
    display: inline-flex; align-items: center; justify-content: center;
    width: 5rem; height: 5rem;
    background: rgba(239,68,68,0.10);
    color: #ef4444;
    border-radius: 9999px;
    margin-bottom: 1.5rem;
  }}
  .access-denied-title {{ font-size: 1.75rem; line-height: 2rem; font-weight: 700; margin: 0 0 0.5rem 0; color: var(--text); }}
  .access-denied-reason {{ color: var(--text-muted); margin: 0 0 1.5rem 0; line-height: 1.5; }}
  .access-denied-perms {{
    background: var(--bg-muted, rgba(255,255,255,0.05));
    border-radius: 0.75rem;
    padding: 1rem;
    margin: 0 0 1.5rem 0;
    width: 100%;
    text-align: left;
  }}
  .access-denied-perms p {{ margin: 0 0 0.5rem 0; font-size: 0.875rem; color: var(--text-muted); }}
  .access-denied-perms ul {{ margin: 0; padding-left: 1rem; display: flex; flex-direction: column; gap: 0.375rem; }}
  .access-denied-perm {{
    font-family: ui-monospace, SFMono-Regular, Menlo, monospace;
    font-size: 0.75rem;
    background: var(--bg, #ffffff);
    color: var(--text);
    padding: 0.375rem 0.5rem;
    border-radius: 0.375rem;
  }}
  .access-denied-actions {{ display: flex; gap: 0.75rem; flex-wrap: wrap; justify-content: center; }}
  .access-denied-actions > a {{
    height: 2.25rem;
    padding-top: 0.5rem;
    padding-bottom: 0.5rem;
    border-radius: 0.375rem;
    font-weight: 500;
  }}
  .access-denied-actions > .btn-outline {{
    border-color: var(--text);
  }}
  .access-denied-reason {{ color: var(--text); }}
  html.dark .access-denied-page {{ background: #121212; }}
  @media (max-width: 639px) {{
    .access-denied-page > .access-denied {{ max-width: 28rem; padding-left: 1rem; padding-right: 1rem; }}
    .access-denied-page .access-denied-icon {{ width: 6rem; height: 6rem; }}
    .access-denied-page .access-denied-title {{ font-size: 1.5rem; }}
    .access-denied-page .access-denied-actions {{ flex-direction: column; width: 100%; }}
    .access-denied-page .access-denied-actions > a {{ width: 100%; }}
  }}

  /* --- progressive auth banner (inline strip) --- */
  .progressive-auth-banner {{
    display: flex; align-items: center; gap: 0.875rem;
    padding: 0.875rem 1rem;
    background: linear-gradient(90deg, rgba(139,92,246,0.08), rgba(31,199,212,0.08));
    border: 1px solid rgba(139,92,246,0.20);
    border-radius: 0.75rem;
    color: var(--text);
  }}
  .progressive-auth-banner .banner-icon {{
    display: inline-flex; align-items: center; justify-content: center;
    width: 2rem; height: 2rem;
    background: rgba(139,92,246,0.12);
    color: #8b5cf6;
    border-radius: 9999px;
    flex-shrink: 0;
  }}
  .progressive-auth-banner .banner-content {{ flex: 1; min-width: 0; }}
  .progressive-auth-banner .banner-title {{ font-size: 0.875rem; font-weight: 600; margin: 0; color: var(--text); }}
  .progressive-auth-banner .banner-subtitle {{ margin: 0.125rem 0 0 0; color: var(--text-muted); }}
  .progressive-auth-banner-dismiss {{
    display: inline-flex; align-items: center; justify-content: center;
    width: 1.75rem; height: 1.75rem;
    background: transparent;
    border: 1px solid var(--epsx-border);
    border-radius: 9999px;
    color: var(--text-muted);
    cursor: pointer;
    flex-shrink: 0;
    transition: all 0.15s ease;
  }}
  .progressive-auth-banner-dismiss:hover {{ background: var(--bg-muted, rgba(255,255,255,0.05)); color: var(--text); }}

  /* --- auth method pill (small chip) --- */
  .auth-method-pill {{
    display: inline-flex; align-items: center; gap: 0.375rem;
    padding: 0.125rem 0.5rem;
    background: rgba(139,92,246,0.10);
    color: #8b5cf6;
    font-size: 0.6875rem; font-weight: 600;
    text-transform: uppercase; letter-spacing: 0.05em;
    border-radius: 9999px;
    border: 1px solid rgba(139,92,246,0.25);
  }}
  .auth-method-pill-icon {{ display: inline-flex; align-items: center; }}
  .auth-method-pill-label {{ line-height: 1; }}

  /* --- connect button (orange→purple gradient, compact / default / full) --- */
  .connect-btn {{
    display: inline-flex; align-items: center; justify-content: center; gap: 0.5rem;
    background: linear-gradient(90deg, #fb923c, #a855f7);
    color: #ffffff;
    font-weight: 600;
    border: 0;
    border-radius: 9999px;
    cursor: pointer;
    text-decoration: none;
    box-shadow: 0 8px 24px -8px rgba(168,85,247,0.5);
    transition: all 0.2s ease;
    white-space: nowrap;
  }}
  .connect-btn:hover {{ transform: translateY(-1px); box-shadow: 0 12px 32px -8px rgba(168,85,247,0.6); }}
  .connect-btn:active {{ transform: translateY(0); }}
  .connect-btn:disabled {{ opacity: 0.6; cursor: not-allowed; }}
  .connect-btn-compact {{ height: 2rem; padding: 0 0.75rem; font-size: 0.75rem; }}
  .connect-btn-default {{ height: 2.5rem; padding: 0 1.25rem; font-size: 0.875rem; }}
  .connect-btn-full {{
    height: 3.25rem; padding: 0 1.5rem; font-size: 1rem;
    border-radius: 1rem;
    width: 100%;
  }}
  .connect-btn-icon {{ display: inline-flex; align-items: center; }}
  .connect-btn-label {{ line-height: 1; }}
  .connect-btn-chevron {{ display: inline-flex; align-items: center; opacity: 0.9; }}

  /* --- connected wallet dropdown (provider card + actions + nav + disconnect) --- */
  .connected-wallet-dropdown {{
    background: var(--surface-solid, #191923);
    color: #ffffff;
    border: 1px solid var(--epsx-border);
    border-radius: 1rem;
    box-shadow: 0 0 50px -12px rgba(0,0,0,0.5);
    width: 18rem;
    overflow: hidden;
    animation: scaleIn 0.15s ease;
  }}
  .wallet-provider-card {{
    padding: 1rem;
    background: linear-gradient(135deg, rgba(255,255,255,0.04), rgba(255,255,255,0.01));
    border-bottom: 1px solid var(--epsx-border);
    display: flex; align-items: center; gap: 0.75rem;
  }}
  .wallet-provider-icon {{
    display: inline-flex; align-items: center; justify-content: center;
    width: 2.5rem; height: 2.5rem;
    background: rgba(255,255,255,0.08);
    border-radius: 9999px;
    font-size: 1.25rem;
    flex-shrink: 0;
  }}
  .wallet-provider-meta {{ flex: 1; min-width: 0; }}
  .wallet-provider-name {{ font-size: 0.875rem; font-weight: 600; color: #ffffff; }}
  .wallet-provider-address {{
    font-size: 0.6875rem;
    font-family: ui-monospace, SFMono-Regular, Menlo, monospace;
    color: rgba(255,255,255,0.5);
    overflow: hidden; text-overflow: ellipsis; white-space: nowrap;
  }}
  .wallet-provider-status {{
    display: flex; align-items: center; gap: 0.375rem;
    margin-top: 0.25rem;
    font-size: 0.6875rem; font-weight: 500;
  }}
  .wallet-status-dot {{ display: inline-block; width: 0.4375rem; height: 0.4375rem; border-radius: 9999px; background: currentColor; }}
  .wallet-status-success {{ color: #10b981; }}
  .wallet-status-warning {{ color: #f59e0b; }}
  .wallet-status-error {{ color: #ef4444; }}
  .wallet-status-neutral {{ color: rgba(255,255,255,0.5); }}

  .wallet-actions-row {{
    display: flex; gap: 0.5rem;
    padding: 0.5rem;
    background: rgba(255,255,255,0.02);
  }}
  .wallet-action-btn {{
    flex: 1;
    display: inline-flex; align-items: center; justify-content: center; gap: 0.375rem;
    padding: 0.5rem 0.75rem;
    background: rgba(255,255,255,0.05);
    color: #ffffff;
    border: 1px solid transparent;
    border-radius: 0.5rem;
    font-size: 0.8125rem; font-weight: 500;
    cursor: pointer;
    transition: all 0.15s ease;
  }}
  .wallet-action-btn:hover {{ background: rgba(255,255,255,0.10); border-color: rgba(255,255,255,0.15); }}

  .wallet-meta-grid {{
    display: grid; grid-template-columns: 1fr 1fr; gap: 0.625rem 1rem;
    padding: 0.75rem 1rem;
    border-top: 1px solid var(--epsx-border);
    background: rgba(255,255,255,0.02);
  }}
  .wallet-meta-cell {{ min-width: 0; }}
  .wallet-meta-label {{
    font-size: 0.625rem; font-weight: 600;
    color: rgba(255,255,255,0.4);
    text-transform: uppercase; letter-spacing: 0.05em;
    margin-bottom: 0.125rem;
  }}
  .wallet-meta-value {{ font-size: 0.8125rem; color: #ffffff; }}
  .wallet-meta-value-role {{ color: #a78bfa; text-transform: capitalize; }}
  .wallet-meta-value-tier {{ color: #22d3ee; }}

  .wallet-network-badge {{
    display: flex; align-items: center; gap: 0.375rem;
    padding: 0.5rem 1rem;
    border-top: 1px solid var(--epsx-border);
    background: rgba(255,255,255,0.02);
    font-size: 0.75rem;
    color: rgba(255,255,255,0.7);
  }}
  .wallet-network-dot {{ width: 0.4375rem; height: 0.4375rem; border-radius: 9999px; background: rgba(255,255,255,0.5); }}
  .wallet-network-live .wallet-network-dot {{ background: #10b981; box-shadow: 0 0 0 2px rgba(16,185,129,0.2); }}
  .wallet-network-testnet .wallet-network-dot {{ background: #f59e0b; }}
  .wallet-network-other .wallet-network-dot {{ background: rgba(255,255,255,0.5); }}

  .wallet-signin-row,
  .wallet-retry-row {{
    width: calc(100% - 1rem);
    margin: 0.5rem;
    display: flex; align-items: center; gap: 0.75rem;
    padding: 0.75rem 1rem;
    border: 1px solid transparent;
    border-radius: 0.75rem;
    text-align: left;
    cursor: pointer;
    transition: all 0.15s ease;
  }}
  .wallet-signin-row {{
    background: linear-gradient(90deg, rgba(16,185,129,0.10), rgba(34,197,94,0.10));
    border-color: rgba(16,185,129,0.25);
    color: #10b981;
  }}
  .wallet-signin-row:hover {{ background: linear-gradient(90deg, rgba(16,185,129,0.18), rgba(34,197,94,0.18)); }}
  .wallet-retry-row {{
    background: rgba(251,146,60,0.10);
    border-color: rgba(251,146,60,0.30);
    color: #fb923c;
  }}
  .wallet-retry-row:hover {{ background: rgba(251,146,60,0.18); }}
  .wallet-signin-meta,
  .wallet-retry-meta {{ flex: 1; min-width: 0; }}
  .wallet-signin-title,
  .wallet-retry-title {{ font-size: 0.875rem; font-weight: 600; color: inherit; }}
  .wallet-signin-sub,
  .wallet-retry-sub {{ font-size: 0.75rem; opacity: 0.75; }}

  .wallet-nav-links {{
    display: flex; flex-direction: column; gap: 0.125rem;
    padding: 0.5rem;
    border-top: 1px solid var(--epsx-border);
  }}
  .wallet-nav-link {{
    display: flex; align-items: center; gap: 0.625rem;
    padding: 0.5rem 0.75rem;
    color: rgba(255,255,255,0.85);
    border-radius: 0.5rem;
    font-size: 0.8125rem; font-weight: 500;
    text-decoration: none;
    transition: all 0.15s ease;
  }}
  .wallet-nav-link:hover {{ background: rgba(255,255,255,0.06); color: #ffffff; }}

  .wallet-disconnect-btn {{
    width: calc(100% - 1rem);
    margin: 0.5rem;
    display: flex; align-items: center; justify-content: center; gap: 0.5rem;
    padding: 0.625rem 1rem;
    background: rgba(239,68,68,0.10);
    border: 1px solid rgba(239,68,68,0.25);
    color: #ef4444;
    border-radius: 0.625rem;
    font-size: 0.8125rem; font-weight: 500;
    cursor: pointer;
    transition: all 0.15s ease;
  }}
  .wallet-disconnect-btn:hover {{ background: rgba(239,68,68,0.20); }}

  /* --- legacy wallet pill (Wave 1 fallback) --- */
  .connected-wallet {{
    display: flex; align-items: center; gap: 0.625rem;
    padding: 0.375rem 0.75rem;
    background: rgba(255,255,255,0.05);
    border-radius: 9999px;
    color: #ffffff;
  }}
  .wallet-pill {{ display: flex; align-items: center; gap: 0.375rem; font-size: 0.8125rem; font-weight: 500; }}
  .wallet-pill .wallet-status-dot {{ background: #10b981; }}
  .wallet-address {{ font-family: ui-monospace, SFMono-Regular, Menlo, monospace; }}
  .wallet-balance {{ display: flex; align-items: baseline; gap: 0.25rem; color: #ffffff; }}

  /* end wave2-chrome-track-c */

  /* === wave3a-wiring-track-a === frontend MainLayout wrapper.
     Track A only adds an empty region marker — the layout component
     in `shared/rust/dioxus_ui/src/layout/main_layout.rs` reuses
     existing classes from the Wave 2 chrome cluster
     (`NavigationClient` → `.epsx-header`, `Footer` → `.site-footer`)
     plus the page-bg / page-content utilities. No new CSS is needed
     for the layout swap. Integration gate can concatenate an empty
     block here safely; future Wave 3a iterations may add overrides
     for per-page body padding or auth-page full-bleed rules. */
  /* end wave3a-wiring-track-a */
  /* === wave3a-wiring-track-c ===
   * Admin shell wiring: pages stopped rendering `<DashboardShell>`
   * themselves; the admin BFF now wraps every page body in
   * `AdminLayout::Auth` (Header + Sidebar + AdminFooter). The CSS
   * classes that the new layout renders (`.admin-shell`,
   * `.admin-main`, `.admin-header`, `.admin-content`,
   * `.admin-footer`, etc.) were already defined in the
   * `wave2-chrome-track-a` block above — no new rules are required
   * for this track. This marker exists so the integration gate can
   * confirm the three wave3a tracks append cleanly into a single
   * CSS region. */
  /* end wave3a-wiring-track-c */
  /* === wave3b-gates-track-a ===
    * Frontend user-page gate enrichment (Track A).
    * The 12 user pages (account, profile, dashboard, portfolio, payment,
    * notifications, analytics, permissions, chat, chat_history,
    * chat_conversation, account_credits) all call <AuthGate> with a
    * required_permissions list and a return_url of `ctx.path.clone()`.
    * The gate's HTML/CSS (`.auth-gate`, `.auth-gate-missing`,
    * `.auth-gate-perms`, `.auth-gate-actions`, etc.) was already
    * defined in the wave2-chrome-track-c block above; this track only
    * enriches the gate CALLSITES, not the gate styles. No new CSS
    * rules are required — the existing gate styles render the
    * permission list, the connect link, and the return_url `?next=...`
    * query string correctly. This marker exists so the integration
    * gate can confirm the three wave3b tracks append cleanly into a
    * single CSS region. */
  /* end wave3b-gates-track-a */

  /* === wave5-page-depth-track-a === hero-pages depth (home + auth + about)
     + MarketingBackground primitive.

     Adds CSS for:
       1. `<MarketingBackground>` — fixed gradient + 4 floating orbs
          (orange / blue / purple / green) + 3 radial mesh overlays
          + 2 geometric decorations. These are reused across the
          home / about / contact / plans pages, so they live here
          rather than in `marketing_bg.rs` (which is presentational
          markup only).
       2. Hero additions: share button, chain selector, mobile
          collapse, the existing 4-stat grid.
       3. New home sections: TestimonialsSection + FAQSection
          (the source has both; Wave 1 port was missing them).
       4. Auth page two-column layout + form (the Wave 1 port was
          form-only; the source has a marketing pitch on the left).
       5. About page: MissionSection, StatsSection, TeamSection,
          TimelineSection, inline DataTechSection port.

     All rules are additive — no existing class is restyled. The
     marker region is the only shared file surface with Track B
     (which uses `// === wave5-page-depth-track-b ===`). */

  /* === MarketingBackground primitive === */
  .marketing-bg {{ position: relative; min-height: 100vh; overflow: hidden; }}
  .marketing-bg-fixed {{
    position: fixed; inset: 0; z-index: 0; pointer-events: none;
    background: linear-gradient(to bottom right, #eff6ff, #fff7ed, #fefce8);
  }}
  .marketing-bg-gradient {{
    position: absolute; inset: 0;
    background:
      radial-gradient(circle at 25% 25%, rgba(255, 133, 27, 0.10) 0%, transparent 50%),
      radial-gradient(circle at 75% 75%, rgba(59, 130, 246, 0.08) 0%, transparent 50%),
      radial-gradient(circle at 50% 50%, rgba(168, 85, 247, 0.06) 0%, transparent 60%);
  }}
  .marketing-orb {{
    position: absolute; border-radius: 9999px; filter: blur(48px);
    animation: marketing-orb-drift 20s ease-in-out infinite;
  }}
  .marketing-orb-orange {{
    top: -10rem; left: -10rem; width: 24rem; height: 24rem;
    background: linear-gradient(to bottom right, rgba(251, 146, 60, 0.30), rgba(250, 204, 21, 0.30));
  }}
  .marketing-orb-blue {{
    top: 5rem; right: -8rem; width: 20rem; height: 20rem;
    background: linear-gradient(to bottom right, rgba(96, 165, 250, 0.25), rgba(34, 211, 238, 0.25));
  }}
  .marketing-orb-purple {{
    bottom: 5rem; left: 5rem; width: 18rem; height: 18rem;
    background: linear-gradient(to bottom right, rgba(192, 132, 252, 0.20), rgba(244, 114, 182, 0.20));
  }}
  .marketing-orb-green {{
    top: 50%; right: 25%; width: 16rem; height: 16rem;
    background: linear-gradient(to bottom right, rgba(74, 222, 128, 0.15), rgba(16, 185, 129, 0.15));
    transform: translateY(-50%);
  }}
  .marketing-mesh {{ position: absolute; inset: 0; pointer-events: none; }}
  .marketing-mesh-orange {{ background: radial-gradient(circle at 25% 25%, rgba(255, 133, 27, 0.10) 0%, transparent 50%); }}
  .marketing-mesh-blue   {{ background: radial-gradient(circle at 75% 75%, rgba(59, 130, 246, 0.08) 0%, transparent 50%); }}
  .marketing-mesh-purple {{ background: radial-gradient(circle at 50% 50%, rgba(168, 85, 247, 0.06) 0%, transparent 60%); }}
  .marketing-shape {{ position: absolute; pointer-events: none; }}
  .marketing-shape-square {{
    top: 25%; left: 25%; width: 8rem; height: 8rem; transform: rotate(45deg); border-radius: 1rem;
    background: linear-gradient(to bottom right, rgba(251, 146, 60, 0.10), rgba(250, 204, 21, 0.10));
  }}
  .marketing-shape-circle {{
    right: 33%; bottom: 33%; width: 6rem; height: 6rem; border-radius: 9999px;
    background: linear-gradient(to bottom right, rgba(96, 165, 250, 0.10), rgba(34, 211, 238, 0.10));
  }}
  .marketing-bg-content {{ position: relative; z-index: 1; }}
  @keyframes marketing-orb-drift {{
    0%, 100% {{ transform: translate(0, 0) scale(1); }}
    33%      {{ transform: translate(2rem, -2rem) scale(1.05); }}
    66%      {{ transform: translate(-2rem, 2rem) scale(0.95); }}
  }}

  /* === Hero additions (share button, chain selector, mobile collapse) === */
  .hero {{
    position: relative; min-height: 80vh; display: flex; align-items: center; justify-content: center;
    padding: 4rem 0; overflow: hidden;
  }}
  .hero-bg {{ position: absolute; inset: 0; pointer-events: none; }}
  .hero-orb {{ position: absolute; border-radius: 9999px; filter: blur(64px); opacity: 0.4; }}
  .hero-orb-1 {{ top: 10%; left: 5%; width: 18rem; height: 18rem; background: rgba(251, 146, 60, 0.30); }}
  .hero-orb-2 {{ bottom: 10%; right: 5%; width: 18rem; height: 18rem; background: rgba(96, 165, 250, 0.30); }}
  .hero-orb-3 {{ top: 40%; right: 25%; width: 14rem; height: 14rem; background: rgba(192, 132, 252, 0.25); }}
  .hero-orb-4 {{ top: 60%; left: 30%; width: 12rem; height: 12rem; background: rgba(74, 222, 128, 0.20); }}
  .hero-inner {{ position: relative; z-index: 1; text-align: center; max-width: 72rem; margin: 0 auto; padding: 0 1.5rem; }}
  .hero-badge {{
    display: inline-flex; align-items: center; gap: 0.5rem; padding: 0.5rem 1rem; border-radius: 9999px;
    background: linear-gradient(to right, rgba(251, 146, 60, 0.10), rgba(250, 204, 21, 0.10));
    border: 1px solid rgba(251, 146, 60, 0.20); font-size: 0.875rem; font-weight: 500;
    color: rgb(194, 65, 12); margin-bottom: 1.5rem;
  }}
  .hero-badge-dot {{
    width: 0.5rem; height: 0.5rem; border-radius: 9999px; background: rgb(251, 146, 60);
    animation: pulse 2s ease-in-out infinite;
  }}
  .hero-title {{ font-size: 3.5rem; line-height: 1.1; font-weight: 800; margin: 0 0 1.5rem; }}
  .hero-title-line {{ display: block; }}
  .hero-title-gradient {{
    background: linear-gradient(to right, rgb(249, 115, 22), rgb(234, 179, 8), rgb(234, 88, 12));
    -webkit-background-clip: text; background-clip: text; color: transparent;
  }}
  .hero-subtitle {{ font-size: 1.25rem; line-height: 1.6; color: rgb(82, 82, 91); max-width: 56rem; margin: 0 auto 2rem; }}
  .hero-subtitle-accent {{
    background: linear-gradient(to right, rgb(59, 130, 246), rgb(168, 85, 247));
    -webkit-background-clip: text; background-clip: text; color: transparent; font-weight: 700;
  }}
  .hero-actions {{ display: flex; flex-wrap: wrap; gap: 1rem; justify-content: center; align-items: center; margin: 2rem 0 1.5rem; }}
  .hero-cta-primary {{ min-width: 220px; height: 3.5rem; font-size: 1.125rem; font-weight: 700; }}
  .hero-share-btn {{ min-width: 220px; height: 3.5rem; font-size: 1.125rem; font-weight: 700; }}
  .hero-chain-selector {{ display: inline-flex; align-items: center; gap: 0.5rem; margin: 1rem 0 2rem; padding: 0.5rem 1rem; border-radius: 9999px; background: rgba(255, 255, 255, 0.6); backdrop-filter: blur(8px); border: 1px solid rgba(0, 0, 0, 0.06); }}
  .hero-chain-label {{ font-size: 0.75rem; font-weight: 600; color: rgb(82, 82, 91); text-transform: uppercase; letter-spacing: 0.05em; }}
  .hero-chain-pill {{ display: inline-flex; align-items: center; gap: 0.375rem; padding: 0.25rem 0.75rem; border-radius: 9999px; font-size: 0.8125rem; font-weight: 600; color: rgb(82, 82, 91); }}
  .hero-chain-pill-active {{ background: rgba(16, 185, 129, 0.10); color: rgb(6, 95, 70); }}
  .hero-chain-dot {{ width: 0.5rem; height: 0.5rem; border-radius: 9999px; background: rgb(16, 185, 129); }}
  .hero-chain-dot-testnet {{ background: rgb(234, 179, 8); }}
  .hero-stats {{ display: grid; grid-template-columns: repeat(4, 1fr); gap: 1.5rem; margin-top: 4rem; }}
  .hero-stat {{
    position: relative; padding: 2rem 1.5rem; border-radius: 1.25rem; text-align: center;
    background: rgba(255, 255, 255, 0.7); backdrop-filter: blur(12px);
    border: 1px solid rgba(251, 146, 60, 0.20); box-shadow: 0 20px 25px -5px rgba(0, 0, 0, 0.10);
  }}
  .hero-stat-icon {{ margin-bottom: 0.5rem; }}
  .hero-stat-value {{ font-size: 2.5rem; font-weight: 800; line-height: 1; background: linear-gradient(to right, rgb(249, 115, 22), rgb(234, 179, 8)); -webkit-background-clip: text; background-clip: text; color: transparent; margin-bottom: 0.5rem; }}
  .hero-stat-label {{ font-size: 0.875rem; font-weight: 500; color: rgb(82, 82, 91); }}
  @media (max-width: 768px) {{
    .hero-title {{ font-size: 2.5rem; }}
    .hero-stats {{ grid-template-columns: 1fr; }}
    .hero-actions {{ flex-direction: column; }}
    .hero-chain-selector {{ flex-direction: column; gap: 0.5rem; padding: 0.75rem; }}
  }}

  /* === TrustBar additions (Binance, Ethereum Foundation logos) === */
  .trust-bar {{ padding: 3rem 0; border-top: 1px solid rgba(0, 0, 0, 0.06); border-bottom: 1px solid rgba(0, 0, 0, 0.06); }}
  .trust-bar-inner {{ text-align: center; }}
  .trust-bar-label {{ font-size: 0.75rem; font-weight: 600; text-transform: uppercase; letter-spacing: 0.1em; color: rgb(113, 113, 122); margin-bottom: 1.5rem; }}
  .trust-bar-logos {{ display: flex; flex-wrap: wrap; gap: 1rem; justify-content: center; align-items: center; }}
  .trust-logo {{
    display: inline-flex; align-items: center; padding: 0.5rem 1.25rem; border-radius: 9999px;
    background: rgba(255, 255, 255, 0.6); backdrop-filter: blur(8px);
    border: 1px solid rgba(0, 0, 0, 0.06); font-size: 0.875rem; font-weight: 600; color: rgb(63, 63, 70);
  }}

  /* === TopPerformers additions (data-freshness timestamp) === */
  .top-performers {{ padding: 5rem 0; }}
  .top-performers-freshness {{
    display: inline-flex; align-items: center; gap: 0.375rem; margin-top: 0.75rem;
    font-size: 0.75rem; color: rgb(113, 113, 122); font-weight: 500;
  }}
  .top-performers-grid {{ display: grid; grid-template-columns: repeat(auto-fit, minmax(220px, 1fr)); gap: 1.25rem; margin-top: 2.5rem; }}
  .performer-card {{
    padding: 1.5rem; text-decoration: none; color: inherit; display: block;
    transition: transform 0.2s ease, box-shadow 0.2s ease;
  }}
  .performer-card:hover {{ transform: translateY(-2px); box-shadow: 0 20px 25px -5px rgba(0, 0, 0, 0.10); }}
  .performer-symbol {{ font-size: 1.5rem; font-weight: 800; color: rgb(24, 24, 27); margin-bottom: 0.5rem; }}
  .performer-price {{ font-size: 1.125rem; font-weight: 600; color: rgb(63, 63, 70); margin-bottom: 0.75rem; }}

  /* === FeaturesGrid === */
  .features-grid-section {{ padding: 5rem 0; }}
  .features-grid {{ display: grid; grid-template-columns: repeat(auto-fit, minmax(280px, 1fr)); gap: 1.5rem; margin-top: 2.5rem; }}
  .feature-card {{ padding: 2rem; }}
  .feature-icon {{ display: inline-flex; padding: 0.75rem; border-radius: 0.75rem; background: rgba(251, 146, 60, 0.10); margin-bottom: 1.25rem; }}
  .feature-title {{ font-size: 1.25rem; font-weight: 700; margin: 0 0 0.5rem; color: rgb(24, 24, 27); }}
  .feature-description {{ font-size: 0.9375rem; line-height: 1.6; margin: 0; }}

  /* === PricingTeaser === */
  .pricing-teaser {{ padding: 5rem 0; }}
  .pricing-teaser-grid {{ display: grid; grid-template-columns: repeat(auto-fit, minmax(260px, 1fr)); gap: 1.5rem; margin-top: 2.5rem; }}
  .pricing-teaser-card {{ padding: 2rem; display: flex; flex-direction: column; gap: 1rem; }}
  .pricing-teaser-tier {{ font-size: 0.875rem; font-weight: 700; text-transform: uppercase; letter-spacing: 0.05em; color: rgb(113, 113, 122); }}
  .pricing-teaser-price {{ font-size: 1.75rem; font-weight: 800; color: rgb(24, 24, 27); }}
  .pricing-teaser-features {{ list-style: none; padding: 0; margin: 0; display: flex; flex-direction: column; gap: 0.5rem; }}
  .pricing-teaser-features li {{ font-size: 0.9375rem; color: rgb(63, 63, 70); padding-left: 1.5rem; position: relative; }}
  .pricing-teaser-features li::before {{ content: "✓"; position: absolute; left: 0; color: rgb(16, 185, 129); font-weight: 700; }}
  .pricing-teaser-card.highlighted {{ border: 2px solid rgba(251, 146, 60, 0.50); }}

  /* === NewsPreview === */
  .news-preview {{ padding: 5rem 0; }}
  .news-preview-grid {{ display: grid; grid-template-columns: repeat(auto-fit, minmax(280px, 1fr)); gap: 1.5rem; margin-top: 2.5rem; }}
  .news-preview-card {{ padding: 2rem; text-decoration: none; color: inherit; display: block; transition: transform 0.2s ease; }}
  .news-preview-card:hover {{ transform: translateY(-2px); }}
  .news-preview-tag {{
    display: inline-block; padding: 0.25rem 0.75rem; border-radius: 9999px;
    background: rgba(59, 130, 246, 0.10); color: rgb(29, 78, 216);
    font-size: 0.75rem; font-weight: 600; margin-bottom: 1rem;
  }}
  .news-preview-title {{ font-size: 1.25rem; font-weight: 700; margin: 0 0 0.5rem; color: rgb(24, 24, 27); }}
  .news-preview-excerpt {{ font-size: 0.9375rem; line-height: 1.6; margin: 0; }}

  /* === TestimonialsSection (NEW) === */
  .testimonials-section {{ padding: 5rem 0; }}
  .testimonials-grid {{ display: grid; grid-template-columns: repeat(auto-fit, minmax(300px, 1fr)); gap: 1.5rem; margin-top: 2.5rem; }}
  .testimonial-card {{ padding: 2rem; display: flex; flex-direction: column; gap: 1.25rem; }}
  .testimonial-rating {{ display: flex; gap: 0.25rem; }}
  .testimonial-star {{
    width: 1.25rem; height: 1.25rem;
    background: linear-gradient(to right, rgb(250, 204, 21), rgb(234, 88, 12));
    clip-path: polygon(50% 0%, 61% 35%, 98% 35%, 68% 57%, 79% 91%, 50% 70%, 21% 91%, 32% 57%, 2% 35%, 39% 35%);
  }}
  .testimonial-quote {{ font-size: 1rem; line-height: 1.7; color: rgb(63, 63, 70); margin: 0; font-style: italic; }}
  .testimonial-meta {{ display: flex; align-items: center; gap: 0.75rem; margin-top: auto; padding-top: 1rem; border-top: 1px solid rgba(0, 0, 0, 0.06); }}
  .testimonial-avatar {{
    width: 2.5rem; height: 2.5rem; border-radius: 9999px; flex-shrink: 0;
    display: flex; align-items: center; justify-content: center; color: white; font-weight: 700;
  }}
  .testimonial-avatar-1 {{ background: linear-gradient(135deg, #f97316, #f59e0b); }}
  .testimonial-avatar-2 {{ background: linear-gradient(135deg, #3b82f6, #06b6d4); }}
  .testimonial-avatar-3 {{ background: linear-gradient(135deg, #a855f7, #ec4899); }}
  .testimonial-name {{ font-weight: 700; color: rgb(24, 24, 27); font-size: 0.9375rem; }}
  .testimonial-role {{ font-size: 0.8125rem; color: rgb(113, 113, 122); }}

  /* === FAQSection (NEW) === */
  .faq-section {{ padding: 5rem 0; }}
  .faq-list {{ max-width: 56rem; margin: 2.5rem auto 0; display: flex; flex-direction: column; gap: 0.75rem; }}
  .faq-item {{
    background: rgba(255, 255, 255, 0.7); backdrop-filter: blur(8px);
    border: 1px solid rgba(0, 0, 0, 0.06); border-radius: 1rem; overflow: hidden;
    transition: border-color 0.2s ease;
  }}
  .faq-item[open] {{ border-color: rgba(251, 146, 60, 0.40); }}
  .faq-item summary {{
    list-style: none; cursor: pointer; padding: 1.25rem 1.5rem; display: flex; align-items: center; justify-content: space-between; gap: 1rem;
    font-weight: 600; color: rgb(24, 24, 27);
  }}
  .faq-item summary::-webkit-details-marker {{ display: none; }}
  .faq-chevron {{ display: inline-flex; transition: transform 0.2s ease; color: rgb(113, 113, 122); }}
  .faq-item[open] .faq-chevron {{ transform: rotate(180deg); color: rgb(249, 115, 22); }}
  .faq-answer {{ padding: 0 1.5rem 1.5rem 1.5rem; color: rgb(63, 63, 70); line-height: 1.7; }}
  .faq-answer p {{ margin: 0; }}

  /* === CTASection additions (Talk to sales secondary link) === */
  .cta-section {{ padding: 5rem 0; }}
  .cta-card {{ padding: 3rem 2rem; text-align: center; display: flex; flex-direction: column; gap: 1.5rem; align-items: center; }}
  .cta-title {{ font-size: 2rem; font-weight: 800; margin: 0; color: white; }}
  .cta-subtitle {{ font-size: 1.125rem; color: rgba(255, 255, 255, 0.85); margin: 0; max-width: 36rem; }}
  .cta-actions {{ display: flex; flex-wrap: wrap; gap: 0.75rem; justify-content: center; }}
  .cta-secondary-link {{ color: white !important; border-color: rgba(255, 255, 255, 0.30) !important; }}

  /* === Auth page two-column layout + form === */
  .auth-page {{
    position: relative; display: flex; min-height: 100vh; width: 100%;
    flex-direction: column; overflow: hidden;
  }}
  /* The source auth page keeps its three blurred color orbs outside the
     desktop pitch column.  Keeping a page-level layer means the same
     depth is visible on tablet/mobile when that column is hidden. */
  .auth-page-background {{ position: absolute; inset: 0; z-index: 0; pointer-events: none; overflow: hidden; }}
  .auth-page-background-orb {{ position: absolute; border-radius: 9999px; filter: blur(120px); opacity: 0.75; animation: pulse 4s ease-in-out infinite; }}
  .auth-page-background-orb-1 {{ top: -10%; left: -10%; width: 60%; height: 60%; background: rgba(249, 115, 22, 0.10); }}
  .auth-page-background-orb-2 {{ bottom: -10%; right: -10%; width: 60%; height: 60%; background: rgba(168, 85, 247, 0.10); animation-delay: 1s; }}
  .auth-page-background-orb-3 {{ top: 20%; right: 10%; width: 40%; height: 40%; background: rgba(59, 130, 246, 0.10); animation-delay: 2s; }}
  .auth-page-theme-toggle {{
    position: absolute; top: 1.5rem; right: 1.5rem; z-index: 3;
    display: inline-flex; align-items: center; justify-content: center;
  }}
  .auth-page-theme-toggle .theme-toggle {{
    width: 2.75rem; height: 2.75rem; padding: 0;
    border: 1px solid rgba(148, 163, 184, 0.24);
    border-radius: 9999px; background: rgba(15, 23, 42, 0.32);
    color: var(--text); backdrop-filter: blur(12px);
  }}
  .auth-page-theme-toggle .theme-toggle:hover {{ background: rgba(15, 23, 42, 0.50); }}
  html:not(.dark) .auth-page-theme-toggle .theme-toggle {{
    border-color: rgba(15, 23, 42, 0.12); background: rgba(255, 255, 255, 0.58);
  }}
  html:not(.dark) .auth-page-theme-toggle .theme-toggle:hover {{ background: rgba(255, 255, 255, 0.82); }}
  @media (min-width: 1024px) {{ .auth-page {{ flex-direction: row; }} }}
  .auth-page-pitch {{
    position: relative; display: none; flex-direction: column; justify-content: center;
    padding: 2rem; color: rgb(24, 24, 27); width: 100%; overflow: hidden;
  }}
  @media (min-width: 1024px) {{ .auth-page-pitch {{ display: flex; width: 60%; padding: 5rem; }} }}
  .auth-page-pitch-bg {{ position: absolute; inset: 0; z-index: 0; pointer-events: none; overflow: hidden; }}
  .auth-page-pitch-orb {{
    position: absolute; border-radius: 9999px; filter: blur(120px); animation: pulse 4s ease-in-out infinite;
  }}
  .auth-page-pitch-orb-1 {{ top: -10%; left: -10%; width: 60%; height: 60%; background: rgba(251, 146, 60, 0.10); }}
  .auth-page-pitch-orb-2 {{ bottom: -10%; right: -10%; width: 60%; height: 60%; background: rgba(168, 85, 247, 0.10); animation-delay: 1s; }}
  .auth-page-pitch-orb-3 {{ top: 20%; right: 10%; width: 40%; height: 40%; background: rgba(59, 130, 246, 0.10); animation-delay: 2s; }}
  .auth-page-pitch-inner {{ position: relative; z-index: 1; max-width: 40rem; }}
  .auth-page-brand {{ margin-bottom: 3rem; font-size: 2rem; font-weight: 900; font-style: italic; letter-spacing: -0.02em; text-transform: uppercase; }}
  .auth-page-brand a {{ color: inherit; text-decoration: none; }}
  .auth-brand-icon {{ display: inline-flex; align-items: center; justify-content: center; width: 3.25rem; height: 3.25rem; margin-right: 0.75rem; border-radius: 1rem; background: linear-gradient(135deg, #f97316, #9333ea); box-shadow: 0 0 32px -8px rgba(249,115,22,0.55); vertical-align: middle; font-style: normal; }}
  .auth-page-headline {{
    font-size: 3.5rem; line-height: 1.1; font-weight: 800; margin: 0 0 1.5rem;
  }}
  .auth-page-headline-line {{ display: inline-block; white-space: nowrap; }}
  @media (min-width: 1280px) {{ .auth-page-headline {{ font-size: 4.25rem; }} }}
  .auth-page-sub {{ font-size: 1.125rem; line-height: 1.7; color: rgb(113, 113, 122); margin: 0 0 3rem; max-width: 32rem; }}
  .auth-page-value-props {{ display: grid; grid-template-columns: repeat(auto-fit, minmax(200px, 1fr)); gap: 2rem; max-width: 36rem; margin-bottom: 3rem; }}
  .auth-page-value-prop {{ display: flex; gap: 1rem; align-items: flex-start; }}
  .auth-page-value-icon {{
    flex-shrink: 0; display: inline-flex; align-items: center; justify-content: center;
    width: 3rem; height: 3rem; border-radius: 0.75rem; background: rgba(251, 146, 60, 0.10);
    border: 1px solid rgba(0, 0, 0, 0.06);
  }}
  .auth-page-value-title {{ font-size: 1rem; font-weight: 700; margin: 0 0 0.25rem; color: rgb(24, 24, 27); }}
  .auth-page-value-desc {{ font-size: 0.875rem; color: rgb(113, 113, 122); margin: 0; line-height: 1.5; }}
  .auth-page-social-proof {{ display: flex; align-items: center; gap: 1.5rem; padding-top: 1.5rem; border-top: 1px solid rgba(0, 0, 0, 0.06); max-width: 24rem; }}
  .auth-page-social-avatars {{ display: flex; }}
  .auth-page-social-avatar {{
    width: 2.25rem; height: 2.25rem; border-radius: 9999px; border: 2px solid white;
    display: flex; align-items: center; justify-content: center; font-size: 0.75rem; font-weight: 800; color: white;
  }}
  .auth-page-social-avatar:not(:first-child) {{ margin-left: -0.5rem; }}
  .auth-page-social-avatar-a {{ background: linear-gradient(135deg, #f97316, #f59e0b); }}
  .auth-page-social-avatar-b {{ background: linear-gradient(135deg, #3b82f6, #06b6d4); }}
  .auth-page-social-avatar-c {{ background: linear-gradient(135deg, #a855f7, #ec4899); }}
  .auth-page-social-avatar-d {{ background: linear-gradient(135deg, #10b981, #06b6d4); }}
  .auth-page-social-text {{ font-size: 0.875rem; color: rgb(113, 113, 122); margin: 0; }}
  .auth-page-social-count {{ font-weight: 700; color: rgb(24, 24, 27); font-size: 1rem; padding: 0 0.25rem; }}

  .auth-page-form-col {{
    position: relative; z-index: 1; display: flex; align-items: center; justify-content: center;
    padding: 1rem; width: 100%;
  }}
  @media (min-width: 1024px) {{ .auth-page-form-col {{ width: 40%; backdrop-filter: blur(24px); border-left: 1px solid rgba(0, 0, 0, 0.06); }} }}
  .auth-page-form-inner {{ width: 100%; max-width: 28rem; display: flex; flex-direction: column; gap: 0; }}
  .auth-page-mobile-header {{ display: none; text-align: center; color: var(--text); }}
  .auth-page-mobile-header h2 {{ font-size: 2rem; font-weight: 700; letter-spacing: -0.02em; margin: 0.75rem 0 0.5rem; }}
  .auth-page-mobile-header p {{ color: var(--text-muted); margin: 0; font-size: 1rem; }}
  .auth-page-mobile-brand {{ display: flex; justify-content: center; align-items: center; margin-bottom: 1rem; font-size: 1.875rem; font-weight: 900; font-style: italic; letter-spacing: -0.04em; text-transform: uppercase; }}
  .auth-page-mobile-brand a {{ color: inherit; text-decoration: none; }}
  .auth-card {{ padding: 2.5rem 2rem; display: flex; flex-direction: column; gap: 2rem; border-radius: 1.5rem; }}
  .auth-card-desktop-heading {{ display: block; }}
  .auth-card-mobile-icon {{ display: flex; justify-content: center; align-items: center; width: 5rem; height: 5rem; margin: 0 auto; border-radius: 1.5rem; background: rgba(249,115,22,0.10); color: var(--epsx-orange); border: 1px solid rgba(249,115,22,0.20); }}
  .auth-card-title {{ font-size: 1.5rem; font-weight: 800; margin: 0; color: rgb(24, 24, 27); text-align: center; }}
  .auth-card-sub {{ font-size: 0.9375rem; color: rgb(113, 113, 122); margin: 0; text-align: center; }}
  .auth-card-cta {{ width: 100%; }}
  /* ConnectButton emits an inline-flex wrapper around the actual button.
     Stretch both layers so the wallet CTA follows the source card geometry
     on compact/tablet screens instead of collapsing to its label width. */
  .auth-card-cta .connect-btn-wrap {{ display: flex; width: 100%; }}
  .auth-card-cta .connect-btn {{ display: flex; width: 100% !important; justify-content: center; height: 3.5rem; font-size: 1.125rem; font-weight: 700; }}
  .auth-card-cta .connect-btn {{ background: linear-gradient(90deg, #f97316, #f59e0b); box-shadow: 0 12px 30px -10px rgba(249,115,22,0.55); }}
  .auth-card-cta .connect-btn:hover {{ background: linear-gradient(90deg, #ea580c, #d97706); box-shadow: 0 16px 34px -10px rgba(249,115,22,0.65); }}
  .auth-card-divider {{ display: flex; align-items: center; gap: 1rem; color: rgb(161, 161, 170); font-size: 0.75rem; font-weight: 600; text-transform: uppercase; letter-spacing: 0.1em; margin: 0.5rem 0; }}
  .auth-card-divider::before, .auth-card-divider::after {{ content: ""; flex: 1; height: 1px; background: rgba(0, 0, 0, 0.08); }}
  .auth-card-divider-thin {{ margin: 0; }}
  .auth-card-features {{ list-style: none; display: flex; flex-direction: column; gap: 0.75rem; margin: 0; padding: 0; }}
  .auth-card-feature {{ display: flex; align-items: center; gap: 0.75rem; color: var(--text-muted); font-size: 0.875rem; line-height: 1.5; }}
  .auth-card-feature-icon {{ display: inline-flex; flex: 0 0 1.25rem; width: 1.25rem; height: 1.25rem; align-items: center; justify-content: center; border: 1px solid rgba(249,115,22,0.55); border-radius: 9999px; color: var(--epsx-orange); font-size: 0.75rem; font-weight: 800; line-height: 1; }}
  .auth-card-email-form {{ display: flex; flex-direction: column; gap: 0.5rem; }}
  .auth-card-email-input {{ width: 100%; }}
  .auth-card-google-btn {{ gap: 0.5rem; }}
  .auth-card-google-glyph {{
    display: inline-flex; align-items: center; justify-content: center;
    width: 1.25rem; height: 1.25rem; border-radius: 0.25rem; font-weight: 900; color: #4285F4;
  }}
  .auth-card-foot {{ max-width: 20rem; align-self: center; font-size: 0.75rem; color: rgb(113, 113, 122); text-align: center; margin: 0; line-height: 1.6; }}
  .auth-card-foot a {{ color: rgb(82, 82, 91); text-decoration: underline; text-underline-offset: 2px; }}
  .auth-card-mobile-features {{ display: none; }}
  .auth-card-mobile-feature {{ display: flex; flex-direction: column; align-items: center; text-align: center; gap: 0.5rem; padding: 0.75rem; border-radius: 0.75rem; border: 1px solid rgba(0,0,0,0.08); background: rgba(255,255,255,0.02); }}
  .auth-card-mobile-feature h4 {{ margin: 0; font-size: 0.75rem; font-weight: 600; color: var(--text); }}
  .auth-card-mobile-feature-icon {{ display: flex; align-items: center; justify-content: center; width: 2.5rem; height: 2.5rem; border-radius: 0.5rem; background: rgba(249,115,22,0.10); color: var(--epsx-orange); }}
  .auth-card-error {{
    display: flex; gap: 0.75rem; padding: 0.75rem 1rem; border-radius: 0.5rem;
    background: rgba(239, 68, 68, 0.10); border: 1px solid rgba(239, 68, 68, 0.30); color: rgb(153, 27, 27);
  }}
  .auth-card-error-icon {{ flex-shrink: 0; padding-top: 0.125rem; }}
  .auth-card-error-body {{ flex: 1; min-width: 0; }}
  .auth-card-error-title {{ font-size: 0.875rem; font-weight: 700; margin-bottom: 0.125rem; }}
  .auth-card-error-msg {{ font-size: 0.8125rem; line-height: 1.5; }}
  .auth-card-status {{ display: flex; align-items: center; gap: 0.5rem; padding: 0.75rem 1rem; border-radius: 0.5rem; background: rgba(59, 130, 246, 0.10); color: rgb(30, 64, 175); font-size: 0.875rem; font-weight: 500; }}
  .auth-page-status-indicator {{
    display: flex; align-items: center; justify-content: center; gap: 0.5rem;
    font-size: 0.625rem; font-weight: 700; text-transform: uppercase; letter-spacing: 0.2em; color: rgb(113, 113, 122);
  }}
  .auth-page-status-dot {{ width: 0.25rem; height: 0.25rem; border-radius: 9999px; background: rgb(16, 185, 129); animation: pulse 2s ease-in-out infinite; }}
  .auth-page-status-compact {{ display: none; }}
  .auth-page-status-wallet {{ display: none; }}
  .auth-page-fallback {{ text-align: center; font-size: 0.75rem; }}
  .auth-page-fallback a {{ color: rgb(113, 113, 122); text-decoration: underline; text-underline-offset: 2px; }}

  html.dark .auth-page-pitch,
  html.dark .auth-page-value-title,
  html.dark .auth-card-title,
  html.dark .auth-card-foot a {{ color: var(--text); }}
  html.dark .auth-page-sub,
  html.dark .auth-page-value-desc,
  html.dark .auth-page-social-text,
  html.dark .auth-card-sub,
  html.dark .auth-card-foot,
  html.dark .auth-page-status-indicator,
  html.dark .auth-page-fallback a {{ color: var(--text-muted); }}
  html.dark .auth-card-error {{
    background: #7f1d1d; color: #fecaca;
  }}
  html.dark .auth-card-status {{ background: #1e3a8a; color: #dbeafe; }}
  html.dark .auth-card-mobile-feature {{ border-color: rgba(148,163,184,0.25); }}
  @media (max-width: 1023px) {{
    .auth-page-form-col {{ padding-left: 1rem; padding-right: 1rem; }}
    .auth-page-mobile-header {{ display: block; margin-top: 1rem; margin-bottom: 1.5rem; }}
    .auth-card-desktop-heading {{ display: none; }}
    .auth-card-mobile-icon {{ display: flex; }}
    .auth-card {{ padding: 2rem; }}
    .auth-card-mobile-features {{ display: grid; grid-template-columns: repeat(2, minmax(0, 1fr)); gap: 0.75rem; padding-top: 1.25rem; border-top: 1px solid rgba(0,0,0,0.08); }}
    html.dark .auth-card-mobile-features {{ border-top-color: rgba(148,163,184,0.25); }}
  }}
  @media (min-width: 640px) and (max-width: 1023px) {{
    .auth-page-form-col {{ padding-top: 1.5rem; padding-bottom: 1.5rem; }}
    .auth-page-form-inner {{ max-width: 24rem; }}
    .auth-page-mobile-header {{ margin-bottom: 2.5rem; }}
    .auth-page-mobile-header h2 {{ font-size: 1.75rem; }}
    .auth-page-mobile-header p {{ font-size: 0.875rem; }}
    .auth-page-mobile-brand {{ margin-bottom: 0.75rem; }}
    .auth-page-mobile-brand .auth-brand-icon {{ width: 3rem; height: 3rem; margin-right: 0.5rem; }}
    .auth-card {{ padding: 1.75rem 1.75rem 1.6875rem; gap: 1.75rem; }}
    .auth-card-mobile-icon {{ width: 4.25rem; height: 4.25rem; border-radius: 1.25rem; }}
    .auth-card-cta .connect-btn {{ height: 3rem; font-size: 1rem; }}
    .auth-card-features {{ gap: 0.5rem; }}
    .auth-card-mobile-features {{ gap: 0.625rem; padding-top: 1.125rem; margin-top: -0.5rem; }}
    .auth-card-mobile-feature {{ min-height: 4.875rem; padding: 0.625rem; gap: 0.25rem; }}
    .auth-card-mobile-feature-icon {{ width: 2rem; height: 2rem; }}
    .auth-card-foot {{ margin-top: -0.125rem; }}
    .auth-page-status-wide {{ display: none; }}
    .auth-page-status-compact {{ display: inline; }}
    .auth-page-status-indicator {{ margin-top: 1.5rem; }}
    .auth-page-fallback {{ margin-top: 1rem; }}
  }}
  @media (max-width: 639px) {{
    .auth-card {{ padding: 2.5rem 2rem; }}
    .auth-card-cta {{ margin-top: 0.125rem; }}
    .auth-card-foot {{ margin-top: 0.25rem; }}
    .auth-card-cta .connect-btn {{ background: linear-gradient(90deg, #ff8a3d, #b34de8); box-shadow: 0 12px 30px -10px rgba(178, 76, 232, 0.42); }}
    .auth-card-cta .connect-btn:hover {{ background: linear-gradient(90deg, #f97316, #9333ea); box-shadow: 0 16px 34px -10px rgba(168, 85, 247, 0.55); }}
    .auth-page-status-wide {{ display: none; }}
    .auth-page-status-compact {{ display: none; }}
    .auth-page-status-wallet {{ display: inline; }}
    .auth-page-status-indicator {{ margin-top: 1.5rem; }}
    .auth-page-fallback {{ margin-top: 1.5rem; }}
  }}

  /* === About page sections === */
  html.dark .about-page .marketing-bg-fixed {{
    background: linear-gradient(to bottom right, #0f172a, #1e293b, #0f172a);
  }}
  html.dark .about-page .marketing-orb-orange {{
    background: linear-gradient(to bottom right, rgba(251, 146, 60, 0.30), rgba(250, 204, 21, 0.30));
  }}
  html.dark .about-page .marketing-orb-blue {{
    background: linear-gradient(to bottom right, rgba(96, 165, 250, 0.25), rgba(34, 211, 238, 0.25));
  }}
  html.dark .about-page .marketing-orb-purple {{
    background: linear-gradient(to bottom right, rgba(192, 132, 252, 0.20), rgba(244, 114, 182, 0.20));
  }}
  html.dark .about-page .marketing-orb-green {{
    background: linear-gradient(to bottom right, rgba(74, 222, 128, 0.15), rgba(16, 185, 129, 0.15));
  }}
  html.dark .about-page .marketing-mesh-orange {{ background: radial-gradient(circle at 25% 25%, rgba(255, 133, 27, 0.10) 0%, transparent 50%); }}
  html.dark .about-page .marketing-mesh-blue {{ background: radial-gradient(circle at 75% 75%, rgba(59, 130, 246, 0.08) 0%, transparent 50%); }}
  html.dark .about-page .marketing-mesh-purple {{ background: radial-gradient(circle at 50% 50%, rgba(168, 85, 247, 0.06) 0%, transparent 60%); }}
  html.dark .about-page .marketing-shape-square {{
    background: linear-gradient(to bottom right, rgba(251, 146, 60, 0.10), rgba(250, 204, 21, 0.10));
  }}
  html.dark .about-page .marketing-shape-circle {{
    background: linear-gradient(to bottom right, rgba(96, 165, 250, 0.10), rgba(34, 211, 238, 0.10));
  }}
  .about-page .marketing-orb {{ filter: blur(64px); }}
  html.dark .about-page .card-glass {{
    background: rgba(30, 41, 59, 0.80);
  }}
  .about-hero-section {{ padding: 4rem 0 2rem; text-align: center; }}
  .about-hero-content {{ max-width: 48rem; margin: 0 auto; }}
  .about-hero-title {{
    font-size: 3rem; line-height: 1; font-weight: 700; margin: 0 0 1rem;
    background: linear-gradient(to right, rgb(249, 115, 22), rgb(234, 179, 8), rgb(234, 88, 12));
    -webkit-background-clip: text; background-clip: text; color: transparent;
  }}
  @media (min-width: 640px) {{ .about-hero-title {{ font-size: 3.75rem; }} }}
  .about-hero-sub {{ font-size: 1.125rem; line-height: 1.625; color: rgb(82, 82, 91); margin: 0 auto 1.5rem; max-width: 48rem; }}
  .about-hero-underline {{ width: 10rem; height: 0.25rem; margin: 0 auto; border-radius: 9999px; background: linear-gradient(to right, rgb(249, 115, 22), rgb(234, 179, 8), rgb(234, 88, 12)); }}

  .mission-section {{ padding: 4rem 0; }}
  .mission-grid {{ display: grid; grid-template-columns: 1fr; gap: 3rem; align-items: center; }}
  @media (min-width: 1024px) {{ .mission-grid {{ grid-template-columns: 1fr 1fr; }} }}
  .mission-card {{ padding: 2rem; }}
  .mission-card-icon {{ display: inline-flex; padding: 0.75rem; border-radius: 0.75rem; background: rgba(251, 146, 60, 0.10); margin-bottom: 1.5rem; }}
  .mission-card-title {{
    font-size: 1.75rem; font-weight: 800; margin: 0 0 1rem;
    background: linear-gradient(to right, rgb(59, 130, 246), rgb(6, 182, 212));
    -webkit-background-clip: text; background-clip: text; color: transparent;
  }}
  .mission-card-vision .mission-card-title {{ background: linear-gradient(to right, rgb(168, 85, 247), rgb(236, 72, 153)); -webkit-background-clip: text; background-clip: text; color: transparent; }}
  .mission-card-values .mission-card-title {{ background: linear-gradient(to right, rgb(16, 185, 129), rgb(5, 150, 105)); -webkit-background-clip: text; background-clip: text; color: transparent; }}
  .mission-card-body {{ font-size: 1rem; line-height: 1.7; color: rgb(63, 63, 70); margin: 0; }}
  .mission-card-values-list {{ list-style: none; padding: 0; margin: 0; display: flex; flex-direction: column; gap: 0.5rem; }}
  .mission-card-values-list li {{ display: flex; align-items: center; gap: 0.5rem; font-size: 0.9375rem; color: rgb(63, 63, 70); }}
  .mission-value-dot {{ color: rgb(16, 185, 129); font-weight: 700; }}

  .about-stats-section {{ padding: 5rem 0; }}
  .about-stats-grid {{ display: grid; grid-template-columns: repeat(auto-fit, minmax(220px, 1fr)); gap: 1.5rem; margin-top: 2.5rem; }}
  .about-stat-card {{ padding: 2rem 1.5rem; text-align: center; }}
  .about-stat-icon {{ display: inline-flex; padding: 0.5rem; border-radius: 0.5rem; background: rgba(251, 146, 60, 0.10); margin-bottom: 1rem; }}
  .about-stat-value {{
    font-size: 2.5rem; font-weight: 800; line-height: 1; margin-bottom: 0.5rem;
    background: linear-gradient(to right, rgb(249, 115, 22), rgb(234, 179, 8));
    -webkit-background-clip: text; background-clip: text; color: transparent;
  }}
  .about-stat-label {{ font-size: 0.875rem; color: rgb(113, 113, 122); font-weight: 500; }}

  .team-section {{ padding: 5rem 0; }}
  .about-team-grid {{ display: grid; grid-template-columns: repeat(auto-fit, minmax(260px, 1fr)); gap: 1.5rem; margin-top: 2.5rem; }}
  .about-team-card {{ padding: 2rem 1.5rem; text-align: center; }}
  .about-team-avatar {{
    width: 5rem; height: 5rem; border-radius: 9999px; margin: 0 auto 1rem;
    background: linear-gradient(135deg, #f97316, #f59e0b);
  }}
  .about-team-avatar-1 {{ background: linear-gradient(135deg, #f97316, #f59e0b); }}
  .about-team-avatar-2 {{ background: linear-gradient(135deg, #3b82f6, #06b6d4); }}
  .about-team-avatar-3 {{ background: linear-gradient(135deg, #a855f7, #ec4899); }}
  .about-team-avatar-4 {{ background: linear-gradient(135deg, #10b981, #06b6d4); }}
  .about-team-avatar-5 {{ background: linear-gradient(135deg, #f59e0b, #ef4444); }}
  .about-team-avatar-6 {{ background: linear-gradient(135deg, #6366f1, #a855f7); }}
  .about-team-name {{ font-size: 1.125rem; font-weight: 700; color: rgb(24, 24, 27); margin-bottom: 0.25rem; }}
  .about-team-role {{ font-size: 0.875rem; color: rgb(249, 115, 22); font-weight: 600; margin-bottom: 0.75rem; }}
  .about-team-bio {{ font-size: 0.875rem; line-height: 1.6; margin: 0; }}

  .timeline-section {{ padding: 5rem 0; }}
  .about-timeline {{ max-width: 48rem; margin: 2.5rem auto 0; position: relative; padding-left: 2rem; }}
  .about-timeline::before {{
    content: ""; position: absolute; left: 0.5rem; top: 0.5rem; bottom: 0.5rem;
    width: 2px; background: linear-gradient(to bottom, rgba(251, 146, 60, 0.40), rgba(168, 85, 247, 0.20));
  }}
  .about-timeline-item {{ position: relative; padding-bottom: 2.5rem; }}
  .about-timeline-item:last-child {{ padding-bottom: 0; }}
  .about-timeline-dot {{
    position: absolute; left: -2rem; top: 0.25rem; width: 1.25rem; height: 1.25rem;
    border-radius: 9999px; background: white; border: 3px solid rgb(249, 115, 22);
    box-shadow: 0 0 0 4px rgba(251, 146, 60, 0.10);
  }}
  .about-timeline-dot-current {{ background: rgb(249, 115, 22); animation: pulse 2s ease-in-out infinite; }}
  .about-timeline-year {{
    display: inline-block; padding: 0.25rem 0.75rem; border-radius: 9999px;
    background: rgba(251, 146, 60, 0.10); color: rgb(194, 65, 12);
    font-size: 0.75rem; font-weight: 700; margin-bottom: 0.5rem;
  }}
  .about-timeline-title {{ font-size: 1.25rem; font-weight: 700; color: rgb(24, 24, 27); margin: 0 0 0.5rem; }}
  .about-timeline-body {{ font-size: 0.9375rem; line-height: 1.7; color: rgb(63, 63, 70); margin: 0; }}

  .datatech-section {{ padding: 3rem 0; }}
  @media (min-width: 640px) {{ .datatech-section {{ padding: 6rem 0; }} }}
  .datatech-overview-grid {{ display: grid; grid-template-columns: 1fr; gap: 1.5rem; }}
  @media (min-width: 1024px) {{
    .datatech-overview-grid {{ grid-template-columns: repeat(3, minmax(0, 1fr)); }}
    .datatech-card-definition {{ grid-column: span 2 / span 2; }}
  }}
  .datatech-card {{
    padding: 1.5rem; position: relative; overflow: hidden;
    border-radius: 1.5rem; box-shadow: 0 1px 2px 0 rgba(0, 0, 0, 0.05);
  }}
  .datatech-card-title {{
    font-size: 1.5rem; line-height: 2rem; font-weight: 700; margin: 0 0 1rem;
    background: linear-gradient(to right, rgb(249, 115, 22), rgb(234, 179, 8));
    -webkit-background-clip: text; background-clip: text; color: transparent;
  }}
  .datatech-card-body {{ font-size: 1rem; line-height: 1.625; color: rgb(63, 63, 70); margin: 0 0 1rem; }}
  .datatech-card-definition .datatech-card-body:first-of-type {{ font-size: 1.125rem; line-height: 1.625; }}
  .datatech-card-body:last-child {{ margin-bottom: 0; }}
  .datatech-highlight {{ background: linear-gradient(to right, rgb(249, 115, 22), rgb(234, 179, 8)); -webkit-background-clip: text; background-clip: text; color: transparent; font-weight: 700; }}
  .datatech-text-orange {{ color: rgb(194, 65, 12); font-weight: 600; }}
  .datatech-text-blue {{ color: rgb(29, 78, 216); font-weight: 600; }}
  .datatech-why-list {{ list-style: none; padding: 0; margin: 0; display: flex; flex-direction: column; gap: 0.75rem; }}
  .datatech-why-list li {{ display: flex; align-items: center; gap: 0.5rem; font-size: 0.875rem; line-height: 1.25rem; color: rgb(63, 63, 70); }}
  .datatech-why-check {{ color: rgb(16, 185, 129); font-weight: 700; }}
  .datatech-card-why {{ border-color: rgba(147, 197, 253, 0.50); }}
  html.dark .about-page .datatech-card-why {{ border-color: rgba(96, 165, 250, 0.20); }}
  .datatech-card-why .datatech-card-title {{ background: linear-gradient(to right, rgb(59, 130, 246), rgb(6, 182, 212)); -webkit-background-clip: text; background-clip: text; color: transparent; }}
  .datatech-card-why .datatech-card-title {{ font-size: 1.25rem; line-height: 1.75rem; }}
  @media (max-width: 639px) {{
    .about-page .datatech-card {{
      margin-left: 0.5rem; margin-right: 0.5rem;
      border-radius: 1.5rem;
    }}
    .about-page .datatech-features-grid {{ padding-left: 0.5rem; padding-right: 0.5rem; }}
  }}

  .datatech-features-grid {{ display: grid; grid-template-columns: repeat(auto-fit, minmax(280px, 1fr)); gap: 1.5rem; margin-top: 2rem; }}
  @media (min-width: 640px) {{ .datatech-features-grid {{ gap: 2rem; padding-left: 0; padding-right: 0; }} }}
  @media (min-width: 1024px) {{ .datatech-features-grid {{ grid-template-columns: repeat(3, minmax(0, 1fr)); }} }}
  .datatech-feature {{
    padding: 1.5rem; position: relative; overflow: hidden;
    border-color: rgba(254, 215, 170, 0.30);
    box-shadow: 0 1px 2px 0 rgba(0, 0, 0, 0.05);
  }}
  html.dark .about-page .datatech-feature {{ border-color: rgba(251, 146, 60, 0.20); }}
  .datatech-feature-title {{
    font-size: 1.25rem; line-height: 1.75rem; font-weight: 700; margin: 0 0 1.5rem; background-clip: text; -webkit-background-clip: text; color: transparent;
  }}
  @media (min-width: 640px) {{ .datatech-feature-title {{ font-size: 1.5rem; line-height: 2rem; }} }}
  .datatech-feature-orange .datatech-feature-title {{ background-image: linear-gradient(to right, rgb(249, 115, 22), rgb(234, 179, 8)); }}
  .datatech-feature-blue .datatech-feature-title   {{ background-image: linear-gradient(to right, rgb(59, 130, 246), rgb(6, 182, 212)); }}
  .datatech-feature-purple .datatech-feature-title {{ background-image: linear-gradient(to right, rgb(168, 85, 247), rgb(236, 72, 153)); }}
  .datatech-feature-green .datatech-feature-title  {{ background-image: linear-gradient(to right, rgb(16, 185, 129), rgb(5, 150, 105)); }}
  .datatech-feature-red .datatech-feature-title    {{ background-image: linear-gradient(to right, rgb(239, 68, 68), rgb(249, 115, 22)); }}
  .datatech-feature-indigo .datatech-feature-title {{ background-image: linear-gradient(to right, rgb(99, 102, 241), rgb(168, 85, 247)); }}
  .datatech-feature-body {{ font-size: 1rem; line-height: 1.625; color: #4b5563; margin: 0 0 1rem; }}
  .datatech-feature-detail {{ font-size: 0.875rem; line-height: 1.625; color: #6b7280; margin: 0; }}
  html.dark .about-page .datatech-feature-body {{ color: #d1d5db; }}
  html.dark .about-page .datatech-feature-detail {{ color: #9ca3af; }}

  .datatech-benefits {{ padding: 2.5rem 2rem; margin-top: 2rem; position: relative; overflow: hidden; }}
  .datatech-benefits-title {{
    font-size: 1.75rem; font-weight: 800; margin: 0 0 1.5rem;
    background: linear-gradient(to right, rgb(16, 185, 129), rgb(5, 150, 105));
    -webkit-background-clip: text; background-clip: text; color: transparent;
  }}
  .datatech-benefits-grid {{ display: grid; grid-template-columns: 1fr; gap: 1.5rem; }}
  @media (min-width: 640px) {{ .datatech-benefits-grid {{ grid-template-columns: 1fr 1fr; }} }}
  .datatech-benefits-col {{ display: flex; flex-direction: column; gap: 1rem; }}
  .datatech-benefit-item {{ display: flex; align-items: center; gap: 0.75rem; font-size: 0.9375rem; color: rgb(63, 63, 70); font-weight: 500; }}
  .datatech-benefit-emoji {{ font-size: 1.25rem; }}

  .about-cta-section {{ padding: 5rem 0; }}
  .about-cta-card {{ padding: 3rem 2rem; text-align: center; display: flex; flex-direction: column; gap: 1.5rem; align-items: center; }}
  .about-cta-title {{ font-size: 2rem; font-weight: 800; margin: 0; color: white; }}
  .about-cta-sub {{ font-size: 1.125rem; color: rgba(255, 255, 255, 0.85); margin: 0; max-width: 36rem; }}
  .about-cta-actions {{ display: flex; flex-wrap: wrap; gap: 0.75rem; justify-content: center; }}

  /* === Section header (shared) === */
  .section-header {{ text-align: center; max-width: 48rem; margin: 0 auto 2.5rem; }}
  .section-title {{ font-size: 2.5rem; font-weight: 800; line-height: 1.2; margin: 0 0 0.75rem; color: rgb(24, 24, 27); }}
  .section-sub {{ font-size: 1.125rem; line-height: 1.6; color: rgb(113, 113, 122); margin: 0; }}

  /* end wave5-page-depth-track-a */

  /* === wave5-page-depth-track-b ===
   * Info-pages depth — 9 static / utility pages (manual, plans,
   * contact, privacy, terms, not_found, error_page, offline,
   * access_denied). All rules below are scoped to the new
   * section-marker class names added by the Track B page
   * ports in shared/rust/dioxus_ui/src/pages/.rs. We deliberately
   * reuse the existing design-system classes (`.card`,
   * `.card-glass`, `.btn`, `.btn-primary`, `.btn-outline`,
   * `.btn-gradient`, `.section-title`, `.section-sub`, `.orb-*`,
   * `.text-muted-foreground`, etc.) — only the new Wave 5
   * surface-area selectors are defined here.
   *
   * No new colors, no new design tokens. CSS is appended cleanly
   * so the integration agent can concatenate Track A + Track B
   * blocks (each marked) without conflicts. */

  /* --- /manual --- two-column layout: sticky sidebar + 8-category sections --- */
  .manual-page {{ max-width: 1280px; }}
  .manual-grid {{ display: grid; grid-template-columns: 16rem 1fr; gap: 2rem; align-items: start; }}
  @media (max-width: 900px) {{ .manual-grid {{ grid-template-columns: 1fr; }} }}
  .manual-sidebar {{ position: sticky; top: 5rem; }}
  .manual-sidebar-card {{ padding: 0; overflow: hidden; }}
  .manual-nav {{ display: flex; flex-direction: column; gap: 0.25rem; }}
  .manual-nav-link {{
    padding: 0.5rem 0.75rem; border-radius: 0.5rem; font-size: 0.875rem;
    color: var(--text-muted, #94a3b8); text-decoration: none;
    transition: background 0.15s ease, color 0.15s ease;
  }}
  .manual-nav-link:hover {{ background: rgba(255, 255, 255, 0.06); color: var(--text, #fff); }}
  .manual-content {{ display: flex; flex-direction: column; gap: 1.5rem; }}
  .manual-category-details {{
    border: 1px solid var(--glass-border, rgba(255, 255, 255, 0.08));
    border-radius: 1rem; background: var(--glass-bg, rgba(255, 255, 255, 0.04));
    padding: 0.5rem 1rem;
  }}
  .manual-category-details > summary {{
    list-style: none; cursor: pointer; padding: 0.75rem 0;
    display: flex; align-items: center; justify-content: space-between;
  }}
  .manual-category-details > summary::-webkit-details-marker {{ display: none; }}
  .manual-category-title {{ font-size: 1.25rem; font-weight: 700; margin: 0; }}
  .manual-category-count {{ font-size: 0.75rem; color: var(--text-muted, #94a3b8); }}
  .manual-feature-grid {{
    display: grid; grid-template-columns: 1fr;
    gap: 1.5rem; padding: 0.5rem 0 1rem;
  }}
  @media (min-width: 640px) {{
    .manual-feature-grid {{ grid-template-columns: repeat(2, minmax(0, 1fr)); }}
  }}
  @media (min-width: 1024px) {{
    .manual-feature-grid {{ grid-template-columns: repeat(3, minmax(0, 1fr)); }}
  }}
  .manual-feature-card {{ display: flex; flex-direction: column; padding: 0; overflow: hidden; }}
  .manual-feature-screenshot {{
    aspect-ratio: 16 / 9; background: rgba(0,0,0,0.2);
    display: flex; align-items: center; justify-content: center;
  }}
  .screenshot-img {{ width: 100%; height: 100%; object-fit: cover; object-position: top; }}
  .screenshot-img-fallback {{
    position: absolute; font-size: 0.75rem; color: var(--text-muted, #94a3b8);
  }}
  .screenshot-img-wrap {{ position: relative; width: 100%; height: 100%; }}
  .manual-feature-body {{ padding: 0.75rem 1rem 1rem; display: flex; flex-direction: column; gap: 0.25rem; }}
  .manual-feature-head {{ display: flex; align-items: center; gap: 0.5rem; }}
  .manual-feature-name {{ font-weight: 600; margin: 0; }}
  .manual-feature-route {{
    font-family: ui-monospace, SFMono-Regular, Menlo, monospace;
    font-size: 0.7rem; color: var(--text-muted, #94a3b8);
    background: rgba(255,255,255,0.04); padding: 0.1rem 0.4rem; border-radius: 0.25rem;
  }}
  .manual-feature-desc {{ margin: 0.25rem 0 0.5rem; }}
  .manual-feature-link {{ font-size: 0.875rem; color: #60a5fa; text-decoration: none; }}
  .manual-feature-link:hover {{ color: #93c5fd; }}
  .manual-summary {{ margin: 0 0 1.5rem; }}
  .manual-summary-title {{ font-size: 2rem; font-weight: 700; margin: 0 0 0.5rem; }}
  .manual-summary-subtitle {{ margin: 0 0 0.75rem; }}
  .manual-summary-meta {{ display: flex; gap: 0.5rem; align-items: center; font-size: 0.875rem; color: var(--text-muted, #94a3b8); }}
  .manual-summary-count, .manual-summary-categories {{ font-weight: 500; }}
  .manual-cta {{ margin-top: 3rem; }}
  .manual-cta-card {{ padding: 2rem; }}

  /* --- /plans --- hero, 3-tier grid, comparison table, FAQ, enterprise CTA --- */
  .plans-hero {{ padding: 3rem 0 2rem; text-align: center; }}
  .plans-hero-title {{
    font-size: 2.5rem; font-weight: 800; margin: 0 0 0.75rem;
    background: linear-gradient(90deg, #10b981, #3b82f6, #a855f7);
    -webkit-background-clip: text; background-clip: text; color: transparent;
  }}
  .plans-hero-subtitle {{ color: var(--text-muted, #94a3b8); max-width: 48rem; margin: 0 auto; font-size: 1.125rem; }}
  .plans-grid-section {{ padding: 2rem 0; }}
  .plan-card {{ padding: 1.5rem; transition: transform 0.2s ease, box-shadow 0.2s ease; }}
  .plan-card.card-featured {{ transform: translateY(-2px); }}
  .plan-features {{ list-style: none; padding: 0; margin: 0; }}
  .plans-comparison-section {{ padding: 3rem 0; }}
  .plans-comparison-table-wrap {{ overflow-x: auto; margin-top: 1.5rem; }}
  .plans-comparison-table {{ width: 100%; border-collapse: collapse; }}
  .plans-comparison-table th, .plans-comparison-table td {{
    padding: 0.75rem 1rem; text-align: left;
    border-bottom: 1px solid var(--epsx-border, rgba(255,255,255,0.08));
  }}
  .plans-comparison-table thead th {{ font-weight: 600; }}
  .plans-comparison-feature-col {{ width: 40%; }}
  .plans-comparison-col-featured {{ color: var(--epsx-primary, #3b82f6); }}
  .plans-comparison-yes {{ color: #10b981; font-weight: 700; }}
  .plans-comparison-no {{ color: var(--text-muted, #94a3b8); }}
  .plans-faq-section {{ padding: 3rem 0; }}
  .plans-faq-list {{ max-width: 48rem; margin: 2rem auto 0; display: flex; flex-direction: column; gap: 0.75rem; }}
  .plans-faq-item {{ padding: 0; overflow: hidden; }}
  .plans-faq-question {{
    list-style: none; cursor: pointer; padding: 1rem 1.25rem;
  }}
  .plans-faq-question::-webkit-details-marker {{ display: none; }}
  .plans-faq-question h3 {{ margin: 0; font-size: 1rem; font-weight: 600; }}
  .plans-faq-answer {{ padding: 0 1.25rem 1.25rem; color: var(--text-muted, #94a3b8); }}
  .plans-faq-link {{ color: #10b981; text-decoration: underline; }}
  .plans-enterprise-cta {{ padding: 3rem 0; }}
  .plans-enterprise-cta-card {{ padding: 2.5rem; }}
  .plans-enterprise-cta-title {{ font-size: 1.75rem; font-weight: 700; margin: 0 0 0.5rem; }}
  .plans-enterprise-cta-subtitle {{ margin: 0 0 1.5rem; }}
  .plans-enterprise-cta-actions {{ display: flex; gap: 0.75rem; justify-content: center; flex-wrap: wrap; }}

  /* --- /contact --- gradient background + form + 3 info cards --- */
  .contact-page {{ position: relative; z-index: 1; }}
  .contact-bg {{
    position: fixed; inset: 0; z-index: 0; pointer-events: none; overflow: hidden;
    background: linear-gradient(135deg, #eff6ff 0%, #fff7ed 50%, #fefce8 100%);
  }}
  :root.dark .contact-bg {{
    background: linear-gradient(135deg, #0f172a 0%, #1e293b 50%, #0f172a 100%);
  }}
  .contact-bg::before, .contact-bg::after {{ content: ""; position: absolute; inset: 0; }}
  .contact-bg::before {{ background: radial-gradient(circle at 25% 25%, rgba(168,85,247,0.10) 0%, transparent 50%); }}
  .contact-bg::after {{ background: radial-gradient(circle at 75% 75%, rgba(255,133,27,0.08) 0%, transparent 50%); }}
  .contact-bg-orb {{ position: absolute; border-radius: 9999px; filter: blur(64px); opacity: 1; }}
  .contact-bg-orb-1 {{ width: 24rem; height: 24rem; top: -10rem; left: -10rem; background: linear-gradient(to bottom right, rgba(192,132,252,0.30), rgba(244,114,182,0.30)); }}
  .contact-bg-orb-2 {{ width: 20rem; height: 20rem; top: 5rem; right: -8rem; background: linear-gradient(to bottom right, rgba(251,146,60,0.25), rgba(250,204,21,0.25)); }}
  .contact-bg-orb-3 {{ width: 18rem; height: 18rem; bottom: 5rem; left: 5rem; background: linear-gradient(to bottom right, rgba(96,165,250,0.20), rgba(34,211,238,0.20)); }}
  .contact-bg-orb-4 {{ display: none; }}
  .contact-hero {{ padding: 4rem 0 2rem; text-align: center; position: relative; }}
  .contact-hero-title {{
    font-size: 3rem; line-height: 1; font-weight: 700; margin: 0 0 1rem;
    background: linear-gradient(90deg, #a855f7, #f97316, #eab308);
    -webkit-background-clip: text; background-clip: text; color: transparent;
  }}
  @media (min-width: 640px) {{ .contact-hero-title {{ font-size: 3.75rem; }} }}
  .contact-hero-subtitle {{ max-width: 42rem; margin: 0 auto; color: var(--text-muted, #475569); font-size: 1.125rem; line-height: 1.625; }}
  .contact-hero-divider {{
    width: 10rem; height: 0.25rem; margin: 1.5rem auto 0;
    background: linear-gradient(90deg, #a855f7, #f97316, #eab308); border-radius: 9999px;
  }}
  .contact-email-section {{ padding: 0 0 3rem; }}
  .contact-email-card {{
    max-width: 32rem; margin: 0 auto; padding: 2rem; text-align: center;
    background: rgba(255, 255, 255, 0.8); backdrop-filter: blur(20px);
    border: 1px solid rgba(168, 85, 247, 0.2);
    border-radius: 1.5rem; box-shadow: 0 25px 50px -12px rgba(0,0,0,0.25);
  }}
  :root.dark .contact-email-card {{ background: rgba(30, 41, 59, 0.8); border-color: rgba(168, 85, 247, 0.3); }}
  .contact-email-icon {{
    display: inline-flex; padding: 1rem; border-radius: 1rem;
    background: linear-gradient(135deg, #a855f7, #f97316); margin-bottom: 1.25rem;
  }}
  .contact-email-title {{ font-size: 1.25rem; line-height: 1.75rem; font-weight: 700; margin: 0 0 0.5rem; color: #1f2937; }}
  .contact-email-subtitle {{ margin: 0 0 1.25rem; font-size: 0.875rem; line-height: 1.25rem; color: #6b7280; }}
  :root.dark .contact-email-title {{ color: #f3f4f6; }}
  :root.dark .contact-email-subtitle {{ color: #9ca3af; }}
  .contact-mailto-btn {{
    display: inline-flex; gap: 0.5rem; align-items: center;
    padding: 0.75rem 2rem; border-radius: 0.75rem; font-weight: 600;
    font-size: 1rem; line-height: 1.5rem;
    background: linear-gradient(to right, #a855f7, #f97316);
  }}
  .contact-email-divider {{ height: 0; background: none; margin: 1rem 0 0; }}
  :root.dark .contact-email-divider {{ background: rgba(255,255,255,0.06); }}
  .contact-copy-btn {{
    display: inline-flex; gap: 0.375rem; align-items: center;
    padding: 0; border: 0; border-radius: 0;
    font-size: 0.875rem; line-height: 1.25rem; color: #a855f7;
  }}
  .contact-info-section {{ padding: 0 0 4rem; }}
  .contact-info-grid {{ display: grid; grid-template-columns: repeat(auto-fit, minmax(220px, 1fr)); gap: 1rem; max-width: 56rem; margin: 0 auto; }}
  .contact-info-card {{
    background: rgba(255,255,255,0.8); backdrop-filter: blur(20px);
    border: 1px solid rgba(168, 85, 247, 0.20);
    border-radius: 1.5rem; box-shadow: 0 10px 25px -5px rgba(0,0,0,0.1);
    padding: 0;
  }}
  .contact-info-card-purple {{ border-color: rgba(233, 213, 255, 0.50); }}
  .contact-info-card-orange {{ border-color: rgba(254, 215, 170, 0.50); }}
  .contact-info-card-blue {{ border-color: rgba(191, 219, 254, 0.50); }}
  :root.dark .contact-info-card {{ background: rgba(30, 41, 59, 0.8); }}
  :root.dark .contact-info-card-purple {{ border-color: rgba(192, 132, 252, 0.20); }}
  :root.dark .contact-info-card-orange {{ border-color: rgba(251, 146, 60, 0.20); }}
  :root.dark .contact-info-card-blue {{ border-color: rgba(96, 165, 250, 0.20); }}
  .contact-info-row {{ display: flex; gap: 1rem; align-items: flex-start; padding: 1.5rem; }}
  .contact-info-icon {{
    display: inline-flex; padding: 0.75rem; border-radius: 1rem; flex-shrink: 0;
  }}
  .contact-info-icon-purple {{ background: linear-gradient(135deg, #a855f7, #3b82f6); }}
  .contact-info-icon-orange {{ background: linear-gradient(135deg, #f97316, #eab308); }}
  .contact-info-icon-blue {{ background: linear-gradient(135deg, #3b82f6, #06b6d4); }}
  .contact-info-title {{ font-weight: 600; margin: 0 0 0.25rem; color: #1f2937; }}
  .contact-info-desc {{ margin: 0; color: #6b7280; }}
  :root.dark .contact-info-title {{ color: #f3f4f6; }}
  :root.dark .contact-info-desc {{ color: #9ca3af; }}
  .contact-form-section {{ padding: 0 0 4rem; }}
  .contact-form-card {{ max-width: 48rem; margin: 0 auto; padding: 0; }}
  .contact-form-title {{ font-size: 1.5rem; font-weight: 700; margin: 0 0 0.5rem; }}
  .contact-form-subtitle {{ margin: 0 0 1.5rem; }}
  .contact-form {{ display: flex; flex-direction: column; gap: 1rem; }}
  .contact-form-row {{ display: grid; grid-template-columns: 1fr 1fr; gap: 1rem; }}
  @media (max-width: 600px) {{ .contact-form-row {{ grid-template-columns: 1fr; }} }}
  .contact-form-actions {{ display: flex; justify-content: flex-end; }}

  /* --- /privacy, /terms --- legal pages: hero + sticky TOC + sections --- */
  .legal-page {{ max-width: 56rem; }}
  .legal-hero {{ text-align: center; padding: 3rem 0 2rem; }}
  .legal-hero-title {{
    font-size: 2.5rem; font-weight: 800; margin: 0 0 0.5rem;
    background: linear-gradient(90deg, #a855f7, #ec4899);
    -webkit-background-clip: text; background-clip: text; color: transparent;
  }}
  .legal-hero-subtitle {{ margin: 0; }}
  .legal-toc {{
    display: flex; flex-wrap: wrap; gap: 0.5rem; align-items: center;
    padding: 0.75rem 1rem; margin: 1rem 0 2rem;
    background: var(--glass-bg, rgba(255,255,255,0.04));
    border: 1px solid var(--glass-border, rgba(255,255,255,0.08));
    border-radius: 0.75rem;
  }}
  .legal-toc-label {{ font-size: 0.875rem; color: var(--text-muted, #94a3b8); margin-right: 0.5rem; }}
  .legal-toc-link {{
    font-size: 0.875rem; color: var(--text, #fff); text-decoration: none;
    padding: 0.25rem 0.6rem; border-radius: 9999px;
    background: rgba(255,255,255,0.04);
  }}
  .legal-toc-link:hover {{ background: rgba(255,255,255,0.1); }}
  .legal-sections {{ display: flex; flex-direction: column; gap: 2rem; }}
  .legal-section-title {{
    font-size: 1.5rem; font-weight: 700; margin: 0 0 0.75rem; color: #a855f7;
  }}
  .legal-section-text {{ margin: 0 0 0.75rem; line-height: 1.7; }}
  .legal-section-list {{ padding-left: 1.5rem; margin: 0 0 0.75rem; line-height: 1.8; }}
  .legal-link {{ color: #60a5fa; text-decoration: underline; }}
  .legal-footer {{
    display: flex; gap: 0.75rem; justify-content: center;
    margin-top: 3rem; padding-top: 2rem; border-top: 1px solid rgba(255,255,255,0.08);
  }}
  .terms-subscribe-section {{ margin-top: 2rem; }}
  .terms-subscribe-card {{ padding: 0; }}
  .terms-subscribe-title {{ font-size: 1.25rem; font-weight: 700; margin: 0 0 0.5rem; color: #a855f7; }}
  .terms-subscribe-subtitle {{ margin: 0 0 1rem; }}
  .terms-subscribe-form {{ display: flex; gap: 0.75rem; align-items: flex-end; flex-wrap: wrap; }}

  /* --- /not-found, /error, /offline --- utility pages --- */
  .not-found {{ text-align: center; padding: 4rem 1rem; max-width: 42rem; margin: 0 auto; }}
  .not-found-code {{
    font-size: 6rem; font-weight: 900; line-height: 1;
    background: linear-gradient(135deg, #a855f7, #3b82f6);
    -webkit-background-clip: text; background-clip: text; color: transparent;
    margin-bottom: 0.5rem;
  }}
  .not-found-title {{ font-size: 2rem; font-weight: 700; margin: 0 0 0.5rem; }}
  .not-found-description {{ margin: 0 0 1.5rem; }}
  .not-found-actions {{ display: flex; gap: 0.75rem; justify-content: center; flex-wrap: wrap; margin-bottom: 2rem; }}
  .not-found-illustration {{ display: flex; justify-content: center; color: var(--text-muted, #94a3b8); margin: 1rem 0; }}
  .not-found-destinations {{ margin-top: 2rem; }}
  .not-found-destinations-title {{ font-size: 1.125rem; font-weight: 600; margin: 0 0 1rem; }}
  .not-found-destinations-grid {{ display: grid; grid-template-columns: repeat(auto-fit, minmax(120px, 1fr)); gap: 0.75rem; }}
  .not-found-destination {{
    display: flex; flex-direction: column; align-items: center; gap: 0.5rem;
    padding: 1rem; text-decoration: none; color: inherit;
    transition: transform 0.2s ease, box-shadow 0.2s ease;
  }}
  .not-found-destination:hover {{ transform: translateY(-2px); }}

  .error-page {{ text-align: center; padding: 4rem 1rem; max-width: 42rem; margin: 0 auto; }}
  .error-page-illustration {{ display: flex; justify-content: center; margin-bottom: 1rem; }}
  .error-page-icon {{
    display: inline-flex; padding: 1rem; border-radius: 9999px; background: rgba(0,0,0,0.05);
  }}
  :root.dark .error-page-icon {{ background: rgba(255,255,255,0.06); }}
  .error-page-title {{ font-size: 2rem; font-weight: 700; margin: 0 0 0.5rem; }}
  .error-page-subtitle {{ margin: 0 0 1rem; }}
  .error-page-message {{
    font-family: ui-monospace, SFMono-Regular, Menlo, monospace;
    font-size: 0.875rem; color: var(--text-muted, #94a3b8);
    background: rgba(0,0,0,0.05); border-radius: 0.5rem; padding: 0.75rem 1rem;
    margin: 0 0 1.5rem; text-align: left; word-break: break-word;
  }}
  :root.dark .error-page-message {{ background: rgba(255,255,255,0.05); }}
  .error-page-actions {{ display: flex; gap: 0.75rem; justify-content: center; flex-wrap: wrap; margin-top: 1rem; }}
  .error-page-hints {{ margin: 1.5rem auto; max-width: 24rem; text-align: left; }}
  .error-page-hints-label {{ margin: 0 0 0.5rem; color: var(--text-muted, #94a3b8); }}
  .error-page-hints ul {{ padding-left: 1.5rem; margin: 0; line-height: 1.7; color: var(--text-muted, #94a3b8); }}

  .offline-page {{
    min-height: calc(100vh + 7rem); display: flex; align-items: center; justify-content: center;
    padding: 2rem 1rem;
    background: linear-gradient(135deg, #f8fafc, #f1f5f9);
  }}
  :root.dark .offline-page {{ background: linear-gradient(135deg, #0f172a, #1e293b); }}
  .offline-card {{
    max-width: 28rem; width: 100%; padding: 2rem 1.5rem; text-align: center;
    border: 1px solid rgba(148, 163, 184, 0.38);
  }}
  :root.dark .offline-card {{
    border-color: rgba(148, 163, 184, 0.55);
    background: rgba(15, 23, 42, 0.54);
  }}
  .offline-icon {{
    display: inline-flex; padding: 1.25rem; border-radius: 9999px;
    background: rgba(249, 115, 22, 0.1); color: #f97316; margin-bottom: 1.5rem;
  }}
  .offline-icon svg {{ width: 2.5rem; height: 2.5rem; }}
  :root.dark .offline-icon {{ background: rgba(249, 115, 22, 0.2); }}
  .offline-title {{ font-size: 1.5rem; font-weight: 700; margin: 0 0 0.5rem; }}
  .offline-subtitle {{ margin: 0 0 1.5rem; }}
  .offline-available {{
    background: rgba(0,0,0,0.03); border-radius: 0.75rem; padding: 1rem 1.25rem;
    text-align: left; margin: 0 0 1.5rem;
  }}
  :root.dark .offline-available {{ background: rgba(255,255,255,0.04); }}
  .offline-available-title {{ font-size: 0.875rem; font-weight: 500; margin: 0 0 0.75rem; }}
  .offline-available-list {{ list-style: none; padding: 0; margin: 0; display: flex; flex-direction: column; gap: 0.5rem; font-size: 0.875rem; }}
  .offline-available-item {{ display: flex; gap: 0.5rem; align-items: center; }}
  .offline-available-dot {{ width: 0.5rem; height: 0.5rem; border-radius: 9999px; flex-shrink: 0; }}
  .offline-available-dot-yes {{ background: #10b981; }}
  .offline-available-dot-limited {{ background: #f97316; }}
  .offline-actions {{ display: flex; flex-direction: column; align-items: center; gap: 0.75rem; }}
  .offline-retry {{
    display: inline-flex; align-items: center; justify-content: center; gap: 0.5rem;
    width: auto; min-height: 2.75rem; padding: 0.5rem 1.25rem;
    border: 0; background: transparent; color: var(--text); box-shadow: none;
    font-weight: 600;
  }}
  .offline-retry:hover {{ background: transparent; color: #f97316; box-shadow: none; }}
  .offline-retry:focus-visible {{ outline: 2px solid #f97316; outline-offset: 2px; }}
  .offline-actions-row {{ display: flex; width: 100%; gap: 0.5rem; }}
  .offline-actions-row .btn {{ flex: 1; display: inline-flex; align-items: center; justify-content: center; gap: 0.4rem; }}
  .offline-tip {{
    margin-top: 1.5rem; padding-top: 1rem; border-top: 1px solid rgba(0,0,0,0.06);
    font-size: 0.75rem; color: var(--text-muted, #94a3b8);
  }}
  :root.dark .offline-tip {{ border-top-color: rgba(255,255,255,0.06); }}
  .offline-tip-label {{ font-weight: 500; margin: 0 0 0.25rem; }}
  .offline-tip-text {{ margin: 0; }}

  .access-denied-page {{ max-width: none; margin: 0; padding: 0; }}
  .access-denied-page > .access-denied {{ min-height: 100vh; }}
  .access-denied-reasons {{ margin-top: 2rem; }}
  .access-denied-reasons-card {{ padding: 0; }}
  .access-denied-reasons-title {{ font-size: 1rem; font-weight: 600; margin: 0 0 0.75rem; }}
  .access-denied-reasons-list {{ list-style: none; padding: 0; margin: 0; display: flex; flex-direction: column; gap: 0.5rem; }}
  .access-denied-reasons-item {{ display: flex; gap: 0.5rem; align-items: flex-start; font-size: 0.875rem; color: var(--text-muted, #94a3b8); }}
  .access-denied-reasons-bullet {{ color: var(--text-muted, #94a3b8); }}

  /* end wave5-page-depth-track-b */

  /* === wave6-auth-pages-depth-track-a ===
   * Wave 6A Track A — auth-required pages depth: dashboard + account
   * + account_credits. All rules below are scoped to the new
   * section-marker class names added by the Track A page ports in
   * `shared/rust/dioxus_ui/src/pages/{{dashboard,account,account_credits}}.rs`.
   * We deliberately reuse the existing design-system classes
   * (`.card`, `.card-glass`, `.card-header`, `.card-body`,
   * `.btn`, `.tab`, etc.) — these are the few genuinely new rules
   * that the markers introduced. The marker region is the only
   * shared file surface with Tracks B/C/D (which use
   * `// === wave6-auth-pages-depth-track-b/c/d ===`). */

  /* === dashboard === */
  .stat-cards-row {{ /* layout: same as inline grid, no extra rules */ }}
  .dashboard-earnings-chart .chart {{ width: 100%; height: auto; }}
  .watchlist-snapshot-row td {{ vertical-align: middle; }}
  .plan-summary-card .progress {{ width: 100%; height: 0.5rem; background: rgba(255, 255, 255, 0.08); border-radius: 999px; overflow: hidden; }}
  .plan-summary-card .progress-bar {{ height: 100%; background: linear-gradient(90deg, #22d3ee, #6366f1); border-radius: 999px; transition: width 0.3s ease; }}
  .your-account-card p {{ margin: 0.25rem 0; }}

  /* === account (6 tabs) === */
  .account-tabs {{ display: flex; flex-wrap: wrap; gap: 0.5rem; }}
  .account-tab {{ display: block; }}
  .notification-toggle-row {{ transition: border-color 0.15s ease, background 0.15s ease; }}
  .notification-toggle-row:hover {{ background: rgba(99, 102, 241, 0.04); }}
  .notification-toggle-input {{ width: 1.25rem; height: 1.25rem; accent-color: #6366f1; cursor: pointer; }}
  .btn-danger {{ background: #ef4444; color: white; border-color: #ef4444; }}
  .btn-danger:hover {{ background: #dc2626; border-color: #dc2626; }}

  /* === account/credits (credit ledger) === */
  .credits-ledger-page {{ /* layout: same as page-content */ }}
  .credits-balance-row {{ margin-bottom: 1.5rem; }}
  .credits-balance-available {{ box-shadow: 0 10px 30px -10px rgba(59, 130, 246, 0.5); }}
  .credits-topup .input {{ width: 100%; }}
  .credits-transaction-list .credits-filter-chip {{ font-size: 0.75rem; }}
  .credits-ledger-row {{ transition: background 0.15s ease; }}
  .credits-ledger-row:hover {{ background: rgba(99, 102, 241, 0.04); }}
  .credits-ledger-row--credit {{ /* default row */ }}
  .credits-ledger-row--debit {{ /* default row */ }}
  .credits-ledger-kind {{ display: block; font-family: ui-monospace, SFMono-Regular, Menlo, monospace; }}

  /* end wave6-auth-pages-depth-track-a */
  /* === wave6-auth-pages-depth-track-b ===
   * Analytics + developer depth — filter panel, export dialog
   * primitive, analytics card grid responsive breakpoints, and
   * developer portal sub-section styles. All rules below are scoped
   * to the new section-marker class names added by the Track B page
   * ports in `shared/rust/dioxus_ui/src/pages/analytics.rs` and
   * `shared/rust/dioxus_ui/src/pages/developer.rs`, and the new
   * `<ExportDialog>` primitive at
   * `shared/rust/dioxus_ui/src/data/export_dialog.rs`. We deliberately
   * reuse the existing design-system classes (`.card`, `.card-glass`,
   * `.btn`, `.btn-primary`, `.btn-outline`, `.input`, `.modal`,
   * `.modal-overlay`, etc.) — only the new Wave 6A surface-area
   * selectors are defined here.
   *
   * No new colors, no new design tokens. CSS is appended cleanly so
   * the integration agent can concatenate Track A + Track B + Track C
   * + Track D blocks (each marked) without conflicts. */

  /* --- <ExportDialog> primitive (data/export_dialog.rs) --- */
  .export-dialog-overlay {{ /* extends .modal-overlay for the analytics export modal */ }}
  .export-dialog {{ max-width: 32rem; width: 100%; }}
  .export-dialog-body {{ display: flex; flex-direction: column; gap: 1rem; }}
  .export-dialog-scopes, .export-dialog-formats {{ gap: 0.5rem; }}
  .export-dialog-scope-btn, .export-dialog-formats > button {{
    padding: 0.375rem 0.75rem;
    border-radius: 0.5rem;
    border: 1px solid var(--epsx-border, rgba(255,255,255,0.1));
    background: var(--bg-secondary, rgba(255,255,255,0.05));
    color: var(--text-muted, #94a3b8);
    font-size: 0.75rem;
    font-weight: 500;
    cursor: pointer;
    transition: background 0.15s ease, color 0.15s ease, border-color 0.15s ease;
  }}
  .export-dialog-scope-btn:hover, .export-dialog-formats > button:hover {{
    background: var(--bg-hover, rgba(255,255,255,0.1));
    color: var(--text, #fff);
  }}
  .export-dialog-scope-btn.active, .export-dialog-formats > button.active {{
    background: rgba(118, 69, 217, 0.2);
    border-color: rgba(118, 69, 217, 0.5);
    color: #c4a8f0;
  }}
  .export-dialog-trigger {{ background: linear-gradient(90deg, #7645d9, #5a33b8); color: #fff; }}

  /* --- /analytics --- section styles --- */
  .analytics-header {{ gap: 0.75rem; }}
  .analytics-header-date {{ font-variant-numeric: tabular-nums; }}
  .analytics-plan-status {{ border-radius: 1rem; }}
  .analytics-filter-panel {{ padding: 0; }}
  .analytics-filter-apply {{ background: linear-gradient(90deg, #7645d9, #5a33b8); }}
  .analytics-card-grid {{ gap: 1rem; }}
  @media (min-width: 640px) {{ .analytics-card-grid {{ gap: 1rem; }} }}
  .analytics-card {{ transition: transform 0.15s ease, box-shadow 0.15s ease; }}
  .analytics-card-tier-premium {{ box-shadow: 0 0 0 1px rgba(168, 85, 247, 0.2); }}
  .analytics-card-tier-standard {{ }}
  .analytics-table .card-body {{ overflow-x: auto; }}
  .analytics-metadata {{ position: relative; }}

  /* --- /developer --- section styles --- */
  .developer-stats-cards {{ margin-bottom: 0; }}
  .api-keys-list {{ overflow: hidden; }}
  .api-key-card {{ position: relative; }}
  .api-key-create-form {{ margin-bottom: 0; }}
  .api-key-create-submit {{ background: linear-gradient(90deg, #7645d9, #5a33b8); color: #fff; }}
  .api-key-create-submit:disabled {{ opacity: 0.5; pointer-events: none; }}
  .plan-transfer-list {{ }}
  .permission-list {{ display: flex; flex-direction: column; gap: 0.5rem; }}
  .permission-list-box {{ scrollbar-width: thin; }}
  .permission-list-display {{ margin-top: 1rem; }}
  .docs-quick-links {{ position: sticky; top: 5rem; }}
  .docs-quick-link {{ transition: background 0.15s ease, color 0.15s ease; text-decoration: none; }}
  .usage-monitor {{ }}
  .usage-monitor .chart {{ margin-top: 0.5rem; }}
  .developer-docs {{ margin-top: 1rem; align-items: flex-start; }}
  .developer-docs-page {{ min-width: 0; }}
  .developer-docs > .min-w-0 {{ min-width: 0; flex: 1 1 auto; }}
  .developer-docs-hero {{ margin-bottom: 2rem; }}
  .developer-docs-auth-card {{ margin-bottom: 2rem; }}
  .developer-docs-curl,
  .docs-code-panel,
  .docs-response-panel {{ max-width: 100%; overflow-x: auto; white-space: pre; }}
  .docs-sidebar {{ width: 14rem; flex: 0 0 14rem; position: sticky; top: 5rem; }}
  .docs-sidebar-toggle,
  .docs-sidebar-overlay {{ display: none; }}
  .docs-sidebar-link {{ text-decoration: none; }}
  .docs-sidebar-link.active {{ background: rgba(118, 69, 217, 0.1); color: #7645d9; font-weight: 500; }}
  .docs-endpoint-section {{ scroll-margin-top: 5rem; margin-bottom: 2.5rem; }}
  .docs-endpoint-card {{ overflow: hidden; margin-top: 0.75rem; }}
  .docs-endpoint-card > button {{ min-width: 0; }}
  .docs-endpoint-card > button code {{ min-width: 0; overflow-wrap: anywhere; }}
  .docs-endpoint-card-chevron {{ flex: 0 0 auto; }}
  .docs-endpoint-card-body[hidden],
  .docs-code-panel[hidden],
  .docs-sidebar-overlay[hidden] {{ display: none !important; }}
  .docs-endpoint-card-body {{ display: grid; gap: 1.25rem; }}
  .docs-endpoint-card-params {{ min-width: 0; }}
  .docs-endpoint-card-params table {{ min-width: 42rem; border-collapse: collapse; }}
  .docs-code-example {{ overflow: hidden; border-radius: 0.75rem; }}
  .docs-code-toolbar {{ display: flex; align-items: center; gap: 0.25rem; padding: 0.5rem 0.75rem; background: #1e293b; }}
  .docs-code-tab,
  .docs-copy-button {{ border: 0; border-radius: 0.375rem; padding: 0.25rem 0.625rem; background: transparent; color: #9ca3af; font-size: 0.75rem; cursor: pointer; }}
  .docs-code-tab.active {{ background: #475569; color: #fff; }}
  .docs-code-tab:focus-visible,
  .docs-copy-button:focus-visible,
  .docs-sidebar-toggle:focus-visible,
  .docs-sidebar-link:focus-visible {{ outline: 2px solid #1fc7d4; outline-offset: 2px; }}
  .docs-copy-button {{ margin-left: auto; background: rgba(255,255,255,0.1); color: #d1d5db; }}
  .docs-code-panel {{ margin: 0; border-radius: 0; padding: 1rem; background: #0f172a; color: #e5e7eb; font: 0.875rem/1.6 ui-monospace, SFMono-Regular, Menlo, monospace; }}
  .docs-response-example {{ position: relative; }}
  .docs-response-copy {{ position: absolute; z-index: 1; top: 0.75rem; right: 0.75rem; }}
  .docs-response-panel {{ margin: 0; border-radius: 0.75rem; padding: 1rem; padding-top: 3.25rem; background: #020617; color: #d1fae5; font: 0.875rem/1.6 ui-monospace, SFMono-Regular, Menlo, monospace; }}
  .docs-try-it {{ overflow: hidden; }}
  .docs-try-it-header {{ padding: 0.75rem 1rem; border-bottom: 1px solid var(--epsx-border); }}
  .docs-try-it-header h4 {{ margin: 0; font-size: 0.875rem; }}
  .docs-try-it-body {{ display: grid; gap: 0.75rem; padding: 1rem; }}
  .docs-field-label {{ display: block; color: var(--text-muted); font-size: 0.75rem; font-weight: 500; }}
  .docs-field-label small {{ opacity: 0.7; }}
  .docs-field-control {{ width: 100%; border: 1px solid var(--epsx-border); border-radius: 0.5rem; padding: 0.625rem 0.75rem; background: var(--bg); color: var(--text); }}
  .docs-field-control:disabled {{ opacity: 0.65; cursor: not-allowed; }}
  .docs-send-button {{ border: 0; border-radius: 0.75rem; padding: 0.625rem 1rem; background: linear-gradient(90deg, #7645d9, #5a33b8); color: white; font-weight: 600; }}
  .docs-send-button:disabled {{ opacity: 0.5; cursor: not-allowed; }}
  .docs-try-it-status {{ margin: 0; color: var(--text-muted); font-size: 0.75rem; }}
  @media (max-width: 1023px) {{
    .developer-docs {{ display: block; }}
    .docs-sidebar {{ position: fixed; z-index: 80; top: 0; left: 0; width: 14rem; height: 100dvh; padding-top: 5rem; background: var(--bg-secondary); border-right: 1px solid var(--epsx-border); transform: translateX(-100%); transition: transform 160ms ease; overflow-y: auto; }}
    .docs-sidebar.open {{ transform: translateX(0); }}
    .docs-sidebar-toggle {{ display: inline-flex; position: fixed; z-index: 90; right: 1rem; bottom: 1rem; width: 3rem; height: 3rem; align-items: center; justify-content: center; border: 0; border-radius: 9999px; background: linear-gradient(90deg, #7645d9, #5a33b8); color: white; box-shadow: 0 10px 25px rgba(0,0,0,0.3); }}
    .docs-sidebar-overlay:not([hidden]) {{ display: block; position: fixed; z-index: 70; inset: 0; border: 0; background: rgba(0,0,0,0.4); }}
  }}
  @media (max-width: 639px) {{
    .developer-docs-page {{ padding-left: 0.75rem; padding-right: 0.75rem; }}
    .developer-docs-hero h1 {{ font-size: 1.875rem; }}
    .developer-docs-auth-card {{ padding: 1rem; }}
    .docs-endpoint-card > button {{ gap: 0.5rem; padding: 0.875rem; }}
    .docs-endpoint-card-body {{ padding: 0.875rem; }}
    .docs-code-toolbar {{ flex-wrap: wrap; }}
  }}

  /* end wave6-auth-pages-depth-track-b */
  /* === wave6-auth-pages-depth-track-c ===
   * Chat + chat_history + chat_conversation + notifications depth.
   *
   * Adds CSS for the 4 pages Wave 6A Track C deepens:
   *   1. /chat              — inbox shell + chat panel + topic selector
   *   2. /chat/history      — filtered list of past conversations
   *   3. /chat/[id]         — single conversation view (uses shared
   *                           <MessageBubble> primitive)
   *   4. /notifications     — list + browser-notifications CTA +
   *                           per-type settings
   *
   * All rules below are scoped to the new section-marker class
   * names added by the Track C page ports in
   * `shared/rust/dioxus_ui/src/pages/chat.rs`,
   * `.../chat_history.rs`, `.../chat_conversation.rs`,
   * `.../notifications.rs` + the new <MessageBubble> primitive in
   * `shared/rust/dioxus_ui/src/chat/message_bubble.rs`.
   *
   * The block reuses the existing design-system classes
   * (`.card`, `.card-glass`, `.btn`, `.btn-primary`, `.btn-outline`,
   * `.text-muted-foreground`, etc.) and only defines new selectors
   * for the Wave 6A surface area. No new colors, no new design
   * tokens. CSS is appended cleanly so the integration agent can
   * concatenate Track A / B / C / D blocks (each marked) without
   * conflicts. */

  /* --- /chat inbox shell + main panel (2-column flex layout) — STUNNING v2 --- */
  .container.chat-public-page {{ width: 100%; max-width: 36rem; margin-left: auto; margin-right: auto; box-sizing: border-box; }}
  .chat-page {{ position: relative; min-height: 70vh; }}
  .chat-page::before {{ content: ''; position: fixed; inset: 0; z-index: -1; pointer-events: none;
                       background:
                         radial-gradient(700px 400px at 15% 10%, rgba(118,69,217,0.12) 0%, transparent 60%),
                         radial-gradient(600px 350px at 85% 85%, rgba(31,199,212,0.10) 0%, transparent 60%),
                         radial-gradient(500px 300px at 50% 0%, rgba(168,85,247,0.08) 0%, transparent 70%); }}
  .chat-inbox-row {{ display: flex; gap: 0; align-items: stretch; min-height: 560px;
                     border: 1px solid rgba(255,255,255,0.09);
                     border-radius: 1.25rem; overflow: hidden;
                     background: linear-gradient(180deg, rgba(255,255,255,0.06) 0%, rgba(255,255,255,0.03) 100%);
                     backdrop-filter: blur(20px) saturate(1.2); -webkit-backdrop-filter: blur(20px) saturate(1.2);
                     box-shadow: 0 20px 60px rgba(0,0,0,0.35), 0 1px 0 rgba(255,255,255,0.06) inset, 0 0 0 1px rgba(255,255,255,0.04) inset; }}
  .chat-inbox {{ width: 340px; flex-shrink: 0; display: flex; flex-direction: column;
                 border-right: 1px solid rgba(255,255,255,0.07);
                 background: linear-gradient(180deg, rgba(255,255,255,0.04) 0%, rgba(255,255,255,0.015) 100%); }}
  .chat-inbox-header {{ padding: 1.125rem 1rem 1rem;
                        border-bottom: 1px solid rgba(255,255,255,0.06);
                        background: linear-gradient(180deg, rgba(255,255,255,0.04) 0%, transparent 100%); }}
  .chat-inbox-brand {{ display: flex; align-items: center; gap: 0.875rem; }}
  .chat-inbox-avatar {{ position: relative; width: 2.75rem; height: 2.75rem;
                        border-radius: 0.875rem;
                        background: linear-gradient(135deg, #7c3aed 0%, #7645d9 45%, #06b6d4 100%);
                        display: flex; align-items: center; justify-content: center;
                        box-shadow: 0 8px 24px rgba(124,58,237,0.35), 0 2px 8px rgba(0,0,0,0.15), inset 0 1px 0 rgba(255,255,255,0.18);
                        border: 1px solid rgba(255,255,255,0.14); color: #fff; }}
  .chat-inbox-avatar::after {{ content: ''; position: absolute; inset: -1px; border-radius: 0.875rem;
                              background: linear-gradient(135deg, rgba(255,255,255,0.18), transparent 55%);
                              pointer-events: none; }}
  .chat-inbox-online-dot {{ position: absolute; bottom: -3px; right: -3px;
                            width: 0.875rem; height: 0.875rem; border-radius: 9999px;
                            background: #22c55e; border: 2.5px solid #0f172a;
                            box-shadow: 0 0 0 3px rgba(34,197,94,0.18), 0 2px 6px rgba(0,0,0,0.2);
                            animation: chat-pulse-dot 2.4s ease-in-out infinite; }}
  @keyframes chat-pulse-dot {{ 0%,100% {{ box-shadow: 0 0 0 3px rgba(34,197,94,0.18), 0 2px 6px rgba(0,0,0,0.2); }} 50% {{ box-shadow: 0 0 0 6px rgba(34,197,94,0.08), 0 2px 6px rgba(0,0,0,0.2); }} }}
  .chat-inbox-titles {{ flex: 1; min-width: 0; }}
  .chat-inbox-title {{ font-size: 0.875rem; font-weight: 800; margin: 0; letter-spacing: -0.015em; line-height: 1.1; }}
  .chat-inbox-subtitle {{ font-size: 0.6875rem; color: var(--text-muted, #94a3b8); margin: 0.1875rem 0 0; display:flex; align-items:center; gap:0.25rem; opacity:0.9; }}
  .chat-inbox-subtitle::before {{ content: ''; width: 0.375rem; height: 0.375rem; border-radius: 50%; background: #22c55e; box-shadow: 0 0 6px rgba(34,197,94,0.5); }}
  .chat-inbox-count {{ font-size: 0.6875rem; font-weight: 800;
                       background: linear-gradient(135deg, #7c3aed 0%, #1fc7d4 100%); color: #fff;
                       padding: 0.1875rem 0.5625rem; border-radius: 9999px;
                       border: 0; box-shadow: 0 2px 8px rgba(124,58,237,0.30);
                       min-width: 1.375rem; text-align:center; }}

  .chat-inbox-search {{ position: relative; padding: 0.625rem 0.875rem;
                        border-bottom: 1px solid rgba(255,255,255,0.06);
                        background: rgba(255,255,255,0.015); }}
  .chat-inbox-search > svg,
  .chat-inbox-search .epsx-icon {{ position: absolute; left: 1.5rem; top: 50%;
                              transform: translateY(-50%); color: var(--text-muted, #94a3b8); opacity: 0.55; }}
  .chat-inbox-search-input {{ width: 100%; padding: 0.625rem 0.875rem 0.625rem 2.375rem;
                              font-size: 0.8125rem; border-radius: 0.875rem;
                              background: rgba(255,255,255,0.06);
                              border: 1px solid rgba(255,255,255,0.09);
                              color: var(--text, #fff); outline: none; font-weight: 500;
                              transition: all 0.2s cubic-bezier(0.16,1,0.3,1);
                              box-shadow: inset 0 1px 0 rgba(255,255,255,0.06); }}
  .chat-inbox-search-input::placeholder {{ color: var(--text-muted, #94a3b8); opacity: 0.55; }}
  .chat-inbox-search-input:focus {{ border-color: rgba(124,58,237,0.45);
                                     box-shadow: 0 0 0 3px rgba(124,58,237,0.14), inset 0 1px 0 rgba(255,255,255,0.08);
                                     background: rgba(255,255,255,0.08); }}

  .chat-inbox-filters {{ display: flex; gap: 0.5rem; padding: 0.625rem 0.875rem;
                          border-bottom: 1px solid rgba(255,255,255,0.06);
                          background: rgba(255,255,255,0.015); }}
  .chat-inbox-filter {{ flex: 1; padding: 0.5rem 0.625rem; font-size: 0.6875rem; font-weight: 600;
                        background: rgba(255,255,255,0.06);
                        border: 1px solid rgba(255,255,255,0.08);
                        border-radius: 0.625rem; color: var(--text, #fff);
                        cursor: pointer; outline: none; letter-spacing: 0.01em;
                        transition: all 0.15s ease; appearance: none;
                        background-image: url("data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' width='12' height='12' viewBox='0 0 24 24' fill='none' stroke='%2394a3b8' stroke-width='2'%3E%3Cpath d='m6 9 6 6 6-6'/%3E%3C/svg%3E");
                        background-repeat: no-repeat; background-position: right 0.5rem center; padding-right: 1.5rem; }}
  .chat-inbox-filter:hover {{ background: rgba(255,255,255,0.08); border-color: rgba(255,255,255,0.12); }}
  .chat-inbox-filter:focus {{ border-color: rgba(124,58,237,0.40); box-shadow: 0 0 0 3px rgba(124,58,237,0.10); }}

  .chat-inbox-list {{ flex: 1; overflow-y: auto; min-height: 0; padding: 0.5rem;
                     scrollbar-width: thin; scrollbar-color: rgba(255,255,255,0.10) transparent; }}
  .chat-inbox-list::-webkit-scrollbar {{ width: 4px; }}
  .chat-inbox-list::-webkit-scrollbar-thumb {{ background: rgba(255,255,255,0.10); border-radius: 9999px; }}
  .chat-inbox-card {{ width: 100%; text-align: left; padding: 0.875rem 0.875rem;
                      background: rgba(255,255,255,0.02); border: 1px solid transparent; border-radius: 0.875rem;
                      margin-bottom: 0.375rem; cursor: pointer; display:block; text-decoration:none;
                      transition: all 0.2s cubic-bezier(0.16,1,0.3,1); color: inherit;
                      position: relative; overflow:hidden; }}
  .chat-inbox-card::before {{ content:''; position:absolute; left:0; top:0; bottom:0; width:3px;
                             background: linear-gradient(180deg, #7c3aed, #06b6d4); opacity:0; transition: opacity 0.2s ease; }}
  .chat-inbox-card:hover {{ background: rgba(255,255,255,0.05); border-color: rgba(255,255,255,0.07);
                           transform: translateY(-1px); box-shadow: 0 4px 12px rgba(0,0,0,0.10); }}
  .chat-inbox-card-selected {{ background: linear-gradient(135deg, rgba(124,58,237,0.14) 0%, rgba(6,182,214,0.08) 100%);
                               border-color: rgba(124,58,237,0.22); box-shadow: 0 4px 16px rgba(124,58,237,0.14), inset 0 1px 0 rgba(255,255,255,0.06); }}
  .chat-inbox-card-selected::before {{ opacity: 1; }}
  .chat-inbox-card-unread {{ background: rgba(124,58,237,0.06); border-color: rgba(124,58,237,0.10); }}
  .chat-inbox-card-row {{ display: flex; align-items: flex-start; justify-content: space-between; gap: 0.625rem; margin-bottom: 0.375rem; }}
  .chat-inbox-subject {{ font-size: 0.8125rem; line-height: 1.35; margin: 0; flex: 1; min-width: 0;
                         overflow: hidden; text-overflow: ellipsis; white-space: nowrap; font-weight: 650; letter-spacing: -0.01em; }}
  .chat-inbox-card-unread .chat-inbox-subject {{ font-weight: 750; }}
  .chat-inbox-card-selected .chat-inbox-subject {{ color: #fff; }}
  .chat-inbox-card-meta {{ display: flex; align-items: center; gap: 0.375rem; flex-shrink: 0; margin-top: 0.125rem; }}
  .chat-inbox-unread {{ min-width: 1.25rem; height: 1.25rem; padding: 0 0.3125rem; border-radius: 9999px;
                        background: linear-gradient(135deg, #7c3aed 0%, #06b6d4 100%);
                        color: #fff; font-size: 0.625rem; font-weight: 800;
                        display: inline-flex; align-items: center; justify-content: center;
                        box-shadow: 0 2px 8px rgba(124,58,237,0.35), 0 0 0 2px rgba(124,58,237,0.15); }}
  .chat-inbox-time {{ font-size: 0.6875rem; color: var(--text-muted, #94a3b8); font-weight: 500; font-variant-numeric: tabular-nums; }}
  .chat-inbox-card-foot {{ display: flex; align-items: center; gap: 0.5rem; }}
  .chat-inbox-topic {{ font-size: 0.6875rem; font-weight: 650; color: #22d3ee; opacity: 0.9; letter-spacing: 0.01em; }}
  .chat-inbox-topic.chip-selected {{ opacity: 1; }}
  .chat-inbox-empty {{ display: flex; flex-direction: column; align-items: center; justify-content: center;
                       height: 100%; padding: 2.5rem 1.5rem; text-align: center; gap: 0.125rem; }}
  .chat-inbox-empty-icon {{ width: 3.5rem; height: 3.5rem; border-radius: 1rem;
                            background: linear-gradient(135deg, rgba(124,58,237,0.12) 0%, rgba(6,182,214,0.10) 100%);
                            border: 1px solid rgba(124,58,237,0.14);
                            display: flex; align-items: center; justify-content: center;
                            margin-bottom: 0.875rem; color: rgba(124,58,237,0.55);
                            box-shadow: 0 8px 24px rgba(124,58,237,0.12), inset 0 1px 0 rgba(255,255,255,0.08); }}
  .chat-inbox-empty-title {{ font-size: 0.8125rem; font-weight: 700; color: var(--text, #fff); opacity: 0.7; margin: 0 0 0.1875rem; letter-spacing: -0.01em; }}
  .chat-inbox-empty-hint {{ font-size: 0.6875rem; color: var(--text-muted, #94a3b8); opacity: 0.6; margin: 0; }}

  .chat-inbox-newbar {{ padding: 0.875rem; border-top: 1px solid rgba(255,255,255,0.06); background: rgba(255,255,255,0.01); }}
  .chat-inbox-new {{ width: 100%; padding: 0.75rem; border-radius: 0.875rem;
                     background: linear-gradient(135deg, #7c3aed 0%, #5b21b6 55%, #1e40af 100%);
                     color: #fff; font-size: 0.8125rem; font-weight: 700; letter-spacing: 0.015em;
                     border: 1px solid rgba(255,255,255,0.14); cursor: pointer; display: flex; align-items: center; justify-content: center; gap: 0.5rem;
                     box-shadow: 0 8px 24px rgba(124,58,237,0.28), 0 2px 8px rgba(0,0,0,0.12), inset 0 1px 0 rgba(255,255,255,0.16);
                     transition: all 0.2s cubic-bezier(0.16,1,0.3,1); position: relative; overflow:hidden; }}
  .chat-inbox-new::before {{ content: ''; position:absolute; inset:0; background: linear-gradient(90deg, transparent, rgba(255,255,255,0.14), transparent);
                            transform: translateX(-100%); transition: transform 0.6s ease; }}
  .chat-inbox-new:hover {{ box-shadow: 0 10px 28px rgba(124,58,237,0.36), 0 4px 12px rgba(0,0,0,0.14); transform: translateY(-1px); }}
  .chat-inbox-new:hover::before {{ transform: translateX(100%); }}
  .chat-inbox-new:active {{ transform: scale(0.98) translateY(0); }}

  /* --- Chat status badge (mirrors chat-status-badge.tsx STATUS_CONFIG) --- */
  .chat-status {{ display: inline-flex; align-items: center; gap: 0.375rem;
                  font-size: 0.6875rem; font-weight: 700; padding: 0.1875rem 0.625rem; border-radius: 9999px;
                  background: var(--chip-bg, rgba(255,255,255,0.05));
                  color: var(--text, #fff); letter-spacing: 0.01em; border: 1px solid transparent; }}
  .chat-status-dot {{ width: 0.375rem; height: 0.375rem; border-radius: 9999px; background: currentColor; opacity: 0.85;
                     box-shadow: 0 0 6px currentColor; }}
  .chat-status-open     {{ background: rgba(251,191,36,0.12);  color: #fbbf24; border-color: rgba(251,191,36,0.18); }}
  .chat-status-progress {{ background: rgba(96,165,250,0.12);  color: #60a5fa; border-color: rgba(96,165,250,0.18); }}
  .chat-status-resolved {{ background: rgba(52,211,153,0.12);  color: #34d399; border-color: rgba(52,211,153,0.18); }}
  .chat-status-closed   {{ background: rgba(148,163,184,0.10); color: #94a3b8; border-color: rgba(148,163,184,0.14); }}

  /* --- /chat main panel (right column) --- */
  .chat-panel {{ flex: 1; min-width: 0; display: flex; flex-direction: column;
                 background: linear-gradient(180deg, rgba(255,255,255,0.015) 0%, rgba(255,255,255,0.005) 100%); }}
  .chat-panel-new {{ padding: 1.25rem; overflow-y: auto; scrollbar-width: thin; scrollbar-color: rgba(255,255,255,0.08) transparent; }}
  .chat-panel-new::-webkit-scrollbar {{ width: 4px; }}
  .chat-panel-new::-webkit-scrollbar-thumb {{ background: rgba(255,255,255,0.10); border-radius: 9999px; }}
  .chat-panel-back {{ display: inline-flex; align-items: center; gap: 0.375rem; font-size: 0.75rem; font-weight: 600;
                      color: var(--text-muted, #94a3b8); margin-bottom: 1rem; padding: 0.375rem 0.625rem; border-radius: 0.5rem;
                      background: rgba(255,255,255,0.04); border: 1px solid rgba(255,255,255,0.06); transition: all 0.15s ease; }}
  .chat-panel-back:hover {{ background: rgba(255,255,255,0.07); color: var(--text, #fff); }}
  .chat-panel-empty {{ align-items: center; justify-content: center; text-align: center; padding: 2.5rem; flex: 1;
                       display: flex; flex-direction: column; }}
  .chat-panel-empty-icon {{ width: 4.5rem; height: 4.5rem; border-radius: 1.25rem;
                            background: linear-gradient(135deg, rgba(124,58,237,0.14) 0%, rgba(6,182,214,0.10) 100%);
                            border: 1px solid rgba(124,58,237,0.16);
                            display: flex; align-items: center; justify-content: center;
                            margin-bottom: 1.125rem; color: rgba(124,58,237,0.45);
                            box-shadow: 0 12px 32px rgba(124,58,237,0.14), inset 0 1px 0 rgba(255,255,255,0.08); }}
  .chat-panel-empty-title {{ font-size: 1rem; font-weight: 750; color: var(--text, #fff); opacity: 0.85; margin: 0 0 0.375rem; letter-spacing: -0.015em; }}
  .chat-panel-empty-hint {{ font-size: 0.8125rem; color: var(--text-muted, #94a3b8); opacity: 0.65; max-width: 18rem;
                            line-height: 1.5; margin: 0; }}

  /* --- Chat header (above the message list) --- */
  .chat-header {{ position: relative; flex-shrink: 0;
                  background: linear-gradient(180deg, rgba(255,255,255,0.06) 0%, rgba(255,255,255,0.02) 100%);
                  border-bottom: 1px solid rgba(255,255,255,0.07);
                  box-shadow: 0 1px 0 rgba(255,255,255,0.04) inset, 0 4px 16px rgba(0,0,0,0.08);
                  backdrop-filter: blur(12px); -webkit-backdrop-filter: blur(12px); }}
  .chat-header::after {{ content:''; position:absolute; inset:0; background: linear-gradient(90deg, rgba(124,58,237,0.04), transparent 40%, rgba(6,182,214,0.03));
                        pointer-events:none; }}
  .chat-header-accent {{ height: 2px;
                         background: linear-gradient(90deg, #7c3aed 0%, #06b6d4 50%, #7c3aed 100%);
                         background-size: 200% 100%;
                         animation: gradient-x 3.5s ease infinite; box-shadow: 0 1px 8px rgba(124,58,237,0.35); }}
  @keyframes gradient-x {{
    0%, 100% {{ background-position: 0% 50%; }}
    50%      {{ background-position: 100% 50%; }}
  }}
  .chat-header-row {{ display: flex; align-items: center; gap: 0.875rem; padding: 1rem 1.25rem; position: relative; z-index: 1; }}
  .chat-header-avatar {{ width: 2.5rem; height: 2.5rem; border-radius: 0.875rem;
                         background: linear-gradient(135deg, rgba(124,58,237,0.16) 0%, rgba(6,182,214,0.14) 100%);
                         border: 1px solid rgba(124,58,237,0.18);
                         display: flex; align-items: center; justify-content: center;
                         color: #8b5cf6; flex-shrink: 0; box-shadow: 0 4px 12px rgba(124,58,237,0.10), inset 0 1px 0 rgba(255,255,255,0.08); }}
  .chat-header-titles {{ flex: 1; min-width: 0; display: flex; flex-direction: column; gap: 0.1875rem; }}
  .chat-header-subject {{ font-size: 0.9375rem; font-weight: 750; margin: 0; line-height: 1.25;
                          overflow: hidden; text-overflow: ellipsis; white-space: nowrap; letter-spacing: -0.015em; }}
  .chat-header-resolve {{ display: inline-flex; align-items: center; gap: 0.375rem;
                          padding: 0.5rem 0.875rem; border-radius: 0.75rem;
                          background: rgba(34,197,94,0.10); color: #22c55e;
                          border: 1px solid rgba(34,197,94,0.20);
                          font-size: 0.6875rem; font-weight: 750; text-transform: uppercase; letter-spacing: 0.04em;
                          cursor: pointer; flex-shrink: 0; transition: all 0.15s ease; }}
  .chat-header-resolve:hover {{ background: rgba(34,197,94,0.16); border-color: rgba(34,197,94,0.28); transform: translateY(-1px); box-shadow: 0 4px 12px rgba(34,197,94,0.15); }}

  /* --- <MessageBubble> primitive (shared with admin chat) --- */
  .chat-messages {{ flex: 1; overflow-y: auto; padding: 1.25rem 1.25rem 1rem;
                    background-image:
                      radial-gradient(circle, rgba(124,58,237,0.055) 1px, transparent 1px),
                      linear-gradient(180deg, rgba(255,255,255,0.01) 0%, transparent 100%);
                    background-size: 22px 22px, 100% 100%;
                    scrollbar-width: thin; scrollbar-color: rgba(255,255,255,0.08) transparent; }}
  .chat-messages::-webkit-scrollbar {{ width: 6px; }}
  .chat-messages::-webkit-scrollbar-thumb {{ background: rgba(255,255,255,0.08); border-radius: 9999px; }}
  .chat-messages::-webkit-scrollbar-thumb:hover {{ background: rgba(255,255,255,0.14); }}
  .chat-date-sep {{ display: flex; align-items: center; gap: 0.75rem; margin: 1.5rem 0; }}
  .chat-date-sep::before, .chat-date-sep::after {{
    content: ''; flex: 1; height: 1px;
    background: linear-gradient(90deg, transparent 0%, var(--glass-border, rgba(255,255,255,0.08)) 50%, transparent 100%);
  }}
  .chat-date-sep-pill {{ font-size: 0.625rem; font-weight: 600; color: var(--text-muted, #94a3b8);
                         text-transform: uppercase; letter-spacing: 0.12em;
                         padding: 0.25rem 0.625rem; border-radius: 9999px;
                         background: var(--chip-bg, rgba(255,255,255,0.05));
                         border: 1px solid var(--glass-border, rgba(255,255,255,0.08)); }}

  .chat-message {{ display: flex; gap: 0.75rem; margin-bottom: 1rem; align-items: flex-end; animation: chat-msg-in 0.28s cubic-bezier(0.16,1,0.3,1) both; }}
  @keyframes chat-msg-in {{ from {{ opacity:0; transform: translateY(6px) scale(0.98); }} to {{ opacity:1; transform: translateY(0) scale(1); }} }}
  .chat-message-other {{ flex-direction: row; }}
  .chat-message-self {{ flex-direction: row-reverse; }}
  .chat-message-avatar {{ width: 2.125rem; height: 2.125rem; border-radius: 9999px; flex-shrink: 0;
                          background: linear-gradient(135deg, rgba(124,58,237,0.18) 0%, rgba(6,182,214,0.16) 100%);
                          border: 1px solid rgba(124,58,237,0.22);
                          display: flex; align-items: center; justify-content: center;
                          margin-bottom: 0.125rem; color: #8b5cf6;
                          box-shadow: 0 2px 8px rgba(124,58,237,0.12); }}
  .chat-message-col {{ max-width: 76%; display: flex; flex-direction: column; min-width: 0; gap: 0.1875rem; }}
  .chat-message-other .chat-message-col {{ align-items: flex-start; }}
  .chat-message-self .chat-message-col   {{ align-items: flex-end; }}
  .chat-message-sender {{ font-size: 0.6875rem; font-weight: 700; color: var(--text-muted, #94a3b8);
                          margin-bottom: 0.125rem; padding: 0 0.375rem; letter-spacing: 0.02em; opacity: 0.85; }}

  .chat-bubble {{ padding: 0.75rem 1rem; border-radius: 1.125rem; font-size: 0.875rem;
                  line-height: 1.55; max-width: 100%; word-wrap: break-word; overflow-wrap: break-word;
                  position: relative; transition: transform 0.15s ease; }}
  .chat-bubble-other {{ background: rgba(255,255,255,0.07);
                        border: 1px solid rgba(255,255,255,0.09);
                        border-bottom-left-radius: 0.375rem;
                        box-shadow: 0 2px 12px rgba(0,0,0,0.08), 0 1px 3px rgba(0,0,0,0.06), inset 0 1px 0 rgba(255,255,255,0.06);
                        backdrop-filter: blur(8px); -webkit-backdrop-filter: blur(8px); }}
  .chat-bubble-self {{ background: linear-gradient(135deg, #7c3aed 0%, #5b21b6 60%, #4c1d95 100%);
                        color: #fff; border-bottom-right-radius: 0.375rem;
                        box-shadow: 0 8px 20px rgba(124,58,237,0.28), 0 2px 8px rgba(0,0,0,0.10), inset 0 1px 0 rgba(255,255,255,0.14);
                        border: 1px solid rgba(255,255,255,0.10); }}
  .chat-bubble-body {{ margin: 0; }}

  .chat-message-meta {{ display: flex; align-items: center; gap: 0.25rem;
                        margin-top: 0.25rem; padding: 0 0.25rem; font-size: 0.625rem;
                        color: var(--text-muted, #94a3b8); opacity: 0.6; }}
  .chat-message-other .chat-message-meta {{ flex-direction: row; }}
  .chat-message-self .chat-message-meta {{ flex-direction: row-reverse; }}
  .chat-message-timestamp {{ font-size: 0.625rem; }}

  .chat-message-system {{ display: flex; justify-content: center; margin: 0.75rem 0; }}
  .chat-message-system-pill {{ display: inline-flex; align-items: center; gap: 0.375rem;
                               padding: 0.375rem 0.75rem; border-radius: 9999px;
                               background: var(--chip-bg, rgba(255,255,255,0.05));
                               border: 1px solid var(--glass-border, rgba(255,255,255,0.08));
                               color: var(--text-muted, #94a3b8); font-size: 0.6875rem; }}
  .chat-message-system-text {{ color: var(--text-muted, #94a3b8); }}

  .chat-attachment {{ display: inline-flex; align-items: center; gap: 0.5rem;
                      padding: 0.5rem 0.75rem; margin-top: 0.5rem; border-radius: 0.75rem;
                      background: rgba(0,0,0,0.10); border: 1px solid rgba(255,255,255,0.12);
                      text-decoration: none; color: inherit; font-size: 0.75rem; }}
  .chat-attachment:hover {{ background: rgba(0,0,0,0.18); }}
  .chat-attachment-image {{ padding: 0; border: 0; background: transparent; }}
  .chat-attachment-thumb {{ max-width: 100%; max-height: 12rem; border-radius: 0.75rem;
                            border: 1px solid rgba(255,255,255,0.15); display: block; }}
  .chat-attachment-info {{ min-width: 0; flex: 1; }}
  .chat-attachment-name {{ font-size: 0.75rem; font-weight: 500; margin: 0;
                            overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }}
  .chat-attachment-size {{ font-size: 0.625rem; opacity: 0.55; margin: 0; }}

  /* --- /chat input composer --- */
  .chat-input {{ flex-shrink: 0; padding: 0.875rem 1rem 1rem;
                 background: linear-gradient(180deg, rgba(255,255,255,0.03) 0%, rgba(255,255,255,0.015) 100%);
                 border-top: 1px solid rgba(255,255,255,0.07);
                 backdrop-filter: blur(10px); -webkit-backdrop-filter: blur(10px); }}
  .chat-input-row {{ display: flex; align-items: flex-end; gap: 0.75rem;
                     padding: 0.625rem 0.875rem; border-radius: 1rem;
                     background: rgba(255,255,255,0.06);
                     border: 1px solid rgba(255,255,255,0.09);
                     box-shadow: inset 0 1px 0 rgba(255,255,255,0.06), 0 2px 8px rgba(0,0,0,0.06);
                     transition: all 0.2s cubic-bezier(0.16,1,0.3,1); }}
  .chat-input-row:focus-within {{ border-color: rgba(124,58,237,0.40);
                                   box-shadow: 0 0 0 3px rgba(124,58,237,0.12), inset 0 1px 0 rgba(255,255,255,0.08);
                                   background: rgba(255,255,255,0.08); }}
  .chat-input-attach {{ width: 2rem; height: 2rem; border-radius: 0.625rem;
                        background: rgba(255,255,255,0.04); border: 1px solid rgba(255,255,255,0.06); cursor: pointer;
                        display: flex; align-items: center; justify-content: center;
                        color: var(--text-muted, #94a3b8); opacity: 0.75;
                        transition: all 0.15s ease; flex-shrink: 0; }}
  .chat-input-attach:hover {{ opacity: 1; background: rgba(255,255,255,0.08); border-color: rgba(255,255,255,0.10); color: var(--text, #fff); }}
  .chat-input-textarea {{ flex: 1; resize: none; background: transparent;
                          border: 0; outline: none; font-size: 0.875rem; line-height: 1.5;
                          color: var(--text, #fff); padding: 0.25rem 0; min-height: 1.75rem; max-height: 7rem; }}
  .chat-input-textarea::placeholder {{ color: var(--text-muted, #94a3b8); opacity: 0.5; }}
  .chat-input-send {{ width: 2.25rem; height: 2.25rem; border-radius: 0.75rem; flex-shrink: 0;
                      border: 1px solid rgba(255,255,255,0.12); cursor: pointer;
                      display: flex; align-items: center; justify-content: center;
                      background: linear-gradient(135deg, #7c3aed 0%, #5b21b6 100%);
                      color: #fff; box-shadow: 0 6px 16px rgba(124,58,237,0.30), inset 0 1px 0 rgba(255,255,255,0.14);
                      transition: all 0.15s cubic-bezier(0.16,1,0.3,1); }}
  .chat-input-send:hover {{ box-shadow: 0 8px 20px rgba(124,58,237,0.38); transform: translateY(-1px); }}
  .chat-input-send:active {{ transform: scale(0.96); }}
  .chat-input-send:disabled {{ background: rgba(255,255,255,0.04); color: var(--text-muted, #94a3b8);
                               opacity: 0.45; cursor: not-allowed; box-shadow: none; border-color: rgba(255,255,255,0.06); }}
  .chat-input-hint {{ font-size: 0.6875rem; color: var(--text-muted, #94a3b8); opacity: 0.45;
                      text-align: center; margin: 0.5rem 0 0; letter-spacing: 0.01em; }}

  /* --- /chat topic selector (new conversation flow) --- */
  .chat-topic-selector {{ padding: 1.25rem; }}
  .chat-topic-title {{ font-size: 1.125rem; font-weight: 800; margin: 0 0 0.1875rem; letter-spacing: -0.02em; line-height: 1.2; }}
  .chat-topic-subtitle {{ font-size: 0.8125rem; color: var(--text-muted, #94a3b8); margin: 0 0 1rem; opacity: 0.85; line-height: 1.4; }}
  .chat-topic-grid {{ display: grid; grid-template-columns: 1fr 1fr; gap: 0.625rem; }}
  @media (max-width: 640px) {{ .chat-topic-grid {{ grid-template-columns: 1fr; }} }}
  .chat-topic-card {{ display: flex; align-items: center; gap: 0.75rem;
                      padding: 0.875rem; border-radius: 1rem;
                      background: linear-gradient(135deg, rgba(255,255,255,0.05) 0%, rgba(255,255,255,0.02) 100%);
                      border: 1px solid rgba(255,255,255,0.07);
                      cursor: pointer; text-align: left; color: inherit;
                      box-shadow: 0 2px 8px rgba(0,0,0,0.06), inset 0 1px 0 rgba(255,255,255,0.05);
                      transition: all 0.22s cubic-bezier(0.16,1,0.3,1); position: relative; overflow:hidden; }}
  .chat-topic-card::before {{ content:''; position:absolute; inset:0; border-radius: 1rem;
                             background: linear-gradient(135deg, rgba(255,255,255,0.06), transparent 60%);
                             opacity:0; transition: opacity 0.2s ease; pointer-events:none; }}
  .chat-topic-card:hover {{ background: linear-gradient(135deg, rgba(255,255,255,0.08) 0%, rgba(255,255,255,0.04) 100%);
                           border-color: rgba(124,58,237,0.28); transform: translateY(-2px);
                           box-shadow: 0 8px 24px rgba(0,0,0,0.12), 0 2px 8px rgba(124,58,237,0.10), inset 0 1px 0 rgba(255,255,255,0.08); }}
  .chat-topic-card:hover::before {{ opacity: 1; }}
  .chat-topic-card:active {{ transform: translateY(0) scale(0.99); }}
  .chat-topic-card-icon {{ width: 2.625rem; height: 2.625rem; border-radius: 0.875rem; flex-shrink: 0;
                           display: flex; align-items: center; justify-content: center;
                           border: 1px solid rgba(255,255,255,0.16);
                           box-shadow: 0 6px 16px rgba(0,0,0,0.22), inset 0 1px 0 rgba(255,255,255,0.14);
                           filter: none;
                           transition: all 0.22s cubic-bezier(0.16,1,0.3,1); }}
  .chat-topic-card-icon .epsx-icon {{ color: inherit !important; opacity: 1 !important; }}
  .chat-topic-card-icon .lucide {{ width: 20px !important; height: 20px !important;
                                   stroke-width: 2.1 !important;
                                   filter: drop-shadow(0 1px 2px rgba(0,0,0,0.22));
                                   opacity: 1 !important; }}
  .chat-topic-card:hover .chat-topic-card-icon {{ transform: scale(1.07) rotate(1deg); box-shadow: 0 8px 20px rgba(0,0,0,0.24); border-color: rgba(255,255,255,0.22); }}
  .chat-topic-card-titles {{ flex: 1; min-width: 0; }}
  .chat-topic-card-label {{ font-size: 0.875rem; font-weight: 700; margin: 0; letter-spacing: -0.01em; }}
  .chat-topic-card-description {{ font-size: 0.75rem; color: var(--text-muted, #94a3b8);
                                  margin: 0.1875rem 0 0; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; opacity: 0.85; line-height: 1.3; }}
  .chat-topic-card > svg,
  .chat-topic-card .epsx-icon:last-child {{ color: var(--text-muted, #94a3b8); opacity: 0.45; flex-shrink: 0; transition: transform 0.2s ease, opacity 0.2s ease; }}
  .chat-topic-card:hover .epsx-icon:last-child {{ opacity: 0.9; transform: translateX(2px); }}

  .chat-topic-composer {{ display: flex; flex-direction: column; gap: 0.875rem; }}
  .chat-topic-back {{ display: inline-flex; align-items: center; gap: 0.375rem;
                      font-size: 0.8125rem; font-weight: 600; color: var(--text-muted, #94a3b8);
                      background: rgba(255,255,255,0.04); border: 1px solid rgba(255,255,255,0.06); cursor: pointer;
                      padding: 0.5rem 0.75rem; border-radius: 0.625rem; align-self: flex-start;
                      transition: all 0.15s ease; }}
  .chat-topic-back:hover {{ background: rgba(255,255,255,0.07); color: var(--text, #fff); transform: translateX(-1px); }}
  .chat-topic-header {{ display: flex; align-items: center; gap: 0.875rem;
                        padding: 0.875rem; border-radius: 1rem;
                        background: linear-gradient(135deg, rgba(255,255,255,0.06) 0%, rgba(255,255,255,0.02) 100%);
                        border: 1px solid rgba(255,255,255,0.08);
                        box-shadow: 0 4px 16px rgba(0,0,0,0.08), inset 0 1px 0 rgba(255,255,255,0.06); }}
  .chat-topic-icon {{ width: 2.75rem; height: 2.75rem; border-radius: 0.875rem; flex-shrink: 0;
                      display: flex; align-items: center; justify-content: center;
                      box-shadow: 0 4px 12px rgba(0,0,0,0.12), inset 0 1px 0 rgba(255,255,255,0.10);
                      border: 1px solid rgba(255,255,255,0.10); }}
  .chat-topic-label {{ font-size: 0.9375rem; font-weight: 750; margin: 0; letter-spacing: -0.01em; }}
  .chat-topic-description {{ font-size: 0.8125rem; color: var(--text-muted, #94a3b8); margin: 0.1875rem 0 0; opacity: 0.85; }}
  .chat-topic-form {{ display: flex; flex-direction: column; gap: 0.75rem; flex: 1; min-height: 0; }}
  .chat-topic-form-label {{ font-size: 0.6875rem; font-weight: 700; color: var(--text-muted, #94a3b8);
                             text-transform: uppercase; letter-spacing: 0.07em; display: block; margin-bottom: 0.375rem; opacity:0.9; }}
  .chat-topic-form-input, .chat-topic-form-textarea {{
    width: 100%; padding: 0.75rem 0.875rem; border-radius: 0.875rem;
    background: rgba(255,255,255,0.06);
    border: 1px solid rgba(255,255,255,0.09);
    color: var(--text, #fff); font-size: 0.875rem; outline: none; font-weight: 500;
    box-shadow: inset 0 1px 0 rgba(255,255,255,0.06);
    transition: all 0.2s cubic-bezier(0.16,1,0.3,1); }}
  .chat-topic-form-input::placeholder, .chat-topic-form-textarea::placeholder {{ color: var(--text-muted, #94a3b8); opacity: 0.45; }}
  .chat-topic-form-input:focus, .chat-topic-form-textarea:focus {{
    border-color: rgba(124,58,237,0.38); box-shadow: 0 0 0 3px rgba(124,58,237,0.14), inset 0 1px 0 rgba(255,255,255,0.08);
    background: rgba(255,255,255,0.08); }}
  .chat-topic-form-textarea {{ flex: 1; resize: none; min-height: 8rem; line-height: 1.5; }}
  .chat-topic-dropzone {{ display: flex; flex-direction: column; align-items: center; gap: 0.5rem;
                          padding: 1rem; border: 2px dashed rgba(255,255,255,0.12);
                          border-radius: 1rem; cursor: pointer;
                          transition: all 0.2s ease; background: rgba(255,255,255,0.02);
                          color: var(--text-muted, #94a3b8); }}
  .chat-topic-dropzone:hover {{ border-color: rgba(124,58,237,0.35); background: rgba(124,58,237,0.04); color: var(--text, #fff); }}
  .chat-topic-dropzone > p {{ margin: 0; font-size: 0.8125rem; font-weight: 600; opacity: 0.75; }}
  .chat-topic-dropzone-hint {{ font-size: 0.6875rem !important; opacity: 0.45 !important; font-weight: 500 !important; }}
  .chat-topic-start {{ width: 100%; padding: 0.875rem; border-radius: 0.875rem;
                       background: linear-gradient(135deg, #7c3aed 0%, #5b21b6 55%, #1e40af 100%);
                       color: #fff; font-size: 0.9375rem; font-weight: 700; letter-spacing: 0.01em;
                       border: 1px solid rgba(255,255,255,0.12); cursor: pointer;
                       display: flex; align-items: center; justify-content: center; gap: 0.5rem;
                       box-shadow: 0 8px 24px rgba(124,58,237,0.26), inset 0 1px 0 rgba(255,255,255,0.14);
                       transition: all 0.2s cubic-bezier(0.16,1,0.3,1); position: relative; overflow:hidden; }}
  .chat-topic-start::before {{ content:''; position:absolute; inset:0; background: linear-gradient(90deg, transparent, rgba(255,255,255,0.12), transparent);
                              transform: translateX(-100%); transition: transform 0.6s ease; }}
  .chat-topic-start:hover {{ transform: translateY(-1px); box-shadow: 0 10px 28px rgba(124,58,237,0.32); }}
  .chat-topic-start:hover::before {{ transform: translateX(100%); }}
  .chat-topic-start:active {{ transform: scale(0.98); }}
  .chat-topic-start:disabled {{ opacity: 0.45; cursor: not-allowed; background: rgba(255,255,255,0.06); border-color: rgba(255,255,255,0.06); box-shadow: none; }}
  .chat-topic-dropzone.drag-over {{ border-color: rgba(96,165,250,0.60); background: rgba(96,165,250,0.08); }}
  .chat-topic-file-list {{ width: 100%; }}
  .chat-topic-file-item {{ display: flex; align-items: center; justify-content: space-between; gap: 0.5rem; padding: 0.5rem 0.625rem; margin-top: 0.5rem; background: rgba(255,255,255,0.06); border: 1px solid rgba(255,255,255,0.08); border-radius: 0.5rem; font-size: 0.75rem; }}
  .chat-topic-file-error {{ width: 100%; margin: 0.375rem 0 0; font-size: 0.6875rem; color: #ef4444; text-align: center; }}
  .chat-topic-form-status {{ width: 100%; margin: 0.375rem 0 0; font-size: 0.75rem; color: var(--text-muted, #94a3b8); text-align: center; }}
  .chat-topic-form-wrap {{ display: flex; flex-direction: column; flex: 1; min-height: 0; }}
  .chat-topic-form-wrap[hidden] {{ display: none !important; }}
  .chat-topic-selector[hidden] {{ display: none !important; }}

  /* --- /chat_history (filtered list) --- */
  .chat-history {{ max-width: 768px; margin: 0 auto; }}
  .chat-history-header {{ display: flex; align-items: center; gap: 0.75rem; margin-bottom: 1.5rem; }}
  .chat-history-back {{ width: 2.25rem; height: 2.25rem; border-radius: 0.75rem;
                        background: var(--card-bg, rgba(255,255,255,0.04));
                        border: 1px solid var(--card-border, rgba(255,255,255,0.08));
                        display: flex; align-items: center; justify-content: center;
                        color: inherit; text-decoration: none; transition: background 0.15s ease; }}
  .chat-history-back:hover {{ background: rgba(255,255,255,0.08); }}
  .chat-history-titles {{ flex: 1; min-width: 0; }}
  .chat-history-title {{ font-size: 1.25rem; font-weight: 700; margin: 0; letter-spacing: -0.01em; }}
  .chat-history-subtitle {{ font-size: 0.75rem; color: var(--text-muted, #94a3b8); margin: 0.125rem 0 0; opacity: 0.6; }}

  .chat-history-filters {{ display: flex; align-items: center; gap: 0.5rem;
                            padding: 0.625rem; border-radius: 1rem;
                            background: var(--card-bg, rgba(255,255,255,0.04));
                            border: 1px solid var(--card-border, rgba(255,255,255,0.08));
                            margin-bottom: 1rem; }}
  .chat-history-filters > svg {{ color: var(--text-muted, #94a3b8); opacity: 0.5; flex-shrink: 0; margin-left: 0.25rem; }}
  .chat-history-filter {{ flex: 1; padding: 0.5rem 0.75rem; border-radius: 0.75rem;
                          background: var(--input-bg, rgba(255,255,255,0.04));
                          border: 1px solid var(--glass-border, rgba(255,255,255,0.08));
                          color: var(--text, #fff); font-size: 0.75rem; font-weight: 500; outline: none;
                          cursor: pointer; transition: border-color 0.15s ease, box-shadow 0.15s ease; }}
  .chat-history-filter:focus {{ border-color: rgba(96,165,250,0.30); box-shadow: 0 0 0 3px rgba(96,165,250,0.20); }}

  .chat-history-list {{ background: var(--card-bg, rgba(255,255,255,0.04));
                        border: 1px solid var(--card-border, rgba(255,255,255,0.08));
                        border-radius: 1.5rem; overflow: hidden;
                        box-shadow: 0 1px 3px rgba(0,0,0,0.04); }}
  .chat-history-card {{ display: flex; align-items: center; gap: 0.75rem;
                        padding: 1rem 1.25rem; text-decoration: none; color: inherit;
                        border-bottom: 1px solid var(--card-border, rgba(255,255,255,0.08));
                        transition: background 0.15s ease; }}
  .chat-history-card:hover {{ background: rgba(255,255,255,0.04); }}
  .chat-history-card-last {{ border-bottom: 0; }}
  .chat-history-card-unread {{ background: rgba(96,165,250,0.05);
                               border-left: 2px solid #3b82f6; }}
  .chat-history-card-main {{ flex: 1; min-width: 0; }}
  .chat-history-card-subject {{ font-size: 0.875rem; line-height: 1.25; margin: 0 0 0.5rem;
                                overflow: hidden; text-overflow: ellipsis; white-space: nowrap;
                                font-weight: 600; }}
  .chat-history-card-unread .chat-history-card-subject {{ font-weight: 700; }}
  .chat-history-card-meta {{ display: flex; align-items: center; gap: 0.5rem; flex-wrap: wrap; }}
  .chat-history-card-topic {{ font-size: 0.625rem; font-weight: 600;
                              color: var(--text-muted, #94a3b8);
                              background: var(--chip-bg, rgba(255,255,255,0.05));
                              padding: 0.125rem 0.5rem; border-radius: 0.375rem;
                              border: 1px solid var(--card-border, rgba(255,255,255,0.08)); }}
  .chat-history-card-time {{ font-size: 0.625rem; color: var(--text-muted, #94a3b8); opacity: 0.5; }}
  .chat-history-card-aside {{ display: flex; align-items: center; gap: 0.5rem; flex-shrink: 0; margin-top: 0.125rem; }}
  .chat-history-card-unread-badge {{ min-width: 1.375rem; height: 1.375rem; padding: 0 0.375rem; border-radius: 9999px;
                                     background: #3b82f6; color: #fff; font-size: 0.625rem; font-weight: 700;
                                     display: inline-flex; align-items: center; justify-content: center;
                                     box-shadow: 0 1px 3px rgba(59,130,246,0.20); }}
  .chat-history-card-aside > svg {{ color: var(--text-muted, #94a3b8); opacity: 0.3; }}

  .chat-history-empty {{ text-align: center; padding: 5rem 1.25rem; }}
  .chat-history-empty-icon {{ width: 3.5rem; height: 3.5rem; border-radius: 1rem;
                              background: var(--card-bg, rgba(255,255,255,0.04));
                              border: 1px solid var(--card-border, rgba(255,255,255,0.08));
                              display: flex; align-items: center; justify-content: center;
                              margin: 0 auto 1rem; color: var(--text-muted, #94a3b8); opacity: 0.5; }}
  .chat-history-empty-title {{ font-size: 0.875rem; font-weight: 500; color: var(--text, #fff); opacity: 0.5; margin: 0 0 0.25rem; }}
  .chat-history-empty-hint {{ font-size: 0.75rem; color: var(--text-muted, #94a3b8); opacity: 0.5; margin: 0; }}
  .chat-history-grouped {{ background: transparent; border: 0; box-shadow: none; display: flex; flex-direction: column; gap: 1rem; }}
  .chat-history-group {{ background: var(--card-bg, rgba(255,255,255,0.04)); border: 1px solid var(--card-border, rgba(255,255,255,0.08)); border-radius: 1.25rem; overflow: hidden; box-shadow: 0 4px 16px rgba(0,0,0,0.06); backdrop-filter: blur(12px); -webkit-backdrop-filter: blur(12px); }}
  .chat-history-group-header {{ display: flex; align-items: center; justify-content: space-between; padding: 0.75rem 1.25rem; background: linear-gradient(180deg, rgba(255,255,255,0.04) 0%, rgba(255,255,255,0.01) 100%); border-bottom: 1px solid rgba(255,255,255,0.06); position: sticky; top: 0; backdrop-filter: blur(8px); z-index: 1; }}
  .chat-history-group-title {{ font-size: 0.75rem; font-weight: 750; letter-spacing: 0.06em; text-transform: uppercase; color: var(--text, #fff); opacity: 0.9; }}
  .chat-history-group-count {{ font-size: 0.6875rem; font-weight: 700; padding: 0.125rem 0.5rem; border-radius: 9999px; background: rgba(124,58,237,0.12); color: #a78bfa; border: 1px solid rgba(124,58,237,0.18); }}

  /* --- /chat/[id] (single conversation view, reuses panel CSS) --- */
  .chat-conversation {{ max-width: 760px; margin: 0 auto; position: relative; }}
  .chat-conversation::before {{ content: ''; position: fixed; inset: 0; z-index: -1; pointer-events: none;
                               background: radial-gradient(700px 400px at 20% 15%, rgba(124,58,237,0.10) 0%, transparent 60%),
                                         radial-gradient(600px 350px at 85% 85%, rgba(6,182,214,0.08) 0%, transparent 60%); }}
  .chat-conversation-full {{ max-width: none; margin: 0; height: 100%; min-height: 0; padding: 0; }}
  .chat-conversation-full::before {{ display: none; }}
  .chat-conv {{ display: flex; flex-direction: column; min-height: 600px; height: 720px;
                background: linear-gradient(180deg, rgba(255,255,255,0.06) 0%, rgba(255,255,255,0.02) 100%);
                border: 1px solid rgba(255,255,255,0.09);
                border-radius: 1.25rem; overflow: hidden;
                box-shadow: 0 20px 60px rgba(0,0,0,0.30), 0 1px 0 rgba(255,255,255,0.06) inset;
                backdrop-filter: blur(20px) saturate(1.15); -webkit-backdrop-filter: blur(20px) saturate(1.15); }}
  .chat-conv-full {{ height: 100%; min-height: 0; border: 0; border-radius: 0; box-shadow: none; background: linear-gradient(180deg, rgba(255,255,255,0.04) 0%, rgba(255,255,255,0.015) 100%); backdrop-filter: blur(16px) saturate(1.1); -webkit-backdrop-filter: blur(16px) saturate(1.1); }}
  .chat-page.chat-conversation-full {{ background: var(--bg); }}
  .chat-conv-full .chat-header {{ background: linear-gradient(180deg, rgba(255,255,255,0.06) 0%, rgba(255,255,255,0.02) 100%); }}
  .chat-conv-full .chat-messages {{ flex: 1; min-height: 0; }}
  .chat-conv-full .chat-input {{ position: sticky; bottom: 0; background: rgba(15,23,42,0.85); backdrop-filter: blur(12px); -webkit-backdrop-filter: blur(12px); }}
  .chat-conv-header {{ display: flex; align-items: center; gap: 0.625rem;
                        padding: 0.875rem 1.125rem; flex-shrink: 0;
                        background: linear-gradient(180deg, rgba(255,255,255,0.05) 0%, rgba(255,255,255,0.015) 100%);
                        border-bottom: 1px solid rgba(255,255,255,0.07);
                        backdrop-filter: blur(12px); -webkit-backdrop-filter: blur(12px); }}
  .chat-conv-back {{ width: 2rem; height: 2rem; border-radius: 0.5rem;
                     background: transparent; border: 0; cursor: pointer; padding: 0;
                     display: flex; align-items: center; justify-content: center;
                     color: var(--text-muted, #94a3b8);
                     transition: background 0.15s ease; flex-shrink: 0;
                     text-decoration: none; }}
  .chat-conv-back:hover {{ background: rgba(255,255,255,0.05); }}
  .chat-conv-header-avatar {{ width: 2.25rem; height: 2.25rem; border-radius: 0.75rem;
                               background: linear-gradient(135deg, rgba(124,58,237,0.14) 0%, rgba(6,182,214,0.12) 100%);
                               border: 1px solid rgba(124,58,237,0.16);
                               display: flex; align-items: center; justify-content: center;
                               color: #8b5cf6; flex-shrink: 0; box-shadow: 0 4px 12px rgba(124,58,237,0.12); }}
  .chat-conv-header-titles {{ flex: 1; min-width: 0; }}
  .chat-conv-header-subject {{ font-size: 0.875rem; font-weight: 600; margin: 0; line-height: 1.2;
                                overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }}
  .chat-conv-header-meta {{ display: flex; align-items: center; gap: 0.5rem; margin-top: 0.125rem; }}
  .chat-conv-header-topic {{ font-size: 0.625rem; font-weight: 600; color: #1fc7d4; opacity: 0.8; }}
  .chat-conv-resolve {{ display: inline-flex; align-items: center; gap: 0.375rem;
                        padding: 0.375rem 0.625rem; border-radius: 0.5rem;
                        background: rgba(52,211,153,0.10); color: #34d399;
                        border: 1px solid rgba(52,211,153,0.20);
                        font-size: 0.625rem; font-weight: 600; cursor: pointer; flex-shrink: 0; }}
  .chat-conv-resolve:hover {{ background: rgba(52,211,153,0.20); }}
  .chat-conv-messages {{ flex: 1; overflow-y: auto; }}
  .chat-conv-input {{ flex-shrink: 0; padding: 0.75rem 1rem;
                       background: var(--card-bg, rgba(248,250,252,0.04));
                       border-top: 1px solid var(--card-border, rgba(255,255,255,0.08)); }}
  .chat-conv-input-row {{ display: flex; align-items: flex-end; gap: 0.5rem;
                           padding: 0.5rem 0.75rem; border-radius: 1rem;
                           background: var(--input-bg, rgba(255,255,255,0.05));
                           border: 1px solid var(--glass-border, rgba(255,255,255,0.08)); }}
  .chat-conv-attach {{ width: 1.75rem; height: 1.75rem; border-radius: 0.5rem;
                        background: transparent; border: 0; cursor: pointer;
                        display: flex; align-items: center; justify-content: center;
                        color: var(--text-muted, #94a3b8); opacity: 0.6; flex-shrink: 0; }}
  .chat-conv-attach:hover {{ opacity: 1; }}
  .chat-conv-textarea {{ flex: 1; resize: none; background: transparent; border: 0; outline: none;
                          font-size: 0.875rem; color: var(--text, #fff); padding: 0.25rem 0;
                          min-height: 1.75rem; line-height: 1.4; }}
  .chat-conv-textarea:disabled {{ cursor: not-allowed; opacity: 0.6; }}
  .chat-conv-send {{ width: 2rem; height: 2rem; border-radius: 0.75rem;
                      background: linear-gradient(135deg, #7645d9 0%, #5a33b8 100%);
                      color: #fff; border: 0; cursor: pointer; flex-shrink: 0;
                      display: flex; align-items: center; justify-content: center;
                      box-shadow: 0 4px 12px rgba(118,69,217,0.25); }}
  .chat-conv-send:disabled {{ background: transparent; color: var(--text-muted, #94a3b8);
                               opacity: 0.3; cursor: not-allowed; box-shadow: none; }}
  .chat-conv-hint {{ font-size: 0.625rem; color: var(--text-muted, #94a3b8); opacity: 0.4;
                      text-align: center; margin: 0.375rem 0 0; }}

  /* --- chat full-page immersive + history (stunning v3) --- */
  .chat-page-full {{ height: calc(100dvh - 3.5rem); min-height: 0; overflow: hidden; background: var(--bg); display: flex; flex-direction: column; }}
  .chat-inbox-row.chat-full {{ flex: 1; min-height: 0; height: 100%; border: 0; border-radius: 0; background: transparent; box-shadow: none; backdrop-filter: none; -webkit-backdrop-filter: none; }}
  .chat-inbox-history-sep {{ display: flex; align-items: center; justify-content: space-between; gap: 0.5rem; padding: 0.75rem 0.875rem 0.5rem; font-size: 0.6875rem; font-weight: 700; letter-spacing: 0.06em; text-transform: uppercase; color: var(--text-muted, #94a3b8); border-top: 1px solid rgba(255,255,255,0.06); background: rgba(255,255,255,0.015); }}
  .chat-inbox-history-sep span:first-child {{ opacity: 0.9; }}
  .chat-inbox-history-link {{ font-size: 0.6875rem; font-weight: 600; text-transform: none; letter-spacing: 0; color: #8b5cf6; text-decoration: none; }}
  .chat-inbox-history-link:hover {{ color: #a78bfa; text-decoration: underline; }}
  .chat-inbox-history-badge {{ font-size: 0.625rem; font-weight: 700; padding: 0.125rem 0.375rem; border-radius: 9999px; background: rgba(124,58,237,0.14); color: #a78bfa; border: 1px solid rgba(124,58,237,0.20); letter-spacing: 0.04em; }}
  .chat-inbox-history-bar {{ padding: 0.75rem 0.875rem; border-top: 1px solid rgba(255,255,255,0.07); background: linear-gradient(180deg, rgba(255,255,255,0.03) 0%, rgba(255,255,255,0.015) 100%); display: flex; flex-direction: column; gap: 0.5rem; }}
  .chat-inbox-history-cta {{ display: flex; align-items: center; gap: 0.5rem; padding: 0.625rem 0.75rem; border-radius: 0.75rem; background: rgba(255,255,255,0.06); border: 1px solid rgba(255,255,255,0.08); font-size: 0.8125rem; font-weight: 650; color: var(--text, #fff); text-decoration: none; transition: all 0.15s ease; }}
  .chat-inbox-history-cta:hover {{ background: rgba(255,255,255,0.08); border-color: rgba(124,58,237,0.22); transform: translateY(-1px); box-shadow: 0 4px 12px rgba(0,0,0,0.08); }}
  .chat-inbox-history-count {{ margin-left: auto; min-width: 1.5rem; height: 1.5rem; padding: 0 0.375rem; border-radius: 9999px; background: linear-gradient(135deg, #7c3aed 0%, #06b6d4 100%); color: #fff; font-size: 0.6875rem; font-weight: 800; display: inline-flex; align-items: center; justify-content: center; }}
  .chat-inbox-history-hint {{ font-size: 0.6875rem; color: var(--text-muted, #94a3b8); opacity: 0.6; text-align: center; letter-spacing: 0.01em; }}

  /* Production chat switches from the two-pane desktop inbox to one focused
   * surface on narrow viewports. Conversation cards navigate to `/chat/:id`,
   * while `?new=1` replaces the list with the topic composer. */
  .chat-mobile-back {{ display: none; }}
  @media (max-width: 767px) {{
    .chat-page-full {{ height: calc(100dvh - 3.5rem); height: calc(100svh - 3.5rem); }}
    .chat-inbox-row {{ display: block; }}
    .chat-inbox-row > .chat-inbox {{ width: 100%; height: 100%; border-right: 0; display: flex; }}
    .chat-inbox-row > .chat-panel {{ display: none; width: 100%; }}
    .chat-inbox-row.chat-new-active > .chat-inbox {{ display: none; }}
    .chat-inbox-row.chat-new-active > .chat-panel {{ display: flex; height: 100%; width: 100%; }}
    .chat-mobile-back {{ display: inline-flex; }}
    .chat-conversation {{ width: 100%; max-width: none; padding: 0; }}
    .chat-conv {{ height: calc(100dvh - 3.5rem); min-height: 0; border: 0; border-radius: 0; }}
    .chat-header-row {{ padding-left: 0.75rem; padding-right: 0.75rem; }}
    .chat-messages {{ padding: 1rem 0.75rem; }}
    .chat-input {{ padding-left: 0.75rem; padding-right: 0.75rem; }}
    .chat-inbox-history-bar {{ flex-direction: row; align-items: center; justify-content: space-between; }}
    .chat-inbox-history-hint {{ display: none; }}
    .chat-topic-grid {{ grid-template-columns: 1fr; gap: 0.5rem; }}
    .chat-history {{ padding: 0 1rem; }}
    .chat-history-filters {{ flex-direction: column; gap: 0.625rem; }}
    .chat-history-card {{ padding: 0.875rem 1rem; }}
  }}
  @media (max-width: 480px) {{
    .chat-inbox {{ width: 100%; }}
    .chat-topic-card {{ padding: 0.75rem; }}
    .chat-topic-card-icon {{ width: 2.25rem; height: 2.25rem; }}
    .chat-header-subject {{ font-size: 0.875rem; }}
    .chat-input-row {{ padding: 0.5rem 0.625rem; }}
  }}

  /* --- /notifications list + browser prompt + settings --- */
  .notifications-page {{ max-width: 960px; margin: 0 auto; display: flex; flex-direction: column; gap: 1.5rem;
                         --notification-state-accent: #9a3412; }}
  html.dark .notifications-page {{ --notification-state-accent: #fdba74; }}
  .notifications-list {{ display: flex; flex-direction: column; gap: 0.75rem; }}
  .notifications-filterbar {{ display: flex; align-items: center; gap: 0.5rem; flex-wrap: wrap; }}
  .notifications-filters {{ display: flex; gap: 0.25rem; }}
  .notifications-filterbar-aside {{ margin-left: auto; display: flex; gap: 0.5rem; align-items: center; }}
  .notifications-filter-preview {{ border: 1px solid var(--card-border, rgba(255,255,255,0.08));
                                  background: var(--card-bg, rgba(248,250,252,0.04)); }}
  .notifications-filter-option {{ min-height: 2.5rem; width: 100%; display: flex; align-items: center;
                                  justify-content: space-between; gap: 0.5rem; padding: 0.375rem 0.75rem;
                                  border: 1px solid #cbd5e1; border-radius: 0.5rem;
                                  background: #f8fafc; color: #334155;
                                  font-size: 0.75rem; font-weight: 500; }}
  html.dark .notifications-filter-option {{
    border-color: rgba(71,85,105,0.7);
    background: rgba(51,65,85,0.7);
    color: #e2e8f0;
  }}
  .notifications-filter-option .epsx-icon {{ flex-shrink: 0; opacity: 0.7; }}
  .notifications-unread-count {{ font-size: 0.75rem; color: var(--notification-state-accent); font-weight: 600; opacity: 1; }}
  .notifications-list-card {{ padding: 0; }}
  .notifications-empty {{ padding: 2rem; text-align: center; color: var(--text-muted, #94a3b8); }}
  .notifications-empty > svg {{ color: var(--text-muted, #94a3b8); opacity: 0.4; margin-bottom: 0.5rem; }}
  .notifications-empty-title {{ font-size: 0.875rem; font-weight: 600; margin: 0.5rem 0 0.25rem; color: var(--text, #fff); opacity: 0.7; }}
  .notifications-empty-hint {{ font-size: 0.75rem; margin: 0; opacity: 0.5; }}

  .notification-row {{ display: flex; align-items: flex-start; gap: 0.75rem;
                      padding: 1rem; border-bottom: 1px solid var(--card-border, rgba(255,255,255,0.08));
                      transition: background 0.15s ease; }}
  .notification-row:last-child {{ border-bottom: 0; }}
  .notification-row-unread {{ background: rgba(249,115,22,0.04); }}
  .notification-row-unread:hover {{ background: rgba(249,115,22,0.06); }}
  .notification-row-read {{ opacity: 1; }}
  .notification-row-read:hover {{ opacity: 1; background: rgba(255,255,255,0.02); }}
  .notification-icon {{ width: 2rem; height: 2rem; border-radius: 9999px; flex-shrink: 0;
                        display: flex; align-items: center; justify-content: center;
                        background: rgba(255,255,255,0.06); color: var(--text, #fff); }}
  .notification-icon-payment     {{ background: rgba(96,165,250,0.10);  color: #60a5fa; }}
  .notification-icon-subscription {{ background: rgba(251,191,36,0.10);  color: #fbbf24; }}
  .notification-icon-wallet      {{ background: rgba(34,197,94,0.10);  color: #22c55e; }}
  .notification-icon-news        {{ background: rgba(168,85,247,0.10); color: #a855f7; }}
  .notification-icon-chat        {{ background: rgba(118,69,217,0.10); color: #7645d9; }}
  .notification-icon-alert       {{ background: rgba(239,68,68,0.10);  color: #ef4444; }}
  .notification-icon-system      {{ background: rgba(148,163,184,0.10); color: #94a3b8; }}
  .notification-body {{ flex: 1; min-width: 0; overflow-wrap: anywhere; }}
  .notification-headline {{ display: flex; align-items: flex-start; justify-content: space-between; gap: 0.5rem; min-width: 0; }}
  .notification-title {{ font-size: 0.875rem; margin: 0; line-height: 1.3; font-weight: 600; flex: 1; min-width: 0;
                          overflow-wrap: anywhere; white-space: normal; }}
  .notification-row-unread .notification-title {{ font-weight: 700; color: var(--text, #fff); }}
  .notification-row-read .notification-title   {{ color: var(--text-muted, #94a3b8); font-weight: 400; }}
  .notification-unread-dot {{ width: 0.625rem; height: 0.625rem; border-radius: 9999px;
                               background: var(--notification-state-accent); flex-shrink: 0; margin-top: 0.375rem; }}
  .notification-unread-dot-empty {{ background: transparent; border: 1px solid var(--card-border, rgba(255,255,255,0.20)); }}
  .notification-text {{ font-size: 0.75rem; margin: 0.125rem 0 0; line-height: 1.4;
                        color: var(--text-muted, #94a3b8); overflow-wrap: anywhere; white-space: normal; }}
  .notification-meta {{ display: flex; align-items: center; gap: 0.375rem; margin-top: 0.25rem; flex-wrap: wrap; }}
  .notification-kind, .notification-priority {{ min-width: 0; overflow-wrap: anywhere; }}
  .notification-priority {{ display: inline-flex; align-items: center; max-width: 100%;
                            padding: 0.0625rem 0.375rem; border-radius: 9999px;
                            font-size: 0.625rem; line-height: 1rem; font-weight: 600; }}
  .notification-priority-critical {{ color: #991b1b; background: #fee2e2; }}
  .notification-priority-high {{ color: #9a3412; background: #ffedd5; }}
  .notification-priority-normal {{ color: #1e3a8a; background: #dbeafe; }}
  .notification-priority-low {{ color: #166534; background: #dcfce7; }}
  .notification-priority-neutral {{ color: #334155; background: #e2e8f0; }}
  html.dark .notification-priority-critical {{ color: #fecaca; background: #7f1d1d; }}
  html.dark .notification-priority-high {{ color: #fed7aa; background: #7c2d12; }}
  html.dark .notification-priority-normal {{ color: #dbeafe; background: #1e3a8a; }}
  html.dark .notification-priority-low {{ color: #dcfce7; background: #14532d; }}
  html.dark .notification-priority-neutral {{ color: #f1f5f9; background: #334155; }}
  .notification-time {{ font-size: 0.625rem; color: var(--text-muted, #94a3b8); opacity: 1; }}
  .notification-meta-sep {{ color: var(--text-muted, #94a3b8); opacity: 0.4; }}
  .notification-action {{ font-size: 0.625rem; color: #f97316; text-decoration: underline; }}
  .notification-actions {{ display: flex; align-items: center; gap: 0.25rem; flex-shrink: 0; }}

  /* --- /notifications browser-notifications card --- */
  .browser-notifications {{ }}
  .browser-notifications-header {{ display: flex; align-items: center; justify-content: space-between;
                                    padding: 1rem 1.25rem; border-bottom: 1px solid var(--card-border, rgba(255,255,255,0.08)); }}
  .browser-notifications-title {{ display: flex; align-items: center; gap: 0.5rem; color: #3b82f6; }}
  .browser-notifications-heading {{ font-size: 1rem; font-weight: 600; margin: 0; }}
  .browser-notifications-body {{ padding: 1.25rem; display: flex; flex-direction: column; gap: 1rem; }}
  .browser-notifications-prompt {{ display: flex; flex-direction: column; gap: 0.75rem; align-items: stretch; }}
  .browser-notifications-prompt-text {{ font-size: 0.875rem; color: var(--text-muted, #94a3b8);
                                          line-height: 1.5; margin: 0; }}
  .browser-notifications-prompt-denied {{ flex-direction: row; align-items: flex-start; gap: 0.75rem; }}
  .browser-notifications-prompt-denied > svg {{ color: #ef4444; flex-shrink: 0; margin-top: 0.125rem; }}
  .browser-notifications-enable {{ align-self: flex-start;
                                    background: linear-gradient(90deg, #3b82f6 0%, #2563eb 100%);
                                    color: #fff; padding: 0.5rem 1rem; border-radius: 0.75rem;
                                    border: 0; cursor: pointer; font-weight: 600;
                                    display: inline-flex; align-items: center; gap: 0.5rem;
                                    font-size: 0.875rem; }}
  .browser-notifications-settings {{ display: flex; flex-direction: column; gap: 0.75rem; }}
  .browser-notifications-toggle {{ display: flex; align-items: center; justify-content: space-between;
                                     padding: 0.5rem 0; }}
  .browser-notifications-types {{ display: flex; flex-direction: column; gap: 0.5rem;
                                   padding-left: 1.5rem; border-left: 2px solid var(--card-border, rgba(255,255,255,0.10)); }}
  .browser-notifications-toggle-row {{ display: flex; align-items: center; justify-content: space-between;
                                        padding: 0.5rem 0; font-size: 0.875rem; }}
  .browser-notifications-test {{ align-self: flex-start; margin-top: 0.5rem; }}
  .browser-notifications-footnotes {{ display: flex; flex-direction: column; gap: 0.25rem;
                                       font-size: 0.75rem; color: var(--text-muted, #94a3b8); }}
  .browser-notifications-footnotes p {{ margin: 0; line-height: 1.4; }}

  .permission-badge {{ font-size: 0.6875rem; font-weight: 600; padding: 0.125rem 0.5rem; border-radius: 9999px; }}
  .permission-badge-default {{ background: rgba(148,163,184,0.10); color: #94a3b8; }}
  .permission-badge-granted  {{ background: rgba(52,211,153,0.10);  color: #34d399; }}
  .permission-badge-denied   {{ background: rgba(239,68,68,0.10);  color: #ef4444; }}

  /* --- /notifications per-type settings panel --- */
  .notification-settings-heading {{ display: flex; align-items: center; gap: 0.5rem;
                                     font-size: 1rem; font-weight: 600; margin: 0; }}
  .notification-settings-body {{ padding: 1.25rem; display: flex; flex-direction: column; gap: 0.75rem; }}
  .notification-settings-row {{ display: flex; align-items: center; justify-content: space-between;
                                 padding: 0.5rem 0; }}
  .notification-settings-row-master {{ font-weight: 600; }}
  .notification-settings-types {{ display: flex; flex-direction: column; gap: 0.5rem;
                                   padding-left: 1.5rem; border-left: 2px solid var(--card-border, rgba(255,255,255,0.10)); }}

  /* end wave6-auth-pages-depth-track-c */
  /* === wave6-auth-pages-depth-track-d ===
   * Track D — payment + permissions + portfolio + profile + news +
   * news_detail (6 small/medium pages). Adds the new
   * `<EmptyChartState>` primitive (reused across portfolio chart
   * placeholders), the payment-step indicator, the permissions
   * matrix grid, the profile tab nav, and the news detail hero
   * accent. Keep CSS minimal — the page sections themselves use
   * the existing tailwind utilities; this block only adds the new
   * `.empty-chart-state-*` classes + a couple of helpers used
   * by the page-level section markers. */
  .empty-chart-state {{
    position: relative;
    border: 1px dashed var(--epsx-border, #cbd5e1);
    border-radius: 1rem;
    padding: 3rem 1.5rem;
    background: var(--bg-secondary, #f8fafc);
    text-align: center;
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 0.75rem;
    min-height: 220px;
    justify-content: center;
  }}
  .empty-chart-state-grid {{
    width: 100%;
    max-width: 360px;
    height: 80px;
    background:
      linear-gradient(to right, rgba(34, 211, 238, 0.18) 1px, transparent 1px) 0 0/40px 40px,
      linear-gradient(to bottom, rgba(34, 211, 238, 0.18) 1px, transparent 1px) 0 0/40px 40px;
    border-radius: 0.5rem;
    margin-bottom: 0.5rem;
  }}
  .empty-chart-state-title {{
    font-weight: 600;
    font-size: 1rem;
    color: var(--text, #0f172a);
    margin: 0;
  }}
  .empty-chart-state-cta {{
    margin-top: 0.5rem;
  }}
  .payment-step-indicator {{
    /* the existing .card card-glass + .stepper covers the visual
       step indicator; this class is the section-marker hook
       used by payment.rs's test_section_markers test. */
  }}
  .payment-detail-hero {{
    /* gradient hero wrapper — tailwind gradient utilities already
       carry the visual styling; this class is the section marker. */
  }}
  .permissions-matrix-table th,
  .permissions-matrix-table td {{
    text-align: center;
  }}
  .permissions-matrix-table thead th:first-child,
  .permissions-matrix-table tbody th {{
    text-align: left;
  }}
  .profile-tab-nav {{
    display: flex;
    flex-wrap: wrap;
    gap: 0.5rem;
  }}
  .news-detail-accent {{
    /* the gradient bar between hero and article body — the
       inline gradient classes carry the styling; this class is
       the section marker. */
  }}
  /* end wave6-auth-pages-depth-track-d */

  /* === wave6b-admin-pages-depth-track-a ===
   * Wave 6B Track A — admin shell primitive (sidebar + breadcrumb
   * header + main content) + the 5 admin pages (dashboard,
   * analytics, policies, settings, media). The admin shell
   * structure mirrors the existing `DashboardShell` from
   * `shell.rs`; the per-page rules below are the genuinely new
   * styles required by the section-marker class names
   * (`admin-stats-cards`, `wallets-by-chain`, `policy-stats-bar`,
   * `email-settings`, `media-browser`, etc.). Track B/C/D will add
   * their own blocks under `// === wave6b-admin-pages-depth-track-b/c/d ===`. */

  /* === AdminShell primitive === */
  .admin-shell {{
    display: flex;
    width: 100%;
    height: 100%;
    min-height: 100vh;
    /* `--background` stores HSL channels, matching the `bg-background`
       utility used by the source MainLayout. Keep the shell on that same
       slate surface instead of falling through to the warm document body. */
    background: hsl(var(--background));
  }}
  .admin-shell-sidebar {{
    flex-shrink: 0;
    height: 100%;
  }}
  .admin-shell-header {{
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 0.75rem 1.25rem;
    border-bottom: 1px solid var(--epsx-border, rgba(255,255,255,0.08));
    background: var(--card, rgba(15,23,42,0.6));
    backdrop-filter: blur(12px);
  }}
  .admin-shell-header-left {{
    flex: 1;
    min-width: 0;
  }}
  .admin-shell-header-right {{
    flex-shrink: 0;
  }}
  .admin-shell-page-title {{
    font-size: 1.125rem;
    font-weight: 700;
    color: var(--foreground, #f8fafc);
    margin: 0;
  }}
  .admin-shell-main {{
    flex: 1;
    overflow-y: auto;
    overflow-x: hidden;
  }}

  /* === Dashboard sections === */
  .admin-pulse-header {{
    background: linear-gradient(135deg, rgba(34, 211, 238, 0.04), rgba(99, 102, 241, 0.04), rgba(168, 85, 247, 0.04));
  }}
  .admin-stats-cards .stat-card,
  .admin-stats-grid .stat-card {{
    /* reuse existing `.stat-card` styles; the wrapper class is the
       section marker. */
  }}
  .wallets-by-chain .chart-donut {{
    margin: 0 auto;
  }}
  .recent-transactions .table th {{
    text-transform: uppercase;
    font-size: 0.7rem;
    letter-spacing: 0.05em;
    color: var(--muted-foreground, #94a3b8);
  }}
  .system-alerts .badge {{
    text-transform: uppercase;
    font-size: 0.65rem;
    letter-spacing: 0.05em;
  }}
  .activity-stream {{
    min-height: 480px;
  }}

  /* === Analytics sections === */
  .admin-analytics .status-pill {{
    display: inline-flex;
    align-items: center;
    gap: 0.35rem;
    padding: 0.3rem 0.7rem;
    border-radius: 9999px;
    font-size: 0.7rem;
    font-weight: 700;
    text-transform: uppercase;
    letter-spacing: 0.05em;
  }}
  .admin-analytics .status-pill-live {{
    background: rgba(34, 197, 94, 0.1);
    color: rgb(74, 222, 128);
    border: 1px solid rgba(34, 197, 94, 0.25);
  }}
  .admin-analytics .status-pill-ai {{
    background: rgba(168, 85, 247, 0.1);
    color: rgb(192, 132, 252);
    border: 1px solid rgba(168, 85, 247, 0.25);
  }}
  .analytics-filter-panel .field-label {{
    display: block;
    font-size: 0.7rem;
    text-transform: uppercase;
    letter-spacing: 0.05em;
    color: var(--muted-foreground, #94a3b8);
    margin-bottom: 0.25rem;
    font-weight: 700;
  }}
  .analytics-export-dialog {{
    /* marker-only wrapper — the actual dialog is rendered by
       Wave 6A's `<ExportDialog>` primitive inside this div. */
  }}

  /* === Policies sections === */
  .policy-stats-bar .hover-scale {{
    transition: transform 0.2s ease, box-shadow 0.2s ease;
  }}
  .policy-stats-bar .hover-scale:hover {{
    transform: translateY(-2px);
    box-shadow: 0 12px 32px -8px rgba(0, 0, 0, 0.4);
  }}
  .policy-card .badge {{
    text-transform: uppercase;
    font-size: 0.65rem;
    letter-spacing: 0.05em;
  }}
  .policy-builder .card-body {{
    gap: 1rem;
  }}
  .policy-monitor .pulse-indicator {{
    display: inline-flex;
    align-items: center;
    gap: 0.35rem;
    font-size: 0.7rem;
    text-transform: uppercase;
    letter-spacing: 0.05em;
    color: var(--success, rgb(74, 222, 128));
  }}

  /* === Settings sections === */
  .settings-dashboard {{
    /* the global control bar wrapper. */
  }}
  .email-settings .field,
  .notification-settings .field,
  .session-management .field {{
    display: flex;
    flex-direction: column;
    gap: 0.35rem;
  }}
  .api-keys-list .table-wrap {{
    border-radius: 0.75rem;
    overflow: hidden;
  }}

  /* === Media sections === */
  .media-stats .stat-card {{
    /* reuse the existing `.stat-card` primitive styles. */
  }}
  .media-filters .btn-sm {{
    font-size: 0.75rem;
  }}
  .media-browser .card-body {{
    padding: 0.5rem;
  }}
  .media-browser .card-body p.text-xs {{
    color: var(--muted-foreground, #94a3b8);
  }}

  /* end wave6b-admin-pages-depth-track-a */

  /* === wave6b-admin-pages-depth-track-b ===
   * Wave 6B Track B — content-moderation pages depth: audit_log +
   * news + notifications. All rules below are scoped to the new
   * section-marker class names added by the Track B page ports in
   * `shared/rust/dioxus_ui/src/pages/admin_pages/audit_log.rs` +
   * `news.rs` + `notifications.rs` and the new
   * `feedback/admin_action_confirm.rs` primitive. The marker region
   * is the only shared file surface with Tracks A/C/D (which use
   * `// === wave6b-admin-pages-depth-track-a/c/d ===`).
   *
   * We deliberately reuse the existing design-system classes
   * (`.card`, `.card-glass`, `.btn`, `.btn-primary`, `.btn-outline`,
   * `.btn-danger`, `.btn-warning`, `.text-muted-foreground`,
   * `.text-destructive`, `.text-foreground`, etc.) — only the new
   * Wave 6B surface-area selectors are defined here. No new colors,
   * no new design tokens. */

  /* --- AdminActionConfirm primitive --- */
  .admin-action-confirm-overlay {{
    position: fixed; inset: 0; z-index: 50;
    background: rgba(0, 0, 0, 0.6);
    display: flex; align-items: center; justify-content: center;
    padding: 1rem;
  }}
  .admin-action-confirm-panel {{
    border-radius: 1rem;
    background: var(--card, #0b0f1a);
    border: 1px solid var(--epsx-border, rgba(255, 255, 255, 0.08));
    box-shadow: 0 25px 50px -12px rgba(0, 0, 0, 0.5);
    padding: 1.5rem;
    max-width: 28rem;
    width: 100%;
  }}
  .admin-action-confirm-title {{
    font-size: 1.125rem; font-weight: 700; color: var(--foreground, #fff);
    margin: 0 0 0.5rem;
  }}
  .admin-action-confirm-message {{
    font-size: 0.875rem; color: var(--text-muted, #94a3b8);
    margin: 0 0 1rem;
  }}
  .admin-action-confirm-actions {{
    display: flex; gap: 0.75rem; justify-content: flex-end;
  }}
  .admin-action-confirm-actions .btn {{
    padding: 0.5rem 1rem; border-radius: 0.5rem;
    font-size: 0.875rem; font-weight: 500;
    transition: all 0.15s ease;
  }}
  .admin-action-confirm-actions .btn-outline {{
    border: 1px solid var(--epsx-border, rgba(255, 255, 255, 0.08));
    background: transparent;
  }}
  .admin-action-confirm-actions .btn-outline:hover {{
    background: var(--muted, rgba(255, 255, 255, 0.04));
  }}
  .admin-action-confirm-actions .btn-danger {{
    background: var(--destructive, #ef4444); color: #fff;
  }}
  .admin-action-confirm-actions .btn-danger:hover {{
    background: var(--destructive-hover, #dc2626);
  }}
  .admin-action-confirm-actions .btn-warning {{
    background: var(--warning, #f59e0b); color: #fff;
  }}
  .admin-action-confirm-actions .btn-warning:hover {{
    background: var(--warning-hover, #d97706);
  }}
  .admin-action-confirm-actions .btn-primary {{
    background: var(--epsx-primary, #3b82f6); color: #fff;
  }}
  .admin-action-confirm-actions .btn-primary:hover {{
    background: var(--epsx-primary-hover, #2563eb);
  }}

  /* --- /admin/audit-log --- 5 sections: filters / timeline / detail / severity / export --- */
  .audit-filters {{ /* the top filter strip — uses the same card/border styles as the existing data-table-toolbar */ }}
  .audit-filters-pills {{ /* category filter pill row; scroll-x on mobile */ }}
  .audit-filters-pill[data-category="all"] {{
    /* the 'All Actions' pill is the default; uses the muted class by default */
  }}
  .audit-filters-pill[data-category].active {{
    background: linear-gradient(90deg, #7645d9, #5a33b8);
    color: #fff;
    box-shadow: 0 4px 6px -1px rgba(0, 0, 0, 0.1);
  }}
  .audit-filters-date-from, .audit-filters-date-to {{
    /* date range inputs — inherit the .input style */ }}
  .audit-timeline {{ /* outer wrapper for the list + pagination */ }}
  .audit-timeline-row {{ /* one log entry row */ }}
  .audit-timeline-action {{ /* the colored action pill inside a row */ }}
  .audit-timeline-pagination {{ /* footer pagination row */ }}
  .audit-entry-detail {{ /* expand-into view shown when a row is clicked */ }}
  .audit-entry-detail-header {{ /* result badge + resource badge + action label */ }}
  .audit-entry-detail-result {{ /* success / denied / error badge */ }}
  .audit-entry-detail-resource {{ /* resource type badge (blue) */ }}
  .audit-entry-detail-meta {{ /* actor / target / timestamp / IP grid */ }}
  .audit-entry-detail-changes {{ /* per-shape diff section (before/after, permission, etc.) */ }}
  .audit-severity-breakdown {{ /* sidebar panel: per-category counts */ }}
  .audit-severity-row {{ /* one row in the breakdown (label + bar) */ }}
  .audit-export-button {{ /* top-right CSV/JSON button pair */ }}

  /* --- /admin/news --- 6 sections: list / editor / featured / card / empty / pagination --- */
  .news-management-list {{ /* outer container for the news list */ }}
  .news-management-filters {{ /* status filter pills (all/draft/published) + count */ }}
  .news-management-articles {{ /* article list body */ }}
  .news-featured-card {{ /* pinned-article highlight (cyan border + gradient bar) */ }}
  .news-featured-card-cover {{ /* cover image slot, gradient placeholder */ }}
  .news-featured-card-pinned {{ /* the 'Pinned' badge */ }}
  .news-featured-card-title {{ /* article title (larger than card rows) */ }}
  .news-featured-card-meta {{ /* author + date line */ }}
  .news-featured-card-actions {{ /* right-side edit/view buttons */ }}
  .news-editor {{ /* outer wrapper for the create/edit form */ }}
  .news-editor-header {{ /* sticky header with status toggle + save button */ }}
  .news-editor-save {{ /* the primary save button in the header */ }}
  .article-card {{ /* one article row: cover + title + tags + actions */ }}
  .article-card-cover {{ /* cover thumbnail slot */ }}
  .article-card-title {{ /* article title in a row */ }}
  .article-card-status {{ /* status badge (draft / published) */ }}
  .article-card-actions {{ /* pin/publish/edit/delete icon buttons */ }}
  .news-empty-state {{ /* empty state when 0 articles */ }}
  .news-pagination {{ /* prev/next page controls */ }}

  /* --- /admin/notifications --- 7 sections: list / form / recipients / template / preview / schedule / filters --- */
  .notification-list {{ /* outer container for the notification list */ }}
  .notification-list-row {{ /* one notification row */ }}
  .notification-list-priority {{ /* priority badge (critical / high / normal / low) */ }}
  .notification-list-actions {{ /* hover-revealed delete button */ }}
  .send-form {{ /* compose form wrapper */ }}
  .recipients-picker {{ /* targeted client vs. global broadcast toggle */ }}
  .notification-template-editor {{ /* title / body / action URL / image URL */ }}
  .notification-preview {{ /* live preview of the notification card */ }}
  .notification-schedule-dialog {{ /* schedule-for-later toggle + datetime picker */ }}
  .notification-management-filters {{ /* filter chips row (all/sent/scheduled/draft) */ }}
  .notification-filter-chip {{ /* one filter chip */ }}
  .notification-stats-grid {{ /* 4-stat-card grid (Total Sent / Today's Pulse / Weekly Volume / System Health) */ }}
  .notification-stat-card {{ /* one stats card */ }}
  .notification-action-buttons {{ /* 2-col grid for Synchronize / Analytics */ }}
  .notification-sync-btn {{ /* synchronize telemetry button */ }}
  .notification-analytics-btn {{ /* analytics deep-dive button */ }}

  /* end wave6b-admin-pages-depth-track-b */

  /* === wave6b-admin-pages-depth-track-c ===
   * Wave 6B Track C — financial-surface pages (payments +
   * wallet_credits + wallet_plans + wallet_access) +
   * <AdminTable> primitive. All rules below are scoped to the new
   * section-marker class names added by the Track C page ports in
   * `shared/rust/dioxus_ui/src/pages/admin_pages/<payments,
   * wallet_credits,wallet_plans,wallet_access>.rs` and the
   * `primitives/admin_table.rs` primitive. We deliberately reuse
   * the existing design-system classes (`.card`, `.card-glass`,
   * `.card-header`, `.card-body`, `.btn`, `.tab`, etc.) — these
   * are the few genuinely new rules that the markers introduced.
   * The marker region is the only shared file surface with Tracks
   * A/B/D (which use
   * `// === wave6b-admin-pages-depth-track-<a,b,d> ===`). */

  /* === admin_table primitive === */
  .admin-table {{ /* extends .data-table; uses shared toolbar + pagination */ }}
  .admin-table-toolbar {{ display: flex; flex-wrap: wrap; gap: 0.5rem; align-items: center; padding: 0.5rem 0; }}
  .admin-table-chips {{ display: flex; flex-wrap: wrap; gap: 0.25rem; }}
  .admin-table-chips .chip {{ font-size: 0.75rem; padding: 0.25rem 0.625rem; border-radius: 999px; background: rgba(255, 255, 255, 0.04); border: 1px solid rgba(255, 255, 255, 0.08); cursor: pointer; transition: all 0.15s ease; }}
  .admin-table-chips .chip-active {{ background: rgba(99, 102, 241, 0.15); border-color: rgba(99, 102, 241, 0.45); color: rgb(165, 180, 252); }}
  .admin-table-filter {{ flex: 1; min-width: 200px; }}
  .admin-table-count {{ white-space: nowrap; }}
  .admin-table-actions {{ white-space: nowrap; }}
  .admin-table-actions .btn {{ margin-left: 0.25rem; }}
  .admin-table-action-delete {{ color: rgb(239, 68, 68); }}
  .admin-table-action-revoke {{ color: rgb(245, 158, 11); }}
  .admin-table-pagination {{ padding: 0.75rem 0; border-top: 1px solid rgba(255, 255, 255, 0.06); }}

  /* === payments === */
  .payments-stats {{ /* grid layout, no extra rules needed */ }}
  .payments-filter-panel {{ /* rounded card; uses shared .input + .btn */ }}
  .payment-links-list {{ /* uses shared .data-table + gradient header */ }}
  .access-management-list {{ /* uses shared .data-table + gradient header */ }}
  .create-link-form {{ /* 2-col form; uses shared .input */ }}
  .link-revoke-confirm {{ /* destructive confirm card */ }}

  /* === wallet_credits === */
  .credits-ledger {{ /* page chrome; uses shared .input */ }}
  .credits-balance-cards {{ /* 4-col grid of .credits-balance-card */ }}
  .credits-balance-card {{ /* uses existing .rounded-xl + .border styles */ }}
  .credits-breakdown-card {{ /* uses existing .card + .card-glass */ }}
  .credits-transaction-list {{ /* uses shared .data-table */ }}
  .credits-topup-form {{ /* grant form; uses existing .input */ }}
  .credits-revoke-dialog {{ /* destructive confirm card */ }}

  /* === wallet_plans === */
  .plan-list-sidebar {{ /* 4-group list; uses existing .input + .btn */ }}
  .plan-item-card {{ /* sidebar row; uses existing .border-l-4 */ }}
  .plan-item-card:hover {{ background: rgba(99, 102, 241, 0.04); }}
  .plan-editor-page {{ /* full-page editor layout */ }}
  .plan-editor-drawer {{ /* slide-in drawer — hidden by default */ }}
  .plan-api-limits {{ /* gradient card; uses existing .input */ }}
  .plan-promotions {{ /* gradient card; uses existing .input */ }}

  /* === wallet_access === */
  .wallet-access-manager {{ /* 2-col grid; uses existing .input */ }}
  .plan-selector-modal {{ /* centered modal — hidden by default */ }}
  .plan-selector-modal[open], .plan-selector-modal.show {{ display: flex; }}
  .access-grant-form {{ /* grant form; uses existing .input */ }}
  .access-revoke-dialog {{ /* destructive confirm card */ }}

  /* end wave6b-admin-pages-depth-track-c */

  /* === wave6b-admin-pages-depth-track-d ===
   * Track D — wallet_wallets + chat + developer_portal + auth_page
   * (5 pages, plus the new `<AdminMetricCard>` primitive). Adds the
   * admin-metric-card visual primitives, the platform-distribution
   * bar, the chat-inbox / conversation view / reply input layout, the
   * developer-portal stat-card and module-card styles, and the
   * auth-method-selector panels. Keep CSS minimal — the page
   * sections themselves use the existing tailwind utilities; this
   * block only adds the new `.admin-metric-card-*`,
   * `.platform-distribution-*`, `.conversation-card-*`, and
   * `.auth-method-selector-*` classes. */
  .admin-metric-card {{
    position: relative;
    padding: 1rem;
  }}
  .admin-metric-card-header {{
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 0.5rem;
  }}
  .admin-metric-card-label-row {{
    display: flex;
    align-items: center;
    gap: 0.5rem;
    min-width: 0;
  }}
  .admin-metric-card-icon {{
    display: inline-flex;
    align-items: center;
    justify-content: center;
    width: 1.75rem;
    height: 1.75rem;
    border-radius: 0.5rem;
    background: var(--bg-tertiary, rgba(255,255,255,0.05));
  }}
  .admin-metric-card-label {{
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }}
  .admin-metric-card-value {{
    font-variant-numeric: tabular-nums;
    line-height: 1.2;
  }}
  .admin-metric-trend {{
    padding: 0.125rem 0.5rem;
    border-radius: 9999px;
    white-space: nowrap;
  }}
  .admin-metric-trend-up {{
    background: rgba(16, 185, 129, 0.1);
    color: rgb(16, 185, 129);
  }}
  .admin-metric-trend-down {{
    background: rgba(239, 68, 68, 0.1);
    color: rgb(239, 68, 68);
  }}
  .admin-metric-trend-flat {{
    background: rgba(148, 163, 184, 0.1);
    color: rgb(148, 163, 184);
  }}
  .admin-metric-card-sparkline {{
    color: rgb(34, 211, 238);
    opacity: 0.85;
  }}
  .platform-distribution-card {{
    padding: 1rem 1.25rem;
  }}
  .wallet-card-avatar {{
    position: relative;
    width: 3rem;
    height: 3rem;
    flex-shrink: 0;
  }}
  .wallet-card-avatar-bg {{
    position: absolute;
    inset: 0;
    border-radius: 1rem;
    background: linear-gradient(135deg, #1fc7d4 0%, #7645d9 100%);
    filter: blur(8px);
    opacity: 0.2;
  }}
  .wallet-card-avatar-text {{
    position: relative;
    display: flex;
    align-items: center;
    justify-content: center;
    width: 100%;
    height: 100%;
    border-radius: 1rem;
    background: linear-gradient(135deg, #1fc7d4 0%, #7645d9 100%);
    color: white;
    font-weight: 900;
    font-size: 0.875rem;
  }}
  .wallet-card-sections {{
    /* The mobile-card variant of the wallet list row. */
  }}
  .wallet-table-row {{
    display: grid;
    grid-template-columns: 30% 20% 20% 30%;
    gap: 0.5rem;
    padding: 0.5rem 0;
    align-items: center;
    border-bottom: 1px solid var(--epsx-border, rgba(255,255,255,0.06));
  }}
  .wallet-detail-view {{
    /* Wrapper for the per-wallet detail view. */
  }}
  .wallet-detail-panel {{
    /* Right-hand panel of the detail view. */
  }}
  .wallet-disable-dialog,
  .wallet-reenable-dialog,
  .api-key-revoke-modal {{
    /* The disable / re-enable / revoke modals. Inline `alert-dialog`
       classes carry the modal styling; this class is the section
       marker. */
  }}
  .admin-chat-page {{
    /* Container for the admin chat inbox + conversation. */
  }}
  .admin-chat-inbox-container {{
    min-height: 24rem;
  }}
  .admin-chat-conversation-container {{
    min-height: 24rem;
  }}
  .conversation-card {{
    /* Individual conversation card in the inbox. */
  }}
  .chat-inbox-search {{
    background: var(--bg-secondary, rgba(255,255,255,0.02));
  }}
  .chat-reply-input {{
    background: var(--card-bg, rgba(255,255,255,0.02));
  }}
  .canned-responses-popover,
  .assign-agent-popover {{
    /* Inline popovers in the chat reply input. */
  }}
  .chat-unread-badge {{
    display: inline-flex;
    align-items: center;
    justify-content: center;
    min-width: 1.25rem;
    height: 1.25rem;
    padding: 0 0.375rem;
    border-radius: 9999px;
    background: rgb(239, 68, 68);
    color: white;
    font-size: 0.625rem;
    font-weight: 700;
  }}
  .admin-chat-conversation-view {{
    /* Wrapper for the per-conversation view. */
  }}
  .developer-portal-stats,
  .developer-portal-overview {{
    /* Container for the developer portal stats + overview. */
  }}
  .api-keys-tab,
  .usage-analytics-tab,
  .documentation-tab {{
    /* Per-tab containers in the developer portal. */
  }}
  .api-key-create-form {{
    /* The create-key form. */
  }}
  .auth-method-selector {{
    /* The "Pick a sign-in method" panel. */
  }}
  .auth-redirect-handler {{
    /* The auto-redirect UI shown briefly before the redirect. */
  }}
  /* end wave6b-admin-pages-depth-track-d */

  /* Respect the same global reduced-motion preference as the development
     frontend/admin styles. Zero delays keep delayed `both` entrances from
     holding content at their initial keyframe while motion is reduced. */
  @media (prefers-reduced-motion: reduce) {{
    *,
    *::before,
    *::after {{
      animation-duration: 0.01ms !important;
      animation-delay: 0ms !important;
      animation-iteration-count: 1 !important;
      transition-duration: 0.01ms !important;
      transition-delay: 0ms !important;
      scroll-behavior: auto !important;
    }}
  }}
</style>"##,
        keywords_meta = keywords_meta,
    )
}

/// Returns the external wasm-bindgen module generated from the Rust browser runtime.
/// The referenced files live under `target/` and are never committed.
/// Bump the revision when the bootstrap/runtime contract changes so an already-open
/// browser cannot keep executing an older ES module graph after a local rebuild.
pub fn global_js() -> &'static str {
    r#"<script type="module" src="/runtime/epsx_browser_runtime_bootstrap.js?rev=2" data-epsx-generated-runtime="wasm-bindgen"></script>"#
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
/// - 3 dropdowns: Market / Developer / Company
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

    // Developer dropdown items
    //
    // **Wave 49+** — matches prod's `NAV_GROUPS` in
    // `apps/frontend/components/nav/nav-config.ts` (Developer
    // group has exactly 2 items: API Keys, Documentation). The
    // previous dev version also rendered a "Usage" item pointing
    // at `/developer/usage` — prod has no such link in the
    // navbar (Usage is reached from the Developer Portal page,
    // not the global nav). Removing the third item brings the
    // dev BFF to 2+2+4 = 8 nav items, identical to prod.
    let developer_items = format!(
        "{}{}",
        nav_item("/developer", "key", "API Keys", "API access status"),
        nav_item(
            "/developer/docs",
            "book",
            "Documentation",
            "Pinned API reference"
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
                    copy_icon = lucide("copy", "14", "epsx-action-icon"),
                    logout_icon = lucide("log-out", "14", "epsx-action-icon"),
                ),
            )
        } else {
            (
                format!(
                    r##"<div class="epsx-desktop-session hidden md:flex items-center gap-1.5">
        <a href="/account" class="epsx-theme-btn" aria-label="Account">{account_icon}</a>
        <button class="epsx-connect-btn" type="button" data-epsx-logout>{logout_icon} Sign out</button>
      </div>"##,
                    account_icon = lucide("user", "16", "epsx-action-icon"),
                    logout_icon = lucide("log-out", "16", "epsx-action-icon"),
                ),
                format!(
                    r##"<div class="epsx-tablet-session hidden sm:flex md:hidden items-center gap-1.5">
        <button class="epsx-connect-btn" type="button" data-epsx-logout style="height:2rem;padding:0 0.75rem;font-size:0.75rem;border-radius:1rem;">{logout_icon} Sign out</button>
      </div>"##,
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
      <button class="epsx-mobile-link" type="button" data-epsx-logout style="width:100%;border:0;background:transparent;text-align:left;">
        {logout_icon} Sign out
      </button>"##,
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
        "Developer" => path_is_active("/developer"),
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
    let mobile_developer_items = format!(
        "{}{}",
        mobile_item("/developer", "key", "API Keys"),
        mobile_item("/developer/docs", "book", "Documentation")
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
        {developer}
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
      {mobile_developer}
      {mobile_company}
    </nav>
    <div class="epsx-mobile-session">
      {mobile_auth}
    </div>
  </div>
</div>"##,
        logo = logo,
        market = nav_block("Market", "chart-column", &market_items),
        developer = nav_block("Developer", "code", &developer_items),
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
        mobile_developer = mobile_group("Developer", "code", &mobile_developer_items),
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
        assert!(shell.contains("epsx_browser_runtime_bootstrap.js?rev=2"));
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
            assert_eq!(header.matches("data-epsx-action=\"toggle-nav\"").count(), 6);
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
        assert!(header.contains("id=\"epsx-mobile-developer-trigger\""));
        assert!(header.contains("id=\"epsx-mobile-company-trigger\""));
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
