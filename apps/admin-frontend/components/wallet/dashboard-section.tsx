'use client';

import { cn } from '@/lib/utils';
import { AlertTriangle, ArrowDownRight, ArrowUpRight, Clock, Package, Users, Wallet } from 'lucide-react';

interface DashboardStats {
    totalWallets: number;
    activeCount: number;
    disabledCount: number;
    subscribedCount: number;
    expiringSoon: number;
}

interface StatCardProps {
    label: string;
    value: number | string;
    trend?: string;
    trendUp?: boolean;
    subValue?: string;
    icon: React.ReactNode;
    colorClass: string;
    bgClass: string;
}

function StatCard({ label, value, trend, trendUp, subValue, icon, colorClass, bgClass }: StatCardProps) {
    return (
        <div className={cn(
            "relative overflow-hidden rounded-2xl p-0.5 transition-transform hover:scale-[1.02] duration-300",
            bgClass
        )}>
            <div className="relative h-full bg-card/90 backdrop-blur-xl rounded-2xl p-5 flex flex-col justify-between">
                <div className="flex items-start justify-between mb-4">
                    <div className={cn("p-2 rounded-lg bg-muted/50", colorClass)}>
                        {icon}
                    </div>
                    {trend && (
                        <div className={cn(
                            "flex items-center gap-1 text-xs font-bold px-2 py-1 rounded-full bg-muted/50",
                            trendUp ? "text-green-500" : "text-red-500"
                        )}>
                            {trendUp ? <ArrowUpRight className="w-3 h-3" /> : <ArrowDownRight className="w-3 h-3" />}
                            {trend}
                        </div>
                    )}
                </div>

                <div>
                    <div className="text-2xl font-bold text-card-foreground mb-1 tracking-tight">{value}</div>
                    <div className="flex items-center justify-between">
                        <span className="text-xs font-medium text-muted-foreground">{label}</span>
                        {subValue && (
                            <span className="text-[10px] text-muted-foreground font-medium bg-muted/50 px-1.5 py-0.5 rounded">
                                {subValue}
                            </span>
                        )}
                    </div>
                </div>
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
        <div className={cn("space-y-6", className)}>
            {/* Key Metrics Grid */}
            <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 xl:grid-cols-5 gap-4">
                <StatCard
                    label="Total Wallets"
                    value={stats.totalWallets}
                    trend="23"
                    trendUp={true}
                    subValue="vs last mo"
                    icon={<Wallet className="w-5 h-5" />}
                    colorClass="text-[#1fc7d4]"
                    bgClass="bg-gradient-to-br from-[#1fc7d4]/20 via-transparent to-transparent"
                />
                <StatCard
                    label="Active Users"
                    value={stats.activeCount}
                    trend="15"
                    trendUp={true}
                    subValue="Active"
                    icon={<Users className="w-5 h-5" />}
                    colorClass="text-[#31d0aa]"
                    bgClass="bg-gradient-to-br from-[#31d0aa]/20 via-transparent to-transparent"
                />
                <StatCard
                    label="Disabled"
                    value={stats.disabledCount}
                    trend="3"
                    trendUp={false}
                    subValue="Attention"
                    icon={<AlertTriangle className="w-5 h-5" />}
                    colorClass="text-[#ed4b9e]"
                    bgClass="bg-gradient-to-br from-[#ed4b9e]/20 via-transparent to-transparent"
                />
                <StatCard
                    label="Subscribed"
                    value={stats.subscribedCount}
                    trend="12.4%"
                    trendUp={true}
                    subValue="Paid plans"
                    icon={<Package className="w-5 h-5" />}
                    colorClass="text-[#7645d9]"
                    bgClass="bg-gradient-to-br from-[#7645d9]/20 via-transparent to-transparent"
                />
                <StatCard
                    label="Expiring"
                    value={stats.expiringSoon}
                    trend="12"
                    trendUp={false}
                    subValue="In 7 days"
                    icon={<Clock className="w-5 h-5" />}
                    colorClass="text-[#ffb237]"
                    bgClass="bg-gradient-to-br from-[#ffb237]/20 via-transparent to-transparent"
                />
            </div>
        </div>
    );
}
