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
            className="w-full space-y-6"
        >
            <div className="border-b border-border/40 pb-px">
                <TabsList className="h-12 w-full justify-start gap-2 bg-transparent p-0">
                    <TabsTrigger
                        value="wallets"
                        className="data-[state=active]:bg-transparent data-[state=active]:shadow-none data-[state=active]:border-b-2 data-[state=active]:border-primary data-[state=active]:text-primary rounded-none border-b-2 border-transparent px-4 py-2 font-medium text-muted-foreground hover:text-foreground transition-none"
                    >
                        <Wallet className="mr-2 h-4 w-4" />
                        Wallets
                    </TabsTrigger>

                    <TabsTrigger
                        value="access-control"
                        className="data-[state=active]:bg-transparent data-[state=active]:shadow-none data-[state=active]:border-b-2 data-[state=active]:border-primary data-[state=active]:text-primary rounded-none border-b-2 border-transparent px-4 py-2 font-medium text-muted-foreground hover:text-foreground transition-none"
                    >
                        <Shield className="mr-2 h-4 w-4" />
                        Access Control
                    </TabsTrigger>

                    <TabsTrigger
                        value="activity"
                        className="data-[state=active]:bg-transparent data-[state=active]:shadow-none data-[state=active]:border-b-2 data-[state=active]:border-primary data-[state=active]:text-primary rounded-none border-b-2 border-transparent px-4 py-2 font-medium text-muted-foreground hover:text-foreground transition-none"
                    >
                        <Activity className="mr-2 h-4 w-4" />
                        Activity Log
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
