import React from 'react';
import { cn } from '../../utils/cn';

export interface PremiumCardProps extends React.HTMLAttributes<HTMLDivElement> {
    variant?: 'default' | 'highlight' | 'glass';
    glowColor?: 'blue' | 'purple' | 'orange' | 'green' | 'none';
    children: React.ReactNode;
}

export function PremiumCard({
    variant = 'default',
    glowColor = 'none',
    className,
    children,
    ...props
}: PremiumCardProps) {
    const getGlowStyles = () => {
        switch (glowColor) {
            case 'blue':
                return 'shadow-[0_0_30px_-5px_theme(colors.blue.500/0.3)] border-blue-500/30';
            case 'purple':
                return 'shadow-[0_0_30px_-5px_theme(colors.purple.500/0.3)] border-purple-500/30';
            case 'orange':
                return 'shadow-[0_0_30px_-5px_theme(colors.orange.500/0.3)] border-orange-500/30';
            case 'green':
                return 'shadow-[0_0_30px_-5px_theme(colors.green.500/0.3)] border-green-500/30';
            default:
                return 'hover:shadow-xl hover:shadow-blue-500/10 border-gray-200 dark:border-white/5';
        }
    };

    const getVariantStyles = () => {
        switch (variant) {
            case 'highlight':
                return 'bg-white dark:bg-gradient-to-b dark:from-gray-800/90 dark:to-gray-900/90';
            case 'glass':
                return 'bg-white/80 dark:bg-gray-900/60 backdrop-blur-md';
            default:
                return 'bg-white dark:bg-gray-900/60';
        }
    };

    return (
        <div
            className={cn(
                'relative rounded-2xl border transition-all duration-300 flex flex-col',
                getVariantStyles(),
                getGlowStyles(),
                className
            )}
            {...props}
        >
            {/* Glossy overlay effect */}
            <div className="absolute inset-0 rounded-2xl bg-gradient-to-b from-black/[0.02] dark:from-white/5 to-transparent pointer-events-none" />

            {/* Content */}
            <div className="relative z-10 h-full">
                {children}
            </div>
        </div>
    );
}
