import { Download, Wallet } from 'lucide-react';
import { Suspense } from 'react';

import { PageHeader, PageLayout } from '@/components/shared';
import { PermissionAccordion, PolicySection } from '@/components/subscriptions';
import { CollapsibleSection } from '@/components/ui/CollapsibleSection';
import { CompactStatStrip } from '@/components/wallet/CompactStatStrip';
import { WalletHub } from '@/components/wallet/WalletHub';
import { fetchAccessManagementData } from '@/lib/data/access-management';

/**
 * Wallet Management Hub Page (Server Component)
 * 
 * Clean & Compact Redesign:
 * - Top: Compact Stat Strip
 * - Main: Wallet Hub with Integrated Toolbar (Table/Cards)
 * - Bottom: Collapsed Policy & Permission sections side-by-side
 */
export default async function WalletManagementPage() {
  const data = await fetchAccessManagementData();

  // Map data to stats format
  const statsData = {
    totalPolicies: data.stats.totalPolicies,
    activeCount: data.stats.activeSubscriptions + data.stats.activeGroups,
    mrr: `$${(data.stats.totalMRR / 1000).toFixed(1)}K`,
    members: formatNumber(data.stats.totalMembers),
    expiringSoon: data.stats.expiringSoon
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

      {/* 1. Compact Stats Strip */}
      <div className="mb-6 mt-2">
        <CompactStatStrip stats={statsData} />
      </div>

      {/* 2. Main Wallet Hub (Full Width) */}
      <div className="flex flex-col h-full mb-8">
        <div className="flex items-center justify-between mb-4 px-1">
          <div className="flex items-center gap-2">
            <div className="p-2 bg-blue-500/10 rounded-lg">
              <Wallet className="h-5 w-5 text-blue-400" />
            </div>
            <h2 className="text-xl font-bold text-foreground">Wallet Directory</h2>
          </div>

          <div className="hidden md:flex items-center gap-2">
            <button className="flex items-center gap-2 px-3 py-1.5 text-xs font-semibold text-muted-foreground hover:text-foreground transition-colors bg-muted/30 hover:bg-muted/50 rounded-lg border border-border/50">
              <Download className="h-3.5 w-3.5" />
              Export CSV
            </button>
          </div>
        </div>

        <div className="flex-1 min-h-[600px]">
          <WalletHub className="h-full border-0" />
        </div>
      </div>

      <div className="h-px bg-gradient-to-r from-transparent via-border to-transparent mb-8" />

      {/* 3. Grid Container for Policies & Permissions (Collapsed by default) */}
      <div className="grid grid-cols-1 lg:grid-cols-2 gap-6 pb-12">
        {/* Policy Section */}
        <CollapsibleSection title="Access Policies & Plans" defaultOpen={false} icon="FileText">
          <Suspense fallback={<PolicySectionSkeleton />}>
            <PolicySection initialPolicies={data.policies} />
          </Suspense>
        </CollapsibleSection>

        {/* Permission Registry */}
        <CollapsibleSection title="Permission Registry" defaultOpen={false} icon="Shield">
          <PermissionAccordion
            count={data.permissionCount}
            platformCount={data.platformCount}
          />
        </CollapsibleSection>
      </div>

    </PageLayout>
  );
}

// ============================================================================
// SUB-COMPONENTS
// ============================================================================

function PolicySectionSkeleton() {
  return (
    <div className="space-y-4 animate-pulse">
      <div className="h-8 bg-muted rounded-lg w-48" />
      <div className="h-12 bg-muted rounded-xl" />
      <div className="space-y-3">
        {Array.from({ length: 3 }).map((_, i) => (
          <div key={i} className="h-24 bg-muted rounded-2xl" />
        ))}
      </div>
    </div>
  );
}

function formatNumber(num: number): string {
  if (num >= 1000000) return `${(num / 1000000).toFixed(1)}M`;
  if (num >= 1000) return `${(num / 1000).toFixed(1)}K`;
  return num.toString();
}

// Force dynamic rendering
export const dynamic = 'force-dynamic';
