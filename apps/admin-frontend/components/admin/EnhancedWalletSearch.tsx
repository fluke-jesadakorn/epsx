/**
 * Enhanced Wallet Search Component
 * Advanced search and filtering for wallet management
 */
'use client';

import React, { useState, useEffect, useCallback, useMemo } from 'react';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';
import { Badge } from '@/components/ui/badge';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select';
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs';
import { Alert, AlertDescription } from '@/components/ui/alert';
import { Skeleton } from '@/components/ui/skeleton';
import { Search, Filter, SortAsc, SortDesc, Calendar, Users, Settings, RefreshCw } from 'lucide-react';

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

const TIER_OPTIONS = [
  { value: '', label: 'All Tiers' },
  { value: 'admin', label: 'Admin' },
  { value: 'platinum', label: 'Platinum' },
  { value: 'gold', label: 'Gold' },
  { value: 'silver', label: 'Silver' },
  { value: 'bronze', label: 'Bronze' },
];

const STATUS_OPTIONS = [
  { value: '', label: 'All Status' },
  { value: 'active', label: 'Active' },
  { value: 'inactive', label: 'Inactive' },
];

const DATE_RANGE_OPTIONS = [
  { value: '', label: 'All Time' },
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

export function EnhancedWalletSearch() {
  const [results, setResults] = useState<SearchResponse | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [page, setPage] = useState(1);
  const [filters, setFilters] = useState<SearchFilters>({
    search: '',
    tier: '',
    status: '',
    dateRange: '',
    hasPermissions: '',
    sortBy: 'created_at',
    sortOrder: 'desc',
  });

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

      const response = await fetch(`/api/v1/admin/wallets/search?${cleanParams.toString()}`, {
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
    
    if (diffInHours < 1) return 'Just now';
    if (diffInHours < 24) return `${diffInHours}h ago`;
    const diffInDays = Math.floor(diffInHours / 24);
    if (diffInDays < 30) return `${diffInDays}d ago`;
    return date.toLocaleDateString();
  };

  const getTierColor = (tier: string) => {
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
      <Card>
        <CardHeader>
          <CardTitle className="flex items-center gap-2">
            <Search className="h-5 w-5" />
            Wallet Search & Filters
          </CardTitle>
          <CardDescription>
            Search and filter wallet users by various criteria
          </CardDescription>
        </CardHeader>
        <CardContent>
          <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-4">
            {/* Search Input */}
            <div className="lg:col-span-2">
              <Label htmlFor="search">Search Wallets</Label>
              <Input
                id="search"
                placeholder="Search by wallet address..."
                value={filters.search}
                onChange={(e) => handleFilterChange('search', e.target.value)}
                className="font-mono"
              />
            </div>

            {/* Tier Filter */}
            <div>
              <Label>Tier Level</Label>
              <Select value={filters.tier} onValueChange={(value) => handleFilterChange('tier', value)}>
                <SelectTrigger>
                  <SelectValue placeholder="All Tiers" />
                </SelectTrigger>
                <SelectContent>
                  {TIER_OPTIONS.map((option) => (
                    <SelectItem key={option.value} value={option.value}>
                      {option.label}
                    </SelectItem>
                  ))}
                </SelectContent>
              </Select>
            </div>

            {/* Status Filter */}
            <div>
              <Label>Status</Label>
              <Select value={filters.status} onValueChange={(value) => handleFilterChange('status', value)}>
                <SelectTrigger>
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
              <Label>Date Range</Label>
              <Select value={filters.dateRange} onValueChange={(value) => handleFilterChange('dateRange', value)}>
                <SelectTrigger>
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
              <Label>Sort By</Label>
              <Select value={filters.sortBy} onValueChange={(value) => handleFilterChange('sortBy', value)}>
                <SelectTrigger>
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
              <Label>Order</Label>
              <Select value={filters.sortOrder} onValueChange={(value) => handleFilterChange('sortOrder', value as 'asc' | 'desc')}>
                <SelectTrigger>
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
            <div className="lg:col-span-2 flex gap-2 items-end">
              <Button 
                onClick={() => searchWallets(1)} 
                disabled={loading}
                className="flex items-center gap-2"
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
              >
                Clear Filters
              </Button>
            </div>
          </div>
        </CardContent>
      </Card>

      {/* Results */}
      <Card>
        <CardHeader>
          <div className="flex items-center justify-between">
            <div>
              <CardTitle className="flex items-center gap-2">
                <Users className="h-5 w-5" />
                Search Results
                {results && (
                  <Badge variant="secondary" className="ml-2">
                    {results.total_count} wallets
                  </Badge>
                )}
              </CardTitle>
              <CardDescription>
                {results && `Showing ${results.wallets.length} of ${results.total_count} results`}
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
                <div key={i} className="flex items-center justify-between p-4 border rounded">
                  <div className="flex items-center gap-4">
                    <Skeleton className="h-10 w-10 rounded-full" />
                    <div className="space-y-2">
                      <Skeleton className="h-4 w-32" />
                      <Skeleton className="h-3 w-24" />
                    </div>
                  </div>
                  <div className="space-y-2">
                    <Skeleton className="h-5 w-16" />
                    <Skeleton className="h-4 w-20" />
                  </div>
                </div>
              ))}
            </div>
          )}

          {!loading && results && results.wallets.length === 0 && (
            <div className="text-center py-8 text-gray-500">
              <Search className="h-12 w-12 mx-auto mb-4 opacity-50" />
              <p className="text-lg font-medium mb-2">No wallets found</p>
              <p>Try adjusting your search criteria or filters</p>
            </div>
          )}

          {!loading && results && results.wallets.length > 0 && (
            <div className="space-y-3">
              {results.wallets.map((wallet) => (
                <div
                  key={wallet.wallet_address}
                  className="flex items-center justify-between p-4 border rounded hover:bg-gray-50 dark:hover:bg-gray-800/50 transition-colors"
                >
                  <div className="flex items-center gap-4">
                    <div className="h-10 w-10 bg-gradient-to-r from-blue-500 to-purple-600 rounded-full flex items-center justify-center text-white text-sm font-bold">
                      {wallet.wallet_address.slice(2, 4).toUpperCase()}
                    </div>
                    <div>
                      <div className="font-mono text-sm font-medium">
                        {formatWalletAddress(wallet.wallet_address)}
                      </div>
                      <div className="flex items-center gap-2 mt-1">
                        <Badge variant={getTierColor(wallet.tier_level)} className="text-xs">
                          {wallet.tier_level}
                        </Badge>
                        <span className="text-xs text-gray-500">
                          {getActivePermissionsCount(wallet.permissions)} permissions
                        </span>
                        {wallet.groups.length > 0 && (
                          <span className="text-xs text-gray-500">
                            • {wallet.groups.length} groups
                          </span>
                        )}
                      </div>
                    </div>
                  </div>
                  <div className="text-right">
                    <div className="text-xs text-gray-500">
                      Created: {formatTimeAgo(wallet.created_at)}
                    </div>
                    {wallet.last_auth_at && (
                      <div className="text-xs text-gray-500">
                        Last seen: {formatTimeAgo(wallet.last_auth_at)}
                      </div>
                    )}
                    <Badge variant={wallet.is_active ? 'default' : 'outline'} className="text-xs mt-1">
                      {wallet.is_active ? 'Active' : 'Inactive'}
                    </Badge>
                  </div>
                </div>
              ))}
            </div>
          )}

          {/* Pagination */}
          {results && results.has_more && (
            <div className="flex justify-center mt-6">
              <Button
                variant="outline"
                onClick={() => searchWallets(page + 1)}
                disabled={loading}
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