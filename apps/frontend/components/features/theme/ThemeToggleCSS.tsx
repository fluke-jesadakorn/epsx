'use client';

import { Moon, Sun } from 'lucide-react';
import { useTheme } from 'next-themes';
import { useNavbarContext } from '@/components/providers/NavbarProvider';

/**
 * CSS-only theme toggle that works without JavaScript during SSR
 * Falls back to interactive version after hydration
 */
export function ThemeToggleCSS() {
  const { theme, setTheme } = useTheme();
  const { isHydrated } = useNavbarContext();

  // CSS-only fallback during SSR/hydration
  if (!isHydrated) {
    return (
      <button
        disabled
        className="flex items-center gap-2 rounded-2xl px-4 py-2.5 text-sm font-semibold text-slate-600 dark:text-slate-300"
        aria-label="Toggle theme (loading)"
      >
        <Moon className="h-4 w-4 text-orange-500" />
        Theme
      </button>
    );
  }

  // Interactive version after hydration
  return (
    <button
      onClick={() => setTheme(theme === "light" ? "dark" : "light")}
      className="flex items-center gap-2 rounded-2xl px-4 py-2.5 text-sm font-semibold text-slate-600 hover:bg-yellow-50 hover:text-orange-600 dark:text-slate-300 dark:hover:bg-slate-800/50 dark:hover:text-orange-400"
      aria-label={`Switch to ${theme === "light" ? "dark" : "light"} theme`}
    >
      {theme === "light" ? (
        <Moon className="h-4 w-4 text-orange-500" />
      ) : (
        <Sun className="h-4 w-4 text-orange-500" />
      )}
      Theme
    </button>
  );
}