'use client';

import { PlanManagementView } from '@/components/access-control/PlanManagementView';
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs';
import { Activity, Shield, Wallet } from 'lucide-react';
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
        <Tabs
            defaultValue="wallets"
            value={activeTab}
            onValueChange={handleTabChange}
            className="w-full space-y-8"
        >
            <div className="bg-slate-900/40 backdrop-blur-2xl p-1.5 rounded-[32px] border border-white/5 shadow-xl max-w-2xl mx-auto">
                <TabsList className="h-14 w-full justify-center gap-2 bg-transparent p-0">
                    <TabsTrigger
                        value="wallets"
                        className="data-[state=active]:bg-[#7645d9] data-[state=active]:text-white data-[state=active]:shadow-lg shadow-purple-500/20 rounded-[28px] px-8 py-3 font-bold text-sm text-muted-foreground hover:text-foreground hover:bg-white/5 transition-all active:scale-95 flex-1"
                    >
                        <Wallet className="mr-2 h-4 w-4" />
                        Wallets
                    </TabsTrigger>

                    <TabsTrigger
                        value="access-control"
                        className="data-[state=active]:bg-[#1fc7d4] data-[state=active]:text-white data-[state=active]:shadow-lg shadow-cyan-500/20 rounded-[28px] px-8 py-3 font-bold text-sm text-muted-foreground hover:text-foreground hover:bg-white/5 transition-all active:scale-95 flex-1"
                    >
                        <Shield className="mr-2 h-4 w-4" />
                        Access
                    </TabsTrigger>

                    <TabsTrigger
                        value="activity"
                        className="data-[state=active]:bg-[#ed4b9e] data-[state=active]:text-white data-[state=active]:shadow-lg shadow-pink-500/20 rounded-[28px] px-8 py-3 font-bold text-sm text-muted-foreground hover:text-foreground hover:bg-white/5 transition-all active:scale-95 flex-1"
                    >
                        <Activity className="mr-2 h-4 w-4" />
                        Activity
                    </TabsTrigger>
                </TabsList>
            </div>

            <div className="relative min-h-[500px]">
                <TabsContent value="wallets" className="m-0 border-none p-0 outline-none animate-in fade-in-50 duration-500">
                    <WalletSection initialData={initialData} />
                </TabsContent>

                <TabsContent value="access-control" className="m-0 border-none p-0 outline-none animate-in fade-in-50 duration-500">
                    <PlanManagementView />
                </TabsContent>

                <TabsContent value="activity" className="m-0 border-none p-0 outline-none animate-in fade-in-50 duration-500">
                    <ActivityLogSection initialEvents={initialActivityLogs} />
                </TabsContent>
            </div>
        </Tabs>
    );
}
