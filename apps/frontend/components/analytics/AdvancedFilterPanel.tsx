'use client';

import React, { useState } from 'react';
import { Card, CardContent, CardHeader, CardTitle } from '@epsx/ui';
import { Button } from '@epsx/ui';
import { Badge } from '@epsx/ui';
import { Input } from '@epsx/ui';
import { Label } from '@epsx/ui';
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@epsx/ui';
import {
  Collapsible,
  CollapsibleContent,
  CollapsibleTrigger,
} from '@/components/ui/collapsible';
import {
  Filter,
  ChevronDown,
  ChevronUp,
  RotateCcw,
  Zap,
  Globe,
  Building,
  TrendingUp,
  DollarSign,
  ArrowUpDown,
} from 'lucide-react';
import type { AnalyticsFilters, FilterOptions } from '@/hooks/useAnalyticsFilters';

interface AdvancedFilterPanelProps {
  filters: AnalyticsFilters;
  options: FilterOptions;
  loading: boolean;
  onFilterChange: <K extends keyof AnalyticsFilters>(key: K, value: AnalyticsFilters[K]) => void;
  onFiltersChange: (filters: Partial<AnalyticsFilters>) => void;
  onResetFilters: () => void;
  onApplyPreset: (presetName: string) => void;
  className?: string;
}

