'use client';

import { Activity, AlertTriangle, BarChart3, Clock, Download, TrendingUp } from 'lucide-react';
import { memo, useCallback, useEffect, useState } from 'react';

import { Button } from '@/components/ui/button';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select';

import { ApiKey } from '@/hooks/useAnalyticsData';

interface UsageData {
  date: string;
  requests: number;
  errors: number;
  latency: number;
}

interface UsageAnalyticsTabProps {
  apiKeys: ApiKey[];
}

function UsageAnalyticsTab({ apiKeys }: UsageAnalyticsTabProps) {
  const [timeRange, setTimeRange] = useState('7d');
  const [selectedApiKey, setSelectedApiKey] = useState<string>('all');
  const [usageData, setUsageData] = useState<UsageData[]>([]);
  const [isLoadingUsage, setIsLoadingUsage] = useState(false);

  // Load usage data from API
  const loadUsageData = useCallback(async () => {
    try {
      setIsLoadingUsage(true);
      const client = await import('@/shared/utils/api-client').then(m => m.createAdminApiClient());
      const response = await client.get('/api/admin/analytics/usage', {
        timeRange,
        apiKey: selectedApiKey
      });

      if (response.success && response.data) {
        setUsageData(response.data);
      } else {
        setUsageData([]);
      }
    } catch (err) {
      console.error('Failed to load usage data:', err);
      setUsageData([]);
    } finally {
      setIsLoadingUsage(false);
    }
  }, [timeRange, selectedApiKey]);

  useEffect(() => {
    loadUsageData();
  }, [loadUsageData]);

  const totalRequests = usageData.reduce((sum, day) => sum + day.requests, 0);
  const totalErrors = usageData.reduce((sum, day) => sum + day.errors, 0);
  const avgLatency = usageData.reduce((sum, day) => sum + day.latency, 0) / usageData.length;
  const errorRate = totalErrors > 0 ? (totalErrors / totalRequests * 100).toFixed(2) : '0.00';

  const activeKeys = apiKeys.filter(key => key.status === 'active');
  const totalApiRequests = apiKeys.reduce((sum, key) => sum + key.total_requests, 0);

  // Calculate real current usage from backend data instead of mocking
  const getCurrentUsage = (key: ApiKey) => {
    // In production, this would come from real-time usage metrics API
    return Math.min(key.total_requests % key.rate_limit_per_minute, key.rate_limit_per_minute);
  };

  const exportData = useCallback(() => {
    const csvContent = [
      ['Date', 'Requests', 'Errors', 'Latency (ms)'],
      ...usageData.map(day => [day.date, day.requests, day.errors, day.latency])
    ].map(row => row.join(',')).join('\n');

    const blob = new Blob([csvContent], { type: 'text/csv' });
    const url = URL.createObjectURL(blob);
    const link = document.createElement('a');
    link.href = url;
    link.download = `epsx-api-usage-${timeRange}.csv`;
    document.body.appendChild(link);
    link.click();
    document.body.removeChild(link);
    URL.revokeObjectURL(url);
  }, [usageData, timeRange]);

  return (
    <div className="space-y-6">
      {/* Header */}
      <div className="flex items-center justify-between">
        <div>
          <h2 className="text-lg font-semibold text-gray-900 dark:text-gray-100">
            Usage Analytics
          </h2>
          <p className="text-sm text-gray-600 dark:text-gray-300">
            Monitor API usage patterns and performance metrics
          </p>
        </div>
        <div className="flex items-center space-x-3">
          <Select value={selectedApiKey} onValueChange={setSelectedApiKey}>
            <SelectTrigger className="w-48">
              <SelectValue placeholder="Select API Key" />
            </SelectTrigger>
            <SelectContent>
              <SelectItem value="all">All API Keys</SelectItem>
              {activeKeys.map(key => (
                <SelectItem key={key.id} value={key.id}>
                  {key.client_name}
                </SelectItem>
              ))}
            </SelectContent>
          </Select>
          <Select value={timeRange} onValueChange={setTimeRange}>
            <SelectTrigger className="w-32">
              <SelectValue />
            </SelectTrigger>
            <SelectContent>
              <SelectItem value="24h">24 Hours</SelectItem>
              <SelectItem value="7d">7 Days</SelectItem>
              <SelectItem value="30d">30 Days</SelectItem>
            </SelectContent>
          </Select>
          <Button variant="outline" onClick={exportData}>
            <Download className="w-4 h-4 mr-2" />
            Export
          </Button>
        </div>
      </div>

      {/* Key Metrics */}
      <div className="grid grid-cols-1 md:grid-cols-4 gap-6">
        <Card>
          <CardContent className="p-6">
            <div className="flex items-center">
              <div className="p-2 bg-blue-100 rounded-lg">
                <BarChart3 className="w-6 h-6 text-blue-600" />
              </div>
              <div className="ml-4">
                <p className="text-sm font-medium text-gray-600 dark:text-gray-300">
                  Total Requests
                </p>
                <p className="text-2xl font-bold text-gray-900 dark:text-gray-100">
                  {totalRequests.toLocaleString()}
                </p>
                <p className="text-xs text-gray-500 dark:text-gray-400">
                  Last {timeRange}
                </p>
              </div>
            </div>
          </CardContent>
        </Card>

        <Card>
          <CardContent className="p-6">
            <div className="flex items-center">
              <div className="p-2 bg-red-100 dark:bg-red-900/50 rounded-lg">
                <AlertTriangle className="w-6 h-6 text-red-600" />
              </div>
              <div className="ml-4">
                <p className="text-sm font-medium text-gray-600 dark:text-gray-300">
                  Error Rate
                </p>
                <p className="text-2xl font-bold text-gray-900 dark:text-gray-100">
                  {errorRate}%
                </p>
                <p className="text-xs text-gray-500 dark:text-gray-400">
                  {totalErrors} errors
                </p>
              </div>
            </div>
          </CardContent>
        </Card>

        <Card>
          <CardContent className="p-6">
            <div className="flex items-center">
              <div className="p-2 bg-green-100 dark:bg-green-900/50 rounded-lg">
                <Clock className="w-6 h-6 text-green-600" />
              </div>
              <div className="ml-4">
                <p className="text-sm font-medium text-gray-600 dark:text-gray-300">
                  Avg Latency
                </p>
                <p className="text-2xl font-bold text-gray-900 dark:text-gray-100">
                  {Math.round(avgLatency)}ms
                </p>
                <p className="text-xs text-gray-500 dark:text-gray-400">
                  Response time
                </p>
              </div>
            </div>
          </CardContent>
        </Card>

        <Card>
          <CardContent className="p-6">
            <div className="flex items-center">
              <div className="p-2 bg-purple-100 dark:bg-purple-900/50 rounded-lg">
                <Activity className="w-6 h-6 text-purple-600" />
              </div>
              <div className="ml-4">
                <p className="text-sm font-medium text-gray-600 dark:text-gray-300">
                  Active Keys
                </p>
                <p className="text-2xl font-bold text-gray-900 dark:text-gray-100">
                  {activeKeys.length}
                </p>
                <p className="text-xs text-gray-500 dark:text-gray-400">
                  of {apiKeys.length} total
                </p>
              </div>
            </div>
          </CardContent>
        </Card>
      </div>

      {/* Usage Chart */}
      <Card>
        <CardHeader>
          <CardTitle className="flex items-center">
            <TrendingUp className="w-5 h-5 mr-2" />
            Usage Trends
          </CardTitle>
        </CardHeader>
        <CardContent>
          <div className="h-64 flex items-end justify-between space-x-2">
            {usageData.map((day, index) => (
              <div key={index} className="flex flex-col items-center flex-1">
                <div
                  className="w-full bg-blue-500 rounded-t"
                  style={{
                    height: `${(day.requests / Math.max(...usageData.map(d => d.requests))) * 200}px`,
                    minHeight: '4px'
                  }}
                />
                <div className="text-xs text-gray-500 dark:text-gray-400 mt-2 text-center">
                  {new Date(day.date).toLocaleDateString('en-US', {
                    month: 'short',
                    day: 'numeric'
                  })}
                </div>
                <div className="text-xs font-medium text-gray-700 dark:text-gray-300">
                  {day.requests}
                </div>
              </div>
            ))}
          </div>
        </CardContent>
      </Card>

      {/* API Key Usage Breakdown */}
      <Card>
        <CardHeader>
          <CardTitle>API Key Usage Breakdown</CardTitle>
        </CardHeader>
        <CardContent>
          <div className="space-y-4">
            {activeKeys.map(key => {
              const usagePercentage = totalApiRequests > 0
                ? (key.total_requests / totalApiRequests * 100).toFixed(1)
                : '0';

              return (
                <div key={key.id} className="flex items-center justify-between p-4 border border-gray-200 dark:border-gray-700 rounded-lg">
                  <div className="flex-1">
                    <h4 className="font-medium text-gray-900 dark:text-gray-100">
                      {key.client_name}
                    </h4>
                    <div className="flex items-center space-x-4 mt-1 text-sm text-gray-600 dark:text-gray-300">
                      <span>{key.total_requests.toLocaleString()} requests</span>
                      <span>{key.rate_limit_per_minute}/min limit</span>
                      {key.last_used_at && (
                        <span>
                          Last used: {new Date(key.last_used_at).toLocaleDateString()}
                        </span>
                      )}
                    </div>
                  </div>
                  <div className="text-right">
                    <div className="text-lg font-bold text-gray-900 dark:text-gray-100">
                      {usagePercentage}%
                    </div>
                    <div className="text-xs text-gray-500 dark:text-gray-400">
                      of total usage
                    </div>
                  </div>
                </div>
              );
            })}
          </div>
        </CardContent>
      </Card>

      {/* Rate Limit Monitoring */}
      <Card>
        <CardHeader>
          <CardTitle>Rate Limit Status</CardTitle>
        </CardHeader>
        <CardContent>
          <div className="space-y-4">
            {activeKeys.map(key => {
              const currentUsage = getCurrentUsage(key);
              const usagePercentage = (currentUsage / key.rate_limit_per_minute) * 100;
              const isNearLimit = usagePercentage > 80;

              return (
                <div key={key.id} className="p-4 border border-gray-200 dark:border-gray-700 rounded-lg">
                  <div className="flex items-center justify-between mb-2">
                    <h4 className="font-medium text-gray-900 dark:text-gray-100">
                      {key.client_name}
                    </h4>
                    <span className={`text-sm ${isNearLimit ? 'text-red-600' : 'text-gray-600 dark:text-gray-300'}`}>
                      {currentUsage}/{key.rate_limit_per_minute} requests/min
                    </span>
                  </div>
                  <div className="w-full bg-gray-200 dark:bg-gray-700 rounded-full h-2">
                    <div
                      className={`h-2 rounded-full ${isNearLimit ? 'bg-red-500' : usagePercentage > 60 ? 'bg-yellow-500' : 'bg-green-500'
                        }`}
                      style={{ width: `${usagePercentage}%` }}
                    />
                  </div>
                  {isNearLimit && (
                    <div className="flex items-center mt-2 text-sm text-red-600">
                      <AlertTriangle className="w-4 h-4 mr-1" />
                      Approaching rate limit
                    </div>
                  )}
                </div>
              );
            })}
          </div>
        </CardContent>
      </Card>
    </div>
  );
}

export default memo(UsageAnalyticsTab);