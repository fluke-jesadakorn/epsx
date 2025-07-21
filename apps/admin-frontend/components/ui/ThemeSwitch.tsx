'use client';
import { Moon, Sun } from 'lucide-react';
import { useTheme } from 'next-themes';

export function ThemeSwitch() {
  const { setTheme, resolvedTheme } = useTheme();

  return (
    <button
      aria-label="Toggle theme"
      onClick={() => setTheme(resolvedTheme === 'dark' ? 'light' : 'dark')}
      className="button flex items-center gap-2 px-3 py-2 rounded"
    >
      {resolvedTheme === 'dark' ? (
        <Moon className="w-5 h-5 text-warning" />
      ) : (
        <Sun className="w-5 h-5 text-primary" />
      )}
    </button>
  );
}
