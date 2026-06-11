//! EPSX design system — shared HTML template helpers.
//!
//! Every BFF (frontend, admin, pay, preview) calls `design_system_head()` to
//! emit the same `<head>` block (system-ui font stack, Tailwind CDN, CSS
//! variables, glassmorphism utilities, animations, dark/light mode FOUC
//! prevention).
//!
//! All visual changes across the platform should go through this module so we
//! can match the original Next.js design without duplicating CSS strings.

pub mod components;

/// Returns the full `<head>` block matching the original Next.js design.
///
/// Includes:
/// - system-ui font stack via CSS variable `--font-sans` (no external font
///   network round-trip — matches epsx.io which uses platform defaults)
/// - Tailwind v2.2.19 CDN (we keep the older CDN for stability with our
///   utility classes; the design intent is identical to v4)
/// - Complete CSS variable system for light + dark mode
/// - Glassmorphism, gradient text, gradient orbs, shadows, hover effects
/// - FOUC prevention script that applies the saved theme before first paint
/// - Toast / modal / dropdown / tab / chat-widget global controllers
pub fn design_system_head(title: &str, description: &str) -> String {
    format!(
        r##"<meta charset="utf-8" />
<meta name="viewport" content="width=device-width, initial-scale=1, maximum-scale=5, user-scalable=yes, viewport-fit=cover" />
<meta name="description" content="{description}" />
<meta name="theme-color" content="#ffffff" media="(prefers-color-scheme: light)" />
<meta name="theme-color" content="#000000" media="(prefers-color-scheme: dark)" />
<title>{title}</title>
<link rel="stylesheet" href="https://cdn.jsdelivr.net/npm/tailwindcss@2.2.19/dist/tailwind.min.css" />
<script>
  // FOUC prevention: apply theme before first paint
  (function() {{
    try {{
      var t = localStorage.getItem('epsx_theme') || 'dark';
      if (t === 'system') {{
        t = window.matchMedia('(prefers-color-scheme: dark)').matches ? 'dark' : 'light';
      }}
      if (t === 'light') document.documentElement.classList.remove('dark');
      else document.documentElement.classList.add('dark');
    }} catch (e) {{}}
  }})();
</script>
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
    --border:          #e2e8f0;
    --border-strong:   #cbd5e1;
    --text:            #0f172a;
    --text-muted:      #475569;
    --text-subtle:     #64748b;
    --primary:         #3b82f6;
    --primary-hover:   #2563eb;
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

    /* Font — epsx.io uses system-ui, not Kanit */
    --font-sans:       ui-sans-serif, system-ui, -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, "Helvetica Neue", Arial, "Apple Color Emoji", "Segoe UI Emoji", "Segoe UI Symbol", "Noto Color Emoji", sans-serif;
  }}

  html.dark {{
    --bg:              #030712;
    --bg-secondary:    #0f172a;
    --bg-tertiary:     #1e293b;
    --surface:         rgba(15, 23, 42, 0.80);
    --surface-hover:   rgba(15, 23, 42, 0.95);
    --surface-solid:   #0f172a;
    --border:          #1e293b;
    --border-strong:   #334155;
    --text:            #f1f5f9;
    --text-muted:      #94a3b8;
    --text-subtle:     #64748b;
    --shadow-sm:       0 1px 2px 0 rgba(0, 0, 0, 0.4);
    --shadow:          0 4px 6px -1px rgba(0, 0, 0, 0.5), 0 2px 4px -2px rgba(0, 0, 0, 0.3);
    --shadow-lg:       0 10px 15px -3px rgba(0, 0, 0, 0.6), 0 4px 6px -4px rgba(0, 0, 0, 0.4);
    --shadow-xl:       0 20px 25px -5px rgba(0, 0, 0, 0.7), 0 8px 10px -6px rgba(0, 0, 0, 0.4);
    --shadow-2xl:      0 25px 50px -12px rgba(0, 0, 0, 0.8);
    --shadow-orange:   0 20px 25px -5px rgba(249, 115, 22, 0.50);
    --gradient-page:   linear-gradient(135deg, #0f172a 0%, #1e1b4b 50%, #0c0a09 100%);
    --gradient-card:   linear-gradient(135deg, rgba(59,130,246,0.10) 0%, rgba(168,85,247,0.10) 100%);
    --glass-bg:        rgba(15, 23, 42, 0.80);
    --glass-border:    rgba(249, 115, 22, 0.30);
    --glass-shadow:    0 8px 32px rgba(0, 0, 0, 0.40);
  }}

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

  /* === Gradient orbs (decorative blur) === */
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
    border: 1px solid var(--border);
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
    border: 1px solid var(--border);
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
    border-top: 1px solid var(--border);
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
    border: 1px solid var(--border);
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
    background: var(--primary);
    color: white;
  }}
  .btn-primary:hover {{ background: var(--primary-hover); box-shadow: var(--shadow); }}
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
    color: var(--text);
    border: 1px solid var(--border-strong);
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
  .badge-primary {{ background: rgba(59,130,246,0.15); color: #3b82f6; }}
  .badge-success {{ background: rgba(16,185,129,0.15); color: #10b981; }}
  .badge-warning {{ background: rgba(245,158,11,0.15); color: #f59e0b; }}
  .badge-danger  {{ background: rgba(239,68,68,0.15);  color: #ef4444; }}
  .badge-info    {{ background: rgba(6,182,212,0.15);  color: #06b6d4; }}
  .badge-purple  {{ background: rgba(168,85,247,0.15); color: #a855f7; }}
  .badge-pink    {{ background: rgba(236,72,153,0.15); color: #ec4899; }}
  .badge-pending {{ background: rgba(245,158,11,0.15); color: #f59e0b; }}
  .badge-active  {{ background: rgba(16,185,129,0.15); color: #10b981; }}
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
    border: 1px solid var(--border);
    color: var(--text-muted);
    backdrop-filter: blur(8px);
  }}

  /* === Form === */
  .input {{
    width: 100%;
    padding: 0.625rem 1rem;
    border-radius: 0.625rem;
    background: var(--bg-secondary);
    border: 1px solid var(--border);
    color: var(--text);
    font-family: inherit;
    font-size: 0.875rem;
    transition: all 0.2s ease;
  }}
  .input:focus {{
    outline: none;
    border-color: var(--primary);
    box-shadow: 0 0 0 3px rgba(59,130,246,0.15);
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
    border: 1px solid var(--border);
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
  .toast-info    {{ border-left: 3px solid var(--primary); }}
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
    border: 1px solid var(--border);
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
    border: 1px solid var(--border);
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
  .auth-wallet-icon {{
    font-size: 1.5rem;
    width: 1.5rem; height: 1.5rem;
    display: inline-flex; align-items: center; justify-content: center;
    flex-shrink: 0;
  }}
  .auth-wallet-name {{ font-size: 1rem; font-weight: 500; color: #ffffff; }}
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
    border: 1px solid var(--border);
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
    border-top: 1px solid var(--border);
    text-align: center;
    font-size: 0.8125rem;
    color: var(--text-subtle);
  }}
  html.dark .epsx-modal-footer {{ border-top-color: rgba(51,65,85,0.5); color: #94a3b8; }}

  /* === Dropdown === */
  .dropdown-menu {{
    position: absolute;
    background: var(--surface-solid);
    border: 1px solid var(--border);
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
    border-bottom: 1px solid var(--border);
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

  /* === Section === */
  .section {{ padding: 4rem 1.5rem; }}
  .section-tight {{ padding: 2.5rem 1.5rem; }}
  @media (min-width: 640px) {{ .section {{ padding: 5rem 2rem; }} }}
  @media (min-width: 1024px) {{ .section {{ padding: 6rem 2rem; }} }}

  /* === Container === */
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
    border-bottom: 1px solid var(--border);
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
    border: 1px solid var(--border);
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
    border-left: 1px solid var(--border);
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
    border-top: 1px solid var(--border);
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
    display: inline-flex; align-items: center; gap: 0.25rem;
    padding: 0.375rem 0.75rem;
    font-size: 0.875rem; font-weight: 500;
    color: var(--text-muted);
    border-radius: 0.375rem;
    background: transparent; border: none; cursor: pointer;
    transition: color 0.15s ease;
  }}
  .epsx-nav-trigger:hover {{ color: var(--text); }}
  html.dark .epsx-nav-trigger {{ color: #94a3b8; }}
  html.dark .epsx-nav-trigger:hover {{ color: white; }}
  .epsx-nav-trigger .nav-icon {{ color: var(--epsx-orange); width: 1rem; height: 1rem; flex-shrink: 0; }}
  .epsx-nav-trigger .nav-chev {{ color: var(--epsx-orange); width: 0.75rem; height: 0.75rem; transition: transform 0.2s ease; }}
  .epsx-nav-wrap.open .epsx-nav-trigger .nav-chev {{ transform: rotate(180deg); }}

  /* === Nav dropdown content (epsx.io Radix menu) === */
  .epsx-nav-menu {{
    position: absolute; top: calc(100% + 0.5rem); left: 0;
    min-width: 13rem;
    background: var(--surface-solid);
    border: 1px solid var(--border);
    border-radius: 0.5rem;
    box-shadow: var(--shadow-xl);
    padding: 0.375rem;
    z-index: 9999;
    display: none;
    animation: scaleIn 0.15s ease;
  }}
  .epsx-nav-wrap.open .epsx-nav-menu {{ display: block; }}
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

  /* === Theme toggle (sun/moon) === */
  .epsx-theme-btn {{
    display: inline-flex; align-items: center; justify-content: center;
    width: 2.5rem; height: 2.5rem;
    border-radius: 0.5rem;
    background: var(--bg-secondary);
    color: var(--text-muted);
    border: 1px solid var(--border);
    cursor: pointer;
    transition: all 0.15s ease;
  }}
  .epsx-theme-btn:hover {{ background: var(--bg-tertiary); color: var(--text); }}
  html.dark .epsx-theme-btn {{ background: #1e293b; border-color: #334155; color: #cbd5e1; }}
  html.dark .epsx-theme-btn:hover {{ background: #334155; color: white; }}

  /* === Mobile menu sheet (< 640px) === */
  .epsx-mobile-sheet {{
    position: fixed;
    inset: 0;
    z-index: 100;
    background: rgba(0,0,0,0.5);
    backdrop-filter: blur(4px);
    -webkit-backdrop-filter: blur(4px);
    display: flex;
    align-items: flex-end;
    justify-content: center;
    padding: 0;
  }}
  .epsx-mobile-sheet-inner {{
    width: 100%;
    max-height: 85vh;
    overflow-y: auto;
    background: var(--bg);
    border-top-left-radius: 1.5rem;
    border-top-right-radius: 1.5rem;
    padding: 1.5rem;
    box-shadow: 0 -25px 50px -12px rgba(0,0,0,0.5);
  }}
  html.dark .epsx-mobile-sheet-inner {{ background: #0f172a; border-top: 1px solid #334155; }}
  .epsx-mobile-section {{
    margin-bottom: 1rem;
    padding-bottom: 1rem;
    border-bottom: 1px solid var(--border);
  }}
  .epsx-mobile-section:last-of-type {{ border-bottom: none; }}
  .epsx-mobile-section-title {{
    font-size: 0.75rem;
    font-weight: 700;
    color: var(--text-subtle);
    text-transform: uppercase;
    letter-spacing: 0.1em;
    margin-bottom: 0.5rem;
  }}
  .epsx-mobile-link {{
    display: flex; align-items: center; gap: 0.625rem;
    padding: 0.75rem 0.5rem;
    border-radius: 0.5rem;
    color: var(--text);
    text-decoration: none;
    font-size: 0.9375rem;
    font-weight: 500;
    transition: background 0.15s ease;
  }}
  .epsx-mobile-link:hover, .epsx-mobile-link:active {{ background: var(--bg-secondary); }}

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
    border: 1px solid var(--border);
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
  .table-wrap {{ overflow-x: auto; border-radius: 0.75rem; border: 1px solid var(--border); }}
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
    border-bottom: 1px solid var(--border);
  }}
  .table td {{
    padding: 0.875rem 1rem;
    border-bottom: 1px solid var(--border);
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
    border-left: 1px solid var(--border);
    padding: 1rem;
    overflow-y: auto;
    animation: slideInRight 0.25s ease-out;
  }}
  .mobile-sheet .hamburger {{ display: block; }}
  @media (min-width: 1024px) {{ .mobile-sheet .hamburger {{ display: none; }} }}

  /* === Utility === */
  .text-balance {{ text-wrap: balance; }}
  .text-pretty  {{ text-wrap: pretty; }}
  .divide-y > * + * {{ border-top: 1px solid var(--border); }}
  .ring-1 {{ box-shadow: 0 0 0 1px var(--border); }}
  .scrollbar-thin::-webkit-scrollbar {{ width: 6px; height: 6px; }}
  .scrollbar-thin::-webkit-scrollbar-thumb {{ background: var(--border-strong); border-radius: 3px; }}

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
    border: 1px solid var(--border);
    border-radius: 0.75rem;
    padding: 1.25rem 1.25rem 1rem;
    background: var(--bg-secondary);
  }}
  .form-section-header {{
    margin-bottom: 1rem;
  }}
  .form-section-title {{
    font-size: 1rem;
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
    color: var(--primary);
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
    border-bottom: 1px solid var(--border);
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
    border: 1px solid var(--border);
    border-radius: 0.625rem;
  }}
  .multiselect-control:focus-within {{
    border-color: var(--primary);
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
    border: 1px dashed var(--border-strong);
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
    border: 1px solid var(--border);
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
    border: 1px solid var(--border);
    border-radius: 0.625rem;
  }}
  .combobox-multi-control:focus-within {{
    border-color: var(--primary);
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
    outline: 2px solid var(--primary);
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
  .SwitchInput:checked {{ background: var(--primary); }}
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
    border: 1px solid var(--border-strong);
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
    background: var(--primary);
    border: 2px solid var(--surface-solid);
    box-shadow: var(--shadow);
  }}
  .slider::-moz-range-thumb {{
    width: 1.125rem;
    height: 1.125rem;
    border-radius: 9999px;
    background: var(--primary);
    border: 2px solid var(--surface-solid);
    box-shadow: var(--shadow);
  }}
  .slider:focus {{
    outline: 2px solid var(--primary);
    outline-offset: 2px;
  }}
  .slider:disabled {{ opacity: 0.5; cursor: not-allowed; }}

  /* === Checkbox indeterminate (visual) === */
  .checkbox-indeterminate {{
    background: var(--primary);
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
    border-top: 1px solid var(--border);
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
    border-bottom: 1px solid var(--border);
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
    border-right: 1px solid var(--border);
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
    border: 1px solid var(--border);
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
    border: 1px solid var(--border);
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
    border: 1px solid var(--border);
    border-radius: 0.5rem;
    overflow: hidden;
  }}
  .accordion-item {{
    border-bottom: 1px solid var(--border);
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
    border: 1px solid var(--border);
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
    border-bottom: 1px solid var(--border);
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
    color: var(--primary);
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
    border-bottom: 1px solid var(--border);
    background: var(--surface-solid);
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
  .admin-nav-row {{
    text-decoration: none;
  }}
  .admin-nav-row-active {{
    font-weight: 600;
  }}
  .admin-footer {{
    flex-shrink: 0;
  }}
  .admin-header-bell {{
    position: relative;
  }}
  .admin-header-theme-toggle {{
    /* matches the `btn-ghost btn-icon` look for the default theme toggle */
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
    color: #ffffff;
    border: 1px solid var(--border);
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
    border-right: 1px solid var(--border);
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

  /* --- auth gate (sign-in / permission-missing / admin variants) --- */
  .auth-gate {{
    display: flex; flex-direction: column; align-items: center; justify-content: center;
    text-align: center;
    padding: 3rem 1.5rem;
    max-width: 32rem; margin: 4rem auto;
    background: var(--surface-solid, #191923);
    border: 1px solid var(--border);
    border-radius: 1.5rem;
    box-shadow: 0 0 50px -12px rgba(0,0,0,0.5);
    color: #ffffff;
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
  .auth-gate-description {{ color: rgba(255,255,255,0.65); margin: 0 0 1.5rem 0; line-height: 1.5; }}
  .auth-gate-perms {{
    text-align: left;
    background: rgba(255,255,255,0.04);
    border: 1px solid var(--border);
    border-radius: 0.75rem;
    padding: 0.875rem 1rem;
    margin: 0 0 1.5rem 0;
    width: 100%;
  }}
  .auth-gate-perms p {{ margin: 0 0 0.5rem 0; font-size: 0.75rem; color: rgba(255,255,255,0.55); text-transform: uppercase; letter-spacing: 0.05em; }}
  .auth-gate-perms ul {{ margin: 0; padding-left: 1.25rem; font-family: ui-monospace, SFMono-Regular, Menlo, monospace; font-size: 0.8125rem; color: rgba(255,255,255,0.85); }}
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
  .access-denied-title {{ font-size: 1.75rem; font-weight: 700; margin: 0 0 0.75rem 0; color: var(--text); }}
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
    border: 1px solid var(--border);
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

  /* --- connected wallet dropdown (provider card + actions + nav + disconnect) --- */
  .connected-wallet-dropdown {{
    background: var(--surface-solid, #191923);
    color: #ffffff;
    border: 1px solid var(--border);
    border-radius: 1rem;
    box-shadow: 0 0 50px -12px rgba(0,0,0,0.5);
    width: 18rem;
    overflow: hidden;
    animation: scaleIn 0.15s ease;
  }}
  .wallet-provider-card {{
    padding: 1rem;
    background: linear-gradient(135deg, rgba(255,255,255,0.04), rgba(255,255,255,0.01));
    border-bottom: 1px solid var(--border);
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
    border-top: 1px solid var(--border);
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
    border-top: 1px solid var(--border);
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
    border-top: 1px solid var(--border);
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
  .auth-page-pitch-inner {{ position: relative; z-index: 1; max-width: 36rem; }}
  .auth-page-brand {{ margin-bottom: 3rem; font-size: 2rem; font-weight: 900; font-style: italic; letter-spacing: -0.02em; text-transform: uppercase; }}
  .auth-page-brand a {{ color: inherit; text-decoration: none; }}
  .auth-page-headline {{
    font-size: 3.5rem; line-height: 1.1; font-weight: 800; margin: 0 0 1.5rem;
  }}
  @media (min-width: 1280px) {{ .auth-page-headline {{ font-size: 4.5rem; }} }}
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
    padding: 2rem 1.5rem; width: 100%;
  }}
  @media (min-width: 1024px) {{ .auth-page-form-col {{ width: 40%; backdrop-filter: blur(24px); border-left: 1px solid rgba(0, 0, 0, 0.06); }} }}
  .auth-page-form-inner {{ width: 100%; max-width: 28rem; display: flex; flex-direction: column; gap: 1.5rem; }}
  .auth-card {{ padding: 2.5rem 2rem; display: flex; flex-direction: column; gap: 1.25rem; }}
  .auth-card-title {{ font-size: 1.5rem; font-weight: 800; margin: 0; color: rgb(24, 24, 27); text-align: center; }}
  .auth-card-sub {{ font-size: 0.9375rem; color: rgb(113, 113, 122); margin: 0; text-align: center; }}
  .auth-card-cta {{ width: 100%; }}
  .auth-card-cta .connect-btn {{ width: 100%; justify-content: center; height: 3.5rem; font-size: 1.125rem; font-weight: 700; }}
  .auth-card-divider {{ display: flex; align-items: center; gap: 1rem; color: rgb(161, 161, 170); font-size: 0.75rem; font-weight: 600; text-transform: uppercase; letter-spacing: 0.1em; margin: 0.5rem 0; }}
  .auth-card-divider::before, .auth-card-divider::after {{ content: ""; flex: 1; height: 1px; background: rgba(0, 0, 0, 0.08); }}
  .auth-card-divider-thin {{ margin: 0; }}
  .auth-card-email-form {{ display: flex; flex-direction: column; gap: 0.5rem; }}
  .auth-card-email-input {{ width: 100%; }}
  .auth-card-google-btn {{ gap: 0.5rem; }}
  .auth-card-google-glyph {{
    display: inline-flex; align-items: center; justify-content: center;
    width: 1.25rem; height: 1.25rem; border-radius: 0.25rem; font-weight: 900; color: #4285F4;
  }}
  .auth-card-foot {{ font-size: 0.75rem; color: rgb(113, 113, 122); text-align: center; margin: 0; line-height: 1.6; }}
  .auth-card-foot a {{ color: rgb(82, 82, 91); text-decoration: underline; text-underline-offset: 2px; }}
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
  .auth-page-fallback {{ text-align: center; font-size: 0.75rem; }}
  .auth-page-fallback a {{ color: rgb(113, 113, 122); text-decoration: underline; text-underline-offset: 2px; }}

  /* === About page sections === */
  .about-hero-section {{ padding: 6rem 0 3rem; text-align: center; }}
  .about-hero-content {{ max-width: 48rem; margin: 0 auto; }}
  .about-hero-title {{
    font-size: 3.5rem; line-height: 1.1; font-weight: 800; margin: 0 0 1.5rem;
    background: linear-gradient(to right, rgb(249, 115, 22), rgb(234, 179, 8), rgb(234, 88, 12));
    -webkit-background-clip: text; background-clip: text; color: transparent;
  }}
  @media (min-width: 640px) {{ .about-hero-title {{ font-size: 4rem; }} }}
  .about-hero-sub {{ font-size: 1.125rem; line-height: 1.7; color: rgb(82, 82, 91); margin: 0 auto 2rem; max-width: 48rem; }}
  .about-hero-underline {{ width: 10rem; height: 0.25rem; margin: 0 auto; border-radius: 9999px; background: linear-gradient(to right, rgb(249, 115, 22), rgb(234, 179, 8), rgb(234, 88, 12)); }}

  .mission-section {{ padding: 5rem 0; }}
  .mission-grid {{ display: grid; grid-template-columns: repeat(auto-fit, minmax(300px, 1fr)); gap: 2rem; }}
  .mission-card {{ padding: 2.5rem 2rem; }}
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

  .datatech-section {{ padding: 5rem 0; }}
  .datatech-overview-grid {{ display: grid; grid-template-columns: 1fr; gap: 1.5rem; margin-top: 2.5rem; }}
  @media (min-width: 1024px) {{ .datatech-overview-grid {{ grid-template-columns: 2fr 1fr; }} }}
  .datatech-card {{ padding: 2.5rem 2rem; position: relative; overflow: hidden; }}
  .datatech-card-title {{
    font-size: 1.5rem; font-weight: 800; margin: 0 0 1rem;
    background: linear-gradient(to right, rgb(249, 115, 22), rgb(234, 179, 8));
    -webkit-background-clip: text; background-clip: text; color: transparent;
  }}
  .datatech-card-body {{ font-size: 1rem; line-height: 1.7; color: rgb(63, 63, 70); margin: 0 0 1rem; }}
  .datatech-card-body:last-child {{ margin-bottom: 0; }}
  .datatech-highlight {{ background: linear-gradient(to right, rgb(249, 115, 22), rgb(234, 179, 8)); -webkit-background-clip: text; background-clip: text; color: transparent; font-weight: 700; }}
  .datatech-text-orange {{ color: rgb(194, 65, 12); font-weight: 600; }}
  .datatech-text-blue {{ color: rgb(29, 78, 216); font-weight: 600; }}
  .datatech-why-list {{ list-style: none; padding: 0; margin: 0; display: flex; flex-direction: column; gap: 0.75rem; }}
  .datatech-why-list li {{ display: flex; align-items: center; gap: 0.5rem; font-size: 0.875rem; color: rgb(63, 63, 70); }}
  .datatech-why-check {{ color: rgb(16, 185, 129); font-weight: 700; }}
  .datatech-card-why .datatech-card-title {{ background: linear-gradient(to right, rgb(59, 130, 246), rgb(6, 182, 212)); -webkit-background-clip: text; background-clip: text; color: transparent; }}

  .datatech-features-grid {{ display: grid; grid-template-columns: repeat(auto-fit, minmax(280px, 1fr)); gap: 1.5rem; margin-top: 1.5rem; }}
  .datatech-feature {{ padding: 2rem 1.5rem; position: relative; overflow: hidden; }}
  .datatech-feature-title {{
    font-size: 1.25rem; font-weight: 800; margin: 0 0 0.75rem; background-clip: text; -webkit-background-clip: text; color: transparent;
  }}
  .datatech-feature-orange .datatech-feature-title {{ background-image: linear-gradient(to right, rgb(249, 115, 22), rgb(234, 179, 8)); }}
  .datatech-feature-blue .datatech-feature-title   {{ background-image: linear-gradient(to right, rgb(59, 130, 246), rgb(6, 182, 212)); }}
  .datatech-feature-purple .datatech-feature-title {{ background-image: linear-gradient(to right, rgb(168, 85, 247), rgb(236, 72, 153)); }}
  .datatech-feature-green .datatech-feature-title  {{ background-image: linear-gradient(to right, rgb(16, 185, 129), rgb(5, 150, 105)); }}
  .datatech-feature-red .datatech-feature-title    {{ background-image: linear-gradient(to right, rgb(239, 68, 68), rgb(249, 115, 22)); }}
  .datatech-feature-indigo .datatech-feature-title {{ background-image: linear-gradient(to right, rgb(99, 102, 241), rgb(168, 85, 247)); }}
  .datatech-feature-body {{ font-size: 0.9375rem; line-height: 1.6; color: rgb(63, 63, 70); margin: 0 0 0.75rem; }}
  .datatech-feature-detail {{ font-size: 0.8125rem; line-height: 1.6; color: rgb(113, 113, 122); margin: 0; }}

  .datatech-benefits {{ padding: 2.5rem 2rem; margin-top: 1.5rem; position: relative; overflow: hidden; }}
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
   * ports in `shared/rust/dioxus_ui/src/pages/*.rs`. We deliberately
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
  .manual-feature-grid {{ display: grid; grid-template-columns: repeat(auto-fill, minmax(260px, 1fr)); gap: 1rem; padding: 0.5rem 0 1rem; }}
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
    border-bottom: 1px solid var(--border, rgba(255,255,255,0.08));
  }}
  .plans-comparison-table thead th {{ font-weight: 600; }}
  .plans-comparison-feature-col {{ width: 40%; }}
  .plans-comparison-col-featured {{ color: var(--primary, #3b82f6); }}
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
  .contact-page {{ position: relative; z-index: 1; padding-bottom: 4rem; }}
  .contact-bg {{
    position: fixed; inset: 0; z-index: 0; pointer-events: none; overflow: hidden;
    background: linear-gradient(135deg, #eff6ff 0%, #fff7ed 50%, #fefce8 100%);
  }}
  :root.dark .contact-bg {{
    background: linear-gradient(135deg, #0f172a 0%, #1e293b 50%, #0f172a 100%);
  }}
  .contact-bg-orb {{ position: absolute; border-radius: 9999px; filter: blur(80px); opacity: 0.35; }}
  .contact-bg-orb-1 {{ width: 24rem; height: 24rem; top: -6rem; left: -8rem; }}
  .contact-bg-orb-2 {{ width: 20rem; height: 20rem; top: 5rem; right: -6rem; }}
  .contact-bg-orb-3 {{ width: 18rem; height: 18rem; bottom: 0; left: 30%; }}
  .contact-bg-orb-4 {{ width: 16rem; height: 16rem; bottom: 6rem; right: 8rem; opacity: 0.25; }}
  .contact-hero {{ padding: 4rem 0 2rem; text-align: center; position: relative; }}
  .contact-hero-title {{
    font-size: 3rem; font-weight: 800; margin: 0 0 1rem;
    background: linear-gradient(90deg, #a855f7, #f97316, #eab308);
    -webkit-background-clip: text; background-clip: text; color: transparent;
  }}
  .contact-hero-subtitle {{ max-width: 36rem; margin: 0 auto; color: var(--text-muted, #475569); }}
  .contact-hero-divider {{
    width: 10rem; height: 0.25rem; margin: 1.5rem auto 0;
    background: linear-gradient(90deg, #a855f7, #f97316, #eab308); border-radius: 9999px;
  }}
  .contact-email-section {{ padding: 1rem 0 3rem; }}
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
  .contact-email-title {{ font-size: 1.25rem; font-weight: 700; margin: 0 0 0.5rem; }}
  .contact-email-subtitle {{ margin: 0 0 1.25rem; }}
  .contact-mailto-btn {{ display: inline-flex; gap: 0.5rem; align-items: center; }}
  .contact-email-divider {{ height: 1px; background: rgba(0,0,0,0.06); margin: 1rem 0; }}
  :root.dark .contact-email-divider {{ background: rgba(255,255,255,0.06); }}
  .contact-copy-btn {{ display: inline-flex; gap: 0.4rem; align-items: center; }}
  .contact-info-section {{ padding: 0 0 3rem; }}
  .contact-info-grid {{ display: grid; grid-template-columns: repeat(auto-fit, minmax(220px, 1fr)); gap: 1rem; max-width: 64rem; margin: 0 auto; }}
  .contact-info-card {{
    background: rgba(255,255,255,0.8); backdrop-filter: blur(20px);
    border-radius: 1.5rem; box-shadow: 0 10px 25px -5px rgba(0,0,0,0.1);
    padding: 0;
  }}
  :root.dark .contact-info-card {{ background: rgba(30, 41, 59, 0.8); }}
  .contact-info-row {{ display: flex; gap: 1rem; align-items: flex-start; padding: 1.25rem; }}
  .contact-info-icon {{
    display: inline-flex; padding: 0.75rem; border-radius: 1rem; flex-shrink: 0;
  }}
  .contact-info-icon-purple {{ background: linear-gradient(135deg, #a855f7, #3b82f6); }}
  .contact-info-icon-orange {{ background: linear-gradient(135deg, #f97316, #eab308); }}
  .contact-info-icon-blue {{ background: linear-gradient(135deg, #3b82f6, #06b6d4); }}
  .contact-info-title {{ font-weight: 600; margin: 0 0 0.25rem; }}
  .contact-info-desc {{ margin: 0; }}
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
    min-height: 80vh; display: flex; align-items: center; justify-content: center;
    padding: 2rem 1rem;
    background: linear-gradient(135deg, #f8fafc, #f1f5f9);
  }}
  :root.dark .offline-page {{ background: linear-gradient(135deg, #0f172a, #1e293b); }}
  .offline-card {{ max-width: 32rem; width: 100%; padding: 2.5rem 2rem; text-align: center; }}
  .offline-icon {{
    display: inline-flex; padding: 1.25rem; border-radius: 9999px;
    background: rgba(249, 115, 22, 0.1); color: #f97316; margin-bottom: 1.5rem;
  }}
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
  .offline-actions {{ display: flex; flex-direction: column; gap: 0.75rem; }}
  .offline-actions-row {{ display: flex; gap: 0.5rem; }}
  .offline-actions-row .btn {{ flex: 1; display: inline-flex; align-items: center; justify-content: center; gap: 0.4rem; }}
  .offline-tip {{
    margin-top: 1.5rem; padding-top: 1rem; border-top: 1px solid rgba(0,0,0,0.06);
    font-size: 0.75rem; color: var(--text-muted, #94a3b8);
  }}
  :root.dark .offline-tip {{ border-top-color: rgba(255,255,255,0.06); }}
  .offline-tip-label {{ font-weight: 500; margin: 0 0 0.25rem; }}
  .offline-tip-text {{ margin: 0; }}

  .access-denied-page {{ max-width: 42rem; margin: 0 auto; padding: 2rem 1rem; }}
  .access-denied-reasons {{ margin-top: 2rem; }}
  .access-denied-reasons-card {{ padding: 0; }}
  .access-denied-reasons-title {{ font-size: 1rem; font-weight: 600; margin: 0 0 0.75rem; }}
  .access-denied-reasons-list {{ list-style: none; padding: 0; margin: 0; display: flex; flex-direction: column; gap: 0.5rem; }}
  .access-denied-reasons-item {{ display: flex; gap: 0.5rem; align-items: flex-start; font-size: 0.875rem; color: var(--text-muted, #94a3b8); }}
  .access-denied-reasons-bullet {{ color: var(--text-muted, #94a3b8); }}

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
    border: 1px dashed var(--border, #cbd5e1);
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
</style>"##
    )
}

/// Returns the global JavaScript controllers that should be loaded on every
/// page. The functions are namespaced under `window.epsx` so BFFs can call
/// `epsx.toast(...)`, `epsx.modal(...)`, etc.
pub fn global_js() -> &'static str {
    r#"<script>
window.epsx = (function() {
  // ============ Toast ============
  function ensureToastHost() {
    let host = document.getElementById('epsx-toast-host');
    if (!host) {
      host = document.createElement('div');
      host.id = 'epsx-toast-host';
      host.className = 'toast-host';
      document.body.appendChild(host);
    }
    return host;
  }
  function toast(message, kind) {
    kind = kind || 'info';
    const host = ensureToastHost();
    const el = document.createElement('div');
    el.className = 'toast toast-' + kind;
    const icon = {success:'check-circle',error:'xmark-circle',info:'info-circle',warning:'exclamation-triangle'}[kind] || 'info-circle';
    el.innerHTML = '<i data-lucide="' + icon + '" style="width:1rem;height:1rem;margin-top:2px;"></i><div style="flex:1;font-size:0.875rem;">' + message + '</div>';
    host.appendChild(el);
    setTimeout(() => { el.style.opacity = '0'; el.style.transform = 'translateX(20px)'; el.style.transition = 'all 0.3s ease'; }, 3500);
    setTimeout(() => el.remove(), 4000);
  }

  // ============ Theme ============
  function setTheme(t) {
    if (t === 'system') {
      t = window.matchMedia('(prefers-color-scheme: dark)').matches ? 'dark' : 'light';
    }
    if (t === 'light') document.documentElement.classList.remove('dark');
    else document.documentElement.classList.add('dark');
    try { localStorage.setItem('epsx_theme', t); } catch (e) {}
    const btn = document.getElementById('epsx-theme-toggle');
    if (btn) updateThemeIcon(btn, t);
    if (typeof updateThemeBtns === 'function') updateThemeBtns();
  }
  function currentTheme() {
    return document.documentElement.classList.contains('dark') ? 'dark' : 'light';
  }
  function updateThemeIcon(btn, t) {
    const sun  = btn.querySelector('[data-icon="sun"]');
    const moon = btn.querySelector('[data-icon="moon"]');
    if (sun) sun.style.display = (t === 'light') ? 'none' : '';
    if (moon) moon.style.display = (t === 'dark') ? 'none' : '';
  }
  function toggleTheme() {
    const next = currentTheme() === 'dark' ? 'light' : 'dark';
    setTheme(next);
  }

  // ============ Modal ============
  function openModal(html) {
    closeModal();
    const back = document.createElement('div');
    back.className = 'modal-backdrop';
    back.id = 'epsx-modal-backdrop';
    back.innerHTML = '<div class="modal">' + html + '</div>';
    back.addEventListener('click', (e) => { if (e.target === back) closeModal(); });
    document.body.appendChild(back);
  }
  function closeModal() {
    const back = document.getElementById('epsx-modal-backdrop');
    if (back) back.remove();
  }

  // ============ Dropdown ============
  function toggleDropdown(id) {
    const menu = document.getElementById(id);
    if (!menu) return;
    const open = menu.style.display === 'block';
    document.querySelectorAll('.dropdown-menu').forEach(m => m.style.display = 'none');
    menu.style.display = open ? 'none' : 'block';
  }
  document.addEventListener('click', (e) => {
    if (!e.target.closest('[data-dropdown-trigger]') && !e.target.closest('.dropdown-menu')) {
      document.querySelectorAll('.dropdown-menu').forEach(m => m.style.display = 'none');
    }
  });

  // ============ Mobile sheet ============
  function openSheet(id) { const el = document.getElementById(id); if (el) el.classList.add('open'); }
  function closeSheet(id) { const el = document.getElementById(id); if (el) el.classList.remove('open'); }

  // ============ Desktop nav dropdown ============
  function toggleNavDropdown(id) {
    const wrap = document.getElementById(id);
    if (!wrap) return;
    const willOpen = !wrap.classList.contains('open');
    document.querySelectorAll('.nav-dropdown-wrap').forEach(w => w.classList.remove('open'));
    if (willOpen) wrap.classList.add('open');
  }
  document.addEventListener('click', (e) => {
    if (!e.target.closest('.nav-dropdown-wrap')) {
      document.querySelectorAll('.nav-dropdown-wrap').forEach(w => w.classList.remove('open'));
    }
  });
  document.addEventListener('keydown', (e) => {
    if (e.key === 'Escape') {
      document.querySelectorAll('.nav-dropdown-wrap').forEach(w => w.classList.remove('open'));
    }
  });

  // ============ Mobile nav accordion ============
  function toggleNavAccordion(id) {
    document.getElementById(id)?.classList.toggle('open');
  }

  // ============ EPSX.io-style nav (Market/Developer/Company) ============
  function toggleNav(btn) {
    const wrap = btn.closest('.epsx-nav-wrap');
    if (!wrap) return;
    const willOpen = !wrap.classList.contains('open');
    document.querySelectorAll('.epsx-nav-wrap').forEach(w => w.classList.remove('open'));
    if (willOpen) {
      wrap.classList.add('open');
      if (window.epsx && window.epsx.initLucide) window.epsx.initLucide();
    }
  }
  // ============ Mobile menu sheet (visible < 640px) ============
  function toggleMobileMenu() {
    let sheet = document.getElementById('epsx-mobile-sheet');
    if (sheet) { sheet.remove(); return; }
    sheet = document.createElement('div');
    sheet.id = 'epsx-mobile-sheet';
    sheet.className = 'epsx-mobile-sheet';
    sheet.innerHTML = `
      <div class="epsx-mobile-sheet-inner animate-slide-in">
        <div class="flex items-center justify-between mb-4">
          <span class="text-sm font-semibold uppercase tracking-wider" style="color:var(--text-muted);">Menu</span>
          <button class="epsx-theme-btn" type="button" aria-label="Close menu" onclick="epsx.toggleMobileMenu()" style="width:2.25rem;height:2.25rem;padding:0;">
            <i data-lucide="x" style="width:1.125rem;height:1.125rem;"></i>
          </button>
        </div>
        <div class="epsx-mobile-section">
          <div class="epsx-mobile-section-title">Market</div>
          <a href="/rankings" class="epsx-mobile-link"><i data-lucide="chart-column" style="width:1rem;height:1rem;color:var(--epsx-orange);"></i> Rankings <span style="color:var(--text-subtle);font-size:0.75rem;">EPS stock rankings</span></a>
          <a href="/portfolio" class="epsx-mobile-link"><i data-lucide="trending-up" style="width:1rem;height:1rem;color:var(--epsx-orange);"></i> Portfolio <span style="color:var(--text-subtle);font-size:0.75rem;">Watchlist &amp; tracking</span></a>
        </div>
        <div class="epsx-mobile-section">
          <div class="epsx-mobile-section-title">Developer</div>
          <a href="/developer" class="epsx-mobile-link"><i data-lucide="key" style="width:1rem;height:1rem;color:var(--epsx-orange);"></i> API Keys</a>
          <a href="/developer/docs" class="epsx-mobile-link"><i data-lucide="book" style="width:1rem;height:1rem;color:var(--epsx-orange);"></i> Documentation</a>
          <a href="/developer/usage" class="epsx-mobile-link"><i data-lucide="layout-dashboard" style="width:1rem;height:1rem;color:var(--epsx-orange);"></i> Usage</a>
        </div>
        <div class="epsx-mobile-section">
          <div class="epsx-mobile-section-title">Company</div>
          <a href="/about" class="epsx-mobile-link"><i data-lucide="info" style="width:1rem;height:1rem;color:var(--epsx-orange);"></i> About</a>
          <a href="/news" class="epsx-mobile-link"><i data-lucide="newspaper" style="width:1rem;height:1rem;color:var(--epsx-orange);"></i> News</a>
          <a href="/contact" class="epsx-mobile-link"><i data-lucide="mail" style="width:1rem;height:1rem;color:var(--epsx-orange);"></i> Contact</a>
          <a href="/chat" class="epsx-mobile-link"><i data-lucide="help-circle" style="width:1rem;height:1rem;color:var(--epsx-orange);"></i> Support</a>
        </div>
        <button class="epsx-connect-btn" type="button" onclick="epsx.toggleMobileMenu();epsx.openAuth();" style="margin-top:1rem;width:100%;">
          <i data-lucide="wallet" style="width:1rem;height:1rem;"></i> Connect Wallet
        </button>
      </div>
    `;
    document.body.appendChild(sheet);
    if (window.epsx && window.epsx.initLucide) window.epsx.initLucide();
  }
  // Click-outside to close
  document.addEventListener('click', (e) => {
    if (!e.target.closest('.epsx-nav-wrap')) {
      document.querySelectorAll('.epsx-nav-wrap').forEach(w => w.classList.remove('open'));
    }
  });
  document.addEventListener('keydown', (e) => {
    if (e.key === 'Escape') {
      document.querySelectorAll('.epsx-nav-wrap').forEach(w => w.classList.remove('open'));
    }
  });

  // ============ Auth modal (epsx.io "Select Wallet" exact structure) ============
  function openAuth() {
    closeAuth();
    const back = document.createElement('div');
    back.className = 'modal-backdrop';
    back.id = 'epsx-auth-back';
    back.setAttribute('role', 'dialog');
    back.setAttribute('aria-modal', 'true');
    back.innerHTML = authHTML();
    back.addEventListener('click', (e) => { if (e.target === back) closeAuth(); });
    document.body.appendChild(back);
    if (window.epsx && window.epsx.initLucide) window.epsx.initLucide();
  }
  function closeAuth() {
    const back = document.getElementById('epsx-auth-back');
    if (back) back.remove();
  }
  function authHTML() {
    // Mirror epsx.io's auth-modal-inner / auth-wallet-btn structure exactly
    return `
      <div class="auth-modal-inner animate-zoom-in" style="max-width:420px;width:100%;">
        <div class="auth-modal-content">
          <div class="auth-step auth-step-enter">
            <div class="auth-step-header">
              <span class="auth-step-number">1</span>
              <span class="auth-step-label">Select Wallet</span>
            </div>
            <div class="auth-wallets">
              <button class="auth-wallet-btn" type="button" onclick="window.location.href='/api/v1/auth/siwe?provider=safe'">
                <span class="auth-wallet-icon">💼</span>
                <span class="auth-wallet-name">Safe</span>
              </button>
              <button class="auth-wallet-btn" type="button" onclick="window.location.href='/api/v1/auth/siwe?provider=walletconnect'">
                <span class="auth-wallet-icon">🔗</span>
                <span class="auth-wallet-name">WalletConnect</span>
              </button>
              <button class="auth-wallet-btn" type="button" onclick="window.location.href='/api/v1/auth/siwe?provider=base'">
                <span class="auth-wallet-icon">💼</span>
                <span class="auth-wallet-name">Base Account</span>
              </button>
            </div>
          </div>
        </div>
        <div class="auth-modal-footer">
          <p class="auth-footer-text">By connecting, you agree to our <a href="/terms" style="color:rgba(255,255,255,0.7);text-decoration:underline;">Terms of Service</a>.</p>
        </div>
      </div>
    `;
  }

  // Update theme toggle visibility
  function updateThemeBtns() {
    const t = currentTheme();
    document.querySelectorAll('.epsx-theme-btn').forEach(btn => {
      const sun  = btn.querySelector('.sun');
      const moon = btn.querySelector('.moon');
      if (sun)  sun.style.display  = (t === 'light') ? '' : 'none';
      if (moon) moon.style.display = (t === 'dark')  ? '' : 'none';
    });
  }

  // ============ Tabs ============
  function activateTab(group, name) {
    document.querySelectorAll('[data-tab-group="' + group + '"]').forEach(el => {
      el.classList.toggle('active', el.getAttribute('data-tab-name') === name);
      el.style.display = el.getAttribute('data-tab-name') === name ? '' : 'none';
    });
  }

  // ============ Init theme icon ============
  document.addEventListener('DOMContentLoaded', () => {
    const btn = document.getElementById('epsx-theme-toggle');
    if (btn) updateThemeIcon(btn, currentTheme());
    updateThemeBtns();
  });

  // ============ API client (fetch wrappers) ============
  async function apiGet(path) {
    try {
      const r = await fetch(path, { headers: { 'Accept': 'application/json' } });
      if (!r.ok) throw new Error('HTTP ' + r.status);
      return await r.json();
    } catch (e) {
      console.warn('apiGet(' + path + ') failed:', e);
      return null;
    }
  }
  async function apiPost(path, body) {
    try {
      const r = await fetch(path, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json', 'Accept': 'application/json' },
        body: JSON.stringify(body || {}),
      });
      if (!r.ok) throw new Error('HTTP ' + r.status);
      return await r.json();
    } catch (e) {
      console.warn('apiPost(' + path + ') failed:', e);
      return null;
    }
  }
  // ============ Countdown timer (used on home pricing cards) ============
  function startCountdown(el, hoursTotal) {
    if (!el) return;
    function tick() {
      const total = hoursTotal * 3600;
      const d = Math.floor(total / 86400);
      const h = Math.floor((total % 86400) / 3600);
      const m = Math.floor((total % 3600) / 60);
      const s = total % 60;
      let txt = 'Ends in ';
      if (d > 0) txt += d + 'd ';
      txt += h + 'h ' + m + 'm';
      if (d === 0) txt += ' ' + s + 's';
      el.textContent = txt;
    }
    tick();
    setInterval(tick, 1000);
  }
  // ============ Renderer: company card from API JSON ============
  function companyCardHTML(c) {
    const growthPct = Math.min(100, c.growth_pct / 50);
    return `
      <div class="company-card">
        <div class="epsx-blob" style="top:0;right:0;width:8rem;height:8rem;background:rgba(59,130,246,0.1);transform:translate(2.5rem,-2.5rem);"></div>
        <div class="epsx-blob" style="bottom:0;left:0;width:6rem;height:6rem;background:rgba(168,85,247,0.1);transform:translate(-2.5rem,2.5rem);"></div>
        <div class="text-center mb-4">
          <h3 style="font-size:0.75rem;font-weight:700;color:var(--text-subtle);text-transform:uppercase;letter-spacing:0.1em;margin-bottom:0.25rem;">RANK #${c.rank}</h3>
          <div class="flex items-center justify-center mb-0.5"><span class="text-4xl font-black tracking-tighter" style="color:#3b82f6;">${c.ticker}</span></div>
          <div style="font-size:0.875rem;font-weight:600;color:var(--text-muted);margin-top:0.125rem;">${c.price}</div>
        </div>
        <div class="space-y-2 mb-4 flex-grow">
          <div class="row-card flex items-center justify-between">
            <div class="flex items-center gap-2">
              <span class="row-icon" style="background:rgba(16,185,129,0.2);color:#10b981;"><i data-lucide="trending-up" style="width:1rem;height:1rem;"></i></span>
              <span style="font-size:0.875rem;color:var(--text-muted);font-weight:500;">Growth</span>
            </div>
            <span style="font-weight:700;font-size:0.875rem;color:#10b981;">${c.growth}</span>
          </div>
          <div class="progress-track"><div class="progress-fill" style="width:${growthPct}%;background:linear-gradient(90deg,#10b981,#34d399);"><div class="progress-shine"></div></div></div>
          <div class="row-card flex items-center justify-between">
            <div class="flex items-center gap-2">
              <span class="row-icon" style="background:rgba(59,130,246,0.2);color:#3b82f6;"><i data-lucide="calendar" style="width:0.875rem;height:0.875rem;"></i></span>
              <span style="font-size:0.75rem;color:var(--text-subtle);font-weight:500;text-transform:uppercase;letter-spacing:0.05em;">Next Action</span>
            </div>
            <span style="font-weight:700;font-size:0.875rem;color:var(--text);">${c.next_action_days} Days</span>
          </div>
          <div class="progress-track"><div class="progress-fill" style="width:${c.next_action_pct}%;background:linear-gradient(90deg,#3b82f6,#06b6d4);"><div class="progress-shine"></div></div></div>
        </div>
        <a href="${c.tradingview_url}" target="_blank" rel="noopener noreferrer" class="block w-full mt-auto">
          <button class="view-btn">View Details <i data-lucide="arrow-right" style="width:1rem;height:1rem;display:inline;margin-left:0.25rem;"></i></button>
        </a>
      </div>
    `;
  }
  // ============ Bootstrap API data on home + /rankings pages ============
  async function loadRankings() {
    const grid = document.getElementById('rankings-grid');
    if (!grid) return;
    const empty = document.getElementById('rankings-grid-empty');
    if (empty) empty.remove();
    const data = await apiGet('/api/v1/rankings');
    if (data && data.companies) {
      grid.innerHTML = data.companies.map(companyCardHTML).join('');
      if (window.epsx && window.epsx.initLucide) window.epsx.initLucide();
    }
  }
  function startCountdowns() {
    document.querySelectorAll('[data-countdown-hours]').forEach(el => {
      const h = parseInt(el.getAttribute('data-countdown-hours'), 10) || 24;
      startCountdown(el, h);
    });
  }
  // Auto-load on DOMContentLoaded
  document.addEventListener('DOMContentLoaded', function() {
    loadRankings();
    startCountdowns();
  });

  return { toast, setTheme, currentTheme, toggleTheme, openModal, closeModal, toggleDropdown, openSheet, closeSheet, activateTab, toggleNavDropdown, toggleNavAccordion, toggleNav, openAuth, closeAuth, apiGet, apiPost, loadRankings, startCountdown, startCountdowns, companyCardHTML, toggleMobileMenu };
})();
</script>

<!-- Lucide icon library (epsx.io uses Lucide, not Font Awesome) -->
<script src="https://unpkg.com/lucide@latest"></script>
<script>
(function() {
  try {
    function initLucide() {
      if (window.lucide && typeof window.lucide.createIcons === 'function') {
        try { window.lucide.createIcons(); } catch (e) { console.warn('lucide createIcons error:', e); }
      }
    }
    // Attach to epsx global (defined earlier in this page)
    if (window.epsx) {
      window.epsx.initLucide = initLucide;
    } else {
      document.addEventListener('DOMContentLoaded', function() {
        if (window.epsx) window.epsx.initLucide = initLucide;
      });
    }
    // Initial render once DOM is ready
    function initialRender() {
      initLucide();
      // Re-render when DOM changes
      if (window.MutationObserver && document.body) {
        try {
          new MutationObserver(function() {
            if (window.lucide && window.lucide.createIcons) {
              clearTimeout(window.__lucideTimer);
              window.__lucideTimer = setTimeout(initLucide, 50);
            }
          }).observe(document.body, { childList: true, subtree: true });
        } catch (e) { console.warn('MutationObserver error:', e); }
      }
    }
    if (document.readyState === 'loading') {
      document.addEventListener('DOMContentLoaded', initialRender);
    } else {
      initialRender();
    }
  } catch (e) { console.error('epsx lucide init error:', e); }
})();
</script>"#
}

/// Returns a theme toggle button (sun/moon icons). Renders as a nav-style icon
/// button. Pair with `set_theme` JS in `global_js()`.
pub fn theme_toggle_button() -> &'static str {
    r##"<button id="epsx-theme-toggle" class="nav-link" onclick="epsx.toggleTheme()" aria-label="Toggle theme" style="width:2.25rem;height:2.25rem;padding:0;justify-content:center;">
  <i data-icon="sun" data-lucide="sun" style="display:none;width:1.125rem;height:1.125rem;"></i>
  <i data-icon="moon" data-lucide="moon" style="width:1.125rem;height:1.125rem;"></i>
</button>"##
}

/// Returns the standard EPSX logo (gradient text "EPSX").
/// Returns the EPSX hexagon-with-chart icon (matches epsx.io's `/logos/epsx-icon.svg`).
pub fn epsx_icon_svg() -> &'static str {
    r##"<svg viewBox="0 0 32 32" fill="none" xmlns="http://www.w3.org/2000/svg" class="epsx-icon" aria-hidden="true">
  <defs>
    <linearGradient id="epsx-logo-grad" x1="0" y1="0" x2="32" y2="32" gradientUnits="userSpaceOnUse">
      <stop offset="0" stop-color="#488BFA"/>
      <stop offset="1" stop-color="#A43FF3"/>
    </linearGradient>
  </defs>
  <path d="M16 1.5L29 8.5V23.5L16 30.5L3 23.5V8.5L16 1.5Z" stroke="url(#epsx-logo-grad)" stroke-width="2" fill="rgba(72,139,250,0.1)"/>
  <path d="M8 22L13 16L17 19L24 9" stroke="url(#epsx-logo-grad)" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round" fill="none"/>
  <circle cx="24" cy="9" r="1.5" fill="#488BFA"/>
</svg>"##
}

/// Lucide icon path data — `name` is the kebab-case lucide name (e.g. `chart-column`).
/// Returns the inner `<path>` content. Caller wraps in a `<svg>` with class.
/// We embed the 50+ icons we use; for anything else, return empty.
pub fn lucide_icon(name: &str) -> &'static str {
    match name {
        "chart-column" => r#"<path d="M3 3v16a2 2 0 0 0 2 2h16"/><path d="M18 17V9"/><path d="M13 17V5"/><path d="M8 17v-3"/>"#,
        "code" => r#"<path d="m16 18 6-6-6-6"/><path d="m8 6-6 6 6 6"/>"#,
        "building" => r#"<path d="M6 22V4a2 2 0 0 1 2-2h8a2 2 0 0 1 2 2v18"/><path d="M6 12H4a2 2 0 0 0-2 2v6a2 2 0 0 0 2 2h2"/><path d="M18 9h2a2 2 0 0 1 2 2v9a2 2 0 0 1-2 2h-2"/><path d="M10 6h4"/><path d="M10 10h4"/><path d="M10 14h4"/><path d="M10 18h4"/>"#,
        "chevron-down" => r#"<path d="m6 9 6 6 6-6"/>"#,
        "chevron-right" => r#"<path d="m9 18 6-6-6-6"/>"#,
        "trending-up" => r#"<path d="M22 7 13.5 15.5 8.5 10.5 2 17"/><path d="M16 7h6v6"/>"#,
        "chart-line" => r#"<path d="M3 3v16a2 2 0 0 0 2 2h16"/><path d="m19 9-5 5-4-4-3 3"/>"#,
        "zap" => r#"<path d="M4 14a1 1 0 0 1-.78-1.63l9.9-10.2a.5.5 0 0 1 .86.46l-1.92 6.02A1 1 0 0 0 13 10h7a1 1 0 0 1 .78 1.63l-9.9 10.2a.5.5 0 0 1-.86-.46l1.92-6.02A1 1 0 0 0 11 14z"/>"#,
        "users" => r#"<path d="M16 21v-2a4 4 0 0 0-4-4H6a4 4 0 0 0-4 4v2"/><circle cx="9" cy="7" r="4"/><path d="M22 21v-2a4 4 0 0 0-3-3.87"/><path d="M16 3.13a4 4 0 0 1 0 7.75"/>"#,
        "calendar" => r#"<path d="M8 2v4"/><path d="M16 2v4"/><rect width="18" height="18" x="3" y="4" rx="2"/><path d="M3 10h18"/>"#,
        "newspaper" => r#"<path d="M4 22h16a2 2 0 0 0 2-2V4a2 2 0 0 0-2-2H8a2 2 0 0 0-2 2v16a2 2 0 0 1-2 2Zm0 0a2 2 0 0 1-2-2v-9c0-1.1.9-2 2-2h2"/><path d="M18 14h-8"/><path d="M15 18h-5"/><path d="M10 6h8v4h-8V6Z"/>"#,
        "pin" => r#"<path d="M12 17v5"/><path d="M9 10.76a2 2 0 0 1-1.11 1.79l-1.78.9A2 2 0 0 0 5 15.24V16a1 1 0 0 0 1 1h12a1 1 0 0 0 1-1v-.76a2 2 0 0 0-1.11-1.79l-1.78-.9A2 2 0 0 1 15 10.76V7a1 1 0 0 1 1-1 2 2 0 0 0 0-4H8a2 2 0 0 0 0 4 1 1 0 0 1 1 1z"/>"#,
        "arrow-right" => r#"<path d="M5 12h14"/><path d="m12 5 7 7-7 7"/>"#,
        "info" => r#"<circle cx="12" cy="12" r="10"/><path d="M12 16v-4"/><path d="M12 8h.01"/>"#,
        "mail" => r#"<rect width="20" height="16" x="2" y="4" rx="2"/><path d="m22 7-8.97 5.7a1.94 1.94 0 0 1-2.06 0L2 7"/>"#,
        "help-circle" => r#"<circle cx="12" cy="12" r="10"/><path d="M9.09 9a3 3 0 0 1 5.83 1c0 2-3 3-3 3"/><path d="M12 17h.01"/>"#,
        "circle-help" => r#"<circle cx="12" cy="12" r="10"/><path d="M9.09 9a3 3 0 0 1 5.83 1c0 2-3 3-3 3"/><path d="M12 17h.01"/>"#,
        "menu" => r#"<line x1="4" x2="20" y1="12" y2="12"/><line x1="4" x2="20" y1="6" y2="6"/><line x1="4" x2="20" y1="18" y2="18"/>"#,
        "x" => r#"<path d="M18 6 6 18"/><path d="m6 6 12 12"/>"#,
        "sun" => r#"<circle cx="12" cy="12" r="4"/><path d="M12 2v2"/><path d="M12 20v2"/><path d="m4.93 4.93 1.41 1.41"/><path d="m17.66 17.66 1.41 1.41"/><path d="M2 12h2"/><path d="M20 12h2"/><path d="m6.34 17.66-1.41 1.41"/><path d="m19.07 4.93-1.41 1.41"/>"#,
        "moon" => r#"<path d="M12 3a6 6 0 0 0 9 9 9 9 0 1 1-9-9Z"/>"#,
        "wallet" => r#"<path d="M19 7V4a1 1 0 0 0-1-1H5a2 2 0 0 0 0 4h15a1 1 0 0 1 1 1v4h-3a2 2 0 0 0 0 4h3a1 1 0 0 0 1-1v-2a1 1 0 0 0-1-1"/><path d="M3 5v14a2 2 0 0 0 2 2h15a1 1 0 0 0 1-1v-4"/>"#,
        "log-out" => r#"<path d="M9 21H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h4"/><polyline points="16 17 21 12 16 7"/><line x1="21" x2="9" y1="12" y2="12"/>"#,
        "user" => r#"<path d="M19 21v-2a4 4 0 0 0-4-4H9a4 4 0 0 0-4 4v2"/><circle cx="12" cy="7" r="4"/>"#,
        "settings" => r#"<path d="M12.22 2h-.44a2 2 0 0 0-2 2v.18a2 2 0 0 1-1 1.73l-.43.25a2 2 0 0 1-2 0l-.15-.08a2 2 0 0 0-2.73.73l-.22.38a2 2 0 0 0 .73 2.73l.15.1a2 2 0 0 1 1 1.72v.51a2 2 0 0 1-1 1.74l-.15.09a2 2 0 0 0-.73 2.73l.22.38a2 2 0 0 0 2.73.73l.15-.08a2 2 0 0 1 2 0l.43.25a2 2 0 0 1 1 1.73V20a2 2 0 0 0 2 2h.44a2 2 0 0 0 2-2v-.18a2 2 0 0 1 1-1.73l.43-.25a2 2 0 0 1 2 0l.15.08a2 2 0 0 0 2.73-.73l.22-.39a2 2 0 0 0-.73-2.73l-.15-.08a2 2 0 0 1-1-1.74v-.5a2 2 0 0 1 1-1.74l.15-.09a2 2 0 0 0 .73-2.73l-.22-.38a2 2 0 0 0-2.73-.73l-.15.08a2 2 0 0 1-2 0l-.43-.25a2 2 0 0 1-1-1.73V4a2 2 0 0 0-2-2z"/><circle cx="12" cy="12" r="3"/>"#,
        "check" => r#"<path d="M20 6 9 17l-5-5"/>"#,
        "plus" => r#"<path d="M5 12h14"/><path d="M12 5v14"/>"#,
        "search" => r#"<circle cx="11" cy="11" r="8"/><path d="m21 21-4.3-4.3"/>"#,
        "share" => r#"<path d="M4 12v8a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2v-8"/><polyline points="16 6 12 2 8 6"/><line x1="12" x2="12" y1="2" y2="15"/>"#,
        "bell" => r#"<path d="M6 8a6 6 0 0 1 12 0c0 7 3 9 3 9H3s3-2 3-9"/><path d="M10.3 21a1.94 1.94 0 0 0 3.4 0"/>"#,
        "book" => r#"<path d="M4 19.5A2.5 2.5 0 0 1 6.5 17H20"/><path d="M6.5 2H20v20H6.5A2.5 2.5 0 0 1 4 19.5v-15A2.5 2.5 0 0 1 6.5 2z"/>"#,
        "key" => r#"<circle cx="7.5" cy="15.5" r="5.5"/><path d="m21 2-9.6 9.6"/><path d="m15.5 7.5 3 3L22 7l-3-3"/>"#,
        "layout-dashboard" => r#"<rect width="7" height="9" x="3" y="3" rx="1"/><rect width="7" height="5" x="14" y="3" rx="1"/><rect width="7" height="9" x="14" y="12" rx="1"/><rect width="7" height="5" x="3" y="16" rx="1"/>"#,
        "message-circle" => r#"<path d="M7.9 20A9 9 0 1 0 4 16.1L2 22Z"/>"#,
        "file-text" => r#"<path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"/><polyline points="14 2 14 8 20 8"/><line x1="16" x2="8" y1="13" y2="13"/><line x1="16" x2="8" y1="17" y2="17"/><line x1="10" x2="8" y1="9" y2="9"/>"#,
        "history" => r#"<path d="M3 12a9 9 0 1 0 9-9 9.75 9.75 0 0 0-6.74 2.74L3 8"/><path d="M3 3v5h5"/><path d="M12 7v5l4 2"/>"#,
        "credit-card" => r#"<rect width="20" height="14" x="2" y="5" rx="2"/><line x1="2" x2="22" y1="10" y2="10"/>"#,
        "link" => r#"<path d="M10 13a5 5 0 0 0 7.54.54l3-3a5 5 0 0 0-7.07-7.07l-1.72 1.71"/><path d="M14 11a5 5 0 0 0-7.54-.54l-3 3a5 5 0 0 0 7.07 7.07l1.71-1.71"/>"#,
        "external-link" => r#"<path d="M15 3h6v6"/><path d="M10 14 21 3"/><path d="M18 13v6a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V8a2 2 0 0 1 2-2h6"/>"#,
        "briefcase" => r#"<path d="M16 20V4a2 2 0 0 0-2-2h-4a2 2 0 0 0-2 2v16"/><rect width="20" height="14" x="2" y="6" rx="2"/>"#,
        // wave2-chrome-track-a: added icons required by admin sidebar/header parity.
        // All paths mirror the official lucide.dev SVG body.
        "home" => r#"<path d="m3 9 9-7 9 7v11a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2z"/><polyline points="9 22 9 12 15 12 15 22"/>"#,
        "lock" => r#"<rect width="18" height="11" x="3" y="11" rx="2" ry="2"/><path d="M7 11V7a5 5 0 0 1 10 0v4"/>"#,
        "shield" => r#"<path d="M12 22s8-4 8-10V5l-8-3-8 3v7c0 6 8 10 8 10z"/>"#,
        "globe" => r#"<circle cx="12" cy="12" r="10"/><line x1="2" x2="22" y1="12" y2="12"/><path d="M12 2a15.3 15.3 0 0 1 4 10 15.3 15.3 0 0 1-4 10 15.3 15.3 0 0 1-4-10 15.3 15.3 0 0 1 4-10z"/>"#,
        "palette" => r#"<circle cx="13.5" cy="6.5" r=".5"/><circle cx="17.5" cy="10.5" r=".5"/><circle cx="8.5" cy="7.5" r=".5"/><circle cx="6.5" cy="12.5" r=".5"/><path d="M12 2C6.5 2 2 6.5 2 12s4.5 10 10 10c.926 0 1.648-.746 1.648-1.688 0-.437-.18-.835-.437-1.125-.29-.289-.438-.652-.438-1.125a1.64 1.64 0 0 1 1.668-1.668h1.996c3.051 0 5.555-2.503 5.555-5.554C21.965 6.012 17.461 2 12 2z"/>"#,
        "send" => r#"<line x1="22" x2="11" y1="2" y2="13"/><polygon points="22 2 15 22 11 13 2 9 22 2"/>"#,
        "coins" => r#"<circle cx="8" cy="8" r="6"/><path d="M18.09 10.37A6 6 0 1 1 10.34 18"/><path d="M7 6h1v4"/><path d="m16.71 13.88.7.71-2.82 2.82"/>"#,
        "link-2" => r#"<path d="M9 17H7A5 5 0 0 1 7 7h2"/><path d="M15 7h2a5 5 0 1 1 0 10h-2"/><line x1="8" x2="16" y1="12" y2="12"/>"#,
        "image" => r#"<rect width="18" height="18" x="3" y="3" rx="2" ry="2"/><circle cx="9" cy="9" r="2"/><path d="m21 15-3.086-3.086a2 2 0 0 0-2.828 0L6 21"/>"#,
        "bar-chart-3" => r#"<path d="M3 3v18h18"/><path d="M18 17V9"/><path d="M13 17V5"/><path d="M8 17v-3"/>"#,
        "book-open" => r#"<path d="M2 3h6a4 4 0 0 1 4 4v14a3 3 0 0 0-3-3H2z"/><path d="M22 3h-6a4 4 0 0 0-4 4v14a3 3 0 0 1 3-3h7z"/>"#,
        // === wave5-page-depth-track-a === new icons required by the
        // expanded home / auth / about hero pages. All paths mirror
        // the official lucide.dev SVG body. No existing icons are
        // restyled.
        "share-2" => r#"<circle cx="18" cy="5" r="3"/><circle cx="6" cy="12" r="3"/><circle cx="18" cy="19" r="3"/><line x1="8.59" x2="15.42" y1="13.51" y2="17.49"/><line x1="15.41" x2="8.59" y1="6.51" y2="10.49"/>"#,
        "clock" => r#"<circle cx="12" cy="12" r="10"/><polyline points="12 6 12 12 16 14"/>"#,
        "star" => r#"<polygon points="12 2 15.09 8.26 22 9.27 17 14.14 18.18 21.02 12 17.77 5.82 21.02 7 14.14 2 9.27 8.91 8.26 12 2"/>"#,
        "circle-check" => r#"<circle cx="12" cy="12" r="10"/><path d="m9 12 2 2 4-4"/>"#,
        "rocket" => r#"<path d="M4.5 16.5c-1.5 1.26-2 5-2 5s3.74-.5 5-2c.71-.84.7-2.13-.09-2.91a2.18 2.18 0 0 0-2.91-.09z"/><path d="m12 15-3-3a22 22 0 0 1 2-3.95A12.88 12.88 0 0 1 22 2c0 2.72-.78 7.5-6 11a22.35 22.35 0 0 1-4 2z"/><path d="M9 12H4s.55-3.03 2-4c1.62-1.08 5 0 5 0"/><path d="M12 15v5s3.03-.55 4-2c1.08-1.62 0-5 0-5"/>"#,
        "target" => r#"<circle cx="12" cy="12" r="10"/><circle cx="12" cy="12" r="6"/><circle cx="12" cy="12" r="2"/>"#,
        "lightbulb" => r#"<path d="M15 14c.2-1 .7-1.7 1.5-2.5 1-.9 1.5-2.2 1.5-3.5A6 6 0 0 0 6 8c0 1 .2 2.2 1.5 3.5.7.7 1.3 1.5 1.5 2.5"/><path d="M9 18h6"/><path d="M10 22h4"/>"#,
        "database" => r#"<ellipse cx="12" cy="5" rx="9" ry="3"/><path d="M3 5V19A9 3 0 0 0 21 19V5"/><path d="M3 12A9 3 0 0 0 21 12"/>"#,
        "message-square" => r#"<path d="M21 15a2 2 0 0 1-2 2H7l-4 4V5a2 2 0 0 1 2-2h14a2 2 0 0 1 2 2z"/>"#,
        "sparkles" => r#"<path d="m12 3-1.9 5.8a2 2 0 0 1-1.3 1.3L3 12l5.8 1.9a2 2 0 0 1 1.3 1.3L12 21l1.9-5.8a2 2 0 0 1 1.3-1.3L21 12l-5.8-1.9a2 2 0 0 1-1.3-1.3z"/><path d="M5 3v4"/><path d="M19 17v4"/><path d="M3 5h4"/><path d="M17 19h4"/>"#,
        "play" => r#"<polygon points="6 3 20 12 6 21 6 3"/>"#,
        "arrow-up-right" => r#"<path d="M7 7h10v10"/><path d="M7 17 17 7"/>"#,
        "circle-x" => r#"<circle cx="12" cy="12" r="10"/><path d="m15 9-6 6"/><path d="m9 9 6 6"/>"#,
        "triangle-alert" => r#"<path d="m21.73 18-8-14a2 2 0 0 0-3.48 0l-8 14A2 2 0 0 0 4 21h16a2 2 0 0 0 1.73-3Z"/><path d="M12 9v4"/><path d="M12 17h.01"/>"#,
        "wifi-off" => r#"<line x1="2" x2="22" y1="2" y2="22"/><path d="M8.5 16.5a5 5 0 0 1 7 0"/><path d="M2 8.82a15 15 0 0 1 4.17-2.65"/><path d="M10.66 5c4.01-.36 8.14.9 11.34 3.76"/><path d="M16.85 11.25a10 10 0 0 1 2.22 1.68"/><path d="M5 13a10 10 0 0 1 5.24-2.76"/><line x1="12" x2="12.01" y1="20" y2="20"/>"#,
        "mail" => r#"<rect width="20" height="16" x="2" y="4" rx="2"/><path d="m22 7-8.97 5.7a1.94 1.94 0 0 1-2.06 0L2 7"/>"#,
        "tag" => r#"<path d="M12.586 2.586A2 2 0 0 0 11.172 2H4a2 2 0 0 0-2 2v7.172a2 2 0 0 0 .586 1.414l8.704 8.704a2.426 2.426 0 0 0 3.42 0l6.58-6.58a2.426 2.426 0 0 0 0-3.42z"/><circle cx="7.5" cy="7.5" r=".5"/>"#,
        _ => "",
    }
}

/// Returns a complete `<svg>` element for a Lucide icon.
/// `size` defaults to 16; pass a number string (e.g. "20") to override.
pub fn lucide(name: &str, size: &str, class: &str) -> String {
    format!(
        r#"<svg xmlns="http://www.w3.org/2000/svg" width="{sz}" height="{sz}" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="lucide lucide-{name} {class}" aria-hidden="true">{body}</svg>"#,
        sz = size,
        name = name,
        class = class,
        body = lucide_icon(name),
    )
}

pub fn logo(href: &str, size: &str) -> String {
    let cls = if size == "sm" { "logo-text-sm" } else { "logo-text" };
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
          Web3 analytics, on-chain subscriptions, and a visual builder for modern DeFi platforms.
        </p>
      </div>
      <div>
        <h4 style="font-size:0.875rem;font-weight:600;color:var(--text);margin-bottom:0.75rem;">Platform</h4>
        <div style="display:flex;flex-direction:column;gap:0.5rem;">
          <a href="/analytics" class="footer-link">Rankings</a>
          <a href="/portfolio" class="footer-link">Portfolio</a>
          <a href="/pricing" class="footer-link">Pricing</a>
          <a href="/news" class="footer-link">News</a>
        </div>
      </div>
      <div>
        <h4 style="font-size:0.875rem;font-weight:600;color:var(--text);margin-bottom:0.75rem;">Developers</h4>
        <div style="display:flex;flex-direction:column;gap:0.5rem;">
          <a href="/developer" class="footer-link">API Keys</a>
          <a href="/developer/docs" class="footer-link">Documentation</a>
          <a href="/chat" class="footer-link">Support</a>
        </div>
      </div>
      <div>
        <h4 style="font-size:0.875rem;font-weight:600;color:var(--text);margin-bottom:0.75rem;">Company</h4>
        <div style="display:flex;flex-direction:column;gap:0.5rem;">
          <a href="/about" class="footer-link">About</a>
          <a href="/contact" class="footer-link">Contact</a>
          <a href="/terms" class="footer-link">Terms of Service</a>
          <a href="/privacy" class="footer-link">Privacy Policy</a>
        </div>
      </div>
    </div>
    <div style="border-top:1px solid var(--border);padding-top:1.5rem;display:flex;flex-wrap:wrap;gap:1rem;justify-content:space-between;align-items:center;font-size:0.8125rem;">
      <span>&copy; 2025 EPSX. All rights reserved.</span>
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
    // Market dropdown items (rankings, portfolio)
    let market_items = r##"
      <a href="/rankings" class="epsx-nav-item">
        <i data-lucide="chart-column" class="item-icon"></i>
        <div>
          <div class="item-label">Rankings</div>
          <div class="item-desc">EPS stock rankings</div>
        </div>
      </a>
      <a href="/portfolio" class="epsx-nav-item">
        <i data-lucide="trending-up" class="item-icon"></i>
        <div>
          <div class="item-label">Portfolio</div>
          <div class="item-desc">Watchlist &amp; tracking</div>
        </div>
      </a>"##;

    // Developer dropdown items
    let developer_items = r##"
      <a href="/developer" class="epsx-nav-item">
        <i data-lucide="key" class="item-icon"></i>
        <div>
          <div class="item-label">API Keys</div>
          <div class="item-desc">Manage your API access</div>
        </div>
      </a>
      <a href="/developer/docs" class="epsx-nav-item">
        <i data-lucide="book" class="item-icon"></i>
        <div>
          <div class="item-label">Documentation</div>
          <div class="item-desc">Integration guides &amp; reference</div>
        </div>
      </a>
      <a href="/developer/usage" class="epsx-nav-item">
        <i data-lucide="layout-dashboard" class="item-icon"></i>
        <div>
          <div class="item-label">Usage</div>
          <div class="item-desc">API usage &amp; analytics</div>
        </div>
      </a>"##;

    // Company dropdown items
    let company_items = r##"
      <a href="/about" class="epsx-nav-item">
        <i data-lucide="info" class="item-icon"></i>
        <div>
          <div class="item-label">About</div>
          <div class="item-desc">Our mission &amp; team</div>
        </div>
      </a>
      <a href="/news" class="epsx-nav-item">
        <i data-lucide="newspaper" class="item-icon"></i>
        <div>
          <div class="item-label">News</div>
          <div class="item-desc">Latest updates</div>
        </div>
      </a>
      <a href="/contact" class="epsx-nav-item">
        <i data-lucide="mail" class="item-icon"></i>
        <div>
          <div class="item-label">Contact</div>
          <div class="item-desc">Get in touch</div>
        </div>
      </a>
      <a href="/chat" class="epsx-nav-item">
        <i data-lucide="help-circle" class="item-icon"></i>
        <div>
          <div class="item-label">Support</div>
          <div class="item-desc">Live chat &amp; help center</div>
        </div>
      </a>"##;

    let logo = epsx_icon_svg();
    let nav_block = |label: &str, icon: &str, items: &str| -> String {
        format!(
            r##"<div class="epsx-nav-wrap" data-nav="{label}">
  <button class="epsx-nav-trigger" type="button" onclick="epsx.toggleNav(this)">
    <i data-lucide="{icon}" class="nav-icon"></i>
    {label}
    <i data-lucide="chevron-down" class="nav-chev"></i>
  </button>
  <div class="epsx-nav-menu" role="menu">{items}</div>
</div>"##,
            label = label,
            icon = icon,
            items = items,
        )
    };

    format!(
        r##"<header class="epsx-header">
  <div class="mx-auto flex h-14 max-w-7xl items-center justify-between px-4 md:px-6">
    <a href="/" class="flex items-center gap-2.5 group" style="text-decoration:none;">
      {logo}
      <span class="text-xl font-black tracking-widest text-transparent bg-clip-text bg-gradient-to-r from-[#488BFA] to-[#A43FF3] leading-none mt-0.5">EPSX</span>
    </a>

    <nav class="hidden lg:flex items-center gap-1">
      {market}
      {developer}
      {company}
    </nav>

    <div class="flex items-center gap-2">
      <button class="epsx-theme-btn" type="button" aria-label="Toggle theme" onclick="epsx.toggleTheme()">
        <i data-lucide="sun" class="sun" style="display:none;"></i>
        <i data-lucide="moon" class="moon"></i>
      </button>
      <div class="hidden md:flex items-center gap-1.5">
        <button class="epsx-connect-btn" type="button" onclick="epsx.openAuth()">
          <i data-lucide="wallet" style="width:1rem;height:1rem;"></i>
          Connect
        </button>
      </div>
      <div class="hidden sm:flex md:hidden items-center gap-1.5">
        <button class="epsx-connect-btn" type="button" onclick="epsx.openAuth()" style="height:2rem;padding:0 0.75rem;font-size:0.75rem;border-radius:1rem;">
          <i data-lucide="wallet" style="width:0.75rem;height:0.75rem;"></i>
          Connect
        </button>
      </div>
      <!-- Mobile menu toggle (< 640px) -->
      <button class="epsx-theme-btn md:hidden" type="button" aria-label="Open menu" onclick="epsx.toggleMobileMenu()" id="epsx-mobile-menu-btn" style="width:2.25rem;height:2.25rem;padding:0;">
        <i data-lucide="menu" style="width:1.125rem;height:1.125rem;"></i>
      </button>
    </div>
  </div>
</header>"##,
        logo = logo,
        market = nav_block("Market", "chart-column", market_items),
        developer = nav_block("Developer", "code", developer_items),
        company = nav_block("Company", "building", company_items),
    )
}

/// A standard page shell. Returns the complete `<!DOCTYPE html>...<body>...</body></html>`
/// wrapper used by every BFF page. BFFs just supply the `<nav>` content and
/// the body content.
pub fn page_shell(title: &str, description: &str, nav: &str, body: &str, include_footer: bool) -> String {
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
    let footer_html = if include_footer { footer() } else { "" };
    format!(
        r##"<!DOCTYPE html>
<html lang="en" class="dark">
<head>
{head}
{js}
</head>
<body class="min-h-screen {body_class}">
{nav}
<main style="min-height:calc(100vh - 3.5rem);">
{body}
</main>
{footer}
</body>
</html>"##,
        head = design_system_head(title, description),
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
