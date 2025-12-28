import { GlobalAuthGuard } from '@/components/auth/GlobalAuthGuard';
import { getCurrentUser } from '@/lib/server-actions';
import { getDebugSessionInfo } from '@/lib/server-actions-user';
import type { ReactNode } from 'react';
import { DeveloperSidebarClient } from './DeveloperSidebarClient';

export const dynamic = 'force-dynamic';

interface DeveloperLayoutProps {
    children: ReactNode;
}

export default async function DeveloperLayout({ children }: DeveloperLayoutProps) {
    const user = await getCurrentUser();
    const debugInfo = !user ? await getDebugSessionInfo() : null;

    if (!user) {
        return (
            <div className="min-h-screen bg-gradient-to-br from-slate-50 via-blue-50 to-indigo-50 dark:from-gray-900 dark:via-gray-900 dark:to-indigo-900 flex items-center justify-center">
                <div className="container mx-auto p-6">
                    <GlobalAuthGuard title="Developer API" debugInfo={debugInfo} />
                </div>
            </div>
        );
    }

    return (
        <div className="flex min-h-screen bg-gradient-to-br from-slate-50 via-blue-50 to-indigo-50 dark:from-gray-900 dark:via-gray-900 dark:to-indigo-900">
            {/* Sidebar */}
            <DeveloperSidebarClient />

            {/* Main Content Area */}
            <main className="flex-1 overflow-y-auto min-h-screen">
                {/* pt-28 on mobile for main nav (56px) + dev header (48px), normal padding on desktop */}
                <div className="pt-28 lg:pt-8 px-4 pb-6 lg:px-8 lg:pb-8">
                    {children}
                </div>
            </main>
        </div>
    );
}
