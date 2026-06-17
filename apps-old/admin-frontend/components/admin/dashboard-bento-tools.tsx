'use client';

import type { DashboardStats } from '@/hooks/use-dashboard-data';
import { Bell, Database, FileText, Settings, Shield, Wallet } from 'lucide-react';
import Link from 'next/link';

interface DashboardBentoToolsProps {
    stats: DashboardStats;
}

const COL_1 = 'col-span-1';
const COL_2 = 'col-span-1 lg:col-span-2';
const ROW_1 = 'row-span-1';

export function DashboardBentoTools({ stats }: DashboardBentoToolsProps) {
    const tools = [
        {
            href: "/wallet-management",
            title: "Wallet Database",
            icon: <Wallet className="w-6 h-6" />,
            description: "Deep inspect connected wallets, view connection history, and force disconnect active sessions.",
            gradient: "from-cyan-500/20 to-blue-500/20",
            iconColor: "text-cyan-400",
            status: `${stats.totalWallets} Registered`,
            colSpan: COL_2,
            rowSpan: ROW_1
        },
        {
            href: "/group-and-permission",
            title: "Security & Perms",
            icon: <Shield className="w-6 h-6" />,
            description: "Critical access control. Manage robust permissions across all modules.",
            gradient: "from-purple-500/20 to-fuchsia-500/20",
            iconColor: "text-purple-400",
            status: `${stats.totalPermissions} Active Nodes`,
            colSpan: COL_1,
            rowSpan: "row-span-2"
        },
        {
            href: "/audit-log",
            title: "Global Audit Log",
            icon: <FileText className="w-6 h-6" />,
            description: "Immutable history of all administrative cross-system actions.",
            gradient: "from-pink-500/20 to-rose-500/20",
            iconColor: "text-pink-400",
            status: "Monitoring Active",
            colSpan: COL_1,
            rowSpan: ROW_1
        },
        {
            href: "/notifications",
            title: "Broadcast Hub",
            icon: <Bell className="w-6 h-6" />,
            description: "Push critical system alerts and global updates.",
            gradient: "from-amber-500/20 to-orange-500/20",
            iconColor: "text-amber-400",
            status: `${stats.pendingNotifications} Pending Broadcasts`,
            colSpan: COL_1,
            rowSpan: ROW_1
        },
        {
            href: "/developer-portal",
            title: "Dev Infrastructure",
            icon: <Database className="w-6 h-6" />,
            description: "Manage system API keys, global integrations and webhooks.",
            gradient: "from-emerald-500/20 to-teal-500/20",
            iconColor: "text-emerald-400",
            status: "SYS_OK",
            colSpan: COL_2,
            rowSpan: ROW_1
        },
        {
            href: "/settings",
            title: "Settings",
            icon: <Settings className="w-6 h-6" />,
            description: "Core Platform config.",
            gradient: "from-slate-500/20 to-gray-500/20",
            iconColor: "text-slate-400",
            status: "V2.4.0",
            colSpan: COL_1,
            rowSpan: ROW_1
        }
    ];

    return (
        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4 auto-rows-[220px]">
            {tools.map((tool) => (
                <Link
                    key={tool.href}
                    href={tool.href}
                    className={`group relative overflow-hidden rounded-2xl border border-border/20 bg-card/60 backdrop-blur-md transition-all duration-300 hover:scale-[1.02] hover:shadow-[0_0_30px_-5px_var(--tw-shadow-color)] hover:border-white/20 ${tool.colSpan} ${tool.rowSpan} flex flex-col`}
                    style={{ '--tw-shadow-color': 'rgba(255, 255, 255, 0.1)' } as React.CSSProperties}
                >
                    {/* Background glow gradient */}
                    <div className={`absolute inset-0 bg-gradient-to-br opacity-50 transition-opacity group-hover:opacity-100 ${tool.gradient}`} />
                    <div className={`absolute -right-20 -top-20 w-64 h-64 bg-current opacity-10 blur-3xl rounded-full ${tool.iconColor}`} />

                    <div className="relative p-6 flex flex-col h-full z-10">
                        <div className="flex justify-between items-start mb-4">
                            <div className={`p-3 rounded-xl bg-background/50 border border-white/5 backdrop-blur-md shadow-inner ${tool.iconColor}`}>
                                {tool.icon}
                            </div>
                            <div className={`text-[10px] font-mono uppercase tracking-widest px-3 py-1 bg-background/50 border border-white/5 rounded-full ${tool.iconColor}`}>
                                {tool.status}
                            </div>
                        </div>

                        <div className="mt-auto">
                            <h3 className="text-xl sm:text-2xl font-black tracking-tight text-foreground mb-2 group-hover:text-white transition-colors">
                                {tool.title}
                            </h3>
                            <p className="text-sm font-medium text-muted-foreground line-clamp-3 leading-relaxed group-hover:text-muted-foreground/90">
                                {tool.description}
                            </p>
                        </div>
                    </div>
                </Link>
            ))}
        </div>
    );
}
