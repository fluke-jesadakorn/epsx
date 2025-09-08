'use client'

import { useState, useEffect, useMemo } from 'react'
import { BarChart, Bar, XAxis, YAxis, CartesianGrid, Tooltip, Legend, ResponsiveContainer, PieChart, Pie, Cell, LineChart, Line, Area, AreaChart } from 'recharts'
import { CalendarIcon, Users, Shield, TrendingUp, AlertTriangle, Eye, Download, Filter, RefreshCw } from 'lucide-react'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs'
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select'
import { Button } from '@/components/ui/button'
import { Badge } from '@/components/ui/badge'
import { Progress } from '@/components/ui/progress'
import { Alert, AlertDescription } from '@/components/ui/alert'
import { Input } from '@/components/ui/input'
import { DatePickerWithRange } from '@/components/ui/date-range-picker'
import { addDays, format, startOfMonth, endOfMonth, startOfWeek, endOfWeek } from 'date-fns'

// Types
interface AnalyticsData {
  permission_distribution: {
    platform: string
    count: number
    percentage: number
  }[]
  role_usage: {
    role_name: string
    display_name?: string
    user_count: number
    is_system_role: boolean
    last_assigned: string
  }[]
  user_permission_stats: {
    total_users: number
    users_with_roles: number
    users_without_roles: number
    avg_roles_per_user: number
    avg_permissions_per_user: number
  }
  security_insights: {
    over_privileged_users: number
    unused_permissions: number
    dormant_roles: number
    expired_permissions: number
    security_violations: SecurityViolation[]
  }
  temporal_trends: {
    date: string
    role_assignments: number
    permission_grants: number
    revocations: number
  }[]
  audit_summary: {
    total_events: number
    events_last_24h: number
    events_last_7d: number
    events_last_30d: number
    most_active_admin: string
    most_changed_user: string
  }
  platform_breakdown: {
    platform: string
    permissions: number
    roles: number
    users: number
  }[]
}

interface SecurityViolation {
  id: string
  type: 'over_privileged' | 'unused_permission' | 'expired_permission' | 'suspicious_activity'
  user_id?: string
  user_email?: string
  role_id?: string
  permission_id?: string
  description: string
  severity: 'low' | 'medium' | 'high' | 'critical'
  created_at: string
  resolved: boolean
}

interface RBACAnalyticsDashboardProps {
  className?: string
}

const COLORS = ['#3b82f6', '#10b981', '#f59e0b', '#ef4444', '#8b5cf6', '#06b6d4', '#84cc16', '#f97316']

