'use client';

/**
 * UNIFIED CARD VARIANTS
 * Specialized card components that wrap the BaseCard
 * Consolidates frontend's UnifiedCard variants and admin's PancakeCard
 */

import type { ReactNode } from 'react';
import { cn } from '../../utils';

// ============================================================================
// VARIANT TYPES
// ============================================================================

export type UnifiedCardVariant = 'default' | 'pancake' | 'admin' | 'analytics' | 'premium';
export type AccentPosition = 'left' | 'top' | 'bottom' | 'right' | 'none';
export type UnifiedCardSize = 'sm' | 'md' | 'lg';
export type UnifiedCardPadding = 'none' | 'sm' | 'md' | 'lg';

// ============================================================================
// UNIFIED CARD PROPS
// ============================================================================

export interface UnifiedCardProps {
    children: ReactNode;
    variant?: UnifiedCardVariant;
    size?: UnifiedCardSize;
    padding?: UnifiedCardPadding;
    hover?: boolean;
    accent?: AccentPosition;
    className?: string;
}

export interface UnifiedCardSectionProps {
    children: ReactNode;
    className?: string;
}

// ============================================================================
// STYLE CONFIGURATIONS
// ============================================================================

const variantStyles = {
    default: {
        bg: 'bg-white dark:bg-gray-900',
        bgGlass: 'bg-white/80 dark:bg-gray-900/80',
        border: 'border-gray-200 dark:border-gray-700',
        accent: 'bg-gradient-to-r from-blue-500 to-blue-600',
        shadow: 'shadow-sm',
        text: 'text-gray-900 dark:text-gray-100'
    },
    pancake: {
        bg: 'bg-gradient-to-br from-orange-50 to-yellow-50 dark:from-orange-950 dark:to-yellow-950',
        bgGlass: 'bg-white/10 dark:bg-gray-900/10',
        border: 'border-orange-200 dark:border-orange-800',
        accent: 'bg-gradient-to-r from-orange-400 to-yellow-500',
        shadow: 'shadow-lg shadow-orange-100 dark:shadow-orange-900/20',
        text: 'text-gray-800 dark:text-gray-100'
    },
    admin: {
        bg: 'bg-slate-800 dark:bg-slate-900',
        bgGlass: 'bg-slate-800/20 dark:bg-slate-900/20',
        border: 'border-blue-500 dark:border-blue-400',
        accent: 'bg-gradient-to-r from-blue-600 to-indigo-700',
        shadow: 'shadow-lg shadow-blue-100 dark:shadow-blue-900/20',
        text: 'text-white'
    },
    analytics: {
        bg: 'bg-gradient-to-br from-blue-50 to-indigo-50 dark:from-blue-950 dark:to-indigo-950',
        bgGlass: 'bg-blue-50/80 dark:bg-blue-950/80',
        border: 'border-blue-200 dark:border-blue-800',
        accent: 'bg-gradient-to-r from-indigo-500 to-purple-600',
        shadow: 'shadow-sm',
        text: 'text-gray-900 dark:text-gray-100'
    },
    premium: {
        bg: 'bg-gradient-to-br from-purple-50 to-pink-50 dark:from-purple-950 dark:to-pink-950',
        bgGlass: 'bg-purple-50/80 dark:bg-purple-950/80',
        border: 'border-purple-200 dark:border-purple-800',
        accent: 'bg-gradient-to-r from-purple-500 to-pink-600',
        shadow: 'shadow-sm',
        text: 'text-gray-900 dark:text-gray-100'
    }
};

const sizeClasses = {
    sm: 'max-w-sm',
    md: 'max-w-md',
    lg: 'max-w-lg'
};

const paddingClasses = {
    none: 'p-0',
    sm: 'p-4',
    md: 'p-6',
    lg: 'p-8'
};

