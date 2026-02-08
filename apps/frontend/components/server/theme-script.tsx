import { getThemeSettings } from '@/app/actions/theme';

// Safe theme values to prevent injection
const SAFE_VARIANTS = ['default', 'pancake', 'trading'] as const;
const SAFE_MODES = ['light', 'dark', 'system'] as const;

function sanitizeThemeValue(value: string, allowed: readonly string[]): string {
  return allowed.includes(value) ? value : allowed[0];
}

/**
 * Secure Server Component that injects theme initialization script
 * Prevents XSS by sanitizing theme values and using minimal script
 */
export async function ThemeScript() {
  const { variant, mode } = await getThemeSettings();
  
  // Sanitize theme values to prevent injection
  const safeVariant = sanitizeThemeValue(variant, SAFE_VARIANTS);
  const safeMode = sanitizeThemeValue(mode, SAFE_MODES);

  // Generate minimal, safe script with sanitized values
  const script = `(function(){try{var v='${safeVariant}',m='${safeMode}',d=m==='dark'||(m==='system'&&window.matchMedia('(prefers-color-scheme: dark)').matches),r=document.documentElement;r.classList.remove('light','dark','theme-default','theme-pancake','theme-trading');r.classList.add(d?'dark':'light','theme-'+v);r.setAttribute('data-theme',v);r.setAttribute('data-mode',d?'dark':'light');r.setAttribute('data-theme-mode',m);window.__EPSX_THEME__={variant:v,mode:m,isDarkMode:d}}catch(e){console.error('Theme error:',e)}})();`;

  return (
    <script
      dangerouslySetInnerHTML={{ __html: script }}
      suppressHydrationWarning
    />
  );
}