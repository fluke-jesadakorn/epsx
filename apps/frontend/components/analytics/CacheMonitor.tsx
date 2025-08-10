'use client';

import React, { useState, useEffect, useCallback } from 'react';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { Badge } from '@/components/ui/badge';
import { Button } from '@/components/ui/button';
import { Progress } from '@/components/ui/progress';
import { 
  RefreshCw, 
  Database, 
  Activity, 
  TrendingUp,
  AlertTriangle,
  CheckCircle,
  BarChart3,
  Clock
} from 'lucide-react';
import { AnalyticsClient } from '@epsx/api-client';
import type { 
  CacheStatsResponse,
  CacheHealthResponse 
} from '@epsx/api-client';
import { useToast } from '@/components/ui/use-toast';

export function CacheMonitor() {
  const { toast } = useToast();
  const [cacheStats, setCacheStats] = useState<CacheStatsResponse | null>(null);
  const [cacheHealth, setCacheHealth] = useState<CacheHealthResponse | null>(null);
  const [loading, setLoading] = useState(true);
  const [refreshing, setRefreshing] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const analyticsClient = new AnalyticsClient();

  const loadCacheData = useCallback(async () => {
    try {
      setError(null);
      
      const [statsResponse, healthResponse] = await Promise.all([
        analyticsClient.getCacheStats(),
        analyticsClient.getCacheHealth()
      ]);
      
      if (statsResponse.data) {
        setCacheStats(statsResponse.data);
      } else {
        throw new Error('Failed to load cache stats');
      }
      
      if (healthResponse.data) {
        setCacheHealth(healthResponse.data);
      } else {
        throw new Error('Failed to load cache health');
      }
    } catch (error) {
      console.error('Failed to load cache data:', error);
      setError(`Failed to load cache data: ${error instanceof Error ? error.message : String(error)}`);
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    loadCacheData();
    
    // Auto-refresh cache stats every 30 seconds
    const interval = setInterval(loadCacheData, 30000);
    return () => clearInterval(interval);
  }, [loadCacheData]);

  const handleRefreshCache = async () => {
    setRefreshing(true);
    
    try {
      const response = await analyticsClient.refreshCache();
      
      if (response.data?.success) {
        toast({
          title: "Cache Refreshed Successfully",
          description: `Refreshed ${response.data.refreshed_entries} entries in ${response.data.duration_ms}ms`,
          variant: "default"
        });
        
        // Reload stats after refresh
        await loadCacheData();
      } else {
        throw new Error('Failed to refresh cache');
      }
    } catch (error) {
      toast({
        title: "Cache Refresh Failed",
        description: `Error: ${error instanceof Error ? error.message : String(error)}`,
        variant: "destructive"
      });
    } finally {
      setRefreshing(false);
    }
  };

  const getHealthColor = (healthy: boolean) => {
    return healthy ? 'text-green-600' : 'text-red-600';
  };

  const getHealthIcon = (healthy: boolean) => {
    return healthy ? CheckCircle : AlertTriangle;
  };

  const formatTimestamp = (timestamp: string) => {
    return new Date(timestamp).toLocaleTimeString();
  };

  if (loading) {
    return (
      <div className="grid gap-4 md:grid-cols-2 lg:grid-cols-4">
        {Array.from({ length: 4 }).map((_, i) => (
          <Card key={i}>
            <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
              <div className="h-4 bg-gray-200 rounded w-20 animate-pulse"></div>
              <div className="h-4 w-4 bg-gray-200 rounded animate-pulse"></div>
            </CardHeader>
            <CardContent>
              <div className="h-8 bg-gray-200 rounded w-16 animate-pulse mb-1"></div>
              <div className="h-3 bg-gray-200 rounded w-24 animate-pulse"></div>
            </CardContent>
          </Card>
        ))}
      </div>
    );
  }

  if (error) {
    return (
      <Card className="border-red-200 bg-red-50 dark:bg-red-950/20">
        <CardContent className="p-6 text-center">
          <AlertTriangle className="h-12 w-12 text-red-500 mx-auto mb-4" />
          <h3 className="text-lg font-semibold mb-2">Cache Monitor Error</h3>
          <p className="text-red-700 dark:text-red-300 mb-4">{error}</p>
          <Button onClick={loadCacheData} variant="outline">
            <RefreshCw className="h-4 w-4 mr-2" />
            Retry
          </Button>
        </CardContent>
      </Card>
    );
  }

  return (
    <div className="space-y-6">
      {/* Main Stats Cards */}
      <div className="grid gap-4 md:grid-cols-2 lg:grid-cols-4">
        {/* Active Entries */}
        {cacheStats && (
          <Card>
            <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
              <CardTitle className="text-sm font-medium">Active Entries</CardTitle>
              <Database className="h-4 w-4 text-muted-foreground" />
            </CardHeader>
            <CardContent>
              <div className="text-2xl font-bold">{cacheStats.stats.active_entries}</div>
              <p className="text-xs text-muted-foreground">
                {cacheStats.stats.total_entries} total
              </p>
              <Progress 
                value={(cacheStats.stats.active_entries / cacheStats.stats.total_entries) * 100} 
                className="mt-2"
              />
            </CardContent>
          </Card>
        )}

        {/* Hit Ratio */}
        {cacheStats && (
          <Card>
            <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
              <CardTitle className="text-sm font-medium">Hit Ratio</CardTitle>
              <TrendingUp className="h-4 w-4 text-muted-foreground" />
            </CardHeader>
            <CardContent>
              <div className="text-2xl font-bold">{(cacheStats.stats.hit_ratio * 100).toFixed(1)}%</div>
              <p className="text-xs text-muted-foreground">
                Cache efficiency
              </p>
              <Progress 
                value={cacheStats.stats.hit_ratio * 100} 
                className="mt-2"
              />
            </CardContent>
          </Card>
        )}

        {/* Cache Health */}
        {cacheHealth && (
          <Card>
            <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
              <CardTitle className="text-sm font-medium">Cache Health</CardTitle>
              {React.createElement(getHealthIcon(cacheHealth.healthy), {
                className: `h-4 w-4 ${getHealthColor(cacheHealth.healthy)}`
              })}
            </CardHeader>
            <CardContent>
              <div className={`text-2xl font-bold ${getHealthColor(cacheHealth.healthy)}`}>
                {cacheHealth.status.toUpperCase()}
              </div>
              <p className="text-xs text-muted-foreground">
                {cacheHealth.cache_stats.cache_size_mb.toFixed(1)}MB used
              </p>
            </CardContent>
          </Card>
        )}

        {/* Cache Control */}
        <Card>
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle className="text-sm font-medium">Cache Control</CardTitle>
            <RefreshCw className="h-4 w-4 text-muted-foreground" />
          </CardHeader>
          <CardContent>
            <Button 
              onClick={handleRefreshCache}
              disabled={refreshing}
              className="w-full mb-2"
              variant="outline"
              size="sm"
            >
              {refreshing ? (
                <>
                  <RefreshCw className="h-3 w-3 mr-1 animate-spin" />
                  Refreshing...
                </>
              ) : (
                <>
                  <RefreshCw className="h-3 w-3 mr-1" />
                  Refresh Cache
                </>
              )}
            </Button>
            <p className="text-xs text-muted-foreground">
              Last updated: {cacheStats && formatTimestamp(cacheStats.timestamp)}
            </p>
          </CardContent>
        </Card>
      </div>

      {/* Detailed Stats */}
      {(cacheStats || cacheHealth) && (
        <div className="grid gap-4 md:grid-cols-2">
          {/* Cache Statistics */}
          {cacheStats && (
            <Card>
              <CardHeader>
                <CardTitle className="flex items-center gap-2">
                  <BarChart3 className="h-5 w-5" />
                  Cache Statistics
                </CardTitle>
              </CardHeader>
              <CardContent>
                <div className="space-y-4">
                  <div className="grid grid-cols-2 gap-4">
                    <div>
                      <div className="text-sm text-muted-foreground">Total Entries</div>
                      <div className="text-lg font-semibold">{cacheStats.stats.total_entries}</div>
                    </div>
                    <div>
                      <div className="text-sm text-muted-foreground">Active Entries</div>
                      <div className="text-lg font-semibold">{cacheStats.stats.active_entries}</div>
                    </div>
                    <div>
                      <div className="text-sm text-muted-foreground">Expired Entries</div>
                      <div className="text-lg font-semibold">{cacheStats.stats.expired_entries}</div>
                    </div>
                    <div>
                      <div className="text-sm text-muted-foreground">Miss Ratio</div>
                      <div className="text-lg font-semibold">{(cacheStats.stats.miss_ratio * 100).toFixed(1)}%</div>
                    </div>
                  </div>
                  
                  <div className="space-y-2">
                    <div className="flex justify-between text-sm">
                      <span>Hit Ratio</span>
                      <span>{(cacheStats.stats.hit_ratio * 100).toFixed(1)}%</span>
                    </div>
                    <Progress value={cacheStats.stats.hit_ratio * 100} />
                  </div>
                  
                  <div className="space-y-2">
                    <div className="flex justify-between text-sm">
                      <span>Memory Usage</span>
                      <span>{cacheStats.stats.cache_size_mb.toFixed(2)}MB</span>
                    </div>
                  </div>
                </div>
              </CardContent>
            </Card>
          )}

          {/* Health & Recommendations */}
          {cacheHealth && (
            <Card>
              <CardHeader>
                <CardTitle className="flex items-center gap-2">
                  <Activity className={getHealthColor(cacheHealth.healthy)} />
                  Health & Recommendations
                </CardTitle>
              </CardHeader>
              <CardContent>
                <div className="space-y-4">
                  <div className="flex items-center justify-between">
                    <span className="text-sm text-muted-foreground">Status</span>
                    <Badge 
                      variant={cacheHealth.healthy ? "default" : "destructive"}
                      className="font-semibold"
                    >
                      {cacheHealth.status.toUpperCase()}
                    </Badge>
                  </div>
                  
                  <div>
                    <div className="text-sm text-muted-foreground mb-2">Recommendations</div>
                    <div className="space-y-2">
                      {cacheHealth.recommendations.length > 0 ? 
                        cacheHealth.recommendations.map((rec, index) => (
                          <Badge key={index} variant="outline" className="text-xs block">
                            {rec}
                          </Badge>
                        )) : 
                        <Badge variant="outline" className="text-xs text-green-600 block">
                          ✓ Cache is performing optimally
                        </Badge>
                      }
                    </div>
                  </div>
                  
                  <div className="pt-2 border-t">
                    <div className="flex items-center gap-2 text-xs text-muted-foreground">
                      <Clock className="h-3 w-3" />
                      Last checked: {formatTimestamp(cacheHealth.timestamp)}
                    </div>
                  </div>
                </div>
              </CardContent>
            </Card>
          )}
        </div>
      )}
    </div>
  );
}