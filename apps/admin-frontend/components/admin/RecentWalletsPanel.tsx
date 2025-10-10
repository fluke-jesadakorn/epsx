/**
 * Recent Wallets Panel Component
 * Displays recently connected wallets with analytics and real-time updates
 */
'use client';

import { RefreshCw, Wallet, TrendingUp, Calendar, Eye } from 'lucide-react';
import React, { useState, useEffect, useCallback } from 'react';

import { Alert, AlertDescription } from '@/components/ui/alert';
import { Badge } from '@/components/ui/badge';
import { Button } from '@/components/ui/button';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Skeleton } from '@/components/ui/skeleton';
import { fetchBackend } from '@/lib/backend-url';

interface WalletConnection {
  wallet_address: string;
  tier_level: string;
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
  const [data, setData] = useState<RecentWalletsData | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [refreshing, setRefreshing] = useState(false);
  const [expanded, setExpanded] = useState(false);

  const fetchRecentWallets = useCallback(async (showRefreshing = false) => {
    if (showRefreshing) {setRefreshing(true);}
    setError(null);

    try {
      const response = await fetchBackend('/api/admin/web3/recent-wallets?limit=10&days=7');

      if (!response.ok) {
        throw new Error(`Failed to fetch recent wallets: ${response.status}`);
      }

      const result = await response.json();
      setData(result);
    } catch (err) {
      // eslint-disable-next-line no-console
      console.error('Error fetching recent wallets:', err);
      setError(err instanceof Error ? err.message : 'Failed to load recent wallets');
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
    const diffInHours = Math.floor((now.getTime() - date.getTime()) / (1000 * 60 * 60));
    
    if (diffInHours < 1) {return 'Just now';}
    if (diffInHours < 24) {return `${diffInHours}h ago`;}
    return `${Math.floor(diffInHours / 24)}d ago`;
  };

  const formatWalletAddress = (address: string) => {
    return `${address.slice(0, 6)}...${address.slice(-4)}`;
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

  if (loading) {
    return (
      <Card className="h-full">
        <CardHeader>
          <CardTitle className="flex items-center gap-2">
            <Wallet className="h-5 w-5" />
            Recent Wallets
          </CardTitle>
          <CardDescription>Recently connected wallet addresses</CardDescription>
        </CardHeader>
        <CardContent>
          <div className="space-y-3">
            {Array.from({ length: 5 }).map((_, i) => (
              <div key={i} className="flex items-center justify-between p-3 border rounded">
                <div className="flex items-center gap-3">
                  <Skeleton className="h-8 w-8 rounded-full" />
                  <div>
                    <Skeleton className="h-4 w-24 mb-1" />
                    <Skeleton className="h-3 w-16" />
                  </div>
                </div>
                <Skeleton className="h-5 w-12" />
              </div>
            ))}
          </div>
        </CardContent>
      </Card>
    );
  }

  if (error) {
    return (
      <Card className="h-full">
        <CardHeader>
          <CardTitle className="flex items-center gap-2">
            <Wallet className="h-5 w-5" />
            Recent Wallets
          </CardTitle>
        </CardHeader>
        <CardContent>
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
        </CardContent>
      </Card>
    );
  }

  if (!data) {return null;}

  const displayedWallets = expanded ? data.recent_wallets : data.recent_wallets.slice(0, 5);

  return (
    <Card className="h-full">
      <CardHeader>
        <div className="flex items-center justify-between">
          <div>
            <CardTitle className="flex items-center gap-2">
              <Wallet className="h-5 w-5" />
              Recent Wallets
              {data.analytics.total_in_period > 0 && (
                <Badge variant="secondary" className="ml-2">
                  {data.analytics.total_in_period} this week
                </Badge>
              )}
            </CardTitle>
            <CardDescription>
              Wallets connected in the last 7 days
            </CardDescription>
          </div>
          <Button
            variant="ghost"
            size="sm"
            onClick={() => fetchRecentWallets(true)}
            disabled={refreshing}
            className="h-8 w-8 p-0"
          >
            <RefreshCw className={`h-4 w-4 ${refreshing ? 'animate-spin' : ''}`} />
          </Button>
        </div>
      </CardHeader>
      <CardContent>
        {/* Analytics Summary */}
        <div className="grid grid-cols-3 gap-4 mb-6">
          <div className="text-center p-3 bg-blue-50 dark:bg-blue-900/20 rounded-lg">
            <div className="flex items-center justify-center gap-1 text-blue-600 mb-1">
              <TrendingUp className="h-4 w-4" />
              <span className="text-sm font-medium">Total</span>
            </div>
            <div className="text-lg font-bold">{data.analytics.total_in_period}</div>
            <div className="text-xs text-gray-500">connections</div>
          </div>
          <div className="text-center p-3 bg-green-50 dark:bg-green-900/20 rounded-lg">
            <div className="flex items-center justify-center gap-1 text-green-600 mb-1">
              <Calendar className="h-4 w-4" />
              <span className="text-sm font-medium">Daily Avg</span>
            </div>
            <div className="text-lg font-bold">{data.analytics.avg_daily.toFixed(1)}</div>
            <div className="text-xs text-gray-500">per day</div>
          </div>
          <div className="text-center p-3 bg-purple-50 dark:bg-purple-900/20 rounded-lg">
            <div className="flex items-center justify-center gap-1 text-purple-600 mb-1">
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
            <div className="text-center py-8 text-gray-500">
              <Wallet className="h-8 w-8 mx-auto mb-2 opacity-50" />
              <p>No recent wallet connections</p>
            </div>
          ) : (
            displayedWallets.map((wallet, index) => (
              <div
                key={wallet.wallet_address}
                className="flex items-center justify-between p-3 border rounded hover:bg-gray-50 dark:hover:bg-gray-800/50 transition-colors"
              >
                <div className="flex items-center gap-3">
                  <div className="relative">
                    <div className="h-8 w-8 bg-gradient-to-r from-blue-500 to-purple-600 rounded-full flex items-center justify-center text-white text-xs font-bold">
                      {index + 1}
                    </div>
                    {wallet.connection_info.is_new && (
                      <div className="absolute -top-1 -right-1 h-3 w-3 bg-green-500 rounded-full border-2 border-white dark:border-gray-900" />
                    )}
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
                    <Badge variant="default" className="text-xs mt-1">
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
              {expanded ? 'Show Less' : `Show All (${data.recent_wallets.length})`}
            </Button>
          </div>
        )}

        {/* View All Link */}
        <div className="mt-4 pt-4 border-t">
          <a
            href="/wallet-management"
            className="text-sm text-blue-600 hover:text-blue-800 font-medium"
          >
            View All Wallets →
          </a>
        </div>
      </CardContent>
    </Card>
  );
}