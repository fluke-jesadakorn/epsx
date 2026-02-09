'use client';

import { useState, useEffect, useRef } from 'react';
import { useRouter } from 'next/navigation';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { Button } from '@/components/ui/button';
import { Label } from '@/components/ui/label';
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue
} from '@/components/ui/select';
import { Sheet, SheetContent, SheetHeader, SheetTitle, SheetTrigger } from '@/components/ui/sheet';
import {
  Filter,
  Globe,
  Search,
  RotateCcw,
  Sparkles,
  ChevronDown
} from 'lucide-react';
import type { FilterOptions, EPSQueryParams } from '@/lib/unified-server-data';

interface FilterFormProps {
  filterOptions: FilterOptions;
  currentParams: EPSQueryParams;
}

// Smart Country Selector Component - iPhone-style on mobile, desktop select on desktop
function SmartCountrySelector({ 
  countries, 
  selectedCountry, 
  onCountryChange 
}: { 
  countries: FilterOptions['countries'], 
  selectedCountry: string, 
  onCountryChange: (country: string) => void 
}) {
  const [isSheetOpen, setIsSheetOpen] = useState(false);
  const [selectedIndex, setSelectedIndex] = useState(0);
  const scrollRef = useRef<HTMLDivElement>(null);

  // Prepare options with "All Countries" at the top
  const options = [
    { value: 'all', label: 'All Countries' },
    // eslint-disable-next-line @typescript-eslint/no-unnecessary-condition
    ...(countries || []).map(country => typeof country === 'string' 
      ? { value: country, label: country } 
      : country
    )
  ];

  // Find current selected index and scroll to it when opened
  useEffect(() => {
    const index = options.findIndex(option => option.value === selectedCountry);
    const newIndex = index >= 0 ? index : 0;
    setSelectedIndex(newIndex);
    
    if (isSheetOpen && scrollRef.current) {
      setTimeout(() => {
        scrollRef.current?.scrollTo({
          top: newIndex * 50,
          behavior: 'smooth'
        });
      }, 100);
    }
  }, [selectedCountry, options, isSheetOpen]);

  const selectedCountryLabel = options.find(c => c.value === selectedCountry)?.label ?? 'All Countries';

  const handleScroll = () => {
    if (!scrollRef.current) {return;}
    
    const scrollTop = scrollRef.current.scrollTop;
    const itemHeight = 50;
    const index = Math.round(scrollTop / itemHeight);
    const clampedIndex = Math.max(0, Math.min(index, options.length - 1));
    
    if (clampedIndex !== selectedIndex) {
      setSelectedIndex(clampedIndex);
    }
  };

  const handleIOSConfirm = () => {
    onCountryChange(options[selectedIndex].value);
    setIsSheetOpen(false);
  };

  return (
    <>
      {/* Desktop Select - hidden on mobile */}
      <div className="hidden md:block">
        <Select value={selectedCountry} onValueChange={onCountryChange}>
          <SelectTrigger>
            <SelectValue placeholder="All Countries" />
          </SelectTrigger>
          <SelectContent>
            <SelectItem value="all">All Countries</SelectItem>
            {/* eslint-disable-next-line @typescript-eslint/no-unnecessary-condition */}
            {countries?.map(country => {
              const countryObj = typeof country === 'string' 
                ? { value: country, label: country } 
                : country;
              return (
                <SelectItem key={countryObj.value} value={countryObj.value}>
                  {countryObj.label}
                </SelectItem>
              );
            })}
          </SelectContent>
        </Select>
      </div>

      {/* Mobile iPhone-style Sheet - visible on mobile only */}
      <div className="md:hidden">
        <Sheet open={isSheetOpen} onOpenChange={setIsSheetOpen}>
          <SheetTrigger asChild>
            <button
              type="button"
              className="w-full flex items-center justify-between p-3 border border-gray-300 dark:border-gray-600 rounded-md bg-white dark:bg-gray-800 hover:bg-gray-50 dark:hover:bg-gray-700"
            >
              <span className="text-left truncate">
                {selectedCountryLabel}
              </span>
              <ChevronDown className="h-4 w-4" />
            </button>
          </SheetTrigger>
          
          <SheetContent side="bottom" className="h-[60vh] p-0 bg-gray-100 dark:bg-gray-900">
            <SheetHeader className="p-4 border-b bg-gray-50 dark:bg-gray-800">
              <div className="flex items-center justify-between">
                <button
                  onClick={() => setIsSheetOpen(false)}
                  className="text-cyan-500 font-medium text-sm uppercase tracking-wide"
                >
                  CANCEL
                </button>
                <SheetTitle className="text-lg font-medium text-gray-600 dark:text-gray-300">
                  Select Country
                </SheetTitle>
                <button
                  onClick={handleIOSConfirm}
                  className="text-cyan-500 font-medium text-sm uppercase tracking-wide"
                >
                  SET
                </button>
              </div>
            </SheetHeader>
            
            {/* iOS-style Wheel Picker */}
            <div className="relative flex-1 flex items-center justify-center bg-white dark:bg-gray-800">
              {/* iOS Selection Indicator - Top Line */}
              <div className="absolute inset-x-0 top-1/2 transform -translate-y-6 h-[1px] bg-cyan-400 pointer-events-none z-10" />
              
              {/* iOS Selection Indicator - Bottom Line */}
              <div className="absolute inset-x-0 top-1/2 transform translate-y-6 h-[1px] bg-cyan-400 pointer-events-none z-10" />
              
              {/* Scroll Container */}
              <div 
                ref={scrollRef}
                onScroll={handleScroll}
                className="h-[280px] overflow-y-auto snap-y snap-mandatory ios-scroll-picker w-full"
                style={{
                  WebkitOverflowScrolling: 'touch',
                  scrollSnapType: 'y mandatory',
                  scrollbarWidth: 'none',
                  msOverflowStyle: 'none'
                }}
              >
                {/* Top padding to center first item */}
                <div className="h-[120px]" />
                
                {options.map((option, index) => (
                  <div
                    key={option.value}
                    className="h-[50px] flex items-center justify-center px-4 snap-start"
                    style={{
                      fontSize: selectedIndex === index ? '18px' : '16px',
                      fontWeight: selectedIndex === index ? '600' : '400',
                      opacity: Math.abs(selectedIndex - index) <= 1 
                        ? 1 - (Math.abs(selectedIndex - index) * 0.3)
                        : 0.4,
                      transition: 'all 0.2s ease',
                      color: selectedIndex === index 
                        ? '#1f2937' 
                        : Math.abs(selectedIndex - index) <= 2 
                          ? '#6b7280'
                          : '#9ca3af'
                    }}
                  >
                    <span className="text-center">
                      {option.label}
                    </span>
                  </div>
                ))}
                
                {/* Bottom padding to center last item */}
                <div className="h-[120px]" />
              </div>
              
              {/* Gradient Fade Effects */}
              <div className="absolute top-0 inset-x-0 h-16 bg-gradient-to-b from-white to-transparent pointer-events-none z-10 dark:from-gray-800" />
              <div className="absolute bottom-0 inset-x-0 h-16 bg-gradient-to-t from-white to-transparent pointer-events-none z-10 dark:from-gray-800" />
            </div>
          </SheetContent>
        </Sheet>
      </div>
    </>
  );
}

