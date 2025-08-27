"use client";

import React, { useState, useEffect } from 'react';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { Button } from '@/components/ui/button';
import { Badge } from '@/components/ui/badge';
import { Progress } from '@/components/ui/progress';
import { 
  BarChart3, 
  TrendingUp, 
  Users, 
  Shield, 
  Clock, 
  AlertTriangle,
  Download,
  RefreshCw
} from 'lucide-react';

interface PermissionAnalyticsData {
  totalUsers: number;
  totalPermissions: number;
  totalProfiles: number;
  activeTemporaryPermissions: number;
  recentAssignments: number;
  securityAlerts: number;
  usageStats: {
    profileUsage: Array<{
      profileName: string;
      assignmentCount: number;
      percentage: number;
    }>;
    permissionTrends: Array<{
      date: string;
      assignments: number;
      revocations: number;
    }>;
    userActivity: Array<{
      userId: string;
      userEmail: string;
      lastActivity: string;
      permissionCount: number;
      riskScore: number;
    }>;
  };
  performanceMetrics: {
    avgResponseTime: number;
    cacheHitRate: number;
    errorRate: number;
    throughput: number;
  };
}

interface PermissionAnalyticsDashboardProps {
  className?: string;
}

export function PermissionAnalyticsDashboard({ 
  className = '' 
}: PermissionAnalyticsDashboardProps) {
  const [data, setData] = useState<PermissionAnalyticsData | null>(null);
  const [loading, setLoading] = useState(true);
  const [selectedTimeRange, setSelectedTimeRange] = useState<'24h' | '7d' | '30d' | '90d'>('7d');
  const [autoRefresh, setAutoRefresh] = useState(false);

  const loadAnalytics = async () => {
    setLoading(true);
    try {
      // Call real analytics API
      const response = await fetch(`/api/v1/admin/analytics/permissions?range=${selectedTimeRange}`, {
        method: 'GET',
        headers: {
          'Content-Type': 'application/json',
        },
        cache: 'no-store'
      });

      if (!response.ok) {
        throw new Error(`Analytics API failed: ${response.status}`);
      }

      const analyticsData = await response.json();
      
      // Transform backend data to component format
      const transformedData: PermissionAnalyticsData = {
        totalUsers: analyticsData.total_users || 0,
        totalPermissions: analyticsData.total_permissions || 0,
        totalProfiles: analyticsData.permission_profiles?.length || 0,
        activeTemporaryPermissions: analyticsData.temporary_permissions || 0,
        recentAssignments: analyticsData.recent_assignments || 0,
        securityAlerts: analyticsData.security_alerts || 0,
        usageStats: {
          profileUsage: analyticsData.profile_usage?.map((profile: any) => ({
            profileName: profile.name,
            assignmentCount: profile.assignments,
            percentage: profile.percentage
          })) || [],
          permissionTrends: analyticsData.permission_trends || [],
          userActivity: analyticsData.user_activity?.map((user: any) => ({
            userId: user.id,
            userEmail: user.email,
            lastActivity: user.last_activity,
            permissionCount: user.permission_count,
            riskScore: user.risk_score || 0
          })) || []
        },
        performanceMetrics: {
          avgResponseTime: analyticsData.performance?.avg_response_time || 0,
          cacheHitRate: analyticsData.performance?.cache_hit_rate || 0,
          errorRate: analyticsData.performance?.error_rate || 0,
          throughput: analyticsData.performance?.throughput || 0
        }
      };
      
      setData(transformedData);
    } catch (error) {
      console.error('Failed to load analytics:', error);
      // Set empty data instead of mock data
      setData(null);
    } finally {
      setLoading(false);
    }
  };

  const exportAnalytics = async () => {
    // Mock export functionality
    const exportData = {
      generatedAt: new Date().toISOString(),
      timeRange: selectedTimeRange,
      data: data
    };
    
    const blob = new Blob([JSON.stringify(exportData, null, 2)], {
      type: 'application/json'
    });
    
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = `permission-analytics-${selectedTimeRange}-${new Date().toISOString().split('T')[0]}.json`;
    document.body.appendChild(a);
    a.click();
    document.body.removeChild(a);
    URL.revokeObjectURL(url);
  };

  const getRiskBadgeColor = (riskScore: number) => {
    if (riskScore < 3) return 'bg-green-500';
    if (riskScore < 6) return 'bg-yellow-500';
    if (riskScore < 8) return 'bg-orange-500';
    return 'bg-red-500';
  };

  const formatTimeAgo = (dateString: string) => {
    const date = new Date(dateString);
    const now = new Date();
    const diffInHours = Math.floor((now.getTime() - date.getTime()) / (1000 * 60 * 60));
    
    if (diffInHours < 1) return 'Just now';
    if (diffInHours < 24) return `${diffInHours}h ago`;
    const diffInDays = Math.floor(diffInHours / 24);
    return `${diffInDays}d ago`;
  };

  useEffect(() => {
    loadAnalytics();
  }, [selectedTimeRange]);

  useEffect(() => {
    let interval: NodeJS.Timeout;
    if (autoRefresh) {
      interval = setInterval(loadAnalytics, 30000); // Refresh every 30 seconds
    }
    return () => {
      if (interval) clearInterval(interval);
    };
  }, [autoRefresh, selectedTimeRange]);

  if (loading) {
    return (
      <div className={`space-y-6 ${className}`}>
        <div className="flex justify-between items-center">
          <h2 className="text-2xl font-bold">Permission Analytics</h2>
          <div className="flex gap-2">
            <div className="h-10 w-32 bg-gray-200 rounded animate-pulse" />
            <div className="h-10 w-24 bg-gray-200 rounded animate-pulse" />
          </div>
        </div>
        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6">
          {[...Array(4)].map((_, i) => (
            <Card key={i} className="animate-pulse">
              <CardContent className="p-6">
                <div className="h-4 bg-gray-200 rounded mb-2" />
                <div className="h-8 bg-gray-200 rounded" />
              </CardContent>
            </Card>
          ))}
        </div>
      </div>
    );
  }

  if (!data) {
    return (
      <div className={`text-center py-12 ${className}`}>
        <AlertTriangle className="mx-auto h-12 w-12 text-gray-400 mb-4" />
        <h3 className="text-lg font-medium mb-2">Failed to Load Analytics</h3>
        <p className="text-gray-600 mb-4">Unable to retrieve analytics data</p>
        <Button onClick={loadAnalytics}>Try Again</Button>
      </div>
    );
  }

  return (
    <div className={`space-y-6 ${className}`}>
      {/* Header */}
      <div className="flex justify-between items-center">
        <h2 className="text-2xl font-bold">Permission Analytics</h2>
        <div className="flex gap-2">
          <select 
            value={selectedTimeRange}
            onChange={(e) => setSelectedTimeRange(e.target.value as any)}
            className="px-3 py-2 border rounded-lg"
          >
            <option value="24h">Last 24 Hours</option>
            <option value="7d">Last 7 Days</option>
            <option value="30d">Last 30 Days</option>
            <option value="90d">Last 90 Days</option>
          </select>
          <Button 
            variant="outline" 
            size="sm"
            onClick={() => setAutoRefresh(!autoRefresh)}
            className={autoRefresh ? 'bg-blue-50 border-blue-300' : ''}
          >
            <RefreshCw className={`h-4 w-4 mr-2 ${autoRefresh ? 'animate-spin' : ''}`} />
            Auto Refresh
          </Button>
          <Button variant="outline" size="sm" onClick={exportAnalytics}>
            <Download className="h-4 w-4 mr-2" />
            Export
          </Button>
        </div>
      </div>

      {/* Key Metrics */}
      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6">
        <Card>
          <CardContent className="p-6">
            <div className="flex items-center justify-between">
              <div>
                <p className="text-sm text-gray-600">Total Users</p>
                <p className="text-3xl font-bold">{data.totalUsers.toLocaleString()}</p>
              </div>
              <Users className="h-8 w-8 text-blue-500" />
            </div>
          </CardContent>
        </Card>
        
        <Card>
          <CardContent className="p-6">
            <div className="flex items-center justify-between">
              <div>
                <p className="text-sm text-gray-600">Total Permissions</p>
                <p className="text-3xl font-bold">{data.totalPermissions.toLocaleString()}</p>
              </div>
              <Shield className="h-8 w-8 text-green-500" />
            </div>
          </CardContent>
        </Card>
        
        <Card>
          <CardContent className="p-6">
            <div className="flex items-center justify-between">
              <div>
                <p className="text-sm text-gray-600">Active Profiles</p>
                <p className="text-3xl font-bold">{data.totalProfiles}</p>
              </div>
              <BarChart3 className="h-8 w-8 text-purple-500" />
            </div>
          </CardContent>
        </Card>
        
        <Card>
          <CardContent className="p-6">
            <div className="flex items-center justify-between">
              <div>
                <p className="text-sm text-gray-600">Temporary Permissions</p>
                <p className="text-3xl font-bold">{data.activeTemporaryPermissions}</p>
              </div>
              <Clock className="h-8 w-8 text-orange-500" />
            </div>
          </CardContent>
        </Card>
      </div>

      {/* Profile Usage Statistics */}
      <Card>
        <CardHeader>
          <CardTitle className="flex items-center gap-2">
            <BarChart3 className="h-5 w-5" />
            Profile Usage Statistics
          </CardTitle>
        </CardHeader>
        <CardContent>
          <div className="space-y-4">
            {data.usageStats.profileUsage.map((profile, index) => (
              <div key={index} className="space-y-2">
                <div className="flex justify-between items-center">
                  <span className="font-medium">{profile.profileName}</span>
                  <div className="flex items-center gap-2">
                    <span className="text-sm text-gray-600">
                      {profile.assignmentCount} users
                    </span>
                    <Badge variant="secondary">
                      {profile.percentage}%
                    </Badge>
                  </div>
                </div>
                <Progress value={profile.percentage} className="h-2" />
              </div>
            ))}
          </div>
        </CardContent>
      </Card>

      {/* User Activity & Risk Analysis */}
      <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
        <Card>
          <CardHeader>
            <CardTitle className="flex items-center gap-2">
              <TrendingUp className="h-5 w-5" />
              Recent Activity Trends
            </CardTitle>
          </CardHeader>
          <CardContent>
            <div className="space-y-3">
              {data.usageStats.permissionTrends.slice(-5).map((trend, index) => (
                <div key={index} className="flex justify-between items-center py-2 border-b border-gray-100 last:border-0">
                  <span className="text-sm font-medium">
                    {new Date(trend.date).toLocaleDateString()}
                  </span>
                  <div className="flex gap-4 text-sm">
                    <span className="text-green-600">
                      +{trend.assignments} assigned
                    </span>
                    <span className="text-red-600">
                      -{trend.revocations} revoked
                    </span>
                  </div>
                </div>
              ))}
            </div>
          </CardContent>
        </Card>

        <Card>
          <CardHeader>
            <CardTitle className="flex items-center gap-2">
              <AlertTriangle className="h-5 w-5" />
              High-Risk Users
            </CardTitle>
          </CardHeader>
          <CardContent>
            <div className="space-y-3">
              {data.usageStats.userActivity
                .sort((a, b) => b.riskScore - a.riskScore)
                .slice(0, 5)
                .map((user, index) => (
                <div key={index} className="flex justify-between items-center py-2 border-b border-gray-100 last:border-0">
                  <div>
                    <p className="font-medium text-sm">{user.userEmail}</p>
                    <p className="text-xs text-gray-600">
                      {user.permissionCount} permissions • {formatTimeAgo(user.lastActivity)}
                    </p>
                  </div>
                  <Badge 
                    className={`text-white ${getRiskBadgeColor(user.riskScore)}`}
                    variant="secondary"
                  >
                    Risk: {user.riskScore.toFixed(1)}
                  </Badge>
                </div>
              ))}
            </div>
          </CardContent>
        </Card>
      </div>

      {/* Performance Metrics */}
      <Card>
        <CardHeader>
          <CardTitle>System Performance</CardTitle>
        </CardHeader>
        <CardContent>
          <div className="grid grid-cols-2 lg:grid-cols-4 gap-6">
            <div className="text-center">
              <p className="text-2xl font-bold text-blue-600">{data.performanceMetrics.avgResponseTime}ms</p>
              <p className="text-sm text-gray-600">Avg Response Time</p>
            </div>
            <div className="text-center">
              <p className="text-2xl font-bold text-green-600">{data.performanceMetrics.cacheHitRate}%</p>
              <p className="text-sm text-gray-600">Cache Hit Rate</p>
            </div>
            <div className="text-center">
              <p className="text-2xl font-bold text-orange-600">{data.performanceMetrics.errorRate}%</p>
              <p className="text-sm text-gray-600">Error Rate</p>
            </div>
            <div className="text-center">
              <p className="text-2xl font-bold text-purple-600">{data.performanceMetrics.throughput}</p>
              <p className="text-sm text-gray-600">Requests/sec</p>
            </div>
          </div>
        </CardContent>
      </Card>
    </div>
  );
}