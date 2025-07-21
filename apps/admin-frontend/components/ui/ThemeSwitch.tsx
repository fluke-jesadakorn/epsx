'use client';
import { Moon, Sun } from 'lucide-react';
import { useTheme } from 'next-themes';

export function ThemeSwitch() {
  const { setTheme, resolvedTheme } = useTheme();

  return (
    <button
      aria-label="Toggle theme"
      onClick={() => setTheme(resolvedTheme === 'dark' ? 'light' : 'dark')}
      className="flex items-center gap-2 px-3 py-2 rounded bg-accent hover:bg-primary transition-colors duration-300"
    >
      <span className="text-xl font-bold">EPSX</span>
      {resolvedTheme === 'dark' ? (
        <Moon className="w-5 h-5 text-yellow-400" />
      ) : (
        <Sun className="w-5 h-5 text-orange-400" />
      )}
    </button>
  );
}
