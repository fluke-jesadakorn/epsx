'use server';

import { cookies } from 'next/headers';
import { COOKIE_NAMES, COOKIE_CONFIG } from '@/lib/cookies';

export type ThemeVariant = 'default' | 'insight' | 'trading';
export type ThemeMode = 'light' | 'dark' | 'system';

/**
 * Get current theme settings from cookies
 */
export async function getThemeSettings(): Promise<{
  variant: ThemeVariant;
  mode: ThemeMode;
  isDarkMode: boolean;
}> {
  const cookieStore = await cookies();

  const variant = (cookieStore.get(COOKIE_NAMES.THEME)?.value as ThemeVariant | undefined) ?? 'default';
  const mode = (cookieStore.get(COOKIE_NAMES.THEME_MODE)?.value as ThemeMode | undefined) ?? 'system';
  
  // Determine if dark mode is active
  let isDarkMode = false;
  if (mode === 'dark') {
    isDarkMode = true;
  } else if (mode === 'system') {
    // For system mode, we'll default to light on server-side
    // Client-side will override based on user's system preference
    isDarkMode = false;
  }
  
  return { variant, mode, isDarkMode };
}

/**
 * Set theme variant (default, insight, trading)
 */
export async function setThemeVariant(variant: ThemeVariant): Promise<void> {
  const cookieStore = await cookies();
  
  cookieStore.set(COOKIE_NAMES.THEME, variant, COOKIE_CONFIG.THEME);
}

/**
 * Set theme mode (light, dark, system)
 */
export async function setThemeMode(mode: ThemeMode): Promise<void> {
  const cookieStore = await cookies();
  
  cookieStore.set(COOKIE_NAMES.THEME_MODE, mode, COOKIE_CONFIG.THEME_MODE);
}

/**
 * Toggle between light and dark mode
 */
export async function toggleThemeMode(): Promise<ThemeMode> {
  const { mode } = await getThemeSettings();
  
  let newMode: ThemeMode;
  switch (mode) {
    case 'light':
      newMode = 'dark';
      break;
    case 'dark':
      newMode = 'light';
      break;
    case 'system':
      newMode = 'light'; // If system, switch to light explicitly
      break;
    default:
      newMode = 'light';
  }
  
  await setThemeMode(newMode);
  return newMode;
}

/**
 * Get theme CSS classes for server-side rendering
 */
export async function getThemeClasses(): Promise<{
  rootClasses: string;
  bodyClasses: string;
  dataAttributes: Record<string, string>;
}> {
  const { variant, mode, isDarkMode } = await getThemeSettings();
  
  const rootClasses = [
    isDarkMode ? 'dark' : 'light',
    `theme-${variant}`,
  ].join(' ');
  
  const bodyClasses = [
    'antialiased',
    'min-h-screen',
    'bg-background',
    'text-foreground',
  ].join(' ');
  
  const dataAttributes = {
    'data-theme': variant,
    'data-mode': isDarkMode ? 'dark' : 'light',
    'data-theme-mode': mode,
  };
  
  return {
    rootClasses,
    bodyClasses,
    dataAttributes,
  };
}