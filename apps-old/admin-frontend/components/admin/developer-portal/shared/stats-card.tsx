'use client';

import type { LucideIcon } from 'lucide-react';
import React from 'react';

interface StatsCardProps {
    title: string;
    value: number | string;
    icon: LucideIcon;
    iconBgColor: string;
    iconColor: string;
}

/**
 * Reusable stats card component for dashboard metrics
 * @param root0
 * @param root0.title
 * @param root0.value
 * @param root0.icon
 * @param root0.iconBgColor
 * @param root0.iconColor
 */
export const StatsCard: React.FC<StatsCardProps> = ({
    title,
    value,
    icon: Icon,
    iconBgColor,
    iconColor,
}) => {
    return (
        <div className="relative group overflow-hidden rounded-2xl bg-card border border-border/20 p-6 shadow-xl transition-all duration-300 hover:scale-[1.02] active:scale-[0.98]">
            {/* Background Glow */}
            <div className={`absolute -right-4 -bottom-4 w-24 h-24 blur-3xl opacity-10 group-hover:opacity-20 transition-opacity rounded-full ${iconBgColor}`} />

            <div className="relative flex items-center">
                <div className={`flex items-center justify-center w-12 h-12 rounded-[20px] bg-muted/30 border border-border/20 shadow-inner transition-transform duration-300 group-hover:rotate-6`}>
                    <Icon className={`w-6 h-6 ${iconColor}`} />
                </div>
                <div className="ml-5">
                    <p className="text-[10px] font-black text-muted-foreground uppercase tracking-[0.2em] mb-1">
                        {title}
                    </p>
                    <p className="text-2xl font-black text-foreground tracking-tight">
                        {typeof value === 'number' ? value.toLocaleString() : value}
                    </p>
                </div>
            </div>
        </div>
    );
};
