/**
 * PolicyFilters Component
 * Search, type filter, and sort controls for the policy list
 */
'use client';

import { RefreshCw, X } from 'lucide-react';

import { Button } from '@/components/ui/button';
import { cn } from '@/lib/utils';

import { PolicyCreateButton } from './filters/create-button';
import { PolicyTypeChips } from './filters/policy-type-chips';
import { SearchBar } from './filters/search-bar';
import { SortControl } from './filters/sort-control';
import { StatusFilter } from './filters/status-filter';
import { TypeFilter } from './filters/type-filter';
import {
  DEFAULT_POLICY_FILTERS,
  type PolicyFilters as PolicyFiltersType,
} from './types';

interface PolicyFiltersProps {
  filters: PolicyFiltersType;
  onFiltersChange: (filters: PolicyFiltersType) => void;
  onRefresh: () => void;
  onCreatePlan?: () => void;
  onCreateGroup?: () => void;
  isLoading?: boolean;
  className?: string;
}

export function PolicyFilters({
  filters,
  onFiltersChange,
  onRefresh,
  onCreatePlan,
  onCreateGroup,
  isLoading = false,
  className,
}: PolicyFiltersProps) {
  const hasActiveFilters =
    filters.search !== '' ||
    filters.types !== 'all' ||
    filters.status !== 'all';

  const clearFilters = () => {
    onFiltersChange(DEFAULT_POLICY_FILTERS);
  };

  const updateFilter = <K extends keyof PolicyFiltersType>(
    key: K,
    value: PolicyFiltersType[K]
  ) => {
    onFiltersChange({ ...filters, [key]: value });
  };

  return (
    <div
      className={cn('rounded-2xl bg-card border border-border p-4', className)}
    >
      <div className="flex flex-col gap-4">
        {/* Search and Quick Actions Row */}
        <div className="flex flex-col sm:flex-row gap-4">
          <SearchBar
            value={filters.search}
            onChange={(val) => updateFilter('search', val)}
          />

          <PolicyCreateButton
            onCreatePlan={onCreatePlan}
            onCreateGroup={onCreateGroup}
          />
        </div>

        {/* Filters Row */}
        <div className="flex flex-wrap items-center gap-3">
          <TypeFilter
            value={filters.types}
            onChange={(val) => updateFilter('types', val)}
          />

          <StatusFilter
            value={filters.status}
            onChange={(val) => updateFilter('status', val)}
          />

          <SortControl
            sortBy={filters.sortBy}
            sortOrder={filters.sortOrder}
            onSortByChange={(val) => updateFilter('sortBy', val)}
            onSortOrderChange={(val) => updateFilter('sortOrder', val)}
          />

          {/* Spacer */}
          <div className="flex-1" />

          {/* Clear Filters */}
          {hasActiveFilters && (
            <Button
              variant="ghost"
              size="sm"
              onClick={clearFilters}
              className="h-9 text-muted-foreground hover:text-foreground"
            >
              <X className="h-4 w-4 mr-1" />
              Clear
            </Button>
          )}

          {/* Refresh */}
          <Button
            variant="outline"
            size="sm"
            onClick={onRefresh}
            disabled={isLoading}
            className="h-9 gap-2"
          >
            <RefreshCw className={cn('h-4 w-4', isLoading && 'animate-spin')} />
            Refresh
          </Button>
        </div>
      </div>
    </div>
  );
}

export { PolicyTypeChips };
