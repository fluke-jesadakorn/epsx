"use client";

import { useCallback, useEffect, useState, useTransition } from 'react';

import { useTheme } from '../providers/theme-provider.js';
import { designTokens } from '../tokens/design-tokens.js';
import { themeConfig } from '../tokens/theme-config.js';

import type { ThemeVariant } from '../tokens/theme-config.js';

export type ThemeMode = 'light' | 'dark' | 'system';
export type UnifiedThemeVariant = ThemeVariant;

export interface UnifiedThemeState {
  variant: UnifiedThemeVariant;
  mode: ThemeMode;
  isDarkMode: boolean;
  isSystemMode: boolean;
  isLoading: boolean;
}

export interface UseUnifiedThemeOptions {
  /** Enable server-side persistence via cookies */
  enableServerPersistence?: boolean;
  /** Enable optimistic updates */
  enableOptimistic?: boolean;
  /** Storage key prefix */
  storageKeyPrefix?: string;
  /** Auto-sync with system preference */
  autoSyncSystem?: boolean;
}

/**
 * Unified theme hook that consolidates all theme management patterns:
 * - Client-side state management
 * - Server-side persistence (cookies)
 * - SSR compatibility
 * - Optimistic updates
 * - System preference detection
 */
export function useUnifiedTheme(options: UseUnifiedThemeOptions = {}) {
  const {
    enableServerPersistence = true,
    enableOptimistic = true,
    storageKeyPrefix = 'epsx',
    autoSyncSystem = true,
  } = options;

  const themeContext = useTheme();
  const [isPending, startTransition] = useTransition();
  const [mode, setModeState] = useState<ThemeMode>('system');
  const [isSystemMode, setIsSystemMode] = useState(true);

  // Initialize mode from localStorage
  useEffect(() => {
    const storedMode = localStorage.getItem(`${storageKeyPrefix}-theme-mode`) as ThemeMode;
    if (storedMode) {
      setModeState(storedMode);
      setIsSystemMode(storedMode === 'system');
    }
  }, [storageKeyPrefix]);

  // Sync with system preference changes
  useEffect(() => {
    if (!autoSyncSystem || mode !== 'system') return;

    const mediaQuery = window.matchMedia('(prefers-color-scheme: dark)');
    const handleChange = (): void => {
      if (mode === 'system') {
        // Only update the actual theme, not the mode
        const systemDarkMode = mediaQuery.matches;
        // This will trigger the theme context to update
        if (systemDarkMode !== themeContext.isDarkMode) {
          themeContext.toggleDarkMode();
        }
      }
    };

    mediaQuery.addEventListener('change', handleChange);
    return () => mediaQuery.removeEventListener('change', handleChange);
  }, [mode, themeContext, autoSyncSystem]);

  // Server action functions (when enabled)
  const setServerThemeVariant = useCallback(async (variant: UnifiedThemeVariant) => {
    if (!enableServerPersistence) return;
    
    try {
      // This would call the server action
      // For now, we'll use a fetch to a server action endpoint
      await fetch('/api/theme/variant', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ variant }),
      });
    } catch (error) {
      console.warn('Failed to persist theme variant to server:', error);
    }
  }, [enableServerPersistence]);

  const setServerThemeMode = useCallback(async (newMode: ThemeMode) => {
    if (!enableServerPersistence) return;
    
    try {
      await fetch('/api/theme/mode', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ mode: newMode }),
      });
    } catch (error) {
      console.warn('Failed to persist theme mode to server:', error);
    }
  }, [enableServerPersistence]);

  // Enhanced theme variant setter with optimistic updates
  const setThemeVariant = useCallback(async (variant: UnifiedThemeVariant) => {
    if (enableOptimistic) {
      startTransition(async () => {
        // Optimistic update
        themeContext.setTheme(variant);
        
        // Persist to server (if enabled)
        await setServerThemeVariant(variant);
      });
    } else {
      // Standard update
      themeContext.setTheme(variant);
      await setServerThemeVariant(variant);
    }
  }, [themeContext, enableOptimistic, setServerThemeVariant]);

  // Enhanced theme mode setter
  const setThemeMode = useCallback(async (newMode: ThemeMode) => {
    const isSystem = newMode === 'system';
    setIsSystemMode(isSystem);
    
    if (enableOptimistic) {
      startTransition(async () => {
        // Update local state
        setModeState(newMode);
        localStorage.setItem(`${storageKeyPrefix}-theme-mode`, newMode);
        
        // Handle the actual dark mode toggle based on new mode
        if (newMode === 'dark' && !themeContext.isDarkMode) {
          themeContext.toggleDarkMode();
        } else if (newMode === 'light' && themeContext.isDarkMode) {
          themeContext.toggleDarkMode();
        } else if (newMode === 'system') {
          // Apply system preference
          const systemDarkMode = window.matchMedia('(prefers-color-scheme: dark)').matches;
          if (systemDarkMode !== themeContext.isDarkMode) {
            themeContext.toggleDarkMode();
          }
        }
        
        // Persist to server
        await setServerThemeMode(newMode);
      });
    } else {
      // Standard update
      setModeState(newMode);
      localStorage.setItem(`${storageKeyPrefix}-theme-mode`, newMode);
      
      if (newMode === 'dark' && !themeContext.isDarkMode) {
        themeContext.toggleDarkMode();
      } else if (newMode === 'light' && themeContext.isDarkMode) {
        themeContext.toggleDarkMode();
      }
      
      await setServerThemeMode(newMode);
    }
  }, [
    themeContext,
    enableOptimistic,
    storageKeyPrefix,
    setServerThemeMode,
  ]);

  // Enhanced toggle that respects mode
  const toggleTheme = useCallback(async () => {
    if (mode === 'system') {
      // If in system mode, switch to explicit light/dark
      const systemDarkMode = window.matchMedia('(prefers-color-scheme: dark)').matches;
      await setThemeMode(systemDarkMode ? 'light' : 'dark');
    } else {
      // Toggle between light and dark
      await setThemeMode(mode === 'light' ? 'dark' : 'light');
    }
  }, [mode, setThemeMode]);

  // Cycle through all available variants
  const cycleThemeVariant = useCallback(async () => {
    const variants: UnifiedThemeVariant[] = ['default', 'pancake', 'trading'];
    const currentTheme = themeContext.theme || 'default';
    const currentIndex = variants.indexOf(currentTheme);
    const nextIndex = (currentIndex + 1) % variants.length;
    const nextVariant = variants[nextIndex] || 'default';
    await setThemeVariant(nextVariant);
  }, [themeContext.theme, setThemeVariant]);

  // Reset to defaults
  const resetTheme = useCallback(async () => {
    await setThemeVariant('default');
    await setThemeMode('system');
  }, [setThemeVariant, setThemeMode]);

  // Get computed state
  const state: UnifiedThemeState = {
    variant: themeContext.theme,
    mode,
    isDarkMode: themeContext.isDarkMode,
    isSystemMode,
    isLoading: isPending,
  };

  // Get theme classes for SSR
  const getThemeClasses = useCallback(() => {
    const rootClasses = [
      themeContext.isDarkMode ? 'dark' : 'light',
      `theme-${themeContext.theme}`,
    ].join(' ');

    const dataAttributes = {
      'data-theme': themeContext.theme,
      'data-mode': themeContext.isDarkMode ? 'dark' : 'light',
      'data-theme-mode': mode,
    };

    return {
      rootClasses,
      dataAttributes,
    };
  }, [themeContext.theme, themeContext.isDarkMode, mode]);

  return {
    // Current state
    ...state,
    
    // Theme context values
    tokens: themeContext.tokens,
    config: themeContext.config,
    
    // Enhanced actions
    setThemeVariant,
    setThemeMode,
    toggleTheme,
    cycleThemeVariant,
    resetTheme,
    
    // Legacy compatibility
    setTheme: setThemeVariant,
    toggleDarkMode: themeContext.toggleDarkMode,
    
    // Utilities
    getThemeClasses,
  };
}

