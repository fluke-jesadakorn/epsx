'use client'

import { useState, useEffect } from 'react'
import { Card, CardContent, CardHeader, CardTitle, Button } from '@/components/ui'
import { Badge } from '@/components/ui/badge'
import { Progress } from '@/components/ui/progress'
import { Alert, AlertDescription } from '@/components/ui/alert'
import { Crown, Zap, Shield, TrendingUp, CheckCircle, XCircle, AlertCircle } from 'lucide-react'
import { PermissionTemplateName, PERMISSION_TEMPLATES, getDisplayTierColor, getRankingLimitFromPermissions } from '@/app/constants/packages'

interface UserPermissionInfo {
  id: string
  permissionTemplate: PermissionTemplateName
  permissions: string[]
  displayTier: string
  status: 'active' | 'expired' | 'cancelled'
  current_usage: Record<string, any>
  quota_limits: Record<string, any>
  granted_at: string
  expires_at?: string
  auto_renew: boolean
}

interface UserPlanDisplayProps {
  userId: string
}

export function UserPlanDisplay({ userId }: UserPlanDisplayProps) {
  const [userPermissions, setUserPermissions] = useState<UserPermissionInfo | null>(null)
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState<string | null>(null)

  useEffect(() => {
    loadUserPermissions()
  }, [userId])

  const loadUserPermissions = async () => {
    try {
      setLoading(true)
      // Fetch user permissions from the new permission-based API
      const response = await fetch(`http://localhost:8080/api/v1/users/permissions`, {
        method: 'GET',
        credentials: 'include', // Include cookies for authentication
        headers: {
          'Content-Type': 'application/json',
        },
      })
      
      if (response.ok) {
        const apiResponse = await response.json()

        if (apiResponse.success && apiResponse.data) {
          const permissionData = apiResponse.data

          if (permissionData.permissions && permissionData.permissions.length > 0) {
            // Extract permission strings from the backend response
            const permissionStrings = permissionData.permissions.map((p: any) => p.permission)

            // Convert backend data to our interface
            const permissionInfo: UserPermissionInfo = {
              id: permissionData.wallet_address || userId,
              permissionTemplate: getTemplateFromPermissions(permissionStrings),
              permissions: permissionStrings,
              displayTier: getDisplayTierFromPermissions(permissionStrings),
              status: 'active', // Backend doesn't provide status, assume active
              current_usage: {}, // Backend doesn't provide usage data yet
              quota_limits: generateQuotaFromPermissions(permissionStrings),
              granted_at: new Date().toISOString(), // Use current time
              expires_at: undefined, // Backend doesn't provide expiry for UI display
              auto_renew: false // Backend doesn't provide auto_renew info
            }
            setUserPermissions(permissionInfo)
          } else {
            setUserPermissions(null) // Free tier - no permissions
          }
        } else {
          setUserPermissions(null) // Free tier
        }
      } else if (response.status === 404 || response.status === 401) {
        setUserPermissions(null) // Free tier or not authenticated
      } else {
        throw new Error(`Failed to load permission data: ${response.status}`)
      }
    } catch (error) {
      console.error('Error loading permissions:', error)
      setError('Unable to load permission information')
    } finally {
      setLoading(false)
    }
  }

  // Helper functions for permission-based display
  const getTemplateFromPermissions = (permissions: string[]): PermissionTemplateName => {
    const displayTier = getDisplayTierFromPermissions(permissions)
    switch (displayTier) {
      case 'ENTERPRISE': return 'Enterprise Template'
      case 'PLATINUM': return 'Platinum Template'
      case 'GOLD': return 'Gold Template'
      case 'SILVER': return 'Silver Template'
      case 'BRONZE': return 'Bronze Template'
      default: return 'Free Template'
    }
  }

  const getDisplayTierFromPermissions = (permissions: string[]): string => {
    for (const permission of permissions) {
      if (permission.startsWith('epsx:rankings:view:')) {
        const limitStr = permission.split(':')[3]
        if (limitStr === 'unlimited') return 'ENTERPRISE'
        const limit = parseInt(limitStr, 10)
        if (limit >= 100) return 'PLATINUM'
        if (limit >= 50) return 'GOLD'
        if (limit >= 25) return 'SILVER'
        if (limit >= 5) return 'BRONZE'
      }
    }
    if (permissions.some(p => p.includes('admin:'))) return 'ENTERPRISE'
    return 'FREE'
  }

  const generateQuotaFromPermissions = (permissions: string[]) => {
    const rankingLimit = getRankingLimitFromPermissions(permissions)
    let apiCalls = 100
    
    if (rankingLimit === -1) {
      apiCalls = -1 // Unlimited
    } else if (rankingLimit >= 100) {
      apiCalls = 5000
    } else if (rankingLimit >= 50) {
      apiCalls = 2000
    } else if (rankingLimit >= 25) {
      apiCalls = 500
    }

    return {
      api_calls: apiCalls,
      rankings_limit: rankingLimit,
      analytics_queries: permissions.some(p => p.includes('analytics')) ? Math.floor(apiCalls / 10) : 0,
      export_limit: rankingLimit > 25 ? 50 : 10
    }
  }

  const calculateUsagePercentage = (used: number, limit: number) => {
    if (limit === 0 || limit === -1) return 0
    return Math.min((used / limit) * 100, 100)
  }

  const getStatusColor = (status: string) => {
    switch (status) {
      case 'active':
        return 'bg-green-100 text-green-600 dark:bg-green-900/20 dark:text-green-400'
      case 'expired':
        return 'bg-yellow-100 text-yellow-600 dark:bg-yellow-900/20 dark:text-yellow-400'
      case 'cancelled':
        return 'bg-red-100 text-red-600 dark:bg-red-900/20 dark:text-red-400'
      default:
        return 'bg-gray-100 text-gray-600 dark:bg-gray-900/20 dark:text-gray-400'
    }
  }

  const getTierIcon = (displayTier: string) => {
    if (displayTier === 'ENTERPRISE') return Crown
    if (displayTier === 'PLATINUM' || displayTier === 'GOLD') return Zap
    if (displayTier === 'SILVER') return Shield
    return TrendingUp
  }

  if (loading) {
    return (
      <Card>
        <CardHeader>
          <CardTitle className="flex items-center gap-2">
            <TrendingUp className="h-5 w-5" />
            Current Plan
          </CardTitle>
        </CardHeader>
        <CardContent>
          <div className="space-y-4">
            <div className="h-4 bg-gray-200 rounded animate-pulse"></div>
            <div className="h-4 bg-gray-200 rounded animate-pulse w-3/4"></div>
            <div className="h-4 bg-gray-200 rounded animate-pulse w-1/2"></div>
          </div>
        </CardContent>
      </Card>
    )
  }

  if (error) {
    return (
      <Card>
        <CardHeader>
          <CardTitle className="flex items-center gap-2">
            <AlertCircle className="h-5 w-5" />
            Plan Information
          </CardTitle>
        </CardHeader>
        <CardContent>
          <Alert variant="destructive">
            <AlertCircle className="h-4 w-4" />
            <AlertDescription>{error}</AlertDescription>
          </Alert>
        </CardContent>
      </Card>
    )
  }

  if (!userPermissions) {
    return (
      <Card>
        <CardHeader>
          <CardTitle className="flex items-center gap-2">
            <TrendingUp className="h-5 w-5" />
            Get Started with EPSX
          </CardTitle>
        </CardHeader>
        <CardContent>
          <div className="text-center space-y-4">
            <div className="text-6xl mb-4">🚀</div>
            <h3 className="text-xl font-semibold text-gray-600 dark:text-gray-400">
              Free Tier Access
            </h3>
            <p className="text-gray-500 dark:text-gray-500 mb-6">
              You have basic access with 3 rankings. Upgrade to unlock more features and data.
            </p>
            <Button className="bg-gradient-to-r from-emerald-400 to-green-500 text-white">
              View Permission Templates
            </Button>
          </div>
        </CardContent>
      </Card>
    )
  }

  const TierIcon = getTierIcon(userPermissions.displayTier)
  const isActive = userPermissions.status === 'active'
  const template = PERMISSION_TEMPLATES[userPermissions.permissionTemplate]

  return (
    <Card>
      <CardHeader>
        <div className="flex items-center justify-between">
          <CardTitle className="flex items-center gap-3">
            <TierIcon className="h-6 w-6 text-emerald-600" />
            {userPermissions.displayTier} Access
          </CardTitle>
          <Badge className={getStatusColor(userPermissions.status)}>
            {userPermissions.status.toUpperCase()}
          </Badge>
        </div>
        <div className="flex items-center gap-4 text-sm text-gray-500">
          <span>Template: {userPermissions.permissionTemplate}</span>
          {userPermissions.expires_at && (
            <span>
              Expires: {new Date(userPermissions.expires_at).toLocaleDateString()}
            </span>
          )}
          <span>
            Auto-renew: {userPermissions.auto_renew ? 'Yes' : 'No'}
          </span>
        </div>
      </CardHeader>
      <CardContent className="space-y-6">
        {/* Usage Statistics */}
        {isActive && Object.keys(userPermissions.current_usage).length > 0 && (
          <div>
            <h4 className="font-semibold text-gray-900 dark:text-white mb-4">Current Usage</h4>
            <div className="space-y-4">
              {Object.entries(userPermissions.current_usage).map(([key, used]) => {
                const limit = userPermissions.quota_limits[key] || 0
                const percentage = calculateUsagePercentage(Number(used), Number(limit))
                const isOverLimit = percentage >= 100 && limit !== -1
                
                return (
                  <div key={key} className="space-y-2">
                    <div className="flex justify-between items-center text-sm">
                      <span className="font-medium capitalize">
                        {key.replace(/_/g, ' ')}
                      </span>
                      <span className={`font-semibold ${isOverLimit ? 'text-red-600' : 'text-gray-600'}`}>
                        {used} / {limit === -1 ? '∞' : limit}
                      </span>
                    </div>
                    {limit !== -1 && (
                      <Progress 
                        value={percentage} 
                        className={`h-2 ${isOverLimit ? 'bg-red-100' : 'bg-gray-100'}`}
                      />
                    )}
                    {isOverLimit && (
                      <div className="text-xs text-red-600 flex items-center gap-1">
                        <AlertCircle className="h-3 w-3" />
                        Usage limit exceeded
                      </div>
                    )}
                  </div>
                )
              })}
            </div>
          </div>
        )}

        {/* Permissions Display */}
        <div>
          <h4 className="font-semibold text-gray-900 dark:text-white mb-4">Active Permissions</h4>
          <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
            {userPermissions.permissions.map((permission, index) => (
              <div key={index} className="flex items-start gap-3 p-3 bg-gray-50 dark:bg-gray-800 rounded-lg">
                <CheckCircle className="h-5 w-5 text-green-500 mt-0.5 flex-shrink-0" />
                <div className="min-w-0 flex-1">
                  <div className="font-medium text-sm">
                    {permission.replace(/:/g, ' → ').replace(/_/g, ' ')}
                  </div>
                  <div className="text-xs text-gray-500 mt-1">
                    <Badge variant="outline" className="text-xs">
                      {permission.split(':')[0]}
                    </Badge>
                    {permission.includes('view:') && (
                      <span className="ml-2">
                        Limit: {permission.split(':')[3] === 'unlimited' ? '∞' : permission.split(':')[3]}
                      </span>
                    )}
                  </div>
                </div>
              </div>
            ))}
          </div>
        </div>

        {/* Template Features */}
        <div>
          <h4 className="font-semibold text-gray-900 dark:text-white mb-4">Template Features</h4>
          <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
            {template.features.map((feature, index) => (
              <div key={index} className="flex items-start gap-3 p-3 bg-gray-50 dark:bg-gray-800 rounded-lg">
                <CheckCircle className="h-5 w-5 text-green-500 mt-0.5 flex-shrink-0" />
                <div className="min-w-0 flex-1">
                  <div className="font-medium text-sm">{feature}</div>
                </div>
              </div>
            ))}
          </div>
        </div>

        {/* Action Buttons */}
        <div className="flex gap-4 pt-4 border-t border-gray-200 dark:border-gray-700">
          {isActive ? (
            <>
              <Button variant="outline" className="flex-1">
                Manage Permissions
              </Button>
              <Button variant="outline">
                View Usage Details
              </Button>
            </>
          ) : (
            <>
              <Button className="flex-1 bg-gradient-to-r from-emerald-400 to-green-500 text-white">
                Reactivate Access
              </Button>
              <Button variant="outline">
                View History
              </Button>
            </>
          )}
        </div>

        {/* Expiry Warning */}
        {isActive && userPermissions.expires_at && (
          (() => {
            const daysUntilExpiry = Math.ceil(
              (new Date(userPermissions.expires_at).getTime() - Date.now()) / (1000 * 60 * 60 * 24)
            )
            
            if (daysUntilExpiry <= 7 && daysUntilExpiry > 0) {
              return (
                <Alert>
                  <AlertCircle className="h-4 w-4" />
                  <AlertDescription>
                    Your access expires in {daysUntilExpiry} day{daysUntilExpiry !== 1 ? 's' : ''}. 
                    {!userPermissions.auto_renew && ' Consider enabling auto-renewal.'}
                  </AlertDescription>
                </Alert>
              )
            }
            
            if (daysUntilExpiry <= 0) {
              return (
                <Alert variant="destructive">
                  <AlertCircle className="h-4 w-4" />
                  <AlertDescription>
                    Your access has expired. Renew now to continue using premium features.
                  </AlertDescription>
                </Alert>
              )
            }
            
            return null
          })()
        )}
      </CardContent>
    </Card>
  )
}