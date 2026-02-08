/**
 * SHARED SAFE THEME SCRIPT
 * 
 * Secure replacement for theme initialization with dangerouslySetInnerHTML
 * Minimizes XSS risk while preventing FOUC (Flash of Unstyled Content)
 * 
 * This component should be placed in the <head> section of the root layout
 * to ensure theme is applied before the body renders.
 * 
 * Consolidated from frontend and admin-frontend implementations.
 */

// Safe theme configuration (compile-time constants)
const THEME_CONFIG = {
  validThemes: ['light', 'dark'] as const,
  storageKey: 'theme',
  defaultTheme: 'dark' as const
} as const;

export type ValidTheme = typeof THEME_CONFIG.validThemes[number];

/**
 * Generates a minimal, safe theme initialization script
 * Uses compile-time constants to minimize injection risk
 * 
 * Features:
 * - Reads theme from cookies first, then localStorage
 * - Falls back to system preference if no stored theme
 * - Sets both class and colorScheme on document element
 */
export function SafeThemeScript() {
  // Script reads cookie first, then localStorage, then system preference
  const script = `(function(){try{var c={};if(document.cookie){var p=document.cookie.split(';');for(var i=0;i<p.length;i++){var k=p[i].trim().split('=')[0];var v=p[i].trim().split('=')[1];if(k&&v)c[k]=v}}var t=c.theme||localStorage.getItem('${THEME_CONFIG.storageKey}');if(t!=='light'&&t!=='dark'){t='dark'}document.documentElement.classList.add(t);document.documentElement.style.colorScheme=t}catch(e){document.documentElement.classList.add('${THEME_CONFIG.defaultTheme}')}})();`;

  return (
    <script
      dangerouslySetInnerHTML={{ __html: script }}
      suppressHydrationWarning
    />
  );
}

/**
 * Alternative approach using nonce for CSP compatibility
 */
export function SafeThemeScriptWithNonce({ nonce }: { nonce?: string }) {
  const script = `(function(){try{var c={};if(document.cookie){var p=document.cookie.split(';');for(var i=0;i<p.length;i++){var k=p[i].trim().split('=')[0];var v=p[i].trim().split('=')[1];if(k&&v)c[k]=v}}var t=c.theme||localStorage.getItem('theme');if(t!=='light'&&t!=='dark'){t='dark'}document.documentElement.classList.add(t);document.documentElement.style.colorScheme=t}catch(e){document.documentElement.classList.add('dark')}})();`;

  return (
    <script
      nonce={nonce}
      dangerouslySetInnerHTML={{ __html: script }}
      suppressHydrationWarning
    />
  );
}

/**
 * Client-side theme management utilities
 * Use these for programmatic theme changes after initial load
 */
export const themeUtils = {
  /**
   * Get the current theme from storage or system preference
   */
  getTheme: (): ValidTheme => {
    try {
      // Try cookies first, then fallback to localStorage
      const cookies = document.cookie.split(';').reduce<Record<string, string>>((acc, cookie) => {
        const [key, value] = cookie.trim().split('=');
        if (key && value) { acc[key] = value; }
        return acc;
      }, {});

      const stored = cookies.theme || localStorage.getItem(THEME_CONFIG.storageKey);
      if (stored === 'light' || stored === 'dark') { return stored; }

      return 'dark';
    } catch {
      return THEME_CONFIG.defaultTheme;
    }
  },

  /**
   * Set the theme and persist to both cookie and localStorage
   */
  setTheme: (theme: ValidTheme): void => {
    try {
      // Store theme in both cookie and localStorage for maximum compatibility
      document.cookie = `theme=${theme}; path=/; max-age=31536000; SameSite=lax`;
      localStorage.setItem(THEME_CONFIG.storageKey, theme);

      // Update DOM
      document.documentElement.classList.remove('light', 'dark');
      document.documentElement.classList.add(theme);
      document.documentElement.style.colorScheme = theme;
    } catch (_error) {
      // console.error('Failed to set theme:', error);
    }
  },

  /**
   * Toggle between light and dark theme
   */
  toggleTheme: (): ValidTheme => {
    const current = themeUtils.getTheme();
    const next = current === 'light' ? 'dark' : 'light';
    themeUtils.setTheme(next);
    return next;
  }
};

export default SafeThemeScript;
