/**
 * Unified User Dashboard Component
 * Consolidates: ActivityTimelineCard, LoginHistoryCard, ModuleQuotaCard,
 * StockRankingPackageCard, QuickActions, and dashboard sections from UserProfile
 */

'use client'

import React, { useState, useEffect } from 'react'
import { useRouter } from 'next/navigation'
import { 
  User as UserIcon, Mail, Calendar, Shield, Activity, Clock, MapPin, Globe, 
  Smartphone, AlertTriangle, CheckCircle, XCircle, TrendingUp, TrendingDown,
  Edit, MoreHorizontal, RefreshCw, Power, Key, Trash2, Star, BarChart3,
  Monitor, LogIn, Package, Settings
} from 'lucide-react'

import { Button } from '@/components/ui/button'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import { Badge } from '@/components/ui/badge'
import { Progress } from '@/components/ui/progress'
import { Avatar, AvatarFallback, AvatarImage } from '@/components/ui/avatar'
import { 
  DropdownMenu, DropdownMenuContent, DropdownMenuItem, DropdownMenuTrigger 
} from '@/components/ui/dropdown-menu'
import { useToast } from '@/components/ui/use-toast'

import type { UnifiedUserData } from '@/lib/types/unified-user'
import { adminCardVariants, cn } from '@/design-system'

// Types
interface UserDashboardProps {
  user: UnifiedUserData
  currentUser?: any
  onUserUpdate?: () => void
  onPermissionChange?: (userId: string, permissions: string[]) => void
  className?: string
}

interface ActivityRecord {
  id: string
  action: string
  category: 'security' | 'permissions' | 'modules' | 'admin'
  status: 'success' | 'error' | 'warning'
  severity: 'high' | 'medium' | 'low'
  timestamp: Date | string
  resource?: string
  details?: string
  metadata?: Record<string, any>
  ipAddress?: string
}

interface LoginRecord {
  id: string
  timestamp: Date | string
  success: boolean
  deviceType?: string
  browser?: string
  ipAddress?: string
  location?: string
  sessionDuration?: number
  failureReason?: string
}

interface ModuleQuota {
  moduleName: string
  quotaType: string
  used: number
  limit: number
  resetDate?: Date | string
  dailyUsage?: number
}

interface StockRankingPackage {
  id: string
  name: string
  description?: string
  tier?: string
  price?: number
  billingCycle?: string
  isActive: boolean
  purchasedAt?: Date | string
  expiresAt?: Date | string
  lastUsed?: Date | string
  usageCount?: number
  features?: string[]
  autoRenew?: boolean
}

