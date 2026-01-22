'use client';

import { cn } from '@/lib/utils';
import { Crown, TrendingUp, Users } from 'lucide-react';

interface DashboardStats {
    totalWallets: number;
    activeCount: number;
    disabledCount: number;
    subscribedCount: number;
    expiringSoon: number;
    mrr: string;
    members: string;
    growth: string;
}

interface CompactStatProps {
    label: string;
    value: number | string;
    trend?: string;
    trendUp?: boolean;
    colorClass: string;
    subValue?: string;
}

function CompactStat({ label, value, trend, trendUp, colorClass, subValue }: CompactStatProps) {
    return (
        <div className="flex flex-col min-w-[120px] px-4 first:pl-0 border-r border-border/40 last:border-0">
            <div className="flex items-center justify-between mb-1">
                <span className="text-xs font-medium text-muted-foreground">{label}</span>
                {trend && (
                    <span className={cn(
                        "text-[10px] font-bold flex items-center gap-0.5",
                        trendUp ? "text-green-500" : "text-red-500"
                    )}>
                        {trendUp ? '↑' : '↓'} {trend}
                    </span>
                )}
            </div>
            <div className="flex items-baseline gap-2">
                <span className="text-xl font-bold tracking-tight text-foreground/90">{value}</span>
                {subValue && (
                    <span className="text-[10px] text-muted-foreground truncate max-w-[80px]">{subValue}</span>
                )}
            </div>
        </div>
    );
}

interface DashboardSectionProps {
    stats: DashboardStats;
    className?: string;
}

export function DashboardSection({ stats, className }: DashboardSectionProps) {
    return (
        <div className={cn("hidden lg:block", className)}>
            <div className="bg-card/50 backdrop-blur-sm border border-border/50 rounded-xl p-4 shadow-sm">
                <div className="flex items-center justify-between">
                    {/* Left: Key Metrics Row */}
                    <div className="flex items-center flex-1">
                        <CompactStat
                            label="Total Wallets"
                            value={stats.totalWallets}
                            trend="23"
                            trendUp={true}
                            colorClass="text-blue-500"
                            subValue="vs last mo"
                        />
                        <CompactStat
                            label="Active"
                            value={stats.activeCount}
                            trend="15"
                            trendUp={true}
                            colorClass="text-green-500"
                            subValue="Active users"
                        />
                        <CompactStat
                            label="Disabled"
                            value={stats.disabledCount}
                            trend="3"
                            trendUp={false}
                            colorClass="text-red-500"
                            subValue="Attention"
                        />
                        <CompactStat
                            label="Subscribed"
                            value={stats.subscribedCount}
                            trend="12.4%"
                            trendUp={true}
                            colorClass="text-purple-500"
                            subValue="Paid plans"
                        />
                        <CompactStat
                            label="Expiring"
                            value={stats.expiringSoon}
                            trend="12"
                            trendUp={false}
                            colorClass="text-amber-500"
                            subValue="In 7 days"
                        />
                    </div>

                    {/* Right: Revenue/Business Summary */}
                    <div className="flex items-center gap-6 pl-6 border-l border-border/40 bg-zinc-50/50 dark:bg-zinc-900/50 py-2 px-6 rounded-lg ml-4">
                        <div className="flex flex-col text-right">
                            <div className="flex items-center justify-end gap-1.5 text-green-600 dark:text-green-500 mb-0.5">
                                <TrendingUp className="h-3 w-3" />
                                <span className="text-[10px] font-bold uppercase tracking-wider">MRR</span>
                            </div>
                            <span className="text-lg font-bold">{stats.mrr}</span>
                        </div>

                        <div className="flex flex-col text-right">
                            <div className="flex items-center justify-end gap-1.5 text-blue-600 dark:text-blue-500 mb-0.5">
                                <Users className="h-3 w-3" />
                                <span className="text-[10px] font-bold uppercase tracking-wider">Members</span>
                            </div>
                            <span className="text-lg font-bold">{stats.members}</span>
                        </div>

                        <div className="flex flex-col text-right">
                            <div className="flex items-center justify-end gap-1.5 text-purple-600 dark:text-purple-500 mb-0.5">
                                <Crown className="h-3 w-3" />
                                <span className="text-[10px] font-bold uppercase tracking-wider">Growth</span>
                            </div>
                            <span className="text-lg font-bold">{stats.growth}</span>
                        </div>
                    </div>
                </div>
            </div>
        </div>
    );
}
