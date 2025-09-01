'use client'

import { useState, useEffect, useMemo } from 'react'
import { 
  AlertTriangle,
  CheckCircle,
  XCircle,
  Clock,
  Shield,
  Users,
  TrendingUp,
  TrendingDown,
  RefreshCw,
  AlertCircle,
  Zap,
  Eye,
  BarChart3,
  Activity
} from 'lucide-react'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import { Badge } from '@/components/ui/badge'
import { Button } from '@/components/ui/button'
import { Alert, AlertDescription } from '@/components/ui/alert'
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs'
import { Progress } from '@/components/ui/progress'
import { parseEmbeddedPermissions } from '../auth/AdminPermissionExpiryIndicator'

// ============================================================================
// TYPES
// ============================================================================

interface UserPermissionData {
  userId: string
  email: string
  name?: string
  permissions: string[]
  healthData?: ReturnType<typeof parseEmbeddedPermissions>
}

interface SystemHealthMetrics {
  totalUsers: number
  usersWithExpired: number
  usersWithExpiring: number
  totalPermissions: number
  expiredPermissions: number
  expiringSoonPermissions: number
  criticalPermissions: number
  systemHealthScore: 'excellent' | 'good' | 'warning' | 'critical'
  healthTrend: 'improving' | 'stable' | 'declining'
}

interface PermissionHealthDashboardProps {
  users?: UserPermissionData[]
  refreshInterval?: number
  showDetails?: boolean
  onUserAction?: (action: 'view' | 'extend' | 'revoke', userId: string) => void
  onSystemAction?: (action: 'cleanup' | 'bulk-extend' | 'export') => void
  className?: string
}

// ============================================================================
// MAIN DASHBOARD COMPONENT
// ============================================================================

export function PermissionHealthDashboard({
  users = [],
  refreshInterval = 30000,
  showDetails = true,
  onUserAction,
  onSystemAction,
  className = ''
}: PermissionHealthDashboardProps) {
  const [refreshing, setRefreshing] = useState(false)
  const [lastRefresh, setLastRefresh] = useState<Date>(new Date())
  
  // Process user data and calculate metrics
  const { processedUsers, metrics } = useMemo(() => {
    const processed = users.map(user => ({
      ...user,
      healthData: parseEmbeddedPermissions(user.permissions)
    }))
    
    const systemMetrics = calculateSystemMetrics(processed)
    
    return {
      processedUsers: processed,
      metrics: systemMetrics
    }
  }, [users])
  
  // Auto-refresh functionality
  useEffect(() => {
    if (refreshInterval <= 0) return
    
    const interval = setInterval(() => {
      setLastRefresh(new Date())
      // Trigger refresh of data (would typically call a parent refresh function)
    }, refreshInterval)
    
    return () => clearInterval(interval)
  }, [refreshInterval])
  
  const handleRefresh = async () => {
    setRefreshing(true)
    // Simulate refresh delay
    await new Promise(resolve => setTimeout(resolve, 1000))
    setLastRefresh(new Date())
    setRefreshing(false)
  }
  
  // Critical users that need immediate attention
  const criticalUsers = processedUsers.filter(user => 
    user.healthData && (user.healthData.expiredCount > 0 || user.healthData.criticalCount > 0)
  )
  
  const expiringUsers = processedUsers.filter(user => 
    user.healthData && user.healthData.expiringSoonCount > 0 && user.healthData.expiredCount === 0
  )
  
  return (
    <div className={`space-y-6 ${className}`}>
      {/* Header */}
      <div className="flex items-center justify-between">
        <div>
          <h2 className="text-2xl font-bold text-gray-900 dark:text-gray-100">
            Permission Health Dashboard
          </h2>
          <p className="text-sm text-gray-500 dark:text-gray-400">
            Monitor embedded timestamp permissions across all users
          </p>
        </div>
        <div className="flex items-center gap-2">
          <span className="text-xs text-muted-foreground">
            Last updated: {lastRefresh.toLocaleTimeString()}
          </span>
          <Button
            variant="outline"
            size="sm"
            onClick={handleRefresh}
            disabled={refreshing}
          >
            <RefreshCw className={`h-4 w-4 ${refreshing ? 'animate-spin' : ''}`} />
          </Button>
        </div>
      </div>
      
      {/* Critical Alerts */}
      {metrics.systemHealthScore === 'critical' && (
        <Alert className="bg-red-50 border-red-200 text-red-800 dark:bg-red-900/20 dark:border-red-800 dark:text-red-200">
          <AlertCircle className="h-5 w-5" />
          <AlertDescription>
            <div className="flex items-center justify-between">
              <div>
                <strong>Critical Permission Issues Detected</strong>
                <p className="mt-1">
                  {metrics.usersWithExpired} users have expired permissions, {metrics.usersWithExpiring} have expiring soon.
                  Immediate action required.
                </p>
              </div>
              <div className="flex gap-2">
                {onSystemAction && (
                  <>
                    <Button
                      size="sm"
                      onClick={() => onSystemAction('cleanup')}
                      className="bg-red-600 hover:bg-red-700"
                    >
                      Cleanup Expired
                    </Button>
                    <Button
                      variant="outline"
                      size="sm"
                      onClick={() => onSystemAction('bulk-extend')}
                    >
                      Bulk Extend
                    </Button>
                  </>
                )}
              </div>
            </div>
          </AlertDescription>
        </Alert>
      )}
      
      {/* System Overview Cards */}
      <SystemOverviewCards metrics={metrics} onSystemAction={onSystemAction} />
      
      {/* Detailed Views */}
      <Tabs defaultValue="overview" className="w-full">
        <TabsList className="grid w-full grid-cols-4">
          <TabsTrigger value="overview">Overview</TabsTrigger>
          <TabsTrigger value="critical">
            Critical ({criticalUsers.length})
          </TabsTrigger>
          <TabsTrigger value="expiring">
            Expiring ({expiringUsers.length})
          </TabsTrigger>
          <TabsTrigger value="analytics">Analytics</TabsTrigger>
        </TabsList>
        
        <TabsContent value="overview" className="space-y-4">
          <SystemHealthChart metrics={metrics} />
          <RecentActivityFeed />
        </TabsContent>
        
        <TabsContent value="critical" className="space-y-4">
          <CriticalUsersList 
            users={criticalUsers} 
            onUserAction={onUserAction}
          />
        </TabsContent>
        
        <TabsContent value="expiring" className="space-y-4">
          <ExpiringUsersList 
            users={expiringUsers} 
            onUserAction={onUserAction}
          />
        </TabsContent>
        
        <TabsContent value="analytics" className="space-y-4">
          <PermissionAnalytics 
            users={processedUsers}
            metrics={metrics}
          />
        </TabsContent>
      </Tabs>
    </div>
  )
}

