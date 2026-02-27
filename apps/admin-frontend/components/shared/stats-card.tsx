import React from 'react';

interface StatsCardProps {
    title: string;
    value: string | number;
    icon: React.ElementType;
    iconBgColor?: string;
    iconColor?: string;
}

export function StatsCard({ title, value, icon: Icon, iconBgColor = 'bg-muted', iconColor = 'text-foreground' }: StatsCardProps) {
    return (
        <div className="rounded-xl bg-card border border-border/20 p-6 flex items-center gap-4 shadow-sm">
            <div className={`w-14 h-14 rounded-2xl ${iconBgColor}/10 border border-transparent dark:border-${iconBgColor.replace('bg-', '')}/20 flex items-center justify-center`}>
                <Icon className={`w-6 h-6 ${iconColor}`} />
            </div>
            <div>
                <p className="text-sm font-bold text-muted-foreground uppercase tracking-widest mb-1">{title}</p>
                <p className="text-3xl font-black text-foreground">{value}</p>
            </div>
        </div>
    );
}
