'use client';

import { Moon, Sun } from 'lucide-react';
import { useTheme } from 'next-themes';
import { useEffect, useState } from 'react';
import { cn } from '@/lib/utils';
import { Button } from '@/components/ui/button';
import { Tooltip, TooltipContent, TooltipProvider, TooltipTrigger } from '@/components/ui/tooltip';

type ThemeToggleVariant = 'default' | 'gradient' | 'minimal' | 'compact';
type IconType = 'lucide' | 'emoji' | 'animated';

interface UnifiedThemeToggleProps {
  variant?: ThemeToggleVariant;
  iconType?: IconType;
  showLabel?: boolean;
  showTooltip?: boolean;
  size?: 'sm' | 'md' | 'lg';
  className?: string;
}

export function UnifiedThemeToggle({
  variant = 'default',
  iconType = 'lucide',
  showLabel = false,
  showTooltip = true,
  size = 'md',
  className
}: UnifiedThemeToggleProps) {
  const { theme, setTheme } = useTheme();
  const [mounted, setMounted] = useState(false);

  useEffect(() => {
    setMounted(true);
  }, []);

  const variants = {
    default: {
      button: 'hover:bg-primary/10 border border-border',
      active: 'bg-background text-foreground',
      loading: 'bg-muted text-muted-foreground opacity-50'
    },
    gradient: {
      button: 'bg-gradient-to-r from-purple-400 to-pink-500 hover:from-purple-500 hover:to-pink-600 text-white shadow-lg hover:shadow-xl',
      active: 'bg-gradient-to-r from-purple-400 to-pink-500 text-white',
      loading: 'bg-gradient-to-r from-purple-400 to-pink-500 text-white opacity-50'
    },
    minimal: {
      button: 'hover:bg-yellow-50 hover:text-orange-600 dark:hover:bg-slate-800/50 dark:hover:text-orange-400 text-slate-600 dark:text-slate-300',
      active: 'text-slate-600 dark:text-slate-300',
      loading: 'text-slate-600 dark:text-slate-300 opacity-50'
    },
    compact: {
      button: 'hover:bg-primary/10',
      active: 'bg-background text-foreground',
      loading: 'bg-muted text-muted-foreground opacity-50'
    }
  };

  const sizes = {
    sm: {
      button: 'h-8 w-8 p-1.5',
      icon: 'h-3 w-3',
      text: 'text-xs',
      padding: 'px-2 py-1'
    },
    md: {
      button: 'h-10 w-10 p-2',
      icon: 'h-4 w-4',
      text: 'text-sm',
      padding: 'px-3 py-2'
    },
    lg: {
      button: 'h-12 w-12 p-3',
      icon: 'h-5 w-5',
      text: 'text-base',
      padding: 'px-4 py-2.5'
    }
  };

  const style = variants[variant];
  const sizeStyle = sizes[size];

  const getIcon = () => {
    const isDark = theme === 'dark';
    
    switch (iconType) {
      case 'emoji':
        return (
          <span className={cn('text-xl', size === 'sm' && 'text-base', size === 'lg' && 'text-2xl')}>
            {isDark ? '☀️' : '🌙'}
          </span>
        );
      
      case 'animated':
        return (
          <div className="relative">
            <Sun className={cn(
              sizeStyle.icon,
              'rotate-0 scale-100 transition-all dark:-rotate-90 dark:scale-0'
            )} />
            <Moon className={cn(
              sizeStyle.icon,
              'absolute top-0 left-0 rotate-90 scale-0 transition-all dark:rotate-0 dark:scale-100'
            )} />
          </div>
        );
      
      default: // lucide
        return isDark ? (
          <Sun className={cn(sizeStyle.icon, variant === 'minimal' && 'text-orange-500')} />
        ) : (
          <Moon className={cn(sizeStyle.icon, variant === 'minimal' && 'text-orange-500')} />
        );
    }
  };

  const getLabel = () => {
    if (!showLabel) return null;
    return (
      <span className={cn(sizeStyle.text, 'hidden sm:inline font-medium')}>
        {theme === 'dark' ? 'Light' : 'Dark'}
      </span>
    );
  };

  const buttonContent = (
    <button
      onClick={mounted ? () => setTheme(theme === 'light' ? 'dark' : 'light') : undefined}
      disabled={!mounted}
      className={cn(
        'relative rounded-2xl font-semibold flex items-center justify-center gap-2',
        showLabel ? sizeStyle.padding : sizeStyle.button,
        mounted ? style.active : style.loading,
        mounted && style.button,
        'transition-all duration-200',
        className
      )}
      aria-label={mounted ? `Switch to ${theme === 'light' ? 'dark' : 'light'} theme` : 'Toggle theme (loading)'}
    >
      {getIcon()}
      {getLabel()}
      <span className="sr-only">Toggle theme</span>
    </button>
  );

  if (showTooltip) {
    return (
      <TooltipProvider>
        <Tooltip>
          <TooltipTrigger asChild>
            {buttonContent}
          </TooltipTrigger>
          <TooltipContent>
            <p>Toggle theme</p>
          </TooltipContent>
        </Tooltip>
      </TooltipProvider>
    );
  }

  return buttonContent;
}

// Specific theme toggle variants for backward compatibility
export function GradientThemeToggle(props: Omit<UnifiedThemeToggleProps, 'variant'>) {
  return <UnifiedThemeToggle variant="gradient" iconType="emoji" {...props} />;
}

export function MinimalThemeToggle(props: Omit<UnifiedThemeToggleProps, 'variant'>) {
  return <UnifiedThemeToggle variant="minimal" showLabel={true} {...props} />;
}

export function AnimatedThemeToggle(props: Omit<UnifiedThemeToggleProps, 'variant'>) {
  return <UnifiedThemeToggle variant="compact" iconType="animated" size="sm" {...props} />;
}

// Legacy component aliases for backward compatibility
export const ThemeToggle = GradientThemeToggle;
export const ThemeToggleCSS = MinimalThemeToggle;
export const OptimizedThemeToggle = AnimatedThemeToggle;