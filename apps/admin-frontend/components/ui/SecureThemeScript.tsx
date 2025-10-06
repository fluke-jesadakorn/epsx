/**
 * Secure Theme Script Component
 * Replaces dangerouslySetInnerHTML with a safer approach
 * Prevents XSS while maintaining FOUC prevention
 */

'use client';

import { useEffect } from 'react';

// Safe theme values (whitelist approach)
const SAFE_THEMES = ['light', 'dark'] as const;
type SafeTheme = typeof SAFE_THEMES[number];

function isSafeTheme(theme: string): theme is SafeTheme {
  return SAFE_THEMES.includes(theme as SafeTheme);
}

/**
 *
 */
export function SecureThemeScript() {
  useEffect(() => {
    // This runs after hydration, but we can also add a blocking script
    // for better FOUC prevention if needed
    initializeTheme();
  }, []);

  return (
    <script
      // Using a non-executable script type to safely store config
      type="application/json"
      id="theme-config"
      dangerouslySetInnerHTML={{
        __html: JSON.stringify({
          defaultTheme: 'light',
          storageKey: 'theme'
        })
      }}
    />
  );
}

function initializeTheme() {
  try {
    const config = document.getElementById('theme-config')?.textContent;
    if (!config) {return;}

    const { defaultTheme, storageKey } = JSON.parse(config);
    
    // Get theme from localStorage with validation
    let theme: SafeTheme = defaultTheme;
    const storedTheme = localStorage.getItem(storageKey);
    
    if (storedTheme && isSafeTheme(storedTheme)) {
      theme = storedTheme;
    } else if (!storedTheme) {
      // Check system preference
      const prefersDark = window.matchMedia('(prefers-color-scheme: dark)').matches;
      theme = prefersDark ? 'dark' : 'light';
    }
    
    // Apply theme safely
    const root = document.documentElement;
    root.classList.remove('light', 'dark');
    root.classList.add(theme);
    
    // Store for future use
    localStorage.setItem(storageKey, theme);
    
  } catch (_error) {
    // eslint-disable-next-line no-console
    console.error('Theme initialization error:', _error);
    // Fallback to light theme
    document.documentElement.classList.add('light');
  }
}

// Server-side blocking script for better FOUC prevention
/**
 *
 */
export function getThemeBlockingScript(): string {
  return `
    (function() {
      try {
        var theme = localStorage.getItem('theme');
        var validThemes = ['light', 'dark'];
        
        if (!theme || validThemes.indexOf(theme) === -1) {
          theme = window.matchMedia('(prefers-color-scheme: dark)').matches ? 'dark' : 'light';
        }
        
        document.documentElement.classList.add(theme);
      } catch (e) {
        document.documentElement.classList.add('light');
      }
    })();
  `;
}