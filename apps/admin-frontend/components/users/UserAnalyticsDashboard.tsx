'use client'

import { useState, useMemo } from 'react'
import { 
  Users, 
  UserPlus, 
  UserCheck, 
  UserX, 
  Activity, 
  TrendingUp, 
  TrendingDown,
  DollarSign,
  Shield,
  Calendar,
  BarChart3,
  PieChart,
  Clock,
  AlertTriangle
} from 'lucide-react'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import { AnalyticsStatsCards } from '@/components/admin/AnalyticsStatsCards'
import type { UnifiedUserData } from '@/lib/types/unified-user'

interface UserAnalyticsDashboardProps {
  users: UnifiedUserData[]
  total: number
  isExpanded?: boolean
  onToggleExpanded?: () => void
}

export function UserAnalyticsDashboard({ 
  users, 
  total, 
  isExpanded = false, 
  onToggleExpanded 
}: UserAnalyticsDashboardProps) {
  const [activeView, setActiveView] = useState<'overview' | 'roles' | 'billing' | 'activity'>('overview')

  // Calculate user metrics from the current dataset
  const userMetrics = useMemo(() => {
    const activeUsers = users.filter(u => u.status === 'active').length
    const newUsers = users.filter(u => {
      const createdAt = new Date(u.createdAt)
      const monthAgo = new Date()
      monthAgo.setMonth(monthAgo.getMonth() - 1)
      return createdAt > monthAgo
    }).length
    
    const premiumUsers = users.filter(u => u.billing?.tier === 'premium' || u.billing?.tier === 'enterprise').length
    const adminUsers = users.filter(u => u.permissions?.some(p => p === 'admin:*:*' || p.startsWith('admin:'))).length
    
    const totalRevenue = users.reduce((sum, u) => {
      const tierValue = u.billing?.tier === 'premium' ? 29.99 : u.billing?.tier === 'enterprise' ? 99.99 : 0
      return sum + tierValue
    }, 0)
    
    const avgApiCalls = users.reduce((sum, u) => sum + (u.usageMetrics?.apiCallsThisMonth || 0), 0) / users.length
    const avgSessions = users.reduce((sum, u) => sum + (u.usageMetrics?.sessionsThisMonth || 0), 0) / users.length
    
    const twoFactorEnabled = users.filter(u => u.twoFactorEnabled).length
    const emailVerified = users.filter(u => u.emailVerified).length
    
    const statusDistribution = {
      active: users.filter(u => u.status === 'active').length,
      disabled: users.filter(u => u.status === 'disabled').length,
      pending: users.filter(u => u.status === 'pending').length,
      suspended: users.filter(u => u.status === 'suspended').length,
    }
    
    const permissionDistribution = {
      admin: users.filter(u => u.permissions?.some(p => p === 'admin:*:*' || p.startsWith('admin:'))).length,
      premium: users.filter(u => u.billing?.tier === 'premium' || u.billing?.tier === 'enterprise').length,
      user: users.filter(u => !u.permissions?.some(p => p.startsWith('admin:')) && (u.billing?.tier === 'free' || !u.billing?.tier)).length,
    }

    return {
      activeUsers,
      newUsers,
      premiumUsers,
      adminUsers,
      totalRevenue,
      avgApiCalls: Math.round(avgApiCalls),
      avgSessions: Math.round(avgSessions),
      twoFactorEnabled,
      emailVerified,
      statusDistribution,
      permissionDistribution,
      securityScore: Math.round((twoFactorEnabled / users.length) * 100),
      verificationRate: Math.round((emailVerified / users.length) * 100),
    }
  }, [users])

  const formatCurrency = (amount: number) => {
    return new Intl.NumberFormat('en-US', {
      style: 'currency',
      currency: 'USD',
    }).format(amount)
  }

  const getChangeIndicator = (current: number, total: number) => {
    const percentage = total > 0 ? (current / total) * 100 : 0
    return {
      value: percentage,
      trend: percentage > 50 ? 'up' : 'down',
      color: percentage > 75 ? 'text-green-600' : percentage > 50 ? 'text-yellow-600' : 'text-red-600'
    }
  }

  if (!isExpanded) {
    // Compact view - just show key stats
    return (
      <Card className="mb-6">
        <CardHeader className="pb-3">
          <div className="flex items-center justify-between">
            <div className="flex items-center gap-2">
              <BarChart3 className="h-5 w-5 text-blue-600" />
              <CardTitle className="text-lg">User Analytics</CardTitle>
            </div>
            <button
              onClick={onToggleExpanded}
              className="text-sm text-blue-600 hover:text-blue-800"
            >
              View Details
            </button>
          </div>
        </CardHeader>
        <CardContent>
          <div className="grid grid-cols-2 md:grid-cols-4 gap-4">
            <div className="text-center">
              <div className="text-2xl font-bold text-green-600">{userMetrics.activeUsers}</div>
              <div className="text-xs text-gray-600">Active Users</div>
            </div>
            <div className="text-center">
              <div className="text-2xl font-bold text-blue-600">{userMetrics.newUsers}</div>
              <div className="text-xs text-gray-600">New This Month</div>
            </div>
            <div className="text-center">
              <div className="text-2xl font-bold text-purple-600">{userMetrics.premiumUsers}</div>
              <div className="text-xs text-gray-600">Premium Users</div>
            </div>
            <div className="text-center">
              <div className="text-2xl font-bold text-orange-600">{formatCurrency(userMetrics.totalRevenue)}</div>
              <div className="text-xs text-gray-600">MRR</div>
            </div>
          </div>
        </CardContent>
      </Card>
    )
  }

  // Expanded view with full analytics
  return (
    <div className="space-y-6 mb-6">
      {/* Header */}
      <Card>
        <CardHeader>
          <div className="flex items-center justify-between">
            <div className="flex items-center gap-2">
              <BarChart3 className="h-6 w-6 text-blue-600" />
              <CardTitle className="text-xl">User Analytics Dashboard</CardTitle>
            </div>
            <button
              onClick={onToggleExpanded}
              className="text-sm text-gray-600 hover:text-gray-800"
            >
              Collapse
            </button>
          </div>
        </CardHeader>
      </Card>

      {/* Analytics Stats Cards */}
      <AnalyticsStatsCards
        totalRequests={userMetrics.avgApiCalls * users.length}
        totalUsers={total}
        totalRevenue={userMetrics.totalRevenue}
        averageResponseTime={120}
        errorRate={0.5}
        activeApiKeys={Math.floor(users.length * 0.3)}
      />

      {/* View Navigation */}
      <div className="flex items-center gap-2 border-b">
        {[
          { id: 'overview', label: 'Overview', icon: BarChart3 },
          { id: 'roles', label: 'Roles & Permissions', icon: Shield },
          { id: 'billing', label: 'Billing & Revenue', icon: DollarSign },
          { id: 'activity', label: 'User Activity', icon: Activity },
        ].map((view) => (
          <button
            key={view.id}
            onClick={() => setActiveView(view.id as any)}
            className={`flex items-center gap-2 px-4 py-2 text-sm font-medium border-b-2 transition-colors ${
              activeView === view.id
                ? 'border-blue-600 text-blue-600'
                : 'border-transparent text-gray-600 hover:text-gray-900'
            }`}
          >
            <view.icon className="h-4 w-4" />
            {view.label}
          </button>
        ))}
      </div>

      {/* Content based on active view */}
      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-4">
        {activeView === 'overview' && (
          <>
            <Card>
              <CardContent className="p-4">
                <div className="flex items-center justify-between">
                  <div>
                    <div className="text-2xl font-bold text-green-600">{userMetrics.activeUsers}</div>
                    <div className="text-sm text-gray-600">Active Users</div>
                  </div>
                  <UserCheck className="h-8 w-8 text-green-500" />
                </div>
                <div className="mt-2 flex items-center gap-1">
                  {userMetrics.activeUsers > total * 0.8 ? (
                    <TrendingUp className="h-4 w-4 text-green-500" />
                  ) : (
                    <TrendingDown className="h-4 w-4 text-red-500" />
                  )}
                  <span className="text-xs text-gray-500">
                    {Math.round((userMetrics.activeUsers / total) * 100)}% of total
                  </span>
                </div>
              </CardContent>
            </Card>

            <Card>
              <CardContent className="p-4">
                <div className="flex items-center justify-between">
                  <div>
                    <div className="text-2xl font-bold text-blue-600">{userMetrics.newUsers}</div>
                    <div className="text-sm text-gray-600">New This Month</div>
                  </div>
                  <UserPlus className="h-8 w-8 text-blue-500" />
                </div>
                <div className="mt-2 flex items-center gap-1">
                  <Calendar className="h-4 w-4 text-gray-400" />
                  <span className="text-xs text-gray-500">30-day growth</span>
                </div>
              </CardContent>
            </Card>

            <Card>
              <CardContent className="p-4">
                <div className="flex items-center justify-between">
                  <div>
                    <div className="text-2xl font-bold text-orange-600">{userMetrics.verificationRate}%</div>
                    <div className="text-sm text-gray-600">Email Verified</div>
                  </div>
                  <UserCheck className="h-8 w-8 text-orange-500" />
                </div>
                <div className="mt-2 flex items-center gap-1">
                  <span className="text-xs text-gray-500">
                    {userMetrics.emailVerified} of {total} users
                  </span>
                </div>
              </CardContent>
            </Card>

            <Card>
              <CardContent className="p-4">
                <div className="flex items-center justify-between">
                  <div>
                    <div className="text-2xl font-bold text-purple-600">{userMetrics.securityScore}%</div>
                    <div className="text-sm text-gray-600">2FA Adoption</div>
                  </div>
                  <Shield className="h-8 w-8 text-purple-500" />
                </div>
                <div className="mt-2 flex items-center gap-1">
                  {userMetrics.securityScore > 50 ? (
                    <TrendingUp className="h-4 w-4 text-green-500" />
                  ) : (
                    <AlertTriangle className="h-4 w-4 text-yellow-500" />
                  )}
                  <span className="text-xs text-gray-500">Security metric</span>
                </div>
              </CardContent>
            </Card>
          </>
        )}

        {activeView === 'roles' && (
          <>
            <Card>
              <CardContent className="p-4">
                <div className="text-2xl font-bold text-red-600">{userMetrics.permissionDistribution.admin}</div>
                <div className="text-sm text-gray-600">Admin Users</div>
                <div className="text-xs text-gray-500 mt-1">
                  {Math.round((userMetrics.permissionDistribution.admin / total) * 100)}% of users
                </div>
              </CardContent>
            </Card>
            <Card>
              <CardContent className="p-4">
                <div className="text-2xl font-bold text-purple-600">{userMetrics.permissionDistribution.premium}</div>
                <div className="text-sm text-gray-600">Premium Users</div>
                <div className="text-xs text-gray-500 mt-1">
                  {Math.round((userMetrics.permissionDistribution.premium / total) * 100)}% of users
                </div>
              </CardContent>
            </Card>
            <Card>
              <CardContent className="p-4">
                <div className="text-2xl font-bold text-gray-600">{userMetrics.permissionDistribution.user}</div>
                <div className="text-sm text-gray-600">Basic Users</div>
                <div className="text-xs text-gray-500 mt-1">
                  {Math.round((userMetrics.permissionDistribution.user / total) * 100)}% of users
                </div>
              </CardContent>
            </Card>
            <Card>
              <CardContent className="p-4">
                <div className="text-2xl font-bold text-gray-600">{userMetrics.permissionDistribution.user}</div>
                <div className="text-sm text-gray-600">Basic Users</div>
                <div className="text-xs text-gray-500 mt-1">
                  {Math.round((userMetrics.permissionDistribution.user / total) * 100)}% of users
                </div>
              </CardContent>
            </Card>
          </>
        )}

        {activeView === 'billing' && (
          <>
            <Card>
              <CardContent className="p-4">
                <div className="text-2xl font-bold text-green-600">{formatCurrency(userMetrics.totalRevenue)}</div>
                <div className="text-sm text-gray-600">Monthly Revenue</div>
                <div className="text-xs text-gray-500 mt-1">From {userMetrics.premiumUsers} premium users</div>
              </CardContent>
            </Card>
            <Card>
              <CardContent className="p-4">
                <div className="text-2xl font-bold text-blue-600">{userMetrics.premiumUsers}</div>
                <div className="text-sm text-gray-600">Premium Subscribers</div>
                <div className="text-xs text-gray-500 mt-1">
                  {Math.round((userMetrics.premiumUsers / total) * 100)}% conversion rate
                </div>
              </CardContent>
            </Card>
            <Card>
              <CardContent className="p-4">
                <div className="text-2xl font-bold text-purple-600">
                  {formatCurrency(userMetrics.totalRevenue / Math.max(userMetrics.premiumUsers, 1))}
                </div>
                <div className="text-sm text-gray-600">ARPU</div>
                <div className="text-xs text-gray-500 mt-1">Average revenue per user</div>
              </CardContent>
            </Card>
            <Card>
              <CardContent className="p-4">
                <div className="text-2xl font-bold text-orange-600">
                  {total - userMetrics.premiumUsers}
                </div>
                <div className="text-sm text-gray-600">Free Users</div>
                <div className="text-xs text-gray-500 mt-1">Conversion opportunity</div>
              </CardContent>
            </Card>
          </>
        )}

        {activeView === 'activity' && (
          <>
            <Card>
              <CardContent className="p-4">
                <div className="text-2xl font-bold text-blue-600">{userMetrics.avgApiCalls}</div>
                <div className="text-sm text-gray-600">Avg API Calls</div>
                <div className="text-xs text-gray-500 mt-1">Per user this month</div>
              </CardContent>
            </Card>
            <Card>
              <CardContent className="p-4">
                <div className="text-2xl font-bold text-green-600">{userMetrics.avgSessions}</div>
                <div className="text-sm text-gray-600">Avg Sessions</div>
                <div className="text-xs text-gray-500 mt-1">Per user this month</div>
              </CardContent>
            </Card>
            <Card>
              <CardContent className="p-4">
                <div className="text-2xl font-bold text-purple-600">
                  {Math.round(userMetrics.avgApiCalls / Math.max(userMetrics.avgSessions, 1))}
                </div>
                <div className="text-sm text-gray-600">API/Session</div>
                <div className="text-xs text-gray-500 mt-1">Usage intensity</div>
              </CardContent>
            </Card>
            <Card>
              <CardContent className="p-4">
                <div className="text-2xl font-bold text-orange-600">
                  {users.filter(u => (u.usageMetrics?.apiCallsThisMonth || 0) > 0).length}
                </div>
                <div className="text-sm text-gray-600">Active API Users</div>
                <div className="text-xs text-gray-500 mt-1">
                  {Math.round((users.filter(u => (u.usageMetrics?.apiCallsThisMonth || 0) > 0).length / total) * 100)}% adoption
                </div>
              </CardContent>
            </Card>
          </>
        )}
      </div>
    </div>
  )
}