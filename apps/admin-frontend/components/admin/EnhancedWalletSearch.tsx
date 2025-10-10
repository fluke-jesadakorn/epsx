/**
 * Enhanced Wallet Search Component
 * Advanced search and filtering for wallet management
 */
'use client';

import { Search, Filter, SortAsc, SortDesc, Calendar, Users, Settings, RefreshCw } from 'lucide-react';
import React, { useState, useEffect, useCallback, useMemo } from 'react';

import { Alert, AlertDescription } from '@/components/ui/alert';
import { Badge } from '@/components/ui/badge';
import { Button } from '@/components/ui/button';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select';
import { Skeleton } from '@/components/ui/skeleton';
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs';

interface WalletSearchResult {
  wallet_address: string;
  tier_level: string;
  permissions: Array<{
    permission: string;
    expires_at?: string;
    is_active: boolean;
  }>;
  groups: Array<{
    group_name: string;
    role?: string;
  }>;
  created_at: string;
  last_auth_at?: string;
  is_active: boolean;
  metadata?: any;
}

interface SearchFilters {
  search: string;
  tier: string;
  status: string;
  dateRange: string;
  hasPermissions: string;
  sortBy: string;
  sortOrder: 'asc' | 'desc';
}

interface SearchResponse {
  wallets: WalletSearchResult[];
  total_count: number;
  has_more: boolean;
  metadata: {
    page: number;
    limit: number;
    total_pages: number;
  };
}

const STATUS_OPTIONS = [
  { value: 'all', label: 'All Status' },
  { value: 'active', label: 'Active' },
  { value: 'inactive', label: 'Inactive' },
];

const DATE_RANGE_OPTIONS = [
  { value: 'all', label: 'All Time' },
  { value: '1d', label: 'Last 24 hours' },
  { value: '7d', label: 'Last 7 days' },
  { value: '30d', label: 'Last 30 days' },
  { value: '90d', label: 'Last 90 days' },
];

const SORT_OPTIONS = [
  { value: 'created_at', label: 'Date Created' },
  { value: 'last_auth_at', label: 'Last Activity' },
  { value: 'wallet_address', label: 'Wallet Address' },
  { value: 'tier_level', label: 'Tier Level' },
  { value: 'permissions_count', label: 'Permissions Count' },
];

/**
 *
 */
