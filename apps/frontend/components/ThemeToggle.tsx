'use client';

import { Moon, Sun } from 'lucide-react';
import { useTheme } from 'next-themes';
import { useEffect, useState } from 'react';

import {
  Tooltip,
  TooltipContent,
  TooltipProvider,
  TooltipTrigger,
} from '@/components/ui/tooltip';
import { useThemeStore } from '@/lib/store/theme';

export default function ThemeToggle() {
  const { theme, setTheme, resolvedTheme } = useTheme();
  const [mounted, setMounted] = useState(false);
  const { setDarkMode } = useThemeStore();

  useEffect(() => {
    setMounted(true);
  }, []);

  useEffect(() => {
    if (resolvedTheme) {
      setDarkMode(resolvedTheme === 'dark');
    }
  }, [resolvedTheme, setDarkMode]);

  if (!mounted) {
    return (
      <button
        disabled
        className="w-16 h-8 rounded-full bg-gradient-to-r from-gray-200 to-gray-300 dark:from-gray-600 dark:to-gray-700 transition-all duration-300 relative z-50 shadow-lg hover:shadow-xl hover:scale-105 pancake-shadow"
      >
        <div className="absolute top-0.5 left-0.5 w-7 h-7 rounded-full bg-white dark:bg-gray-800 shadow-lg transition-all duration-300 transform translate-x-0 flex items-center justify-center group">
          <Sun className="h-4 w-4 text-pancake-primary group-hover:animate-wiggle" />
        </div>
      </button>
    );
  }

  return (
    <TooltipProvider>
      <Tooltip>
        <TooltipTrigger asChild>
          <button
            onClick={() => {
              const newTheme = theme === 'dark' ? 'light' : 'dark';
              setTheme(newTheme);
            }}
            className="w-16 h-8 rounded-full bg-gradient-to-r from-gray-200 to-gray-300 dark:from-gray-600 dark:to-gray-700 transition-all duration-300 relative z-50 shadow-lg hover:shadow-xl hover:scale-105 pancake-shadow"
          >
            <div
              className={`absolute top-0.5 left-0.5 w-7 h-7 rounded-full bg-white dark:bg-gray-800 shadow-lg transition-all duration-300 transform ${
                theme === 'dark' ? 'translate-x-8' : 'translate-x-0'
              } flex items-center justify-center group`}
            >
              {theme === 'dark' ? (
                <Moon className="h-4 w-4 text-blue-400 group-hover:animate-wiggle" />
              ) : (
                <Sun className="h-4 w-4 text-pancake-primary group-hover:animate-wiggle" />
              )}
            </div>
          </button>
        </TooltipTrigger>
        <TooltipContent className="z-100 bg-card border border-border text-foreground">
          <p>Toggle theme</p>
        </TooltipContent>
      </Tooltip>
    </TooltipProvider>
  );
}