// ============================================================================
// SYSTEM OVERVIEW CARDS
// ============================================================================

function SystemOverviewCards({ 
  metrics, 
  onSystemAction 
}: { 
  metrics: SystemHealthMetrics
  onSystemAction?: (action: 'cleanup' | 'bulk-extend' | 'export') => void 
}) {
  const healthPercentage = Math.round(
    ((metrics.totalPermissions - metrics.expiredPermissions) / metrics.totalPermissions) * 100
  )
  
  return (
    <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6">
      {/* System Health */}
      <Card className={`${getHealthCardStyle(metrics.systemHealthScore)}`}>
        <CardContent className="p-6">
          <div className="flex items-center justify-between">
            <div className="flex items-center gap-3">
              <div className={`p-2 rounded-lg ${getHealthIconBg(metrics.systemHealthScore)}`}>
                {getHealthIcon(metrics.systemHealthScore, 'h-6 w-6')}
              </div>
              <div>
                <h3 className="text-lg font-semibold capitalize">{metrics.systemHealthScore}</h3>
                <p className="text-sm opacity-75">System Health</p>
              </div>
            </div>
            <div className="text-right">
              <div className="text-2xl font-bold">{healthPercentage}%</div>
              <div className="flex items-center gap-1 text-xs">
                {getTrendIcon(metrics.healthTrend, 'h-3 w-3')}
                <span className="capitalize">{metrics.healthTrend}</span>
              </div>
            </div>
          </div>
          <div className="mt-4">
            <Progress 
              value={healthPercentage} 
              className={`h-2 ${getProgressColor(metrics.systemHealthScore)}`}
            />
          </div>
        </CardContent>
      </Card>
      
      {/* Users with Issues */}
      <Card className={`${metrics.usersWithExpired > 0 ? 'bg-red-50 border-red-200 dark:bg-red-950/20 dark:border-red-800' : 'bg-gray-50 border-gray-200'}`}>
        <CardContent className="p-6">
          <div className="flex items-center justify-between">
            <div className="flex items-center gap-3">
              <div className={`p-2 rounded-lg ${metrics.usersWithExpired > 0 ? 'bg-red-100 dark:bg-red-900/30' : 'bg-gray-100'}`}>
                <XCircle className={`h-6 w-6 ${metrics.usersWithExpired > 0 ? 'text-red-600 dark:text-red-400' : 'text-gray-500'}`} />
              </div>
              <div>
                <h3 className="text-2xl font-bold">{metrics.usersWithExpired}</h3>
                <p className="text-sm">Users w/ Expired</p>
              </div>
            </div>
            {metrics.usersWithExpired > 0 && (
              <div className="h-3 w-3 bg-red-500 rounded-full animate-pulse"></div>
            )}
          </div>
          {onSystemAction && metrics.usersWithExpired > 0 && (
            <div className="mt-4">
              <Button
                size="sm"
                onClick={() => onSystemAction('cleanup')}
                className="w-full bg-red-600 hover:bg-red-700"
              >
                Cleanup Expired
              </Button>
            </div>
          )}
        </CardContent>
      </Card>
      
      {/* Users Expiring Soon */}
      <Card className={`${metrics.usersWithExpiring > 0 ? 'bg-yellow-50 border-yellow-200 dark:bg-yellow-950/20 dark:border-yellow-800' : 'bg-gray-50 border-gray-200'}`}>
        <CardContent className="p-6">
          <div className="flex items-center justify-between">
            <div className="flex items-center gap-3">
              <div className={`p-2 rounded-lg ${metrics.usersWithExpiring > 0 ? 'bg-yellow-100 dark:bg-yellow-900/30' : 'bg-gray-100'}`}>
                <Clock className={`h-6 w-6 ${metrics.usersWithExpiring > 0 ? 'text-yellow-600 dark:text-yellow-400' : 'text-gray-500'}`} />
              </div>
              <div>
                <h3 className="text-2xl font-bold">{metrics.usersWithExpiring}</h3>
                <p className="text-sm">Users w/ Expiring</p>
              </div>
            </div>
            {metrics.usersWithExpiring > 0 && (
              <div className="h-3 w-3 bg-yellow-500 rounded-full"></div>
            )}
          </div>
          {onSystemAction && metrics.usersWithExpiring > 0 && (
            <div className="mt-4">
              <Button
                variant="outline"
                size="sm"
                onClick={() => onSystemAction('bulk-extend')}
                className="w-full"
              >
                <Zap className="h-3 w-3 mr-1" />
                Bulk Extend
              </Button>
            </div>
          )}
        </CardContent>
      </Card>
      
      {/* Total Users */}
      <Card className="bg-blue-50 border-blue-200 dark:bg-blue-950/20 dark:border-blue-800">
        <CardContent className="p-6">
          <div className="flex items-center justify-between">
            <div className="flex items-center gap-3">
              <div className="p-2 bg-blue-100 rounded-lg dark:bg-blue-900/30">
                <Users className="h-6 w-6 text-blue-600 dark:text-blue-400" />
              </div>
              <div>
                <h3 className="text-2xl font-bold">{metrics.totalUsers}</h3>
                <p className="text-sm">Total Users</p>
              </div>
            </div>
            <div className="text-right text-xs">
              <div className="font-medium">{metrics.totalPermissions}</div>
              <div className="text-muted-foreground">Permissions</div>
            </div>
          </div>
        </CardContent>
      </Card>
    </div>
  )
}

