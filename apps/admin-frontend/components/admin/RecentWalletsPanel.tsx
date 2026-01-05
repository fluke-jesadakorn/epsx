/**
 * Recent Wallets Panel Component
 * Displays recently connected wallets with analytics and real-time updates
 */
'use client';

import { Calendar, Eye, RefreshCw, TrendingUp, Wallet } from 'lucide-react';
import { useCallback, useEffect, useState } from 'react';

import { Alert, AlertDescription } from '@/components/ui/alert';
import { Badge } from '@/components/ui/badge';
import { Button } from '@/components/ui/button';
import { Skeleton } from '@/components/ui/skeleton';
import { useSimpleFetch } from '@/hooks/useSimpleFetch';

interface WalletConnection {
  wallet_address: string;
  metadata: any;
  created_at: string;
  last_auth_at?: string;
  is_active: boolean;
  active_permissions_count: number;
  connection_info: {
    is_new: boolean;
    last_seen?: number;
  };
}

interface WalletAnalytics {
  total_in_period: number;
  daily_breakdown: Array<{
    date: string;
    connections: number;
  }>;
  period_days: number;
  avg_daily: number;
}

interface RecentWalletsData {
  recent_wallets: WalletConnection[];
  analytics: WalletAnalytics;
  metadata: {
    limit: number;
    total_count: number;
    has_more: boolean;
    generated_at: string;
  };
}

/**
 *
 */
