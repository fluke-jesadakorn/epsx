
import React from 'react'
import { cn } from '../../utils/cn'

export type LoaderVariant = 'default' | 'pancake' | 'admin' | 'analytics' | 'premium' | 'white'
export type LoaderSize = 'sm' | 'md' | 'lg'

export interface LoaderStyle {
    primary: string
    secondary: string
    accent: string
    text: string
    icon: string
}

export const loaderVariants: Record<LoaderVariant, LoaderStyle> = {
    default: {
        primary: 'border-blue-500',
        secondary: 'border-blue-200',
        accent: 'bg-blue-500',
        text: 'text-blue-600',
        icon: '⚡'
    },
    pancake: {
        primary: 'border-orange-500',
        secondary: 'border-orange-200',
        accent: 'bg-gradient-to-r from-orange-400 to-yellow-500',
        text: 'text-orange-600',
        icon: '🥞'
    },
    admin: {
        primary: 'border-blue-600',
        secondary: 'border-blue-200',
        accent: 'bg-gradient-to-r from-blue-600 to-indigo-700',
        text: 'text-blue-600',
        icon: '⚡'
    },
    analytics: {
        primary: 'border-indigo-500',
        secondary: 'border-indigo-200',
        accent: 'bg-gradient-to-r from-indigo-500 to-purple-600',
        text: 'text-indigo-600',
        icon: '📊'
    },
    premium: {
        primary: 'border-purple-500',
        secondary: 'border-purple-200',
        accent: 'bg-gradient-to-r from-purple-500 to-pink-600',
        text: 'text-purple-600',
        icon: '💎'
    },
    white: {
        primary: 'border-white',
        secondary: 'border-white/50',
        accent: 'bg-white',
        text: 'text-white',
        icon: '⚡'
    }
}

export const sizeClasses = {
    sm: 'w-4 h-4',
    md: 'w-6 h-6',
    lg: 'w-8 h-8'
}

interface MiniLoaderProps {
    size: LoaderSize
    style: LoaderStyle
}

export const SpinnerLoader: React.FC<MiniLoaderProps> = ({ size, style }) => (
    <div
        className={cn(
            sizeClasses[size],
            'border-2 border-t-transparent rounded-full animate-spin',
            style.primary
        )}
    />
)

export const DotsLoader: React.FC<MiniLoaderProps> = ({ size, style }) => (
    <div className="flex gap-1">
        {Array.from({ length: 3 }).map((_, i) => (
            <div
                // eslint-disable-next-line react/no-array-index-key
                key={`dot-${i}`}
                className={cn(
                    'rounded-full animate-pulse',
                    size === 'sm' ? 'w-1 h-1' : size === 'md' ? 'w-1.5 h-1.5' : 'w-2 h-2',
                    style.accent
                )}
                style={{
                    animationDelay: `${i * 0.2}s`,
                    animationDuration: '1s'
                }}
            />
        ))}
    </div>
)

export const BarsLoader: React.FC<MiniLoaderProps> = ({ size, style }) => (
    <div className="flex gap-1 items-end">
        {Array.from({ length: 4 }).map((_, i) => (
            <div
                // eslint-disable-next-line react/no-array-index-key
                key={`bar-${i}`}
                className={cn(
                    'animate-pulse',
                    size === 'sm' ? 'w-1 h-3' : size === 'md' ? 'w-1.5 h-4' : 'w-2 h-6',
                    style.accent
                )}
                style={{
                    animationDelay: `${i * 0.15}s`,
                    animationDuration: '0.8s'
                }}
            />
        ))}
    </div>
)

export const PulseLoader: React.FC<MiniLoaderProps> = ({ size, style }) => (
    <div className="relative">
        <div
            className={cn(
                sizeClasses[size],
                'rounded-full animate-ping absolute opacity-75',
                style.accent
            )}
        />
        <div
            className={cn(
                sizeClasses[size],
                'rounded-full relative',
                style.accent
            )}
        />
    </div>
)

export const StackLoader: React.FC<MiniLoaderProps> = ({ size, style }) => (
    <div className="relative">
        {/* Bottom Layer */}
        <div className={cn(sizeClasses[size], style.accent, 'rounded-full relative z-10')} />

        {/* Middle Layer */}
        <div className={cn(sizeClasses[size], style.secondary, 'rounded-full absolute top-0 left-0 z-20')} />

        {/* Top Layer with Icon */}
        <div className={cn(
            sizeClasses[size],
            style.accent,
            'rounded-full absolute top-0 left-0 z-30 flex items-center justify-center text-white'
        )}>
            <span className={size === 'sm' ? 'text-xs' : size === 'md' ? 'text-sm' : 'text-base'}>
                {style.icon}
            </span>
        </div>
    </div>
)
