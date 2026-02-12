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
            {/* Sub-navigation */}
            <div className="flex items-center gap-1 border-b border-white/10 shrink-0">
                {tabs.map((tab) => {
                    const isActive = pathname.startsWith(tab.href);
                    const Icon = tab.icon;
                    return (
                        <Link
                            key={tab.id}
                            href={tab.href}
                            className={cn(
                                "flex items-center gap-2 px-4 py-2.5 text-sm font-medium transition-all border-b-2 -mb-px",
                                isActive
                                    ? "text-[#1fc7d4] border-[#1fc7d4]"
                                    : "text-muted-foreground border-transparent hover:text-white hover:border-white/20"
                            )}
                        >
                            <Icon className="w-4 h-4" />
                            {tab.label}
                        </Link>
                    );
                })}
            </div>

            {/* CONTENT */}
            <div className="flex-1 min-h-0">
                {children}
            </div>
        </div>
    );
}
