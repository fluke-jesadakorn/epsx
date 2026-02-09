import { PageHeader, PageLayout } from '@/components/shared';
import { DashboardSection } from '@/components/wallet/dashboard-section';
import { WalletTabsNavigation } from '@/components/wallet/wallet-tabs-navigation';
import { fetchAccessManagementData, fetchWalletStats } from '@/lib/data/access-management';

/**
 * Wallet Management Layout
 *
 * Wraps all wallet management pages with:
 * 1. Data fetching for shared stats
 * 2. Page Header
 * 3. Dashboard Stats
 * 4. Navigation Tabs
 */
export default async function WalletManagementLayout({
    children,
}: {
    children: React.ReactNode;
}) {
    const [data, walletStats] = await Promise.all([
        fetchAccessManagementData().catch(() => ({ stats: { activeSubscriptions: 0, expiringSoon: 0, totalMRR: 0, totalMembers: 0 }, policies: [], permissionCount: 0, platformCount: 0 })),
        fetchWalletStats().catch(() => ({ total_users: 0, active_users: 0, inactive_users: 0, growth_rate: 0 })),
    ]);

    // Transform stats for DashboardSection
    const dashboardStats = {
        totalWallets: walletStats?.total_users ?? 0,
        activeCount: walletStats?.active_users ?? 0,
        disabledCount: walletStats?.inactive_users ?? 0,
        subscribedCount: data?.stats?.activeSubscriptions ?? 0,
        expiringSoon: data?.stats?.expiringSoon ?? 0,
    };

    return (
        <PageLayout>
            {/* Page Header */}
            <PageHeader
                title="Wallet Management Hub"
                subtitle="Unified management for EPSX ecosystem wallets, permissions, and subscriptions"
                icon="Wallet"
                gradient="info"
            />

            {/* 1. Dashboard Stats (Full Width) */}
            <div className="mb-6 mt-2">
                <DashboardSection stats={dashboardStats} />
            </div>

            {/* 2. Navigation Tabs */}
            <WalletTabsNavigation />

            {/* 3. Page Content */}
            <div className="pb-12 animate-in fade-in-50 duration-500">
                {children}
            </div>
        </PageLayout>
    );
}

// Force dynamic rendering
export const dynamic = 'force-dynamic';
