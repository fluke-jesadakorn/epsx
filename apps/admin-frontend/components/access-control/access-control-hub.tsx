/**
 * AccessControlHub Component
 * Main container for the unified Access Control page
 * Combines policies (plans + groups) into a single view
 */
'use client';

import { Clock, Plus, Shield, Trash2, Users } from 'lucide-react';
import Link from 'next/link';
import { useRouter, useSearchParams } from 'next/navigation';
import { useCallback, useEffect, useMemo, useState } from 'react';
import { toast } from 'sonner';

import { Button } from '@/components/ui/button';
import { Skeleton } from '@/components/ui/skeleton';
import { accessPolicyClient } from '@/lib/api/access-policy-client';
import { cn } from '@/lib/utils';
import { logger } from '@/shared/utils/logger';

import { PolicyCard } from './policy-card';
import { PolicyFilters, PolicyTypeChips } from './policy-filters';
import { PolicyStatsBar } from './policy-stats-bar';
import {
  type AccessPolicy,
  type PolicyFilters as PolicyFiltersType,
  type PolicyStats,
  type PolicyType,
  DEFAULT_POLICY_FILTERS,
  DEFAULT_POLICY_STATS,
  POLICY_TYPE_CONFIG,
} from './types';

interface AccessControlHubProps {
  className?: string;
}

