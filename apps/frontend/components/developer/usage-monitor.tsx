'use client';

import {
  getApiKeysAction,
  getTopEndpointsAction,
  getUsageHistoryAction,
  getUsageStatsAction
} from '@/app/actions/developer';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui';
import { Badge } from '@/components/ui/badge';
import { useQuery } from '@tanstack/react-query';
import type { AuthUser } from '@/lib/server-actions';

interface UsageMonitorProps {
  currentUser: AuthUser;
}

interface KeyUsageSummary {
  id: string;
  name: string;
  total_requests: number;
  status: string;
  created_at: string;
  expires_at: string | null;
}

export function UsageMonitor({ currentUser }: UsageMonitorProps) {
  // Use TanStack Query for all data fetching
  const { data: keysRes, isLoading: isLoadingKeys } = useQuery({ queryKey: ['dev-api-keys'], queryFn: () => getApiKeysAction({ limit: 100 }) });
  const { data: statsRes, isLoading: isLoadingStats } = useQuery({ queryKey: ['dev-usage-stats'], queryFn: getUsageStatsAction });
  const { data: historyRes, isLoading: isLoadingHistory } = useQuery({ queryKey: ['dev-usage-history'], queryFn: () => getUsageHistoryAction(7) });
  const { data: endpointsRes, isLoading: isLoadingEndpoints } = useQuery({ queryKey: ['dev-top-endpoints'], queryFn: () => getTopEndpointsAction(7) });

  const isLoading = isLoadingKeys || isLoadingStats || isLoadingHistory || isLoadingEndpoints;

  // Process API Keys
  const apiKeys: KeyUsageSummary[] = (keysRes?.success && keysRes.data) ? (keysRes.data as any).api_keys?.map((k: any) => ({
    id: k.id,
    name: k.name,
    total_requests: k.usage_count ?? 0,
    status: k.is_active ? 'active' : 'inactive',
    created_at: k.created_at,
    expires_at: null
  })) ?? [] : [];

  const totalKeys = apiKeys.length;
  const activeKeys = apiKeys.filter(k => k.status === 'active').length;

  // Process Stats
  const stats = (statsRes?.success && statsRes.data) ? ((statsRes.data).data ?? statsRes.data) : {
    total_requests: 0,
    average_success_rate: 100,
    requests_24h: 0,
    error_rate_24h: 0
  };

  // Process History
  const history = (historyRes?.success && historyRes.data) ? ((historyRes.data).data ?? historyRes.data) : [];

  // Process Top Endpoints
  const topEndpoints = (endpointsRes?.success && endpointsRes.data) ? ((endpointsRes.data).data ?? endpointsRes.data) : [];

  return (
    <div className="space-y-6">
      {/* Real Usage Overview */}
      <div className="grid md:grid-cols-2 lg:grid-cols-4 gap-4">
        <Card className="bg-gradient-to-br from-emerald-50 to-teal-50 dark:from-emerald-900/20 dark:to-teal-900/20 border-0">
          <CardContent className="p-6">
            <div className="flex items-center gap-3 mb-3">
              <div className="w-10 h-10 rounded-xl bg-emerald-500/20 flex items-center justify-center">
                <svg className="w-5 h-5 text-emerald-600" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M9 19v-6a2 2 0 00-2-2H5a2 2 0 00-2 2v6a2 2 0 002 2h2a2 2 0 002-2zm0 0V9a2 2 0 012-2h2a2 2 0 012 2v10m-6 0a2 2 0 002 2h2a2 2 0 002-2m0 0V5a2 2 0 012-2h2a2 2 0 012 2v14a2 2 0 01-2 2h-2a2 2 0 01-2-2z" />
                </svg>
              </div>
              <span className="text-sm font-medium text-gray-500 dark:text-gray-400">Total Requests</span>
            </div>
            <div className="text-3xl font-bold text-emerald-600 dark:text-emerald-400">
              {isLoading ? '...' : stats.total_requests.toLocaleString()}
            </div>
            <div className="text-xs text-gray-500 mt-1">All time across all keys</div>
          </CardContent>
        </Card>

        <Card className="bg-gradient-to-br from-blue-50 to-indigo-50 dark:from-blue-900/20 dark:to-indigo-900/20 border-0">
          <CardContent className="p-6">
            <div className="flex items-center gap-3 mb-3">
              <div className="w-10 h-10 rounded-xl bg-blue-500/20 flex items-center justify-center">
                <svg className="w-5 h-5 text-blue-600" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M15 7a2 2 0 012 2m4 0a6 6 0 01-7.743 5.743L11 17H9v2H7v2H4a1 1 0 01-1-1v-2.586a1 1 0 01.293-.707l5.964-5.964A6 6 0 1121 9z" />
                </svg>
              </div>
              <span className="text-sm font-medium text-gray-500 dark:text-gray-400">Requests (24h)</span>
            </div>
            <div className="text-3xl font-bold text-blue-600 dark:text-blue-400">
              {isLoading ? '...' : stats.requests_24h.toLocaleString()}
            </div>
            <div className="text-xs text-gray-500 mt-1">
              {totalKeys} keys ({activeKeys} active)
            </div>
          </CardContent>
        </Card>

        <Card className="bg-gradient-to-br from-purple-50 to-pink-50 dark:from-purple-900/20 dark:to-pink-900/20 border-0">
          <CardContent className="p-6">
            <div className="flex items-center gap-3 mb-3">
              <div className="w-10 h-10 rounded-xl bg-purple-500/20 flex items-center justify-center">
                <svg className="w-5 h-5 text-purple-600" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M13 10V3L4 14h7v7l9-11h-7z" />
                </svg>
              </div>
              <span className="text-sm font-medium text-gray-500 dark:text-gray-400">Error Rate (24h)</span>
            </div>
            <div className="text-3xl font-bold text-purple-600 dark:text-purple-400">
              {isLoading ? '...' : `${stats.error_rate_24h.toFixed(2)}%`}
            </div>
            <div className="text-xs text-gray-500 mt-1">Failed requests</div>
          </CardContent>
        </Card>

        <Card className="bg-gradient-to-br from-amber-50 to-orange-50 dark:from-amber-900/20 dark:to-orange-900/20 border-0">
          <CardContent className="p-6">
            <div className="flex items-center gap-3 mb-3">
              <div className="w-10 h-10 rounded-xl bg-amber-500/20 flex items-center justify-center">
                <svg className="w-5 h-5 text-amber-600" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M9 12l2 2 4-4m6 2a9 9 0 11-18 0 9 9 0 0118 0z" />
                </svg>
              </div>
              <span className="text-sm font-medium text-gray-500 dark:text-gray-400">Success Rate</span>
            </div>
            <div className="text-3xl font-bold text-amber-600 dark:text-amber-400">
              {isLoading ? '...' : `${stats.average_success_rate.toFixed(1)}%`}
            </div>
            <div className="text-xs text-gray-500 mt-1">Global average</div>
          </CardContent>
        </Card>
      </div>

      {/* Per-Key Usage */}
      <Card>
        <CardHeader>
          <CardTitle className="flex items-center gap-2">
            <svg className="w-5 h-5 text-blue-500" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M15 7a2 2 0 012 2m4 0a6 6 0 01-7.743 5.743L11 17H9v2H7v2H4a1 1 0 01-1-1v-2.586a1 1 0 01.293-.707l5.964-5.964A6 6 0 1121 9z" />
            </svg>
            Usage by API Key
          </CardTitle>
        </CardHeader>
        <CardContent>
          {isLoading ? (
            <div className="flex items-center justify-center py-8">
              <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-blue-500" />
            </div>
          ) : apiKeys.length === 0 ? (
            <div className="text-center py-8 text-gray-500">
              <div className="text-4xl mb-3">🔑</div>
              <p className="font-medium">No API keys yet</p>
              <p className="text-sm">Create your first API key to start tracking usage</p>
            </div>
          ) : (
            <div className="space-y-3">
              {apiKeys.map((key) => (
                <div key={key.id} className="flex items-center justify-between p-4 bg-gray-50 dark:bg-gray-800 rounded-xl">
                  <div className="flex-1">
                    <div className="flex items-center gap-2">
                      <span className="font-medium">{key.name}</span>
                      <Badge
                        variant={key.status === 'active' ? 'default' : 'secondary'}
                        className={key.status === 'active'
                          ? 'bg-green-100 text-green-700 dark:bg-green-900/30 dark:text-green-400'
                          : 'bg-gray-100 text-gray-600 dark:bg-gray-700 dark:text-gray-400'
                        }
                      >
                        {key.status}
                      </Badge>
                    </div>
                    <div className="text-xs text-gray-500 mt-1">
                      Created {new Date(key.created_at).toLocaleDateString()}
                      {key.expires_at && ` • Expires ${new Date(key.expires_at).toLocaleDateString()}`}
                    </div>
                  </div>
                  <div className="text-right">
                    <div className="text-2xl font-bold text-emerald-600 dark:text-emerald-400">
                      {key.total_requests.toLocaleString()}
                    </div>
                    <div className="text-xs text-gray-500">requests</div>
                  </div>
                </div>
              ))}
            </div>
          )}
        </CardContent>
      </Card>

      {/* Usage History */}
      <Card className="">
        <CardHeader>
          <div className="flex justify-between items-center">
            <CardTitle>Usage History (Last 7 Days)</CardTitle>
            <div className="flex space-x-2">
              <button
                className="px-3 py-1 rounded text-sm bg-gray-200 text-gray-700 dark:bg-gray-700 dark:text-gray-300"
                disabled
              >
                7d
              </button>
            </div>
          </div>
        </CardHeader>
        <CardContent>
          <div className="h-64 flex items-end justify-between gap-2 pt-8">
            {isLoading ? (
              <div className="w-full flex items-center justify-center h-full">
                <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-blue-500" />
              </div>
            ) : history.length === 0 ? (
              <div className="w-full text-center text-gray-500">No usage data available</div>
            ) : (
              history.map((point, i) => {
                const maxCount = Math.max(...history.map(h => h.count), 1);
                const heightPercent = (point.count / maxCount) * 100;
                // Parse date - point.bucket is ISO string
                const date = new Date(point.bucket);
                const label = date.toLocaleDateString(undefined, { weekday: 'short', day: 'numeric' });

                return (
                  <div key={i} className="flex-1 flex flex-col items-center group relative">
                    <div
                      className="w-full bg-emerald-500/20 hover:bg-emerald-500/40 rounded-t transition-all relative"
                      style={{ height: `${Math.max(heightPercent, 5)}%` }} // Min 5% height for visibility
                    >
                      <div className="absolute -top-8 left-1/2 -translate-x-1/2 bg-black/80 text-white text-xs px-2 py-1 rounded opacity-0 group-hover:opacity-100 transition-opacity whitespace-nowrap z-10">
                        {point.count} reqs
                      </div>
                      <div
                        className="absolute bottom-0 left-0 right-0 bg-emerald-500 rounded-t"
                        style={{ height: `${heightPercent}%` }}
                      />
                    </div>
                    <div className="text-xs text-gray-500 mt-2 rotate-0 truncate w-full text-center">
                      {label}
                    </div>
                  </div>
                );
              })
            )}
          </div>
        </CardContent>
      </Card>

      {/* Top Endpoints */}
      <Card>
        <CardHeader>
          <CardTitle>Top Endpoints (7 Days)</CardTitle>
        </CardHeader>
        <CardContent>
          <div className="space-y-4">
            {isLoading ? (
              <div className="text-center text-gray-500 py-4">Loading stats...</div>
            ) : topEndpoints.length === 0 ? (
              <div className="text-center text-gray-500 py-4">No endpoint usage data available</div>
            ) : (
              topEndpoints.map((endpoint, idx) => (
                <div key={idx} className="flex items-center justify-between p-4 bg-gray-50 dark:bg-gray-800 rounded-lg">
                  <div className="flex items-center gap-3">
                    <Badge className={
                      endpoint.method === 'GET' ? 'bg-blue-100 text-blue-700' :
                        endpoint.method === 'POST' ? 'bg-green-100 text-green-700' :
                          'bg-gray-100 text-gray-700'
                    }>
                      {endpoint.method}
                    </Badge>
                    <div className="font-mono text-sm">{endpoint.endpoint}</div>
                  </div>
                  <div className="text-lg font-bold text-emerald-600">
                    {endpoint.count.toLocaleString()}
                  </div>
                </div>
              ))
            )}
          </div>
        </CardContent>
      </Card>

      {/* Info Card */}
      <Card className="bg-blue-50 dark:bg-blue-900/20 border-blue-200 dark:border-blue-800">
        <CardContent className="p-6">
          <h3 className="font-semibold text-blue-800 dark:text-blue-300 mb-2 flex items-center gap-2">
            📊 About Usage Tracking
          </h3>
          <p className="text-sm text-blue-700 dark:text-blue-400">
            Currently, we track <strong>total requests</strong> per API key.
            Detailed time-series analytics (hourly/daily breakdowns, endpoint-level stats)
            are planned for a future release. The total request count updates in real-time
            as you make API calls.
          </p>
        </CardContent>
      </Card>
    </div>
  );
}