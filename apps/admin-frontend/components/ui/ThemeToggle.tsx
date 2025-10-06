'use client';

import { Moon, Sun } from 'lucide-react';
import { useTheme } from 'next-themes';
import { useEffect, useState } from 'react';

interface ThemeToggleProps {
  variant?: 'default' | 'simple' | 'inline';
  size?: 'sm' | 'md' | 'lg';
  className?: string;
}

/**
 *
 * @param root0
 * @param root0.variant
 * @param root0.size
 * @param root0.className
 */
export function ThemeToggle({ 
  variant = 'default', 
  size = 'md',
  className = ''
}: ThemeToggleProps) {
  const { theme: _theme, setTheme, resolvedTheme } = useTheme();
  const [mounted, setMounted] = useState(false);

  useEffect(() => {
    setMounted(true);
  }, []);

  // Size classes
  const sizeClasses = {
    sm: 'w-4 h-4',
    md: 'w-5 h-5', 
    lg: 'w-6 h-6'
  };

  // Variant styles
  const variantClasses = {
    default: 'button p-2 rounded-full border-none',
    simple: 'p-2 rounded-lg border border-gray-200 dark:border-gray-700 bg-white dark:bg-gray-800',
    inline: 'p-2 rounded-lg border border-gray-200 dark:border-gray-700 bg-white dark:bg-gray-800'
  };

  if (!mounted) {
    if (variant === 'simple') {
      return (
        <div className={`${variantClasses[variant]} ${className}`}>
          <div className={sizeClasses[size]}>🌙</div>
        </div>
      );
    }
    
    return (
      <button
        aria-label="Toggle theme"
        className={`${variantClasses[variant]} ${className}`}
        disabled
      >
        <Sun className={`${sizeClasses[size]} text-gray-400`} />
      </button>
    );
  }

  const handleToggle = () => {
    if (variant === 'simple') {
      // Manual implementation for simple variant
      const html = document.documentElement;
      if (html.classList.contains('dark')) {
        html.classList.remove('dark');
        localStorage.setItem('theme', 'light');
      } else {
        html.classList.add('dark');
        localStorage.setItem('theme', 'dark');
      }
    } else {
      // Use next-themes for other variants
      setTheme(resolvedTheme === 'dark' ? 'light' : 'dark');
    }
  };

  if (variant === 'simple') {
    const isDark = document.documentElement.classList.contains('dark');
    return (
      <button
        onClick={handleToggle}
        className={`${variantClasses[variant]} ${className}`}
        aria-label="Toggle theme"
      >
        <div className={`${sizeClasses[size]} text-blue-600 dark:text-yellow-500`}>
          {isDark ? '☀️' : '🌙'}
        </div>
      </button>
    );
  }

  return (
    <button
      aria-label="Toggle theme"
      className={`${variantClasses[variant]} ${className}`}
      onClick={handleToggle}
    >
      {resolvedTheme === 'dark' ? (
        <Sun className={`${sizeClasses[size]} ${variant === 'default' ? 'text-warning' : 'text-yellow-500'}`} />
      ) : (
        <Moon className={`${sizeClasses[size]} ${variant === 'default' ? 'text-primary' : 'text-blue-600'}`} />
      )}
    </button>
  );
}
