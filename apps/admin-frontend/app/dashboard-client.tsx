'use client';

import { DashboardActionGrid } from '@/components/admin/dashboard-action-grid';
import { DashboardQuickStats } from '@/components/admin/dashboard-quick-stats';
import { DashboardStatCard } from '@/components/admin/dashboard-stat-card';
import { RecentWalletsPanel } from '@/components/admin/recent-wallets-panel';
import { PageHeader, PageLayout } from '@/components/shared';
import type { RecentWalletsData } from '@/hooks/use-analytics-data';
import { useDashboardData } from '@/hooks/use-dashboard-data';
import { useSharedAuth } from '@/shared/components/auth';

interface DashboardClientProps {
  initialRecentWallets?: RecentWalletsData;
}

export default function DashboardClient({ initialRecentWallets }: DashboardClientProps) {
  const { user } = useSharedAuth();
  const { dashboardStats } = useDashboardData(true);

  // eslint-disable-next-line @typescript-eslint/strict-boolean-expressions
  const subtitle = `Welcome back, ${user?.wallet_address ? `${user.wallet_address.slice(0, 6)}...${user.wallet_address.slice(-4)}` : 'Admin'}`;

  return (
    <PageLayout>
      {/* Page Header */}
      <PageHeader
        title="EPSX Admin Center"
        subtitle={subtitle}
        icon="Home"
        gradient="primary"
        centered
      />

      {/* System Status */}
      <div className="text-center text-sm text-muted-foreground flex items-center justify-center gap-2 -mt-4 mb-6">
        <span>{new Date().toLocaleDateString()}</span>
        <span>•</span>
        <span className="flex items-center gap-1">
          System Status: <span className="text-success font-medium">Operational</span>
          <span className="relative flex h-2 w-2">
            <span className="animate-ping absolute inline-flex h-full w-full rounded-full bg-success/40 opacity-75" />
            <span className="relative inline-flex rounded-full h-2 w-2 bg-success" />
          </span>
        </span>
      </div>

      <DashboardStatCard stats={dashboardStats} />

      {/* Recent Wallets Panel */}
      <div className="grid grid-cols-1 lg:grid-cols-3 gap-6">
        <div className="lg:col-span-2">
          <RecentWalletsPanel initialData={initialRecentWallets} />
        </div>
        <div className="space-y-4">
          <DashboardQuickStats stats={dashboardStats} />
        </div>
      </div>

      <DashboardActionGrid stats={dashboardStats} />
    </PageLayout>
  );
}
