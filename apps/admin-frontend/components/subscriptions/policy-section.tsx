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

import { RefreshCw, Shield } from 'lucide-react';

import { PolicyFilters, PolicyTypeChips } from '@/components/access-control/policy-filters';
import type { AccessPolicy } from '@/components/access-control/types';
import { Button } from '@/components/ui/button';
import { usePolicySection } from '@/hooks/use-policy-section';
import { cn } from '@/lib/utils';
import { PolicyDeleteModal, PolicyList } from './policy/policy-components';

interface PolicySectionProps {
  initialPolicies: AccessPolicy[];
  className?: string;
}

export function PolicySection({ initialPolicies, className }: PolicySectionProps) {
  const {
    policies,
    filteredPolicies,
    isRefreshing,
    filters,
    setFilters,
    deleteConfirm,
    setDeleteConfirm,
    isDeleting,
    handleRefresh,
    handleSelectPolicy,
    handleDeletePolicy,
    handleCreatePlan,
    handleCreateGroup,
    selectedPolicies,
  } = usePolicySection(initialPolicies);

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
          onClick={() => { void handleRefresh(); }}
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
        onRefresh={() => { void handleRefresh(); }}
        onCreatePlan={handleCreatePlan}
        onCreateGroup={handleCreateGroup}
        isLoading={isRefreshing}
      />

      {/* Policy List */}
      <PolicyList
        isRefreshing={isRefreshing}
        policiesCount={policies.length}
        filteredPolicies={filteredPolicies}
        selectedPolicies={selectedPolicies}
        onSelect={handleSelectPolicy}
        onDeleteConfirm={(policy) => setDeleteConfirm({ policy })}
      />

      {/* Results count */}
      {filteredPolicies.length > 0 && (
        <div className="text-center text-sm text-muted-foreground">
          Showing {filteredPolicies.length} of {policies.length} policies
        </div>
      )}

      {/* Delete Confirmation Modal */}
      {deleteConfirm && (
        <PolicyDeleteModal
          policy={deleteConfirm.policy}
          onClose={() => setDeleteConfirm(null)}
          onDelete={handleDeletePolicy}
          isDeleting={isDeleting}
        />
      )}
    </div>
  );
}

export default PolicySection;
