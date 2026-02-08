'use client';

import { cn } from '@/lib/utils';
import { Bell, Send } from 'lucide-react';
import Link from 'next/link';
import { usePathname } from 'next/navigation';

export function NotificationTabsNavigation() {
    const pathname = usePathname();

    const tabs = [
        {
            id: 'manage',
            label: 'Overview',
            href: '/notifications/manage',
            icon: Bell,
            activeGradient: 'bg-gradient-to-r from-[#1fc7d4] to-[#7645d9]'
        },
        {
            id: 'create',
            label: 'Send Signal',
            href: '/notifications/create',
            icon: Send,
            activeGradient: 'bg-gradient-to-r from-[#ffb237] to-[#ed4b9e]'
        }
    ];

    return (
        <div className="bg-slate-900/40 backdrop-blur-2xl p-1.5 rounded-[32px] border border-white/5 shadow-xl max-w-xl mx-auto mb-8">
            <div className="flex h-14 w-full justify-center gap-2 bg-transparent p-0">
                {tabs.map((tab) => {
                    const isActive = pathname === tab.href;
                    const Icon = tab.icon;
                    return (
                        <Link
                            key={tab.id}
                            href={tab.href}
                            data-active={isActive}
                            className={cn(
                                "flex items-center justify-center flex-1 rounded-[28px] px-8 py-3 font-bold text-sm transition-all active:scale-95",
                                "text-muted-foreground hover:text-foreground hover:bg-white/5",
                                "data-[active=true]:text-white data-[active=true]:shadow-lg data-[active=true]:shadow-primary/20",
                                isActive ? tab.activeGradient : ""
                            )}
                        >
                            <Icon className="mr-2 h-4 w-4" />
                            {tab.label}
                        </Link>
                    );
                })}
            </div>
        </div>
    );
}