export function EnhancedWalletSearch() {
  const [results, setResults] = useState<SearchResponse | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [page, setPage] = useState(1);
  const [tiers, setTiers] = useState<Array<{ value: string; label: string }>>([{ value: 'all', label: 'All Tiers' }]);
  const [filters, setFilters] = useState<SearchFilters>({
    search: '',
    tier: '',
    status: '',
    dateRange: '',
    hasPermissions: '',
    sortBy: 'created_at',
    sortOrder: 'desc',
  });

  useEffect(() => {
    const fetchTiers = async () => {
      try {
        const backendUrl = process.env.NEXT_PUBLIC_BACKEND_URL || 'http://localhost:8080';
        const response = await fetch(`${backendUrl}/api/admin/tiers`, {
          credentials: 'include',
        });
        if (response.ok) {
          const data = await response.json();
          setTiers([
            { value: 'all', label: 'All Tiers' },
            ...data.map((tier: string) => ({
              value: tier.toLowerCase(),
              label: tier.charAt(0).toUpperCase() + tier.slice(1)
            }))
          ]);
        }
      } catch (err) {
        console.error('Failed to fetch tiers:', err);
      }
    };
    fetchTiers();
  }, []);

  const searchWallets = useCallback(async (newPage = 1, newFilters = filters) => {
    setLoading(true);
    setError(null);

    try {
      const searchParams = new URLSearchParams({
        page: newPage.toString(),
        limit: '20',
        search: newFilters.search,
        tier: newFilters.tier,
        status: newFilters.status,
        date_range: newFilters.dateRange,
        has_permissions: newFilters.hasPermissions,
        sort_by: newFilters.sortBy,
        sort_order: newFilters.sortOrder,
      });

      // Remove empty parameters
      const cleanParams = new URLSearchParams();
      searchParams.forEach((value, key) => {
        if (value.trim() !== '') {
          cleanParams.append(key, value);
        }
      });

      const backendUrl = process.env.NEXT_PUBLIC_BACKEND_URL || 'http://localhost:8080';
      const response = await fetch(`${backendUrl}/api/admin/wallets/search?${cleanParams.toString()}`, {
        credentials: 'include',
        headers: {
          'Content-Type': 'application/json',
        },
      });

      if (!response.ok) {
        throw new Error(`Search failed: ${response.status}`);
      }

      const data = await response.json();
      setResults(data);
      setPage(newPage);
    } catch (err) {
      // eslint-disable-next-line no-console
      console.error('Search error:', err);
      setError(err instanceof Error ? err.message : 'Search failed');
    } finally {
      setLoading(false);
    }
  }, [filters]);

  const handleFilterChange = useCallback((key: keyof SearchFilters, value: string) => {
    const newFilters = { ...filters, [key]: value };
    setFilters(newFilters);
    
    // Auto-search with debounce for text inputs
    if (key === 'search') {
      const timeoutId = setTimeout(() => {
        searchWallets(1, newFilters);
      }, 500);
      return () => clearTimeout(timeoutId);
    } else {
      // Immediate search for dropdown changes
      searchWallets(1, newFilters);
    }
  }, [filters, searchWallets]);

  // Initial search
  useEffect(() => {
    searchWallets();
  }, []);

  const formatWalletAddress = (address: string) => {
    return `${address.slice(0, 6)}...${address.slice(-4)}`;
  };

  const formatTimeAgo = (timestamp: string) => {
    const date = new Date(timestamp);
    const now = new Date();
    const diffInHours = Math.floor((now.getTime() - date.getTime()) / (1000 * 60 * 60));
    
    if (diffInHours < 1) {return 'Just now';}
    if (diffInHours < 24) {return `${diffInHours}h ago`;}
    const diffInDays = Math.floor(diffInHours / 24);
    if (diffInDays < 30) {return `${diffInDays}d ago`;}
    return date.toLocaleDateString();
  };

  const getTierColor = (tier: string | null | undefined) => {
    if (!tier) return 'outline';
    switch (tier.toLowerCase()) {
      case 'admin': return 'destructive';
      case 'platinum': case 'diamond': return 'default';
      case 'gold': return 'secondary';
      case 'silver': return 'outline';
      default: return 'outline';
    }
  };

  const getActivePermissionsCount = (permissions: WalletSearchResult['permissions']) => {
    return permissions.filter(p => p.is_active).length;
  };

  return (
    <div className="space-y-6">
      {/* Search and Filters */}
      <Card className="bg-white/80 dark:bg-gray-800/80 backdrop-blur-sm border-2 border-blue-300/50 dark:border-blue-700/50 shadow-xl">
        <CardHeader className="pb-4">
          <CardTitle className="flex items-center gap-3 text-xl">
            <div className="text-2xl">🔍</div>
            <span className="bg-gradient-to-r from-blue-600 to-cyan-600 bg-clip-text text-transparent font-bold">
              Search & Filters
            </span>
          </CardTitle>
          <CardDescription className="text-base text-gray-600 dark:text-gray-400">
            Find wallets by address, tier, status, or activity
          </CardDescription>
        </CardHeader>
        <CardContent>
          <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4 gap-4">
            {/* Search Input */}
            <div className="md:col-span-2 lg:col-span-2 xl:col-span-2">
              <Label htmlFor="search" className="text-sm font-medium">Search Wallets</Label>
              <Input
                id="search"
                placeholder="0x... or wallet address"
                value={filters.search}
                onChange={(e) => handleFilterChange('search', e.target.value)}
                className="font-mono text-sm h-10 mt-1.5"
              />
            </div>

            {/* Tier Filter */}
            <div>
              <Label className="text-sm font-medium">Tier Level</Label>
              <Select value={filters.tier} onValueChange={(value) => handleFilterChange('tier', value)}>
                <SelectTrigger className="h-10 mt-1.5">
                  <SelectValue placeholder="All Tiers" />
                </SelectTrigger>
                <SelectContent>
                  {tiers.map((option) => (
                    <SelectItem key={option.value} value={option.value}>
                      {option.label}
                    </SelectItem>
                  ))}
                </SelectContent>
              </Select>
            </div>

            {/* Status Filter */}
            <div>
              <Label className="text-sm font-medium">Status</Label>
              <Select value={filters.status} onValueChange={(value) => handleFilterChange('status', value)}>
                <SelectTrigger className="h-10 mt-1.5">
                  <SelectValue placeholder="All Status" />
                </SelectTrigger>
                <SelectContent>
                  {STATUS_OPTIONS.map((option) => (
                    <SelectItem key={option.value} value={option.value}>
                      {option.label}
                    </SelectItem>
                  ))}
                </SelectContent>
              </Select>
            </div>

            {/* Date Range Filter */}
            <div>
              <Label className="text-sm font-medium">Date Range</Label>
              <Select value={filters.dateRange} onValueChange={(value) => handleFilterChange('dateRange', value)}>
                <SelectTrigger className="h-10 mt-1.5">
                  <SelectValue placeholder="All Time" />
                </SelectTrigger>
                <SelectContent>
                  {DATE_RANGE_OPTIONS.map((option) => (
                    <SelectItem key={option.value} value={option.value}>
                      {option.label}
                    </SelectItem>
                  ))}
                </SelectContent>
              </Select>
            </div>

            {/* Sort Options */}
            <div>
              <Label className="text-sm font-medium">Sort By</Label>
              <Select value={filters.sortBy} onValueChange={(value) => handleFilterChange('sortBy', value)}>
                <SelectTrigger className="h-10 mt-1.5">
                  <SelectValue />
                </SelectTrigger>
                <SelectContent>
                  {SORT_OPTIONS.map((option) => (
                    <SelectItem key={option.value} value={option.value}>
                      {option.label}
                    </SelectItem>
                  ))}
                </SelectContent>
              </Select>
            </div>

            {/* Sort Order */}
            <div>
              <Label className="text-sm font-medium">Order</Label>
              <Select value={filters.sortOrder} onValueChange={(value) => handleFilterChange('sortOrder', value as 'asc' | 'desc')}>
                <SelectTrigger className="h-10 mt-1.5">
                  <SelectValue />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem value="desc">
                    <div className="flex items-center gap-2">
                      <SortDesc className="h-4 w-4" />
                      Descending
                    </div>
                  </SelectItem>
                  <SelectItem value="asc">
                    <div className="flex items-center gap-2">
                      <SortAsc className="h-4 w-4" />
                      Ascending
                    </div>
                  </SelectItem>
                </SelectContent>
              </Select>
            </div>

            {/* Action Buttons */}
            <div className="md:col-span-2 lg:col-span-3 xl:col-span-2 flex gap-3 items-end">
              <Button
                onClick={() => searchWallets(1)}
                disabled={loading}
                className="flex items-center gap-2 h-10 px-6 bg-gradient-to-r from-blue-500 to-cyan-500 hover:from-blue-600 hover:to-cyan-600 text-white border-0"
              >
                <RefreshCw className={`h-4 w-4 ${loading ? 'animate-spin' : ''}`} />
                Search
              </Button>
              <Button
                variant="outline"
                onClick={() => {
                  setFilters({
                    search: '',
                    tier: '',
                    status: '',
                    dateRange: '',
                    hasPermissions: '',
                    sortBy: 'created_at',
                    sortOrder: 'desc',
                  });
                  searchWallets(1, {
                    search: '',
                    tier: '',
                    status: '',
                    dateRange: '',
                    hasPermissions: '',
                    sortBy: 'created_at',
                    sortOrder: 'desc',
                  });
                }}
                className="h-10 px-6 border-2 border-gray-300 dark:border-gray-600 hover:bg-gray-100 dark:hover:bg-gray-800"
              >
                Clear
              </Button>
            </div>
          </div>
        </CardContent>
      </Card>

      {/* Results */}
      <Card className="bg-white/80 dark:bg-gray-800/80 backdrop-blur-sm border-2 border-purple-300/50 dark:border-purple-700/50 shadow-xl">
        <CardHeader className="pb-4">
          <div className="flex items-center justify-between">
            <div>
              <CardTitle className="flex items-center gap-3 text-xl">
                <div className="text-2xl">📊</div>
                <span className="bg-gradient-to-r from-purple-600 to-pink-600 bg-clip-text text-transparent font-bold">
                  Results
                </span>
                {results && (
                  <Badge className="bg-gradient-to-r from-purple-400 to-pink-500 text-white border-0 text-sm font-medium px-3 py-1">
                    {results.total_count}
                  </Badge>
                )}
              </CardTitle>
              <CardDescription className="text-base mt-1 text-gray-600 dark:text-gray-400">
                {results ? `Showing ${results.wallets.length} of ${results.total_count} wallets` : 'Enter search criteria above'}
              </CardDescription>
            </div>
          </div>
        </CardHeader>
        <CardContent>
          {error && (
            <Alert className="mb-4 border-red-200 bg-red-50">
              <AlertDescription>{error}</AlertDescription>
            </Alert>
          )}

          {loading && (
            <div className="space-y-3">
              {Array.from({ length: 5 }).map((_, i) => (
                <div key={i} className="bg-gradient-to-r from-blue-400/10 via-purple-400/10 to-pink-400/10 rounded-2xl p-0.5">
                  <div className="bg-white/95 dark:bg-gray-900/95 backdrop-blur-xl rounded-2xl p-5 flex items-center justify-between">
                    <div className="flex items-center gap-4">
                      <Skeleton className="h-12 w-12 rounded-xl" />
                      <div className="space-y-2">
                        <Skeleton className="h-4 w-40" />
                        <Skeleton className="h-3 w-32" />
                      </div>
                    </div>
                    <div className="space-y-2 text-right">
                      <Skeleton className="h-3 w-24" />
                      <Skeleton className="h-3 w-20" />
                      <Skeleton className="h-5 w-16" />
                    </div>
                  </div>
                </div>
              ))}
            </div>
          )}

          {!loading && results && results.wallets.length === 0 && (
            <div className="text-center py-12 text-muted-foreground">
              <Search className="h-16 w-16 mx-auto mb-4 opacity-40" />
              <p className="text-lg font-semibold mb-2">No wallets found</p>
              <p className="text-sm">Try different search criteria or filters</p>
            </div>
          )}

          {!loading && results && results.wallets.length > 0 && (
            <div className="space-y-3">
              {results.wallets.map((wallet) => (
                <div
                  key={wallet.wallet_address}
                  className="relative overflow-hidden rounded-2xl bg-gradient-to-r from-blue-400/10 via-purple-400/10 to-pink-400/10 p-0.5 hover:scale-[1.02] transition-all duration-300"
                >
                  <div className="bg-white/95 dark:bg-gray-900/95 backdrop-blur-xl rounded-2xl p-5 flex items-center justify-between">
                    <div className="absolute top-4 right-4 w-3 h-3 bg-gradient-to-r from-blue-400 to-cyan-500 rounded-full blur-sm opacity-60"></div>
                    <div className="flex items-center gap-4">
                      <div className="relative h-12 w-12 bg-gradient-to-br from-blue-500 via-purple-500 to-pink-500 rounded-xl flex items-center justify-center text-white font-bold shrink-0 shadow-lg">
                        {wallet.wallet_address.slice(2, 4).toUpperCase()}
                        <div className="absolute -top-1 -right-1 w-3 h-3 bg-green-400 rounded-full border-2 border-white"></div>
                      </div>
                      <div className="min-w-0">
                        <div className="font-mono text-sm font-semibold mb-1.5">
                          {formatWalletAddress(wallet.wallet_address)}
                        </div>
                        <div className="flex items-center gap-2 flex-wrap">
                          <Badge variant={getTierColor(wallet.tier_level)} className="text-xs font-medium">
                            {wallet.tier_level}
                          </Badge>
                          <span className="text-xs text-muted-foreground">
                            {getActivePermissionsCount(wallet.permissions)} permissions
                          </span>
                          {wallet.groups.length > 0 && (
                            <span className="text-xs text-muted-foreground">
                              • {wallet.groups.length} groups
                            </span>
                          )}
                        </div>
                      </div>
                    </div>
                    <div className="text-right shrink-0 ml-4">
                      <div className="text-xs text-gray-500 dark:text-gray-400 mb-1">
                        Created {formatTimeAgo(wallet.created_at)}
                      </div>
                      {wallet.last_auth_at && (
                        <div className="text-xs text-gray-500 dark:text-gray-400 mb-2">
                          Active {formatTimeAgo(wallet.last_auth_at)}
                        </div>
                      )}
                      <Badge
                        className={wallet.is_active
                          ? 'bg-gradient-to-r from-green-400 to-emerald-500 text-white border-0'
                          : 'bg-gray-200 dark:bg-gray-700 text-gray-600 dark:text-gray-400 border-0'
                        }
                      >
                        {wallet.is_active ? '🟢 Active' : '⚪ Inactive'}
                      </Badge>
                    </div>
                  </div>
                </div>
              ))}
            </div>
          )}

          {/* Pagination */}
          {results && results.has_more && (
            <div className="flex justify-center mt-8 pt-6 border-t">
              <Button
                variant="outline"
                onClick={() => searchWallets(page + 1)}
                disabled={loading}
                className="h-10 px-8"
              >
                Load More ({results.total_count - results.wallets.length} remaining)
              </Button>
            </div>
          )}
        </CardContent>
      </Card>
    </div>
  );
}