// SSR-compatible theme hook
export function useSSRTheme(
  initialState?: Partial<UnifiedThemeState>,
  options: UseUnifiedThemeOptions = {}
): ReturnType<typeof useUnifiedTheme> {
  const [isHydrated, setIsHydrated] = useState(false);
  // eslint-disable-next-line @typescript-eslint/no-unsafe-assignment
  const clientTheme = useUnifiedTheme(options);

  useEffect(() => {
    setIsHydrated(true);
  }, []);

  if (!isHydrated && initialState) {
    // Return server-side state before hydration
    return {
      ...initialState,
      variant: initialState.variant || 'default',
      mode: initialState.mode || 'system',
      isDarkMode: initialState.isDarkMode ?? false,
      isSystemMode: initialState.isSystemMode ?? false,
      isLoading: false,
      // Provide no-op functions for server-side
      setThemeVariant: async () => {},
      setThemeMode: async () => {},
      toggleTheme: async () => {},
      cycleThemeVariant: async () => {},
      resetTheme: async () => {},
      setTheme: async () => {},
      toggleDarkMode: () => {},
      getThemeClasses: () => ({
        rootClasses: `${initialState.isDarkMode ? 'dark' : 'light'} theme-${initialState.variant || 'default'}`,
        dataAttributes: {
          'data-theme': initialState.variant || 'default',
          'data-mode': initialState.isDarkMode ? 'dark' : 'light',
          'data-theme-mode': initialState.mode || 'system',
        },
      }),
      tokens: designTokens,
      config: themeConfig,
    };
  }

  return clientTheme;
}

// Optimistic theme hook for better UX
export function useOptimisticTheme(options: UseUnifiedThemeOptions = {}) {
  return useUnifiedTheme({
    ...options,
    enableOptimistic: true,
  });
}