// ============================================================================
// SYSTEM HEALTH CHART
// ============================================================================

function SystemHealthChart({ metrics }: { metrics: SystemHealthMetrics }) {
  const healthData = [
    { label: 'Active', value: metrics.totalPermissions - metrics.expiredPermissions, color: 'text-green-600', bg: 'bg-green-100' },
    { label: 'Expiring', value: metrics.expiringSoonPermissions, color: 'text-yellow-600', bg: 'bg-yellow-100' },
    { label: 'Critical', value: metrics.criticalPermissions, color: 'text-orange-600', bg: 'bg-orange-100' },
    { label: 'Expired', value: metrics.expiredPermissions, color: 'text-red-600', bg: 'bg-red-100' }
  ]
  
  return (
    <Card>
      <CardHeader>
        <CardTitle className="flex items-center gap-2">
          <BarChart3 className="h-5 w-5" />
          Permission Distribution
        </CardTitle>
      </CardHeader>
      <CardContent>
        <div className="space-y-4">
          {healthData.map((item) => {
            const percentage = metrics.totalPermissions > 0 ? (item.value / metrics.totalPermissions) * 100 : 0
            return (
              <div key={item.label} className="flex items-center justify-between">
                <div className="flex items-center gap-3 flex-1">
                  <div className={`w-4 h-4 rounded ${item.bg}`}></div>
                  <span className="text-sm font-medium">{item.label}</span>
                  <Progress value={percentage} className="flex-1 h-2" />
                </div>
                <div className="flex items-center gap-2 text-sm">
                  <span className={`font-bold ${item.color}`}>{item.value}</span>
                  <span className="text-muted-foreground">({Math.round(percentage)}%)</span>
                </div>
              </div>
            )
          })}
        </div>
      </CardContent>
    </Card>
  )
}

