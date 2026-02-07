'use client';

import { DashboardActionGrid } from '@/components/admin/DashboardActionGrid';
import { DashboardQuickStats } from '@/components/admin/DashboardQuickStats';
import { DashboardStatCard } from '@/components/admin/DashboardStatCard';
import { RecentWalletsData, RecentWalletsPanel } from '@/components/admin/RecentWalletsPanel';
import { PageAuthRequired, PageHeader, PageLayout, PageSkeleton } from '@/components/shared';
import { useDashboardData } from '@/hooks/useDashboardData';
import { useSharedAuth } from '@/shared/components/auth/Provider';
import { Shield } from 'lucide-react';

/**
 * Admin Dashboard Client Component
 */
interface DashboardClientProps {
  initialRecentWallets?: RecentWalletsData;
}

/**
 *
 */
export default function DashboardClient({ initialRecentWallets }: DashboardClientProps) {
  const { user, isAuthenticated, isLoading } = useSharedAuth();
  const { dashboardStats, accessError } = useDashboardData(isAuthenticated);

  // Show loading state
  if (isLoading) {
    return <PageSkeleton showHeader stats={4} rows={6} />;
  }

  // Show authentication required if not authenticated
  if (!isAuthenticated) {
    return <PageAuthRequired />;
  }

  // Show access error if backend rejected the request
  if (accessError) {
    return (
      <PageLayout>
        <div className="text-center max-w-md mx-auto py-16">
          <div className="w-20 h-20 bg-destructive/10 rounded-3xl flex items-center justify-center mx-auto mb-6">
            <Shield className="w-10 h-10 text-destructive" />
          </div>
          <h1 className="text-2xl sm:text-3xl font-bold text-foreground mb-4">
            Access Denied
          </h1>
          <p className="text-muted-foreground mb-8">
            {accessError}
          </p>
          <a
            href="/auth"
            className="inline-flex items-center gap-2 px-6 py-3 bg-destructive text-destructive-foreground rounded-2xl font-semibold hover:opacity-90 transition-opacity"
          >
            Try Again
          </a>
        </div>
      </PageLayout>
    );
  }

  return (
    <PageLayout>
      {/* Page Header */}
      <PageHeader
        title="EPSX Admin Center"
        subtitle={`Welcome back, ${user?.wallet_address ? `${user.wallet_address.slice(0, 6)}...${user.wallet_address.slice(-4)}` : 'Admin'}`}
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
            <span className="animate-ping absolute inline-flex h-full w-full rounded-full bg-success/40 opacity-75"></span>
            <span className="relative inline-flex rounded-full h-2 w-2 bg-success"></span>
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
