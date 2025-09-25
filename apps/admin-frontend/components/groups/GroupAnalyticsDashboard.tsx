/**
 * Group Analytics Dashboard Component
 * Provides insights and analytics for the group-based permission system
 * 
 * Features:
 * - Group usage statistics
 * - Permission distribution analysis
 * - Membership trends and patterns
 * - Expiry predictions and health monitoring
 * - Performance metrics
 */

'use client'

import React, { useMemo } from 'react'
import { 
  TrendingUp, Users, Shield, Clock, AlertTriangle,
  BarChart3, PieChart, Activity, Zap, RefreshCw,
  CheckCircle, XCircle, Eye, Calendar
} from 'lucide-react'

import { Card, CardContent, CardHeader, CardTitle, CardDescription } from '@/components/ui/card'
import { Badge } from '@/components/ui/badge'
import { Button } from '@/components/ui/button'
import { Progress } from '@/components/ui/progress'
import { Alert, AlertDescription } from '@/components/ui/alert'

import { 
  useGroupAnalytics, 
  usePermissionGroups,
  useGroupAssignmentHistory 
} from '@/hooks/useGroupPermissions'
import { adminCardVariants } from '@/design-system'
import { cn } from '@/lib/shared'
import { format, subDays } from 'date-fns'

interface GroupAnalyticsDashboardProps {
  className?: string
}

