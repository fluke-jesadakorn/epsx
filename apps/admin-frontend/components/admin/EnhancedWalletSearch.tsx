/**
 * Enhanced Wallet Search Component
 * Advanced search and filtering for wallet management
 */
'use client';

import { useAuthenticatedFetch } from '@/hooks/useAuthenticatedFetch';
import { useSharedAuth } from '@/shared/components/auth/Provider';
import { RefreshCw, Search, SortAsc, SortDesc } from 'lucide-react';
import { useCallback, useEffect, useState } from 'react';

import { Alert, AlertDescription } from '@/components/ui/alert';
import { Badge } from '@/components/ui/badge';
import { Button } from '@/components/ui/button';
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from '@/components/ui/card';
import { Label } from '@/components/ui/label';
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select';
import { Skeleton } from '@/components/ui/skeleton';
import { WalletAutocomplete } from '@/components/ui/WalletAutocomplete';

interface WalletSearchResult {
  wallet_address: string;
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
  { value: 'permissions_count', label: 'Permissions Count' },
];

/**
 *
 */
export function EnhancedWalletSearch() {
  const { isAuthenticated, isLoading: authLoading } = useSharedAuth();
  const { fetchWithAuth } = useAuthenticatedFetch();
  const [results, setResults] = useState<SearchResponse | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [page, setPage] = useState(1);
  const [searchTimeoutId, setSearchTimeoutId] = useState<any>(null);
  const [tiers, setTiers] = useState<Array<{ value: string; label: string }>>([
    { value: 'all', label: 'All Tiers' },
  ]);
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
    // Debug: Check authentication state
    console.log('🔍 EnhancedWalletSearch: useEffect triggered', {
      isAuthenticated,
      authLoading,
    });

    // Only fetch tiers if user is authenticated
    if (!isAuthenticated || authLoading) {
      console.log('⏸️ EnhancedWalletSearch: Skipping API call - auth not ready');
      return;
    }

    console.log('🚀 EnhancedWalletSearch: Starting to fetch tiers');

    const fetchTiers = async () => {
      try {
        const response = await fetchWithAuth('/api/v1/admin/tiers');
        const data = await response.json();
        setTiers([
          { value: 'all', label: 'All Tiers' },
          ...data.map((tier: string) => ({
            value: tier.toLowerCase(),
            label: tier.charAt(0).toUpperCase() + tier.slice(1),
          })),
        ]);
      } catch (err) {
        console.error('Failed to fetch tiers:', err);
        setError(err instanceof Error ? err.message : 'Failed to load tiers');
      }
    };
    fetchTiers();
  }, [isAuthenticated, authLoading, fetchWithAuth]);

  const searchWallets = useCallback(
    async (newPage = 1, newFilters = filters) => {
      // Only proceed if user is authenticated
      if (!isAuthenticated) {
        setError('Authentication required to search wallets');
        setLoading(false);
        return;
      }

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

        const response = await fetchWithAuth(
          `/api/v1/admin/wallets/search?${cleanParams.toString()}`
        );
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
    },
    [filters]
  );

  const handleFilterChange = useCallback(
    (key: keyof SearchFilters, value: string) => {
      const newFilters = { ...filters, [key]: value };
      setFilters(newFilters);

      // Auto-search with debounce for text inputs
      if (key === 'search') {
        if (searchTimeoutId) clearTimeout(searchTimeoutId);
        const id = setTimeout(() => {
          searchWallets(1, newFilters);
        }, 500);
        setSearchTimeoutId(id);
      } else {
        // Immediate search for dropdown changes
        searchWallets(1, newFilters);
      }
    },
    [filters, searchWallets, searchTimeoutId]
  );

  // Initial search
  useEffect(() => {
    // Only search if user is authenticated
    if (isAuthenticated && !authLoading) {
      searchWallets();
    }
  }, [isAuthenticated, authLoading, searchWallets]);

  const formatWalletAddress = (address: string) => {
    return `${address.slice(0, 6)}...${address.slice(-4)}`;
  };

  const formatTimeAgo = (timestamp: string) => {
    const date = new Date(timestamp);
    const now = new Date();
    const diffInHours = Math.floor(
      (now.getTime() - date.getTime()) / (1000 * 60 * 60)
    );

    if (diffInHours < 1) {
      return 'Just now';
    }
    if (diffInHours < 24) {
      return `${diffInHours}h ago`;
    }
    const diffInDays = Math.floor(diffInHours / 24);
    if (diffInDays < 30) {
      return `${diffInDays}d ago`;
    }
    return date.toLocaleDateString();
  };

  const getTierColor = (tier: string | null | undefined) => {
    if (!tier) return 'outline';
    switch (tier.toLowerCase()) {
      case 'admin':
        return 'destructive';
      case 'platinum':
      case 'diamond':
        return 'default';
      case 'gold':
        return 'secondary';
      case 'silver':
        return 'outline';
      default:
        return 'outline';
    }
  };

  const getActivePermissionsCount = (
    permissions: WalletSearchResult['permissions']
  ) => {
    return permissions.filter(p => p.is_active).length;
  };

  // Show authentication loading state
  if (authLoading) {
    return (
      <div className="flex items-center justify-center p-8">
        <div className="text-center">
          <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-blue-600 mx-auto mb-4"></div>
          <p className="text-gray-600 dark:text-gray-400">Checking authentication...</p>
        </div>
      </div>
    );
  }

  // Show unauthenticated state
  if (!isAuthenticated) {
    return (
      <div className="flex items-center justify-center p-8">
        <div className="text-center max-w-md">
          <div className="text-4xl mb-4">🔐</div>
          <h3 className="text-lg font-semibold text-gray-900 dark:text-white mb-2">
            Authentication Required
          </h3>
          <p className="text-gray-600 dark:text-gray-400">
            Please connect your wallet to access the wallet search functionality.
          </p>
        </div>
      </div>
    );
  }

  return (
    <div className="space-y-6">
      {/* Search and Filters */}
      <Card className="border-2 border-blue-300/50 bg-white/80 shadow-xl backdrop-blur-sm dark:border-blue-700/50 dark:bg-gray-800/80">
        <CardHeader className="pb-4">
          <CardTitle className="flex items-center gap-3 text-xl">
            <div className="text-2xl">🔍</div>
            <span className="bg-gradient-to-r from-blue-600 to-cyan-600 bg-clip-text font-bold text-transparent">
              Search & Filters
            </span>
          </CardTitle>
          <CardDescription className="text-base text-gray-600 dark:text-gray-400">
            Find wallets by address, tier, status, or activity
          </CardDescription>
        </CardHeader>
        <CardContent>
          <div className="grid grid-cols-1 gap-4 md:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4">
            {/* Search Input with Autocomplete */}
            <div className="md:col-span-2 lg:col-span-2 xl:col-span-2">
              <Label htmlFor="search" className="text-sm font-medium">
                Search Wallets
              </Label>
              <WalletAutocomplete
                value={filters.search}
                onChange={(value) => handleFilterChange('search', value)}
                onSelect={(wallet) => {
                  // When a wallet is selected from autocomplete, search immediately
                  const newFilters = { ...filters, search: wallet.wallet_address };
                  setFilters(newFilters);
                  searchWallets(1, newFilters);
                }}
                placeholder="0x... or wallet address"
                className="mt-1.5"
              />
            </div>

            {/* Tier Filter */}
            <div>
              <Label className="text-sm font-medium">Tier Level</Label>
              <Select
                value={filters.tier}
                onValueChange={value => handleFilterChange('tier', value)}
              >
                <SelectTrigger className="mt-1.5 h-10 bg-white dark:bg-gray-800 border-gray-300 dark:border-gray-600">
                  <SelectValue placeholder="All Tiers" />
                </SelectTrigger>
                <SelectContent className="bg-white dark:bg-gray-800 border border-gray-300 dark:border-gray-600">
                  {tiers.map(option => (
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
              <Select
                value={filters.status}
                onValueChange={value => handleFilterChange('status', value)}
              >
                <SelectTrigger className="mt-1.5 h-10 bg-white dark:bg-gray-800 border-gray-300 dark:border-gray-600">
                  <SelectValue placeholder="All Status" />
                </SelectTrigger>
                <SelectContent className="bg-white dark:bg-gray-800 border border-gray-300 dark:border-gray-600">
                  {STATUS_OPTIONS.map(option => (
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
              <Select
                value={filters.dateRange}
                onValueChange={value => handleFilterChange('dateRange', value)}
              >
                <SelectTrigger className="mt-1.5 h-10 bg-white dark:bg-gray-800 border-gray-300 dark:border-gray-600">
                  <SelectValue placeholder="All Time" />
                </SelectTrigger>
                <SelectContent className="bg-white dark:bg-gray-800 border border-gray-300 dark:border-gray-600">
                  {DATE_RANGE_OPTIONS.map(option => (
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
              <Select
                value={filters.sortBy}
                onValueChange={value => handleFilterChange('sortBy', value)}
              >
                <SelectTrigger className="mt-1.5 h-10 bg-white dark:bg-gray-800 border-gray-300 dark:border-gray-600">
                  <SelectValue />
                </SelectTrigger>
                <SelectContent className="bg-white dark:bg-gray-800 border border-gray-300 dark:border-gray-600">
                  {SORT_OPTIONS.map(option => (
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
              <Select
                value={filters.sortOrder}
                onValueChange={value =>
                  handleFilterChange('sortOrder', value as 'asc' | 'desc')
                }
              >
                <SelectTrigger className="mt-1.5 h-10 bg-white dark:bg-gray-800 border-gray-300 dark:border-gray-600">
                  <SelectValue />
                </SelectTrigger>
                <SelectContent className="bg-white dark:bg-gray-800 border border-gray-300 dark:border-gray-600">
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
            <div className="flex items-end gap-3 md:col-span-2 lg:col-span-3 xl:col-span-2">
              <Button
                onClick={() => searchWallets(1)}
                disabled={loading}
                className="flex h-10 items-center gap-2 border-0 bg-gradient-to-r from-blue-500 to-cyan-500 px-6 text-white hover:from-blue-600 hover:to-cyan-600"
              >
                <RefreshCw
                  className={`h-4 w-4 ${loading ? 'animate-spin' : ''}`}
                />
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
                className="h-10 border-2 border-gray-300 px-6 hover:bg-gray-100 dark:border-gray-600 dark:hover:bg-gray-800"
              >
                Clear
              </Button>
            </div>
          </div>
        </CardContent>
      </Card>

      {/* Results */}
      <Card className="border-2 border-purple-300/50 bg-white/80 shadow-xl backdrop-blur-sm dark:border-purple-700/50 dark:bg-gray-800/80">
        <CardHeader className="pb-4">
          <div className="flex items-center justify-between">
            <div>
              <CardTitle className="flex items-center gap-3 text-xl">
                <div className="text-2xl">📊</div>
                <span className="bg-gradient-to-r from-purple-600 to-pink-600 bg-clip-text font-bold text-transparent">
                  Results
                </span>
                {results && (
                  <Badge className="border-0 bg-gradient-to-r from-purple-400 to-pink-500 px-3 py-1 text-sm font-medium text-white">
                    {results.total_count}
                  </Badge>
                )}
              </CardTitle>
              <CardDescription className="mt-1 text-base text-gray-600 dark:text-gray-400">
                {results
                  ? `Showing ${results.wallets.length} of ${results.total_count} wallets`
                  : 'Enter search criteria above'}
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
                <div
                  key={i}
                  className="rounded-2xl bg-gradient-to-r from-blue-400/10 via-purple-400/10 to-pink-400/10 p-0.5"
                >
                  <div className="flex items-center justify-between rounded-2xl bg-white/95 p-5 backdrop-blur-xl dark:bg-gray-900/95">
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
            <div className="text-muted-foreground py-12 text-center">
              <Search className="mx-auto mb-4 h-16 w-16 opacity-40" />
              <p className="mb-2 text-lg font-semibold">No wallets found</p>
              <p className="text-sm">
                Try different search criteria or filters
              </p>
            </div>
          )}

          {!loading && results && results.wallets.length > 0 && (
            <div className="space-y-3">
              {results.wallets.map(wallet => (
                <div
                  key={wallet.wallet_address}
                  className="relative overflow-hidden rounded-2xl bg-gradient-to-r from-blue-400/10 via-purple-400/10 to-pink-400/10 p-0.5 transition-all duration-300 hover:scale-[1.02]"
                >
                  <div className="flex items-center justify-between rounded-2xl bg-white/95 p-5 backdrop-blur-xl dark:bg-gray-900/95">
                    <div className="absolute top-4 right-4 h-3 w-3 rounded-full bg-gradient-to-r from-blue-400 to-cyan-500 opacity-60 blur-sm"></div>
                    <div className="flex items-center gap-4">
                      <div className="relative flex h-12 w-12 shrink-0 items-center justify-center rounded-xl bg-gradient-to-br from-blue-500 via-purple-500 to-pink-500 font-bold text-white shadow-lg">
                        {wallet.wallet_address.slice(2, 4).toUpperCase()}
                        <div className="absolute -top-1 -right-1 h-3 w-3 rounded-full border-2 border-white bg-green-400"></div>
                      </div>
                      <div className="min-w-0">
                        <div className="mb-1.5 font-mono text-sm font-semibold">
                          {formatWalletAddress(wallet.wallet_address)}
                        </div>
                        <div className="flex flex-wrap items-center gap-2">
                          <span className="text-muted-foreground text-xs">
                            {getActivePermissionsCount(wallet.permissions)}{' '}
                            permissions
                          </span>
                          {wallet.groups.length > 0 && (
                            <span className="text-muted-foreground text-xs">
                              • {wallet.groups.length} groups
                            </span>
                          )}
                        </div>
                      </div>
                    </div>
                    <div className="ml-4 shrink-0 text-right">
                      <div className="mb-1 text-xs text-gray-500 dark:text-gray-400">
                        Created {formatTimeAgo(wallet.created_at)}
                      </div>
                      {wallet.last_auth_at && (
                        <div className="mb-2 text-xs text-gray-500 dark:text-gray-400">
                          Active {formatTimeAgo(wallet.last_auth_at)}
                        </div>
                      )}
                      <Badge
                        className={
                          wallet.is_active
                            ? 'border-0 bg-gradient-to-r from-green-400 to-emerald-500 text-white'
                            : 'border-0 bg-gray-200 text-gray-600 dark:bg-gray-700 dark:text-gray-400'
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
            <div className="mt-8 flex justify-center border-t pt-6">
              <Button
                variant="outline"
                onClick={() => searchWallets(page + 1)}
                disabled={loading}
                className="h-10 px-8"
              >
                Load More ({results.total_count - results.wallets.length}{' '}
                remaining)
              </Button>
            </div>
          )}
        </CardContent>
      </Card>
    </div>
  );
}
