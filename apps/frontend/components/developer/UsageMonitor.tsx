'use client';

import { useState, useEffect } from 'react';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui';
import { Badge } from '@/components/ui/badge';
import type { AuthUser } from '@/lib/server-actions';

interface UsageMonitorProps {
  currentUser: AuthUser;
}

interface UsageStats {
  current_hour: {
    requests: number;
    limit: number;
  };
  today: {
    requests: number;
    errors: number;
  };
  this_month: {
    requests: number;
    successful: number;
    errors: number;
  };
  endpoints: {
    endpoint: string;
    requests: number;
    avg_response_time: number;
  }[];
}

interface UsageHistory {
  date: string;
  requests: number;
  errors: number;
  avg_response_time: number;
}

export function UsageMonitor({ currentUser }: UsageMonitorProps) {
  const [usageStats, setUsageStats] = useState<UsageStats | null>(null);
  const [usageHistory, setUsageHistory] = useState<UsageHistory[]>([]);
  const [timeframe, setTimeframe] = useState<'7d' | '30d' | '90d'>('7d');

  // Mock data for demonstration
  useEffect(() => {
    const mockStats: UsageStats = {
      current_hour: {
        requests: 45,
        limit: currentUser.role === 'admin' ? 999999 : currentUser.role === 'premium' ? 1000 : 100
      },
      today: {
        requests: 342,
        errors: 5
      },
      this_month: {
        requests: 8250,
        successful: 8100,
        errors: 150
      },
      endpoints: [
        { endpoint: '/api/v1/analytics/rankings', requests: 4200, avg_response_time: 145 },
        { endpoint: '/api/v1/analytics/stock/{symbol}', requests: 2800, avg_response_time: 89 },
        { endpoint: '/api/v1/webhooks/rankings-update', requests: 1250, avg_response_time: 203 }
      ]
    };

    const mockHistory: UsageHistory[] = Array.from({ length: 30 }, (_, i) => ({
      date: new Date(Date.now() - i * 24 * 60 * 60 * 1000).toISOString().split('T')[0],
      requests: Math.floor(Math.random() * 500) + 100,
      errors: Math.floor(Math.random() * 20),
      avg_response_time: Math.floor(Math.random() * 200) + 50
    })).reverse();

    setUsageStats(mockStats);
    setUsageHistory(mockHistory);
  }, [currentUser.role]);

  const getRateLimitPercentage = () => {
    if (!usageStats) return 0;
    return (usageStats.current_hour.requests / usageStats.current_hour.limit) * 100;
  };

  const getStatusColor = (percentage: number) => {
    if (percentage >= 90) return 'text-red-500 bg-red-100 dark:bg-red-900/20';
    if (percentage >= 75) return 'text-yellow-500 bg-yellow-100 dark:bg-yellow-900/20';
    return 'text-green-500 bg-green-100 dark:bg-green-900/20';
  };

  if (!usageStats) {
    return <div className="p-6 text-center">Loading usage data...</div>;
  }

  const filteredHistory = usageHistory.slice(-parseInt(timeframe.replace('d', '')));

  return (
    <div className="space-y-6">
      {/* Current Usage Overview */}
      <div className="grid md:grid-cols-2 lg:grid-cols-4 gap-4">
        <Card>
          <CardContent className="p-6">
            <div className="text-2xl font-bold text-emerald-600 mb-2">
              {usageStats.current_hour.requests}
            </div>
            <div className="text-sm text-gray-600 dark:text-gray-400">
              Requests This Hour
            </div>
            <div className="mt-2">
              <div className={`inline-flex items-center px-2 py-1 rounded-full text-xs font-medium ${getStatusColor(getRateLimitPercentage())}`}>
                {getRateLimitPercentage().toFixed(1)}% of limit
              </div>
            </div>
          </CardContent>
        </Card>

        <Card>
          <CardContent className="p-6">
            <div className="text-2xl font-bold text-blue-600 mb-2">
              {usageStats.today.requests}
            </div>
            <div className="text-sm text-gray-600 dark:text-gray-400">
              Requests Today
            </div>
            <div className="text-xs text-gray-500 mt-1">
              {usageStats.today.errors} errors
            </div>
          </CardContent>
        </Card>

        <Card>
          <CardContent className="p-6">
            <div className="text-2xl font-bold text-purple-600 mb-2">
              {usageStats.this_month.requests.toLocaleString()}
            </div>
            <div className="text-sm text-gray-600 dark:text-gray-400">
              Requests This Month
            </div>
            <div className="text-xs text-gray-500 mt-1">
              {((usageStats.this_month.successful / usageStats.this_month.requests) * 100).toFixed(1)}% success rate
            </div>
          </CardContent>
        </Card>

        <Card>
          <CardContent className="p-6">
            <div className="text-2xl font-bold text-orange-600 mb-2">
              {usageStats.current_hour.limit === 999999 ? '∞' : usageStats.current_hour.limit}
            </div>
            <div className="text-sm text-gray-600 dark:text-gray-400">
              Hourly Rate Limit
            </div>
            <div className="mt-2">
              <Badge className="bg-gray-100 text-gray-800 dark:bg-gray-700 dark:text-gray-300">
                {currentUser.role}
              </Badge>
            </div>
          </CardContent>
        </Card>
      </div>

      {/* Rate Limit Progress */}
      <Card>
        <CardHeader>
          <CardTitle>Rate Limit Usage</CardTitle>
        </CardHeader>
        <CardContent>
          <div className="space-y-4">
            <div className="flex justify-between items-center">
              <span className="text-sm font-medium">Current Hour</span>
              <span className="text-sm text-gray-600 dark:text-gray-400">
                {usageStats.current_hour.requests} / {usageStats.current_hour.limit === 999999 ? '∞' : usageStats.current_hour.limit}
              </span>
            </div>
            <div className="w-full bg-gray-200 dark:bg-gray-700 rounded-full h-2">
              <div 
                className={`h-2 rounded-full ${
                  getRateLimitPercentage() >= 90 ? 'bg-red-500' :
                  getRateLimitPercentage() >= 75 ? 'bg-yellow-500' : 'bg-green-500'
                }`}
                style={{ width: `${Math.min(getRateLimitPercentage(), 100)}%` }}
              ></div>
            </div>
            {getRateLimitPercentage() >= 75 && (
              <div className="text-sm text-yellow-600 dark:text-yellow-400">
                ⚠️ Approaching rate limit. Consider upgrading your plan for higher limits.
              </div>
            )}
          </div>
        </CardContent>
      </Card>

      {/* Usage History Chart */}
      <Card>
        <CardHeader>
          <div className="flex justify-between items-center">
            <CardTitle>Usage History</CardTitle>
            <div className="flex space-x-2">
              {(['7d', '30d', '90d'] as const).map((period) => (
                <button
                  key={period}
                  onClick={() => setTimeframe(period)}
                  className={`px-3 py-1 rounded text-sm ${
                    timeframe === period
                      ? 'bg-emerald-500 text-white'
                      : 'bg-gray-200 text-gray-700 hover:bg-gray-300 dark:bg-gray-700 dark:text-gray-300 dark:hover:bg-gray-600'
                  }`}
                >
                  {period}
                </button>
              ))}
            </div>
          </div>
        </CardHeader>
        <CardContent>
          <div className="space-y-4">
            {/* Simple bar chart representation */}
            <div className="grid grid-cols-7 gap-2">
              {filteredHistory.slice(-7).map((day, idx) => (
                <div key={idx} className="text-center">
                  <div 
                    className="bg-emerald-500 rounded-t mb-1 mx-auto"
                    style={{ 
                      height: `${(day.requests / Math.max(...filteredHistory.map(d => d.requests))) * 60}px`,
                      width: '20px'
                    }}
                  ></div>
                  <div className="text-xs text-gray-600 dark:text-gray-400">
                    {new Date(day.date).getDate()}
                  </div>
                  <div className="text-xs font-medium">
                    {day.requests}
                  </div>
                </div>
              ))}
            </div>
          </div>
        </CardContent>
      </Card>

      {/* Top Endpoints */}
      <Card>
        <CardHeader>
          <CardTitle>Top Endpoints</CardTitle>
        </CardHeader>
        <CardContent>
          <div className="space-y-4">
            {usageStats.endpoints.map((endpoint, idx) => (
              <div key={idx} className="flex items-center justify-between p-4 bg-gray-50 dark:bg-gray-800 rounded-lg">
                <div className="flex-1">
                  <div className="font-mono text-sm font-medium">
                    {endpoint.endpoint}
                  </div>
                  <div className="text-xs text-gray-600 dark:text-gray-400 mt-1">
                    Avg response: {endpoint.avg_response_time}ms
                  </div>
                </div>
                <div className="text-right">
                  <div className="text-lg font-bold text-emerald-600">
                    {endpoint.requests.toLocaleString()}
                  </div>
                  <div className="text-xs text-gray-600 dark:text-gray-400">
                    requests
                  </div>
                </div>
              </div>
            ))}
          </div>
        </CardContent>
      </Card>

      {/* Error Analysis */}
      <Card>
        <CardHeader>
          <CardTitle>Error Analysis</CardTitle>
        </CardHeader>
        <CardContent>
          <div className="grid md:grid-cols-2 gap-6">
            <div>
              <h4 className="font-semibold mb-3">Common Error Codes</h4>
              <div className="space-y-2">
                {[
                  { code: '429', description: 'Rate limit exceeded', count: 85 },
                  { code: '401', description: 'Unauthorized', count: 42 },
                  { code: '400', description: 'Bad request', count: 18 },
                  { code: '500', description: 'Server error', count: 5 }
                ].map((error, idx) => (
                  <div key={idx} className="flex justify-between items-center p-2 bg-red-50 dark:bg-red-900/20 rounded">
                    <div>
                      <span className="font-mono text-sm font-medium">{error.code}</span>
                      <span className="text-sm text-gray-600 dark:text-gray-400 ml-2">
                        {error.description}
                      </span>
                    </div>
                    <Badge variant="outline" className="text-red-600 dark:text-red-400">
                      {error.count}
                    </Badge>
                  </div>
                ))}
              </div>
            </div>
            
            <div>
              <h4 className="font-semibold mb-3">Performance Metrics</h4>
              <div className="space-y-4">
                <div className="flex justify-between">
                  <span className="text-sm">Success Rate</span>
                  <span className="font-medium text-green-600">
                    {((usageStats.this_month.successful / usageStats.this_month.requests) * 100).toFixed(2)}%
                  </span>
                </div>
                <div className="flex justify-between">
                  <span className="text-sm">Avg Response Time</span>
                  <span className="font-medium text-blue-600">145ms</span>
                </div>
                <div className="flex justify-between">
                  <span className="text-sm">Uptime</span>
                  <span className="font-medium text-green-600">99.9%</span>
                </div>
              </div>
            </div>
          </div>
        </CardContent>
      </Card>
    </div>
  );
}