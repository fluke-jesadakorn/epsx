/**
 * PolicySection Component
 * 
 * Client component that handles the main policy list display with:
 * - Search and filtering (client-side for instant response)
 * - Policy card display
 * - Delete confirmation modal
 * - Links to edit/detail pages
 */
'use client';

import { RefreshCw, Shield, Trash2 } from 'lucide-react';
import { useRouter, useSearchParams } from 'next/navigation';
import { useCallback, useEffect, useMemo, useState } from 'react';
import { toast } from 'sonner';

import { PolicyCard } from '@/components/access-control/policy-card';
import { PolicyFilters, PolicyTypeChips } from '@/components/access-control/policy-filters';
import {
  type AccessPolicy,
  type PolicyFilters as PolicyFiltersType,
  type PolicyType,
  DEFAULT_POLICY_FILTERS,
} from '@/components/access-control/types';
import { Button } from '@/components/ui/button';
import { Skeleton } from '@/components/ui/skeleton';
import { accessPolicyClient } from '@/lib/api/access-policy-client';
import { cn } from '@/lib/utils';

interface PolicySectionProps {
  initialPolicies: AccessPolicy[];
  className?: string;
}

/**
 * Filter and sort policies client-side
 */
function filterPolicies(policies: AccessPolicy[], filters: PolicyFiltersType): AccessPolicy[] {
  let result = [...policies];

  // Search filter
  if (filters.search) {
    const searchLower = filters.search.toLowerCase();
    result = result.filter(policy =>
      policy.name.toLowerCase().includes(searchLower) ||
      policy.description.toLowerCase().includes(searchLower) ||
      policy.permissions.some(p => p.toLowerCase().includes(searchLower))
    );
  }

  // Type filter
  if (filters.types !== 'all') {
    result = result.filter(policy => filters.types.includes(policy.type));
  }

  // Status filter
  if (filters.status !== 'all') {
    const isActive = filters.status === 'active';
    result = result.filter(policy => policy.isActive === isActive);
  }

  // Sort
  result.sort((a, b) => {
    let comparison = 0;

    switch (filters.sortBy) {
      case 'name':
        comparison = a.name.localeCompare(b.name);
        break;
      case 'members':
        comparison = a.memberCount - b.memberCount;
        break;
      case 'created_at':
        comparison = new Date(a.createdAt).getTime() - new Date(b.createdAt).getTime();
        break;
      case 'revenue':
        comparison = (a.revenue ?? 0) - (b.revenue ?? 0);
        break;
      case 'type':
        comparison = a.type.localeCompare(b.type);
        break;
    }

    return filters.sortOrder === 'asc' ? comparison : -comparison;
  });

  return result;
}

export function PolicySection({ initialPolicies, className }: PolicySectionProps) {
  const router = useRouter();
  const searchParams = useSearchParams();

  // Initialize filters from URL query params
  const getInitialFilters = (): PolicyFiltersType => {
    const typeParam = searchParams.get('type');
    if (typeParam && ['subscription', 'manual', 'web3_asset', 'dao', 'system'].includes(typeParam)) {
      return {
        ...DEFAULT_POLICY_FILTERS,
        types: [typeParam as PolicyType],
      };
    }
    return DEFAULT_POLICY_FILTERS;
  };

  // State
  const [policies, setPolicies] = useState<AccessPolicy[]>(initialPolicies);
  const [isRefreshing, setIsRefreshing] = useState(false);
  const [filters, setFilters] = useState<PolicyFiltersType>(getInitialFilters());

  // Update filters when URL changes
  useEffect(() => {
    const typeParam = searchParams.get('type');
    if (typeParam && ['subscription', 'manual', 'web3_asset', 'dao', 'system'].includes(typeParam)) {
      setFilters(prev => ({
        ...prev,
        types: [typeParam as PolicyType],
      }));
    }
  }, [searchParams]);

  // Delete confirmation
  const [deleteConfirm, setDeleteConfirm] = useState<{ policy: AccessPolicy } | null>(null);
  const [isDeleting, setIsDeleting] = useState(false);

  // Selection for bulk actions (future use)
  const [selectedPolicies, setSelectedPolicies] = useState<Set<string>>(new Set());

  // Filtered policies (computed client-side for instant filtering)
  const filteredPolicies = useMemo(() => {
    return filterPolicies(policies, filters);
  }, [policies, filters]);

  // Refresh handler
  const handleRefresh = useCallback(async () => {
    setIsRefreshing(true);
    try {
      const freshPolicies = await accessPolicyClient.getPolicies();
      setPolicies(freshPolicies);
      toast.success('Policies refreshed');
    } catch (error) {
      console.error('Failed to refresh policies:', error);
      toast.error('Failed to refresh policies');
    } finally {
      setIsRefreshing(false);
    }
  }, []);

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
    if (!deleteConfirm) {return;}

    setIsDeleting(true);
    try {
      await accessPolicyClient.deletePolicy(deleteConfirm.policy.id);
      toast.success(`"${deleteConfirm.policy.name}" deleted successfully`);
      setDeleteConfirm(null);
      // Remove from local state for instant feedback
      setPolicies(prev => prev.filter(p => p.id !== deleteConfirm.policy.id));
    } catch (error) {
      console.error('Failed to delete policy:', error);
      toast.error('Failed to delete policy');
    } finally {
      setIsDeleting(false);
    }
  };

  // Navigation handlers
  const handleCreatePlan = () => router.push('/subscriptions/plans/new');
  const handleCreateGroup = () => router.push('/wallet-management/groups/new');

  return (
    <div className={cn('space-y-4', className)}>
      {/* Section Header */}
      <div className="flex items-center justify-between">
        <div>
          <h2 className="text-xl font-bold text-foreground flex items-center gap-2">
            <Shield className="h-5 w-5 text-primary" />
            Access Policies
          </h2>
          <p className="text-sm text-muted-foreground mt-0.5">
            Manage subscription plans and permission groups
          </p>
        </div>
        <Button
          variant="ghost"
          size="sm"
          onClick={handleRefresh}
          disabled={isRefreshing}
          className="gap-2"
        >
          <RefreshCw className={cn('h-4 w-4', isRefreshing && 'animate-spin')} />
          Refresh
        </Button>
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
        onRefresh={handleRefresh}
        onCreatePlan={handleCreatePlan}
        onCreateGroup={handleCreateGroup}
        isLoading={isRefreshing}
      />

      {/* Policy List */}
      <div className="space-y-3">
        {isRefreshing && policies.length === 0 ? (
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
      {filteredPolicies.length > 0 && (
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
              Are you sure you want to delete <strong>&quot;{deleteConfirm.policy.name}&quot;</strong>?
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

export default PolicySection;
