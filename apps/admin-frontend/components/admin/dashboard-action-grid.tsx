'use client';

import type { DashboardStats } from '@/hooks/use-dashboard-data';
import { Bell, FileText, Settings, Shield, Wallet, Zap } from 'lucide-react';

interface DashboardActionGridProps {
    stats: DashboardStats;
}

/**
 *
 */
export function DashboardActionGrid({ stats }: DashboardActionGridProps) {
    const actions = [
        {
            href: "/wallet-management",
            title: "Wallet Management",
            icon: <Wallet className="w-6 h-6" />,
            description: "Manage wallets and permissions",
            gradient: "from-[#1fc7d4] via-[#7645d9] to-[#1fc7d4]",
            textGradient: "text-[#1fc7d4]",
            bgGradient: "bg-[#1fc7d4]/5",
            status: `${stats.totalWallets} wallets`
        },
        {
            href: "/group-and-permission",
            title: "Permissions",
            icon: <Shield className="w-6 h-6" />,
            description: "Grant and manage access permissions",
            gradient: "from-[#31d0aa] via-[#1fc7d4] to-[#31d0aa]",
            textGradient: "text-[#31d0aa]",
            bgGradient: "bg-[#31d0aa]/5",
            status: `${stats.totalPermissions} permissions`
        },
        {
            href: "/audit-log",
            title: "Audit Log",
            icon: <FileText className="w-6 h-6" />,
            description: "Track admin actions and changes",
            gradient: "from-[#ed4b9e] via-[#7645d9] to-[#ed4b9e]",
            textGradient: "text-[#ed4b9e]",
            bgGradient: "bg-[#ed4b9e]/5",
            status: "View history"
        },
        {
            href: "/notifications",
            title: "Notifications",
            icon: <Bell className="w-6 h-6" />,
            description: "Send notifications and manage alerts",
            gradient: "from-[#7645d9] via-[#ed4b9e] to-[#7645d9]",
            textGradient: "text-[#7645d9]",
            bgGradient: "bg-[#7645d9]/5",
            status: `${stats.pendingNotifications} pending`
        },
        {
            href: "/settings",
            title: "Settings",
            icon: <Settings className="w-6 h-6" />,
            description: "Configure system and user settings",
            gradient: "from-slate-400 via-slate-500 to-slate-400",
            textGradient: "text-slate-400",
            bgGradient: "bg-slate-500/5",
            status: "System config"
        },
        {
            href: "/developer-portal",
            title: "Developer",
            icon: <Zap className="w-6 h-6" />,
            description: "API documentation and developer tools",
            gradient: "from-[#ffb237] via-[#1fc7d4] to-[#ffb237]",
            textGradient: "text-[#ffb237]",
            bgGradient: "bg-[#ffb237]/5",
            status: "API & Tools"
        }
    ];

    return (
        <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-4 sm:gap-6 mt-8">
            {actions.map((action) => (
                <a
                    key={action.href}
                    href={action.href}
                    className="block group active:scale-[0.98] transition-all"
                >
                    <div className={`relative overflow-hidden rounded-[32px] ${action.bgGradient} p-0.5 border border-gray-200 dark:border-slate-700 hover:border-[#1fc7d4]/30 transition-colors bg-white dark:bg-slate-900 backdrop-blur-xl`}>
                        <div className="relative p-6 sm:p-8">
                            {/* Floating decoration */}
                            <div className={`absolute -top-4 -right-4 w-16 h-16 bg-gradient-to-r ${action.gradient} rounded-full blur-2xl opacity-10 group-hover:opacity-20 transition-opacity`} />

                            <div className="relative z-10">
                                <div className="flex items-center gap-4 mb-4">
                                    <div className={`p-3 rounded-2xl bg-white dark:bg-white/[0.04] border border-gray-200 dark:border-slate-700 ${action.textGradient}`}>
                                        {action.icon}
                                    </div>
                                    <div className="flex-1 min-w-0">
                                        <h3 className={`text-xl sm:text-2xl font-bold bg-gradient-to-r ${action.gradient} bg-clip-text text-transparent truncate`}>
                                            {action.title}
                                        </h3>
                                        <div className="text-[10px] font-bold text-muted-foreground uppercase tracking-widest mt-0.5">
                                            {action.status}
                                        </div>
                                    </div>
                                </div>

                                <p className="text-sm text-muted-foreground mb-8 line-clamp-2 font-medium">
                                    {action.description}
                                </p>

                                <div className="flex items-center justify-between">
                                    <div className="px-5 py-2 bg-[#1fc7d4] text-white rounded-2xl text-xs font-bold shadow-lg shadow-cyan-500/10 group-hover:shadow-cyan-500/30 transition-all">
                                        Open Tool
                                    </div>
                                    <div className="w-8 h-8 rounded-full bg-white dark:bg-white/[0.04] border border-gray-200 dark:border-slate-700 flex items-center justify-center text-muted-foreground group-hover:text-[#1fc7d4] transition-colors">
                                        →
                                    </div>
                                </div>
                            </div>
                        </div>
                    </div>
                </a>
            ))}
        </div>
    );
}
