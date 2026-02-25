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
            textGradient: "text-[#1fc7d4]",
            status: `${stats.totalWallets} wallets`
        },
        {
            href: "/group-and-permission",
            title: "Permissions",
            icon: <Shield className="w-6 h-6" />,
            description: "Grant and manage access permissions",
            textGradient: "text-[#31d0aa]",
            status: `${stats.totalPermissions} permissions`
        },
        {
            href: "/audit-log",
            title: "Audit Log",
            icon: <FileText className="w-6 h-6" />,
            description: "Track admin actions and changes",
            textGradient: "text-[#ed4b9e]",
            status: "View history"
        },
        {
            href: "/notifications",
            title: "Notifications",
            icon: <Bell className="w-6 h-6" />,
            description: "Send notifications and manage alerts",
            textGradient: "text-[#7645d9]",
            status: `${stats.pendingNotifications} pending`
        },
        {
            href: "/settings",
            title: "Settings",
            icon: <Settings className="w-6 h-6" />,
            description: "Configure system and user settings",
            textGradient: "text-slate-400",
            status: "System config"
        },
        {
            href: "/developer-portal",
            title: "Developer",
            icon: <Zap className="w-6 h-6" />,
            description: "API documentation and developer tools",
            textGradient: "text-[#ffb237]",
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
                    <div className="relative overflow-hidden rounded-xl border border-border/20 bg-card hover:border-border/50 transition-colors">
                        <div className="p-6 sm:p-8">
                            <div className="flex items-center gap-4 mb-4">
                                <div className={`p-3 rounded-xl bg-muted/30 border border-border/40 ${action.textGradient}`}>
                                    {action.icon}
                                </div>
                                <div className="flex-1 min-w-0">
                                    <h3 className="text-xl sm:text-2xl font-bold text-foreground truncate">
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
                                <div className="px-5 py-2 bg-gradient-to-r from-[#7645d9] to-[#5a33b8] text-white rounded-xl text-xs font-bold shadow-lg transition-all">
                                    Open Tool
                                </div>
                                <div className="w-8 h-8 rounded-full bg-muted/30 border border-border/40 flex items-center justify-center text-muted-foreground group-hover:text-[#1fc7d4] transition-colors">
                                    →
                                </div>
                            </div>
                        </div>
                    </div>
                </a>
            ))}
        </div>
    );
}
