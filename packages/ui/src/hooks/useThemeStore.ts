"use client";

import React from 'react';
import { create } from 'zustand';
import { persist, createJSONStorage } from 'zustand/middleware';
import type { ThemeVariant } from '../tokens/theme-config.js';

export type ThemeMode = 'light' | 'dark' | 'system';

export interface ThemeStoreState {
  // Core state
  variant: ThemeVariant;
  mode: ThemeMode;
  isDarkMode: boolean;
  isSystemMode: boolean;
  
  // Preferences
  enableTransitions: boolean;
  enableSystemSync: boolean;
  
  // Actions
  setVariant: (variant: ThemeVariant) => void;
  setMode: (mode: ThemeMode) => void;
  setDarkMode: (isDark: boolean) => void;
  toggleMode: () => void;
  cycleVariant: () => void;
  
  // Preferences
  setEnableTransitions: (enable: boolean) => void;
  setEnableSystemSync: (enable: boolean) => void;
  
  // Utilities
  reset: () => void;
  getClasses: () => { rootClasses: string; dataAttributes: Record<string, string> };
}

// Default state
const defaultState = {
  variant: 'default' as ThemeVariant,
  mode: 'system' as ThemeMode,
  isDarkMode: false,
  isSystemMode: true,
  enableTransitions: true,
  enableSystemSync: true,
};

/**
 * Zustand-based theme store that consolidates theme state management
 * This provides a global store alternative to the React Context approach
 */
export const useThemeStore = create<ThemeStoreState>()(
  persist(
    (set, get) => ({
      ...defaultState,
      
      // Core actions
      setVariant: (variant: ThemeVariant) => {
        set({ variant });
        
        // Apply to DOM immediately
        const { getClasses } = get();
        const { rootClasses, dataAttributes } = getClasses();
        const root = document.documentElement;
        
        // Update classes
        root.className = rootClasses;
        
        // Update data attributes
        Object.entries(dataAttributes).forEach(([key, value]) => {
          root.setAttribute(key, value);
        });
      },
      
      setMode: (mode: ThemeMode) => {
        const isSystemMode = mode === 'system';
        let isDarkMode = get().isDarkMode;
        
        // Determine dark mode state based on new mode
        if (mode === 'dark') {
          isDarkMode = true;
        } else if (mode === 'light') {
          isDarkMode = false;
        } else if (mode === 'system') {
          // Use system preference
          isDarkMode = window.matchMedia('(prefers-color-scheme: dark)').matches;
        }
        
        set({ mode, isSystemMode, isDarkMode });
        
        // Apply to DOM
        const { getClasses } = get();
        const { rootClasses, dataAttributes } = getClasses();
        const root = document.documentElement;
        
        root.className = rootClasses;
        Object.entries(dataAttributes).forEach(([key, value]) => {
          root.setAttribute(key, value);
        });
      },
      
      setDarkMode: (isDarkMode: boolean) => {
        // If we're setting dark mode explicitly, switch out of system mode
        const mode = isDarkMode ? 'dark' : 'light';
        set({ isDarkMode, mode, isSystemMode: false });
        
        // Apply to DOM
        const { getClasses } = get();
        const { rootClasses, dataAttributes } = getClasses();
        const root = document.documentElement;
        
        root.className = rootClasses;
        Object.entries(dataAttributes).forEach(([key, value]) => {
          root.setAttribute(key, value);
        });
      },
      
      toggleMode: () => {
        const { mode, isDarkMode } = get();
        
        if (mode === 'system') {
          // If in system mode, switch to explicit opposite
          get().setMode(isDarkMode ? 'light' : 'dark');
        } else {
          // Toggle between light and dark
          get().setMode(mode === 'light' ? 'dark' : 'light');
        }
      },
      
      cycleVariant: () => {
        const variants: ThemeVariant[] = ['default', 'pancake', 'trading'];
        const { variant } = get();
        const currentIndex = variants.indexOf(variant);
        const nextIndex = (currentIndex + 1) % variants.length;
        get().setVariant(variants[nextIndex]);
      },
      
      // Preferences
      setEnableTransitions: (enableTransitions: boolean) => {
        set({ enableTransitions });
        
        // Apply to DOM
        const root = document.documentElement;
        if (!enableTransitions) {
          root.style.setProperty('--transition-duration', '0ms');
        } else {
          root.style.removeProperty('--transition-duration');
        }
      },
      
      setEnableSystemSync: (enableSystemSync: boolean) => {
        set({ enableSystemSync });
      },
      
      // Utilities
      reset: () => {
        set(defaultState);
        get().setMode('system'); // This will apply the reset to DOM
      },
      
      getClasses: () => {
        const { variant, isDarkMode, mode } = get();
        
        const rootClasses = [
          isDarkMode ? 'dark' : 'light',
          `theme-${variant}`,
        ].join(' ');
        
        const dataAttributes = {
          'data-theme': variant,
          'data-mode': isDarkMode ? 'dark' : 'light',
          'data-theme-mode': mode,
        };
        
        return { rootClasses, dataAttributes };
      },
    }),
    {
      name: 'epsx-theme-store',
      storage: createJSONStorage(() => localStorage),
      // Only persist state, not functions
      partialize: (state) => ({
        variant: state.variant,
        mode: state.mode,
        isDarkMode: state.isDarkMode,
        isSystemMode: state.isSystemMode,
        enableTransitions: state.enableTransitions,
        enableSystemSync: state.enableSystemSync,
      }),
    }
  )
);

// System preference sync hook
export function useSystemThemeSync() {
  const { mode, enableSystemSync, setDarkMode } = useThemeStore();
  
  // Set up system preference listener
  React.useEffect(() => {
    if (!enableSystemSync || mode !== 'system') return;
    
    const mediaQuery = window.matchMedia('(prefers-color-scheme: dark)');
    const handleChange = () => {
      if (mode === 'system') {
        setDarkMode(mediaQuery.matches);
      }
    };
    
    // Set initial value
    handleChange();
    
    mediaQuery.addEventListener('change', handleChange);
    return () => mediaQuery.removeEventListener('change', handleChange);
  }, [mode, enableSystemSync, setDarkMode]);
}

// Convenience hook for theme classes
export function useThemeClasses() {
  const getClasses = useThemeStore((state) => state.getClasses);
  return getClasses();
}

// Selector hooks for better performance
export const useThemeVariant = () => useThemeStore((state) => state.variant);
export const useThemeMode = () => useThemeStore((state) => state.mode);
export const useIsDarkMode = () => useThemeStore((state) => state.isDarkMode);
export const useIsSystemMode = () => useThemeStore((state) => state.isSystemMode);

// Action selector hooks
export const useThemeActions = () => useThemeStore((state) => ({
  setVariant: state.setVariant,
  setMode: state.setMode,
  setDarkMode: state.setDarkMode,
  toggleMode: state.toggleMode,
  cycleVariant: state.cycleVariant,
  reset: state.reset,
}));