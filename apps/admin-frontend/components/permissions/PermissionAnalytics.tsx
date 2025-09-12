/**
 * Permission Analytics - Health, Timeline, and Analytics
 * Consolidates: PermissionHealthDashboard, PermissionTimeline, PermissionAnalyticsDashboard,
 * RBACAnalyticsDashboard, permission_analytics.tsx, PermissionStatsCards, PermissionExpiryIndicator
 */

'use client';

import { useState, useEffect, useMemo } from 'react';
import { 
  TrendingUp, 
  TrendingDown, 
  Activity, 
  Clock, 
  Shield, 
  Users, 
  AlertTriangle,
  CheckCircle,
  BarChart3,
  Calendar,
  Filter,
  Download,
  RefreshCw
} from 'lucide-react';

import { Button } from '@/components/ui/button';
import { Card } from '@/components/ui/card';
import { Badge } from '@/components/ui/badge';
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select';
import { Progress } from '@/components/ui/progress';

import type { PermissionAnalytics, User, PermissionHealth, Platform } from '@/types/core';
import { adminClient } from '@/lib/api/unified-admin-client';

interface PermissionAnalyticsProps {
  users?: User[];
  onRefresh?: () => void;
  className?: string;
}

interface TimelineEvent {
  id: string;
  type: 'granted' | 'revoked' | 'expired' | 'extended' | 'requested';
  userId: string;
  userEmail: string;
  permission: string;
  platform: Platform;
  timestamp: string;
  performedBy: string;
  details?: string;
}

interface HealthMetric {
  label: string;
  value: number;
  total: number;
  status: 'healthy' | 'warning' | 'critical';
  trend: 'up' | 'down' | 'neutral';
  description: string;
}

