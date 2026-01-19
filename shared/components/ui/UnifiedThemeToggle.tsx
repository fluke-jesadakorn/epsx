'use client';

/**
 * UNIFIED THEME TOGGLE
 * Shared theme toggle component for all EPSX applications
 * Consolidates frontend and admin implementations
 */

import { Moon, Sun } from 'lucide-react';
import { useTheme } from 'next-themes';
import { useEffect, useState } from 'react';

import { cn } from '../../utils';
import { Tooltip, TooltipContent, TooltipProvider, TooltipTrigger } from '../ui/tooltip';

// ============================================================================
// TYPES
// ============================================================================

export type ThemeToggleVariant = 'default' | 'gradient' | 'minimal' | 'compact' | 'simple' | 'inline';
export type ThemeToggleIconType = 'lucide' | 'emoji' | 'animated';
export type ThemeToggleSize = 'sm' | 'md' | 'lg';

export interface UnifiedThemeToggleProps {
    variant?: ThemeToggleVariant;
    iconType?: ThemeToggleIconType;
    showLabel?: boolean;
    showTooltip?: boolean;
    size?: ThemeToggleSize;
    className?: string;
}

// ============================================================================
// STYLE CONFIGURATIONS
// ============================================================================

const variantStyles = {
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
    },
    // Admin-compatible variants
    simple: {
        button: 'p-2 rounded-lg border border-gray-200 dark:border-gray-700 bg-white dark:bg-gray-800',
        active: 'text-blue-600 dark:text-yellow-500',
        loading: 'text-gray-400'
    },
    inline: {
        button: 'p-2 rounded-lg border border-gray-200 dark:border-gray-700 bg-white dark:bg-gray-800',
        active: 'text-blue-600 dark:text-yellow-500',
        loading: 'text-gray-400'
    }
};

const sizeStyles = {
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

// ============================================================================
// MAIN COMPONENT
// ============================================================================

export function UnifiedThemeToggle({
    variant = 'default',
    iconType = 'lucide',
    showLabel = false,
    showTooltip = true,
    size = 'md',
    className
}: UnifiedThemeToggleProps) {
    const { theme, resolvedTheme, setTheme } = useTheme();
    const [mounted, setMounted] = useState(false);

    useEffect(() => {
        setMounted(true);
    }, []);

    const style = variantStyles[variant];
    const sizeStyle = sizeStyles[size];
    const isDark = (resolvedTheme ?? theme) === 'dark';

    const getIcon = () => {
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
                {isDark ? 'Light' : 'Dark'}
            </span>
        );
    };

    const handleToggle = () => {
        if (!mounted) return;

        const newTheme = isDark ? 'light' : 'dark';

        // Standard next-themes approach
        setTheme(newTheme);

        // ALWAYS sync with the manual system (cookie and localStorage) 
        // to ensure consistency across different implementations
        try {
            document.cookie = `theme=${newTheme}; path=/; max-age=31536000; SameSite=lax`;
            localStorage.setItem('theme', newTheme);
        } catch (e) {
            console.warn('Failed to sync theme to storage:', e);
        }
    };

    const buttonContent = (
        <button
            onClick={handleToggle}
            disabled={!mounted}
            className={cn(
                'relative rounded-2xl font-semibold flex items-center justify-center gap-2',
                showLabel ? sizeStyle.padding : sizeStyle.button,
                mounted ? style.active : style.loading,
                mounted && style.button,
                'transition-all duration-200',
                className
            )}
            aria-label={mounted ? `Switch to ${isDark ? 'light' : 'dark'} theme` : 'Toggle theme (loading)'}
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

// ============================================================================
// PRESET VARIANTS (For backward compatibility)
// ============================================================================

/** Gradient theme toggle with emoji icons - from frontend */
export function GradientThemeToggle(props: Omit<UnifiedThemeToggleProps, 'variant'>) {
    return <UnifiedThemeToggle variant="gradient" iconType="emoji" {...props} />;
}

/** Minimal theme toggle with label - from frontend */
export function MinimalThemeToggle(props: Omit<UnifiedThemeToggleProps, 'variant'>) {
    return <UnifiedThemeToggle variant="minimal" showLabel={true} {...props} />;
}

/** Animated compact theme toggle - from frontend */
export function AnimatedThemeToggle(props: Omit<UnifiedThemeToggleProps, 'variant'>) {
    return <UnifiedThemeToggle variant="compact" iconType="animated" size="sm" {...props} />;
}

/** Simple theme toggle with direct DOM manipulation - from admin */
export function SimpleThemeToggle(props: Omit<UnifiedThemeToggleProps, 'variant'>) {
    return <UnifiedThemeToggle variant="simple" iconType="emoji" showTooltip={false} {...props} />;
}

/** Admin default theme toggle - from admin */
export function AdminThemeToggle(props: Omit<UnifiedThemeToggleProps, 'variant'>) {
    return <UnifiedThemeToggle variant="default" showTooltip={false} {...props} />;
}

// ============================================================================
// LEGACY ALIASES (For migration)
// ============================================================================

export const ThemeToggle = GradientThemeToggle;
export const ThemeToggleCSS = MinimalThemeToggle;
export const OptimizedThemeToggle = AnimatedThemeToggle;

export default UnifiedThemeToggle;
