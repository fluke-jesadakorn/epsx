/**
 * Safe Theme Script Component
 * Secure replacement for theme initialization with dangerouslySetInnerHTML
 * Minimizes XSS risk while preventing FOUC
 */

// Safe theme configuration (compile-time constants)
const THEME_CONFIG = {
  validThemes: ['light', 'dark'] as const,
  storageKey: 'theme',
  defaultTheme: 'light' as const
} as const;

type ValidTheme = typeof THEME_CONFIG.validThemes[number];

/**
 * Generates a minimal, safe theme initialization script
 * Uses compile-time constants to minimize injection risk
 */
export function SafeThemeScript() {
  // Generate script with compile-time constants (no user input)
  const script = `(function(){try{var t=localStorage.getItem('${THEME_CONFIG.storageKey}');if(t!=='light'&&t!=='dark'){t=window.matchMedia('(prefers-color-scheme: dark)').matches?'dark':'light'}document.documentElement.classList.add(t)}catch(e){document.documentElement.classList.add('${THEME_CONFIG.defaultTheme}')}})();`;

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
  const script = `(function(){try{var t=localStorage.getItem('theme');if(t!=='light'&&t!=='dark'){t=window.matchMedia('(prefers-color-scheme: dark)').matches?'dark':'light'}document.documentElement.classList.add(t)}catch(e){document.documentElement.classList.add('light')}})();`;

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
 */
export const themeUtils = {
  getTheme: (): ValidTheme => {
    try {
      const stored = localStorage.getItem(THEME_CONFIG.storageKey);
      if (stored === 'light' || stored === 'dark') return stored;
      
      return window.matchMedia('(prefers-color-scheme: dark)').matches 
        ? 'dark' 
        : 'light';
    } catch {
      return THEME_CONFIG.defaultTheme;
    }
  },
  
  setTheme: (theme: ValidTheme): void => {
    try {
      localStorage.setItem(THEME_CONFIG.storageKey, theme);
      document.documentElement.classList.remove('light', 'dark');
      document.documentElement.classList.add(theme);
    } catch (error) {
      console.error('Failed to set theme:', error);
    }
  },
  
  toggleTheme: (): ValidTheme => {
    const current = themeUtils.getTheme();
    const next = current === 'light' ? 'dark' : 'light';
    themeUtils.setTheme(next);
    return next;
  }
};