'use client';

import { useState } from 'react';
import { useRouter } from 'next/navigation';
import { Card } from '@/components/ui/card';
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
import type { FilterOptions, EPSQueryParams } from '@/lib/analytics-server';

interface FilterFormProps {
  filterOptions: FilterOptions;
  currentParams: EPSQueryParams;
}

export default function FilterForm({ filterOptions, currentParams }: FilterFormProps) {
  const router = useRouter();
  const [filters, setFilters] = useState({
    country: currentParams.country || '',
    sector: currentParams.sector || '',
    sort_by: currentParams.sort_by || 'growth_factor',
    min_eps: currentParams.min_eps?.toString() || '',
    min_growth: currentParams.min_growth?.toString() || '',
  });

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    const params = new URLSearchParams();
    
    // Always include page and limit
    params.set('page', '1'); // Reset to page 1 when filters change
    params.set('limit', String(currentParams.limit));
    
    // Add other params if they exist
    if (filters.country) params.set('country', filters.country);
    if (filters.sector) params.set('sector', filters.sector);
    if (filters.sort_by) params.set('sort_by', filters.sort_by);
    if (filters.min_eps) params.set('min_eps', filters.min_eps);
    if (filters.min_growth) params.set('min_growth', filters.min_growth);

    router.push(`/analytics?${params.toString()}`);
  };

  const handleReset = () => {
    const params = new URLSearchParams();
    params.set('page', '1');
    params.set('limit', String(currentParams.limit));
    params.set('sort_by', 'growth_factor');
    
    router.push(`/analytics?${params.toString()}`);
  };

  const updateFilter = (key: string, value: string) => {
    setFilters(prev => ({ ...prev, [key]: value }));
  };

  return (
    <Card className="p-6">
      <form onSubmit={handleSubmit}>
        <div className="grid grid-cols-1 gap-4 md:grid-cols-2 lg:grid-cols-4">
          {/* Country Filter */}
          <div>
            <Label htmlFor="country">Country</Label>
            <Select value={filters.country} onValueChange={(value) => updateFilter('country', value)}>
              <SelectTrigger>
                <SelectValue placeholder="All Countries" />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="">All Countries</SelectItem>
                {filterOptions.countries.map(country => (
                  <SelectItem key={country.value} value={country.value}>
                    {country.label}
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>
          </div>

          {/* Sector Filter */}
          <div>
            <Label htmlFor="sector">Sector</Label>
            <Select value={filters.sector} onValueChange={(value) => updateFilter('sector', value)}>
              <SelectTrigger>
                <SelectValue placeholder="All Sectors" />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="">All Sectors</SelectItem>
                {filterOptions.sectors.map(sector => (
                  <SelectItem key={sector} value={sector}>
                    {sector}
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>
          </div>

          {/* Sort By Filter */}
          <div>
            <Label htmlFor="sort_by">Sort By</Label>
            <Select value={filters.sort_by} onValueChange={(value) => updateFilter('sort_by', value)}>
              <SelectTrigger>
                <SelectValue placeholder="Sort By" />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="growth_factor">Growth Factor</SelectItem>
                <SelectItem value="ranking_position">Ranking Position</SelectItem>
                <SelectItem value="latest_date">Latest Date</SelectItem>
                <SelectItem value="symbol">Symbol</SelectItem>
              </SelectContent>
            </Select>
          </div>

          {/* Min EPS Filter */}
          <div>
            <Label htmlFor="min_eps">Min EPS</Label>
            <Input
              type="number"
              value={filters.min_eps}
              onChange={(e) => updateFilter('min_eps', e.target.value)}
              placeholder="0.0"
              step="0.1"
            />
          </div>

          {/* Min Growth Filter */}
          <div className="md:col-span-2 lg:col-span-1">
            <Label htmlFor="min_growth">Min Growth %</Label>
            <Input
              type="number"
              value={filters.min_growth}
              onChange={(e) => updateFilter('min_growth', e.target.value)}
              placeholder="0.0"
              step="0.1"
            />
          </div>
        </div>

        <div className="mt-4 flex items-center gap-2">
          <Button type="submit" size="sm">
            Apply Filters
          </Button>
          
          <Button variant="outline" size="sm" type="button" onClick={handleReset}>
            Reset Filters
          </Button>
        </div>
      </form>
    </Card>
  );
}