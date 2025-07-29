import { getThemeSettings } from '@/app/actions/theme';

/**
 * Server Component that injects theme initialization script
 * This prevents theme flickering on page load
 */
export async function ThemeScript() {
  const { variant, mode } = await getThemeSettings();

  const script = `
    (function() {
      try {
        var variant = '${variant}';
        var mode = '${mode}';
        var isDarkMode = false;

        // Determine dark mode
        if (mode === 'dark') {
          isDarkMode = true;
        } else if (mode === 'system') {
          isDarkMode = window.matchMedia('(prefers-color-scheme: dark)').matches;
        }

        // Apply theme classes immediately
        var root = document.documentElement;
        root.classList.remove('light', 'dark');
        root.classList.remove('theme-default', 'theme-pancake', 'theme-trading');
        
        root.classList.add(isDarkMode ? 'dark' : 'light');
        root.classList.add('theme-' + variant);
        
        // Set data attributes
        root.setAttribute('data-theme', variant);
        root.setAttribute('data-mode', isDarkMode ? 'dark' : 'light');
        root.setAttribute('data-theme-mode', mode);
        
        // Store computed values for client hydration
        window.__EPSX_THEME__ = {
          variant: variant,
          mode: mode,
          isDarkMode: isDarkMode
        };
      } catch (e) {
        console.error('Theme initialization error:', e);
      }
    })();
  `;

  return (
    <script
      dangerouslySetInnerHTML={{ __html: script }}
      suppressHydrationWarning
    />
  );
}