export function AccessControlHub({ className }: AccessControlHubProps) {
  const router = useRouter();
  const searchParams = useSearchParams();

  // State
  const [policies, setPolicies] = useState<AccessPolicy[]>([]);
  const [stats, setStats] = useState<PolicyStats>(DEFAULT_POLICY_STATS);
  const [isLoading, setIsLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  // Filters - initialized from URL params
  const [filters, setFilters] = useState<PolicyFiltersType>(() => {
    const typeParam = searchParams.get('type');
    const types: PolicyType[] | 'all' = typeParam
      ? typeParam.split(',').filter(t => t in POLICY_TYPE_CONFIG) as PolicyType[]
      : 'all';

    return {
      ...DEFAULT_POLICY_FILTERS,
      types: types.length === 0 ? 'all' : types,
    };
  });

  // Delete confirmation
  const [deleteConfirm, setDeleteConfirm] = useState<{ policy: AccessPolicy } | null>(null);
  const [isDeleting, setIsDeleting] = useState(false);

  // Selection for bulk actions
  const [selectedPolicies, setSelectedPolicies] = useState<Set<string>>(new Set());

  // Load data
  const loadData = useCallback(async () => {
    setIsLoading(true);
    setError(null);

    try {
      const [policiesData, statsData] = await Promise.all([
        accessPolicyClient.getPolicies(),
        accessPolicyClient.getStats(),
      ]);

      setPolicies(policiesData);
      setStats(statsData);
    } catch (err: unknown) {
      logger.error('Failed to load access control data:', err instanceof Error ? err.message : String(err));
      setError(err instanceof Error ? err.message : 'Failed to load data');
      toast.error('Failed to load access control data');
    } finally {
      setIsLoading(false);
    }
  }, []);

  useEffect(() => {
    loadData();
  }, [loadData]);

  // Filter and sort policies
  const filteredPolicies = useMemo(() => {
    return accessPolicyClient.filterPolicies(policies, filters);
  }, [policies, filters]);

  // Group policies by type for optional grouped view
  const policiesByType = useMemo(() => {
    const grouped: Record<PolicyType, AccessPolicy[]> = {
      subscription: [],
      manual: [],
      web3_asset: [],
      dao: [],
      system: [],
    };

    filteredPolicies.forEach(policy => {
      grouped[policy.type].push(policy);
    });

    return grouped;
  }, [filteredPolicies]);

  // Selection handlers
  const handleSelectPolicy = (policyId: string, selected: boolean) => {
    setSelectedPolicies(prev => {
      const next = new Set(prev);
      if (selected) {
        next.add(policyId);
      } else {
        next.delete(policyId);
      }
      return next;
    });
  };

  // Delete handler
  const handleDeletePolicy = async () => {
    if (!deleteConfirm) { return; }

    setIsDeleting(true);
    try {
      await accessPolicyClient.deletePolicy(deleteConfirm.policy.id);
      toast.success(`"${deleteConfirm.policy.name}" deleted successfully`);
      setDeleteConfirm(null);
      await loadData();
    } catch (err: unknown) {
      logger.error('Failed to delete policy:', err instanceof Error ? err.message : String(err));
      toast.error('Failed to delete policy');
    } finally {
      setIsDeleting(false);
    }
  };

  // Navigation handlers
  const handleCreatePlan = () => router.push('/subscriptions/plans/new');
  const handleCreateGroup = () => router.push('/subscriptions/manual-access/create-group');

  return (
    <div className={cn('space-y-6', className)}>
      {/* Stats Dashboard */}
      <PolicyStatsBar stats={stats} isLoading={isLoading && stats.totalPolicies === 0} />

      {/* Quick Actions */}
      <div className="grid grid-cols-1 sm:grid-cols-3 gap-4">
        {/* Create Plan */}
        <Link href="/subscriptions/plans/new" className="block group h-full">
          <div className="relative overflow-hidden rounded-2xl bg-gradient-to-r from-blue-400/20 via-indigo-400/20 to-blue-400/20 p-0.5 hover:scale-[1.02] transition-all duration-300 h-full">
            <div className="relative bg-card rounded-2xl h-full flex flex-col">
              <div className="absolute top-4 right-4 w-4 h-4 bg-gradient-to-r from-blue-400 to-indigo-500 rounded-full blur-sm opacity-60" />
              <div className="p-4 sm:p-6 flex-1 flex flex-col">
                <div className="flex items-center gap-2 mb-2">
                  <Plus className="w-5 h-5 text-blue-500" />
                  <h3 className="text-lg font-bold bg-gradient-to-r from-blue-400 to-indigo-500 bg-clip-text text-transparent">
                    New Plan
                  </h3>
                </div>
                <p className="text-sm text-muted-foreground mb-3">
                  Create a subscription plan
                </p>
                <div className="flex items-center justify-between mt-auto">
                  <div className="px-3 py-1 bg-gradient-to-r from-blue-400 to-indigo-500 text-white rounded-full text-xs font-medium">
                    💳 Plan
                  </div>
                  <div className="text-muted-foreground group-hover:translate-x-1 transition-transform duration-200">→</div>
                </div>
              </div>
            </div>
          </div>
        </Link>

        {/* Create Group */}
        <Link href="/subscriptions/manual-access/create-group" className="block group h-full">
          <div className="relative overflow-hidden rounded-2xl bg-gradient-to-r from-amber-400/20 via-orange-400/20 to-amber-400/20 p-0.5 hover:scale-[1.02] transition-all duration-300 h-full">
            <div className="relative bg-card rounded-2xl h-full flex flex-col">
              <div className="absolute top-4 right-4 w-4 h-4 bg-gradient-to-r from-amber-400 to-orange-500 rounded-full blur-sm opacity-60" />
              <div className="p-4 sm:p-6 flex-1 flex flex-col">
                <div className="flex items-center gap-2 mb-2">
                  <Users className="w-5 h-5 text-amber-500" />
                  <h3 className="text-lg font-bold bg-gradient-to-r from-amber-400 to-orange-500 bg-clip-text text-transparent">
                    New Group
                  </h3>
                </div>
                <p className="text-sm text-muted-foreground mb-3">
                  Create a manual access group
                </p>
                <div className="flex items-center justify-between mt-auto">
                  <div className="px-3 py-1 bg-gradient-to-r from-amber-400 to-orange-500 text-white rounded-full text-xs font-medium">
                    👥 Group
                  </div>
                  <div className="text-muted-foreground group-hover:translate-x-1 transition-transform duration-200">→</div>
                </div>
              </div>
            </div>
          </div>
        </Link>

        {/* View Expiring */}
        <Link href="/subscriptions/manual-access/expiring" className="block group h-full">
          <div className="relative overflow-hidden rounded-2xl bg-gradient-to-r from-orange-400/20 via-pink-400/20 to-orange-400/20 p-0.5 hover:scale-[1.02] transition-all duration-300 h-full">
            <div className="relative bg-card rounded-2xl h-full flex flex-col">
              <div className="absolute top-4 right-4 w-4 h-4 bg-gradient-to-r from-orange-400 to-pink-500 rounded-full blur-sm opacity-60" />
              <div className="p-4 sm:p-6 flex-1 flex flex-col">
                <div className="flex items-center gap-2 mb-2">
                  <Clock className="w-5 h-5 text-orange-500" />
                  <h3 className="text-lg font-bold bg-gradient-to-r from-orange-400 to-pink-500 bg-clip-text text-transparent">
                    Expiring Soon
                  </h3>
                </div>
                <p className="text-sm text-muted-foreground mb-3">
                  {stats.expiringSoon > 0 ? `${stats.expiringSoon} expiring within 7 days` : 'View expiring access'}
                </p>
                <div className="flex items-center justify-between mt-auto">
                  <div className="px-3 py-1 bg-gradient-to-r from-orange-400 to-pink-500 text-white rounded-full text-xs font-medium">
                    {stats.expiringSoon > 0 ? stats.expiringSoon : 'View'}
                  </div>
                  <div className="text-muted-foreground group-hover:translate-x-1 transition-transform duration-200">→</div>
                </div>
              </div>
            </div>
          </div>
        </Link>
      </div>

      {/* Type Quick Filters */}
      <PolicyTypeChips
        activeTypes={filters.types}
        onChange={(types) => setFilters(prev => ({ ...prev, types }))}
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
      {error && (
        <div className="p-4 rounded-xl bg-destructive/10 border border-destructive/20 text-destructive">
          ⚠️ {error}
        </div>
      )}

      {/* Policy List */}
      <div className="space-y-3">
        {isLoading ? (
          // Loading skeletons
          Array.from({ length: 5 }).map((_, i) => (
            <div key={i} className="rounded-2xl bg-card border border-border p-6 animate-pulse">
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
            <p className="text-sm mt-1">Try adjusting your filters or create a new policy</p>
          </div>
        ) : (
          // Policy cards
          filteredPolicies.map((policy) => (
            <PolicyCard
              key={policy.id}
              policy={policy}
              isSelected={selectedPolicies.has(policy.id)}
              onSelect={(selected) => handleSelectPolicy(policy.id, selected)}
              onDelete={policy.isSystemGroup ? undefined : () => setDeleteConfirm({ policy })}
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

      {/* Delete Confirmation Modal */}
      {deleteConfirm && (
        <div className="fixed inset-0 bg-black/70 flex items-center justify-center z-50">
          <div className="bg-card rounded-2xl p-6 max-w-md w-full mx-4 shadow-2xl border border-border">
            <div className="flex items-center gap-3 mb-4">
              <div className="p-3 bg-destructive/10 rounded-full">
                <Trash2 className="w-6 h-6 text-destructive" />
              </div>
              <h3 className="text-lg font-semibold text-foreground">
                Delete Policy
              </h3>
            </div>
            <p className="text-muted-foreground mb-6">
              Are you sure you want to delete <strong>"{deleteConfirm.policy.name}"</strong>?
              This action cannot be undone.
              {deleteConfirm.policy.memberCount > 0 && (
                <span className="block mt-2 text-amber-600 dark:text-amber-400">
                  ⚠️ This policy has {deleteConfirm.policy.memberCount} members who will lose access.
                </span>
              )}
            </p>
            <div className="flex gap-3">
              <Button
                variant="outline"
                onClick={() => setDeleteConfirm(null)}
                className="flex-1"
              >
                Cancel
              </Button>
              <Button
                variant="destructive"
                onClick={handleDeletePolicy}
                disabled={isDeleting}
                className="flex-1"
              >
                {isDeleting ? 'Deleting...' : 'Delete'}
              </Button>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}

export default AccessControlHub;
