import { PageHeader, PageLayout } from '@/components/shared';
import { DashboardSection } from '@/components/wallet/DashboardSection';
import { WalletManagementTabs } from '@/components/wallet/WalletManagementTabs';
import { fetchAccessManagementData, fetchWalletStats } from '@/lib/data/access-management';

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
import { fetchActivityLogsAction, fetchWalletsAction } from './actions';

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
  const [data, walletStats, initialWalletsData, initialActivityLogsRaw] = await Promise.all([
    fetchAccessManagementData().catch(() => ({ stats: { activeSubscriptions: 0, expiringSoon: 0, totalMRR: 0, totalMembers: 0 }, policies: [], permissionCount: 0, platformCount: 0 })),
    fetchWalletStats().catch(() => ({ total_users: 0, active_users: 0, inactive_users: 0, growth_rate: 0 })),
    fetchWalletsAction({ platform: 'all', status: 'all', sortBy: 'created_at', sortOrder: 'desc', search: '' }).catch(() => ({ wallets: [], pagination: { page: 1, limit: 20, total: 0, total_pages: 1, has_next_page: false, has_previous_page: false } })),
    fetchActivityLogsAction(undefined, 1, 10).catch(() => [])
  ]);

  // Map activity logs to frontend format
  const initialActivityLogs = (initialActivityLogsRaw || []).map((log: any) => ({
    id: log.id,
    type: 'wallet_created', // Fallback, client component maps properly but we need basic structure
    description: log.details?.description || 'Activity logged', // Simplified mapping for server-side prop
    timestamp: log.timestamp,
    performedBy: log.wallet_address || 'System',
    metadata: log.details
  }));

  // Transform stats for DashboardSection
  const dashboardStats = {
    totalWallets: walletStats?.total_users || 0,
    activeCount: walletStats?.active_users || 0,
    disabledCount: walletStats?.inactive_users || 0,
    subscribedCount: data?.stats?.activeSubscriptions || 0,
    expiringSoon: data?.stats?.expiringSoon || 0,
    mrr: `$${((data?.stats?.totalMRR || 0) / 1000).toFixed(1)}K`,
    members: formatNumber(data?.stats?.totalMembers || 0),
    growth: walletStats?.growth_rate ? `+${walletStats.growth_rate.toFixed(1)}%` : "+0%"
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
        <WalletManagementTabs
          initialData={initialWalletsData}
          initialActivityLogs={initialActivityLogs.map((log: any) => ({
            ...log,
            type: (log.metadata?.action?.includes('grant') ? 'permission_granted' :
              log.metadata?.action?.includes('revoke') ? 'permission_revoked' : 'wallet_created')
          }))} // Simplified mapping, let client refine if needed or duplicate logic here
        />
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