// ============================================================================
// CRITICAL USERS LIST
// ============================================================================

function CriticalUsersList({ 
  users, 
  onUserAction 
}: { 
  users: UserPermissionData[]
  onUserAction?: (action: 'view' | 'extend' | 'revoke', userId: string) => void
}) {
  return (
    <Card>
      <CardHeader>
        <CardTitle className="flex items-center gap-2 text-red-600">
          <AlertCircle className="h-5 w-5" />
          Critical Users ({users.length})
        </CardTitle>
      </CardHeader>
      <CardContent>
        <div className="space-y-3">
          {users.map((user) => (
            <div key={user.userId} className="flex items-center justify-between p-3 bg-red-50 border border-red-200 rounded-lg dark:bg-red-950/20 dark:border-red-800">
              <div className="flex-1">
                <div className="font-medium">{user.name || user.email}</div>
                <div className="text-sm text-muted-foreground">{user.email}</div>
                <div className="flex gap-2 mt-1">
                  {user.healthData && user.healthData.expiredCount > 0 && (
                    <Badge variant="destructive" className="text-xs">
                      {user.healthData.expiredCount} expired
                    </Badge>
                  )}
                  {user.healthData && user.healthData.criticalCount > 0 && (
                    <Badge className="bg-orange-100 text-orange-800 text-xs">
                      {user.healthData.criticalCount} critical
                    </Badge>
                  )}
                </div>
              </div>
              <div className="flex gap-2">
                {onUserAction && (
                  <>
                    <Button
                      variant="outline"
                      size="sm"
                      onClick={() => onUserAction('view', user.userId)}
                    >
                      <Eye className="h-3 w-3" />
                    </Button>
                    <Button
                      size="sm"
                      onClick={() => onUserAction('extend', user.userId)}
                      className="bg-blue-600 hover:bg-blue-700"
                    >
                      <Zap className="h-3 w-3" />
                    </Button>
                  </>
                )}
              </div>
            </div>
          ))}
          
          {users.length === 0 && (
            <div className="text-center py-8 text-muted-foreground">
              <CheckCircle className="h-12 w-12 mx-auto mb-2 text-green-500" />
              <p>No critical permission issues!</p>
            </div>
          )}
        </div>
      </CardContent>
    </Card>
  )
}

// ============================================================================
// EXPIRING USERS LIST
// ============================================================================

