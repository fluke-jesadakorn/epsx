
'use client';

import { useState, useEffect } from 'react';
import { useRouter } from 'next/navigation';
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue
} from '@/components/ui/select';
import { Filter, Globe, Sparkles, RotateCcw, Search } from 'lucide-react';
import type { FilterOptions, EPSQueryParams } from '@/lib/server/data';

interface FilterFormProps {
  filterOptions: FilterOptions;
  currentParams: EPSQueryParams;
}

export default function FilterForm({ filterOptions, currentParams }: FilterFormProps) {
  const router = useRouter();
  const [filters, setFilters] = useState({
    country: currentParams.country ?? 'all',
    sector: currentParams.sector ?? 'all',
    sort_by: currentParams.sort_by || 'growth_factor',
    min_eps: currentParams.min_eps?.toString() ?? '',
    min_growth: currentParams.min_growth?.toString() ?? '',
  });

  const [hasChanges, setHasChanges] = useState(false);

  useEffect(() => {
    const changes =
      filters.country !== (currentParams.country ?? 'all') ||
      filters.sector !== (currentParams.sector ?? 'all') ||
      filters.sort_by !== (currentParams.sort_by || 'growth_factor') ||
      filters.min_eps !== (currentParams.min_eps?.toString() ?? '') ||
      filters.min_growth !== (currentParams.min_growth?.toString() ?? '');
    setHasChanges(changes);
  }, [filters, currentParams]);

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    const params = new URLSearchParams();
    params.set('page', '1');
    params.set('limit', String(currentParams.limit));
    if (filters.country && filters.country !== 'all') {params.set('country', filters.country);}
    if (filters.sector && filters.sector !== 'all') {params.set('sector', filters.sector);}
    if (filters.sort_by) {params.set('sort_by', filters.sort_by);}
    if (filters.min_eps) {params.set('min_eps', filters.min_eps);}
    if (filters.min_growth) {params.set('min_growth', filters.min_growth);}
    router.push(`/analytics?${params.toString()}`);
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

  const activeCount = [
    currentParams.country,
    currentParams.sector,
    currentParams.min_eps !== undefined ? 'x' : undefined,
    currentParams.min_growth !== undefined ? 'x' : undefined,
  ].filter(Boolean).length;

  const countries = filterOptions.countries ?? [];
  const sectors = filterOptions.sectors ?? [];

  return (
    <form onSubmit={handleSubmit}>
      <div className="rounded-xl border border-gray-200 dark:border-white/[0.06] bg-white dark:bg-card backdrop-blur-sm">
        {/* Filter bar */}
        <div className="flex flex-col gap-3 p-3 sm:flex-row sm:items-end sm:gap-3 sm:p-4">
          {/* Label */}
          <div className="flex items-center gap-2 sm:hidden">
            <Filter className="h-4 w-4 text-slate-400" />
            <span className="text-sm font-medium text-slate-300">Filters</span>
            {activeCount > 0 && (
              <span className="rounded-full bg-purple-500/20 px-2 py-0.5 text-xs font-semibold text-purple-300">
                {activeCount}
              </span>
            )}
          </div>

          {/* Country */}
          <div className="flex-1 min-w-0">
            <label className="mb-1.5 flex items-center gap-1.5 text-xs font-medium text-slate-400">
              <Globe className="h-3 w-3" />
              Country
            </label>
            <Select value={filters.country} onValueChange={(v) => updateFilter('country', v)}>
              <SelectTrigger className="h-9 border-gray-200 dark:border-white/[0.08] bg-gray-100 dark:bg-slate-800/60 text-sm text-slate-200 hover:bg-gray-100 dark:bg-slate-800">
                <SelectValue placeholder="All Countries" />
              </SelectTrigger>
              <SelectContent className="border-gray-200 dark:border-border bg-gray-100 dark:bg-slate-800">
                <SelectItem value="all">All Countries</SelectItem>
                {countries.map(c => (
                  <SelectItem key={c.value} value={c.value}>{c.label}</SelectItem>
                ))}
              </SelectContent>
            </Select>
          </div>

          {/* Sector */}
          <div className="flex-1 min-w-0">
            <label className="mb-1.5 flex items-center gap-1.5 text-xs font-medium text-slate-400">
              <Sparkles className="h-3 w-3" />
              Sector
            </label>
            <Select value={filters.sector} onValueChange={(v) => updateFilter('sector', v)}>
              <SelectTrigger className="h-9 border-gray-200 dark:border-white/[0.08] bg-gray-100 dark:bg-slate-800/60 text-sm text-slate-200 hover:bg-gray-100 dark:bg-slate-800">
                <SelectValue placeholder="All Sectors" />
              </SelectTrigger>
              <SelectContent className="border-gray-200 dark:border-border bg-gray-100 dark:bg-slate-800">
                <SelectItem value="all">All Sectors</SelectItem>
                {sectors.map(s => (
                  <SelectItem key={s} value={s}>{s}</SelectItem>
                ))}
              </SelectContent>
            </Select>
          </div>

          {/* Actions */}
          <div className="flex items-end gap-2">
            <button
              type="submit"
              disabled={!hasChanges}
              className="inline-flex h-9 items-center gap-1.5 rounded-lg bg-purple-600 px-4 text-sm font-medium text-white transition-colors hover:bg-purple-500 disabled:opacity-40 disabled:pointer-events-none"
            >
              <Search className="h-3.5 w-3.5" />
              Apply
            </button>
            {(hasChanges || activeCount > 0) && (
              <button
                type="button"
                onClick={handleReset}
                className="inline-flex h-9 items-center gap-1.5 rounded-lg border border-gray-200 dark:border-white/[0.08] bg-gray-100 dark:bg-slate-800/60 px-3 text-sm font-medium text-slate-300 transition-colors hover:bg-slate-700/60"
              >
                <RotateCcw className="h-3.5 w-3.5" />
                <span className="hidden sm:inline">Reset</span>
              </button>
            )}
          </div>
        </div>

        {/* Active filters summary */}
        {activeCount > 0 && (
          <div className="border-t border-white/[0.04] px-4 py-2 flex items-center gap-2 text-xs text-slate-400">
            <div className="h-1.5 w-1.5 rounded-full bg-emerald-400" />
            <span>{activeCount} filter{activeCount !== 1 ? 's' : ''} active</span>
            {currentParams.country && (
              <span className="rounded bg-gray-100 dark:bg-slate-800 px-1.5 py-0.5 text-slate-300">{currentParams.country}</span>
            )}
            {currentParams.sector && (
              <span className="rounded bg-gray-100 dark:bg-slate-800 px-1.5 py-0.5 text-slate-300">{currentParams.sector}</span>
            )}
          </div>
        )}
      </div>
    </form>
  );
}
