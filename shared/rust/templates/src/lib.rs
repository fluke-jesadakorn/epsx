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
  }}
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
