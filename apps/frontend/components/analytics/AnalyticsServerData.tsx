'use client';

import React, { useState, useEffect } from 'react';
import { analyticsClient } from '@/lib/api-client.client';
import {
  BarChart3,
  TrendingUp,
  Users,
  Activity,
  DollarSign,
  RefreshCw,
  AlertCircle,
} from 'lucide-react';
import { Card, CardContent, CardHeader, CardTitle, Badge, Button, Tabs, TabsContent, TabsList, TabsTrigger } from '@epsx/ui';

interface SystemMetrics {
  cpu_usage: number;
  memory_usage: number;
  active_connections: number;
  requests_per_minute: number;
  response_time_avg: number;
  error_rate: number;
  uptime_seconds: number;
}

interface DataPoint {
  timestamp: string;
  value: number;
}

interface SymbolData {
  symbol: string;
  volume: number;
  count: number;
}

interface ActivityData {
  hour: number;
  active_users: number;
  transactions: number;
}

interface AnalyticsData {
  user_growth: DataPoint[];
  trading_volume: DataPoint[];
  popular_symbols: SymbolData[];
  user_activity: ActivityData[];
}

interface RealtimeMetrics {
  active_users: number;
  concurrent_trades: number;
  websocket_connections: number;
  api_requests_per_second: number;
  database_connections: number;
  cache_hit_rate: number;
  queue_size: number;
}

interface RevenueAnalytics {
  total_revenue: number;
  revenue_by_period: DataPoint[];
  revenue_by_product: {
    product_name: string;
    revenue: number;
    percentage: number;
  }[];
  subscription_metrics: {
    active_subscriptions: number;
    new_subscriptions: number;
    churned_subscriptions: number;
    mrr: number;
    arr: number;
  };
  payment_methods: {
    method: string;
    count: number;
    total_amount: number;
  }[];
}

