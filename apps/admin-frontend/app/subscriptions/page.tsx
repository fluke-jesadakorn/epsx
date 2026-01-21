import { Suspense } from 'react';
import { ShieldCheck, Plus, Users, Wallet, Clock } from 'lucide-react';
import Link from 'next/link';

import { PageLayout, PageHeader, PageSkeleton } from '@/components/shared';
import { PolicySection, PromotionSection, PermissionAccordion } from '@/components/subscriptions';
import { fetchAccessManagementData } from '@/lib/data/access-management';

/**
 * Access Management Page (Server Component)
 * 
 * Unified single page for managing:
 * - Access Policies (Plans + Groups) - Primary concern
 * - Promotions - Secondary, compact display
 * - Permission Registry - Tertiary, collapsed accordion
 */
export default async function AccessManagementPage() {
  // Fetch all data server-side in parallel
  const data = await fetchAccessManagementData();

  return (
    <PageLayout>
      {/* Page Header */}
      <PageHeader
        title="Access Management"
        subtitle="Unified control for policies, promotions, and permissions"
        icon="ShieldCheck"
        gradient="primary"
      />

      {/* Stats Dashboard */}
      <div className="grid grid-cols-2 sm:grid-cols-3 lg:grid-cols-6 gap-3 sm:gap-4">
        <StatCard
          label="Policies"
          value={data.stats.totalPolicies}
          icon="📊"
          color="primary"
        />
        <StatCard
          label="Active"
          value={data.stats.activeSubscriptions + data.stats.activeGroups}
          icon="✅"
          color="success"
        />
        <StatCard
          label="MRR"
          value={`$${(data.stats.totalMRR / 1000).toFixed(1)}K`}
          icon="💰"
          color="primary"
        />
        <StatCard
          label="Members"
          value={formatNumber(data.stats.totalMembers)}
          icon="👥"
          color="secondary"
        />
        <StatCard
          label="Expiring"
          value={data.stats.expiringSoon}
          icon="⏰"
          color="warning"
          highlight={data.stats.expiringSoon > 0}
        />
        <StatCard
          label="Promotions"
          value={data.promotions.filter(p => p.isActive).length}
          icon="🎁"
          color="success"
        />
      </div>

      {/* Quick Actions */}
      <div className="grid grid-cols-2 sm:grid-cols-4 gap-3">
        <QuickActionCard
          href="/subscriptions/plans/new"
          icon={<Plus className="h-4 w-4" />}
          label="New Plan"
          color="primary"
        />
        <QuickActionCard
          href="/subscriptions/manual-access/create-group"
          icon={<Users className="h-4 w-4" />}
          label="New Group"
          color="secondary"
        />
        <QuickActionCard
          href="/subscriptions/manual-access/assign"
          icon={<Wallet className="h-4 w-4" />}
          label="Assign Access"
          color="info"
        />
        <QuickActionCard
          href="/subscriptions/manual-access/expiring"
          icon={<Clock className="h-4 w-4" />}
          label="Expiring Soon"
          color="warning"
          badge={data.stats.expiringSoon > 0 ? data.stats.expiringSoon : undefined}
        />
      </div>

      {/* Main Content: Policy Section */}
      <Suspense fallback={<PolicySectionSkeleton />}>
        <PolicySection initialPolicies={data.policies} />
      </Suspense>

      {/* Promotions Section (Compact) */}
      <PromotionSection initialPromotions={data.promotions} />

      {/* Permission Registry (Collapsed Accordion) */}
      <PermissionAccordion
        count={data.permissionCount}
        platformCount={data.platformCount}
      />
    </PageLayout>
  );
}

// ============================================================================
// SUB-COMPONENTS
// ============================================================================

interface StatCardProps {
  label: string;
  value: string | number;
  icon: string;
  color: 'primary' | 'secondary' | 'success' | 'warning' | 'info';
  highlight?: boolean;
}

function StatCard({ label, value, icon, color, highlight }: StatCardProps) {
  const colorClasses = {
    primary: 'border-primary/20 text-primary',
    secondary: 'border-secondary/20 text-secondary',
    success: 'border-success/20 text-success',
    warning: 'border-warning/20 text-warning',
    info: 'border-info/20 text-info',
  };

  return (
    <div className={`
      bg-card rounded-xl border p-3 sm:p-4 transition-all duration-200
      ${colorClasses[color]}
      ${highlight ? 'ring-2 ring-warning/50 animate-pulse' : ''}
    `}>
      <div className="flex items-center justify-between mb-1">
        <span className="text-lg">{icon}</span>
        <span className="text-[10px] uppercase text-muted-foreground font-medium tracking-wider">
          {label}
        </span>
      </div>
      <div className="text-xl sm:text-2xl font-bold">{value}</div>
    </div>
  );
}

interface QuickActionCardProps {
  href: string;
  icon: React.ReactNode;
  label: string;
  color: 'primary' | 'secondary' | 'success' | 'warning' | 'info';
  badge?: number;
}

function QuickActionCard({ href, icon, label, color, badge }: QuickActionCardProps) {
  const colorClasses = {
    primary: 'bg-primary/10 hover:bg-primary/20 text-primary border-primary/20',
    secondary: 'bg-secondary/10 hover:bg-secondary/20 text-secondary border-secondary/20',
    success: 'bg-success/10 hover:bg-success/20 text-success border-success/20',
    warning: 'bg-warning/10 hover:bg-warning/20 text-warning border-warning/20',
    info: 'bg-info/10 hover:bg-info/20 text-info border-info/20',
  };

  return (
    <Link
      href={href}
      className={`
        relative flex items-center gap-2 px-3 py-2.5 rounded-xl border
        font-medium text-sm transition-all duration-200
        hover:scale-[1.02] active:scale-[0.98]
        ${colorClasses[color]}
      `}
    >
      {icon}
      <span>{label}</span>
      {badge !== undefined && badge > 0 && (
        <span className="absolute -top-1.5 -right-1.5 min-w-5 h-5 flex items-center justify-center text-[10px] font-bold bg-warning text-warning-foreground rounded-full px-1">
          {badge}
        </span>
      )}
    </Link>
  );
}

function PolicySectionSkeleton() {
  return (
    <div className="space-y-4 animate-pulse">
      <div className="h-8 bg-muted rounded-lg w-48" />
      <div className="h-12 bg-muted rounded-xl" />
      <div className="space-y-3">
        {Array.from({ length: 5 }).map((_, i) => (
          <div key={i} className="h-32 bg-muted rounded-2xl" />
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

// Force dynamic rendering to always get fresh data
export const dynamic = 'force-dynamic';
