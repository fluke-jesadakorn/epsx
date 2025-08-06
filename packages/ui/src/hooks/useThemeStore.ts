"use client";

import React from 'react';
import { create } from 'zustand';
import { persist, createJSONStorage } from 'zustand/middleware';

import type { ThemeVariant } from '../tokens/theme-config.js';

// Define types for Zustand store methods
type SetState<T> = (partial: T | Partial<T> | ((state: T) => T | Partial<T>), replace?: boolean | undefined) => void;
type GetState<T> = () => T;

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
/* eslint-disable @typescript-eslint/no-unsafe-assignment, @typescript-eslint/no-unsafe-call, @typescript-eslint/no-unsafe-member-access */
export const useThemeStore = create<ThemeStoreState>()(
  persist(
    (set: SetState<ThemeStoreState>, get: GetState<ThemeStoreState>) => ({
      ...defaultState,
      
      // Core actions
      setVariant: (variant: ThemeVariant) => {
        set({ variant });
        
        // Apply to DOM immediately
        try {
          const state = get();
          const { rootClasses, dataAttributes } = state.getClasses();
          const root = document.documentElement;
          
          // Update classes
          root.className = rootClasses;
          
          // Update data attributes
          Object.entries(dataAttributes).forEach(([key, value]) => {
            root.setAttribute(key, value);
          });
        } catch (_error) {
          // Silently handle DOM errors in SSR or testing environments
        }
      },
      
      setMode: (mode: ThemeMode) => {
        const isSystemMode = mode === 'system';
        const state = get();
        let isDarkMode = state.isDarkMode;
        
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
        try {
          const state = get();
          const { rootClasses, dataAttributes } = state.getClasses();
          const root = document.documentElement;
          
          root.className = rootClasses;
          Object.entries(dataAttributes).forEach(([key, value]) => {
            root.setAttribute(key, value);
          });
        } catch (_error) {
          // Silently handle DOM errors in SSR or testing environments
        }
      },
      
      setDarkMode: (isDarkMode: boolean) => {
        // If we're setting dark mode explicitly, switch out of system mode
        const mode = isDarkMode ? 'dark' : 'light';
        set({ isDarkMode, mode, isSystemMode: false });
        
        // Apply to DOM
        try {
          const state = get();
          const { rootClasses, dataAttributes } = state.getClasses();
          const root = document.documentElement;
          
          root.className = rootClasses;
          Object.entries(dataAttributes).forEach(([key, value]) => {
            root.setAttribute(key, value);
          });
        } catch (_error) {
          // Silently handle DOM errors in SSR or testing environments
        }
      },
      
      toggleMode: () => {
        const { mode, isDarkMode } = get();
        
        const state = get();
        if (mode === 'system') {
          // If in system mode, switch to explicit opposite
          state.setMode(isDarkMode ? 'light' : 'dark');
        } else {
          // Toggle between light and dark
          state.setMode(mode === 'light' ? 'dark' : 'light');
        }
      },
      
      cycleVariant: () => {
        const variants: ThemeVariant[] = ['default', 'pancake', 'trading'];
        const state = get();
        const { variant } = state;
        const currentIndex = variants.indexOf(variant);
        const nextIndex = (currentIndex + 1) % variants.length;
        state.setVariant(variants[nextIndex]);
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
        const state = get();
        state.setMode('system'); // This will apply the reset to DOM
      },
      
      getClasses: (): { rootClasses: string; dataAttributes: Record<string, string> } => {
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
      partialize: (state): Partial<ThemeStoreState> => ({
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
/* eslint-enable @typescript-eslint/no-unsafe-assignment, @typescript-eslint/no-unsafe-call, @typescript-eslint/no-unsafe-member-access */

// System preference sync hook
/* eslint-disable @typescript-eslint/no-unsafe-assignment, @typescript-eslint/no-unsafe-call, @typescript-eslint/no-unsafe-member-access */
export function useSystemThemeSync(): void {
  const { mode, enableSystemSync, setDarkMode } = useThemeStore();
  
  // Set up system preference listener
  React.useEffect(() => {
    if (!enableSystemSync || mode !== 'system') return;
    
    const mediaQuery = window.matchMedia('(prefers-color-scheme: dark)');
    const handleChange = (): void => {
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
export function useThemeClasses(): { rootClasses: string; dataAttributes: Record<string, string> } {
  const getClasses = useThemeStore((state: ThemeStoreState) => state.getClasses);
  return getClasses() as { rootClasses: string; dataAttributes: Record<string, string> };
}

// Selector hooks for better performance
/* eslint-disable @typescript-eslint/no-unsafe-return, @typescript-eslint/no-unsafe-call */
export const useThemeVariant = (): ThemeVariant => useThemeStore((state: ThemeStoreState) => state.variant);
export const useThemeMode = (): ThemeMode => useThemeStore((state: ThemeStoreState) => state.mode);
export const useIsDarkMode = (): boolean => useThemeStore((state: ThemeStoreState) => state.isDarkMode);
export const useIsSystemMode = (): boolean => useThemeStore((state: ThemeStoreState) => state.isSystemMode);
/* eslint-enable @typescript-eslint/no-unsafe-return, @typescript-eslint/no-unsafe-call */

// Action selector hooks
/* eslint-disable @typescript-eslint/no-unsafe-return, @typescript-eslint/no-unsafe-call */
export const useThemeActions = (): Pick<ThemeStoreState, 'setVariant' | 'setMode' | 'setDarkMode' | 'toggleMode' | 'cycleVariant' | 'reset'> => useThemeStore((state) => ({
  setVariant: state.setVariant,
  setMode: state.setMode,
  setDarkMode: state.setDarkMode,
  toggleMode: state.toggleMode,
  cycleVariant: state.cycleVariant,
  reset: state.reset,
}));
/* eslint-enable @typescript-eslint/no-unsafe-assignment, @typescript-eslint/no-unsafe-call, @typescript-eslint/no-unsafe-member-access, @typescript-eslint/no-unsafe-return */