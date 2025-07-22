'use client';
import { Moon, Sun } from 'lucide-react';
import { useTheme } from 'next-themes';
import { useEffect, useState } from 'react';

export function ThemeSwitch() {
  const { setTheme, resolvedTheme, theme } = useTheme();
  const [mounted, setMounted] = useState(false);

  useEffect(() => {
    setMounted(true);
  }, []);

  if (!mounted) {
    return (
      <div 
        className="flex h-11 w-11 items-center justify-center rounded-full bg-muted/50 animate-pulse"
        aria-hidden="true"
      />
    );
  }

  const isDark = resolvedTheme === 'dark';
  const isSystemTheme = theme === 'system';

  return (
    <div className="relative">
      <button
        aria-label={`Switch to ${isDark ? 'light' : 'dark'} theme`}
        aria-pressed={isDark}
        onClick={() => setTheme(isDark ? 'light' : 'dark')}
        className="group relative flex h-11 w-11 items-center justify-center rounded-full border-2 border-border/50 bg-background/50 backdrop-blur-sm transition-all duration-300 hover:border-primary/50 hover:bg-primary/5 hover:scale-110 focus:outline-none focus:ring-2 focus:ring-primary/50 focus:ring-offset-2 focus:ring-offset-background active:scale-95"
      >
        <div className="relative overflow-hidden">
          {/* Sun icon - visible in light mode */}
          <Sun 
            className={`h-5 w-5 transition-all duration-500 ${
              isDark 
                ? 'rotate-90 scale-0 opacity-0 text-muted-foreground' 
                : 'rotate-0 scale-100 opacity-100 text-primary'
            }`} 
          />
          
          {/* Moon icon - visible in dark mode */}
          <Moon 
            className={`absolute inset-0 h-5 w-5 transition-all duration-500 ${
              isDark 
                ? 'rotate-0 scale-100 opacity-100 text-warning' 
                : '-rotate-90 scale-0 opacity-0 text-muted-foreground'
            }`} 
          />
        </div>
        
        {/* Animated background indicator */}
        <div 
          className={`absolute inset-0 rounded-full bg-gradient-to-r transition-all duration-500 ${
            isDark 
              ? 'from-blue-500/20 to-purple-500/20 opacity-100' 
              : 'from-orange-500/20 to-yellow-500/20 opacity-100'
          }`}
        />
        
        {/* Hover glow effect */}
        <div className="absolute inset-0 rounded-full bg-primary/10 opacity-0 transition-all duration-300 group-hover:opacity-100 group-hover:scale-110" />
      </button>
      
      {/* Theme indicator tooltip */}
      <div 
        className="pointer-events-none absolute -bottom-8 left-1/2 -translate-x-1/2 rounded-md bg-popover px-2 py-1 text-xs text-popover-foreground opacity-0 transition-all duration-200 group-hover:opacity-100 z-50"
        role="tooltip"
      >
        {isDark ? 'Dark' : 'Light'} mode
        {isSystemTheme && (
          <span className="text-muted-foreground"> (auto)</span>
        )}
      </div>
    </div>
  );
}
