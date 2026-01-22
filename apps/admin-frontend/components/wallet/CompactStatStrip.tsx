'use client';

import { CheckCircle2, Clock, FileText, LucideIcon, TrendingUp, Users } from 'lucide-react';

import { Separator } from '@/components/ui/separator';
import { cn } from '@/lib/utils';

interface StatItemProps {
    icon: LucideIcon;
    label: string;
    value: string | number;
    colorClass: string;
    highlight?: boolean;
}

function StatItem({ icon: Icon, label, value, colorClass, highlight }: StatItemProps) {
    return (
        <div className={cn(
            "flex items-center gap-2.5 px-3 py-1.5 rounded-lg transition-colors group",
            highlight ? "bg-warning/10" : "hover:bg-muted/50"
        )}>
            <div className={cn("p-1.5 rounded-md", colorClass)}>
                <Icon className="h-3.5 w-3.5" />
            </div>
            <div className="flex flex-col">
                <span className="text-[10px] uppercase text-muted-foreground font-bold tracking-wider leading-none mb-0.5">
                    {label}
                </span>
                <span className={cn(
                    "text-sm font-bold leading-none tracking-tight",
                    highlight ? "text-warning" : "text-foreground"
                )}>
                    {value}
                </span>
            </div>
        </div>
    );
}

interface CompactStatStripProps {
    stats: {
        totalPolicies: number;
        activeCount: number;
        mrr: string;
        members: string;
        expiringSoon: number;
    };
    className?: string;
}

export function CompactStatStrip({ stats, className }: CompactStatStripProps) {
    return (
        <div className={cn(
            "flex flex-wrap items-center gap-2 p-1.5 bg-card/50 border border-border/60 rounded-xl backdrop-blur-sm self-start shadow-sm",
            className
        )}>
            <StatItem
                icon={FileText}
                label="Policies"
                value={stats.totalPolicies}
                colorClass="bg-primary/10 text-primary"
            />

            <Separator orientation="vertical" className="h-8 mx-1 hidden sm:block" />

            <StatItem
                icon={CheckCircle2}
                label="Active"
                value={stats.activeCount}
                colorClass="bg-success/10 text-success"
            />

            <Separator orientation="vertical" className="h-8 mx-1 hidden sm:block" />

            <StatItem
                icon={TrendingUp}
                label="MRR"
                value={stats.mrr}
                colorClass="bg-info/10 text-info"
            />

            <Separator orientation="vertical" className="h-8 mx-1 hidden sm:block" />

            <StatItem
                icon={Users}
                label="Members"
                value={stats.members}
                colorClass="bg-secondary/10 text-secondary"
            />

            <Separator orientation="vertical" className="h-8 mx-1 hidden sm:block" />

            <StatItem
                icon={Clock}
                label="Expiring"
                value={stats.expiringSoon}
                colorClass="bg-warning/10 text-warning"
                highlight={stats.expiringSoon > 0}
            />
        </div>
    );
}
