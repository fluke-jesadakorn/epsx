'use client';

import { cn } from '@/lib/utils';
import { Activity, Shield, Wallet } from 'lucide-react';
import Link from 'next/link';
import { usePathname } from 'next/navigation';

export function WalletTabsNavigation() {
    const pathname = usePathname();

    const tabs = [
        {
            id: 'wallets',
            label: 'Wallets',
            href: '/wallet-management/wallets',
            icon: Wallet,
            colorClass: 'data-[active=true]:bg-[#7645d9] shadow-purple-500/20'
        },
        {
            id: 'access',
            label: 'Access',
            href: '/wallet-management/access',
            icon: Shield,
            colorClass: 'data-[active=true]:bg-[#1fc7d4] shadow-cyan-500/20'
        },
        {
            id: 'activity',
            label: 'Activity',
            href: '/wallet-management/activity',
            icon: Activity,
            colorClass: 'data-[active=true]:bg-[#ed4b9e] shadow-pink-500/20'
        }
    ];

    return (
        <div className="bg-slate-900/40 backdrop-blur-2xl p-1.5 rounded-[32px] border border-white/5 shadow-xl max-w-2xl mx-auto mb-8">
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
                                "data-[active=true]:text-white data-[active=true]:shadow-lg",
                                tab.colorClass
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