function ExpiringUsersList({ 
  users, 
  onUserAction 
}: { 
  users: UserPermissionData[]
  onUserAction?: (action: 'view' | 'extend' | 'revoke', userId: string) => void
}) {
  return (
    <Card>
      <CardHeader>
        <CardTitle className="flex items-center gap-2 text-yellow-600">
          <Clock className="h-5 w-5" />
          Users with Expiring Permissions ({users.length})
        </CardTitle>
      </CardHeader>
      <CardContent>
        <div className="space-y-3">
          {users.map((user) => (
            <div key={user.userId} className="flex items-center justify-between p-3 bg-yellow-50 border border-yellow-200 rounded-lg dark:bg-yellow-950/20 dark:border-yellow-800">
              <div className="flex-1">
                <div className="font-medium">{user.name || user.email}</div>
                <div className="text-sm text-muted-foreground">{user.email}</div>
                <div className="flex gap-2 mt-1">
                  {user.healthData && user.healthData.expiringSoonCount > 0 && (
                    <Badge className="bg-yellow-100 text-yellow-800 text-xs">
                      {user.healthData.expiringSoonCount} expiring soon
                    </Badge>
                  )}
                  {user.healthData && user.healthData.nextExpiry && (
                    <span className="text-xs text-muted-foreground">
                      Next: {formatTimeRemaining(user.healthData.nextExpiry.timeRemaining)}
                    </span>
                  )}
                </div>
              </div>
              <div className="flex gap-2">
                {onUserAction && (
                  <>
                    <Button
                      variant="outline"
                      size="sm"
                      onClick={() => onUserAction('view', user.userId)}
                    >
                      <Eye className="h-3 w-3" />
                    </Button>
                    <Button
                      size="sm"
                      onClick={() => onUserAction('extend', user.userId)}
                      className="bg-blue-600 hover:bg-blue-700"
                    >
                      <Zap className="h-3 w-3" />
                    </Button>
                  </>
                )}
              </div>
            </div>
          ))}
          
          {users.length === 0 && (
            <div className="text-center py-8 text-muted-foreground">
              <CheckCircle className="h-12 w-12 mx-auto mb-2 text-green-500" />
              <p>No permissions expiring soon!</p>
            </div>
          )}
        </div>
      </CardContent>
    </Card>
  )
}

// ============================================================================
// RECENT ACTIVITY FEED
// ============================================================================

function RecentActivityFeed() {
  // Mock recent activities - would come from actual data source
  const activities = [
    { id: 1, type: 'expired', user: 'john@example.com', permission: 'epsx:analytics:view', time: '2 minutes ago' },
    { id: 2, type: 'extended', user: 'jane@example.com', permission: 'admin:users:manage', time: '5 minutes ago' },
    { id: 3, type: 'granted', user: 'bob@example.com', permission: 'epsx:rankings:view:100', time: '10 minutes ago' },
  ]
  
  return (
    <Card>
      <CardHeader>
        <CardTitle className="flex items-center gap-2">
          <Activity className="h-5 w-5" />
          Recent Activity
        </CardTitle>
      </CardHeader>
      <CardContent>
        <div className="space-y-3">
          {activities.map((activity) => (
            <div key={activity.id} className="flex items-center gap-3 p-2 rounded">
              <div className={`w-2 h-2 rounded-full ${
                activity.type === 'expired' ? 'bg-red-500' :
                activity.type === 'extended' ? 'bg-blue-500' : 'bg-green-500'
              }`}></div>
              <div className="flex-1 text-sm">
                <span className="font-medium">{activity.user}</span>
                <span className="text-muted-foreground"> {activity.type} </span>
                <span className="font-mono text-xs bg-gray-100 px-1 rounded">{activity.permission}</span>
              </div>
              <span className="text-xs text-muted-foreground">{activity.time}</span>
            </div>
          ))}
        </div>
      </CardContent>
    </Card>
  )
}

// ============================================================================
// PERMISSION ANALYTICS
// ============================================================================

function PermissionAnalytics({ 
  users, 
  metrics 
}: { 
  users: UserPermissionData[]
  metrics: SystemHealthMetrics 
}) {
  return (
    <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
      <Card>
        <CardHeader>
          <CardTitle>Permission Types Distribution</CardTitle>
        </CardHeader>
        <CardContent>
          <div className="space-y-3">
            <div className="flex justify-between items-center">
              <span>EPSX Platform</span>
              <span className="font-bold">65%</span>
            </div>
            <div className="flex justify-between items-center">
              <span>Admin</span>
              <span className="font-bold">25%</span>
            </div>
            <div className="flex justify-between items-center">
              <span>EPSX Pay</span>
              <span className="font-bold">10%</span>
            </div>
          </div>
        </CardContent>
      </Card>
      
      <Card>
        <CardHeader>
          <CardTitle>Health Score Trends</CardTitle>
        </CardHeader>
        <CardContent>
          <div className="text-center">
            <div className="text-3xl font-bold text-blue-600 mb-2">
              {Math.round(((metrics.totalPermissions - metrics.expiredPermissions) / metrics.totalPermissions) * 100)}%
            </div>
            <p className="text-sm text-muted-foreground">Overall System Health</p>
            <div className="flex items-center justify-center gap-1 mt-2">
              {getTrendIcon(metrics.healthTrend, 'h-4 w-4')}
              <span className="text-sm capitalize">{metrics.healthTrend}</span>
            </div>
          </div>
        </CardContent>
      </Card>
    </div>
  )
}

