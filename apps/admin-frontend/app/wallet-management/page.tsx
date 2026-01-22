import { PageHeader, PageLayout } from '@/components/shared';
import { DashboardSection } from '@/components/wallet/DashboardSection';
import { WalletManagementTabs } from '@/components/wallet/WalletManagementTabs';
import { fetchAccessManagementData } from '@/lib/data/access-management';

/**
 * Wallet Management Hub Page (Server Component)
 * 
 * Redesigned Layout:
 * - Top: Dashboard Stats (Full Width)
 * - Grid: 2x2 Layout (Responsive)
 *   1. Activity Log
 *   2. Wallet Management
 *   3. Access Policy
 *   4. Permission Registry
 */
export default async function WalletManagementPage() {
  const data = await fetchAccessManagementData();

  // Transform stats for DashboardSection
  const dashboardStats = {
    totalWallets: data.stats.totalPolicies, // Approx proxy or specific count
    activeCount: data.stats.activeSubscriptions + data.stats.activeGroups,
    disabledCount: 0, // Need accurate count if available, using 0 placeholder or calc
    subscribedCount: data.stats.activeSubscriptions,
    expiringSoon: data.stats.expiringSoon,
    mrr: `$${(data.stats.totalMRR / 1000).toFixed(1)}K`,
    members: formatNumber(data.stats.totalMembers),
    growth: "+12.4%" // Placeholder or calc
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

      {/* 2. Main Tabbed Layout */}
      <div className="pb-12">
        <WalletManagementTabs />
      </div>
    </PageLayout>
  );
}

function formatNumber(num: number): string {
  if (num >= 1000000) return `${(num / 1000000).toFixed(1)}M`;
  if (num >= 1000) return `${(num / 1000).toFixed(1)}K`;
  return num.toString();
}

// Force dynamic rendering
export const dynamic = 'force-dynamic';