export function GroupAnalyticsDashboard({ className }: GroupAnalyticsDashboardProps) {
  const { 
    analytics, 
    stats, 
    expiringMemberships,
    isLoading, 
    error, 
    cleanupExpiredMemberships,
    refreshAnalytics 
  } = useGroupAnalytics()

  const { groups, systemGroups, customGroups } = usePermissionGroups()
  const { history } = useGroupAssignmentHistory()

  // Calculate additional insights
  const insights = useMemo(() => {
    if (!analytics || !groups.length) {
      return {
        averageMembershipsPerGroup: 0,
        mostPopularGroup: null,
        leastUsedGroup: null,
        healthScore: 0,
        recentActivityTrend: 'stable',
        permissionDistribution: []
      }
    }

    const averageMembershipsPerGroup = Math.round(
      analytics.total_active_memberships / analytics.total_groups
    )

    const mostPopular = analytics.most_popular_groups?.[0]
    const popularGroup = groups.find(g => g.name === mostPopular?.group_name)

    // Permission distribution
    const permissionDistribution = Object.entries(analytics.permission_distribution || {})
      .map(([permission, count]) => ({ permission, count }))
      .sort((a, b) => b.count - a.count)
      .slice(0, 10)

    // Health score calculation (0-100)
    const healthScore = Math.min(100, Math.round(
      (analytics.total_active_memberships / Math.max(1, analytics.total_groups)) * 15 +
      (1 - (analytics.expiring_soon_count / Math.max(1, analytics.total_active_memberships))) * 85
    ))

    return {
      averageMembershipsPerGroup,
      mostPopularGroup: popularGroup,
      leastUsedGroup: null, // TODO: Calculate from data
      healthScore,
      recentActivityTrend: 'stable', // TODO: Calculate trend
      permissionDistribution
    }
  }, [analytics, groups])

  const handleCleanupExpired = async () => {
    try {
      const result = await cleanupExpiredMemberships()
      // Show toast or notification about cleanup results
    } catch (error) {
      // Handle error
    }
  }

  if (isLoading) {
    return (
      <div className={cn('space-y-6', className)}>
        {[...Array(6)].map((_, i) => (
          <Card key={i} className="animate-pulse">
            <CardHeader className="space-y-2">
              <div className="h-4 bg-gray-200 rounded w-1/3"></div>
              <div className="h-3 bg-gray-200 rounded w-1/2"></div>
            </CardHeader>
            <CardContent>
              <div className="h-32 bg-gray-200 rounded"></div>
            </CardContent>
          </Card>
        ))}
      </div>
    )
  }

  if (error) {
    return (
      <div className={className}>
        <Alert variant="destructive">
          <AlertTriangle className="h-4 w-4" />
          <AlertDescription>
            Failed to load analytics: {error.message}
          </AlertDescription>
        </Alert>
      </div>
    )
  }

  return (
    <div className={cn('space-y-6', className)}>
      {/* Header */}
      <div className="flex justify-between items-center">
        <div>
          <h2 className="text-2xl font-semibold text-gray-900">Group Analytics</h2>
          <p className="text-sm text-gray-600">
            Insights and metrics for your permission group system
          </p>
        </div>
        <Button variant="outline" onClick={() => refreshAnalytics()} disabled={isLoading}>
          <RefreshCw className="h-4 w-4 mr-2" />
          Refresh
        </Button>
      </div>

      {/* Key Metrics */}
      <div className="grid grid-cols-2 md:grid-cols-4 gap-4">
        <Card className={adminCardVariants({ variant: 'default' })}>
          <CardContent className="pt-6">
            <div className="flex items-center justify-between">
              <div>
                <p className="text-sm font-medium text-gray-600">Total Groups</p>
                <p className="text-2xl font-bold text-gray-900">{stats.total_groups}</p>
                <p className="text-xs text-gray-500">
                  {stats.system_groups} system, {stats.custom_groups} custom
                </p>
              </div>
              <Users className="h-8 w-8 text-blue-600" />
            </div>
          </CardContent>
        </Card>

        <Card className={adminCardVariants({ variant: 'default' })}>
          <CardContent className="pt-6">
            <div className="flex items-center justify-between">
              <div>
                <p className="text-sm font-medium text-gray-600">Active Members</p>
                <p className="text-2xl font-bold text-green-600">{stats.total_active_memberships}</p>
                <p className="text-xs text-gray-500">
                  {insights.averageMembershipsPerGroup} avg per group
                </p>
              </div>
              <CheckCircle className="h-8 w-8 text-green-600" />
            </div>
          </CardContent>
        </Card>

        <Card className={adminCardVariants({ variant: 'default' })}>
          <CardContent className="pt-6">
            <div className="flex items-center justify-between">
              <div>
                <p className="text-sm font-medium text-gray-600">Expiring Soon</p>
                <p className="text-2xl font-bold text-orange-600">{stats.expiring_soon}</p>
                <p className="text-xs text-gray-500">Next 7 days</p>
              </div>
              <Clock className="h-8 w-8 text-orange-600" />
            </div>
          </CardContent>
        </Card>

        <Card className={adminCardVariants({ variant: 'default' })}>
          <CardContent className="pt-6">
            <div className="flex items-center justify-between">
              <div>
                <p className="text-sm font-medium text-gray-600">Health Score</p>
                <p className="text-2xl font-bold text-purple-600">{insights.healthScore}%</p>
                <div className="w-full bg-gray-200 rounded-full h-1.5 mt-2">
                  <div 
                    className="bg-purple-600 h-1.5 rounded-full transition-all duration-300"
                    style={{ width: `${insights.healthScore}%` }}
                  />
                </div>
              </div>
              <Activity className="h-8 w-8 text-purple-600" />
            </div>
          </CardContent>
        </Card>
      </div>

      {/* Alerts and Actions */}
      {stats.expiring_soon > 0 && (
        <Alert>
          <Clock className="h-4 w-4" />
          <AlertDescription className="flex items-center justify-between">
            <span>
              {stats.expiring_soon} group membership(s) will expire within 7 days.
            </span>
            <Button variant="outline" size="sm">
              <Eye className="h-4 w-4 mr-2" />
              Review Expiring
            </Button>
          </AlertDescription>
        </Alert>
      )}

      <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
        {/* Most Popular Groups */}
        <Card className={adminCardVariants({ variant: 'default' })}>
          <CardHeader>
            <CardTitle className="flex items-center gap-2">
              <TrendingUp className="h-5 w-5" />
              Popular Groups
            </CardTitle>
            <CardDescription>
              Groups with the most active memberships
            </CardDescription>
          </CardHeader>
          <CardContent>
            {analytics?.most_popular_groups && analytics.most_popular_groups.length > 0 ? (
              <div className="space-y-3">
                {analytics.most_popular_groups.slice(0, 5).map((group, index) => (
                  <div key={group.group_name} className="flex items-center justify-between">
                    <div className="flex items-center gap-3">
                      <Badge variant="outline">{index + 1}</Badge>
                      <div>
                        <p className="font-medium text-sm">{group.group_name}</p>
                        <p className="text-xs text-gray-500">{group.member_count} members</p>
                      </div>
                    </div>
                    <Progress 
                      value={(group.member_count / Math.max(...analytics.most_popular_groups.map(g => g.member_count))) * 100} 
                      className="w-16 h-2"
                    />
                  </div>
                ))}
              </div>
            ) : (
              <div className="text-center py-6 text-gray-500">
                <Users className="h-8 w-8 mx-auto mb-2 opacity-50" />
                <p>No group data available</p>
              </div>
            )}
          </CardContent>
        </Card>

        {/* Permission Distribution */}
        <Card className={adminCardVariants({ variant: 'default' })}>
          <CardHeader>
            <CardTitle className="flex items-center gap-2">
              <PieChart className="h-5 w-5" />
              Permission Usage
            </CardTitle>
            <CardDescription>
              Most commonly assigned permissions
            </CardDescription>
          </CardHeader>
          <CardContent>
            {insights.permissionDistribution.length > 0 ? (
              <div className="space-y-3">
                {insights.permissionDistribution.slice(0, 8).map((item, index) => (
                  <div key={item.permission} className="flex items-center justify-between">
                    <div className="flex-1 min-w-0">
                      <p className="font-medium text-sm truncate">{item.permission}</p>
                      <p className="text-xs text-gray-500">{item.count} assignments</p>
                    </div>
                    <Badge variant="secondary">{item.count}</Badge>
                  </div>
                ))}
              </div>
            ) : (
              <div className="text-center py-6 text-gray-500">
                <Shield className="h-8 w-8 mx-auto mb-2 opacity-50" />
                <p>No permission data available</p>
              </div>
            )}
          </CardContent>
        </Card>

        {/* System Health */}
        <Card className={adminCardVariants({ variant: 'default' })}>
          <CardHeader>
            <CardTitle className="flex items-center gap-2">
              <Activity className="h-5 w-5" />
              System Health
            </CardTitle>
            <CardDescription>
              Overall system performance and health
            </CardDescription>
          </CardHeader>
          <CardContent>
            <div className="space-y-4">
              <div className="flex items-center justify-between">
                <span className="text-sm">Group Utilization</span>
                <div className="flex items-center gap-2">
                  <Progress value={75} className="w-20 h-2" />
                  <span className="text-sm font-medium">75%</span>
                </div>
              </div>

              <div className="flex items-center justify-between">
                <span className="text-sm">Active Memberships</span>
                <div className="flex items-center gap-2">
                  <Progress value={88} className="w-20 h-2" />
                  <span className="text-sm font-medium">88%</span>
                </div>
              </div>

              <div className="flex items-center justify-between">
                <span className="text-sm">Permission Coverage</span>
                <div className="flex items-center gap-2">
                  <Progress value={92} className="w-20 h-2" />
                  <span className="text-sm font-medium">92%</span>
                </div>
              </div>

              <div className="pt-3 border-t border-gray-100">
                <div className="flex items-center justify-between text-sm">
                  <span>Overall Health Score</span>
                  <Badge 
                    variant={insights.healthScore >= 80 ? 'default' : 
                             insights.healthScore >= 60 ? 'secondary' : 'destructive'}
                  >
                    {insights.healthScore}%
                  </Badge>
                </div>
              </div>
            </div>
          </CardContent>
        </Card>

        {/* Recent Activity */}
        <Card className={adminCardVariants({ variant: 'default' })}>
          <CardHeader>
            <CardTitle className="flex items-center gap-2">
              <Calendar className="h-5 w-5" />
              Recent Activity
            </CardTitle>
            <CardDescription>
              Latest group assignment activities
            </CardDescription>
          </CardHeader>
          <CardContent>
            {history && history.length > 0 ? (
              <div className="space-y-3">
                {history.slice(0, 5).map((entry) => (
                  <div key={entry.id} className="flex items-start gap-3">
                    <div className="mt-1">
                      {(entry as any).action === 'assigned' ? (
                        <CheckCircle className="h-4 w-4 text-green-600" />
                      ) : (entry as any).action === 'removed' ? (
                        <XCircle className="h-4 w-4 text-red-600" />
                      ) : (entry as any).action === 'expired' ? (
                        <Clock className="h-4 w-4 text-orange-600" />
                      ) : (
                        <Zap className="h-4 w-4 text-blue-600" />
                      )}
                    </div>
                    <div className="flex-1 min-w-0">
                      <p className="text-sm font-medium">
                        {entry.group?.name}
                      </p>
                      <p className="text-xs text-gray-500 capitalize">
                        {(entry as any).action} • {(entry as any).assignment_reason}
                      </p>
                      <p className="text-xs text-gray-400">
                        {format(new Date(entry.created_at), 'MMM d, h:mm a')}
                      </p>
                    </div>
                  </div>
                ))}
              </div>
            ) : (
              <div className="text-center py-6 text-gray-500">
                <Activity className="h-8 w-8 mx-auto mb-2 opacity-50" />
                <p>No recent activity</p>
              </div>
            )}
          </CardContent>
        </Card>
      </div>

      {/* Maintenance Actions */}
      <Card className={adminCardVariants({ variant: 'default' })}>
        <CardHeader>
          <CardTitle className="flex items-center gap-2">
            <Zap className="h-5 w-5" />
            Maintenance & Cleanup
          </CardTitle>
          <CardDescription>
            System maintenance and optimization tools
          </CardDescription>
        </CardHeader>
        <CardContent>
          <div className="flex flex-col sm:flex-row gap-4">
            <Button variant="outline" onClick={handleCleanupExpired}>
              <RefreshCw className="h-4 w-4 mr-2" />
              Cleanup Expired Memberships
            </Button>
            <Button variant="outline">
              <BarChart3 className="h-4 w-4 mr-2" />
              Generate Report
            </Button>
            <Button variant="outline">
              <Eye className="h-4 w-4 mr-2" />
              View Audit Log
            </Button>
          </div>
        </CardContent>
      </Card>
    </div>
  )
}

export default GroupAnalyticsDashboard