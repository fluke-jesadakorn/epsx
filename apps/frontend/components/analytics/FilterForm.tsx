'use client';

import { useState, useEffect, useRef } from 'react';
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
import { Sheet, SheetContent, SheetHeader, SheetTitle, SheetTrigger } from '@/components/ui/sheet';
import { 
  Filter,
  Globe,
  TrendingUp,
  DollarSign,
  Search,
  RotateCcw,
  X,
  Sparkles,
  Activity,
  Eye,
  EyeOff,
  ChevronDown
} from 'lucide-react';
import type { FilterOptions, EPSQueryParams } from '@/lib/analytics-server';

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
    ...(countries || [])
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

  const selectedCountryLabel = options.find(c => c.value === selectedCountry)?.label || 'All Countries';

  const handleScroll = () => {
    if (!scrollRef.current) return;
    
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
          <SelectTrigger className="bg-white/80 dark:bg-gray-800/80 backdrop-blur-sm border-gray-300/50 dark:border-gray-600/50 rounded-2xl hover:bg-white/90 hover:dark:bg-gray-700/90 focus:bg-white/90 focus:dark:bg-gray-700/90 focus:border-blue-500 focus:ring-2 focus:ring-blue-200 dark:focus:ring-blue-400/50 transition-all shadow-sm min-h-[56px]">
            <div className="flex items-center gap-3">
              <div className="w-6 h-6 bg-blue-500 dark:bg-blue-400 rounded-full flex items-center justify-center shadow-sm flex-shrink-0">
                <div className="w-3 h-3 bg-white dark:bg-gray-900 rounded-full" />
              </div>
              <SelectValue placeholder="All Countries" className="text-gray-800 dark:text-gray-200 font-medium" />
            </div>
          </SelectTrigger>
          <SelectContent className="max-h-64 bg-white/95 dark:bg-gray-800/95 backdrop-blur-md border-gray-200 dark:border-gray-600 rounded-2xl shadow-lg">
            <SelectItem value="all">
              <div className="flex items-center gap-3">
                <div className="w-5 h-5 bg-gray-400 dark:bg-gray-500 rounded-full flex items-center justify-center shadow-sm flex-shrink-0">
                  <Globe className="h-3 w-3 text-white" />
                </div>
                <span className="font-medium">All Countries</span>
              </div>
            </SelectItem>
            {countries?.map(country => (
              <SelectItem key={country.value} value={country.value}>
                <div className="flex items-center gap-3">
                  <div className="w-5 h-5 bg-blue-500 dark:bg-blue-400 rounded-full flex items-center justify-center shadow-sm flex-shrink-0">
                    <div className="w-2 h-2 bg-white dark:bg-gray-900 rounded-full" />
                  </div>
                  <span className="font-medium">{country.label}</span>
                </div>
              </SelectItem>
            ))}
          </SelectContent>
        </Select>
        {selectedCountry && selectedCountry !== 'all' && (
          <button
            type="button"
            onClick={() => onCountryChange('all')}
            className="absolute right-8 top-1/2 -translate-y-1/2 p-1 text-gray-400 hover:text-red-500 transition-colors"
          >
            <X className="h-3 w-3" />
          </button>
        )}
      </div>

      {/* Mobile iPhone-style Sheet - visible on mobile only */}
      <div className="md:hidden">
        <Sheet open={isSheetOpen} onOpenChange={setIsSheetOpen}>
          <SheetTrigger asChild>
            <button
              type="button"
              className="w-full flex items-center justify-between p-4 bg-white/80 dark:bg-gray-800/80 backdrop-blur-sm border border-gray-300/50 dark:border-gray-600/50 rounded-2xl hover:bg-white/90 hover:dark:bg-gray-700/90 focus:bg-white/90 focus:dark:bg-gray-700/90 focus:outline-none transition-all shadow-sm min-h-[56px] touch-manipulation"
            >
              <div className="flex items-center gap-3">
                <div className="w-6 h-6 bg-blue-500 dark:bg-blue-400 rounded-full flex items-center justify-center shadow-sm flex-shrink-0">
                  <div className="w-3 h-3 bg-white dark:bg-gray-900 rounded-full" />
                </div>
                <span className="text-left font-medium text-gray-800 dark:text-gray-200 truncate">
                  {selectedCountryLabel}
                </span>
              </div>
              <ChevronDown className="h-5 w-5 text-gray-600 dark:text-gray-400 flex-shrink-0" />
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
    country: currentParams.country || 'all',
    sort_by: currentParams.sort_by || 'growth_factor',
    min_eps: currentParams.min_eps?.toString() || '',
    min_growth: currentParams.min_growth?.toString() || '',
  });

  const [hasChanges, setHasChanges] = useState(false);
  const [isCompact, setIsCompact] = useState(false);
  const [isAnimating, setIsAnimating] = useState(false);
  const [isMounted, setIsMounted] = useState(false);

  // Handle client-side mounting only
  useEffect(() => {
    setIsMounted(true);
  }, []);

  // Check if any filters have changed from current state
  useEffect(() => {
    const changes = 
      filters.country !== (currentParams.country || 'all') ||
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
    if (filters.sort_by) params.set('sort_by', filters.sort_by);
    if (filters.min_eps) params.set('min_eps', filters.min_eps);
    if (filters.min_growth) params.set('min_growth', filters.min_growth);

    router.push(`/analytics?${params.toString()}`);
    
    setTimeout(() => setIsAnimating(false), 1000);
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

  const clearFilter = (filterKey: string) => {
    if (filterKey === 'country') {
      updateFilter(filterKey, 'all');
    } else {
      updateFilter(filterKey, '');
    }
  };

  const getActiveFilterCount = () => {
    let count = 0;
    if (currentParams.country) count++;
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
          <div className="grid gap-6 grid-cols-1">
            
            {/* Country Filter */}
            <div className="space-y-2">
              <Label htmlFor="country" className="flex items-center gap-2 text-sm font-semibold text-gray-700 dark:text-gray-200">
                <Globe className="h-4 w-4 text-blue-600 dark:text-blue-400" />
                Country
              </Label>
              <div className="relative">
                {!isMounted ? (
                  /* Loading state - prevent hydration mismatch */
                  <div className="w-full p-4 bg-gray-100 dark:bg-gray-800 rounded-lg animate-pulse">
                    <div className="h-4 bg-gray-300 dark:bg-gray-600 rounded w-32"></div>
                  </div>
                ) : (
                  /* Smart Country Selector - handles mobile/desktop automatically */
                  <SmartCountrySelector 
                    countries={filterOptions.countries} 
                    selectedCountry={filters.country}
                    onCountryChange={(value) => updateFilter('country', value)}
                  />
                )}
              </div>
            </div>

          </div>

          {/* Enhanced Action Buttons */}
          <div className={`pt-6 border-t border-gradient-to-r border-pink-200/60 dark:border-pink-400/20 ${
            !isMounted ? 'space-y-4' : 'flex items-center justify-between md:flex md:items-center md:justify-between space-y-4 md:space-y-0'
          }`}>
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
            
            <div className={`flex items-center gap-3 ${!isMounted ? 'w-full' : 'w-full md:w-auto'}`}>
              <Button
                type="button"
                variant="outline"
                size={!isMounted ? "default" : "sm"}
                onClick={handleReset}
                disabled={!hasChanges && activeFilterCount === 0}
                className={`flex items-center gap-2 border-gray-300 hover:border-red-400 hover:bg-red-50 dark:hover:bg-red-900/20 transition-all ${
                  !isMounted ? 'flex-1 min-h-[48px]' : 'flex-1 min-h-[48px] md:flex-none md:h-auto'
                }`}
              >
                <RotateCcw className="h-4 w-4" />
                Reset
              </Button>
              
              <Button
                type="submit"
                size={!isMounted ? "default" : "sm"}
                disabled={!hasChanges}
                className={`flex items-center gap-2 transition-all duration-300 ${
                  hasChanges 
                    ? 'bg-gradient-to-r from-pink-500 to-orange-500 hover:from-pink-600 hover:to-orange-600 shadow-lg shadow-pink-500/30 animate-pulse' 
                    : 'bg-gray-400'
                } ${isAnimating ? 'scale-105 shadow-xl' : ''} ${
                  !isMounted ? 'flex-1 min-h-[48px]' : 'flex-1 min-h-[48px] md:flex-none md:h-auto'
                }`}
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