export default function RBACAnalyticsDashboard({ className = '' }: RBACAnalyticsDashboardProps) {
  const [activeTab, setActiveTab] = useState('overview')
  const [dateRange, setDateRange] = useState<{from: Date, to: Date}>({
    from: startOfMonth(new Date()),
    to: endOfMonth(new Date())
  })
  const [analyticsData, setAnalyticsData] = useState<AnalyticsData | null>(null)
  const [loading, setLoading] = useState(true)
  const [refreshing, setRefreshing] = useState(false)
  const [selectedPlatform, setSelectedPlatform] = useState('all')

  // Mock analytics data - replace with real API calls
  const mockAnalyticsData = (): AnalyticsData => ({
    permission_distribution: [
      { platform: 'admin', count: 15, percentage: 35.7 },
      { platform: 'epsx', count: 18, percentage: 42.9 },
      { platform: 'epsx-pay', count: 6, percentage: 14.3 },
      { platform: 'epsx-token', count: 3, percentage: 7.1 }
    ],
    role_usage: [
      { role_name: 'analytics_user', display_name: 'Analytics User', user_count: 45, is_system_role: false, last_assigned: new Date(Date.now() - 1000 * 60 * 30).toISOString() },
      { role_name: 'premium_user', display_name: 'Premium User', user_count: 28, is_system_role: false, last_assigned: new Date(Date.now() - 1000 * 60 * 60 * 2).toISOString() },
      { role_name: 'analytics_admin', display_name: 'Analytics Administrator', user_count: 8, is_system_role: true, last_assigned: new Date(Date.now() - 1000 * 60 * 60 * 6).toISOString() },
      { role_name: 'super_admin', display_name: 'Super Administrator', user_count: 3, is_system_role: true, last_assigned: new Date(Date.now() - 1000 * 60 * 60 * 24 * 2).toISOString() },
      { role_name: 'guest_user', display_name: 'Guest User', user_count: 12, is_system_role: true, last_assigned: new Date(Date.now() - 1000 * 60 * 60 * 24).toISOString() }
    ],
    user_permission_stats: {
      total_users: 96,
      users_with_roles: 88,
      users_without_roles: 8,
      avg_roles_per_user: 1.8,
      avg_permissions_per_user: 12.4
    },
    security_insights: {
      over_privileged_users: 5,
      unused_permissions: 3,
      dormant_roles: 2,
      expired_permissions: 7,
      security_violations: [
        {
          id: '1',
          type: 'over_privileged',
          user_email: 'temp.contractor@example.com',
          description: 'User has admin permissions but last login was 45 days ago',
          severity: 'high',
          created_at: new Date(Date.now() - 1000 * 60 * 60 * 24).toISOString(),
          resolved: false
        },
        {
          id: '2',
          type: 'expired_permission',
          user_email: 'seasonal.worker@example.com',
          description: '3 permissions expired but still active in system',
          severity: 'medium',
          created_at: new Date(Date.now() - 1000 * 60 * 60 * 12).toISOString(),
          resolved: false
        }
      ]
    },
    temporal_trends: Array.from({ length: 30 }, (_, i) => ({
      date: format(addDays(new Date(), -29 + i), 'yyyy-MM-dd'),
      role_assignments: Math.floor(Math.random() * 10) + 1,
      permission_grants: Math.floor(Math.random() * 5) + 1,
      revocations: Math.floor(Math.random() * 3)
    })),
    audit_summary: {
      total_events: 1247,
      events_last_24h: 23,
      events_last_7d: 156,
      events_last_30d: 892,
      most_active_admin: 'admin@epsx.io',
      most_changed_user: 'john.doe@example.com'
    },
    platform_breakdown: [
      { platform: 'admin', permissions: 15, roles: 4, users: 18 },
      { platform: 'epsx', permissions: 18, roles: 6, users: 72 },
      { platform: 'epsx-pay', permissions: 6, roles: 2, users: 15 },
      { platform: 'epsx-token', permissions: 3, roles: 1, users: 8 }
    ]
  })

  useEffect(() => {
    loadAnalyticsData()
  }, [dateRange])

  const loadAnalyticsData = async () => {
    setLoading(true)
    try {
      // Simulate API call
      await new Promise(resolve => setTimeout(resolve, 1000))
      const data = mockAnalyticsData()
      setAnalyticsData(data)
    } catch (error) {
      console.error('Failed to load analytics data:', error)
    } finally {
      setLoading(false)
    }
  }

  const refreshData = async () => {
    setRefreshing(true)
    await loadAnalyticsData()
    setRefreshing(false)
  }

  const filteredPlatformData = useMemo(() => {
    if (!analyticsData || selectedPlatform === 'all') return analyticsData
    
    return {
      ...analyticsData,
      permission_distribution: analyticsData.permission_distribution.filter(p => p.platform === selectedPlatform),
      platform_breakdown: analyticsData.platform_breakdown.filter(p => p.platform === selectedPlatform)
    }
  }, [analyticsData, selectedPlatform])

  const getSeverityColor = (severity: SecurityViolation['severity']) => {
    switch (severity) {
      case 'critical': return 'bg-red-100 text-red-800 border-red-200'
      case 'high': return 'bg-orange-100 text-orange-800 border-orange-200'
      case 'medium': return 'bg-yellow-100 text-yellow-800 border-yellow-200'
      case 'low': return 'bg-blue-100 text-blue-800 border-blue-200'
      default: return 'bg-gray-100 text-gray-800 border-gray-200'
    }
  }

  const exportData = (type: string) => {
    if (!analyticsData) return
    
    let data: any[] = []
    let filename = ''
    
    switch (type) {
      case 'roles':
        data = analyticsData.role_usage
        filename = 'role-usage-analytics'
        break
      case 'permissions':
        data = analyticsData.permission_distribution
        filename = 'permission-distribution-analytics'
        break
      case 'security':
        data = analyticsData.security_insights.security_violations
        filename = 'security-violations-report'
        break
      default:
        return
    }
    
    const csvContent = [
      Object.keys(data[0] || {}).join(','),
      ...data.map(row => Object.values(row).map(val => `"${val}"`).join(','))
    ].join('\n')
    
    const blob = new Blob([csvContent], { type: 'text/csv' })
    const url = URL.createObjectURL(blob)
    const a = document.createElement('a')
    a.href = url
    a.download = `${filename}-${format(new Date(), 'yyyy-MM-dd')}.csv`
    document.body.appendChild(a)
    a.click()
    document.body.removeChild(a)
    URL.revokeObjectURL(url)
  }

  if (loading && !analyticsData) {
    return (
      <div className={`space-y-6 ${className}`}>
        <div className="flex items-center justify-center h-96">
          <div className="flex items-center space-x-2">
            <RefreshCw className="h-6 w-6 animate-spin" />
            <span>Loading analytics data...</span>
          </div>
        </div>
      </div>
    )
  }

  if (!analyticsData) {
    return (
      <div className={`space-y-6 ${className}`}>
        <Alert variant="destructive">
          <AlertTriangle className="h-4 w-4" />
          <AlertDescription>
            Failed to load analytics data. Please try refreshing the page.
          </AlertDescription>
        </Alert>
      </div>
    )
  }

  return (
    <div className={`space-y-6 ${className}`}>
      {/* Header */}
      <div className="flex justify-between items-start">
        <div>
          <h2 className="text-3xl font-bold">RBAC Analytics</h2>
          <p className="text-muted-foreground">Comprehensive insights into role and permission usage</p>
        </div>
        <div className="flex items-center space-x-2">
          <DatePickerWithRange
            from={dateRange.from}
            to={dateRange.to}
            onSelect={(range) => {
              if (range?.from && range?.to) {
                setDateRange({ from: range.from, to: range.to })
              }
            }}
          />
          <Select value={selectedPlatform} onValueChange={setSelectedPlatform}>
            <SelectTrigger className="w-48">
              <SelectValue />
            </SelectTrigger>
            <SelectContent>
              <SelectItem value="all">All Platforms</SelectItem>
              {analyticsData.platform_breakdown.map(platform => (
                <SelectItem key={platform.platform} value={platform.platform} className="capitalize">
                  {platform.platform}
                </SelectItem>
              ))}
            </SelectContent>
          </Select>
          <Button variant="outline" onClick={refreshData} disabled={refreshing}>
            <RefreshCw className={`w-4 h-4 mr-2 ${refreshing ? 'animate-spin' : ''}`} />
            Refresh
          </Button>
        </div>
      </div>

      {/* KPI Cards */}
      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-4">
        <Card>
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle className="text-sm font-medium">Total Users</CardTitle>
            <Users className="h-4 w-4 text-muted-foreground" />
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold">{analyticsData.user_permission_stats.total_users}</div>
            <p className="text-xs text-muted-foreground">
              {((analyticsData.user_permission_stats.users_with_roles / analyticsData.user_permission_stats.total_users) * 100).toFixed(1)}% with roles
            </p>
          </CardContent>
        </Card>
        
        <Card>
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle className="text-sm font-medium">Active Permissions</CardTitle>
            <Shield className="h-4 w-4 text-muted-foreground" />
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold">
              {analyticsData.permission_distribution.reduce((sum, p) => sum + p.count, 0)}
            </div>
            <p className="text-xs text-muted-foreground">
              Across {analyticsData.platform_breakdown.length} platforms
            </p>
          </CardContent>
        </Card>
        
        <Card>
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle className="text-sm font-medium">Security Issues</CardTitle>
            <AlertTriangle className="h-4 w-4 text-muted-foreground" />
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold text-red-600">
              {analyticsData.security_insights.security_violations.filter(v => !v.resolved).length}
            </div>
            <p className="text-xs text-muted-foreground">
              {analyticsData.security_insights.security_violations.filter(v => v.severity === 'critical' || v.severity === 'high').length} high priority
            </p>
          </CardContent>
        </Card>
        
        <Card>
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle className="text-sm font-medium">Audit Events</CardTitle>
            <Eye className="h-4 w-4 text-muted-foreground" />
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold">{analyticsData.audit_summary.events_last_30d}</div>
            <p className="text-xs text-muted-foreground">
              {analyticsData.audit_summary.events_last_24h} in last 24h
            </p>
          </CardContent>
        </Card>
      </div>

      {/* Main Analytics Tabs */}
      <Tabs value={activeTab} onValueChange={setActiveTab} className="space-y-4">
        <TabsList className="grid w-full grid-cols-4">
          <TabsTrigger value="overview">Overview</TabsTrigger>
          <TabsTrigger value="roles">Role Usage</TabsTrigger>
          <TabsTrigger value="security">Security</TabsTrigger>
          <TabsTrigger value="trends">Trends</TabsTrigger>
        </TabsList>

        {/* Overview Tab */}
        <TabsContent value="overview" className="space-y-6">
          <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
            {/* Permission Distribution */}
            <Card>
              <CardHeader>
                <div className="flex justify-between items-center">
                  <div>
                    <CardTitle>Permission Distribution</CardTitle>
                    <CardDescription>Permissions by platform</CardDescription>
                  </div>
                  <Button variant="outline" size="sm" onClick={() => exportData('permissions')}>
                    <Download className="w-4 h-4 mr-2" />
                    Export
                  </Button>
                </div>
              </CardHeader>
              <CardContent>
                <ResponsiveContainer width="100%" height={300}>
                  <PieChart>
                    <Pie
                      data={filteredPlatformData?.permission_distribution || []}
                      cx="50%"
                      cy="50%"
                      labelLine={false}
                      label={({ platform, percentage }) => `${platform} (${percentage}%)`}
                      outerRadius={80}
                      fill="#8884d8"
                      dataKey="count"
                    >
                      {(filteredPlatformData?.permission_distribution || []).map((entry, index) => (
                        <Cell key={`cell-${index}`} fill={COLORS[index % COLORS.length]} />
                      ))}
                    </Pie>
                    <Tooltip />
                  </PieChart>
                </ResponsiveContainer>
              </CardContent>
            </Card>

            {/* Platform Breakdown */}
            <Card>
              <CardHeader>
                <CardTitle>Platform Breakdown</CardTitle>
                <CardDescription>Resources across platforms</CardDescription>
              </CardHeader>
              <CardContent>
                <ResponsiveContainer width="100%" height={300}>
                  <BarChart data={filteredPlatformData?.platform_breakdown || []}>
                    <CartesianGrid strokeDasharray="3 3" />
                    <XAxis dataKey="platform" />
                    <YAxis />
                    <Tooltip />
                    <Legend />
                    <Bar dataKey="permissions" fill="#3b82f6" />
                    <Bar dataKey="roles" fill="#10b981" />
                    <Bar dataKey="users" fill="#f59e0b" />
                  </BarChart>
                </ResponsiveContainer>
              </CardContent>
            </Card>
          </div>

          {/* User Statistics */}
          <Card>
            <CardHeader>
              <CardTitle>User Permission Statistics</CardTitle>
              <CardDescription>Distribution of roles and permissions among users</CardDescription>
            </CardHeader>
            <CardContent>
              <div className="grid grid-cols-1 md:grid-cols-3 gap-6">
                <div className="space-y-2">
                  <div className="flex justify-between">
                    <span className="text-sm font-medium">Users with Roles</span>
                    <span className="text-sm text-muted-foreground">
                      {analyticsData.user_permission_stats.users_with_roles} of {analyticsData.user_permission_stats.total_users}
                    </span>
                  </div>
                  <Progress 
                    value={(analyticsData.user_permission_stats.users_with_roles / analyticsData.user_permission_stats.total_users) * 100} 
                    className="h-2"
                  />
                </div>
                
                <div className="space-y-2">
                  <div className="flex justify-between">
                    <span className="text-sm font-medium">Avg Roles per User</span>
                    <span className="text-sm text-muted-foreground">
                      {analyticsData.user_permission_stats.avg_roles_per_user}
                    </span>
                  </div>
                  <Progress value={analyticsData.user_permission_stats.avg_roles_per_user * 20} className="h-2" />
                </div>
                
                <div className="space-y-2">
                  <div className="flex justify-between">
                    <span className="text-sm font-medium">Avg Permissions per User</span>
                    <span className="text-sm text-muted-foreground">
                      {analyticsData.user_permission_stats.avg_permissions_per_user}
                    </span>
                  </div>
                  <Progress value={analyticsData.user_permission_stats.avg_permissions_per_user * 5} className="h-2" />
                </div>
              </div>
            </CardContent>
          </Card>
        </TabsContent>

        {/* Role Usage Tab */}
        <TabsContent value="roles" className="space-y-6">
          <div className="flex justify-between items-center">
            <div>
              <h3 className="text-lg font-semibold">Role Usage Analysis</h3>
              <p className="text-sm text-muted-foreground">Understand how roles are distributed across users</p>
            </div>
            <Button variant="outline" onClick={() => exportData('roles')}>
              <Download className="w-4 h-4 mr-2" />
              Export Role Data
            </Button>
          </div>

          <Card>
            <CardHeader>
              <CardTitle>Role Distribution</CardTitle>
              <CardDescription>Number of users assigned to each role</CardDescription>
            </CardHeader>
            <CardContent>
              <ResponsiveContainer width="100%" height={400}>
                <BarChart data={analyticsData.role_usage} layout="horizontal">
                  <CartesianGrid strokeDasharray="3 3" />
                  <XAxis type="number" />
                  <YAxis dataKey="display_name" type="category" width={120} />
                  <Tooltip />
                  <Bar dataKey="user_count" fill="#3b82f6" />
                </BarChart>
              </ResponsiveContainer>
            </CardContent>
          </Card>

          <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
            {analyticsData.role_usage.map(role => (
              <Card key={role.role_name}>
                <CardHeader>
                  <div className="flex justify-between items-start">
                    <div>
                      <CardTitle className="text-base">{role.display_name || role.role_name}</CardTitle>
                      <CardDescription className="font-mono text-xs">{role.role_name}</CardDescription>
                    </div>
                    <div className="flex items-center space-x-2">
                      <Badge variant={role.is_system_role ? "default" : "secondary"}>
                        {role.is_system_role ? "System" : "Custom"}
                      </Badge>
                      <Badge variant="outline">{role.user_count} users</Badge>
                    </div>
                  </div>
                </CardHeader>
                <CardContent>
                  <div className="space-y-2">
                    <div className="flex justify-between text-sm">
                      <span className="text-muted-foreground">Last Assigned</span>
                      <span>{new Date(role.last_assigned).toLocaleDateString()}</span>
                    </div>
                    <div className="flex justify-between text-sm">
                      <span className="text-muted-foreground">Usage</span>
                      <span>{((role.user_count / analyticsData.user_permission_stats.total_users) * 100).toFixed(1)}%</span>
                    </div>
                    <Progress value={(role.user_count / analyticsData.user_permission_stats.total_users) * 100} className="h-2" />
                  </div>
                </CardContent>
              </Card>
            ))}
          </div>
        </TabsContent>

        {/* Security Tab */}
        <TabsContent value="security" className="space-y-6">
          <div className="flex justify-between items-center">
            <div>
              <h3 className="text-lg font-semibold">Security Analysis</h3>
              <p className="text-sm text-muted-foreground">Monitor security violations and potential risks</p>
            </div>
            <Button variant="outline" onClick={() => exportData('security')}>
              <Download className="w-4 h-4 mr-2" />
              Export Security Report
            </Button>
          </div>

          {/* Security Overview */}
          <div className="grid grid-cols-1 md:grid-cols-4 gap-4">
            <Card>
              <CardHeader className="pb-3">
                <CardTitle className="text-sm">Over-Privileged Users</CardTitle>
              </CardHeader>
              <CardContent>
                <div className="text-2xl font-bold text-orange-600">{analyticsData.security_insights.over_privileged_users}</div>
                <p className="text-xs text-muted-foreground">Users with excessive permissions</p>
              </CardContent>
            </Card>
            
            <Card>
              <CardHeader className="pb-3">
                <CardTitle className="text-sm">Unused Permissions</CardTitle>
              </CardHeader>
              <CardContent>
                <div className="text-2xl font-bold text-blue-600">{analyticsData.security_insights.unused_permissions}</div>
                <p className="text-xs text-muted-foreground">Never used in last 90 days</p>
              </CardContent>
            </Card>
            
            <Card>
              <CardHeader className="pb-3">
                <CardTitle className="text-sm">Dormant Roles</CardTitle>
              </CardHeader>
              <CardContent>
                <div className="text-2xl font-bold text-yellow-600">{analyticsData.security_insights.dormant_roles}</div>
                <p className="text-xs text-muted-foreground">Not assigned in last 30 days</p>
              </CardContent>
            </Card>
            
            <Card>
              <CardHeader className="pb-3">
                <CardTitle className="text-sm">Expired Permissions</CardTitle>
              </CardHeader>
              <CardContent>
                <div className="text-2xl font-bold text-red-600">{analyticsData.security_insights.expired_permissions}</div>
                <p className="text-xs text-muted-foreground">Need immediate attention</p>
              </CardContent>
            </Card>
          </div>

          {/* Security Violations */}
          <Card>
            <CardHeader>
              <CardTitle>Security Violations</CardTitle>
              <CardDescription>Active security issues requiring attention</CardDescription>
            </CardHeader>
            <CardContent>
              <div className="space-y-3">
                {analyticsData.security_insights.security_violations.filter(v => !v.resolved).map(violation => (
                  <Alert key={violation.id} className={getSeverityColor(violation.severity)}>
                    <AlertTriangle className="h-4 w-4" />
                    <AlertDescription>
                      <div className="flex items-center justify-between">
                        <div>
                          <div className="font-medium">{violation.description}</div>
                          <div className="text-xs mt-1">
                            {violation.user_email && (
                              <span>User: {violation.user_email} • </span>
                            )}
                            <span>Type: {violation.type.replace('_', ' ')} • </span>
                            <span>Detected: {new Date(violation.created_at).toLocaleString()}</span>
                          </div>
                        </div>
                        <div className="flex items-center space-x-2">
                          <Badge variant="outline" className="capitalize">
                            {violation.severity}
                          </Badge>
                          <Button size="sm" variant="outline">
                            Resolve
                          </Button>
                        </div>
                      </div>
                    </AlertDescription>
                  </Alert>
                ))}
                
                {analyticsData.security_insights.security_violations.filter(v => !v.resolved).length === 0 && (
                  <div className="text-center py-8">
                    <Shield className="h-12 w-12 text-green-500 mx-auto mb-4" />
                    <h3 className="text-lg font-medium text-green-800 mb-2">All Clear!</h3>
                    <p className="text-sm text-muted-foreground">No active security violations detected.</p>
                  </div>
                )}
              </div>
            </CardContent>
          </Card>
        </TabsContent>

        {/* Trends Tab */}
        <TabsContent value="trends" className="space-y-6">
          <div>
            <h3 className="text-lg font-semibold">Activity Trends</h3>
            <p className="text-sm text-muted-foreground">Track permission and role changes over time</p>
          </div>

          <Card>
            <CardHeader>
              <CardTitle>Daily Activity</CardTitle>
              <CardDescription>Role assignments, permission grants, and revocations</CardDescription>
            </CardHeader>
            <CardContent>
              <ResponsiveContainer width="100%" height={400}>
                <AreaChart data={analyticsData.temporal_trends}>
                  <CartesianGrid strokeDasharray="3 3" />
                  <XAxis dataKey="date" />
                  <YAxis />
                  <Tooltip />
                  <Legend />
                  <Area type="monotone" dataKey="role_assignments" stackId="1" stroke="#3b82f6" fill="#3b82f6" />
                  <Area type="monotone" dataKey="permission_grants" stackId="1" stroke="#10b981" fill="#10b981" />
                  <Area type="monotone" dataKey="revocations" stackId="1" stroke="#ef4444" fill="#ef4444" />
                </AreaChart>
              </ResponsiveContainer>
            </CardContent>
          </Card>

          <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
            <Card>
              <CardHeader>
                <CardTitle>Audit Summary</CardTitle>
                <CardDescription>Event statistics for the selected period</CardDescription>
              </CardHeader>
              <CardContent className="space-y-4">
                <div className="flex justify-between items-center">
                  <span className="text-sm font-medium">Total Events</span>
                  <Badge variant="outline">{analyticsData.audit_summary.total_events}</Badge>
                </div>
                <div className="flex justify-between items-center">
                  <span className="text-sm font-medium">Last 24 Hours</span>
                  <Badge variant="secondary">{analyticsData.audit_summary.events_last_24h}</Badge>
                </div>
                <div className="flex justify-between items-center">
                  <span className="text-sm font-medium">Last 7 Days</span>
                  <Badge variant="secondary">{analyticsData.audit_summary.events_last_7d}</Badge>
                </div>
                <div className="flex justify-between items-center">
                  <span className="text-sm font-medium">Last 30 Days</span>
                  <Badge variant="secondary">{analyticsData.audit_summary.events_last_30d}</Badge>
                </div>
              </CardContent>
            </Card>

            <Card>
              <CardHeader>
                <CardTitle>Key Statistics</CardTitle>
                <CardDescription>Notable activity patterns</CardDescription>
              </CardHeader>
              <CardContent className="space-y-4">
                <div>
                  <div className="text-sm font-medium mb-1">Most Active Admin</div>
                  <div className="text-sm text-muted-foreground font-mono">
                    {analyticsData.audit_summary.most_active_admin}
                  </div>
                </div>
                <div>
                  <div className="text-sm font-medium mb-1">Most Changed User</div>
                  <div className="text-sm text-muted-foreground font-mono">
                    {analyticsData.audit_summary.most_changed_user}
                  </div>
                </div>
                <div>
                  <div className="text-sm font-medium mb-1">Average Daily Activity</div>
                  <div className="text-sm text-muted-foreground">
                    {(analyticsData.audit_summary.events_last_30d / 30).toFixed(1)} events/day
                  </div>
                </div>
              </CardContent>
            </Card>
          </div>
        </TabsContent>
      </Tabs>
    </div>
  )
}