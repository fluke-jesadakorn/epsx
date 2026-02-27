'use client';

import { Activity, AlertTriangle, BarChart3, Clock, Download, TrendingUp } from 'lucide-react';
import React, { memo, useCallback, useEffect, useState } from 'react';

import { Button } from '@/components/ui/button';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select';
import type { ApiKey } from '@/hooks/use-analytics-data';

interface UsageData {
  date: string;
  requests: number;
  errors: number;
  latency: number;
}

interface UsageAnalyticsTabProps {
  apiKeys: ApiKey[];
}

interface MetricCardProps {
  icon: React.ReactNode;
  label: string;
  value: string | number;
  sub: string;
  iconBg: string;
}

function MetricCard({ icon, label, value, sub, iconBg }: MetricCardProps) {
  return (
    <Card>
      <CardContent className="p-6">
        <div className="flex items-center">
          <div className={`p-2 ${iconBg} rounded-lg`}>
            {icon}
          </div>
          <div className="ml-4">
            <p className="text-sm font-medium text-muted-foreground">{label}</p>
            <p className="text-2xl font-bold text-foreground">{value}</p>
            <p className="text-xs text-muted-foreground">{sub}</p>
          </div>
        </div>
      </CardContent>
    </Card>
  );
}

function UsageBar({ usagePercentage, isNearLimit, currentUsage, rateLimit }: {
  usagePercentage: number;
  isNearLimit: boolean;
  currentUsage: number;
  rateLimit: number;
}) {
  const barColor = isNearLimit ? 'bg-red-500' : usagePercentage > 60 ? 'bg-yellow-500' : 'bg-green-500';
  return (
    <>
      <div className="w-full bg-gray-200 dark:bg-muted rounded-full h-2">
        <div
          className={`h-2 rounded-full ${barColor}`}
          style={{ width: `${usagePercentage}%` }}
        />
      </div>
      {isNearLimit && (
        <div className="flex items-center mt-2 text-sm text-red-600">
          <AlertTriangle className="w-4 h-4 mr-1" />
          Approaching rate limit ({currentUsage}/{rateLimit} requests/min)
        </div>
      )}
    </>
  );
}

function UsageChart({ usageData }: { usageData: UsageData[] }) {
  const maxRequests = Math.max(...usageData.map(d => d.requests));
  return (
    <Card>
      <CardHeader>
        <CardTitle className="flex items-center">
          <TrendingUp className="w-5 h-5 mr-2" />
          Usage Trends
        </CardTitle>
      </CardHeader>
      <CardContent>
        <div className="h-64 flex items-end justify-between space-x-2">
          {usageData.map((day) => (
            <div key={day.date} className="flex flex-col items-center flex-1">
              <div
                className="w-full bg-blue-500 rounded-t"
                style={{
                  height: `${(day.requests / maxRequests) * 200}px`,
                  minHeight: '4px'
                }}
              />
              <div className="text-xs text-muted-foreground mt-2 text-center">
                {new Date(day.date).toLocaleDateString('en-US', {
                  month: 'short',
                  day: 'numeric'
                })}
              </div>
              <div className="text-xs font-medium text-gray-700 dark:text-muted-foreground">
                {day.requests}
              </div>
            </div>
          ))}
        </div>
      </CardContent>
    </Card>
  );
}

function KeyBreakdown({ activeKeys, totalApiRequests }: { activeKeys: ApiKey[]; totalApiRequests: number }) {
  return (
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
              <div key={key.id} className="flex items-center justify-between p-4 border border-gray-200 dark:border-border/40 rounded-lg">
                <div className="flex-1">
                  <h4 className="font-medium text-foreground">{key.client_name}</h4>
                  <div className="flex items-center space-x-4 mt-1 text-sm text-muted-foreground">
                    <span>{key.total_requests.toLocaleString()} requests</span>
                    <span>{key.rate_limit_per_minute}/min limit</span>
                    {key.last_used_at !== undefined && (
                      <span>Last used: {new Date(key.last_used_at).toLocaleDateString()}</span>
                    )}
                  </div>
                </div>
                <div className="text-right">
                  <div className="text-lg font-bold text-foreground">{usagePercentage}%</div>
                  <div className="text-xs text-muted-foreground">of total usage</div>
                </div>
              </div>
            );
          })}
        </div>
      </CardContent>
    </Card>
  );
}

function RateLimitStatus({ activeKeys, getCurrentUsage }: { activeKeys: ApiKey[]; getCurrentUsage: (key: ApiKey) => number }) {
  return (
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
              <div key={key.id} className="p-4 border border-gray-200 dark:border-border/40 rounded-lg">
                <div className="flex items-center justify-between mb-2">
                  <h4 className="font-medium text-foreground">{key.client_name}</h4>
                  <span className={`text-sm ${isNearLimit ? 'text-red-600' : 'text-muted-foreground'}`}>
                    {currentUsage}/{key.rate_limit_per_minute} requests/min
                  </span>
                </div>
                <UsageBar
                  usagePercentage={usagePercentage}
                  isNearLimit={isNearLimit}
                  currentUsage={currentUsage}
                  rateLimit={key.rate_limit_per_minute}
                />
              </div>
            );
          })}
        </div>
      </CardContent>
    </Card>
  );
}