export function AnalyticsServerData() {
  const [systemMetrics, setSystemMetrics] = useState<SystemMetrics | null>(null);
  const [analyticsData, setAnalyticsData] = useState<AnalyticsData | null>(null);
  const [realtimeMetrics, setRealtimeMetrics] = useState<RealtimeMetrics | null>(null);
  const [revenueAnalytics, setRevenueAnalytics] = useState<RevenueAnalytics | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [activeTab, setActiveTab] = useState('system');

  const fetchData = async (type: string) => {
    setLoading(true);
    setError(null);

    try {
      switch (type) {
        case 'system': {
          const response = await analyticsClient.getSystemMetrics();
          if (response.data) {
            setSystemMetrics(response.data);
          } else if (response.error) {
            setError(`System metrics error: ${response.error}`);
          }
          break;
        }
        case 'analytics': {
          const response = await analyticsClient.getAnalyticsData();
          if (response.data) {
            setAnalyticsData(response.data);
          } else if (response.error) {
            setError(`Analytics data error: ${response.error}`);
          }
          break;
        }
        case 'realtime': {
          const response = await analyticsClient.getRealtimeMetrics();
          if (response.data) {
            setRealtimeMetrics(response.data);
          } else if (response.error) {
            setError(`Realtime metrics error: ${response.error}`);
          }
          break;
        }
        case 'revenue': {
          const response = await analyticsClient.getRevenueAnalytics();
          if (response.data) {
            setRevenueAnalytics(response.data);
          } else if (response.error) {
            setError(`Revenue analytics error: ${response.error}`);
          }
          break;
        }
      }
    } catch (err) {
      setError(`Failed to fetch ${type} data: ${err instanceof Error ? err.message : 'Unknown error'}`);
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    fetchData(activeTab);
  }, [activeTab]);

  const formatUptime = (seconds: number) => {
    const days = Math.floor(seconds / 86400);
    const hours = Math.floor((seconds % 86400) / 3600);
    const mins = Math.floor((seconds % 3600) / 60);
    return `${days}d ${hours}h ${mins}m`;
  };

  const formatCurrency = (amount: number) => {
    return new Intl.NumberFormat('en-US', {
      style: 'currency',
      currency: 'USD',
    }).format(amount);
  };

  return (
    <div className="space-y-8">
      {/* Header */}
      <div className="text-center space-y-4">
        <div className="flex items-center justify-center gap-3">
          <BarChart3 className="h-10 w-10 text-blue-600" />
          <h1 className="text-4xl font-bold">Backend Analytics</h1>
        </div>
        <p className="text-xl text-muted-foreground">
          Real-time system metrics and business analytics from the Rust backend
        </p>
      </div>

      {/* Error Display */}
      {error && (
        <Card className="border-red-200 bg-red-50 dark:bg-red-950/20">
          <CardContent className="p-4">
            <div className="flex items-center gap-2 text-red-800 dark:text-red-200">
              <AlertCircle className="h-4 w-4" />
              <span className="text-sm">{error}</span>
              <Button
                variant="outline"
                size="sm"
                onClick={() => fetchData(activeTab)}
                className="ml-auto"
              >
                <RefreshCw className="h-4 w-4 mr-2" />
                Retry
              </Button>
            </div>
          </CardContent>
        </Card>
      )}

      {/* Tabs */}
      <Tabs value={activeTab} onValueChange={setActiveTab} className="space-y-6">
        <TabsList className="grid w-full grid-cols-4">
          <TabsTrigger value="system" className="gap-2">
            <Activity className="h-4 w-4" />
            System
          </TabsTrigger>
          <TabsTrigger value="analytics" className="gap-2">
            <BarChart3 className="h-4 w-4" />
            Analytics
          </TabsTrigger>
          <TabsTrigger value="realtime" className="gap-2">
            <TrendingUp className="h-4 w-4" />
            Realtime
          </TabsTrigger>
          <TabsTrigger value="revenue" className="gap-2">
            <DollarSign className="h-4 w-4" />
            Revenue
          </TabsTrigger>
        </TabsList>

        {/* System Metrics Tab */}
        <TabsContent value="system" className="space-y-6">
          {loading ? (
            <div className="grid gap-4 md:grid-cols-3">
              {Array.from({ length: 6 }).map((_, i) => (
                <Card key={i} className="animate-pulse">
                  <CardContent className="p-4">
                    <div className="h-20 bg-gray-200 rounded"></div>
                  </CardContent>
                </Card>
              ))}
            </div>
          ) : systemMetrics ? (
            <div className="grid gap-4 md:grid-cols-2 lg:grid-cols-3">
              <Card>
                <CardHeader className="pb-2">
                  <CardTitle className="text-sm font-medium">CPU Usage</CardTitle>
                </CardHeader>
                <CardContent>
                  <div className="text-2xl font-bold">{systemMetrics.cpu_usage}%</div>
                  <Badge variant={systemMetrics.cpu_usage > 80 ? 'destructive' : 'secondary'}>
                    {systemMetrics.cpu_usage > 80 ? 'High' : 'Normal'}
                  </Badge>
                </CardContent>
              </Card>

              <Card>
                <CardHeader className="pb-2">
                  <CardTitle className="text-sm font-medium">Memory Usage</CardTitle>
                </CardHeader>
                <CardContent>
                  <div className="text-2xl font-bold">{systemMetrics.memory_usage}%</div>
                  <Badge variant={systemMetrics.memory_usage > 85 ? 'destructive' : 'secondary'}>
                    {systemMetrics.memory_usage > 85 ? 'High' : 'Normal'}
                  </Badge>
                </CardContent>
              </Card>

              <Card>
                <CardHeader className="pb-2">
                  <CardTitle className="text-sm font-medium">Active Connections</CardTitle>
                </CardHeader>
                <CardContent>
                  <div className="text-2xl font-bold">{systemMetrics.active_connections}</div>
                  <p className="text-xs text-muted-foreground">Current connections</p>
                </CardContent>
              </Card>

              <Card>
                <CardHeader className="pb-2">
                  <CardTitle className="text-sm font-medium">Requests/Min</CardTitle>
                </CardHeader>
                <CardContent>
                  <div className="text-2xl font-bold">{systemMetrics.requests_per_minute}</div>
                  <p className="text-xs text-muted-foreground">Request rate</p>
                </CardContent>
              </Card>

              <Card>
                <CardHeader className="pb-2">
                  <CardTitle className="text-sm font-medium">Response Time</CardTitle>
                </CardHeader>
                <CardContent>
                  <div className="text-2xl font-bold">{systemMetrics.response_time_avg}ms</div>
                  <Badge variant={systemMetrics.response_time_avg > 200 ? 'destructive' : 'secondary'}>
                    {systemMetrics.response_time_avg > 200 ? 'Slow' : 'Fast'}
                  </Badge>
                </CardContent>
              </Card>

              <Card>
                <CardHeader className="pb-2">
                  <CardTitle className="text-sm font-medium">Uptime</CardTitle>
                </CardHeader>
                <CardContent>
                  <div className="text-2xl font-bold">{formatUptime(systemMetrics.uptime_seconds)}</div>
                  <p className="text-xs text-muted-foreground">System uptime</p>
                </CardContent>
              </Card>
            </div>
          ) : null}
        </TabsContent>

        {/* Analytics Data Tab */}
        <TabsContent value="analytics" className="space-y-6">
          {loading ? (
            <div className="animate-pulse space-y-4">
              <div className="h-64 bg-gray-200 rounded"></div>
              <div className="grid gap-4 md:grid-cols-2">
                <div className="h-32 bg-gray-200 rounded"></div>
                <div className="h-32 bg-gray-200 rounded"></div>
              </div>
            </div>
          ) : analyticsData ? (
            <div className="space-y-6">
              {/* Popular Symbols */}
              <Card>
                <CardHeader>
                  <CardTitle>Popular Symbols</CardTitle>
                </CardHeader>
                <CardContent>
                  <div className="space-y-2">
                    {analyticsData.popular_symbols.map((symbol) => (
                      <div key={symbol.symbol} className="flex items-center justify-between p-2 border rounded">
                        <Badge variant="outline">{symbol.symbol}</Badge>
                        <div className="text-sm text-muted-foreground">
                          Volume: {symbol.volume.toLocaleString()} | Count: {symbol.count}
                        </div>
                      </div>
                    ))}
                  </div>
                </CardContent>
              </Card>

              {/* User Activity */}
              <Card>
                <CardHeader>
                  <CardTitle>User Activity by Hour</CardTitle>
                </CardHeader>
                <CardContent>
                  <div className="space-y-2">
                    {analyticsData.user_activity.map((activity) => (
                      <div key={activity.hour} className="flex items-center justify-between p-2 border rounded">
                        <div className="font-medium">{activity.hour}:00</div>
                        <div className="flex gap-4 text-sm text-muted-foreground">
                          <span>Users: {activity.active_users}</span>
                          <span>Transactions: {activity.transactions}</span>
                        </div>
                      </div>
                    ))}
                  </div>
                </CardContent>
              </Card>
            </div>
          ) : null}
        </TabsContent>

        {/* Realtime Metrics Tab */}
        <TabsContent value="realtime" className="space-y-6">
          {loading ? (
            <div className="grid gap-4 md:grid-cols-3">
              {Array.from({ length: 6 }).map((_, i) => (
                <Card key={i} className="animate-pulse">
                  <CardContent className="p-4">
                    <div className="h-20 bg-gray-200 rounded"></div>
                  </CardContent>
                </Card>
              ))}
            </div>
          ) : realtimeMetrics ? (
            <div className="grid gap-4 md:grid-cols-2 lg:grid-cols-3">
              <Card>
                <CardHeader className="pb-2">
                  <CardTitle className="text-sm font-medium">Active Users</CardTitle>
                </CardHeader>
                <CardContent>
                  <div className="text-2xl font-bold text-green-600">{realtimeMetrics.active_users}</div>
                </CardContent>
              </Card>

              <Card>
                <CardHeader className="pb-2">
                  <CardTitle className="text-sm font-medium">Concurrent Trades</CardTitle>
                </CardHeader>
                <CardContent>
                  <div className="text-2xl font-bold text-blue-600">{realtimeMetrics.concurrent_trades}</div>
                </CardContent>
              </Card>

              <Card>
                <CardHeader className="pb-2">
                  <CardTitle className="text-sm font-medium">WebSocket Connections</CardTitle>
                </CardHeader>
                <CardContent>
                  <div className="text-2xl font-bold text-purple-600">{realtimeMetrics.websocket_connections}</div>
                </CardContent>
              </Card>

              <Card>
                <CardHeader className="pb-2">
                  <CardTitle className="text-sm font-medium">API Requests/sec</CardTitle>
                </CardHeader>
                <CardContent>
                  <div className="text-2xl font-bold text-orange-600">{realtimeMetrics.api_requests_per_second}</div>
                </CardContent>
              </Card>

              <Card>
                <CardHeader className="pb-2">
                  <CardTitle className="text-sm font-medium">Cache Hit Rate</CardTitle>
                </CardHeader>
                <CardContent>
                  <div className="text-2xl font-bold text-teal-600">{Math.round(realtimeMetrics.cache_hit_rate * 100)}%</div>
                </CardContent>
              </Card>

              <Card>
                <CardHeader className="pb-2">
                  <CardTitle className="text-sm font-medium">Queue Size</CardTitle>
                </CardHeader>
                <CardContent>
                  <div className="text-2xl font-bold text-red-600">{realtimeMetrics.queue_size}</div>
                </CardContent>
              </Card>
            </div>
          ) : null}
        </TabsContent>

        {/* Revenue Analytics Tab */}
        <TabsContent value="revenue" className="space-y-6">
          {loading ? (
            <div className="space-y-4">
              <div className="h-32 bg-gray-200 rounded animate-pulse"></div>
              <div className="grid gap-4 md:grid-cols-2">
                <div className="h-64 bg-gray-200 rounded animate-pulse"></div>
                <div className="h-64 bg-gray-200 rounded animate-pulse"></div>
              </div>
            </div>
          ) : revenueAnalytics ? (
            <div className="space-y-6">
              {/* Revenue Overview */}
              <Card>
                <CardHeader>
                  <CardTitle>Revenue Overview</CardTitle>
                </CardHeader>
                <CardContent>
                  <div className="text-3xl font-bold text-green-600 mb-4">
                    {formatCurrency(revenueAnalytics.total_revenue)}
                  </div>
                  <div className="grid gap-4 md:grid-cols-4 text-center">
                    <div>
                      <div className="text-2xl font-bold">{revenueAnalytics.subscription_metrics.active_subscriptions}</div>
                      <p className="text-xs text-muted-foreground">Active Subscriptions</p>
                    </div>
                    <div>
                      <div className="text-2xl font-bold text-green-600">+{revenueAnalytics.subscription_metrics.new_subscriptions}</div>
                      <p className="text-xs text-muted-foreground">New This Month</p>
                    </div>
                    <div>
                      <div className="text-2xl font-bold">{formatCurrency(revenueAnalytics.subscription_metrics.mrr)}</div>
                      <p className="text-xs text-muted-foreground">MRR</p>
                    </div>
                    <div>
                      <div className="text-2xl font-bold">{formatCurrency(revenueAnalytics.subscription_metrics.arr)}</div>
                      <p className="text-xs text-muted-foreground">ARR</p>
                    </div>
                  </div>
                </CardContent>
              </Card>

              {/* Revenue by Product */}
              <Card>
                <CardHeader>
                  <CardTitle>Revenue by Product</CardTitle>
                </CardHeader>
                <CardContent>
                  <div className="space-y-2">
                    {revenueAnalytics.revenue_by_product.map((product) => (
                      <div key={product.product_name} className="flex items-center justify-between p-3 border rounded">
                        <div>
                          <div className="font-medium">{product.product_name}</div>
                          <Badge variant="outline">{product.percentage}% of total</Badge>
                        </div>
                        <div className="text-lg font-bold">{formatCurrency(product.revenue)}</div>
                      </div>
                    ))}
                  </div>
                </CardContent>
              </Card>

              {/* Payment Methods */}
              <Card>
                <CardHeader>
                  <CardTitle>Payment Methods</CardTitle>
                </CardHeader>
                <CardContent>
                  <div className="space-y-2">
                    {revenueAnalytics.payment_methods.map((method) => (
                      <div key={method.method} className="flex items-center justify-between p-3 border rounded">
                        <div>
                          <div className="font-medium">{method.method}</div>
                          <div className="text-sm text-muted-foreground">{method.count} transactions</div>
                        </div>
                        <div className="text-lg font-bold">{formatCurrency(method.total_amount)}</div>
                      </div>
                    ))}
                  </div>
                </CardContent>
              </Card>
            </div>
          ) : null}
        </TabsContent>
      </Tabs>

      {/* Refresh Button */}
      <div className="flex justify-center">
        <Button
          onClick={() => fetchData(activeTab)}
          disabled={loading}
          className="gap-2"
        >
          <RefreshCw className={`h-4 w-4 ${loading ? 'animate-spin' : ''}`} />
          {loading ? 'Loading...' : 'Refresh Data'}
        </Button>
      </div>
    </div>
  );
}