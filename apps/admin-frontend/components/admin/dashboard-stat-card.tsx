'use client';

import type { DashboardStats } from '@/hooks/use-dashboard-data';
import { Activity, Clock, Wallet, Zap } from 'lucide-react';

interface DashboardStatCardProps {
    stats: DashboardStats;
}

/**
 *
 */
export function DashboardStatCard({ stats }: DashboardStatCardProps) {
    return (
        <div className="grid grid-cols-2 lg:grid-cols-4 gap-4 sm:gap-6">
            {/* Total Wallets */}
            <div className="bg-white dark:bg-slate-900 rounded-[32px] p-6 shadow-xl border border-gray-200 dark:border-slate-700 relative overflow-hidden group">
                <div className="absolute -right-6 -top-6 w-24 h-24 bg-[#1fc7d4]/10 rounded-full blur-2xl group-hover:bg-[#1fc7d4]/20 transition-colors" />
                <div className="relative z-10">
                    <div className="flex items-center justify-between mb-4">
                        <div className="p-3 bg-gradient-to-br from-[#1fc7d4]/10 to-[#7645d9]/10 rounded-[18px] text-[#1fc7d4] border border-[#1fc7d4]/20">
                            <Wallet className="w-6 h-6" />
                        </div>
                        <span className="text-xs font-bold uppercase tracking-widest text-muted-foreground">Total</span>
                    </div>
                    <div className="space-y-1">
                        <div className="text-3xl sm:text-4xl font-bold bg-gradient-to-r from-[#1fc7d4] to-[#7645d9] bg-clip-text text-transparent">{stats.totalWallets}</div>
                        <div className="text-sm font-bold text-foreground/80 uppercase tracking-wide">Wallets</div>
                        <div className="text-xs text-muted-foreground font-medium">{stats.activeWallets} active today</div>
                    </div>
                </div>
            </div>

            {/* System Health */}
            <div className="bg-white dark:bg-slate-900 rounded-[32px] p-6 shadow-xl border border-gray-200 dark:border-slate-700 relative overflow-hidden group">
                <div className="absolute -right-6 -top-6 w-24 h-24 bg-[#31d0aa]/10 rounded-full blur-2xl group-hover:bg-[#31d0aa]/20 transition-colors" />
                <div className="relative z-10">
                    <div className="flex items-center justify-between mb-4">
                        <div className="p-3 bg-gradient-to-br from-[#31d0aa]/10 to-[#1fc7d4]/10 rounded-[18px] text-[#31d0aa] border border-[#31d0aa]/20">
                            <Activity className="w-6 h-6" />
                        </div>
                        <span className="text-xs font-bold uppercase tracking-widest text-muted-foreground">Health</span>
                    </div>
                    <div className="space-y-1">
                        <div className="text-3xl sm:text-4xl font-bold text-[#31d0aa]">{stats.systemHealth}%</div>
                        <div className="text-sm font-bold text-foreground/80 uppercase tracking-wide">System Health</div>
                        <div className="text-xs text-muted-foreground font-medium">{stats.systemUptime} uptime</div>
                    </div>
                </div>
            </div>

            {/* Today's Activity */}
            <div className="bg-white dark:bg-slate-900 rounded-[32px] p-6 shadow-xl border border-gray-200 dark:border-slate-700 relative overflow-hidden group">
                <div className="absolute -right-6 -top-6 w-24 h-24 bg-[#ed4b9e]/10 rounded-full blur-2xl group-hover:bg-[#ed4b9e]/20 transition-colors" />
                <div className="relative z-10">
                    <div className="flex items-center justify-between mb-4">
                        <div className="p-3 bg-gradient-to-br from-[#ed4b9e]/10 to-[#7645d9]/10 rounded-[18px] text-[#ed4b9e] border border-[#ed4b9e]/20">
                            <Zap className="w-6 h-6" />
                        </div>
                        <span className="text-xs font-bold uppercase tracking-widest text-muted-foreground">Today</span>
                    </div>
                    <div className="space-y-1">
                        <div className="text-3xl sm:text-4xl font-bold text-[#ed4b9e]">{stats.todayConnections}</div>
                        <div className="text-sm font-bold text-foreground/80 uppercase tracking-wide">Connections</div>
                        <div className="text-xs text-muted-foreground font-medium">{stats.totalPermissions} permissions</div>
                    </div>
                </div>
            </div>

            {/* Performance */}
            <div className="bg-white dark:bg-slate-900 rounded-[32px] p-6 shadow-xl border border-gray-200 dark:border-slate-700 relative overflow-hidden group">
                <div className="absolute -right-6 -top-6 w-24 h-24 bg-[#ffb237]/10 rounded-full blur-2xl group-hover:bg-[#ffb237]/20 transition-colors" />
                <div className="relative z-10">
                    <div className="flex items-center justify-between mb-4">
                        <div className="p-3 bg-gradient-to-br from-[#ffb237]/10 to-[#ffb237]/10 rounded-[18px] text-[#ffb237] border border-[#ffb237]/20">
                            <Clock className="w-6 h-6" />
                        </div>
                        <span className="text-xs font-bold uppercase tracking-widest text-muted-foreground">Speed</span>
                    </div>
                    <div className="space-y-1">
                        <div className="text-3xl sm:text-4xl font-bold text-[#ffb237]">{stats.avgResponseTime}</div>
                        <div className="text-sm font-bold text-foreground/80 uppercase tracking-wide">Avg Response</div>
                        <div className="text-xs text-muted-foreground font-medium">{stats.pendingNotifications} pending</div>
                    </div>
                </div>
            </div>
        </div>
    );
}
