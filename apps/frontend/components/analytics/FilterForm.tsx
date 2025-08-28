'use client';

import { useState, useEffect } from 'react';
import { useRouter } from 'next/navigation';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { Button } from '@/components/ui/button';
import { Label } from '@/components/ui/label';
import { Input } from '@/components/ui/input';
import { 
  Select, 
  SelectContent, 
  SelectItem, 
  SelectTrigger, 
  SelectValue 
} from '@/components/ui/select';
import { 
  Filter,
  Globe,
  Building2,
  TrendingUp,
  DollarSign,
  Search,
  RotateCcw,
  X,
  Sparkles,
  Activity,
  Eye,
  EyeOff
} from 'lucide-react';
import type { FilterOptions, EPSQueryParams } from '@/lib/analytics-server';

interface FilterFormProps {
  filterOptions: FilterOptions;
  currentParams: EPSQueryParams;
}

export default function FilterForm({ filterOptions, currentParams }: FilterFormProps) {
  const router = useRouter();
  const [filters, setFilters] = useState({
    country: currentParams.country || 'all',
    sector: currentParams.sector || 'all',
    sort_by: currentParams.sort_by || 'growth_factor',
    min_eps: currentParams.min_eps?.toString() || '',
    min_growth: currentParams.min_growth?.toString() || '',
  });

  const [hasChanges, setHasChanges] = useState(false);
  const [isCompact, setIsCompact] = useState(false);
  const [isAnimating, setIsAnimating] = useState(false);

  // Check if any filters have changed from current state
  useEffect(() => {
    const changes = 
      filters.country !== (currentParams.country || 'all') ||
      filters.sector !== (currentParams.sector || 'all') ||
      filters.sort_by !== (currentParams.sort_by || 'growth_factor') ||
      filters.min_eps !== (currentParams.min_eps?.toString() || '') ||
      filters.min_growth !== (currentParams.min_growth?.toString() || '');
    
    setHasChanges(changes);
  }, [filters, currentParams]);

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    setIsAnimating(true);
    
    const params = new URLSearchParams();
    
    // Always include page and limit
    params.set('page', '1'); // Reset to page 1 when filters change
    params.set('limit', String(currentParams.limit));
    
    // Add other params if they exist
    if (filters.country && filters.country !== 'all') params.set('country', filters.country);
    if (filters.sector && filters.sector !== 'all') params.set('sector', filters.sector);
    if (filters.sort_by) params.set('sort_by', filters.sort_by);
    if (filters.min_eps) params.set('min_eps', filters.min_eps);
    if (filters.min_growth) params.set('min_growth', filters.min_growth);

    router.push(`/analytics?${params.toString()}`);
    
    setTimeout(() => setIsAnimating(false), 1000);
  };

  const handleReset = () => {
    setFilters({
      country: 'all',
      sector: 'all',
      sort_by: 'growth_factor',
      min_eps: '',
      min_growth: '',
    });

    const params = new URLSearchParams();
    params.set('page', '1');
    params.set('limit', String(currentParams.limit));
    params.set('sort_by', 'growth_factor');
    
    router.push(`/analytics?${params.toString()}`);
  };

  const updateFilter = (key: string, value: string) => {
    setFilters(prev => ({ ...prev, [key]: value }));
  };

  const clearFilter = (filterKey: string) => {
    if (filterKey === 'country' || filterKey === 'sector') {
      updateFilter(filterKey, 'all');
    } else {
      updateFilter(filterKey, '');
    }
  };

  const getActiveFilterCount = () => {
    let count = 0;
    if (currentParams.country) count++;
    if (currentParams.sector) count++;
    if (currentParams.min_eps) count++;
    if (currentParams.min_growth) count++;
    return count;
  };

  const activeFilterCount = getActiveFilterCount();

  return (
    <Card className="card-smart-filters">
      {/* Animated background elements */}
      <div className="absolute inset-0 opacity-50">
        <div className="animate-float absolute -top-4 -left-4 h-16 w-16 rounded-full bg-gradient-to-br from-pink-400/30 to-orange-400/30 blur-xl" />
        <div className="animate-bounce-gentle absolute -top-2 -right-8 h-12 w-12 rounded-full bg-gradient-to-br from-yellow-400/25 to-pink-400/25 blur-lg" />
        <div className="animate-pulse-gentle absolute -bottom-4 -left-8 h-20 w-20 rounded-full bg-gradient-to-br from-orange-400/20 to-yellow-400/20 blur-xl" />
      </div>

      <CardHeader className="relative z-10 pb-4">
        <div className="flex items-center justify-between">
          <CardTitle className="flex items-center gap-3 text-xl font-bold bg-gradient-to-r from-pink-600 to-orange-600 bg-clip-text text-transparent dark:from-cyan-400 dark:to-blue-400">
            <div className="relative">
              <Filter className="h-6 w-6 text-pink-600 dark:text-cyan-400" />
              {hasChanges && (
                <div className="absolute -top-1 -right-1 h-3 w-3 bg-orange-400 rounded-full animate-ping" />
              )}
            </div>
            🔥 Smart Filters
            {activeFilterCount > 0 && (
              <span className="px-3 py-1 bg-gradient-to-r from-pink-500 to-orange-500 text-white text-xs font-semibold rounded-full shadow-lg animate-pulse">
                {activeFilterCount} active
              </span>
            )}
          </CardTitle>
          
          <div className="flex items-center gap-2">
            {hasChanges && (
              <div className="flex items-center gap-2 px-3 py-1 bg-orange-100 dark:bg-orange-900/30 rounded-full">
                <div className="w-2 h-2 bg-orange-500 rounded-full animate-pulse" />
                <span className="text-xs font-semibold text-orange-700 dark:text-orange-300">
                  Changes pending
                </span>
              </div>
            )}
            
            <Button
              variant="ghost"
              size="sm"
              onClick={() => setIsCompact(!isCompact)}
              className="p-2 hover:bg-pink-100 dark:hover:bg-pink-900/30"
            >
              {isCompact ? <Eye className="h-4 w-4" /> : <EyeOff className="h-4 w-4" />}
            </Button>
          </div>
        </div>

        {/* Filter Stats */}
        <div className="flex items-center gap-4 text-sm text-gray-600 dark:text-gray-300">
          <div className="flex items-center gap-1">
            <Activity className="h-4 w-4 text-pink-500" />
            <span>{filterOptions.countries?.length || 0} countries</span>
          </div>
          <div className="flex items-center gap-1">
            <Building2 className="h-4 w-4 text-green-500" />
            <span>{filterOptions.sectors?.length || 0} sectors</span>
          </div>
          {activeFilterCount > 0 && (
            <div className="flex items-center gap-1">
              <Sparkles className="h-4 w-4 text-purple-500" />
              <span className="font-semibold">Filtered view active</span>
            </div>
          )}
        </div>
      </CardHeader>
      
      <CardContent className="relative z-10">
        <form onSubmit={handleSubmit} className="space-y-6">
          <div className={`grid gap-6 transition-all duration-500 ${
            isCompact ? 'grid-cols-1 lg:grid-cols-2' : 'grid-cols-1 md:grid-cols-2'
          }`}>
            
            {/* Country Filter */}
            <div className="space-y-2">
              <Label htmlFor="country" className="flex items-center gap-2 text-sm font-semibold text-gray-700 dark:text-gray-200">
                <Globe className="h-4 w-4 text-blue-600 dark:text-blue-400" />
                Country
              </Label>
              <div className="relative">
                <Select value={filters.country} onValueChange={(value) => updateFilter('country', value)}>
                  <SelectTrigger className="bg-white/80 backdrop-blur-sm border-gray-300 hover:border-blue-400 focus:border-blue-500 focus:ring-2 focus:ring-blue-200 transition-all shadow-sm">
                    <SelectValue placeholder="🌍 All Countries" />
                  </SelectTrigger>
                  <SelectContent className="max-h-64 bg-white/95 backdrop-blur-md border-blue-200 dark:bg-slate-800/90">
                    <SelectItem value="all">
                      <div className="flex items-center gap-2">
                        <Globe className="h-4 w-4 text-gray-500 dark:text-gray-400" />
                        All Countries
                      </div>
                    </SelectItem>
                    {filterOptions.countries?.map(country => (
                      <SelectItem key={country.value} value={country.value}>
                        <div className="flex items-center gap-2">
                          <div className="w-4 h-3 bg-gradient-to-r from-blue-400 to-blue-600 rounded-sm shadow-sm" />
                          {country.label}
                        </div>
                      </SelectItem>
                    ))}
                  </SelectContent>
                </Select>
                {filters.country && filters.country !== 'all' && (
                  <button
                    type="button"
                    onClick={() => clearFilter('country')}
                    className="absolute right-8 top-1/2 -translate-y-1/2 p-1 text-gray-400 hover:text-red-500 transition-colors"
                  >
                    <X className="h-3 w-3" />
                  </button>
                )}
              </div>
            </div>

            {/* Sector Filter */}
            <div className="space-y-2">
              <Label htmlFor="sector" className="flex items-center gap-2 text-sm font-semibold text-gray-700 dark:text-gray-200">
                <Building2 className="h-4 w-4 text-green-600 dark:text-green-400" />
                Sector
              </Label>
              <div className="relative">
                <Select value={filters.sector} onValueChange={(value) => updateFilter('sector', value)}>
                  <SelectTrigger className="bg-white/80 backdrop-blur-sm border-gray-300 hover:border-green-400 focus:border-green-500 focus:ring-2 focus:ring-green-200 transition-all shadow-sm">
                    <SelectValue placeholder="🏢 All Sectors" />
                  </SelectTrigger>
                  <SelectContent className="bg-white/95 backdrop-blur-md border-green-200 dark:bg-slate-800/90">
                    <SelectItem value="all">
                      <div className="flex items-center gap-2">
                        <Building2 className="h-4 w-4 text-gray-500 dark:text-gray-400" />
                        All Sectors
                      </div>
                    </SelectItem>
                    {filterOptions.sectors?.map(sector => (
                      <SelectItem key={sector} value={sector}>
                        <div className="flex items-center gap-2">
                          <div className="w-2 h-2 bg-gradient-to-r from-green-400 to-emerald-500 rounded-full" />
                          {sector}
                        </div>
                      </SelectItem>
                    ))}
                  </SelectContent>
                </Select>
                {filters.sector && filters.sector !== 'all' && (
                  <button
                    type="button"
                    onClick={() => clearFilter('sector')}
                    className="absolute right-8 top-1/2 -translate-y-1/2 p-1 text-gray-400 hover:text-red-500 transition-colors"
                  >
                    <X className="h-3 w-3" />
                  </button>
                )}
              </div>
            </div>
          </div>

          {/* Enhanced Action Buttons */}
          <div className="flex items-center justify-between pt-6 border-t border-gradient-to-r border-pink-200/60 dark:border-pink-400/20">
            <div className="text-sm text-gray-600 dark:text-gray-300">
              {activeFilterCount > 0 ? (
                <div className="flex items-center gap-2">
                  <div className="w-2 h-2 bg-pink-500 rounded-full animate-pulse" />
                  <span className="font-medium">
                    {activeFilterCount} filter{activeFilterCount !== 1 ? 's' : ''} applied
                  </span>
                </div>
              ) : (
                <div className="flex items-center gap-2">
                  <Sparkles className="h-4 w-4 text-gray-400" />
                  <span>No filters applied</span>
                </div>
              )}
            </div>
            
            <div className="flex items-center gap-3">
              <Button
                type="button"
                variant="outline"
                size="sm"
                onClick={handleReset}
                disabled={!hasChanges && activeFilterCount === 0}
                className="flex items-center gap-2 border-gray-300 hover:border-red-400 hover:bg-red-50 dark:hover:bg-red-900/20 transition-all"
              >
                <RotateCcw className="h-4 w-4" />
                Reset
              </Button>
              
              <Button
                type="submit"
                size="sm"
                disabled={!hasChanges}
                className={`flex items-center gap-2 transition-all duration-300 ${
                  hasChanges 
                    ? 'bg-gradient-to-r from-pink-500 to-orange-500 hover:from-pink-600 hover:to-orange-600 shadow-lg shadow-pink-500/30 animate-pulse' 
                    : 'bg-gray-400'
                } ${isAnimating ? 'scale-105 shadow-xl' : ''}`}
              >
                <Search className={`h-4 w-4 ${isAnimating ? 'animate-spin' : ''}`} />
                {isAnimating ? 'Applying...' : 'Apply Filters'}
              </Button>
            </div>
          </div>
        </form>
      </CardContent>
    </Card>
  );
}