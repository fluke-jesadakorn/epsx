'use client';

import React from 'react';
import {
  Globe,
  Building,
  ArrowUpDown,
  DollarSign,
  TrendingUp,
  X,
  RotateCcw,
  Filter,
  Zap,
} from 'lucide-react';
import type { AnalyticsFilters } from '@/hooks/useAnalyticsFilters';
import { Badge, Button, Card, CardContent } from '@epsx/ui';

interface ActiveFilterIndicatorsProps {
  filters: AnalyticsFilters;
  onRemoveFilter: <K extends keyof AnalyticsFilters>(key: K) => void;
  onClearAllFilters: () => void;
  className?: string;
}

interface FilterIndicator {
  key: keyof AnalyticsFilters;
  label: string;
  value: string | number;
  displayValue: string;
  icon: React.ComponentType<{ className?: string }>;
  color: 'default' | 'secondary' | 'destructive' | 'outline';
  canRemove: boolean;
}

export function ActiveFilterIndicators({
  filters,
  onRemoveFilter,
  onClearAllFilters,
  className,
}: ActiveFilterIndicatorsProps) {
  // Get sort option display name
  const getSortDisplayName = (sortValue: string) => {
    const sortOptions: Record<string, string> = {
      ranking_score: '🏆 Best Ranked',
      current_eps: '📈 Highest EPS',
      qoq_growth: '🚀 Best Growth',
      market_cap: '💰 Market Cap',
      symbol: '🔤 Symbol A-Z',
    };
    return sortOptions[sortValue] || sortValue;
  };

  // Build filter indicators
  const buildFilterIndicators = (): FilterIndicator[] => {
    const indicators: FilterIndicator[] = [];

    // Country filter
    if (filters.country) {
      indicators.push({
        key: 'country',
        label: 'Country',
        value: filters.country,
        displayValue: filters.country,
        icon: Globe,
        color: 'default',
        canRemove: true,
      });
    }

    // Sector filter
    if (filters.sector) {
      indicators.push({
        key: 'sector',
        label: 'Sector',
        value: filters.sector,
        displayValue: filters.sector,
        icon: Building,
        color: 'default',
        canRemove: true,
      });
    }

    // Sort filter (show if not default)
    if (filters.sort_by && filters.sort_by !== 'ranking_score') {
      indicators.push({
        key: 'sort_by',
        label: 'Sort',
        value: filters.sort_by,
        displayValue: getSortDisplayName(filters.sort_by),
        icon: ArrowUpDown,
        color: 'secondary',
        canRemove: true,
      });
    }

    // Min EPS filter
    if (filters.min_eps !== undefined) {
      indicators.push({
        key: 'min_eps',
        label: 'Min EPS',
        value: filters.min_eps,
        displayValue: `≥ $${filters.min_eps}`,
        icon: DollarSign,
        color: 'outline',
        canRemove: true,
      });
    }

    // Min Growth filter
    if (filters.min_growth !== undefined) {
      indicators.push({
        key: 'min_growth',
        label: 'Min Growth',
        value: filters.min_growth,
        displayValue: `≥ ${filters.min_growth}%`,
        icon: TrendingUp,
        color: 'outline',
        canRemove: true,
      });
    }

    return indicators;
  };

  const indicators = buildFilterIndicators();
  const hasActiveFilters = indicators.length > 0;

  // Get pagination info for page-based display
  const getPaginationInfo = () => {
    const currentPage = filters.page;
    const isCustomPagination = filters.page > 1 || filters.limit !== 12;
    
    return {
      currentPage,
      isCustomPagination,
      displayText: isCustomPagination 
        ? `Page ${currentPage} (${filters.limit} per page)`
        : `Page ${currentPage}`
    };
  };

  const paginationInfo = getPaginationInfo();

  if (!hasActiveFilters && !paginationInfo.isCustomPagination) {
    return null;
  }

  return (
    <Card className={`border-l-4 border-l-primary/50 ${className}`}>
      <CardContent className="p-4">
        <div className="space-y-3">
          {/* Header */}
          <div className="flex items-center justify-between">
            <div className="flex items-center gap-2">
              <Filter className="h-4 w-4 text-primary" />
              <span className="font-medium text-sm">Active Filters</span>
              {hasActiveFilters && (
                <Badge variant="secondary" className="text-xs">
                  {indicators.length}
                </Badge>
              )}
            </div>
            
            {hasActiveFilters && (
              <Button
                variant="ghost"
                size="sm"
                onClick={onClearAllFilters}
                className="h-6 px-2 text-xs text-muted-foreground hover:text-destructive"
              >
                <RotateCcw className="h-3 w-3 mr-1" />
                Clear all
              </Button>
            )}
          </div>

          {/* Filter Badges */}
          <div className="flex flex-wrap gap-2">
            {/* Active Filters */}
            {indicators.map((indicator) => {
              const Icon = indicator.icon;
              return (
                <Badge
                  key={indicator.key}
                  variant={indicator.color}
                  className="gap-1 pr-1 group hover:bg-destructive/10 transition-colors"
                >
                  <Icon className="h-3 w-3" />
                  <span className="text-xs">{indicator.displayValue}</span>
                  {indicator.canRemove && (
                    <span
                      onClick={() => onRemoveFilter(indicator.key)}
                      className="inline-flex items-center justify-center h-4 w-4 p-0 hover:bg-destructive/20 hover:text-destructive cursor-pointer rounded-sm transition-colors"
                    >
                      <X className="h-2 w-2" />
                    </span>
                  )}
                </Badge>
              );
            })}

            {/* Pagination Info */}
            {paginationInfo.isCustomPagination && (
              <Badge variant="outline" className="gap-1">
                <Zap className="h-3 w-3" />
                <span className="text-xs">{paginationInfo.displayText}</span>
              </Badge>
            )}

            {/* Show default state when no filters */}
            {!hasActiveFilters && !paginationInfo.isCustomPagination && (
              <Badge variant="outline" className="text-muted-foreground">
                <Filter className="h-3 w-3 mr-1" />
                No filters applied
              </Badge>
            )}
          </div>

          {/* Filter Summary */}
          {hasActiveFilters && (
            <div className="text-xs text-muted-foreground pt-2 border-t">
              <span>
                Showing filtered results
                {paginationInfo.currentPage > 1 && ` (${paginationInfo.displayText})`}
              </span>
            </div>
          )}
        </div>
      </CardContent>
    </Card>
  );
}