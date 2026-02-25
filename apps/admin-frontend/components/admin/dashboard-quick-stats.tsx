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
        <div className="rounded-2xl border border-border/20 overflow-hidden bg-card shadow-xl">
            <div className="h-[3px] bg-gradient-to-r from-[#1fc7d4] to-[#7645d9]" />
            <div className="flex items-center justify-between px-4 py-3 border-b border-border/20">
                <h2 className="text-xs font-bold text-[#1fc7d4] uppercase tracking-[0.2em] flex items-center gap-3">
                    <Activity className="w-4 h-4" /> QUICK STATS
                </h2>
            </div>
            <div className="p-6 space-y-4">
                <div className="flex justify-between items-center p-3 rounded-xl bg-muted/30 border border-border/40 hover:bg-muted/50 transition-colors">
                    <span className="text-sm font-bold text-muted-foreground uppercase tracking-widest">Active Wallets</span>
                    <span className="font-bold text-foreground text-lg">{stats.activeWallets}</span>
                </div>
                <div className="flex justify-between items-center p-3 rounded-xl bg-muted/30 border border-border/40 hover:bg-muted/50 transition-colors">
                    <span className="text-sm font-bold text-muted-foreground uppercase tracking-widest">System Health</span>
                    <span className="font-bold text-[#31d0aa] text-lg">{stats.systemHealth}%</span>
                </div>
                <div className="flex justify-between items-center p-3 rounded-xl bg-muted/30 border border-border/40 hover:bg-muted/50 transition-colors">
                    <span className="text-sm font-bold text-muted-foreground uppercase tracking-widest">Response Time</span>
                    <span className="font-bold text-[#ffb237] text-lg">{stats.avgResponseTime}</span>
                </div>
            </div>
        </div>
    );
}
