'use client';

import { useState, useEffect, useTransition } from 'react';
import { setThemeMode as _setThemeMode, setThemeVariant, toggleThemeMode } from '@/app/actions/theme';
import type { ThemeVariant, ThemeMode } from '@/app/actions/theme';

interface ThemeState {
  variant: ThemeVariant;
  mode: ThemeMode;
  isDarkMode: boolean;
}

export function ThemeToggleSSR() {
  const [isPending, startTransition] = useTransition();
  const [themeState, setThemeState] = useState<ThemeState>({
    variant: 'default',
    mode: 'system',
    isDarkMode: false,
  });

  // Initialize theme from server-injected data
  useEffect(() => {
    const serverTheme = (window as any).__EPSX_THEME__;
    if (serverTheme) {
      setThemeState(serverTheme);
    }
  }, []);

  const handleToggleDarkMode = () => {
    startTransition(async () => {
      const newMode = await toggleThemeMode();
      const newIsDarkMode = newMode === 'dark' || 
        (newMode === 'system' && window.matchMedia('(prefers-color-scheme: dark)').matches);
      
      // Update local state
      setThemeState(prev => ({
        ...prev,
        mode: newMode,
        isDarkMode: newIsDarkMode,
      }));
      
      // Apply theme immediately for smooth transition
      const root = document.documentElement;
      root.classList.remove('light', 'dark');
      root.classList.add(newIsDarkMode ? 'dark' : 'light');
      root.setAttribute('data-mode', newIsDarkMode ? 'dark' : 'light');
      root.setAttribute('data-theme-mode', newMode);
    });
  };

  const handleVariantChange = (variant: ThemeVariant) => {
    startTransition(async () => {
      await setThemeVariant(variant);
      
      // Update local state
      setThemeState(prev => ({
        ...prev,
        variant,
      }));
      
      // Apply theme immediately
      const root = document.documentElement;
      root.classList.remove('theme-default', 'theme-pancake', 'theme-trading');
      root.classList.add(`theme-${variant}`);
      root.setAttribute('data-theme', variant);
    });
  };

  return (
    <div className="flex items-center gap-4">
      {/* Dark Mode Toggle */}
      <button
        onClick={handleToggleDarkMode}
        disabled={isPending}
        className="inline-flex items-center gap-2 p-2 rounded-md transition-colors hover:bg-accent hover:text-accent-foreground disabled:opacity-50"
        aria-label="Toggle dark mode"
      >
        {themeState.isDarkMode ? (
          <svg className="h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M12 3v1m0 16v1m9-9h-1M4 12H3m15.364 6.364l-.707-.707M6.343 6.343l-.707-.707m12.728 0l-.707.707M6.343 17.657l-.707.707M16 12a4 4 0 11-8 0 4 4 0 018 0z" />
          </svg>
        ) : (
          <svg className="h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M20.354 15.354A9 9 0 018.646 3.646 9.003 9.003 0 0012 21a9.003 9.003 0 008.354-5.646z" />
          </svg>
        )}
        <span className="text-sm">{themeState.isDarkMode ? 'Light' : 'Dark'}</span>
      </button>

      {/* Theme Variant Selector */}
      <div className="flex gap-1">
        {(['default', 'pancake', 'trading'] as ThemeVariant[]).map((variant) => (
          <button
            key={variant}
            onClick={() => handleVariantChange(variant)}
            disabled={isPending || themeState.variant === variant}
            className={`px-3 py-1 rounded-md text-sm font-medium transition-colors ${
              themeState.variant === variant
                ? 'bg-primary text-primary-foreground'
                : 'bg-secondary text-secondary-foreground hover:bg-secondary/80'
            } disabled:opacity-50`}
          >
            {variant.charAt(0).toUpperCase() + variant.slice(1)}
          </button>
        ))}
      </div>
    </div>
  );
}