const accentPositions = {
    left: 'left-0 top-0 bottom-0 w-1',
    right: 'right-0 top-0 bottom-0 w-1',
    top: 'top-0 left-0 right-0 h-1',
    bottom: 'bottom-0 left-0 right-0 h-1',
    none: 'hidden'
};

// ============================================================================
// UNIFIED CARD COMPONENT
// ============================================================================

export function UnifiedCard({
    children,
    variant = 'default',
    size = 'md',
    padding = 'md',
    hover = true,
    accent = 'none',
    className
}: UnifiedCardProps) {
    const style = variantStyles[variant];

    return (
        <div
            className={cn(
                'relative border rounded-lg overflow-hidden',
                style.bg,
                style.border,
                style.text,
                style.shadow,
                sizeClasses[size],
                paddingClasses[padding],
                hover && 'hover:shadow-xl transition-all duration-200',
                className
            )}
        >
            {/* Accent Bar */}
            {accent !== 'none' && (
                <div
                    className={cn(
                        'absolute z-10 rounded-sm',
                        accentPositions[accent],
                        style.accent
                    )}
                />
            )}

            {/* Content */}
            <div className="relative z-5">
                {children}
            </div>
        </div>
    );
}

// ============================================================================
// CARD SECTIONS
// ============================================================================

export function UnifiedCardHeader({ children, className }: UnifiedCardSectionProps) {
    return (
        <div className={cn('pb-4 border-b border-gray-200 dark:border-gray-700', className)}>
            {children}
        </div>
    );
}

export function UnifiedCardContent({ children, className }: UnifiedCardSectionProps) {
    return (
        <div className={cn('py-4', className)}>
            {children}
        </div>
    );
}

export function UnifiedCardFooter({ children, className }: UnifiedCardSectionProps) {
    return (
        <div className={cn('pt-4 border-t border-gray-200 dark:border-gray-700', className)}>
            {children}
        </div>
    );
}

// ============================================================================
// SPECIALIZED CARD VARIANTS
// ============================================================================

/** PancakeSwap-style gradient card */
export function PancakeCard(props: Omit<UnifiedCardProps, 'variant'>) {
    return <UnifiedCard variant="pancake" {...props} />;
}

/** Admin/dashboard style card */
export function AdminCard(props: Omit<UnifiedCardProps, 'variant'>) {
    return <UnifiedCard variant="admin" {...props} />;
}

/** Analytics/data visualization card */
export function AnalyticsCard(props: Omit<UnifiedCardProps, 'variant'>) {
    return <UnifiedCard variant="analytics" {...props} />;
}

/** Premium/subscription card */
export function PremiumCard(props: Omit<UnifiedCardProps, 'variant'>) {
    return <UnifiedCard variant="premium" {...props} />;
}

// ============================================================================
// STATS CARD
// ============================================================================

export interface UnifiedStatsCardProps {
    variant?: UnifiedCardVariant;
    icon?: ReactNode;
    value?: string | number;
    label?: string;
    trend?: 'up' | 'down' | 'neutral';
    percentage?: string;
    className?: string;
}

export function UnifiedStatsCard({
    variant = 'default',
    icon,
    value,
    label,
    trend,
    percentage,
    className
}: UnifiedStatsCardProps) {
    const trendIcons = {
        up: '↗',
        down: '↘',
        neutral: '→'
    };

    const trendColors = {
        up: 'text-green-600 dark:text-green-400',
        down: 'text-red-600 dark:text-red-400',
        neutral: 'text-gray-500 dark:text-gray-400'
    };

    return (
        <UnifiedCard variant={variant} className={cn('min-w-48', className)}>
            <div className="flex items-start justify-between">
                <div className="flex-1">
                    {Boolean(icon) && (
                        <div className="text-2xl mb-3">
                            {icon}
                        </div>
                    )}

                    <div className="text-2xl font-bold mb-1">
                        {value}
                    </div>

                    {Boolean(label) && (
                        <div className="text-sm opacity-70 font-medium">
                            {label}
                        </div>
                    )}
                </div>

                {Boolean(trend) && Boolean(percentage) && (
                    <div className={cn(
                        'flex items-center space-x-1 text-sm font-medium',
                        trendColors[trend as 'up' | 'down' | 'neutral']
                    )}>
                        <span className="text-base">{trendIcons[trend as 'up' | 'down' | 'neutral']}</span>
                        <span>{percentage}</span>
                    </div>
                )}
            </div>
        </UnifiedCard>
    );
}