export function UserDashboard({
  user,
  currentUser,
  onUserUpdate,
  onPermissionChange,
  className = ''
}: UserDashboardProps) {
  const router = useRouter()
  const { toast } = useToast()

  // State
  const [loading, setLoading] = useState(false)
  const [activityLogs, setActivityLogs] = useState<ActivityRecord[]>([])
  const [loginHistory, setLoginHistory] = useState<LoginRecord[]>([])
  const [moduleQuotas, setModuleQuotas] = useState<ModuleQuota[]>([])
  const [stockPackages, setStockPackages] = useState<StockRankingPackage[]>([])

  // User stats
  const userStats = {
    accountAge: Math.floor((Date.now() - new Date(user.createdAt || Date.now()).getTime()) / (1000 * 60 * 60 * 24)),
    totalPermissions: (user.customPermissions || []).length,
    totalRoles: (user.roles || []).length,
    totalProfiles: (user.permissionProfiles || []).length,
    lastLoginDays: user.lastLoginAt ? Math.floor((Date.now() - new Date(user.lastLoginAt).getTime()) / (1000 * 60 * 60 * 24)) : null
  }

  // Permission checks
  const canModifyUser = currentUser?.permissions?.includes('admin:users:manage') || 
                       currentUser?.permissions?.includes('admin:*:*') ||
                       currentUser?.canManageUsers

  // Load dashboard data
  useEffect(() => {
    loadDashboardData()
  }, [user.id])

  const loadDashboardData = async () => {
    setLoading(true)
    try {
      // Mock data - replace with actual API calls
      const mockActivity: ActivityRecord[] = [
        {
          id: '1',
          action: 'User logged in from new device',
          category: 'security',
          status: 'success',
          severity: 'low',
          timestamp: new Date(Date.now() - 3600000),
          metadata: { device: 'Chrome on Windows', ip: '192.168.1.100' }
        },
        {
          id: '2',
          action: 'Permission granted: epsx:analytics:view',
          category: 'permissions',
          status: 'success',
          severity: 'medium',
          timestamp: new Date(Date.now() - 7200000),
          resource: 'analytics',
          metadata: { permission: 'epsx:analytics:view', grantedBy: 'admin@epsx.io' }
        },
        {
          id: '3',
          action: 'API quota exceeded',
          category: 'modules',
          status: 'warning',
          severity: 'high',
          timestamp: new Date(Date.now() - 10800000),
          resource: 'api_calls',
          details: 'Monthly API quota limit reached'
        }
      ]

      const mockLogins: LoginRecord[] = [
        {
          id: '1',
          timestamp: new Date(Date.now() - 3600000),
          success: true,
          deviceType: 'desktop',
          browser: 'Chrome',
          ipAddress: '192.168.1.100',
          location: 'San Francisco, US',
          sessionDuration: 150
        },
        {
          id: '2',
          timestamp: new Date(Date.now() - 86400000),
          success: true,
          deviceType: 'mobile',
          browser: 'Safari',
          ipAddress: '10.0.0.50',
          location: 'San Francisco, US',
          sessionDuration: 45
        },
        {
          id: '3',
          timestamp: new Date(Date.now() - 172800000),
          success: false,
          deviceType: 'desktop',
          browser: 'Firefox',
          ipAddress: '203.0.113.1',
          location: 'Unknown',
          failureReason: 'Invalid credentials'
        }
      ]

      const mockQuotas: ModuleQuota[] = [
        {
          moduleName: 'API Calls',
          quotaType: 'monthly',
          used: 8500,
          limit: 10000,
          resetDate: new Date(Date.now() + 86400000 * 10),
          dailyUsage: 283
        },
        {
          moduleName: 'Data Exports',
          quotaType: 'monthly',
          used: 15,
          limit: 50,
          resetDate: new Date(Date.now() + 86400000 * 10),
          dailyUsage: 2
        },
        {
          moduleName: 'Analytics Queries',
          quotaType: 'daily',
          used: 245,
          limit: 1000,
          resetDate: new Date(Date.now() + 86400000),
          dailyUsage: 245
        }
      ]

      const mockPackages: StockRankingPackage[] = [
        {
          id: '1',
          name: 'Premium Analytics Package',
          description: 'Advanced EPS rankings and analytics tools',
          tier: 'premium',
          price: 99.99,
          billingCycle: 'month',
          isActive: true,
          purchasedAt: new Date(Date.now() - 86400000 * 30),
          expiresAt: new Date(Date.now() + 86400000 * 335),
          lastUsed: new Date(Date.now() - 3600000),
          usageCount: 1247,
          features: ['Advanced Analytics', 'Custom Reports', 'Real-time Data'],
          autoRenew: true
        },
        {
          id: '2',
          name: 'Basic Stock Rankings',
          description: 'Essential stock ranking data',
          tier: 'basic',
          price: 29.99,
          billingCycle: 'month',
          isActive: false,
          purchasedAt: new Date(Date.now() - 86400000 * 60),
          expiresAt: new Date(Date.now() - 86400000 * 30),
          lastUsed: new Date(Date.now() - 86400000 * 35),
          usageCount: 342,
          features: ['Basic Rankings', 'Monthly Reports'],
          autoRenew: false
        }
      ]

      setActivityLogs(mockActivity)
      setLoginHistory(mockLogins)
      setModuleQuotas(mockQuotas)
      setStockPackages(mockPackages)
    } catch (error) {
      console.error('Failed to load dashboard data:', error)
    } finally {
      setLoading(false)
    }
  }

  // Utility functions
  const formatDate = (date: Date | string) => {
    return new Date(date).toLocaleDateString('en-US', {
      year: 'numeric',
      month: 'short',
      day: 'numeric'
    })
  }

  const formatTimestamp = (timestamp: Date | string) => {
    const date = new Date(timestamp)
    const now = new Date()
    const diffMs = now.getTime() - date.getTime()
    const diffHours = Math.floor(diffMs / (1000 * 60 * 60))
    const diffDays = Math.floor(diffMs / (1000 * 60 * 60 * 24))

    if (diffDays > 7) {
      return date.toLocaleDateString('en-US', {
        month: 'short',
        day: 'numeric',
        year: date.getFullYear() !== now.getFullYear() ? 'numeric' : undefined
      })
    } else if (diffDays > 0) {
      return `${diffDays}d ago`
    } else if (diffHours > 0) {
      return `${diffHours}h ago`
    } else {
      const diffMinutes = Math.floor(diffMs / (1000 * 60))
      return diffMinutes > 0 ? `${diffMinutes}m ago` : 'Just now'
    }
  }

  const formatPrice = (price: number) => {
    return new Intl.NumberFormat('en-US', {
      style: 'currency',
      currency: 'USD'
    }).format(price)
  }

  const formatNumber = (num: number) => {
    if (num >= 1000000) return `${(num / 1000000).toFixed(1)}M`
    if (num >= 1000) return `${(num / 1000).toFixed(1)}K`
    return num.toString()
  }

  // Event handlers
  const handleToggleUserStatus = async () => {
    setLoading(true)
    try {
      // Mock API call
      toast({
        title: 'User Status Updated',
        description: `User ${user.isActive ? 'deactivated' : 'activated'} successfully`,
      })
      onUserUpdate?.()
    } catch (error) {
      toast({
        title: 'Error',
        description: 'Failed to update user status',
        variant: 'destructive'
      })
    } finally {
      setLoading(false)
    }
  }

  const handleSendEmail = () => {
    window.location.href = `mailto:${user.email}`
  }

  // Icon helpers
  const getActivityIcon = (category: ActivityRecord['category']) => {
    switch (category) {
      case 'security': return <Shield className="h-4 w-4 text-red-500" />
      case 'permissions': return <Shield className="h-4 w-4 text-blue-500" />
      case 'modules': return <Package className="h-4 w-4 text-green-500" />
      case 'admin': return <Settings className="h-4 w-4 text-purple-500" />
      default: return <Activity className="h-4 w-4 text-gray-500" />
    }
  }

  const getStatusIcon = (status: ActivityRecord['status']) => {
    switch (status) {
      case 'success': return <CheckCircle className="h-3 w-3 text-green-500" />
      case 'error': return <XCircle className="h-3 w-3 text-red-500" />
      case 'warning': return <AlertTriangle className="h-3 w-3 text-orange-500" />
      default: return null
    }
  }

  const getDeviceIcon = (deviceType?: string) => {
    switch (deviceType) {
      case 'mobile': return <Smartphone className="h-4 w-4 text-green-500" />
      case 'tablet': return <Smartphone className="h-4 w-4 text-blue-500" />
      default: return <Monitor className="h-4 w-4 text-purple-500" />
    }
  }

  return (
    <div className={`space-y-6 ${className}`}>
      {/* Header */}
      <div className="flex items-center justify-between">
        <div>
          <h2 className="text-2xl font-bold text-gray-900">User Dashboard</h2>
          <p className="text-gray-600">Comprehensive overview of {user.email}'s account</p>
        </div>
        <Button 
          onClick={loadDashboardData} 
          variant="outline"
          disabled={loading}
        >
          <RefreshCw className={`h-4 w-4 mr-2 ${loading ? 'animate-spin' : ''}`} />
          Refresh
        </Button>
      </div>

      {/* User Header Card */}
      <Card className={cn(adminCardVariants({ variant: 'pancake' }))}>
        <CardContent className="p-6">
          <div className="flex flex-col lg:flex-row items-start gap-6">
            <div className="flex items-center gap-4 flex-1">
              {/* Avatar */}
              <div className="relative">
                <Avatar className="w-16 h-16 border-2 border-blue-200">
                  <AvatarImage src={`https://api.dicebear.com/7.x/initials/svg?seed=${user.email}`} />
                  <AvatarFallback className="text-lg font-bold bg-blue-100 text-blue-700">
                    {user.email.slice(0, 2).toUpperCase()}
                  </AvatarFallback>
                </Avatar>
                <div className={`absolute -bottom-1 -right-1 w-5 h-5 rounded-full border-2 border-white ${user.isActive ? 'bg-green-500' : 'bg-gray-400'}`}></div>
              </div>

              {/* User Info */}
              <div className="flex-1">
                <div className="flex flex-col sm:flex-row sm:items-center gap-2 sm:gap-3 mb-3">
                  <h3 className="text-xl font-bold text-gray-800">
                    {user.displayName || user.email.split('@')[0]}
                  </h3>
                  <div className="flex gap-2">
                    <Badge variant={user.isActive ? 'default' : 'secondary'}>
                      {user.isActive ? 'Active' : 'Inactive'}
                    </Badge>
                    <Badge variant={user.role === 'admin' ? 'destructive' : 'outline'}>
                      {user.role}
                    </Badge>
                  </div>
                </div>

                <div className="grid grid-cols-1 sm:grid-cols-3 gap-3 text-sm text-gray-600">
                  <div className="flex items-center gap-2">
                    <Mail className="w-4 h-4 text-blue-500" />
                    <span className="truncate">{user.email}</span>
                  </div>
                  <div className="flex items-center gap-2">
                    <Calendar className="w-4 h-4 text-purple-500" />
                    <span>Joined {formatDate(user.createdAt || Date.now())}</span>
                  </div>
                  <div className="flex items-center gap-2">
                    <Shield className="w-4 h-4 text-green-500" />
                    <span>{userStats.totalPermissions} permissions</span>
                  </div>
                </div>

                {user.lastLoginAt && (
                  <p className="text-xs text-gray-500 mt-2 flex items-center gap-1">
                    <Clock className="w-3 h-3" />
                    Last seen: {formatTimestamp(user.lastLoginAt)}
                  </p>
                )}
              </div>
            </div>

            {/* Quick Actions */}
            <div className="flex gap-2">
              <Button
                onClick={() => router.push(`/users/${user.id}/edit`)}
                className="flex items-center gap-2"
              >
                <Edit className="w-4 h-4" />
                Edit
              </Button>
              
              <DropdownMenu>
                <DropdownMenuTrigger asChild>
                  <Button variant="outline" size="sm">
                    <MoreHorizontal className="w-4 h-4" />
                  </Button>
                </DropdownMenuTrigger>
                <DropdownMenuContent align="end">
                  <DropdownMenuItem onClick={handleSendEmail}>
                    <Mail className="w-4 h-4 mr-2" />
                    Send Email
                  </DropdownMenuItem>
                  {canModifyUser && (
                    <>
                      <DropdownMenuItem onClick={handleToggleUserStatus}>
                        <Power className="w-4 h-4 mr-2" />
                        {user.isActive ? 'Deactivate' : 'Activate'}
                      </DropdownMenuItem>
                      <DropdownMenuItem>
                        <Key className="w-4 h-4 mr-2" />
                        Reset Keys
                      </DropdownMenuItem>
                      <DropdownMenuItem className="text-red-600">
                        <Trash2 className="w-4 h-4 mr-2" />
                        Delete User
                      </DropdownMenuItem>
                    </>
                  )}
                </DropdownMenuContent>
              </DropdownMenu>
            </div>
          </div>
        </CardContent>
      </Card>

      {/* Stats Grid */}
      <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-4">
        <Card className={cn(adminCardVariants({ variant: 'pancake' }))}>
          <CardContent className="p-6 text-center">
            <Calendar className="w-8 h-8 text-blue-500 mx-auto mb-3" />
            <p className="text-sm text-gray-500 mb-1">Account Age</p>
            <p className="text-2xl font-bold text-blue-600">{userStats.accountAge}d</p>
          </CardContent>
        </Card>

        <Card className={cn(adminCardVariants({ variant: 'pancake' }))}>
          <CardContent className="p-6 text-center">
            <Shield className="w-8 h-8 text-green-500 mx-auto mb-3" />
            <p className="text-sm text-gray-500 mb-1">Permissions</p>
            <p className="text-2xl font-bold text-green-600">{userStats.totalPermissions}</p>
          </CardContent>
        </Card>

        <Card className={cn(adminCardVariants({ variant: 'pancake' }))}>
          <CardContent className="p-6 text-center">
            <UserIcon className="w-8 h-8 text-purple-500 mx-auto mb-3" />
            <p className="text-sm text-gray-500 mb-1">Roles</p>
            <p className="text-2xl font-bold text-purple-600">{userStats.totalRoles}</p>
          </CardContent>
        </Card>

        <Card className={cn(adminCardVariants({ variant: 'pancake' }))}>
          <CardContent className="p-6 text-center">
            <Clock className="w-8 h-8 text-orange-500 mx-auto mb-3" />
            <p className="text-sm text-gray-500 mb-1">Last Login</p>
            <p className="text-lg font-bold text-orange-600">
              {userStats.lastLoginDays !== null ? `${userStats.lastLoginDays}d ago` : 'Never'}
            </p>
          </CardContent>
        </Card>
      </div>

      <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
        {/* Recent Activity */}
        <Card>
          <CardHeader>
            <CardTitle className="flex items-center gap-2">
              <Activity className="h-5 w-5" />
              Recent Activity
            </CardTitle>
          </CardHeader>
          <CardContent>
            <div className="space-y-4">
              {activityLogs.slice(0, 5).map((activity) => (
                <div key={activity.id} className="flex items-start gap-3 p-3 rounded-lg border">
                  <div className="flex-shrink-0 mt-0.5">
                    {getActivityIcon(activity.category)}
                  </div>
                  <div className="flex-1 min-w-0">
                    <div className="flex items-center gap-2">
                      <p className="text-sm font-medium">{activity.action}</p>
                      {getStatusIcon(activity.status)}
                    </div>
                    <div className="text-xs text-muted-foreground flex items-center gap-2 mt-1">
                      <Clock className="h-3 w-3" />
                      <span>{formatTimestamp(activity.timestamp)}</span>
                      {activity.resource && (
                        <>
                          <span>•</span>
                          <span>{activity.resource}</span>
                        </>
                      )}
                    </div>
                    {activity.details && (
                      <p className="text-xs text-muted-foreground mt-1">{activity.details}</p>
                    )}
                  </div>
                </div>
              ))}
            </div>
          </CardContent>
        </Card>

        {/* Login History */}
        <Card>
          <CardHeader>
            <CardTitle className="flex items-center gap-2">
              <LogIn className="h-5 w-5" />
              Login History
            </CardTitle>
          </CardHeader>
          <CardContent>
            <div className="space-y-4">
              {loginHistory.slice(0, 5).map((login) => (
                <div key={login.id} className={`p-3 border rounded-lg ${!login.success ? 'border-red-200 bg-red-50' : ''}`}>
                  <div className="flex items-center justify-between mb-2">
                    <div className="flex items-center gap-2">
                      <LogIn className={`h-4 w-4 ${login.success ? 'text-green-500' : 'text-red-500'}`} />
                      <span className="text-sm font-medium">
                        {login.success ? 'Login' : 'Failed Login'}
                      </span>
                    </div>
                    <span className="text-xs text-muted-foreground">
                      {formatTimestamp(login.timestamp)}
                    </span>
                  </div>

                  <div className="space-y-1 text-xs text-muted-foreground">
                    <div className="flex items-center gap-2">
                      {getDeviceIcon(login.deviceType)}
                      <span>{login.deviceType || 'Unknown device'}</span>
                      {login.browser && (
                        <>
                          <span>•</span>
                          <span>{login.browser}</span>
                        </>
                      )}
                    </div>

                    {login.ipAddress && (
                      <div className="flex items-center gap-2">
                        <Globe className="h-3 w-3" />
                        <span className="font-mono">{login.ipAddress}</span>
                        {login.location && (
                          <>
                            <span>•</span>
                            <span>{login.location}</span>
                          </>
                        )}
                      </div>
                    )}

                    {!login.success && login.failureReason && (
                      <div className="text-red-600">
                        <span>Reason: {login.failureReason}</span>
                      </div>
                    )}
                  </div>
                </div>
              ))}
            </div>
          </CardContent>
        </Card>

        {/* Module Quotas */}
        <Card>
          <CardHeader>
            <CardTitle className="flex items-center gap-2">
              <BarChart3 className="h-5 w-5" />
              Module Quotas
            </CardTitle>
          </CardHeader>
          <CardContent>
            <div className="space-y-4">
              {moduleQuotas.map((quota, index) => {
                const usagePercent = Math.round((quota.used / quota.limit) * 100)
                const isNearLimit = usagePercent >= 80
                const isOverLimit = usagePercent >= 100

                return (
                  <div key={index} className="space-y-2">
                    <div className="flex items-center justify-between">
                      <span className="font-medium text-sm">{quota.moduleName}</span>
                      <span className="text-sm text-muted-foreground">
                        {formatNumber(quota.used)} / {formatNumber(quota.limit)}
                      </span>
                    </div>
                    <Progress 
                      value={Math.min(usagePercent, 100)} 
                      className={`h-2 ${isOverLimit ? 'bg-red-100' : isNearLimit ? 'bg-orange-100' : 'bg-green-100'}`}
                    />
                    <div className="flex justify-between text-xs text-muted-foreground">
                      <span>{usagePercent}% used</span>
                      {quota.resetDate && (
                        <span>Resets {formatDate(quota.resetDate)}</span>
                      )}
                    </div>
                  </div>
                )
              })}
            </div>
          </CardContent>
        </Card>

        {/* Stock Ranking Packages */}
        <Card>
          <CardHeader>
            <CardTitle className="flex items-center gap-2">
              <Star className="h-5 w-5" />
              Stock Packages
            </CardTitle>
          </CardHeader>
          <CardContent>
            <div className="space-y-4">
              {stockPackages.map((pkg) => {
                const isExpiringSoon = pkg.expiresAt && 
                  new Date(pkg.expiresAt).getTime() - Date.now() < 30 * 24 * 60 * 60 * 1000

                return (
                  <div key={pkg.id} className="p-3 border rounded-lg">
                    <div className="flex items-center justify-between mb-2">
                      <div className="flex items-center gap-2">
                        <Star className={`h-4 w-4 ${pkg.isActive ? 'text-yellow-500' : 'text-gray-400'}`} />
                        <span className="font-medium text-sm">{pkg.name}</span>
                        <Badge variant={pkg.isActive ? 'default' : 'secondary'} size="sm">
                          {pkg.isActive ? 'Active' : 'Inactive'}
                        </Badge>
                      </div>
                    </div>

                    <div className="space-y-1 text-xs text-muted-foreground">
                      {pkg.tier && (
                        <div className="flex justify-between">
                          <span>Tier:</span>
                          <span className="font-medium capitalize">{pkg.tier}</span>
                        </div>
                      )}
                      {pkg.price && (
                        <div className="flex justify-between">
                          <span>Price:</span>
                          <span className="font-medium">
                            {formatPrice(pkg.price)}/{pkg.billingCycle}
                          </span>
                        </div>
                      )}
                      {pkg.expiresAt && (
                        <div className="flex justify-between">
                          <span>Expires:</span>
                          <span className={isExpiringSoon ? 'text-orange-600 font-medium' : ''}>
                            {formatDate(pkg.expiresAt)}
                          </span>
                        </div>
                      )}
                    </div>

                    {pkg.features && (
                      <div className="mt-2">
                        <div className="flex flex-wrap gap-1">
                          {pkg.features.slice(0, 2).map((feature, index) => (
                            <Badge key={index} variant="outline" className="text-xs">
                              {feature}
                            </Badge>
                          ))}
                          {pkg.features.length > 2 && (
                            <span className="text-xs text-muted-foreground">
                              +{pkg.features.length - 2} more
                            </span>
                          )}
                        </div>
                      </div>
                    )}
                  </div>
                )
              })}
            </div>
          </CardContent>
        </Card>
      </div>
    </div>
  )
}