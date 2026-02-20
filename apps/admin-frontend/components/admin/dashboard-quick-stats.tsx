'use client';

import type { DashboardStats } from '@/hooks/use-dashboard-data';
import { Activity } from 'lucide-react';

interface DashboardQuickStatsProps {
    stats: DashboardStats;
}

/**
 *
 */
export function DashboardQuickStats({ stats }: DashboardQuickStatsProps) {
    return (
        <div className="bg-white dark:bg-slate-900 rounded-[32px] p-8 shadow-xl border border-gray-200 dark:border-slate-700 relative overflow-hidden group">
            <div className="absolute -right-6 -top-6 w-20 h-20 bg-[#1fc7d4]/10 rounded-full blur-2xl group-hover:bg-[#1fc7d4]/20 transition-colors" />
            <h3 className="text-xl font-bold bg-gradient-to-r from-[#1fc7d4] to-[#7645d9] bg-clip-text text-transparent mb-6 flex items-center gap-3">
                <Activity className="w-6 h-6 text-[#1fc7d4]" /> Quick Stats
            </h3>
            <div className="space-y-4">
                <div className="flex justify-between items-center p-3 rounded-2xl bg-white dark:bg-white/[0.04] border border-gray-200 dark:border-slate-700 transition-colors hover:bg-black/[0.05] dark:hover:bg-white/10">
                    <span className="text-sm font-bold text-muted-foreground uppercase tracking-widest">Active Wallets</span>
                    <span className="font-bold text-foreground text-lg">{stats.activeWallets}</span>
                </div>
                <div className="flex justify-between items-center p-3 rounded-2xl bg-white dark:bg-white/[0.04] border border-gray-200 dark:border-slate-700 transition-colors hover:bg-black/[0.05] dark:hover:bg-white/10">
                    <span className="text-sm font-bold text-muted-foreground uppercase tracking-widest">System Health</span>
                    <span className="font-bold text-[#31d0aa] text-lg">{stats.systemHealth}%</span>
                </div>
                <div className="flex justify-between items-center p-3 rounded-2xl bg-white dark:bg-white/[0.04] border border-gray-200 dark:border-slate-700 transition-colors hover:bg-black/[0.05] dark:hover:bg-white/10">
                    <span className="text-sm font-bold text-muted-foreground uppercase tracking-widest">Response Time</span>
                    <span className="font-bold text-[#ffb237] text-lg">{stats.avgResponseTime}</span>
                </div>
            </div>
        </div>
    );
}