export function AdvancedFilterPanel({
  filters,
  options,
  loading,
  onFilterChange,
  onFiltersChange,
  onResetFilters,
  onApplyPreset,
  className,
}: AdvancedFilterPanelProps) {
  const [isExpanded, setIsExpanded] = useState(false);

  // Get current filter counts for active indicators
  const getActiveFilterCount = () => {
    let count = 0;
    if (filters.country) count++;
    if (filters.sector) count++;
    if (filters.min_eps) count++;
    if (filters.min_growth) count++;
    if (filters.sort_by !== 'market_cap') count++;
    return count;
  };

  const activeFilterCount = getActiveFilterCount();

  // Filter presets
  const presets = [
    {
      id: 'topPerformers',
      name: 'Top Performers',
      description: 'High growth (>5%) stocks',
      icon: TrendingUp,
      badge: 'Popular',
    },
    {
      id: 'usStocks',
      name: 'US Market',
      description: 'American companies only',
      icon: Globe,
      badge: null,
    },
    {
      id: 'techSector',
      name: 'Technology',
      description: 'Tech sector focus',
      icon: Building,
      badge: null,
    },
    {
      id: 'highEps',
      name: 'High EPS',
      description: 'EPS > $1.00',
      icon: DollarSign,
      badge: null,
    },
  ];

  // Sort options
  const sortOptions = [
    { value: 'ranking_score', label: 'Best Ranked', icon: '🏆' },
    { value: 'current_eps', label: 'Highest EPS', icon: '📈' },
    { value: 'qoq_growth', label: 'Best Growth', icon: '🚀' },
    { value: 'market_cap', label: 'Market Cap', icon: '💰' },
    { value: 'symbol', label: 'Symbol A-Z', icon: '🔤' },
  ];

  return (
    <Card className={`border-2 ${className}`}>
      <Collapsible open={isExpanded} onOpenChange={setIsExpanded}>
        <div className="relative">
          <CollapsibleTrigger className="hover:bg-muted/50 transition-colors p-6 w-full">
            <div className="flex items-center justify-between">
              <div className="flex items-center gap-3">
                <div className="flex items-center gap-2">
                  <Filter className="h-5 w-5 text-primary" />
                  <CardTitle className="text-lg">Advanced Filters</CardTitle>
                </div>
                {activeFilterCount > 0 && (
                  <Badge variant="secondary" className="bg-primary/10 text-primary">
                    {activeFilterCount} active
                  </Badge>
                )}
              </div>
              <div className="flex items-center gap-2">
                <div className="w-8 h-8"></div> {/* Placeholder for reset button space */}
                {isExpanded ? (
                  <ChevronUp className="h-4 w-4" />
                ) : (
                  <ChevronDown className="h-4 w-4" />
                )}
              </div>
            </div>
          </CollapsibleTrigger>
          {/* Reset button positioned absolutely to avoid nesting */}
          <div
            onClick={(e) => {
              e.stopPropagation();
              onResetFilters();
            }}
            className="absolute top-6 right-12 inline-flex items-center justify-center h-8 w-8 px-2 rounded-md border border-input bg-background hover:bg-accent hover:text-accent-foreground disabled:pointer-events-none disabled:opacity-50 text-sm cursor-pointer"
            style={{ opacity: loading ? 0.5 : 1, pointerEvents: loading ? 'none' : 'auto' }}
          >
            <RotateCcw className="h-3 w-3" />
          </div>
        </div>

        <CollapsibleContent>
          <CardContent className="space-y-6">
            {/* Filter Presets */}
            <div className="space-y-3">
              <Label className="text-sm font-semibold flex items-center gap-2">
                <Zap className="h-4 w-4" />
                Quick Filters
              </Label>
              <div className="grid grid-cols-2 lg:grid-cols-4 gap-2">
                {presets.map((preset) => {
                  const Icon = preset.icon;
                  return (
                    <Button
                      key={preset.id}
                      variant="outline"
                      size="sm"
                      onClick={() => onApplyPreset(preset.id)}
                      disabled={loading}
                      className="h-auto p-3 flex flex-col items-start gap-1 hover:bg-primary/5"
                    >
                      <div className="flex items-center justify-between w-full">
                        <div className="flex items-center gap-2">
                          <Icon className="h-4 w-4" />
                          <span className="font-medium text-xs">{preset.name}</span>
                        </div>
                        {preset.badge && (
                          <Badge variant="secondary" className="text-xs py-0 px-1">
                            {preset.badge}
                          </Badge>
                        )}
                      </div>
                      <p className="text-xs text-muted-foreground text-left">
                        {preset.description}
                      </p>
                    </Button>
                  );
                })}
              </div>
            </div>

            {/* Main Filters Grid */}
            <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
              {/* Country Filter */}
              <div className="space-y-2">
                <Label className="text-sm font-medium flex items-center gap-2">
                  <Globe className="h-4 w-4" />
                  Country
                </Label>
                <Select
                  value={filters.country || 'All Countries'}
                  onValueChange={(value) =>
                    onFilterChange('country', value === 'All Countries' ? undefined : value)
                  }
                  disabled={loading}
                >
                  <SelectTrigger>
                    <SelectValue />
                  </SelectTrigger>
                  <SelectContent>
                    {options.countries.map((country) => (
                      <SelectItem key={country} value={country}>
                        {country}
                      </SelectItem>
                    ))}
                  </SelectContent>
                </Select>
              </div>

              {/* Sector Filter */}
              <div className="space-y-2">
                <Label className="text-sm font-medium flex items-center gap-2">
                  <Building className="h-4 w-4" />
                  Sector
                </Label>
                <Select
                  value={filters.sector || 'All Sectors'}
                  onValueChange={(value) =>
                    onFilterChange('sector', value === 'All Sectors' ? undefined : value)
                  }
                  disabled={loading}
                >
                  <SelectTrigger>
                    <SelectValue />
                  </SelectTrigger>
                  <SelectContent>
                    {options.sectors.map((sector) => (
                      <SelectItem key={sector} value={sector}>
                        {sector}
                      </SelectItem>
                    ))}
                  </SelectContent>
                </Select>
              </div>

              {/* Sort By */}
              <div className="space-y-2">
                <Label className="text-sm font-medium flex items-center gap-2">
                  <ArrowUpDown className="h-4 w-4" />
                  Sort By
                </Label>
                <Select
                  value={filters.sort_by}
                  onValueChange={(value) => onFilterChange('sort_by', value)}
                  disabled={loading}
                >
                  <SelectTrigger>
                    <SelectValue />
                  </SelectTrigger>
                  <SelectContent>
                    {sortOptions.map((option) => (
                      <SelectItem key={option.value} value={option.value}>
                        <span className="flex items-center gap-2">
                          <span>{option.icon}</span>
                          {option.label}
                        </span>
                      </SelectItem>
                    ))}
                  </SelectContent>
                </Select>
              </div>
            </div>

            {/* Advanced Numeric Filters */}
            <div className="space-y-4">
              <Label className="text-sm font-semibold">Advanced Criteria</Label>
              <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
                {/* Minimum EPS */}
                <div className="space-y-2">
                  <Label className="text-sm font-medium flex items-center gap-2">
                    <DollarSign className="h-4 w-4" />
                    Minimum EPS ($)
                  </Label>
                  <Input
                    type="number"
                    step="0.01"
                    placeholder="e.g. 1.00"
                    value={filters.min_eps || ''}
                    onChange={(e) =>
                      onFilterChange('min_eps', e.target.value ? parseFloat(e.target.value) : undefined)
                    }
                    disabled={loading}
                  />
                </div>

                {/* Minimum Growth */}
                <div className="space-y-2">
                  <Label className="text-sm font-medium flex items-center gap-2">
                    <TrendingUp className="h-4 w-4" />
                    Minimum Growth (%)
                  </Label>
                  <Input
                    type="number"
                    step="0.1"
                    placeholder="e.g. 5.0"
                    value={filters.min_growth || ''}
                    onChange={(e) =>
                      onFilterChange('min_growth', e.target.value ? parseFloat(e.target.value) : undefined)
                    }
                    disabled={loading}
                  />
                </div>
              </div>
            </div>

            {/* Active Filters Summary */}
            {activeFilterCount > 0 && (
              <div className="space-y-2 pt-4 border-t">
                <Label className="text-sm font-semibold">Active Filters</Label>
                <div className="flex flex-wrap gap-2">
                  {filters.country && (
                    <div className="inline-flex items-center gap-1 px-2 py-1 rounded-md bg-secondary text-secondary-foreground text-sm">
                      <Globe className="h-3 w-3" />
                      {filters.country}
                      <span
                        onClick={() => onFilterChange('country', undefined)}
                        className="ml-1 text-xs hover:text-destructive cursor-pointer"
                      >
                        ×
                      </span>
                    </div>
                  )}
                  {filters.sector && (
                    <div className="inline-flex items-center gap-1 px-2 py-1 rounded-md bg-secondary text-secondary-foreground text-sm">
                      <Building className="h-3 w-3" />
                      {filters.sector}
                      <span
                        onClick={() => onFilterChange('sector', undefined)}
                        className="ml-1 text-xs hover:text-destructive cursor-pointer"
                      >
                        ×
                      </span>
                    </div>
                  )}
                  {filters.sort_by !== 'market_cap' && (
                    <div className="inline-flex items-center gap-1 px-2 py-1 rounded-md bg-secondary text-secondary-foreground text-sm">
                      <ArrowUpDown className="h-3 w-3" />
                      {sortOptions.find(s => s.value === filters.sort_by)?.label}
                    </div>
                  )}
                  {filters.min_eps && (
                    <div className="inline-flex items-center gap-1 px-2 py-1 rounded-md bg-secondary text-secondary-foreground text-sm">
                      <DollarSign className="h-3 w-3" />
                      EPS ≥ ${filters.min_eps}
                      <span
                        onClick={() => onFilterChange('min_eps', undefined)}
                        className="ml-1 text-xs hover:text-destructive cursor-pointer"
                      >
                        ×
                      </span>
                    </div>
                  )}
                  {filters.min_growth && (
                    <div className="inline-flex items-center gap-1 px-2 py-1 rounded-md bg-secondary text-secondary-foreground text-sm">
                      <TrendingUp className="h-3 w-3" />
                      Growth ≥ {filters.min_growth}%
                      <span
                        onClick={() => onFilterChange('min_growth', undefined)}
                        className="ml-1 text-xs hover:text-destructive cursor-pointer"
                      >
                        ×
                      </span>
                    </div>
                  )}
                </div>
              </div>
            )}

            {/* Action Buttons */}
            <div className="flex justify-end gap-2 pt-2">
              <Button
                variant="outline"
                size="sm"
                onClick={onResetFilters}
                disabled={loading || activeFilterCount === 0}
              >
                <RotateCcw className="h-4 w-4 mr-2" />
                Reset All
              </Button>
            </div>
          </CardContent>
        </CollapsibleContent>
      </Collapsible>
    </Card>
  );
}