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
  // Updated to use cookies with localStorage fallback
  const script = `(function(){try{var c={};if(document.cookie){var p=document.cookie.split(';');for(var i=0;i<p.length;i++){var k=p[i].trim().split('=')[0];var v=p[i].trim().split('=')[1];if(k&&v)c[k]=v}var t=c.theme||localStorage.getItem('${THEME_CONFIG.storageKey}');if(t!=='light'&&t!=='dark'){t=window.matchMedia('(prefers-color-scheme: dark)').matches?'dark':'light'}document.documentElement.classList.add(t)}catch(e){document.documentElement.classList.add('${THEME_CONFIG.defaultTheme}')}})();`;

  return (
    <script
      dangerouslySetInnerHTML={{ __html: script }}
      suppressHydrationWarning
    />
  );
}

/**
 * Alternative approach using nonce for CSP compatibility
 * @param root0
 * @param root0.nonce
 */
export function SafeThemeScriptWithNonce({ nonce }: { nonce?: string }) {
  const script = `(function(){try{var c={};if(document.cookie){var p=document.cookie.split(';');for(var i=0;i<p.length;i++){var k=p[i].trim().split('=')[0];var v=p[i].trim().split('=')[1];if(k&&v)c[k]=v}var t=c.theme||localStorage.getItem('theme');if(t!=='light'&&t!=='dark'){t=window.matchMedia('(prefers-color-scheme: dark)').matches?'dark':'light'}document.documentElement.classList.add(t)}catch(e){document.documentElement.classList.add('light')}})();`;

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
      // Try cookies first, then fallback to localStorage
      const cookies = document.cookie.split(';').reduce((acc, cookie) => {
        const [key, value] = cookie.trim().split('=');
        if (key && value) acc[key] = value;
        return acc;
      }, {} as Record<string, string>);
      
      const stored = cookies.theme || localStorage.getItem(THEME_CONFIG.storageKey);
      if (stored === 'light' || stored === 'dark') {return stored;}
      
      return window.matchMedia('(prefers-color-scheme: dark)').matches 
        ? 'dark' 
        : 'light';
    } catch {
      return THEME_CONFIG.defaultTheme;
    }
  },
  
  setTheme: (theme: ValidTheme): void => {
    try {
      // Store theme in cookie instead of localStorage
      document.cookie = `theme=${theme}; path=/; max-age=31536000; SameSite=lax`;
      document.documentElement.classList.remove('light', 'dark');
      document.documentElement.classList.add(theme);
    } catch (_error) {
      // eslint-disable-next-line no-console
      console.error('Failed to set theme:', _error);
    }
  },
  
  toggleTheme: (): ValidTheme => {
    const current = themeUtils.getTheme();
    const next = current === 'light' ? 'dark' : 'light';
    themeUtils.setTheme(next);
    return next;
  }
};