export default function FilterForm({ filterOptions, currentParams }: FilterFormProps) {
  const router = useRouter();
  const [filters, setFilters] = useState({
    country: currentParams.country ?? 'all',
    sort_by: currentParams.sort_by || 'growth_factor',
    min_eps: currentParams.min_eps?.toString() ?? '',
    min_growth: currentParams.min_growth?.toString() ?? '',
  });

  const [hasChanges, setHasChanges] = useState(false);
  const [_isCompact, _setIsCompact] = useState(false);
  const [_isAnimating, _setIsAnimating] = useState(false);
  const [isMounted, setIsMounted] = useState(false);

  // Handle client-side mounting only
  useEffect(() => {
    setIsMounted(true);
  }, []);

  // Check if any filters have changed from current state
  useEffect(() => {
    const changes = 
      filters.country !== (currentParams.country ?? 'all') ||
      filters.sort_by !== (currentParams.sort_by || 'growth_factor') ||
      filters.min_eps !== (currentParams.min_eps?.toString() ?? '') ||
      filters.min_growth !== (currentParams.min_growth?.toString() ?? '');
    
    setHasChanges(changes);
  }, [filters, currentParams]);

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    _setIsAnimating(true);
    
    const params = new URLSearchParams();
    
    // Always include page and limit
    params.set('page', '1'); // Reset to page 1 when filters change
    params.set('limit', String(currentParams.limit));
    
    // Add other params if they exist
    if (filters.country && filters.country !== 'all') {params.set('country', filters.country);}
    if (filters.sort_by) {params.set('sort_by', filters.sort_by);}
    if (filters.min_eps) {params.set('min_eps', filters.min_eps);}
    if (filters.min_growth) {params.set('min_growth', filters.min_growth);}

    router.push(`/analytics?${params.toString()}`);
    
    setTimeout(() => _setIsAnimating(false), 1000);
  };

  const handleReset = () => {
    setFilters({
      country: 'all',
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

  const _clearFilter = (filterKey: string) => {
    if (filterKey === 'country') {
      updateFilter(filterKey, 'all');
    } else {
      updateFilter(filterKey, '');
    }
  };

  const getActiveFilterCount = () => {
    let count = 0;
    if (currentParams.country) {count++;}
    if (currentParams.min_eps) {count++;}
    if (currentParams.min_growth) {count++;}
    return count;
  };

  const activeFilterCount = getActiveFilterCount();

  return (
    <>
      {/* Mobile Simplified Layout */}
      <div className="md:hidden">
        <Card className="border border-gray-200 dark:border-gray-700 bg-white/90 dark:bg-gray-800/90 backdrop-blur-sm">
          <CardHeader className="pb-3">
            <CardTitle className="flex items-center gap-3 text-lg font-bold">
              <Filter className="h-5 w-5 text-orange-500" />
              <div>
                <div className="flex items-center gap-2">
                  <span>🔍 Smart Filters</span>
                  {activeFilterCount > 0 && (
                    <span className="px-2 py-1 bg-orange-100 dark:bg-orange-900/30 text-xs font-bold rounded-full text-orange-700 dark:text-orange-300">
                      {activeFilterCount}
                    </span>
                  )}
                </div>
                <p className="text-sm text-gray-600 dark:text-gray-400 font-normal">
                  {/* eslint-disable-next-line @typescript-eslint/no-unnecessary-condition */}
                  ✨ {filterOptions.countries?.length || 68} countries • Lightning fast search
                </p>
              </div>
            </CardTitle>
          </CardHeader>
          
          <CardContent>
            <form onSubmit={handleSubmit} className="space-y-4">
              {/* Simplified Country Selection */}
              <div className="space-y-2">
                <Label className="flex items-center gap-2 text-sm font-semibold">
                  <Globe className="h-4 w-4 text-blue-500" />
                  🌍 Country Selection
                </Label>
                {!isMounted ? (
                  <div className="w-full p-3 bg-gray-100 dark:bg-gray-700 rounded-lg">
                    <div className="h-4 bg-gray-300 dark:bg-gray-600 rounded w-32" />
                  </div>
                ) : (
                  <SmartCountrySelector 
                    countries={filterOptions.countries} 
                    selectedCountry={filters.country}
                    onCountryChange={(value) => updateFilter('country', value)}
                  />
                )}
              </div>

              {/* Mobile Action Buttons */}
              <div className="flex items-center justify-between gap-3 pt-4">
                <div className="flex items-center gap-2 text-sm">
                  {activeFilterCount > 0 ? (
                    <>
                      <div className="w-2 h-2 bg-green-500 rounded-full" />
                      <span className="font-semibold text-green-600 dark:text-green-400">
                        ⚡ {activeFilterCount} filter{activeFilterCount !== 1 ? 's' : ''} applied
                      </span>
                    </>
                  ) : (
                    <span className="text-gray-500 dark:text-gray-400">
                      ✨ No filters applied — ready to explore
                    </span>
                  )}
                </div>
                
                <div className="flex gap-2">
                  <Button
                    type="button"
                    variant="outline"
                    size="sm"
                    onClick={handleReset}
                    disabled={!hasChanges && activeFilterCount === 0}
                    className="px-3 py-2 text-xs font-bold rounded-lg"
                  >
                    🔄 RESET
                  </Button>
                  
                  <Button
                    type="submit"
                    size="sm"
                    disabled={!hasChanges}
                    className="px-3 py-2 text-xs font-bold rounded-lg bg-blue-500 hover:bg-blue-600 text-white"
                  >
                    🚀 APPLY
                  </Button>
                </div>
              </div>
            </form>
          </CardContent>
        </Card>
      </div>

      {/* Desktop Original Layout */}
      <div className="hidden md:block">
        <Card className="relative border border-gray-200 dark:border-gray-700 bg-gradient-to-br from-slate-50/50 via-white to-slate-50/80 dark:from-gray-900/80 dark:via-gray-800 dark:to-gray-900/90 shadow-lg backdrop-blur-sm overflow-hidden">
          {/* Decorative background elements */}
          <div className="absolute top-0 right-0 w-32 h-32 bg-gradient-to-bl from-orange-100/30 via-transparent to-transparent dark:from-orange-900/20 rounded-full -translate-y-16 translate-x-16" />
          <div className="absolute bottom-0 left-0 w-24 h-24 bg-gradient-to-tr from-blue-100/30 via-transparent to-transparent dark:from-blue-900/20 rounded-full translate-y-12 -translate-x-12" />
          
          {/* Subtle grid pattern overlay */}
          <div className="absolute inset-0 opacity-10 dark:opacity-5" style={{
            backgroundImage: `radial-gradient(circle at 1px 1px, rgba(156, 163, 175, 0.4) 1px, transparent 0)`,
            backgroundSize: '20px 20px'
          }} />
          
          <CardHeader className="pb-4 relative z-10">
            <div className="flex items-center justify-between">
              <CardTitle className="flex items-center gap-3 text-xl font-black tracking-tight">
                <div className="relative">
                  <div className="absolute inset-0 bg-orange-400/20 dark:bg-orange-500/20 rounded-lg blur-md" />
                  <div className="relative bg-gradient-to-br from-orange-400 to-orange-500 p-2 rounded-lg">
                    <Filter className="h-5 w-5 text-white" />
                  </div>
                </div>
                <div>
                  <div className="flex items-center gap-2">
                    <span className="text-2xl font-extrabold bg-gradient-to-r from-slate-800 via-slate-900 to-slate-800 dark:from-slate-100 dark:via-white dark:to-slate-100 bg-clip-text text-transparent tracking-wide drop-shadow-sm">
                      🔍 Smart Filters
                    </span>
                    {activeFilterCount > 0 && (
                      <span className="px-3 py-1.5 bg-gradient-to-r from-orange-400/20 to-orange-500/20 border border-orange-300/30 dark:border-orange-600/30 text-xs font-bold tracking-wider uppercase rounded-full text-orange-700 dark:text-orange-300">
                        {activeFilterCount} active
                      </span>
                    )}
                  </div>
                  <p className="text-sm text-slate-600 dark:text-slate-400 mt-1 font-medium tracking-wide">
                    {/* eslint-disable-next-line @typescript-eslint/no-unnecessary-condition */}
                    ✨ <span className="font-semibold text-orange-600 dark:text-orange-400">{filterOptions.countries?.length || 68}</span> countries • <span className="italic">Lightning fast search</span>
                  </p>
                </div>
              </CardTitle>
            </div>
          </CardHeader>
          
          <CardContent className="relative z-10">
            <form onSubmit={handleSubmit} className="space-y-4">
              {/* Country Filter */}
              <div className="space-y-2">
                <Label htmlFor="country" className="flex items-center gap-2 text-base font-bold text-slate-800 dark:text-slate-200 tracking-wide">
                  <div className="relative">
                    <div className="absolute inset-0 bg-blue-400/20 dark:bg-blue-500/20 rounded blur-sm" />
                    <Globe className="relative h-5 w-5 text-blue-600 dark:text-blue-400" />
                  </div>
                  <span className="bg-gradient-to-r from-slate-800 to-slate-700 dark:from-slate-200 dark:to-slate-100 bg-clip-text text-transparent">
                    🌍 Country Selection
                  </span>
                </Label>
                <div className="relative">
                  {!isMounted ? (
                    <div className="w-full p-3 bg-gray-100 dark:bg-gray-800 rounded-lg">
                      <div className="h-4 bg-gray-300 dark:bg-gray-600 rounded w-32" />
                    </div>
                  ) : (
                    <SmartCountrySelector 
                      countries={filterOptions.countries} 
                      selectedCountry={filters.country}
                      onCountryChange={(value) => updateFilter('country', value)}
                    />
                  )}
                </div>
              </div>

              {/* Enhanced Action Buttons */}
              <div className="pt-6 border-t border-gradient-to-r from-slate-200/60 via-slate-300/40 to-slate-200/60 dark:from-slate-600/40 dark:via-slate-500/60 dark:to-slate-600/40 flex items-center justify-between gap-3 relative">
                {/* Decorative line accent */}
                <div className="absolute top-0 left-1/2 transform -translate-x-1/2 -translate-y-px w-12 h-0.5 bg-gradient-to-r from-orange-400 to-orange-500 rounded-full" />
                
                <div className="flex items-center gap-2 text-sm text-slate-600 dark:text-slate-300">
                  {activeFilterCount > 0 ? (
                    <>
                      <div className="w-3 h-3 bg-gradient-to-r from-green-400 to-green-500 rounded-full shadow-lg shadow-green-400/30" />
                      <span className="font-bold tracking-wide bg-gradient-to-r from-green-700 to-green-600 dark:from-green-400 dark:to-green-300 bg-clip-text text-transparent">
                        ⚡ {activeFilterCount} filter{activeFilterCount !== 1 ? 's' : ''} applied
                      </span>
                    </>
                  ) : (
                    <>
                      <Sparkles className="h-4 w-4 text-slate-400 dark:text-slate-500" />
                      <span className="font-medium italic text-slate-500 dark:text-slate-400">
                        ✨ No filters applied — <span className="font-semibold">ready to explore</span>
                      </span>
                    </>
                  )}
                </div>
                
                <div className="flex items-center gap-2">
                  <Button
                    type="button"
                    variant="outline"
                    size="sm"
                    onClick={handleReset}
                    disabled={!hasChanges && activeFilterCount === 0}
                    className="flex items-center gap-2 rounded-2xl bg-slate-50/80 border border-slate-200/50 text-slate-600 hover:bg-slate-100/80 hover:border-slate-300/60 dark:bg-slate-800/40 dark:border-slate-700/40 dark:text-slate-300 dark:hover:bg-slate-700/60 font-bold tracking-wider"
                  >
                    <RotateCcw className="h-4 w-4" />
                    <span className="text-sm">🔄 RESET</span>
                  </Button>
                  
                  <Button
                    type="submit"
                    size="sm"
                    disabled={!hasChanges}
                    className="flex items-center gap-2 rounded-2xl bg-slate-50/80 border border-slate-200/50 text-slate-600 hover:bg-slate-100/80 hover:border-slate-300/60 dark:bg-slate-800/40 dark:border-slate-700/40 dark:text-slate-300 dark:hover:bg-slate-700/60 font-bold tracking-wider"
                  >
                    <Search className="h-4 w-4" />
                    <span className="text-sm">🚀 APPLY</span>
                  </Button>
                </div>
              </div>
            </form>
          </CardContent>
        </Card>
      </div>
    </>
  );
}