'use client';

import { memo } from 'react';
import { Search, Filter } from 'lucide-react';
import { Input } from '@/components/ui/input';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select';

interface PermissionFiltersProps {
  searchTerm: string;
  onSearchChange: (value: string) => void;
  filter: 'all' | 'active' | 'expired' | 'revoked';
  onFilterChange: (value: 'all' | 'active' | 'expired' | 'revoked') => void;
  totalCount: number;
  filteredCount: number;
}

function PermissionFilters({
  searchTerm,
  onSearchChange,
  filter,
  onFilterChange,
  totalCount,
  filteredCount
}: PermissionFiltersProps) {
  return (
    <div className="flex flex-col sm:flex-row gap-4 mb-6">
      {/* Search Input */}
      <div className="relative flex-1">
        <Search className="absolute left-3 top-1/2 transform -translate-y-1/2 h-4 w-4 text-muted-foreground" />
        <Input
          placeholder="Search permissions, resources, actions, or reasons..."
          value={searchTerm}
          onChange={(e) => onSearchChange(e.target.value)}
          className="pl-10 w-full"
        />
      </div>

      {/* Filter Dropdown */}
      <div className="flex items-center gap-2 min-w-[200px]">
        <Filter className="h-4 w-4 text-muted-foreground" />
        <Select value={filter} onValueChange={onFilterChange}>
          <SelectTrigger>
            <SelectValue placeholder="Filter permissions" />
          </SelectTrigger>
          <SelectContent>
            <SelectItem value="all">All Permissions</SelectItem>
            <SelectItem value="active">Active Only</SelectItem>
            <SelectItem value="expired">Expired Only</SelectItem>
            <SelectItem value="revoked">Revoked Only</SelectItem>
          </SelectContent>
        </Select>
      </div>

      {/* Results Counter */}
      <div className="flex items-center text-sm text-muted-foreground whitespace-nowrap">
        Showing {filteredCount} of {totalCount} permissions
      </div>
    </div>
  );
}

export default memo(PermissionFilters);