/**
 * AccessControlHub Component
 * Main container for the unified Access Control page
 * Combines policies (plans + groups) into a single view
 */
'use client';

import { Shield } from 'lucide-react';
import { useRouter } from 'next/navigation';

import { Skeleton } from '@/components/ui/skeleton';
import { cn } from '@/lib/utils';

import { DeletePolicyModal } from './hub/delete-policy-modal';
import { useAccessControlHub } from './hub/hooks';
import { QuickActions } from './hub/quick-actions';
import { PolicyCard } from './policy-card';
import { PolicyFilters, PolicyTypeChips } from './policy-filters';
import { PolicyStatsBar } from './policy-stats-bar';

interface AccessControlHubProps {
  className?: string;
}

export function AccessControlHub({ className }: AccessControlHubProps) {
  const router = useRouter();
  const {
    policies,
    stats,
    isLoading,
    error,
    filters,
    setFilters,
    filteredPolicies,
    selectedPolicies,
    handleSelectPolicy,
    deleteConfirm,
    setDeleteConfirm,
    isDeleting,
    handleDeletePolicy,
    loadData,
  } = useAccessControlHub();

  // Navigation handlers
  const handleCreatePlan = () => router.push('/subscriptions/plans/new');
  const handleCreateGroup = () =>
    router.push('/subscriptions/manual-access/create-group');

  return (
    <div className={cn('space-y-6', className)}>
      {/* Stats Dashboard */}
      <PolicyStatsBar
        stats={stats}
        isLoading={isLoading && stats.totalPolicies === 0}
      />

      <QuickActions stats={stats} />

      {/* Type Quick Filters */}
      <PolicyTypeChips
        activeTypes={filters.types}
        onChange={(types) => setFilters((prev) => ({ ...prev, types }))}
      />

      {/* Filters */}
      <PolicyFilters
        filters={filters}
        onFiltersChange={setFilters}
        onRefresh={loadData}
        onCreatePlan={handleCreatePlan}
        onCreateGroup={handleCreateGroup}
        isLoading={isLoading}
      />

      {/* Error */}
      {(error ?? '').length > 0 && (
        <div className="p-4 rounded-xl bg-destructive/10 border border-destructive/20 text-destructive">
          ⚠️ {error}
        </div>
      )}

      {/* Policy List */}
      <div className="space-y-3">
        {isLoading ? (
          // Loading skeletons
          Array.from({ length: 5 }).map((_, i) => (
            <div
              key={i}
              className="rounded-2xl bg-card border border-border p-6 animate-pulse"
            >
              <div className="flex items-center gap-4">
                <Skeleton className="h-12 w-12 rounded-xl" />
                <div className="space-y-2 flex-1">
                  <Skeleton className="h-4 w-40" />
                  <Skeleton className="h-3 w-32" />
                </div>
                <Skeleton className="h-9 w-20" />
              </div>
            </div>
          ))
        ) : filteredPolicies.length === 0 ? (
          // Empty state
          <div className="text-center py-12 text-muted-foreground">
            <Shield className="h-12 w-12 mx-auto mb-4 opacity-30" />
            <p className="font-medium">No policies found</p>
            <p className="text-sm mt-1">
              Try adjusting your filters or create a new policy
            </p>
          </div>
        ) : (
          // Policy cards
          filteredPolicies.map((policy) => (
            <PolicyCard
              key={policy.id}
              policy={policy}
              isSelected={selectedPolicies.has(policy.id)}
              onSelect={(selected) => handleSelectPolicy(policy.id, selected)}
              onDelete={
                policy.isSystemGroup === true
                  ? undefined
                  : () => setDeleteConfirm({ policy })
              }
            />
          ))
        )}
      </div>

      {/* Results count */}
      {!isLoading && filteredPolicies.length > 0 && (
        <div className="text-center text-sm text-muted-foreground">
          Showing {filteredPolicies.length} of {policies.length} policies
        </div>
      )}

      <DeletePolicyModal
        policy={deleteConfirm?.policy ?? null}
        isOpen={deleteConfirm !== null}
        onClose={() => setDeleteConfirm(null)}
        onConfirm={handleDeletePolicy}
        isDeleting={isDeleting}
      />
    </div>
  );
}

export default AccessControlHub;
