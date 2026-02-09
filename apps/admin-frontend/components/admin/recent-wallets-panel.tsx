/**
 * Recent Wallets Panel Component
 * Displays recently connected wallets with analytics and real-time updates
 */
'use client';

import { Calendar, Eye, RefreshCw, TrendingUp, Wallet } from 'lucide-react';
import Link from 'next/link';
import { useCallback, useEffect, useState } from 'react';

import { getRecentWalletsAction } from '@/app/analytics/actions';
import { Alert, AlertDescription } from '@/components/ui/alert';
import { Badge } from '@/components/ui/badge';
import { Button } from '@/components/ui/button';
import { Skeleton } from '@/components/ui/skeleton';
import type { RecentWalletsData } from '@/hooks/use-analytics-data';
import { logger } from '@/lib/logger';

// Types moved to @/hooks/useAnalyticsData

/**
 *
 */
export function RecentWalletsPanel({ initialData }: { initialData?: RecentWalletsData }) {

  const [data, setData] = useState<RecentWalletsData | null>(initialData ?? null);
  const [loading, setLoading] = useState(!initialData);
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
      // Use longer timeout (60s) for this endpoint as it performs complex aggregations
      const result = await getRecentWalletsAction(10, 30);
      setData(result);
    } catch (err: unknown) {
      if (err instanceof Error && err.message === 'NEXT_REDIRECT') {
        throw err;
      }
      logger.error('Error fetching recent wallets', { error: err instanceof Error ? err.message : String(err) });
      setError(
        err instanceof Error ? err.message : 'Failed to load recent wallets'
      );
    } finally {
      setLoading(false);
      setRefreshing(false);
    }
  }, []);

  useEffect(() => {
    if (!initialData) {
      void fetchRecentWallets();
    }

    // Set up auto-refresh every 30 seconds
    const interval = setInterval(() => { void fetchRecentWallets(false); }, 30000);
    return () => clearInterval(interval);
  }, [fetchRecentWallets, initialData]);

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

  if (loading) {
    return (
      <div className="h-full bg-card backdrop-blur-sm rounded-2xl sm:rounded-3xl shadow-xl border border-border p-6">
        <div className="flex items-center justify-between mb-6">
          <div>
            <div className="flex items-center gap-2 mb-2">
              <Wallet className="h-5 w-5" />
              <h3 className="text-lg font-semibold">Recent Wallets</h3>
            </div>
            <p className="text-sm text-muted-foreground">Recently connected wallet addresses</p>
          </div>
        </div>
        <div className="space-y-3">
          {Array.from({ length: 5 }).map((_, i) => (
            <div
              key={`recent-wallet-skeleton-${i}`}
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

  if (typeof error === 'string') {
    return (
      <div className="h-full bg-card backdrop-blur-sm rounded-2xl sm:rounded-3xl shadow-xl border border-border p-6">
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
          onClick={() => { void fetchRecentWallets(true); }}
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
    <div className="h-full bg-slate-900/40 backdrop-blur-2xl rounded-[32px] shadow-xl border border-white/5 p-8 relative overflow-hidden group">
      {/* Decorative background element */}
      <div className="absolute -right-12 -top-12 w-48 h-48 bg-[#1fc7d4]/5 rounded-full blur-3xl group-hover:bg-[#1fc7d4]/10 transition-colors" />

      <div className="relative z-10 flex items-center justify-between mb-8">
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
          <p className="text-sm text-muted-foreground">
            Wallets connected in the last 30 days
          </p>
        </div>
        <Button
          variant="ghost"
          size="sm"
          onClick={() => { void fetchRecentWallets(true); }}
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
        <div className="mb-8 grid grid-cols-3 gap-4">
          <div className="rounded-3xl bg-blue-50/5 p-4 text-center border border-blue-500/10 backdrop-blur-sm">
            <div className="mb-1 flex items-center justify-center gap-1.5 text-[#1fc7d4]">
              <TrendingUp className="h-4 w-4" />
              <span className="text-xs font-bold uppercase tracking-wider">Total</span>
            </div>
            <div className="text-2xl font-bold text-foreground">
              {data.analytics.total_in_period}
            </div>
            <div className="text-[10px] text-muted-foreground uppercase font-bold tracking-widest">connections</div>
          </div>
          <div className="rounded-3xl bg-emerald-50/5 p-4 text-center border border-[#31d0aa]/10 backdrop-blur-sm">
            <div className="mb-1 flex items-center justify-center gap-1.5 text-[#31d0aa]">
              <Calendar className="h-4 w-4" />
              <span className="text-xs font-bold uppercase tracking-wider">Daily Avg</span>
            </div>
            <div className="text-2xl font-bold text-foreground">
              {data.analytics.avg_daily.toFixed(1)}
            </div>
            <div className="text-[10px] text-muted-foreground uppercase font-bold tracking-widest">per day</div>
          </div>
          <div className="rounded-3xl bg-purple-50/5 p-4 text-center border border-[#7645d9]/10 backdrop-blur-sm">
            <div className="mb-1 flex items-center justify-center gap-1.5 text-[#7645d9]">
              <Eye className="h-4 w-4" />
              <span className="text-xs font-bold uppercase tracking-wider">Active</span>
            </div>
            <div className="text-2xl font-bold text-foreground">
              {data.recent_wallets.filter(w => w.is_active).length}
            </div>
            <div className="text-[10px] text-muted-foreground uppercase font-bold tracking-widest">wallets</div>
          </div>
        </div>

        {/* Wallet List */}
        <div className="space-y-2">
          {displayedWallets.length === 0 ? (
            <div className="py-8 text-center text-muted-foreground">
              <Wallet className="mx-auto mb-2 h-8 w-8 opacity-50" />
              <p className="font-medium">No recent wallet connections</p>
              <p className="text-sm mt-1">New wallets will appear here when users connect</p>
              <div className="mt-4">
                <Link
                  href="/wallet-management"
                  className="inline-flex items-center gap-2 rounded-md bg-blue-100 px-3 py-2 text-sm font-medium text-blue-800 hover:bg-blue-200 dark:bg-blue-900/30 dark:text-blue-300 dark:hover:bg-blue-900/50"
                >
                  View All Wallets
                </Link>
              </div>
            </div>
          ) : (
            displayedWallets.map((wallet, index) => (
              <div
                key={wallet.wallet_address}
                className="flex items-center justify-between rounded-2xl border border-white/5 p-4 hover:bg-white/5 transition-all group/item active:scale-[0.99]"
              >
                <div className="flex items-center gap-4">
                  <div className="relative">
                    <div className="flex h-10 w-10 items-center justify-center rounded-2xl bg-gradient-to-br from-[#1fc7d4] to-[#7645d9] text-xs font-bold text-white shadow-lg shadow-cyan-500/10">
                      {index + 1}
                    </div>
                    {wallet.connection_info.is_new && (
                      <div className="absolute -top-1 -right-1 h-3.5 w-3.5 rounded-full border-2 border-[#1a1c24] bg-[#31d0aa] animate-pulse" />
                    )}
                  </div>
                  <div>
                    <div className="font-mono text-sm font-bold text-foreground">
                      {formatWalletAddress(wallet.wallet_address)}
                    </div>
                    <div className="mt-1 flex items-center gap-2">
                      <span className="text-[10px] font-bold text-muted-foreground uppercase tracking-widest bg-white/5 px-2 py-0.5 rounded-lg border border-white/5">
                        {wallet.active_permissions_count} permissions
                      </span>
                    </div>
                  </div>
                </div>
                <div className="text-right">
                  <div className="text-[10px] font-bold text-muted-foreground uppercase tracking-widest mb-1">
                    {formatTimeAgo(wallet.created_at)}
                  </div>
                  {wallet.connection_info.is_new && (
                    <Badge variant="default" className="rounded-lg bg-[#31d0aa] text-white text-[9px] font-bold py-0 h-4 border-none shadow-sm shadow-[#31d0aa]/20">
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
          <Link
            href="/wallet-management"
            className="text-sm font-medium text-blue-600 hover:text-blue-800"
          >
            View All Wallets →
          </Link>
        </div>
      </div>
    </div>
  );
}