// ============================================================================
// UTILITY FUNCTIONS
// ============================================================================

function calculateSystemMetrics(users: UserPermissionData[]): SystemHealthMetrics {
  let totalPermissions = 0
  let expiredPermissions = 0
  let expiringSoonPermissions = 0
  let criticalPermissions = 0
  let usersWithExpired = 0
  let usersWithExpiring = 0
  
  users.forEach(user => {
    if (user.healthData) {
      totalPermissions += user.healthData.totalCount
      expiredPermissions += user.healthData.expiredCount
      expiringSoonPermissions += user.healthData.expiringSoonCount
      criticalPermissions += user.healthData.criticalCount
      
      if (user.healthData.expiredCount > 0) usersWithExpired++
      if (user.healthData.expiringSoonCount > 0) usersWithExpiring++
    }
  })
  
  const healthRatio = totalPermissions > 0 ? (totalPermissions - expiredPermissions) / totalPermissions : 1
  
  let systemHealthScore: SystemHealthMetrics['systemHealthScore'] = 'excellent'
  if (expiredPermissions > 0 || criticalPermissions > 5) systemHealthScore = 'critical'
  else if (expiringSoonPermissions > 10 || healthRatio < 0.8) systemHealthScore = 'warning'
  else if (healthRatio < 0.95) systemHealthScore = 'good'
  
  return {
    totalUsers: users.length,
    usersWithExpired,
    usersWithExpiring,
    totalPermissions,
    expiredPermissions,
    expiringSoonPermissions,
    criticalPermissions,
    systemHealthScore,
    healthTrend: 'stable' // Would be calculated based on historical data
  }
}

function getHealthIcon(health: string, className: string) {
  const icons = {
    excellent: <CheckCircle className={`${className} text-green-600`} />,
    good: <Shield className={`${className} text-blue-600`} />,
    warning: <AlertTriangle className={`${className} text-yellow-600`} />,
    critical: <AlertCircle className={`${className} text-red-600`} />
  }
  return icons[health] || icons.good
}

function getTrendIcon(trend: string, className: string) {
  const icons = {
    improving: <TrendingUp className={`${className} text-green-600`} />,
    stable: <Activity className={`${className} text-blue-600`} />,
    declining: <TrendingDown className={`${className} text-red-600`} />
  }
  return icons[trend] || icons.stable
}

function getHealthCardStyle(health: string): string {
  return {
    excellent: 'bg-green-50 border-green-200 dark:bg-green-950/20 dark:border-green-800',
    good: 'bg-blue-50 border-blue-200 dark:bg-blue-950/20 dark:border-blue-800',
    warning: 'bg-yellow-50 border-yellow-200 dark:bg-yellow-950/20 dark:border-yellow-800',
    critical: 'bg-red-50 border-red-200 dark:bg-red-950/20 dark:border-red-800'
  }[health] || 'bg-gray-50 border-gray-200'
}

function getHealthIconBg(health: string): string {
  return {
    excellent: 'bg-green-100 dark:bg-green-900/30',
    good: 'bg-blue-100 dark:bg-blue-900/30',
    warning: 'bg-yellow-100 dark:bg-yellow-900/30',
    critical: 'bg-red-100 dark:bg-red-900/30'
  }[health] || 'bg-gray-100'
}

function getProgressColor(health: string): string {
  return {
    excellent: '[&_[data-state=complete]]:bg-green-500',
    good: '[&_[data-state=complete]]:bg-blue-500',
    warning: '[&_[data-state=complete]]:bg-yellow-500',
    critical: '[&_[data-state=complete]]:bg-red-500'
  }[health] || '[&_[data-state=complete]]:bg-gray-500'
}

function formatTimeRemaining(seconds: number): string {
  if (seconds <= 0) return 'Expired'
  
  const minutes = Math.floor(seconds / 60)
  const hours = Math.floor(minutes / 60)
  const days = Math.floor(hours / 24)
  
  if (days > 0) return `${days}d ${hours % 24}h`
  if (hours > 0) return `${hours}h ${minutes % 60}m`
  if (minutes > 0) return `${minutes}m`
  return `${seconds}s`
}