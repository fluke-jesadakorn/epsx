'use client';

import { DashboardActivityStream } from '@/components/admin/dashboard-activity-stream';
import { DashboardBentoTools } from '@/components/admin/dashboard-bento-tools';
import { DashboardHudMetrics } from '@/components/admin/dashboard-hud-metrics';
import { DashboardPulseHeader } from '@/components/admin/dashboard-pulse-header';
import { PageLayout } from '@/components/shared';
import type { RecentWalletsData } from '@/hooks/use-analytics-data';
import { useDashboardData } from '@/hooks/use-dashboard-data';

interface DashboardClientProps {
  initialRecentWallets?: RecentWalletsData; // Kept for backwards compatibility with page.tsx
}

export default function DashboardClient({ initialRecentWallets: _initialRecentWallets }: DashboardClientProps) {
  // We don't necessarily need 'user' for the generic dashboard, the HUD is more systems-focused now.
  const { dashboardStats } = useDashboardData(true);

  return (
    <PageLayout maxWidth="full">
      <div className="max-w-[1600px] mx-auto w-full @container pb-12">
        {/* Command Center Pulse Header */}
        <DashboardPulseHeader stats={dashboardStats} />

        {/* HUD Metrics */}
        <DashboardHudMetrics stats={dashboardStats} />

        {/* Main Grid: Bento Tools + Activity Stream */}
        <div className="grid grid-cols-1 xl:grid-cols-4 gap-6">
          {/* Bento Tools take up 3 columns on extra large screens */}
          <div className="xl:col-span-3">
            <div className="mb-4 flex items-center justify-between">
              <h2 className="text-sm font-bold text-muted-foreground uppercase tracking-widest">
                Operational Modules
              </h2>
            </div>
            <DashboardBentoTools stats={dashboardStats} />
          </div>

          {/* Activity Stream takes up 1 column on extra large screens, acts as a sidebar feed */}
          <div className="xl:col-span-1 h-full">
            <div className="mb-4 flex items-center justify-between">
              <h2 className="text-sm font-bold text-muted-foreground uppercase tracking-widest hidden xl:block opacity-0">
                Global Event Stream
              </h2>
            </div>
            <DashboardActivityStream />
          </div>
        </div>
      </div>
    </PageLayout>
  );
}
