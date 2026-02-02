'use client';

import { PlanManagementView } from '@/components/access-control/PlanManagementView';
import { PageTabs, type TabItem } from '@/components/shared';
import { usePathname, useRouter, useSearchParams } from 'next/navigation';
import { useCallback } from 'react';
import { ActivityLogSection } from './ActivityLogSection';
import { WalletSection } from './WalletSection';
import { WalletActivityEvent, WalletData } from './types';

export interface WalletManagementTabsProps {
    initialData?: {
        wallets: WalletData[];
        pagination: {
            page: number;
            limit: number;
            total: number;
            total_pages: number;
        }
    };
    initialActivityLogs?: WalletActivityEvent[];
}

const WALLET_TABS: TabItem[] = [
    { id: 'wallets', label: 'Wallets', icon: 'Wallet' },
    { id: 'access-control', label: 'Access', icon: 'Shield', gradient: 'primary' },
    { id: 'activity', label: 'Activity', icon: 'Activity', gradient: 'purple' },
];

export function WalletManagementTabs({ initialData, initialActivityLogs }: WalletManagementTabsProps) {
    const router = useRouter();
    const pathname = usePathname();
    const searchParams = useSearchParams();

    // Get active tab from URL or default to 'wallets'
    const activeTab = searchParams.get('tab') || 'wallets';

    const handleTabChange = useCallback((value: string) => {
        const params = new URLSearchParams(searchParams);
        params.set('tab', value);
        router.push(`${pathname}?${params.toString()}`);
    }, [pathname, router, searchParams]);

    return (
        <div className="w-full space-y-8">
            <PageTabs
                tabs={WALLET_TABS}
                activeTab={activeTab}
                onTabChange={handleTabChange}
            />

            <div className="relative min-h-[500px]">
                {activeTab === 'wallets' && (
                    <div className="animate-in fade-in-50 duration-500">
                        <WalletSection initialData={initialData} />
                    </div>
                )}

                {activeTab === 'access-control' && (
                    <div className="animate-in fade-in-50 duration-500">
                        <PlanManagementView />
                    </div>
                )}

                {activeTab === 'activity' && (
                    <div className="animate-in fade-in-50 duration-500">
                        <ActivityLogSection initialEvents={initialActivityLogs} />
                    </div>
                )}
            </div>
        </div>
    );
}
