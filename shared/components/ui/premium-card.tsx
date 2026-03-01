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
                return 'shadow-[0_0_30px_-5px_theme(colors.blue.500/0.3)] border-blue-400/40';
            case 'purple':
                return 'shadow-[0_0_30px_-5px_theme(colors.purple.500/0.3)] border-purple-400/40';
            case 'orange':
                return 'shadow-[0_0_30px_-5px_theme(colors.orange.500/0.3)] border-orange-400/40';
            case 'green':
                return 'shadow-[0_0_30px_-5px_theme(colors.green.500/0.3)] border-green-400/40';
            default:
                return 'hover:shadow-xl hover:shadow-blue-500/10 border-white/20 dark:border-white/10';
        }
    };

    const getVariantStyles = () => {
        switch (variant) {
            case 'highlight':
                return 'bg-white/8 dark:bg-white/5 backdrop-blur-xl border-white/20 dark:border-white/10';
            case 'glass':
                return 'bg-white/12 dark:bg-white/8 backdrop-blur-2xl border-white/25 dark:border-white/15';
            default:
                return 'bg-white/8 dark:bg-white/5 backdrop-blur-xl border-white/20 dark:border-white/10';
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
            <div className="absolute inset-0 rounded-2xl bg-gradient-to-b from-white/10 dark:from-white/8 to-transparent pointer-events-none" />

            {/* Content */}
            <div className="relative z-10 h-full">
                {children}
            </div>
        </div>
    );
}
