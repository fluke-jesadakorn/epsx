'use client';

import { memo, useState } from 'react';
import type { FilterPanelProps } from '@/types/analytics';

const FilterPanel = memo<FilterPanelProps>(({ filters, options, onFiltersChange, isLoading }) => {
  const [isExpanded, setIsExpanded] = useState(false);

  const handleSortChange = (sort_by: string) => {
    onFiltersChange({ sort_by: sort_by as any });
  };

  const handleCountryChange = (country: string) => {
    onFiltersChange({ country: country || undefined });
  };

  const handleSectorChange = (sector: string) => {
    onFiltersChange({ sector: sector || undefined });
  };

  const handleMinEpsChange = (min_eps: string) => {
    const value = parseFloat(min_eps);
    onFiltersChange({ min_eps: isNaN(value) ? undefined : value });
  };

  const handleMinGrowthChange = (min_growth: string) => {
    const value = parseFloat(min_growth);
    onFiltersChange({ min_growth: isNaN(value) ? undefined : value });
  };

  const clearAllFilters = () => {
    onFiltersChange({
      country: undefined,
      sector: undefined,
      min_eps: undefined,
      min_growth: undefined,
      sort_by: 'qoq_growth',
      page: 1,
    });
  };

  const activeFilterCount = [filters.country, filters.sector, filters.min_eps, filters.min_growth]
    .filter(Boolean).length;

  return (
    <div className="bg-white rounded-lg border border-gray-200 overflow-hidden">
      {/* Header - always visible on mobile */}
      <div className="p-4 border-b border-gray-100">
        <div className="flex items-center justify-between">
          <div className="flex items-center gap-2">
            <h3 className="font-semibold text-gray-900">Filters</h3>
            {activeFilterCount > 0 && (
              <span className="bg-orange-100 text-orange-600 px-2 py-1 rounded-full text-xs font-medium">
                {activeFilterCount}
              </span>
            )}
          </div>
          
          <div className="flex items-center gap-2">
            {activeFilterCount > 0 && (
              <button
                onClick={clearAllFilters}
                className="text-sm text-gray-500 hover:text-gray-700 px-2 py-1 rounded"
                disabled={isLoading}
              >
                Clear
              </button>
            )}
            
            <button
              onClick={() => setIsExpanded(!isExpanded)}
              className="lg:hidden p-1 rounded hover:bg-gray-100"
              aria-label="Toggle filters"
            >
              <svg
                className={`w-5 h-5 transition-transform ${isExpanded ? 'rotate-180' : ''}`}
                fill="none"
                stroke="currentColor"
                viewBox="0 0 24 24"
              >
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="m19 9-7 7-7-7" />
              </svg>
            </button>
          </div>
        </div>
      </div>

      {/* Filter content - collapsible on mobile */}
      <div className={`${isExpanded ? 'block' : 'hidden'} lg:block p-4 space-y-4`}>
        {/* Sort By */}
        <div>
          <label className="block text-sm font-medium text-gray-700 mb-2">Sort By</label>
          <select
            value={filters.sort_by}
            onChange={(e) => handleSortChange(e.target.value)}
            className="w-full p-2.5 border border-gray-300 rounded-lg text-sm focus:ring-2 focus:ring-orange-500 focus:border-orange-500 min-h-[44px]"
            disabled={isLoading}
          >
            <option value="qoq_growth">EPS Growth</option>
            <option value="current_eps">Current EPS</option>
            <option value="market_cap">Market Cap</option>
            <option value="ranking_position">Ranking Position</option>
          </select>
        </div>

        {/* Country Filter */}
        <div>
          <label className="block text-sm font-medium text-gray-700 mb-2">Country</label>
          <select
            value={filters.country || ''}
            onChange={(e) => handleCountryChange(e.target.value)}
            className="w-full p-2.5 border border-gray-300 rounded-lg text-sm focus:ring-2 focus:ring-orange-500 focus:border-orange-500 min-h-[44px]"
            disabled={isLoading}
          >
            <option value="">All Countries</option>
            {options.countries.map((country) => (
              <option key={country} value={country}>
                {country}
              </option>
            ))}
          </select>
        </div>

        {/* Sector Filter */}
        <div>
          <label className="block text-sm font-medium text-gray-700 mb-2">Sector</label>
          <select
            value={filters.sector || ''}
            onChange={(e) => handleSectorChange(e.target.value)}
            className="w-full p-2.5 border border-gray-300 rounded-lg text-sm focus:ring-2 focus:ring-orange-500 focus:border-orange-500 min-h-[44px]"
            disabled={isLoading}
          >
            <option value="">All Sectors</option>
            {options.sectors.map((sector) => (
              <option key={sector} value={sector}>
                {sector}
              </option>
            ))}
          </select>
        </div>

        {/* Minimum EPS */}
        <div>
          <label className="block text-sm font-medium text-gray-700 mb-2">Min EPS ($)</label>
          <input
            type="number"
            step="0.01"
            placeholder="e.g. 1.00"
            value={filters.min_eps ?? ''}
            onChange={(e) => handleMinEpsChange(e.target.value)}
            className="w-full p-2.5 border border-gray-300 rounded-lg text-sm focus:ring-2 focus:ring-orange-500 focus:border-orange-500 min-h-[44px]"
            disabled={isLoading}
          />
        </div>

        {/* Minimum Growth */}
        <div>
          <label className="block text-sm font-medium text-gray-700 mb-2">Min Growth (%)</label>
          <input
            type="number"
            step="0.1"
            placeholder="e.g. 5.0"
            value={filters.min_growth ?? ''}
            onChange={(e) => handleMinGrowthChange(e.target.value)}
            className="w-full p-2.5 border border-gray-300 rounded-lg text-sm focus:ring-2 focus:ring-orange-500 focus:border-orange-500 min-h-[44px]"
            disabled={isLoading}
          />
        </div>
      </div>
    </div>
  );
});

FilterPanel.displayName = 'FilterPanel';

export default FilterPanel;