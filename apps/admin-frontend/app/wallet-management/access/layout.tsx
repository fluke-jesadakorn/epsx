'use client';

import { cn } from '@/lib/utils';
import { Key, Package } from 'lucide-react';
import Link from 'next/link';
import { usePathname } from 'next/navigation';

export default function AccessLayout({
    children,
}: {
    children: React.ReactNode;
}) {
    const pathname = usePathname();

    const tabs = [
        {
            id: 'permissions',
            label: 'Permissions',
            href: '/wallet-management/access/permissions',
            icon: Key,
            activeColor: 'bg-emerald-500'
        },
        {
            id: 'plans',
            label: 'Plans',
            href: '/wallet-management/access/plans',
            icon: Package,
            activeColor: 'bg-[#1fc7d4]'
        }
    ];

    return (
        <div className="flex flex-col gap-6 h-full">
            {/* TOP HEADER / TABS */}
            <div className="flex items-center justify-between shrink-0">
                <div className="flex bg-slate-900/50 border border-white/5 p-1 rounded-xl">
                    {tabs.map((tab) => {
                        const isActive = pathname === tab.href;
                        const Icon = tab.icon;
                        return (
                            <Link
                                key={tab.id}
                                href={tab.href}
                                className={cn(
                                    "flex items-center rounded-lg px-4 py-1.5 text-sm font-medium transition-all",
                                    isActive
                                        ? cn("text-white shadow-lg", tab.activeColor)
                                        : "text-muted-foreground hover:text-white hover:bg-white/5"
                                )}
                            >
                                <Icon className="w-4 h-4 mr-2" />
                                {tab.label}
                            </Link>
                        );
                    })}
                </div>
            </div>

            {/* CONTENT */}
            <div className="flex-1 min-h-0">
                {children}
            </div>
        </div>
    );
}