export function PermissionAnalytics({
  users = [],
  onRefresh,
  className = ''
}: PermissionAnalyticsProps) {
  const [analytics, setAnalytics] = useState<PermissionAnalytics | null>(null);
  const [timeline, setTimeline] = useState<TimelineEvent[]>([]);
  const [healthData, setHealthData] = useState<PermissionHealth[]>([]);
  const [isLoading, setIsLoading] = useState(false);
  const [activeTab, setActiveTab] = useState('overview');
  const [timeRange, setTimeRange] = useState('30d');
  const [platformFilter, setPlatformFilter] = useState('all');
  const [lastUpdated, setLastUpdated] = useState<Date | null>(null);

  // Health metrics calculation
  const healthMetrics = useMemo((): HealthMetric[] => {
    if (!analytics) return [];

    const totalPermissions = analytics.totalPermissions;
    const activePermissions = totalPermissions - analytics.expired;
    const healthScore = analytics.healthScore;

    return [
      {
        label: 'Overall Health',
        value: healthScore,
        total: 100,
        status: healthScore >= 80 ? 'healthy' : healthScore >= 60 ? 'warning' : 'critical',
        trend: 'neutral',
        description: 'Overall permission system health score'
      },
      {
        label: 'Active Permissions',
        value: activePermissions,
        total: totalPermissions,
        status: activePermissions > 0 ? 'healthy' : 'warning',
        trend: 'up',
        description: 'Permissions currently in use'
      },
      {
        label: 'Expiring Soon',
        value: analytics.expiringSoon,
        total: totalPermissions,
        status: analytics.expiringSoon === 0 ? 'healthy' : analytics.expiringSoon < 10 ? 'warning' : 'critical',
        trend: 'down',
        description: 'Permissions expiring within 7 days'
      },
      {
        label: 'Recent Activity',
        value: analytics.recentActivity,
        total: 100,
        status: analytics.recentActivity > 0 ? 'healthy' : 'warning',
        trend: 'up',
        description: 'Permission changes in the last 24 hours'
      }
    ];
  }, [analytics]);

  // Platform distribution data
  const platformDistribution = useMemo(() => {
    if (!timeline.length) return {};

    const distribution: Record<Platform, number> = {} as any;
    timeline.forEach(event => {
      distribution[event.platform] = (distribution[event.platform] || 0) + 1;
    });

    return distribution;
  }, [timeline]);

  // Load analytics data
  const loadAnalytics = async () => {
    setIsLoading(true);
    try {
      const response = await adminClient.getPermissionAnalytics();
      if (response.success) {
        setAnalytics(response.data);
      }
    } catch (error) {
      console.error('Failed to load permission analytics:', error);
    } finally {
      setIsLoading(false);
      setLastUpdated(new Date());
    }
  };

  // Load timeline data (mock implementation)
  const loadTimeline = async () => {
    // In a real implementation, this would fetch from the API
    const mockTimeline: TimelineEvent[] = [
      {
        id: '1',
        type: 'granted',
        userId: 'user1',
        userEmail: 'john@example.com',
        permission: 'epsx:analytics:view',
        platform: 'epsx',
        timestamp: new Date(Date.now() - 3600000).toISOString(),
        performedBy: 'admin@epsx.io',
        details: 'Requested for quarterly report access'
      },
      {
        id: '2',
        type: 'expired',
        userId: 'user2',
        userEmail: 'jane@example.com',
        permission: 'epsx:export:csv',
        platform: 'epsx',
        timestamp: new Date(Date.now() - 7200000).toISOString(),
        performedBy: 'system',
        details: '30-day temporary access expired'
      },
      {
        id: '3',
        type: 'revoked',
        userId: 'user3',
        userEmail: 'mike@example.com',
        permission: 'admin:users:manage',
        platform: 'admin',
        timestamp: new Date(Date.now() - 10800000).toISOString(),
        performedBy: 'admin@epsx.io',
        details: 'User role change from admin to user'
      }
    ];
    
    setTimeline(mockTimeline);
  };

  // Load health data for users
  const loadHealthData = async () => {
    const mockHealthData: PermissionHealth[] = users.slice(0, 5).map(user => ({
      userId: user.id,
      totalPermissions: Math.floor(Math.random() * 20) + 5,
      expiringPermissions: Math.random() > 0.7 ? [`epsx:analytics:view:${Math.floor(Date.now() / 1000) + 86400}`] : [],
      expiredPermissions: Math.random() > 0.8 ? ['epsx:export:csv'] : [],
      healthScore: Math.floor(Math.random() * 40) + 60,
      lastCalculated: new Date().toISOString(),
      recommendations: Math.random() > 0.5 ? ['Review expiring permissions', 'Clean up unused access'] : []
    }));
    
    setHealthData(mockHealthData);
  };

  // Handle refresh
  const handleRefresh = async () => {
    await Promise.all([
      loadAnalytics(),
      loadTimeline(),
      loadHealthData()
    ]);
    onRefresh?.();
  };

  // Initial load
  useEffect(() => {
    handleRefresh();
  }, []);

  // Filter timeline by platform
  const filteredTimeline = useMemo(() => {
    if (platformFilter === 'all') return timeline;
    return timeline.filter(event => event.platform === platformFilter);
  }, [timeline, platformFilter]);

  // Get timeline icon
  const getTimelineIcon = (type: TimelineEvent['type']) => {
    switch (type) {
      case 'granted': return <Shield className="w-4 h-4 text-green-500" />;
      case 'revoked': return <Shield className="w-4 h-4 text-red-500" />;
      case 'expired': return <Clock className="w-4 h-4 text-orange-500" />;
      case 'extended': return <TrendingUp className="w-4 h-4 text-blue-500" />;
      case 'requested': return <AlertTriangle className="w-4 h-4 text-yellow-500" />;
      default: return <Activity className="w-4 h-4 text-gray-500" />;
    }
  };

  // Get health status color
  const getHealthColor = (status: HealthMetric['status']) => {
    switch (status) {
      case 'healthy': return 'text-green-400';
      case 'warning': return 'text-yellow-400';
      case 'critical': return 'text-red-400';
      default: return 'text-gray-400';
    }
  };

  return (
    <div className={`space-y-6 ${className}`}>
      {/* Header */}
      <div className="flex flex-col sm:flex-row justify-between items-start sm:items-center gap-4">
        <div>
          <h2 className="text-2xl font-bold text-white">Permission Analytics</h2>
          <p className="text-gray-400">
            Monitor permission health, activity, and trends
            {lastUpdated && (
              <span className="ml-2 text-xs">
                (Updated {lastUpdated.toLocaleTimeString()})
              </span>
            )}
          </p>
        </div>
        <div className="flex gap-2">
          <Button
            variant="outline"
            size="sm"
            onClick={handleRefresh}
            disabled={isLoading}
          >
            <RefreshCw className={`w-4 h-4 mr-1 ${isLoading ? 'animate-spin' : ''}`} />
            Refresh
          </Button>
          <Button variant="outline" size="sm">
            <Download className="w-4 h-4 mr-1" />
            Export
          </Button>
        </div>
      </div>

      {/* Health Metrics Overview */}
      <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-4">
        {healthMetrics.map((metric, index) => (
          <Card key={index} className="p-4">
            <div className="flex items-center justify-between mb-2">
              <h3 className="text-sm font-medium text-gray-300">{metric.label}</h3>
              <div className={getHealthColor(metric.status)}>
                {metric.trend === 'up' && <TrendingUp className="w-4 h-4" />}
                {metric.trend === 'down' && <TrendingDown className="w-4 h-4" />}
                {metric.trend === 'neutral' && <Activity className="w-4 h-4" />}
              </div>
            </div>
            <div className="space-y-2">
              <div className="flex items-baseline gap-2">
                <span className={`text-2xl font-bold ${getHealthColor(metric.status)}`}>
                  {metric.value}
                </span>
                {metric.total !== 100 && (
                  <span className="text-sm text-gray-500">
                    / {metric.total}
                  </span>
                )}
                {metric.total === 100 && metric.value <= 100 && (
                  <span className="text-sm text-gray-500">%</span>
                )}
              </div>
              <Progress 
                value={metric.total === 100 ? metric.value : (metric.value / metric.total) * 100} 
                className="h-2"
              />
              <p className="text-xs text-gray-500">{metric.description}</p>
            </div>
          </Card>
        ))}
      </div>

      {/* Analytics Tabs */}
      <Tabs value={activeTab} onValueChange={setActiveTab}>
        <div className="flex flex-col sm:flex-row justify-between items-start sm:items-center gap-4">
          <TabsList className="grid w-full sm:w-auto grid-cols-3">
            <TabsTrigger value="overview">Overview</TabsTrigger>
            <TabsTrigger value="timeline">Activity Timeline</TabsTrigger>
            <TabsTrigger value="health">User Health</TabsTrigger>
          </TabsList>
          
          <div className="flex gap-2">
            <Select value={timeRange} onValueChange={setTimeRange}>
              <SelectTrigger className="w-32">
                <SelectValue />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="7d">Last 7 days</SelectItem>
                <SelectItem value="30d">Last 30 days</SelectItem>
                <SelectItem value="90d">Last 3 months</SelectItem>
                <SelectItem value="1y">Last year</SelectItem>
              </SelectContent>
            </Select>
            
            <Select value={platformFilter} onValueChange={setPlatformFilter}>
              <SelectTrigger className="w-40">
                <SelectValue placeholder="All Platforms" />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="all">All Platforms</SelectItem>
                <SelectItem value="admin">Admin</SelectItem>
                <SelectItem value="epsx">EPSX</SelectItem>
                <SelectItem value="epsx-pay">EPSX Pay</SelectItem>
                <SelectItem value="epsx-token">EPSX Token</SelectItem>
              </SelectContent>
            </Select>
          </div>
        </div>

        {/* Overview Tab */}
        <TabsContent value="overview" className="space-y-6">
          <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
            {/* Platform Distribution */}
            <Card className="p-6">
              <h3 className="text-lg font-medium text-white mb-4">Platform Distribution</h3>
              <div className="space-y-3">
                {Object.entries(platformDistribution).map(([platform, count]) => (
                  <div key={platform} className="flex items-center justify-between">
                    <div className="flex items-center gap-2">
                      <Badge variant="secondary" size="sm">
                        {platform}
                      </Badge>
                      <span className="text-sm text-gray-300">
                        {count} permissions
                      </span>
                    </div>
                    <div className="w-24">
                      <Progress 
                        value={(count / Object.values(platformDistribution).reduce((a, b) => a + b, 0)) * 100} 
                        className="h-2"
                      />
                    </div>
                  </div>
                ))}
              </div>
            </Card>

            {/* System Health Summary */}
            <Card className="p-6">
              <h3 className="text-lg font-medium text-white mb-4">System Health Summary</h3>
              <div className="space-y-4">
                {analytics && (
                  <>
                    <div className="flex items-center justify-between">
                      <span className="text-sm text-gray-400">Total Users with Permissions</span>
                      <span className="font-medium text-white">{analytics.usersWithPermissions}</span>
                    </div>
                    <div className="flex items-center justify-between">
                      <span className="text-sm text-gray-400">Active Permissions</span>
                      <span className="font-medium text-green-400">
                        {analytics.totalPermissions - analytics.expired}
                      </span>
                    </div>
                    <div className="flex items-center justify-between">
                      <span className="text-sm text-gray-400">Expired Permissions</span>
                      <span className="font-medium text-red-400">{analytics.expired}</span>
                    </div>
                    <div className="flex items-center justify-between">
                      <span className="text-sm text-gray-400">Overall Health Score</span>
                      <div className="flex items-center gap-2">
                        <span className={`font-medium ${getHealthColor(analytics.healthScore >= 80 ? 'healthy' : analytics.healthScore >= 60 ? 'warning' : 'critical')}`}>
                          {analytics.healthScore}%
                        </span>
                        {analytics.healthScore >= 80 ? (
                          <CheckCircle className="w-4 h-4 text-green-500" />
                        ) : (
                          <AlertTriangle className="w-4 h-4 text-yellow-500" />
                        )}
                      </div>
                    </div>
                  </>
                )}
              </div>
            </Card>
          </div>
        </TabsContent>

        {/* Timeline Tab */}
        <TabsContent value="timeline" className="space-y-4">
          <Card className="p-6">
            <h3 className="text-lg font-medium text-white mb-4">Recent Permission Activity</h3>
            
            {filteredTimeline.length === 0 ? (
              <div className="text-center py-8 text-gray-400">
                <Activity className="w-12 h-12 mx-auto mb-4 opacity-50" />
                <p>No recent activity found</p>
              </div>
            ) : (
              <div className="space-y-4">
                {filteredTimeline.map((event, index) => (
                  <div key={event.id} className="flex gap-4 p-3 bg-gray-800/30 rounded-lg">
                    <div className="flex-shrink-0 mt-1">
                      {getTimelineIcon(event.type)}
                    </div>
                    <div className="flex-1 min-w-0">
                      <div className="flex items-start justify-between gap-2">
                        <div>
                          <p className="text-sm font-medium text-white">
                            Permission {event.type} for {event.userEmail}
                          </p>
                          <div className="flex items-center gap-2 mt-1">
                            <Badge variant="secondary" size="sm">
                              {event.platform}
                            </Badge>
                            <code className="text-xs bg-gray-800 px-2 py-1 rounded">
                              {event.permission}
                            </code>
                          </div>
                          {event.details && (
                            <p className="text-xs text-gray-400 mt-2">
                              {event.details}
                            </p>
                          )}
                        </div>
                        <div className="text-right text-xs text-gray-500">
                          <p>{new Date(event.timestamp).toLocaleTimeString()}</p>
                          <p>by {event.performedBy}</p>
                        </div>
                      </div>
                    </div>
                  </div>
                ))}
              </div>
            )}
          </Card>
        </TabsContent>

        {/* User Health Tab */}
        <TabsContent value="health" className="space-y-4">
          <div className="grid gap-4">
            {healthData.map((health) => {
              const user = users.find(u => u.id === health.userId);
              if (!user) return null;

              return (
                <Card key={health.userId} className="p-4">
                  <div className="flex items-start justify-between mb-4">
                    <div className="flex items-center gap-3">
                      <div className="w-10 h-10 rounded-full bg-blue-500/20 flex items-center justify-center font-medium">
                        {user.email[0].toUpperCase()}
                      </div>
                      <div>
                        <h3 className="font-medium text-white">{user.email}</h3>
                        <p className="text-sm text-gray-400">{user.name || 'No name'}</p>
                      </div>
                    </div>
                    <div className="text-right">
                      <div className={`text-lg font-bold ${getHealthColor(health.healthScore >= 80 ? 'healthy' : health.healthScore >= 60 ? 'warning' : 'critical')}`}>
                        {health.healthScore}%
                      </div>
                      <p className="text-xs text-gray-500">Health Score</p>
                    </div>
                  </div>

                  <div className="grid grid-cols-1 sm:grid-cols-3 gap-4 mb-4">
                    <div className="text-center">
                      <div className="text-lg font-medium text-white">{health.totalPermissions}</div>
                      <p className="text-xs text-gray-400">Total Permissions</p>
                    </div>
                    <div className="text-center">
                      <div className="text-lg font-medium text-yellow-400">{health.expiringPermissions.length}</div>
                      <p className="text-xs text-gray-400">Expiring Soon</p>
                    </div>
                    <div className="text-center">
                      <div className="text-lg font-medium text-red-400">{health.expiredPermissions.length}</div>
                      <p className="text-xs text-gray-400">Expired</p>
                    </div>
                  </div>

                  {health.recommendations && health.recommendations.length > 0 && (
                    <div className="mt-4 p-3 bg-yellow-500/10 border border-yellow-500/20 rounded-lg">
                      <h4 className="text-sm font-medium text-yellow-400 mb-2">Recommendations</h4>
                      <ul className="space-y-1">
                        {health.recommendations.map((rec, index) => (
                          <li key={index} className="text-xs text-gray-300 flex items-start gap-2">
                            <AlertTriangle className="w-3 h-3 text-yellow-500 mt-0.5 flex-shrink-0" />
                            {rec}
                          </li>
                        ))}
                      </ul>
                    </div>
                  )}
                </Card>
              );
            })}

            {healthData.length === 0 && (
              <Card className="p-8 text-center">
                <Users className="w-12 h-12 text-gray-400 mx-auto mb-4" />
                <h3 className="text-lg font-medium text-white mb-2">No Health Data Available</h3>
                <p className="text-gray-400">Health data will appear here as users are analyzed.</p>
              </Card>
            )}
          </div>
        </TabsContent>
      </Tabs>
    </div>
  );
}

export default PermissionAnalytics;