// ============================================================================
// LIST CARD
// ============================================================================

export interface UnifiedListItem {
    icon?: ReactNode;
    title: string;
    subtitle?: string;
    action?: ReactNode;
    href?: string;
}

export interface UnifiedListCardProps {
    variant?: UnifiedCardVariant;
    items: UnifiedListItem[];
    title?: string;
    className?: string;
}

export function UnifiedListCard({
    variant = 'default',
    items,
    title,
    className
}: UnifiedListCardProps) {
    return (
        <UnifiedCard variant={variant} className={className}>
            {Boolean(title) && (
                <h3 className="font-semibold text-lg mb-4 opacity-90">{title}</h3>
            )}

            <div className="space-y-3">
                {items.map((item) => {
                    const Component = item.href !== undefined && item.href !== '' ? 'a' : 'div';
                    return (
                        <Component
                            key={item.title}
                            href={item.href}
                            className={cn(
                                'flex items-center justify-between py-3 border-b border-gray-100 dark:border-gray-800 last:border-b-0',
                                Boolean(item.href) && 'hover:bg-gray-50 dark:hover:bg-gray-800 cursor-pointer rounded-md px-2 -mx-2'
                            )}
                        >
                            <div className="flex items-center space-x-3">
                                {Boolean(item.icon) && (
                                    <div className="text-lg">{item.icon}</div>
                                )}
                                <div>
                                    <div className="font-medium">{item.title}</div>
                                    {Boolean(item.subtitle) && (
                                        <div className="text-sm opacity-70">{item.subtitle}</div>
                                    )}
                                </div>
                            </div>

                            {Boolean(item.action) && (
                                <div className="text-sm opacity-70">{item.action}</div>
                            )}
                        </Component>
                    );
                })}
            </div>
        </UnifiedCard>
    );
}

// ============================================================================
// FEATURE CARD
// ============================================================================

export interface UnifiedFeatureCardProps {
    variant?: UnifiedCardVariant;
    icon?: ReactNode;
    title: string;
    description: string;
    action?: ReactNode;
    className?: string;
}

export function UnifiedFeatureCard({
    variant = 'default',
    icon,
    title,
    description,
    action,
    className
}: UnifiedFeatureCardProps) {
    return (
        <UnifiedCard variant={variant} className={cn('text-center', className)}>
            {Boolean(icon) && (
                <div className="text-3xl mb-4 flex justify-center">
                    {icon}
                </div>
            )}

            <h3 className="font-semibold text-lg mb-2">{title}</h3>
            <p className="opacity-70 mb-4">{description}</p>

            {Boolean(action) && (
                <div className="mt-4">
                    {action}
                </div>
            )}
        </UnifiedCard>
    );
}

// ============================================================================
// LEGACY ALIASES
// ============================================================================

export const MetroCard = UnifiedCard;
export const EPSXCard = UnifiedCard;
export const ProfessionalCard = UnifiedCard;
export const MetroStatsCard = UnifiedStatsCard;
export const ProfessionalStatsCard = UnifiedStatsCard;
export const MetroListCard = UnifiedListCard;
export const ProfessionalListCard = UnifiedListCard;
export const ProfessionalFeatureCard = UnifiedFeatureCard;
export const EPSXCardHeader = UnifiedCardHeader;
export const EPSXCardContent = UnifiedCardContent;
export const EPSXCardFooter = UnifiedCardFooter;

export default UnifiedCard;
