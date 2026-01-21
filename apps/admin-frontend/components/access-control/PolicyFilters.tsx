/**
 * PolicyFilters Component
 * Search, type filter, and sort controls for the policy list
 */
'use client';

import { Search, RefreshCw, Plus, ChevronDown, X } from 'lucide-react';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select';
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
} from '@/components/ui/dropdown-menu';
import { cn } from '@/lib/utils';
import { 
  type PolicyFilters as PolicyFiltersType, 
  type PolicyType,
  POLICY_TYPE_CONFIG,
  DEFAULT_POLICY_FILTERS,
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

const POLICY_TYPES: { value: PolicyType | 'all'; label: string; icon: string }[] = [
  { value: 'all', label: 'All Types', icon: '📋' },
  { value: 'subscription', label: 'Subscription', icon: '💳' },
  { value: 'manual', label: 'Manual', icon: '👥' },
  { value: 'web3_asset', label: 'Web3 Asset', icon: '🔗' },
  { value: 'dao', label: 'DAO', icon: '🏛️' },
  { value: 'system', label: 'System', icon: '⚙️' },
];

const SORT_OPTIONS = [
  { value: 'name', label: 'Name' },
  { value: 'members', label: 'Members' },
  { value: 'created_at', label: 'Date Created' },
  { value: 'revenue', label: 'Revenue' },
  { value: 'type', label: 'Type' },
];

const STATUS_OPTIONS = [
  { value: 'all', label: 'All Status' },
  { value: 'active', label: 'Active' },
  { value: 'inactive', label: 'Inactive' },
];

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

  // Handle type selection for single type filter
  const handleTypeChange = (value: string) => {
    if (value === 'all') {
      updateFilter('types', 'all');
    } else {
      updateFilter('types', [value as PolicyType]);
    }
  };

  // Get current type value for select
  const currentTypeValue = filters.types === 'all' 
    ? 'all' 
    : Array.isArray(filters.types) && filters.types.length === 1 
      ? filters.types[0] 
      : 'all';

  return (
    <div className={cn(
      'rounded-2xl bg-card border border-border p-4',
      className
    )}>
      <div className="flex flex-col gap-4">
        {/* Search and Quick Actions Row */}
        <div className="flex flex-col sm:flex-row gap-4">
          {/* Search */}
          <div className="flex-1">
            <div className="relative">
              <Search className="absolute left-3 top-1/2 -translate-y-1/2 h-4 w-4 text-muted-foreground" />
              <Input
                placeholder="Search policies..."
                value={filters.search}
                onChange={(e) => updateFilter('search', e.target.value)}
                className="pl-10 h-10"
              />
              {filters.search && (
                <button
                  onClick={() => updateFilter('search', '')}
                  className="absolute right-3 top-1/2 -translate-y-1/2 text-muted-foreground hover:text-foreground"
                >
                  <X className="h-4 w-4" />
                </button>
              )}
            </div>
          </div>

          {/* Create Button */}
          {(onCreatePlan || onCreateGroup) && (
            <DropdownMenu>
              <DropdownMenuTrigger asChild>
                <Button className="gap-2 h-10">
                  <Plus className="h-4 w-4" />
                  <span>New Policy</span>
                  <ChevronDown className="h-4 w-4" />
                </Button>
              </DropdownMenuTrigger>
              <DropdownMenuContent align="end" className="w-56 rounded-xl p-1">
                {onCreatePlan && (
                  <DropdownMenuItem onClick={onCreatePlan} className="rounded-lg">
                    <span className="mr-2">💳</span>
                    <span>Subscription Plan</span>
                  </DropdownMenuItem>
                )}
                {onCreateGroup && (
                  <DropdownMenuItem onClick={onCreateGroup} className="rounded-lg">
                    <span className="mr-2">👥</span>
                    <span>Manual Group</span>
                  </DropdownMenuItem>
                )}
              </DropdownMenuContent>
            </DropdownMenu>
          )}
        </div>

        {/* Filters Row */}
        <div className="flex flex-wrap items-center gap-3">
          {/* Type Filter */}
          <Select value={currentTypeValue} onValueChange={handleTypeChange}>
            <SelectTrigger className="w-40 h-9">
              <SelectValue placeholder="Type" />
            </SelectTrigger>
            <SelectContent>
              {POLICY_TYPES.map(type => (
                <SelectItem key={type.value} value={type.value}>
                  <span className="flex items-center gap-2">
                    <span>{type.icon}</span>
                    <span>{type.label}</span>
                  </span>
                </SelectItem>
              ))}
            </SelectContent>
          </Select>

          {/* Status Filter */}
          <Select
            value={filters.status}
            onValueChange={(v) => updateFilter('status', v as PolicyFiltersType['status'])}
          >
            <SelectTrigger className="w-32 h-9">
              <SelectValue placeholder="Status" />
            </SelectTrigger>
            <SelectContent>
              {STATUS_OPTIONS.map(status => (
                <SelectItem key={status.value} value={status.value}>
                  {status.label}
                </SelectItem>
              ))}
            </SelectContent>
          </Select>

          {/* Sort By */}
          <Select
            value={filters.sortBy}
            onValueChange={(v) => updateFilter('sortBy', v as PolicyFiltersType['sortBy'])}
          >
            <SelectTrigger className="w-36 h-9">
              <SelectValue placeholder="Sort by" />
            </SelectTrigger>
            <SelectContent>
              {SORT_OPTIONS.map(sort => (
                <SelectItem key={sort.value} value={sort.value}>
                  {sort.label}
                </SelectItem>
              ))}
            </SelectContent>
          </Select>

          {/* Sort Order Toggle */}
          <Button
            variant="outline"
            size="sm"
            onClick={() => updateFilter('sortOrder', filters.sortOrder === 'asc' ? 'desc' : 'asc')}
            className="h-9 px-3"
          >
            {filters.sortOrder === 'asc' ? '↑ Asc' : '↓ Desc'}
          </Button>

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

/**
 * Compact type filter pills for quick filtering
 */
export function PolicyTypeChips({
  activeTypes,
  onChange,
  className,
}: {
  activeTypes: PolicyType[] | 'all';
  onChange: (types: PolicyType[] | 'all') => void;
  className?: string;
}) {
  const isAllActive = activeTypes === 'all';
  const activeTypesArray = isAllActive ? [] : activeTypes;

  const toggleType = (type: PolicyType) => {
    if (isAllActive) {
      // Switch from all to just this type
      onChange([type]);
    } else if (activeTypesArray.includes(type)) {
      // Remove this type
      const newTypes = activeTypesArray.filter(t => t !== type);
      onChange(newTypes.length === 0 ? 'all' : newTypes);
    } else {
      // Add this type
      onChange([...activeTypesArray, type]);
    }
  };

  return (
    <div className={cn('flex flex-wrap gap-2', className)}>
      <button
        onClick={() => onChange('all')}
        className={cn(
          'px-3 py-1.5 rounded-full text-sm font-medium transition-all duration-200',
          isAllActive
            ? 'bg-primary text-primary-foreground shadow-md'
            : 'bg-muted text-muted-foreground hover:bg-muted/80'
        )}
      >
        All
      </button>
      
      {Object.entries(POLICY_TYPE_CONFIG).map(([type, config]) => {
        const isActive = !isAllActive && activeTypesArray.includes(type as PolicyType);
        return (
          <button
            key={type}
            onClick={() => toggleType(type as PolicyType)}
            className={cn(
              'px-3 py-1.5 rounded-full text-sm font-medium transition-all duration-200 flex items-center gap-1.5',
              isActive
                ? 'bg-primary text-primary-foreground shadow-md'
                : 'bg-muted text-muted-foreground hover:bg-muted/80'
            )}
          >
            <span>{config.icon}</span>
            <span>{config.label}</span>
          </button>
        );
      })}
    </div>
  );
}