export function RecentWalletsPanel() {
  const { fetchSimple } = useSimpleFetch();
  const [data, setData] = useState<RecentWalletsData | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [refreshing, setRefreshing] = useState(false);
  const [expanded, setExpanded] = useState(false);

  const fetchRecentWallets = useCallback(async (showRefreshing = false) => {
    if (showRefreshing) {
      setRefreshing(true);
    }
    setError(null);

    try {
      // Try to get 30 days of data to show more meaningful results
      const response = await fetchSimple('/api/admin/web3/recent-wallets?limit=10&days=30');
      const result = await response.json();
      setData(result);
    } catch (err) {
      // eslint-disable-next-line no-console
      console.error('Error fetching recent wallets:', err);
      setError(
        err instanceof Error ? err.message : 'Failed to load recent wallets'
      );
    } finally {
      setLoading(false);
      setRefreshing(false);
    }
  }, []);

  useEffect(() => {
    fetchRecentWallets();

    // Set up auto-refresh every 30 seconds
    const interval = setInterval(() => fetchRecentWallets(), 30000);
    return () => clearInterval(interval);
  }, [fetchRecentWallets]);

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
    return `${Math.floor(diffInHours / 24)}d ago`;
  };

  const formatWalletAddress = (address: string) => {
    return `${address.slice(0, 6)}...${address.slice(-4)}`;
  };

  const getTierColor = (tier?: string | null) => {
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

  if (loading) {
    return (
      <div className="h-full bg-white/80 dark:bg-gray-800/80 backdrop-blur-sm rounded-2xl sm:rounded-3xl shadow-xl border-2 border-indigo-300/50 dark:border-indigo-700/50 p-6">
        <div className="flex items-center justify-between mb-6">
          <div>
            <div className="flex items-center gap-2 mb-2">
              <Wallet className="h-5 w-5" />
              <h3 className="text-lg font-semibold">Recent Wallets</h3>
            </div>
            <p className="text-sm text-gray-600 dark:text-gray-400">Recently connected wallet addresses</p>
          </div>
        </div>
        <div className="space-y-3">
          {Array.from({ length: 5 }).map((_, i) => (
            <div
              key={i}
              className="flex items-center justify-between rounded border p-3"
            >
              <div className="flex items-center gap-3">
                <Skeleton className="h-8 w-8 rounded-full" />
                <div>
                  <Skeleton className="mb-1 h-4 w-24" />
                  <Skeleton className="h-3 w-16" />
                </div>
              </div>
              <Skeleton className="h-5 w-12" />
            </div>
          ))}
        </div>
      </div>
    );
  }

  if (error) {
    return (
      <div className="h-full bg-white/80 dark:bg-gray-800/80 backdrop-blur-sm rounded-2xl sm:rounded-3xl shadow-xl border-2 border-indigo-300/50 dark:border-indigo-700/50 p-6">
        <div className="flex items-center justify-between mb-6">
          <div>
            <div className="flex items-center gap-2 mb-2">
              <Wallet className="h-5 w-5" />
              <h3 className="text-lg font-semibold">Recent Wallets</h3>
            </div>
          </div>
        </div>
        <Alert className="border-red-200 bg-red-50">
          <AlertDescription>{error}</AlertDescription>
        </Alert>
        <Button
          variant="outline"
          size="sm"
          onClick={() => fetchRecentWallets(true)}
          className="mt-4"
        >
          Try Again
        </Button>
      </div>
    );
  }

  if (!data) {
    return null;
  }

  const displayedWallets = expanded
    ? data.recent_wallets
    : data.recent_wallets.slice(0, 5);

  return (
    <div className="h-full bg-white/80 dark:bg-gray-800/80 backdrop-blur-sm rounded-2xl sm:rounded-3xl shadow-xl border-2 border-indigo-300/50 dark:border-indigo-700/50 p-6">
      <div className="flex items-center justify-between mb-6">
        <div>
          <div className="flex items-center gap-2 mb-2">
            <Wallet className="h-5 w-5" />
            <h3 className="text-lg font-semibold">Recent Wallets</h3>
            {data.analytics.total_in_period > 0 && (
              <Badge variant="secondary" className="ml-2">
                {data.analytics.total_in_period} this month
              </Badge>
            )}
          </div>
          <p className="text-sm text-gray-600 dark:text-gray-400">
            Wallets connected in the last 30 days
          </p>
        </div>
        <Button
          variant="ghost"
          size="sm"
          onClick={() => fetchRecentWallets(true)}
          disabled={refreshing}
          className="h-8 w-8 p-0"
        >
          <RefreshCw
            className={`h-4 w-4 ${refreshing ? 'animate-spin' : ''}`}
          />
        </Button>
      </div>

      <div>
        {/* Analytics Summary */}
        <div className="mb-6 grid grid-cols-3 gap-4">
          <div className="rounded-lg bg-blue-50 p-3 text-center dark:bg-blue-900/20">
            <div className="mb-1 flex items-center justify-center gap-1 text-blue-600">
              <TrendingUp className="h-4 w-4" />
              <span className="text-sm font-medium">Total</span>
            </div>
            <div className="text-lg font-bold">
              {data.analytics.total_in_period}
            </div>
            <div className="text-xs text-gray-500">connections</div>
          </div>
          <div className="rounded-lg bg-green-50 p-3 text-center dark:bg-green-900/20">
            <div className="mb-1 flex items-center justify-center gap-1 text-green-600">
              <Calendar className="h-4 w-4" />
              <span className="text-sm font-medium">Daily Avg</span>
            </div>
            <div className="text-lg font-bold">
              {data.analytics.avg_daily.toFixed(1)}
            </div>
            <div className="text-xs text-gray-500">per day</div>
          </div>
          <div className="rounded-lg bg-purple-50 p-3 text-center dark:bg-purple-900/20">
            <div className="mb-1 flex items-center justify-center gap-1 text-purple-600">
              <Eye className="h-4 w-4" />
              <span className="text-sm font-medium">Active</span>
            </div>
            <div className="text-lg font-bold">
              {data.recent_wallets.filter(w => w.is_active).length}
            </div>
            <div className="text-xs text-gray-500">wallets</div>
          </div>
        </div>

        {/* Wallet List */}
        <div className="space-y-2">
          {displayedWallets.length === 0 ? (
            <div className="py-8 text-center text-gray-500">
              <Wallet className="mx-auto mb-2 h-8 w-8 opacity-50" />
              <p className="font-medium">No recent wallet connections</p>
              <p className="text-sm mt-1">New wallets will appear here when users connect</p>
              <div className="mt-4">
                <a
                  href="/wallet-management"
                  className="inline-flex items-center gap-2 rounded-md bg-blue-100 px-3 py-2 text-sm font-medium text-blue-800 hover:bg-blue-200 dark:bg-blue-900/30 dark:text-blue-300 dark:hover:bg-blue-900/50"
                >
                  View All Wallets
                </a>
              </div>
            </div>
          ) : (
            displayedWallets.map((wallet, index) => (
              <div
                key={wallet.wallet_address}
                className="flex items-center justify-between rounded border p-3 hover:bg-gray-50 dark:hover:bg-gray-800/50"
              >
                <div className="flex items-center gap-3">
                  <div className="relative">
                    <div className="flex h-8 w-8 items-center justify-center rounded-full bg-gradient-to-r from-blue-500 to-purple-600 text-xs font-bold text-white">
                      {index + 1}
                    </div>
                    {wallet.connection_info.is_new && (
                      <div className="absolute -top-1 -right-1 h-3 w-3 rounded-full border-2 border-white bg-green-500 dark:border-gray-900" />
                    )}
                  </div>
                  <div>
                    <div className="font-mono text-sm font-medium">
                      {formatWalletAddress(wallet.wallet_address)}
                    </div>
                    <div className="mt-1 flex items-center gap-2">
                      <span className="text-xs text-gray-500">
                        {wallet.active_permissions_count} permissions
                      </span>
                    </div>
                  </div>
                </div>
                <div className="text-right">
                  <div className="text-xs text-gray-500">
                    {formatTimeAgo(wallet.created_at)}
                  </div>
                  {wallet.connection_info.is_new && (
                    <Badge variant="default" className="mt-1 text-xs">
                      NEW
                    </Badge>
                  )}
                </div>
              </div>
            ))
          )}
        </div>

        {/* Show More/Less Button */}
        {data.recent_wallets.length > 5 && (
          <div className="mt-4 text-center">
            <Button
              variant="ghost"
              size="sm"
              onClick={() => setExpanded(!expanded)}
            >
              {expanded
                ? 'Show Less'
                : `Show All (${data.recent_wallets.length})`}
            </Button>
          </div>
        )}

        {/* View All Link */}
        <div className="mt-4 border-t pt-4">
          <a
            href="/wallet-management"
            className="text-sm font-medium text-blue-600 hover:text-blue-800"
          >
            View All Wallets →
          </a>
        </div>
      </div>
    </div>
  );
}