function useUsageData(timeRange: string, selectedApiKey: string) {
  const [usageData, setUsageData] = useState<UsageData[]>([]);
  const [_isLoading, setIsLoading] = useState(false);

  const load = useCallback(async (): Promise<void> => {
    try {
      setIsLoading(true);
      const client = await import('@/shared/utils/api-client').then(m => m.createAdminApiClient());
      const response = await client.get('/api/admin/analytics/usage', { timeRange, apiKey: selectedApiKey });
      if (response.success === true && response.data !== null && response.data !== undefined) {
        setUsageData(response.data as UsageData[]);
      } else {
        setUsageData([]);
      }
    } catch {
      setUsageData([]);
    } finally {
      setIsLoading(false);
    }
  }, [timeRange, selectedApiKey]);

  useEffect(() => { void load(); }, [load]);

  return usageData;
}

function TabHeader({ activeKeys, timeRange, setTimeRange, selectedApiKey, setSelectedApiKey, onExport }: {
  activeKeys: ApiKey[];
  timeRange: string;
  setTimeRange: (v: string) => void;
  selectedApiKey: string;
  setSelectedApiKey: (v: string) => void;
  onExport: () => void;
}) {
  return (
    <div className="flex items-center justify-between">
      <div>
        <h2 className="text-lg font-semibold text-foreground">Usage Analytics</h2>
        <p className="text-sm text-muted-foreground">Monitor API usage patterns and performance metrics</p>
      </div>
      <div className="flex items-center space-x-3">
        <Select value={selectedApiKey} onValueChange={setSelectedApiKey}>
          <SelectTrigger className="w-48">
            <SelectValue placeholder="Select API Key" />
          </SelectTrigger>
          <SelectContent>
            <SelectItem value="all">All API Keys</SelectItem>
            {activeKeys.map(key => (
              <SelectItem key={key.id} value={key.id}>{key.client_name}</SelectItem>
            ))}
          </SelectContent>
        </Select>
        <Select value={timeRange} onValueChange={setTimeRange}>
          <SelectTrigger className="w-32"><SelectValue /></SelectTrigger>
          <SelectContent>
            <SelectItem value="24h">24 Hours</SelectItem>
            <SelectItem value="7d">7 Days</SelectItem>
            <SelectItem value="30d">30 Days</SelectItem>
          </SelectContent>
        </Select>
        <Button variant="outline" onClick={onExport}>
          <Download className="w-4 h-4 mr-2" />
          Export
        </Button>
      </div>
    </div>
  );
}

function UsageAnalyticsTab({ apiKeys }: UsageAnalyticsTabProps): React.JSX.Element {
  const [timeRange, setTimeRange] = useState('7d');
  const [selectedApiKey, setSelectedApiKey] = useState<string>('all');
  const usageData = useUsageData(timeRange, selectedApiKey);

  const totalRequests = usageData.reduce((sum, day) => sum + day.requests, 0);
  const totalErrors = usageData.reduce((sum, day) => sum + day.errors, 0);
  const avgLatency = usageData.length > 0
    ? usageData.reduce((sum, day) => sum + day.latency, 0) / usageData.length
    : 0;
  const errorRate = totalErrors > 0 ? (totalErrors / totalRequests * 100).toFixed(2) : '0.00';

  const activeKeys = apiKeys.filter(key => key.status === 'active');
  const totalApiRequests = apiKeys.reduce((sum, key) => sum + key.total_requests, 0);
  const getCurrentUsage = (key: ApiKey): number =>
    Math.min(key.total_requests % key.rate_limit_per_minute, key.rate_limit_per_minute);

  const exportData = useCallback((): void => {
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
      <TabHeader
        activeKeys={activeKeys}
        timeRange={timeRange}
        setTimeRange={setTimeRange}
        selectedApiKey={selectedApiKey}
        setSelectedApiKey={setSelectedApiKey}
        onExport={exportData}
      />
      <div className="grid grid-cols-1 md:grid-cols-4 gap-6">
        <MetricCard icon={<BarChart3 className="w-6 h-6 text-blue-600" />} iconBg="bg-blue-100" label="Total Requests" value={totalRequests.toLocaleString()} sub={`Last ${timeRange}`} />
        <MetricCard icon={<AlertTriangle className="w-6 h-6 text-red-600" />} iconBg="bg-red-100 dark:bg-red-900/50" label="Error Rate" value={`${errorRate}%`} sub={`${totalErrors} errors`} />
        <MetricCard icon={<Clock className="w-6 h-6 text-green-600" />} iconBg="bg-green-100 dark:bg-green-900/50" label="Avg Latency" value={`${Math.round(avgLatency)}ms`} sub="Response time" />
        <MetricCard icon={<Activity className="w-6 h-6 text-purple-600" />} iconBg="bg-purple-100 dark:bg-purple-900/50" label="Active Keys" value={activeKeys.length} sub={`of ${apiKeys.length} total`} />
      </div>
      {usageData.length > 0 && <UsageChart usageData={usageData} />}
      <KeyBreakdown activeKeys={activeKeys} totalApiRequests={totalApiRequests} />
      <RateLimitStatus activeKeys={activeKeys} getCurrentUsage={getCurrentUsage} />
    </div>
  );
}

export default memo(UsageAnalyticsTab);
