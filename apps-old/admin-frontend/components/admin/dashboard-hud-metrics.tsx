'use client';

import type { DashboardStats } from '@/hooks/use-dashboard-data';
import { Activity, Clock, Users, Wallet } from 'lucide-react';

interface DashboardHudMetricsProps {
    stats: DashboardStats;
}

export function DashboardHudMetrics({ stats }: DashboardHudMetricsProps) {
    const metrics = [
        {
            label: "Total Wallets",
            value: stats.totalWallets,
            subtext: `${stats.activeWallets} active today`,
            icon: <Wallet className="w-5 h-5 text-cyan-400" />,
            borderClass: "border-cyan-500/30",
            bgClass: "bg-cyan-500/5",
            valueClass: "text-cyan-400"
        },
        {
            label: "Sys Health",
            value: `${stats.systemHealth}%`,
            subtext: "Core services running",
            icon: <Activity className="w-5 h-5 text-[#31d0aa]" />,
            borderClass: "border-[#31d0aa]/30",
            bgClass: "bg-[#31d0aa]/5",
            valueClass: "text-[#31d0aa]"
        },
        {
            label: "Daily Conns.",
            value: stats.todayConnections,
            subtext: `${stats.totalPermissions} active perms`,
            icon: <Users className="w-5 h-5 text-[#ed4b9e]" />,
            borderClass: "border-[#ed4b9e]/30",
            bgClass: "bg-[#ed4b9e]/5",
            valueClass: "text-[#ed4b9e]"
        },
        {
            label: "Avg Resp.",
            value: stats.avgResponseTime,
            subtext: "API latency",
            icon: <Clock className="w-5 h-5 text-[#ffb237]" />,
            borderClass: "border-[#ffb237]/30",
            bgClass: "bg-[#ffb237]/5",
            valueClass: "text-[#ffb237]"
        }
    ];

    return (
        <div className="grid grid-cols-2 lg:grid-cols-4 gap-4 mb-6">
            {metrics.map((metric) => (
                <div
                    key={metric.label}
                    className={`relative overflow-hidden rounded-xl border ${metric.borderClass} ${metric.bgClass} p-5 backdrop-blur-md group hover:bg-opacity-10 transition-all duration-300`}
                >
                    <div className="flex items-center justify-between mb-4">
                        <span className="text-[10px] font-bold uppercase tracking-[0.2em] text-muted-foreground whitespace-nowrap">
                            {metric.label}
                        </span>
                        <div className="p-2 rounded-lg bg-background/50 border border-white/5 shadow-inner">
                            {metric.icon}
                        </div>
                    </div>

                    <div className={`text-3xl sm:text-4xl font-black font-mono tracking-tight ${metric.valueClass} drop-shadow-[0_0_8px_rgba(255,255,255,0.1)] mb-1`}>
                        {metric.value}
                    </div>

                    <div className="text-xs font-medium text-muted-foreground flex items-center gap-1.5 opacity-80">
                        <span className="w-1 h-1 rounded-full bg-current opacity-50 block" /> {metric.subtext}
                    </div>

                    {/* Subtle corner scans/accents for sci-fi feel */}
                    <div className={`absolute top-0 right-0 w-8 h-8 opacity-20 group-hover:opacity-40 transition-opacity bg-gradient-to-bl from-current to-transparent ${metric.valueClass}`} />
                </div>
            ))}
        </div>
    );
}
